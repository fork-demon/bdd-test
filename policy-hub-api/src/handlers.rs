//! API request handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use policy_hub_core::{
    CreatePolicyRequest, CreateRuleTemplateRequest, ExecutePolicyRequest, Policy, RuleTemplate,
    RuleTemplateVersionInfo, RuleTemplateVersionsResponse,
};
use policy_hub_storage::{PolicyStorage, RuleTemplateStorage};
use policy_hub_bundler::Bundler;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::{ApiError, AppState};

/// Helper to rebuild the WASM bundle and save to file system
/// Accepts an optional new_policy to ensure it's included (bypasses N1QL eventual consistency)
async fn rebuild_bundle(state: &AppState, new_policy: Option<Policy>) -> Result<(), ApiError> {
    let mut policies = PolicyStorage::list(state.policy_storage.as_ref()).await?;
    
    // If a new policy is provided, ensure it's in the list (handles Couchbase eventual consistency)
    if let Some(ref policy) = new_policy {
        if !policies.iter().any(|p| p.id == policy.id) {
            tracing::debug!("Adding new policy {} to bundle (not yet visible in N1QL)", policy.id);
            policies.push(policy.clone());
        }
    }
    
    tracing::debug!("Bundling {} policies: {:?}", policies.len(), policies.iter().map(|p| &p.id).collect::<Vec<_>>());
    
    let mut templates = HashMap::new();
    
    let mut unique_template_ids = HashSet::new();
    for p in &policies {
        unique_template_ids.insert(p.rule_template_id);
    }
    
    // Load templates from DB and compile them on-demand
    for id in unique_template_ids {
        if let Some(mut template) = RuleTemplateStorage::get_by_id(state.rule_storage.as_ref(), id).await? {
            // Compile the template source (not stored in DB, compiled on-demand)
            let compiled_js = state.compiler.compile(&template.source)?;
            template.compiled_js = Some(compiled_js);
            templates.insert(id, template);
        }
    }
    
    let bundle = Bundler::bundle_all(&policies, &templates)
        .map_err(|e| ApiError::Internal(format!("Bundling failed: {}", e)))?;
    
    // Save bundle to file system
    let bundle_dir = std::path::Path::new("./bundles");
    if !bundle_dir.exists() {
        std::fs::create_dir_all(bundle_dir)
            .map_err(|e| ApiError::Internal(format!("Failed to create bundles dir: {}", e)))?;
    }
    
    let bundle_path = bundle_dir.join("policy_bundle.wasm");
    std::fs::write(&bundle_path, &bundle)
        .map_err(|e| ApiError::Internal(format!("Failed to save bundle to file: {}", e)))?;
    
    tracing::info!("Saved WASM bundle to {:?} ({} bytes)", bundle_path, bundle.len());
    
    // Also update in-memory cache
    let mut cache = state.cached_bundle.write().await;
    *cache = Some(bundle);
    
    tracing::info!("Rebuilt WASM bundle with {} policies", policies.len());
    Ok(())
}

// ==================== Rule Template Handlers ====================

/// Create a new rule template
pub async fn create_rule_template(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateRuleTemplateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate the source (ensures it compiles correctly)
    state.compiler.validate(&req.source)?;

    // Check if a template with this name already exists
    let existing = RuleTemplateStorage::get_latest_by_name(state.rule_storage.as_ref(), &req.name).await?;

    // Create template WITHOUT compiled_js (only store source in DB)
    let template = if let Some(existing) = existing {
        // Create a new version
        existing.new_version(req.source.clone())
    } else {
        // Create a new template
        RuleTemplate::new(req.name.clone(), req.source.clone())
    };

    let saved = RuleTemplateStorage::save(state.rule_storage.as_ref(), template).await?;

    // Trigger bundle rebuild (compilation happens here, saved to file system)
    if let Err(e) = rebuild_bundle(&state, None).await {
        tracing::error!("Failed to rebuild bundle: {}", e);
        // We continue, as the rule is saved, but execution might use old bundle until next update.
    }

    tracing::info!(
        "Created rule template '{}' version {}",
        saved.name,
        saved.version
    );

    Ok((StatusCode::CREATED, Json(saved)))
}

/// Get a rule template by ID
pub async fn get_rule_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {

    let template = RuleTemplateStorage::get_by_id(state.rule_storage.as_ref(), id).await?;

    match template {
        Some(t) => Ok(Json(t)),
        None => Err(ApiError::NotFound(format!("Rule template {} not found", id))),
    }
}

/// Get all versions of a rule template by name
pub async fn get_rule_template_versions(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ApiError> {

    let versions = RuleTemplateStorage::get_versions_by_name(state.rule_storage.as_ref(), &name).await?;

    if versions.is_empty() {
        return Err(ApiError::NotFound(format!(
            "Rule template '{}' not found",
            name
        )));
    }

    let response = RuleTemplateVersionsResponse {
        name: name.clone(),
        versions: versions
            .into_iter()
            .map(|v| RuleTemplateVersionInfo {
                id: v.id,
                version: v.version,
                created_at: v.created_at,
                is_latest: v.is_latest,
            })
            .collect(),
    };

    Ok(Json(response))
}

/// List all rule template names
pub async fn list_rule_templates(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let names = RuleTemplateStorage::list_names(state.rule_storage.as_ref()).await?;
    Ok(Json(names))
}

// ==================== Policy Handlers ====================

/// Create a new policy
pub async fn create_policy(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreatePolicyRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Get the rule template
    let template = if let Some(version) = req.rule_template_version {
        // Get specific version by ID and version number
        let found = RuleTemplateStorage::get_by_id(state.rule_storage.as_ref(), req.rule_template_id).await?;
        if let Some(t) = found {
            if t.version != version {
                return Err(ApiError::NotFound(format!(
                    "Rule template {} version {} not found",
                    req.rule_template_id, version
                )));
            }
            t
        } else {
            return Err(ApiError::NotFound(format!(
                "Rule template {} not found",
                req.rule_template_id
            )));
        }
    } else {
        // Get by ID only
        RuleTemplateStorage::get_by_id(state.rule_storage.as_ref(), req.rule_template_id)
            .await?
            .ok_or_else(|| {
                ApiError::NotFound(format!("Rule template {} not found", req.rule_template_id))
            })?
    };

    let mut policy = Policy::new(
        req.name.clone(),
        template.id,
        template.version,
        req.metadata.clone(),
    );
    policy.description = req.description.clone();

    let saved = PolicyStorage::save(state.policy_storage.as_ref(), policy).await?;

    // Trigger bundle rebuild
    if let Err(e) = rebuild_bundle(&state, Some(saved.clone())).await {
        tracing::error!("Failed to rebuild bundle: {}", e);
    }

    tracing::info!("Created policy '{}'", saved.name);

    Ok((StatusCode::CREATED, Json(saved)))
}

/// Get a policy by ID
pub async fn get_policy(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {

    let policy = PolicyStorage::get_by_id(state.policy_storage.as_ref(), id).await?;

    match policy {
        Some(p) => Ok(Json(p)),
        None => Err(ApiError::NotFound(format!("Policy {} not found", id))),
    }
}

/// List all policies
pub async fn list_policies(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let policies = PolicyStorage::list(state.policy_storage.as_ref()).await?;
    Ok(Json(policies))
}

// ==================== Execution Handler ====================

/// Execute a policy with input facts
pub async fn execute_policy(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ExecutePolicyRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Get the policy
    let policy = PolicyStorage::get_by_id(state.policy_storage.as_ref(), req.policy_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Policy {} not found", req.policy_id)))?;

    // Use Cached Bundle Execution
    // Logic: Try to get read lock. If None, try to rebuild.
    let bundle_guard = state.cached_bundle.read().await;
    let bundle = if let Some(b) = &*bundle_guard {
        b.clone()
    } else {
        drop(bundle_guard);
        rebuild_bundle(&state, None).await?;
        let guard = state.cached_bundle.read().await;
        guard.as_ref().ok_or_else(|| ApiError::Internal("Failed to build bundle".into()))?.clone()
    };
    
    // Execute using the bundle
    let result = state.executor.execute_bundle(&bundle, &req.policy_id.to_string(), &req.facts)?;

    tracing::info!(
        "Executed policy '{}' in {}ms - condition_met: {}",
        policy.name,
        result.execution_time_ms,
        result.condition_met
    );

    Ok(Json(result))
}

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "policy-hub"
    }))
}

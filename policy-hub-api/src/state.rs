//! Application state shared across handlers

use policy_hub_bundler::Bundler;
use policy_hub_compiler::RuleCompiler;
use policy_hub_executor::{WasmExecutor, WasmLimits};
use policy_hub_storage::{InMemoryStorage, PolicyStorage, RuleTemplateStorage, Storage};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared application state
pub struct AppState {
    pub rule_storage: Arc<dyn RuleTemplateStorage + Send + Sync>,
    pub policy_storage: Arc<dyn PolicyStorage + Send + Sync>,
    pub compiler: RuleCompiler,
    pub executor: WasmExecutor,
    pub cached_bundle: Arc<RwLock<Option<Vec<u8>>>>,
}

impl AppState {
    pub fn new() -> Self {
        let store = Arc::new(InMemoryStorage::new());
        Self::with_storage(store)
    }

    /// Create with custom storage backend
    pub fn with_storage(storage: Arc<dyn Storage>) -> Self {
        let limits = WasmLimits {
            max_memory_bytes: 16 * 1024 * 1024,
            max_fuel: 1_000_000,
            timeout_ms: 5000,
        };

        let executor = WasmExecutor::with_limits(limits)
            .expect("Failed to create WASM executor");

        Self {
            rule_storage: storage.clone(),
            policy_storage: storage.clone(),
            compiler: RuleCompiler::new(),
            executor,
            cached_bundle: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with custom WASM limits for stricter sandboxing
    pub fn with_limits(limits: WasmLimits) -> Self {
        let executor = WasmExecutor::with_limits(limits)
            .expect("Failed to create WASM executor");
            
        let store = Arc::new(InMemoryStorage::new());

        Self {
            rule_storage: store.clone(),
            policy_storage: store.clone(),
            compiler: RuleCompiler::new(),
            executor,
            cached_bundle: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize the WASM bundle on server startup
    /// First tries to load from file system, then falls back to rebuilding from storage
    pub async fn initialize_bundle(&self) -> Result<usize, String> {
        let bundle_path = std::path::Path::new("./bundles/policy_bundle.wasm");
        
        // Try to load from file system first
        if bundle_path.exists() {
            match std::fs::read(bundle_path) {
                Ok(bundle) => {
                    let size = bundle.len();
                    let mut cache = self.cached_bundle.write().await;
                    *cache = Some(bundle);
                    tracing::info!(
                        "Loaded WASM bundle from file system ({} bytes)",
                        size
                    );
                    
                    // Count policies for return value
                    let policies = PolicyStorage::list(self.policy_storage.as_ref())
                        .await
                        .map_err(|e| format!("Failed to list policies: {}", e))?;
                    return Ok(policies.len());
                }
                Err(e) => {
                    tracing::warn!("Failed to load bundle from file, will rebuild: {}", e);
                }
            }
        }
        
        // Fall back to rebuilding from storage
        let policies = PolicyStorage::list(self.policy_storage.as_ref())
            .await
            .map_err(|e| format!("Failed to list policies: {}", e))?;

        if policies.is_empty() {
            tracing::info!("No policies found in storage, bundle not initialized");
            return Ok(0);
        }

        // Collect unique template IDs
        let mut unique_template_ids = HashSet::new();
        for p in &policies {
            unique_template_ids.insert(p.rule_template_id);
        }

        // Load templates from DB and compile them on-demand
        let mut templates = HashMap::new();
        for id in unique_template_ids {
            if let Some(mut template) = RuleTemplateStorage::get_by_id(self.rule_storage.as_ref(), id)
                .await
                .map_err(|e| format!("Failed to get template {}: {}", id, e))?
            {
                // Compile the template source (not stored in DB, compiled on-demand)
                let compiled_js = self.compiler.compile(&template.source)
                    .map_err(|e| format!("Failed to compile template {}: {}", id, e))?;
                template.compiled_js = Some(compiled_js);
                templates.insert(id, template);
            }
        }

        // Build the bundle
        let bundle = Bundler::bundle_all(&policies, &templates)
            .map_err(|e| format!("Bundling failed: {}", e))?;

        // Save to file system
        let bundle_dir = std::path::Path::new("./bundles");
        if !bundle_dir.exists() {
            std::fs::create_dir_all(bundle_dir)
                .map_err(|e| format!("Failed to create bundles dir: {}", e))?;
        }
        std::fs::write(bundle_path, &bundle)
            .map_err(|e| format!("Failed to save bundle to file: {}", e))?;

        let policy_count = policies.len();
        let mut cache = self.cached_bundle.write().await;
        *cache = Some(bundle);

        tracing::info!(
            "Rebuilt WASM bundle with {} policies and {} templates (saved to file)",
            policy_count,
            templates.len()
        );

        Ok(policy_count)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

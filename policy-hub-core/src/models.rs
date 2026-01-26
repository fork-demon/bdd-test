//! Core domain models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A rule template containing the TypeScript DSL source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleTemplate {
    /// Unique identifier
    pub id: Uuid,
    /// Human-readable name
    pub name: String,
    /// Version number (auto-incremented)
    pub version: u32,
    /// TypeScript source code with when/then DSL
    pub source: String,
    /// Path to compiled WASM bundle (if applicable)
    pub wasm_path: Option<String>,
    /// Compiled JavaScript (transpiled from TypeScript)
    pub compiled_js: Option<String>,
    /// When this version was created
    pub created_at: DateTime<Utc>,
    /// Whether this is the latest version
    pub is_latest: bool,
}

impl RuleTemplate {
    pub fn new(name: String, source: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            version: 1,
            source,
            wasm_path: None,
            compiled_js: None,
            created_at: Utc::now(),
            is_latest: true,
        }
    }

    /// Create a new version of this template
    pub fn new_version(&self, source: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: self.name.clone(),
            version: self.version + 1,
            source,
            wasm_path: None,
            compiled_js: None,
            created_at: Utc::now(),
            is_latest: true,
        }
    }
}

/// A policy that references a rule template version and includes metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Unique identifier
    pub id: Uuid,
    /// Human-readable name
    pub name: String,
    /// Reference to the rule template
    pub rule_template_id: Uuid,
    /// Specific version of the rule template to use
    pub rule_template_version: u32,
    /// User-defined metadata (created at policy creation time)
    pub metadata: serde_json::Value,
    /// When this policy was created
    pub created_at: DateTime<Utc>,
    /// Optional description
    pub description: Option<String>,
    /// Whether this policy is active
    pub is_active: bool,
}

impl Policy {
    pub fn new(
        name: String,
        rule_template_id: Uuid,
        rule_template_version: u32,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            rule_template_id,
            rule_template_version,
            metadata,
            created_at: Utc::now(),
            description: None,
            is_active: true,
        }
    }
}

/// Input facts provided during policy execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputFacts {
    /// The actual fact data as JSON
    pub data: serde_json::Value,
}

impl InputFacts {
    pub fn new(data: serde_json::Value) -> Self {
        Self { data }
    }
}

/// Result of executing a policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Whether the execution was successful
    pub success: bool,
    /// Whether the rule conditions were satisfied
    pub condition_met: bool,
    /// Output facts produced by the rule
    pub output_facts: serde_json::Value,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Any error message if execution failed
    pub error: Option<String>,
    /// Timestamp of execution
    pub executed_at: DateTime<Utc>,
}

impl ExecutionResult {
    pub fn success(condition_met: bool, output_facts: serde_json::Value, execution_time_ms: u64) -> Self {
        Self {
            success: true,
            condition_met,
            output_facts,
            execution_time_ms,
            error: None,
            executed_at: Utc::now(),
        }
    }

    pub fn failure(error: String, execution_time_ms: u64) -> Self {
        Self {
            success: false,
            condition_met: false,
            output_facts: serde_json::Value::Null,
            execution_time_ms,
            error: Some(error),
            executed_at: Utc::now(),
        }
    }
}

/// Request to create a new rule template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRuleTemplateRequest {
    pub name: String,
    pub source: String,
}

/// Request to create a new policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePolicyRequest {
    pub name: String,
    pub rule_template_id: Uuid,
    pub rule_template_version: Option<u32>, // If None, use latest
    pub metadata: serde_json::Value,
    pub description: Option<String>,
}

/// Request to execute a policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutePolicyRequest {
    pub policy_id: Uuid,
    pub facts: serde_json::Value,
}

/// Response containing a list of rule template versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleTemplateVersionsResponse {
    pub name: String,
    pub versions: Vec<RuleTemplateVersionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleTemplateVersionInfo {
    pub id: Uuid,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub is_latest: bool,
}

//! Error types for the core crate

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid rule template: {0}")]
    InvalidRuleTemplate(String),

    #[error("Invalid policy: {0}")]
    InvalidPolicy(String),

    #[error("Version not found: {0}")]
    VersionNotFound(String),

    #[error("Rule template not found: {0}")]
    RuleTemplateNotFound(String),

    #[error("Policy not found: {0}")]
    PolicyNotFound(String),
}

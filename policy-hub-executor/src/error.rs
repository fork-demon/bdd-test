//! Executor error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecutorError {
    #[error("Runtime error: {0}")]
    RuntimeError(String),

    #[error("Execution timeout")]
    Timeout,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Script not loaded")]
    ScriptNotLoaded,
}

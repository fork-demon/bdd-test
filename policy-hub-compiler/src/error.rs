//! Compiler error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("Syntax error: {0}")]
    SyntaxError(String),

    #[error("Compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Invalid rule structure: {0}")]
    InvalidRuleStructure(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

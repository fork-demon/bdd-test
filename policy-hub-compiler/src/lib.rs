//! Compiler for TypeScript rule templates
//!
//! This crate handles the compilation of TypeScript DSL rule templates
//! into executable JavaScript that can be run by the executor.

pub mod compiler;
pub mod error;

pub use compiler::RuleCompiler;
pub use error::CompilerError;

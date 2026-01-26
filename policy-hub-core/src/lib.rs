//! Core domain models for Policy Hub
//!
//! This crate contains the shared data structures used across
//! the policy engine: RuleTemplate, Policy, Facts, and ExecutionResult.

pub mod error;
pub mod models;

pub use error::CoreError;
pub use models::*;

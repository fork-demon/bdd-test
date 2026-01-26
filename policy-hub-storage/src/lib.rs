//! Storage layer for Policy Hub
//!
//! Provides persistence for rule templates and policies.
//! Supports both in-memory (for development) and Couchbase backends.

pub mod error;
pub mod memory;
pub mod traits;

#[cfg(feature = "couchbase")]
pub mod couchbase;

pub use error::StorageError;
pub use memory::InMemoryStorage;
pub use traits::{PolicyStorage, RuleTemplateStorage};

#[cfg(feature = "couchbase")]
pub use couchbase::CouchbaseStorage;

/// Unified storage trait
#[async_trait::async_trait]
pub trait Storage: RuleTemplateStorage + PolicyStorage + Send + Sync {}

#[async_trait::async_trait]
impl<T> Storage for T where T: RuleTemplateStorage + PolicyStorage + Send + Sync {}

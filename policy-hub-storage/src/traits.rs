//! Storage traits defining the interface for persistence

use async_trait::async_trait;
use policy_hub_core::{Policy, RuleTemplate};
use uuid::Uuid;

use crate::StorageError;

/// Trait for rule template storage operations
#[async_trait]
pub trait RuleTemplateStorage: Send + Sync {
    /// Save a new rule template
    async fn save(&self, template: RuleTemplate) -> Result<RuleTemplate, StorageError>;

    /// Get a rule template by ID
    async fn get_by_id(&self, id: Uuid) -> Result<Option<RuleTemplate>, StorageError>;

    /// Get all versions of a rule template by name
    async fn get_versions_by_name(&self, name: &str) -> Result<Vec<RuleTemplate>, StorageError>;

    /// Get the latest version of a rule template by name
    async fn get_latest_by_name(&self, name: &str) -> Result<Option<RuleTemplate>, StorageError>;

    /// Get a specific version of a rule template by name
    async fn get_by_name_and_version(
        &self,
        name: &str,
        version: u32,
    ) -> Result<Option<RuleTemplate>, StorageError>;

    /// Update an existing rule template
    async fn update(&self, template: RuleTemplate) -> Result<RuleTemplate, StorageError>;

    /// List all rule template names
    async fn list_names(&self) -> Result<Vec<String>, StorageError>;
}

/// Trait for policy storage operations
#[async_trait]
pub trait PolicyStorage: Send + Sync {
    /// Save a new policy
    async fn save(&self, policy: Policy) -> Result<Policy, StorageError>;

    /// Get a policy by ID
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Policy>, StorageError>;

    /// List all policies
    async fn list(&self) -> Result<Vec<Policy>, StorageError>;

    /// Update an existing policy
    async fn update(&self, policy: Policy) -> Result<Policy, StorageError>;

    /// Delete a policy
    async fn delete(&self, id: Uuid) -> Result<(), StorageError>;
}

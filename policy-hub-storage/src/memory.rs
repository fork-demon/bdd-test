//! In-memory storage implementation for development and testing

use async_trait::async_trait;
use policy_hub_core::{Policy, RuleTemplate};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

use crate::{PolicyStorage, RuleTemplateStorage, StorageError};

/// In-memory storage for development and testing
pub struct InMemoryStorage {
    rule_templates: RwLock<HashMap<Uuid, RuleTemplate>>,
    policies: RwLock<HashMap<Uuid, Policy>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            rule_templates: RwLock::new(HashMap::new()),
            policies: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RuleTemplateStorage for InMemoryStorage {
    async fn save(&self, template: RuleTemplate) -> Result<RuleTemplate, StorageError> {
        let mut templates = self.rule_templates.write().unwrap();

        // Mark previous versions as not latest
        for existing in templates.values_mut() {
            if existing.name == template.name {
                existing.is_latest = false;
            }
        }

        templates.insert(template.id, template.clone());
        Ok(template)
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<RuleTemplate>, StorageError> {
        let templates = self.rule_templates.read().unwrap();
        Ok(templates.get(&id).cloned())
    }

    async fn get_versions_by_name(&self, name: &str) -> Result<Vec<RuleTemplate>, StorageError> {
        let templates = self.rule_templates.read().unwrap();
        let mut versions: Vec<_> = templates
            .values()
            .filter(|t| t.name == name)
            .cloned()
            .collect();
        versions.sort_by(|a, b| a.version.cmp(&b.version));
        Ok(versions)
    }

    async fn get_latest_by_name(&self, name: &str) -> Result<Option<RuleTemplate>, StorageError> {
        let templates = self.rule_templates.read().unwrap();
        Ok(templates
            .values()
            .filter(|t| t.name == name && t.is_latest)
            .cloned()
            .next())
    }

    async fn get_by_name_and_version(
        &self,
        name: &str,
        version: u32,
    ) -> Result<Option<RuleTemplate>, StorageError> {
        let templates = self.rule_templates.read().unwrap();
        Ok(templates
            .values()
            .find(|t| t.name == name && t.version == version)
            .cloned())
    }

    async fn update(&self, template: RuleTemplate) -> Result<RuleTemplate, StorageError> {
        let mut templates = self.rule_templates.write().unwrap();
        if templates.contains_key(&template.id) {
            templates.insert(template.id, template.clone());
            Ok(template)
        } else {
            Err(StorageError::NotFound(format!(
                "RuleTemplate with id {} not found",
                template.id
            )))
        }
    }

    async fn list_names(&self) -> Result<Vec<String>, StorageError> {
        let templates = self.rule_templates.read().unwrap();
        let mut names: Vec<_> = templates
            .values()
            .filter(|t| t.is_latest)
            .map(|t| t.name.clone())
            .collect();
        names.sort();
        names.dedup();
        Ok(names)
    }
}

#[async_trait]
impl PolicyStorage for InMemoryStorage {
    async fn save(&self, policy: Policy) -> Result<Policy, StorageError> {
        let mut policies = self.policies.write().unwrap();
        policies.insert(policy.id, policy.clone());
        Ok(policy)
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Policy>, StorageError> {
        let policies = self.policies.read().unwrap();
        Ok(policies.get(&id).cloned())
    }

    async fn list(&self) -> Result<Vec<Policy>, StorageError> {
        let policies = self.policies.read().unwrap();
        Ok(policies.values().cloned().collect())
    }

    async fn update(&self, policy: Policy) -> Result<Policy, StorageError> {
        let mut policies = self.policies.write().unwrap();
        if policies.contains_key(&policy.id) {
            policies.insert(policy.id, policy.clone());
            Ok(policy)
        } else {
            Err(StorageError::NotFound(format!(
                "Policy with id {} not found",
                policy.id
            )))
        }
    }

    async fn delete(&self, id: Uuid) -> Result<(), StorageError> {
        let mut policies = self.policies.write().unwrap();
        if policies.remove(&id).is_some() {
            Ok(())
        } else {
            Err(StorageError::NotFound(format!(
                "Policy with id {} not found",
                id
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PolicyStorage, RuleTemplateStorage};

    #[tokio::test]
    async fn test_save_and_get_rule_template() {
        let storage = InMemoryStorage::new();
        let template = RuleTemplate::new(
            "test-rule".to_string(),
            "when(true).then({})".to_string(),
        );

        let saved = RuleTemplateStorage::save(&storage, template.clone()).await.unwrap();
        assert_eq!(saved.name, "test-rule");
        assert_eq!(saved.version, 1);
        assert!(saved.is_latest);

        let retrieved = RuleTemplateStorage::get_by_id(&storage, saved.id).await.unwrap().unwrap();
        assert_eq!(retrieved.id, saved.id);
    }

    #[tokio::test]
    async fn test_version_management() {
        let storage = InMemoryStorage::new();
        
        let v1 = RuleTemplate::new("test-rule".to_string(), "v1 source".to_string());
        RuleTemplateStorage::save(&storage, v1.clone()).await.unwrap();

        let v2 = v1.new_version("v2 source".to_string());
        RuleTemplateStorage::save(&storage, v2.clone()).await.unwrap();

        let versions = storage.get_versions_by_name("test-rule").await.unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version, 1);
        assert_eq!(versions[1].version, 2);

        let latest = storage.get_latest_by_name("test-rule").await.unwrap().unwrap();
        assert_eq!(latest.version, 2);
        assert!(latest.is_latest);
    }

    #[tokio::test]
    async fn test_save_and_get_policy() {
        let storage = InMemoryStorage::new();
        let policy = Policy::new(
            "test-policy".to_string(),
            Uuid::new_v4(),
            1,
            serde_json::json!({"key": "value"}),
        );

        let saved = PolicyStorage::save(&storage, policy.clone()).await.unwrap();
        let retrieved = PolicyStorage::get_by_id(&storage, saved.id).await.unwrap().unwrap();
        assert_eq!(retrieved.name, "test-policy");
    }
}

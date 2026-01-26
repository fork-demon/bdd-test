//! Couchbase storage implementation
//!
//! Provides persistent storage for rule templates and policies using Couchbase.

use async_trait::async_trait;
use couchbase::{
    cluster::Cluster,
    collection::Collection,
    options::{

        cluster_options::ClusterOptions,
        kv_options::{GetOptions, UpsertOptions, RemoveOptions},
        query_options::QueryOptions,
        diagnostic_options::WaitUntilReadyOptions,
    },
};
use tokio_stream::StreamExt;

// Fallback imports/structs if not found in specific modules
use couchbase::authenticator::{Authenticator, PasswordAuthenticator}; 

use policy_hub_core::{Policy, RuleTemplate};
use std::sync::Arc;
use uuid::Uuid;

use crate::{PolicyStorage, RuleTemplateStorage, StorageError};

/// Document type markers for N1QL queries
const DOC_TYPE_RULE_TEMPLATE: &str = "rule_template";
const DOC_TYPE_POLICY: &str = "policy";

/// Couchbase storage configuration
#[derive(Debug, Clone)]
pub struct CouchbaseConfig {
    pub connection_string: String,
    pub username: String,
    pub password: String,
    pub bucket_name: String,
}

impl Default for CouchbaseConfig {
    fn default() -> Self {
        Self {
            connection_string: "couchbase://localhost".to_string(),
            username: "admin".to_string(),
            password: "password123".to_string(),
            bucket_name: "policy-hub".to_string(),
        }
    }
}

/// Couchbase storage for rule templates and policies
pub struct CouchbaseStorage {
    cluster: Arc<Cluster>,
    collection: Collection,
    bucket_name: String,
}

impl CouchbaseStorage {
    /// Create a new Couchbase storage instance
    pub async fn new(config: CouchbaseConfig) -> Result<Self, StorageError> {
        let authenticator = PasswordAuthenticator::new(&config.username, &config.password);
        let options = ClusterOptions::new(Authenticator::PasswordAuthenticator(authenticator));
        let cluster = Cluster::connect(&config.connection_string, options).await
            .map_err(|e| StorageError::Connection(format!("Failed to connect to cluster: {}", e)))?;

        // Wait for cluster to be ready
        let bucket = cluster.bucket(&config.bucket_name);
        let _: () = bucket
            .wait_until_ready(WaitUntilReadyOptions::default())
            .await
            .map_err(|e: couchbase::error::Error| StorageError::Connection(format!("Failed to connect to bucket: {}", e)))?;

        let collection = bucket.default_collection();

        tracing::info!(
            "Connected to Couchbase cluster at {}, bucket: {}",
            config.connection_string,
            config.bucket_name
        );

        Ok(Self {
            cluster: Arc::new(cluster),
            collection,
            bucket_name: config.bucket_name,
        })
    }

    /// Create a new instance with default configuration
    pub async fn with_defaults() -> Result<Self, StorageError> {
        Self::new(CouchbaseConfig::default()).await
    }

    /// Execute a N1QL query
    async fn query<T: serde::de::DeserializeOwned>(
        &self,
        statement: &str,
    ) -> Result<Vec<T>, StorageError> {
        let mut result = self
            .cluster
            .query(statement, QueryOptions::default())
            .await
            .map_err(|e: couchbase::error::Error| StorageError::Internal(format!("Query failed: {}", e)))?;

        let mut rows = Vec::new();
        let mut row_iter = result.rows::<T>();
        
        while let Some(row) = row_iter.next().await {
            match row {
                Ok(r) => rows.push(r),
                Err(e) => {
                    tracing::warn!("Failed to deserialize row: {}", e);
                }
            }
        }

        Ok(rows)
    }
}

/// Wrapper for documents with type field
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TypedDocument<T> {
    #[serde(rename = "type")]
    doc_type: String,
    #[serde(flatten)]
    data: T,
}

#[async_trait]
impl RuleTemplateStorage for CouchbaseStorage {
    async fn save(&self, template: RuleTemplate) -> Result<RuleTemplate, StorageError> {
        let doc_id = format!("rule_template::{}", template.id);

        // If this is the latest version, mark previous versions as not latest
        if template.is_latest {
            let update_query = format!(
                r#"
                UPDATE `{}` 
                SET is_latest = false 
                WHERE type = '{}' AND name = '{}' AND is_latest = true
                "#,
                self.bucket_name, DOC_TYPE_RULE_TEMPLATE, template.name
            );
            let _ = self.query::<serde_json::Value>(&update_query).await;
        }

        let doc = TypedDocument {
            doc_type: DOC_TYPE_RULE_TEMPLATE.to_string(),
            data: template.clone(),
        };

        let _ = self.collection
            .upsert(&doc_id, &doc, UpsertOptions::default())
            .await
            .map_err(|e| StorageError::Internal(format!("Failed to save rule template: {}", e)))?;

        tracing::debug!("Saved rule template {} version {}", template.name, template.version);
        Ok(template)
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<RuleTemplate>, StorageError> {
        let doc_id = format!("rule_template::{}", id);

        match self.collection.get(&doc_id, GetOptions::default()).await {
            Ok(result) => {
                let doc: TypedDocument<RuleTemplate> = result
                    .content_as::<TypedDocument<RuleTemplate>>()
                    .map_err(|e| StorageError::Serialization(serde_json::Error::io(
                        std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
                    )))?;
                Ok(Some(doc.data))
            }
            Err(e) => {
                if e.to_string().contains("DocumentNotFound") {
                    Ok(None)
                } else {
                    Err(StorageError::Internal(format!(
                        "Failed to get rule template: {}",
                        e
                    )))
                }
            }
        }
    }

    async fn get_versions_by_name(&self, name: &str) -> Result<Vec<RuleTemplate>, StorageError> {
        let query = format!(
            r#"
            SELECT META().id, t.*
            FROM `{}` t
            WHERE t.type = '{}' AND t.name = '{}'
            ORDER BY t.version ASC
            "#,
            self.bucket_name, DOC_TYPE_RULE_TEMPLATE, name
        );

        self.query(&query).await
    }

    async fn get_latest_by_name(&self, name: &str) -> Result<Option<RuleTemplate>, StorageError> {
        let query = format!(
            r#"
            SELECT t.*
            FROM `{}` t
            WHERE t.type = '{}' AND t.name = '{}' AND t.is_latest = true
            LIMIT 1
            "#,
            self.bucket_name, DOC_TYPE_RULE_TEMPLATE, name
        );

        let results: Vec<RuleTemplate> = self.query(&query).await?;
        Ok(results.into_iter().next())
    }

    async fn get_by_name_and_version(
        &self,
        name: &str,
        version: u32,
    ) -> Result<Option<RuleTemplate>, StorageError> {
        let query = format!(
            r#"
            SELECT t.*
            FROM `{}` t
            WHERE t.type = '{}' AND t.name = '{}' AND t.version = {}
            LIMIT 1
            "#,
            self.bucket_name, DOC_TYPE_RULE_TEMPLATE, name, version
        );

        let results: Vec<RuleTemplate> = self.query(&query).await?;
        Ok(results.into_iter().next())
    }

    async fn update(&self, template: RuleTemplate) -> Result<RuleTemplate, StorageError> {
        // Check if exists first
        if RuleTemplateStorage::get_by_id(self, template.id).await?.is_none() {
            return Err(StorageError::NotFound(format!(
                "RuleTemplate with id {} not found",
                template.id
            )));
        }
        RuleTemplateStorage::save(self, template).await
    }

    async fn list_names(&self) -> Result<Vec<String>, StorageError> {
        let query = format!(
            r#"
            SELECT DISTINCT t.name
            FROM `{}` t
            WHERE t.type = '{}' AND t.is_latest = true
            ORDER BY t.name ASC
            "#,
            self.bucket_name, DOC_TYPE_RULE_TEMPLATE
        );

        #[derive(serde::Deserialize)]
        struct NameRow {
            name: String,
        }

        let results: Vec<NameRow> = self.query(&query).await?;
        Ok(results.into_iter().map(|r| r.name).collect())
    }
}

#[async_trait]
impl PolicyStorage for CouchbaseStorage {
    async fn save(&self, policy: Policy) -> Result<Policy, StorageError> {
        let doc_id = format!("policy::{}", policy.id);

        let doc = TypedDocument {
            doc_type: DOC_TYPE_POLICY.to_string(),
            data: policy.clone(),
        };

        let _ = self.collection
            .upsert(&doc_id, &doc, UpsertOptions::default())
            .await
            .map_err(|e| StorageError::Internal(format!("Failed to save policy: {}", e)))?;

        tracing::debug!("Saved policy {}", policy.name);
        Ok(policy)
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Policy>, StorageError> {
        let doc_id = format!("policy::{}", id);

        match self.collection.get(&doc_id, GetOptions::default()).await {
            Ok(result) => {
                let doc: TypedDocument<Policy> = result
                    .content_as::<TypedDocument<Policy>>()
                    .map_err(|e| StorageError::Serialization(serde_json::Error::io(
                        std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
                    )))?;
                Ok(Some(doc.data))
            }
            Err(e) => {
                if e.to_string().contains("DocumentNotFound") {
                    Ok(None)
                } else {
                    Err(StorageError::Internal(format!(
                        "Failed to get policy: {}",
                        e
                    )))
                }
            }
        }
    }

    async fn list(&self) -> Result<Vec<Policy>, StorageError> {
        let query = format!(
            r#"
            SELECT p.*
            FROM `{}` p
            WHERE p.type = '{}'
            ORDER BY p.created_at DESC
            "#,
            self.bucket_name, DOC_TYPE_POLICY
        );

        self.query(&query).await
    }

    async fn update(&self, policy: Policy) -> Result<Policy, StorageError> {
        if PolicyStorage::get_by_id(self, policy.id).await?.is_none() {
            return Err(StorageError::NotFound(format!(
                "Policy with id {} not found",
                policy.id
            )));
        }
        PolicyStorage::save(self, policy).await
    }

    async fn delete(&self, id: Uuid) -> Result<(), StorageError> {
        let doc_id = format!("policy::{}", id);

        match self.collection.remove(&doc_id, RemoveOptions::default()).await {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.to_string().contains("DocumentNotFound") {
                    Err(StorageError::NotFound(format!("Policy with id {} not found", id)))
                } else {
                    Err(StorageError::Internal(format!(
                        "Failed to delete policy: {}",
                        e
                    )))
                }
            }
        }
    }
}

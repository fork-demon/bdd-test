//! Policy Hub - Main Application Entry Point
//!
//! A dynamic policy engine with TypeScript rule templates
//! compiled to executable JavaScript for high-performance evaluation.

use policy_hub_api::AppState;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,policy_hub=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid u16");

    tracing::info!("Starting Policy Hub server on {}:{}", host, port);

    // Initialize storage backend
    let storage_type = std::env::var("STORAGE_TYPE").unwrap_or_else(|_| "memory".to_string());
    
    let storage: std::sync::Arc<dyn policy_hub_storage::Storage> = if storage_type == "couchbase" {
        #[cfg(feature = "couchbase")]
        {
            use policy_hub_storage::CouchbaseStorage;
            tracing::info!("Initializing Couchbase storage...");
            let store = CouchbaseStorage::with_defaults().await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            std::sync::Arc::new(store)
        }
        #[cfg(not(feature = "couchbase"))]
        {
            panic!("Couchbase storage requested but 'couchbase' feature not enabled");
        }
    } else {
        tracing::info!("Initializing InMemory storage...");
        std::sync::Arc::new(policy_hub_storage::InMemoryStorage::new())
    };

    // Create shared application state
    let app_state = Arc::new(AppState::with_storage(storage));

    // Initialize WASM bundle from existing policies in storage
    match app_state.initialize_bundle().await {
        Ok(count) => {
            if count > 0 {
                tracing::info!("Pre-loaded WASM bundle with {} policies from storage", count);
            }
        }
        Err(e) => {
            tracing::warn!("Failed to initialize bundle on startup: {}", e);
        }
    }

    // Build our application with routes
    let app = policy_hub_api::create_router(app_state);

    // Run it
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

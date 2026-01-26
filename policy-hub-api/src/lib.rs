//! Policy Hub API Server
//!
//! REST API for managing rule templates, policies, and execution.

pub mod error;
pub mod handlers;
pub mod state;

pub use error::ApiError;
pub use state::AppState;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(handlers::health_check))
        // Rule Templates
        .route("/api/rule-templates", post(handlers::create_rule_template).get(handlers::list_rule_templates))
        .route("/api/rule-templates/:id", get(handlers::get_rule_template))
        .route("/api/rule-templates/name/:name/versions", get(handlers::get_rule_template_versions))
        // Policies
        .route("/api/policies", post(handlers::create_policy).get(handlers::list_policies))
        .route("/api/policies/:id", get(handlers::get_policy))
        // Execution
        .route("/api/execute", post(handlers::execute_policy))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}

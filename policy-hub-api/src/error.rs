//! API error types and error handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use policy_hub_compiler::CompilerError;
use policy_hub_executor::ExecutorError;
use policy_hub_storage::StorageError;
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Compilation error: {0}")]
    Compilation(String),

    #[error("Execution error: {0}")]
    Execution(String),
}

impl From<StorageError> for ApiError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::NotFound(msg) => ApiError::NotFound(msg),
            StorageError::AlreadyExists(msg) => ApiError::BadRequest(msg),
            _ => ApiError::Internal(err.to_string()),
        }
    }
}

impl From<CompilerError> for ApiError {
    fn from(err: CompilerError) -> Self {
        ApiError::Compilation(err.to_string())
    }
}

impl From<ExecutorError> for ApiError {
    fn from(err: ExecutorError) -> Self {
        ApiError::Execution(err.to_string())
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type) = match self {
            ApiError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            ApiError::Compilation(_) => (StatusCode::BAD_REQUEST, "compilation_error"),
            ApiError::Execution(_) => (StatusCode::INTERNAL_SERVER_ERROR, "execution_error"),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
        });

        (status, body).into_response()
    }
}

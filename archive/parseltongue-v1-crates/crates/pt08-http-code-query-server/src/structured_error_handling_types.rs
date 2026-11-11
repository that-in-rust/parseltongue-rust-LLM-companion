//! Structured error types for HTTP server
//!
//! # 4-Word Naming: structured_error_handling_types
//!
//! Uses thiserror for library errors per S06 principles.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// HTTP server error types
///
/// # 4-Word Name: HttpServerErrorTypes
#[derive(Error, Debug)]
pub enum HttpServerErrorTypes {
    #[error("Database operation failed: {0}")]
    DatabaseOperationFailedError(String),

    #[error("Entity not found: {0}")]
    EntityNotFoundQueryError(String),

    #[error("Invalid request parameter: {0}")]
    InvalidRequestParameterError(String),

    #[error("Server startup failed: {0}")]
    ServerStartupFailedError(String),

    #[error("Ingestion operation failed: {0}")]
    IngestionOperationFailedError(String),

    #[error("Git operation not available: {0}")]
    GitOperationNotAvailableError(String),

    #[error("Internal server error: {0}")]
    InternalServerProcessingError(String),
}

impl IntoResponse for HttpServerErrorTypes {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            HttpServerErrorTypes::EntityNotFoundQueryError(_) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            HttpServerErrorTypes::InvalidRequestParameterError(_) => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
            HttpServerErrorTypes::GitOperationNotAvailableError(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string())
            }
            HttpServerErrorTypes::DatabaseOperationFailedError(_)
            | HttpServerErrorTypes::ServerStartupFailedError(_)
            | HttpServerErrorTypes::IngestionOperationFailedError(_)
            | HttpServerErrorTypes::InternalServerProcessingError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
        };

        let body = Json(json!({
            "success": false,
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

/// Result type alias for HTTP handlers
///
/// # 4-Word Name: HttpHandlerResultType
pub type HttpHandlerResultType<T> = Result<T, HttpServerErrorTypes>;

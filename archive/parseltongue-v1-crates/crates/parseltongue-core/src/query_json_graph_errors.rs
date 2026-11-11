//! Error types for JSON graph query operations
//!
//! # Design (S77 Pattern A.6)
//! - thiserror for library errors
//! - Structured variants for specific failures

use thiserror::Error;

/// Errors that can occur during JSON graph queries
#[derive(Debug, Error, PartialEq)]
pub enum JsonGraphQueryError {
    /// Entity with given ISG key not found
    #[error("Entity not found with key: {0}")]
    EntityNotFound(String),

    /// JSON structure is malformed (missing fields, wrong types)
    #[error("Malformed JSON structure: {0}")]
    MalformedJson(String),

    /// Invalid edge type requested
    #[error("Invalid edge type: {0}. Valid: Calls, Uses, Implements")]
    InvalidEdgeType(String),
}

impl From<&str> for JsonGraphQueryError {
    fn from(s: &str) -> Self {
        Self::MalformedJson(s.to_string())
    }
}

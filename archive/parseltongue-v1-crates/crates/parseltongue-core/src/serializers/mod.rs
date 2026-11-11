//! Serialization abstractions for Parseltongue data exports
//!
//! # Architecture: Layered Design (L2 - Standard Library)
//!
//! Following S06 Principle #2: Serialization is a core capability that belongs
//! in parseltongue-core, not scattered across tool implementations.
//!
//! # Design Principles (S06)
//!
//! 1. **Trait-based abstraction** (#3: Dependency Injection)
//! 2. **Single source of truth** for format logic
//! 3. **Testable independently** of export operations
//! 4. **Reusable across all tools** (pt02, pt03, pt05, etc.)
//!
//! # Supported Formats
//!
//! - **JSON**: Standard format for tool compatibility
//! - **TOON**: Tab-Oriented Object Notation for 30-40% token reduction
//! - **Mermaid**: GitHub-native graph visualization with semantic edge directionality

use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};

pub mod json;
// pub mod mermaid; // v0.9.8+: Deferred per user request (agent queryability first)
pub mod toon;

pub use json::JsonSerializer;
// pub use mermaid::{render_graph_as_mermaid, MermaidConfig}; // v0.9.8+
pub use toon::{ToonDelimiter, ToonSerializer};

/// Core serialization trait for data export formats
///
/// # Contract (Executable Specification)
///
/// **Preconditions**:
/// - Data slice is valid (may be empty)
/// - Serializable type implements `serde::Serialize`
///
/// **Postconditions**:
/// - Returns `Ok(String)` with formatted data
/// - Empty arrays produce valid format-specific empty representation
/// - Token estimate matches actual output within 10% margin
///
/// **Error Conditions**:
/// - Serialization failure (e.g., non-UTF8 data, circular references)
///
/// # Example
///
/// ```rust,ignore
/// let serializer = JsonSerializer;
/// let data = vec![Entity { name: "foo".into() }];
/// let output = serializer.serialize(&data)?;
/// assert!(output.contains("foo"));
/// ```
pub trait Serializer: Send + Sync {
    /// Serialize data to format-specific string
    ///
    /// # Arguments
    /// - `data`: Slice of serializable items
    ///
    /// # Returns
    /// Formatted string representation of data
    fn serialize<T: Serialize>(&self, data: &[T]) -> Result<String>;

    /// Get file extension for this format
    ///
    /// Used for automatic filename derivation (e.g., "data.json" → "data.toon")
    fn extension(&self) -> &'static str;

    /// Estimate token count for LLM context optimization
    ///
    /// # Arguments
    /// - `entity_count`: Number of entities in export
    ///
    /// # Returns
    /// Estimated token count (must be within 10% of actual for validation)
    fn estimate_tokens(&self, entity_count: usize) -> usize;
}

/// Derive output path for a serializer format
///
/// # Example
/// ```rust,ignore
/// let json_path = PathBuf::from("data.json");
/// let toon_path = derive_output_path(&json_path, "toon");
/// assert_eq!(toon_path, PathBuf::from("data.toon"));
/// ```
pub fn derive_output_path(base_path: &Path, extension: &str) -> PathBuf {
    base_path.with_extension(extension)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_output_path() {
        let json = PathBuf::from("entities.json");
        let toon = derive_output_path(&json, "toon");
        assert_eq!(toon, PathBuf::from("entities.toon"));

        let no_ext = PathBuf::from("data");
        let with_ext = derive_output_path(&no_ext, "json");
        assert_eq!(with_ext, PathBuf::from("data.json"));
    }
}

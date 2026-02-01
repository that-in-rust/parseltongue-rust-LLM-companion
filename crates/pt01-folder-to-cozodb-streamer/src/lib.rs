//! Parseltongue Tool 01: folder-to-cozoDB-streamer
//!
//! Ultra-minimalist streaming tool that reads code files from a directory,
//! generates ISGL1 keys using tree-sitter, and stores them in CozoDB.
//!
//! ## CLI Examples
//!
//! ```bash
//! # Index current directory (default)
//! parseltongue folder-to-cozodb-streamer .
//!
//! # Index specific directory with custom database
//! parseltongue folder-to-cozodb-streamer ./crates --db rocksdb:analysis.db --verbose
//! ```
//!
//! ## How it Works
//!
//! Processes ALL files (`*` pattern) - tree-sitter determines what it can parse.
//! Gracefully skips non-code files (.md, .json, .toml, etc.).

#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
#![allow(missing_docs)]

use std::path::PathBuf;
use std::sync::Arc;

pub mod cli;
pub mod errors;
pub mod external_dependency_handler;
pub mod file_watcher;
pub mod isgl1_generator;
pub mod lsp_client;
pub mod streamer;
pub mod test_detector;
pub mod v090_specifications;

// Bug #4: External dependency placeholder tests
#[cfg(test)]
mod external_dependency_tests;

// Re-export commonly used types
pub use errors::*;
pub use file_watcher::*;
pub use isgl1_generator::*;
pub use lsp_client::*;
pub use streamer::{FileStreamerImpl, *};
pub use test_detector::*;

/// Tool metadata and configuration
#[derive(Debug, Clone)]
pub struct StreamerConfig {
    /// Root directory to scan for code files
    pub root_dir: PathBuf,
    /// Database connection string
    pub db_path: String,
    /// Maximum file size to process (bytes)
    pub max_file_size: usize,
    /// File patterns to include
    pub include_patterns: Vec<String>,
    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Parsing library to use (default: "tree-sitter")
    pub parsing_library: String,
    /// Chunking strategy to use (default: "ISGL1")
    pub chunking: String,
}

impl Default for StreamerConfig {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("."),
            db_path: "mem".to_string(), // Use in-memory database by default
            max_file_size: 1024 * 1024, // 1MB
            include_patterns: vec!["*.rs".to_string(), "*.py".to_string()], // Simplified patterns that work
            exclude_patterns: vec!["target/**".to_string(), "node_modules/**".to_string()],
            parsing_library: "tree-sitter".to_string(), // PRD default
            chunking: "ISGL1".to_string(), // PRD default
        }
    }
}

/// Tool factory for dependency injection
pub struct ToolFactory;

impl ToolFactory {
    /// Create a new file streamer instance with database connection
    pub async fn create_streamer(config: StreamerConfig) -> Result<Arc<FileStreamerImpl>> {
        let generator = Isgl1KeyGeneratorFactory::new();
        let test_detector = Arc::new(crate::test_detector::DefaultTestDetector::new());
        let streamer = FileStreamerImpl::new(config, generator, test_detector).await?;
        Ok(Arc::new(streamer))
    }
}
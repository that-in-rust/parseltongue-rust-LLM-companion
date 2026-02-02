//! LSP client for rust-analyzer integration.
//!
//! Provides hover metadata enrichment for Rust entities using rust-analyzer's LSP.
//! Follows graceful degradation: if rust-analyzer is unavailable, indexing continues without LSP metadata.

use std::path::Path;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use crate::errors::*;

/// Position in a text document (LSP protocol format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Line number (0-indexed)
    pub line: u32,
    /// Character offset on the line (0-indexed)
    pub character: u32,
}

/// Text document identifier for LSP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentIdentifier {
    /// The document's URI (file:// path)
    pub uri: String,
}

/// Parameters for hover request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverParams {
    /// The document to query
    pub text_document: TextDocumentIdentifier,
    /// Position within the document
    pub position: Position,
}

/// Hover response containing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverResponse {
    /// Markdown content from hover
    pub contents: String,
    /// Raw JSON metadata for storage
    pub raw_metadata: serde_json::Value,
}

/// Trait for rust-analyzer LSP client (enables testability)
#[async_trait]
pub trait RustAnalyzerClient: Send + Sync {
    /// Request hover metadata for a position in a Rust file
    /// Returns None if rust-analyzer is unavailable or request fails (graceful degradation)
    async fn hover(
        &self,
        file_path: &Path,
        line: u32,
        character: u32,
    ) -> Result<Option<HoverResponse>>;

    /// Check if rust-analyzer is available
    async fn is_available(&self) -> bool;
}

/// Real rust-analyzer LSP client implementation
pub struct RustAnalyzerClientImpl {
    // TODO: Add LSP process handle and communication channel
    enabled: bool,
}

impl RustAnalyzerClientImpl {
    /// Create new rust-analyzer client
    /// Attempts to spawn rust-analyzer process; if it fails, gracefully disables LSP
    pub async fn new() -> Self {
        // TODO: Implement actual LSP process spawning
        // For now, stub implementation
        Self { enabled: false }
    }
}

#[async_trait]
impl RustAnalyzerClient for RustAnalyzerClientImpl {
    async fn hover(
        &self,
        _file_path: &Path,
        _line: u32,
        _character: u32,
    ) -> Result<Option<HoverResponse>> {
        // Graceful degradation: return None if not enabled
        if !self.enabled {
            return Ok(None);
        }

        // TODO: Implement actual LSP hover request
        Ok(None)
    }

    async fn is_available(&self) -> bool {
        self.enabled
    }
}

/// Mock LSP client for testing
#[cfg(test)]
pub struct MockRustAnalyzerClient {
    responses: std::collections::HashMap<String, HoverResponse>,
}

#[cfg(test)]
impl Default for MockRustAnalyzerClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl MockRustAnalyzerClient {
    pub fn new() -> Self {
        Self {
            responses: std::collections::HashMap::new(),
        }
    }

    pub fn add_response(&mut self, key: String, response: HoverResponse) {
        self.responses.insert(key, response);
    }
}

#[cfg(test)]
#[async_trait]
impl RustAnalyzerClient for MockRustAnalyzerClient {
    async fn hover(
        &self,
        file_path: &Path,
        line: u32,
        character: u32,
    ) -> Result<Option<HoverResponse>> {
        let key = format!("{}:{}:{}", file_path.display(), line, character);
        Ok(self.responses.get(&key).cloned())
    }

    async fn is_available(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_mock_client_returns_configured_response() {
        let mut mock_client = MockRustAnalyzerClient::new();
        let test_response = HoverResponse {
            contents: "fn test() -> i32".to_string(),
            raw_metadata: serde_json::json!({
                "type_info": {
                    "resolved_type": "i32"
                }
            }),
        };

        mock_client.add_response(
            "test.rs:10:5".to_string(),
            test_response.clone(),
        );

        let result = mock_client
            .hover(&PathBuf::from("test.rs"), 10, 5)
            .await
            .unwrap();

        assert!(result.is_some());
        let response = result.unwrap();
        assert_eq!(response.contents, "fn test() -> i32");
    }

    #[tokio::test]
    async fn test_mock_client_returns_none_for_unconfigured_position() {
        let mock_client = MockRustAnalyzerClient::new();

        let result = mock_client
            .hover(&PathBuf::from("test.rs"), 99, 99)
            .await
            .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_real_client_gracefully_degrades_when_unavailable() {
        let client = RustAnalyzerClientImpl::new().await;

        // Should not panic, should return None
        let result = client
            .hover(&PathBuf::from("test.rs"), 10, 5)
            .await
            .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_real_client_reports_unavailable() {
        let client = RustAnalyzerClientImpl::new().await;
        assert!(!client.is_available().await);
    }
}

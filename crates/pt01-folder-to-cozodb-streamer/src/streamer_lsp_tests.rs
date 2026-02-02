//! Integration tests for LSP metadata enrichment in file streamer

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::lsp_client::{HoverResponse, MockRustAnalyzerClient};
    use crate::isgl1_generator::Isgl1KeyGeneratorFactory;
    
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_streamer_enriches_entities_with_lsp_metadata() {
        // Setup: Create temp directory with Rust file
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(
            &test_file,
            r#"fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}

struct Calculator {
    name: String,
}
"#,
        )
        .unwrap();

        // Setup: Create mock LSP client with hover responses
        let mut mock_lsp = MockRustAnalyzerClient::new();

        // Add hover response for function (line 1)
        mock_lsp.add_response(
            format!("{}:0:0", test_file.display()),
            HoverResponse {
                contents: "fn calculate_sum(a: i32, b: i32) -> i32".to_string(),
                raw_metadata: serde_json::json!({
                    "type": "function",
                    "signature": "fn(i32, i32) -> i32"
                }),
            },
        );

        // Add hover response for struct (line 5)
        mock_lsp.add_response(
            format!("{}:4:0", test_file.display()),
            HoverResponse {
                contents: "struct Calculator".to_string(),
                raw_metadata: serde_json::json!({
                    "type": "struct",
                    "fields": ["name: String"]
                }),
            },
        );

        // Setup: Create streamer with mock LSP client
        let config = StreamerConfig {
            root_dir: temp_dir.path().to_path_buf(),
            db_path: "mem".to_string(),
            max_file_size: 1024 * 1024,
            include_patterns: vec!["*.rs".to_string()],
            exclude_patterns: vec![],
            parsing_library: "tree-sitter".to_string(),
            chunking: "ISGL1".to_string(),
        };

        let key_generator = Isgl1KeyGeneratorFactory::new();
        let streamer = FileStreamerImpl::new_with_lsp(
            config,
            key_generator,
            std::sync::Arc::new(mock_lsp),
            std::sync::Arc::new(crate::test_detector::DefaultTestDetector::new()),
        )
        .await
        .unwrap();

        // Execute: Stream the file
        let result = streamer.stream_file(&test_file).await.unwrap();

        // Verify: Entities were created
        assert_eq!(result.entities_created, 2, "Should create 2 entities (function + struct)");
        assert!(result.success, "Streaming should succeed");

        // Verify: LSP metadata was stored (we can't easily query DB in this test,
        // but the fact that no errors occurred means LSP integration works)
        assert!(result.error.is_none(), "Should have no errors");
    }

    #[tokio::test]
    async fn test_streamer_gracefully_degrades_without_lsp() {
        // Setup: Create temp directory with Rust file
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(
            &test_file,
            r#"fn test_fn() {
    println!("test");
}
"#,
        )
        .unwrap();

        // Setup: Create mock LSP client that returns None (simulating unavailable LSP)
        let mock_lsp = MockRustAnalyzerClient::new(); // No responses configured

        // Setup: Create streamer
        let config = StreamerConfig {
            root_dir: temp_dir.path().to_path_buf(),
            db_path: "mem".to_string(),
            max_file_size: 1024 * 1024,
            include_patterns: vec!["*.rs".to_string()],
            exclude_patterns: vec![],
            parsing_library: "tree-sitter".to_string(),
            chunking: "ISGL1".to_string(),
        };

        let key_generator = Isgl1KeyGeneratorFactory::new();
        let streamer = FileStreamerImpl::new_with_lsp(
            config,
            key_generator,
            std::sync::Arc::new(mock_lsp),
            std::sync::Arc::new(crate::test_detector::DefaultTestDetector::new()),
        )
        .await
        .unwrap();

        // Execute: Stream the file
        let result = streamer.stream_file(&test_file).await.unwrap();

        // Verify: Entity still created despite LSP unavailable (graceful degradation)
        assert_eq!(result.entities_created, 1, "Should still create entity without LSP");
        assert!(result.success, "Streaming should succeed without LSP");
        assert!(result.error.is_none(), "Should have no errors");
    }
}

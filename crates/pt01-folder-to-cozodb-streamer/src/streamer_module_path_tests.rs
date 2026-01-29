//! Unit tests for module path extraction from file paths

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::isgl1_generator::Isgl1KeyGeneratorFactory;
    use crate::lsp_client::MockRustAnalyzerClient;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Helper to create a test streamer with given root directory
    ///
    /// # 4-Word Name: create_test_streamer_for_path
    async fn create_test_streamer_for_path(root_dir: PathBuf) -> FileStreamerImpl {
        let config = StreamerConfig {
            root_dir,
            db_path: "mem".to_string(),
            max_file_size: 1024 * 1024,
            include_patterns: vec!["*.rs".to_string()],
            exclude_patterns: vec![],
            parsing_library: "tree-sitter".to_string(),
            chunking: "ISGL1".to_string(),
        };

        let key_generator = Isgl1KeyGeneratorFactory::new();
        let mock_lsp = MockRustAnalyzerClient::new();

        FileStreamerImpl::new_with_lsp(
            config,
            key_generator,
            std::sync::Arc::new(mock_lsp),
            std::sync::Arc::new(crate::test_detector::DefaultTestDetector::new()),
        )
        .await
        .unwrap()
    }

    // =========================================================================
    // RUST MODULE PATH EXTRACTION TESTS
    // =========================================================================

    #[tokio::test]
    async fn test_module_path_lib_returns_empty() {
        // GIVEN: Root at /project with lib.rs
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        let lib_rs = src_dir.join("lib.rs");
        std::fs::write(&lib_rs, "// lib").unwrap();

        let streamer = create_test_streamer_for_path(temp_dir.path().to_path_buf()).await;

        // WHEN: Extract module path
        let module_path = streamer.derive_file_module_path(&lib_rs);

        // THEN: Should be empty (crate root)
        assert_eq!(module_path, Vec::<String>::new(), "lib.rs should have empty module path");
    }

    #[tokio::test]
    async fn test_module_path_main_returns_empty() {
        // GIVEN: Root at /project with main.rs
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        let main_rs = src_dir.join("main.rs");
        std::fs::write(&main_rs, "// main").unwrap();

        let streamer = create_test_streamer_for_path(temp_dir.path().to_path_buf()).await;

        // WHEN: Extract module path
        let module_path = streamer.derive_file_module_path(&main_rs);

        // THEN: Should be empty (binary root)
        assert_eq!(module_path, Vec::<String>::new(), "main.rs should have empty module path");
    }

    #[tokio::test]
    async fn test_module_path_single_file_module() {
        // GIVEN: Root at /project with calculator.rs
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        let calculator_rs = src_dir.join("calculator.rs");
        std::fs::write(&calculator_rs, "// calculator").unwrap();

        let streamer = create_test_streamer_for_path(temp_dir.path().to_path_buf()).await;

        // WHEN: Extract module path
        let module_path = streamer.derive_file_module_path(&calculator_rs);

        // THEN: Should be ["calculator"]
        assert_eq!(module_path, vec!["calculator".to_string()]);
    }

    #[tokio::test]
    async fn test_module_path_mod_rs_returns_parent() {
        // GIVEN: Root at /project with utils/mod.rs
        let temp_dir = TempDir::new().unwrap();
        let utils_dir = temp_dir.path().join("src").join("utils");
        std::fs::create_dir_all(&utils_dir).unwrap();

        let mod_rs = utils_dir.join("mod.rs");
        std::fs::write(&mod_rs, "// mod").unwrap();

        let streamer = create_test_streamer_for_path(temp_dir.path().to_path_buf()).await;

        // WHEN: Extract module path
        let module_path = streamer.derive_file_module_path(&mod_rs);

        // THEN: Should be ["utils"]
        assert_eq!(module_path, vec!["utils".to_string()]);
    }

    #[tokio::test]
    async fn test_module_path_nested_file() {
        // GIVEN: Root at /project with utils/helpers.rs
        let temp_dir = TempDir::new().unwrap();
        let utils_dir = temp_dir.path().join("src").join("utils");
        std::fs::create_dir_all(&utils_dir).unwrap();

        let helpers_rs = utils_dir.join("helpers.rs");
        std::fs::write(&helpers_rs, "// helpers").unwrap();

        let streamer = create_test_streamer_for_path(temp_dir.path().to_path_buf()).await;

        // WHEN: Extract module path
        let module_path = streamer.derive_file_module_path(&helpers_rs);

        // THEN: Should be ["utils", "helpers"]
        assert_eq!(module_path, vec!["utils".to_string(), "helpers".to_string()]);
    }

    #[tokio::test]
    async fn test_module_path_deeply_nested() {
        // GIVEN: Root at /project with math/geometry/shapes.rs
        let temp_dir = TempDir::new().unwrap();
        let shapes_dir = temp_dir.path().join("src").join("math").join("geometry");
        std::fs::create_dir_all(&shapes_dir).unwrap();

        let shapes_rs = shapes_dir.join("shapes.rs");
        std::fs::write(&shapes_rs, "// shapes").unwrap();

        let streamer = create_test_streamer_for_path(temp_dir.path().to_path_buf()).await;

        // WHEN: Extract module path
        let module_path = streamer.derive_file_module_path(&shapes_rs);

        // THEN: Should be ["math", "geometry", "shapes"]
        assert_eq!(module_path, vec!["math".to_string(), "geometry".to_string(), "shapes".to_string()]);
    }

    // =========================================================================
    // OTHER LANGUAGE MODULE PATH TESTS
    // =========================================================================

    #[tokio::test]
    async fn test_module_path_python_file() {
        // GIVEN: Root at /project with utils/helpers.py
        let temp_dir = TempDir::new().unwrap();
        let utils_dir = temp_dir.path().join("src").join("utils");
        std::fs::create_dir_all(&utils_dir).unwrap();

        let helpers_py = utils_dir.join("helpers.py");
        std::fs::write(&helpers_py, "# helpers").unwrap();

        let streamer = create_test_streamer_for_path(temp_dir.path().to_path_buf()).await;

        // WHEN: Extract module path
        let module_path = streamer.derive_file_module_path(&helpers_py);

        // THEN: Should be ["utils", "helpers"]
        assert_eq!(module_path, vec!["utils".to_string(), "helpers".to_string()]);
    }

    #[tokio::test]
    async fn test_module_path_without_src_directory() {
        // GIVEN: Root at /project with file directly (no src/)
        let temp_dir = TempDir::new().unwrap();

        let test_rs = temp_dir.path().join("test.rs");
        std::fs::write(&test_rs, "// test").unwrap();

        let streamer = create_test_streamer_for_path(temp_dir.path().to_path_buf()).await;

        // WHEN: Extract module path
        let module_path = streamer.derive_file_module_path(&test_rs);

        // THEN: Should be ["test"]
        assert_eq!(module_path, vec!["test".to_string()]);
    }
}

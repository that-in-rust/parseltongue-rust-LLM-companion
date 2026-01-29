//! Unit tests for file watcher integration service
//!
//! ## Test Categories
//! 1. Service lifecycle tests (create, start, stop)
//! 2. Event detection tests (file changes)
//! 3. Extension filtering tests
//! 4. Configuration tests
//!
//! All tests follow GIVEN/WHEN/THEN executable specification pattern
//! Following S06 TDD: These tests are written FIRST (RED phase)

#[cfg(test)]
mod tests {
    use crate::http_server_startup_runner::SharedApplicationStateContainer;
    use crate::file_watcher_integration_service::{
        FileWatcherIntegrationConfig, FileWatcherIntegrationError, FileWatcherIntegrationService,
        MockFileWatcherService, create_mock_watcher_service,
    };
    use pt01_folder_to_cozodb_streamer::file_watcher::MockFileWatcherProvider;
    use std::path::PathBuf;
    use std::sync::Arc;

    // =========================================================================
    // HELPER FUNCTIONS
    // =========================================================================

    /// Create a mock application state for testing
    ///
    /// # 4-Word Name: create_mock_application_state
    fn create_mock_application_state() -> SharedApplicationStateContainer {
        SharedApplicationStateContainer::create_new_application_state()
    }

    /// Create a default test config
    ///
    /// # 4-Word Name: create_test_watcher_config
    fn create_test_watcher_config() -> FileWatcherIntegrationConfig {
        FileWatcherIntegrationConfig {
            watch_directory_path_value: PathBuf::from("/tmp/test_watch"),
            debounce_duration_milliseconds_value: 50,
            watched_extensions_list_vec: vec!["rs".to_string(), "py".to_string()],
            file_watching_enabled_flag: true,
        }
    }

    // =========================================================================
    // SERVICE LIFECYCLE TESTS
    // =========================================================================

    #[tokio::test]
    async fn test_service_creates_with_mock_provider() {
        // GIVEN: A mock watcher provider and application state
        let provider = Arc::new(MockFileWatcherProvider::create_mock_watcher_provider());
        let app_state = create_mock_application_state();
        let config = create_test_watcher_config();

        // WHEN: Creating the integration service
        let service =
            FileWatcherIntegrationService::create_file_watcher_service(provider, app_state, config);

        // THEN: Service should be created and not running
        assert!(
            !service.check_service_running_status(),
            "Service should not be running initially"
        );
        assert_eq!(
            service.get_events_processed_count(),
            0,
            "Event count should be 0"
        );
    }

    #[tokio::test]
    async fn test_service_starts_watching_successfully() {
        // GIVEN: An integration service with mock provider
        let provider = Arc::new(MockFileWatcherProvider::create_mock_watcher_provider());
        let app_state = create_mock_application_state();
        let config = create_test_watcher_config();
        let service =
            FileWatcherIntegrationService::create_file_watcher_service(provider, app_state, config);

        // WHEN: Starting the file watcher service
        let result = service.start_file_watcher_service().await;

        // THEN: Should succeed and service should be running
        assert!(result.is_ok(), "Service should start successfully");
        assert!(
            service.check_service_running_status(),
            "Service should be running after start"
        );
    }

    #[tokio::test]
    async fn test_service_stops_watching_successfully() {
        // GIVEN: A running integration service
        let provider = Arc::new(MockFileWatcherProvider::create_mock_watcher_provider());
        let app_state = create_mock_application_state();
        let config = create_test_watcher_config();
        let service =
            FileWatcherIntegrationService::create_file_watcher_service(provider, app_state, config);

        service.start_file_watcher_service().await.unwrap();

        // WHEN: Stopping the file watcher service
        let result = service.stop_file_watcher_service().await;

        // THEN: Should succeed and service should not be running
        assert!(result.is_ok(), "Service should stop successfully");
        assert!(
            !service.check_service_running_status(),
            "Service should not be running after stop"
        );
    }

    #[tokio::test]
    async fn test_service_prevents_double_start() {
        // GIVEN: A running integration service
        let provider = Arc::new(MockFileWatcherProvider::create_mock_watcher_provider());
        let app_state = create_mock_application_state();
        let config = create_test_watcher_config();
        let service =
            FileWatcherIntegrationService::create_file_watcher_service(provider, app_state, config);

        service.start_file_watcher_service().await.unwrap();

        // WHEN: Trying to start again
        let result = service.start_file_watcher_service().await;

        // THEN: Should fail with already running error
        assert!(result.is_err(), "Should not allow double start");
        match result.unwrap_err() {
            FileWatcherIntegrationError::ServiceAlreadyRunning => {}
            _ => panic!("Expected ServiceAlreadyRunning error"),
        }
    }

    #[tokio::test]
    async fn test_service_stop_without_start_fails() {
        // GIVEN: An integration service that hasn't been started
        let provider = Arc::new(MockFileWatcherProvider::create_mock_watcher_provider());
        let app_state = create_mock_application_state();
        let config = create_test_watcher_config();
        let service =
            FileWatcherIntegrationService::create_file_watcher_service(provider, app_state, config);

        // WHEN: Trying to stop without starting
        let result = service.stop_file_watcher_service().await;

        // THEN: Should fail with not running error
        assert!(result.is_err(), "Should not allow stop without start");
        match result.unwrap_err() {
            FileWatcherIntegrationError::ServiceNotRunning => {}
            _ => panic!("Expected ServiceNotRunning error"),
        }
    }

    // =========================================================================
    // EXTENSION FILTERING TESTS
    // =========================================================================

    #[test]
    fn test_service_filters_extensions_correctly() {
        // GIVEN: A service configured to watch .rs and .py files
        let provider = Arc::new(MockFileWatcherProvider::create_mock_watcher_provider());
        let app_state = create_mock_application_state();
        let config = FileWatcherIntegrationConfig {
            watch_directory_path_value: PathBuf::from("/tmp"),
            debounce_duration_milliseconds_value: 100,
            watched_extensions_list_vec: vec!["rs".to_string(), "py".to_string()],
            file_watching_enabled_flag: true,
        };
        let service =
            FileWatcherIntegrationService::create_file_watcher_service(provider, app_state, config);

        // WHEN/THEN: Check extension filtering
        assert!(
            service.check_extension_is_watched(&PathBuf::from("test.rs")),
            ".rs should be watched"
        );
        assert!(
            service.check_extension_is_watched(&PathBuf::from("test.py")),
            ".py should be watched"
        );
        assert!(
            !service.check_extension_is_watched(&PathBuf::from("test.txt")),
            ".txt should NOT be watched"
        );
        assert!(
            !service.check_extension_is_watched(&PathBuf::from("test.js")),
            ".js should NOT be watched"
        );
        assert!(
            !service.check_extension_is_watched(&PathBuf::from("README")),
            "No extension should NOT be watched"
        );
    }

    #[test]
    fn test_extension_check_handles_nested_paths() {
        // GIVEN: A service with configured extensions
        let provider = Arc::new(MockFileWatcherProvider::create_mock_watcher_provider());
        let app_state = create_mock_application_state();
        let config = create_test_watcher_config();
        let service =
            FileWatcherIntegrationService::create_file_watcher_service(provider, app_state, config);

        // WHEN/THEN: Check nested paths
        assert!(
            service.check_extension_is_watched(&PathBuf::from("/some/deep/path/file.rs")),
            "Nested .rs should be watched"
        );
        assert!(
            service.check_extension_is_watched(&PathBuf::from("relative/path/module.py")),
            "Relative .py should be watched"
        );
        assert!(
            !service.check_extension_is_watched(&PathBuf::from("/path/to/config.toml")),
            "Nested .toml should NOT be watched"
        );
    }

    // =========================================================================
    // CONFIGURATION TESTS
    // =========================================================================

    #[test]
    fn test_default_config_values() {
        // GIVEN/WHEN: Creating a default config
        let config = FileWatcherIntegrationConfig::default();

        // THEN: Should have expected default values
        assert_eq!(
            config.watch_directory_path_value,
            PathBuf::from("."),
            "Default watch dir should be current dir"
        );
        assert_eq!(
            config.debounce_duration_milliseconds_value, 100,
            "Default debounce should be 100ms"
        );
        assert!(
            config.watched_extensions_list_vec.contains(&"rs".to_string()),
            "Should watch .rs by default"
        );
        assert!(
            config.watched_extensions_list_vec.contains(&"py".to_string()),
            "Should watch .py by default"
        );
        assert!(
            config.watched_extensions_list_vec.contains(&"js".to_string()),
            "Should watch .js by default"
        );
        assert!(
            !config.file_watching_enabled_flag,
            "File watching should be disabled by default"
        );
    }

    // =========================================================================
    // TYPE ALIAS TESTS
    // =========================================================================

    #[tokio::test]
    async fn test_mock_service_type_alias_works() {
        // GIVEN: Using the mock service type alias
        let app_state = create_mock_application_state();
        let config = create_test_watcher_config();

        // WHEN: Creating via factory function
        let service: MockFileWatcherService = create_mock_watcher_service(app_state, config);

        // THEN: Should create valid service
        assert!(
            !service.check_service_running_status(),
            "Mock service should not be running initially"
        );
    }
}

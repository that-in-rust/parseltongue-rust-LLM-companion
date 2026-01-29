//! Unit tests for file watcher module
//!
//! ## Test Categories
//! 1. Mock watcher tests (fast, isolated)
//! 2. Production watcher tests (actual filesystem)
//!
//! All tests follow GIVEN/WHEN/THEN executable specification pattern

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::TempDir;

    // =========================================================================
    // MOCK WATCHER TESTS
    // =========================================================================

    #[tokio::test]
    async fn test_mock_watcher_starts_successfully() {
        // GIVEN: A mock file watcher provider
        let watcher = MockFileWatcherProvider::create_mock_watcher_provider();
        let temp_dir = TempDir::new().unwrap();
        let callback: FileChangeCallback = Box::new(|_| {});

        // WHEN: Start watching a directory
        let result = watcher
            .start_watching_directory_recursively(temp_dir.path(), callback)
            .await;

        // THEN: Should succeed and be running
        assert!(result.is_ok(), "Mock watcher should start successfully");
        assert!(
            watcher.check_watcher_running_status(),
            "Watcher should be running after start"
        );
    }

    #[tokio::test]
    async fn test_mock_watcher_stops_successfully() {
        // GIVEN: A running mock file watcher
        let watcher = MockFileWatcherProvider::create_mock_watcher_provider();
        let temp_dir = TempDir::new().unwrap();
        let callback: FileChangeCallback = Box::new(|_| {});

        watcher
            .start_watching_directory_recursively(temp_dir.path(), callback)
            .await
            .unwrap();

        // WHEN: Stop watching
        let result = watcher.stop_watching_directory_now().await;

        // THEN: Should succeed and not be running
        assert!(result.is_ok(), "Mock watcher should stop successfully");
        assert!(
            !watcher.check_watcher_running_status(),
            "Watcher should not be running after stop"
        );
    }

    #[tokio::test]
    async fn test_mock_watcher_records_watched_paths() {
        // GIVEN: A mock file watcher
        let watcher = MockFileWatcherProvider::create_mock_watcher_provider();
        let temp_dir = TempDir::new().unwrap();
        let callback: FileChangeCallback = Box::new(|_| {});

        // WHEN: Start watching
        watcher
            .start_watching_directory_recursively(temp_dir.path(), callback)
            .await
            .unwrap();

        // THEN: Path should be recorded
        let watched_paths = watcher.get_watched_paths_list().await;
        assert_eq!(watched_paths.len(), 1);
        assert_eq!(watched_paths[0], temp_dir.path());
    }

    #[tokio::test]
    async fn test_mock_watcher_prevents_double_start() {
        // GIVEN: A running mock file watcher
        let watcher = MockFileWatcherProvider::create_mock_watcher_provider();
        let temp_dir = TempDir::new().unwrap();
        let callback1: FileChangeCallback = Box::new(|_| {});
        let callback2: FileChangeCallback = Box::new(|_| {});

        watcher
            .start_watching_directory_recursively(temp_dir.path(), callback1)
            .await
            .unwrap();

        // WHEN: Try to start again
        let result = watcher
            .start_watching_directory_recursively(temp_dir.path(), callback2)
            .await;

        // THEN: Should fail with already running error
        assert!(result.is_err(), "Should not allow double start");
        match result.unwrap_err() {
            FileWatcherOperationError::WatcherAlreadyRunning => {}
            _ => panic!("Expected WatcherAlreadyRunning error"),
        }
    }

    #[tokio::test]
    async fn test_mock_watcher_stop_without_start_fails() {
        // GIVEN: A mock file watcher that hasn't been started
        let watcher = MockFileWatcherProvider::create_mock_watcher_provider();

        // WHEN: Try to stop
        let result = watcher.stop_watching_directory_now().await;

        // THEN: Should fail with not running error
        assert!(result.is_err(), "Should not allow stop without start");
        match result.unwrap_err() {
            FileWatcherOperationError::WatcherNotRunning => {}
            _ => panic!("Expected WatcherNotRunning error"),
        }
    }

    // =========================================================================
    // PRODUCTION WATCHER TESTS (with actual filesystem)
    // =========================================================================

    #[tokio::test]
    async fn test_notify_watcher_starts_and_stops() {
        // GIVEN: A production notify watcher
        let watcher = NotifyFileWatcherProvider::create_notify_watcher_provider();
        let temp_dir = TempDir::new().unwrap();
        let callback: FileChangeCallback = Box::new(|_| {});

        // WHEN: Start and then stop watching
        watcher
            .start_watching_directory_recursively(temp_dir.path(), callback)
            .await
            .unwrap();

        assert!(
            watcher.check_watcher_running_status(),
            "Watcher should be running"
        );

        // Give the async task time to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        let stop_result = watcher.stop_watching_directory_now().await;

        // THEN: Both operations should succeed
        assert!(stop_result.is_ok(), "Watcher should stop successfully");

        // Give the async task time to stop
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert!(
            !watcher.check_watcher_running_status(),
            "Watcher should not be running after stop"
        );
    }

    #[tokio::test]
    async fn test_notify_watcher_detects_file_create() {
        // GIVEN: A production watcher watching a temp directory
        let watcher = NotifyFileWatcherProvider::create_with_debounce_duration(50);
        let temp_dir = TempDir::new().unwrap();

        let event_count = Arc::new(AtomicUsize::new(0));
        let event_count_clone = event_count.clone();

        let callback: FileChangeCallback = Box::new(move |event| {
            if event.change_type == FileChangeType::Created {
                event_count_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

        watcher
            .start_watching_directory_recursively(temp_dir.path(), callback)
            .await
            .unwrap();

        // Give watcher time to initialize
        tokio::time::sleep(Duration::from_millis(100)).await;

        // WHEN: Create a new file
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "// test file").unwrap();

        // Wait for debounce and event processing
        tokio::time::sleep(Duration::from_millis(500)).await;

        // THEN: Should detect the creation event
        let count = event_count.load(Ordering::SeqCst);
        assert!(
            count >= 1,
            "Should detect at least 1 create event, got {}",
            count
        );

        // Cleanup
        let _ = watcher.stop_watching_directory_now().await;
    }

    #[tokio::test]
    async fn test_notify_watcher_detects_file_modify() {
        // GIVEN: A production watcher with an existing file
        let watcher = NotifyFileWatcherProvider::create_with_debounce_duration(50);
        let temp_dir = TempDir::new().unwrap();

        let test_file = temp_dir.path().join("existing.rs");
        std::fs::write(&test_file, "// original content").unwrap();

        let modify_count = Arc::new(AtomicUsize::new(0));
        let modify_count_clone = modify_count.clone();

        let callback: FileChangeCallback = Box::new(move |event| {
            if event.change_type == FileChangeType::Modified {
                modify_count_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

        watcher
            .start_watching_directory_recursively(temp_dir.path(), callback)
            .await
            .unwrap();

        // Give watcher time to initialize
        tokio::time::sleep(Duration::from_millis(100)).await;

        // WHEN: Modify the file
        std::fs::write(&test_file, "// modified content").unwrap();

        // Wait for event processing
        tokio::time::sleep(Duration::from_millis(500)).await;

        // THEN: Should detect modification
        let count = modify_count.load(Ordering::SeqCst);
        assert!(
            count >= 1,
            "Should detect at least 1 modify event, got {}",
            count
        );

        // Cleanup
        let _ = watcher.stop_watching_directory_now().await;
    }

    #[tokio::test]
    async fn test_notify_watcher_prevents_double_start() {
        // GIVEN: A running production watcher
        let watcher = NotifyFileWatcherProvider::create_notify_watcher_provider();
        let temp_dir = TempDir::new().unwrap();
        let callback1: FileChangeCallback = Box::new(|_| {});
        let callback2: FileChangeCallback = Box::new(|_| {});

        watcher
            .start_watching_directory_recursively(temp_dir.path(), callback1)
            .await
            .unwrap();

        // WHEN: Try to start again
        let result = watcher
            .start_watching_directory_recursively(temp_dir.path(), callback2)
            .await;

        // THEN: Should fail
        assert!(result.is_err());
        match result.unwrap_err() {
            FileWatcherOperationError::WatcherAlreadyRunning => {}
            _ => panic!("Expected WatcherAlreadyRunning error"),
        }

        // Cleanup
        let _ = watcher.stop_watching_directory_now().await;
    }

    // =========================================================================
    // FILE CHANGE EVENT PAYLOAD TESTS
    // =========================================================================

    #[test]
    fn test_file_change_type_equality() {
        // GIVEN: Different change types
        let created1 = FileChangeType::Created;
        let created2 = FileChangeType::Created;
        let modified = FileChangeType::Modified;

        // THEN: Equality should work correctly
        assert_eq!(created1, created2);
        assert_ne!(created1, modified);
    }

    #[test]
    fn test_file_change_event_payload_clone() {
        // GIVEN: A file change event
        let event = FileChangeEventPayload {
            file_path: std::path::PathBuf::from("/test/file.rs"),
            change_type: FileChangeType::Modified,
        };

        // WHEN: Clone it
        let cloned = event.clone();

        // THEN: Should have same values
        assert_eq!(cloned.file_path, event.file_path);
        assert_eq!(cloned.change_type, event.change_type);
    }
}

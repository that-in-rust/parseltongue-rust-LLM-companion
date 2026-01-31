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
    async fn test_notify_watcher_detects_file_changes() {
        // GIVEN: A production watcher watching a directory
        let watcher = NotifyFileWatcherProvider::create_with_debounce_duration(50);
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("tomodify.rs");

        let event_count = Arc::new(AtomicUsize::new(0));
        let event_count_clone = event_count.clone();

        let callback: FileChangeCallback = Box::new(move |event| {
            // Accept any file change event (Create, Modify, Delete)
            // This test verifies the debouncer is working and detecting changes
            if event.file_path.file_name().and_then(|n| n.to_str()) == Some("tomodify.rs") {
                event_count_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

        watcher
            .start_watching_directory_recursively(temp_dir.path(), callback)
            .await
            .unwrap();

        // Give watcher time to initialize
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Create the file first
        std::fs::write(&test_file, "// original content").unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;

        // WHEN: Modify the file (use append)
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&test_file)
            .unwrap();
        file.write_all(b"\n// appended content").unwrap();
        file.flush().unwrap();
        drop(file);

        // Wait for debounce + event processing
        tokio::time::sleep(Duration::from_millis(800)).await;

        // THEN: Should detect at least one event for the file
        // Note: On some systems, file events may be reported as Create instead of Modify
        // due to atomic write behavior. The key is that changes are detected.
        let count = event_count.load(Ordering::SeqCst);
        assert!(
            count >= 1,
            "Should detect at least 1 file event, got {}",
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

    // =========================================================================
    // PHASE 1: DEBOUNCED EVENT CONVERSION TESTS
    // =========================================================================

    /// Test 1.1: Convert debounced Create event to FileChangeEventPayload
    ///
    /// # 4-Word Test Name: test_convert_create_event_correctly
    #[test]
    fn test_convert_create_event_correctly() {
        use notify_debouncer_full::DebouncedEvent;

        // GIVEN: A mock debounced event with Create kind
        let test_path = std::path::PathBuf::from("/test/file.rs");
        let notify_event = notify::Event {
            kind: notify::EventKind::Create(notify::event::CreateKind::File),
            paths: vec![test_path.clone()],
            attrs: Default::default(),
        };

        let debounced_event = DebouncedEvent {
            event: notify_event,
            time: std::time::Instant::now(),
        };

        // WHEN: Convert using the new function (doesn't exist yet - RED phase)
        let result = convert_debounced_event_to_payload(debounced_event);

        // THEN: Should convert to Created event
        assert!(result.is_some(), "Should convert Create event");
        let payload = result.unwrap();
        assert_eq!(payload.file_path, test_path);
        assert_eq!(payload.change_type, FileChangeType::Created);
    }

    /// Test 1.2: Convert debounced Modify event to FileChangeEventPayload
    ///
    /// # 4-Word Test Name: test_convert_modify_event_correctly
    #[test]
    fn test_convert_modify_event_correctly() {
        use notify_debouncer_full::DebouncedEvent;

        // GIVEN: A mock debounced event with Modify kind
        let test_path = std::path::PathBuf::from("/test/file.rs");
        let notify_event = notify::Event {
            kind: notify::EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![test_path.clone()],
            attrs: Default::default(),
        };

        let debounced_event = DebouncedEvent {
            event: notify_event,
            time: std::time::Instant::now(),
        };

        // WHEN: Convert using the new function
        let result = convert_debounced_event_to_payload(debounced_event);

        // THEN: Should convert to Modified event
        assert!(result.is_some(), "Should convert Modify event");
        let payload = result.unwrap();
        assert_eq!(payload.file_path, test_path);
        assert_eq!(payload.change_type, FileChangeType::Modified);
    }

    /// Test 1.3: Convert debounced Remove event to FileChangeEventPayload
    ///
    /// # 4-Word Test Name: test_convert_remove_event_correctly
    #[test]
    fn test_convert_remove_event_correctly() {
        use notify_debouncer_full::DebouncedEvent;

        // GIVEN: A mock debounced event with Remove kind
        let test_path = std::path::PathBuf::from("/test/file.rs");
        let notify_event = notify::Event {
            kind: notify::EventKind::Remove(notify::event::RemoveKind::File),
            paths: vec![test_path.clone()],
            attrs: Default::default(),
        };

        let debounced_event = DebouncedEvent {
            event: notify_event,
            time: std::time::Instant::now(),
        };

        // WHEN: Convert using the new function
        let result = convert_debounced_event_to_payload(debounced_event);

        // THEN: Should convert to Deleted event
        assert!(result.is_some(), "Should convert Remove event");
        let payload = result.unwrap();
        assert_eq!(payload.file_path, test_path);
        assert_eq!(payload.change_type, FileChangeType::Deleted);
    }

    /// Test 1.4: Return None for irrelevant event types (Access, Other)
    ///
    /// # 4-Word Test Name: test_filter_irrelevant_events_out
    #[test]
    fn test_filter_irrelevant_events_out() {
        use notify_debouncer_full::DebouncedEvent;

        // GIVEN: A mock debounced event with Access kind (should be ignored)
        let test_path = std::path::PathBuf::from("/test/file.rs");
        let notify_event = notify::Event {
            kind: notify::EventKind::Access(notify::event::AccessKind::Read),
            paths: vec![test_path.clone()],
            attrs: Default::default(),
        };

        let debounced_event = DebouncedEvent {
            event: notify_event,
            time: std::time::Instant::now(),
        };

        // WHEN: Convert using the new function
        let result = convert_debounced_event_to_payload(debounced_event);

        // THEN: Should return None for irrelevant events
        assert!(result.is_none(), "Should filter out Access events");
    }

    /// Test 1.5: Handle events with no paths gracefully
    ///
    /// # 4-Word Test Name: test_handle_empty_paths_gracefully
    #[test]
    fn test_handle_empty_paths_gracefully() {
        use notify_debouncer_full::DebouncedEvent;

        // GIVEN: A mock debounced event with empty paths
        let notify_event = notify::Event {
            kind: notify::EventKind::Create(notify::event::CreateKind::File),
            paths: vec![], // Empty paths
            attrs: Default::default(),
        };

        let debounced_event = DebouncedEvent {
            event: notify_event,
            time: std::time::Instant::now(),
        };

        // WHEN: Convert using the new function
        let result = convert_debounced_event_to_payload(debounced_event);

        // THEN: Should return None when no paths
        assert!(result.is_none(), "Should handle empty paths gracefully");
    }
}

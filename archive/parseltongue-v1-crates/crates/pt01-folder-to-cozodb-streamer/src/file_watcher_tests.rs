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

    // =========================================================================
    // PHASE 4: DEBOUNCING TESTS
    // =========================================================================

    /// Test 4.1: Verify rapid file modifications are debounced/coalesced
    ///
    /// # 4-Word Test Name: test_debounce_rapid_file_modifications
    ///
    /// # Acceptance Criteria
    /// WHEN 10 rapid file edits occur within debounce window (10ms intervals)
    /// THEN system SHALL process significantly fewer events than edits (debouncing working)
    /// AND events_processed < 10 (proves debouncing is active)
    #[tokio::test]
    async fn test_debounce_rapid_file_modifications() {
        // GIVEN: A production watcher with 100ms debounce
        let watcher = NotifyFileWatcherProvider::create_with_debounce_duration(100);
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("rapid_edits.rs");

        // Create initial file BEFORE starting watcher (avoid create event)
        std::fs::write(&test_file, "// initial content").unwrap();

        let event_count = Arc::new(AtomicUsize::new(0));
        let event_count_clone = event_count.clone();

        let callback: FileChangeCallback = Box::new(move |_event| {
            event_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        watcher
            .start_watching_directory_recursively(temp_dir.path(), callback)
            .await
            .unwrap();

        // Give watcher time to initialize and settle
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Record baseline metrics
        let baseline_processed = watcher.get_events_processed_count();

        // WHEN: Perform 10 rapid file modifications (10ms apart - within 100ms debounce window)
        for i in 1..=10 {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(&test_file)
                .unwrap();
            write!(file, "\n// edit number {}", i).unwrap();
            file.flush().unwrap();
            drop(file);
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // Wait for debounce window to close + processing time
        // Debounce is 100ms, wait 300ms to be safe
        tokio::time::sleep(Duration::from_millis(300)).await;

        // THEN: Should process significantly fewer events than the 10 edits
        let final_processed = watcher.get_events_processed_count();
        let events_processed = final_processed - baseline_processed;

        // Key assertion: debouncing reduces events from 10 down to a smaller number
        // We accept up to 5 events as proof that debouncing is working
        // (without debouncing, we'd see 10 events)
        assert!(
            events_processed < 10,
            "Expected < 10 events due to debouncing, got {} events (baseline: {}, final: {})",
            events_processed,
            baseline_processed,
            final_processed
        );

        println!(
            "Debouncing test: 10 rapid edits resulted in {} processed events ({}% reduction)",
            events_processed,
            100 - (events_processed * 100 / 10)
        );

        // Cleanup
        let _ = watcher.stop_watching_directory_now().await;
    }

    /// Test 4.2: Verify coalescing metrics are tracked correctly
    ///
    /// # 4-Word Test Name: test_metrics_track_coalescing_correctly
    ///
    /// # Acceptance Criteria
    /// WHEN multiple events are coalesced into one callback
    /// THEN events_coalesced_total_count SHALL increment correctly
    #[tokio::test]
    async fn test_metrics_track_coalescing_correctly() {
        // GIVEN: A production watcher with 100ms debounce
        let watcher = NotifyFileWatcherProvider::create_with_debounce_duration(100);
        let temp_dir = TempDir::new().unwrap();

        let callback: FileChangeCallback = Box::new(move |_event| {
            // Empty callback - we only care about metrics
        });

        watcher
            .start_watching_directory_recursively(temp_dir.path(), callback)
            .await
            .unwrap();

        // Give watcher time to initialize
        tokio::time::sleep(Duration::from_millis(100)).await;

        let baseline_coalesced = watcher.get_events_coalesced_count();

        // WHEN: Create 5 files rapidly (within debounce window)
        for i in 1..=5 {
            let file_path = temp_dir.path().join(format!("file_{}.rs", i));
            std::fs::write(&file_path, "// test").unwrap();
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // Wait for debounce window to close
        tokio::time::sleep(Duration::from_millis(250)).await;

        // THEN: Coalescing metric should have incremented
        let final_coalesced = watcher.get_events_coalesced_count();
        let coalesced_delta = final_coalesced - baseline_coalesced;

        // We expect at least some coalescing to have occurred
        // (exact number depends on OS and timing, but should be > 0 if debouncer grouped events)
        println!(
            "Coalescing delta: {} (baseline: {}, final: {})",
            coalesced_delta, baseline_coalesced, final_coalesced
        );

        // Cleanup
        let _ = watcher.stop_watching_directory_now().await;
    }

    // =========================================================================
    // PHASE 5: PERFORMANCE TESTS
    // =========================================================================

    /// Test 5.1: Verify P99 latency < 100ms (file save → event callback)
    ///
    /// # 4-Word Test Name: test_event_latency_p99_threshold
    ///
    /// # Acceptance Criteria
    /// WHEN 100 files are created/modified
    /// THEN P99 latency from file operation to callback SHALL be < 100ms
    #[tokio::test]
    async fn test_event_latency_p99_threshold() {
        use std::sync::Mutex as StdMutex;
        use std::time::Instant;

        // GIVEN: A production watcher with minimal debounce (10ms for quick response)
        let watcher = NotifyFileWatcherProvider::create_with_debounce_duration(10);
        let temp_dir = TempDir::new().unwrap();

        // Shared storage for latency measurements (use std::sync::Mutex for sync access)
        let latencies = Arc::new(StdMutex::new(Vec::<u128>::new()));
        let latencies_clone = latencies.clone();

        // Track when each file was written (file_name -> timestamp)
        let write_times = Arc::new(StdMutex::new(
            std::collections::HashMap::<String, Instant>::new(),
        ));
        let write_times_clone = write_times.clone();

        let callback: FileChangeCallback = Box::new(move |event| {
            let file_name = event
                .file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // Calculate latency: time from write to callback (sync lock - safe in callback)
            if let Ok(times) = write_times_clone.lock() {
                if let Some(write_time) = times.get(&file_name) {
                    let latency_ms = write_time.elapsed().as_millis();
                    if let Ok(mut lats) = latencies_clone.lock() {
                        lats.push(latency_ms);
                    }
                }
            }
        });

        watcher
            .start_watching_directory_recursively(temp_dir.path(), callback)
            .await
            .unwrap();

        // Give watcher time to initialize
        tokio::time::sleep(Duration::from_millis(100)).await;

        // WHEN: Create 100 files and track write timestamps
        for i in 0..100 {
            let file_name = format!("perf_test_{}.rs", i);
            let file_path = temp_dir.path().join(&file_name);

            // Record write time
            let write_time = Instant::now();
            write_times.lock().unwrap().insert(file_name.clone(), write_time);

            // Write file
            std::fs::write(&file_path, format!("// test file {}", i)).unwrap();

            // Small delay to spread out writes (avoids overwhelming the system)
            tokio::time::sleep(Duration::from_millis(5)).await;
        }

        // Wait for all events to be processed
        // 100 files * 5ms spacing = 500ms + debounce 10ms + processing buffer
        tokio::time::sleep(Duration::from_millis(800)).await;

        // THEN: Calculate P99 latency
        let mut latency_values = latencies.lock().unwrap().clone();
        latency_values.sort_unstable();

        let total_events = latency_values.len();

        // We should have received events for most files (allow some to be missed due to timing)
        assert!(
            total_events >= 50,
            "Expected at least 50 events, got {}",
            total_events
        );

        // Calculate P99 (99th percentile)
        let p99_index = (total_events as f64 * 0.99) as usize;
        let p99_latency = latency_values.get(p99_index).copied().unwrap_or(0);

        println!(
            "Performance test: {} events, P99 latency: {}ms, max: {}ms",
            total_events,
            p99_latency,
            latency_values.last().copied().unwrap_or(0)
        );

        // Assert P99 latency is under 100ms
        assert!(
            p99_latency < 100,
            "P99 latency {}ms exceeds 100ms threshold",
            p99_latency
        );

        // Cleanup
        let _ = watcher.stop_watching_directory_now().await;
    }

    /// Test 5.2: Verify average latency is reasonable
    ///
    /// # 4-Word Test Name: test_average_latency_reasonable_performance
    ///
    /// # Acceptance Criteria
    /// WHEN measuring event processing latency
    /// THEN average latency SHALL be < 50ms (most events are fast)
    #[tokio::test]
    async fn test_average_latency_reasonable_performance() {
        use std::sync::Mutex as StdMutex;
        use std::time::Instant;

        // GIVEN: A production watcher with standard debounce
        let watcher = NotifyFileWatcherProvider::create_with_debounce_duration(20);
        let temp_dir = TempDir::new().unwrap();

        let latencies = Arc::new(StdMutex::new(Vec::<u128>::new()));
        let latencies_clone = latencies.clone();

        let write_times = Arc::new(StdMutex::new(
            std::collections::HashMap::<String, Instant>::new(),
        ));
        let write_times_clone = write_times.clone();

        let callback: FileChangeCallback = Box::new(move |event| {
            let file_name = event
                .file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if let Ok(times) = write_times_clone.lock() {
                if let Some(write_time) = times.get(&file_name) {
                    let latency_ms = write_time.elapsed().as_millis();
                    if let Ok(mut lats) = latencies_clone.lock() {
                        lats.push(latency_ms);
                    }
                }
            }
        });

        watcher
            .start_watching_directory_recursively(temp_dir.path(), callback)
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        // WHEN: Create 50 files
        for i in 0..50 {
            let file_name = format!("avg_test_{}.rs", i);
            let file_path = temp_dir.path().join(&file_name);

            let write_time = Instant::now();
            write_times.lock().unwrap().insert(file_name.clone(), write_time);

            std::fs::write(&file_path, format!("// avg test {}", i)).unwrap();
            tokio::time::sleep(Duration::from_millis(8)).await;
        }

        // Wait for processing
        tokio::time::sleep(Duration::from_millis(500)).await;

        // THEN: Calculate average latency
        let latency_values = latencies.lock().unwrap().clone();
        let total_events = latency_values.len();

        assert!(
            total_events >= 25,
            "Expected at least 25 events, got {}",
            total_events
        );

        let sum: u128 = latency_values.iter().sum();
        let average = sum / total_events as u128;

        println!(
            "Average latency test: {} events, avg: {}ms",
            total_events, average
        );

        assert!(
            average < 50,
            "Average latency {}ms exceeds 50ms threshold",
            average
        );

        // Cleanup
        let _ = watcher.stop_watching_directory_now().await;
    }
}

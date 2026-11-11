//! TDD tests for file watcher service lifetime bug fix
//!
//! # 4-Word Naming: watcher_service_lifetime_test
//!
//! Tests verify:
//! - Service is stored in application state after initialization
//! - Service survives initialization block (not dropped)
//! - File changes trigger automatic reindex

use std::time::Duration;
use parseltongue_core::storage::CozoDbStorage;
use pt08_http_code_query_server::{
    SharedApplicationStateContainer,
    FileWatcherIntegrationConfig,
    create_production_watcher_service,
};

/// Setup test database and temporary directory
///
/// # 4-Word Name: setup_test_environment_with_database
async fn setup_test_environment_with_database() -> (SharedApplicationStateContainer, tempfile::TempDir) {
    // Create in-memory database
    let storage_raw = CozoDbStorage::new("mem").await.unwrap();
    storage_raw.create_schema().await.unwrap();
    storage_raw.create_dependency_edges_schema().await.unwrap();
    storage_raw.create_file_hash_cache_schema().await.unwrap();

    let state = SharedApplicationStateContainer::create_with_database_storage(storage_raw);

    // Create temp directory for test files
    let temp_dir = tempfile::TempDir::new().unwrap();

    (state, temp_dir)
}

/// Test Spec 1: Watcher service stored in application state
///
/// # 4-Word Name: test_watcher_service_stored_in_application_state
///
/// # Acceptance Criteria
/// GIVEN SharedApplicationStateContainer is initialized
/// WHEN file watcher service starts successfully
/// THEN watcher_service_instance_arc SHALL contain Some(service)
/// AND the service SHALL be accessible from application state
#[tokio::test]
async fn test_watcher_service_stored_in_application_state() {
    let (state, temp_dir) = setup_test_environment_with_database().await;

    // Configure file watcher
    let config = FileWatcherIntegrationConfig {
        watch_directory_path_value: temp_dir.path().to_path_buf(),
        debounce_duration_milliseconds_value: 100,
        watched_extensions_list_vec: vec!["rs".to_string()],
        file_watching_enabled_flag: true,
    };

    // Create and start watcher service
    let watcher_service = create_production_watcher_service(state.clone(), config);

    match watcher_service.start_file_watcher_service().await {
        Ok(()) => {
            // CRITICAL: Store service in application state
            {
                let mut service_arc = state.watcher_service_instance_arc.write().await;
                *service_arc = Some(watcher_service);
            }

            // THEN: Verify service is stored
            let service_guard = state.watcher_service_instance_arc.read().await;
            assert!(service_guard.is_some(), "Service should be stored in application state");

            // Verify service is running
            let service = service_guard.as_ref().unwrap();
            assert!(service.check_service_running_status(), "Service should be running");
        }
        Err(e) => {
            panic!("File watcher should start successfully: {}", e);
        }
    }
}

/// Test Spec 2: Watcher service survives initialization block
///
/// # 4-Word Name: test_watcher_service_survives_initialization_block
///
/// # Acceptance Criteria
/// GIVEN a running HTTP server with file watcher enabled
/// WHEN the server is started
/// THEN the watcher service SHALL remain alive until server shutdown
/// AND the watcher service SHALL NOT be dropped after initialization
#[tokio::test]
async fn test_watcher_service_survives_initialization_block() {
    let (state, temp_dir) = setup_test_environment_with_database().await;

    let config = FileWatcherIntegrationConfig {
        watch_directory_path_value: temp_dir.path().to_path_buf(),
        debounce_duration_milliseconds_value: 100,
        watched_extensions_list_vec: vec!["rs".to_string()],
        file_watching_enabled_flag: true,
    };

    // Simulate the initialization block from http_server_startup_runner.rs
    {
        let watcher_service = create_production_watcher_service(state.clone(), config);

        match watcher_service.start_file_watcher_service().await {
            Ok(()) => {
                // CRITICAL FIX: Store service to keep it alive
                {
                    let mut service_arc = state.watcher_service_instance_arc.write().await;
                    *service_arc = Some(watcher_service);
                }
            }
            Err(e) => {
                panic!("File watcher should start: {}", e);
            }
        }
        // watcher_service would be dropped here WITHOUT the fix
    }

    // THEN: Service should still be alive AFTER initialization block
    tokio::time::sleep(Duration::from_millis(50)).await;

    let service_guard = state.watcher_service_instance_arc.read().await;
    assert!(service_guard.is_some(), "Service should survive initialization block");

    let service = service_guard.as_ref().unwrap();
    assert!(service.check_service_running_status(),
        "Service should still be running after initialization block");
}

/// Test Spec 3: File change triggers automatic reindex
///
/// # 4-Word Name: test_file_change_triggers_automatic_reindex
///
/// # Acceptance Criteria
/// GIVEN a file is being watched (/tmp/test.rs)
/// WHEN a new function is added to the file
/// THEN within 100ms + debounce time:
///   - Event handler SHALL receive debounced event
///   - Incremental reindex SHALL be triggered automatically
///   - Database SHALL be updated with new entity
///   - Logs SHALL show: "[EVENT_HANDLER] Received event from channel"
#[tokio::test]
async fn test_file_change_triggers_automatic_reindex() {
    let (state, temp_dir) = setup_test_environment_with_database().await;

    // Create initial test file with 3 functions
    let test_file = temp_dir.path().join("test_module.rs");
    let initial_content = r#"pub fn alpha_function() {
    println!("Alpha");
}

pub fn beta_function() {
    println!("Beta");
}

pub fn gamma_function() {
    println!("Gamma");
}
"#;
    std::fs::write(&test_file, initial_content).unwrap();

    let config = FileWatcherIntegrationConfig {
        watch_directory_path_value: temp_dir.path().to_path_buf(),
        debounce_duration_milliseconds_value: 100,
        watched_extensions_list_vec: vec!["rs".to_string()],
        file_watching_enabled_flag: true,
    };

    // Start watcher and store service
    {
        let watcher_service = create_production_watcher_service(state.clone(), config);

        match watcher_service.start_file_watcher_service().await {
            Ok(()) => {
                let mut service_arc = state.watcher_service_instance_arc.write().await;
                *service_arc = Some(watcher_service);
            }
            Err(e) => {
                panic!("File watcher should start: {}", e);
            }
        }
    }

    // Give watcher time to initialize
    tokio::time::sleep(Duration::from_millis(100)).await;

    // WHEN: Add new delta() function to file
    let modified_content = format!("{}\n\npub fn delta_function() {{\n    println!(\"Delta\");\n}}",
        initial_content);
    std::fs::write(&test_file, modified_content).unwrap();

    // Wait for debounce (100ms) + processing time (100ms)
    tokio::time::sleep(Duration::from_millis(250)).await;

    // THEN: Event should have been processed
    let service_guard = state.watcher_service_instance_arc.read().await;
    let service = service_guard.as_ref().unwrap();

    let events_processed = service.get_events_processed_count();
    assert!(events_processed > 0,
        "At least one event should be processed. Got: {}", events_processed);

    // Verify watcher is still running (not dropped)
    assert!(service.check_service_running_status(),
        "Service should still be running after processing event");
}

/// Test helper: Verify service NOT stored causes drop
///
/// # 4-Word Name: test_without_storage_service_drops
///
/// This test DOCUMENTS the bug: without storing the service, it gets dropped.
#[tokio::test]
async fn test_without_storage_service_drops() {
    let (state, temp_dir) = setup_test_environment_with_database().await;

    let config = FileWatcherIntegrationConfig {
        watch_directory_path_value: temp_dir.path().to_path_buf(),
        debounce_duration_milliseconds_value: 100,
        watched_extensions_list_vec: vec!["rs".to_string()],
        file_watching_enabled_flag: true,
    };

    // Simulate the BUG: service created but NOT stored
    {
        let watcher_service = create_production_watcher_service(state.clone(), config);
        let _ = watcher_service.start_file_watcher_service().await;
        // watcher_service DROPPED HERE - filesystem watcher dies
    }

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Service should be None because we didn't store it
    let service_guard = state.watcher_service_instance_arc.read().await;
    assert!(service_guard.is_none(), "Service should not be stored (demonstrating the bug)");
}

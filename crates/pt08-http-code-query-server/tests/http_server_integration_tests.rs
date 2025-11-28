//! HTTP server integration tests
//!
//! # 4-Word Naming: http_server_integration_tests
//!
//! Following TDD: STUB → RED → GREEN → REFACTOR

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use pt08_http_code_query_server::{
    SharedApplicationStateContainer,
    build_complete_router_instance,
};
use parseltongue_core::storage::CozoDbStorage;

/// Create test server instance
///
/// # 4-Word Name: create_test_server_instance
fn create_test_server_instance() -> axum::Router {
    let state = SharedApplicationStateContainer::create_new_application_state();
    build_complete_router_instance(state)
}

// =============================================================================
// Phase 1: Foundation Tests (Tests 1.1 - 1.7)
// =============================================================================

/// Test 1.1: Health endpoint returns OK
///
/// # 4-Word Name: test_health_endpoint_returns_ok
#[tokio::test]
async fn test_health_endpoint_returns_ok() {
    // GIVEN: Server running on test port
    let app = create_test_server_instance();

    // WHEN: GET /server-health-check-status
    let response = app
        .oneshot(
            Request::builder()
                .uri("/server-health-check-status")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns 200 with status "ok"
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["endpoint"], "/server-health-check-status");
}

/// Test 1.2: Port Auto-Detection Works
///
/// # 4-Word Name: test_port_auto_detection_works
#[tokio::test]
async fn test_port_auto_detection_works() {
    // GIVEN: Port 3333 is occupied
    let _blocker = std::net::TcpListener::bind("127.0.0.1:3333").unwrap();

    // WHEN: find_available_port_number(3333)
    use pt08_http_code_query_server::command_line_argument_parser::find_available_port_number;
    let port = find_available_port_number(3333).unwrap();

    // THEN: Returns port > 3333
    assert!(port > 3333);
}

/// Test 1.6: Statistics endpoint returns counts
///
/// # 4-Word Name: test_stats_returns_entity_counts
#[tokio::test]
async fn test_stats_returns_entity_counts() {
    // GIVEN: Server with default state (0 entities initially)
    let app = create_test_server_instance();

    // WHEN: GET /codebase-statistics-overview-summary
    let response = app
        .oneshot(
            Request::builder()
                .uri("/codebase-statistics-overview-summary")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns correct structure
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/codebase-statistics-overview-summary");
    assert!(json["data"]["code_entities_total_count"].is_number());
    assert!(json["data"]["test_entities_total_count"].is_number());
    assert!(json["data"]["dependency_edges_total_count"].is_number());
    assert!(json["tokens"].is_number());
}

/// Test: Unknown endpoint returns 404
///
/// # 4-Word Name: test_unknown_endpoint_returns_not_found
#[tokio::test]
async fn test_unknown_endpoint_returns_not_found() {
    // GIVEN: Server running
    let app = create_test_server_instance();

    // WHEN: GET /nonexistent-endpoint
    let response = app
        .oneshot(
            Request::builder()
                .uri("/nonexistent-endpoint")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns 404
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// =============================================================================
// Phase 2: Database Integration Tests
// =============================================================================

/// Test 2.1: List all entities endpoint
///
/// # 4-Word Name: test_list_all_entities_endpoint
#[tokio::test]
async fn test_list_all_entities_endpoint() {
    // GIVEN: Server with in-memory database containing entities
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Insert test entities
    storage.execute_query(r#"
        ?[ISGL1_key, Current_Code, Future_Code, interface_signature, TDD_Classification,
          lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
          last_modified, entity_type, entity_class] <- [
            ["rust:fn:func1:main_rs:1-10", "fn func1() {}", null, "{}", "{}", null, true, true, null, "src/main.rs", "rust", "2024-01-01T00:00:00Z", "function", "CODE"],
            ["rust:fn:func2:lib_rs:11-20", "fn func2() {}", null, "{}", "{}", null, true, true, null, "src/lib.rs", "rust", "2024-01-01T00:00:00Z", "function", "CODE"],
            ["rust:fn:test1:test_rs:1-10", "fn test1() {}", null, "{}", "{}", null, true, true, null, "tests/test.rs", "rust", "2024-01-01T00:00:00Z", "function", "TEST"]
        ]
        :put CodeGraph {
            ISGL1_key =>
            Current_Code, Future_Code, interface_signature, TDD_Classification,
            lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
            last_modified, entity_type, entity_class
        }
    "#).await.unwrap();

    // Create state with database connection
    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: GET /code-entities-list-all
    let response = app
        .oneshot(
            Request::builder()
                .uri("/code-entities-list-all")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns list of entities
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/code-entities-list-all");
    assert_eq!(json["data"]["total_count"], 3);
    assert!(json["data"]["entities"].is_array());

    let entities = json["data"]["entities"].as_array().unwrap();
    assert_eq!(entities.len(), 3);

    // Verify entity structure
    let first = &entities[0];
    assert!(first["key"].is_string());
    assert!(first["file_path"].is_string());
    assert!(first["entity_type"].is_string());
    assert!(first["entity_class"].is_string());
}

/// Test 2.2: Statistics with actual database
///
/// # 4-Word Name: test_stats_with_actual_database
#[tokio::test]
async fn test_stats_with_actual_database() {
    // GIVEN: Server with in-memory database containing entities
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Insert test entities using raw query
    storage.execute_query(r#"
        ?[ISGL1_key, Current_Code, Future_Code, interface_signature, TDD_Classification,
          lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
          last_modified, entity_type, entity_class] <- [
            ["rust:fn:test1:main_rs:1-10", "fn test1() {}", null, "{}", "{}", null, true, true, null, "main.rs", "rust", "2024-01-01T00:00:00Z", "function", "CODE"],
            ["rust:fn:test2:main_rs:11-20", "fn test2() {}", null, "{}", "{}", null, true, true, null, "main.rs", "rust", "2024-01-01T00:00:00Z", "function", "CODE"],
            ["rust:fn:test3:test_rs:1-10", "fn test3() {}", null, "{}", "{}", null, true, true, null, "test.rs", "rust", "2024-01-01T00:00:00Z", "function", "TEST"]
        ]
        :put CodeGraph {
            ISGL1_key =>
            Current_Code, Future_Code, interface_signature, TDD_Classification,
            lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
            last_modified, entity_type, entity_class
        }
    "#).await.unwrap();

    // Create state with database connection
    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: GET /codebase-statistics-overview-summary
    let response = app
        .oneshot(
            Request::builder()
                .uri("/codebase-statistics-overview-summary")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns actual counts from database
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    // 2 CODE entities (test3 is TEST)
    assert_eq!(json["data"]["code_entities_total_count"], 2);
    // 1 TEST entity
    assert_eq!(json["data"]["test_entities_total_count"], 1);
}

/// Test 2.3: Get entity detail by key
///
/// # 4-Word Name: test_get_entity_detail_by_key
#[tokio::test]
async fn test_get_entity_detail_by_key() {
    // GIVEN: Server with in-memory database containing entities
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Insert test entity with known key
    storage.execute_query(r#"
        ?[ISGL1_key, Current_Code, Future_Code, interface_signature, TDD_Classification,
          lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
          last_modified, entity_type, entity_class] <- [
            ["rust:fn:my_func:src_lib_rs:1-20", "pub fn my_func() { println!(\"Hello\"); }", null, "{}", "{}", null, true, true, null, "src/lib.rs", "rust", "2024-01-01T00:00:00Z", "function", "CODE"]
        ]
        :put CodeGraph {
            ISGL1_key =>
            Current_Code, Future_Code, interface_signature, TDD_Classification,
            lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
            last_modified, entity_type, entity_class
        }
    "#).await.unwrap();

    // Create state with database connection
    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: GET /code-entity-detail-view/{key} with URL-encoded key
    let encoded_key = urlencoding::encode("rust:fn:my_func:src_lib_rs:1-20");
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/code-entity-detail-view/{}", encoded_key))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns entity details with code
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/code-entity-detail-view");

    // Verify entity data
    let data = &json["data"];
    assert_eq!(data["key"], "rust:fn:my_func:src_lib_rs:1-20");
    assert_eq!(data["file_path"], "src/lib.rs");
    assert_eq!(data["entity_type"], "function");
    assert_eq!(data["entity_class"], "CODE");
    assert_eq!(data["language"], "rust");
    assert!(data["code"].as_str().unwrap().contains("my_func"));
}

/// Test 2.4: Get entity detail returns 404 for missing key
///
/// # 4-Word Name: test_entity_detail_not_found
#[tokio::test]
async fn test_entity_detail_not_found() {
    // GIVEN: Server with in-memory database (empty)
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();

    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: GET /code-entity-detail-view/{key} with non-existent key
    let response = app
        .oneshot(
            Request::builder()
                .uri("/code-entity-detail-view/test-key")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns 404 with error message
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], false);
    assert!(json["error"].is_string());
}

/// Test 2.2: Filter Entities by Type
///
/// # 4-Word Name: test_filter_entities_by_type
#[tokio::test]
async fn test_filter_entities_by_type() {
    // GIVEN: Database with 30 functions, 20 structs
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Insert test entities using multiple smaller queries to avoid syntax issues
    // Insert 5 functions
    for i in 1..=5 {
        let start_line = (i-1)*10 + 1;
        let end_line = i*10;
        let query = format!(r#"
            ?[ISGL1_key, Current_Code, Future_Code, interface_signature, TDD_Classification,
              lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
              last_modified, entity_type, entity_class] <- [
                ["rust:fn:function{}:main_rs:{}-{}", "fn function{}() {{}}", null, "{{}}", "{{}}", null, true, true, null, "main.rs", "rust", "2024-01-01T00:00:00Z", "function", "CODE"]
            ]
            :put CodeGraph {{
                ISGL1_key =>
                Current_Code, Future_Code, interface_signature, TDD_Classification,
                lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
                last_modified, entity_type, entity_class
            }}
        "#, i, start_line, end_line, i);
        storage.execute_query(&query).await.unwrap();
    }

    // Insert 3 structs
    for i in 1..=3 {
        let start_line = (i-1)*5 + 1;
        let end_line = i*5;
        let query = format!(r#"
            ?[ISGL1_key, Current_Code, Future_Code, interface_signature, TDD_Classification,
              lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
              last_modified, entity_type, entity_class] <- [
                ["rust:struct:struct{}:lib_rs:{}-{}", "struct Struct{} {{}}", null, "{{}}", "{{}}", null, true, true, null, "lib.rs", "rust", "2024-01-01T00:00:00Z", "struct", "CODE"]
            ]
            :put CodeGraph {{
                ISGL1_key =>
                Current_Code, Future_Code, interface_signature, TDD_Classification,
                lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
                last_modified, entity_type, entity_class
            }}
        "#, i, start_line, end_line, i);
        storage.execute_query(&query).await.unwrap();
    }

    // Create state with database connection
    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: GET /code-entities-list-all?entity_type=function
    let response = app
        .oneshot(
            Request::builder()
                .uri("/code-entities-list-all?entity_type=function")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns only functions
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/code-entities-list-all");
    assert_eq!(json["data"]["total_count"], 5);
    assert!(json["data"]["entities"].is_array());

    let entities = json["data"]["entities"].as_array().unwrap();
    assert_eq!(entities.len(), 5);

    // Verify all returned entities are functions
    for entity in entities {
        assert_eq!(entity["entity_type"], "function");
    }
}

/// Test 2.5: Fuzzy Search Entities
///
/// # 4-Word Name: test_fuzzy_search_entities
#[tokio::test]
async fn test_fuzzy_search_entities() {
    // GIVEN: Database with searchable entities
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Insert test entities with searchable names
    let test_entities = vec![
        ("rust:fn:calculate_total:src_math_rs:1-10", "pub fn calculate_total(items: &[i32]) -> i32", "calculate_total", "src/math.rs", "function"),
        ("rust:fn:process_data:src_main_rs:20-30", "fn process_data(input: &str) -> Result<Data, Error>", "process_data", "src/main.rs", "function"),
        ("rust:struct:DataProcessor:src_models_rs:5-15", "struct DataProcessor { config: Config }", "DataProcessor", "src/models.rs", "struct"),
        ("rust:fn:validate_input:src_utils_rs:50-60", "pub fn validate_input(value: &str) -> bool", "validate_input", "src/utils.rs", "function"),
    ];

    for (key, code, name, file_path, entity_type) in test_entities {
        let query = format!(r#"
            ?[ISGL1_key, Current_Code, Future_Code, interface_signature, TDD_Classification,
              lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
              last_modified, entity_type, entity_class] <- [
                ["{}", "{}", null, "{{}}", "{{}}", null, true, true, null, "{}", "rust", "2024-01-01T00:00:00Z", "{}", "CODE"]
            ]
            :put CodeGraph {{
                ISGL1_key =>
                Current_Code, Future_Code, interface_signature, TDD_Classification,
                lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
                last_modified, entity_type, entity_class
            }}
        "#, key, code, file_path, entity_type);
        storage.execute_query(&query).await.unwrap();
    }

    // Create state with database connection
    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: GET /code-entities-search-fuzzy?q=total
    let response = app
        .oneshot(
            Request::builder()
                .uri("/code-entities-search-fuzzy?q=total")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns entities matching "total"
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/code-entities-search-fuzzy");
    assert!(json["data"]["total_count"].as_u64().unwrap() >= 1);
    assert!(json["data"]["entities"].is_array());

    let entities = json["data"]["entities"].as_array().unwrap();

    // Should find calculate_total function
    let found = entities.iter().any(|entity| {
        entity["key"].as_str().unwrap().contains("calculate_total")
    });
    assert!(found, "Should find calculate_total function");
}

/// Test 2.6: Empty Search Returns Bad Request
///
/// # 4-Word Name: test_empty_search_returns_bad_request
#[tokio::test]
async fn test_empty_search_returns_bad_request() {
    // GIVEN: Database with entities
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Insert test entity
    let query = r#"
        ?[ISGL1_key, Current_Code, Future_Code, interface_signature, TDD_Classification,
          lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
          last_modified, entity_type, entity_class] <- [
            ["rust:fn:test_func:src_main_rs:1-10", "pub fn test_func() {}", null, "{}", "{}", null, true, true, null, "src/main.rs", "rust", "2024-01-01T00:00:00Z", "function", "CODE"]
        ]
        :put CodeGraph {
            ISGL1_key =>
            Current_Code, Future_Code, interface_signature, TDD_Classification,
            lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
            last_modified, entity_type, entity_class
        }
    "#;
    storage.execute_query(query).await.unwrap();

    // Create state with database connection
    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: GET /code-entities-search-fuzzy?q= (empty search)
    let response = app
        .oneshot(
            Request::builder()
                .uri("/code-entities-search-fuzzy?q=")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns 400 Bad Request
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], false);
    assert_eq!(json["endpoint"], "/code-entities-search-fuzzy");
    assert!(json["error"].as_str().unwrap().contains("empty"));
}

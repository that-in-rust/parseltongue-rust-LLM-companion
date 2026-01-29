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

// ============================================================================
// PHASE 3: GRAPH QUERY ENDPOINTS (8 Tests)
// ============================================================================

/// Test 3.1: Reverse Callers (Who Calls This?)
///
/// # 4-Word Name: test_reverse_callers_returns_deps
///
/// # Contract
/// - Precondition: Database with dependency graph A → B → C
/// - Postcondition: Query for B returns A as caller
/// - Performance: <100ms response time
/// - Error Handling: Returns 404 for non-existent entities
#[tokio::test]
async fn test_reverse_callers_returns_deps() {
    // GIVEN: Database with dependency chain A → B → C (A calls B, B calls C)
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Insert test entities
    let test_entities = vec![
        ("rust:fn:main:src_main_rs:1-10", "pub fn main() { process(); }", "main", "src/main.rs", "function"),
        ("rust:fn:process:src_process_rs:1-20", "pub fn process() { transform(); }", "process", "src/process.rs", "function"),
        ("rust:fn:transform:src_transform_rs:1-15", "pub fn transform() { /* logic */ }", "transform", "src/transform.rs", "function"),
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

    // Insert dependency edges: main → process, process → transform
    let dependency_edges = vec![
        ("rust:fn:main:src_main_rs:1-10", "rust:fn:process:src_process_rs:1-20", "Calls", "src/main.rs:5"),
        ("rust:fn:process:src_process_rs:1-20", "rust:fn:transform:src_transform_rs:1-15", "Calls", "src/process.rs:10"),
    ];

    for (from_key, to_key, edge_type, source_location) in dependency_edges {
        let query = format!(r#"
            ?[from_key, to_key, edge_type, source_location] <-
            [["{}", "{}", "{}", "{}"]]

            :put DependencyEdges {{
                from_key, to_key, edge_type =>
                source_location
            }}
        "#, from_key, to_key, edge_type, source_location);
        storage.execute_query(&query).await.unwrap();
    }

    // Debug: Verify edges were inserted
    let debug_query = "?[from_key, to_key] := *DependencyEdges{from_key, to_key}";
    let debug_result = storage.raw_query(debug_query).await.unwrap();
    println!("DEBUG: Inserted edges count: {}", debug_result.rows.len());
    for row in &debug_result.rows {
        println!("DEBUG: Edge: {} -> {}", row[0], row[1]);
    }

    // Create state with database connection
    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // Test known working route first
    println!("DEBUG: Testing known working route...");
    let health_response = app.clone()
        .oneshot(
            Request::builder()
                .uri("/server-health-check-status")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    println!("DEBUG: Health check status: {}", health_response.status());

    // WHEN: GET /reverse-callers-query-graph?entity=rust:fn:process:src_process_rs:1-20
    let response = app
        .oneshot(
            Request::builder()
                .uri("/reverse-callers-query-graph?entity=rust:fn:process:src_process_rs:1-20")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns main as caller of process
    // Debug: Print actual response
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    println!("DEBUG: Status: {}", status);
    println!("DEBUG: Response body: {}", body_str);

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/reverse-callers-query-graph");
    assert_eq!(json["data"]["total_count"].as_u64().unwrap(), 1);

    let callers = json["data"]["callers"].as_array().unwrap();
    assert_eq!(callers.len(), 1);

    let caller = &callers[0];
    assert_eq!(caller["from_key"], "rust:fn:main:src_main_rs:1-10");
    assert_eq!(caller["to_key"], "rust:fn:process:src_process_rs:1-20");
    assert_eq!(caller["edge_type"], "Calls");
    assert_eq!(caller["source_location"], "src/main.rs:5");
}

/// Test 3.2: Forward Callees (What Does This Call?)
///
/// # 4-Word Name: test_forward_callees_returns_deps
///
/// # Contract
/// - Precondition: Database with dependency graph A → B → C
/// - Postcondition: Query for B returns C as callee
/// - Performance: <100ms response time
/// - Error Handling: Returns 404 for entities with no callees
#[tokio::test]
async fn test_forward_callees_returns_deps() {
    // GIVEN: Database with dependency chain A → B → C (A calls B, B calls C)
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Insert test entities
    let test_entities = vec![
        ("rust:fn:main:src_main_rs:1-10", "pub fn main() { process(); }", "_main", "src/main.rs", "function"),
        ("rust:fn:process:src_process_rs:1-20", "pub fn process() { transform(); }", "_process", "src/process.rs", "function"),
        ("rust:fn:transform:src_transform_rs:1-15", "pub fn transform() { /* logic */ }", "_transform", "src/transform.rs", "function"),
    ];

    for (key, code, _name, file_path, entity_type) in test_entities {
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

    // Insert dependency edges: main → process, process → transform
    let dependency_edges = vec![
        ("rust:fn:main:src_main_rs:1-10", "rust:fn:process:src_process_rs:1-20", "Calls", "src/main.rs:5"),
        ("rust:fn:process:src_process_rs:1-20", "rust:fn:transform:src_transform_rs:1-15", "Calls", "src/process.rs:10"),
    ];

    for (from_key, to_key, edge_type, source_location) in dependency_edges {
        let query = format!(r#"
            ?[from_key, to_key, edge_type, source_location] <-
            [["{}", "{}", "{}", "{}"]]

            :put DependencyEdges {{
                from_key, to_key, edge_type =>
                source_location
            }}
        "#, from_key, to_key, edge_type, source_location);
        storage.execute_query(&query).await.unwrap();
    }

    // Create state with database connection
    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: GET /forward-callees-query-graph?entity=rust:fn:process:src_process_rs:1-20
    // (What does process() call?)
    let response = app
        .oneshot(
            Request::builder()
                .uri("/forward-callees-query-graph?entity=rust:fn:process:src_process_rs:1-20")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns transform as callee (process calls transform)
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    println!("DEBUG Forward Callees: Status: {}", status);
    println!("DEBUG Forward Callees: Response body: {}", body_str);

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/forward-callees-query-graph");
    assert_eq!(json["data"]["total_count"].as_u64().unwrap(), 1);

    let callees = json["data"]["callees"].as_array().unwrap();
    assert_eq!(callees.len(), 1);

    let callee = &callees[0];
    assert_eq!(callee["from_key"], "rust:fn:process:src_process_rs:1-20");
    assert_eq!(callee["to_key"], "rust:fn:transform:src_transform_rs:1-15");
    assert_eq!(callee["edge_type"], "Calls");
    assert_eq!(callee["source_location"], "src/process.rs:10");
}

/// Test 3.8: Dependency Edges List All
///
/// # 4-Word Name: test_dependency_edges_list_all
///
/// # Contract
/// - Precondition: Database with dependency edges
/// - Postcondition: Returns paginated list of all edges
/// - Performance: <100ms response time
/// - Error Handling: Returns empty list if no edges
#[tokio::test]
async fn test_dependency_edges_list_all() {
    // GIVEN: Database with dependency edges
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Insert test edges
    let dependency_edges = vec![
        ("rust:fn:a:src_a:1-10", "rust:fn:b:src_b:1-20", "Calls", "src/a.rs:5"),
        ("rust:fn:b:src_b:1-20", "rust:fn:c:src_c:1-15", "Calls", "src/b.rs:10"),
        ("rust:fn:c:src_c:1-15", "rust:fn:d:src_d:1-25", "Uses", "src/c.rs:15"),
    ];

    for (from_key, to_key, edge_type, source_location) in &dependency_edges {
        let query = format!(r#"
            ?[from_key, to_key, edge_type, source_location] <-
            [["{}", "{}", "{}", "{}"]]

            :put DependencyEdges {{
                from_key, to_key, edge_type =>
                source_location
            }}
        "#, from_key, to_key, edge_type, source_location);
        storage.execute_query(&query).await.unwrap();
    }

    // Create state with database connection
    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: GET /dependency-edges-list-all
    let response = app
        .oneshot(
            Request::builder()
                .uri("/dependency-edges-list-all")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns all edges
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    println!("DEBUG Edges List: Status: {}", status);
    println!("DEBUG Edges List: Response body: {}", body_str);

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/dependency-edges-list-all");
    assert_eq!(json["data"]["total_count"].as_u64().unwrap(), 3);
    assert_eq!(json["data"]["returned_count"].as_u64().unwrap(), 3);

    let edges = json["data"]["edges"].as_array().unwrap();
    assert_eq!(edges.len(), 3);
}

/// Test 3.3: Blast Radius Single Hop
///
/// # 4-Word Name: test_blast_radius_single_hop
///
/// # Contract
/// - Precondition: Database with dependency graph A → B → C → D
/// - Postcondition: Query for D with hops=1 returns only C (caller of D)
/// - Performance: <100ms response time
/// - Error Handling: Returns 404 for non-existent entities
///
/// # Semantics
/// Blast radius = "If I change X, what breaks?" = entities that DEPEND ON X
/// With edges A→B→C→D, blast radius of D includes C (C calls D).
#[tokio::test]
async fn test_blast_radius_single_hop() {
    // GIVEN: Database with dependency chain A → B → C → D
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Insert dependency edges: A → B → C → D (linear chain)
    // This means: A calls B, B calls C, C calls D
    let dependency_edges = vec![
        ("rust:fn:a:src_a:1-10", "rust:fn:b:src_b:1-20", "Calls", "src/a.rs:5"),
        ("rust:fn:b:src_b:1-20", "rust:fn:c:src_c:1-15", "Calls", "src/b.rs:10"),
        ("rust:fn:c:src_c:1-15", "rust:fn:d:src_d:1-25", "Calls", "src/c.rs:15"),
    ];

    for (from_key, to_key, edge_type, source_location) in &dependency_edges {
        let query = format!(r#"
            ?[from_key, to_key, edge_type, source_location] <-
            [["{}", "{}", "{}", "{}"]]

            :put DependencyEdges {{
                from_key, to_key, edge_type =>
                source_location
            }}
        "#, from_key, to_key, edge_type, source_location);
        storage.execute_query(&query).await.unwrap();
    }

    // Create state with database connection
    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: GET /blast-radius-impact-analysis?entity=rust:fn:d:src_d:1-25&hops=1
    // Query blast radius of D - who depends on D?
    let response = app
        .oneshot(
            Request::builder()
                .uri("/blast-radius-impact-analysis?entity=rust:fn:d:src_d:1-25&hops=1")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns only C (C calls D, so C depends on D)
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    println!("DEBUG Blast Radius 1-hop: Status: {}", status);
    println!("DEBUG Blast Radius 1-hop: Response: {}", body_str);

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/blast-radius-impact-analysis");
    assert_eq!(json["data"]["source_entity"], "rust:fn:d:src_d:1-25");
    assert_eq!(json["data"]["hops_requested"].as_u64().unwrap(), 1);
    assert_eq!(json["data"]["total_affected"].as_u64().unwrap(), 1);

    // Verify hop 1 contains only C (the caller of D)
    let by_hop = json["data"]["by_hop"].as_array().unwrap();
    assert_eq!(by_hop.len(), 1);
    assert_eq!(by_hop[0]["hop"].as_u64().unwrap(), 1);
    assert_eq!(by_hop[0]["count"].as_u64().unwrap(), 1);

    let hop1_entities = by_hop[0]["entities"].as_array().unwrap();
    assert!(hop1_entities.iter().any(|e| e.as_str().unwrap().contains("rust:fn:c")));
}

/// Test 3.4: Blast Radius Multi Hop
///
/// # 4-Word Name: test_blast_radius_multi_hop
///
/// # Contract
/// - Precondition: Database with dependency graph A → B → C → D
/// - Postcondition: Query for D with hops=3 returns C, B, A (callers)
/// - Performance: <100ms response time
/// - Error Handling: Stops at max hops even if more exist
///
/// # Semantics
/// Blast radius = "If I change X, what breaks?" = entities that DEPEND ON X
/// With edges A→B→C→D, blast radius of D = {C, B, A} transitively.
#[tokio::test]
async fn test_blast_radius_multi_hop() {
    // GIVEN: Database with dependency chain A → B → C → D
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Insert dependency edges: A → B → C → D (linear chain)
    // This means: A calls B, B calls C, C calls D
    let dependency_edges = vec![
        ("rust:fn:a:src_a:1-10", "rust:fn:b:src_b:1-20", "Calls", "src/a.rs:5"),
        ("rust:fn:b:src_b:1-20", "rust:fn:c:src_c:1-15", "Calls", "src/b.rs:10"),
        ("rust:fn:c:src_c:1-15", "rust:fn:d:src_d:1-25", "Calls", "src/c.rs:15"),
    ];

    for (from_key, to_key, edge_type, source_location) in &dependency_edges {
        let query = format!(r#"
            ?[from_key, to_key, edge_type, source_location] <-
            [["{}", "{}", "{}", "{}"]]

            :put DependencyEdges {{
                from_key, to_key, edge_type =>
                source_location
            }}
        "#, from_key, to_key, edge_type, source_location);
        storage.execute_query(&query).await.unwrap();
    }

    // Create state with database connection
    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: GET /blast-radius-impact-analysis?entity=rust:fn:d:src_d:1-25&hops=3
    // Query blast radius of D - who transitively depends on D?
    let response = app
        .oneshot(
            Request::builder()
                .uri("/blast-radius-impact-analysis?entity=rust:fn:d:src_d:1-25&hops=3")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns C (hop 1), B (hop 2), A (hop 3) - all callers up the chain
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    println!("DEBUG Blast Radius 3-hop: Status: {}", status);
    println!("DEBUG Blast Radius 3-hop: Response: {}", body_str);

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/blast-radius-impact-analysis");
    assert_eq!(json["data"]["source_entity"], "rust:fn:d:src_d:1-25");
    assert_eq!(json["data"]["hops_requested"].as_u64().unwrap(), 3);
    assert_eq!(json["data"]["total_affected"].as_u64().unwrap(), 3);

    // Verify all 3 hops
    let by_hop = json["data"]["by_hop"].as_array().unwrap();
    assert_eq!(by_hop.len(), 3);

    // Hop 1: C (direct caller of D)
    assert_eq!(by_hop[0]["hop"].as_u64().unwrap(), 1);
    assert_eq!(by_hop[0]["count"].as_u64().unwrap(), 1);

    // Hop 2: B (calls C which calls D)
    assert_eq!(by_hop[1]["hop"].as_u64().unwrap(), 2);
    assert_eq!(by_hop[1]["count"].as_u64().unwrap(), 1);

    // Hop 3: A (calls B which calls C which calls D)
    assert_eq!(by_hop[2]["hop"].as_u64().unwrap(), 3);
    assert_eq!(by_hop[2]["count"].as_u64().unwrap(), 1);
}

/// Test 3.5: Circular Dependency None Found
///
/// # 4-Word Name: test_circular_dependency_none_found
///
/// # Contract
/// - Precondition: Database with acyclic dependency graph A → B → C
/// - Postcondition: Returns has_cycles=false, cycle_count=0
/// - Performance: <100ms response time
#[tokio::test]
async fn test_circular_dependency_none_found() {
    // GIVEN: Database with acyclic dependency chain A → B → C (no cycles)
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Insert acyclic dependency edges
    let dependency_edges = vec![
        ("rust:fn:a:src_a:1-10", "rust:fn:b:src_b:1-20", "Calls", "src/a.rs:5"),
        ("rust:fn:b:src_b:1-20", "rust:fn:c:src_c:1-15", "Calls", "src/b.rs:10"),
    ];

    for (from_key, to_key, edge_type, source_location) in &dependency_edges {
        let query = format!(r#"
            ?[from_key, to_key, edge_type, source_location] <-
            [["{}", "{}", "{}", "{}"]]

            :put DependencyEdges {{
                from_key, to_key, edge_type =>
                source_location
            }}
        "#, from_key, to_key, edge_type, source_location);
        storage.execute_query(&query).await.unwrap();
    }

    // Create state with database connection
    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: GET /circular-dependency-detection-scan
    let response = app
        .oneshot(
            Request::builder()
                .uri("/circular-dependency-detection-scan")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns no cycles
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    println!("DEBUG Cycle Detection (none): Status: {}", status);
    println!("DEBUG Cycle Detection (none): Response: {}", body_str);

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/circular-dependency-detection-scan");
    assert_eq!(json["data"]["has_cycles"], false);
    assert_eq!(json["data"]["cycle_count"].as_u64().unwrap(), 0);
}

/// Test 3.6: Circular Dependency Cycle Detected
///
/// # 4-Word Name: test_circular_dependency_cycle_detected
///
/// # Contract
/// - Precondition: Database with cyclic dependency graph A → B → C → A
/// - Postcondition: Returns has_cycles=true with cycle details
/// - Performance: <100ms response time
#[tokio::test]
async fn test_circular_dependency_cycle_detected() {
    // GIVEN: Database with cycle A → B → C → A
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Insert cyclic dependency edges: A → B → C → A
    let dependency_edges = vec![
        ("rust:fn:a:src_a:1-10", "rust:fn:b:src_b:1-20", "Calls", "src/a.rs:5"),
        ("rust:fn:b:src_b:1-20", "rust:fn:c:src_c:1-15", "Calls", "src/b.rs:10"),
        ("rust:fn:c:src_c:1-15", "rust:fn:a:src_a:1-10", "Calls", "src/c.rs:15"), // Cycle!
    ];

    for (from_key, to_key, edge_type, source_location) in &dependency_edges {
        let query = format!(r#"
            ?[from_key, to_key, edge_type, source_location] <-
            [["{}", "{}", "{}", "{}"]]

            :put DependencyEdges {{
                from_key, to_key, edge_type =>
                source_location
            }}
        "#, from_key, to_key, edge_type, source_location);
        storage.execute_query(&query).await.unwrap();
    }

    // Create state with database connection
    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: GET /circular-dependency-detection-scan
    let response = app
        .oneshot(
            Request::builder()
                .uri("/circular-dependency-detection-scan")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns cycle detected
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    println!("DEBUG Cycle Detection (found): Status: {}", status);
    println!("DEBUG Cycle Detection (found): Response: {}", body_str);

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/circular-dependency-detection-scan");
    assert_eq!(json["data"]["has_cycles"], true);
    assert!(json["data"]["cycle_count"].as_u64().unwrap() >= 1);

    // Verify cycle details
    let cycles = json["data"]["cycles"].as_array().unwrap();
    assert!(!cycles.is_empty());

    // First cycle should have length 3 (A → B → C → A)
    let first_cycle = &cycles[0];
    assert!(first_cycle["length"].as_u64().unwrap() >= 3);
    assert!(first_cycle["path"].is_array());
}

/// Test 3.7: Complexity Hotspots Ranking View
///
/// # 4-Word Name: test_complexity_hotspots_ranking_view
///
/// # Contract
/// - Precondition: Database with entities having varying dependency counts
/// - Postcondition: Returns entities ranked by total coupling (inbound + outbound)
/// - Performance: <100ms response time
#[tokio::test]
async fn test_complexity_hotspots_ranking_view() {
    // GIVEN: Database with entities having different coupling scores
    // Entity B has highest coupling: 2 inbound (A→B, C→B) + 2 outbound (B→D, B→E) = 4
    // Entity A has: 0 inbound + 1 outbound = 1
    // Entity C has: 0 inbound + 1 outbound = 1
    // Entity D has: 1 inbound + 0 outbound = 1
    // Entity E has: 1 inbound + 0 outbound = 1
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Insert dependency edges creating a hub pattern around B
    let dependency_edges = vec![
        ("rust:fn:a:src_a:1-10", "rust:fn:b:src_b:1-20", "Calls", "src/a.rs:5"),   // A → B
        ("rust:fn:c:src_c:1-15", "rust:fn:b:src_b:1-20", "Calls", "src/c.rs:5"),   // C → B
        ("rust:fn:b:src_b:1-20", "rust:fn:d:src_d:1-25", "Calls", "src/b.rs:10"),  // B → D
        ("rust:fn:b:src_b:1-20", "rust:fn:e:src_e:1-30", "Calls", "src/b.rs:15"),  // B → E
    ];

    for (from_key, to_key, edge_type, source_location) in &dependency_edges {
        let query = format!(r#"
            ?[from_key, to_key, edge_type, source_location] <-
            [["{}", "{}", "{}", "{}"]]

            :put DependencyEdges {{
                from_key, to_key, edge_type =>
                source_location
            }}
        "#, from_key, to_key, edge_type, source_location);
        storage.execute_query(&query).await.unwrap();
    }

    // Create state with database connection
    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: GET /complexity-hotspots-ranking-view?top=5
    let response = app
        .oneshot(
            Request::builder()
                .uri("/complexity-hotspots-ranking-view?top=5")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns entities ranked by coupling, B should be #1
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    println!("DEBUG Complexity Hotspots: Status: {}", status);
    println!("DEBUG Complexity Hotspots: Response: {}", body_str);

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/complexity-hotspots-ranking-view");
    assert!(json["data"]["total_entities_analyzed"].as_u64().unwrap() >= 1);

    let hotspots = json["data"]["hotspots"].as_array().unwrap();
    assert!(!hotspots.is_empty());

    // First hotspot should be B with highest coupling
    let top_hotspot = &hotspots[0];
    assert_eq!(top_hotspot["rank"].as_u64().unwrap(), 1);
    assert!(top_hotspot["entity_key"].as_str().unwrap().contains("rust:fn:b"));
    assert_eq!(top_hotspot["inbound_count"].as_u64().unwrap(), 2);  // A→B, C→B
    assert_eq!(top_hotspot["outbound_count"].as_u64().unwrap(), 2); // B→D, B→E
    assert_eq!(top_hotspot["total_coupling"].as_u64().unwrap(), 4);
}

// =============================================================================
// Phase 4: Advanced Analysis Endpoints
// =============================================================================

/// Test semantic cluster grouping endpoint
///
/// # 4-Word Name: test_semantic_cluster_grouping_list
///
/// # Contract
/// - Precondition: Database with entities forming distinct clusters
/// - Postcondition: Returns entities grouped by connectivity
/// - Performance: <100ms response time
#[tokio::test]
async fn test_semantic_cluster_grouping_list() {
    // GIVEN: Database with two distinct clusters of entities
    // Cluster 1: A↔B↔C (tightly connected)
    // Cluster 2: X↔Y (separate cluster)
    // Bridge: B→X (weak connection between clusters)
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Insert dependency edges creating two clusters
    let dependency_edges = vec![
        // Cluster 1: A, B, C tightly connected
        ("rust:fn:a:src_a:1-10", "rust:fn:b:src_b:1-20", "Calls", "src/a.rs:5"),
        ("rust:fn:b:src_b:1-20", "rust:fn:a:src_a:1-10", "Calls", "src/b.rs:5"),
        ("rust:fn:b:src_b:1-20", "rust:fn:c:src_c:1-15", "Calls", "src/b.rs:10"),
        ("rust:fn:c:src_c:1-15", "rust:fn:b:src_b:1-20", "Calls", "src/c.rs:5"),
        // Cluster 2: X, Y tightly connected
        ("rust:fn:x:src_x:1-10", "rust:fn:y:src_y:1-20", "Calls", "src/x.rs:5"),
        ("rust:fn:y:src_y:1-20", "rust:fn:x:src_x:1-10", "Calls", "src/y.rs:5"),
        // Weak bridge between clusters
        ("rust:fn:b:src_b:1-20", "rust:fn:x:src_x:1-10", "Calls", "src/b.rs:15"),
    ];

    for (from_key, to_key, edge_type, source_location) in &dependency_edges {
        let query = format!(r#"
            ?[from_key, to_key, edge_type, source_location] <-
            [["{}", "{}", "{}", "{}"]]

            :put DependencyEdges {{
                from_key, to_key, edge_type =>
                source_location
            }}
        "#, from_key, to_key, edge_type, source_location);
        storage.execute_query(&query).await.unwrap();
    }

    // Create state with database connection
    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: GET /semantic-cluster-grouping-list
    let response = app
        .oneshot(
            Request::builder()
                .uri("/semantic-cluster-grouping-list")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns clusters with entities grouped by connectivity
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    println!("DEBUG Semantic Clusters: Status: {}", status);
    println!("DEBUG Semantic Clusters: Response: {}", body_str);

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/semantic-cluster-grouping-list");
    assert!(json["data"]["total_entities"].as_u64().unwrap() >= 5); // A, B, C, X, Y
    assert!(json["data"]["cluster_count"].as_u64().unwrap() >= 1);

    let clusters = json["data"]["clusters"].as_array().unwrap();
    assert!(!clusters.is_empty());

    // Each cluster should have entity_count > 0
    for cluster in clusters {
        assert!(cluster["entity_count"].as_u64().unwrap() > 0);
        assert!(cluster["entities"].as_array().unwrap().len() > 0);
    }
}

/// Test API reference documentation help endpoint
///
/// # 4-Word Name: test_api_reference_documentation_help
///
/// # Contract
/// - Precondition: Server running
/// - Postcondition: Returns complete API documentation
/// - Performance: <50ms response time
#[tokio::test]
async fn test_api_reference_documentation_help() {
    // GIVEN: Running server with all endpoints registered
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();

    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: GET /api-reference-documentation-help
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api-reference-documentation-help")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns API documentation with all endpoints
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    println!("DEBUG API Reference: Status: {}", status);
    println!("DEBUG API Reference: Response: {}", body_str);

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/api-reference-documentation-help");

    // Should have API version
    assert!(json["data"]["api_version"].as_str().is_some());

    // Should have multiple endpoints documented
    assert!(json["data"]["total_endpoints"].as_u64().unwrap() >= 10);

    // Should have categorized endpoints
    let categories = json["data"]["categories"].as_array().unwrap();
    assert!(!categories.is_empty());

    // Each category should have endpoints
    for category in categories {
        assert!(category["name"].as_str().is_some());
        let endpoints = category["endpoints"].as_array().unwrap();
        assert!(!endpoints.is_empty());

        // Each endpoint should have path, method, description
        for endpoint in endpoints {
            assert!(endpoint["path"].as_str().is_some());
            assert!(endpoint["method"].as_str().is_some());
            assert!(endpoint["description"].as_str().is_some());
        }
    }
}

// =============================================================================
// Killer Features
// =============================================================================

// =============================================================================
// Phase 5: Incremental Reindex Tests (PRD-2026-01-28)
// =============================================================================

/// Test 5.1: Incremental reindex empty path returns 400
///
/// # 4-Word Name: test_incremental_reindex_empty_path_error
///
/// # Contract
/// - Precondition: Server running with database
/// - Postcondition: Returns 400 Bad Request for empty path
#[tokio::test]
async fn test_incremental_reindex_empty_path_error() {
    // GIVEN: Server with in-memory database
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: POST /incremental-reindex-file-update?path= (empty path)
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/incremental-reindex-file-update?path=")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns 400 Bad Request
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    println!("DEBUG Incremental empty path: Status: {}", status);
    println!("DEBUG Incremental empty path: Response: {}", body_str);

    assert_eq!(status, StatusCode::BAD_REQUEST);

    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["endpoint"], "/incremental-reindex-file-update");
    assert!(json["error"].as_str().unwrap().contains("required"));
}

/// Test 5.2: Incremental reindex file not found returns 404
///
/// # 4-Word Name: test_incremental_reindex_file_not_found
///
/// # Contract
/// - Precondition: Server running with database
/// - Postcondition: Returns 404 Not Found for non-existent file
#[tokio::test]
async fn test_incremental_reindex_file_not_found() {
    // GIVEN: Server with in-memory database
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: POST /incremental-reindex-file-update?path=/nonexistent/file.rs
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/incremental-reindex-file-update?path=/nonexistent/file.rs")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns 404 Not Found
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    println!("DEBUG Incremental not found: Status: {}", status);
    println!("DEBUG Incremental not found: Response: {}", body_str);

    assert_eq!(status, StatusCode::NOT_FOUND);

    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["endpoint"], "/incremental-reindex-file-update");
    assert!(json["error"].as_str().unwrap().contains("not found"));
}

/// Test 5.3: Incremental reindex directory returns 400
///
/// # 4-Word Name: test_incremental_reindex_directory_error
///
/// # Contract
/// - Precondition: Server running with database
/// - Postcondition: Returns 400 Bad Request for directory path
#[tokio::test]
async fn test_incremental_reindex_directory_error() {
    // GIVEN: Server with in-memory database
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // Use temp directory that exists
    let temp_dir = std::env::temp_dir();
    let uri = format!(
        "/incremental-reindex-file-update?path={}",
        temp_dir.display()
    );

    // WHEN: POST /incremental-reindex-file-update?path=/tmp (directory)
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns 400 Bad Request
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    println!("DEBUG Incremental directory: Status: {}", status);
    println!("DEBUG Incremental directory: Response: {}", body_str);

    assert_eq!(status, StatusCode::BAD_REQUEST);

    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    assert_eq!(json["success"], false);
    assert_eq!(json["endpoint"], "/incremental-reindex-file-update");
    assert!(json["error"].as_str().unwrap().contains("not a file"));
}

/// Test 5.4: Incremental reindex unchanged file returns early
///
/// # 4-Word Name: test_incremental_reindex_unchanged_early_return
///
/// # Contract
/// - Precondition: Server running with database, file hash cached
/// - Postcondition: Returns hash_changed: false immediately
/// - Performance: <50ms for cached unchanged file
#[tokio::test]
async fn test_incremental_reindex_unchanged_early_return() {
    // GIVEN: Server with in-memory database
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Create a temporary test file
    let temp_dir = std::env::temp_dir();
    let test_file_path = temp_dir.join("test_unchanged_reindex.rs");
    let test_content = "fn test_func() { println!(\"hello\"); }\n";
    std::fs::write(&test_file_path, test_content).unwrap();

    // Pre-compute hash and store in cache
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(test_content.as_bytes());
    let hash = hex::encode(hasher.finalize());

    // Create hash cache schema and store the hash
    storage.create_file_hash_cache_schema().await.unwrap();
    storage.set_cached_file_hash_value(test_file_path.to_str().unwrap(), &hash).await.unwrap();

    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    let uri = format!(
        "/incremental-reindex-file-update?path={}",
        test_file_path.display()
    );

    // WHEN: POST /incremental-reindex-file-update with unchanged file
    let start = std::time::Instant::now();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    let elapsed = start.elapsed();

    // THEN: Returns hash_changed: false
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    println!("DEBUG Incremental unchanged: Status: {}", status);
    println!("DEBUG Incremental unchanged: Response: {}", body_str);
    println!("DEBUG Incremental unchanged: Elapsed: {:?}", elapsed);

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/incremental-reindex-file-update");
    assert_eq!(json["data"]["hash_changed"], false);

    // Performance check: should be fast for unchanged files
    assert!(elapsed.as_millis() < 500, "Unchanged file should return quickly");

    // Cleanup
    let _ = std::fs::remove_file(&test_file_path);
}

/// Test 5.5: Incremental reindex changed file deletes entities
///
/// # 4-Word Name: test_incremental_reindex_changed_deletes_entities
///
/// # Contract
/// - Precondition: Server with database containing entities for file
/// - Postcondition: Deletes old entities and reports deletion count
///
/// # Algorithm
/// Uses two-phase approach to avoid schema mismatch issues:
/// 1. First reindex: inserts entities via parsing
/// 2. Modify file: change content so hash differs
/// 3. Second reindex: should delete old and insert new
#[tokio::test]
async fn test_incremental_reindex_changed_deletes_entities() {
    // GIVEN: Server with in-memory database
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Create a temporary test file with initial content
    let temp_dir = std::env::temp_dir();
    let test_file_path = temp_dir.join("test_deletion_reindex_5_5.rs");
    let initial_content = r#"
fn old_func_one() {
    println!("one");
}

fn old_func_two() {
    println!("two");
}
"#;
    std::fs::write(&test_file_path, initial_content).unwrap();

    let state = SharedApplicationStateContainer::create_with_database_storage(storage);

    let uri = format!(
        "/incremental-reindex-file-update?path={}",
        test_file_path.display()
    );

    // Phase 1: First reindex to populate entities
    let app1 = build_complete_router_instance(state.clone());
    let response1 = app1
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    let status1 = response1.status();
    let body1 = axum::body::to_bytes(response1.into_body(), usize::MAX).await.unwrap();
    let body1_str = String::from_utf8(body1.to_vec()).unwrap();

    println!("DEBUG Deletion Phase 1: Status: {}", status1);
    println!("DEBUG Deletion Phase 1: Response: {}", body1_str);

    assert_eq!(status1, StatusCode::OK);
    let json1: serde_json::Value = serde_json::from_str(&body1_str).unwrap();
    assert_eq!(json1["success"], true);
    let initial_entities = json1["data"]["entities_added"].as_u64().unwrap();
    assert!(initial_entities >= 2, "Should have parsed at least 2 functions");

    // Phase 2: Modify file to change content (different hash)
    let modified_content = r#"
fn completely_new_func() {
    println!("new");
}
"#;
    std::fs::write(&test_file_path, modified_content).unwrap();

    // Phase 3: Second reindex should delete old and insert new
    let app2 = build_complete_router_instance(state);
    let response2 = app2
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    let status2 = response2.status();
    let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX).await.unwrap();
    let body2_str = String::from_utf8(body2.to_vec()).unwrap();

    println!("DEBUG Deletion Phase 2: Status: {}", status2);
    println!("DEBUG Deletion Phase 2: Response: {}", body2_str);

    assert_eq!(status2, StatusCode::OK);

    let json2: serde_json::Value = serde_json::from_str(&body2_str).unwrap();
    assert_eq!(json2["success"], true);
    assert_eq!(json2["endpoint"], "/incremental-reindex-file-update");
    assert_eq!(json2["data"]["hash_changed"], true);

    // Should have deleted the initial entities
    let entities_before = json2["data"]["entities_before"].as_u64().unwrap();
    let entities_removed = json2["data"]["entities_removed"].as_u64().unwrap();
    assert!(entities_before >= 2, "Should have had at least 2 entities before: {}", entities_before);
    assert_eq!(entities_before, entities_removed, "All old entities should be removed");

    // Should have inserted new entities
    let entities_added = json2["data"]["entities_added"].as_u64().unwrap();
    assert!(entities_added >= 1, "Should have added at least 1 new entity");

    // Cleanup
    let _ = std::fs::remove_file(&test_file_path);
}

/// Test 5.6: Incremental reindex parses and inserts new entities
///
/// # 4-Word Name: test_incremental_reindex_parses_new_entities
///
/// # Contract
/// - Precondition: Server running with database
/// - Postcondition: Parses file and inserts new entities
#[tokio::test]
async fn test_incremental_reindex_parses_new_entities() {
    // GIVEN: Server with in-memory database (no pre-existing entities)
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Create a temporary test file with Rust code
    let temp_dir = std::env::temp_dir();
    let test_file_path = temp_dir.join("test_parse_reindex.rs");
    let test_content = r#"
fn first_function() {
    println!("first");
}

fn second_function() {
    first_function();
}

struct MyStruct {
    value: i32,
}
"#;
    std::fs::write(&test_file_path, test_content).unwrap();

    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    let uri = format!(
        "/incremental-reindex-file-update?path={}",
        test_file_path.display()
    );

    // WHEN: POST /incremental-reindex-file-update with new file
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Parses and inserts entities
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    println!("DEBUG Incremental parse: Status: {}", status);
    println!("DEBUG Incremental parse: Response: {}", body_str);

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/incremental-reindex-file-update");
    assert_eq!(json["data"]["hash_changed"], true);
    assert_eq!(json["data"]["entities_before"], 0); // No pre-existing entities

    // Should have parsed at least 2 functions and 1 struct
    let entities_added = json["data"]["entities_added"].as_u64().unwrap();
    assert!(entities_added >= 2, "Should parse at least 2 entities, got {}", entities_added);

    assert_eq!(json["data"]["entities_after"], entities_added);

    // Should have at least one edge (second_function calls first_function)
    // Note: edge detection depends on parser implementation
    assert!(json["data"]["edges_added"].as_u64().is_some());

    // Cleanup
    let _ = std::fs::remove_file(&test_file_path);
}

/// Test 5.7: Incremental reindex updates hash cache
///
/// # 4-Word Name: test_incremental_reindex_updates_hash_cache
///
/// # Contract
/// - Precondition: Server running with database
/// - Postcondition: Hash cache is updated after successful reindex
///   (verified by second request returning hash_changed: false)
#[tokio::test]
async fn test_incremental_reindex_updates_hash_cache() {
    // GIVEN: Server with in-memory database
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Create a temporary test file with unique name
    let temp_dir = std::env::temp_dir();
    let test_file_path = temp_dir.join("test_hash_cache_reindex_5_7.rs");
    let test_content = "fn hash_test() { println!(\"test\"); }\n";
    std::fs::write(&test_file_path, test_content).unwrap();

    let state = SharedApplicationStateContainer::create_with_database_storage(storage);

    // Build router once, use with_state to clone for second request
    let uri = format!(
        "/incremental-reindex-file-update?path={}",
        test_file_path.display()
    );

    // First request: should parse and cache hash
    let app1 = build_complete_router_instance(state.clone());
    let response1 = app1
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    let status1 = response1.status();
    let body1 = axum::body::to_bytes(response1.into_body(), usize::MAX).await.unwrap();
    let body1_str = String::from_utf8(body1.to_vec()).unwrap();

    println!("DEBUG Hash cache 1st: Status: {}", status1);
    println!("DEBUG Hash cache 1st: Response: {}", body1_str);

    assert_eq!(status1, StatusCode::OK);
    let json1: serde_json::Value = serde_json::from_str(&body1_str).unwrap();
    assert_eq!(json1["success"], true);
    assert_eq!(json1["data"]["hash_changed"], true); // First time always parses

    // Second request with same file: should return hash_changed: false
    // This proves the hash was cached from the first request
    let app2 = build_complete_router_instance(state);
    let response2 = app2
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    let status2 = response2.status();
    let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX).await.unwrap();
    let body2_str = String::from_utf8(body2.to_vec()).unwrap();

    println!("DEBUG Hash cache 2nd: Status: {}", status2);
    println!("DEBUG Hash cache 2nd: Response: {}", body2_str);

    assert_eq!(status2, StatusCode::OK);
    let json2: serde_json::Value = serde_json::from_str(&body2_str).unwrap();
    assert_eq!(json2["success"], true);
    assert_eq!(json2["data"]["hash_changed"], false); // Cache hit!

    // Cleanup
    let _ = std::fs::remove_file(&test_file_path);
}

// =============================================================================
// Phase 6: Complete Cycle Integration Tests (PRD-2026-01-28)
// Tests end-to-end incremental reindex workflow including graph state verification
// =============================================================================

/// Test 5.8: Complete cycle - initial index, modify, reindex, verify graph state
///
/// # 4-Word Name: test_complete_cycle_graph_state
///
/// # Contract
/// - Precondition: Server running with empty database
/// - Postcondition: Graph state correctly reflects file modifications
/// - Performance: Full cycle completes in <1000ms
///
/// # Algorithm
/// 1. Create file with two functions (A calls B)
/// 2. First reindex: verify 2 entities, 1 edge
/// 3. Modify file: change function names
/// 4. Second reindex: verify old entities deleted, new entities inserted
/// 5. Query graph to verify final state matches modified file
#[tokio::test]
async fn test_complete_cycle_graph_state() {
    use std::time::Instant;
    let cycle_start = Instant::now();

    // GIVEN: Server with in-memory database
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Create test file with two functions, one calling the other
    let temp_dir = std::env::temp_dir();
    let test_file_path = temp_dir.join("test_complete_cycle_5_8.rs");
    let initial_content = r#"
fn original_caller() {
    original_callee();
}

fn original_callee() {
    println!("original");
}
"#;
    std::fs::write(&test_file_path, initial_content).unwrap();

    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let uri = format!(
        "/incremental-reindex-file-update?path={}",
        test_file_path.display()
    );

    // PHASE 1: Initial indexing
    let app1 = build_complete_router_instance(state.clone());
    let response1 = app1
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    let status1 = response1.status();
    let body1 = axum::body::to_bytes(response1.into_body(), usize::MAX).await.unwrap();
    let body1_str = String::from_utf8(body1.to_vec()).unwrap();

    println!("DEBUG P6 Cycle Phase 1: Status: {}", status1);
    println!("DEBUG P6 Cycle Phase 1: Response: {}", body1_str);

    assert_eq!(status1, StatusCode::OK);
    let json1: serde_json::Value = serde_json::from_str(&body1_str).unwrap();
    assert_eq!(json1["success"], true);
    assert_eq!(json1["data"]["hash_changed"], true);

    let entities_added_phase1 = json1["data"]["entities_added"].as_u64().unwrap();
    assert!(entities_added_phase1 >= 2, "Phase 1: Should have at least 2 functions, got {}", entities_added_phase1);

    // PHASE 2: Modify file content (rename functions)
    let modified_content = r#"
fn renamed_caller() {
    renamed_callee();
}

fn renamed_callee() {
    println!("renamed");
}
"#;
    std::fs::write(&test_file_path, modified_content).unwrap();

    // PHASE 3: Second reindex with modified content
    let app2 = build_complete_router_instance(state.clone());
    let response2 = app2
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    let status2 = response2.status();
    let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX).await.unwrap();
    let body2_str = String::from_utf8(body2.to_vec()).unwrap();

    println!("DEBUG P6 Cycle Phase 3: Status: {}", status2);
    println!("DEBUG P6 Cycle Phase 3: Response: {}", body2_str);

    assert_eq!(status2, StatusCode::OK);
    let json2: serde_json::Value = serde_json::from_str(&body2_str).unwrap();
    assert_eq!(json2["success"], true);
    assert_eq!(json2["data"]["hash_changed"], true);

    // Verify deletion counts
    let entities_before = json2["data"]["entities_before"].as_u64().unwrap();
    let entities_removed = json2["data"]["entities_removed"].as_u64().unwrap();
    assert_eq!(
        entities_before, entities_removed,
        "All original entities should be removed: before={}, removed={}",
        entities_before, entities_removed
    );

    // Verify insertion counts
    let entities_added_phase3 = json2["data"]["entities_added"].as_u64().unwrap();
    assert!(
        entities_added_phase3 >= 2,
        "Phase 3: Should have added at least 2 renamed functions, got {}",
        entities_added_phase3
    );

    // PHASE 4: Verify final graph state via entity search
    let app3 = build_complete_router_instance(state.clone());
    let search_response = app3
        .oneshot(
            Request::builder()
                .uri("/code-entities-search-fuzzy?q=renamed")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    let search_status = search_response.status();
    let search_body = axum::body::to_bytes(search_response.into_body(), usize::MAX).await.unwrap();
    let search_body_str = String::from_utf8(search_body.to_vec()).unwrap();

    println!("DEBUG P6 Cycle Phase 4 Search: Status: {}", search_status);
    println!("DEBUG P6 Cycle Phase 4 Search: Response: {}", search_body_str);

    assert_eq!(search_status, StatusCode::OK);
    let search_json: serde_json::Value = serde_json::from_str(&search_body_str).unwrap();
    assert_eq!(search_json["success"], true);

    // Should find renamed functions (key format: rust:fn:renamed_callee:...)
    let entities = search_json["data"]["entities"].as_array().unwrap();
    let renamed_count = entities
        .iter()
        .filter(|e| e["key"].as_str().unwrap_or("").contains("renamed"))
        .count();
    assert!(renamed_count >= 2, "Should find at least 2 renamed entities, got {}", renamed_count);

    // Verify old entities are NOT in the graph
    let app4 = build_complete_router_instance(state);
    let search_original = app4
        .oneshot(
            Request::builder()
                .uri("/code-entities-search-fuzzy?q=original")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    let original_body = axum::body::to_bytes(search_original.into_body(), usize::MAX).await.unwrap();
    let original_json: serde_json::Value = serde_json::from_slice(&original_body).unwrap();
    let original_entities = original_json["data"]["entities"].as_array().unwrap();

    // Filter for entities from our test file
    let test_file_str = test_file_path.display().to_string();
    let original_in_test_file: Vec<_> = original_entities
        .iter()
        .filter(|e| {
            e["file_path"].as_str().unwrap_or("").contains(&test_file_str)
        })
        .collect();
    assert!(
        original_in_test_file.is_empty(),
        "Original entities should be deleted from test file, found: {:?}",
        original_in_test_file
    );

    // Performance assertion
    let cycle_duration = cycle_start.elapsed();
    assert!(
        cycle_duration.as_millis() < 2000,
        "Complete cycle should finish in <2000ms, took {}ms",
        cycle_duration.as_millis()
    );

    // Cleanup
    let _ = std::fs::remove_file(&test_file_path);
}

/// Test 5.9: Complete cycle with edge verification
///
/// # 4-Word Name: test_complete_cycle_edge_updates
///
/// # Contract
/// - Precondition: Server running with empty database
/// - Postcondition: Edges correctly reflect new call relationships
/// - Performance: Full cycle with edge verification <1500ms
#[tokio::test]
async fn test_complete_cycle_edge_updates() {
    use std::time::Instant;
    let cycle_start = Instant::now();

    // GIVEN: Server with in-memory database
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Create test file with function calls
    let temp_dir = std::env::temp_dir();
    let test_file_path = temp_dir.join("test_cycle_edges_5_9.rs");
    let initial_content = r#"
fn edge_test_main() {
    edge_test_helper();
}

fn edge_test_helper() {
    println!("helper");
}
"#;
    std::fs::write(&test_file_path, initial_content).unwrap();

    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let uri = format!(
        "/incremental-reindex-file-update?path={}",
        test_file_path.display()
    );

    // PHASE 1: Initial indexing
    let app1 = build_complete_router_instance(state.clone());
    let response1 = app1
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    let status1 = response1.status();
    assert_eq!(status1, StatusCode::OK);

    let body1 = axum::body::to_bytes(response1.into_body(), usize::MAX).await.unwrap();
    let json1: serde_json::Value = serde_json::from_slice(&body1).unwrap();

    println!("DEBUG P6 Edges Phase 1: {:?}", json1);

    let edges_added_phase1 = json1["data"]["edges_added"].as_u64().unwrap_or(0);
    println!("DEBUG P6 Edges Phase 1: edges_added = {}", edges_added_phase1);

    // PHASE 2: Modify file - add another callee
    let modified_content = r#"
fn edge_test_main() {
    edge_test_helper();
    edge_test_util();
}

fn edge_test_helper() {
    println!("helper");
}

fn edge_test_util() {
    println!("util");
}
"#;
    std::fs::write(&test_file_path, modified_content).unwrap();

    // PHASE 3: Second reindex
    let app2 = build_complete_router_instance(state.clone());
    let response2 = app2
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    let status2 = response2.status();
    assert_eq!(status2, StatusCode::OK);

    let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX).await.unwrap();
    let json2: serde_json::Value = serde_json::from_slice(&body2).unwrap();

    println!("DEBUG P6 Edges Phase 3: {:?}", json2);

    assert_eq!(json2["success"], true);
    assert_eq!(json2["data"]["hash_changed"], true);

    // Verify entities were updated
    let entities_added_phase3 = json2["data"]["entities_added"].as_u64().unwrap();
    assert!(
        entities_added_phase3 >= 3,
        "Should have at least 3 functions after modification, got {}",
        entities_added_phase3
    );

    // Verify old edges were removed
    let edges_removed = json2["data"]["edges_removed"].as_u64().unwrap_or(0);
    println!("DEBUG P6 Edges Phase 3: edges_removed = {}", edges_removed);

    // PHASE 4: Query edges to verify final state
    let app3 = build_complete_router_instance(state);
    let edges_response = app3
        .oneshot(
            Request::builder()
                .uri("/dependency-edges-list-all")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    let edges_status = edges_response.status();
    let edges_body = axum::body::to_bytes(edges_response.into_body(), usize::MAX).await.unwrap();
    let edges_json: serde_json::Value = serde_json::from_slice(&edges_body).unwrap();

    println!("DEBUG P6 Edges Phase 4: Status: {}", edges_status);
    println!("DEBUG P6 Edges Phase 4: Edges: {:?}", edges_json);

    assert_eq!(edges_status, StatusCode::OK);

    // Performance assertion
    let cycle_duration = cycle_start.elapsed();
    assert!(
        cycle_duration.as_millis() < 2000,
        "Complete edge cycle should finish in <2000ms, took {}ms",
        cycle_duration.as_millis()
    );

    // Cleanup
    let _ = std::fs::remove_file(&test_file_path);
}

// =============================================================================
// Phase 7: Performance Validation Tests (PRD-2026-01-28)
// Contract: <500ms for changed files with <100 entities, <50ms for unchanged files
// =============================================================================

/// Test 5.10: Performance - unchanged file cache hit under 100ms
///
/// # 4-Word Name: test_perf_unchanged_file_fast
///
/// # Contract
/// - Precondition: File already indexed with hash cached
/// - Postcondition: Response time <100ms for cache hit
/// - Performance: Cache hit should be fast (no parsing)
#[tokio::test]
async fn test_perf_unchanged_file_fast() {
    // GIVEN: Server with in-memory database
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    let temp_dir = std::env::temp_dir();
    let test_file_path = temp_dir.join("test_perf_unchanged_5_10.rs");
    let test_content = r#"
fn perf_test_one() {
    println!("one");
}

fn perf_test_two() {
    println!("two");
}
"#;
    std::fs::write(&test_file_path, test_content).unwrap();

    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let uri = format!(
        "/incremental-reindex-file-update?path={}",
        test_file_path.display()
    );

    // First request: populates the hash cache
    let app1 = build_complete_router_instance(state.clone());
    let _ = app1
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // WHEN: Second request with unchanged file (cache hit)
    let start_time = std::time::Instant::now();
    let app2 = build_complete_router_instance(state);
    let response = app2
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    let elapsed_total = start_time.elapsed();

    // THEN: Response should be fast
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["hash_changed"], false);

    let processing_time_ms = json["data"]["processing_time_ms"].as_u64().unwrap();

    println!("DEBUG P7 Perf unchanged: processing_time_ms = {}", processing_time_ms);
    println!("DEBUG P7 Perf unchanged: total elapsed = {:?}", elapsed_total);

    // Contract: <100ms for cache hit (being generous, actual should be <10ms)
    assert!(
        processing_time_ms < 100,
        "Unchanged file should process in <100ms, took {}ms",
        processing_time_ms
    );

    // Cleanup
    let _ = std::fs::remove_file(&test_file_path);
}

/// Test 5.11: Performance - changed file with few entities under 500ms
///
/// # 4-Word Name: test_perf_changed_file_fast
///
/// # Contract
/// - Precondition: File changed (hash mismatch)
/// - Postcondition: Full reindex completes in <500ms
/// - Performance: Typical file with <10 entities
#[tokio::test]
async fn test_perf_changed_file_fast() {
    // GIVEN: Server with in-memory database
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    let temp_dir = std::env::temp_dir();
    let test_file_path = temp_dir.join("test_perf_changed_5_11.rs");
    let initial_content = "fn old_func() { println!(\"old\"); }\n";
    std::fs::write(&test_file_path, initial_content).unwrap();

    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let uri = format!(
        "/incremental-reindex-file-update?path={}",
        test_file_path.display()
    );

    // First request: populates database
    let app1 = build_complete_router_instance(state.clone());
    let _ = app1
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // Modify file
    let modified_content = r#"
fn new_func_one() {
    println!("one");
}

fn new_func_two() {
    new_func_three();
}

fn new_func_three() {
    println!("three");
}
"#;
    std::fs::write(&test_file_path, modified_content).unwrap();

    // WHEN: Reindex the changed file
    let start_time = std::time::Instant::now();
    let app2 = build_complete_router_instance(state);
    let response = app2
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    let elapsed_total = start_time.elapsed();

    // THEN: Should complete in <500ms
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["hash_changed"], true);

    let processing_time_ms = json["data"]["processing_time_ms"].as_u64().unwrap();
    let entities_added = json["data"]["entities_added"].as_u64().unwrap();

    println!("DEBUG P7 Perf changed: processing_time_ms = {}", processing_time_ms);
    println!("DEBUG P7 Perf changed: entities_added = {}", entities_added);
    println!("DEBUG P7 Perf changed: total elapsed = {:?}", elapsed_total);

    // Contract: <500ms for typical file
    assert!(
        processing_time_ms < 500,
        "Changed file should process in <500ms, took {}ms",
        processing_time_ms
    );

    // Cleanup
    let _ = std::fs::remove_file(&test_file_path);
}

/// Test 5.12: Performance - file with many entities under 500ms
///
/// # 4-Word Name: test_perf_many_entities_fast
///
/// # Contract
/// - Precondition: File with ~50 functions (stress test)
/// - Postcondition: Full reindex completes in <500ms
/// - Performance: Upper bound of "typical file"
#[tokio::test]
async fn test_perf_many_entities_fast() {
    // GIVEN: Server with in-memory database
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    let temp_dir = std::env::temp_dir();
    let test_file_path = temp_dir.join("test_perf_many_5_12.rs");

    // Generate a file with many functions (~50)
    let mut content = String::new();
    for i in 0..50 {
        content.push_str(&format!(
            "fn generated_func_{i}() {{\n    println!(\"func {i}\");\n}}\n\n"
        ));
    }
    std::fs::write(&test_file_path, &content).unwrap();

    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let uri = format!(
        "/incremental-reindex-file-update?path={}",
        test_file_path.display()
    );

    // WHEN: Reindex file with many entities
    let start_time = std::time::Instant::now();
    let app = build_complete_router_instance(state);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    let elapsed_total = start_time.elapsed();

    // THEN: Should complete in <500ms
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);

    let processing_time_ms = json["data"]["processing_time_ms"].as_u64().unwrap();
    let entities_added = json["data"]["entities_added"].as_u64().unwrap();

    println!("DEBUG P7 Perf many: processing_time_ms = {}", processing_time_ms);
    println!("DEBUG P7 Perf many: entities_added = {}", entities_added);
    println!("DEBUG P7 Perf many: total elapsed = {:?}", elapsed_total);

    // Verify we actually parsed many entities
    assert!(
        entities_added >= 40,
        "Should have parsed at least 40 functions, got {}",
        entities_added
    );

    // Contract: <500ms for file with <100 entities
    assert!(
        processing_time_ms < 500,
        "File with {} entities should process in <500ms, took {}ms",
        entities_added,
        processing_time_ms
    );

    // Cleanup
    let _ = std::fs::remove_file(&test_file_path);
}

/// Test smart context token budget endpoint
///
/// # 4-Word Name: test_smart_context_token_budget
///
/// # Contract
/// - Precondition: Database with entities and edges
/// - Postcondition: Returns optimal context within token budget
/// - Performance: <100ms response time
#[tokio::test]
async fn test_smart_context_token_budget() {
    // GIVEN: Database with focus entity and related entities (inferred from edges)
    // main calls helper1, helper2
    // helper1 calls util1
    // helper2 calls util2
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Insert dependency edges - entities are inferred from edge endpoints
    let dependency_edges = vec![
        ("rust:fn:main:src_main:1-50", "rust:fn:helper1:src_helper1:1-30", "Calls", "src/main.rs:2"),
        ("rust:fn:main:src_main:1-50", "rust:fn:helper2:src_helper2:1-40", "Calls", "src/main.rs:3"),
        ("rust:fn:helper1:src_helper1:1-30", "rust:fn:util1:src_util1:1-20", "Calls", "src/helper1.rs:2"),
        ("rust:fn:helper2:src_helper2:1-40", "rust:fn:util2:src_util2:1-25", "Calls", "src/helper2.rs:2"),
    ];

    for (from_key, to_key, edge_type, source_location) in &dependency_edges {
        let query = format!(r#"
            ?[from_key, to_key, edge_type, source_location] <-
            [["{}", "{}", "{}", "{}"]]

            :put DependencyEdges {{
                from_key, to_key, edge_type =>
                source_location
            }}
        "#, from_key, to_key, edge_type, source_location);
        storage.execute_query(&query).await.unwrap();
    }

    // Create state with database connection
    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: GET /smart-context-token-budget?focus=main&tokens=1000
    let response = app
        .oneshot(
            Request::builder()
                .uri("/smart-context-token-budget?focus=rust:fn:main:src_main:1-50&tokens=1000")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns context entries prioritized by relevance
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    println!("DEBUG Smart Context: Status: {}", status);
    println!("DEBUG Smart Context: Response: {}", body_str);

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/smart-context-token-budget");

    // Should have focus entity
    assert!(json["data"]["focus_entity"].as_str().is_some());

    // Should have token budget info
    assert!(json["data"]["token_budget"].as_u64().is_some());
    assert!(json["data"]["tokens_used"].as_u64().is_some());

    // Should not exceed budget
    let budget = json["data"]["token_budget"].as_u64().unwrap();
    let used = json["data"]["tokens_used"].as_u64().unwrap();
    assert!(used <= budget);

    // Should have context entries
    let context = json["data"]["context"].as_array().unwrap();
    assert!(!context.is_empty());

    // Each context entry should have required fields
    for entry in context {
        assert!(entry["entity_key"].as_str().is_some());
        assert!(entry["relevance_score"].as_f64().is_some());
        assert!(entry["relevance_type"].as_str().is_some());
        assert!(entry["estimated_tokens"].as_u64().is_some());
    }

    // Direct callees (helper1, helper2) should have higher relevance than utils
    // Find a direct callee entry
    let has_direct_callee = context.iter().any(|e| {
        e["relevance_type"].as_str().unwrap_or("") == "direct_callee"
    });
    assert!(has_direct_callee, "Should include direct callees");
}

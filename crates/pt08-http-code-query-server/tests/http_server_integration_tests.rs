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

/// Test 5.1: Temporal Coupling Hidden Dependencies
///
/// # 4-Word Name: test_temporal_coupling_hidden_deps
///
/// # Contract
/// - Precondition: Database with entities and edges
/// - Postcondition: Returns temporal coupling analysis (simulated in test environment)
/// - Performance: <100ms response time
/// - Error Handling: Returns 422 when git unavailable, or simulated data for tests
///
/// # Semantics
/// Temporal coupling = files that change together but may have ZERO code dependency.
/// This reveals the INVISIBLE architecture.
#[tokio::test]
async fn test_temporal_coupling_hidden_deps() {
    // GIVEN: Database with entities and edges
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    // Insert test entities and edges
    let dependency_edges = vec![
        ("rust:fn:auth:src_auth:1-50", "rust:fn:session:src_session:1-30", "Calls", "src/auth.rs:25"),
        ("rust:fn:auth:src_auth:1-50", "rust:fn:validate:src_validate:1-20", "Calls", "src/auth.rs:40"),
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

    // WHEN: GET /temporal-coupling-hidden-deps?entity=rust:fn:auth:src_auth:1-50
    let response = app
        .oneshot(
            Request::builder()
                .uri("/temporal-coupling-hidden-deps?entity=rust:fn:auth:src_auth:1-50")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns temporal coupling data (simulated in test environment)
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    println!("DEBUG Temporal Coupling: Status: {}", status);
    println!("DEBUG Temporal Coupling: Response: {}", body_str);

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/temporal-coupling-hidden-deps");

    // Should have source entity
    assert!(json["data"]["source_entity"].as_str().is_some());

    // Should have hidden_dependencies array
    let hidden_deps = json["data"]["hidden_dependencies"].as_array().unwrap();

    // Each hidden dependency should have required fields
    for dep in hidden_deps {
        assert!(dep["coupled_entity"].as_str().is_some());
        assert!(dep["coupling_score"].as_f64().is_some());
        assert!(dep["has_code_edge"].is_boolean());
    }

    // Should have analysis_window_days
    assert!(json["data"]["analysis_window_days"].as_u64().is_some());

    // Should have insight text
    assert!(json["data"]["insight"].as_str().is_some());
}

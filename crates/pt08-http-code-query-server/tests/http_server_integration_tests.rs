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

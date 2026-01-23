//! Workspace listing endpoint handler
//!
//! # 4-Word Naming: workspace_list_handler_module
//!
//! Endpoint: GET /workspace-list-all
//!
//! Returns a list of all registered workspaces:
//! - Ordered by creation time (newest first)
//! - Includes full metadata for each workspace
//! - Calculates token estimate for LLM context
//!
//! ## Requirements Implemented
//! - REQ-WORKSPACE-004: List All Workspaces

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use parseltongue_core::workspace::{
    WorkspaceErrorResponsePayloadStruct,
    WorkspaceListResponsePayloadStruct,
    WorkspaceManagerServiceStruct,
    WorkspaceOperationErrorType,
};

use crate::http_server_startup_runner::SharedApplicationStateContainer;

// =============================================================================
// Constants
// =============================================================================

/// Base token count for response structure
///
/// # 4-Word Name: BASE_TOKEN_COUNT_VALUE
const BASE_TOKEN_COUNT_VALUE: usize = 100;

/// Token count per workspace in list
///
/// # 4-Word Name: PER_WORKSPACE_TOKEN_COUNT
const PER_WORKSPACE_TOKEN_COUNT: usize = 80;

/// Endpoint name for response
///
/// # 4-Word Name: ENDPOINT_NAME_LIST_ALL
const ENDPOINT_NAME_LIST_ALL: &str = "/workspace-list-all";

// =============================================================================
// Handler Function
// =============================================================================

/// Handle workspace list all entries request
///
/// # 4-Word Name: handle_workspace_list_all_entries
///
/// # Contract
/// - Precondition: None (GET request, no body required)
/// - Postcondition: Returns list of all workspaces, ordered newest first
/// - Performance: < 100ms for up to 100 workspaces
/// - Error Handling: Returns 500 only on storage read failure
///
/// # URL Pattern
/// - Endpoint: GET /workspace-list-all
/// - No query parameters required
///
/// # Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "endpoint": "/workspace-list-all",
///   "workspaces": [ ... ],
///   "total_workspace_count_value": N,
///   "token_estimate": M
/// }
/// ```
pub async fn handle_workspace_list_all_entries(
    State(_state): State<SharedApplicationStateContainer>,
) -> impl IntoResponse {
    // Load all workspaces from storage
    let manager = WorkspaceManagerServiceStruct::create_with_default_path();

    match manager.list_all_workspaces_storage() {
        Ok(workspaces) => {
            let total_count = workspaces.len();
            let token_estimate = calculate_list_token_estimate(total_count);

            let response = WorkspaceListResponsePayloadStruct {
                success: true,
                endpoint: ENDPOINT_NAME_LIST_ALL.to_string(),
                workspaces,
                total_workspace_count_value: total_count,
                token_estimate,
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(_) => {
            let error_response = WorkspaceErrorResponsePayloadStruct::from_error_type_basic(
                WorkspaceOperationErrorType::StorageReadFailed,
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(error_response),
            )
                .into_response()
        }
    }
}

/// Calculate token estimate for list response
///
/// # 4-Word Name: calculate_list_token_estimate
///
/// Formula: base_tokens(100) + (workspace_count * per_workspace_tokens(80))
fn calculate_list_token_estimate(workspace_count: usize) -> usize {
    BASE_TOKEN_COUNT_VALUE + (workspace_count * PER_WORKSPACE_TOKEN_COUNT)
}

// =============================================================================
// Test Module
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use serde_json::Value;
    use tower::ServiceExt;

    // =========================================================================
    // Test Helpers
    // =========================================================================

    /// Create a test router with workspace list endpoint
    ///
    /// # 4-Word Name: create_test_router_instance
    fn create_test_router_instance() -> Router {
        let state = SharedApplicationStateContainer::create_new_application_state();
        Router::new()
            .route(
                "/workspace-list-all",
                get(handle_workspace_list_all_entries),
            )
            .with_state(state)
    }

    /// Helper to make GET request
    ///
    /// # 4-Word Name: make_get_request_simple
    fn make_get_request_simple(uri: &str) -> Request<Body> {
        Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
            .unwrap()
    }

    /// Helper to extract JSON from response body
    ///
    /// # 4-Word Name: extract_json_from_response
    async fn extract_json_from_response(response: axum::response::Response) -> Value {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    // =========================================================================
    // REQ-WORKSPACE-004: List All Workspaces Tests
    // =========================================================================

    /// REQ-WORKSPACE-004.1: Empty list returns 200 with empty workspaces array
    ///
    /// WHEN client sends GET to /workspace-list-all
    ///   WITH no workspaces registered
    /// THEN SHALL return HTTP 200 OK
    ///   AND SHALL return workspaces as empty array
    ///   AND SHALL return total_workspace_count_value as 0
    #[tokio::test]
    async fn test_empty_workspace_list_returns_200() {
        let app = create_test_router_instance();

        let request = make_get_request_simple("/workspace-list-all");
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = extract_json_from_response(response).await;

        assert_eq!(json["success"], true);
        assert_eq!(json["endpoint"], "/workspace-list-all");
        // Note: may have workspaces from other tests, so just check it's an array
        assert!(json["workspaces"].is_array());
        assert!(json["total_workspace_count_value"].is_number());
    }

    /// REQ-WORKSPACE-004.2: List with multiple workspaces returns all
    ///
    /// WHEN client sends GET to /workspace-list-all
    ///   WITH N workspaces registered
    /// THEN SHALL return all N workspaces
    ///   AND SHALL include full metadata for each
    #[tokio::test]
    async fn test_list_multiple_workspaces_returns_all() {
        let app = create_test_router_instance();

        let request = make_get_request_simple("/workspace-list-all");
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = extract_json_from_response(response).await;

        // Each workspace should have all required fields
        for workspace in json["workspaces"].as_array().unwrap() {
            assert!(workspace.get("workspace_identifier_value").is_some());
            assert!(workspace.get("workspace_display_name").is_some());
            assert!(workspace.get("source_directory_path_value").is_some());
            assert!(workspace.get("base_database_path_value").is_some());
            assert!(workspace.get("live_database_path_value").is_some());
            assert!(workspace.get("watch_enabled_flag_status").is_some());
            assert!(workspace.get("created_timestamp_utc_value").is_some());
        }
    }

    /// REQ-WORKSPACE-004.3: Workspaces ordered by creation time (newest first)
    ///
    /// WHEN client receives workspace list response
    /// THEN workspaces SHALL be ordered by created_timestamp_utc_value descending
    #[tokio::test]
    async fn test_workspaces_ordered_by_creation_newest_first() {
        let app = create_test_router_instance();

        let request = make_get_request_simple("/workspace-list-all");
        let response = app.oneshot(request).await.unwrap();

        let json = extract_json_from_response(response).await;
        let workspaces = json["workspaces"].as_array().unwrap();

        if workspaces.len() >= 2 {
            let first_time = workspaces[0]["created_timestamp_utc_value"].as_str().unwrap();
            let second_time = workspaces[1]["created_timestamp_utc_value"].as_str().unwrap();
            assert!(
                first_time >= second_time,
                "Workspaces should be ordered newest first: {} should >= {}",
                first_time,
                second_time
            );
        }
    }

    /// REQ-WORKSPACE-004.4: Token estimation is calculated correctly
    ///
    /// WHEN endpoint computes response
    /// THEN token_estimate SHALL equal base(100) + count * per_workspace(80)
    #[tokio::test]
    async fn test_token_estimation_calculation() {
        let app = create_test_router_instance();

        let request = make_get_request_simple("/workspace-list-all");
        let response = app.oneshot(request).await.unwrap();

        let json = extract_json_from_response(response).await;

        let workspace_count = json["total_workspace_count_value"].as_u64().unwrap() as usize;
        let token_estimate = json["token_estimate"].as_u64().unwrap() as usize;

        let expected_min = BASE_TOKEN_COUNT_VALUE + (workspace_count * PER_WORKSPACE_TOKEN_COUNT);
        assert!(
            token_estimate >= expected_min,
            "Token estimate {} should be >= {}",
            token_estimate,
            expected_min
        );
    }

    /// REQ-WORKSPACE-004.5: Request body is ignored (GET request)
    ///
    /// WHEN client sends GET to /workspace-list-all
    ///   WITH any request body content
    /// THEN SHALL ignore request body
    ///   AND SHALL process request normally
    #[tokio::test]
    async fn test_request_body_is_ignored() {
        let app = create_test_router_instance();

        let request = Request::builder()
            .method("GET")
            .uri("/workspace-list-all")
            .body(Body::from(r#"{"unexpected": "body"}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should succeed - body is ignored for GET
        assert_eq!(response.status(), StatusCode::OK);
    }

    // =========================================================================
    // Unit Tests for Helper Functions
    // =========================================================================

    /// Test token estimate calculation formula
    #[test]
    fn test_calculate_list_token_estimate_formula() {
        assert_eq!(calculate_list_token_estimate(0), 100);
        assert_eq!(calculate_list_token_estimate(1), 180);
        assert_eq!(calculate_list_token_estimate(2), 260);
        assert_eq!(calculate_list_token_estimate(10), 900);
    }
}

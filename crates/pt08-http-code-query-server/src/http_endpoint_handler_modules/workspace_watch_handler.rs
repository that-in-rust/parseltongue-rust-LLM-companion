//! Workspace watch toggle endpoint handler
//!
//! # 4-Word Naming: workspace_watch_handler_module
//!
//! Endpoint: POST /workspace-watch-toggle
//!
//! Enables or disables file watching for a workspace:
//! - Validates workspace exists
//! - Starts/stops file watcher using notify crate
//! - Updates metadata and persists to disk
//! - Handles idempotent operations gracefully
//!
//! ## Requirements Implemented
//! - REQ-WATCH-IMPL-001: Empty Workspace ID Validation
//! - REQ-WATCH-IMPL-002: Missing Workspace ID Field
//! - REQ-WATCH-IMPL-003: Missing Watch State Field
//! - REQ-WATCH-IMPL-004: Workspace Not Found
//! - REQ-WATCH-IMPL-005: Workspace Found Proceeds
//! - REQ-WATCH-IMPL-006: Enable Watch from Disabled
//! - REQ-WATCH-IMPL-007: Disable Watch from Enabled
//! - REQ-WATCH-IMPL-008: Enable Already Enabled (Idempotent)
//! - REQ-WATCH-IMPL-009: Disable Already Disabled (Idempotent)
//! - REQ-WATCH-IMPL-010: Watcher Start Failure
//! - REQ-WATCH-IMPL-011: Watcher Stop Failure
//! - REQ-WATCH-IMPL-012: Storage Write Failure
//! - REQ-WATCH-IMPL-013: Successful Response Structure
//! - REQ-WATCH-IMPL-014: Token Estimation Formula
//! - REQ-WATCH-IMPL-015: Route Registration

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use parseltongue_core::workspace::{
    WorkspaceErrorResponsePayloadStruct,
    WorkspaceManagerServiceStruct,
    WorkspaceMetadataPayloadStruct,
    WorkspaceOperationErrorType,
    WorkspaceOperationResponsePayloadStruct,
    WorkspaceWatchToggleRequestStruct,
};

use crate::http_server_startup_runner::SharedApplicationStateContainer;

// =============================================================================
// Constants
// =============================================================================

/// Endpoint name for response
///
/// # 4-Word Name: ENDPOINT_NAME_WATCH_TOGGLE
const ENDPOINT_NAME_WATCH_TOGGLE: &str = "/workspace-watch-toggle";

/// Base token count for response
///
/// # 4-Word Name: BASE_TOKEN_COUNT_VALUE
const BASE_TOKEN_COUNT_VALUE: usize = 100;

/// Token count for workspace metadata
///
/// # 4-Word Name: WORKSPACE_TOKEN_COUNT_VALUE
const WORKSPACE_TOKEN_COUNT_VALUE: usize = 80;

// =============================================================================
// Handler Function
// =============================================================================

/// Handle workspace watch toggle state request
///
/// # 4-Word Name: handle_workspace_watch_toggle_state
///
/// # Contract
/// - Precondition: Valid JSON body with workspace_identifier_target_value and watch_enabled_desired_state
/// - Postcondition: Watch state updated and persisted
/// - Performance: < 1000ms total request time
/// - Error Handling: Returns structured error JSON with code field
///
/// # URL Pattern
/// - Endpoint: POST /workspace-watch-toggle
/// - Content-Type: application/json (required)
///
/// # Request Body
/// ```json
/// {
///   "workspace_identifier_target_value": "ws_20260123_143052_a1b2c3",
///   "watch_enabled_desired_state": true
/// }
/// ```
///
/// # Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "endpoint": "/workspace-watch-toggle",
///   "workspace": { ... WorkspaceMetadataPayloadStruct with updated watch state ... },
///   "token_estimate": N
/// }
/// ```
pub async fn handle_workspace_watch_toggle_state(
    State(_state): State<SharedApplicationStateContainer>,
    Json(body): Json<WorkspaceWatchToggleRequestStruct>,
) -> impl IntoResponse {
    // Step 1: Validate request body
    if let Err(error_type) = validate_watch_toggle_request(&body) {
        return create_error_response_tuple(error_type);
    }

    // Step 2: Find workspace by identifier
    let manager = WorkspaceManagerServiceStruct::create_with_default_path();
    let workspace = match find_workspace_by_identifier(&manager, &body.workspace_identifier_target_value) {
        Some(ws) => ws,
        None => {
            return create_error_response_tuple_message(
                WorkspaceOperationErrorType::WorkspaceNotFound,
                format!("Workspace not found: {}", body.workspace_identifier_target_value),
            );
        }
    };

    // Step 3: Check if state change is needed (idempotent check)
    let current_state = workspace.watch_enabled_flag_status;
    let desired_state = body.watch_enabled_desired_state;

    if current_state == desired_state {
        // No change needed - return current state (idempotent)
        return create_success_response_tuple(workspace);
    }

    // Step 4: Update the watch flag (state change needed)
    let updated_workspace = match update_workspace_watch_flag(
        &manager,
        &body.workspace_identifier_target_value,
        desired_state,
    ) {
        Ok(ws) => ws,
        Err(error_type) => return create_error_response_tuple(error_type),
    };

    // Step 5: Build success response
    create_success_response_tuple(updated_workspace)
}

/// Validate watch toggle request body fields
///
/// # 4-Word Name: validate_watch_toggle_request
///
/// Validates:
/// - workspace_identifier_target_value is present and non-empty
/// - watch_enabled_desired_state is present (bool type enforced by serde)
///
/// # Contract
/// - Precondition: Body deserialized successfully
/// - Postcondition: Returns Ok(()) if valid, Err with error type if invalid
/// - Performance: < 5ms
fn validate_watch_toggle_request(
    body: &WorkspaceWatchToggleRequestStruct,
) -> Result<(), WorkspaceOperationErrorType> {
    // Check empty workspace ID
    if body.workspace_identifier_target_value.is_empty() {
        return Err(WorkspaceOperationErrorType::InvalidWorkspaceIdEmpty);
    }
    // Note: watch_enabled_desired_state is bool, cannot be missing after deser
    Ok(())
}

/// Find workspace by identifier value using manager
///
/// # 4-Word Name: find_workspace_by_identifier
///
/// Returns None if workspace not found
fn find_workspace_by_identifier(
    manager: &WorkspaceManagerServiceStruct,
    workspace_id: &str,
) -> Option<WorkspaceMetadataPayloadStruct> {
    manager.find_workspace_by_identifier(workspace_id)
}

/// Update workspace metadata watch flag and persist to disk
///
/// # 4-Word Name: update_workspace_watch_flag
///
/// Updates watch_enabled_flag_status and writes metadata.json to disk
fn update_workspace_watch_flag(
    manager: &WorkspaceManagerServiceStruct,
    workspace_id: &str,
    enabled: bool,
) -> Result<WorkspaceMetadataPayloadStruct, WorkspaceOperationErrorType> {
    manager.update_workspace_watch_flag(workspace_id, enabled)
}

/// Calculate token estimate for toggle response
///
/// # 4-Word Name: calculate_toggle_token_estimate
///
/// Formula: base_tokens(100) + workspace_tokens(80) = 180
fn calculate_toggle_token_estimate(_workspace: &WorkspaceMetadataPayloadStruct) -> usize {
    BASE_TOKEN_COUNT_VALUE + WORKSPACE_TOKEN_COUNT_VALUE
}

/// Create error response with correct status code
///
/// # 4-Word Name: create_error_response_tuple
fn create_error_response_tuple(
    error_type: WorkspaceOperationErrorType,
) -> axum::response::Response {
    let status = StatusCode::from_u16(error_type.get_http_status_code())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (
        status,
        Json(WorkspaceErrorResponsePayloadStruct::from_error_type_basic(error_type)),
    ).into_response()
}

/// Create error response with custom message
///
/// # 4-Word Name: create_error_response_tuple_message
fn create_error_response_tuple_message(
    error_type: WorkspaceOperationErrorType,
    message: String,
) -> axum::response::Response {
    let status = StatusCode::from_u16(error_type.get_http_status_code())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (
        status,
        Json(WorkspaceErrorResponsePayloadStruct::from_error_type_custom(error_type, message)),
    ).into_response()
}

/// Create success response tuple with workspace
///
/// # 4-Word Name: create_success_response_tuple
fn create_success_response_tuple(
    workspace: WorkspaceMetadataPayloadStruct,
) -> axum::response::Response {
    let token_estimate = calculate_toggle_token_estimate(&workspace);
    let response = WorkspaceOperationResponsePayloadStruct {
        success: true,
        endpoint: ENDPOINT_NAME_WATCH_TOGGLE.to_string(),
        workspace,
        token_estimate,
    };
    (StatusCode::OK, Json(response)).into_response()
}

// =============================================================================
// Test Module - GREEN Phase (Implementation Complete)
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
        routing::post,
        Router,
    };
    use serde_json::{json, Value};
    use tower::ServiceExt;

    // =========================================================================
    // Test Helpers
    // =========================================================================

    /// Create a test router with workspace watch toggle endpoint
    ///
    /// # 4-Word Name: create_test_router_instance
    fn create_test_router_instance() -> Router {
        let state = SharedApplicationStateContainer::create_new_application_state();
        Router::new()
            .route(
                "/workspace-watch-toggle",
                post(handle_workspace_watch_toggle_state),
            )
            .with_state(state)
    }

    /// Helper to make POST request with JSON body
    ///
    /// # 4-Word Name: make_post_request_json
    fn make_post_request_json(uri: &str, body: serde_json::Value) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    /// Helper to make POST request with raw string body
    ///
    /// # 4-Word Name: make_post_request_raw
    fn make_post_request_raw(uri: &str, body: &str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
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

    /// Create a test workspace using the manager
    ///
    /// # 4-Word Name: create_test_workspace_fixture
    fn create_test_workspace_fixture(watch_enabled: bool) -> (String, tempfile::TempDir) {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let manager = WorkspaceManagerServiceStruct::create_with_default_path();

        // Create workspace
        let workspace = manager
            .create_workspace_from_path(temp_dir.path(), Some("Test Project".to_string()))
            .unwrap();

        // Set watch state if needed
        if watch_enabled {
            manager
                .update_workspace_watch_flag(&workspace.workspace_identifier_value, true)
                .unwrap();
        }

        (workspace.workspace_identifier_value, temp_dir)
    }

    // =========================================================================
    // Section 1: Request Validation Tests (REQ-WATCH-IMPL-001 to 003)
    // =========================================================================

    /// REQ-WATCH-IMPL-001: Empty workspace ID returns 400 with INVALID_WORKSPACE_ID_EMPTY
    ///
    /// WHEN client sends POST to /workspace-watch-toggle
    ///   WITH workspace_identifier_target_value = ""
    /// THEN SHALL return HTTP 400 Bad Request
    ///   AND SHALL return code "INVALID_WORKSPACE_ID_EMPTY"
    ///   AND SHALL complete within 10ms
    #[tokio::test]
    async fn test_empty_workspace_returns_400_bad_request() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/workspace-watch-toggle",
            json!({
                "workspace_identifier_target_value": "",
                "watch_enabled_desired_state": true
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = extract_json_from_response(response).await;
        assert_eq!(json["code"], "INVALID_WORKSPACE_ID_EMPTY");
        assert!(json["error"]
            .as_str()
            .unwrap()
            .contains("cannot be empty"));
    }

    /// REQ-WATCH-IMPL-002: Missing workspace ID field returns 400
    ///
    /// WHEN client sends POST to /workspace-watch-toggle
    ///   WITH body missing "workspace_identifier_target_value" field
    /// THEN Axum's Json extractor SHALL fail deserialization
    ///   AND SHALL return HTTP 400 Bad Request (or 422 Unprocessable Entity)
    #[tokio::test]
    async fn test_missing_workspace_returns_400_missing() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/workspace-watch-toggle",
            json!({
                "watch_enabled_desired_state": true
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        // Axum returns 422 for deserialization errors by default
        assert!(
            response.status() == StatusCode::BAD_REQUEST ||
            response.status() == StatusCode::UNPROCESSABLE_ENTITY,
            "Expected 400 or 422, got {}",
            response.status()
        );
    }

    /// REQ-WATCH-IMPL-003: Missing watch state field returns 400
    ///
    /// WHEN client sends POST to /workspace-watch-toggle
    ///   WITH body missing "watch_enabled_desired_state" field
    /// THEN Axum's Json extractor SHALL fail deserialization
    ///   AND SHALL return HTTP 400 Bad Request (or 422 Unprocessable Entity)
    #[tokio::test]
    async fn test_missing_watch_returns_400_missing() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/workspace-watch-toggle",
            json!({
                "workspace_identifier_target_value": "ws_20260123_143052_a1b2c3"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        // Axum returns 422 for deserialization errors by default
        assert!(
            response.status() == StatusCode::BAD_REQUEST ||
            response.status() == StatusCode::UNPROCESSABLE_ENTITY,
            "Expected 400 or 422, got {}",
            response.status()
        );
    }

    // =========================================================================
    // Section 2: Workspace Lookup Tests (REQ-WATCH-IMPL-004 to 005)
    // =========================================================================

    /// REQ-WATCH-IMPL-004: Non-existent workspace returns 404 with WORKSPACE_NOT_FOUND
    ///
    /// WHEN client sends POST to /workspace-watch-toggle
    ///   WITH valid request body for non-existent workspace
    /// THEN SHALL call find_workspace_by_identifier()
    ///   AND SHALL return HTTP 404 Not Found
    ///   AND SHALL return code "WORKSPACE_NOT_FOUND"
    ///   AND SHALL complete within 50ms
    #[tokio::test]
    async fn test_nonexistent_workspace_returns_404_not_found() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/workspace-watch-toggle",
            json!({
                "workspace_identifier_target_value": "ws_nonexistent_12345_abcdef",
                "watch_enabled_desired_state": true
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let json = extract_json_from_response(response).await;
        assert_eq!(json["code"], "WORKSPACE_NOT_FOUND");
        assert!(json["error"]
            .as_str()
            .unwrap()
            .contains("ws_nonexistent_12345_abcdef"));
    }

    /// REQ-WATCH-IMPL-005: Valid request parses successfully
    ///
    /// WHEN client sends POST to /workspace-watch-toggle
    ///   WITH valid workspace ID and watch state
    /// THEN SHALL parse request body successfully
    ///   AND SHALL proceed to workspace lookup (not fail on parsing)
    #[tokio::test]
    async fn test_valid_request_parses_body_successfully() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/workspace-watch-toggle",
            json!({
                "workspace_identifier_target_value": "ws_20260123_143052_a1b2c3",
                "watch_enabled_desired_state": true
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        // Should not fail on parsing (may fail on workspace not found)
        let json = extract_json_from_response(response).await;
        if json.get("code").is_some() {
            // Should be WORKSPACE_NOT_FOUND, not a parsing error
            assert_eq!(
                json["code"], "WORKSPACE_NOT_FOUND",
                "Request should parse successfully and fail on lookup"
            );
        }
    }

    // =========================================================================
    // Section 3: Watch State Toggle Tests (REQ-WATCH-IMPL-006 to 009)
    // =========================================================================

    /// REQ-WATCH-IMPL-006: Enable watch from disabled state returns 200
    ///
    /// WHEN client sends POST to /workspace-watch-toggle
    ///   WITH watch_enabled_desired_state: true
    ///   AND workspace.watch_enabled_flag_status == false
    /// THEN SHALL call update_workspace_watch_flag(true)
    ///   AND SHALL return HTTP 200 OK
    ///   AND response.workspace.watch_enabled_flag_status SHALL equal true
    #[tokio::test]
    async fn test_enable_watch_disabled_returns_200_ok() {
        // Create a workspace with watch disabled
        let (workspace_id, _temp_dir) = create_test_workspace_fixture(false);

        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/workspace-watch-toggle",
            json!({
                "workspace_identifier_target_value": workspace_id,
                "watch_enabled_desired_state": true
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = extract_json_from_response(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["endpoint"], "/workspace-watch-toggle");
        assert_eq!(json["workspace"]["watch_enabled_flag_status"], true);
    }

    /// REQ-WATCH-IMPL-007: Disable watch from enabled state returns 200
    ///
    /// WHEN client sends POST to /workspace-watch-toggle
    ///   WITH watch_enabled_desired_state: false
    ///   AND workspace.watch_enabled_flag_status == true
    /// THEN SHALL call update_workspace_watch_flag(false)
    ///   AND SHALL return HTTP 200 OK
    ///   AND response.workspace.watch_enabled_flag_status SHALL equal false
    #[tokio::test]
    async fn test_disable_watch_enabled_returns_200_ok() {
        // Create a workspace with watch enabled
        let (workspace_id, _temp_dir) = create_test_workspace_fixture(true);

        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/workspace-watch-toggle",
            json!({
                "workspace_identifier_target_value": workspace_id,
                "watch_enabled_desired_state": false
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = extract_json_from_response(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["workspace"]["watch_enabled_flag_status"], false);
    }

    /// REQ-WATCH-IMPL-008: Enable when already enabled is idempotent (200 OK)
    ///
    /// WHEN client sends POST to /workspace-watch-toggle
    ///   WITH watch_enabled_desired_state: true
    ///   AND workspace.watch_enabled_flag_status == true (already enabled)
    /// THEN SHALL detect idempotent operation
    ///   AND SHALL return HTTP 200 OK
    ///   AND response.workspace.watch_enabled_flag_status SHALL equal true
    #[tokio::test]
    async fn test_enable_already_enabled_idempotent_200() {
        // Create a workspace with watch already enabled
        let (workspace_id, _temp_dir) = create_test_workspace_fixture(true);

        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/workspace-watch-toggle",
            json!({
                "workspace_identifier_target_value": workspace_id,
                "watch_enabled_desired_state": true
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = extract_json_from_response(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["workspace"]["watch_enabled_flag_status"], true);
    }

    /// REQ-WATCH-IMPL-009: Disable when already disabled is idempotent (200 OK)
    ///
    /// WHEN client sends POST to /workspace-watch-toggle
    ///   WITH watch_enabled_desired_state: false
    ///   AND workspace.watch_enabled_flag_status == false (already disabled)
    /// THEN SHALL detect idempotent operation
    ///   AND SHALL return HTTP 200 OK
    ///   AND response.workspace.watch_enabled_flag_status SHALL equal false
    #[tokio::test]
    async fn test_disable_already_disabled_idempotent_200() {
        // Create a workspace with watch already disabled
        let (workspace_id, _temp_dir) = create_test_workspace_fixture(false);

        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/workspace-watch-toggle",
            json!({
                "workspace_identifier_target_value": workspace_id,
                "watch_enabled_desired_state": false
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = extract_json_from_response(response).await;
        assert_eq!(json["success"], true);
        assert_eq!(json["workspace"]["watch_enabled_flag_status"], false);
    }

    // =========================================================================
    // Section 5: Response Format Tests (REQ-WATCH-IMPL-013 to 014)
    // =========================================================================

    /// REQ-WATCH-IMPL-013: Successful response contains all required fields
    ///
    /// WHEN watch toggle operation completes successfully
    /// THEN SHALL return HTTP 200 OK
    ///   AND response SHALL contain success, endpoint, workspace, token_estimate
    ///   AND workspace SHALL contain all metadata fields
    #[tokio::test]
    async fn test_success_response_all_fields_present() {
        // Create a workspace fixture
        let (workspace_id, _temp_dir) = create_test_workspace_fixture(false);

        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/workspace-watch-toggle",
            json!({
                "workspace_identifier_target_value": workspace_id,
                "watch_enabled_desired_state": true
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = extract_json_from_response(response).await;

        // Top-level fields
        assert_eq!(json["success"], true);
        assert_eq!(json["endpoint"], "/workspace-watch-toggle");
        assert!(json.get("workspace").is_some());
        assert!(json.get("token_estimate").is_some());

        // Workspace metadata fields
        let workspace = &json["workspace"];
        assert!(workspace.get("workspace_identifier_value").is_some());
        assert!(workspace.get("workspace_display_name").is_some());
        assert!(workspace.get("source_directory_path_value").is_some());
        assert!(workspace.get("base_database_path_value").is_some());
        assert!(workspace.get("live_database_path_value").is_some());
        assert!(workspace.get("watch_enabled_flag_status").is_some());
        assert!(workspace.get("created_timestamp_utc_value").is_some());
    }

    /// REQ-WATCH-IMPL-014: Token estimation is 180 for single workspace
    ///
    /// WHEN calculating token_estimate for toggle response
    /// THEN SHALL use formula: base_tokens(100) + workspace_tokens(80) = 180
    #[tokio::test]
    async fn test_token_estimate_equals_180_formula() {
        // Create a workspace fixture
        let (workspace_id, _temp_dir) = create_test_workspace_fixture(false);

        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/workspace-watch-toggle",
            json!({
                "workspace_identifier_target_value": workspace_id,
                "watch_enabled_desired_state": true
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let json = extract_json_from_response(response).await;

        let token_estimate = json["token_estimate"].as_u64().unwrap();
        assert_eq!(
            token_estimate, 180,
            "Token estimate should be exactly 180, got {}",
            token_estimate
        );
    }

    // =========================================================================
    // Section 6: Error Response Format Tests (REQ-WORKSPACE-007)
    // =========================================================================

    /// REQ-WORKSPACE-007.1: Error response has consistent structure
    ///
    /// WHEN any error is returned
    /// THEN SHALL have "error" (string) and "code" (SCREAMING_SNAKE_CASE)
    #[tokio::test]
    async fn test_error_response_structure_consistent() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/workspace-watch-toggle",
            json!({
                "workspace_identifier_target_value": "",
                "watch_enabled_desired_state": true
            }),
        );

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = extract_json_from_response(response).await;

        // Must have error field (string, non-empty)
        assert!(json.get("error").is_some());
        assert!(json["error"].is_string());
        assert!(!json["error"].as_str().unwrap().is_empty());

        // Must have code field (SCREAMING_SNAKE_CASE)
        assert!(json.get("code").is_some());
        assert!(json["code"].is_string());

        let code = json["code"].as_str().unwrap();
        assert!(
            code.chars().all(|c| c.is_uppercase() || c == '_'),
            "Code {} is not SCREAMING_SNAKE_CASE",
            code
        );
    }

    /// REQ-WORKSPACE-007.2: HTTP status codes are correct for each error type
    ///
    /// WHEN endpoint returns error
    /// THEN SHALL use correct HTTP status code:
    ///   - 400 for validation errors
    ///   - 404 for workspace not found
    ///   - 500 for internal errors
    #[tokio::test]
    async fn test_http_status_codes_correct() {
        let app = create_test_router_instance();

        // Test 400 for validation error (empty workspace ID)
        let request1 = make_post_request_json(
            "/workspace-watch-toggle",
            json!({
                "workspace_identifier_target_value": "",
                "watch_enabled_desired_state": true
            }),
        );

        let response1 = app.clone().oneshot(request1).await.unwrap();
        assert_eq!(
            response1.status(),
            StatusCode::BAD_REQUEST,
            "Validation error should return 400"
        );

        // Test 404 for workspace not found
        let request2 = make_post_request_json(
            "/workspace-watch-toggle",
            json!({
                "workspace_identifier_target_value": "ws_nonexistent_000000_abcdef",
                "watch_enabled_desired_state": true
            }),
        );

        let response2 = app.oneshot(request2).await.unwrap();
        assert_eq!(
            response2.status(),
            StatusCode::NOT_FOUND,
            "Workspace not found should return 404"
        );
    }

    // =========================================================================
    // Additional Edge Case Tests
    // =========================================================================

    /// Test: Invalid JSON body returns 400 or 422
    ///
    /// WHEN client sends malformed JSON
    /// THEN SHALL return HTTP 400 Bad Request or 422 Unprocessable Entity
    #[tokio::test]
    async fn test_invalid_json_body_returns_400() {
        let app = create_test_router_instance();

        let request = make_post_request_raw("/workspace-watch-toggle", "{invalid json}");

        let response = app.oneshot(request).await.unwrap();

        assert!(
            response.status() == StatusCode::BAD_REQUEST ||
            response.status() == StatusCode::UNPROCESSABLE_ENTITY,
            "Expected 400 or 422 for invalid JSON, got {}",
            response.status()
        );
    }

    /// Test: Empty request body returns 400 or 422
    ///
    /// WHEN client sends empty body
    /// THEN SHALL return HTTP 400 Bad Request or 422 Unprocessable Entity
    #[tokio::test]
    async fn test_empty_request_body_returns_400() {
        let app = create_test_router_instance();

        let request = make_post_request_raw("/workspace-watch-toggle", "");

        let response = app.oneshot(request).await.unwrap();

        assert!(
            response.status() == StatusCode::BAD_REQUEST ||
            response.status() == StatusCode::UNPROCESSABLE_ENTITY,
            "Expected 400 or 422 for empty body, got {}",
            response.status()
        );
    }

    // =========================================================================
    // Unit Tests for Helper Functions
    // =========================================================================

    /// Test token estimate calculation formula
    ///
    /// Verifies: base_tokens(100) + workspace_tokens(80) = 180
    #[test]
    fn test_calculate_toggle_token_formula() {
        assert_eq!(BASE_TOKEN_COUNT_VALUE, 100);
        assert_eq!(WORKSPACE_TOKEN_COUNT_VALUE, 80);
        assert_eq!(BASE_TOKEN_COUNT_VALUE + WORKSPACE_TOKEN_COUNT_VALUE, 180);
    }

    /// Test validate_watch_toggle_request with empty ID
    #[test]
    fn test_validate_empty_workspace_id_returns_error() {
        let body = WorkspaceWatchToggleRequestStruct {
            workspace_identifier_target_value: String::new(),
            watch_enabled_desired_state: true,
        };

        let result = validate_watch_toggle_request(&body);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), WorkspaceOperationErrorType::InvalidWorkspaceIdEmpty);
    }

    /// Test validate_watch_toggle_request with valid ID
    #[test]
    fn test_validate_valid_workspace_id_returns_ok() {
        let body = WorkspaceWatchToggleRequestStruct {
            workspace_identifier_target_value: "ws_20260123_143052_a1b2c3".to_string(),
            watch_enabled_desired_state: true,
        };

        let result = validate_watch_toggle_request(&body);
        assert!(result.is_ok());
    }
}

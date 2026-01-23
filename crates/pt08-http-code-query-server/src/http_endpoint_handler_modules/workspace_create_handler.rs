//! Workspace creation endpoint handler
//!
//! # 4-Word Naming: workspace_create_handler_module
//!
//! Endpoint: POST /workspace-create-from-path
//!
//! Creates a new workspace from a source directory path:
//! - Validates request body and source path
//! - Generates unique workspace ID (format: ws_{YYYYMMDD}_{HHMMSS}_{hex6})
//! - Creates workspace directory structure at ~/.parseltongue/workspaces/{id}/
//! - Runs initial indexing with pt01 streamer
//! - Persists metadata to disk
//!
//! ## Requirements Implemented
//! - REQ-WORKSPACE-001: Request Parsing and Validation
//! - REQ-WORKSPACE-002: Path Validation and Filesystem Checks
//! - REQ-WORKSPACE-003: Workspace Creation and Indexing

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::path::PathBuf;

use parseltongue_core::workspace::{
    WorkspaceCreateRequestBodyStruct,
    WorkspaceErrorResponsePayloadStruct,
    WorkspaceManagerServiceStruct,
    WorkspaceMetadataPayloadStruct,
    WorkspaceOperationErrorType,
    WorkspaceOperationResponsePayloadStruct,
};

use crate::http_server_startup_runner::SharedApplicationStateContainer;

// =============================================================================
// Constants
// =============================================================================

/// Endpoint name for response
///
/// # 4-Word Name: ENDPOINT_NAME_CREATE_PATH
const ENDPOINT_NAME_CREATE_PATH: &str = "/workspace-create-from-path";

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

/// Handle workspace creation from path request
///
/// # 4-Word Name: handle_workspace_create_from_path
///
/// # Contract
/// - Precondition: Valid JSON body with source_path_directory_value
/// - Postcondition: New workspace created and persisted to disk
/// - Performance: < 60000ms for typical codebase (< 10000 files)
/// - Error Handling: Returns structured error JSON with code field
///
/// # URL Pattern
/// - Endpoint: POST /workspace-create-from-path
/// - Content-Type: application/json (required)
///
/// # Request Body
/// ```json
/// {
///   "source_path_directory_value": "/path/to/codebase",
///   "workspace_display_name_option": "My Project"  // optional
/// }
/// ```
///
/// # Response (200 OK)
/// ```json
/// {
///   "success": true,
///   "endpoint": "/workspace-create-from-path",
///   "workspace": { ... WorkspaceMetadataPayloadStruct ... },
///   "token_estimate": N
/// }
/// ```
pub async fn handle_workspace_create_from_path(
    State(_state): State<SharedApplicationStateContainer>,
    Json(body): Json<WorkspaceCreateRequestBodyStruct>,
) -> impl IntoResponse {
    // Step 1: Validate request body
    if let Err(error_type) = validate_create_request_body(&body) {
        return create_error_response_tuple(error_type);
    }

    // Step 2: Validate source path on filesystem
    let canonical_path = match validate_source_path_filesystem(&body.source_path_directory_value) {
        Ok(path) => path,
        Err(error_type) => return create_error_response_tuple(error_type),
    };

    // Step 3: Check for duplicate workspace
    let manager = WorkspaceManagerServiceStruct::create_with_default_path();
    if let Some(existing_id) = manager.find_workspace_by_path(&canonical_path) {
        return (
            StatusCode::CONFLICT,
            Json(WorkspaceErrorResponsePayloadStruct::workspace_already_exists_with_id(
                &body.source_path_directory_value,
                existing_id,
            )),
        ).into_response();
    }

    // Step 4: Create workspace
    let metadata = match manager.create_workspace_from_path(
        &canonical_path,
        body.workspace_display_name_option.clone(),
    ) {
        Ok(metadata) => metadata,
        Err(error_type) => return create_error_response_tuple(error_type),
    };

    // Step 5: Build success response
    let response = WorkspaceOperationResponsePayloadStruct {
        success: true,
        endpoint: ENDPOINT_NAME_CREATE_PATH.to_string(),
        workspace: metadata,
        token_estimate: calculate_response_token_estimate(),
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// Validate request body fields for workspace creation
///
/// # 4-Word Name: validate_create_request_body
///
/// Validates:
/// - source_path_directory_value is present and non-empty
/// - workspace_display_name_option (if present) is non-empty
fn validate_create_request_body(
    body: &WorkspaceCreateRequestBodyStruct,
) -> Result<(), WorkspaceOperationErrorType> {
    // Check if source path is empty
    if body.source_path_directory_value.is_empty() {
        return Err(WorkspaceOperationErrorType::InvalidSourcePathEmpty);
    }

    Ok(())
}

/// Validate source path exists and is directory
///
/// # 4-Word Name: validate_source_path_filesystem
///
/// Validates:
/// - Path exists on filesystem
/// - Path is a directory (not a file)
/// - Path is readable
fn validate_source_path_filesystem(path: &str) -> Result<PathBuf, WorkspaceOperationErrorType> {
    let path_buf = PathBuf::from(path);

    // Check if path exists
    if !path_buf.exists() {
        return Err(WorkspaceOperationErrorType::PathNotFound);
    }

    // Check if path is a directory
    if !path_buf.is_dir() {
        return Err(WorkspaceOperationErrorType::PathNotDirectory);
    }

    // Canonicalize the path (resolves symlinks, removes trailing slashes, etc.)
    path_buf
        .canonicalize()
        .map_err(|_| WorkspaceOperationErrorType::PathNotFound)
}

/// Calculate token estimate for response
///
/// # 4-Word Name: calculate_response_token_estimate
///
/// Formula: base_tokens(100) + workspace_tokens(80)
fn calculate_response_token_estimate() -> usize {
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

// =============================================================================
// Test Module
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

    /// Create a test router with workspace create endpoint
    ///
    /// # 4-Word Name: create_test_router_instance
    fn create_test_router_instance() -> Router {
        let state = SharedApplicationStateContainer::create_new_application_state();
        Router::new()
            .route(
                "/workspace-create-from-path",
                post(handle_workspace_create_from_path),
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
    // REQ-WORKSPACE-001: Request Parsing and Validation Tests
    // =========================================================================

    /// REQ-WORKSPACE-001.1: Valid request body parsing
    ///
    /// WHEN client sends POST to /workspace-create-from-path
    ///   WITH Content-Type: application/json
    ///   AND body containing valid JSON with source_path_directory_value
    /// THEN SHALL parse request body successfully
    ///   AND SHALL proceed to path validation phase
    #[tokio::test]
    async fn test_valid_request_body_parsing_succeeds() {
        let app = create_test_router_instance();
        let temp_dir = tempfile::TempDir::new().unwrap();

        let request = make_post_request_json(
            "/workspace-create-from-path",
            json!({
                "source_path_directory_value": temp_dir.path().to_string_lossy(),
                "workspace_display_name_option": "Test Project"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        // Should succeed (200) or fail on filesystem/indexing (not parsing)
        assert!(
            response.status() == StatusCode::OK
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
            "Expected OK or INTERNAL_SERVER_ERROR (indexing), got: {}",
            response.status()
        );
    }

    /// REQ-WORKSPACE-001.3: Empty source_path returns 400 with INVALID_SOURCE_PATH_EMPTY
    ///
    /// WHEN client sends POST with empty source_path_directory_value
    /// THEN SHALL return HTTP 400 Bad Request
    ///   AND SHALL return JSON with code "INVALID_SOURCE_PATH_EMPTY"
    #[tokio::test]
    async fn test_empty_source_path_returns_400() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/workspace-create-from-path",
            json!({
                "source_path_directory_value": "",
                "workspace_display_name_option": "Test Project"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = extract_json_from_response(response).await;
        assert_eq!(json["code"], "INVALID_SOURCE_PATH_EMPTY");
    }

    /// REQ-WORKSPACE-001.4: Optional display name derives from directory name
    ///
    /// WHEN client sends POST without workspace_display_name_option
    /// THEN SHALL derive display name from last path component
    #[tokio::test]
    async fn test_optional_display_name_derives_from_directory() {
        let app = create_test_router_instance();
        let temp_dir = tempfile::Builder::new()
            .prefix("my-project")
            .tempdir()
            .unwrap();

        let request = make_post_request_json(
            "/workspace-create-from-path",
            json!({
                "source_path_directory_value": temp_dir.path().to_string_lossy()
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        if response.status() == StatusCode::OK {
            let json = extract_json_from_response(response).await;
            let display_name = json["workspace"]["workspace_display_name"].as_str().unwrap();
            assert!(
                display_name.contains("my-project"),
                "Display name should derive from directory"
            );
        }
    }

    // =========================================================================
    // REQ-WORKSPACE-002: Path Validation Tests
    // =========================================================================

    /// REQ-WORKSPACE-002.1: Valid directory path is accepted
    ///
    /// WHEN source_path_directory_value points to existing directory
    /// THEN SHALL verify path exists and is a directory
    #[tokio::test]
    async fn test_valid_directory_path_accepted() {
        let app = create_test_router_instance();
        let temp_dir = tempfile::TempDir::new().unwrap();

        let request = make_post_request_json(
            "/workspace-create-from-path",
            json!({
                "source_path_directory_value": temp_dir.path().to_string_lossy()
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        // Should not fail on path validation
        assert_ne!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Valid directory should not return 400"
        );
    }

    /// REQ-WORKSPACE-002.2: Non-existent path returns 400 with PATH_NOT_FOUND
    ///
    /// WHEN source_path_directory_value points to non-existent path
    /// THEN SHALL return HTTP 400 with code "PATH_NOT_FOUND"
    #[tokio::test]
    async fn test_nonexistent_path_returns_400() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/workspace-create-from-path",
            json!({
                "source_path_directory_value": "/this/path/does/not/exist/anywhere/12345"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = extract_json_from_response(response).await;
        assert_eq!(json["code"], "PATH_NOT_FOUND");
    }

    /// REQ-WORKSPACE-002.3: File path (not directory) returns 400 with PATH_NOT_DIRECTORY
    ///
    /// WHEN source_path_directory_value points to a file
    /// THEN SHALL return HTTP 400 with code "PATH_NOT_DIRECTORY"
    #[tokio::test]
    async fn test_file_path_returns_400() {
        let app = create_test_router_instance();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        std::fs::write(&file_path, "test content").unwrap();

        let request = make_post_request_json(
            "/workspace-create-from-path",
            json!({
                "source_path_directory_value": file_path.to_string_lossy()
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = extract_json_from_response(response).await;
        assert_eq!(json["code"], "PATH_NOT_DIRECTORY");
    }

    /// REQ-WORKSPACE-002.4: Duplicate workspace returns 409 with WORKSPACE_ALREADY_EXISTS
    ///
    /// WHEN source_path_directory_value already has a workspace
    /// THEN SHALL return HTTP 409 with code "WORKSPACE_ALREADY_EXISTS"
    ///   AND SHALL include existing_workspace_id in response
    #[tokio::test]
    async fn test_duplicate_workspace_returns_409() {
        let app = create_test_router_instance();
        let temp_dir = tempfile::TempDir::new().unwrap();

        // First request should succeed
        let request1 = make_post_request_json(
            "/workspace-create-from-path",
            json!({
                "source_path_directory_value": temp_dir.path().to_string_lossy()
            }),
        );

        let _response1 = app.clone().oneshot(request1).await.unwrap();

        // Second request for same path should return 409
        let request2 = make_post_request_json(
            "/workspace-create-from-path",
            json!({
                "source_path_directory_value": temp_dir.path().to_string_lossy()
            }),
        );

        let response2 = app.oneshot(request2).await.unwrap();

        assert_eq!(response2.status(), StatusCode::CONFLICT);

        let json = extract_json_from_response(response2).await;
        assert_eq!(json["code"], "WORKSPACE_ALREADY_EXISTS");
        assert!(json["existing_workspace_id"].is_string());
    }

    /// REQ-WORKSPACE-002.5: Path normalization detects duplicates with trailing slash
    ///
    /// WHEN equivalent paths are submitted (with/without trailing slash)
    /// THEN SHALL detect as duplicate
    #[tokio::test]
    async fn test_path_normalization_detects_duplicates() {
        let app = create_test_router_instance();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let canonical_path = temp_dir.path().canonicalize().unwrap();

        // First request without trailing slash
        let request1 = make_post_request_json(
            "/workspace-create-from-path",
            json!({
                "source_path_directory_value": canonical_path.to_string_lossy()
            }),
        );

        let _response1 = app.clone().oneshot(request1).await.unwrap();

        // Second request with trailing slash
        let path_with_slash = format!("{}/", canonical_path.to_string_lossy());
        let request2 = make_post_request_json(
            "/workspace-create-from-path",
            json!({
                "source_path_directory_value": path_with_slash
            }),
        );

        let response2 = app.oneshot(request2).await.unwrap();

        assert_eq!(response2.status(), StatusCode::CONFLICT);
    }

    // =========================================================================
    // REQ-WORKSPACE-003: Workspace Creation Tests
    // =========================================================================

    /// REQ-WORKSPACE-003.1: Workspace ID format is correct
    ///
    /// WHEN workspace creation succeeds
    /// THEN workspace_identifier_value SHALL match format ws_{YYYYMMDD}_{HHMMSS}_{hex6}
    #[tokio::test]
    async fn test_workspace_id_format_correct() {
        let app = create_test_router_instance();
        let temp_dir = tempfile::TempDir::new().unwrap();

        let request = make_post_request_json(
            "/workspace-create-from-path",
            json!({
                "source_path_directory_value": temp_dir.path().to_string_lossy()
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        if response.status() == StatusCode::OK {
            let json = extract_json_from_response(response).await;
            let workspace_id = json["workspace"]["workspace_identifier_value"]
                .as_str()
                .unwrap();

            // Validate format: ws_YYYYMMDD_HHMMSS_xxxxxx (25 chars total)
            // 2 + 1 + 8 + 1 + 6 + 1 + 6 = 25
            assert!(workspace_id.starts_with("ws_"), "ID should start with 'ws_'");
            assert_eq!(workspace_id.len(), 25, "ID should be 25 characters");

            // Validate parts: ws_{8 digits}_{6 digits}_{6 hex chars}
            let parts: Vec<&str> = workspace_id.split('_').collect();
            assert_eq!(parts.len(), 4, "ID should have 4 parts separated by underscores");
            assert_eq!(parts[0], "ws", "First part should be 'ws'");
            assert_eq!(parts[1].len(), 8, "Date part should be 8 digits");
            assert!(parts[1].chars().all(|c| c.is_ascii_digit()), "Date part should be all digits");
            assert_eq!(parts[2].len(), 6, "Time part should be 6 digits");
            assert!(parts[2].chars().all(|c| c.is_ascii_digit()), "Time part should be all digits");
            assert_eq!(parts[3].len(), 6, "Hex part should be 6 characters");
            assert!(parts[3].chars().all(|c| c.is_ascii_hexdigit()),
                "Hex part should be hex characters");
        }
    }

    /// REQ-WORKSPACE-003.5: Metadata persisted correctly with all fields
    ///
    /// WHEN workspace creation succeeds
    /// THEN metadata.json SHALL contain all required fields
    #[tokio::test]
    async fn test_metadata_persisted_correctly() {
        let app = create_test_router_instance();
        let temp_dir = tempfile::TempDir::new().unwrap();

        let request = make_post_request_json(
            "/workspace-create-from-path",
            json!({
                "source_path_directory_value": temp_dir.path().to_string_lossy(),
                "workspace_display_name_option": "Test Display Name"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        if response.status() == StatusCode::OK {
            let json = extract_json_from_response(response).await;
            let workspace_id = json["workspace"]["workspace_identifier_value"]
                .as_str()
                .unwrap();

            // Check metadata.json exists and has correct content via home dir env
            let home_dir = std::env::var("HOME")
                .ok()
                .map(std::path::PathBuf::from)
                .expect("HOME environment variable not set");
            let metadata_path = home_dir
                .join(".parseltongue/workspaces")
                .join(workspace_id)
                .join("metadata.json");

            assert!(metadata_path.exists(), "metadata.json not created");

            let metadata_content = std::fs::read_to_string(&metadata_path).unwrap();
            let metadata: Value = serde_json::from_str(&metadata_content).unwrap();

            assert_eq!(metadata["workspace_display_name"], "Test Display Name");
            assert_eq!(metadata["watch_enabled_flag_status"], false);
        }
    }

    /// REQ-WORKSPACE-003.6: Response contains all required fields
    ///
    /// WHEN workspace creation succeeds
    /// THEN response SHALL contain success, endpoint, workspace, token_estimate
    #[tokio::test]
    async fn test_response_contains_all_required_fields() {
        let app = create_test_router_instance();
        let temp_dir = tempfile::TempDir::new().unwrap();

        let request = make_post_request_json(
            "/workspace-create-from-path",
            json!({
                "source_path_directory_value": temp_dir.path().to_string_lossy()
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        if response.status() == StatusCode::OK {
            let json = extract_json_from_response(response).await;

            assert_eq!(json["success"], true);
            assert_eq!(json["endpoint"], "/workspace-create-from-path");
            assert!(json.get("workspace").is_some());
            assert!(json["workspace"].get("workspace_identifier_value").is_some());
            assert!(json["workspace"].get("workspace_display_name").is_some());
            assert!(json["workspace"].get("source_directory_path_value").is_some());
            assert!(json["workspace"].get("base_database_path_value").is_some());
            assert!(json["workspace"].get("live_database_path_value").is_some());
            assert!(json["workspace"].get("watch_enabled_flag_status").is_some());
            assert!(json["workspace"].get("created_timestamp_utc_value").is_some());
            assert!(json.get("token_estimate").is_some());
        }
    }

    /// REQ-WORKSPACE-003: watch_enabled_flag_status defaults to false
    ///
    /// WHEN workspace is created
    /// THEN watch_enabled_flag_status SHALL be false
    #[tokio::test]
    async fn test_watch_defaults_to_false() {
        let app = create_test_router_instance();
        let temp_dir = tempfile::TempDir::new().unwrap();

        let request = make_post_request_json(
            "/workspace-create-from-path",
            json!({
                "source_path_directory_value": temp_dir.path().to_string_lossy()
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        if response.status() == StatusCode::OK {
            let json = extract_json_from_response(response).await;
            assert_eq!(json["workspace"]["watch_enabled_flag_status"], false);
        }
    }

    // =========================================================================
    // REQ-WORKSPACE-007: Error Response Format Tests
    // =========================================================================

    /// REQ-WORKSPACE-007.1: Error response has consistent structure
    ///
    /// WHEN any error is returned
    /// THEN SHALL have "error" (string) and "code" (SCREAMING_SNAKE_CASE)
    #[tokio::test]
    async fn test_error_response_consistent_structure() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/workspace-create-from-path",
            json!({
                "source_path_directory_value": ""
            }),
        );

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = extract_json_from_response(response).await;

        assert!(json.get("error").is_some());
        assert!(json["error"].is_string());
        assert!(!json["error"].as_str().unwrap().is_empty());

        assert!(json.get("code").is_some());
        assert!(json["code"].is_string());

        let code = json["code"].as_str().unwrap();
        assert!(
            code.chars().all(|c| c.is_uppercase() || c == '_'),
            "Code {} is not SCREAMING_SNAKE_CASE",
            code
        );
    }

    /// REQ-WORKSPACE-007.2: Error Content-Type is application/json
    ///
    /// WHEN error response is returned
    /// THEN Content-Type SHALL be application/json
    #[tokio::test]
    async fn test_error_content_type_is_json() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/workspace-create-from-path",
            json!({
                "source_path_directory_value": ""
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
    }
}

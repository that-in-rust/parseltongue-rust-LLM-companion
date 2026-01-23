# Implementation Specification: Workspace Watch Toggle Endpoint

## REQ-WATCH-TOGGLE-IMPL: Watch Toggle Handler and Route Registration

**Document Version**: 1.0.0
**Created**: 2026-01-23
**Status**: Implementation Ready
**Phase**: 2.1 - Workspace Management Backend
**Dependencies**: REQ-WORKSPACE-005, REQ-WORKSPACE-006, REQ-WORKSPACE-007

---

## Problem Statement

The workspace-watch-toggle endpoint exists as a STUB returning 501 Not Implemented. There are 12 ignored tests waiting for implementation. The route is not registered in the route builder. Developers cannot enable/disable file watching for workspaces, blocking real-time diff visualization in Phase 2.2.

### Current State
- `workspace_watch_handler.rs` contains STUB handler returning 501
- 5 helper functions marked `todo!()`:
  - `validate_watch_toggle_request`
  - `find_workspace_by_identifier`
  - `start_file_watcher_workspace`
  - `stop_file_watcher_workspace`
  - `update_workspace_watch_flag`
- Route `/workspace-watch-toggle` not registered in `route_definition_builder_module.rs`
- 12 tests in handler file marked `#[ignore]`

### Target State
- Fully implemented handler with request validation
- All helper functions implemented
- Route registered with POST method
- All 12 tests passing (no `#[ignore]`)

---

## Section 1: Request Validation Contracts

### REQ-WATCH-IMPL-001: Empty Workspace ID Validation

```
WHEN client sends POST to /workspace-watch-toggle
  WITH Content-Type: application/json
  AND body containing:
    {
      "workspace_identifier_target_value": "",
      "watch_enabled_desired_state": true
    }
THEN SHALL call validate_watch_toggle_request()
  AND SHALL detect empty workspace_identifier_target_value (length == 0)
  AND SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body:
    {
      "error": "workspace_identifier_target_value cannot be empty",
      "code": "INVALID_WORKSPACE_ID_EMPTY"
    }
  AND SHALL complete within 10ms
  AND SHALL NOT access SharedApplicationStateContainer
  AND SHALL NOT attempt workspace lookup
```

### REQ-WATCH-IMPL-002: Missing Workspace ID Field (Deserialization)

```
WHEN client sends POST to /workspace-watch-toggle
  WITH Content-Type: application/json
  AND body missing "workspace_identifier_target_value" field:
    {
      "watch_enabled_desired_state": true
    }
THEN Axum's Json extractor SHALL fail deserialization
  AND SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body with:
    {
      "error": "Missing required field: workspace_identifier_target_value",
      "code": "MISSING_WORKSPACE_ID"
    }
  AND SHALL NOT invoke handle_workspace_watch_toggle_state
```

**Implementation Note**: Requires custom Json extractor rejection handler or Option<String> with explicit validation.

### REQ-WATCH-IMPL-003: Missing Watch State Field (Deserialization)

```
WHEN client sends POST to /workspace-watch-toggle
  WITH Content-Type: application/json
  AND body missing "watch_enabled_desired_state" field:
    {
      "workspace_identifier_target_value": "ws_20260123_143052_a1b2c3"
    }
THEN Axum's Json extractor SHALL fail deserialization
  AND SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body with:
    {
      "error": "Missing required field: watch_enabled_desired_state",
      "code": "MISSING_WATCH_STATE"
    }
```

**Implementation Note**: Since `watch_enabled_desired_state` is `bool`, serde will fail if missing. Custom error mapping required.

---

## Section 2: Workspace Lookup Contracts

### REQ-WATCH-IMPL-004: Workspace Not Found Returns 404

```
WHEN client sends POST to /workspace-watch-toggle
  WITH valid request body:
    {
      "workspace_identifier_target_value": "ws_nonexistent_12345_abcdef",
      "watch_enabled_desired_state": true
    }
  AND workspace_identifier_target_value does NOT exist in SharedApplicationStateContainer.workspaces
THEN SHALL call find_workspace_by_identifier("ws_nonexistent_12345_abcdef")
  AND find_workspace_by_identifier SHALL return None
  AND SHALL return HTTP 404 Not Found
  AND SHALL return JSON body:
    {
      "error": "Workspace not found: ws_nonexistent_12345_abcdef",
      "code": "WORKSPACE_NOT_FOUND"
    }
  AND SHALL complete within 50ms
  AND SHALL NOT attempt to start or stop watcher
```

### REQ-WATCH-IMPL-005: Workspace Found Proceeds to Toggle

```
WHEN client sends POST to /workspace-watch-toggle
  WITH valid request body containing existing workspace ID
  AND workspace exists in SharedApplicationStateContainer.workspaces
THEN SHALL call find_workspace_by_identifier(workspace_id)
  AND find_workspace_by_identifier SHALL return Some(WorkspaceMetadataPayloadStruct)
  AND SHALL proceed to watch state toggle logic
```

---

## Section 3: Watch State Toggle Contracts

### REQ-WATCH-IMPL-006: Enable Watch from Disabled State

```
WHEN client sends POST to /workspace-watch-toggle
  WITH watch_enabled_desired_state: true
  AND workspace.watch_enabled_flag_status == false
THEN SHALL call start_file_watcher_workspace(&workspace)
  AND start_file_watcher_workspace SHALL:
    - Create notify::RecommendedWatcher instance
    - Configure watcher for workspace.source_directory_path_value
    - Register watcher handle in SharedApplicationStateContainer.watchers
  AND SHALL call update_workspace_watch_flag(&mut workspace, true)
  AND update_workspace_watch_flag SHALL:
    - Set workspace.watch_enabled_flag_status = true
    - Write updated metadata to ~/.parseltongue/workspaces/{id}/metadata.json
  AND SHALL return HTTP 200 OK
  AND SHALL return JSON body:
    {
      "success": true,
      "endpoint": "/workspace-watch-toggle",
      "workspace": { ...updated workspace with watch_enabled_flag_status: true... },
      "token_estimate": 180
    }
```

### REQ-WATCH-IMPL-007: Disable Watch from Enabled State

```
WHEN client sends POST to /workspace-watch-toggle
  WITH watch_enabled_desired_state: false
  AND workspace.watch_enabled_flag_status == true
THEN SHALL call stop_file_watcher_workspace(&workspace_id)
  AND stop_file_watcher_workspace SHALL:
    - Retrieve watcher handle from SharedApplicationStateContainer.watchers
    - Drop watcher handle to stop watching
    - Remove entry from SharedApplicationStateContainer.watchers
  AND SHALL call update_workspace_watch_flag(&mut workspace, false)
  AND update_workspace_watch_flag SHALL:
    - Set workspace.watch_enabled_flag_status = false
    - Write updated metadata to disk
  AND SHALL return HTTP 200 OK
  AND SHALL return JSON body with watch_enabled_flag_status: false
```

### REQ-WATCH-IMPL-008: Enable When Already Enabled (Idempotent)

```
WHEN client sends POST to /workspace-watch-toggle
  WITH watch_enabled_desired_state: true
  AND workspace.watch_enabled_flag_status == true (already enabled)
THEN SHALL detect idempotent operation (desired == current)
  AND SHALL NOT call start_file_watcher_workspace (avoid duplicate watcher)
  AND SHALL NOT call update_workspace_watch_flag (no change needed)
  AND SHALL return HTTP 200 OK
  AND SHALL return JSON body with current workspace state
  AND response.workspace.watch_enabled_flag_status SHALL equal true
```

### REQ-WATCH-IMPL-009: Disable When Already Disabled (Idempotent)

```
WHEN client sends POST to /workspace-watch-toggle
  WITH watch_enabled_desired_state: false
  AND workspace.watch_enabled_flag_status == false (already disabled)
THEN SHALL detect idempotent operation (desired == current)
  AND SHALL NOT call stop_file_watcher_workspace (no watcher to stop)
  AND SHALL NOT call update_workspace_watch_flag (no change needed)
  AND SHALL return HTTP 200 OK
  AND SHALL return JSON body with current workspace state
  AND response.workspace.watch_enabled_flag_status SHALL equal false
```

---

## Section 4: Error Handling Contracts

### REQ-WATCH-IMPL-010: Watcher Start Failure

```
WHEN client sends POST to /workspace-watch-toggle
  WITH watch_enabled_desired_state: true
  AND start_file_watcher_workspace fails (e.g., permission denied, inotify limit)
THEN SHALL catch error from notify crate
  AND SHALL return HTTP 500 Internal Server Error
  AND SHALL return JSON body:
    {
      "error": "Failed to start file watcher: {detailed_error_message}",
      "code": "WATCHER_START_FAILED"
    }
  AND SHALL NOT update workspace.watch_enabled_flag_status
  AND SHALL NOT persist metadata changes
  AND SHALL log error at ERROR level
```

### REQ-WATCH-IMPL-011: Watcher Stop Failure

```
WHEN client sends POST to /workspace-watch-toggle
  WITH watch_enabled_desired_state: false
  AND stop_file_watcher_workspace fails (e.g., watcher handle already dropped)
THEN SHALL catch error
  AND SHALL return HTTP 500 Internal Server Error
  AND SHALL return JSON body:
    {
      "error": "Failed to stop file watcher: {detailed_error_message}",
      "code": "WATCHER_STOP_FAILED"
    }
  AND SHALL NOT update workspace.watch_enabled_flag_status
  AND SHALL log error at ERROR level
```

### REQ-WATCH-IMPL-012: Storage Write Failure

```
WHEN update_workspace_watch_flag attempts to persist metadata
  AND filesystem write to metadata.json fails
THEN SHALL catch std::io::Error
  AND SHALL return HTTP 500 Internal Server Error
  AND SHALL return JSON body:
    {
      "error": "Failed to persist workspace metadata: {io_error_message}",
      "code": "STORAGE_WRITE_FAILED"
    }
  AND SHALL attempt to rollback watch state change if possible
```

---

## Section 5: Response Format Contracts

### REQ-WATCH-IMPL-013: Successful Response Structure

```
WHEN watch toggle operation completes successfully
THEN SHALL return HTTP 200 OK
  AND SHALL set Content-Type: application/json
  AND SHALL return JSON body matching WorkspaceOperationResponsePayloadStruct:
    {
      "success": true,
      "endpoint": "/workspace-watch-toggle",
      "workspace": {
        "workspace_identifier_value": "<string matching ws_YYYYMMDD_HHMMSS_XXXXXX>",
        "workspace_display_name": "<string>",
        "source_directory_path_value": "<absolute path string>",
        "base_database_path_value": "<rocksdb: prefixed path>",
        "live_database_path_value": "<rocksdb: prefixed path>",
        "watch_enabled_flag_status": <boolean matching requested state>,
        "created_timestamp_utc_value": "<ISO 8601 timestamp>",
        "last_indexed_timestamp_option": <ISO 8601 timestamp or null>
      },
      "token_estimate": <positive integer>
    }
  AND workspace.watch_enabled_flag_status SHALL match watch_enabled_desired_state from request
```

### REQ-WATCH-IMPL-014: Token Estimation Formula

```
WHEN calculating token_estimate for toggle response
THEN SHALL use formula:
  token_estimate = BASE_TOKENS + WORKSPACE_TOKENS
  WHERE BASE_TOKENS = 100
  AND WORKSPACE_TOKENS = 80
  RESULTING IN token_estimate = 180 for single workspace response
```

---

## Section 6: Route Registration Contracts

### REQ-WATCH-IMPL-015: Watch Toggle Route Registration

```
WHEN route_definition_builder_module.rs builds complete router
THEN SHALL include route registration:
  .route(
    "/workspace-watch-toggle",
    post(workspace_watch_handler::handle_workspace_watch_toggle_state)
  )
  AND route SHALL use POST method (not GET, PUT, DELETE)
  AND route SHALL be connected to handle_workspace_watch_toggle_state function
  AND route SHALL receive SharedApplicationStateContainer via State extractor
```

### REQ-WATCH-IMPL-016: All Workspace Routes Registered

```
WHEN route_definition_builder_module.rs builds complete router
THEN SHALL include all three workspace routes:
  1. POST /workspace-create-from-path -> workspace_create_handler::handle_workspace_create_from_path
  2. GET  /workspace-list-all         -> workspace_list_handler::handle_workspace_list_all_entries
  3. POST /workspace-watch-toggle     -> workspace_watch_handler::handle_workspace_watch_toggle_state
  AND all routes SHALL share the same SharedApplicationStateContainer state
```

### REQ-WATCH-IMPL-017: Handler Module Imports

```
WHEN route_definition_builder_module.rs compiles
THEN SHALL have import for workspace_watch_handler:
  use crate::http_endpoint_handler_modules::workspace_watch_handler;
  OR
  use crate::http_endpoint_handler_modules::{
    ...,
    workspace_watch_handler,
  };
```

---

## Section 7: Performance Contracts

### Performance Targets

| Operation | Target | P99 Target | Measurement Method |
|-----------|--------|------------|-------------------|
| Request validation | < 5ms | < 10ms | Start of handler to validation complete |
| Workspace lookup | < 20ms | < 50ms | HashMap lookup in Arc<RwLock<...>> |
| Watcher start | < 500ms | < 1000ms | notify crate initialization |
| Watcher stop | < 100ms | < 200ms | Drop watcher handle |
| Metadata persistence | < 50ms | < 100ms | Filesystem write |
| Total request (state change) | < 800ms | < 1500ms | End-to-end timing |
| Total request (idempotent) | < 50ms | < 100ms | No state change path |

---

## Section 8: Implementation Steps

### Step 1: Implement validate_watch_toggle_request Function

**File**: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/workspace_watch_handler.rs`

**4-Word Name**: `validate_watch_toggle_request`

```rust
/// Validate watch toggle request body fields
///
/// # 4-Word Name: validate_watch_toggle_request
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
```

### Step 2: Implement Handler Logic

**4-Word Name**: `handle_workspace_watch_toggle_state`

Replace STUB with:
1. Call `validate_watch_toggle_request(&body)`
2. Look up workspace in state using `find_workspace_by_identifier`
3. Check if state change needed (idempotent check)
4. If change needed: start/stop watcher, update flag, persist
5. Return success response with updated workspace

### Step 3: Implement find_workspace_by_identifier

**4-Word Name**: `find_workspace_by_identifier`

```rust
/// Find workspace by identifier value in shared state
///
/// # 4-Word Name: find_workspace_by_identifier
fn find_workspace_by_identifier(
    state: &SharedApplicationStateContainer,
    workspace_id: &str,
) -> Option<WorkspaceMetadataPayloadStruct> {
    let workspaces = state.workspace_registry_map.read().unwrap();
    workspaces.get(workspace_id).cloned()
}
```

### Step 4: Implement start_file_watcher_workspace

**4-Word Name**: `start_file_watcher_workspace`

```rust
/// Start file watcher for workspace using notify crate
///
/// # 4-Word Name: start_file_watcher_workspace
fn start_file_watcher_workspace(
    state: &SharedApplicationStateContainer,
    workspace: &WorkspaceMetadataPayloadStruct,
) -> Result<(), WorkspaceOperationErrorType> {
    // Create notify watcher
    // Watch workspace.source_directory_path_value recursively
    // Store handle in state.watcher_handles_map
}
```

### Step 5: Implement stop_file_watcher_workspace

**4-Word Name**: `stop_file_watcher_workspace`

```rust
/// Stop file watcher for workspace
///
/// # 4-Word Name: stop_file_watcher_workspace
fn stop_file_watcher_workspace(
    state: &SharedApplicationStateContainer,
    workspace_id: &str,
) -> Result<(), WorkspaceOperationErrorType> {
    // Remove handle from state.watcher_handles_map
    // Handle will be dropped, stopping watcher
}
```

### Step 6: Implement update_workspace_watch_flag

**4-Word Name**: `update_workspace_watch_flag`

```rust
/// Update workspace watch flag and persist to disk
///
/// # 4-Word Name: update_workspace_watch_flag
fn update_workspace_watch_flag(
    state: &SharedApplicationStateContainer,
    workspace_id: &str,
    enabled: bool,
) -> Result<WorkspaceMetadataPayloadStruct, WorkspaceOperationErrorType> {
    // Update in-memory state
    // Write metadata.json to disk
    // Return updated workspace
}
```

### Step 7: Register Routes

**File**: `crates/pt08-http-code-query-server/src/route_definition_builder_module.rs`

Add to imports:
```rust
use crate::http_endpoint_handler_modules::{
    // ... existing imports ...
    workspace_create_handler,
    workspace_list_handler,
    workspace_watch_handler,
};
```

Add routes after existing routes:
```rust
// Phase 2.1: Workspace Management Endpoints
.route(
    "/workspace-create-from-path",
    post(workspace_create_handler::handle_workspace_create_from_path)
)
.route(
    "/workspace-list-all",
    get(workspace_list_handler::handle_workspace_list_all_entries)
)
.route(
    "/workspace-watch-toggle",
    post(workspace_watch_handler::handle_workspace_watch_toggle_state)
)
```

### Step 8: Update mod.rs (Already Done)

**File**: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/mod.rs`

Already contains:
```rust
pub mod workspace_watch_handler;
```

---

## Section 9: Verification Test Templates

### Test: Empty Workspace ID Validation

```rust
/// REQ-WATCH-IMPL-001: Empty workspace ID returns 400
#[tokio::test]
async fn test_empty_workspace_id_returns_400_invalid_workspace_id_empty() {
    // GIVEN a test router with workspace state
    let app = create_test_router_instance();

    // WHEN client sends request with empty workspace ID
    let request = make_post_request_json(
        "/workspace-watch-toggle",
        json!({
            "workspace_identifier_target_value": "",
            "watch_enabled_desired_state": true
        }),
    );

    let response = app.oneshot(request).await.unwrap();

    // THEN should return 400 with INVALID_WORKSPACE_ID_EMPTY
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let json = extract_json_from_response(response).await;
    assert_eq!(json["code"], "INVALID_WORKSPACE_ID_EMPTY");
    assert!(json["error"].as_str().unwrap().contains("cannot be empty"));
}
```

### Test: Workspace Not Found

```rust
/// REQ-WATCH-IMPL-004: Non-existent workspace returns 404
#[tokio::test]
async fn test_nonexistent_workspace_returns_404_workspace_not_found() {
    // GIVEN a test router with no workspaces
    let app = create_test_router_instance();

    // WHEN client sends request for non-existent workspace
    let request = make_post_request_json(
        "/workspace-watch-toggle",
        json!({
            "workspace_identifier_target_value": "ws_nonexistent_12345_abcdef",
            "watch_enabled_desired_state": true
        }),
    );

    let response = app.oneshot(request).await.unwrap();

    // THEN should return 404 with WORKSPACE_NOT_FOUND
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let json = extract_json_from_response(response).await;
    assert_eq!(json["code"], "WORKSPACE_NOT_FOUND");
}
```

### Test: Enable Watch Success

```rust
/// REQ-WATCH-IMPL-006: Enable watch from disabled state
#[tokio::test]
async fn test_enable_watch_from_disabled_returns_200_with_enabled_flag() {
    // GIVEN a router with workspace having watch disabled
    let (app, workspace_id) = create_router_with_workspace_watch_disabled();

    // WHEN client enables watch
    let request = make_post_request_json(
        "/workspace-watch-toggle",
        json!({
            "workspace_identifier_target_value": workspace_id,
            "watch_enabled_desired_state": true
        }),
    );

    let response = app.oneshot(request).await.unwrap();

    // THEN should return 200 with watch enabled
    assert_eq!(response.status(), StatusCode::OK);

    let json = extract_json_from_response(response).await;
    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/workspace-watch-toggle");
    assert_eq!(json["workspace"]["watch_enabled_flag_status"], true);
    assert!(json["token_estimate"].as_u64().unwrap() >= 180);
}
```

### Test: Idempotent Enable

```rust
/// REQ-WATCH-IMPL-008: Enable when already enabled is idempotent
#[tokio::test]
async fn test_enable_watch_when_already_enabled_is_idempotent() {
    // GIVEN a router with workspace having watch already enabled
    let (app, workspace_id) = create_router_with_workspace_watch_enabled();

    // WHEN client enables watch again
    let request = make_post_request_json(
        "/workspace-watch-toggle",
        json!({
            "workspace_identifier_target_value": workspace_id,
            "watch_enabled_desired_state": true
        }),
    );

    let response = app.oneshot(request).await.unwrap();

    // THEN should return 200 (idempotent success)
    assert_eq!(response.status(), StatusCode::OK);

    let json = extract_json_from_response(response).await;
    assert_eq!(json["success"], true);
    assert_eq!(json["workspace"]["watch_enabled_flag_status"], true);
}
```

### Test: Route Registration

```rust
/// REQ-WATCH-IMPL-015: Watch toggle route is registered
#[tokio::test]
async fn test_workspace_watch_toggle_route_is_registered() {
    // GIVEN the full router built by route_definition_builder
    let state = SharedApplicationStateContainer::create_new_application_state();
    let app = build_complete_router_instance(state);

    // WHEN client sends POST to /workspace-watch-toggle
    let request = Request::builder()
        .method("POST")
        .uri("/workspace-watch-toggle")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({
            "workspace_identifier_target_value": "ws_test",
            "watch_enabled_desired_state": true
        }).to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // THEN should NOT return 404 (route exists)
    assert_ne!(response.status(), StatusCode::NOT_FOUND,
        "Route /workspace-watch-toggle should be registered");
}
```

---

## Section 10: Acceptance Criteria Checklist

### Request Validation
- [ ] Empty workspace_identifier_target_value returns 400 with INVALID_WORKSPACE_ID_EMPTY
- [ ] Missing workspace_identifier_target_value returns 400 with MISSING_WORKSPACE_ID
- [ ] Missing watch_enabled_desired_state returns 400 with MISSING_WATCH_STATE
- [ ] Valid request proceeds to workspace lookup

### Workspace Lookup
- [ ] Non-existent workspace returns 404 with WORKSPACE_NOT_FOUND
- [ ] Existing workspace proceeds to toggle logic

### Watch State Toggle
- [ ] Enable from disabled starts watcher and returns 200
- [ ] Disable from enabled stops watcher and returns 200
- [ ] Enable when already enabled is idempotent (200, no duplicate watcher)
- [ ] Disable when already disabled is idempotent (200, no error)

### Error Handling
- [ ] Watcher start failure returns 500 with WATCHER_START_FAILED
- [ ] Watcher stop failure returns 500 with WATCHER_STOP_FAILED
- [ ] Storage write failure returns 500 with STORAGE_WRITE_FAILED

### Response Format
- [ ] Success response contains success, endpoint, workspace, token_estimate
- [ ] workspace.watch_enabled_flag_status matches requested state
- [ ] Error response contains error and code fields
- [ ] Error code is SCREAMING_SNAKE_CASE

### Route Registration
- [ ] POST /workspace-watch-toggle route is registered
- [ ] POST /workspace-create-from-path route is registered
- [ ] GET /workspace-list-all route is registered
- [ ] All routes use correct HTTP methods

### Performance
- [ ] Total request time < 1500ms at p99
- [ ] Idempotent request time < 100ms at p99
- [ ] Validation time < 10ms

---

## Section 11: Traceability Matrix

| Implementation Requirement | Parent Specification | Test Count | Functions |
|---------------------------|---------------------|------------|-----------|
| REQ-WATCH-IMPL-001 | REQ-WORKSPACE-005.3 | 1 | validate_watch_toggle_request |
| REQ-WATCH-IMPL-002 | REQ-WORKSPACE-005.2 | 1 | Axum Json extractor |
| REQ-WATCH-IMPL-003 | REQ-WORKSPACE-005.4 | 1 | Axum Json extractor |
| REQ-WATCH-IMPL-004 | REQ-WORKSPACE-005.5 | 1 | find_workspace_by_identifier |
| REQ-WATCH-IMPL-005 | REQ-WORKSPACE-005.1 | 1 | find_workspace_by_identifier |
| REQ-WATCH-IMPL-006 | REQ-WORKSPACE-006.1 | 1 | start_file_watcher_workspace, update_workspace_watch_flag |
| REQ-WATCH-IMPL-007 | REQ-WORKSPACE-006.2 | 1 | stop_file_watcher_workspace, update_workspace_watch_flag |
| REQ-WATCH-IMPL-008 | REQ-WORKSPACE-006.3 | 1 | Idempotent check in handler |
| REQ-WATCH-IMPL-009 | REQ-WORKSPACE-006.4 | 1 | Idempotent check in handler |
| REQ-WATCH-IMPL-010 | REQ-WORKSPACE-006.5 | 1 | start_file_watcher_workspace error handling |
| REQ-WATCH-IMPL-011 | REQ-WORKSPACE-006.6 | 1 | stop_file_watcher_workspace error handling |
| REQ-WATCH-IMPL-012 | REQ-WORKSPACE-007 | 1 | update_workspace_watch_flag error handling |
| REQ-WATCH-IMPL-013 | REQ-WORKSPACE-006.7 | 1 | Response construction |
| REQ-WATCH-IMPL-014 | REQ-WORKSPACE-006.7 | 0 | calculate_toggle_token_estimate |
| REQ-WATCH-IMPL-015 | N/A | 1 | route_definition_builder_module |
| REQ-WATCH-IMPL-016 | N/A | 1 | route_definition_builder_module |
| REQ-WATCH-IMPL-017 | N/A | 0 | Compile-time check |
| **Total** | | **15** | **7 functions** |

---

## Quality Checklist

Before implementation is complete, verify:

- [ ] All quantities are specific and measurable (ms, counts)
- [ ] All behaviors are testable with provided templates
- [ ] Error conditions are specified with exact codes
- [ ] Performance boundaries are defined with p99 targets
- [ ] Test templates provided for all 15 requirements
- [ ] Acceptance criteria are binary (pass/fail)
- [ ] No ambiguous language remains
- [ ] 4-word naming convention followed for all 7 functions
- [ ] Token estimation formula documented
- [ ] Idempotent operations handled correctly
- [ ] Route registration includes all 3 workspace endpoints

---

*Implementation specification created 2026-01-23*
*Target: Remove all 12 #[ignore] attributes, implement 7 functions, register 3 routes*
*Test target: 15 tests for watch-toggle + route registration verification*

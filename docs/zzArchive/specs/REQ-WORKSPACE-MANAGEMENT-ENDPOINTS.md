# HTTP Endpoint Specification: Workspace Management Endpoints

## Phase 2.1 - Workspace Management System

**Document Version**: 1.0.0
**Created**: 2026-01-23
**Status**: Specification Complete
**Phase**: 2.1 - Workspace Management Backend

---

## Overview

### Problem Statement

Developers using Parseltongue need a way to manage multiple code analysis workspaces that persist across sessions. Currently, users must manually track database paths and restart ingestion for each codebase. This creates friction when:

1. Switching between projects
2. Enabling real-time file watching for live diff updates
3. Managing multiple codebases simultaneously

### Solution

Three HTTP endpoints for workspace lifecycle management:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/workspace-create-from-path` | POST | Create new workspace from directory path |
| `/workspace-list-all` | GET | List all registered workspaces |
| `/workspace-watch-toggle` | POST | Enable/disable file watching for a workspace |

These endpoints integrate with the existing pt08 HTTP server and prepare infrastructure for WebSocket streaming in Phase 2.2.

### Storage Strategy

Workspaces are persisted in `~/.parseltongue/workspaces/`:

```
~/.parseltongue/
  workspaces/
    {workspace_id}/
      metadata.json          # WorkspaceMetadataPayloadStruct
      base.db/               # RocksDB base snapshot
      live.db/               # RocksDB live snapshot
```

---

## Data Types

### WorkspaceMetadataPayloadStruct

```rust
/// Workspace metadata stored on disk
/// # 4-Word Name: WorkspaceMetadataPayloadStruct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetadataPayloadStruct {
    pub workspace_identifier_value: String,
    pub workspace_display_name: String,
    pub source_directory_path_value: PathBuf,
    pub base_database_path_value: String,
    pub live_database_path_value: String,
    pub watch_enabled_flag_status: bool,
    pub created_timestamp_utc_value: DateTime<Utc>,
    pub last_indexed_timestamp_option: Option<DateTime<Utc>>,
}
```

### Request/Response Types

```rust
/// Request body for workspace creation
/// # 4-Word Name: WorkspaceCreateRequestBodyStruct
#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceCreateRequestBodyStruct {
    pub source_path_directory_value: String,
    pub workspace_display_name_option: Option<String>,
}

/// Response for single workspace operations
/// # 4-Word Name: WorkspaceOperationResponsePayloadStruct
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceOperationResponsePayloadStruct {
    pub success: bool,
    pub endpoint: String,
    pub workspace: WorkspaceMetadataPayloadStruct,
    pub token_estimate: usize,
}

/// Response for workspace listing
/// # 4-Word Name: WorkspaceListResponsePayloadStruct
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceListResponsePayloadStruct {
    pub success: bool,
    pub endpoint: String,
    pub workspaces: Vec<WorkspaceMetadataPayloadStruct>,
    pub total_workspace_count_value: usize,
    pub token_estimate: usize,
}

/// Request for watch toggle
/// # 4-Word Name: WorkspaceWatchToggleRequestStruct
#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceWatchToggleRequestStruct {
    pub workspace_identifier_target_value: String,
    pub watch_enabled_desired_state: bool,
}
```

---

## Error Codes Reference

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `MISSING_SOURCE_PATH` | 400 | Request missing source_path_directory_value |
| `INVALID_SOURCE_PATH_EMPTY` | 400 | source_path_directory_value is empty string |
| `PATH_NOT_FOUND` | 400 | Source path does not exist on filesystem |
| `PATH_NOT_DIRECTORY` | 400 | Source path exists but is not a directory |
| `WORKSPACE_ALREADY_EXISTS` | 409 | Workspace already registered for this path |
| `INDEXING_FAILED` | 500 | pt01 streamer failed during initial indexing |
| `WORKSPACE_NOT_FOUND` | 404 | Workspace ID does not exist |
| `MISSING_WORKSPACE_ID` | 400 | Request missing workspace_identifier_target_value |
| `INVALID_WORKSPACE_ID_EMPTY` | 400 | workspace_identifier_target_value is empty |
| `MISSING_WATCH_STATE` | 400 | Request missing watch_enabled_desired_state |
| `WATCHER_START_FAILED` | 500 | Failed to start file watcher (notify crate error) |
| `WATCHER_STOP_FAILED` | 500 | Failed to stop file watcher |
| `STORAGE_WRITE_FAILED` | 500 | Failed to write workspace metadata to disk |
| `STORAGE_READ_FAILED` | 500 | Failed to read workspace data from disk |
| `INVALID_JSON` | 400 | Malformed JSON in request body |
| `UNSUPPORTED_MEDIA_TYPE` | 415 | Content-Type is not application/json |
| `INTERNAL_ERROR` | 500 | Unexpected internal error |

---

# Endpoint 1: POST /workspace-create-from-path

## REQ-WORKSPACE-001: Request Parsing and Validation

### Problem Statement

The workspace creation endpoint must correctly parse incoming JSON request bodies, validate the source path, and reject invalid requests before attempting any filesystem or database operations.

### Specification

#### REQ-WORKSPACE-001.1: Valid Request Body Parsing

```
WHEN client sends POST to /workspace-create-from-path
  WITH Content-Type: application/json
  AND body containing valid JSON:
    {
      "source_path_directory_value": "/path/to/codebase",
      "workspace_display_name_option": "My Project"
    }
THEN SHALL parse request body successfully
  AND SHALL extract source_path_directory_value as String
  AND SHALL extract workspace_display_name_option as Option<String>
  AND SHALL proceed to path validation phase
  AND SHALL NOT return 400 Bad Request
```

#### REQ-WORKSPACE-001.2: Missing source_path_directory_value Field

```
WHEN client sends POST to /workspace-create-from-path
  WITH Content-Type: application/json
  AND body missing "source_path_directory_value" field:
    {
      "workspace_display_name_option": "My Project"
    }
THEN SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body:
    {
      "error": "Missing required field: source_path_directory_value",
      "code": "MISSING_SOURCE_PATH"
    }
  AND SHALL NOT attempt filesystem access
  AND SHALL complete within 10ms
```

#### REQ-WORKSPACE-001.3: Empty source_path_directory_value Field

```
WHEN client sends POST to /workspace-create-from-path
  WITH Content-Type: application/json
  AND body with empty "source_path_directory_value":
    {
      "source_path_directory_value": "",
      "workspace_display_name_option": "My Project"
    }
THEN SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body:
    {
      "error": "source_path_directory_value cannot be empty",
      "code": "INVALID_SOURCE_PATH_EMPTY"
    }
  AND SHALL NOT attempt filesystem access
  AND SHALL complete within 10ms
```

#### REQ-WORKSPACE-001.4: Optional workspace_display_name_option

```
WHEN client sends POST to /workspace-create-from-path
  WITH Content-Type: application/json
  AND body without "workspace_display_name_option":
    {
      "source_path_directory_value": "/path/to/codebase"
    }
THEN SHALL accept request
  AND SHALL derive display name from directory name (last path component)
  AND SHALL proceed with workspace creation
  AND SHALL NOT return error for missing optional field
```

#### REQ-WORKSPACE-001.5: Invalid JSON Body

```
WHEN client sends POST to /workspace-create-from-path
  WITH Content-Type: application/json
  AND body containing malformed JSON: "{ invalid json"
THEN SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body:
    {
      "error": "Invalid JSON in request body",
      "code": "INVALID_JSON"
    }
  AND SHALL complete within 10ms
```

#### REQ-WORKSPACE-001.6: Missing Content-Type Header

```
WHEN client sends POST to /workspace-create-from-path
  WITH no Content-Type header
  AND valid JSON body
THEN SHALL return HTTP 415 Unsupported Media Type
  AND SHALL return JSON body:
    {
      "error": "Content-Type must be application/json",
      "code": "UNSUPPORTED_MEDIA_TYPE"
    }
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_workspace_001_tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;
    use serde_json::json;

    /// REQ-WORKSPACE-001.1: Valid request body parsing
    #[tokio::test]
    async fn test_valid_request_body_parsing() {
        // GIVEN a configured router with workspace endpoints
        let app = create_test_router_with_workspace_state();
        let temp_dir = create_temp_codebase_directory();

        // WHEN client sends valid POST request
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-create-from-path")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "source_path_directory_value": temp_dir.path().to_string_lossy(),
                "workspace_display_name_option": "Test Project"
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should succeed with 200 OK
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// REQ-WORKSPACE-001.2: Missing source_path returns 400
    #[tokio::test]
    async fn test_missing_source_path_returns_400() {
        // GIVEN a configured router
        let app = create_test_router_with_workspace_state();

        // WHEN client sends request without source_path
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-create-from-path")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "workspace_display_name_option": "Test Project"
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 400 with MISSING_SOURCE_PATH code
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "MISSING_SOURCE_PATH");
    }

    /// REQ-WORKSPACE-001.3: Empty source_path returns 400
    #[tokio::test]
    async fn test_empty_source_path_returns_400() {
        // GIVEN a configured router
        let app = create_test_router_with_workspace_state();

        // WHEN client sends request with empty source_path
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-create-from-path")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "source_path_directory_value": "",
                "workspace_display_name_option": "Test Project"
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 400 with INVALID_SOURCE_PATH_EMPTY code
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "INVALID_SOURCE_PATH_EMPTY");
    }

    /// REQ-WORKSPACE-001.4: Optional display name derives from directory
    #[tokio::test]
    async fn test_optional_display_name_derives_from_directory() {
        // GIVEN a configured router and temp directory named "my-project"
        let app = create_test_router_with_workspace_state();
        let temp_dir = create_temp_codebase_directory_with_name("my-project");

        // WHEN client sends request without display name
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-create-from-path")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "source_path_directory_value": temp_dir.path().to_string_lossy()
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // THEN display name should be derived from directory name
        assert_eq!(json["workspace"]["workspace_display_name"], "my-project");
    }

    /// REQ-WORKSPACE-001.5: Invalid JSON returns 400
    #[tokio::test]
    async fn test_invalid_json_returns_400() {
        // GIVEN a configured router
        let app = create_test_router_with_workspace_state();

        // WHEN client sends malformed JSON
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-create-from-path")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from("{ invalid json"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 400 with INVALID_JSON code
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
```

### Acceptance Criteria

- [ ] Valid JSON with required fields parses successfully
- [ ] Missing source_path_directory_value returns 400 with MISSING_SOURCE_PATH
- [ ] Empty source_path_directory_value returns 400 with INVALID_SOURCE_PATH_EMPTY
- [ ] Missing workspace_display_name_option is accepted (derives from directory)
- [ ] Malformed JSON returns 400 with INVALID_JSON
- [ ] All validation errors complete within 10ms

---

## REQ-WORKSPACE-002: Path Validation and Filesystem Checks

### Problem Statement

Before creating a workspace, the endpoint must validate that the source path exists and is a directory. It must also check for duplicate workspaces pointing to the same path.

### Specification

#### REQ-WORKSPACE-002.1: Path Exists and Is Directory

```
WHEN client sends POST to /workspace-create-from-path
  WITH source_path_directory_value pointing to existing directory
THEN SHALL verify path exists on filesystem
  AND SHALL verify path is a directory (not a file)
  AND SHALL proceed to duplicate check
  AND SHALL complete filesystem checks within 100ms
```

#### REQ-WORKSPACE-002.2: Path Does Not Exist

```
WHEN client sends POST to /workspace-create-from-path
  WITH source_path_directory_value pointing to non-existent path: "/nonexistent/path"
THEN SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body:
    {
      "error": "Source path does not exist: /nonexistent/path",
      "code": "PATH_NOT_FOUND"
    }
  AND SHALL NOT create workspace directory
  AND SHALL complete within 100ms
```

#### REQ-WORKSPACE-002.3: Path Is File Not Directory

```
WHEN client sends POST to /workspace-create-from-path
  WITH source_path_directory_value pointing to a file: "/path/to/file.txt"
THEN SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body:
    {
      "error": "Source path is not a directory: /path/to/file.txt",
      "code": "PATH_NOT_DIRECTORY"
    }
  AND SHALL NOT create workspace directory
```

#### REQ-WORKSPACE-002.4: Duplicate Workspace Detection

```
WHEN client sends POST to /workspace-create-from-path
  WITH source_path_directory_value pointing to path already registered
THEN SHALL return HTTP 409 Conflict
  AND SHALL return JSON body:
    {
      "error": "Workspace already exists for path: /path/to/codebase",
      "code": "WORKSPACE_ALREADY_EXISTS",
      "existing_workspace_id": "ws_20260123_143052_a1b2c3"
    }
  AND SHALL NOT create new workspace
  AND SHALL include existing workspace ID for client reference
```

#### REQ-WORKSPACE-002.5: Path Normalization

```
WHEN client sends POST to /workspace-create-from-path
  WITH source_path_directory_value containing trailing slash: "/path/to/codebase/"
  OR containing relative components: "/path/to/../to/codebase"
THEN SHALL normalize path before duplicate check
  AND SHALL store canonical absolute path in workspace metadata
  AND SHALL treat normalized equivalent paths as duplicates
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_workspace_002_tests {
    use axum::{body::Body, http::{Request, StatusCode, header}};
    use tower::ServiceExt;
    use serde_json::json;
    use tempfile::TempDir;
    use std::fs::File;

    /// REQ-WORKSPACE-002.1: Valid directory path accepted
    #[tokio::test]
    async fn test_valid_directory_path_accepted() {
        // GIVEN a valid directory
        let temp_dir = TempDir::new().unwrap();
        let app = create_test_router_with_workspace_state();

        // WHEN client sends request with valid directory path
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-create-from-path")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "source_path_directory_value": temp_dir.path().to_string_lossy()
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should succeed
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// REQ-WORKSPACE-002.2: Non-existent path returns 400
    #[tokio::test]
    async fn test_nonexistent_path_returns_400() {
        // GIVEN a non-existent path
        let app = create_test_router_with_workspace_state();

        // WHEN client sends request with non-existent path
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-create-from-path")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "source_path_directory_value": "/this/path/does/not/exist/anywhere"
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 400 with PATH_NOT_FOUND
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "PATH_NOT_FOUND");
    }

    /// REQ-WORKSPACE-002.3: File path (not directory) returns 400
    #[tokio::test]
    async fn test_file_path_returns_400() {
        // GIVEN a file path (not directory)
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        File::create(&file_path).unwrap();

        let app = create_test_router_with_workspace_state();

        // WHEN client sends request with file path
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-create-from-path")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "source_path_directory_value": file_path.to_string_lossy()
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 400 with PATH_NOT_DIRECTORY
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "PATH_NOT_DIRECTORY");
    }

    /// REQ-WORKSPACE-002.4: Duplicate workspace returns 409
    #[tokio::test]
    async fn test_duplicate_workspace_returns_409() {
        // GIVEN a directory with existing workspace
        let temp_dir = TempDir::new().unwrap();
        let app = create_test_router_with_workspace_state();

        // First request creates workspace
        let request1 = Request::builder()
            .method("POST")
            .uri("/workspace-create-from-path")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "source_path_directory_value": temp_dir.path().to_string_lossy()
            }).to_string()))
            .unwrap();

        let _ = app.clone().oneshot(request1).await.unwrap();

        // WHEN client sends second request for same path
        let request2 = Request::builder()
            .method("POST")
            .uri("/workspace-create-from-path")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "source_path_directory_value": temp_dir.path().to_string_lossy()
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request2).await.unwrap();

        // THEN should return 409 with WORKSPACE_ALREADY_EXISTS
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "WORKSPACE_ALREADY_EXISTS");
        assert!(json["existing_workspace_id"].is_string());
    }

    /// REQ-WORKSPACE-002.5: Path normalization for duplicate detection
    #[tokio::test]
    async fn test_path_normalization_detects_duplicates() {
        // GIVEN a directory with existing workspace
        let temp_dir = TempDir::new().unwrap();
        let canonical_path = temp_dir.path().canonicalize().unwrap();
        let app = create_test_router_with_workspace_state();

        // First request creates workspace
        let request1 = Request::builder()
            .method("POST")
            .uri("/workspace-create-from-path")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "source_path_directory_value": canonical_path.to_string_lossy()
            }).to_string()))
            .unwrap();

        let _ = app.clone().oneshot(request1).await.unwrap();

        // WHEN client sends request with trailing slash (equivalent path)
        let path_with_slash = format!("{}/", canonical_path.to_string_lossy());
        let request2 = Request::builder()
            .method("POST")
            .uri("/workspace-create-from-path")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "source_path_directory_value": path_with_slash
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request2).await.unwrap();

        // THEN should detect as duplicate
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }
}
```

### Acceptance Criteria

- [ ] Valid directory path is accepted
- [ ] Non-existent path returns 400 with PATH_NOT_FOUND
- [ ] File path (not directory) returns 400 with PATH_NOT_DIRECTORY
- [ ] Duplicate workspace returns 409 with WORKSPACE_ALREADY_EXISTS
- [ ] Paths with trailing slashes are normalized for duplicate detection
- [ ] Filesystem checks complete within 100ms

---

## REQ-WORKSPACE-003: Workspace Creation and Indexing

### Problem Statement

After validation passes, the endpoint must create the workspace directory structure, run initial indexing using pt01 streamer, and persist metadata to disk.

### Specification

#### REQ-WORKSPACE-003.1: Workspace ID Generation

```
WHEN workspace creation proceeds after validation
THEN SHALL generate unique workspace ID with format:
  "ws_{YYYYMMDD}_{HHMMSS}_{random6hex}"
  Example: "ws_20260123_143052_a1b2c3"
  AND SHALL ensure ID uniqueness across all workspaces
  AND SHALL use UTC timestamp
```

#### REQ-WORKSPACE-003.2: Directory Structure Creation

```
WHEN workspace ID is generated
THEN SHALL create directory structure:
  ~/.parseltongue/workspaces/{workspace_id}/
    metadata.json
    base.db/
    live.db/
  AND SHALL set appropriate file permissions (0755 for dirs, 0644 for files)
  AND SHALL handle ~/.parseltongue not existing by creating it
```

#### REQ-WORKSPACE-003.3: Initial Indexing with pt01

```
WHEN workspace directory is created
THEN SHALL invoke pt01 streamer on source_path_directory_value
  AND SHALL store output in base.db
  AND SHALL copy base.db to live.db (initial state)
  AND SHALL complete indexing within 60000ms for typical codebase (<10000 files)
  AND SHALL update last_indexed_timestamp_option on success
```

#### REQ-WORKSPACE-003.4: Indexing Failure Handling

```
WHEN pt01 streamer fails during indexing
THEN SHALL return HTTP 500 Internal Server Error
  AND SHALL return JSON body:
    {
      "error": "Failed to index codebase: {detailed_error}",
      "code": "INDEXING_FAILED"
    }
  AND SHALL clean up partial workspace directory
  AND SHALL NOT persist workspace metadata
```

#### REQ-WORKSPACE-003.5: Metadata Persistence

```
WHEN indexing completes successfully
THEN SHALL write metadata.json with WorkspaceMetadataPayloadStruct
  AND SHALL include all required fields
  AND SHALL set watch_enabled_flag_status to false initially
  AND SHALL serialize timestamps in ISO 8601 format
```

#### REQ-WORKSPACE-003.6: Successful Response

```
WHEN workspace creation completes
THEN SHALL return HTTP 200 OK
  AND SHALL return JSON body:
    {
      "success": true,
      "endpoint": "/workspace-create-from-path",
      "workspace": {
        "workspace_identifier_value": "ws_20260123_143052_a1b2c3",
        "workspace_display_name": "My Project",
        "source_directory_path_value": "/path/to/codebase",
        "base_database_path_value": "rocksdb:~/.parseltongue/workspaces/ws_xxx/base.db",
        "live_database_path_value": "rocksdb:~/.parseltongue/workspaces/ws_xxx/live.db",
        "watch_enabled_flag_status": false,
        "created_timestamp_utc_value": "2026-01-23T14:30:52Z",
        "last_indexed_timestamp_option": "2026-01-23T14:30:55Z"
      },
      "token_estimate": N
    }
```

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| ID generation | < 1ms | Timestamp from generation call |
| Directory creation | < 50ms | Filesystem operation timing |
| Small codebase indexing (<100 files) | < 5000ms | End-to-end request timing |
| Medium codebase indexing (100-1000 files) | < 30000ms | End-to-end request timing |
| Large codebase indexing (1000-10000 files) | < 60000ms | End-to-end request timing |
| Metadata write | < 10ms | File write timing |

### Verification Test Template

```rust
#[cfg(test)]
mod req_workspace_003_tests {
    use axum::{body::Body, http::{Request, StatusCode, header}};
    use tower::ServiceExt;
    use serde_json::{json, Value};
    use tempfile::TempDir;
    use std::path::Path;

    /// REQ-WORKSPACE-003.1: Workspace ID format is correct
    #[tokio::test]
    async fn test_workspace_id_format() {
        // GIVEN a valid directory
        let temp_dir = create_temp_codebase_with_files();
        let app = create_test_router_with_workspace_state();

        // WHEN client creates workspace
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-create-from-path")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "source_path_directory_value": temp_dir.path().to_string_lossy()
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // THEN workspace ID should match expected format
        let workspace_id = json["workspace"]["workspace_identifier_value"].as_str().unwrap();
        let id_regex = regex::Regex::new(r"^ws_\d{8}_\d{6}_[a-f0-9]{6}$").unwrap();
        assert!(id_regex.is_match(workspace_id), "Invalid workspace ID format: {}", workspace_id);
    }

    /// REQ-WORKSPACE-003.2: Directory structure is created
    #[tokio::test]
    async fn test_directory_structure_created() {
        // GIVEN a valid directory
        let temp_dir = create_temp_codebase_with_files();
        let app = create_test_router_with_workspace_state();

        // WHEN client creates workspace
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-create-from-path")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "source_path_directory_value": temp_dir.path().to_string_lossy()
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // THEN directory structure should exist
        let workspace_id = json["workspace"]["workspace_identifier_value"].as_str().unwrap();
        let workspace_dir = dirs::home_dir()
            .unwrap()
            .join(".parseltongue/workspaces")
            .join(workspace_id);

        assert!(workspace_dir.exists(), "Workspace directory not created");
        assert!(workspace_dir.join("metadata.json").exists(), "metadata.json not created");
        assert!(workspace_dir.join("base.db").exists(), "base.db not created");
        assert!(workspace_dir.join("live.db").exists(), "live.db not created");
    }

    /// REQ-WORKSPACE-003.3: Initial indexing populates databases
    #[tokio::test]
    async fn test_initial_indexing_populates_databases() {
        // GIVEN a directory with Rust source files
        let temp_dir = create_temp_codebase_with_rust_files();
        let app = create_test_router_with_workspace_state();

        // WHEN client creates workspace
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-create-from-path")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "source_path_directory_value": temp_dir.path().to_string_lossy()
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // THEN last_indexed_timestamp should be set
        assert!(json["workspace"]["last_indexed_timestamp_option"].is_string());
    }

    /// REQ-WORKSPACE-003.5: Metadata is persisted correctly
    #[tokio::test]
    async fn test_metadata_persisted_correctly() {
        // GIVEN a valid directory
        let temp_dir = create_temp_codebase_with_files();
        let app = create_test_router_with_workspace_state();

        // WHEN client creates workspace with display name
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-create-from-path")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "source_path_directory_value": temp_dir.path().to_string_lossy(),
                "workspace_display_name_option": "Test Display Name"
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // THEN metadata.json should contain correct data
        let workspace_id = json["workspace"]["workspace_identifier_value"].as_str().unwrap();
        let metadata_path = dirs::home_dir()
            .unwrap()
            .join(".parseltongue/workspaces")
            .join(workspace_id)
            .join("metadata.json");

        let metadata_content = std::fs::read_to_string(&metadata_path).unwrap();
        let metadata: Value = serde_json::from_str(&metadata_content).unwrap();

        assert_eq!(metadata["workspace_display_name"], "Test Display Name");
        assert_eq!(metadata["watch_enabled_flag_status"], false);
    }

    /// REQ-WORKSPACE-003.6: Response contains all required fields
    #[tokio::test]
    async fn test_response_contains_all_fields() {
        // GIVEN a valid directory
        let temp_dir = create_temp_codebase_with_files();
        let app = create_test_router_with_workspace_state();

        // WHEN client creates workspace
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-create-from-path")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "source_path_directory_value": temp_dir.path().to_string_lossy()
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // THEN all required fields should be present
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
```

### Acceptance Criteria

- [ ] Workspace ID follows format ws_{YYYYMMDD}_{HHMMSS}_{random6hex}
- [ ] Directory structure is created at ~/.parseltongue/workspaces/{id}/
- [ ] metadata.json is created with all fields
- [ ] base.db is populated via pt01 streamer
- [ ] live.db is created as copy of base.db
- [ ] watch_enabled_flag_status defaults to false
- [ ] Response contains all required fields
- [ ] Small codebase indexing completes within 5000ms

---

# Endpoint 2: GET /workspace-list-all

## REQ-WORKSPACE-004: List All Workspaces

### Problem Statement

Clients need to retrieve a list of all registered workspaces to display in UI or for programmatic workspace selection. The list should include metadata for each workspace.

### Specification

#### REQ-WORKSPACE-004.1: Empty List Response

```
WHEN client sends GET to /workspace-list-all
  WITH no workspaces registered
THEN SHALL return HTTP 200 OK
  AND SHALL return JSON body:
    {
      "success": true,
      "endpoint": "/workspace-list-all",
      "workspaces": [],
      "total_workspace_count_value": 0,
      "token_estimate": 100
    }
```

#### REQ-WORKSPACE-004.2: List With Multiple Workspaces

```
WHEN client sends GET to /workspace-list-all
  WITH N workspaces registered (N > 0)
THEN SHALL return HTTP 200 OK
  AND SHALL return JSON body with workspaces array containing N elements
  AND SHALL include full WorkspaceMetadataPayloadStruct for each workspace
  AND SHALL set total_workspace_count_value to N
  AND SHALL complete within 100ms for up to 100 workspaces
```

#### REQ-WORKSPACE-004.3: Workspaces Ordered by Creation Time

```
WHEN client receives workspace list response
THEN workspaces array SHALL be ordered by created_timestamp_utc_value
  AND order SHALL be descending (newest first)
```

#### REQ-WORKSPACE-004.4: Token Estimation

```
WHEN endpoint computes response
THEN SHALL calculate token_estimate as:
  base_tokens = 100
  per_workspace_tokens = 80
  total = base_tokens + (workspaces.len() * per_workspace_tokens)
AND SHALL include token_estimate in response
```

#### REQ-WORKSPACE-004.5: No Request Body Required

```
WHEN client sends GET to /workspace-list-all
  WITH any request body content
THEN SHALL ignore request body
  AND SHALL process request normally
  AND SHALL NOT return error for body presence
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_workspace_004_tests {
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;
    use serde_json::Value;

    /// REQ-WORKSPACE-004.1: Empty list response
    #[tokio::test]
    async fn test_empty_workspace_list() {
        // GIVEN a router with no workspaces
        let app = create_test_router_with_empty_workspace_state();

        // WHEN client sends GET request
        let request = Request::builder()
            .method("GET")
            .uri("/workspace-list-all")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return empty list
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["endpoint"], "/workspace-list-all");
        assert_eq!(json["workspaces"].as_array().unwrap().len(), 0);
        assert_eq!(json["total_workspace_count_value"], 0);
    }

    /// REQ-WORKSPACE-004.2: List with multiple workspaces
    #[tokio::test]
    async fn test_list_multiple_workspaces() {
        // GIVEN a router with 3 workspaces
        let app = create_test_router_with_n_workspaces(3);

        // WHEN client sends GET request
        let request = Request::builder()
            .method("GET")
            .uri("/workspace-list-all")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return all 3 workspaces
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["workspaces"].as_array().unwrap().len(), 3);
        assert_eq!(json["total_workspace_count_value"], 3);

        // Each workspace should have all required fields
        for workspace in json["workspaces"].as_array().unwrap() {
            assert!(workspace.get("workspace_identifier_value").is_some());
            assert!(workspace.get("workspace_display_name").is_some());
            assert!(workspace.get("source_directory_path_value").is_some());
            assert!(workspace.get("watch_enabled_flag_status").is_some());
        }
    }

    /// REQ-WORKSPACE-004.3: Workspaces ordered by creation time (newest first)
    #[tokio::test]
    async fn test_workspaces_ordered_by_creation_time() {
        // GIVEN a router with workspaces created at different times
        let app = create_test_router_with_timed_workspaces();

        // WHEN client sends GET request
        let request = Request::builder()
            .method("GET")
            .uri("/workspace-list-all")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // THEN workspaces should be ordered newest first
        let workspaces = json["workspaces"].as_array().unwrap();
        if workspaces.len() >= 2 {
            let first_time = workspaces[0]["created_timestamp_utc_value"].as_str().unwrap();
            let second_time = workspaces[1]["created_timestamp_utc_value"].as_str().unwrap();
            assert!(first_time >= second_time, "Workspaces should be ordered newest first");
        }
    }

    /// REQ-WORKSPACE-004.4: Token estimation is calculated
    #[tokio::test]
    async fn test_token_estimation() {
        // GIVEN a router with 2 workspaces
        let app = create_test_router_with_n_workspaces(2);

        // WHEN client sends GET request
        let request = Request::builder()
            .method("GET")
            .uri("/workspace-list-all")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // THEN token estimate should be base + (count * per_workspace)
        let token_estimate = json["token_estimate"].as_u64().unwrap();
        let expected_min = 100 + (2 * 80); // 260
        assert!(token_estimate >= expected_min as u64,
            "Token estimate {} should be >= {}", token_estimate, expected_min);
    }
}
```

### Acceptance Criteria

- [ ] Empty workspace state returns empty array with count 0
- [ ] Multiple workspaces are all returned in response
- [ ] Each workspace includes all metadata fields
- [ ] Workspaces are ordered by creation time (newest first)
- [ ] Token estimate is calculated and included
- [ ] Response completes within 100ms for up to 100 workspaces

---

# Endpoint 3: POST /workspace-watch-toggle

## REQ-WORKSPACE-005: Request Validation for Watch Toggle

### Problem Statement

The watch toggle endpoint must validate the incoming request contains a valid workspace ID and the desired watch state before attempting to modify the watcher.

### Specification

#### REQ-WORKSPACE-005.1: Valid Request Body Parsing

```
WHEN client sends POST to /workspace-watch-toggle
  WITH Content-Type: application/json
  AND body containing valid JSON:
    {
      "workspace_identifier_target_value": "ws_20260123_143052_a1b2c3",
      "watch_enabled_desired_state": true
    }
THEN SHALL parse request body successfully
  AND SHALL extract workspace_identifier_target_value as String
  AND SHALL extract watch_enabled_desired_state as boolean
  AND SHALL proceed to workspace lookup
```

#### REQ-WORKSPACE-005.2: Missing workspace_identifier_target_value

```
WHEN client sends POST to /workspace-watch-toggle
  WITH Content-Type: application/json
  AND body missing "workspace_identifier_target_value":
    {
      "watch_enabled_desired_state": true
    }
THEN SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body:
    {
      "error": "Missing required field: workspace_identifier_target_value",
      "code": "MISSING_WORKSPACE_ID"
    }
```

#### REQ-WORKSPACE-005.3: Empty workspace_identifier_target_value

```
WHEN client sends POST to /workspace-watch-toggle
  WITH body having empty workspace ID:
    {
      "workspace_identifier_target_value": "",
      "watch_enabled_desired_state": true
    }
THEN SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body:
    {
      "error": "workspace_identifier_target_value cannot be empty",
      "code": "INVALID_WORKSPACE_ID_EMPTY"
    }
```

#### REQ-WORKSPACE-005.4: Missing watch_enabled_desired_state

```
WHEN client sends POST to /workspace-watch-toggle
  WITH body missing "watch_enabled_desired_state":
    {
      "workspace_identifier_target_value": "ws_20260123_143052_a1b2c3"
    }
THEN SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body:
    {
      "error": "Missing required field: watch_enabled_desired_state",
      "code": "MISSING_WATCH_STATE"
    }
```

#### REQ-WORKSPACE-005.5: Workspace Not Found

```
WHEN client sends POST to /workspace-watch-toggle
  WITH workspace_identifier_target_value that does not exist: "ws_nonexistent"
THEN SHALL return HTTP 404 Not Found
  AND SHALL return JSON body:
    {
      "error": "Workspace not found: ws_nonexistent",
      "code": "WORKSPACE_NOT_FOUND"
    }
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_workspace_005_tests {
    use axum::{body::Body, http::{Request, StatusCode, header}};
    use tower::ServiceExt;
    use serde_json::json;

    /// REQ-WORKSPACE-005.1: Valid request body parsing
    #[tokio::test]
    async fn test_valid_watch_toggle_request() {
        // GIVEN a router with existing workspace
        let (app, workspace_id) = create_router_with_single_workspace();

        // WHEN client sends valid toggle request
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-watch-toggle")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "workspace_identifier_target_value": workspace_id,
                "watch_enabled_desired_state": true
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should succeed
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// REQ-WORKSPACE-005.2: Missing workspace ID returns 400
    #[tokio::test]
    async fn test_missing_workspace_id_returns_400() {
        // GIVEN a configured router
        let app = create_test_router_with_workspace_state();

        // WHEN client sends request without workspace ID
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-watch-toggle")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "watch_enabled_desired_state": true
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 400 with MISSING_WORKSPACE_ID
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "MISSING_WORKSPACE_ID");
    }

    /// REQ-WORKSPACE-005.3: Empty workspace ID returns 400
    #[tokio::test]
    async fn test_empty_workspace_id_returns_400() {
        // GIVEN a configured router
        let app = create_test_router_with_workspace_state();

        // WHEN client sends request with empty workspace ID
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-watch-toggle")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "workspace_identifier_target_value": "",
                "watch_enabled_desired_state": true
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 400 with INVALID_WORKSPACE_ID_EMPTY
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "INVALID_WORKSPACE_ID_EMPTY");
    }

    /// REQ-WORKSPACE-005.4: Missing watch state returns 400
    #[tokio::test]
    async fn test_missing_watch_state_returns_400() {
        // GIVEN a configured router
        let app = create_test_router_with_workspace_state();

        // WHEN client sends request without watch state
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-watch-toggle")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "workspace_identifier_target_value": "ws_test"
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 400 with MISSING_WATCH_STATE
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "MISSING_WATCH_STATE");
    }

    /// REQ-WORKSPACE-005.5: Non-existent workspace returns 404
    #[tokio::test]
    async fn test_nonexistent_workspace_returns_404() {
        // GIVEN a router with no matching workspace
        let app = create_test_router_with_workspace_state();

        // WHEN client sends request for non-existent workspace
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-watch-toggle")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "workspace_identifier_target_value": "ws_nonexistent_12345",
                "watch_enabled_desired_state": true
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 404 with WORKSPACE_NOT_FOUND
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "WORKSPACE_NOT_FOUND");
    }
}
```

### Acceptance Criteria

- [ ] Valid request with both fields is accepted
- [ ] Missing workspace ID returns 400 with MISSING_WORKSPACE_ID
- [ ] Empty workspace ID returns 400 with INVALID_WORKSPACE_ID_EMPTY
- [ ] Missing watch state returns 400 with MISSING_WATCH_STATE
- [ ] Non-existent workspace returns 404 with WORKSPACE_NOT_FOUND

---

## REQ-WORKSPACE-006: Watch Enable/Disable Operations

### Problem Statement

The endpoint must correctly start or stop the file watcher based on the requested state, handling idempotent requests and watcher failures gracefully.

### Specification

#### REQ-WORKSPACE-006.1: Enable Watch (Currently Disabled)

```
WHEN client sends POST to /workspace-watch-toggle
  WITH watch_enabled_desired_state: true
  AND workspace has watch_enabled_flag_status: false
THEN SHALL start file watcher on source_directory_path_value
  AND SHALL register watcher in SharedWorkspaceStateContainer
  AND SHALL update watch_enabled_flag_status to true
  AND SHALL persist updated metadata to disk
  AND SHALL return HTTP 200 OK with updated workspace
```

#### REQ-WORKSPACE-006.2: Disable Watch (Currently Enabled)

```
WHEN client sends POST to /workspace-watch-toggle
  WITH watch_enabled_desired_state: false
  AND workspace has watch_enabled_flag_status: true
THEN SHALL stop existing file watcher
  AND SHALL remove watcher from SharedWorkspaceStateContainer
  AND SHALL update watch_enabled_flag_status to false
  AND SHALL persist updated metadata to disk
  AND SHALL return HTTP 200 OK with updated workspace
```

#### REQ-WORKSPACE-006.3: Enable Watch (Already Enabled) - Idempotent

```
WHEN client sends POST to /workspace-watch-toggle
  WITH watch_enabled_desired_state: true
  AND workspace has watch_enabled_flag_status: true (already enabled)
THEN SHALL NOT start duplicate watcher
  AND SHALL return HTTP 200 OK with current workspace
  AND SHALL keep watch_enabled_flag_status as true
  AND response.workspace SHALL match current state
```

#### REQ-WORKSPACE-006.4: Disable Watch (Already Disabled) - Idempotent

```
WHEN client sends POST to /workspace-watch-toggle
  WITH watch_enabled_desired_state: false
  AND workspace has watch_enabled_flag_status: false (already disabled)
THEN SHALL NOT attempt to stop non-existent watcher
  AND SHALL return HTTP 200 OK with current workspace
  AND SHALL keep watch_enabled_flag_status as false
```

#### REQ-WORKSPACE-006.5: Watcher Start Failure

```
WHEN client sends POST to /workspace-watch-toggle
  WITH watch_enabled_desired_state: true
  AND file watcher fails to start (e.g., permission denied, inotify limit)
THEN SHALL return HTTP 500 Internal Server Error
  AND SHALL return JSON body:
    {
      "error": "Failed to start file watcher: {detailed_error}",
      "code": "WATCHER_START_FAILED"
    }
  AND SHALL NOT update watch_enabled_flag_status
  AND SHALL NOT modify persistent metadata
```

#### REQ-WORKSPACE-006.6: Watcher Stop Failure

```
WHEN client sends POST to /workspace-watch-toggle
  WITH watch_enabled_desired_state: false
  AND file watcher fails to stop gracefully
THEN SHALL return HTTP 500 Internal Server Error
  AND SHALL return JSON body:
    {
      "error": "Failed to stop file watcher: {detailed_error}",
      "code": "WATCHER_STOP_FAILED"
    }
  AND SHALL attempt to force-kill watcher
  AND SHALL log error for investigation
```

#### REQ-WORKSPACE-006.7: Successful Toggle Response

```
WHEN watch toggle completes successfully
THEN SHALL return HTTP 200 OK
  AND SHALL return JSON body:
    {
      "success": true,
      "endpoint": "/workspace-watch-toggle",
      "workspace": {
        "workspace_identifier_value": "ws_20260123_143052_a1b2c3",
        "watch_enabled_flag_status": <new_state>,
        ...other fields unchanged...
      },
      "token_estimate": N
    }
  AND watch_enabled_flag_status SHALL match requested state
```

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Watcher start time | < 500ms | Time from request to watcher active |
| Watcher stop time | < 200ms | Time from request to watcher stopped |
| State persistence | < 50ms | Time to write metadata.json |
| Total request time | < 1000ms | End-to-end request timing |

### Verification Test Template

```rust
#[cfg(test)]
mod req_workspace_006_tests {
    use axum::{body::Body, http::{Request, StatusCode, header}};
    use tower::ServiceExt;
    use serde_json::{json, Value};

    /// REQ-WORKSPACE-006.1: Enable watch from disabled state
    #[tokio::test]
    async fn test_enable_watch_from_disabled() {
        // GIVEN a workspace with watch disabled
        let (app, workspace_id) = create_router_with_workspace_watch_disabled();

        // WHEN client enables watch
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-watch-toggle")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "workspace_identifier_target_value": workspace_id,
                "watch_enabled_desired_state": true
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // THEN watch should be enabled
        assert_eq!(json["workspace"]["watch_enabled_flag_status"], true);
    }

    /// REQ-WORKSPACE-006.2: Disable watch from enabled state
    #[tokio::test]
    async fn test_disable_watch_from_enabled() {
        // GIVEN a workspace with watch enabled
        let (app, workspace_id) = create_router_with_workspace_watch_enabled();

        // WHEN client disables watch
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-watch-toggle")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "workspace_identifier_target_value": workspace_id,
                "watch_enabled_desired_state": false
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // THEN watch should be disabled
        assert_eq!(json["workspace"]["watch_enabled_flag_status"], false);
    }

    /// REQ-WORKSPACE-006.3: Enable watch when already enabled (idempotent)
    #[tokio::test]
    async fn test_enable_watch_when_already_enabled() {
        // GIVEN a workspace with watch already enabled
        let (app, workspace_id) = create_router_with_workspace_watch_enabled();

        // WHEN client enables watch again
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-watch-toggle")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "workspace_identifier_target_value": workspace_id,
                "watch_enabled_desired_state": true
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should succeed without error
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["workspace"]["watch_enabled_flag_status"], true);
    }

    /// REQ-WORKSPACE-006.4: Disable watch when already disabled (idempotent)
    #[tokio::test]
    async fn test_disable_watch_when_already_disabled() {
        // GIVEN a workspace with watch already disabled
        let (app, workspace_id) = create_router_with_workspace_watch_disabled();

        // WHEN client disables watch again
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-watch-toggle")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "workspace_identifier_target_value": workspace_id,
                "watch_enabled_desired_state": false
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should succeed without error
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["workspace"]["watch_enabled_flag_status"], false);
    }

    /// REQ-WORKSPACE-006.7: Response contains all required fields
    #[tokio::test]
    async fn test_toggle_response_has_all_fields() {
        // GIVEN a workspace
        let (app, workspace_id) = create_router_with_single_workspace();

        // WHEN client toggles watch
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-watch-toggle")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "workspace_identifier_target_value": workspace_id,
                "watch_enabled_desired_state": true
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // THEN response should have all required fields
        assert_eq!(json["success"], true);
        assert_eq!(json["endpoint"], "/workspace-watch-toggle");
        assert!(json.get("workspace").is_some());
        assert!(json.get("token_estimate").is_some());
    }
}
```

### Acceptance Criteria

- [ ] Enabling watch on disabled workspace starts watcher
- [ ] Disabling watch on enabled workspace stops watcher
- [ ] Enabling already-enabled watch is idempotent (no error)
- [ ] Disabling already-disabled watch is idempotent (no error)
- [ ] Watcher start failure returns 500 with WATCHER_START_FAILED
- [ ] Watcher stop failure returns 500 with WATCHER_STOP_FAILED
- [ ] Successful toggle persists metadata to disk
- [ ] Total toggle time < 1000ms

---

## REQ-WORKSPACE-007: Error Response Format

### Problem Statement

All error responses across workspace endpoints must follow a consistent structure for predictable client handling.

### Specification

#### REQ-WORKSPACE-007.1: Consistent Error Structure

```
WHEN any workspace endpoint returns an error response
THEN SHALL return JSON body with structure:
    {
      "error": "<human-readable error message>",
      "code": "<MACHINE_READABLE_ERROR_CODE>"
    }
  AND SHALL include Content-Type: application/json header
  AND "error" SHALL be non-empty string
  AND "code" SHALL be SCREAMING_SNAKE_CASE format
```

#### REQ-WORKSPACE-007.2: HTTP Status Code Mapping

```
WHEN endpoint returns error with specific code
THEN SHALL map to correct HTTP status:
  - 400 Bad Request: MISSING_SOURCE_PATH, INVALID_SOURCE_PATH_EMPTY,
                     PATH_NOT_FOUND, PATH_NOT_DIRECTORY,
                     MISSING_WORKSPACE_ID, INVALID_WORKSPACE_ID_EMPTY,
                     MISSING_WATCH_STATE, INVALID_JSON
  - 404 Not Found: WORKSPACE_NOT_FOUND
  - 409 Conflict: WORKSPACE_ALREADY_EXISTS
  - 415 Unsupported Media Type: UNSUPPORTED_MEDIA_TYPE
  - 500 Internal Server Error: INDEXING_FAILED, WATCHER_START_FAILED,
                               WATCHER_STOP_FAILED, STORAGE_WRITE_FAILED,
                               STORAGE_READ_FAILED, INTERNAL_ERROR
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_workspace_007_tests {
    use axum::{body::Body, http::{Request, StatusCode, header}};
    use tower::ServiceExt;
    use serde_json::Value;

    /// REQ-WORKSPACE-007.1: All errors have consistent structure
    #[tokio::test]
    async fn test_error_response_structure() {
        // GIVEN a router
        let app = create_test_router_with_workspace_state();

        // WHEN client sends request that will error
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-create-from-path")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{}"#))  // Missing required field
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // THEN error structure should be correct
        assert!(json.get("error").is_some());
        assert!(json["error"].is_string());
        assert!(!json["error"].as_str().unwrap().is_empty());

        assert!(json.get("code").is_some());
        assert!(json["code"].is_string());
        let code = json["code"].as_str().unwrap();
        assert!(code.chars().all(|c| c.is_uppercase() || c == '_'));
    }

    /// REQ-WORKSPACE-007.2: Status codes map correctly
    #[tokio::test]
    async fn test_status_code_mapping() {
        let app = create_test_router_with_workspace_state();

        // Test 400 for validation error
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-create-from-path")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"source_path_directory_value": ""}"#))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Test 404 for workspace not found
        let request = Request::builder()
            .method("POST")
            .uri("/workspace-watch-toggle")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{
                "workspace_identifier_target_value": "ws_nonexistent",
                "watch_enabled_desired_state": true
            }"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
```

### Acceptance Criteria

- [ ] All error responses have "error" and "code" fields
- [ ] "code" is always SCREAMING_SNAKE_CASE
- [ ] Status codes match the specification mapping
- [ ] Content-Type is always application/json for errors

---

## Implementation Guide

### Handler Function Signatures

Following the 4-word naming convention:

```rust
/// Handle workspace creation from path request
///
/// # 4-Word Name: handle_workspace_create_from_path
pub async fn handle_workspace_create_from_path(
    State(state): State<SharedWorkspaceStateContainer>,
    Json(body): Json<WorkspaceCreateRequestBodyStruct>,
) -> impl IntoResponse {
    // Implementation
}

/// Handle workspace list all entries request
///
/// # 4-Word Name: handle_workspace_list_all_entries
pub async fn handle_workspace_list_all_entries(
    State(state): State<SharedWorkspaceStateContainer>,
) -> impl IntoResponse {
    // Implementation
}

/// Handle workspace watch toggle state request
///
/// # 4-Word Name: handle_workspace_watch_toggle_state
pub async fn handle_workspace_watch_toggle_state(
    State(state): State<SharedWorkspaceStateContainer>,
    Json(body): Json<WorkspaceWatchToggleRequestStruct>,
) -> impl IntoResponse {
    // Implementation
}
```

### Route Registration

Add to `route_definition_builder_module.rs`:

```rust
use axum::routing::{get, post};

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

### Shared State Container

```rust
/// Container for workspace-related state
///
/// # 4-Word Name: SharedWorkspaceStateContainer
pub struct SharedWorkspaceStateContainer {
    /// All registered workspaces
    pub workspaces: Arc<RwLock<HashMap<String, WorkspaceMetadataPayloadStruct>>>,
    /// Active file watchers by workspace ID
    pub watchers: Arc<RwLock<HashMap<String, WatcherHandle>>>,
    /// WebSocket connections per workspace (for Phase 2.2)
    pub ws_connections: Arc<RwLock<HashMap<String, Vec<WsSender>>>>,
}
```

---

## Quality Checklist

Before implementation is complete, verify:

- [ ] All quantities are specific and measurable
- [ ] All behaviors are testable
- [ ] Error conditions are specified with codes
- [ ] Performance boundaries are defined
- [ ] Test templates are provided for all requirements
- [ ] Acceptance criteria are binary (pass/fail)
- [ ] No ambiguous language remains
- [ ] 4-word naming convention is followed
- [ ] Token estimation is included in responses
- [ ] Handlers follow existing patterns in codebase
- [ ] Idempotent operations are handled correctly
- [ ] Filesystem operations handle permissions gracefully

---

## Traceability Matrix

| Requirement | Endpoint | Error Codes | Test Count |
|-------------|----------|-------------|------------|
| REQ-WORKSPACE-001 | POST /workspace-create-from-path | MISSING_SOURCE_PATH, INVALID_SOURCE_PATH_EMPTY, INVALID_JSON, UNSUPPORTED_MEDIA_TYPE | 6 |
| REQ-WORKSPACE-002 | POST /workspace-create-from-path | PATH_NOT_FOUND, PATH_NOT_DIRECTORY, WORKSPACE_ALREADY_EXISTS | 5 |
| REQ-WORKSPACE-003 | POST /workspace-create-from-path | INDEXING_FAILED, STORAGE_WRITE_FAILED | 6 |
| REQ-WORKSPACE-004 | GET /workspace-list-all | STORAGE_READ_FAILED | 4 |
| REQ-WORKSPACE-005 | POST /workspace-watch-toggle | MISSING_WORKSPACE_ID, INVALID_WORKSPACE_ID_EMPTY, MISSING_WATCH_STATE, WORKSPACE_NOT_FOUND | 5 |
| REQ-WORKSPACE-006 | POST /workspace-watch-toggle | WATCHER_START_FAILED, WATCHER_STOP_FAILED | 5 |
| REQ-WORKSPACE-007 | All | All error codes | 2 |
| **Total** | **3 endpoints** | **17 error codes** | **33 tests** |

---

*Specification document created 2026-01-23*
*Phase 2.1 target: Workspace Management Backend*
*Test target: 30+ new tests*

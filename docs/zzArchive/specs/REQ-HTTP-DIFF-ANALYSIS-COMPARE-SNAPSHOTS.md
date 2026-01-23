# HTTP Endpoint Specification: `/diff-analysis-compare-snapshots`

## Phase 7 - Diff Visualization System HTTP Endpoint

**Document Version**: 1.0.0
**Created**: 2026-01-23
**Status**: Specification Complete

---

## Overview

### Problem Statement

Developers and LLM agents need a programmatic way to compare two database snapshots (e.g., before and after code changes) and understand:
1. What entities changed (added, removed, modified)
2. What is the blast radius of those changes
3. A visualization-ready representation of the diff

Currently, the diff logic exists in `parseltongue-core/src/diff/` but is not exposed via HTTP. Users must write custom code to compare snapshots.

### Solution

A POST endpoint `/diff-analysis-compare-snapshots` that:
- Accepts two database paths (base and live)
- Returns unified diff results with blast radius and visualization data
- Follows existing handler patterns (4-word naming, token estimation)

---

## REQ-HTTP-DIFF-001: Request Parsing and Validation

### Problem Statement

The endpoint must correctly parse incoming JSON request bodies and validate required fields before attempting any database operations. Invalid requests should fail fast with clear error messages.

### Specification

#### REQ-HTTP-DIFF-001.1: Valid Request Body Parsing

```
WHEN client sends POST to /diff-analysis-compare-snapshots
  WITH Content-Type: application/json
  AND body containing valid JSON:
    {
      "base_db": "rocksdb:path/to/base.db",
      "live_db": "rocksdb:path/to/live.db"
    }
THEN SHALL parse request body successfully
  AND SHALL extract base_db field as String
  AND SHALL extract live_db field as String
  AND SHALL proceed to database connection phase
  AND SHALL NOT return 400 Bad Request
```

#### REQ-HTTP-DIFF-001.2: Missing base_db Field Error

```
WHEN client sends POST to /diff-analysis-compare-snapshots
  WITH Content-Type: application/json
  AND body missing "base_db" field:
    {
      "live_db": "rocksdb:path/to/live.db"
    }
THEN SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body:
    {
      "error": "Missing required field: base_db",
      "code": "MISSING_BASE_DB"
    }
  AND SHALL NOT attempt database connection
  AND SHALL complete within 10ms
```

#### REQ-HTTP-DIFF-001.3: Missing live_db Field Error

```
WHEN client sends POST to /diff-analysis-compare-snapshots
  WITH Content-Type: application/json
  AND body missing "live_db" field:
    {
      "base_db": "rocksdb:path/to/base.db"
    }
THEN SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body:
    {
      "error": "Missing required field: live_db",
      "code": "MISSING_LIVE_DB"
    }
  AND SHALL NOT attempt database connection
  AND SHALL complete within 10ms
```

#### REQ-HTTP-DIFF-001.4: Empty base_db Field Error

```
WHEN client sends POST to /diff-analysis-compare-snapshots
  WITH Content-Type: application/json
  AND body with empty "base_db" field:
    {
      "base_db": "",
      "live_db": "rocksdb:path/to/live.db"
    }
THEN SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body:
    {
      "error": "Invalid base_db: path cannot be empty",
      "code": "INVALID_BASE_DB_PATH"
    }
  AND SHALL NOT attempt database connection
```

#### REQ-HTTP-DIFF-001.5: Empty live_db Field Error

```
WHEN client sends POST to /diff-analysis-compare-snapshots
  WITH Content-Type: application/json
  AND body with empty "live_db" field:
    {
      "base_db": "rocksdb:path/to/base.db",
      "live_db": ""
    }
THEN SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body:
    {
      "error": "Invalid live_db: path cannot be empty",
      "code": "INVALID_LIVE_DB_PATH"
    }
  AND SHALL NOT attempt database connection
```

#### REQ-HTTP-DIFF-001.6: Invalid JSON Body Error

```
WHEN client sends POST to /diff-analysis-compare-snapshots
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

#### REQ-HTTP-DIFF-001.7: Missing Content-Type Header

```
WHEN client sends POST to /diff-analysis-compare-snapshots
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
mod req_http_diff_001_tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;
    use serde_json::json;

    /// REQ-HTTP-DIFF-001.1: Valid request body parsing
    #[tokio::test]
    async fn test_valid_request_body_parsing() {
        // GIVEN a configured router with the diff endpoint
        let app = create_test_router_with_mock_dbs();

        // WHEN client sends valid POST request
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "base_db": "rocksdb:test/base.db",
                "live_db": "rocksdb:test/live.db"
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should succeed (200) or return diff data
        assert!(response.status() == StatusCode::OK ||
                response.status() == StatusCode::NOT_FOUND);
    }

    /// REQ-HTTP-DIFF-001.2: Missing base_db field error
    #[tokio::test]
    async fn test_missing_base_db_returns_400() {
        // GIVEN a configured router
        let app = create_test_router();

        // WHEN client sends request without base_db
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "live_db": "rocksdb:test/live.db"
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 400 with MISSING_BASE_DB code
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "MISSING_BASE_DB");
    }

    /// REQ-HTTP-DIFF-001.3: Missing live_db field error
    #[tokio::test]
    async fn test_missing_live_db_returns_400() {
        // GIVEN a configured router
        let app = create_test_router();

        // WHEN client sends request without live_db
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "base_db": "rocksdb:test/base.db"
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 400 with MISSING_LIVE_DB code
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "MISSING_LIVE_DB");
    }

    /// REQ-HTTP-DIFF-001.4: Empty base_db field error
    #[tokio::test]
    async fn test_empty_base_db_returns_400() {
        // GIVEN a configured router
        let app = create_test_router();

        // WHEN client sends request with empty base_db
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "base_db": "",
                "live_db": "rocksdb:test/live.db"
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 400 with INVALID_BASE_DB_PATH code
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "INVALID_BASE_DB_PATH");
    }

    /// REQ-HTTP-DIFF-001.6: Invalid JSON body error
    #[tokio::test]
    async fn test_invalid_json_returns_400() {
        // GIVEN a configured router
        let app = create_test_router();

        // WHEN client sends malformed JSON
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots")
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

- [ ] Valid JSON with both fields parses successfully
- [ ] Missing base_db returns 400 with code MISSING_BASE_DB
- [ ] Missing live_db returns 400 with code MISSING_LIVE_DB
- [ ] Empty base_db returns 400 with code INVALID_BASE_DB_PATH
- [ ] Empty live_db returns 400 with code INVALID_LIVE_DB_PATH
- [ ] Malformed JSON returns 400 with code INVALID_JSON
- [ ] All validation errors complete within 10ms

---

## REQ-HTTP-DIFF-002: Database Connection Handling

### Problem Statement

The endpoint must connect to two separate CozoDB databases (base and live snapshots) and handle connection failures gracefully. Connection errors should be distinguishable from other errors.

### Specification

#### REQ-HTTP-DIFF-002.1: Successful Connection to Both Databases

```
WHEN client sends valid POST to /diff-analysis-compare-snapshots
  WITH base_db pointing to existing valid RocksDB database
  AND live_db pointing to existing valid RocksDB database
THEN SHALL open connection to base_db
  AND SHALL open connection to live_db
  AND SHALL proceed to diff computation
  AND SHALL NOT return error
  AND SHALL complete connection phase within 5000ms
```

#### REQ-HTTP-DIFF-002.2: Base Database Not Found Error

```
WHEN client sends POST to /diff-analysis-compare-snapshots
  WITH base_db pointing to non-existent path: "rocksdb:nonexistent/base.db"
  AND live_db pointing to valid database
THEN SHALL return HTTP 404 Not Found
  AND SHALL return JSON body:
    {
      "error": "Base database not found: rocksdb:nonexistent/base.db",
      "code": "BASE_DB_NOT_FOUND"
    }
  AND SHALL NOT attempt to open live_db
  AND SHALL complete within 1000ms
```

#### REQ-HTTP-DIFF-002.3: Live Database Not Found Error

```
WHEN client sends POST to /diff-analysis-compare-snapshots
  WITH base_db pointing to valid database
  AND live_db pointing to non-existent path: "rocksdb:nonexistent/live.db"
THEN SHALL return HTTP 404 Not Found
  AND SHALL return JSON body:
    {
      "error": "Live database not found: rocksdb:nonexistent/live.db",
      "code": "LIVE_DB_NOT_FOUND"
    }
  AND SHALL close base_db connection
  AND SHALL complete within 1000ms
```

#### REQ-HTTP-DIFF-002.4: Database Connection Timeout

```
WHEN client sends POST to /diff-analysis-compare-snapshots
  WITH database paths that cause connection to hang
THEN SHALL timeout after 30000ms
  AND SHALL return HTTP 504 Gateway Timeout
  AND SHALL return JSON body:
    {
      "error": "Database connection timed out",
      "code": "DB_CONNECTION_TIMEOUT"
    }
  AND SHALL clean up any partial connections
```

#### REQ-HTTP-DIFF-002.5: Invalid Database Path Format

```
WHEN client sends POST to /diff-analysis-compare-snapshots
  WITH base_db not starting with "rocksdb:" prefix: "invalid/path.db"
THEN SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body:
    {
      "error": "Invalid database path format. Expected 'rocksdb:path/to/db'",
      "code": "INVALID_DB_PATH_FORMAT"
    }
```

#### REQ-HTTP-DIFF-002.6: Database Permission Denied

```
WHEN client sends POST to /diff-analysis-compare-snapshots
  WITH base_db pointing to path with insufficient permissions
THEN SHALL return HTTP 403 Forbidden
  AND SHALL return JSON body:
    {
      "error": "Permission denied accessing database: rocksdb:protected/base.db",
      "code": "DB_PERMISSION_DENIED"
    }
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_http_diff_002_tests {
    use axum::{body::Body, http::{Request, StatusCode, header}};
    use tower::ServiceExt;
    use serde_json::json;
    use tempfile::TempDir;

    /// REQ-HTTP-DIFF-002.1: Successful connection to both databases
    #[tokio::test]
    async fn test_successful_connection_both_dbs() {
        // GIVEN two valid test databases
        let temp_dir = TempDir::new().unwrap();
        let base_path = create_test_db(&temp_dir, "base.db").await;
        let live_path = create_test_db(&temp_dir, "live.db").await;

        let app = create_test_router();

        // WHEN client sends request with valid paths
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "base_db": base_path,
                "live_db": live_path
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 200 OK with diff data
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// REQ-HTTP-DIFF-002.2: Base database not found error
    #[tokio::test]
    async fn test_base_db_not_found_returns_404() {
        // GIVEN only live database exists
        let temp_dir = TempDir::new().unwrap();
        let live_path = create_test_db(&temp_dir, "live.db").await;

        let app = create_test_router();

        // WHEN client sends request with non-existent base_db
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "base_db": "rocksdb:nonexistent/base.db",
                "live_db": live_path
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 404 with BASE_DB_NOT_FOUND code
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "BASE_DB_NOT_FOUND");
    }

    /// REQ-HTTP-DIFF-002.3: Live database not found error
    #[tokio::test]
    async fn test_live_db_not_found_returns_404() {
        // GIVEN only base database exists
        let temp_dir = TempDir::new().unwrap();
        let base_path = create_test_db(&temp_dir, "base.db").await;

        let app = create_test_router();

        // WHEN client sends request with non-existent live_db
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "base_db": base_path,
                "live_db": "rocksdb:nonexistent/live.db"
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 404 with LIVE_DB_NOT_FOUND code
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "LIVE_DB_NOT_FOUND");
    }

    /// REQ-HTTP-DIFF-002.5: Invalid database path format
    #[tokio::test]
    async fn test_invalid_db_path_format_returns_400() {
        // GIVEN router configured
        let app = create_test_router();

        // WHEN client sends request with invalid path format
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "base_db": "invalid/path.db",  // Missing rocksdb: prefix
                "live_db": "rocksdb:valid/live.db"
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 400 with INVALID_DB_PATH_FORMAT code
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "INVALID_DB_PATH_FORMAT");
    }
}
```

### Acceptance Criteria

- [ ] Both valid databases connect successfully
- [ ] Non-existent base_db returns 404 with code BASE_DB_NOT_FOUND
- [ ] Non-existent live_db returns 404 with code LIVE_DB_NOT_FOUND
- [ ] Connection timeout returns 504 with code DB_CONNECTION_TIMEOUT
- [ ] Invalid path format returns 400 with code INVALID_DB_PATH_FORMAT
- [ ] Connection phase completes within 5000ms for valid databases

---

## REQ-HTTP-DIFF-003: Diff Computation and Response

### Problem Statement

Once databases are connected, the endpoint must compute entity and edge diffs, calculate blast radius, and transform results into a visualization-ready format with token estimation.

### Specification

#### REQ-HTTP-DIFF-003.1: Successful Diff With Changes

```
WHEN client sends valid POST to /diff-analysis-compare-snapshots
  WITH base_db containing entities A, B, C
  AND live_db containing entities A, B_modified, D (where B was modified, C removed, D added)
THEN SHALL return HTTP 200 OK
  AND SHALL return JSON body with structure:
    {
      "success": true,
      "endpoint": "/diff-analysis-compare-snapshots",
      "diff": {
        "summary": {
          "total_before_count": 3,
          "total_after_count": 3,
          "added_entity_count": 1,
          "removed_entity_count": 1,
          "modified_entity_count": 1,
          "unchanged_entity_count": 1,
          "relocated_entity_count": 0
        },
        "entity_changes": [...],
        "edge_changes": [...]
      },
      "blast_radius": {
        "origin_entity": "<combined>",
        "affected_by_distance": {...},
        "total_affected_count": N,
        "max_depth_reached": M
      },
      "visualization": {
        "nodes": [...],
        "edges": [...],
        "diff_summary": {...},
        "max_blast_radius_depth": 2
      },
      "token_estimate": N
    }
  AND SHALL include all changed entities in entity_changes array
  AND SHALL compute blast radius for all modified/added/removed entities
```

#### REQ-HTTP-DIFF-003.2: Successful Diff With No Changes

```
WHEN client sends valid POST to /diff-analysis-compare-snapshots
  WITH base_db containing entities A, B, C
  AND live_db containing identical entities A, B, C
THEN SHALL return HTTP 200 OK
  AND SHALL return JSON body with:
    {
      "success": true,
      "diff": {
        "summary": {
          "total_before_count": 3,
          "total_after_count": 3,
          "added_entity_count": 0,
          "removed_entity_count": 0,
          "modified_entity_count": 0,
          "unchanged_entity_count": 3,
          "relocated_entity_count": 0
        },
        "entity_changes": [],
        "edge_changes": []
      },
      "blast_radius": {
        "total_affected_count": 0
      },
      "visualization": {
        "nodes": [],
        "edges": []
      },
      "token_estimate": N
    }
```

#### REQ-HTTP-DIFF-003.3: Response Includes All Required Fields

```
WHEN client receives successful response from /diff-analysis-compare-snapshots
THEN response SHALL contain field "success" of type boolean
  AND SHALL contain field "endpoint" of type string
  AND SHALL contain field "diff" of type object
  AND SHALL contain field "diff.summary" of type object
  AND SHALL contain field "diff.entity_changes" of type array
  AND SHALL contain field "diff.edge_changes" of type array
  AND SHALL contain field "blast_radius" of type object
  AND SHALL contain field "visualization" of type object
  AND SHALL contain field "token_estimate" of type integer
  AND SHALL NOT contain null values for required fields
```

#### REQ-HTTP-DIFF-003.4: Token Estimation Calculation

```
WHEN endpoint computes response
THEN SHALL calculate token_estimate as:
    base_tokens = 100
    diff_tokens = (entity_changes.len() * 50) + (edge_changes.len() * 30)
    blast_tokens = total_affected_count * 20
    viz_tokens = (nodes.len() * 40) + (edges.len() * 25)
    total = base_tokens + diff_tokens + blast_tokens + viz_tokens
  AND SHALL include token_estimate in response
  AND SHALL ensure token_estimate >= 100 (minimum)
```

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Latency (small diff, <100 entities) | < 500ms p99 | End-to-end request timing |
| Latency (medium diff, 100-1000 entities) | < 2000ms p99 | End-to-end request timing |
| Latency (large diff, 1000-10000 entities) | < 10000ms p99 | End-to-end request timing |
| Memory (response size) | < 10MB | Response body size |
| Memory (working set) | < 500MB | Process memory during computation |

### Verification Test Template

```rust
#[cfg(test)]
mod req_http_diff_003_tests {
    use axum::{body::Body, http::{Request, StatusCode, header}};
    use tower::ServiceExt;
    use serde_json::{json, Value};
    use tempfile::TempDir;

    /// REQ-HTTP-DIFF-003.1: Successful diff with changes
    #[tokio::test]
    async fn test_diff_with_changes_returns_complete_response() {
        // GIVEN base_db with entities A, B, C and live_db with A, B_modified, D
        let temp_dir = TempDir::new().unwrap();
        let base_path = create_test_db_with_entities(
            &temp_dir, "base.db",
            vec!["rust:fn:a:path:1-10", "rust:fn:b:path:11-20", "rust:fn:c:path:21-30"]
        ).await;
        let live_path = create_test_db_with_entities(
            &temp_dir, "live.db",
            vec!["rust:fn:a:path:1-10", "rust:fn:b:path:11-25", "rust:fn:d:path:31-40"]  // b modified, c removed, d added
        ).await;

        let app = create_test_router();

        // WHEN client sends diff request
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "base_db": base_path,
                "live_db": live_path
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 200 with complete diff data
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert!(json["diff"]["summary"]["added_entity_count"].as_u64().unwrap() >= 1);
        assert!(json["diff"]["summary"]["removed_entity_count"].as_u64().unwrap() >= 1);
        assert!(json["token_estimate"].as_u64().is_some());
    }

    /// REQ-HTTP-DIFF-003.2: Successful diff with no changes
    #[tokio::test]
    async fn test_diff_with_no_changes_returns_empty_diff() {
        // GIVEN identical databases
        let temp_dir = TempDir::new().unwrap();
        let entities = vec!["rust:fn:a:path:1-10", "rust:fn:b:path:11-20"];
        let base_path = create_test_db_with_entities(&temp_dir, "base.db", entities.clone()).await;
        let live_path = create_test_db_with_entities(&temp_dir, "live.db", entities).await;

        let app = create_test_router();

        // WHEN client sends diff request
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "base_db": base_path,
                "live_db": live_path
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 200 with zero changes
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["diff"]["summary"]["added_entity_count"], 0);
        assert_eq!(json["diff"]["summary"]["removed_entity_count"], 0);
        assert_eq!(json["diff"]["summary"]["modified_entity_count"], 0);
    }

    /// REQ-HTTP-DIFF-003.3: Response includes all required fields
    #[tokio::test]
    async fn test_response_contains_all_required_fields() {
        // GIVEN valid databases
        let temp_dir = TempDir::new().unwrap();
        let base_path = create_test_db(&temp_dir, "base.db").await;
        let live_path = create_test_db(&temp_dir, "live.db").await;

        let app = create_test_router();

        // WHEN client sends diff request
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "base_db": base_path,
                "live_db": live_path
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // THEN all required fields must be present
        assert!(json.get("success").is_some(), "Missing 'success' field");
        assert!(json.get("endpoint").is_some(), "Missing 'endpoint' field");
        assert!(json.get("diff").is_some(), "Missing 'diff' field");
        assert!(json["diff"].get("summary").is_some(), "Missing 'diff.summary' field");
        assert!(json["diff"].get("entity_changes").is_some(), "Missing 'diff.entity_changes' field");
        assert!(json["diff"].get("edge_changes").is_some(), "Missing 'diff.edge_changes' field");
        assert!(json.get("blast_radius").is_some(), "Missing 'blast_radius' field");
        assert!(json.get("visualization").is_some(), "Missing 'visualization' field");
        assert!(json.get("token_estimate").is_some(), "Missing 'token_estimate' field");
    }

    /// REQ-HTTP-DIFF-003.4: Token estimation is reasonable
    #[tokio::test]
    async fn test_token_estimate_is_reasonable() {
        // GIVEN valid databases with some entities
        let temp_dir = TempDir::new().unwrap();
        let base_path = create_test_db_with_entities(
            &temp_dir, "base.db",
            vec!["rust:fn:a:path:1-10", "rust:fn:b:path:11-20"]
        ).await;
        let live_path = create_test_db_with_entities(
            &temp_dir, "live.db",
            vec!["rust:fn:a:path:1-10", "rust:fn:c:path:21-30"]  // b removed, c added
        ).await;

        let app = create_test_router();

        // WHEN client sends diff request
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "base_db": base_path,
                "live_db": live_path
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // THEN token estimate should be at least minimum (100)
        let token_estimate = json["token_estimate"].as_u64().unwrap();
        assert!(token_estimate >= 100, "Token estimate {} is below minimum 100", token_estimate);
    }
}
```

### Acceptance Criteria

- [ ] Diff with changes returns complete response with all categories
- [ ] Diff with no changes returns zero counts
- [ ] Response contains all required fields (success, endpoint, diff, blast_radius, visualization, token_estimate)
- [ ] Token estimate is calculated and >= 100
- [ ] Small diff completes within 500ms p99
- [ ] Response size does not exceed 10MB

---

## REQ-HTTP-DIFF-004: Query Parameter Handling

### Problem Statement

The endpoint accepts optional query parameters to customize diff behavior. The `max_hops` parameter controls blast radius depth.

### Specification

#### REQ-HTTP-DIFF-004.1: Default max_hops When Not Provided

```
WHEN client sends POST to /diff-analysis-compare-snapshots
  WITH no query parameters
  AND valid request body
THEN SHALL use default max_hops value of 2
  AND SHALL compute blast radius to depth 2
  AND SHALL include max_depth_reached in blast_radius response
  AND SHALL NOT return error
```

#### REQ-HTTP-DIFF-004.2: Custom max_hops Value

```
WHEN client sends POST to /diff-analysis-compare-snapshots?max_hops=5
  WITH valid request body
THEN SHALL parse max_hops as integer 5
  AND SHALL compute blast radius to depth 5
  AND SHALL return blast_radius.max_depth_reached <= 5
  AND SHALL NOT return error
```

#### REQ-HTTP-DIFF-004.3: max_hops Zero Value

```
WHEN client sends POST to /diff-analysis-compare-snapshots?max_hops=0
  WITH valid request body
THEN SHALL accept max_hops=0
  AND SHALL compute blast radius to depth 0 (changed entities only)
  AND SHALL return blast_radius.affected_by_distance as empty or minimal
```

#### REQ-HTTP-DIFF-004.4: max_hops Maximum Boundary

```
WHEN client sends POST to /diff-analysis-compare-snapshots?max_hops=10
  WITH valid request body
THEN SHALL accept max_hops=10 (maximum allowed)
  AND SHALL compute blast radius to maximum depth 10
```

#### REQ-HTTP-DIFF-004.5: max_hops Exceeds Maximum

```
WHEN client sends POST to /diff-analysis-compare-snapshots?max_hops=100
  WITH valid request body
THEN SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body:
    {
      "error": "max_hops cannot exceed 10",
      "code": "MAX_HOPS_EXCEEDED"
    }
```

#### REQ-HTTP-DIFF-004.6: Invalid max_hops Value (Non-Integer)

```
WHEN client sends POST to /diff-analysis-compare-snapshots?max_hops=abc
  WITH valid request body
THEN SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body:
    {
      "error": "Invalid max_hops: must be a non-negative integer",
      "code": "INVALID_MAX_HOPS"
    }
```

#### REQ-HTTP-DIFF-004.7: Negative max_hops Value

```
WHEN client sends POST to /diff-analysis-compare-snapshots?max_hops=-1
  WITH valid request body
THEN SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body:
    {
      "error": "Invalid max_hops: must be a non-negative integer",
      "code": "INVALID_MAX_HOPS"
    }
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_http_diff_004_tests {
    use axum::{body::Body, http::{Request, StatusCode, header}};
    use tower::ServiceExt;
    use serde_json::{json, Value};
    use tempfile::TempDir;

    /// REQ-HTTP-DIFF-004.1: Default max_hops when not provided
    #[tokio::test]
    async fn test_default_max_hops_is_two() {
        // GIVEN valid databases
        let temp_dir = TempDir::new().unwrap();
        let base_path = create_test_db(&temp_dir, "base.db").await;
        let live_path = create_test_db(&temp_dir, "live.db").await;

        let app = create_test_router();

        // WHEN client sends request without max_hops parameter
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots")  // No query params
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "base_db": base_path,
                "live_db": live_path
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // THEN max_depth_reached should not exceed default of 2
        let max_depth = json["blast_radius"]["max_depth_reached"].as_u64().unwrap_or(0);
        assert!(max_depth <= 2, "Default max_hops should be 2, got depth {}", max_depth);
    }

    /// REQ-HTTP-DIFF-004.2: Custom max_hops value
    #[tokio::test]
    async fn test_custom_max_hops_value() {
        // GIVEN valid databases
        let temp_dir = TempDir::new().unwrap();
        let base_path = create_test_db(&temp_dir, "base.db").await;
        let live_path = create_test_db(&temp_dir, "live.db").await;

        let app = create_test_router();

        // WHEN client sends request with max_hops=5
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots?max_hops=5")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "base_db": base_path,
                "live_db": live_path
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should accept and use max_hops=5
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        let max_depth = json["blast_radius"]["max_depth_reached"].as_u64().unwrap_or(0);
        assert!(max_depth <= 5, "max_hops=5 should limit depth to 5");
    }

    /// REQ-HTTP-DIFF-004.5: max_hops exceeds maximum returns 400
    #[tokio::test]
    async fn test_max_hops_exceeds_maximum_returns_400() {
        // GIVEN valid databases
        let temp_dir = TempDir::new().unwrap();
        let base_path = create_test_db(&temp_dir, "base.db").await;
        let live_path = create_test_db(&temp_dir, "live.db").await;

        let app = create_test_router();

        // WHEN client sends request with max_hops=100 (exceeds max of 10)
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots?max_hops=100")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "base_db": base_path,
                "live_db": live_path
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 400 with MAX_HOPS_EXCEEDED code
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "MAX_HOPS_EXCEEDED");
    }

    /// REQ-HTTP-DIFF-004.6: Invalid max_hops (non-integer) returns 400
    #[tokio::test]
    async fn test_invalid_max_hops_non_integer_returns_400() {
        // GIVEN valid databases
        let temp_dir = TempDir::new().unwrap();
        let base_path = create_test_db(&temp_dir, "base.db").await;
        let live_path = create_test_db(&temp_dir, "live.db").await;

        let app = create_test_router();

        // WHEN client sends request with invalid max_hops
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots?max_hops=abc")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "base_db": base_path,
                "live_db": live_path
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 400 with INVALID_MAX_HOPS code
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "INVALID_MAX_HOPS");
    }

    /// REQ-HTTP-DIFF-004.7: Negative max_hops returns 400
    #[tokio::test]
    async fn test_negative_max_hops_returns_400() {
        // GIVEN valid databases
        let temp_dir = TempDir::new().unwrap();
        let base_path = create_test_db(&temp_dir, "base.db").await;
        let live_path = create_test_db(&temp_dir, "live.db").await;

        let app = create_test_router();

        // WHEN client sends request with negative max_hops
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots?max_hops=-1")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "base_db": base_path,
                "live_db": live_path
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN should return 400 with INVALID_MAX_HOPS code
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
```

### Acceptance Criteria

- [ ] Missing max_hops defaults to 2
- [ ] max_hops=5 is accepted and used
- [ ] max_hops=0 is accepted (minimal blast radius)
- [ ] max_hops=10 is accepted (maximum)
- [ ] max_hops=100 returns 400 with MAX_HOPS_EXCEEDED
- [ ] max_hops=abc returns 400 with INVALID_MAX_HOPS
- [ ] max_hops=-1 returns 400 with INVALID_MAX_HOPS

---

## REQ-HTTP-DIFF-005: Error Response Format

### Problem Statement

All error responses must follow a consistent structure for predictable client handling. The format must include both human-readable messages and machine-parseable codes.

### Specification

#### REQ-HTTP-DIFF-005.1: Consistent Error Structure

```
WHEN endpoint returns any error response
THEN SHALL return JSON body with structure:
    {
      "error": "<human-readable error message>",
      "code": "<MACHINE_READABLE_ERROR_CODE>"
    }
  AND SHALL include Content-Type: application/json header
  AND SHALL NOT include additional unexpected fields
  AND "error" SHALL be non-empty string
  AND "code" SHALL be SCREAMING_SNAKE_CASE format
```

#### REQ-HTTP-DIFF-005.2: Error Code Enumeration

```
WHEN endpoint returns error response
THEN "code" field SHALL be one of:
  - "MISSING_BASE_DB" (400)
  - "MISSING_LIVE_DB" (400)
  - "INVALID_BASE_DB_PATH" (400)
  - "INVALID_LIVE_DB_PATH" (400)
  - "INVALID_DB_PATH_FORMAT" (400)
  - "INVALID_JSON" (400)
  - "UNSUPPORTED_MEDIA_TYPE" (415)
  - "BASE_DB_NOT_FOUND" (404)
  - "LIVE_DB_NOT_FOUND" (404)
  - "DB_CONNECTION_TIMEOUT" (504)
  - "DB_PERMISSION_DENIED" (403)
  - "MAX_HOPS_EXCEEDED" (400)
  - "INVALID_MAX_HOPS" (400)
  - "DIFF_COMPUTATION_FAILED" (500)
  - "INTERNAL_ERROR" (500)
```

#### REQ-HTTP-DIFF-005.3: HTTP Status Code Mapping

```
WHEN endpoint returns error with specific code
THEN SHALL map to correct HTTP status:
  - 400 Bad Request: validation errors (MISSING_*, INVALID_*, MAX_HOPS_*)
  - 403 Forbidden: permission errors (DB_PERMISSION_DENIED)
  - 404 Not Found: resource errors (*_NOT_FOUND)
  - 415 Unsupported Media Type: content type errors
  - 500 Internal Server Error: computation/system errors
  - 504 Gateway Timeout: timeout errors
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_http_diff_005_tests {
    use axum::{body::Body, http::{Request, StatusCode, header}};
    use tower::ServiceExt;
    use serde_json::{json, Value};

    /// REQ-HTTP-DIFF-005.1: Consistent error structure
    #[tokio::test]
    async fn test_error_response_has_consistent_structure() {
        // GIVEN a request that will cause an error
        let app = create_test_router();

        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "live_db": "rocksdb:test/live.db"
                // Missing base_db - will cause error
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // THEN response should have correct error structure
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // Must have "error" field as non-empty string
        assert!(json.get("error").is_some(), "Missing 'error' field");
        assert!(json["error"].is_string(), "'error' must be string");
        assert!(!json["error"].as_str().unwrap().is_empty(), "'error' must not be empty");

        // Must have "code" field in SCREAMING_SNAKE_CASE
        assert!(json.get("code").is_some(), "Missing 'code' field");
        assert!(json["code"].is_string(), "'code' must be string");
        let code = json["code"].as_str().unwrap();
        assert!(
            code.chars().all(|c| c.is_uppercase() || c == '_'),
            "'code' must be SCREAMING_SNAKE_CASE, got: {}", code
        );
    }

    /// REQ-HTTP-DIFF-005.2: All error codes are from enumeration
    #[tokio::test]
    async fn test_error_codes_match_enumeration() {
        let valid_codes = vec![
            "MISSING_BASE_DB",
            "MISSING_LIVE_DB",
            "INVALID_BASE_DB_PATH",
            "INVALID_LIVE_DB_PATH",
            "INVALID_DB_PATH_FORMAT",
            "INVALID_JSON",
            "UNSUPPORTED_MEDIA_TYPE",
            "BASE_DB_NOT_FOUND",
            "LIVE_DB_NOT_FOUND",
            "DB_CONNECTION_TIMEOUT",
            "DB_PERMISSION_DENIED",
            "MAX_HOPS_EXCEEDED",
            "INVALID_MAX_HOPS",
            "DIFF_COMPUTATION_FAILED",
            "INTERNAL_ERROR",
        ];

        // Test MISSING_BASE_DB
        let app = create_test_router();
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "live_db": "rocksdb:test/live.db"
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        let code = json["code"].as_str().unwrap();
        assert!(
            valid_codes.contains(&code),
            "Unknown error code: {}. Valid codes: {:?}", code, valid_codes
        );
    }

    /// REQ-HTTP-DIFF-005.3: HTTP status codes are correct
    #[tokio::test]
    async fn test_error_status_codes_are_correct() {
        let app = create_test_router();

        // Test 400 for missing field
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "live_db": "rocksdb:test/live.db"
            }).to_string()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Test 404 for not found
        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({
                "base_db": "rocksdb:nonexistent/base.db",
                "live_db": "rocksdb:test/live.db"
            }).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        // Should be 404 NOT_FOUND for database not found
        assert!(
            response.status() == StatusCode::NOT_FOUND ||
            response.status() == StatusCode::BAD_REQUEST,
            "Expected 404 or 400, got {}", response.status()
        );
    }
}
```

### Acceptance Criteria

- [ ] All error responses have "error" and "code" fields
- [ ] "error" field is always a non-empty string
- [ ] "code" field is always SCREAMING_SNAKE_CASE
- [ ] Error codes match the enumerated list
- [ ] HTTP status codes match the specification mapping
- [ ] Content-Type is always application/json for errors

---

## Implementation Guide

### Handler Function Signature

Following the 4-word naming convention:

```rust
/// Handle diff analysis compare snapshots request
///
/// # 4-Word Name: handle_diff_analysis_compare_snapshots
///
/// # Contract
/// - Precondition: Valid JSON body with base_db and live_db
/// - Postcondition: Returns unified diff with blast radius and visualization
/// - Performance: <2000ms for codebases up to 1000 entities
/// - Error Handling: Returns structured error JSON with code field
///
/// # URL Pattern
/// - Endpoint: POST /diff-analysis-compare-snapshots?max_hops=N
/// - Default max_hops: 2
pub async fn handle_diff_analysis_compare_snapshots(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<DiffAnalysisQueryParamsStruct>,
    Json(body): Json<DiffAnalysisRequestBodyStruct>,
) -> impl IntoResponse {
    // Implementation
}
```

### Request/Response Types

```rust
/// Request body for diff analysis endpoint
///
/// # 4-Word Name: DiffAnalysisRequestBodyStruct
#[derive(Debug, Deserialize)]
pub struct DiffAnalysisRequestBodyStruct {
    /// Path to base (before) database
    pub base_db: String,
    /// Path to live (after) database
    pub live_db: String,
}

/// Query parameters for diff analysis endpoint
///
/// # 4-Word Name: DiffAnalysisQueryParamsStruct
#[derive(Debug, Deserialize)]
pub struct DiffAnalysisQueryParamsStruct {
    /// Maximum hops for blast radius (default: 2, max: 10)
    #[serde(default = "default_max_hops")]
    pub max_hops: u32,
}

fn default_max_hops() -> u32 {
    2
}

/// Response payload for diff analysis endpoint
///
/// # 4-Word Name: DiffAnalysisResponsePayloadStruct
#[derive(Debug, Serialize)]
pub struct DiffAnalysisResponsePayloadStruct {
    pub success: bool,
    pub endpoint: String,
    pub diff: DiffResultDataPayload,
    pub blast_radius: BlastRadiusResultPayload,
    pub visualization: VisualizationGraphDataPayload,
    pub token_estimate: usize,
}

/// Error response for diff analysis endpoint
///
/// # 4-Word Name: DiffAnalysisErrorResponseStruct
#[derive(Debug, Serialize)]
pub struct DiffAnalysisErrorResponseStruct {
    pub error: String,
    pub code: String,
}
```

### Route Registration

Add to `route_definition_builder_module.rs`:

```rust
use axum::routing::post;

.route(
    "/diff-analysis-compare-snapshots",
    post(diff_analysis_compare_handler::handle_diff_analysis_compare_snapshots)
)
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
- [ ] Token estimation is included in response
- [ ] Handler follows existing patterns in codebase

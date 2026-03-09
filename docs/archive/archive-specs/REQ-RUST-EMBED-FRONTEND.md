# REQ-RUST-EMBED-FRONTEND: Static Frontend Embedding Specification

## Document Metadata

| Field | Value |
|-------|-------|
| Requirement ID | REQ-RUST-EMBED-FRONTEND |
| Title | Rust-Embed Frontend Static File Integration |
| Version | 1.0.0 |
| Status | Draft |
| Created | 2026-01-23 |
| Author | Executable Specs Agent |
| Phase | 2.6 |

---

## 1. Problem Statement

### 1.1 What Pain Exists

The Parseltongue HTTP server (`pt08-http-code-query-server`) currently serves only API endpoints. The React frontend is developed separately and requires:

1. **Separate deployment complexity**: Users must serve the frontend via a separate process (e.g., `npm run dev`, nginx, or another web server)
2. **CORS configuration headaches**: Cross-origin requests between frontend and backend require careful configuration
3. **Operational overhead**: Two processes to manage instead of one
4. **Distribution friction**: Users cannot simply run a single binary to get both API and UI

### 1.2 Who Feels This Pain

- **End users** who want a simple `./parseltongue pt08-http-code-query-server` command to get a fully functional UI
- **DevOps engineers** who want simpler deployment (single binary, single port)
- **Developers** who want to avoid CORS issues during development
- **LLM agents** that need a single URL for both API and UI access

### 1.3 What Would Success Look Like

A single Rust binary that:
- Serves the React SPA at the root URL (`/`)
- Serves static assets with correct MIME types and caching headers
- Handles SPA client-side routing via fallback to `index.html`
- Preserves all 15+ existing API endpoints without modification
- Has zero runtime file system dependencies (all assets embedded at compile time)

---

## 2. Architecture Overview

### 2.1 Technology Choice: rust-embed

The `rust-embed` crate provides compile-time embedding of static files into the Rust binary.

```toml
# Cargo.toml addition
[dependencies]
rust-embed = "8.2"
mime_guess = "2.0"
```

### 2.2 Directory Structure

```
parseltongue-dependency-graph-generator/
├── frontend/
│   ├── dist/                    # Build output (embedded into binary)
│   │   ├── index.html           # Main HTML entry point
│   │   ├── assets/
│   │   │   ├── index-{hash}.js  # Bundled JavaScript
│   │   │   ├── index-{hash}.css # Bundled CSS
│   │   │   └── ...              # Other assets (images, fonts)
│   │   └── vite.svg             # Favicon/icon
│   └── package.json
└── crates/
    └── pt08-http-code-query-server/
        ├── Cargo.toml
        ├── build.rs             # Build script for frontend compilation
        └── src/
            ├── static_file_embed_module.rs  # NEW: rust-embed integration
            └── route_definition_builder_module.rs  # Updated: add static routes
```

### 2.3 Route Precedence

```
Request → Router
    ├── /server-health-check-status → API Handler (priority 1)
    ├── /workspace-list-all → API Handler (priority 1)
    ├── /... (14 more API routes) → API Handler (priority 1)
    ├── /assets/* → Static File Handler (priority 2)
    ├── / → index.html (priority 3)
    └── /* (anything else) → SPA Fallback to index.html (priority 4)
```

---

## 3. REQ-EMBED-001: Static File Serving

### 3.1 Problem Statement

The server must serve static files from the embedded `frontend/dist/` directory with correct MIME types and efficient caching.

### 3.2 Specification

#### REQ-EMBED-001.1: Root Path Serves index.html

```
WHEN client sends GET request to path "/"
  WITH Accept header including "text/html"
THEN SHALL respond with HTTP 200 OK
  AND SHALL set Content-Type header to "text/html; charset=utf-8"
  AND SHALL return body containing contents of embedded "index.html"
  AND SHALL NOT set Cache-Control header (or set to "no-cache" for HTML)
```

#### REQ-EMBED-001.2: JavaScript Asset Serving

```
WHEN client sends GET request to path "/assets/index-{hash}.js"
  WITH {hash} being any valid content hash string
  AND embedded file "assets/index-{hash}.js" exists
THEN SHALL respond with HTTP 200 OK
  AND SHALL set Content-Type header to "application/javascript; charset=utf-8"
  AND SHALL return body containing file contents
  AND SHALL set Cache-Control header to "public, max-age=31536000, immutable"
```

#### REQ-EMBED-001.3: CSS Asset Serving

```
WHEN client sends GET request to path "/assets/index-{hash}.css"
  WITH {hash} being any valid content hash string
  AND embedded file "assets/index-{hash}.css" exists
THEN SHALL respond with HTTP 200 OK
  AND SHALL set Content-Type header to "text/css; charset=utf-8"
  AND SHALL return body containing file contents
  AND SHALL set Cache-Control header to "public, max-age=31536000, immutable"
```

#### REQ-EMBED-001.4: Image Asset Serving

```
WHEN client sends GET request to path "/assets/{filename}.{ext}"
  WITH {ext} being one of: png, jpg, jpeg, gif, svg, webp, ico
  AND embedded file "assets/{filename}.{ext}" exists
THEN SHALL respond with HTTP 200 OK
  AND SHALL set Content-Type header to appropriate MIME type:
    - "image/png" for .png
    - "image/jpeg" for .jpg, .jpeg
    - "image/gif" for .gif
    - "image/svg+xml" for .svg
    - "image/webp" for .webp
    - "image/x-icon" for .ico
  AND SHALL return body containing file contents
  AND SHALL set Cache-Control header to "public, max-age=31536000, immutable"
```

#### REQ-EMBED-001.5: Font Asset Serving

```
WHEN client sends GET request to path "/assets/{filename}.{ext}"
  WITH {ext} being one of: woff, woff2, ttf, otf, eot
  AND embedded file "assets/{filename}.{ext}" exists
THEN SHALL respond with HTTP 200 OK
  AND SHALL set Content-Type header to appropriate MIME type:
    - "font/woff" for .woff
    - "font/woff2" for .woff2
    - "font/ttf" for .ttf
    - "font/otf" for .otf
    - "application/vnd.ms-fontobject" for .eot
  AND SHALL return body containing file contents
  AND SHALL set Cache-Control header to "public, max-age=31536000, immutable"
```

#### REQ-EMBED-001.6: 404 for Missing Assets

```
WHEN client sends GET request to path "/assets/{any_path}"
  WITH embedded file "assets/{any_path}" NOT existing
THEN SHALL respond with HTTP 404 Not Found
  AND SHALL set Content-Type header to "application/json"
  AND SHALL return JSON body: {"success": false, "error": "Asset not found", "path": "/assets/{any_path}"}
```

### 3.3 Error Conditions

```
WHEN client sends GET request to static file path
  WITH embedded file existing but corrupted/unreadable
THEN SHALL respond with HTTP 500 Internal Server Error
  AND SHALL return JSON body: {"success": false, "error": "Failed to read embedded asset"}
```

### 3.4 Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Response latency (index.html) | < 1ms | Timer around handler |
| Response latency (JS/CSS) | < 1ms | Timer around handler |
| Memory overhead | < 10MB for typical frontend | Measure binary size delta |
| Startup time impact | < 50ms | Benchmark server startup |

### 3.5 Test Template

```rust
// File: crates/pt08-http-code-query-server/tests/static_file_embedding_tests.rs

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use tower::ServiceExt;

/// REQ-EMBED-001.1: Root path serves index.html
///
/// # 4-Word Name: test_root_path_serves_index
#[tokio::test]
async fn test_root_path_serves_index() {
    // GIVEN: Server with embedded static files
    let app = create_test_server_with_embedded_files();

    // WHEN: GET /
    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .header("Accept", "text/html")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns index.html with correct content type
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "text/html; charset=utf-8"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("<!DOCTYPE html>"));
    assert!(body_str.contains("Parseltongue"));
}

/// REQ-EMBED-001.2: JavaScript assets served with correct MIME type
///
/// # 4-Word Name: test_javascript_asset_mime_type
#[tokio::test]
async fn test_javascript_asset_mime_type() {
    // GIVEN: Server with embedded static files
    let app = create_test_server_with_embedded_files();

    // WHEN: GET /assets/index-abc123.js (simulated hash)
    let response = app
        .oneshot(
            Request::builder()
                .uri("/assets/index-abc123.js")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns JavaScript with correct headers
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/javascript; charset=utf-8"
    );
    assert!(
        response.headers().get(header::CACHE_CONTROL).unwrap()
            .to_str().unwrap().contains("max-age=31536000")
    );
}

/// REQ-EMBED-001.3: CSS assets served with correct MIME type
///
/// # 4-Word Name: test_css_asset_mime_type
#[tokio::test]
async fn test_css_asset_mime_type() {
    // GIVEN: Server with embedded static files
    let app = create_test_server_with_embedded_files();

    // WHEN: GET /assets/index-abc123.css
    let response = app
        .oneshot(
            Request::builder()
                .uri("/assets/index-abc123.css")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns CSS with correct headers
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "text/css; charset=utf-8"
    );
}

/// REQ-EMBED-001.6: Missing assets return 404
///
/// # 4-Word Name: test_missing_asset_returns_not_found
#[tokio::test]
async fn test_missing_asset_returns_not_found() {
    // GIVEN: Server with embedded static files
    let app = create_test_server_with_embedded_files();

    // WHEN: GET /assets/nonexistent-file.js
    let response = app
        .oneshot(
            Request::builder()
                .uri("/assets/nonexistent-file.js")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns 404 with JSON error
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], false);
    assert!(json["error"].as_str().unwrap().contains("not found"));
}
```

### 3.6 Acceptance Criteria

- [ ] GET / returns index.html with Content-Type: text/html; charset=utf-8
- [ ] GET /assets/*.js returns JavaScript with Content-Type: application/javascript; charset=utf-8
- [ ] GET /assets/*.css returns CSS with Content-Type: text/css; charset=utf-8
- [ ] GET /assets/*.png returns PNG with Content-Type: image/png
- [ ] GET /assets/*.svg returns SVG with Content-Type: image/svg+xml
- [ ] Hashed assets include Cache-Control: public, max-age=31536000, immutable
- [ ] index.html does NOT have long-term caching (no-cache or no Cache-Control)
- [ ] Missing asset paths return HTTP 404 with JSON error body
- [ ] All static file responses complete in < 1ms

---

## 4. REQ-EMBED-002: SPA Fallback Routing

### 4.1 Problem Statement

Single-Page Applications (SPAs) use client-side routing. When a user navigates to `/workspace-details/abc123` and refreshes, the server receives this request directly. Without SPA fallback, the server returns 404.

### 4.2 Specification

#### REQ-EMBED-002.1: Non-API, Non-Asset Paths Fallback to index.html

```
WHEN client sends GET request to path "/{any_path}"
  WITH {any_path} NOT matching any registered API endpoint
  AND {any_path} NOT starting with "/assets/"
  AND {any_path} NOT being "/"
THEN SHALL respond with HTTP 200 OK
  AND SHALL set Content-Type header to "text/html; charset=utf-8"
  AND SHALL return body containing contents of embedded "index.html"
```

#### REQ-EMBED-002.2: SPA Route Examples

```
WHEN client sends GET request to any of these paths:
  - "/workspace-details"
  - "/workspace-details/abc123"
  - "/entity/rust:fn:main:main_rs:1-10"
  - "/settings"
  - "/help"
  - "/any/deeply/nested/path"
THEN SHALL return index.html with HTTP 200
  AND SHALL NOT return HTTP 404
```

#### REQ-EMBED-002.3: API Routes Are NOT Affected

```
WHEN client sends GET request to path "/server-health-check-status"
THEN SHALL return JSON health check response
  AND SHALL NOT return index.html

WHEN client sends GET request to path "/workspace-list-all"
THEN SHALL return JSON workspace list response
  AND SHALL NOT return index.html

WHEN client sends GET request to path "/code-entities-list-all"
THEN SHALL return JSON entities response
  AND SHALL NOT return index.html
```

#### REQ-EMBED-002.4: Asset Routes Are NOT Affected

```
WHEN client sends GET request to path "/assets/index-abc123.js"
  WITH embedded file existing
THEN SHALL return the JavaScript file
  AND SHALL NOT return index.html

WHEN client sends GET request to path "/assets/missing-file.css"
  WITH embedded file NOT existing
THEN SHALL return HTTP 404
  AND SHALL NOT return index.html
```

### 4.3 Error Conditions

```
WHEN SPA fallback is triggered
  AND index.html is missing from embedded files
THEN SHALL respond with HTTP 500 Internal Server Error
  AND SHALL return JSON body: {"success": false, "error": "index.html not found in embedded files"}
```

### 4.4 Test Template

```rust
// File: crates/pt08-http-code-query-server/tests/spa_fallback_routing_tests.rs

/// REQ-EMBED-002.1: Unknown paths return index.html
///
/// # 4-Word Name: test_spa_fallback_returns_index
#[tokio::test]
async fn test_spa_fallback_returns_index() {
    // GIVEN: Server with SPA fallback enabled
    let app = create_test_server_with_embedded_files();

    // WHEN: GET /workspace-details (SPA route)
    let response = app
        .oneshot(
            Request::builder()
                .uri("/workspace-details")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns index.html, not 404
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "text/html; charset=utf-8"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("<!DOCTYPE html>"));
}

/// REQ-EMBED-002.2: Deeply nested SPA routes work
///
/// # 4-Word Name: test_nested_spa_route_fallback
#[tokio::test]
async fn test_nested_spa_route_fallback() {
    // GIVEN: Server with SPA fallback enabled
    let app = create_test_server_with_embedded_files();

    // WHEN: GET /entity/rust:fn:main:main_rs:1-10 (complex SPA route)
    let response = app
        .oneshot(
            Request::builder()
                .uri("/entity/rust:fn:main:main_rs:1-10")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns index.html
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert!(String::from_utf8(body.to_vec()).unwrap().contains("<!DOCTYPE html>"));
}

/// REQ-EMBED-002.3: API routes take precedence over SPA fallback
///
/// # 4-Word Name: test_api_routes_take_precedence
#[tokio::test]
async fn test_api_routes_take_precedence() {
    // GIVEN: Server with both API routes and SPA fallback
    let app = create_test_server_with_embedded_files();

    // WHEN: GET /server-health-check-status (API endpoint)
    let response = app
        .oneshot(
            Request::builder()
                .uri("/server-health-check-status")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns JSON, not index.html
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response.headers().get(header::CONTENT_TYPE).unwrap()
            .to_str().unwrap().contains("application/json")
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["status"], "ok");
}

/// REQ-EMBED-002.4: Asset routes take precedence
///
/// # 4-Word Name: test_asset_routes_take_precedence
#[tokio::test]
async fn test_asset_routes_take_precedence() {
    // GIVEN: Server with both asset routes and SPA fallback
    let app = create_test_server_with_embedded_files();

    // WHEN: GET /assets/missing-file.css (missing asset)
    let response = app
        .oneshot(
            Request::builder()
                .uri("/assets/missing-file.css")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns 404, not index.html
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    // Verify it's the asset 404, not SPA fallback
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert!(!String::from_utf8(body.to_vec()).unwrap().contains("<!DOCTYPE html>"));
}
```

### 4.5 Acceptance Criteria

- [ ] GET /workspace-details returns index.html with HTTP 200
- [ ] GET /entity/rust:fn:main returns index.html with HTTP 200
- [ ] GET /any/deep/nested/path returns index.html with HTTP 200
- [ ] GET /server-health-check-status returns JSON (NOT index.html)
- [ ] GET /workspace-list-all returns JSON (NOT index.html)
- [ ] GET /assets/missing.js returns 404 (NOT index.html)
- [ ] SPA fallback response time is < 1ms

---

## 5. REQ-EMBED-003: API Route Preservation

### 5.1 Problem Statement

The server has 18+ existing API endpoints that must continue working exactly as before. The static file serving and SPA fallback must NOT break any existing functionality.

### 5.2 Specification

#### REQ-EMBED-003.1: Complete API Endpoint List

```
WHEN rust-embed integration is complete
THEN SHALL preserve all existing API endpoints:

  Core Endpoints:
    - GET  /server-health-check-status
    - GET  /codebase-statistics-overview-summary
    - GET  /api-reference-documentation-help

  Entity Endpoints:
    - GET  /code-entities-list-all
    - GET  /code-entity-detail-view?key={key}
    - GET  /code-entities-search-fuzzy?q={query}

  Graph Query Endpoints:
    - GET  /dependency-edges-list-all
    - GET  /reverse-callers-query-graph?entity={key}
    - GET  /forward-callees-query-graph?entity={key}

  Analysis Endpoints:
    - GET  /blast-radius-impact-analysis?entity={key}&hops={n}
    - GET  /circular-dependency-detection-scan
    - GET  /complexity-hotspots-ranking-view?top={n}
    - GET  /semantic-cluster-grouping-list

  Advanced Endpoints:
    - GET  /smart-context-token-budget?focus={key}&tokens={n}
    - GET  /temporal-coupling-hidden-deps?entity={key}
    - POST /diff-analysis-compare-snapshots

  Workspace Management Endpoints:
    - POST /workspace-create-from-path
    - GET  /workspace-list-all
    - POST /workspace-watch-toggle

  WebSocket Endpoint:
    - GET  /websocket-diff-stream (WebSocket upgrade)
```

#### REQ-EMBED-003.2: Regression Test Contract

```
WHEN server starts with rust-embed integration
THEN SHALL pass all existing integration tests:
  - test_health_endpoint_returns_ok
  - test_stats_returns_entity_counts
  - test_list_all_entities_endpoint
  - test_fuzzy_search_entities
  - test_reverse_callers_returns_deps
  - test_forward_callees_returns_deps
  - test_blast_radius_single_hop
  - test_blast_radius_multi_hop
  - test_circular_dependency_none_found
  - test_circular_dependency_cycle_detected
  - test_complexity_hotspots_ranking_view
  - test_semantic_cluster_grouping_list
  - test_api_reference_documentation_help
  - test_smart_context_token_budget
  - test_temporal_coupling_hidden_deps
  - (and all other existing tests)
```

#### REQ-EMBED-003.3: API Response Format Unchanged

```
WHEN client calls any API endpoint
THEN SHALL return response with same JSON structure as before:
  {
    "success": boolean,
    "endpoint": string,
    "data": object | array,
    "tokens": number (optional)
  }
AND SHALL NOT add, remove, or modify any response fields
AND SHALL NOT change HTTP status codes
AND SHALL NOT change Content-Type headers
```

### 5.3 Test Template

```rust
// File: crates/pt08-http-code-query-server/tests/api_route_preservation_tests.rs

/// REQ-EMBED-003.2: Health check still works after rust-embed
///
/// # 4-Word Name: test_health_check_preserved_after_embed
#[tokio::test]
async fn test_health_check_preserved_after_embed() {
    // GIVEN: Server with rust-embed integration
    let app = create_test_server_with_embedded_files();

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

    // THEN: Returns exact same response as before
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify unchanged response structure
    assert_eq!(json["success"], true);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["endpoint"], "/server-health-check-status");
}

/// REQ-EMBED-003.2: Workspace list still works
///
/// # 4-Word Name: test_workspace_list_preserved_after_embed
#[tokio::test]
async fn test_workspace_list_preserved_after_embed() {
    // GIVEN: Server with rust-embed integration
    let app = create_test_server_with_embedded_files();

    // WHEN: GET /workspace-list-all
    let response = app
        .oneshot(
            Request::builder()
                .uri("/workspace-list-all")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns JSON workspace list (not index.html)
    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["endpoint"], "/workspace-list-all");
    assert!(json["data"]["workspaces"].is_array());
}

/// REQ-EMBED-003.2: POST endpoints still work
///
/// # 4-Word Name: test_post_endpoints_preserved_after_embed
#[tokio::test]
async fn test_post_endpoints_preserved_after_embed() {
    // GIVEN: Server with rust-embed integration
    let app = create_test_server_with_embedded_files();

    // WHEN: POST /workspace-create-from-path
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/workspace-create-from-path")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"path": "/tmp/test"}"#))
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns JSON response (success or error, but not index.html)
    let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));
}

/// REQ-EMBED-003.2: WebSocket upgrade still works
///
/// # 4-Word Name: test_websocket_upgrade_preserved_after_embed
#[tokio::test]
async fn test_websocket_upgrade_preserved_after_embed() {
    // GIVEN: Server with rust-embed integration
    let app = create_test_server_with_embedded_files();

    // WHEN: GET /websocket-diff-stream with upgrade headers
    let response = app
        .oneshot(
            Request::builder()
                .uri("/websocket-diff-stream")
                .header("Connection", "Upgrade")
                .header("Upgrade", "websocket")
                .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
                .header("Sec-WebSocket-Version", "13")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns 101 Switching Protocols (or 400 if missing workspace_id)
    // Not index.html or 404
    assert!(
        response.status() == StatusCode::SWITCHING_PROTOCOLS ||
        response.status() == StatusCode::BAD_REQUEST
    );
}
```

### 5.4 Acceptance Criteria

- [ ] All 18+ existing integration tests pass without modification
- [ ] GET /server-health-check-status returns JSON (not HTML)
- [ ] GET /workspace-list-all returns JSON (not HTML)
- [ ] POST /workspace-create-from-path returns JSON (not HTML)
- [ ] GET /websocket-diff-stream handles WebSocket upgrade (not HTML)
- [ ] No API response structure changes
- [ ] No HTTP status code changes
- [ ] No Content-Type header changes

---

## 6. REQ-EMBED-004: Build Integration

### 6.1 Problem Statement

The frontend must be built before Rust compilation, and the build artifacts must be available for embedding. This requires coordination between npm and cargo build systems.

### 6.2 Specification

#### REQ-EMBED-004.1: Build Script Existence

```
WHEN building pt08-http-code-query-server crate
THEN SHALL execute build.rs build script
  AND build.rs SHALL:
    1. Check if frontend/dist/ directory exists
    2. If missing or empty, print cargo:warning about building frontend first
    3. Set cargo:rerun-if-changed=../../../frontend/dist for incremental builds
```

#### REQ-EMBED-004.2: Frontend Build Command

```
WHEN developer runs frontend build
  WITH command: cd frontend && npm run build
THEN SHALL produce output in frontend/dist/ containing:
  - index.html
  - assets/index-{hash}.js
  - assets/index-{hash}.css
  - (optional) vite.svg or other static assets
AND SHALL exit with code 0 on success
```

#### REQ-EMBED-004.3: Rust-Embed Configuration

```
WHEN pt08-http-code-query-server compiles
THEN SHALL use rust-embed with configuration:

  #[derive(RustEmbed)]
  #[folder = "../../../frontend/dist/"]
  struct StaticAssetEmbedFolder;

AND SHALL embed all files from frontend/dist/ into binary
AND SHALL allow runtime access via StaticAssetEmbedFolder::get(path)
```

#### REQ-EMBED-004.4: Missing Frontend Handling

```
WHEN pt08-http-code-query-server compiles
  WITH frontend/dist/ directory missing or empty
THEN SHALL compile successfully (embedded assets will be empty)
  AND SHALL print cargo:warning="Frontend not built. Run 'cd frontend && npm run build' first."

WHEN server runs with empty embedded assets
  AND client requests "/"
THEN SHALL respond with HTTP 503 Service Unavailable
  AND SHALL return JSON body: {"success": false, "error": "Frontend not built. Run build command."}
```

#### REQ-EMBED-004.5: Development Workflow

```
WHEN developer wants to build complete application
THEN SHALL run these commands in order:
  1. cd frontend && npm install && npm run build
  2. cd .. && cargo build --release -p pt08-http-code-query-server

AND resulting binary SHALL contain embedded frontend assets
AND binary SHALL be self-contained (no runtime file dependencies)
```

### 6.3 Test Template

```rust
// File: crates/pt08-http-code-query-server/tests/build_integration_tests.rs

/// REQ-EMBED-004.3: Embedded assets are accessible
///
/// # 4-Word Name: test_embedded_assets_are_accessible
#[test]
fn test_embedded_assets_are_accessible() {
    use pt08_http_code_query_server::static_file_embed_module::StaticAssetEmbedFolder;
    use rust_embed::RustEmbed;

    // GIVEN: Compiled binary with embedded assets

    // WHEN: Accessing index.html
    let index_html = StaticAssetEmbedFolder::get("index.html");

    // THEN: File exists and contains expected content
    assert!(index_html.is_some(), "index.html should be embedded");

    let content = std::str::from_utf8(index_html.unwrap().data.as_ref()).unwrap();
    assert!(content.contains("<!DOCTYPE html>"));
}

/// REQ-EMBED-004.3: Asset iteration works
///
/// # 4-Word Name: test_embedded_asset_iteration_works
#[test]
fn test_embedded_asset_iteration_works() {
    use pt08_http_code_query_server::static_file_embed_module::StaticAssetEmbedFolder;
    use rust_embed::RustEmbed;

    // GIVEN: Compiled binary with embedded assets

    // WHEN: Iterating all embedded files
    let files: Vec<_> = StaticAssetEmbedFolder::iter().collect();

    // THEN: Contains expected files
    assert!(files.contains(&std::borrow::Cow::Borrowed("index.html")));
    assert!(files.iter().any(|f| f.starts_with("assets/")));
}

/// REQ-EMBED-004.4: Server handles missing frontend gracefully
///
/// # 4-Word Name: test_missing_frontend_returns_error
#[tokio::test]
async fn test_missing_frontend_returns_error() {
    // GIVEN: Server configured with empty embedded assets (simulated)
    let app = create_test_server_with_empty_assets();

    // WHEN: GET /
    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // THEN: Returns 503 with helpful error message
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], false);
    assert!(json["error"].as_str().unwrap().contains("Frontend not built"));
}
```

### 6.4 Build Script Implementation

```rust
// File: crates/pt08-http-code-query-server/build.rs

fn main() {
    // Tell Cargo to rerun if frontend dist changes
    println!("cargo:rerun-if-changed=../../../frontend/dist/");

    let frontend_dist = std::path::Path::new("../../../frontend/dist");

    if !frontend_dist.exists() || !frontend_dist.join("index.html").exists() {
        println!("cargo:warning=Frontend not built. Run 'cd frontend && npm run build' first.");
        println!("cargo:warning=The server will return 503 for static file requests until frontend is built.");
    }
}
```

### 6.5 Acceptance Criteria

- [ ] build.rs exists and runs during cargo build
- [ ] build.rs warns if frontend/dist/ is missing
- [ ] cargo:rerun-if-changed is set for incremental builds
- [ ] `npm run build` in frontend/ produces dist/ directory
- [ ] rust-embed successfully embeds all dist/ files
- [ ] StaticAssetEmbedFolder::get("index.html") returns content
- [ ] Server returns 503 with helpful message if frontend not built
- [ ] Complete build workflow documented and working

---

## 7. Implementation Guide

### 7.1 New Files to Create

| File | Purpose |
|------|---------|
| `crates/pt08-http-code-query-server/build.rs` | Build script for frontend detection |
| `crates/pt08-http-code-query-server/src/static_file_embed_module.rs` | rust-embed struct and handlers |

### 7.2 Files to Modify

| File | Modification |
|------|--------------|
| `crates/pt08-http-code-query-server/Cargo.toml` | Add rust-embed and mime_guess dependencies |
| `crates/pt08-http-code-query-server/src/lib.rs` | Export static_file_embed_module |
| `crates/pt08-http-code-query-server/src/route_definition_builder_module.rs` | Add static file routes with fallback |

### 7.3 Route Registration Order

```rust
// In route_definition_builder_module.rs

pub fn build_complete_router_instance(state: SharedApplicationStateContainer) -> Router {
    Router::new()
        // 1. API routes FIRST (highest priority)
        .route("/server-health-check-status", get(health_handler))
        .route("/workspace-list-all", get(workspace_list_handler))
        // ... all other API routes ...

        // 2. Static asset routes SECOND
        .route("/assets/*path", get(serve_embedded_asset_handler))

        // 3. Root path THIRD
        .route("/", get(serve_index_html_handler))

        // 4. SPA fallback LAST (catch-all)
        .fallback(get(serve_spa_fallback_handler))

        .with_state(state)
}
```

### 7.4 Handler Implementation Pattern

```rust
// In static_file_embed_module.rs

use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../../frontend/dist/"]
pub struct StaticAssetEmbedFolder;

/// Serve index.html for root path
///
/// # 4-Word Name: serve_index_html_handler
pub async fn serve_index_html_handler() -> Response {
    match StaticAssetEmbedFolder::get("index.html") {
        Some(content) => {
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                content.data.to_vec(),
            ).into_response()
        }
        None => {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                [(header::CONTENT_TYPE, "application/json")],
                r#"{"success":false,"error":"Frontend not built. Run 'cd frontend && npm run build' first."}"#,
            ).into_response()
        }
    }
}

/// Serve static assets from /assets/* path
///
/// # 4-Word Name: serve_embedded_asset_handler
pub async fn serve_embedded_asset_handler(Path(path): Path<String>) -> Response {
    let asset_path = format!("assets/{}", path);

    match StaticAssetEmbedFolder::get(&asset_path) {
        Some(content) => {
            let mime_type = mime_guess::from_path(&asset_path)
                .first_or_octet_stream()
                .to_string();

            let cache_control = if path.contains('-') {
                // Hashed filename - cache forever
                "public, max-age=31536000, immutable"
            } else {
                "public, max-age=3600"
            };

            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, mime_type),
                    (header::CACHE_CONTROL, cache_control.to_string()),
                ],
                content.data.to_vec(),
            ).into_response()
        }
        None => {
            (
                StatusCode::NOT_FOUND,
                [(header::CONTENT_TYPE, "application/json")],
                format!(r#"{{"success":false,"error":"Asset not found","path":"/assets/{}"}}"#, path),
            ).into_response()
        }
    }
}

/// SPA fallback - return index.html for unknown routes
///
/// # 4-Word Name: serve_spa_fallback_handler
pub async fn serve_spa_fallback_handler() -> Response {
    serve_index_html_handler().await
}
```

---

## 8. Quality Checklist

Before implementation is complete, verify:

- [ ] All quantities are specific and measurable
- [ ] All behaviors are testable
- [ ] Error conditions are specified
- [ ] Performance boundaries are defined (< 1ms response, < 10MB overhead)
- [ ] Test templates provided in Rust
- [ ] Acceptance criteria are binary (pass/fail)
- [ ] No ambiguous language remains
- [ ] 4-word naming convention followed for all functions
- [ ] Route precedence is clearly defined
- [ ] Build integration workflow documented
- [ ] All 18+ existing API endpoints preserved
- [ ] SPA fallback does not interfere with API routes

---

## 9. Appendix: MIME Type Reference

| Extension | MIME Type |
|-----------|-----------|
| .html | text/html; charset=utf-8 |
| .js | application/javascript; charset=utf-8 |
| .css | text/css; charset=utf-8 |
| .json | application/json |
| .png | image/png |
| .jpg, .jpeg | image/jpeg |
| .gif | image/gif |
| .svg | image/svg+xml |
| .webp | image/webp |
| .ico | image/x-icon |
| .woff | font/woff |
| .woff2 | font/woff2 |
| .ttf | font/ttf |
| .otf | font/otf |

---

## 10. Appendix: Full API Endpoint Reference

For regression testing, here is the complete list of API endpoints from `route_definition_builder_module.rs`:

```rust
// Core endpoints
GET  /server-health-check-status
GET  /codebase-statistics-overview-summary
GET  /api-reference-documentation-help

// Entity endpoints
GET  /code-entities-list-all
GET  /code-entity-detail-view
GET  /code-entities-search-fuzzy

// Graph query endpoints
GET  /reverse-callers-query-graph
GET  /forward-callees-query-graph
GET  /dependency-edges-list-all
GET  /blast-radius-impact-analysis
GET  /circular-dependency-detection-scan
GET  /complexity-hotspots-ranking-view
GET  /semantic-cluster-grouping-list
GET  /smart-context-token-budget
GET  /temporal-coupling-hidden-deps

// Diff analysis
POST /diff-analysis-compare-snapshots

// Workspace management
POST /workspace-create-from-path
GET  /workspace-list-all
POST /workspace-watch-toggle

// WebSocket
GET  /websocket-diff-stream
```

Each endpoint must return JSON with `Content-Type: application/json` and must NOT be affected by the SPA fallback.

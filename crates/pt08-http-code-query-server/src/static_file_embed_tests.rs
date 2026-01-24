//! Static file embedding integration tests
//!
//! # 4-Word Naming: static_file_embed_tests
//!
//! Tests for rust-embed integration per REQ-RUST-EMBED-FRONTEND specification.
//! Phase 2.6: Rust-embed Integration
//!
//! ## Test Categories
//! - REQ-EMBED-001: Static File Serving
//! - REQ-EMBED-002: SPA Fallback Routing
//! - REQ-EMBED-003: API Route Preservation

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
    };
    use tower::ServiceExt;

    use crate::http_server_startup_runner::SharedApplicationStateContainer;
    use crate::route_definition_builder_module::build_complete_router_instance;
    use parseltongue_core::storage::CozoDbStorage;

    // =========================================================================
    // REQ-EMBED-001: Static File Serving
    // =========================================================================

    /// REQ-EMBED-001.1: Root path serves index.html
    ///
    /// # 4-Word Name: test_root_path_serves_index_html
    ///
    /// WHEN client sends GET request to path "/"
    /// THEN SHALL respond with HTTP 200 OK
    ///   AND SHALL set Content-Type header to "text/html; charset=utf-8"
    ///   AND SHALL return body containing contents of embedded "index.html"
    #[tokio::test]
    async fn test_root_path_serves_index_html() {
        // GIVEN: Server with embedded static files
        let app = create_test_server_with_embedded_assets().await;

        // WHEN: GET /
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("Accept", "text/html")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // THEN: Returns index.html with correct content type
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("<!DOCTYPE html>"));
        assert!(body_str.contains("Parseltongue"));
    }

    /// REQ-EMBED-001.2: JavaScript assets served with correct MIME type
    ///
    /// # 4-Word Name: test_javascript_asset_correct_content_type
    ///
    /// WHEN client sends GET request to path "/assets/index-{hash}.js"
    /// THEN SHALL respond with HTTP 200 OK
    ///   AND SHALL set Content-Type header to "application/javascript; charset=utf-8"
    #[tokio::test]
    async fn test_javascript_asset_correct_content_type() {
        // GIVEN: Server with embedded static files
        let app = create_test_server_with_embedded_assets().await;

        // Get the actual JS filename from embedded assets
        use crate::static_file_embed_module::StaticAssetEmbedFolder;
        use rust_embed::Embed;

        let js_file = StaticAssetEmbedFolder::iter()
            .find(|f| f.starts_with("assets/") && f.ends_with(".js"))
            .expect("JS file should exist in embedded assets");

        // Remove 'assets/' prefix for request path
        let js_path = js_file.strip_prefix("assets/").unwrap();

        // WHEN: GET /assets/{actual-js-file}
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/assets/{}", js_path))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // THEN: Returns JavaScript with correct headers
        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(
            content_type.contains("application/javascript")
                || content_type.contains("text/javascript")
        );
    }

    /// REQ-EMBED-001.3: CSS assets served with correct MIME type
    ///
    /// # 4-Word Name: test_css_asset_correct_content_type
    ///
    /// WHEN client sends GET request to path "/assets/index-{hash}.css"
    /// THEN SHALL respond with HTTP 200 OK
    ///   AND SHALL set Content-Type header to "text/css; charset=utf-8"
    #[tokio::test]
    async fn test_css_asset_correct_content_type() {
        // GIVEN: Server with embedded static files
        let app = create_test_server_with_embedded_assets().await;

        // Get the actual CSS filename from embedded assets
        use crate::static_file_embed_module::StaticAssetEmbedFolder;
        use rust_embed::Embed;

        let css_file = StaticAssetEmbedFolder::iter()
            .find(|f| f.starts_with("assets/") && f.ends_with(".css"))
            .expect("CSS file should exist in embedded assets");

        // Remove 'assets/' prefix for request path
        let css_path = css_file.strip_prefix("assets/").unwrap();

        // WHEN: GET /assets/{actual-css-file}
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/assets/{}", css_path))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // THEN: Returns CSS with correct headers
        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(content_type.contains("text/css"));
    }

    /// REQ-EMBED-001.6: Missing assets return 404 with JSON error
    ///
    /// # 4-Word Name: test_missing_asset_returns_404
    ///
    /// WHEN client sends GET request to path "/assets/{any_path}"
    ///   WITH embedded file "assets/{any_path}" NOT existing
    /// THEN SHALL respond with HTTP 404 Not Found
    ///   AND SHALL set Content-Type header to "application/json"
    #[tokio::test]
    async fn test_missing_asset_returns_404() {
        // GIVEN: Server with embedded static files
        let app = create_test_server_with_embedded_assets().await;

        // WHEN: GET /assets/nonexistent-file.js
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/assets/nonexistent-file.js")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // THEN: Returns 404 with JSON error
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert!(json["error"].as_str().unwrap().to_lowercase().contains("not found"));
    }

    /// REQ-EMBED-001.2/001.3: Hashed assets include immutable cache headers
    ///
    /// # 4-Word Name: test_cache_headers_for_hashed_assets
    ///
    /// WHEN client sends GET request to hashed asset path
    /// THEN SHALL set Cache-Control header to "public, max-age=31536000, immutable"
    #[tokio::test]
    async fn test_cache_headers_for_hashed_assets() {
        // GIVEN: Server with embedded static files
        let app = create_test_server_with_embedded_assets().await;

        // Get the actual JS filename from embedded assets (hashed)
        use crate::static_file_embed_module::StaticAssetEmbedFolder;
        use rust_embed::Embed;

        let js_file = StaticAssetEmbedFolder::iter()
            .find(|f| f.starts_with("assets/") && f.ends_with(".js"))
            .expect("JS file should exist in embedded assets");

        // Remove 'assets/' prefix for request path
        let js_path = js_file.strip_prefix("assets/").unwrap();

        // WHEN: GET /assets/{hashed-js-file}
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/assets/{}", js_path))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // THEN: Returns asset with immutable cache headers
        assert_eq!(response.status(), StatusCode::OK);
        let cache_control = response
            .headers()
            .get(header::CACHE_CONTROL)
            .expect("Cache-Control header must be present")
            .to_str()
            .unwrap();
        assert!(cache_control.contains("max-age=31536000"));
        assert!(cache_control.contains("immutable"));
    }

    // =========================================================================
    // REQ-EMBED-002: SPA Fallback Routing
    // =========================================================================

    /// REQ-EMBED-002.1: SPA routes return index.html
    ///
    /// # 4-Word Name: test_spa_route_returns_index_html
    ///
    /// WHEN client sends GET request to path "/{any_path}"
    ///   WITH {any_path} NOT matching any registered API endpoint
    ///   AND {any_path} NOT starting with "/assets/"
    /// THEN SHALL respond with HTTP 200 OK
    ///   AND SHALL return body containing contents of embedded "index.html"
    #[tokio::test]
    async fn test_spa_route_returns_index_html() {
        // GIVEN: Server with SPA fallback enabled
        let app = create_test_server_with_embedded_assets().await;

        // WHEN: GET /some-unknown-route (SPA route)
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/some-unknown-route")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // THEN: Returns index.html, not 404
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("<!DOCTYPE html>"));
    }

    /// REQ-EMBED-002.2: Workspace details SPA route works
    ///
    /// # 4-Word Name: test_spa_route_workspace_details
    ///
    /// WHEN client sends GET request to "/workspace-details/abc123"
    /// THEN SHALL return index.html with HTTP 200
    #[tokio::test]
    async fn test_spa_route_workspace_details() {
        // GIVEN: Server with SPA fallback enabled
        let app = create_test_server_with_embedded_assets().await;

        // WHEN: GET /workspace-details/abc123 (complex SPA route)
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/workspace-details/abc123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // THEN: Returns index.html
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(String::from_utf8(body.to_vec())
            .unwrap()
            .contains("<!DOCTYPE html>"));
    }

    /// REQ-EMBED-002.3: API routes take precedence over SPA fallback
    ///
    /// # 4-Word Name: test_api_route_takes_precedence
    ///
    /// WHEN client sends GET request to path "/server-health-check-status"
    /// THEN SHALL return JSON health check response
    ///   AND SHALL NOT return index.html
    #[tokio::test]
    async fn test_api_route_takes_precedence() {
        // GIVEN: Server with both API routes and SPA fallback
        let app = create_test_server_with_embedded_assets().await;

        // WHEN: GET /server-health-check-status (API endpoint)
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/server-health-check-status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // THEN: Returns JSON, not index.html
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap()
            .contains("application/json"));

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["status"], "ok");
    }

    // =========================================================================
    // REQ-EMBED-003: API Route Preservation
    // =========================================================================

    /// REQ-EMBED-003.2: Health check still returns JSON after rust-embed
    ///
    /// # 4-Word Name: test_health_check_still_returns_json
    ///
    /// WHEN server starts with rust-embed integration
    /// THEN GET /server-health-check-status SHALL return JSON response unchanged
    #[tokio::test]
    async fn test_health_check_still_returns_json() {
        // GIVEN: Server with rust-embed integration
        let app = create_test_server_with_embedded_assets().await;

        // WHEN: GET /server-health-check-status
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/server-health-check-status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // THEN: Returns exact same response as before
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // Verify unchanged response structure
        assert_eq!(json["success"], true);
        assert_eq!(json["status"], "ok");
        assert_eq!(json["endpoint"], "/server-health-check-status");
    }

    /// REQ-EMBED-003.2: Workspace list still works after rust-embed
    ///
    /// # 4-Word Name: test_workspace_list_still_works
    ///
    /// WHEN server starts with rust-embed integration
    /// THEN GET /workspace-list-all SHALL return JSON workspace list
    #[tokio::test]
    async fn test_workspace_list_still_works() {
        // GIVEN: Server with rust-embed integration
        let app = create_test_server_with_embedded_assets().await;

        // WHEN: GET /workspace-list-all
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/workspace-list-all")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // THEN: Returns JSON workspace list (not index.html)
        assert_eq!(response.status(), StatusCode::OK);

        let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
        assert!(content_type.to_str().unwrap().contains("application/json"));

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["endpoint"], "/workspace-list-all");
        // Note: workspaces is at top level, not nested under data
        assert!(json["workspaces"].is_array());
    }

    /// REQ-EMBED-003.2: Code entities endpoint still works after rust-embed
    ///
    /// # 4-Word Name: test_code_entities_endpoint_works
    ///
    /// WHEN server starts with rust-embed integration
    /// THEN GET /code-entities-list-all SHALL return JSON entities list
    #[tokio::test]
    async fn test_code_entities_endpoint_works() {
        // GIVEN: Server with rust-embed integration and database
        let storage = CozoDbStorage::new("mem").await.unwrap();
        storage.create_schema().await.unwrap();
        storage.create_dependency_edges_schema().await.unwrap();

        let state = SharedApplicationStateContainer::create_with_database_storage(storage);
        let app = build_complete_router_instance(state);

        // WHEN: GET /code-entities-list-all
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/code-entities-list-all")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // THEN: Returns JSON entities list (not index.html)
        assert_eq!(response.status(), StatusCode::OK);

        let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
        assert!(content_type.to_str().unwrap().contains("application/json"));

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["success"], true);
        assert_eq!(json["endpoint"], "/code-entities-list-all");
    }

    // =========================================================================
    // Test Helpers
    // =========================================================================

    /// Create test server with embedded assets
    ///
    /// # 4-Word Name: create_test_server_with_embedded_assets
    ///
    /// Creates an Axum router with:
    /// - All existing API routes
    /// - Static file serving from rust-embed
    /// - SPA fallback handler
    async fn create_test_server_with_embedded_assets() -> axum::Router {
        let state = SharedApplicationStateContainer::create_new_application_state();
        build_complete_router_instance(state)
    }
}

//! Static file embedding module for serving frontend assets
//!
//! # 4-Word Naming: static_file_embed_module
//!
//! This module provides rust-embed integration for serving the React frontend
//! from the compiled binary. Implements REQ-RUST-EMBED-FRONTEND specification.
//!
//! ## Route Precedence
//! 1. API routes (highest priority)
//! 2. Static asset routes (/assets/*)
//! 3. Root path (/)
//! 4. SPA fallback (catch-all)
//!
//! ## Handlers
//! - `serve_root_index_handler` - Serves index.html for root path
//! - `serve_static_asset_handler` - Serves assets from /assets/*
//! - `serve_spa_fallback_handler` - Returns index.html for unknown routes

use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use rust_embed::Embed;

/// Embedded static files from frontend/dist/
///
/// # 4-Word Name: StaticAssetEmbedFolder
///
/// This struct uses rust-embed to embed all files from the frontend dist
/// directory into the binary at compile time.
///
/// Note: The path is relative to the crate root (crates/pt08-http-code-query-server/).
#[derive(Embed)]
#[folder = "../../frontend/dist/"]
pub struct StaticAssetEmbedFolder;

/// Serve index.html for root path request
///
/// # 4-Word Name: serve_root_index_handler
///
/// ## Contract
/// - Precondition: Server is running with embedded assets
/// - Postcondition: Returns index.html content with Content-Type: text/html
/// - Performance: < 1ms
///
/// ## REQ-EMBED-001.1
/// WHEN client sends GET request to path "/"
/// THEN SHALL respond with HTTP 200 OK
///   AND SHALL set Content-Type header to "text/html; charset=utf-8"
///   AND SHALL return body containing contents of embedded "index.html"
pub async fn serve_root_index_handler() -> Response {
    match StaticAssetEmbedFolder::get("index.html") {
        Some(content) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            content.data.to_vec(),
        )
            .into_response(),
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            [(header::CONTENT_TYPE, "application/json")],
            r#"{"success":false,"error":"Frontend not built. Run 'cd frontend && npm run build' first."}"#,
        )
            .into_response(),
    }
}

/// Serve static asset from embedded files
///
/// # 4-Word Name: serve_static_asset_handler
///
/// ## Contract
/// - Precondition: Request path starts with /assets/
/// - Postcondition: Returns asset content with correct MIME type
/// - Performance: < 1ms
///
/// ## REQ-EMBED-001.2/001.3/001.4/001.5
/// WHEN client sends GET request to path "/assets/{path}"
/// THEN SHALL respond with asset content and correct MIME type
///   AND SHALL set Cache-Control for hashed assets
///   OR SHALL return 404 if asset not found
pub async fn serve_static_asset_handler(Path(path): Path<String>) -> Response {
    let asset_path = format!("assets/{}", path);

    match StaticAssetEmbedFolder::get(&asset_path) {
        Some(content) => {
            let mime_type = get_mime_type_guess(&asset_path);
            let cache_control = create_cache_control_header(&path);

            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, mime_type),
                    (header::CACHE_CONTROL, cache_control),
                ],
                content.data.to_vec(),
            )
                .into_response()
        }
        None => {
            let error_body = format!(
                r#"{{"success":false,"error":"Asset not found","path":"/assets/{}"}}"#,
                path
            );
            (
                StatusCode::NOT_FOUND,
                [(header::CONTENT_TYPE, "application/json")],
                error_body,
            )
                .into_response()
        }
    }
}

/// Serve index.html as SPA fallback for unknown routes
///
/// # 4-Word Name: serve_spa_fallback_handler
///
/// ## Contract
/// - Precondition: Route does not match API or asset paths
/// - Postcondition: Returns index.html for client-side routing
/// - Performance: < 1ms
///
/// ## REQ-EMBED-002.1
/// WHEN client sends GET request to path "/{any_path}"
///   WITH {any_path} NOT matching any registered API endpoint
///   AND {any_path} NOT starting with "/assets/"
/// THEN SHALL respond with HTTP 200 OK
///   AND SHALL return body containing contents of embedded "index.html"
pub async fn serve_spa_fallback_handler() -> Response {
    serve_root_index_handler().await
}

/// Determine MIME type for file path using mime_guess
///
/// # 4-Word Name: get_mime_type_guess
///
/// ## Contract
/// - Precondition: Valid file path string
/// - Postcondition: Returns appropriate MIME type string with charset for text types
fn get_mime_type_guess(path: &str) -> &'static str {
    // Use mime_guess to determine MIME type
    let guess = mime_guess::from_path(path).first_or_octet_stream();

    // Return static string based on MIME type
    // We need to handle charset for text-based types
    // Note: mime_guess uses "svg" for subtype (not "svg+xml"), handle both
    match (guess.type_().as_str(), guess.subtype().as_str()) {
        ("text", "html") => "text/html; charset=utf-8",
        ("text", "css") => "text/css; charset=utf-8",
        ("text", "javascript") | ("application", "javascript") => {
            "application/javascript; charset=utf-8"
        }
        ("application", "json") => "application/json",
        ("image", "png") => "image/png",
        ("image", "jpeg") => "image/jpeg",
        ("image", "gif") => "image/gif",
        ("image", "svg+xml") | ("image", "svg") => "image/svg+xml",
        ("image", "webp") => "image/webp",
        ("image", "x-icon") | ("image", "vnd.microsoft.icon") => "image/x-icon",
        ("font", "woff") => "font/woff",
        ("font", "woff2") => "font/woff2",
        ("font", "ttf") => "font/ttf",
        ("font", "otf") => "font/otf",
        ("application", "vnd.ms-fontobject") => "application/vnd.ms-fontobject",
        _ => "application/octet-stream",
    }
}

/// Create appropriate Cache-Control header based on asset path
///
/// # 4-Word Name: create_cache_control_header
///
/// ## Contract
/// - Precondition: Valid asset path string
/// - Postcondition: Returns cache control directive
///
/// Hashed filenames (e.g., index-abc123.js) should have immutable caching.
/// Non-hashed assets get shorter cache times.
fn create_cache_control_header(path: &str) -> &'static str {
    if should_use_immutable_cache_headers(path) {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=3600"
    }
}

/// Check if asset path should have immutable caching
///
/// # 4-Word Name: should_use_immutable_cache_headers
///
/// ## Contract
/// - Precondition: Valid asset path string
/// - Postcondition: Returns true if path contains content hash
///
/// Hashed filenames (e.g., index-abc123.js) should have immutable caching.
fn should_use_immutable_cache_headers(path: &str) -> bool {
    // Vite/webpack typically produce filenames like:
    // - index-abc123.js
    // - index-abc123.css
    // - chunk-def456.js
    // - index-CG_x_Avx.css (note: underscores are allowed in hashes)
    // These contain a hash after a hyphen before the extension

    // Get the filename without extension
    let filename = path.rsplit('/').next().unwrap_or(path);
    let stem = filename.rsplit('.').skip(1).collect::<Vec<_>>().join(".");

    // Check if stem contains a hyphen followed by hash-like characters
    if let Some(hash_start) = stem.rfind('-') {
        let potential_hash = &stem[hash_start + 1..];
        // Check if it looks like a hash (alphanumeric + underscore characters)
        !potential_hash.is_empty()
            && potential_hash
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
    } else {
        false
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    /// Test MIME type detection for common file types
    ///
    /// # 4-Word Name: test_mime_type_detection_works
    #[test]
    fn test_mime_type_detection_works() {
        assert_eq!(
            get_mime_type_guess("index.html"),
            "text/html; charset=utf-8"
        );
        assert_eq!(
            get_mime_type_guess("app.js"),
            "application/javascript; charset=utf-8"
        );
        assert_eq!(
            get_mime_type_guess("styles.css"),
            "text/css; charset=utf-8"
        );
        assert_eq!(get_mime_type_guess("logo.png"), "image/png");
        assert_eq!(get_mime_type_guess("icon.svg"), "image/svg+xml");
    }

    /// Test immutable cache header detection for hashed assets
    ///
    /// # 4-Word Name: test_immutable_cache_header_detection
    #[test]
    fn test_immutable_cache_header_detection() {
        // Hashed assets should have immutable caching
        assert!(should_use_immutable_cache_headers("index-abc123.js"));
        assert!(should_use_immutable_cache_headers("styles-def456.css"));
        assert!(should_use_immutable_cache_headers("chunk-789abc.js"));
        assert!(should_use_immutable_cache_headers("index-CywoU0pv.js"));
        assert!(should_use_immutable_cache_headers("index-CG_x_Avx.css"));

        // Non-hashed assets should not
        assert!(!should_use_immutable_cache_headers("index.html"));
        assert!(!should_use_immutable_cache_headers("vite.svg"));
        assert!(!should_use_immutable_cache_headers("favicon.ico"));
    }

    /// Test cache control header creation
    ///
    /// # 4-Word Name: test_cache_control_header_creation
    #[test]
    fn test_cache_control_header_creation() {
        assert_eq!(
            create_cache_control_header("index-abc123.js"),
            "public, max-age=31536000, immutable"
        );
        assert_eq!(
            create_cache_control_header("favicon.ico"),
            "public, max-age=3600"
        );
    }

    /// Test embedded assets are accessible
    ///
    /// # 4-Word Name: test_embedded_assets_are_accessible
    #[test]
    fn test_embedded_assets_are_accessible() {
        // Check that index.html is embedded
        let index_html = StaticAssetEmbedFolder::get("index.html");
        assert!(index_html.is_some(), "index.html should be embedded");

        let file_data = index_html.unwrap();
        let content = std::str::from_utf8(file_data.data.as_ref()).unwrap();
        assert!(content.contains("<!DOCTYPE html>"));
        assert!(content.contains("Parseltongue"));
    }

    /// Test embedded asset iteration works
    ///
    /// # 4-Word Name: test_embedded_asset_iteration_works
    #[test]
    fn test_embedded_asset_iteration_works() {
        let files: Vec<_> = StaticAssetEmbedFolder::iter().collect();

        // Should contain index.html
        assert!(files.iter().any(|f| f.as_ref() == "index.html"));
        // Should contain assets
        assert!(files.iter().any(|f| f.starts_with("assets/")));
    }
}

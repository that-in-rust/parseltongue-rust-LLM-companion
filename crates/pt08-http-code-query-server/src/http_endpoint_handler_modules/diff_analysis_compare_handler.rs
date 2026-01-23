//! Diff analysis compare snapshots endpoint handler
//!
//! # 4-Word Naming: diff_analysis_compare_handler
//!
//! Endpoint: POST /diff-analysis-compare-snapshots?max_hops=N
//!
//! Compares two database snapshots (base and live) and returns:
//! - Entity and edge diffs (added, removed, modified)
//! - Blast radius of changes
//! - Visualization-ready representation
//!
//! This endpoint enables LLM agents to understand code changes
//! between two points in time without custom code.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use parseltongue_core::diff::{
    DefaultEntityDifferImpl, DefaultBlastRadiusCalculatorImpl,
    DefaultDiffVisualizationTransformerImpl, EntityDifferTrait,
    BlastRadiusCalculatorTrait, DiffVisualizationTransformerTrait,
    EntityDataPayload as CoreEntityDataPayload,
    EdgeDataPayload as CoreEdgeDataPayload,
    LineRangeData as CoreLineRangeData,
    EntityChangeTypeClassification,
    EdgeChangeTypeClassification,
    NodeVisualizationStatusType,
};
use parseltongue_core::storage::CozoDbStorage;

use crate::http_server_startup_runner::SharedApplicationStateContainer;

// =============================================================================
// Constants
// =============================================================================

/// Default max_hops value when not specified
///
/// # 4-Word Name: DEFAULT_MAX_HOPS_VALUE
const DEFAULT_MAX_HOPS_VALUE: u32 = 2;

/// Maximum allowed max_hops value
///
/// # 4-Word Name: MAXIMUM_ALLOWED_HOPS_LIMIT
const MAXIMUM_ALLOWED_HOPS_LIMIT: u32 = 10;

// =============================================================================
// Request Types
// =============================================================================

/// Request body for diff analysis endpoint
///
/// # 4-Word Name: DiffAnalysisRequestBodyStruct
#[derive(Debug, Clone, Deserialize)]
pub struct DiffAnalysisRequestBodyStruct {
    /// Path to base (before) database, e.g., "rocksdb:path/to/base.db"
    pub base_db: Option<String>,
    /// Path to live (after) database, e.g., "rocksdb:path/to/live.db"
    pub live_db: Option<String>,
}

/// Query parameters for diff analysis endpoint
///
/// # 4-Word Name: DiffAnalysisQueryParamsStruct
#[derive(Debug, Clone, Deserialize)]
pub struct DiffAnalysisQueryParamsStruct {
    /// Maximum hops for blast radius (default: 2, max: 10)
    pub max_hops: Option<u32>,
}

// =============================================================================
// Response Types - Diff Data
// =============================================================================

/// Summary of diff results
///
/// # 4-Word Name: DiffSummaryDataPayloadStruct
#[derive(Debug, Clone, Serialize)]
pub struct DiffSummaryDataPayloadStruct {
    pub total_before_count: usize,
    pub total_after_count: usize,
    pub added_entity_count: usize,
    pub removed_entity_count: usize,
    pub modified_entity_count: usize,
    pub unchanged_entity_count: usize,
    pub relocated_entity_count: usize,
}

/// Single entity change entry
///
/// # 4-Word Name: EntityChangeDataPayloadStruct
#[derive(Debug, Clone, Serialize)]
pub struct EntityChangeDataPayloadStruct {
    pub entity_key: String,
    pub change_type: String,  // "added", "removed", "modified", "relocated"
    pub before_hash: Option<String>,
    pub after_hash: Option<String>,
}

/// Single edge change entry
///
/// # 4-Word Name: EdgeChangeDataPayloadStruct
#[derive(Debug, Clone, Serialize)]
pub struct EdgeChangeDataPayloadStruct {
    pub from_key: String,
    pub to_key: String,
    pub change_type: String,  // "added", "removed"
}

/// Complete diff result payload
///
/// # 4-Word Name: DiffResultDataPayloadStruct
#[derive(Debug, Clone, Serialize)]
pub struct DiffResultDataPayloadStruct {
    pub summary: DiffSummaryDataPayloadStruct,
    pub entity_changes: Vec<EntityChangeDataPayloadStruct>,
    pub edge_changes: Vec<EdgeChangeDataPayloadStruct>,
}

// =============================================================================
// Response Types - Blast Radius
// =============================================================================

/// Blast radius result payload
///
/// # 4-Word Name: BlastRadiusResultPayloadStruct
#[derive(Debug, Clone, Serialize)]
pub struct BlastRadiusResultPayloadStruct {
    pub origin_entity: String,
    pub affected_by_distance: HashMap<usize, Vec<String>>,
    pub total_affected_count: usize,
    pub max_depth_reached: usize,
}

// =============================================================================
// Response Types - Visualization
// =============================================================================

/// Visualization node data
///
/// # 4-Word Name: VisualizationNodeDataPayload
#[derive(Debug, Clone, Serialize)]
pub struct VisualizationNodeDataPayload {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub change_type: Option<String>,
}

/// Visualization edge data
///
/// # 4-Word Name: VisualizationEdgeDataPayload
#[derive(Debug, Clone, Serialize)]
pub struct VisualizationEdgeDataPayload {
    pub source: String,
    pub target: String,
    pub edge_type: String,
}

/// Visualization graph data payload
///
/// # 4-Word Name: VisualizationGraphDataPayload
#[derive(Debug, Clone, Serialize)]
pub struct VisualizationGraphDataPayload {
    pub nodes: Vec<VisualizationNodeDataPayload>,
    pub edges: Vec<VisualizationEdgeDataPayload>,
    pub diff_summary: DiffSummaryDataPayloadStruct,
    pub max_blast_radius_depth: usize,
}

// =============================================================================
// Response Types - Main Response
// =============================================================================

/// Successful response payload for diff analysis endpoint
///
/// # 4-Word Name: DiffAnalysisResponsePayloadStruct
#[derive(Debug, Clone, Serialize)]
pub struct DiffAnalysisResponsePayloadStruct {
    pub success: bool,
    pub endpoint: String,
    pub diff: DiffResultDataPayloadStruct,
    pub blast_radius: BlastRadiusResultPayloadStruct,
    pub visualization: VisualizationGraphDataPayload,
    pub token_estimate: usize,
}

/// Error response for diff analysis endpoint
///
/// # 4-Word Name: DiffAnalysisErrorResponseStruct
#[derive(Debug, Clone, Serialize)]
pub struct DiffAnalysisErrorResponseStruct {
    pub error: String,
    pub code: String,
}

// =============================================================================
// Validation Functions
// =============================================================================

/// Validate request body and return validation result
///
/// # 4-Word Name: validate_request_body_fields
fn validate_request_body_fields(
    body: &DiffAnalysisRequestBodyStruct,
) -> Result<(String, String), DiffAnalysisErrorResponseStruct> {
    // Check base_db is present
    let base_db = match &body.base_db {
        Some(db) if !db.is_empty() => db.clone(),
        Some(_) => {
            return Err(DiffAnalysisErrorResponseStruct {
                error: "Invalid base_db: path cannot be empty".to_string(),
                code: "INVALID_BASE_DB_PATH".to_string(),
            });
        }
        None => {
            return Err(DiffAnalysisErrorResponseStruct {
                error: "Missing required field: base_db".to_string(),
                code: "MISSING_BASE_DB".to_string(),
            });
        }
    };

    // Check live_db is present
    let live_db = match &body.live_db {
        Some(db) if !db.is_empty() => db.clone(),
        Some(_) => {
            return Err(DiffAnalysisErrorResponseStruct {
                error: "Invalid live_db: path cannot be empty".to_string(),
                code: "INVALID_LIVE_DB_PATH".to_string(),
            });
        }
        None => {
            return Err(DiffAnalysisErrorResponseStruct {
                error: "Missing required field: live_db".to_string(),
                code: "MISSING_LIVE_DB".to_string(),
            });
        }
    };

    // Validate path format
    if !base_db.starts_with("rocksdb:") {
        return Err(DiffAnalysisErrorResponseStruct {
            error: "Invalid database path format. Expected 'rocksdb:path/to/db'".to_string(),
            code: "INVALID_DB_PATH_FORMAT".to_string(),
        });
    }

    if !live_db.starts_with("rocksdb:") {
        return Err(DiffAnalysisErrorResponseStruct {
            error: "Invalid database path format. Expected 'rocksdb:path/to/db'".to_string(),
            code: "INVALID_DB_PATH_FORMAT".to_string(),
        });
    }

    Ok((base_db, live_db))
}

/// Validate max_hops query parameter
///
/// # 4-Word Name: validate_max_hops_parameter
fn validate_max_hops_parameter(
    max_hops: Option<u32>,
) -> Result<u32, DiffAnalysisErrorResponseStruct> {
    match max_hops {
        Some(hops) if hops > MAXIMUM_ALLOWED_HOPS_LIMIT => {
            Err(DiffAnalysisErrorResponseStruct {
                error: format!("max_hops cannot exceed {}", MAXIMUM_ALLOWED_HOPS_LIMIT),
                code: "MAX_HOPS_EXCEEDED".to_string(),
            })
        }
        Some(hops) => Ok(hops),
        None => Ok(DEFAULT_MAX_HOPS_VALUE),
    }
}

// =============================================================================
// Helper Functions for Loading Data
// =============================================================================

/// Load entities from a CozoDB database
///
/// # 4-Word Name: load_entities_from_database
async fn load_entities_from_database(
    storage: &CozoDbStorage,
) -> Result<HashMap<String, CoreEntityDataPayload>, DiffAnalysisErrorResponseStruct> {
    let entities = storage.get_all_entities().await.map_err(|e| {
        DiffAnalysisErrorResponseStruct {
            error: format!("Failed to load entities: {}", e),
            code: "DB_CONNECTION_FAILED".to_string(),
        }
    })?;

    let mut entity_map = HashMap::new();
    for entity in entities {
        // Extract line range from key if possible
        let line_range = extract_line_range_from_key(&entity.isgl1_key);

        entity_map.insert(
            entity.isgl1_key.clone(),
            CoreEntityDataPayload {
                key: entity.isgl1_key.clone(),
                file_path: entity.interface_signature.file_path.to_string_lossy().to_string(),
                entity_type: format!("{:?}", entity.interface_signature.entity_type),
                line_range,
                content_hash: entity.current_code.as_ref().map(|c| {
                    // Simple hash for content comparison
                    format!("{:x}", md5_hash(c))
                }),
            },
        );
    }

    Ok(entity_map)
}

/// Extract line range from ISGL1 key
///
/// # 4-Word Name: extract_line_range_from_key
fn extract_line_range_from_key(key: &str) -> Option<CoreLineRangeData> {
    // ISGL1 key format: language:type:name:path:start-end
    let parts: Vec<&str> = key.split(':').collect();
    if parts.len() >= 5 {
        let line_part = parts.last()?;
        let line_parts: Vec<&str> = line_part.split('-').collect();
        if line_parts.len() == 2 {
            let start = line_parts[0].parse().ok()?;
            let end = line_parts[1].parse().ok()?;
            return Some(CoreLineRangeData { start, end });
        }
    }
    None
}

/// Simple MD5-like hash for content comparison
///
/// # 4-Word Name: md5_hash
fn md5_hash(content: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// Load edges from a CozoDB database
///
/// # 4-Word Name: load_edges_from_database
async fn load_edges_from_database(
    storage: &CozoDbStorage,
) -> Result<Vec<CoreEdgeDataPayload>, DiffAnalysisErrorResponseStruct> {
    let deps = storage.get_all_dependencies().await.map_err(|e| {
        DiffAnalysisErrorResponseStruct {
            error: format!("Failed to load dependencies: {}", e),
            code: "DB_CONNECTION_FAILED".to_string(),
        }
    })?;

    let edges: Vec<CoreEdgeDataPayload> = deps
        .into_iter()
        .map(|dep| CoreEdgeDataPayload {
            from_key: dep.from_key.to_string(),
            to_key: dep.to_key.to_string(),
            edge_type: dep.edge_type.as_str().to_string(),
            source_location: dep.source_location.clone(),
        })
        .collect();

    Ok(edges)
}

/// Convert change type enum to string
///
/// # 4-Word Name: entity_change_type_string
fn entity_change_type_string(change_type: &EntityChangeTypeClassification) -> String {
    match change_type {
        EntityChangeTypeClassification::AddedToCodebase => "added".to_string(),
        EntityChangeTypeClassification::RemovedFromCodebase => "removed".to_string(),
        EntityChangeTypeClassification::ModifiedInCodebase => "modified".to_string(),
        EntityChangeTypeClassification::RelocatedInCodebase => "relocated".to_string(),
        EntityChangeTypeClassification::UnchangedInCodebase => "unchanged".to_string(),
    }
}

/// Convert edge change type to string
///
/// # 4-Word Name: edge_change_type_string
fn edge_change_type_string(change_type: &EdgeChangeTypeClassification) -> String {
    match change_type {
        EdgeChangeTypeClassification::AddedToGraph => "added".to_string(),
        EdgeChangeTypeClassification::RemovedFromGraph => "removed".to_string(),
        EdgeChangeTypeClassification::UnchangedInGraph => "unchanged".to_string(),
        EdgeChangeTypeClassification::ModifiedInGraph => "modified".to_string(),
    }
}

/// Convert node visualization status to string
///
/// # 4-Word Name: node_status_to_string
fn node_status_to_string(status: &NodeVisualizationStatusType) -> Option<String> {
    match status {
        NodeVisualizationStatusType::AddedNode => Some("added".to_string()),
        NodeVisualizationStatusType::RemovedNode => Some("removed".to_string()),
        NodeVisualizationStatusType::ModifiedNode => Some("modified".to_string()),
        NodeVisualizationStatusType::AffectedNode => Some("affected".to_string()),
        NodeVisualizationStatusType::UnchangedNode => None,
    }
}

// =============================================================================
// Handler Function
// =============================================================================

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
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Validate request body
    let (base_db_path, live_db_path) = match validate_request_body_fields(&body) {
        Ok((base, live)) => (base, live),
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(err),
            ).into_response();
        }
    };

    // Validate max_hops parameter
    let max_hops = match validate_max_hops_parameter(params.max_hops) {
        Ok(hops) => hops,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(err),
            ).into_response();
        }
    };

    // Connect to base database
    let base_storage = match CozoDbStorage::new(&base_db_path).await {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(DiffAnalysisErrorResponseStruct {
                    error: format!("Failed to connect to base database: {}", e),
                    code: "DB_CONNECTION_FAILED".to_string(),
                }),
            ).into_response();
        }
    };

    // Connect to live database
    let live_storage = match CozoDbStorage::new(&live_db_path).await {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(DiffAnalysisErrorResponseStruct {
                    error: format!("Failed to connect to live database: {}", e),
                    code: "DB_CONNECTION_FAILED".to_string(),
                }),
            ).into_response();
        }
    };

    // Load entities from both databases
    let base_entities = match load_entities_from_database(&base_storage).await {
        Ok(e) => e,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response();
        }
    };

    let live_entities = match load_entities_from_database(&live_storage).await {
        Ok(e) => e,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response();
        }
    };

    // Load edges from both databases
    let base_edges = match load_edges_from_database(&base_storage).await {
        Ok(e) => e,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response();
        }
    };

    let live_edges = match load_edges_from_database(&live_storage).await {
        Ok(e) => e,
        Err(err) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response();
        }
    };

    // Compute diff using DefaultEntityDifferImpl
    let differ = DefaultEntityDifferImpl::new();
    let mut diff_result = differ.compute_entity_diff_result(&base_entities, &live_entities);
    differ.compute_edge_diff_result(&base_edges, &live_edges, &mut diff_result);

    // Collect changed entities for blast radius calculation
    let changed_entities: Vec<String> = diff_result
        .entity_changes
        .iter()
        .filter(|c| c.change_type.is_significant_change())
        .filter_map(|c| {
            c.after_entity.as_ref().map(|e| e.key.clone())
                .or_else(|| c.before_entity.as_ref().map(|e| e.key.clone()))
        })
        .collect();

    // Compute blast radius using DefaultBlastRadiusCalculatorImpl
    let blast_calculator = DefaultBlastRadiusCalculatorImpl::new();
    let blast_result = blast_calculator.compute_combined_blast_radius(
        &changed_entities,
        &live_edges,
        max_hops,
    );

    // Transform to visualization using DefaultDiffVisualizationTransformerImpl
    let viz_transformer = DefaultDiffVisualizationTransformerImpl::new();
    let visualization = viz_transformer.transform_diff_to_visualization(&diff_result, &blast_result);

    // Convert to response types
    let diff_response = DiffResultDataPayloadStruct {
        summary: DiffSummaryDataPayloadStruct {
            total_before_count: diff_result.summary.total_before_count,
            total_after_count: diff_result.summary.total_after_count,
            added_entity_count: diff_result.summary.added_entity_count,
            removed_entity_count: diff_result.summary.removed_entity_count,
            modified_entity_count: diff_result.summary.modified_entity_count,
            unchanged_entity_count: diff_result.summary.unchanged_entity_count,
            relocated_entity_count: diff_result.summary.relocated_entity_count,
        },
        entity_changes: diff_result
            .entity_changes
            .iter()
            .filter(|c| c.change_type.is_significant_change() || matches!(c.change_type, EntityChangeTypeClassification::RelocatedInCodebase))
            .map(|c| EntityChangeDataPayloadStruct {
                entity_key: c.stable_identity.clone(),
                change_type: entity_change_type_string(&c.change_type),
                before_hash: c.before_entity.as_ref().and_then(|e| e.content_hash.clone()),
                after_hash: c.after_entity.as_ref().and_then(|e| e.content_hash.clone()),
            })
            .collect(),
        edge_changes: diff_result
            .edge_changes
            .iter()
            .filter(|c| !matches!(c.change_type, EdgeChangeTypeClassification::UnchangedInGraph))
            .map(|c| EdgeChangeDataPayloadStruct {
                from_key: c.from_stable_identity.clone(),
                to_key: c.to_stable_identity.clone(),
                change_type: edge_change_type_string(&c.change_type),
            })
            .collect(),
    };

    let blast_response = BlastRadiusResultPayloadStruct {
        origin_entity: blast_result.origin_entity,
        affected_by_distance: blast_result
            .affected_by_distance
            .into_iter()
            .map(|(k, v)| (k as usize, v))
            .collect(),
        total_affected_count: blast_result.total_affected_count,
        max_depth_reached: blast_result.max_depth_reached as usize,
    };

    let viz_response = VisualizationGraphDataPayload {
        nodes: visualization
            .nodes
            .iter()
            .map(|n| VisualizationNodeDataPayload {
                id: n.id.clone(),
                label: n.label.clone(),
                node_type: n.entity_type.clone(),
                change_type: node_status_to_string(&n.status),
            })
            .collect(),
        edges: visualization
            .edges
            .iter()
            .map(|e| VisualizationEdgeDataPayload {
                source: e.from_id.clone(),
                target: e.to_id.clone(),
                edge_type: e.edge_type.clone(),
            })
            .collect(),
        diff_summary: DiffSummaryDataPayloadStruct {
            total_before_count: visualization.diff_summary.total_before_count,
            total_after_count: visualization.diff_summary.total_after_count,
            added_entity_count: visualization.diff_summary.added_entity_count,
            removed_entity_count: visualization.diff_summary.removed_entity_count,
            modified_entity_count: visualization.diff_summary.modified_entity_count,
            unchanged_entity_count: visualization.diff_summary.unchanged_entity_count,
            relocated_entity_count: visualization.diff_summary.relocated_entity_count,
        },
        max_blast_radius_depth: visualization.max_blast_radius_depth as usize,
    };

    // Calculate token estimate using formula from spec
    let token_estimate = 100
        + (diff_response.entity_changes.len() * 50)
        + (diff_response.edge_changes.len() * 30)
        + (blast_response.total_affected_count * 20)
        + (viz_response.nodes.len() * 40)
        + (viz_response.edges.len() * 25);

    // Return successful response
    (
        StatusCode::OK,
        Json(DiffAnalysisResponsePayloadStruct {
            success: true,
            endpoint: "/diff-analysis-compare-snapshots".to_string(),
            diff: diff_response,
            blast_radius: blast_response,
            visualization: viz_response,
            token_estimate,
        }),
    ).into_response()
}

/// Handle JSON parsing errors for diff analysis endpoint
///
/// # 4-Word Name: handle_json_parse_rejection
///
/// This is used to handle cases where Axum fails to parse the JSON body
/// before our handler is even called.
pub fn create_invalid_json_error_response() -> (StatusCode, Json<DiffAnalysisErrorResponseStruct>) {
    (
        StatusCode::BAD_REQUEST,
        Json(DiffAnalysisErrorResponseStruct {
            error: "Invalid JSON in request body".to_string(),
            code: "INVALID_JSON".to_string(),
        }),
    )
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

    /// Create a test router with the diff analysis endpoint
    ///
    /// # 4-Word Name: create_test_router_instance
    fn create_test_router_instance() -> Router {
        let state = SharedApplicationStateContainer::create_new_application_state();
        Router::new()
            .route(
                "/diff-analysis-compare-snapshots",
                post(handle_diff_analysis_compare_snapshots),
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
    async fn extract_json_from_response(
        response: axum::response::Response,
    ) -> Value {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    // =========================================================================
    // REQ-HTTP-DIFF-001: Request Parsing Tests
    // =========================================================================

    /// REQ-HTTP-DIFF-001.1: Valid request body parsing
    ///
    /// WHEN client sends POST with valid base_db and live_db
    /// THEN SHALL parse request successfully (or return NOT_IMPLEMENTED for stub)
    #[tokio::test]
    async fn test_parse_valid_request_body() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/diff-analysis-compare-snapshots",
            json!({
                "base_db": "rocksdb:test/base.db",
                "live_db": "rocksdb:test/live.db"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        // Implementation will try to connect to DBs and may fail if they don't exist
        // But it should not fail on parsing - check for non-validation errors
        assert!(
            response.status() == StatusCode::BAD_REQUEST  // DB connection failed
                || response.status() == StatusCode::OK
                || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
            "Expected BAD_REQUEST (DB error), OK, or INTERNAL_SERVER_ERROR, got: {}",
            response.status()
        );
    }

    /// REQ-HTTP-DIFF-001.2: Missing base_db field returns 400
    ///
    /// WHEN client sends POST without base_db field
    /// THEN SHALL return 400 with code MISSING_BASE_DB
    #[tokio::test]
    async fn test_parse_missing_base_db_returns_error() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/diff-analysis-compare-snapshots",
            json!({
                "live_db": "rocksdb:test/live.db"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = extract_json_from_response(response).await;
        assert_eq!(json["code"], "MISSING_BASE_DB");
        assert!(json["error"].as_str().unwrap().contains("base_db"));
    }

    /// REQ-HTTP-DIFF-001.3: Missing live_db field returns 400
    ///
    /// WHEN client sends POST without live_db field
    /// THEN SHALL return 400 with code MISSING_LIVE_DB
    #[tokio::test]
    async fn test_parse_missing_live_db_returns_error() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/diff-analysis-compare-snapshots",
            json!({
                "base_db": "rocksdb:test/base.db"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = extract_json_from_response(response).await;
        assert_eq!(json["code"], "MISSING_LIVE_DB");
        assert!(json["error"].as_str().unwrap().contains("live_db"));
    }

    /// REQ-HTTP-DIFF-001.4: Empty base_db field returns 400
    ///
    /// WHEN client sends POST with empty base_db
    /// THEN SHALL return 400 with code INVALID_BASE_DB_PATH
    #[tokio::test]
    async fn test_parse_empty_base_db_returns_error() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/diff-analysis-compare-snapshots",
            json!({
                "base_db": "",
                "live_db": "rocksdb:test/live.db"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = extract_json_from_response(response).await;
        assert_eq!(json["code"], "INVALID_BASE_DB_PATH");
    }

    /// REQ-HTTP-DIFF-001.5: Empty live_db field returns 400
    ///
    /// WHEN client sends POST with empty live_db
    /// THEN SHALL return 400 with code INVALID_LIVE_DB_PATH
    #[tokio::test]
    async fn test_parse_empty_live_db_returns_error() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/diff-analysis-compare-snapshots",
            json!({
                "base_db": "rocksdb:test/base.db",
                "live_db": ""
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = extract_json_from_response(response).await;
        assert_eq!(json["code"], "INVALID_LIVE_DB_PATH");
    }

    /// REQ-HTTP-DIFF-001.6: Invalid JSON returns 400
    ///
    /// WHEN client sends malformed JSON
    /// THEN SHALL return 400
    /// NOTE: This test verifies behavior at the Axum layer
    #[tokio::test]
    async fn test_parse_invalid_json_returns_error() {
        let app = create_test_router_instance();

        let request = Request::builder()
            .method("POST")
            .uri("/diff-analysis-compare-snapshots")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from("{ invalid json"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Axum returns 400 or 422 for invalid JSON
        assert!(
            response.status() == StatusCode::BAD_REQUEST
                || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
            "Expected BAD_REQUEST or UNPROCESSABLE_ENTITY for invalid JSON, got: {}",
            response.status()
        );
    }

    // =========================================================================
    // REQ-HTTP-DIFF-002: Database Connection Tests
    // =========================================================================

    /// REQ-HTTP-DIFF-002.5: Invalid database path format returns 400
    ///
    /// WHEN client sends base_db without rocksdb: prefix
    /// THEN SHALL return 400 with code INVALID_DB_PATH_FORMAT
    #[tokio::test]
    async fn test_invalid_db_path_returns_error() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/diff-analysis-compare-snapshots",
            json!({
                "base_db": "invalid/path.db",
                "live_db": "rocksdb:test/live.db"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = extract_json_from_response(response).await;
        assert_eq!(json["code"], "INVALID_DB_PATH_FORMAT");
    }

    // =========================================================================
    // REQ-HTTP-DIFF-003: Response Structure Tests
    // =========================================================================

    /// REQ-HTTP-DIFF-003.3: Response contains all required fields
    ///
    /// WHEN endpoint returns successful response
    /// THEN SHALL contain all required fields
    /// NOTE: This test will pass once implementation is complete
    #[tokio::test]
    async fn test_response_contains_all_required_fields() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/diff-analysis-compare-snapshots",
            json!({
                "base_db": "rocksdb:test/base.db",
                "live_db": "rocksdb:test/live.db"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        // If we get OK, verify the response structure
        if response.status() == StatusCode::OK {
            let json = extract_json_from_response(response).await;

            // Required fields check
            assert!(json.get("success").is_some(), "Missing 'success' field");
            assert!(json.get("endpoint").is_some(), "Missing 'endpoint' field");
            assert!(json.get("diff").is_some(), "Missing 'diff' field");
            assert!(
                json["diff"].get("summary").is_some(),
                "Missing 'diff.summary' field"
            );
            assert!(
                json["diff"].get("entity_changes").is_some(),
                "Missing 'diff.entity_changes' field"
            );
            assert!(
                json["diff"].get("edge_changes").is_some(),
                "Missing 'diff.edge_changes' field"
            );
            assert!(
                json.get("blast_radius").is_some(),
                "Missing 'blast_radius' field"
            );
            assert!(
                json.get("visualization").is_some(),
                "Missing 'visualization' field"
            );
            assert!(
                json.get("token_estimate").is_some(),
                "Missing 'token_estimate' field"
            );
        }
        // Otherwise, it's a DB connection error which is expected for test paths
    }

    /// REQ-HTTP-DIFF-003.4: Response includes token estimate
    ///
    /// WHEN endpoint returns successful response
    /// THEN SHALL include token_estimate >= 100
    /// NOTE: This test will pass once implementation is complete
    #[tokio::test]
    async fn test_response_includes_token_estimate() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/diff-analysis-compare-snapshots",
            json!({
                "base_db": "rocksdb:test/base.db",
                "live_db": "rocksdb:test/live.db"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        // If we get OK, verify token_estimate
        if response.status() == StatusCode::OK {
            let json = extract_json_from_response(response).await;

            let token_estimate = json["token_estimate"].as_u64().unwrap();
            assert!(
                token_estimate >= 100,
                "Token estimate {} is below minimum 100",
                token_estimate
            );
        }
        // Otherwise, it's a DB connection error which is expected
    }

    // =========================================================================
    // REQ-HTTP-DIFF-004: Query Parameter Tests
    // =========================================================================

    /// REQ-HTTP-DIFF-004.1: Default max_hops is 2
    ///
    /// WHEN client sends request without max_hops
    /// THEN SHALL use default value of 2
    #[tokio::test]
    async fn test_default_max_hops_is_two() {
        let app = create_test_router_instance();

        // Request without max_hops query parameter
        let request = make_post_request_json(
            "/diff-analysis-compare-snapshots",
            json!({
                "base_db": "rocksdb:test/base.db",
                "live_db": "rocksdb:test/live.db"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        // If we get OK, verify max_hops was used
        if response.status() == StatusCode::OK {
            let json = extract_json_from_response(response).await;
            let max_depth = json["blast_radius"]["max_depth_reached"]
                .as_u64()
                .unwrap_or(0);
            assert!(
                max_depth <= 2,
                "Default max_hops should be 2, got depth {}",
                max_depth
            );
        }
        // Otherwise, we just verify the request was processed (DB error is OK)
    }

    /// REQ-HTTP-DIFF-004.2: Custom max_hops value is accepted
    ///
    /// WHEN client sends request with max_hops=5
    /// THEN SHALL accept and use that value
    #[tokio::test]
    async fn test_custom_max_hops_value() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/diff-analysis-compare-snapshots?max_hops=5",
            json!({
                "base_db": "rocksdb:test/base.db",
                "live_db": "rocksdb:test/live.db"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        // Should not return BAD_REQUEST for valid max_hops (unless DB error)
        let json = extract_json_from_response(response).await;
        // If we got a validation error, it shouldn't be about max_hops
        if json.get("code").is_some() {
            assert_ne!(
                json["code"], "MAX_HOPS_EXCEEDED",
                "max_hops=5 should be accepted"
            );
        }
    }

    /// REQ-HTTP-DIFF-004.5: max_hops exceeds maximum returns 400
    ///
    /// WHEN client sends max_hops=100 (exceeds limit of 10)
    /// THEN SHALL return 400 with code MAX_HOPS_EXCEEDED
    #[tokio::test]
    async fn test_max_hops_exceeds_limit_returns_error() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/diff-analysis-compare-snapshots?max_hops=100",
            json!({
                "base_db": "rocksdb:test/base.db",
                "live_db": "rocksdb:test/live.db"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = extract_json_from_response(response).await;
        assert_eq!(json["code"], "MAX_HOPS_EXCEEDED");
    }

    /// REQ-HTTP-DIFF-004.4: max_hops at maximum boundary is accepted
    ///
    /// WHEN client sends max_hops=10 (exactly at limit)
    /// THEN SHALL accept the value
    #[tokio::test]
    async fn test_max_hops_at_limit_accepted() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/diff-analysis-compare-snapshots?max_hops=10",
            json!({
                "base_db": "rocksdb:test/base.db",
                "live_db": "rocksdb:test/live.db"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        // Should not return MAX_HOPS_EXCEEDED error
        let json = extract_json_from_response(response).await;
        if json.get("code").is_some() {
            assert_ne!(
                json["code"], "MAX_HOPS_EXCEEDED",
                "max_hops=10 should be accepted (at limit)"
            );
        }
    }

    /// REQ-HTTP-DIFF-004.3: max_hops=0 is accepted
    ///
    /// WHEN client sends max_hops=0
    /// THEN SHALL accept the value (minimal blast radius)
    #[tokio::test]
    async fn test_max_hops_zero_accepted() {
        let app = create_test_router_instance();

        let request = make_post_request_json(
            "/diff-analysis-compare-snapshots?max_hops=0",
            json!({
                "base_db": "rocksdb:test/base.db",
                "live_db": "rocksdb:test/live.db"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        // Should not return MAX_HOPS_EXCEEDED error
        let json = extract_json_from_response(response).await;
        if json.get("code").is_some() {
            assert_ne!(
                json["code"], "MAX_HOPS_EXCEEDED",
                "max_hops=0 should be accepted"
            );
        }
    }

    // =========================================================================
    // REQ-HTTP-DIFF-005: Error Format Tests
    // =========================================================================

    /// REQ-HTTP-DIFF-005.1: Error response has consistent structure
    ///
    /// WHEN endpoint returns any error
    /// THEN SHALL have "error" (non-empty string) and "code" (SCREAMING_SNAKE_CASE)
    #[tokio::test]
    async fn test_error_response_has_consistent_structure() {
        let app = create_test_router_instance();

        // Trigger an error by omitting base_db
        let request = make_post_request_json(
            "/diff-analysis-compare-snapshots",
            json!({
                "live_db": "rocksdb:test/live.db"
            }),
        );

        let response = app.oneshot(request).await.unwrap();

        // Verify it's an error response
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Verify Content-Type
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );

        let json = extract_json_from_response(response).await;

        // Must have "error" field as non-empty string
        assert!(json.get("error").is_some(), "Missing 'error' field");
        assert!(json["error"].is_string(), "'error' must be string");
        assert!(
            !json["error"].as_str().unwrap().is_empty(),
            "'error' must not be empty"
        );

        // Must have "code" field in SCREAMING_SNAKE_CASE
        assert!(json.get("code").is_some(), "Missing 'code' field");
        assert!(json["code"].is_string(), "'code' must be string");

        let code = json["code"].as_str().unwrap();
        assert!(
            code.chars().all(|c| c.is_uppercase() || c == '_'),
            "'code' must be SCREAMING_SNAKE_CASE, got: {}",
            code
        );
    }

    /// REQ-HTTP-DIFF-005.2: Error codes match enumeration
    ///
    /// WHEN endpoint returns error
    /// THEN code SHALL be from the defined enumeration
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
            "DB_CONNECTION_FAILED",
            "MAX_HOPS_EXCEEDED",
            "INVALID_MAX_HOPS",
            "DIFF_COMPUTATION_FAILED",
            "INTERNAL_ERROR",
            "NOT_IMPLEMENTED",
        ];

        let app = create_test_router_instance();

        // Test MISSING_BASE_DB
        let request = make_post_request_json(
            "/diff-analysis-compare-snapshots",
            json!({
                "live_db": "rocksdb:test/live.db"
            }),
        );

        let response = app.oneshot(request).await.unwrap();
        let json = extract_json_from_response(response).await;

        let code = json["code"].as_str().unwrap();
        assert!(
            valid_codes.contains(&code),
            "Unknown error code: {}. Valid codes: {:?}",
            code,
            valid_codes
        );
    }

    /// REQ-HTTP-DIFF-005.3: HTTP status codes are correct
    ///
    /// WHEN endpoint returns error with specific code
    /// THEN SHALL use correct HTTP status code
    #[tokio::test]
    async fn test_error_status_codes_are_correct() {
        let app = create_test_router_instance();

        // Test 400 for missing field (MISSING_BASE_DB)
        let request = make_post_request_json(
            "/diff-analysis-compare-snapshots",
            json!({
                "live_db": "rocksdb:test/live.db"
            }),
        );

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "MISSING_BASE_DB should return 400"
        );

        // Test 400 for MAX_HOPS_EXCEEDED
        let app2 = create_test_router_instance();
        let request2 = make_post_request_json(
            "/diff-analysis-compare-snapshots?max_hops=100",
            json!({
                "base_db": "rocksdb:test/base.db",
                "live_db": "rocksdb:test/live.db"
            }),
        );

        let response2 = app2.oneshot(request2).await.unwrap();
        assert_eq!(
            response2.status(),
            StatusCode::BAD_REQUEST,
            "MAX_HOPS_EXCEEDED should return 400"
        );
    }

    // =========================================================================
    // Validation Function Unit Tests
    // =========================================================================

    #[test]
    fn test_validate_request_body_valid() {
        let body = DiffAnalysisRequestBodyStruct {
            base_db: Some("rocksdb:test/base.db".to_string()),
            live_db: Some("rocksdb:test/live.db".to_string()),
        };

        let result = validate_request_body_fields(&body);
        assert!(result.is_ok());

        let (base, live) = result.unwrap();
        assert_eq!(base, "rocksdb:test/base.db");
        assert_eq!(live, "rocksdb:test/live.db");
    }

    #[test]
    fn test_validate_request_body_missing_base() {
        let body = DiffAnalysisRequestBodyStruct {
            base_db: None,
            live_db: Some("rocksdb:test/live.db".to_string()),
        };

        let result = validate_request_body_fields(&body);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code, "MISSING_BASE_DB");
    }

    #[test]
    fn test_validate_request_body_missing_live() {
        let body = DiffAnalysisRequestBodyStruct {
            base_db: Some("rocksdb:test/base.db".to_string()),
            live_db: None,
        };

        let result = validate_request_body_fields(&body);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code, "MISSING_LIVE_DB");
    }

    #[test]
    fn test_validate_request_body_empty_base() {
        let body = DiffAnalysisRequestBodyStruct {
            base_db: Some("".to_string()),
            live_db: Some("rocksdb:test/live.db".to_string()),
        };

        let result = validate_request_body_fields(&body);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code, "INVALID_BASE_DB_PATH");
    }

    #[test]
    fn test_validate_request_body_invalid_format() {
        let body = DiffAnalysisRequestBodyStruct {
            base_db: Some("invalid/path.db".to_string()),
            live_db: Some("rocksdb:test/live.db".to_string()),
        };

        let result = validate_request_body_fields(&body);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code, "INVALID_DB_PATH_FORMAT");
    }

    #[test]
    fn test_validate_max_hops_default() {
        let result = validate_max_hops_parameter(None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DEFAULT_MAX_HOPS_VALUE);
    }

    #[test]
    fn test_validate_max_hops_custom() {
        let result = validate_max_hops_parameter(Some(5));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5);
    }

    #[test]
    fn test_validate_max_hops_at_limit() {
        let result = validate_max_hops_parameter(Some(MAXIMUM_ALLOWED_HOPS_LIMIT));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), MAXIMUM_ALLOWED_HOPS_LIMIT);
    }

    #[test]
    fn test_validate_max_hops_exceeds_limit() {
        let result = validate_max_hops_parameter(Some(MAXIMUM_ALLOWED_HOPS_LIMIT + 1));
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code, "MAX_HOPS_EXCEEDED");
    }

    #[test]
    fn test_validate_max_hops_zero() {
        let result = validate_max_hops_parameter(Some(0));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}

//! Blast radius impact analysis endpoint handler
//!
//! # 4-Word Naming: blast_radius_impact_handler
//!
//! Endpoint: GET /blast-radius-impact-analysis?entity={key}&hops=N
//!
//! Returns all entities that would be affected if the source entity changes.
//! This is a transitive closure of REVERSE dependencies (callers) up to N hops.
//! Blast radius = "If I change X, what breaks?" = entities that DEPEND ON X.
//!
//! v1.0.4 FIX: Added fuzzy key matching for stdlib function calls.
//! Edge targets may use simplified keys like `rust:fn:new:unknown:0-0`
//! but we query with full entity keys like `rust:method:new:__path:38-54`.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

use crate::http_server_startup_runner::SharedApplicationStateContainer;
use crate::scope_filter_utilities_module::parse_scope_build_filter_clause;

/// Extract function name from ISGL1 key for fuzzy matching
///
/// # 4-Word Name: extract_function_name_key
///
/// # Examples
/// - "rust:fn:main:path:1-50" -> Some("main")
/// - "rust:method:new:path:38-54" -> Some("new")
/// - "invalid" -> None
fn extract_function_name_key(key: &str) -> Option<&str> {
    // ISGL1 key format: language:type:name:path:lines
    let parts: Vec<&str> = key.split(':').collect();
    if parts.len() >= 3 {
        Some(parts[2])
    } else {
        None
    }
}

/// Query parameters for blast radius endpoint
///
/// # 4-Word Name: BlastRadiusQueryParamsStruct
#[derive(Debug, Deserialize)]
pub struct BlastRadiusQueryParamsStruct {
    /// Entity key to analyze (required)
    pub entity: String,
    /// Maximum hops to traverse (default: 3)
    #[serde(default = "default_hops")]
    pub hops: usize,
    /// Filter by folder scope (e.g., "crates||parseltongue-core")
    pub scope: Option<String>,
}

fn default_hops() -> usize {
    3
}

/// Single hop data in blast radius
///
/// # 4-Word Name: BlastRadiusHopDataItem
#[derive(Debug, Serialize)]
pub struct BlastRadiusHopDataItem {
    pub hop: usize,
    pub count: usize,
    pub entities: Vec<String>,
}

/// Blast radius response data
///
/// # 4-Word Name: BlastRadiusDataPayloadStruct
#[derive(Debug, Serialize)]
pub struct BlastRadiusDataPayloadStruct {
    pub source_entity: String,
    pub hops_requested: usize,
    pub total_affected: usize,
    pub by_hop: Vec<BlastRadiusHopDataItem>,
}

/// Blast radius response payload
///
/// # 4-Word Name: BlastRadiusResponsePayloadStruct
#[derive(Debug, Serialize)]
pub struct BlastRadiusResponsePayloadStruct {
    pub success: bool,
    pub endpoint: String,
    pub data: BlastRadiusDataPayloadStruct,
    pub tokens: usize,
}

/// Blast radius error response
///
/// # 4-Word Name: BlastRadiusErrorResponseStruct
#[derive(Debug, Serialize)]
pub struct BlastRadiusErrorResponseStruct {
    pub success: bool,
    pub error: String,
    pub endpoint: String,
    pub tokens: usize,
}

/// Handle blast radius impact analysis request
///
/// # 4-Word Name: handle_blast_radius_impact_analysis
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns entities affected by changes to source
/// - Performance: <100ms for reasonable hop counts
/// - Error Handling: Returns 400 for missing entity, 404 for no results
///
/// # URL Pattern
/// - Endpoint: GET /blast-radius-impact-analysis?entity={key}&hops=N
/// - Default hops: 3
pub async fn handle_blast_radius_impact_analysis(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<BlastRadiusQueryParamsStruct>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Validate entity parameter
    if params.entity.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(BlastRadiusErrorResponseStruct {
                success: false,
                error: "Entity parameter is required".to_string(),
                endpoint: "/blast-radius-impact-analysis".to_string(),
                tokens: 35,
            }),
        ).into_response();
    }

    // Compute blast radius using BFS traversal
    let by_hop = compute_blast_radius_by_hops(&state, &params.entity, params.hops, &params.scope).await;

    // Calculate total affected
    let total_affected: usize = by_hop.iter().map(|h| h.count).sum();

    // Estimate tokens
    let tokens = 80 + (total_affected * 30) + params.entity.len();

    if total_affected == 0 {
        return (
            StatusCode::NOT_FOUND,
            Json(BlastRadiusErrorResponseStruct {
                success: false,
                error: format!("No affected entities found for: {}", params.entity),
                endpoint: "/blast-radius-impact-analysis".to_string(),
                tokens,
            }),
        ).into_response();
    }

    (
        StatusCode::OK,
        Json(BlastRadiusResponsePayloadStruct {
            success: true,
            endpoint: "/blast-radius-impact-analysis".to_string(),
            data: BlastRadiusDataPayloadStruct {
                source_entity: params.entity,
                hops_requested: params.hops,
                total_affected,
                by_hop,
            },
            tokens,
        }),
    ).into_response()
}

/// Compute blast radius using BFS traversal with fuzzy key matching
///
/// # 4-Word Name: compute_blast_radius_by_hops
///
/// Uses breadth-first search to find all entities that DEPEND ON
/// the source entity (callers) within the specified number of hops.
/// This answers: "If I change X, what else might break?"
///
/// # v1.0.4 Fix
/// Added fuzzy key matching: Edge targets may use simplified keys like
/// `rust:fn:new:unknown:0-0` but we query with full entity keys.
/// Now matches on function name pattern when exact key match fails.
async fn compute_blast_radius_by_hops(
    state: &SharedApplicationStateContainer,
    source_entity: &str,
    max_hops: usize,
    scope_filter: &Option<String>,
) -> Vec<BlastRadiusHopDataItem> {
    // Clone Arc, release lock, then await
    let storage = {
        let db_guard = state.database_storage_connection_arc.read().await;
        match db_guard.as_ref() {
            Some(s) => s.clone(),
            None => return Vec::new(),
        }
    }; // Lock released here

    // Build scope filter clause
    let scope_clause = parse_scope_build_filter_clause(scope_filter);
    let scope_join = if scope_clause.is_empty() {
        String::new()
    } else {
        format!(", *CodeGraph{{ISGL1_key: from_key, root_subfolder_L1, root_subfolder_L2}}{}", scope_clause)
    };

    let mut result: Vec<BlastRadiusHopDataItem> = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut current_frontier: VecDeque<String> = VecDeque::new();

    // Start from source
    visited.insert(source_entity.to_string());
    current_frontier.push_back(source_entity.to_string());

    for hop in 1..=max_hops {
        let mut next_frontier: VecDeque<String> = VecDeque::new();
        let mut hop_entities: Vec<String> = Vec::new();

        // Process all entities in current frontier
        while let Some(entity) = current_frontier.pop_front() {
            // Query REVERSE dependencies (what CALLS this entity)
            // Blast radius = entities that depend on source
            let escaped_entity = entity
                .replace('\\', "\\\\")
                .replace('"', "\\\"");

            // v1.0.4: Build fuzzy matching query based on function name
            let query = match extract_function_name_key(&entity) {
                Some(func_name) => {
                    let escaped_func_name = func_name
                        .replace('\\', "\\\\")
                        .replace('"', "\\\"");
                    // Fuzzy match: exact key OR keys with matching function name
                    format!(
                        r#"
                        ?[from_key] := *DependencyEdges{{from_key, to_key}},
                            (to_key == "{}" or
                             starts_with(to_key, "rust:fn:{}:") or
                             starts_with(to_key, "rust:method:{}:")){}
                        "#,
                        escaped_entity, escaped_func_name, escaped_func_name, scope_join
                    )
                }
                None => {
                    // Fallback to exact match only
                    format!(
                        r#"
                        ?[from_key] := *DependencyEdges{{from_key, to_key}},
                            to_key == "{}"{}
                        "#,
                        escaped_entity, scope_join
                    )
                }
            };

            if let Ok(query_result) = storage.raw_query(&query).await {
                for row in query_result.rows {
                    if let Some(caller_key) = extract_string_value(&row[0]) {
                        if !visited.contains(&caller_key) {
                            visited.insert(caller_key.clone());
                            hop_entities.push(caller_key.clone());
                            next_frontier.push_back(caller_key);
                        }
                    }
                }
            }
        }

        // Only add hop if we found entities
        if !hop_entities.is_empty() {
            result.push(BlastRadiusHopDataItem {
                hop,
                count: hop_entities.len(),
                entities: hop_entities,
            });
        }

        // Move to next frontier
        current_frontier = next_frontier;

        // Stop if no more entities to explore
        if current_frontier.is_empty() {
            break;
        }
    }

    result
}

/// Extract string value from CozoDB DataValue
///
/// # 4-Word Name: extract_string_value_helper
fn extract_string_value(value: &cozo::DataValue) -> Option<String> {
    match value {
        cozo::DataValue::Str(s) => Some(s.to_string()),
        cozo::DataValue::Null => None,
        _ => Some(format!("{:?}", value)),
    }
}

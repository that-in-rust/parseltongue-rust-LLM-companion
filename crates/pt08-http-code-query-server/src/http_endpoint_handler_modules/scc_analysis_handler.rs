//! Strongly connected components analysis endpoint handler
//!
//! # 4-Word Naming: scc_analysis_handler
//!
//! Endpoint: GET /strongly-connected-components-analysis
//!
//! Discovers all strongly connected components (SCCs) in the dependency graph
//! using Tarjan's algorithm. An SCC is a maximal group of entities where every
//! node can reach every other node through directed paths.
//!
//! Performance: <100ms at p99 for 10,000 entities, O(V+E) single-pass algorithm.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::http_server_startup_runner::SharedApplicationStateContainer;

/// Query parameters for SCC analysis endpoint
///
/// # 4-Word Name: SccAnalysisQueryParamsStruct
#[derive(Debug, Deserialize, Default)]
pub struct SccAnalysisQueryParamsStruct {
    /// Minimum component size to include (1-1000)
    #[serde(default = "default_min_size_value")]
    pub min_size: Option<usize>,

    /// Include singleton components (size=1)
    #[serde(default = "default_include_singletons_value")]
    pub include_singletons: Option<bool>,
}

fn default_min_size_value() -> Option<usize> {
    Some(2) // Exclude singletons by default
}

fn default_include_singletons_value() -> Option<bool> {
    Some(false)
}

/// Single SCC component entry
///
/// # 4-Word Name: SccComponentEntryPayload
#[derive(Debug, Serialize)]
pub struct SccComponentEntryPayload {
    /// Component ID (1-based, ordered by size desc)
    pub id: usize,

    /// Number of entities in component
    pub size: usize,

    /// True if size > 1 (actual cycle)
    pub is_cyclic: bool,

    /// Entity keys in this component
    pub members: Vec<String>,
}

/// SCC analysis response data
///
/// # 4-Word Name: SccAnalysisDataPayload
#[derive(Debug, Serialize)]
pub struct SccAnalysisDataPayload {
    /// Total components found (before filtering)
    pub total_components: usize,

    /// Count of cyclic components (size > 1)
    pub cyclic_components: usize,

    /// Count of components in response
    pub components_returned: usize,

    /// Size of largest component
    pub largest_component_size: usize,

    /// Algorithm used
    pub algorithm: String,

    /// List of components
    pub components: Vec<SccComponentEntryPayload>,
}

/// SCC analysis response payload
///
/// # 4-Word Name: SccAnalysisResponsePayload
#[derive(Debug, Serialize)]
pub struct SccAnalysisResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: SccAnalysisDataPayload,
    pub tokens: usize,
}

/// SCC analysis error response
///
/// # 4-Word Name: SccAnalysisErrorResponseStruct
#[derive(Debug, Serialize)]
pub struct SccAnalysisErrorResponseStruct {
    pub success: bool,
    pub error: String,
    pub endpoint: String,
    pub tokens: usize,
}

/// Tarjan algorithm node state
///
/// # 4-Word Name: TarjanNodeStateStruct
#[derive(Debug, Clone)]
struct TarjanNodeStateStruct {
    /// DFS discovery index
    index: usize,

    /// Lowest index reachable from this node
    lowlink: usize,

    /// Whether node is on DFS stack
    on_stack: bool,
}

/// Handle strongly connected components analysis request
///
/// # 4-Word Name: handle_scc_analysis_request
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns all SCCs using Tarjan's algorithm
/// - Performance: <100ms at p99 for 10,000 entities, O(V+E) complexity
/// - Error Handling: Returns 400 for invalid parameters, 200 with empty array for no SCCs
///
/// # Algorithm
/// Uses Tarjan's strongly connected components algorithm:
/// - Single DFS pass with index/lowlink tracking
/// - Stack-based SCC extraction when root node found
/// - O(V+E) time, O(V) space complexity
///
/// # URL Pattern
/// - Endpoint: GET /strongly-connected-components-analysis?min_size=N&include_singletons=BOOL
/// - Default min_size: 2 (cyclic only)
/// - Default include_singletons: false
pub async fn handle_scc_analysis_request(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<SccAnalysisQueryParamsStruct>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Validate min_size parameter
    let min_size = params.min_size.unwrap_or(2);
    if min_size < 1 || min_size > 1000 {
        return (
            StatusCode::BAD_REQUEST,
            Json(SccAnalysisErrorResponseStruct {
                success: false,
                error: format!(
                    "Invalid min_size parameter. Must be between 1 and 1000. Got: {}",
                    min_size
                ),
                endpoint: "/strongly-connected-components-analysis".to_string(),
                tokens: 50,
            }),
        ).into_response();
    }

    let include_singletons = params.include_singletons.unwrap_or(false);

    // Run Tarjan's algorithm to find all SCCs
    let all_sccs = tarjan_find_scc_groups(&state).await;

    // Calculate statistics
    let total_components = all_sccs.len();
    let cyclic_components = all_sccs.iter().filter(|scc| scc.len() > 1).count();
    let largest_component_size = all_sccs.iter().map(|scc| scc.len()).max().unwrap_or(0);

    // Filter SCCs based on parameters
    let mut filtered_sccs: Vec<Vec<String>> = if include_singletons {
        // Include all components, but still respect min_size if explicitly set
        if params.min_size.is_some() {
            all_sccs.into_iter().filter(|scc| scc.len() >= min_size).collect()
        } else {
            all_sccs
        }
    } else {
        // Exclude singletons by default
        all_sccs.into_iter().filter(|scc| scc.len() >= min_size).collect()
    };

    // Sort by size descending (largest first)
    filtered_sccs.sort_by(|a, b| b.len().cmp(&a.len()));

    // Convert to response format with IDs
    let components: Vec<SccComponentEntryPayload> = filtered_sccs
        .into_iter()
        .enumerate()
        .map(|(idx, members)| {
            let size = members.len();
            SccComponentEntryPayload {
                id: idx + 1, // 1-based ID
                size,
                is_cyclic: size > 1,
                members,
            }
        })
        .collect();

    let components_returned = components.len();

    // Estimate tokens
    let tokens = estimate_response_token_count(&components);

    (
        StatusCode::OK,
        Json(SccAnalysisResponsePayload {
            success: true,
            endpoint: "/strongly-connected-components-analysis".to_string(),
            data: SccAnalysisDataPayload {
                total_components,
                cyclic_components,
                components_returned,
                largest_component_size,
                algorithm: "tarjan".to_string(),
                components,
            },
            tokens,
        }),
    ).into_response()
}

/// Find all strongly connected components using Tarjan's algorithm
///
/// # 4-Word Name: tarjan_find_scc_groups
///
/// # Algorithm
/// Tarjan's algorithm (1972) uses a single DFS pass with:
/// - index: DFS discovery time
/// - lowlink: Lowest index reachable from this node
/// - on_stack: Whether node is currently on DFS stack
///
/// When a root node is found (index == lowlink), all nodes on stack
/// up to root form a complete SCC.
///
/// # Performance
/// - Time: O(V+E) - single DFS pass
/// - Space: O(V) - state tracking per node
async fn tarjan_find_scc_groups(
    state: &SharedApplicationStateContainer,
) -> Vec<Vec<String>> {
    let db_guard = state.database_storage_connection_arc.read().await;
    let storage = match db_guard.as_ref() {
        Some(s) => s,
        None => return Vec::new(),
    };

    // Fetch all edges from database
    let query = "?[from_key, to_key] := *DependencyEdges{from_key, to_key}";
    let edges = match storage.raw_query(query).await {
        Ok(result) => result.rows,
        Err(_) => return Vec::new(),
    };

    // Build adjacency list representation
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();

    for row in edges {
        if row.len() >= 2 {
            let from = extract_string_value_helper(&row[0]).unwrap_or_default();
            let to = extract_string_value_helper(&row[1]).unwrap_or_default();

            if !from.is_empty() && !to.is_empty() {
                graph.entry(from).or_default().push(to);
            }
        }
    }

    // Initialize Tarjan's algorithm state
    let mut index_counter = 0;
    let mut stack: Vec<String> = Vec::new();
    let mut state_map: HashMap<String, TarjanNodeStateStruct> = HashMap::new();
    let mut sccs: Vec<Vec<String>> = Vec::new();

    // Get all nodes (both sources and targets)
    let all_nodes: Vec<String> = graph.keys().cloned().collect();

    // Run DFS from each unvisited node
    for node in all_nodes {
        if !state_map.contains_key(&node) {
            tarjan_strongconnect_recursive_dfs(
                &node,
                &graph,
                &mut index_counter,
                &mut stack,
                &mut state_map,
                &mut sccs,
            );
        }
    }

    sccs
}

/// Tarjan's strongconnect recursive DFS
///
/// # 4-Word Name: tarjan_strongconnect_recursive_dfs
///
/// This is the core recursive function of Tarjan's algorithm.
/// It performs DFS while maintaining index/lowlink values and
/// extracts SCCs when root nodes are found.
fn tarjan_strongconnect_recursive_dfs(
    v: &str,
    graph: &HashMap<String, Vec<String>>,
    index_counter: &mut usize,
    stack: &mut Vec<String>,
    state_map: &mut HashMap<String, TarjanNodeStateStruct>,
    sccs: &mut Vec<Vec<String>>,
) {
    // Set depth index and lowlink for v
    let current_index = *index_counter;
    state_map.insert(
        v.to_string(),
        TarjanNodeStateStruct {
            index: current_index,
            lowlink: current_index,
            on_stack: true,
        },
    );
    *index_counter += 1;
    stack.push(v.to_string());

    // Explore successors
    if let Some(successors) = graph.get(v) {
        for w in successors {
            if !state_map.contains_key(w) {
                // Successor w not yet visited; recurse
                tarjan_strongconnect_recursive_dfs(
                    w,
                    graph,
                    index_counter,
                    stack,
                    state_map,
                    sccs,
                );

                // Update lowlink after recursion
                let w_lowlink = state_map.get(w).unwrap().lowlink;
                let v_state = state_map.get_mut(v).unwrap();
                v_state.lowlink = v_state.lowlink.min(w_lowlink);
            } else if state_map.get(w).unwrap().on_stack {
                // Successor w is on stack (part of current SCC)
                let w_index = state_map.get(w).unwrap().index;
                let v_state = state_map.get_mut(v).unwrap();
                v_state.lowlink = v_state.lowlink.min(w_index);
            }
        }
    }

    // If v is a root node, pop the stack to create SCC
    let v_state = state_map.get(v).unwrap();
    if v_state.lowlink == v_state.index {
        let mut scc = Vec::new();
        loop {
            let w = stack.pop().unwrap();
            state_map.get_mut(&w).unwrap().on_stack = false;
            scc.push(w.clone());
            if w == v {
                break;
            }
        }
        sccs.push(scc);
    }
}

/// Estimate token count for SCC response
///
/// # 4-Word Name: estimate_response_token_count
///
/// Token estimation:
/// - Base response structure: 100 tokens
/// - Per component metadata: ~15 tokens
/// - Per member entity key: ~12 tokens
fn estimate_response_token_count(
    components: &[SccComponentEntryPayload],
) -> usize {
    // Base response structure
    let base = 100;

    // Per component metadata
    let component_overhead = components.len() * 15;

    // Per member entity key
    let total_members: usize = components.iter()
        .map(|c| c.members.len())
        .sum();
    let member_tokens = total_members * 12;

    base + component_overhead + member_tokens
}

/// Extract string value from CozoDB DataValue
///
/// # 4-Word Name: extract_string_value_helper
fn extract_string_value_helper(value: &cozo::DataValue) -> Option<String> {
    match value {
        cozo::DataValue::Str(s) => Some(s.to_string()),
        cozo::DataValue::Null => None,
        _ => Some(format!("{:?}", value)),
    }
}

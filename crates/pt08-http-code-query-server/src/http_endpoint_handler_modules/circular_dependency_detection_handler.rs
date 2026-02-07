//! Circular dependency detection endpoint handler
//!
//! # 4-Word Naming: circular_dependency_detection_handler
//!
//! Endpoint: GET /circular-dependency-detection-scan
//!
//! Detects cycles in the dependency graph using DFS-based cycle detection.
//! Cycles indicate architectural problems where A → B → C → A.

use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

use crate::http_server_startup_runner::SharedApplicationStateContainer;

/// Single cycle data structure
///
/// # 4-Word Name: DetectedCycleDataPayload
#[derive(Debug, Serialize)]
pub struct DetectedCycleDataPayload {
    pub length: usize,
    pub path: Vec<String>,
}

/// Circular dependency scan response data
///
/// # 4-Word Name: CircularDependencyDataPayload
#[derive(Debug, Serialize)]
pub struct CircularDependencyDataPayload {
    pub has_cycles: bool,
    pub cycle_count: usize,
    pub cycles: Vec<DetectedCycleDataPayload>,
}

/// Circular dependency response payload
///
/// # 4-Word Name: CircularDependencyResponsePayload
#[derive(Debug, Serialize)]
pub struct CircularDependencyResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: CircularDependencyDataPayload,
    pub tokens: usize,
}

/// Handle circular dependency detection scan request
///
/// # 4-Word Name: handle_circular_dependency_detection_scan
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns all detected cycles in the graph
/// - Performance: O(V + E) using DFS-based detection
/// - Error Handling: Returns empty cycles array if no cycles
///
/// # Algorithm
/// Uses depth-first search with coloring:
/// - WHITE (0): Unvisited
/// - GRAY (1): Currently in recursion stack (visiting)
/// - BLACK (2): Completely processed
///
/// A cycle exists when we encounter a GRAY node during DFS.
pub async fn handle_circular_dependency_detection_scan(
    State(state): State<SharedApplicationStateContainer>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Detect cycles using DFS
    let cycles = detect_cycles_using_dfs_traversal(&state).await;

    let has_cycles = !cycles.is_empty();
    let cycle_count = cycles.len();

    // Estimate tokens
    let total_path_len: usize = cycles.iter().map(|c| c.path.len()).sum();
    let tokens = 60 + (cycle_count * 20) + (total_path_len * 15);

    (
        StatusCode::OK,
        Json(CircularDependencyResponsePayload {
            success: true,
            endpoint: "/circular-dependency-detection-scan".to_string(),
            data: CircularDependencyDataPayload {
                has_cycles,
                cycle_count,
                cycles,
            },
            tokens,
        }),
    ).into_response()
}

/// Detect cycles using DFS traversal algorithm
///
/// # 4-Word Name: detect_cycles_using_dfs_traversal
///
/// Uses DFS with three-color marking to detect back edges.
/// When a GRAY node is encountered, we've found a cycle.
async fn detect_cycles_using_dfs_traversal(
    state: &SharedApplicationStateContainer,
) -> Vec<DetectedCycleDataPayload> {
    // Clone Arc, release lock, then await
    let storage = {
        let db_guard = state.database_storage_connection_arc.read().await;
        match db_guard.as_ref() {
            Some(s) => s.clone(),
            None => return Vec::new(),
        }
    }; // Lock released here

    // Build adjacency list from edges
    let query = "?[from_key, to_key] := *DependencyEdges{from_key, to_key}";
    let edges = match storage.raw_query(query).await {
        Ok(result) => result.rows,
        Err(_) => return Vec::new(),
    };

    // Build graph as adjacency list
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    let mut all_nodes: HashSet<String> = HashSet::new();

    for row in edges {
        if row.len() >= 2 {
            let from = extract_string_value(&row[0]).unwrap_or_default();
            let to = extract_string_value(&row[1]).unwrap_or_default();

            all_nodes.insert(from.clone());
            all_nodes.insert(to.clone());

            graph.entry(from).or_default().push(to);
        }
    }

    // DFS with coloring
    // 0 = WHITE (unvisited), 1 = GRAY (in stack), 2 = BLACK (done)
    let mut color: HashMap<String, u8> = HashMap::new();
    let mut cycles: Vec<DetectedCycleDataPayload> = Vec::new();

    for node in &all_nodes {
        if *color.get(node).unwrap_or(&0) == 0 {
            let mut path: Vec<String> = Vec::new();
            dfs_find_cycles_recursive(
                node,
                &graph,
                &mut color,
                &mut path,
                &mut cycles,
            );
        }
    }

    cycles
}

/// DFS recursive function for cycle detection
///
/// # 4-Word Name: dfs_find_cycles_recursive
fn dfs_find_cycles_recursive(
    node: &str,
    graph: &HashMap<String, Vec<String>>,
    color: &mut HashMap<String, u8>,
    path: &mut Vec<String>,
    cycles: &mut Vec<DetectedCycleDataPayload>,
) {
    // Mark as GRAY (visiting)
    color.insert(node.to_string(), 1);
    path.push(node.to_string());

    // Explore neighbors
    if let Some(neighbors) = graph.get(node) {
        for neighbor in neighbors {
            let neighbor_color = *color.get(neighbor).unwrap_or(&0);

            if neighbor_color == 1 {
                // Found a cycle! Back edge to a GRAY node
                // Extract cycle from path
                if let Some(cycle_start) = path.iter().position(|n| n == neighbor) {
                    let mut cycle_path: Vec<String> = path[cycle_start..].to_vec();
                    cycle_path.push(neighbor.clone()); // Complete the cycle

                    cycles.push(DetectedCycleDataPayload {
                        length: cycle_path.len() - 1, // Edges in cycle
                        path: cycle_path,
                    });
                }
            } else if neighbor_color == 0 {
                // Unvisited, recurse
                dfs_find_cycles_recursive(neighbor, graph, color, path, cycles);
            }
            // If BLACK (2), already processed, skip
        }
    }

    // Mark as BLACK (done)
    color.insert(node.to_string(), 2);
    path.pop();
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

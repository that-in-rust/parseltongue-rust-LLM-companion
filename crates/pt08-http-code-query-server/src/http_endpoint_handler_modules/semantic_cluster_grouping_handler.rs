//! Semantic cluster grouping endpoint handler
//!
//! # 4-Word Naming: semantic_cluster_grouping_handler
//!
//! Endpoint: GET /semantic-cluster-grouping-list
//!
//! Groups code entities into semantic clusters based on their dependency
//! relationships using Label Propagation Algorithm (LPA).
//! Entities that call each other frequently belong to the same cluster.

use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

use crate::http_server_startup_runner::SharedApplicationStateContainer;

/// Single cluster entry in the response
///
/// # 4-Word Name: SemanticClusterEntryPayload
#[derive(Debug, Serialize)]
pub struct SemanticClusterEntryPayload {
    pub cluster_id: usize,
    pub entity_count: usize,
    pub entities: Vec<String>,
    pub internal_edges: usize,
    pub external_edges: usize,
}

/// Semantic cluster grouping response data
///
/// # 4-Word Name: SemanticClusterDataPayload
#[derive(Debug, Serialize)]
pub struct SemanticClusterDataPayload {
    pub total_entities: usize,
    pub cluster_count: usize,
    pub clusters: Vec<SemanticClusterEntryPayload>,
}

/// Semantic cluster grouping response payload
///
/// # 4-Word Name: SemanticClusterResponsePayload
#[derive(Debug, Serialize)]
pub struct SemanticClusterResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: SemanticClusterDataPayload,
    pub tokens: usize,
}

/// Handle semantic cluster grouping list request
///
/// # 4-Word Name: handle_semantic_cluster_grouping_list
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns entities grouped into clusters
/// - Performance: O(E * iterations) where E is edges, iterations ~5
/// - Error Handling: Returns empty clusters if no edges
///
/// # Algorithm: Label Propagation (LPA)
/// 1. Assign unique label to each node
/// 2. Each node adopts most common neighbor label
/// 3. Repeat until convergence (~5 iterations)
/// 4. Group nodes by final label
pub async fn handle_semantic_cluster_grouping_list(
    State(state): State<SharedApplicationStateContainer>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Run label propagation clustering
    let clusters = run_label_propagation_clustering(&state).await;

    let total_entities: usize = clusters.iter().map(|c| c.entity_count).sum();
    let cluster_count = clusters.len();

    // Estimate tokens
    let total_entity_names: usize = clusters.iter()
        .map(|c| c.entities.iter().map(|e| e.len()).sum::<usize>())
        .sum();
    let tokens = 80 + (cluster_count * 30) + (total_entity_names / 4);

    (
        StatusCode::OK,
        Json(SemanticClusterResponsePayload {
            success: true,
            endpoint: "/semantic-cluster-grouping-list".to_string(),
            data: SemanticClusterDataPayload {
                total_entities,
                cluster_count,
                clusters,
            },
            tokens,
        }),
    ).into_response()
}

/// Run label propagation clustering algorithm
///
/// # 4-Word Name: run_label_propagation_clustering
///
/// Uses Label Propagation Algorithm (LPA):
/// - Each node starts with unique label
/// - Iteratively adopt most common neighbor label
/// - Converge when labels stabilize
async fn run_label_propagation_clustering(
    state: &SharedApplicationStateContainer,
) -> Vec<SemanticClusterEntryPayload> {
    // Clone Arc, release lock, then await
    let storage = {
        let db_guard = state.database_storage_connection_arc.read().await;
        match db_guard.as_ref() {
            Some(s) => s.clone(),
            None => return Vec::new(),
        }
    }; // Lock released here

    // Query all edges
    let query = "?[from_key, to_key] := *DependencyEdges{from_key, to_key}";
    let edges = match storage.raw_query(query).await {
        Ok(result) => result.rows,
        Err(_) => return Vec::new(),
    };

    // Build bidirectional adjacency list (treat as undirected for clustering)
    let mut graph: HashMap<String, HashSet<String>> = HashMap::new();
    let mut all_nodes: HashSet<String> = HashSet::new();
    let mut edge_list: Vec<(String, String)> = Vec::new();

    for row in edges {
        if row.len() >= 2 {
            let from = extract_string_value_helper(&row[0]).unwrap_or_default();
            let to = extract_string_value_helper(&row[1]).unwrap_or_default();

            all_nodes.insert(from.clone());
            all_nodes.insert(to.clone());

            // Bidirectional for clustering
            graph.entry(from.clone()).or_default().insert(to.clone());
            graph.entry(to.clone()).or_default().insert(from.clone());

            edge_list.push((from, to));
        }
    }

    if all_nodes.is_empty() {
        return Vec::new();
    }

    // Initialize labels: each node gets its own label
    let nodes: Vec<String> = all_nodes.into_iter().collect();
    let mut labels: HashMap<String, usize> = nodes
        .iter()
        .enumerate()
        .map(|(idx, node)| (node.clone(), idx))
        .collect();

    // Label propagation iterations
    let max_iterations = 10;
    for _ in 0..max_iterations {
        let mut changed = false;

        for node in &nodes {
            if let Some(neighbors) = graph.get(node) {
                if neighbors.is_empty() {
                    continue;
                }

                // Count neighbor labels
                let mut label_counts: HashMap<usize, usize> = HashMap::new();
                for neighbor in neighbors {
                    if let Some(&label) = labels.get(neighbor) {
                        *label_counts.entry(label).or_insert(0) += 1;
                    }
                }

                // Find most common label
                if let Some((&most_common_label, _)) = label_counts
                    .iter()
                    .max_by_key(|(_, &count)| count)
                {
                    let current_label = *labels.get(node).unwrap();
                    if most_common_label != current_label {
                        labels.insert(node.clone(), most_common_label);
                        changed = true;
                    }
                }
            }
        }

        if !changed {
            break; // Converged
        }
    }

    // Group nodes by label
    let mut clusters_map: HashMap<usize, Vec<String>> = HashMap::new();
    for (node, label) in &labels {
        clusters_map.entry(*label).or_default().push(node.clone());
    }

    // Build cluster response with edge counts
    let mut clusters: Vec<SemanticClusterEntryPayload> = clusters_map
        .into_iter()
        .enumerate()
        .map(|(idx, (_, entities))| {
            let entity_set: HashSet<&String> = entities.iter().collect();

            // Count internal vs external edges
            let mut internal_edges = 0;
            let mut external_edges = 0;

            for (from, to) in &edge_list {
                let from_in = entity_set.contains(from);
                let to_in = entity_set.contains(to);

                if from_in && to_in {
                    internal_edges += 1;
                } else if from_in || to_in {
                    external_edges += 1;
                }
            }

            SemanticClusterEntryPayload {
                cluster_id: idx + 1,
                entity_count: entities.len(),
                entities,
                internal_edges,
                external_edges,
            }
        })
        .collect();

    // Sort by entity count descending
    clusters.sort_by(|a, b| b.entity_count.cmp(&a.entity_count));

    // Renumber cluster IDs after sorting
    for (idx, cluster) in clusters.iter_mut().enumerate() {
        cluster.cluster_id = idx + 1;
    }

    clusters
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

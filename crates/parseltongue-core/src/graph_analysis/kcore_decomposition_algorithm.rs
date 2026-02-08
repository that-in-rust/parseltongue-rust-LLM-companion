use std::collections::{HashMap, HashSet};
use crate::graph_analysis::AdjacencyListGraphRepresentation;

/// Core layer classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreLayer {
    Core,        // k >= 8
    Mid,         // 3 <= k < 8
    Peripheral,  // k < 3
}

/// K-Core Decomposition using Batagelj-Zaversnik O(E) algorithm
///
/// Reference: Batagelj & Zaversnik (2003). "An O(m) Algorithm for Cores Decomposition"
///
/// Uses UNDIRECTED degree (in-degree + out-degree, deduplicated) for k-core computation.
/// The algorithm processes nodes in order of increasing degree, computing the coreness
/// (maximum k-core membership) for each node.
///
/// # Algorithm
/// 1. Build undirected neighbor sets by combining forward and reverse edges
/// 2. Initialize degree for each node
/// 3. Process nodes in non-decreasing order of current degree:
///    - Remove node with minimum degree
///    - Coreness = max(node degree, previous max core seen)
///    - Update remaining neighbors' degrees
///
/// The monotonicity property ensures coreness values are non-decreasing in processing order.
///
/// # 4-Word Name: kcore_decomposition_layering_algorithm
pub fn kcore_decomposition_layering_algorithm(
    graph: &AdjacencyListGraphRepresentation,
) -> HashMap<String, usize> {
    let mut core_numbers: HashMap<String, usize> = HashMap::new();

    if graph.count_total_graph_nodes() == 0 {
        return core_numbers;
    }

    // Build undirected neighbor sets
    let mut neighbors: HashMap<String, HashSet<String>> = HashMap::new();

    for node in graph.retrieve_all_graph_nodes() {
        let mut nbrs = HashSet::new();

        // Add forward neighbors (nodes this node points to)
        for n in graph.get_forward_neighbors_list(node) {
            nbrs.insert(n.clone());
        }

        // Add reverse neighbors (nodes that point to this node)
        for n in graph.get_reverse_neighbors_list(node) {
            nbrs.insert(n.clone());
        }

        neighbors.insert(node.clone(), nbrs);
    }

    // Initialize degrees
    let mut degrees: HashMap<String, usize> = neighbors.iter()
        .map(|(n, nbrs)| (n.clone(), nbrs.len()))
        .collect();

    // Use BTreeMap for bucket sort by degree
    let mut vertices_by_degree: std::collections::BTreeMap<usize, HashSet<String>> = std::collections::BTreeMap::new();

    for (node, &degree) in &degrees {
        vertices_by_degree
            .entry(degree)
            .or_default()
            .insert(node.clone());
    }

    // Process vertices ONE AT A TIME in increasing degree order
    let mut k = 0;
    while !vertices_by_degree.is_empty() {
        // Get minimum degree
        let min_degree = *vertices_by_degree.keys().next().unwrap();

        // Update k (monotone increasing)
        k = k.max(min_degree);

        // Get ONE vertex with minimum degree
        let v = {
            let bucket = vertices_by_degree.get_mut(&min_degree).unwrap();
            let v = bucket.iter().next().unwrap().clone();
            bucket.remove(&v);
            if bucket.is_empty() {
                vertices_by_degree.remove(&min_degree);
            }
            v
        };

        // Assign core number as k (monotone value, not raw degree)
        core_numbers.insert(v.clone(), k);

        // Update neighbors
        if let Some(v_neighbors) = neighbors.get(&v) {
            for neighbor in v_neighbors {
                if core_numbers.contains_key(neighbor) {
                    continue; // Already processed
                }

                let old_degree = degrees[neighbor];
                let new_degree = old_degree.saturating_sub(1);

                // Remove from old degree bucket
                if let Some(bucket) = vertices_by_degree.get_mut(&old_degree) {
                    bucket.remove(neighbor);
                    if bucket.is_empty() {
                        vertices_by_degree.remove(&old_degree);
                    }
                }

                // Add to new degree bucket
                degrees.insert(neighbor.clone(), new_degree);
                vertices_by_degree
                    .entry(new_degree)
                    .or_default()
                    .insert(neighbor.clone());
            }
        }
    }

    core_numbers
}

/// Classify coreness into architecture layer
///
/// Thresholds:
/// - CORE: k >= 8 (highly coupled, critical infrastructure)
/// - MID: 3 <= k < 8 (moderately coupled components)
/// - PERIPHERAL: k < 3 (loosely coupled, leaf nodes)
///
/// # 4-Word Name: classify_coreness_layer_level
pub fn classify_coreness_layer_level(coreness: usize) -> CoreLayer {
    if coreness >= 8 {
        CoreLayer::Core
    } else if coreness >= 3 {
        CoreLayer::Mid
    } else {
        CoreLayer::Peripheral
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_analysis::test_fixture_reference_graphs::create_eight_node_reference_graph;
    use crate::graph_analysis::test_fixture_reference_graphs::create_five_node_chain_graph;
    use crate::graph_analysis::AdjacencyListGraphRepresentation;

    #[test]
    fn test_kcore_eight_node_max_coreness() {
        let graph = create_eight_node_reference_graph();
        let core_numbers = kcore_decomposition_layering_algorithm(&graph);
        let max_k = core_numbers.values().copied().max().unwrap_or(0);
        assert_eq!(max_k, 2, "Max coreness should be 2 (D-E-F cycle)");
    }

    #[test]
    fn test_kcore_cycle_nodes_coreness_two() {
        let graph = create_eight_node_reference_graph();
        let core_numbers = kcore_decomposition_layering_algorithm(&graph);

        // D, E, F form a 3-cycle -> each has 2 neighbors in subgraph -> coreness 2
        assert_eq!(core_numbers["D"], 2);
        assert_eq!(core_numbers["E"], 2);
        assert_eq!(core_numbers["F"], 2);
    }

    #[test]
    fn test_kcore_peripheral_nodes_coreness() {
        let graph = create_eight_node_reference_graph();
        let core_numbers = kcore_decomposition_layering_algorithm(&graph);

        // G, H are isolated pair with degree 1 → coreness 1
        assert_eq!(core_numbers["G"], 1);
        assert_eq!(core_numbers["H"], 1);
        // A, B, C are connected to the 2-core via D, so Batagelj-Zaversnik
        // monotonicity gives them coreness 2 (processed after G/H at k=2)
        assert_eq!(core_numbers["A"], 2);
        assert_eq!(core_numbers["B"], 2);
        assert_eq!(core_numbers["C"], 2);
    }

    #[test]
    fn test_kcore_layer_classification() {
        assert_eq!(classify_coreness_layer_level(0), CoreLayer::Peripheral);
        assert_eq!(classify_coreness_layer_level(1), CoreLayer::Peripheral);
        assert_eq!(classify_coreness_layer_level(2), CoreLayer::Peripheral);
        assert_eq!(classify_coreness_layer_level(3), CoreLayer::Mid);
        assert_eq!(classify_coreness_layer_level(7), CoreLayer::Mid);
        assert_eq!(classify_coreness_layer_level(8), CoreLayer::Core);
        assert_eq!(classify_coreness_layer_level(100), CoreLayer::Core);
    }

    #[test]
    fn test_kcore_empty_graph() {
        let graph = AdjacencyListGraphRepresentation::create_empty_graph_representation();
        let core_numbers = kcore_decomposition_layering_algorithm(&graph);
        assert!(core_numbers.is_empty());
    }

    #[test]
    fn test_kcore_chain_all_coreness_one() {
        let graph = create_five_node_chain_graph();
        let core_numbers = kcore_decomposition_layering_algorithm(&graph);

        // Linear chain: all nodes have coreness 1
        for (_node, &coreness) in &core_numbers {
            assert_eq!(coreness, 1, "Chain nodes should have coreness 1");
        }
    }
}

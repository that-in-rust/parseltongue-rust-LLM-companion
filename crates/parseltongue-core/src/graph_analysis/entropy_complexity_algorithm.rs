use std::collections::HashMap;
use crate::graph_analysis::AdjacencyListGraphRepresentation;

/// Entropy-based complexity level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntropyComplexity {
    Low,       // H < 1.0
    Moderate,  // 1.0 <= H < 1.4
    High,      // H >= 1.4 (near H_max = log2(3) = 1.585)
}

/// Calculate Shannon entropy for a single entity's outgoing edges
///
/// Reference: Shannon, C. (1948). "A Mathematical Theory of Communication"
/// Formula: H(X) = -Σ p(x) log₂ p(x)
///
/// Measures diversity of edge types. H=0 means all one type, H=1.585 means perfectly uniform across 3 types.
///
/// # 4-Word Name: calculate_entity_entropy_score
pub fn calculate_entity_entropy_score(
    graph: &AdjacencyListGraphRepresentation,
    node: &str,
) -> f64 {
    let neighbors = graph.get_forward_neighbors_list(node);
    if neighbors.is_empty() {
        return 0.0;
    }

    // Count edge type frequencies
    let mut type_counts: HashMap<&str, usize> = HashMap::new();
    let mut total = 0usize;

    for neighbor in neighbors {
        if let Some(edge_type) = graph.lookup_edge_type_between(node, neighbor) {
            *type_counts.entry(edge_type.as_str()).or_default() += 1;
            total += 1;
        }
    }

    if total == 0 {
        return 0.0;
    }

    // Shannon entropy: H = -Σ p(x) × log₂(p(x))
    let mut entropy = 0.0f64;
    for &count in type_counts.values() {
        let p = count as f64 / total as f64;
        if p > 0.0 {
            entropy -= p * p.log2();
        }
    }

    entropy
}

/// Compute entropy for ALL entities in the graph
///
/// # 4-Word Name: compute_all_entity_entropy
pub fn compute_all_entity_entropy(
    graph: &AdjacencyListGraphRepresentation,
) -> HashMap<String, f64> {
    graph.retrieve_all_graph_nodes()
        .iter()
        .map(|node| {
            let entropy = calculate_entity_entropy_score(graph, node);
            (node.clone(), entropy)
        })
        .collect()
}

/// Classify entropy into complexity level
///
/// Thresholds adjusted for 3 edge types (H_max = log2(3) = 1.585):
/// - LOW: H < 1.0
/// - MODERATE: 1.0 ≤ H < 1.4
/// - HIGH: H ≥ 1.4 (approaching uniform, indicates mixed responsibility)
///
/// # 4-Word Name: classify_entropy_complexity_level
pub fn classify_entropy_complexity_level(entropy: f64) -> EntropyComplexity {
    if entropy < 1.0 {
        EntropyComplexity::Low
    } else if entropy < 1.4 {
        EntropyComplexity::Moderate
    } else {
        EntropyComplexity::High
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_analysis::AdjacencyListGraphRepresentation;

    #[test]
    fn test_entropy_uniform_three_types() {
        // 6 edges: 2 Calls, 2 Uses, 2 Implements → uniform distribution
        let edges = vec![
            ("X".to_string(), "A".to_string(), "Calls".to_string()),
            ("X".to_string(), "B".to_string(), "Calls".to_string()),
            ("X".to_string(), "C".to_string(), "Uses".to_string()),
            ("X".to_string(), "D".to_string(), "Uses".to_string()),
            ("X".to_string(), "E".to_string(), "Implements".to_string()),
            ("X".to_string(), "F".to_string(), "Implements".to_string()),
        ];
        let graph = AdjacencyListGraphRepresentation::build_from_dependency_edges(&edges);
        let entropy = calculate_entity_entropy_score(&graph, "X");

        // H = -3 × (1/3 × log2(1/3)) = log2(3) ≈ 1.585
        assert!((entropy - 1.585).abs() < 0.01, "Expected ~1.585, got {}", entropy);
    }

    #[test]
    fn test_entropy_single_type_zero() {
        // All edges are Calls → entropy = 0
        let edges = vec![
            ("Y".to_string(), "A".to_string(), "Calls".to_string()),
            ("Y".to_string(), "B".to_string(), "Calls".to_string()),
            ("Y".to_string(), "C".to_string(), "Calls".to_string()),
        ];
        let graph = AdjacencyListGraphRepresentation::build_from_dependency_edges(&edges);
        let entropy = calculate_entity_entropy_score(&graph, "Y");

        assert!((entropy - 0.0).abs() < 0.001, "Single type should be 0 entropy, got {}", entropy);
    }

    #[test]
    fn test_entropy_no_edges_zero() {
        let mut graph = AdjacencyListGraphRepresentation::create_empty_graph_representation();
        graph.insert_node_into_graph("Z".to_string());
        let entropy = calculate_entity_entropy_score(&graph, "Z");
        assert_eq!(entropy, 0.0, "No edges should be 0 entropy");
    }

    #[test]
    fn test_entropy_nonexistent_node() {
        let graph = AdjacencyListGraphRepresentation::create_empty_graph_representation();
        let entropy = calculate_entity_entropy_score(&graph, "NOPE");
        assert_eq!(entropy, 0.0);
    }

    #[test]
    fn test_classify_entropy_levels() {
        assert_eq!(classify_entropy_complexity_level(0.0), EntropyComplexity::Low);
        assert_eq!(classify_entropy_complexity_level(0.5), EntropyComplexity::Low);
        assert_eq!(classify_entropy_complexity_level(0.99), EntropyComplexity::Low);
        assert_eq!(classify_entropy_complexity_level(1.0), EntropyComplexity::Moderate);
        assert_eq!(classify_entropy_complexity_level(1.3), EntropyComplexity::Moderate);
        assert_eq!(classify_entropy_complexity_level(1.4), EntropyComplexity::High);
        assert_eq!(classify_entropy_complexity_level(1.585), EntropyComplexity::High);
    }

    #[test]
    fn test_entropy_two_types_mixed() {
        // 3 Calls + 1 Uses → p(Calls)=0.75, p(Uses)=0.25
        // H = -(0.75×log2(0.75) + 0.25×log2(0.25)) = -(0.75×(-0.415) + 0.25×(-2)) = 0.311 + 0.5 = 0.811
        let edges = vec![
            ("M".to_string(), "A".to_string(), "Calls".to_string()),
            ("M".to_string(), "B".to_string(), "Calls".to_string()),
            ("M".to_string(), "C".to_string(), "Calls".to_string()),
            ("M".to_string(), "D".to_string(), "Uses".to_string()),
        ];
        let graph = AdjacencyListGraphRepresentation::build_from_dependency_edges(&edges);
        let entropy = calculate_entity_entropy_score(&graph, "M");

        assert!((entropy - 0.811).abs() < 0.01, "Expected ~0.811, got {}", entropy);
    }

    #[test]
    fn test_compute_all_entropy_scores() {
        let edges = vec![
            ("X".to_string(), "A".to_string(), "Calls".to_string()),
            ("X".to_string(), "B".to_string(), "Uses".to_string()),
            ("Y".to_string(), "C".to_string(), "Calls".to_string()),
            ("Y".to_string(), "D".to_string(), "Calls".to_string()),
        ];
        let graph = AdjacencyListGraphRepresentation::build_from_dependency_edges(&edges);
        let scores = compute_all_entity_entropy(&graph);

        // X has mixed types → entropy > 0
        assert!(scores["X"] > 0.0);
        // Y has all Calls → entropy = 0
        assert!((scores["Y"] - 0.0).abs() < 0.001);
        // A, B, C, D have no outgoing edges → entropy = 0
        assert_eq!(scores["A"], 0.0);
    }
}

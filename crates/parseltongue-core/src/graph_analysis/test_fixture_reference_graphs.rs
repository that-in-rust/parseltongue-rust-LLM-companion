//! Test Fixtures for Graph Analysis Algorithms
//!
//! Provides reference graphs with known properties for testing:
//! - 8-node graph with SCCs and cycles (for Tarjan, K-Core, Entropy, CK, Leiden)
//! - 5-node chain graph (for PageRank)

use super::adjacency_list_graph_representation::AdjacencyListGraphRepresentation;

/// Create the 8-node reference graph with known properties
///
/// Graph structure:
/// ```text
/// A → B (Calls), A → C (Calls)
/// B → D (Calls), C → D (Calls)
/// D → E (Calls), E → F (Calls), F → D (Calls)  // D-E-F cycle
/// G → H (Calls), H → G (Calls)                  // G-H cycle
/// ```
///
/// Properties:
/// - 8 nodes: A, B, C, D, E, F, G, H
/// - 9 edges
/// - SCCs: {D,E,F}, {G,H}, {A}, {B}, {C}
/// - D has in-degree=3 (B, C, F), out-degree=1 (E)
/// - A has in-degree=0, out-degree=2 (B, C)
pub fn create_eight_node_reference_graph() -> AdjacencyListGraphRepresentation {
    let edges = vec![
        ("A".to_string(), "B".to_string(), "Calls".to_string()),
        ("A".to_string(), "C".to_string(), "Calls".to_string()),
        ("B".to_string(), "D".to_string(), "Calls".to_string()),
        ("C".to_string(), "D".to_string(), "Calls".to_string()),
        ("D".to_string(), "E".to_string(), "Calls".to_string()),
        ("E".to_string(), "F".to_string(), "Calls".to_string()),
        ("F".to_string(), "D".to_string(), "Calls".to_string()),
        ("G".to_string(), "H".to_string(), "Calls".to_string()),
        ("H".to_string(), "G".to_string(), "Calls".to_string()),
    ];

    AdjacencyListGraphRepresentation::build_from_dependency_edges(&edges)
}

/// Create the 5-node chain graph for PageRank testing
///
/// Graph structure:
/// ```text
/// A → B (Calls)
/// B → C (Calls)
/// C → D (Calls)
/// D → E (Calls)
/// ```
///
/// Properties:
/// - 5 nodes: A, B, C, D, E
/// - 4 edges (linear chain)
/// - No cycles
/// - All in/out degrees are 0 or 1
pub fn create_five_node_chain_graph() -> AdjacencyListGraphRepresentation {
    let edges = vec![
        ("A".to_string(), "B".to_string(), "Calls".to_string()),
        ("B".to_string(), "C".to_string(), "Calls".to_string()),
        ("C".to_string(), "D".to_string(), "Calls".to_string()),
        ("D".to_string(), "E".to_string(), "Calls".to_string()),
    ];

    AdjacencyListGraphRepresentation::build_from_dependency_edges(&edges)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_eight_node_fixture() {
        let graph = create_eight_node_reference_graph();
        assert_eq!(graph.count_total_graph_nodes(), 8);
        assert_eq!(graph.count_total_graph_edges(), 9);
    }

    #[test]
    fn test_create_five_node_fixture() {
        let graph = create_five_node_chain_graph();
        assert_eq!(graph.count_total_graph_nodes(), 5);
        assert_eq!(graph.count_total_graph_edges(), 4);
    }

    #[test]
    fn test_eight_node_graph_properties() {
        let graph = create_eight_node_reference_graph();

        // Test specific node degrees
        assert_eq!(graph.calculate_node_out_degree("A"), 2);
        assert_eq!(graph.calculate_node_in_degree("A"), 0);

        assert_eq!(graph.calculate_node_out_degree("D"), 1);
        assert_eq!(graph.calculate_node_in_degree("D"), 3); // B, C, F

        assert_eq!(graph.calculate_node_out_degree("G"), 1);
        assert_eq!(graph.calculate_node_in_degree("G"), 1); // H
    }

    #[test]
    fn test_five_node_chain_linearity() {
        let graph = create_five_node_chain_graph();

        // A is source
        assert_eq!(graph.calculate_node_in_degree("A"), 0);
        assert_eq!(graph.calculate_node_out_degree("A"), 1);

        // E is sink
        assert_eq!(graph.calculate_node_in_degree("E"), 1);
        assert_eq!(graph.calculate_node_out_degree("E"), 0);

        // Middle nodes have degree 1 in and 1 out
        assert_eq!(graph.calculate_node_in_degree("C"), 1);
        assert_eq!(graph.calculate_node_out_degree("C"), 1);
    }
}

//! Adjacency List Graph Representation
//!
//! Shared graph structure for all 7 graph analysis algorithms in Parseltongue v1.6.0.
//! Uses adjacency lists for O(1) neighbor access and supports bidirectional traversal.

use std::collections::{HashMap, HashSet};

/// Shared graph representation for all 7 graph analysis algorithms.
/// Uses adjacency lists for O(1) neighbor access.
///
/// # Design
/// - Forward edges: node → list of nodes it calls/uses/implements
/// - Reverse edges: node → list of nodes that call/use/implement it
/// - Edge types: (from, to) → EdgeType string (e.g., "Calls", "Uses", "Implements")
/// - Generic over String keys (not EntityKey) for reusability
#[derive(Clone, Debug)]
pub struct AdjacencyListGraphRepresentation {
    /// Forward edges: node → list of nodes it calls/uses/implements
    forward: HashMap<String, Vec<String>>,
    /// Reverse edges: node → list of nodes that call/use/implement it
    reverse: HashMap<String, Vec<String>>,
    /// Edge types: (from, to) → EdgeType string
    edge_types: HashMap<(String, String), String>,
    /// All unique nodes in the graph
    nodes: HashSet<String>,
    /// Total edge count
    edge_count: usize,
}

impl AdjacencyListGraphRepresentation {
    /// Create empty graph representation
    pub fn create_empty_graph_representation() -> Self {
        Self {
            forward: HashMap::new(),
            reverse: HashMap::new(),
            edge_types: HashMap::new(),
            nodes: HashSet::new(),
            edge_count: 0,
        }
    }

    /// Add a node to the graph
    ///
    /// If the node already exists, this is a no-op.
    pub fn insert_node_into_graph(&mut self, node: String) {
        self.nodes.insert(node);
    }

    /// Add an edge with type
    ///
    /// Automatically adds both nodes if they don't exist.
    /// Updates edge count and maintains both forward and reverse adjacency lists.
    pub fn insert_edge_with_type(&mut self, from: String, to: String, edge_type: String) {
        // Insert nodes
        self.nodes.insert(from.clone());
        self.nodes.insert(to.clone());

        // Add forward edge
        self.forward
            .entry(from.clone())
            .or_default()
            .push(to.clone());

        // Add reverse edge
        self.reverse
            .entry(to.clone())
            .or_default()
            .push(from.clone());

        // Store edge type
        self.edge_types.insert((from, to), edge_type);

        // Increment edge count
        self.edge_count += 1;
    }

    /// Get forward neighbors (who does this call?)
    ///
    /// Returns an empty slice if the node doesn't exist or has no outgoing edges.
    pub fn get_forward_neighbors_list(&self, node: &str) -> &[String] {
        self.forward.get(node).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get reverse neighbors (who calls this?)
    ///
    /// Returns an empty slice if the node doesn't exist or has no incoming edges.
    pub fn get_reverse_neighbors_list(&self, node: &str) -> &[String] {
        self.reverse.get(node).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get out-degree (number of outgoing edges)
    ///
    /// Returns 0 if the node doesn't exist.
    pub fn calculate_node_out_degree(&self, node: &str) -> usize {
        self.forward.get(node).map(|v| v.len()).unwrap_or(0)
    }

    /// Get in-degree (number of incoming edges)
    ///
    /// Returns 0 if the node doesn't exist.
    pub fn calculate_node_in_degree(&self, node: &str) -> usize {
        self.reverse.get(node).map(|v| v.len()).unwrap_or(0)
    }

    /// Get all nodes
    pub fn retrieve_all_graph_nodes(&self) -> &HashSet<String> {
        &self.nodes
    }

    /// Get total node count
    pub fn count_total_graph_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Get total edge count
    pub fn count_total_graph_edges(&self) -> usize {
        self.edge_count
    }

    /// Get edge type between two nodes
    ///
    /// Returns None if the edge doesn't exist.
    pub fn lookup_edge_type_between(&self, from: &str, to: &str) -> Option<&String> {
        self.edge_types.get(&(from.to_string(), to.to_string()))
    }

    /// Build from database dependency edges (the key integration point)
    ///
    /// This will be called by HTTP handlers after fetching edges from CozoDB.
    /// Edges format: (from_entity_key, to_entity_key, edge_type)
    pub fn build_from_dependency_edges(edges: &[(String, String, String)]) -> Self {
        let mut graph = Self::create_empty_graph_representation();

        for (from, to, edge_type) in edges {
            graph.insert_edge_with_type(from.clone(), to.clone(), edge_type.clone());
        }

        graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_graph_representation() {
        let graph = AdjacencyListGraphRepresentation::create_empty_graph_representation();
        assert_eq!(graph.count_total_graph_nodes(), 0);
        assert_eq!(graph.count_total_graph_edges(), 0);
    }

    #[test]
    fn test_insert_node_into_graph() {
        let mut graph = AdjacencyListGraphRepresentation::create_empty_graph_representation();
        graph.insert_node_into_graph("A".to_string());
        graph.insert_node_into_graph("B".to_string());
        graph.insert_node_into_graph("A".to_string()); // Duplicate should be no-op

        assert_eq!(graph.count_total_graph_nodes(), 2);
        assert!(graph.retrieve_all_graph_nodes().contains("A"));
        assert!(graph.retrieve_all_graph_nodes().contains("B"));
    }

    #[test]
    fn test_insert_edge_with_type() {
        let mut graph = AdjacencyListGraphRepresentation::create_empty_graph_representation();
        graph.insert_edge_with_type("A".to_string(), "B".to_string(), "Calls".to_string());

        assert_eq!(graph.count_total_graph_nodes(), 2);
        assert_eq!(graph.count_total_graph_edges(), 1);
        assert_eq!(
            graph.lookup_edge_type_between("A", "B"),
            Some(&"Calls".to_string())
        );
    }

    #[test]
    fn test_forward_neighbors_correctness_check() {
        let mut graph = AdjacencyListGraphRepresentation::create_empty_graph_representation();
        graph.insert_edge_with_type("A".to_string(), "B".to_string(), "Calls".to_string());
        graph.insert_edge_with_type("A".to_string(), "C".to_string(), "Calls".to_string());

        let neighbors = graph.get_forward_neighbors_list("A");
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&"B".to_string()));
        assert!(neighbors.contains(&"C".to_string()));
    }

    #[test]
    fn test_reverse_neighbors_correctness_check() {
        let mut graph = AdjacencyListGraphRepresentation::create_empty_graph_representation();
        graph.insert_edge_with_type("A".to_string(), "C".to_string(), "Calls".to_string());
        graph.insert_edge_with_type("B".to_string(), "C".to_string(), "Calls".to_string());

        let callers = graph.get_reverse_neighbors_list("C");
        assert_eq!(callers.len(), 2);
        assert!(callers.contains(&"A".to_string()));
        assert!(callers.contains(&"B".to_string()));
    }

    #[test]
    fn test_degree_calculations_exact_values() {
        let mut graph = AdjacencyListGraphRepresentation::create_empty_graph_representation();
        graph.insert_edge_with_type("A".to_string(), "B".to_string(), "Calls".to_string());
        graph.insert_edge_with_type("A".to_string(), "C".to_string(), "Calls".to_string());
        graph.insert_edge_with_type("B".to_string(), "D".to_string(), "Calls".to_string());
        graph.insert_edge_with_type("C".to_string(), "D".to_string(), "Calls".to_string());

        assert_eq!(graph.calculate_node_out_degree("A"), 2);
        assert_eq!(graph.calculate_node_in_degree("A"), 0);
        assert_eq!(graph.calculate_node_out_degree("D"), 0);
        assert_eq!(graph.calculate_node_in_degree("D"), 2);
    }

    #[test]
    fn test_build_from_dependency_edges() {
        let edges = vec![
            ("X".to_string(), "Y".to_string(), "Calls".to_string()),
            ("Y".to_string(), "Z".to_string(), "Uses".to_string()),
        ];
        let graph = AdjacencyListGraphRepresentation::build_from_dependency_edges(&edges);

        assert_eq!(graph.count_total_graph_nodes(), 3);
        assert_eq!(graph.count_total_graph_edges(), 2);
        assert_eq!(
            graph.lookup_edge_type_between("X", "Y"),
            Some(&"Calls".to_string())
        );
        assert_eq!(
            graph.lookup_edge_type_between("Y", "Z"),
            Some(&"Uses".to_string())
        );
    }

    #[test]
    fn test_neighbors_for_nonexistent_node() {
        let graph = AdjacencyListGraphRepresentation::create_empty_graph_representation();

        let empty: &[String] = &[];
        assert_eq!(graph.get_forward_neighbors_list("NONEXISTENT"), empty);
        assert_eq!(graph.get_reverse_neighbors_list("NONEXISTENT"), empty);
        assert_eq!(graph.calculate_node_out_degree("NONEXISTENT"), 0);
        assert_eq!(graph.calculate_node_in_degree("NONEXISTENT"), 0);
    }

    #[test]
    fn test_lookup_edge_type_nonexistent() {
        let graph = AdjacencyListGraphRepresentation::create_empty_graph_representation();
        assert_eq!(graph.lookup_edge_type_between("A", "B"), None);
    }

    #[test]
    fn test_clone_and_debug_traits() {
        let edges = vec![("A".to_string(), "B".to_string(), "Calls".to_string())];
        let graph = AdjacencyListGraphRepresentation::build_from_dependency_edges(&edges);

        // Test Clone
        let cloned = graph.clone();
        assert_eq!(cloned.count_total_graph_nodes(), 2);
        assert_eq!(cloned.count_total_graph_edges(), 1);

        // Test Debug (just ensure it doesn't panic)
        let debug_str = format!("{:?}", graph);
        assert!(debug_str.contains("AdjacencyListGraphRepresentation"));
    }
}

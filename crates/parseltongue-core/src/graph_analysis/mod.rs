//! Graph Analysis Module
//!
//! Shared infrastructure for all 7 graph analysis algorithms in Parseltongue v1.6.0:
//! - Tarjan SCC (Strongly Connected Components)
//! - K-Core Decomposition
//! - PageRank + Betweenness Centrality
//! - Shannon Entropy Complexity
//! - CK Metrics Suite
//! - SQALE Technical Debt Scoring
//! - Leiden Community Detection
//!
//! # Phase 0: Shared Infrastructure
//! This module provides:
//! - `AdjacencyListGraphRepresentation`: Bidirectional graph with O(1) neighbor access
//! - Test fixtures: 8-node graph (with cycles/SCCs) and 5-node chain (linear)

pub mod adjacency_list_graph_representation;
pub mod test_fixture_reference_graphs;

// Re-export for convenience
pub use adjacency_list_graph_representation::AdjacencyListGraphRepresentation;
pub use test_fixture_reference_graphs::{
    create_eight_node_reference_graph, create_five_node_chain_graph,
};

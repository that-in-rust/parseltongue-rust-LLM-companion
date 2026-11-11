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
pub mod kcore_decomposition_algorithm;
pub mod tarjan_scc_algorithm;
pub mod entropy_complexity_algorithm;
pub mod centrality_measures_algorithm;
pub mod leiden_community_algorithm;
pub mod ck_metrics_suite_algorithm;
pub mod sqale_debt_scoring_algorithm;
pub mod integration_cross_algorithm_tests;

// Re-export for convenience
pub use adjacency_list_graph_representation::AdjacencyListGraphRepresentation;
pub use test_fixture_reference_graphs::{
    create_eight_node_reference_graph, create_five_node_chain_graph,
};
pub use kcore_decomposition_algorithm::{
    kcore_decomposition_layering_algorithm, classify_coreness_layer_level, CoreLayer,
};
pub use tarjan_scc_algorithm::{
    tarjan_strongly_connected_components, classify_scc_risk_level, SccRiskLevel,
};
pub use entropy_complexity_algorithm::{
    calculate_entity_entropy_score, compute_all_entity_entropy,
    classify_entropy_complexity_level, EntropyComplexity,
};
pub use centrality_measures_algorithm::{
    compute_pagerank_centrality_scores, compute_betweenness_centrality_scores,
};
pub use leiden_community_algorithm::{
    leiden_community_detection_clustering, calculate_modularity_score_value,
};
pub use ck_metrics_suite_algorithm::{
    calculate_coupling_between_objects, calculate_lack_cohesion_methods,
    calculate_response_for_class, calculate_weighted_methods_class,
    compute_ck_metrics_suite, grade_ck_metrics_health,
    CkMetricsResult, HealthGrade, MetricStatus, evaluate_single_metric_status,
};
pub use sqale_debt_scoring_algorithm::{
    calculate_technical_debt_sqale, compute_all_entities_sqale,
    classify_debt_severity_level, SqaleViolationType, SqaleViolationRecord,
    SqaleDebtResult, DebtSeverity,
};

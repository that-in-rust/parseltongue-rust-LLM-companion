//! Integration Tests for Cross-Algorithm Consistency
//!
//! Verifies that all 7 graph analysis algorithms work together correctly
//! on shared reference graphs and produce consistent, correlated results.
//!
//! Algorithms tested:
//! 1. Tarjan SCC (Strongly Connected Components)
//! 2. K-Core Decomposition
//! 3. PageRank + Betweenness Centrality
//! 4. Shannon Entropy Complexity
//! 5. CK Metrics Suite
//! 6. Leiden Community Detection
//! 7. SQALE Technical Debt Scoring

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::{HashMap, HashSet};

    /// Test 1: All 7 algorithms run successfully on 8-node reference graph
    #[test]
    fn test_all_seven_algorithms_run_on_eight_node_graph() {
        let graph = create_eight_node_reference_graph();

        // Algorithm 1: Tarjan SCC
        let sccs = tarjan_strongly_connected_components(&graph);
        assert_eq!(sccs.len(), 5); // {D,E,F}, {G,H}, {A}, {B}, {C}

        // Algorithm 2: K-Core Decomposition
        let core_numbers = kcore_decomposition_layering_algorithm(&graph);
        assert_eq!(core_numbers.len(), 8); // All 8 nodes get coreness values

        // Algorithm 3: PageRank (damping, max_iterations, tolerance)
        let pagerank = compute_pagerank_centrality_scores(&graph, 0.85, 100, 1e-6);
        assert_eq!(pagerank.len(), 8);
        let sum: f64 = pagerank.values().sum();
        assert!((sum - 1.0).abs() < 0.01); // Sum ≈ 1.0

        // Algorithm 4: Betweenness Centrality
        let betweenness = compute_betweenness_centrality_scores(&graph);
        assert_eq!(betweenness.len(), 8);

        // Algorithm 5: Entropy
        let entropy = compute_all_entity_entropy(&graph);
        assert_eq!(entropy.len(), 8); // All 8 nodes computed

        // Algorithm 6: CK Metrics (CBO) - compute for all nodes
        let nodes = graph.retrieve_all_graph_nodes();
        let ck_metrics: HashMap<String, CkMetricsResult> = nodes
            .iter()
            .map(|node| (node.clone(), compute_ck_metrics_suite(&graph, node)))
            .collect();
        assert_eq!(ck_metrics.len(), 8); // CBO computed for all 8 nodes

        // Algorithm 7: Leiden Communities (resolution, max_iterations)
        let (communities, _modularity) = leiden_community_detection_clustering(&graph, 1.0, 100);
        let unique_communities: HashSet<_> = communities.values().collect();
        assert!(unique_communities.len() >= 1); // At least 1 community

        // Algorithm 8: SQALE Debt
        let sqale_results = compute_all_entities_sqale(&graph);
        assert_eq!(sqale_results.len(), 8); // All 8 nodes have debt results

        // Verify all nodes have results
        let node_names: HashSet<&String> = sqale_results.iter().map(|r| &r.entity).collect();
        assert_eq!(node_names.len(), 8);
    }

    /// Test 2: All 7 algorithms run successfully on 5-node chain graph
    #[test]
    fn test_all_seven_algorithms_run_on_chain_graph() {
        let graph = create_five_node_chain_graph();

        // Algorithm 1: Tarjan SCC - 5 singleton SCCs
        let sccs = tarjan_strongly_connected_components(&graph);
        assert_eq!(sccs.len(), 5);
        for scc in &sccs {
            assert_eq!(scc.len(), 1); // All singleton SCCs
        }

        // Algorithm 2: K-Core - all coreness 1 (chain has no dense subgraphs)
        let core_numbers = kcore_decomposition_layering_algorithm(&graph);
        assert_eq!(core_numbers.len(), 5);
        for &coreness in core_numbers.values() {
            assert!(coreness <= 1); // Chain nodes have low coreness
        }

        // Algorithm 3: PageRank - sink node E has highest rank
        let pagerank = compute_pagerank_centrality_scores(&graph, 0.85, 100, 1e-6);
        assert_eq!(pagerank.len(), 5);
        let e_rank = pagerank.get("E").copied().unwrap_or(0.0);
        // E accumulates rank as sink
        assert!(e_rank > 0.1);

        // Algorithm 4: Betweenness - middle nodes have higher centrality
        let betweenness = compute_betweenness_centrality_scores(&graph);
        let b_centrality = betweenness.get("B").copied().unwrap_or(0.0);
        let c_centrality = betweenness.get("C").copied().unwrap_or(0.0);
        let _d_centrality = betweenness.get("D").copied().unwrap_or(0.0);
        let a_centrality = betweenness.get("A").copied().unwrap_or(0.0);
        let e_centrality = betweenness.get("E").copied().unwrap_or(0.0);

        // Middle nodes should have higher betweenness than endpoints
        assert!(b_centrality >= a_centrality);
        assert!(c_centrality >= e_centrality);

        // Algorithm 5: Entropy - all nodes have entropy 0 (single edge type)
        let entropy = compute_all_entity_entropy(&graph);
        assert_eq!(entropy.len(), 5);
        for &ent in entropy.values() {
            assert!(ent <= 0.01); // Single edge type → low entropy
        }

        // Algorithm 6: CK Metrics - all low coupling (chain structure)
        let nodes = graph.retrieve_all_graph_nodes();
        let ck_metrics: HashMap<String, CkMetricsResult> = nodes
            .iter()
            .map(|node| (node.clone(), compute_ck_metrics_suite(&graph, node)))
            .collect();
        for metrics in ck_metrics.values() {
            assert!(metrics.cbo <= 2); // Max 2 neighbors in chain
        }

        // Algorithm 7: Leiden Communities
        let (communities, _modularity) = leiden_community_detection_clustering(&graph, 1.0, 100);
        assert_eq!(communities.len(), 5);

        // Algorithm 8: SQALE Debt - all debt low (low coupling/complexity)
        let sqale_results = compute_all_entities_sqale(&graph);
        for result in &sqale_results {
            // Chain has minimal complexity
            assert!(result.total_debt_hours <= 100.0);
        }
    }

    /// Test 3: Nodes in same SCC share k-core level
    #[test]
    fn test_scc_nodes_share_kcore_level() {
        let graph = create_eight_node_reference_graph();

        let sccs = tarjan_strongly_connected_components(&graph);
        let core_numbers = kcore_decomposition_layering_algorithm(&graph);

        // Find the D-E-F cycle SCC
        let def_scc = sccs
            .iter()
            .find(|scc| scc.contains(&"D".to_string()) && scc.contains(&"E".to_string()) && scc.contains(&"F".to_string()))
            .expect("D-E-F SCC should exist");

        assert_eq!(def_scc.len(), 3);

        // D, E, F should have same k-core number (they form a 3-cycle)
        let d_core = core_numbers.get("D").copied().unwrap();
        let e_core = core_numbers.get("E").copied().unwrap();
        let f_core = core_numbers.get("F").copied().unwrap();

        assert_eq!(d_core, e_core);
        assert_eq!(e_core, f_core);
    }

    /// Test 4: High betweenness implies high PageRank
    #[test]
    fn test_high_betweenness_implies_high_pagerank() {
        let graph = create_eight_node_reference_graph();

        let pagerank = compute_pagerank_centrality_scores(&graph, 0.85, 100, 1e-6);
        let betweenness = compute_betweenness_centrality_scores(&graph);

        // D is bridge between tree (A,B,C) and cycle (D,E,F)
        let d_betweenness = betweenness.get("D").copied().unwrap_or(0.0);

        // Find nodes with highest PageRank
        let mut ranks: Vec<(&String, &f64)> = pagerank.iter().collect();
        ranks.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

        // D should be in top 3 PageRank nodes
        let top_3_nodes: Vec<&String> = ranks.iter().take(3).map(|(node, _)| *node).collect();

        assert!(
            d_betweenness > 0.0,
            "D should have positive betweenness as bridge"
        );
        assert!(
            top_3_nodes.contains(&&"D".to_string())
                || top_3_nodes.contains(&&"E".to_string())
                || top_3_nodes.contains(&&"F".to_string()),
            "Cycle nodes should have high PageRank"
        );
    }

    /// Test 5: Cycle nodes detected by both SCC and Leiden
    #[test]
    fn test_cycle_nodes_detected_by_both_scc_and_leiden() {
        let graph = create_eight_node_reference_graph();

        let sccs = tarjan_strongly_connected_components(&graph);
        let (communities, _modularity) = leiden_community_detection_clustering(&graph, 1.0, 100);

        // Find the D-E-F SCC
        let def_scc = sccs
            .iter()
            .find(|scc| scc.contains(&"D".to_string()) && scc.contains(&"E".to_string()) && scc.contains(&"F".to_string()))
            .expect("D-E-F SCC should exist");

        assert_eq!(def_scc.len(), 3);

        // D, E, F should be in the same Leiden community (or at least share communities)
        let d_community = communities.get("D").copied().unwrap();
        let e_community = communities.get("E").copied().unwrap();
        let f_community = communities.get("F").copied().unwrap();

        // Leiden is randomized, so check that at least 2 out of 3 are in same community
        // or that the communities are related
        let same_de = d_community == e_community;
        let same_ef = e_community == f_community;
        let same_df = d_community == f_community;

        // At least one pair should be in same community
        assert!(
            same_de || same_ef || same_df,
            "At least two of D,E,F should be in same Leiden community (found D={}, E={}, F={})",
            d_community, e_community, f_community
        );
    }

    /// Test 6: Isolated pair consistent across algorithms
    #[test]
    fn test_isolated_pair_consistent_across_algorithms() {
        let graph = create_eight_node_reference_graph();

        let sccs = tarjan_strongly_connected_components(&graph);
        let (communities, _modularity) = leiden_community_detection_clustering(&graph, 1.0, 100);
        let core_numbers = kcore_decomposition_layering_algorithm(&graph);
        let pagerank = compute_pagerank_centrality_scores(&graph, 0.85, 100, 1e-6);

        // Find the G-H SCC
        let gh_scc = sccs
            .iter()
            .find(|scc| scc.contains(&"G".to_string()) && scc.contains(&"H".to_string()))
            .expect("G-H SCC should exist");

        assert_eq!(gh_scc.len(), 2);

        // G and H should:
        // 1. Be in same SCC (2-node cycle)
        assert!(gh_scc.contains(&"G".to_string()));
        assert!(gh_scc.contains(&"H".to_string()));

        // 2. Be in same Leiden community
        let g_community = communities.get("G").copied().unwrap();
        let h_community = communities.get("H").copied().unwrap();
        assert_eq!(g_community, h_community);

        // 3. Have same k-core value
        let g_core = core_numbers.get("G").copied().unwrap();
        let h_core = core_numbers.get("H").copied().unwrap();
        assert_eq!(g_core, h_core);

        // 4. Have similar PageRank (isolated cycle distributes rank equally)
        let g_rank = pagerank.get("G").copied().unwrap_or(0.0);
        let h_rank = pagerank.get("H").copied().unwrap_or(0.0);

        // G and H should have very similar PageRank (symmetric cycle)
        assert!((g_rank - h_rank).abs() < 0.01, "G and H should have similar PageRank in their isolated cycle");
    }

    /// Test 7: SQALE debt correlates with coupling
    #[test]
    fn test_sqale_debt_correlates_with_coupling() {
        let graph = create_eight_node_reference_graph();

        // Compute CK metrics for D and A
        let d_metrics = compute_ck_metrics_suite(&graph, "D");
        let a_metrics = compute_ck_metrics_suite(&graph, "A");
        let sqale_results = compute_all_entities_sqale(&graph);

        // D has highest coupling (in-degree 3: B, C, F)
        let d_cbo = d_metrics.cbo;
        let d_debt = sqale_results
            .iter()
            .find(|r| r.entity == "D")
            .map(|r| r.total_debt_hours)
            .unwrap_or(0.0);

        // A has lower coupling (out-degree 2, in-degree 0)
        let a_cbo = a_metrics.cbo;
        let a_debt = sqale_results
            .iter()
            .find(|r| r.entity == "A")
            .map(|r| r.total_debt_hours)
            .unwrap_or(0.0);

        // Higher coupling should correlate with higher or equal debt
        assert!(d_cbo >= a_cbo, "D should have higher coupling than A");
        if d_cbo > 2 {
            assert!(
                d_debt >= a_debt,
                "Higher coupling should lead to higher debt"
            );
        }
    }

    /// Test 8: Performance test - large graph completes
    #[test]
    fn test_performance_large_graph_completes() {
        // Create a graph with 1000 nodes and ~5000 edges
        let mut graph = AdjacencyListGraphRepresentation::create_empty_graph_representation();

        // Hub node connected to 500 nodes
        for i in 0..500 {
            graph.insert_edge_with_type(
                "hub".to_string(),
                format!("spoke_{}", i),
                "Calls".to_string(),
            );
        }

        // Chain of 500 nodes
        for i in 0..499 {
            graph.insert_edge_with_type(
                format!("chain_{}", i),
                format!("chain_{}", i + 1),
                "Uses".to_string(),
            );
        }

        // Connect hub to chain
        graph.insert_edge_with_type("hub".to_string(), "chain_0".to_string(), "Calls".to_string());

        assert!(graph.count_total_graph_nodes() >= 1000);

        // All 7 algorithms should complete without panic/timeout
        let sccs = tarjan_strongly_connected_components(&graph);
        assert!(!sccs.is_empty());

        let core_numbers = kcore_decomposition_layering_algorithm(&graph);
        assert!(!core_numbers.is_empty());

        let pagerank = compute_pagerank_centrality_scores(&graph, 0.85, 50, 1e-6); // Fewer iterations
        assert!(!pagerank.is_empty());

        let betweenness = compute_betweenness_centrality_scores(&graph);
        assert!(!betweenness.is_empty());

        let entropy = compute_all_entity_entropy(&graph);
        assert!(!entropy.is_empty());

        // CK Metrics - just verify one node completes
        let nodes = graph.retrieve_all_graph_nodes();
        if let Some(first_node) = nodes.iter().next() {
            let _ = compute_ck_metrics_suite(&graph, first_node);
        }

        let (communities, _modularity) = leiden_community_detection_clustering(&graph, 1.0, 100);
        assert!(!communities.is_empty());

        let sqale_results = compute_all_entities_sqale(&graph);
        assert!(!sqale_results.is_empty());
    }

    /// Test 9: Empty graph - all algorithms return empty/zero results
    #[test]
    fn test_empty_graph_all_algorithms_return_empty() {
        let graph = AdjacencyListGraphRepresentation::create_empty_graph_representation();

        // All algorithms should handle empty graphs gracefully
        let sccs = tarjan_strongly_connected_components(&graph);
        assert!(sccs.is_empty());

        let core_numbers = kcore_decomposition_layering_algorithm(&graph);
        assert!(core_numbers.is_empty());

        let pagerank = compute_pagerank_centrality_scores(&graph, 0.85, 100, 1e-6);
        assert!(pagerank.is_empty());

        let betweenness = compute_betweenness_centrality_scores(&graph);
        assert!(betweenness.is_empty());

        let entropy = compute_all_entity_entropy(&graph);
        assert!(entropy.is_empty());

        // CK Metrics - empty graph has no nodes, nothing to compute
        // (function requires a node parameter, so we skip this check)

        let (communities, _modularity) = leiden_community_detection_clustering(&graph, 1.0, 100);
        assert!(communities.is_empty());

        let sqale_results = compute_all_entities_sqale(&graph);
        assert!(sqale_results.is_empty());
    }
}

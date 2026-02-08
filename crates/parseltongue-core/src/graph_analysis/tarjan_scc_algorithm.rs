use std::collections::{HashMap, HashSet};
use crate::graph_analysis::AdjacencyListGraphRepresentation;

/// Risk level classification for SCC size
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SccRiskLevel {
    None,    // Single node (no cycle)
    Medium,  // 2-node cycle
    High,    // 3+ node cycle
}

/// Internal state for Tarjan's algorithm
struct TarjanState {
    index_counter: usize,
    stack: Vec<String>,
    on_stack: HashSet<String>,
    indices: HashMap<String, usize>,
    lowlinks: HashMap<String, usize>,
    sccs: Vec<Vec<String>>,
}

impl TarjanState {
    fn new() -> Self {
        Self {
            index_counter: 0,
            stack: Vec::new(),
            on_stack: HashSet::new(),
            indices: HashMap::new(),
            lowlinks: HashMap::new(),
            sccs: Vec::new(),
        }
    }
}

/// Tarjan's SCC Algorithm - O(V+E) single DFS pass
///
/// Reference: Tarjan, R. (1972). "Depth-first search and linear graph algorithms"
///
/// # 4-Word Name: tarjan_strongly_connected_components
pub fn tarjan_strongly_connected_components(
    graph: &AdjacencyListGraphRepresentation,
) -> Vec<Vec<String>> {
    let mut state = TarjanState::new();

    for node in graph.retrieve_all_graph_nodes() {
        if !state.indices.contains_key(node) {
            strongconnect(node, graph, &mut state);
        }
    }

    // Sort SCCs by size descending for consistent output
    state.sccs.sort_by_key(|b| std::cmp::Reverse(b.len()));
    state.sccs
}

fn strongconnect(
    v: &str,
    graph: &AdjacencyListGraphRepresentation,
    state: &mut TarjanState,
) {
    let v_index = state.index_counter;
    state.indices.insert(v.to_string(), v_index);
    state.lowlinks.insert(v.to_string(), v_index);
    state.index_counter += 1;
    state.stack.push(v.to_string());
    state.on_stack.insert(v.to_string());

    // Consider successors of v
    for w in graph.get_forward_neighbors_list(v) {
        if !state.indices.contains_key(w.as_str()) {
            // Successor w not yet visited; recurse
            strongconnect(w, graph, state);
            let new_lowlink = std::cmp::min(state.lowlinks[v], state.lowlinks[w.as_str()]);
            state.lowlinks.insert(v.to_string(), new_lowlink);
        } else if state.on_stack.contains(w.as_str()) {
            // Successor w is on stack → in current SCC
            let new_lowlink = std::cmp::min(state.lowlinks[v], state.indices[w.as_str()]);
            state.lowlinks.insert(v.to_string(), new_lowlink);
        }
    }

    // If v is a root node, pop the stack to form an SCC
    if state.lowlinks[v] == state.indices[v] {
        let mut scc = Vec::new();
        loop {
            let w = state.stack.pop().expect("Stack should not be empty");
            state.on_stack.remove(&w);
            scc.push(w.clone());
            if w == v {
                break;
            }
        }
        state.sccs.push(scc);
    }
}

/// Classify SCC risk based on size
///
/// # 4-Word Name: classify_scc_risk_level
pub fn classify_scc_risk_level(scc_size: usize) -> SccRiskLevel {
    match scc_size {
        0 | 1 => SccRiskLevel::None,
        2 => SccRiskLevel::Medium,
        _ => SccRiskLevel::High,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_analysis::test_fixture_reference_graphs::{
        create_eight_node_reference_graph,
        create_five_node_chain_graph,
    };

    #[test]
    fn test_tarjan_scc_count_exact() {
        let graph = create_eight_node_reference_graph();
        let sccs = tarjan_strongly_connected_components(&graph);
        assert_eq!(sccs.len(), 5, "Expected exactly 5 SCCs");
    }

    #[test]
    fn test_tarjan_scc_three_node_cycle() {
        let graph = create_eight_node_reference_graph();
        let sccs = tarjan_strongly_connected_components(&graph);

        // Find the 3-member SCC
        let large_scc = sccs.iter().find(|scc| scc.len() == 3).expect("Expected a 3-node SCC");
        let mut members: Vec<&str> = large_scc.iter().map(|s| s.as_str()).collect();
        members.sort();
        assert_eq!(members, vec!["D", "E", "F"]);
    }

    #[test]
    fn test_tarjan_scc_two_node_cycle() {
        let graph = create_eight_node_reference_graph();
        let sccs = tarjan_strongly_connected_components(&graph);

        let two_scc = sccs.iter().find(|scc| scc.len() == 2).expect("Expected a 2-node SCC");
        let mut members: Vec<&str> = two_scc.iter().map(|s| s.as_str()).collect();
        members.sort();
        assert_eq!(members, vec!["G", "H"]);
    }

    #[test]
    fn test_tarjan_scc_singleton_nodes() {
        let graph = create_eight_node_reference_graph();
        let sccs = tarjan_strongly_connected_components(&graph);

        let singletons: Vec<&Vec<String>> = sccs.iter().filter(|scc| scc.len() == 1).collect();
        assert_eq!(singletons.len(), 3, "Expected 3 singleton SCCs (A, B, C)");

        let mut singleton_names: Vec<&str> = singletons.iter()
            .map(|scc| scc[0].as_str())
            .collect();
        singleton_names.sort();
        assert_eq!(singleton_names, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_classify_scc_risk_level() {
        assert_eq!(classify_scc_risk_level(1), SccRiskLevel::None);
        assert_eq!(classify_scc_risk_level(2), SccRiskLevel::Medium);
        assert_eq!(classify_scc_risk_level(3), SccRiskLevel::High);
        assert_eq!(classify_scc_risk_level(10), SccRiskLevel::High);
    }

    #[test]
    fn test_tarjan_empty_graph() {
        let graph = AdjacencyListGraphRepresentation::create_empty_graph_representation();
        let sccs = tarjan_strongly_connected_components(&graph);
        assert_eq!(sccs.len(), 0);
    }

    #[test]
    fn test_tarjan_chain_all_singletons() {
        let graph = create_five_node_chain_graph();
        let sccs = tarjan_strongly_connected_components(&graph);
        assert_eq!(sccs.len(), 5, "Linear chain should have 5 singleton SCCs");
        assert!(sccs.iter().all(|scc| scc.len() == 1), "All SCCs should be singletons");
    }
}

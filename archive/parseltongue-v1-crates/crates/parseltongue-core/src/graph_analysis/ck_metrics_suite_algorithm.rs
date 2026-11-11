//! CK Metrics Suite Algorithm (Chidamber & Kemerer 1994)
//!
//! Implements 4 of 6 CK metrics (v1.6.0 ships CBO, LCOM, RFC, WMC; defers DIT, NOC to v1.7.0):
//! - **CBO** (Coupling Between Objects): Count unique entities coupled (forward + reverse)
//! - **LCOM** (Lack of Cohesion of Methods): Measure cohesion via shared-dependency analysis
//! - **RFC** (Response For a Class): Count unique methods in 1-hop transitive closure
//! - **WMC** (Weighted Methods per Class): Sum of method complexity (using out-degree as proxy)
//!
//! # Thresholds (Chidamber & Kemerer 1994)
//! - **FAIL**: CBO > 10, LCOM > 0.8
//! - **WARNING**: RFC > 50, WMC > 50
//!
//! # Grading Scale
//! - **A**: All OK
//! - **B**: 1 WARNING
//! - **C**: 2 WARNING
//! - **D**: 1 FAIL or 3+ WARNING
//! - **F**: 2+ FAIL

use std::collections::HashSet;
use crate::graph_analysis::AdjacencyListGraphRepresentation;

/// CK Metrics result for a single entity
#[derive(Debug, Clone)]
pub struct CkMetricsResult {
    pub cbo: usize,
    pub lcom: f64,
    pub rfc: usize,
    pub wmc: usize,
}

/// Health grade for CK metrics (A-F scale)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthGrade {
    A,  // All OK
    B,  // 1 WARNING
    C,  // 2 WARNING
    D,  // 1 FAIL or 3+ WARNING
    F,  // 2+ FAIL
}

/// Metric status (OK, WARNING, FAIL)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricStatus {
    Ok,
    Warning,
    Fail,
}

// CK Thresholds (Chidamber & Kemerer 1994)
const CBO_THRESHOLD: usize = 10;
const LCOM_THRESHOLD: f64 = 0.8;
const RFC_THRESHOLD: usize = 50;
const WMC_THRESHOLD: usize = 50;

/// CBO: Coupling Between Objects
/// Count unique entities coupled (forward + reverse neighbors deduplicated)
///
/// # Algorithm
/// 1. Collect all forward neighbors (entities this node calls/uses)
/// 2. Collect all reverse neighbors (entities that call/use this node)
/// 3. Union both sets and count unique entities
///
/// # 4-Word Name Convention
/// calculate_coupling_between_objects
pub fn calculate_coupling_between_objects(
    graph: &AdjacencyListGraphRepresentation,
    node: &str,
) -> usize {
    let mut coupled: HashSet<&str> = HashSet::new();

    // Add forward neighbors (who this calls)
    for n in graph.get_forward_neighbors_list(node) {
        coupled.insert(n.as_str());
    }

    // Add reverse neighbors (who calls this)
    for n in graph.get_reverse_neighbors_list(node) {
        coupled.insert(n.as_str());
    }

    coupled.len()
}

/// LCOM: Lack of Cohesion of Methods
/// Compares pairs of forward neighbors (treating them as "methods")
/// by checking if they share any common forward targets ("attributes")
///
/// # Algorithm
/// 1. Get all forward neighbors (treat as "methods")
/// 2. For each method, get its forward neighbors (treat as "attributes")
/// 3. Count pairs:
///    - P = pairs sharing NO attributes
///    - Q = pairs sharing ≥1 attribute
/// 4. LCOM = P / (P + Q)
///
/// # Special Cases
/// - 0-1 methods: return 0.0 (perfect cohesion)
/// - No pairs: return 0.0
///
/// # 4-Word Name Convention
/// calculate_lack_cohesion_methods
pub fn calculate_lack_cohesion_methods(
    graph: &AdjacencyListGraphRepresentation,
    node: &str,
) -> f64 {
    let children: Vec<&str> = graph.get_forward_neighbors_list(node)
        .iter()
        .map(|s| s.as_str())
        .collect();

    if children.len() <= 1 {
        return 0.0; // Perfect cohesion for 0-1 methods
    }

    // For each child, get its forward neighbors (what it calls/uses)
    let child_targets: Vec<HashSet<&str>> = children.iter()
        .map(|child| {
            graph.get_forward_neighbors_list(child)
                .iter()
                .map(|s| s.as_str())
                .collect()
        })
        .collect();

    let mut p = 0usize; // Pairs sharing NO targets
    let mut q = 0usize; // Pairs sharing ≥1 target

    // Compare all pairs of children
    for i in 0..children.len() {
        for j in (i + 1)..children.len() {
            let shared = child_targets[i].intersection(&child_targets[j]).count();
            if shared == 0 {
                p += 1;
            } else {
                q += 1;
            }
        }
    }

    if p + q == 0 {
        return 0.0;
    }

    p as f64 / (p + q) as f64
}

/// RFC: Response For a Class
/// Count unique methods reachable in 1 hop (direct calls + their calls)
///
/// # Algorithm
/// 1. Get all direct forward neighbors (methods this calls)
/// 2. For each direct neighbor, get its forward neighbors (1 more hop)
/// 3. Union all reachable methods and count unique
///
/// # 4-Word Name Convention
/// calculate_response_for_class
pub fn calculate_response_for_class(
    graph: &AdjacencyListGraphRepresentation,
    node: &str,
) -> usize {
    let mut response_set: HashSet<&str> = HashSet::new();

    // Direct calls
    for callee in graph.get_forward_neighbors_list(node) {
        response_set.insert(callee.as_str());

        // Calls made by callees (1 more hop)
        for transitive in graph.get_forward_neighbors_list(callee) {
            response_set.insert(transitive.as_str());
        }
    }

    response_set.len()
}

/// WMC: Weighted Methods per Class
/// Sum of complexity of methods (using out-degree as complexity proxy)
///
/// # Algorithm
/// Use out-degree as proxy for cyclomatic complexity (no AST access)
/// WMC = out-degree of node (number of calls it makes)
///
/// # 4-Word Name Convention
/// calculate_weighted_methods_class
pub fn calculate_weighted_methods_class(
    graph: &AdjacencyListGraphRepresentation,
    node: &str,
) -> usize {
    graph.calculate_node_out_degree(node)
}

/// Compute all 4 CK metrics for a node
///
/// # Returns
/// CkMetricsResult with CBO, LCOM, RFC, WMC values
///
/// # 4-Word Name Convention
/// compute_ck_metrics_suite
pub fn compute_ck_metrics_suite(
    graph: &AdjacencyListGraphRepresentation,
    node: &str,
) -> CkMetricsResult {
    CkMetricsResult {
        cbo: calculate_coupling_between_objects(graph, node),
        lcom: calculate_lack_cohesion_methods(graph, node),
        rfc: calculate_response_for_class(graph, node),
        wmc: calculate_weighted_methods_class(graph, node),
    }
}

/// Grade CK metrics health A-F
///
/// # Grading Rules
/// - **A**: All OK
/// - **B**: 1 WARNING
/// - **C**: 2 WARNING
/// - **D**: 1 FAIL or 3+ WARNING
/// - **F**: 2+ FAIL
///
/// # Thresholds
/// - **FAIL**: CBO > 10, LCOM > 0.8
/// - **WARNING**: RFC > 50, WMC > 50
///
/// # 4-Word Name Convention
/// grade_ck_metrics_health
pub fn grade_ck_metrics_health(metrics: &CkMetricsResult) -> HealthGrade {
    let mut failures = 0;
    let mut warnings = 0;

    // FAIL conditions
    if metrics.cbo > CBO_THRESHOLD {
        failures += 1;
    }
    if metrics.lcom > LCOM_THRESHOLD {
        failures += 1;
    }

    // WARNING conditions
    if metrics.rfc > RFC_THRESHOLD {
        warnings += 1;
    }
    if metrics.wmc > WMC_THRESHOLD {
        warnings += 1;
    }

    // Grade based on failures and warnings
    match (failures, warnings) {
        (0, 0) => HealthGrade::A,
        (0, 1) => HealthGrade::B,
        (0, 2) => HealthGrade::C,
        (1, _) | (0, 3..) => HealthGrade::D,
        _ => HealthGrade::F,
    }
}

/// Get metric status for a single metric
///
/// # Arguments
/// - `value`: Current metric value
/// - `threshold`: Threshold for this metric
/// - `is_critical`: True if exceeding threshold is FAIL, false if WARNING
///
/// # 4-Word Name Convention
/// evaluate_single_metric_status
pub fn evaluate_single_metric_status(
    value: f64,
    threshold: f64,
    is_critical: bool
) -> MetricStatus {
    if value <= threshold {
        MetricStatus::Ok
    } else if is_critical {
        MetricStatus::Fail
    } else {
        MetricStatus::Warning
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_analysis::AdjacencyListGraphRepresentation;
    use crate::graph_analysis::test_fixture_reference_graphs::create_eight_node_reference_graph;

    #[test]
    fn test_cbo_node_d_high_coupling() {
        let graph = create_eight_node_reference_graph();
        let cbo = calculate_coupling_between_objects(&graph, "D");
        // D: forward neighbors = {E}, reverse neighbors = {B, C, F}
        // CBO = |{E}| + |{B, C, F}| = 1 + 3 = 4
        assert_eq!(cbo, 4);
    }

    #[test]
    fn test_cbo_node_a_source() {
        let graph = create_eight_node_reference_graph();
        let cbo = calculate_coupling_between_objects(&graph, "A");
        // A: forward = {B, C}, reverse = {} → CBO = 2 + 0 = 2
        assert_eq!(cbo, 2);
    }

    #[test]
    fn test_cbo_nonexistent_node() {
        let graph = create_eight_node_reference_graph();
        let cbo = calculate_coupling_between_objects(&graph, "NONEXISTENT");
        assert_eq!(cbo, 0);
    }

    #[test]
    fn test_rfc_node_a() {
        let graph = create_eight_node_reference_graph();
        let rfc = calculate_response_for_class(&graph, "A");
        // A calls B, C. B calls D. C calls D.
        // RFC(A) = forward_neighbors(A) ∪ forward_neighbors(B) ∪ forward_neighbors(C)
        //        = {B, C} ∪ {D} ∪ {D} = {B, C, D}
        // RFC = |{B, C, D}| = 3 unique methods
        assert_eq!(rfc, 3);
    }

    #[test]
    fn test_rfc_leaf_node() {
        let graph = create_eight_node_reference_graph();
        // E calls F only. F calls D only.
        let rfc = calculate_response_for_class(&graph, "E");
        // Direct calls: {F}. F's calls: {D}.
        // RFC = |{F, D}| = 2
        assert_eq!(rfc, 2);
    }

    #[test]
    fn test_wmc_proxy_out_degree() {
        let graph = create_eight_node_reference_graph();
        let wmc = calculate_weighted_methods_class(&graph, "A");
        // WMC(A) = out_degree(A) = 2 (calls B and C)
        assert_eq!(wmc, 2);
    }

    #[test]
    fn test_wmc_node_d() {
        let graph = create_eight_node_reference_graph();
        let wmc = calculate_weighted_methods_class(&graph, "D");
        // WMC(D) = out_degree(D) = 1 (calls E only)
        assert_eq!(wmc, 1);
    }

    #[test]
    fn test_lcom_independent_branches() {
        // Build a graph where A→B, A→C, B→D, C→E (B and C share nothing)
        let edges = vec![
            ("A".to_string(), "B".to_string(), "Calls".to_string()),
            ("A".to_string(), "C".to_string(), "Calls".to_string()),
            ("B".to_string(), "D".to_string(), "Calls".to_string()),
            ("C".to_string(), "E".to_string(), "Calls".to_string()),
        ];
        let graph = AdjacencyListGraphRepresentation::build_from_dependency_edges(&edges);
        let lcom = calculate_lack_cohesion_methods(&graph, "A");
        // A's children (B, C) call different targets (D vs E)
        // P=1 (pair sharing nothing), Q=0 → LCOM = 1.0
        assert!((lcom - 1.0).abs() < 0.01, "Expected LCOM ~1.0, got {}", lcom);
    }

    #[test]
    fn test_lcom_shared_target() {
        // A→B, A→C, B→D, C→D (B and C both call D)
        let edges = vec![
            ("A".to_string(), "B".to_string(), "Calls".to_string()),
            ("A".to_string(), "C".to_string(), "Calls".to_string()),
            ("B".to_string(), "D".to_string(), "Calls".to_string()),
            ("C".to_string(), "D".to_string(), "Calls".to_string()),
        ];
        let graph = AdjacencyListGraphRepresentation::build_from_dependency_edges(&edges);
        let lcom = calculate_lack_cohesion_methods(&graph, "A");
        // A's children (B, C) both call D → shared target
        // P=0, Q=1 → LCOM = 0.0 (good cohesion)
        assert!((lcom - 0.0).abs() < 0.01, "Expected LCOM ~0.0, got {}", lcom);
    }

    #[test]
    fn test_health_grade_all_ok() {
        let metrics = CkMetricsResult { cbo: 5, lcom: 0.3, rfc: 10, wmc: 8 };
        assert_eq!(grade_ck_metrics_health(&metrics), HealthGrade::A);
    }

    #[test]
    fn test_health_grade_one_warning() {
        let metrics = CkMetricsResult { cbo: 5, lcom: 0.3, rfc: 55, wmc: 8 }; // RFC > 50
        assert_eq!(grade_ck_metrics_health(&metrics), HealthGrade::B);
    }

    #[test]
    fn test_health_grade_one_fail() {
        let metrics = CkMetricsResult { cbo: 15, lcom: 0.3, rfc: 10, wmc: 8 }; // CBO > 10
        assert_eq!(grade_ck_metrics_health(&metrics), HealthGrade::D);
    }

    #[test]
    fn test_health_grade_two_fails() {
        let metrics = CkMetricsResult { cbo: 15, lcom: 0.9, rfc: 10, wmc: 8 }; // CBO > 10, LCOM > 0.8
        assert_eq!(grade_ck_metrics_health(&metrics), HealthGrade::F);
    }
}

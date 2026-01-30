//! PageRank importance ranking endpoint handler
//!
//! # 4-Word Naming: pagerank_ranking_handler
//!
//! Endpoint: GET /pagerank-importance-ranking-view
//!
//! Ranks code entities by importance using the PageRank algorithm.
//! PageRank identifies critical entities based on graph topology - entities
//! that are called by many important entities score higher than those with
//! simple coupling counts.
//!
//! # Algorithm
//! - Formula: PR(A) = (1-d) + d * Σ(PR(T)/C(T))
//! - Default damping: 0.85 (probability of following edges)
//! - Default iterations: 50
//! - Early convergence: stops when delta < 0.0001
//! - Score normalization: [0.0, 1.0] range
//!
//! # Performance Contract
//! - <200ms p99 latency for 10,000 entities at 50 iterations
//! - <1MB memory allocation during computation

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

use crate::http_server_startup_runner::SharedApplicationStateContainer;

/// Query parameters for PageRank ranking
///
/// # 4-Word Name: PageRankQueryParamsStruct
#[derive(Debug, Deserialize)]
pub struct PageRankQueryParamsStruct {
    /// Maximum iterations (1-1000, default: 50)
    pub iterations: Option<usize>,

    /// Damping factor (0.0-1.0, default: 0.85)
    pub damping: Option<f64>,

    /// Number of top results (1-100, default: 20)
    pub limit: Option<usize>,
}

impl Default for PageRankQueryParamsStruct {
    fn default() -> Self {
        Self {
            iterations: Some(50),
            damping: Some(0.85),
            limit: Some(20),
        }
    }
}

/// Single PageRank ranking entry
///
/// # 4-Word Name: PageRankRankingEntryPayload
#[derive(Debug, Serialize)]
pub struct PageRankRankingEntryPayload {
    pub rank: usize,
    pub entity_key: String,
    pub pagerank_score: f64,
    pub normalized_score: f64,
    pub inbound_edges: usize,
    pub outbound_edges: usize,
}

/// PageRank ranking response data
///
/// # 4-Word Name: PageRankRankingDataPayload
#[derive(Debug, Serialize)]
pub struct PageRankRankingDataPayload {
    pub entities_analyzed: usize,
    pub rankings_returned: usize,
    pub iterations_run: usize,
    pub damping_factor: f64,
    pub converged: bool,
    pub convergence_delta: f64,
    pub computation_time_ms: u128,
    pub rankings: Vec<PageRankRankingEntryPayload>,
}

/// PageRank ranking response payload
///
/// # 4-Word Name: PageRankRankingResponsePayload
#[derive(Debug, Serialize)]
pub struct PageRankRankingResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: PageRankRankingDataPayload,
    pub tokens: usize,
}

/// PageRank ranking error response
///
/// # 4-Word Name: PageRankRankingErrorResponseStruct
#[derive(Debug, Serialize)]
pub struct PageRankRankingErrorResponseStruct {
    pub success: bool,
    pub error: String,
    pub endpoint: String,
    pub tokens: usize,
}

/// Internal PageRank computation result
///
/// # 4-Word Name: PageRankComputationResultStruct
struct PageRankComputationResultStruct {
    pub scores: HashMap<String, f64>,
    pub iterations_run: usize,
    pub converged: bool,
    pub convergence_delta: f64,
    pub computation_time_ms: u128,
    pub inbound_counts: HashMap<String, usize>,
    pub outbound_counts: HashMap<String, usize>,
}

/// Handle PageRank importance ranking view request
///
/// # 4-Word Name: handle_pagerank_importance_ranking_view
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns entities ranked by PageRank score
/// - Performance: <200ms at p99 for 10,000 entities at 50 iterations
/// - Error Handling: Returns 400 for invalid parameters, 200 with empty array for no entities
///
/// # URL Pattern
/// - Endpoint: GET /pagerank-importance-ranking-view?iterations=N&damping=D&limit=L
/// - Default iterations: 50
/// - Default damping: 0.85
/// - Default limit: 20
pub async fn handle_pagerank_importance_ranking_view(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<PageRankQueryParamsStruct>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Extract and validate parameters
    let iterations = params.iterations.unwrap_or(50);
    let damping = params.damping.unwrap_or(0.85);
    let limit = params.limit.unwrap_or(20);

    // Validate iterations range [1, 1000]
    if iterations < 1 || iterations > 1000 {
        return (
            StatusCode::BAD_REQUEST,
            Json(PageRankRankingErrorResponseStruct {
                success: false,
                error: "Invalid iterations parameter. Must be between 1 and 1000".to_string(),
                endpoint: "/pagerank-importance-ranking-view".to_string(),
                tokens: 42,
            }),
        )
            .into_response();
    }

    // Validate damping range [0.0, 1.0]
    if damping < 0.0 || damping > 1.0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(PageRankRankingErrorResponseStruct {
                success: false,
                error: "Invalid damping parameter. Must be between 0.0 and 1.0".to_string(),
                endpoint: "/pagerank-importance-ranking-view".to_string(),
                tokens: 42,
            }),
        )
            .into_response();
    }

    // Validate limit range [1, 100]
    if limit < 1 || limit > 100 {
        return (
            StatusCode::BAD_REQUEST,
            Json(PageRankRankingErrorResponseStruct {
                success: false,
                error: "Invalid limit parameter. Must be between 1 and 100".to_string(),
                endpoint: "/pagerank-importance-ranking-view".to_string(),
                tokens: 42,
            }),
        )
            .into_response();
    }

    // Compute PageRank scores
    let computation_result =
        calculate_pagerank_scores_iteratively(&state, iterations, damping).await;

    // Handle empty graph case
    if computation_result.scores.is_empty() {
        return (
            StatusCode::OK,
            Json(PageRankRankingResponsePayload {
                success: true,
                endpoint: "/pagerank-importance-ranking-view".to_string(),
                data: PageRankRankingDataPayload {
                    entities_analyzed: 0,
                    rankings_returned: 0,
                    iterations_run: 0,
                    damping_factor: damping,
                    converged: true,
                    convergence_delta: 0.0,
                    computation_time_ms: computation_result.computation_time_ms,
                    rankings: Vec::new(),
                },
                tokens: 120,
            }),
        )
            .into_response();
    }

    // Normalize scores to [0.0, 1.0]
    let normalized_scores =
        normalize_pagerank_scores_to_range(&computation_result.scores);

    // Sort entities by normalized score descending
    let mut sorted_entities: Vec<(String, f64, f64)> = computation_result
        .scores
        .iter()
        .map(|(key, &raw_score)| {
            let normalized = *normalized_scores.get(key).unwrap_or(&0.0);
            (key.clone(), raw_score, normalized)
        })
        .collect();

    sorted_entities.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

    // Take top N and assign ranks
    let rankings: Vec<PageRankRankingEntryPayload> = sorted_entities
        .into_iter()
        .take(limit)
        .enumerate()
        .map(|(idx, (entity_key, raw_score, normalized_score))| {
            let inbound = *computation_result
                .inbound_counts
                .get(&entity_key)
                .unwrap_or(&0);
            let outbound = *computation_result
                .outbound_counts
                .get(&entity_key)
                .unwrap_or(&0);

            PageRankRankingEntryPayload {
                rank: idx + 1,
                entity_key,
                pagerank_score: raw_score,
                normalized_score,
                inbound_edges: inbound,
                outbound_edges: outbound,
            }
        })
        .collect();

    // Estimate token count
    let tokens = estimate_token_count_for_response(&rankings);

    (
        StatusCode::OK,
        Json(PageRankRankingResponsePayload {
            success: true,
            endpoint: "/pagerank-importance-ranking-view".to_string(),
            data: PageRankRankingDataPayload {
                entities_analyzed: computation_result.scores.len(),
                rankings_returned: rankings.len(),
                iterations_run: computation_result.iterations_run,
                damping_factor: damping,
                converged: computation_result.converged,
                convergence_delta: computation_result.convergence_delta,
                computation_time_ms: computation_result.computation_time_ms,
                rankings,
            },
            tokens,
        }),
    )
        .into_response()
}

/// Calculate PageRank scores iteratively
///
/// # 4-Word Name: calculate_pagerank_scores_iteratively
///
/// # Algorithm
/// 1. Load all edges from database
/// 2. Build adjacency lists (inbound/outbound per entity)
/// 3. Initialize all entities with score = 1.0 / N
/// 4. For each iteration:
///    a. For each entity A:
///       - Sum contributions from all inbound entities
///       - PR(A) = (1-d) + d * Σ(PR(T_i) / C(T_i))
///    b. Calculate max delta across all entities
///    c. If max_delta < 0.0001, break (converged)
/// 5. Return scores with metadata
///
/// # Performance
/// - <200ms for 10,000 entities at 50 iterations
/// - Memory: O(E + V) where E=edges, V=vertices
async fn calculate_pagerank_scores_iteratively(
    state: &SharedApplicationStateContainer,
    max_iterations: usize,
    damping: f64,
) -> PageRankComputationResultStruct {
    let start_time = Instant::now();

    let db_guard = state.database_storage_connection_arc.read().await;
    let storage = match db_guard.as_ref() {
        Some(s) => s,
        None => {
            return PageRankComputationResultStruct {
                scores: HashMap::new(),
                iterations_run: 0,
                converged: true,
                convergence_delta: 0.0,
                computation_time_ms: start_time.elapsed().as_millis(),
                inbound_counts: HashMap::new(),
                outbound_counts: HashMap::new(),
            };
        }
    };

    // Load all edges from database
    let query = "?[from_key, to_key] := *DependencyEdges{from_key, to_key}";
    let edges = match storage.raw_query(query).await {
        Ok(result) => result.rows,
        Err(_) => {
            return PageRankComputationResultStruct {
                scores: HashMap::new(),
                iterations_run: 0,
                converged: true,
                convergence_delta: 0.0,
                computation_time_ms: start_time.elapsed().as_millis(),
                inbound_counts: HashMap::new(),
                outbound_counts: HashMap::new(),
            };
        }
    };

    // Build adjacency structures
    let mut inbound_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut outbound_counts: HashMap<String, usize> = HashMap::new();
    let mut inbound_counts: HashMap<String, usize> = HashMap::new();
    let mut all_entities: Vec<String> = Vec::new();

    for row in edges {
        if row.len() >= 2 {
            let from_key = extract_string_value_helper(&row[0]).unwrap_or_default();
            let to_key = extract_string_value_helper(&row[1]).unwrap_or_default();

            // Track inbound edges: to_key receives edge from from_key
            inbound_map
                .entry(to_key.clone())
                .or_insert_with(Vec::new)
                .push(from_key.clone());

            // Count outbound edges
            *outbound_counts.entry(from_key.clone()).or_insert(0) += 1;

            // Count inbound edges
            *inbound_counts.entry(to_key.clone()).or_insert(0) += 1;

            // Collect all unique entities
            if !all_entities.contains(&from_key) {
                all_entities.push(from_key.clone());
            }
            if !all_entities.contains(&to_key) {
                all_entities.push(to_key);
            }
        }
    }

    // Handle empty graph
    if all_entities.is_empty() {
        return PageRankComputationResultStruct {
            scores: HashMap::new(),
            iterations_run: 0,
            converged: true,
            convergence_delta: 0.0,
            computation_time_ms: start_time.elapsed().as_millis(),
            inbound_counts: HashMap::new(),
            outbound_counts: HashMap::new(),
        };
    }

    // Initialize PageRank scores: all entities start with 1.0 / N
    let num_entities = all_entities.len();
    let initial_score = 1.0 / num_entities as f64;
    let mut current_scores: HashMap<String, f64> = all_entities
        .iter()
        .map(|e| (e.clone(), initial_score))
        .collect();

    let mut previous_scores: HashMap<String, f64>;
    let mut iterations_run = 0;
    let mut converged = false;
    let mut convergence_delta = 0.0;

    // PageRank iteration loop
    for iteration in 0..max_iterations {
        iterations_run = iteration + 1;

        // Copy current scores to previous
        previous_scores = current_scores.clone();

        // Calculate new scores for all entities
        for entity in &all_entities {
            let mut new_score = 1.0 - damping;

            // Sum contributions from inbound entities
            if let Some(inbound_entities) = inbound_map.get(entity) {
                for inbound_entity in inbound_entities {
                    let inbound_score = previous_scores.get(inbound_entity).unwrap_or(&initial_score);
                    let outbound_count = *outbound_counts.get(inbound_entity).unwrap_or(&1);

                    // Add contribution: PR(T) / C(T)
                    new_score += damping * (inbound_score / outbound_count as f64);
                }
            }

            current_scores.insert(entity.clone(), new_score);
        }

        // Check for convergence
        let (has_converged, max_delta) =
            check_pagerank_convergence_threshold(&current_scores, &previous_scores);
        convergence_delta = max_delta;

        if has_converged {
            converged = true;
            break;
        }
    }

    PageRankComputationResultStruct {
        scores: current_scores,
        iterations_run,
        converged,
        convergence_delta,
        computation_time_ms: start_time.elapsed().as_millis(),
        inbound_counts,
        outbound_counts,
    }
}

/// Check if PageRank has converged
///
/// # 4-Word Name: check_pagerank_convergence_threshold
///
/// Convergence occurs when maximum score change < 0.0001.
/// Returns (converged: bool, max_delta: f64).
fn check_pagerank_convergence_threshold(
    current_scores: &HashMap<String, f64>,
    previous_scores: &HashMap<String, f64>,
) -> (bool, f64) {
    let mut max_delta: f64 = 0.0;

    for (key, current) in current_scores {
        let previous = previous_scores.get(key).unwrap_or(&0.0);
        let delta = (current - previous).abs();
        max_delta = max_delta.max(delta);
    }

    (max_delta < 0.0001, max_delta)
}

/// Normalize PageRank scores to [0.0, 1.0]
///
/// # 4-Word Name: normalize_pagerank_scores_to_range
///
/// Applies min-max normalization: (score - min) / (max - min).
/// If all scores are equal, returns all as 1.0.
fn normalize_pagerank_scores_to_range(
    scores: &HashMap<String, f64>,
) -> HashMap<String, f64> {
    if scores.is_empty() {
        return HashMap::new();
    }

    let max_score = scores
        .values()
        .copied()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(1.0);

    let min_score = scores
        .values()
        .copied()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0);

    let range = max_score - min_score;

    if range == 0.0 {
        // All scores equal - return all as 1.0
        scores.keys().map(|k| (k.clone(), 1.0)).collect()
    } else {
        scores
            .iter()
            .map(|(k, v)| {
                let normalized = (v - min_score) / range;
                (k.clone(), normalized)
            })
            .collect()
    }
}

/// Estimate token count for PageRank response
///
/// # 4-Word Name: estimate_token_count_for_response
///
/// Base response structure: 120 tokens (metadata)
/// Per ranking entry: ~40 tokens
/// Total = 120 + (rankings.len() * 40)
fn estimate_token_count_for_response(rankings: &[PageRankRankingEntryPayload]) -> usize {
    let base = 120;
    let per_ranking = 40;
    base + (rankings.len() * per_ranking)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_empty_scores() {
        let scores = HashMap::new();
        let result = normalize_pagerank_scores_to_range(&scores);
        assert!(result.is_empty());
    }

    #[test]
    fn test_normalize_equal_scores() {
        let mut scores = HashMap::new();
        scores.insert("a".to_string(), 0.5);
        scores.insert("b".to_string(), 0.5);
        scores.insert("c".to_string(), 0.5);

        let result = normalize_pagerank_scores_to_range(&scores);

        // All equal scores should normalize to 1.0
        assert_eq!(result.get("a"), Some(&1.0));
        assert_eq!(result.get("b"), Some(&1.0));
        assert_eq!(result.get("c"), Some(&1.0));
    }

    #[test]
    fn test_normalize_varied_scores() {
        let mut scores = HashMap::new();
        scores.insert("a".to_string(), 0.0);
        scores.insert("b".to_string(), 0.5);
        scores.insert("c".to_string(), 1.0);

        let result = normalize_pagerank_scores_to_range(&scores);

        // Min-max normalization
        assert_eq!(result.get("a"), Some(&0.0));
        assert_eq!(result.get("b"), Some(&0.5));
        assert_eq!(result.get("c"), Some(&1.0));
    }

    #[test]
    fn test_convergence_threshold_converged() {
        let mut current = HashMap::new();
        current.insert("a".to_string(), 0.500001);
        current.insert("b".to_string(), 0.300001);

        let mut previous = HashMap::new();
        previous.insert("a".to_string(), 0.5);
        previous.insert("b".to_string(), 0.3);

        let (converged, delta) = check_pagerank_convergence_threshold(&current, &previous);

        assert!(converged);
        assert!(delta < 0.0001);
    }

    #[test]
    fn test_convergence_threshold_not_converged() {
        let mut current = HashMap::new();
        current.insert("a".to_string(), 0.6);
        current.insert("b".to_string(), 0.4);

        let mut previous = HashMap::new();
        previous.insert("a".to_string(), 0.5);
        previous.insert("b".to_string(), 0.3);

        let (converged, delta) = check_pagerank_convergence_threshold(&current, &previous);

        assert!(!converged);
        assert!(delta >= 0.0001);
        assert!((delta - 0.1).abs() < 0.001); // Should be 0.1
    }

    #[test]
    fn test_token_count_estimation() {
        let rankings = vec![
            PageRankRankingEntryPayload {
                rank: 1,
                entity_key: "test:fn:main:path:1-10".to_string(),
                pagerank_score: 1.0,
                normalized_score: 1.0,
                inbound_edges: 10,
                outbound_edges: 5,
            },
        ];

        let tokens = estimate_token_count_for_response(&rankings);
        assert_eq!(tokens, 160); // 120 base + 1 * 40
    }

    #[test]
    fn test_default_parameters() {
        let params = PageRankQueryParamsStruct::default();
        assert_eq!(params.iterations, Some(50));
        assert_eq!(params.damping, Some(0.85));
        assert_eq!(params.limit, Some(20));
    }
}

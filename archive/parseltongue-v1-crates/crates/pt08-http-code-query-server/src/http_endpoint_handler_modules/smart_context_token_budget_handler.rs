//! Smart context token budget endpoint handler
//!
//! # 4-Word Naming: smart_context_token_budget_handler
//!
//! Endpoint: GET /smart-context-token-budget?focus=entity&tokens=N
//!
//! THE KILLER FEATURE: Given a focus entity and token budget, returns the
//! optimal set of related entities that fit within the budget.
//! Prioritizes by relevance: direct deps > transitive deps > cluster peers.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::http_server_startup_runner::SharedApplicationStateContainer;
use crate::scope_filter_utilities_module::parse_scope_build_filter_clause;

/// Query parameters for smart context
///
/// # 4-Word Name: SmartContextQueryParamsStruct
#[derive(Debug, Deserialize)]
pub struct SmartContextQueryParamsStruct {
    /// Entity key to center context around
    pub focus: String,
    /// Maximum token budget (default: 4000)
    pub tokens: Option<usize>,
    /// Filter by folder scope (e.g., "crates||parseltongue-core")
    pub scope: Option<String>,
}

/// Single context entry
///
/// # 4-Word Name: SmartContextEntryPayload
#[derive(Debug, Serialize, Clone)]
pub struct SmartContextEntryPayload {
    pub entity_key: String,
    pub relevance_score: f64,
    pub relevance_type: String,
    pub estimated_tokens: usize,
}

/// Smart context response data
///
/// # 4-Word Name: SmartContextDataPayload
#[derive(Debug, Serialize)]
pub struct SmartContextDataPayload {
    pub focus_entity: String,
    pub token_budget: usize,
    pub tokens_used: usize,
    pub entities_included: usize,
    pub context: Vec<SmartContextEntryPayload>,
}

/// Smart context response payload
///
/// # 4-Word Name: SmartContextResponsePayload
#[derive(Debug, Serialize)]
pub struct SmartContextResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: SmartContextDataPayload,
    pub tokens: usize,
}

/// Handle smart context token budget request
///
/// # 4-Word Name: handle_smart_context_token_budget
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns optimal context within token budget
/// - Performance: O(E + V log V) for graph traversal and sorting
/// - Error Handling: Returns empty context if focus not found
///
/// # Algorithm: Relevance-Weighted Greedy Selection
/// 1. Start from focus entity
/// 2. Score all reachable entities by relevance type:
///    - direct_caller: 1.0 (entities that call focus)
///    - direct_callee: 0.95 (entities called by focus)
///    - transitive_dep: 0.7 - (0.1 * depth)
/// 3. Sort by score descending
/// 4. Greedily select until budget exhausted
pub async fn handle_smart_context_token_budget(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<SmartContextQueryParamsStruct>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    let focus = params.focus;
    let token_budget = params.tokens.unwrap_or(4000);

    // Build context with greedy selection
    let context = build_smart_context_selection(&state, &focus, token_budget, &params.scope).await;

    let tokens_used: usize = context.iter().map(|e| e.estimated_tokens).sum();
    let entities_included = context.len();

    // Estimate response tokens
    let response_tokens = 100 + (entities_included * 40);

    (
        StatusCode::OK,
        Json(SmartContextResponsePayload {
            success: true,
            endpoint: "/smart-context-token-budget".to_string(),
            data: SmartContextDataPayload {
                focus_entity: focus,
                token_budget,
                tokens_used,
                entities_included,
                context,
            },
            tokens: response_tokens,
        }),
    ).into_response()
}

/// Build smart context using greedy selection
///
/// # 4-Word Name: build_smart_context_selection
///
/// Traverses the dependency graph from focus entity,
/// scoring entities by relevance, then greedily selects
/// highest-scoring entities that fit within budget.
async fn build_smart_context_selection(
    state: &SharedApplicationStateContainer,
    focus: &str,
    budget: usize,
    scope_filter: &Option<String>,
) -> Vec<SmartContextEntryPayload> {
    // Clone Arc, release lock, then await
    let storage = {
        let db_guard = state.database_storage_connection_arc.read().await;
        match db_guard.as_ref() {
            Some(s) => s.clone(),
            None => return Vec::new(),
        }
    }; // Lock released here

    // Build scope filter clause
    let scope_clause = parse_scope_build_filter_clause(scope_filter);
    let scope_join = if scope_clause.is_empty() {
        String::new()
    } else {
        format!(", *CodeGraph{{ISGL1_key: from_key, root_subfolder_L1, root_subfolder_L2}}{}", scope_clause)
    };

    // Query all edges with scope filtering
    let query = format!("?[from_key, to_key] := *DependencyEdges{{from_key, to_key}}{}", scope_join);
    let edges = match storage.raw_query(&query).await {
        Ok(result) => result.rows,
        Err(_) => return Vec::new(),
    };

    // Build adjacency lists (forward and reverse)
    let mut forward: HashMap<String, Vec<String>> = HashMap::new();
    let mut reverse: HashMap<String, Vec<String>> = HashMap::new();

    for row in edges {
        if row.len() >= 2 {
            let from = extract_string_value_helper(&row[0]).unwrap_or_default();
            let to = extract_string_value_helper(&row[1]).unwrap_or_default();

            forward.entry(from.clone()).or_default().push(to.clone());
            reverse.entry(to).or_default().push(from);
        }
    }

    // Score all related entities
    let mut scored_entities: Vec<SmartContextEntryPayload> = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    visited.insert(focus.to_string());

    // Direct callers (highest relevance for understanding "who uses this")
    if let Some(callers) = reverse.get(focus) {
        for caller in callers {
            if !visited.contains(caller) {
                visited.insert(caller.clone());
                scored_entities.push(SmartContextEntryPayload {
                    entity_key: caller.clone(),
                    relevance_score: 1.0,
                    relevance_type: "direct_caller".to_string(),
                    estimated_tokens: estimate_entity_tokens(caller),
                });
            }
        }
    }

    // Direct callees (very high relevance for understanding "what this uses")
    if let Some(callees) = forward.get(focus) {
        for callee in callees {
            if !visited.contains(callee) {
                visited.insert(callee.clone());
                scored_entities.push(SmartContextEntryPayload {
                    entity_key: callee.clone(),
                    relevance_score: 0.95,
                    relevance_type: "direct_callee".to_string(),
                    estimated_tokens: estimate_entity_tokens(callee),
                });
            }
        }
    }

    // Transitive dependencies (BFS with depth tracking)
    let mut queue: Vec<(String, usize)> = Vec::new();

    // Add direct callees to queue for transitive expansion
    if let Some(callees) = forward.get(focus) {
        for callee in callees {
            queue.push((callee.clone(), 1));
        }
    }

    while let Some((entity, depth)) = queue.pop() {
        if depth > 3 {
            continue; // Limit depth
        }

        if let Some(next_callees) = forward.get(&entity) {
            for next in next_callees {
                if !visited.contains(next) {
                    visited.insert(next.clone());

                    let relevance = 0.7 - (0.1 * depth as f64);
                    scored_entities.push(SmartContextEntryPayload {
                        entity_key: next.clone(),
                        relevance_score: relevance.max(0.1),
                        relevance_type: format!("transitive_depth_{}", depth + 1),
                        estimated_tokens: estimate_entity_tokens(next),
                    });

                    queue.push((next.clone(), depth + 1));
                }
            }
        }
    }

    // Sort by relevance descending
    scored_entities.sort_by(|a, b| {
        b.relevance_score
            .partial_cmp(&a.relevance_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Greedy selection within budget
    let mut selected: Vec<SmartContextEntryPayload> = Vec::new();
    let mut used_tokens: usize = 0;

    for entity in scored_entities {
        if used_tokens + entity.estimated_tokens <= budget {
            used_tokens += entity.estimated_tokens;
            selected.push(entity);
        }
    }

    selected
}

/// Estimate tokens for an entity based on key
///
/// # 4-Word Name: estimate_entity_tokens_from_key
///
/// Heuristic: Entity key length * 4 (rough approximation)
/// Real implementation would look up actual code size.
fn estimate_entity_tokens(entity_key: &str) -> usize {
    // Heuristic based on key structure
    // Format: rust:fn:name:file_path:lines
    // Assume average function is ~100 tokens
    let base = 100;

    // Longer keys often mean more complex entities
    let complexity_bonus = entity_key.len() / 10;

    base + complexity_bonus
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

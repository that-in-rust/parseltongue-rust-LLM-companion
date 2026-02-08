//! Technical debt SQALE scoring endpoint handler
//!
//! # 4-Word Naming: technical_debt_sqale_handler
//!
//! Endpoint: GET /technical-debt-sqale-scoring
//!
//! Calculates technical debt using SQALE methodology based on
//! coupling and complexity metrics violations.

use axum::{
    extract::{State, Query},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

use crate::http_server_startup_runner::SharedApplicationStateContainer;
use parseltongue_core::graph_analysis::{
    AdjacencyListGraphRepresentation,
    compute_all_entities_sqale,
    classify_debt_severity_level,
    DebtSeverity,
    SqaleViolationRecord,
};

/// Query parameters for technical debt
///
/// # 4-Word Name: TechnicalDebtQueryParameters
#[derive(Debug, Deserialize)]
pub struct TechnicalDebtQueryParameters {
    pub entity: Option<String>,
    pub min_debt: Option<f64>,
}

/// Single violation in SQALE analysis
///
/// # 4-Word Name: SqaleViolationDataStructure
#[derive(Debug, Serialize)]
pub struct SqaleViolationDataStructure {
    pub violation_type: String,
    pub metric: String,
    pub value: f64,
    pub threshold: f64,
    pub remediation_hours: f64,
}

/// Technical debt for single entity
///
/// # 4-Word Name: EntityTechnicalDebtData
#[derive(Debug, Serialize)]
pub struct EntityTechnicalDebtData {
    pub entity: String,
    pub total_debt_hours: f64,
    pub violations: Vec<SqaleViolationDataStructure>,
    pub severity: String,
}

/// Technical debt response data payload
///
/// # 4-Word Name: TechnicalDebtDataPayload
#[derive(Debug, Serialize)]
pub struct TechnicalDebtDataPayload {
    pub entities: Vec<EntityTechnicalDebtData>,
    pub codebase_total_debt_hours: f64,
}

/// Technical debt response payload
///
/// # 4-Word Name: TechnicalDebtResponsePayload
#[derive(Debug, Serialize)]
pub struct TechnicalDebtResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: TechnicalDebtDataPayload,
    pub tokens: usize,
}

/// Error response payload structure
///
/// # 4-Word Name: ErrorResponsePayloadStructure
#[derive(Debug, Serialize)]
pub struct ErrorResponsePayloadStructure {
    pub success: bool,
    pub endpoint: String,
    pub error: String,
}

/// Handle technical debt SQALE scoring request
///
/// # 4-Word Name: handle_technical_debt_sqale_scoring
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns technical debt scores for entities
/// - Performance: O(V * E) for computing CK metrics
/// - Error Handling: Returns error JSON if database unavailable
pub async fn handle_technical_debt_sqale_scoring(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<TechnicalDebtQueryParameters>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Clone Arc inside RwLock scope, release lock
    let storage = {
        let db_guard = state.database_storage_connection_arc.read().await;
        match db_guard.as_ref() {
            Some(s) => s.clone(),
            None => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponsePayloadStructure {
                        success: false,
                        endpoint: "/technical-debt-sqale-scoring".to_string(),
                        error: "Database not connected".to_string(),
                    }),
                )
                    .into_response()
            }
        }
    }; // Lock released here

    // Build graph from database
    let graph = match build_graph_from_database_edges(&storage).await {
        Ok(g) => g,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponsePayloadStructure {
                    success: false,
                    endpoint: "/technical-debt-sqale-scoring".to_string(),
                    error: e,
                }),
            )
                .into_response()
        }
    };

    // Compute SQALE debt for all entities
    let debt_results = compute_all_entities_sqale(&graph);

    // Filter and format results
    let mut entity_data: Vec<EntityTechnicalDebtData> = Vec::new();
    let mut total_debt = 0.0;

    for sqale_result in debt_results.iter() {
        let entity = &sqale_result.entity;
        // Apply filters
        if let Some(ref entity_filter) = params.entity {
            if !entity.contains(entity_filter) {
                continue;
            }
        }

        if let Some(min_debt) = params.min_debt {
            if sqale_result.total_debt_hours < min_debt {
                continue;
            }
        }

        total_debt += sqale_result.total_debt_hours;

        let violations: Vec<SqaleViolationDataStructure> = sqale_result
            .violations
            .iter()
            .map(|v| convert_violation_to_structure(v))
            .collect();

        let severity = classify_debt_severity_level(sqale_result.total_debt_hours);
        let severity_str = match severity {
            DebtSeverity::None => "NONE",
            DebtSeverity::Low => "LOW",
            DebtSeverity::Medium => "MEDIUM",
            DebtSeverity::High => "HIGH",
        };

        entity_data.push(EntityTechnicalDebtData {
            entity: entity.clone(),
            total_debt_hours: sqale_result.total_debt_hours,
            violations,
            severity: severity_str.to_string(),
        });
    }

    // Sort by debt descending
    entity_data.sort_by(|a, b| {
        b.total_debt_hours
            .partial_cmp(&a.total_debt_hours)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Estimate tokens
    let total_violations: usize = entity_data.iter().map(|e| e.violations.len()).sum();
    let tokens = 60 + (entity_data.len() * 40) + (total_violations * 25);

    (
        StatusCode::OK,
        Json(TechnicalDebtResponsePayload {
            success: true,
            endpoint: "/technical-debt-sqale-scoring".to_string(),
            data: TechnicalDebtDataPayload {
                entities: entity_data,
                codebase_total_debt_hours: total_debt,
            },
            tokens,
        }),
    )
        .into_response()
}

/// Convert violation to serializable structure
///
/// # 4-Word Name: convert_violation_to_structure
fn convert_violation_to_structure(v: &SqaleViolationRecord) -> SqaleViolationDataStructure {
    use parseltongue_core::graph_analysis::SqaleViolationType;

    let violation_type_str = match v.violation_type {
        SqaleViolationType::HighCoupling => "HIGH_COUPLING",
        SqaleViolationType::LowCohesion => "LOW_COHESION",
        SqaleViolationType::HighComplexity => "HIGH_COMPLEXITY",
    };

    SqaleViolationDataStructure {
        violation_type: violation_type_str.to_string(),
        metric: v.metric_name.clone(),
        value: v.value,
        threshold: v.threshold,
        remediation_hours: v.remediation_hours,
    }
}

/// Build graph from database edges query
///
/// # 4-Word Name: build_graph_from_database_edges
async fn build_graph_from_database_edges(
    storage: &Arc<parseltongue_core::storage::CozoDbStorage>,
) -> Result<AdjacencyListGraphRepresentation, String> {
    let query = "?[from_key, to_key, edge_type] := *DependencyEdges{from_key, to_key, edge_type}";
    let result = storage
        .raw_query(query)
        .await
        .map_err(|e| format!("Database query failed: {}", e))?;

    let mut edges: Vec<(String, String, String)> = Vec::new();
    for row in &result.rows {
        if row.len() >= 3 {
            let from = extract_string_from_datavalue(&row[0]);
            let to = extract_string_from_datavalue(&row[1]);
            let edge_type = extract_string_from_datavalue(&row[2]);
            edges.push((from, to, edge_type));
        }
    }

    Ok(AdjacencyListGraphRepresentation::build_from_dependency_edges(&edges))
}

/// Extract string from CozoDB DataValue
///
/// # 4-Word Name: extract_string_from_datavalue
fn extract_string_from_datavalue(value: &cozo::DataValue) -> String {
    match value {
        cozo::DataValue::Str(s) => s.to_string(),
        _ => format!("{:?}", value),
    }
}

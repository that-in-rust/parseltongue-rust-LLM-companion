//! API reference documentation endpoint handler
//!
//! # 4-Word Naming: api_reference_documentation_handler
//!
//! Endpoint: GET /api-reference-documentation-help
//!
//! Returns comprehensive API documentation listing all available endpoints,
//! their parameters, and response formats. Self-documenting API for LLM consumers.

use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::Serialize;

use crate::http_server_startup_runner::SharedApplicationStateContainer;

/// Single parameter documentation
///
/// # 4-Word Name: EndpointParameterDocPayload
#[derive(Debug, Serialize)]
pub struct EndpointParameterDocPayload {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: String,
}

/// Single endpoint documentation
///
/// # 4-Word Name: EndpointDocumentationEntryPayload
#[derive(Debug, Serialize)]
pub struct EndpointDocumentationEntryPayload {
    pub path: String,
    pub method: String,
    pub description: String,
    pub parameters: Vec<EndpointParameterDocPayload>,
}

/// Category of endpoints
///
/// # 4-Word Name: EndpointCategoryDocPayload
#[derive(Debug, Serialize)]
pub struct EndpointCategoryDocPayload {
    pub name: String,
    pub endpoints: Vec<EndpointDocumentationEntryPayload>,
}

/// API reference response data
///
/// # 4-Word Name: ApiReferenceDataPayload
#[derive(Debug, Serialize)]
pub struct ApiReferenceDataPayload {
    pub api_version: String,
    pub total_endpoints: usize,
    pub categories: Vec<EndpointCategoryDocPayload>,
}

/// API reference response payload
///
/// # 4-Word Name: ApiReferenceResponsePayload
#[derive(Debug, Serialize)]
pub struct ApiReferenceResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: ApiReferenceDataPayload,
    pub tokens: usize,
}

/// Handle API reference documentation help request
///
/// # 4-Word Name: handle_api_reference_documentation_help
///
/// # Contract
/// - Precondition: Server running with routes registered
/// - Postcondition: Returns complete API documentation
/// - Performance: <10ms (static data)
/// - Error Handling: Always succeeds (static data)
pub async fn handle_api_reference_documentation_help(
    State(state): State<SharedApplicationStateContainer>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Build static API documentation
    let categories = build_api_documentation_categories();
    let total_endpoints: usize = categories.iter()
        .map(|c| c.endpoints.len())
        .sum();

    // Estimate tokens
    let tokens = 100 + (total_endpoints * 50);

    (
        StatusCode::OK,
        Json(ApiReferenceResponsePayload {
            success: true,
            endpoint: "/api-reference-documentation-help".to_string(),
            data: ApiReferenceDataPayload {
                api_version: "1.0.2".to_string(),
                total_endpoints,
                categories,
            },
            tokens,
        }),
    ).into_response()
}

/// Build API documentation categories with all endpoints
///
/// # 4-Word Name: build_api_documentation_categories
fn build_api_documentation_categories() -> Vec<EndpointCategoryDocPayload> {
    vec![
        EndpointCategoryDocPayload {
            name: "Core".to_string(),
            endpoints: vec![
                EndpointDocumentationEntryPayload {
                    path: "/server-health-check-status".to_string(),
                    method: "GET".to_string(),
                    description: "Health check endpoint - returns server status".to_string(),
                    parameters: vec![],
                },
                EndpointDocumentationEntryPayload {
                    path: "/codebase-statistics-overview-summary".to_string(),
                    method: "GET".to_string(),
                    description: "Returns entity and edge counts for the codebase".to_string(),
                    parameters: vec![],
                },
                EndpointDocumentationEntryPayload {
                    path: "/api-reference-documentation-help".to_string(),
                    method: "GET".to_string(),
                    description: "Returns this API documentation".to_string(),
                    parameters: vec![],
                },
            ],
        },
        EndpointCategoryDocPayload {
            name: "Entity".to_string(),
            endpoints: vec![
                EndpointDocumentationEntryPayload {
                    path: "/code-entities-list-all".to_string(),
                    method: "GET".to_string(),
                    description: "Lists all code entities in the database".to_string(),
                    parameters: vec![
                        EndpointParameterDocPayload {
                            name: "entity_type".to_string(),
                            param_type: "query".to_string(),
                            required: false,
                            description: "Filter by entity type (fn, struct, etc)".to_string(),
                        },
                    ],
                },
                EndpointDocumentationEntryPayload {
                    path: "/code-entity-detail-view/{key}".to_string(),
                    method: "GET".to_string(),
                    description: "Returns details for a specific entity by key".to_string(),
                    parameters: vec![
                        EndpointParameterDocPayload {
                            name: "key".to_string(),
                            param_type: "path".to_string(),
                            required: true,
                            description: "Entity key (e.g., rust:fn:main:src_main_rs:1-50)".to_string(),
                        },
                    ],
                },
                EndpointDocumentationEntryPayload {
                    path: "/code-entities-search-fuzzy".to_string(),
                    method: "GET".to_string(),
                    description: "Fuzzy search entities by name pattern".to_string(),
                    parameters: vec![
                        EndpointParameterDocPayload {
                            name: "q".to_string(),
                            param_type: "query".to_string(),
                            required: true,
                            description: "Search pattern (case-insensitive substring match)".to_string(),
                        },
                    ],
                },
            ],
        },
        EndpointCategoryDocPayload {
            name: "Edge".to_string(),
            endpoints: vec![
                EndpointDocumentationEntryPayload {
                    path: "/dependency-edges-list-all".to_string(),
                    method: "GET".to_string(),
                    description: "Lists all dependency edges in the graph".to_string(),
                    parameters: vec![],
                },
                EndpointDocumentationEntryPayload {
                    path: "/reverse-callers-query-graph".to_string(),
                    method: "GET".to_string(),
                    description: "Returns entities that call the specified entity".to_string(),
                    parameters: vec![
                        EndpointParameterDocPayload {
                            name: "entity".to_string(),
                            param_type: "query".to_string(),
                            required: true,
                            description: "Entity key to find callers for".to_string(),
                        },
                    ],
                },
                EndpointDocumentationEntryPayload {
                    path: "/forward-callees-query-graph".to_string(),
                    method: "GET".to_string(),
                    description: "Returns entities called by the specified entity".to_string(),
                    parameters: vec![
                        EndpointParameterDocPayload {
                            name: "entity".to_string(),
                            param_type: "query".to_string(),
                            required: true,
                            description: "Entity key to find callees for".to_string(),
                        },
                    ],
                },
            ],
        },
        EndpointCategoryDocPayload {
            name: "Analysis".to_string(),
            endpoints: vec![
                EndpointDocumentationEntryPayload {
                    path: "/blast-radius-impact-analysis".to_string(),
                    method: "GET".to_string(),
                    description: "Calculates transitive impact of changes to an entity".to_string(),
                    parameters: vec![
                        EndpointParameterDocPayload {
                            name: "entity".to_string(),
                            param_type: "query".to_string(),
                            required: true,
                            description: "Entity key to analyze blast radius for".to_string(),
                        },
                        EndpointParameterDocPayload {
                            name: "hops".to_string(),
                            param_type: "query".to_string(),
                            required: false,
                            description: "Maximum depth to traverse (default: 3)".to_string(),
                        },
                    ],
                },
                EndpointDocumentationEntryPayload {
                    path: "/circular-dependency-detection-scan".to_string(),
                    method: "GET".to_string(),
                    description: "Detects cycles in the dependency graph".to_string(),
                    parameters: vec![],
                },
                EndpointDocumentationEntryPayload {
                    path: "/complexity-hotspots-ranking-view".to_string(),
                    method: "GET".to_string(),
                    description: "Ranks entities by coupling complexity".to_string(),
                    parameters: vec![
                        EndpointParameterDocPayload {
                            name: "top".to_string(),
                            param_type: "query".to_string(),
                            required: false,
                            description: "Number of hotspots to return (default: 10)".to_string(),
                        },
                    ],
                },
                EndpointDocumentationEntryPayload {
                    path: "/semantic-cluster-grouping-list".to_string(),
                    method: "GET".to_string(),
                    description: "Groups entities into semantic clusters by connectivity".to_string(),
                    parameters: vec![],
                },
            ],
        },
    ]
}

//! Route definition builder for HTTP server
//!
//! # 4-Word Naming: route_definition_builder_module

use axum::{
    Router,
    routing::get,
};

use crate::http_server_startup_runner::SharedApplicationStateContainer;
use crate::http_endpoint_handler_modules::{
    server_health_check_handler,
    codebase_statistics_overview_handler,
    code_entities_list_all_handler,
    code_entity_detail_view_handler,
    code_entities_fuzzy_search_handler,
};

/// Build the complete router with all endpoints
///
/// # 4-Word Name: build_complete_router_instance
///
/// ## Core Endpoints
/// - GET /server-health-check-status
/// - GET /codebase-statistics-overview-summary
/// - GET /api-reference-documentation-help
///
/// ## Entity Endpoints
/// - GET /code-entities-list-all
/// - GET /code-entity-detail-view/{key}
/// - GET /fuzzy-entity-search-query?q=pattern
///
/// ## Edge Endpoints
/// - GET /dependency-edges-list-all
/// - GET /reverse-callers-query-graph/{entity}
/// - GET /forward-callees-query-graph/{entity}
///
/// ## Analysis Endpoints
/// - GET /blast-radius-impact-analysis/{entity}?hops=N
/// - GET /circular-dependency-detection-scan
/// - GET /complexity-hotspots-ranking-view?top=N
/// - GET /semantic-cluster-grouping-list
///
/// ## Killer Features
/// - GET /temporal-coupling-hidden-deps/{entity}
/// - GET /smart-context-token-budget?focus=X&tokens=N
pub fn build_complete_router_instance(state: SharedApplicationStateContainer) -> Router {
    Router::new()
        // Core endpoints
        .route(
            "/server-health-check-status",
            get(server_health_check_handler::handle_server_health_check_status)
        )
        .route(
            "/codebase-statistics-overview-summary",
            get(codebase_statistics_overview_handler::handle_codebase_statistics_overview_summary)
        )
        // Entity endpoints
        .route(
            "/code-entities-list-all",
            get(code_entities_list_all_handler::handle_code_entities_list_all)
        )
        .route(
            "/code-entity-detail-view/{key}",
            get(code_entity_detail_view_handler::handle_code_entity_detail_view)
        )
        .route(
            "/code-entities-search-fuzzy",
            get(code_entities_fuzzy_search_handler::handle_code_entities_fuzzy_search)
        )
        // More endpoints will be added in subsequent phases
        .with_state(state)
}

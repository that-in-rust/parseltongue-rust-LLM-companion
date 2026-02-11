//! Route definition builder for HTTP server
//!
//! # 4-Word Naming: route_definition_builder_module

use axum::{
    Router,
    routing::{get, post},
};

use crate::http_server_startup_runner::SharedApplicationStateContainer;
use crate::http_endpoint_handler_modules::{
    server_health_check_handler,
    codebase_statistics_overview_handler,
    code_entities_list_all_handler,
    code_entity_detail_view_handler,
    code_entities_fuzzy_search_handler,
    reverse_callers_query_graph_handler,
    forward_callees_query_graph_handler,
    dependency_edges_list_handler,
    blast_radius_impact_handler,
    circular_dependency_detection_handler,
    complexity_hotspots_ranking_handler,
    semantic_cluster_grouping_handler,
    api_reference_documentation_handler,
    smart_context_token_budget_handler,
    // v1.5.0: ISGL1 v2 integration complete - re-enabled
    incremental_reindex_file_handler,
    file_watcher_status_handler,
    // v1.6.0: Graph analysis endpoints
    strongly_connected_components_handler,
    technical_debt_sqale_handler,
    kcore_decomposition_layering_handler,
    centrality_measures_entity_handler,
    entropy_complexity_measurement_handler,
    coupling_cohesion_metrics_handler,
    leiden_community_detection_handler,
    // v1.6.1: Ingestion coverage reporting
    ingestion_coverage_folder_handler,
    // v1.6.5: Diagnostics and folder discovery
    ingestion_diagnostics_coverage_handler,
    folder_structure_discovery_handler,
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
/// - GET /code-entity-detail-view/{*key}
/// - GET /fuzzy-entity-search-query?q=pattern
///
/// ## Edge Endpoints
/// - GET /dependency-edges-list-all
/// - GET /reverse-callers-query-graph/{*entity}
/// - GET /forward-callees-query-graph/{*entity}
///
/// ## Analysis Endpoints
/// - GET /blast-radius-impact-analysis/{entity}?hops=N
/// - GET /circular-dependency-detection-scan
/// - GET /complexity-hotspots-ranking-view?top=N
/// - GET /semantic-cluster-grouping-list
///
/// ## Context Optimization
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
        .route(
            "/api-reference-documentation-help",
            get(api_reference_documentation_handler::handle_api_reference_documentation_help)
        )
        // Entity endpoints
        .route(
            "/code-entities-list-all",
            get(code_entities_list_all_handler::handle_code_entities_list_all)
        )
        .route(
            "/code-entity-detail-view",
            get(code_entity_detail_view_handler::handle_code_entity_detail_view)
        )
        .route(
            "/code-entities-search-fuzzy",
            get(code_entities_fuzzy_search_handler::handle_code_entities_fuzzy_search)
        )
        // Graph query endpoints (using query parameters to avoid colon routing issues)
        .route(
            "/reverse-callers-query-graph",
            get(reverse_callers_query_graph_handler::handle_reverse_callers_query_graph)
        )
        .route(
            "/forward-callees-query-graph",
            get(forward_callees_query_graph_handler::handle_forward_callees_query_graph)
        )
        .route(
            "/dependency-edges-list-all",
            get(dependency_edges_list_handler::handle_dependency_edges_list_all)
        )
        .route(
            "/blast-radius-impact-analysis",
            get(blast_radius_impact_handler::handle_blast_radius_impact_analysis)
        )
        .route(
            "/circular-dependency-detection-scan",
            get(circular_dependency_detection_handler::handle_circular_dependency_detection_scan)
        )
        .route(
            "/complexity-hotspots-ranking-view",
            get(complexity_hotspots_ranking_handler::handle_complexity_hotspots_ranking_view)
        )
        .route(
            "/semantic-cluster-grouping-list",
            get(semantic_cluster_grouping_handler::handle_semantic_cluster_grouping_list)
        )
        .route(
            "/smart-context-token-budget",
            get(smart_context_token_budget_handler::handle_smart_context_token_budget)
        )
        // v1.5.0: ISGL1 v2 integration complete - re-enabled
        // Incremental reindex endpoint (PRD-2026-01-28)
        .route(
            "/incremental-reindex-file-update",
            post(incremental_reindex_file_handler::handle_incremental_reindex_file_request)
        )
        // File watcher status endpoint (PRD-2026-01-29)
        .route(
            "/file-watcher-status-check",
            get(file_watcher_status_handler::handle_file_watcher_status_check)
        )
        // v1.6.0: Graph Analysis Endpoints
        .route(
            "/strongly-connected-components-analysis",
            get(strongly_connected_components_handler::handle_strongly_connected_components_analysis)
        )
        .route(
            "/technical-debt-sqale-scoring",
            get(technical_debt_sqale_handler::handle_technical_debt_sqale_scoring)
        )
        .route(
            "/kcore-decomposition-layering-analysis",
            get(kcore_decomposition_layering_handler::handle_kcore_decomposition_layering_analysis)
        )
        .route(
            "/centrality-measures-entity-ranking",
            get(centrality_measures_entity_handler::handle_centrality_measures_entity_ranking)
        )
        .route(
            "/entropy-complexity-measurement-scores",
            get(entropy_complexity_measurement_handler::handle_entropy_complexity_measurement_scores)
        )
        .route(
            "/coupling-cohesion-metrics-suite",
            get(coupling_cohesion_metrics_handler::handle_coupling_cohesion_metrics_suite)
        )
        .route(
            "/leiden-community-detection-clusters",
            get(leiden_community_detection_handler::handle_leiden_community_detection_clusters)
        )
        // v1.6.1: Ingestion coverage reporting
        .route(
            "/ingestion-coverage-folder-report",
            get(ingestion_coverage_folder_handler::handle_ingestion_coverage_folder_report)
        )
        // v1.6.5: Diagnostics and folder discovery
        .route(
            "/ingestion-diagnostics-coverage-report",
            get(ingestion_diagnostics_coverage_handler::handle_ingestion_diagnostics_coverage_report)
        )
        .route(
            "/folder-structure-discovery-tree",
            get(folder_structure_discovery_handler::handle_folder_structure_discovery_tree)
        )
        // Test route for debugging
        .route(
            "/test-simple/{param}",
            get(reverse_callers_query_graph_handler::handle_reverse_callers_query_graph)
        )
        // More endpoints will be added in subsequent phases
        .with_state(state)
}

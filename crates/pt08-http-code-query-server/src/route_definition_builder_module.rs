//! Route definition builder for HTTP server
//!
//! # 4-Word Naming: route_definition_builder_module

use axum::{
    Router,
    routing::get,
    routing::post,
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
    temporal_coupling_hidden_deps_handler,
    diff_analysis_compare_handler,
    // Phase 2.1: Workspace Management Handlers
    workspace_create_handler,
    workspace_list_handler,
    workspace_watch_handler,
};
use crate::websocket_streaming_module::handler::handle_websocket_diff_stream_upgrade;
// Phase 2.6: Static file embedding handlers
use crate::static_file_embed_module::{
    serve_root_index_handler,
    serve_static_asset_handler,
    serve_spa_fallback_handler,
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
        .route(
            "/temporal-coupling-hidden-deps",
            get(temporal_coupling_hidden_deps_handler::handle_temporal_coupling_hidden_deps)
        )
        // Test route for debugging
        .route(
            "/test-simple/{param}",
            get(reverse_callers_query_graph_handler::handle_reverse_callers_query_graph)
        )
        // Diff analysis endpoints
        .route(
            "/diff-analysis-compare-snapshots",
            post(diff_analysis_compare_handler::handle_diff_analysis_compare_snapshots)
        )
        // Phase 2.1: Workspace Management Endpoints
        .route(
            "/workspace-create-from-path",
            post(workspace_create_handler::handle_workspace_create_from_path)
        )
        .route(
            "/workspace-list-all",
            get(workspace_list_handler::handle_workspace_list_all_entries)
        )
        .route(
            "/workspace-watch-toggle",
            post(workspace_watch_handler::handle_workspace_watch_toggle_state)
        )
        // Phase 2.3: WebSocket Diff Streaming
        .route(
            "/websocket-diff-stream",
            get(handle_websocket_diff_stream_upgrade)
        )
        // Phase 2.6: Static file embedding routes
        // Static asset routes (priority 2 - after API routes)
        .route("/assets/*path", get(serve_static_asset_handler))
        // Root path (priority 3)
        .route("/", get(serve_root_index_handler))
        // SPA fallback (priority 4 - catch-all for client-side routing)
        .fallback(get(serve_spa_fallback_handler))
        .with_state(state)
}

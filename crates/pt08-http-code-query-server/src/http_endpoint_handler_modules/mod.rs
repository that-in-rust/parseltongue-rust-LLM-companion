//! HTTP endpoint handler modules
//!
//! # 4-Word Naming: http_endpoint_handler_modules
//!
//! Each handler file follows 4-word naming convention.

pub mod server_health_check_handler;
pub mod codebase_statistics_overview_handler;
pub mod code_entities_list_all_handler;
pub mod code_entity_detail_view_handler;
pub mod code_entities_fuzzy_search_handler;
pub mod reverse_callers_query_graph_handler;
pub mod forward_callees_query_graph_handler;
pub mod dependency_edges_list_handler;
pub mod blast_radius_impact_handler;
pub mod circular_dependency_detection_handler;
pub mod complexity_hotspots_ranking_handler;
pub mod semantic_cluster_grouping_handler;
pub mod api_reference_documentation_handler;
pub mod smart_context_token_budget_handler;
// v1.5.0: ISGL1 v2 integration complete - re-enabled
pub mod incremental_reindex_file_handler;
pub mod file_watcher_status_handler;
// v1.6.0: Graph analysis endpoints
pub mod strongly_connected_components_handler;
pub mod technical_debt_sqale_handler;
pub mod kcore_decomposition_layering_handler;
pub mod centrality_measures_entity_handler;
pub mod entropy_complexity_measurement_handler;
pub mod coupling_cohesion_metrics_handler;
pub mod leiden_community_detection_handler;
// v1.6.1: Ingestion coverage reporting
pub mod ingestion_coverage_folder_handler;

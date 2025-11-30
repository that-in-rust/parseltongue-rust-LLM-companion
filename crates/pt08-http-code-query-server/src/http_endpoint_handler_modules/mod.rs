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

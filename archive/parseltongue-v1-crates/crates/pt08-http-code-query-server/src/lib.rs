//! # pt08-http-code-query-server
//!
//! HTTP server that exposes CozoDB graph database via REST endpoints.
//! Any LLM agent can query codebase architecture without file-based JSON exports.
//!
//! ## 4-Word Naming Convention
//!
//! ALL names follow exactly 4 words for LLM tokenization optimization:
//! - Files: `server_health_check_handler.rs`
//! - Functions: `handle_server_health_check_status()`
//! - Structs: `HttpServerStartupConfig`
//! - Endpoints: `/server-health-check-status`

pub mod command_line_argument_parser;
pub mod file_watcher_integration_service;
pub mod http_server_startup_runner;
pub mod initial_scan;
// v1.5.0: ISGL1 v2 integration complete - re-enabled
pub mod incremental_reindex_core_logic;
pub mod port_selection;
pub mod route_definition_builder_module;
pub mod structured_error_handling_types;
pub mod http_endpoint_handler_modules;
pub mod scope_filter_utilities_module;

// Re-export main types for convenience
pub use command_line_argument_parser::HttpServerStartupConfig;
pub use http_server_startup_runner::{SharedApplicationStateContainer, start_http_server_blocking_loop};
pub use file_watcher_integration_service::{
    FileWatcherIntegrationConfig,
    create_production_watcher_service,
    create_mock_watcher_service,
};
pub use initial_scan::{execute_initial_codebase_scan, InitialScanStatistics};
pub use port_selection::{find_and_bind_port_available, PortSelectionError, ValidatedPortNumber};
pub use route_definition_builder_module::build_complete_router_instance;
pub use structured_error_handling_types::HttpServerErrorTypes;

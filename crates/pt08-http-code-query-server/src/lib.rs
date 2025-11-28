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
pub mod http_server_startup_runner;
pub mod route_definition_builder_module;
pub mod structured_error_handling_types;
pub mod http_endpoint_handler_modules;

// Re-export main types for convenience
pub use command_line_argument_parser::HttpServerStartupConfig;
pub use http_server_startup_runner::SharedApplicationStateContainer;
pub use route_definition_builder_module::build_complete_router_instance;
pub use structured_error_handling_types::HttpServerErrorTypes;

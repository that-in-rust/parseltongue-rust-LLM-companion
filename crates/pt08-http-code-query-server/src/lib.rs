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
pub mod file_watcher_service_module;
pub mod http_server_startup_runner;
pub mod port_selection;
pub mod route_definition_builder_module;
pub mod structured_error_handling_types;
pub mod http_endpoint_handler_modules;
pub mod websocket_streaming_module;
pub mod static_file_embed_module;

#[cfg(test)]
mod static_file_embed_tests;

// Re-export main types for convenience
pub use command_line_argument_parser::HttpServerStartupConfig;
pub use http_server_startup_runner::{SharedApplicationStateContainer, start_http_server_blocking_loop};
pub use port_selection::{find_and_bind_port_available, PortSelectionError, ValidatedPortNumber};
pub use route_definition_builder_module::build_complete_router_instance;
pub use structured_error_handling_types::HttpServerErrorTypes;

// WebSocket streaming re-exports
pub use websocket_streaming_module::{
    WebSocketClientInboundMessageType,
    WebSocketServerOutboundMessageType,
    ConnectionStateDataStruct,
    SharedConnectionStateContainer,
    handle_websocket_diff_stream_upgrade,
};

// File watcher service re-exports
pub use file_watcher_service_module::{
    FileEventKindType,
    RawFileEventDataStruct,
    DebouncedFileChangeEventStruct,
    WatcherConfigurationStruct,
    FileWatcherErrorType,
    FileWatcherServiceStruct,
    PathFilterConfigurationStruct,
    DebouncerServiceStruct,
    create_watcher_for_workspace,
    start_watching_workspace_directory,
    stop_watching_workspace_directory,
    trigger_incremental_reindex_update,
    broadcast_diff_to_subscribers,
};

// Static file embed re-exports (Phase 2.6: rust-embed integration)
pub use static_file_embed_module::{
    serve_root_index_handler,
    serve_static_asset_handler,
    serve_spa_fallback_handler,
    StaticAssetEmbedFolder,
};

//! WebSocket streaming module for real-time diff events
//!
//! # 4-Word Naming: websocket_streaming_module
//!
//! This module provides WebSocket functionality for streaming diff events
//! to connected clients in real-time. It integrates with Phase 2.1 workspace
//! management and Phase 2.2 file watching systems.
//!
//! ## Submodules
//!
//! - `message_types`: Client and server message type definitions
//! - `handler`: WebSocket upgrade and connection management
//! - `connection_state`: Per-connection state tracking
//!
//! ## Requirements Implemented
//!
//! - REQ-WEBSOCKET-001: Connection Establishment
//! - REQ-WEBSOCKET-002: Subscribe to Workspace
//! - REQ-WEBSOCKET-003: Unsubscribe from Workspace
//! - REQ-WEBSOCKET-004: Heartbeat Mechanism
//! - REQ-WEBSOCKET-005: Connection Closure
//! - REQ-WEBSOCKET-006: Diff Started Event
//! - REQ-WEBSOCKET-007: Entity Change Events
//! - REQ-WEBSOCKET-008: Edge Change Events
//! - REQ-WEBSOCKET-009: Diff Completed Event
//! - REQ-WEBSOCKET-010: Message Parsing Errors
//! - REQ-WEBSOCKET-011: Broadcast Error Handling
//! - REQ-WEBSOCKET-012: Multi-Client Subscription
//! - REQ-WEBSOCKET-013: Performance Contract

pub mod message_types;
pub mod handler;
pub mod connection_state;

// Re-export main types for convenience
pub use message_types::{
    WebSocketClientInboundMessageType,
    WebSocketServerOutboundMessageType,
    LineRangeDataStruct,
    DiffSummaryDataPayloadStruct,
};
pub use handler::handle_websocket_diff_stream_upgrade;
pub use connection_state::{
    ConnectionStateDataStruct,
    SharedConnectionStateContainer,
};

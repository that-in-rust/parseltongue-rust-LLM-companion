//! WebSocket message type definitions
//!
//! # 4-Word Naming: message_types_definition_module
//!
//! Defines all message types for WebSocket communication:
//! - Client-to-server (inbound) messages
//! - Server-to-client (outbound) messages
//!
//! ## Requirements Implemented
//!
//! - REQ-WEBSOCKET-002: Subscribe message type
//! - REQ-WEBSOCKET-003: Unsubscribe message type
//! - REQ-WEBSOCKET-004: Ping/Pong message types
//! - REQ-WEBSOCKET-006: DiffStarted event type
//! - REQ-WEBSOCKET-007: Entity event types (added, removed, modified)
//! - REQ-WEBSOCKET-008: Edge event types (added, removed)
//! - REQ-WEBSOCKET-009: DiffCompleted event type
//! - REQ-WEBSOCKET-010: Error event type

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// =============================================================================
// Supporting Data Structures
// =============================================================================

/// Line range data for entity locations
///
/// # 4-Word Name: LineRangeDataStruct
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineRangeDataStruct {
    /// Starting line number (1-indexed)
    pub start: usize,
    /// Ending line number (1-indexed, inclusive)
    pub end: usize,
}

/// Diff summary data payload for completed events
///
/// # 4-Word Name: DiffSummaryDataPayloadStruct
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiffSummaryDataPayloadStruct {
    /// Total entities in "before" state
    pub total_before_count: usize,
    /// Total entities in "after" state
    pub total_after_count: usize,
    /// Count of entities added in diff
    pub added_entity_count: usize,
    /// Count of entities removed in diff
    pub removed_entity_count: usize,
    /// Count of entities modified in diff
    pub modified_entity_count: usize,
    /// Count of unchanged entities
    pub unchanged_entity_count: usize,
    /// Count of relocated entities (moved between files)
    pub relocated_entity_count: usize,
}

// =============================================================================
// Client-to-Server Messages (Inbound)
// =============================================================================

/// Client to server message types
///
/// # 4-Word Name: WebSocketClientInboundMessageType
///
/// These messages are sent from WebSocket clients to the server.
/// Uses tagged enum serialization for the "action" field.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(tag = "action")]
pub enum WebSocketClientInboundMessageType {
    /// Subscribe to workspace diff updates
    ///
    /// Client sends this to start receiving diff events for a workspace.
    #[serde(rename = "subscribe")]
    SubscribeToWorkspaceUpdates {
        /// Workspace identifier to subscribe to
        workspace_id: String,
    },

    /// Unsubscribe from current workspace
    ///
    /// Client sends this to stop receiving diff events.
    #[serde(rename = "unsubscribe")]
    UnsubscribeFromWorkspaceUpdates,

    /// Heartbeat ping request
    ///
    /// Client sends this to keep connection alive and verify server health.
    #[serde(rename = "ping")]
    PingHeartbeatRequest,
}

// =============================================================================
// Server-to-Client Messages (Outbound)
// =============================================================================

/// Server to client message types
///
/// # 4-Word Name: WebSocketServerOutboundMessageType
///
/// These messages are sent from the server to WebSocket clients.
/// Uses tagged enum serialization for the "event" field.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "event")]
pub enum WebSocketServerOutboundMessageType {
    /// Subscription confirmed notification
    ///
    /// Sent after successful subscription to a workspace.
    #[serde(rename = "subscribed")]
    SubscriptionConfirmedNotification {
        /// Workspace identifier that was subscribed to
        workspace_id: String,
        /// Display name of the workspace
        workspace_name: String,
        /// Timestamp when subscription was confirmed
        timestamp: DateTime<Utc>,
    },

    /// Unsubscription confirmed notification
    ///
    /// Sent after successful unsubscription from a workspace.
    #[serde(rename = "unsubscribed")]
    UnsubscriptionConfirmedNotification {
        /// Timestamp when unsubscription was confirmed
        timestamp: DateTime<Utc>,
    },

    /// Heartbeat pong response
    ///
    /// Sent in response to client ping.
    #[serde(rename = "pong")]
    PongHeartbeatResponse {
        /// Timestamp when pong was sent
        timestamp: DateTime<Utc>,
    },

    /// Diff analysis started notification
    ///
    /// Sent when diff analysis begins for a workspace.
    #[serde(rename = "diff_started")]
    DiffAnalysisStartedNotification {
        /// Workspace identifier being analyzed
        workspace_id: String,
        /// Number of files that changed
        files_changed: usize,
        /// What triggered the diff: "file_watcher" or "manual"
        triggered_by: String,
        /// Timestamp when analysis started
        timestamp: DateTime<Utc>,
    },

    /// Entity added event notification
    ///
    /// Sent when a new entity is detected in the codebase.
    #[serde(rename = "entity_added")]
    EntityAddedEventNotification {
        /// Workspace identifier
        workspace_id: String,
        /// Unique key for the entity
        entity_key: String,
        /// Type of entity (function, struct, class, etc.)
        entity_type: String,
        /// File path where entity was added
        file_path: String,
        /// Line range of the entity (optional)
        line_range: Option<LineRangeDataStruct>,
        /// Timestamp of the event
        timestamp: DateTime<Utc>,
    },

    /// Entity removed event notification
    ///
    /// Sent when an entity is removed from the codebase.
    #[serde(rename = "entity_removed")]
    EntityRemovedEventNotification {
        /// Workspace identifier
        workspace_id: String,
        /// Unique key for the entity
        entity_key: String,
        /// Type of entity (function, struct, class, etc.)
        entity_type: String,
        /// File path where entity was located
        file_path: String,
        /// Timestamp of the event
        timestamp: DateTime<Utc>,
    },

    /// Entity modified event notification
    ///
    /// Sent when an existing entity is modified.
    #[serde(rename = "entity_modified")]
    EntityModifiedEventNotification {
        /// Workspace identifier
        workspace_id: String,
        /// Unique key for the entity
        entity_key: String,
        /// Type of entity (function, struct, class, etc.)
        entity_type: String,
        /// File path of the entity
        file_path: String,
        /// Line range before modification (optional)
        before_line_range: Option<LineRangeDataStruct>,
        /// Line range after modification (optional)
        after_line_range: Option<LineRangeDataStruct>,
        /// Timestamp of the event
        timestamp: DateTime<Utc>,
    },

    /// Edge added event notification
    ///
    /// Sent when a new dependency edge is created.
    #[serde(rename = "edge_added")]
    EdgeAddedEventNotification {
        /// Workspace identifier
        workspace_id: String,
        /// Source entity key
        from_entity_key: String,
        /// Target entity key
        to_entity_key: String,
        /// Type of edge (Calls, Uses, Imports, etc.)
        edge_type: String,
        /// Timestamp of the event
        timestamp: DateTime<Utc>,
    },

    /// Edge removed event notification
    ///
    /// Sent when a dependency edge is removed.
    #[serde(rename = "edge_removed")]
    EdgeRemovedEventNotification {
        /// Workspace identifier
        workspace_id: String,
        /// Source entity key
        from_entity_key: String,
        /// Target entity key
        to_entity_key: String,
        /// Type of edge (Calls, Uses, Imports, etc.)
        edge_type: String,
        /// Timestamp of the event
        timestamp: DateTime<Utc>,
    },

    /// Diff analysis completed notification
    ///
    /// Sent when diff analysis completes with summary statistics.
    #[serde(rename = "diff_completed")]
    DiffAnalysisCompletedNotification {
        /// Workspace identifier
        workspace_id: String,
        /// Summary of diff results
        summary: DiffSummaryDataPayloadStruct,
        /// Number of entities in blast radius
        blast_radius_count: usize,
        /// Duration of analysis in milliseconds
        duration_ms: u64,
        /// Timestamp when analysis completed
        timestamp: DateTime<Utc>,
    },

    /// Error occurred notification
    ///
    /// Sent when an error occurs during WebSocket operations.
    #[serde(rename = "error")]
    ErrorOccurredNotification {
        /// Error code (SCREAMING_SNAKE_CASE)
        code: String,
        /// Human-readable error message
        message: String,
        /// Timestamp when error occurred
        timestamp: DateTime<Utc>,
    },
}

// =============================================================================
// Error Code Constants
// =============================================================================

/// Error code: Invalid JSON message format
///
/// # 4-Word Name: ERROR_CODE_INVALID_JSON
pub const ERROR_CODE_INVALID_JSON: &str = "INVALID_JSON_MESSAGE";

/// Error code: Unknown action type
///
/// # 4-Word Name: ERROR_CODE_UNKNOWN_ACTION
pub const ERROR_CODE_UNKNOWN_ACTION: &str = "UNKNOWN_ACTION_TYPE";

/// Error code: Missing workspace ID
///
/// # 4-Word Name: ERROR_CODE_MISSING_WORKSPACE
pub const ERROR_CODE_MISSING_WORKSPACE: &str = "MISSING_WORKSPACE_ID";

/// Error code: Workspace not found
///
/// # 4-Word Name: ERROR_CODE_WORKSPACE_NOTFOUND
pub const ERROR_CODE_WORKSPACE_NOTFOUND: &str = "WORKSPACE_NOT_FOUND";

/// Error code: Workspace not watching
///
/// # 4-Word Name: ERROR_CODE_NOT_WATCHING
pub const ERROR_CODE_NOT_WATCHING: &str = "WORKSPACE_NOT_WATCHING";

/// Error code: Already subscribed
///
/// # 4-Word Name: ERROR_CODE_ALREADY_SUBSCRIBED
pub const ERROR_CODE_ALREADY_SUBSCRIBED: &str = "ALREADY_SUBSCRIBED";

/// Error code: Not subscribed
///
/// # 4-Word Name: ERROR_CODE_NOT_SUBSCRIBED
pub const ERROR_CODE_NOT_SUBSCRIBED: &str = "NOT_SUBSCRIBED";

/// Error code: Subscription limit exceeded
///
/// # 4-Word Name: ERROR_CODE_SUBSCRIPTION_LIMIT
pub const ERROR_CODE_SUBSCRIPTION_LIMIT: &str = "SUBSCRIPTION_LIMIT_EXCEEDED";

/// Error code: Internal server error
///
/// # 4-Word Name: ERROR_CODE_INTERNAL_ERROR
pub const ERROR_CODE_INTERNAL_ERROR: &str = "INTERNAL_ERROR";

/// Error code: Connection timeout
///
/// # 4-Word Name: ERROR_CODE_CONNECTION_TIMEOUT
pub const ERROR_CODE_CONNECTION_TIMEOUT: &str = "CONNECTION_TIMEOUT";

/// Error code: Invalid message type
///
/// # 4-Word Name: ERROR_CODE_INVALID_TYPE
pub const ERROR_CODE_INVALID_TYPE: &str = "INVALID_MESSAGE_TYPE";

// =============================================================================
// Helper Functions
// =============================================================================

/// Create error notification with timestamp
///
/// # 4-Word Name: create_error_notification_message
pub fn create_error_notification_message(
    code: &str,
    message: &str,
) -> WebSocketServerOutboundMessageType {
    WebSocketServerOutboundMessageType::ErrorOccurredNotification {
        code: code.to_string(),
        message: message.to_string(),
        timestamp: Utc::now(),
    }
}

/// Create pong response with timestamp
///
/// # 4-Word Name: create_pong_response_message
pub fn create_pong_response_message() -> WebSocketServerOutboundMessageType {
    WebSocketServerOutboundMessageType::PongHeartbeatResponse {
        timestamp: Utc::now(),
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test client message deserialization for subscribe
    #[test]
    fn test_subscribe_message_deserializes_correctly() {
        let json = r#"{"action": "subscribe", "workspace_id": "ws_123"}"#;
        let msg: WebSocketClientInboundMessageType = serde_json::from_str(json).unwrap();

        match msg {
            WebSocketClientInboundMessageType::SubscribeToWorkspaceUpdates { workspace_id } => {
                assert_eq!(workspace_id, "ws_123");
            }
            _ => panic!("Expected SubscribeToWorkspaceUpdates"),
        }
    }

    /// Test client message deserialization for unsubscribe
    #[test]
    fn test_unsubscribe_message_deserializes_correctly() {
        let json = r#"{"action": "unsubscribe"}"#;
        let msg: WebSocketClientInboundMessageType = serde_json::from_str(json).unwrap();

        assert!(matches!(
            msg,
            WebSocketClientInboundMessageType::UnsubscribeFromWorkspaceUpdates
        ));
    }

    /// Test client message deserialization for ping
    #[test]
    fn test_ping_message_deserializes_correctly() {
        let json = r#"{"action": "ping"}"#;
        let msg: WebSocketClientInboundMessageType = serde_json::from_str(json).unwrap();

        assert!(matches!(
            msg,
            WebSocketClientInboundMessageType::PingHeartbeatRequest
        ));
    }

    /// Test server message serialization for subscribed
    #[test]
    fn test_subscribed_message_serializes_correctly() {
        let msg = WebSocketServerOutboundMessageType::SubscriptionConfirmedNotification {
            workspace_id: "ws_123".to_string(),
            workspace_name: "Test Project".to_string(),
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""event":"subscribed""#));
        assert!(json.contains(r#""workspace_id":"ws_123""#));
    }

    /// Test server message serialization for error
    #[test]
    fn test_error_message_serializes_correctly() {
        let msg = create_error_notification_message(
            ERROR_CODE_WORKSPACE_NOTFOUND,
            "Workspace not found",
        );

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""event":"error""#));
        assert!(json.contains(r#""code":"WORKSPACE_NOT_FOUND""#));
    }

    /// Test pong response creation
    #[test]
    fn test_pong_response_creates_correctly() {
        let msg = create_pong_response_message();

        match msg {
            WebSocketServerOutboundMessageType::PongHeartbeatResponse { timestamp } => {
                assert!(timestamp <= Utc::now());
            }
            _ => panic!("Expected PongHeartbeatResponse"),
        }
    }

    /// Test diff summary serialization
    #[test]
    fn test_diff_summary_serializes_correctly() {
        let summary = DiffSummaryDataPayloadStruct {
            total_before_count: 100,
            total_after_count: 105,
            added_entity_count: 8,
            removed_entity_count: 3,
            modified_entity_count: 5,
            unchanged_entity_count: 89,
            relocated_entity_count: 0,
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains(r#""total_before_count":100"#));
        assert!(json.contains(r#""added_entity_count":8"#));
    }

    /// Test line range serialization
    #[test]
    fn test_line_range_serializes_correctly() {
        let range = LineRangeDataStruct { start: 10, end: 25 };
        let json = serde_json::to_string(&range).unwrap();
        assert!(json.contains(r#""start":10"#));
        assert!(json.contains(r#""end":25"#));
    }
}

//! WebSocket connection state management
//!
//! # 4-Word Naming: connection_state_management_module
//!
//! Manages per-connection state for WebSocket connections:
//! - Subscription status
//! - Activity timestamps
//! - Connection identifiers
//!
//! ## Requirements Implemented
//!
//! - REQ-WEBSOCKET-001.3: Connection state initialization
//! - REQ-WEBSOCKET-002.6: Subscription registration
//! - REQ-WEBSOCKET-003.3: Unsubscribe cleanup
//! - REQ-WEBSOCKET-004: Activity timestamp tracking

use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// =============================================================================
// Connection State
// =============================================================================

/// State data for a single WebSocket connection
///
/// # 4-Word Name: ConnectionStateDataStruct
///
/// Tracks the state of a single WebSocket connection including:
/// - Current subscription status
/// - Activity timestamps for timeout detection
/// - Unique connection identifier
/// - Message sequence counter for debugging
#[derive(Debug, Clone)]
pub struct ConnectionStateDataStruct {
    /// Currently subscribed workspace (None if not subscribed)
    ///
    /// A connection can only subscribe to one workspace at a time.
    pub subscribed_workspace_id: Option<String>,

    /// Timestamp of last activity (ping, message, etc.)
    ///
    /// Used for connection timeout detection (60 second timeout).
    pub last_activity_timestamp: DateTime<Utc>,

    /// Unique connection identifier for logging
    ///
    /// Generated using UUID v4 on connection establishment.
    pub connection_unique_identifier: String,

    /// Message sequence counter for debugging
    ///
    /// Incremented for each message processed.
    pub message_sequence_counter: u64,
}

impl ConnectionStateDataStruct {
    /// Create new connection state with defaults
    ///
    /// # 4-Word Name: create_new_connection_state
    ///
    /// Initializes a new connection state with:
    /// - No subscription
    /// - Current timestamp as last activity
    /// - Generated UUID as connection identifier
    /// - Zero message counter
    pub fn create_new_connection_state() -> Self {
        Self {
            subscribed_workspace_id: None,
            last_activity_timestamp: Utc::now(),
            connection_unique_identifier: Uuid::new_v4().to_string(),
            message_sequence_counter: 0,
        }
    }

    /// Update last activity timestamp
    ///
    /// # 4-Word Name: update_last_activity_timestamp
    ///
    /// Called whenever any message is received or processed.
    pub fn update_last_activity_timestamp(&mut self) {
        self.last_activity_timestamp = Utc::now();
    }

    /// Increment message sequence counter
    ///
    /// # 4-Word Name: increment_message_sequence_counter
    ///
    /// Returns the new counter value after incrementing.
    pub fn increment_message_sequence_counter(&mut self) -> u64 {
        self.message_sequence_counter += 1;
        self.message_sequence_counter
    }

    /// Check if subscribed to workspace
    ///
    /// # 4-Word Name: is_subscribed_to_workspace
    pub fn is_subscribed_to_workspace(&self) -> bool {
        self.subscribed_workspace_id.is_some()
    }

    /// Set workspace subscription status
    ///
    /// # 4-Word Name: set_workspace_subscription_status
    pub fn set_workspace_subscription_status(&mut self, workspace_id: Option<String>) {
        self.subscribed_workspace_id = workspace_id;
        self.update_last_activity_timestamp();
    }

    /// Get seconds since last activity
    ///
    /// # 4-Word Name: get_seconds_since_activity
    ///
    /// Used for timeout detection.
    pub fn get_seconds_since_activity(&self) -> i64 {
        let now = Utc::now();
        (now - self.last_activity_timestamp).num_seconds()
    }

    /// Check if connection has timed out
    ///
    /// # 4-Word Name: is_connection_timed_out
    ///
    /// Returns true if more than 60 seconds have passed since last activity.
    pub fn is_connection_timed_out(&self) -> bool {
        self.get_seconds_since_activity() > 60
    }
}

impl Default for ConnectionStateDataStruct {
    fn default() -> Self {
        Self::create_new_connection_state()
    }
}

// =============================================================================
// Type Aliases
// =============================================================================

/// Shared connection state container for thread-safe access
///
/// # 4-Word Name: SharedConnectionStateContainer
///
/// Allows multiple tasks to access and modify connection state safely.
pub type SharedConnectionStateContainer = Arc<RwLock<ConnectionStateDataStruct>>;

/// Create a new shared connection state
///
/// # 4-Word Name: create_shared_connection_state
pub fn create_shared_connection_state() -> SharedConnectionStateContainer {
    Arc::new(RwLock::new(ConnectionStateDataStruct::create_new_connection_state()))
}

// =============================================================================
// Connection Timeout Constants
// =============================================================================

/// Connection timeout in seconds
///
/// # 4-Word Name: CONNECTION_TIMEOUT_SECONDS_VALUE
pub const CONNECTION_TIMEOUT_SECONDS_VALUE: u64 = 60;

/// Protocol ping interval in seconds
///
/// # 4-Word Name: PROTOCOL_PING_INTERVAL_SECONDS
pub const PROTOCOL_PING_INTERVAL_SECONDS: u64 = 30;

/// Maximum connections per workspace
///
/// # 4-Word Name: MAX_CONNECTIONS_PER_WORKSPACE
pub const MAX_CONNECTIONS_PER_WORKSPACE: usize = 100;

/// Broadcast timeout in milliseconds
///
/// # 4-Word Name: BROADCAST_TIMEOUT_MS_VALUE
pub const BROADCAST_TIMEOUT_MS_VALUE: u64 = 5000;

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test connection state initialization
    #[test]
    fn test_create_new_connection_state() {
        let state = ConnectionStateDataStruct::create_new_connection_state();

        assert!(state.subscribed_workspace_id.is_none());
        assert_eq!(state.message_sequence_counter, 0);
        assert!(!state.connection_unique_identifier.is_empty());
    }

    /// Test is_subscribed_to_workspace
    #[test]
    fn test_is_subscribed_to_workspace() {
        let mut state = ConnectionStateDataStruct::create_new_connection_state();

        assert!(!state.is_subscribed_to_workspace());

        state.subscribed_workspace_id = Some("ws_123".to_string());
        assert!(state.is_subscribed_to_workspace());
    }

    /// Test set_workspace_subscription_status
    #[test]
    fn test_set_workspace_subscription_status() {
        let mut state = ConnectionStateDataStruct::create_new_connection_state();

        state.set_workspace_subscription_status(Some("ws_123".to_string()));
        assert_eq!(state.subscribed_workspace_id, Some("ws_123".to_string()));

        state.set_workspace_subscription_status(None);
        assert!(state.subscribed_workspace_id.is_none());
    }

    /// Test increment_message_sequence_counter
    #[test]
    fn test_increment_message_sequence_counter() {
        let mut state = ConnectionStateDataStruct::create_new_connection_state();

        assert_eq!(state.increment_message_sequence_counter(), 1);
        assert_eq!(state.increment_message_sequence_counter(), 2);
        assert_eq!(state.increment_message_sequence_counter(), 3);
    }

    /// Test unique identifiers are unique
    #[test]
    fn test_connection_identifiers_unique() {
        let state1 = ConnectionStateDataStruct::create_new_connection_state();
        let state2 = ConnectionStateDataStruct::create_new_connection_state();

        assert_ne!(
            state1.connection_unique_identifier,
            state2.connection_unique_identifier
        );
    }

    /// Test shared connection state creation
    #[test]
    fn test_create_shared_connection_state() {
        let shared = create_shared_connection_state();

        // Should be able to read without blocking
        let state = shared.try_read().unwrap();
        assert!(state.subscribed_workspace_id.is_none());
    }

    /// Test constants are set correctly
    #[test]
    fn test_timeout_constants() {
        assert_eq!(CONNECTION_TIMEOUT_SECONDS_VALUE, 60);
        assert_eq!(PROTOCOL_PING_INTERVAL_SECONDS, 30);
        assert_eq!(MAX_CONNECTIONS_PER_WORKSPACE, 100);
        assert_eq!(BROADCAST_TIMEOUT_MS_VALUE, 5000);
    }
}

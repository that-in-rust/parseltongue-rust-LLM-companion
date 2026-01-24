# WebSocket Endpoint Specification: Real-Time Diff Streaming

## Phase 2.2 - WebSocket Streaming System

**Document Version**: 1.0.0
**Created**: 2026-01-23
**Status**: Specification Complete
**Phase**: 2.2 - WebSocket Streaming Backend
**Dependency**: Phase 2.1 (Workspace Management) Complete

---

## Overview

### Problem Statement

Developers need real-time visibility into how their code changes affect the dependency graph. Currently, analyzing impact requires manual re-indexing and comparing database snapshots. This creates friction when:

1. Making incremental changes during development
2. Understanding blast radius of modifications in real-time
3. Visualizing dependency changes as they occur
4. Collaborating with team members viewing the same workspace

### Solution

A WebSocket endpoint that provides real-time streaming of diff events:

| Endpoint | Protocol | Purpose |
|----------|----------|---------|
| `/websocket-diff-stream` | WebSocket (RFC 6455) | Real-time diff event streaming |

The WebSocket connection enables:
- Subscribe to workspace updates
- Receive granular diff events (entity/edge added/removed/modified)
- Track diff lifecycle (started/completed)
- Heartbeat mechanism for connection health
- Multi-client broadcasting per workspace

### Integration with Phase 2.1

This endpoint integrates with the workspace management system:
- Uses `workspace_identifier_value` from Phase 2.1 for subscription
- Requires workspace to exist and have `watch_enabled_flag_status: true`
- Leverages `SharedWorkspaceStateContainer` for connection tracking
- Triggered by `FileWatcherServiceImpl` file change events

---

## WebSocket Message Types

### Client-to-Server Messages (Inbound)

```rust
/// Client to server message types
/// # 4-Word Name: WebSocketClientInboundMessageType
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action")]
pub enum WebSocketClientInboundMessageType {
    /// Subscribe to workspace diff updates
    #[serde(rename = "subscribe")]
    SubscribeToWorkspaceUpdates {
        workspace_id: String,
    },
    /// Unsubscribe from current workspace
    #[serde(rename = "unsubscribe")]
    UnsubscribeFromWorkspaceUpdates,
    /// Heartbeat ping
    #[serde(rename = "ping")]
    PingHeartbeatRequest,
}
```

### Server-to-Client Messages (Outbound)

```rust
/// Server to client message types
/// # 4-Word Name: WebSocketServerOutboundMessageType
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event")]
pub enum WebSocketServerOutboundMessageType {
    /// Subscription confirmed
    #[serde(rename = "subscribed")]
    SubscriptionConfirmedNotification {
        workspace_id: String,
        workspace_name: String,
        timestamp: DateTime<Utc>,
    },
    /// Unsubscription confirmed
    #[serde(rename = "unsubscribed")]
    UnsubscriptionConfirmedNotification {
        timestamp: DateTime<Utc>,
    },
    /// Diff analysis started
    #[serde(rename = "diff_started")]
    DiffAnalysisStartedNotification {
        workspace_id: String,
        files_changed: usize,
        triggered_by: String,  // "file_watcher" | "manual"
        timestamp: DateTime<Utc>,
    },
    /// Entity added to codebase
    #[serde(rename = "entity_added")]
    EntityAddedEventNotification {
        workspace_id: String,
        entity_key: String,
        entity_type: String,
        file_path: String,
        line_range: Option<LineRangeData>,
        timestamp: DateTime<Utc>,
    },
    /// Entity removed from codebase
    #[serde(rename = "entity_removed")]
    EntityRemovedEventNotification {
        workspace_id: String,
        entity_key: String,
        entity_type: String,
        file_path: String,
        timestamp: DateTime<Utc>,
    },
    /// Entity modified in codebase
    #[serde(rename = "entity_modified")]
    EntityModifiedEventNotification {
        workspace_id: String,
        entity_key: String,
        entity_type: String,
        file_path: String,
        before_line_range: Option<LineRangeData>,
        after_line_range: Option<LineRangeData>,
        timestamp: DateTime<Utc>,
    },
    /// Edge (dependency) added
    #[serde(rename = "edge_added")]
    EdgeAddedEventNotification {
        workspace_id: String,
        from_entity_key: String,
        to_entity_key: String,
        edge_type: String,
        timestamp: DateTime<Utc>,
    },
    /// Edge (dependency) removed
    #[serde(rename = "edge_removed")]
    EdgeRemovedEventNotification {
        workspace_id: String,
        from_entity_key: String,
        to_entity_key: String,
        edge_type: String,
        timestamp: DateTime<Utc>,
    },
    /// Diff analysis completed
    #[serde(rename = "diff_completed")]
    DiffAnalysisCompletedNotification {
        workspace_id: String,
        summary: DiffSummaryDataPayload,
        blast_radius_count: usize,
        duration_ms: u64,
        timestamp: DateTime<Utc>,
    },
    /// Error occurred
    #[serde(rename = "error")]
    ErrorOccurredNotification {
        code: String,
        message: String,
        timestamp: DateTime<Utc>,
    },
    /// Heartbeat response
    #[serde(rename = "pong")]
    PongHeartbeatResponse {
        timestamp: DateTime<Utc>,
    },
}
```

---

## Error Codes Reference

| Code | Description | Client Action |
|------|-------------|---------------|
| `INVALID_JSON_MESSAGE` | Client message is not valid JSON | Fix message format |
| `UNKNOWN_ACTION_TYPE` | Action field not recognized | Use valid action |
| `MISSING_WORKSPACE_ID` | Subscribe missing workspace_id | Include workspace_id |
| `WORKSPACE_NOT_FOUND` | Workspace does not exist | Check workspace ID |
| `WORKSPACE_NOT_WATCHING` | Workspace has watching disabled | Enable watching first |
| `ALREADY_SUBSCRIBED` | Client already subscribed to a workspace | Unsubscribe first |
| `NOT_SUBSCRIBED` | Unsubscribe called without subscription | Subscribe first |
| `SUBSCRIPTION_LIMIT_EXCEEDED` | Too many connections to workspace | Wait for slot |
| `INTERNAL_ERROR` | Server-side error | Retry or report |
| `CONNECTION_TIMEOUT` | No ping received within timeout | Reconnect |

---

# Connection Lifecycle Requirements

## REQ-WEBSOCKET-001: Connection Establishment

### Problem Statement

Clients must be able to establish WebSocket connections to the server using the standard HTTP upgrade mechanism. The server must validate the upgrade request and establish a bidirectional communication channel.

### Specification

#### REQ-WEBSOCKET-001.1: HTTP Upgrade Request

```
WHEN client sends GET request to /websocket-diff-stream
  WITH header "Upgrade: websocket"
  AND header "Connection: Upgrade"
  AND header "Sec-WebSocket-Version: 13"
  AND header "Sec-WebSocket-Key: {base64_key}"
THEN SHALL respond with HTTP 101 Switching Protocols
  AND SHALL include header "Upgrade: websocket"
  AND SHALL include header "Connection: Upgrade"
  AND SHALL include header "Sec-WebSocket-Accept: {computed_accept}"
  AND SHALL establish WebSocket connection
  AND SHALL complete handshake within 1000ms
```

#### REQ-WEBSOCKET-001.2: Missing Upgrade Headers

```
WHEN client sends GET request to /websocket-diff-stream
  WITH missing or invalid WebSocket upgrade headers
THEN SHALL return HTTP 400 Bad Request
  AND SHALL return JSON body:
    {
      "error": "WebSocket upgrade required",
      "code": "UPGRADE_REQUIRED"
    }
  AND SHALL NOT establish connection
```

#### REQ-WEBSOCKET-001.3: Connection State Initialization

```
WHEN WebSocket connection is established
THEN SHALL initialize connection state:
  - subscription_status = None
  - last_ping_timestamp = now()
  - message_sequence_counter = 0
  AND SHALL start heartbeat monitoring
  AND SHALL NOT send any messages until client subscribes
```

#### REQ-WEBSOCKET-001.4: Connection Limit Per Workspace

```
WHEN client establishes WebSocket connection
  WITH intent to subscribe to workspace
  AND workspace already has 100 connected clients
THEN SHALL allow connection establishment
  AND SHALL reject subscription with error:
    {
      "event": "error",
      "code": "SUBSCRIPTION_LIMIT_EXCEEDED",
      "message": "Maximum 100 connections per workspace"
    }
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_websocket_001_tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tokio_tungstenite::tungstenite::Message;
    use futures_util::{SinkExt, StreamExt};

    /// REQ-WEBSOCKET-001.1: Valid upgrade request succeeds
    #[tokio::test]
    async fn test_valid_websocket_upgrade_succeeds() {
        // GIVEN a configured server with WebSocket endpoint
        let addr = start_test_server_with_websocket().await;

        // WHEN client connects via WebSocket
        let url = format!("ws://{}/websocket-diff-stream", addr);
        let result = tokio_tungstenite::connect_async(&url).await;

        // THEN connection should succeed
        assert!(result.is_ok(), "WebSocket connection should succeed");
    }

    /// REQ-WEBSOCKET-001.2: Missing upgrade headers returns 400
    #[tokio::test]
    async fn test_missing_upgrade_headers_returns_400() {
        // GIVEN a configured server
        let addr = start_test_server_with_websocket().await;
        let client = reqwest::Client::new();

        // WHEN client sends regular GET without WebSocket headers
        let response = client
            .get(format!("http://{}/websocket-diff-stream", addr))
            .send()
            .await
            .unwrap();

        // THEN should return 400 Bad Request
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// REQ-WEBSOCKET-001.3: Connection initializes without subscription
    #[tokio::test]
    async fn test_connection_initializes_without_subscription() {
        // GIVEN a WebSocket connection
        let (mut ws_stream, _) = connect_to_test_websocket().await;

        // WHEN waiting briefly without subscribing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // THEN no messages should be received (connection is idle)
        let result = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            ws_stream.next()
        ).await;

        assert!(result.is_err(), "Should timeout - no messages without subscription");
    }
}
```

### Acceptance Criteria

- [ ] Valid WebSocket upgrade request returns 101
- [ ] Missing upgrade headers return 400
- [ ] Connection state initializes correctly
- [ ] Handshake completes within 1000ms
- [ ] Connection limit (100) is enforced per workspace

---

## REQ-WEBSOCKET-002: Subscribe to Workspace

### Problem Statement

After establishing a connection, clients must subscribe to a specific workspace to receive diff events. The subscription must validate the workspace exists and has watching enabled.

### Specification

#### REQ-WEBSOCKET-002.1: Valid Subscribe Request

```
WHEN connected client sends message:
    {
      "action": "subscribe",
      "workspace_id": "ws_20260123_143000_abc123"
    }
  WITH workspace existing in system
  AND workspace having watch_enabled_flag_status: true
THEN SHALL register client for workspace broadcasts
  AND SHALL respond with:
    {
      "event": "subscribed",
      "workspace_id": "ws_20260123_143000_abc123",
      "workspace_name": "My Project",
      "timestamp": "2026-01-23T14:30:00Z"
    }
  AND SHALL complete subscription within 100ms
```

#### REQ-WEBSOCKET-002.2: Subscribe Without workspace_id

```
WHEN connected client sends message:
    {
      "action": "subscribe"
    }
THEN SHALL respond with error:
    {
      "event": "error",
      "code": "MISSING_WORKSPACE_ID",
      "message": "workspace_id is required for subscription",
      "timestamp": "..."
    }
  AND SHALL NOT register subscription
```

#### REQ-WEBSOCKET-002.3: Subscribe to Non-Existent Workspace

```
WHEN connected client sends message:
    {
      "action": "subscribe",
      "workspace_id": "ws_nonexistent_123456"
    }
  WITH workspace_id not existing in system
THEN SHALL respond with error:
    {
      "event": "error",
      "code": "WORKSPACE_NOT_FOUND",
      "message": "Workspace not found: ws_nonexistent_123456",
      "timestamp": "..."
    }
  AND SHALL NOT register subscription
```

#### REQ-WEBSOCKET-002.4: Subscribe to Workspace with Watching Disabled

```
WHEN connected client sends message:
    {
      "action": "subscribe",
      "workspace_id": "ws_20260123_143000_abc123"
    }
  WITH workspace having watch_enabled_flag_status: false
THEN SHALL respond with error:
    {
      "event": "error",
      "code": "WORKSPACE_NOT_WATCHING",
      "message": "File watching is disabled for workspace. Enable via /workspace-watch-toggle",
      "timestamp": "..."
    }
  AND SHALL NOT register subscription
```

#### REQ-WEBSOCKET-002.5: Already Subscribed Client

```
WHEN connected client with existing subscription sends:
    {
      "action": "subscribe",
      "workspace_id": "ws_different_workspace"
    }
THEN SHALL respond with error:
    {
      "event": "error",
      "code": "ALREADY_SUBSCRIBED",
      "message": "Already subscribed to workspace. Unsubscribe first.",
      "timestamp": "..."
    }
  AND SHALL maintain existing subscription
```

#### REQ-WEBSOCKET-002.6: Subscription Registration in State

```
WHEN subscription is successful
THEN SHALL add connection sender to SharedWorkspaceStateContainer.ws_connections
  AND SHALL increment workspace connection count
  AND client SHALL receive all subsequent diff events for workspace
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_websocket_002_tests {
    use serde_json::{json, Value};
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    /// REQ-WEBSOCKET-002.1: Valid subscription succeeds
    #[tokio::test]
    async fn test_valid_subscription_succeeds() {
        // GIVEN a WebSocket connection and existing workspace with watching enabled
        let (mut ws_stream, workspace_id) = setup_websocket_with_watched_workspace().await;

        // WHEN client subscribes
        let subscribe_msg = json!({
            "action": "subscribe",
            "workspace_id": workspace_id
        });
        ws_stream.send(Message::Text(subscribe_msg.to_string())).await.unwrap();

        // THEN should receive subscribed confirmation
        let response = ws_stream.next().await.unwrap().unwrap();
        let response_json: Value = serde_json::from_str(&response.to_text().unwrap()).unwrap();

        assert_eq!(response_json["event"], "subscribed");
        assert_eq!(response_json["workspace_id"], workspace_id);
        assert!(response_json.get("workspace_name").is_some());
        assert!(response_json.get("timestamp").is_some());
    }

    /// REQ-WEBSOCKET-002.2: Missing workspace_id returns error
    #[tokio::test]
    async fn test_missing_workspace_id_returns_error() {
        // GIVEN a WebSocket connection
        let (mut ws_stream, _) = setup_websocket_connection().await;

        // WHEN client subscribes without workspace_id
        let subscribe_msg = json!({
            "action": "subscribe"
        });
        ws_stream.send(Message::Text(subscribe_msg.to_string())).await.unwrap();

        // THEN should receive error
        let response = ws_stream.next().await.unwrap().unwrap();
        let response_json: Value = serde_json::from_str(&response.to_text().unwrap()).unwrap();

        assert_eq!(response_json["event"], "error");
        assert_eq!(response_json["code"], "MISSING_WORKSPACE_ID");
    }

    /// REQ-WEBSOCKET-002.3: Non-existent workspace returns error
    #[tokio::test]
    async fn test_nonexistent_workspace_returns_error() {
        // GIVEN a WebSocket connection
        let (mut ws_stream, _) = setup_websocket_connection().await;

        // WHEN client subscribes to non-existent workspace
        let subscribe_msg = json!({
            "action": "subscribe",
            "workspace_id": "ws_nonexistent_000000_000000"
        });
        ws_stream.send(Message::Text(subscribe_msg.to_string())).await.unwrap();

        // THEN should receive error
        let response = ws_stream.next().await.unwrap().unwrap();
        let response_json: Value = serde_json::from_str(&response.to_text().unwrap()).unwrap();

        assert_eq!(response_json["event"], "error");
        assert_eq!(response_json["code"], "WORKSPACE_NOT_FOUND");
    }

    /// REQ-WEBSOCKET-002.4: Workspace with watching disabled returns error
    #[tokio::test]
    async fn test_workspace_watching_disabled_returns_error() {
        // GIVEN a WebSocket connection and workspace with watching disabled
        let (mut ws_stream, workspace_id) = setup_websocket_with_unwatched_workspace().await;

        // WHEN client subscribes
        let subscribe_msg = json!({
            "action": "subscribe",
            "workspace_id": workspace_id
        });
        ws_stream.send(Message::Text(subscribe_msg.to_string())).await.unwrap();

        // THEN should receive error
        let response = ws_stream.next().await.unwrap().unwrap();
        let response_json: Value = serde_json::from_str(&response.to_text().unwrap()).unwrap();

        assert_eq!(response_json["event"], "error");
        assert_eq!(response_json["code"], "WORKSPACE_NOT_WATCHING");
    }

    /// REQ-WEBSOCKET-002.5: Already subscribed returns error
    #[tokio::test]
    async fn test_already_subscribed_returns_error() {
        // GIVEN a subscribed WebSocket connection
        let (mut ws_stream, workspace_id) = setup_subscribed_websocket().await;

        // WHEN client tries to subscribe again
        let subscribe_msg = json!({
            "action": "subscribe",
            "workspace_id": workspace_id
        });
        ws_stream.send(Message::Text(subscribe_msg.to_string())).await.unwrap();

        // THEN should receive error
        let response = ws_stream.next().await.unwrap().unwrap();
        let response_json: Value = serde_json::from_str(&response.to_text().unwrap()).unwrap();

        assert_eq!(response_json["event"], "error");
        assert_eq!(response_json["code"], "ALREADY_SUBSCRIBED");
    }
}
```

### Acceptance Criteria

- [ ] Valid subscription returns "subscribed" event with workspace details
- [ ] Missing workspace_id returns MISSING_WORKSPACE_ID error
- [ ] Non-existent workspace returns WORKSPACE_NOT_FOUND error
- [ ] Workspace with watching disabled returns WORKSPACE_NOT_WATCHING error
- [ ] Already subscribed client returns ALREADY_SUBSCRIBED error
- [ ] Subscription completes within 100ms

---

## REQ-WEBSOCKET-003: Unsubscribe from Workspace

### Problem Statement

Clients must be able to cleanly unsubscribe from a workspace to stop receiving diff events without closing the connection.

### Specification

#### REQ-WEBSOCKET-003.1: Valid Unsubscribe Request

```
WHEN subscribed client sends message:
    {
      "action": "unsubscribe"
    }
THEN SHALL remove client from workspace broadcasts
  AND SHALL respond with:
    {
      "event": "unsubscribed",
      "timestamp": "2026-01-23T14:35:00Z"
    }
  AND client SHALL stop receiving diff events
  AND connection SHALL remain open
```

#### REQ-WEBSOCKET-003.2: Unsubscribe When Not Subscribed

```
WHEN client without subscription sends message:
    {
      "action": "unsubscribe"
    }
THEN SHALL respond with error:
    {
      "event": "error",
      "code": "NOT_SUBSCRIBED",
      "message": "No active subscription to unsubscribe from",
      "timestamp": "..."
    }
```

#### REQ-WEBSOCKET-003.3: Unsubscribe Cleanup

```
WHEN unsubscribe is successful
THEN SHALL remove connection sender from SharedWorkspaceStateContainer.ws_connections
  AND SHALL decrement workspace connection count
  AND client MAY subscribe to different workspace
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_websocket_003_tests {
    use serde_json::{json, Value};
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    /// REQ-WEBSOCKET-003.1: Valid unsubscribe succeeds
    #[tokio::test]
    async fn test_valid_unsubscribe_succeeds() {
        // GIVEN a subscribed WebSocket connection
        let (mut ws_stream, _) = setup_subscribed_websocket().await;

        // WHEN client unsubscribes
        let unsubscribe_msg = json!({
            "action": "unsubscribe"
        });
        ws_stream.send(Message::Text(unsubscribe_msg.to_string())).await.unwrap();

        // THEN should receive unsubscribed confirmation
        let response = ws_stream.next().await.unwrap().unwrap();
        let response_json: Value = serde_json::from_str(&response.to_text().unwrap()).unwrap();

        assert_eq!(response_json["event"], "unsubscribed");
        assert!(response_json.get("timestamp").is_some());
    }

    /// REQ-WEBSOCKET-003.2: Unsubscribe when not subscribed returns error
    #[tokio::test]
    async fn test_unsubscribe_not_subscribed_returns_error() {
        // GIVEN a WebSocket connection without subscription
        let (mut ws_stream, _) = setup_websocket_connection().await;

        // WHEN client unsubscribes
        let unsubscribe_msg = json!({
            "action": "unsubscribe"
        });
        ws_stream.send(Message::Text(unsubscribe_msg.to_string())).await.unwrap();

        // THEN should receive error
        let response = ws_stream.next().await.unwrap().unwrap();
        let response_json: Value = serde_json::from_str(&response.to_text().unwrap()).unwrap();

        assert_eq!(response_json["event"], "error");
        assert_eq!(response_json["code"], "NOT_SUBSCRIBED");
    }

    /// REQ-WEBSOCKET-003.3: Unsubscribe allows re-subscription
    #[tokio::test]
    async fn test_unsubscribe_allows_resubscription() {
        // GIVEN a subscribed WebSocket connection
        let (mut ws_stream, workspace_id) = setup_subscribed_websocket().await;

        // WHEN client unsubscribes
        let unsubscribe_msg = json!({ "action": "unsubscribe" });
        ws_stream.send(Message::Text(unsubscribe_msg.to_string())).await.unwrap();
        let _ = ws_stream.next().await; // consume unsubscribed response

        // AND subscribes again
        let subscribe_msg = json!({
            "action": "subscribe",
            "workspace_id": workspace_id
        });
        ws_stream.send(Message::Text(subscribe_msg.to_string())).await.unwrap();

        // THEN should succeed
        let response = ws_stream.next().await.unwrap().unwrap();
        let response_json: Value = serde_json::from_str(&response.to_text().unwrap()).unwrap();

        assert_eq!(response_json["event"], "subscribed");
    }
}
```

### Acceptance Criteria

- [ ] Valid unsubscribe returns "unsubscribed" event
- [ ] Unsubscribe when not subscribed returns NOT_SUBSCRIBED error
- [ ] Connection remains open after unsubscribe
- [ ] Client can re-subscribe after unsubscribe
- [ ] Workspace connection count is decremented

---

## REQ-WEBSOCKET-004: Heartbeat Mechanism

### Problem Statement

The server and client need a heartbeat mechanism to detect stale connections and maintain connection health.

### Specification

#### REQ-WEBSOCKET-004.1: Client Ping Request

```
WHEN client sends message:
    {
      "action": "ping"
    }
THEN SHALL respond immediately with:
    {
      "event": "pong",
      "timestamp": "2026-01-23T14:30:00Z"
    }
  AND SHALL update last_activity_timestamp for connection
  AND SHALL respond within 50ms
```

#### REQ-WEBSOCKET-004.2: Connection Timeout

```
WHEN server detects no ping from client
  WITH duration exceeding 60000ms (60 seconds)
THEN SHALL send error:
    {
      "event": "error",
      "code": "CONNECTION_TIMEOUT",
      "message": "Connection timeout - no heartbeat received",
      "timestamp": "..."
    }
  AND SHALL close WebSocket connection
  AND SHALL cleanup connection state
```

#### REQ-WEBSOCKET-004.3: WebSocket Protocol Ping/Pong

```
WHEN server sends WebSocket protocol Ping frame
THEN client SHOULD respond with Pong frame
  AND SHALL update last_activity_timestamp
  AND server SHALL send Ping every 30000ms
```

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Ping-Pong latency | < 50ms | Time from ping to pong |
| Timeout detection | 60000ms +/- 1000ms | Time from last activity |
| Protocol ping interval | 30000ms | Fixed interval |

### Verification Test Template

```rust
#[cfg(test)]
mod req_websocket_004_tests {
    use serde_json::{json, Value};
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;
    use std::time::Instant;

    /// REQ-WEBSOCKET-004.1: Ping returns pong
    #[tokio::test]
    async fn test_ping_returns_pong() {
        // GIVEN a WebSocket connection
        let (mut ws_stream, _) = setup_websocket_connection().await;

        // WHEN client sends ping
        let start = Instant::now();
        let ping_msg = json!({ "action": "ping" });
        ws_stream.send(Message::Text(ping_msg.to_string())).await.unwrap();

        // THEN should receive pong
        let response = ws_stream.next().await.unwrap().unwrap();
        let elapsed = start.elapsed();
        let response_json: Value = serde_json::from_str(&response.to_text().unwrap()).unwrap();

        assert_eq!(response_json["event"], "pong");
        assert!(response_json.get("timestamp").is_some());
        assert!(elapsed.as_millis() < 50, "Pong should respond within 50ms");
    }

    /// REQ-WEBSOCKET-004.2: Connection timeout closes connection
    #[tokio::test]
    #[ignore] // Long-running test
    async fn test_connection_timeout_closes_connection() {
        // GIVEN a WebSocket connection with short timeout for testing
        let (mut ws_stream, _) = setup_websocket_with_short_timeout().await;

        // WHEN no ping is sent for timeout duration
        tokio::time::sleep(tokio::time::Duration::from_secs(65)).await;

        // THEN connection should be closed
        let result = ws_stream.next().await;
        assert!(result.is_none() || matches!(result, Some(Err(_))));
    }
}
```

### Acceptance Criteria

- [ ] Ping returns pong with timestamp
- [ ] Pong response within 50ms
- [ ] Connection closes after 60 seconds of inactivity
- [ ] Protocol-level ping frames are sent every 30 seconds
- [ ] Activity timestamp updated on any message

---

## REQ-WEBSOCKET-005: Connection Closure

### Problem Statement

Connections must be cleanly closed when clients disconnect, error out, or the server shuts down.

### Specification

#### REQ-WEBSOCKET-005.1: Client-Initiated Close

```
WHEN client sends WebSocket Close frame
THEN SHALL acknowledge with Close frame
  AND SHALL remove subscription if active
  AND SHALL cleanup connection state
  AND SHALL decrement workspace connection count
```

#### REQ-WEBSOCKET-005.2: Server-Initiated Close

```
WHEN server needs to close connection (shutdown, error, timeout)
THEN SHALL send Close frame with appropriate code:
  - 1000: Normal closure
  - 1001: Server going away
  - 1011: Server error
  AND SHALL cleanup connection state
  AND SHALL log closure reason
```

#### REQ-WEBSOCKET-005.3: Abnormal Disconnection

```
WHEN connection drops without Close frame (network failure)
THEN SHALL detect via read/write error
  AND SHALL cleanup connection state within 5000ms
  AND SHALL NOT leave orphaned subscription
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_websocket_005_tests {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    /// REQ-WEBSOCKET-005.1: Client close is acknowledged
    #[tokio::test]
    async fn test_client_close_acknowledged() {
        // GIVEN a subscribed WebSocket connection
        let (mut ws_stream, _) = setup_subscribed_websocket().await;

        // WHEN client closes connection
        ws_stream.close(None).await.unwrap();

        // THEN connection should be cleanly closed
        assert!(ws_stream.next().await.is_none());
    }

    /// REQ-WEBSOCKET-005.3: Subscription cleaned up on disconnect
    #[tokio::test]
    async fn test_subscription_cleaned_up_on_disconnect() {
        // GIVEN a subscribed WebSocket connection
        let (mut ws_stream, workspace_id) = setup_subscribed_websocket().await;
        let connection_count_before = get_workspace_connection_count(&workspace_id).await;

        // WHEN client disconnects
        drop(ws_stream);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // THEN workspace connection count should decrease
        let connection_count_after = get_workspace_connection_count(&workspace_id).await;
        assert_eq!(connection_count_after, connection_count_before - 1);
    }
}
```

### Acceptance Criteria

- [ ] Client close frame is acknowledged
- [ ] Subscription is removed on close
- [ ] Connection count is decremented
- [ ] Abnormal disconnect is detected within 5 seconds
- [ ] No orphaned subscriptions after disconnect

---

# Diff Event Streaming Requirements

## REQ-WEBSOCKET-006: Diff Started Event

### Problem Statement

When file changes trigger a diff analysis, all subscribed clients must be notified that analysis is beginning.

### Specification

#### REQ-WEBSOCKET-006.1: Diff Started Broadcast

```
WHEN file watcher detects changes in watched workspace
  AND debounce period (500ms) has elapsed
THEN SHALL broadcast to all subscribed clients:
    {
      "event": "diff_started",
      "workspace_id": "ws_20260123_143000_abc123",
      "files_changed": 3,
      "triggered_by": "file_watcher",
      "timestamp": "2026-01-23T14:30:00Z"
    }
  AND SHALL start diff computation
  AND SHALL broadcast within 100ms of debounce completion
```

#### REQ-WEBSOCKET-006.2: Manual Diff Trigger

```
WHEN diff is triggered manually (not by file watcher)
THEN SHALL broadcast with triggered_by: "manual"
  AND SHALL include files_changed: 0 if full diff
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_websocket_006_tests {
    use serde_json::Value;
    use futures_util::StreamExt;

    /// REQ-WEBSOCKET-006.1: Diff started event is broadcast
    #[tokio::test]
    async fn test_diff_started_broadcast() {
        // GIVEN a subscribed WebSocket connection
        let (mut ws_stream, workspace_id) = setup_subscribed_websocket().await;

        // WHEN files change in workspace
        trigger_file_change_in_workspace(&workspace_id).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await; // Wait for debounce

        // THEN should receive diff_started event
        let response = ws_stream.next().await.unwrap().unwrap();
        let response_json: Value = serde_json::from_str(&response.to_text().unwrap()).unwrap();

        assert_eq!(response_json["event"], "diff_started");
        assert_eq!(response_json["workspace_id"], workspace_id);
        assert!(response_json["files_changed"].as_u64().unwrap() >= 1);
        assert_eq!(response_json["triggered_by"], "file_watcher");
    }
}
```

### Acceptance Criteria

- [ ] diff_started event broadcast on file changes
- [ ] files_changed count is accurate
- [ ] triggered_by reflects trigger source
- [ ] Event sent within 100ms of debounce completion

---

## REQ-WEBSOCKET-007: Entity Change Events

### Problem Statement

As diff analysis processes entities, clients must receive granular events for each entity change (added, removed, modified).

### Specification

#### REQ-WEBSOCKET-007.1: Entity Added Event

```
WHEN diff analysis detects new entity in "after" state
THEN SHALL broadcast to all subscribed clients:
    {
      "event": "entity_added",
      "workspace_id": "ws_20260123_143000_abc123",
      "entity_key": "rust:fn:new_function:__src_lib_rs:50-75",
      "entity_type": "function",
      "file_path": "src/lib.rs",
      "line_range": { "start": 50, "end": 75 },
      "timestamp": "2026-01-23T14:30:01Z"
    }
```

#### REQ-WEBSOCKET-007.2: Entity Removed Event

```
WHEN diff analysis detects entity missing from "after" state
THEN SHALL broadcast to all subscribed clients:
    {
      "event": "entity_removed",
      "workspace_id": "ws_20260123_143000_abc123",
      "entity_key": "rust:fn:old_function:__src_lib_rs:20-45",
      "entity_type": "function",
      "file_path": "src/lib.rs",
      "timestamp": "2026-01-23T14:30:01Z"
    }
```

#### REQ-WEBSOCKET-007.3: Entity Modified Event

```
WHEN diff analysis detects entity changed between states
THEN SHALL broadcast to all subscribed clients:
    {
      "event": "entity_modified",
      "workspace_id": "ws_20260123_143000_abc123",
      "entity_key": "rust:fn:existing_func:__src_lib_rs",
      "entity_type": "function",
      "file_path": "src/lib.rs",
      "before_line_range": { "start": 10, "end": 30 },
      "after_line_range": { "start": 10, "end": 35 },
      "timestamp": "2026-01-23T14:30:01Z"
    }
```

#### REQ-WEBSOCKET-007.4: Entity Event Ordering

```
WHEN multiple entity changes occur in single diff
THEN SHALL send events in deterministic order:
  1. entity_removed events (alphabetically by entity_key)
  2. entity_added events (alphabetically by entity_key)
  3. entity_modified events (alphabetically by entity_key)
  AND SHALL maintain order consistency across clients
```

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Entity event latency | < 10ms per event | Time from detection to broadcast |
| Maximum events per diff | 10000 | Hard limit before batching |
| Event throughput | > 1000 events/sec | Events broadcast per second |

### Verification Test Template

```rust
#[cfg(test)]
mod req_websocket_007_tests {
    use serde_json::Value;
    use futures_util::StreamExt;

    /// REQ-WEBSOCKET-007.1: Entity added event has correct format
    #[tokio::test]
    async fn test_entity_added_event_format() {
        // GIVEN a subscribed WebSocket connection
        let (mut ws_stream, workspace_id) = setup_subscribed_websocket().await;

        // WHEN a new entity is added
        add_entity_to_workspace(&workspace_id, "fn", "new_func", "src/lib.rs").await;
        wait_for_diff_completion().await;

        // THEN should receive entity_added event
        let events = collect_events_until_diff_completed(&mut ws_stream).await;
        let added_event = events.iter().find(|e| e["event"] == "entity_added");

        assert!(added_event.is_some());
        let event = added_event.unwrap();
        assert!(event["entity_key"].as_str().unwrap().contains("new_func"));
        assert_eq!(event["entity_type"], "function");
        assert_eq!(event["file_path"], "src/lib.rs");
        assert!(event.get("line_range").is_some());
    }

    /// REQ-WEBSOCKET-007.2: Entity removed event has correct format
    #[tokio::test]
    async fn test_entity_removed_event_format() {
        // GIVEN a subscribed WebSocket connection with existing entity
        let (mut ws_stream, workspace_id) = setup_subscribed_websocket().await;

        // WHEN entity is removed
        remove_entity_from_workspace(&workspace_id, "existing_func").await;
        wait_for_diff_completion().await;

        // THEN should receive entity_removed event
        let events = collect_events_until_diff_completed(&mut ws_stream).await;
        let removed_event = events.iter().find(|e| e["event"] == "entity_removed");

        assert!(removed_event.is_some());
        let event = removed_event.unwrap();
        assert!(event["entity_key"].as_str().unwrap().contains("existing_func"));
    }

    /// REQ-WEBSOCKET-007.4: Events are ordered deterministically
    #[tokio::test]
    async fn test_entity_events_ordered_deterministically() {
        // GIVEN a subscribed WebSocket connection
        let (mut ws_stream, workspace_id) = setup_subscribed_websocket().await;

        // WHEN multiple entity changes occur
        modify_multiple_entities(&workspace_id).await;
        wait_for_diff_completion().await;

        // THEN events should be ordered: removed, added, modified
        let events = collect_entity_events(&mut ws_stream).await;
        let event_types: Vec<&str> = events.iter()
            .map(|e| e["event"].as_str().unwrap())
            .collect();

        // Verify ordering
        let mut saw_added = false;
        let mut saw_modified = false;
        for event_type in &event_types {
            match *event_type {
                "entity_removed" => {
                    assert!(!saw_added && !saw_modified, "Removed should come first");
                }
                "entity_added" => {
                    assert!(!saw_modified, "Added should come before modified");
                    saw_added = true;
                }
                "entity_modified" => {
                    saw_modified = true;
                }
                _ => {}
            }
        }
    }
}
```

### Acceptance Criteria

- [ ] entity_added events contain all required fields
- [ ] entity_removed events contain all required fields
- [ ] entity_modified events contain before/after line ranges
- [ ] Events are ordered deterministically (removed, added, modified)
- [ ] Event latency < 10ms per event
- [ ] System handles up to 10000 entity events per diff

---

## REQ-WEBSOCKET-008: Edge Change Events

### Problem Statement

Dependency edge changes must be streamed to clients to visualize how relationships between entities evolve.

### Specification

#### REQ-WEBSOCKET-008.1: Edge Added Event

```
WHEN diff analysis detects new edge in "after" state
THEN SHALL broadcast to all subscribed clients:
    {
      "event": "edge_added",
      "workspace_id": "ws_20260123_143000_abc123",
      "from_entity_key": "rust:fn:caller:__src_main_rs:10-20",
      "to_entity_key": "rust:fn:callee:__src_lib_rs:30-40",
      "edge_type": "Calls",
      "timestamp": "2026-01-23T14:30:02Z"
    }
```

#### REQ-WEBSOCKET-008.2: Edge Removed Event

```
WHEN diff analysis detects edge missing from "after" state
THEN SHALL broadcast to all subscribed clients:
    {
      "event": "edge_removed",
      "workspace_id": "ws_20260123_143000_abc123",
      "from_entity_key": "rust:fn:old_caller:__src_main_rs:50-60",
      "to_entity_key": "rust:fn:old_callee:__src_lib_rs:70-80",
      "edge_type": "Calls",
      "timestamp": "2026-01-23T14:30:02Z"
    }
```

#### REQ-WEBSOCKET-008.3: Edge Event Ordering

```
WHEN edge changes occur in diff
THEN SHALL send edge events after all entity events
  AND SHALL order: edge_removed first, then edge_added
  AND edge keys SHALL reference stable entity identities
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_websocket_008_tests {
    use serde_json::Value;
    use futures_util::StreamExt;

    /// REQ-WEBSOCKET-008.1: Edge added event has correct format
    #[tokio::test]
    async fn test_edge_added_event_format() {
        // GIVEN a subscribed WebSocket connection
        let (mut ws_stream, workspace_id) = setup_subscribed_websocket().await;

        // WHEN a new dependency is created
        create_dependency_in_workspace(&workspace_id, "fn_a", "fn_b").await;
        wait_for_diff_completion().await;

        // THEN should receive edge_added event
        let events = collect_events_until_diff_completed(&mut ws_stream).await;
        let edge_event = events.iter().find(|e| e["event"] == "edge_added");

        assert!(edge_event.is_some());
        let event = edge_event.unwrap();
        assert!(event.get("from_entity_key").is_some());
        assert!(event.get("to_entity_key").is_some());
        assert!(event.get("edge_type").is_some());
    }

    /// REQ-WEBSOCKET-008.3: Edge events come after entity events
    #[tokio::test]
    async fn test_edge_events_after_entity_events() {
        // GIVEN a subscribed WebSocket connection
        let (mut ws_stream, workspace_id) = setup_subscribed_websocket().await;

        // WHEN entities and edges change
        make_complex_changes(&workspace_id).await;
        wait_for_diff_completion().await;

        // THEN edge events should come after entity events
        let events = collect_all_events(&mut ws_stream).await;
        let mut saw_edge_event = false;
        for event in &events {
            let event_type = event["event"].as_str().unwrap();
            if event_type.starts_with("edge_") {
                saw_edge_event = true;
            }
            if event_type.starts_with("entity_") && saw_edge_event {
                panic!("Entity events should come before edge events");
            }
        }
    }
}
```

### Acceptance Criteria

- [ ] edge_added events contain from/to keys and edge type
- [ ] edge_removed events contain from/to keys and edge type
- [ ] Edge events sent after all entity events
- [ ] Edge events ordered: removed before added

---

## REQ-WEBSOCKET-009: Diff Completed Event

### Problem Statement

When diff analysis completes, clients must receive a summary event with statistics and timing information.

### Specification

#### REQ-WEBSOCKET-009.1: Diff Completed Broadcast

```
WHEN diff analysis completes successfully
THEN SHALL broadcast to all subscribed clients:
    {
      "event": "diff_completed",
      "workspace_id": "ws_20260123_143000_abc123",
      "summary": {
        "total_before_count": 150,
        "total_after_count": 155,
        "added_entity_count": 8,
        "removed_entity_count": 3,
        "modified_entity_count": 5,
        "unchanged_entity_count": 139,
        "relocated_entity_count": 0
      },
      "blast_radius_count": 25,
      "duration_ms": 150,
      "timestamp": "2026-01-23T14:30:03Z"
    }
  AND summary SHALL match DiffSummaryDataPayload from parseltongue-core
```

#### REQ-WEBSOCKET-009.2: Diff Summary Accuracy

```
WHEN diff_completed event is sent
THEN summary counts SHALL satisfy:
  - added_entity_count + removed_entity_count + modified_entity_count
    + unchanged_entity_count + relocated_entity_count
    = max(total_before_count, total_after_count) + overlap
  AND total_after_count - total_before_count = added_entity_count - removed_entity_count
  AND blast_radius_count >= modified_entity_count + added_entity_count
```

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Small diff (<100 entities) | < 500ms | duration_ms field |
| Medium diff (100-1000 entities) | < 2000ms | duration_ms field |
| Large diff (1000-10000 entities) | < 10000ms | duration_ms field |

### Verification Test Template

```rust
#[cfg(test)]
mod req_websocket_009_tests {
    use serde_json::Value;
    use futures_util::StreamExt;

    /// REQ-WEBSOCKET-009.1: Diff completed contains accurate summary
    #[tokio::test]
    async fn test_diff_completed_summary() {
        // GIVEN a subscribed WebSocket connection
        let (mut ws_stream, workspace_id) = setup_subscribed_websocket().await;

        // WHEN changes trigger diff analysis
        make_changes_to_workspace(&workspace_id, 5, 3, 2).await; // 5 added, 3 removed, 2 modified

        // THEN should receive diff_completed with correct summary
        let events = collect_events_until_diff_completed(&mut ws_stream).await;
        let completed_event = events.iter().find(|e| e["event"] == "diff_completed").unwrap();

        assert!(completed_event.get("summary").is_some());
        let summary = &completed_event["summary"];
        assert_eq!(summary["added_entity_count"], 5);
        assert_eq!(summary["removed_entity_count"], 3);
        assert_eq!(summary["modified_entity_count"], 2);
        assert!(completed_event["duration_ms"].as_u64().unwrap() > 0);
        assert!(completed_event.get("blast_radius_count").is_some());
    }

    /// REQ-WEBSOCKET-009.2: Summary counts are mathematically consistent
    #[tokio::test]
    async fn test_diff_summary_mathematically_consistent() {
        // GIVEN a subscribed WebSocket connection
        let (mut ws_stream, workspace_id) = setup_subscribed_websocket().await;

        // WHEN changes occur
        make_random_changes(&workspace_id).await;

        // THEN summary should be consistent
        let events = collect_events_until_diff_completed(&mut ws_stream).await;
        let completed = events.iter().find(|e| e["event"] == "diff_completed").unwrap();
        let summary = &completed["summary"];

        let added = summary["added_entity_count"].as_i64().unwrap();
        let removed = summary["removed_entity_count"].as_i64().unwrap();
        let before = summary["total_before_count"].as_i64().unwrap();
        let after = summary["total_after_count"].as_i64().unwrap();

        assert_eq!(after - before, added - removed);
    }
}
```

### Acceptance Criteria

- [ ] diff_completed event contains complete summary
- [ ] Summary matches DiffSummaryDataPayload structure
- [ ] Summary counts are mathematically consistent
- [ ] duration_ms accurately reflects processing time
- [ ] blast_radius_count is included

---

# Error Handling Requirements

## REQ-WEBSOCKET-010: Message Parsing Errors

### Problem Statement

Invalid messages from clients must be handled gracefully without crashing the connection.

### Specification

#### REQ-WEBSOCKET-010.1: Invalid JSON Message

```
WHEN client sends invalid JSON message: "{ not valid json"
THEN SHALL respond with error:
    {
      "event": "error",
      "code": "INVALID_JSON_MESSAGE",
      "message": "Failed to parse message as JSON",
      "timestamp": "..."
    }
  AND SHALL NOT close connection
  AND SHALL continue processing subsequent valid messages
```

#### REQ-WEBSOCKET-010.2: Unknown Action Type

```
WHEN client sends message with unknown action:
    {
      "action": "unknown_action",
      "data": "..."
    }
THEN SHALL respond with error:
    {
      "event": "error",
      "code": "UNKNOWN_ACTION_TYPE",
      "message": "Unknown action: unknown_action. Valid actions: subscribe, unsubscribe, ping",
      "timestamp": "..."
    }
  AND SHALL NOT close connection
```

#### REQ-WEBSOCKET-010.3: Binary Message Handling

```
WHEN client sends binary WebSocket message
THEN SHALL respond with error:
    {
      "event": "error",
      "code": "INVALID_MESSAGE_TYPE",
      "message": "Binary messages not supported. Send JSON text.",
      "timestamp": "..."
    }
  AND SHALL NOT close connection
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_websocket_010_tests {
    use serde_json::Value;
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    /// REQ-WEBSOCKET-010.1: Invalid JSON returns error but keeps connection
    #[tokio::test]
    async fn test_invalid_json_returns_error() {
        // GIVEN a WebSocket connection
        let (mut ws_stream, _) = setup_websocket_connection().await;

        // WHEN client sends invalid JSON
        ws_stream.send(Message::Text("{ not valid json".to_string())).await.unwrap();

        // THEN should receive error
        let response = ws_stream.next().await.unwrap().unwrap();
        let response_json: Value = serde_json::from_str(&response.to_text().unwrap()).unwrap();

        assert_eq!(response_json["event"], "error");
        assert_eq!(response_json["code"], "INVALID_JSON_MESSAGE");

        // AND connection should still work
        ws_stream.send(Message::Text(r#"{"action":"ping"}"#.to_string())).await.unwrap();
        let pong = ws_stream.next().await.unwrap().unwrap();
        let pong_json: Value = serde_json::from_str(&pong.to_text().unwrap()).unwrap();
        assert_eq!(pong_json["event"], "pong");
    }

    /// REQ-WEBSOCKET-010.2: Unknown action returns error
    #[tokio::test]
    async fn test_unknown_action_returns_error() {
        // GIVEN a WebSocket connection
        let (mut ws_stream, _) = setup_websocket_connection().await;

        // WHEN client sends unknown action
        ws_stream.send(Message::Text(r#"{"action":"invalid_action"}"#.to_string())).await.unwrap();

        // THEN should receive error
        let response = ws_stream.next().await.unwrap().unwrap();
        let response_json: Value = serde_json::from_str(&response.to_text().unwrap()).unwrap();

        assert_eq!(response_json["event"], "error");
        assert_eq!(response_json["code"], "UNKNOWN_ACTION_TYPE");
    }
}
```

### Acceptance Criteria

- [ ] Invalid JSON returns INVALID_JSON_MESSAGE error
- [ ] Unknown action returns UNKNOWN_ACTION_TYPE error
- [ ] Binary messages return INVALID_MESSAGE_TYPE error
- [ ] Connection remains open after errors
- [ ] Subsequent valid messages are processed

---

## REQ-WEBSOCKET-011: Broadcast Error Handling

### Problem Statement

Errors during broadcast (e.g., client disconnected) must not affect other clients or crash the server.

### Specification

#### REQ-WEBSOCKET-011.1: Broadcast to Disconnected Client

```
WHEN server broadcasts diff event to workspace
  AND one client has disconnected without Close frame
THEN SHALL detect write error for disconnected client
  AND SHALL remove disconnected client from subscription list
  AND SHALL continue broadcast to remaining clients
  AND SHALL log error for monitoring
```

#### REQ-WEBSOCKET-011.2: Broadcast Timeout

```
WHEN server attempts to send message to slow client
  WITH write taking longer than 5000ms
THEN SHALL timeout the write operation
  AND SHALL close connection to slow client
  AND SHALL continue broadcast to other clients
  AND SHALL log timeout event
```

#### REQ-WEBSOCKET-011.3: Broadcast Consistency

```
WHEN multiple clients are subscribed to same workspace
THEN all connected clients SHALL receive identical messages
  AND message order SHALL be preserved across clients
  AND no message SHALL be skipped for any connected client
```

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Broadcast timeout | 5000ms max per client | Per-client write timeout |
| Error recovery | < 100ms | Time to remove failed client |
| Cross-client consistency | 100% | Message hash comparison |

### Verification Test Template

```rust
#[cfg(test)]
mod req_websocket_011_tests {
    use futures_util::StreamExt;

    /// REQ-WEBSOCKET-011.3: Multiple clients receive same messages
    #[tokio::test]
    async fn test_broadcast_consistency() {
        // GIVEN multiple WebSocket connections subscribed to same workspace
        let (mut ws1, workspace_id) = setup_subscribed_websocket().await;
        let (mut ws2, _) = setup_websocket_and_subscribe(&workspace_id).await;

        // WHEN changes trigger diff events
        trigger_file_change_in_workspace(&workspace_id).await;

        // THEN both clients should receive identical events
        let events1 = collect_events_until_diff_completed(&mut ws1).await;
        let events2 = collect_events_until_diff_completed(&mut ws2).await;

        assert_eq!(events1.len(), events2.len());
        for (e1, e2) in events1.iter().zip(events2.iter()) {
            assert_eq!(e1["event"], e2["event"]);
            assert_eq!(e1.get("entity_key"), e2.get("entity_key"));
        }
    }
}
```

### Acceptance Criteria

- [ ] Disconnected clients are removed without affecting others
- [ ] Slow clients are timed out after 5 seconds
- [ ] All connected clients receive identical messages
- [ ] Message order is preserved across clients
- [ ] Server remains stable during client failures

---

# Multi-Client Requirements

## REQ-WEBSOCKET-012: Multi-Client Subscription

### Problem Statement

Multiple clients must be able to subscribe to the same workspace and receive identical diff events.

### Specification

#### REQ-WEBSOCKET-012.1: Multiple Clients Same Workspace

```
WHEN N clients subscribe to same workspace (N <= 100)
THEN all N clients SHALL receive diff events
  AND events SHALL be delivered within 100ms of each other
  AND no client SHALL be starved of events
```

#### REQ-WEBSOCKET-012.2: Client Isolation

```
WHEN client A is subscribed to workspace X
  AND client B is subscribed to workspace Y
THEN client A SHALL NOT receive events from workspace Y
  AND client B SHALL NOT receive events from workspace X
  AND events SHALL be routed based on subscription only
```

#### REQ-WEBSOCKET-012.3: Connection Tracking

```
WHEN managing multiple connections
THEN SharedWorkspaceStateContainer.ws_connections SHALL contain:
  - Mapping of workspace_id to Vec<WsSender>
  - Efficient O(1) lookup by workspace_id
  - Efficient O(n) broadcast to n clients
  AND memory usage SHALL scale linearly with connection count
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_websocket_012_tests {
    use futures_util::StreamExt;
    use std::time::Instant;

    /// REQ-WEBSOCKET-012.1: Multiple clients receive events concurrently
    #[tokio::test]
    async fn test_multiple_clients_receive_events() {
        // GIVEN 5 clients subscribed to same workspace
        let (mut clients, workspace_id) = setup_multiple_subscribed_clients(5).await;

        // WHEN diff event is triggered
        let start = Instant::now();
        trigger_file_change_in_workspace(&workspace_id).await;

        // THEN all clients should receive events
        let mut receive_times = Vec::new();
        for client in &mut clients {
            let event = client.next().await.unwrap().unwrap();
            receive_times.push(start.elapsed());
            assert!(event.to_text().unwrap().contains("diff_started"));
        }

        // AND receive times should be within 100ms of each other
        let max_time = receive_times.iter().max().unwrap();
        let min_time = receive_times.iter().min().unwrap();
        let spread = max_time.as_millis() - min_time.as_millis();
        assert!(spread < 100, "Event delivery spread {}ms exceeds 100ms", spread);
    }

    /// REQ-WEBSOCKET-012.2: Clients are isolated by workspace
    #[tokio::test]
    async fn test_client_workspace_isolation() {
        // GIVEN clients subscribed to different workspaces
        let (mut client_a, workspace_x) = setup_subscribed_websocket().await;
        let (mut client_b, workspace_y) = setup_subscribed_websocket_to_different_workspace().await;

        // WHEN changes occur in workspace X only
        trigger_file_change_in_workspace(&workspace_x).await;

        // THEN only client A should receive events
        let timeout = tokio::time::Duration::from_millis(1000);

        // Client A should receive event
        let result_a = tokio::time::timeout(timeout, client_a.next()).await;
        assert!(result_a.is_ok());

        // Client B should NOT receive event (timeout)
        let result_b = tokio::time::timeout(timeout, client_b.next()).await;
        assert!(result_b.is_err(), "Client B should not receive events from workspace X");
    }
}
```

### Acceptance Criteria

- [ ] Up to 100 clients can subscribe to same workspace
- [ ] All subscribed clients receive identical events
- [ ] Event delivery within 100ms across all clients
- [ ] Clients isolated by workspace subscription
- [ ] Memory scales linearly with connections

---

# Performance Requirements

## REQ-WEBSOCKET-013: Performance Contract

### Problem Statement

The WebSocket streaming system must meet defined performance targets for responsiveness and throughput.

### Specification

#### REQ-WEBSOCKET-013.1: End-to-End Latency

```
WHEN file change occurs in watched workspace
THEN first diff_started event SHALL be delivered within:
  - 600ms (500ms debounce + 100ms processing)
  AND diff_completed event SHALL be delivered within:
  - 2000ms for typical diffs (<100 changed entities)
  - 10000ms for large diffs (100-1000 changed entities)
```

#### REQ-WEBSOCKET-013.2: Message Throughput

```
WHEN streaming diff events to subscribed clients
THEN SHALL support:
  - Minimum 1000 events/second per workspace
  - Minimum 100 concurrent WebSocket connections per server
  - Minimum 10 concurrent workspaces with active streaming
```

#### REQ-WEBSOCKET-013.3: Memory Efficiency

```
WHEN managing WebSocket connections
THEN memory usage SHALL be:
  - < 1MB per idle connection
  - < 10MB per active streaming workspace
  - Total < 500MB for 100 connections across 10 workspaces
```

### Performance Contract Summary

| Metric | Target | Acceptable Range | Measurement |
|--------|--------|------------------|-------------|
| File change to diff_started | < 600ms | 500-700ms | Timestamp delta |
| Small diff completion | < 2000ms | 1000-3000ms | duration_ms |
| Large diff completion | < 10000ms | 5000-15000ms | duration_ms |
| Event throughput | > 1000/sec | 800-2000/sec | Events per second |
| Concurrent connections | > 100 | 80-150 | Active WebSockets |
| Memory per connection | < 1MB | 0.5-2MB | Process memory delta |
| Ping-pong latency | < 50ms | 10-100ms | Round-trip time |

### Verification Test Template

```rust
#[cfg(test)]
mod req_websocket_013_tests {
    use std::time::Instant;
    use futures_util::StreamExt;

    /// REQ-WEBSOCKET-013.1: End-to-end latency within targets
    #[tokio::test]
    async fn test_end_to_end_latency() {
        // GIVEN a subscribed WebSocket connection
        let (mut ws_stream, workspace_id) = setup_subscribed_websocket().await;

        // WHEN file change occurs
        let start = Instant::now();
        trigger_file_change_in_workspace(&workspace_id).await;

        // THEN diff_started should arrive within 600ms
        let first_event = ws_stream.next().await.unwrap().unwrap();
        let latency = start.elapsed();

        assert!(latency.as_millis() < 700, "Latency {}ms exceeds 700ms target", latency.as_millis());
        assert!(first_event.to_text().unwrap().contains("diff_started"));
    }

    /// REQ-WEBSOCKET-013.2: Supports many concurrent connections
    #[tokio::test]
    #[ignore] // Resource-intensive test
    async fn test_concurrent_connections() {
        // GIVEN server configured for test
        let workspace_id = setup_watched_workspace().await;

        // WHEN 100 clients connect and subscribe
        let mut clients = Vec::new();
        for _ in 0..100 {
            let client = connect_and_subscribe(&workspace_id).await;
            clients.push(client);
        }

        // THEN all should be connected
        assert_eq!(clients.len(), 100);

        // AND triggering event should reach all
        trigger_file_change_in_workspace(&workspace_id).await;
        for client in &mut clients {
            let event = tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                client.next()
            ).await;
            assert!(event.is_ok(), "Client should receive event");
        }
    }
}
```

### Acceptance Criteria

- [ ] diff_started delivered within 600ms of file change
- [ ] Small diff completed within 2000ms
- [ ] System supports 100+ concurrent connections
- [ ] Memory stays under 500MB for max load
- [ ] Event throughput exceeds 1000/sec

---

# Implementation Guide

## Handler Function Signatures

Following the 4-word naming convention:

```rust
/// Handle WebSocket upgrade request
///
/// # 4-Word Name: handle_websocket_diff_stream_upgrade
pub async fn handle_websocket_diff_stream_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<SharedWorkspaceStateContainer>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        manage_websocket_connection_lifecycle(socket, state)
    })
}

/// Manage WebSocket connection lifecycle
///
/// # 4-Word Name: manage_websocket_connection_lifecycle
async fn manage_websocket_connection_lifecycle(
    socket: WebSocket,
    state: SharedWorkspaceStateContainer,
) {
    let (sender, receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));
    let connection_state = ConnectionStateData::new();

    // Spawn read and heartbeat tasks
    let read_task = tokio::spawn(process_incoming_client_messages(
        receiver,
        sender.clone(),
        state.clone(),
        connection_state.clone(),
    ));

    let heartbeat_task = tokio::spawn(monitor_connection_heartbeat_status(
        sender.clone(),
        connection_state.clone(),
    ));

    // Wait for connection close
    let _ = tokio::select! {
        _ = read_task => {}
        _ = heartbeat_task => {}
    };

    // Cleanup
    cleanup_websocket_connection_state(&state, &connection_state).await;
}

/// Process incoming client messages
///
/// # 4-Word Name: process_incoming_client_messages
async fn process_incoming_client_messages(
    mut receiver: SplitStream<WebSocket>,
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    state: SharedWorkspaceStateContainer,
    connection_state: Arc<RwLock<ConnectionStateData>>,
) {
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                handle_text_message_from_client(&text, &sender, &state, &connection_state).await;
            }
            Ok(Message::Binary(_)) => {
                send_error_to_client(&sender, "INVALID_MESSAGE_TYPE", "Binary messages not supported").await;
            }
            Ok(Message::Close(_)) => break,
            Ok(Message::Ping(data)) => {
                let _ = sender.lock().await.send(Message::Pong(data)).await;
            }
            Err(_) => break,
            _ => {}
        }
    }
}

/// Broadcast diff event to workspace
///
/// # 4-Word Name: broadcast_diff_event_to_workspace
pub async fn broadcast_diff_event_to_workspace(
    state: &SharedWorkspaceStateContainer,
    workspace_id: &str,
    event: WebSocketServerOutboundMessageType,
) {
    let connections = state.ws_connections.read().await;
    if let Some(senders) = connections.get(workspace_id) {
        let message = serde_json::to_string(&event).unwrap();
        let mut failed_indices = Vec::new();

        for (idx, sender) in senders.iter().enumerate() {
            let send_result = tokio::time::timeout(
                Duration::from_millis(5000),
                sender.lock().await.send(Message::Text(message.clone()))
            ).await;

            if send_result.is_err() || send_result.unwrap().is_err() {
                failed_indices.push(idx);
            }
        }

        // Remove failed connections
        if !failed_indices.is_empty() {
            drop(connections);
            let mut connections = state.ws_connections.write().await;
            if let Some(senders) = connections.get_mut(workspace_id) {
                for idx in failed_indices.into_iter().rev() {
                    senders.remove(idx);
                }
            }
        }
    }
}
```

## Route Registration

Add to `route_definition_builder_module.rs`:

```rust
use axum::routing::get;
use crate::websocket::handler::handle_websocket_diff_stream_upgrade;

// Add WebSocket route
.route(
    "/websocket-diff-stream",
    get(handle_websocket_diff_stream_upgrade)
)
```

## Connection State Data

```rust
/// State data for a single WebSocket connection
///
/// # 4-Word Name: ConnectionStateData
pub struct ConnectionStateData {
    /// Currently subscribed workspace (None if not subscribed)
    pub subscribed_workspace_id: Option<String>,
    /// Timestamp of last activity (ping, message, etc.)
    pub last_activity_timestamp: DateTime<Utc>,
    /// Unique connection identifier for logging
    pub connection_unique_identifier: String,
    /// Message sequence counter for debugging
    pub message_sequence_counter: u64,
}
```

## File Watcher Integration

```rust
/// Trigger diff computation on file changes
///
/// # 4-Word Name: trigger_diff_on_file_change
pub async fn trigger_diff_on_file_change(
    state: &SharedWorkspaceStateContainer,
    workspace_id: &str,
    changed_files: Vec<PathBuf>,
) {
    // Broadcast diff_started
    broadcast_diff_event_to_workspace(
        state,
        workspace_id,
        WebSocketServerOutboundMessageType::DiffAnalysisStartedNotification {
            workspace_id: workspace_id.to_string(),
            files_changed: changed_files.len(),
            triggered_by: "file_watcher".to_string(),
            timestamp: Utc::now(),
        },
    ).await;

    // Compute diff (this generates entity/edge events)
    let diff_result = compute_incremental_diff_result(state, workspace_id, &changed_files).await;

    // Stream entity events
    for change in &diff_result.entity_changes {
        let event = match change.change_type {
            EntityChangeTypeClassification::AddedToCodebase => {
                WebSocketServerOutboundMessageType::EntityAddedEventNotification { /* ... */ }
            }
            EntityChangeTypeClassification::RemovedFromCodebase => {
                WebSocketServerOutboundMessageType::EntityRemovedEventNotification { /* ... */ }
            }
            EntityChangeTypeClassification::ModifiedInCodebase => {
                WebSocketServerOutboundMessageType::EntityModifiedEventNotification { /* ... */ }
            }
            _ => continue,
        };
        broadcast_diff_event_to_workspace(state, workspace_id, event).await;
    }

    // Stream edge events
    for change in &diff_result.edge_changes {
        // Similar pattern for edges
    }

    // Broadcast diff_completed
    broadcast_diff_event_to_workspace(
        state,
        workspace_id,
        WebSocketServerOutboundMessageType::DiffAnalysisCompletedNotification {
            workspace_id: workspace_id.to_string(),
            summary: diff_result.summary,
            blast_radius_count: compute_blast_radius_count(&diff_result),
            duration_ms: start.elapsed().as_millis() as u64,
            timestamp: Utc::now(),
        },
    ).await;
}
```

---

## Quality Checklist

Before implementation is complete, verify:

- [ ] All quantities are specific and measurable
- [ ] All behaviors are testable
- [ ] Error conditions are specified with codes
- [ ] Performance boundaries are defined
- [ ] Test templates are provided for all requirements
- [ ] Acceptance criteria are binary (pass/fail)
- [ ] No ambiguous language remains
- [ ] 4-word naming convention is followed
- [ ] Message formats match DiffSummaryDataPayload from parseltongue-core
- [ ] WebSocket handler follows Axum patterns
- [ ] Event ordering is deterministic
- [ ] Multi-client broadcast is consistent

---

## Traceability Matrix

| Requirement | Feature | Error Codes | Test Count |
|-------------|---------|-------------|------------|
| REQ-WEBSOCKET-001 | Connection Establishment | UPGRADE_REQUIRED | 4 |
| REQ-WEBSOCKET-002 | Subscribe to Workspace | MISSING_WORKSPACE_ID, WORKSPACE_NOT_FOUND, WORKSPACE_NOT_WATCHING, ALREADY_SUBSCRIBED | 6 |
| REQ-WEBSOCKET-003 | Unsubscribe from Workspace | NOT_SUBSCRIBED | 3 |
| REQ-WEBSOCKET-004 | Heartbeat Mechanism | CONNECTION_TIMEOUT | 3 |
| REQ-WEBSOCKET-005 | Connection Closure | - | 3 |
| REQ-WEBSOCKET-006 | Diff Started Event | - | 2 |
| REQ-WEBSOCKET-007 | Entity Change Events | - | 4 |
| REQ-WEBSOCKET-008 | Edge Change Events | - | 3 |
| REQ-WEBSOCKET-009 | Diff Completed Event | - | 2 |
| REQ-WEBSOCKET-010 | Message Parsing Errors | INVALID_JSON_MESSAGE, UNKNOWN_ACTION_TYPE, INVALID_MESSAGE_TYPE | 3 |
| REQ-WEBSOCKET-011 | Broadcast Error Handling | - | 2 |
| REQ-WEBSOCKET-012 | Multi-Client Subscription | SUBSCRIPTION_LIMIT_EXCEEDED | 3 |
| REQ-WEBSOCKET-013 | Performance Contract | - | 3 |
| **Total** | **13 requirement groups** | **10 error codes** | **41 tests** |

---

## Files to Create

Following the project structure:

```
crates/pt08-http-code-query-server/
  src/
    websocket/
      mod.rs                        # Module exports
      message_types.rs              # WebSocketClientInboundMessageType, WebSocketServerOutboundMessageType
      handler.rs                    # handle_websocket_diff_stream_upgrade, manage_websocket_connection_lifecycle
      connection_state.rs           # ConnectionStateData
      broadcaster.rs                # broadcast_diff_event_to_workspace
    file_watcher/
      mod.rs                        # Module exports
      service.rs                    # FileWatcherServiceImpl
      debouncer.rs                  # Debounce logic (500ms)
      trigger.rs                    # trigger_diff_on_file_change
```

---

*Specification document created 2026-01-23*
*Phase 2.2 target: WebSocket Streaming Backend*
*Test target: 41 new tests*
*Depends on: Phase 2.1 (Workspace Management) - Complete*

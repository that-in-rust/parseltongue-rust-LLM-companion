# Parseltongue Phase 2: Real-Time Diff Visualization Architecture

> **Version**: 2.0.0 (2026-01-23)
> **Status**: PLANNING
> **Dependencies**: Phase 1 Complete (244 tests passing)

---

## 1. System Architecture Diagram

```
                                    PHASE 2 ARCHITECTURE
+=============================================================================================+
|                                                                                             |
|   FRONTEND (React + Three.js)                                                               |
|   +--------------------------------------------------------------------------------------+  |
|   |                                                                                      |  |
|   |  +----------------+    +------------------+    +---------------------------+         |  |
|   |  | WorkspacePanel |    | DiffGraphCanvas  |    | ControlsOverlay           |         |  |
|   |  |                |    | (r3f-forcegraph) |    | - CameraControls          |         |  |
|   |  | - List All     |    |                  |    | - FilterPanel             |         |  |
|   |  | - Create New   |    | - ForceGraph3D   |    | - SearchBar               |         |  |
|   |  | - Watch Toggle |    | - NodeRenderer   |    | - LegendPanel             |         |  |
|   |  +-------+--------+    | - LinkRenderer   |    +---------------------------+         |  |
|   |          |             +--------+---------+                                          |  |
|   |          |                      |                                                    |  |
|   +----------|----------------------|----------------------------------------------------+  |
|              |                      |                                                       |
|              | HTTP REST            | WebSocket (real-time)                                 |
|              v                      v                                                       |
|   +=============================================================================================+
|   |                          BACKEND (Rust/Axum)                                            |
|   |                                                                                         |
|   |   +---------------------------+     +-------------------------------------------+       |
|   |   | Workspace Endpoints       |     | WebSocket Handler                         |       |
|   |   |                           |     |                                           |       |
|   |   | POST /workspace-create-   |     | WS /workspace-live-stream/{id}            |       |
|   |   |      from-path            |     |   - Split sender/receiver                 |       |
|   |   | GET  /workspace-list-all  |     |   - Debounced file events (500ms)         |       |
|   |   | POST /workspace-watch-    |     |   - Push DiffUpdatePayload                |       |
|   |   |      toggle               |     |                                           |       |
|   |   +-------------+-------------+     +---------------------+---------------------+       |
|   |                 |                                         |                             |
|   |                 v                                         v                             |
|   |   +---------------------------+     +-------------------------------------------+       |
|   |   | WorkspaceManagerService   |     | FileWatcherService                        |       |
|   |   |                           |     |                                           |       |
|   |   | - Workspace CRUD          |     | - notify crate integration                |       |
|   |   | - Database path mgmt      |     | - Debounced event batching                |       |
|   |   | - Persistence to disk     |     | - Trigger re-index on change              |       |
|   |   +-------------+-------------+     +---------------------+---------------------+       |
|   |                 |                                         |                             |
|   |                 v                                         v                             |
|   |   +-----------------------------------------------------------------------------------+ |
|   |   |                          SharedWorkspaceStateContainer                            | |
|   |   |                                                                                   | |
|   |   |  workspaces: Arc<RwLock<HashMap<WorkspaceId, WorkspaceMetadata>>>                 | |
|   |   |  watchers: Arc<RwLock<HashMap<WorkspaceId, WatcherHandle>>>                       | |
|   |   |  ws_connections: Arc<RwLock<HashMap<WorkspaceId, Vec<WsSender>>>>                 | |
|   |   +-----------------------------------------------------------------------------------+ |
|   |                 |                                                                       |
|   |                 v                                                                       |
|   |   +-----------------------------------------------------------------------------------+ |
|   |   |                          EXISTING pt08 Infrastructure                             | |
|   |   |                                                                                   | |
|   |   |  SharedApplicationStateContainer + CozoDbStorage + 16 Endpoints                   | |
|   |   +-----------------------------------------------------------------------------------+ |
|   |                                                                                         |
|   +=============================================================================================+
|                                                                                             |
+=============================================================================================+
```

---

## 2. Component Breakdown by Workstream

### Workstream A: Workspace Management (3 Endpoints)

#### A.1 Data Types

```rust
/// Workspace unique identifier
/// # 4-Word Name: WorkspaceUniqueIdentifierType
pub type WorkspaceUniqueIdentifierType = String;

/// Workspace metadata stored on disk
/// # 4-Word Name: WorkspaceMetadataPayloadStruct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetadataPayloadStruct {
    pub workspace_identifier_value: WorkspaceUniqueIdentifierType,
    pub workspace_display_name: String,
    pub source_directory_path_value: PathBuf,
    pub base_database_path_value: String,
    pub live_database_path_value: String,
    pub watch_enabled_flag_status: bool,
    pub created_timestamp_utc_value: DateTime<Utc>,
    pub last_indexed_timestamp_option: Option<DateTime<Utc>>,
}

/// Request body for workspace creation
/// # 4-Word Name: WorkspaceCreateRequestBodyStruct
#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceCreateRequestBodyStruct {
    pub source_path_directory_value: String,
    pub workspace_display_name_option: Option<String>,
}

/// Response for workspace listing
/// # 4-Word Name: WorkspaceListResponsePayloadStruct
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceListResponsePayloadStruct {
    pub success: bool,
    pub endpoint: String,
    pub workspaces: Vec<WorkspaceMetadataPayloadStruct>,
    pub total_workspace_count_value: usize,
}

/// Request for watch toggle
/// # 4-Word Name: WorkspaceWatchToggleRequestStruct
#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceWatchToggleRequestStruct {
    pub workspace_identifier_target_value: WorkspaceUniqueIdentifierType,
    pub watch_enabled_desired_state: bool,
}
```

#### A.2 Endpoints

| Endpoint | Method | Request | Response | Handler Function |
|----------|--------|---------|----------|------------------|
| `/workspace-create-from-path` | POST | `WorkspaceCreateRequestBodyStruct` | `WorkspaceMetadataPayloadStruct` | `handle_workspace_create_from_path` |
| `/workspace-list-all` | GET | None | `WorkspaceListResponsePayloadStruct` | `handle_workspace_list_all_entries` |
| `/workspace-watch-toggle` | POST | `WorkspaceWatchToggleRequestStruct` | `WorkspaceMetadataPayloadStruct` | `handle_workspace_watch_toggle_state` |

#### A.3 Storage Strategy

Workspaces stored in `~/.parseltongue/workspaces/`:
```
~/.parseltongue/
  workspaces/
    {workspace_id}/
      metadata.json          # WorkspaceMetadataPayloadStruct
      base.db/               # RocksDB base snapshot
      live.db/               # RocksDB live snapshot
```

---

### Workstream B: WebSocket Streaming

#### B.1 WebSocket Message Types

```rust
/// Client to server message types
/// # 4-Word Name: WebSocketClientMessageType
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketClientMessageType {
    #[serde(rename = "subscribe")]
    SubscribeToWorkspaceUpdates { workspace_id: WorkspaceUniqueIdentifierType },
    #[serde(rename = "unsubscribe")]
    UnsubscribeFromWorkspaceUpdates { workspace_id: WorkspaceUniqueIdentifierType },
    #[serde(rename = "ping")]
    PingHeartbeatMessage,
}

/// Server to client message types
/// # 4-Word Name: WebSocketServerMessageType
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum WebSocketServerMessageType {
    #[serde(rename = "diff_update")]
    DiffUpdatePushNotification {
        workspace_id: WorkspaceUniqueIdentifierType,
        diff: DiffResultDataPayloadStruct,
        visualization: VisualizationGraphDataPayload,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "reindex_started")]
    ReindexStartedNotification {
        workspace_id: WorkspaceUniqueIdentifierType,
        triggered_by: String,  // "file_change" | "manual"
    },
    #[serde(rename = "reindex_completed")]
    ReindexCompletedNotification {
        workspace_id: WorkspaceUniqueIdentifierType,
        entities_indexed_count: usize,
        duration_ms: u64,
    },
    #[serde(rename = "error")]
    ErrorOccurredNotification {
        code: String,
        message: String,
    },
    #[serde(rename = "pong")]
    PongHeartbeatResponse,
}
```

#### B.2 WebSocket Handler Pattern (Axum)

```rust
/// Handle WebSocket upgrade request
/// # 4-Word Name: handle_workspace_live_stream_upgrade
pub async fn handle_workspace_live_stream_upgrade(
    ws: WebSocketUpgrade,
    Path(workspace_id): Path<WorkspaceUniqueIdentifierType>,
    State(state): State<SharedWorkspaceStateContainer>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        handle_websocket_connection_lifecycle(socket, workspace_id, state)
    })
}

/// Manage WebSocket connection lifecycle
/// # 4-Word Name: handle_websocket_connection_lifecycle
async fn handle_websocket_connection_lifecycle(
    socket: WebSocket,
    workspace_id: WorkspaceUniqueIdentifierType,
    state: SharedWorkspaceStateContainer,
) {
    let (sender, receiver) = socket.split();

    // Wrap sender for broadcasting
    let sender = Arc::new(Mutex::new(sender));

    // Register connection
    state.register_websocket_connection_sender(&workspace_id, sender.clone()).await;

    // Spawn read task
    let read_task = tokio::spawn(handle_incoming_websocket_messages(
        receiver,
        workspace_id.clone(),
        state.clone(),
    ));

    // Wait for connection close
    let _ = read_task.await;

    // Cleanup connection
    state.remove_websocket_connection_sender(&workspace_id, sender).await;
}
```

#### B.3 File Watcher Integration

```rust
/// File watcher service using notify crate
/// # 4-Word Name: FileWatcherServiceImpl
pub struct FileWatcherServiceImpl {
    watcher_instance_handle: RecommendedWatcher,
    debounce_duration_milliseconds: u64,
    pending_events_queue_arc: Arc<RwLock<Vec<DebouncedEvent>>>,
}

impl FileWatcherServiceImpl {
    /// Create new file watcher for workspace
    /// # 4-Word Name: create_watcher_for_workspace
    pub fn create_watcher_for_workspace(
        workspace: &WorkspaceMetadataPayloadStruct,
        event_sender: mpsc::Sender<WorkspaceFileChangeEvent>,
    ) -> Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();

        let watcher = notify::recommended_watcher(move |res| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        })?;

        // Spawn debounce processor task
        tokio::spawn(async move {
            process_debounced_file_events(rx, event_sender, 500).await;
        });

        Ok(Self { ... })
    }
}
```

---

### Workstream C: React Frontend

#### C.1 Technology Stack

| Library | Version | Purpose |
|---------|---------|---------|
| React | 18.x | UI framework |
| TypeScript | 5.x | Type safety |
| react-force-graph-3d | 1.24.x | 3D force-directed graph |
| @react-three/fiber | 8.x | React Three.js bindings |
| @react-three/drei | 9.x | Three.js helpers |
| zustand | 4.x | State management |
| socket.io-client | 4.x | WebSocket client |

#### C.2 Data Transformation (API to react-force-graph-3d)

```typescript
// API response format (from Phase 1)
interface DiffVisualizationResponse {
  visualization: {
    nodes: Array<{
      id: string;
      label: string;
      node_type: string;
      change_type: "added" | "removed" | "modified" | "affected" | null;
    }>;
    edges: Array<{
      source: string;
      target: string;
      edge_type: string;
    }>;
  };
}

// react-force-graph-3d expected format
interface ForceGraphData {
  nodes: Array<{
    id: string;
    name: string;
    val: number;       // Node size
    color: string;     // Node color based on change_type
    group: string;     // For clustering
  }>;
  links: Array<{
    source: string;
    target: string;
    color: string;     // Edge color
  }>;
}

// Transformer function (4-word naming)
function transform_diff_to_forcegraph(
  response: DiffVisualizationResponse
): ForceGraphData {
  const colorMap = {
    added: "#22c55e",     // green-500
    removed: "#ef4444",   // red-500
    modified: "#f59e0b",  // amber-500
    affected: "#3b82f6",  // blue-500
    null: "#6b7280",      // gray-500
  };

  return {
    nodes: response.visualization.nodes.map(node => ({
      id: node.id,
      name: node.label,
      val: node.change_type ? 15 : 5,  // Changed nodes are larger
      color: colorMap[node.change_type ?? "null"],
      group: node.node_type,
    })),
    links: response.visualization.edges.map(edge => ({
      source: edge.source,
      target: edge.target,
      color: "#4b5563",  // gray-600
    })),
  };
}
```

#### C.3 Component Hierarchy

```
App
├── WorkspaceProvider (Zustand store)
│   └── WorkspaceContext
│
├── Layout
│   ├── Sidebar
│   │   ├── WorkspaceList
│   │   │   └── WorkspaceListItem (per workspace)
│   │   │       ├── WatchToggle
│   │   │       └── DeleteButton
│   │   └── CreateWorkspaceButton
│   │
│   └── MainContent
│       ├── ToolbarHeader
│       │   ├── SearchBar
│       │   ├── FilterDropdown
│       │   └── CameraControlButtons
│       │
│       ├── DiffGraphCanvas (main 3D view)
│       │   ├── ForceGraph3D (react-force-graph-3d)
│       │   │   ├── NodeRenderer (custom)
│       │   │   └── LinkRenderer (custom)
│       │   └── ConnectionStatus (WebSocket indicator)
│       │
│       └── DetailsPanel (right drawer)
│           ├── SelectedNodeDetails
│           │   ├── EntityInfo
│           │   └── CodePreview
│           └── DiffSummaryStats
│
└── ToastNotifications (reindex events)
```

---

## 3. Technology Stack

### Backend (Rust/Axum)

| Dependency | Version | Purpose |
|------------|---------|---------|
| axum | 0.7 | HTTP framework + WebSocket |
| tokio | 1.0 | Async runtime |
| notify | 6.x | File system watching |
| serde | 1.0 | JSON serialization |
| chrono | 0.4 | Timestamps |
| uuid | 1.x | Workspace IDs |
| dirs | 5.x | Home directory resolution |

### Frontend (Node.js/React)

| Dependency | Version | Purpose |
|------------|---------|---------|
| react | 18.x | UI framework |
| typescript | 5.x | Type safety |
| vite | 5.x | Build tool |
| react-force-graph-3d | 1.24.x | 3D graph visualization |
| three | 0.160.x | 3D rendering (peer dep) |
| zustand | 4.x | State management |
| @tanstack/react-query | 5.x | Data fetching |
| tailwindcss | 3.x | Styling |

---

## 4. Data Flow: File Change to Visualization

```
1. FILE CHANGE DETECTED
   [User edits code file]
         │
         ▼
2. NOTIFY CRATE EVENT
   FileWatcherServiceImpl receives Create/Modify/Delete event
         │
         ▼
3. DEBOUNCE (500ms)
   Batch rapid events together
         │
         ▼
4. TRIGGER REINDEX
   Call pt01 streamer on changed files only (incremental)
   Update live.db with new entities/edges
         │
         ▼
5. COMPUTE DIFF
   EntityDifferImpl.compute_entity_diff_result(base.db, live.db)
   BlastRadiusCalculatorImpl.compute_combined_blast_radius()
   DiffVisualizationTransformerImpl.transform_diff_to_visualization()
         │
         ▼
6. BROADCAST VIA WEBSOCKET
   For each connected client to this workspace:
     sender.send(WebSocketServerMessageType::DiffUpdatePushNotification { ... })
         │
         ▼
7. FRONTEND RECEIVES UPDATE
   WebSocket message handler in React
   Update Zustand store with new diff data
         │
         ▼
8. REACT RE-RENDER
   ForceGraph3D component re-renders with new data
   Animated transitions for added/removed nodes
```

---

## 5. API Specifications

### 5.1 Workspace Endpoints

#### POST /workspace-create-from-path

**Request:**
```json
{
  "source_path_directory_value": "/path/to/codebase",
  "workspace_display_name_option": "My Project"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "endpoint": "/workspace-create-from-path",
  "workspace": {
    "workspace_identifier_value": "ws_20260123_143052_a1b2c3",
    "workspace_display_name": "My Project",
    "source_directory_path_value": "/path/to/codebase",
    "base_database_path_value": "rocksdb:~/.parseltongue/workspaces/ws_xxx/base.db",
    "live_database_path_value": "rocksdb:~/.parseltongue/workspaces/ws_xxx/live.db",
    "watch_enabled_flag_status": false,
    "created_timestamp_utc_value": "2026-01-23T14:30:52Z",
    "last_indexed_timestamp_option": "2026-01-23T14:30:55Z"
  }
}
```

**Errors:**
| Code | HTTP Status | Meaning |
|------|-------------|---------|
| `PATH_NOT_FOUND` | 400 | Source path doesn't exist |
| `PATH_NOT_DIRECTORY` | 400 | Source path is a file |
| `WORKSPACE_ALREADY_EXISTS` | 409 | Duplicate workspace for path |
| `INDEXING_FAILED` | 500 | pt01 streamer error |

#### GET /workspace-list-all

**Response (200 OK):**
```json
{
  "success": true,
  "endpoint": "/workspace-list-all",
  "workspaces": [
    { ... WorkspaceMetadataPayloadStruct ... },
    { ... }
  ],
  "total_workspace_count_value": 3
}
```

#### POST /workspace-watch-toggle

**Request:**
```json
{
  "workspace_identifier_target_value": "ws_20260123_143052_a1b2c3",
  "watch_enabled_desired_state": true
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "endpoint": "/workspace-watch-toggle",
  "workspace": {
    ... // Updated workspace with watch_enabled_flag_status: true
  }
}
```

**Errors:**
| Code | HTTP Status | Meaning |
|------|-------------|---------|
| `WORKSPACE_NOT_FOUND` | 404 | Workspace ID invalid |
| `WATCHER_START_FAILED` | 500 | notify crate error |

### 5.2 WebSocket Endpoint

#### WS /workspace-live-stream/{workspace_id}

**Connection URL:** `ws://localhost:7777/workspace-live-stream/ws_xxx`

**Client Messages:**
```json
// Subscribe (optional - auto-subscribed on connect)
{ "type": "subscribe", "workspace_id": "ws_xxx" }

// Heartbeat
{ "type": "ping" }
```

**Server Messages:**
```json
// Diff update (main payload)
{
  "type": "diff_update",
  "workspace_id": "ws_xxx",
  "diff": { ... DiffResultDataPayloadStruct ... },
  "visualization": { ... VisualizationGraphDataPayload ... },
  "timestamp": "2026-01-23T14:35:12Z"
}

// Reindex lifecycle
{ "type": "reindex_started", "workspace_id": "ws_xxx", "triggered_by": "file_change" }
{ "type": "reindex_completed", "workspace_id": "ws_xxx", "entities_indexed_count": 215, "duration_ms": 1234 }

// Heartbeat response
{ "type": "pong" }
```

---

## 6. Implementation Phases

### Phase 2.1: Workspace Management Backend (Week 1)

**Deliverables:**
1. `WorkspaceMetadataPayloadStruct` and related types in `parseltongue-core/src/workspace/types.rs`
2. `WorkspaceManagerService` in `parseltongue-core/src/workspace/`
3. Three HTTP handlers in `pt08-http-code-query-server/`
4. Storage layer for `~/.parseltongue/workspaces/`
5. Unit tests (target: 30 new tests)

**Files to Create:**
- `crates/parseltongue-core/src/workspace/mod.rs`
- `crates/parseltongue-core/src/workspace/types.rs`
- `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/workspace_create_handler.rs`
- `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/workspace_list_handler.rs`
- `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/workspace_watch_handler.rs`

### Phase 2.2: WebSocket Streaming Backend (Week 2)

**Deliverables:**
1. WebSocket handler using `axum::extract::ws`
2. `FileWatcherServiceImpl` using `notify` crate
3. Debounced event processing (500ms)
4. Broadcast mechanism for multiple clients
5. Integration tests (target: 20 new tests)

**Files to Create:**
- `crates/pt08-http-code-query-server/src/websocket/mod.rs`
- `crates/pt08-http-code-query-server/src/websocket/handler.rs`
- `crates/pt08-http-code-query-server/src/websocket/message_types.rs`
- `crates/pt08-http-code-query-server/src/file_watcher/mod.rs`
- `crates/pt08-http-code-query-server/src/file_watcher/debouncer.rs`

### Phase 2.3: React Frontend Core (Week 3)

**Deliverables:**
1. Vite + React + TypeScript project setup
2. Zustand store for workspace state
3. HTTP client hooks (react-query)
4. WebSocket connection manager
5. Basic UI layout with Tailwind

**Files to Create:**
- `frontend/package.json`
- `frontend/src/App.tsx`
- `frontend/src/stores/workspaceStore.ts`
- `frontend/src/hooks/useWorkspaces.ts`
- `frontend/src/hooks/useWorkspaceWebSocket.ts`
- `frontend/src/components/Layout.tsx`
- `frontend/src/components/Sidebar.tsx`

### Phase 2.4: 3D Visualization Integration (Week 4)

**Deliverables:**
1. react-force-graph-3d integration
2. Data transformation layer
3. Node/link styling based on change type
4. Camera controls (orbit, zoom, pan)
5. Node click/hover interactions
6. End-to-end integration tests

**Files to Create:**
- `frontend/src/components/DiffGraphCanvas.tsx`
- `frontend/src/components/CameraControls.tsx`
- `frontend/src/components/NodeDetails.tsx`
- `frontend/src/utils/transformDiffToForceGraph.ts`
- `frontend/src/components/Legend.tsx`

---

## 7. File/Folder Structure for New Code

```
parseltongue-dependency-graph-generator/
├── crates/
│   ├── parseltongue-core/
│   │   └── src/
│   │       ├── workspace/              # NEW
│   │       │   ├── mod.rs
│   │       │   ├── types.rs            # WorkspaceMetadataPayloadStruct
│   │       │   └── manager.rs          # WorkspaceManagerService
│   │       ├── diff/                   # EXISTING (Phase 1)
│   │       └── ...
│   │
│   └── pt08-http-code-query-server/
│       └── src/
│           ├── http_endpoint_handler_modules/
│           │   ├── mod.rs              # UPDATE: add new handlers
│           │   ├── workspace_create_handler.rs          # NEW
│           │   ├── workspace_list_handler.rs            # NEW
│           │   ├── workspace_watch_handler.rs           # NEW
│           │   └── ...existing handlers...
│           │
│           ├── websocket/              # NEW
│           │   ├── mod.rs
│           │   ├── handler.rs          # WebSocket upgrade + lifecycle
│           │   ├── message_types.rs    # Client/server message enums
│           │   └── broadcaster.rs      # Multi-client broadcast
│           │
│           ├── file_watcher/           # NEW
│           │   ├── mod.rs
│           │   ├── service.rs          # FileWatcherServiceImpl
│           │   └── debouncer.rs        # Event debouncing
│           │
│           ├── route_definition_builder_module.rs  # UPDATE: add new routes
│           └── http_server_startup_runner.rs       # UPDATE: init workspace state
│
└── frontend/                           # NEW (entire directory)
    ├── package.json
    ├── vite.config.ts
    ├── tsconfig.json
    ├── tailwind.config.js
    ├── index.html
    └── src/
        ├── main.tsx
        ├── App.tsx
        ├── stores/
        │   └── workspaceStore.ts
        ├── hooks/
        │   ├── useWorkspaces.ts
        │   ├── useWorkspaceWebSocket.ts
        │   └── useDiffVisualization.ts
        ├── components/
        │   ├── Layout.tsx
        │   ├── Sidebar/
        │   │   ├── index.tsx
        │   │   ├── WorkspaceList.tsx
        │   │   └── CreateWorkspaceButton.tsx
        │   ├── DiffGraphCanvas/
        │   │   ├── index.tsx
        │   │   ├── CameraControls.tsx
        │   │   └── ConnectionIndicator.tsx
        │   └── DetailsPanel/
        │       ├── index.tsx
        │       ├── NodeDetails.tsx
        │       └── DiffSummary.tsx
        ├── utils/
        │   └── transformDiffToForceGraph.ts
        └── types/
            └── api.ts
```

---

## 8. Dependency Updates Required

### Cargo.toml (Workspace Root)

```toml
[workspace.dependencies]
# Existing...

# Phase 2 additions
notify = "6.1"          # File system watching
uuid = { version = "1.6", features = ["v4", "serde"] }
dirs = "5.0"            # Cross-platform home directory
```

### pt08-http-code-query-server/Cargo.toml

```toml
[dependencies]
# Existing...

# Phase 2 additions
notify = { workspace = true }
uuid = { workspace = true }
dirs = { workspace = true }
```

---

## 9. Risk Mitigation

| Risk | Mitigation |
|------|------------|
| File watcher performance on large repos | Implement path filtering (ignore `node_modules`, `target`) |
| WebSocket connection drops | Client-side reconnection with exponential backoff |
| Browser memory with large graphs | LOD (Level of Detail) - simplify distant nodes |
| Rapid file saves cause reindex storms | 500ms debounce, batch multiple file changes |
| Cross-platform home directory | Use `dirs` crate instead of hardcoded paths |

---

## 10. Testing Strategy

### Backend Tests

| Module | Test Type | Target Count |
|--------|-----------|--------------|
| Workspace types | Unit | 15 |
| Workspace manager | Integration | 10 |
| HTTP handlers | Integration | 15 |
| WebSocket handler | Integration | 10 |
| File watcher | Integration | 10 |
| **Total** | | **60** |

### Frontend Tests

| Component | Test Type | Target Count |
|-----------|-----------|--------------|
| Data transformation | Unit | 10 |
| Zustand store | Unit | 8 |
| Hooks | Unit | 12 |
| Components | Component (RTL) | 15 |
| E2E (Playwright) | E2E | 5 |
| **Total** | | **50** |

---

## 11. Success Metrics

| Metric | Target |
|--------|--------|
| File change to visualization update latency | < 2 seconds |
| WebSocket message size (typical diff) | < 50KB |
| Frontend initial load time | < 3 seconds |
| Smooth 3D rendering (nodes) | Up to 1000 nodes at 60fps |
| Test coverage (new code) | > 80% |
| Total tests after Phase 2 | 350+ (244 existing + 110 new) |

---

## 12. Research Sources

### Three.js Force-Directed Graph
- [three-forcegraph](https://vasturiano.github.io/three-forcegraph/) - Core ThreeJS library
- [r3f-forcegraph](https://github.com/vasturiano/r3f-forcegraph) - React Three Fiber bindings
- [react-force-graph](https://vasturiano.github.io/react-force-graph/) - React component for 2D/3D graphs

### Rust WebSocket
- [Axum WebSocket Guide](https://docs.rs/axum/latest/axum/extract/ws/index.html) - Official axum::extract::ws docs
- [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite) - Underlying WebSocket library

### Dependency Graph Visualization
- [Emerge](https://github.com/glato/emerge) - Browser-based codebase visualization tool
- [3D Force Graph Examples](https://vasturiano.github.io/3d-force-graph/) - Interactive demos

---

*Architecture document created 2026-01-23*
*Phase 1 baseline: 244 tests passing*

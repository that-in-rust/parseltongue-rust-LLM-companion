# Rust-Analyzer Control Flow Analysis

**Analysis Date:** 2026-01-29
**Source:** Parseltongue analysis of rust-analyzer codebase

## Main Event Loop

### Entry Point: `main_loop`

Location: `crates/rust-analyzer/src/main_loop.rs:41-72`

```rust
pub fn main_loop(config: Config, connection: Connection) -> anyhow::Result<()>
```

**Key Operations:**
1. **Thread Priority Boost (Windows):** Sets main thread to `THREAD_PRIORITY_ABOVE_NORMAL` to prevent worker thread displacement
2. **DHAT Profiling Setup:** Optionally initializes heap profiling
3. **GlobalState Creation:** `GlobalState::new(connection.sender, config)`
4. **Event Loop Launch:** `GlobalState.run(connection.receiver)`

### Main Run Loop

Location: `crates/rust-analyzer/src/main_loop.rs:178-219`

**Initialization Sequence:**
1. `update_status_or_notify()` - Initial status report
2. **Dynamic Registration:** If configured, register didSave capability with file watchers
3. **Workspace Discovery:** Trigger initial workspace fetch if no discovery config

**Event Loop Structure:**
```rust
while let Ok(event) = self.next_event(&inbox) {
    // Exit check
    if matches!(Event::Lsp(Exit notification)) {
        return Ok(());
    }
    self.handle_event(event);
}
```

**Termination:**
- Clean exit on `Exit` notification
- Error if channel drops (panic detection)

## Event Types

The system processes 7 distinct event types:

### 1. Event::Lsp (LSP Messages)
**Sub-types:**
- `Message::Request` → `on_new_request()`
- `Message::Notification` → `on_notification()`
- `Message::Response` → `complete_request()`

### 2. Event::DeferredTask
**Handling:**
- Process database-dependent work deferred from sync handlers
- **Coalescing:** Drains queue with `try_recv()` to batch processing
- Runs after `process_changes()` for correctness

### 3. Event::Task
**Handling:**
- General background task completion
- **Special case:** Tracks `PrimeCachesProgress` for indexing reports
- **Coalescing:** Batches multiple task completions

**Prime Caches Progress Flow:**
```
PrimeCachesProgress::Begin
  ↓
[Multiple] PrimeCachesProgress::Report
  ↓ (batched, only last reported to client)
PrimeCachesProgress::End
  ↓
trigger_garbage_collection()
```

### 4. Event::Vfs (Virtual File System)
**Handling:**
- File system change notifications
- **Coalescing:** Batches file events for efficiency
- **Progress Reporting:** "Roots Scanned" with fraction complete

### 5. Event::Flycheck (Cargo Check)
**Handling:**
- Diagnostic messages from `cargo check`
- **Coalescing:** Batches flycheck updates
- **Completion Action:** Triggers `WorkspaceDiagnosticRefresh` on cargo finish

### 6. Event::TestResult
**Handling:**
- Test execution results
- **Coalescing:** Batches test result events

### 7. Event::DiscoverProject
**Handling:**
- Project discovery messages
- **Coalescing:** Batches discovery events

### 8. Event::FetchWorkspaces
**Handling:**
- Queue workspace fetch operation
- Stores reason: "project structure change"

## Event Handling Flow

### Event Processing Pipeline

```
┌─────────────────────────────────────────────────────────────┐
│ 1. next_event() - Wait for/select next event               │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. handle_event() - Process specific event type            │
│    - Record loop_start timestamp                           │
│    - Enter tracing span                                    │
│    - Check if system was_quiescent                         │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. Event-Specific Handler                                  │
│    - on_new_request / on_notification / etc.               │
│    - Coalesce similar events from channel                  │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. Post-Event Processing (if vfs_done)                     │
│    a. switch_workspaces() if requested                     │
│    b. process_changes() - Apply VFS/state changes          │
│    c. mem_docs.take_changes()                              │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. Quiescence Detection & Actions                          │
│    - is_quiescent() checks if all work complete            │
│    - If became_quiescent:                                  │
│      • Trigger initial flycheck                            │
│      • Request prime_caches if proc macros loaded          │
│    - If client_refresh needed:                             │
│      • SemanticTokensRefresh                               │
│      • CodeLensRefresh                                     │
│      • InlayHintRefreshRequest                             │
│      • WorkspaceDiagnosticRefresh                          │
│    - If project/docs changed:                              │
│      • update_diagnostics()                                │
│      • update_tests()                                      │
│    - Garbage collection if idle                            │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ 6. Housekeeping                                            │
│    - cleanup_discover_handles()                            │
│    - Publish diagnostic changes                            │
│    - Start queued operations:                              │
│      • fetch_workspaces                                    │
│      • fetch_build_data                                    │
│      • fetch_proc_macros                                   │
│      • prime_caches                                        │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│ 7. Performance Monitoring                                  │
│    - update_status_or_notify()                             │
│    - Warn if loop > 100ms when quiescent                   │
│    - Log event handling duration                           │
└─────────────────────────────────────────────────────────────┘
```

## LSP Request Handling

### Request Path

Location: `crates/rust-analyzer/src/main_loop.rs:1229-1234` (`on_new_request`)

**Flow:**
1. **Cancellation Check:** If shutdown requested, respond with server error
2. **Request Registration:** Add to `req_queue` for tracking
3. **Dispatch:** Route to appropriate handler via `on_request()`

### Request Routing

Location: `crates/rust-analyzer/src/main_loop.rs:1237-1351` (`on_request`)

**Handler Categories:**
- **Immediate Handlers:** Process in main loop (fast queries)
- **Deferred Handlers:** Queue to task pool (expensive queries)
- **Special Handlers:** Custom logic (e.g., shutdown, cancelRequest)

### Notification Handling

Location: `crates/rust-analyzer/src/main_loop.rs:1354-1381` (`on_notification`)

**Common Notifications:**
- `DidOpenTextDocument` - New file opened
- `DidChangeTextDocument` - File edited
- `DidSaveTextDocument` - File saved
- `DidCloseTextDocument` - File closed
- `DidChangeConfiguration` - Settings changed
- `DidChangeWatchedFiles` - File system changes

## State Transitions

### Quiescence State Machine

```
┌──────────────┐
│   LOADING    │  (vfs_done = false, work in progress)
└──────┬───────┘
       │ VFS complete
       │ All tasks done
       ▼
┌──────────────┐
│  QUIESCENT   │  (became_quiescent = true)
└──────┬───────┘
       │ Triggers:
       │ - Initial flycheck
       │ - Prime caches
       │ - Client refreshes
       │
       │ New work arrives
       ▼
┌──────────────┐
│    BUSY      │  (is_quiescent() = false)
└──────┬───────┘
       │ Work completes
       │
       └────→ Back to QUIESCENT
```

### Workspace Switch State Machine

```
┌────────────────┐
│  CURRENT WS    │
└────────┬───────┘
         │ wants_to_switch = Some(cause)
         │ vfs_done = true
         ▼
┌────────────────┐
│ switch_work    │
│   spaces()     │
└────────┬───────┘
         │ - Load new crate graph
         │ - Update proc macros
         │ - Invalidate caches
         ▼
┌────────────────┐
│    NEW WS      │
└────────────────┘
```

## Coalescing Pattern

**Purpose:** Batch similar events to reduce overhead

**Implementation:**
```rust
// Process first event
self.handle_X(event);

// Drain queue of similar events
while let Ok(event) = self.X_receiver.try_recv() {
    self.handle_X(event);
}
```

**Used for:**
- Task completions
- VFS events
- Flycheck messages
- Test results
- Project discovery
- Deferred tasks

**Benefit:** Reduces:
- LSP notification overhead
- Progress report spam
- State update cycles
- Diagnostic publishing

## Performance Optimizations

### 1. Loop Duration Monitoring
- Warns if single loop iteration > 100ms (when quiescent)
- Logs event type and duration
- Helps identify bottlenecks

### 2. Lazy Garbage Collection
- Only GC when:
  - System is quiescent
  - Both task pools are empty
  - Revision has changed since last GC

### 3. Debounced Refreshes
- Client refreshes only on state change or quiescence
- Batches multiple refresh requests

### 4. Progress Report Batching
- For prime caches: accumulates reports, sends only last before completion
- For VFS: sends single consolidated progress

## Deferred Work Queue

**Purpose:** Move database-dependent work out of sync handlers

**Scheduling:**
```rust
self.deferred_task_queue.sender.send(task);
```

**Execution:**
- Runs in Event::DeferredTask handler
- **Critical:** Executes AFTER `process_changes()`
- Ensures database consistency

**Use Cases:**
- `DiscoverProject` handler (crate graph dependent)
- Operations requiring fresh analysis state

## Operation Queues

### OpQueue Pattern

**Usage:**
- `fetch_workspaces_queue`
- `fetch_build_data_queue`
- `fetch_proc_macros_queue`
- `prime_caches_queue`

**Lifecycle:**
1. `request_op(reason, request)` - Queue operation
2. `should_start_op()` - Check if ready to start
3. (Perform operation)
4. `op_completed(result)` - Mark done

**Sequencing:**
```
fetch_workspaces (cargo metadata)
   ↓ (waits for completion)
fetch_build_data (build scripts)
   ↓ (waits for completion)
fetch_proc_macros (compile proc macros)
   ↓ (waits for completion)
prime_caches (warm up analysis)
```

**Invariant:** Only one workspace operation at a time

## Key Control Flow Patterns

### 1. Event Coalescing
- Drain channels with `try_recv()`
- Process similar events in batch
- Reduce notification overhead

### 2. State-Dependent Execution
- `vfs_done` gates workspace switching
- `is_quiescent()` triggers optimizations
- `became_quiescent` performs one-time actions

### 3. Progress Reporting
- Track operation phases (Begin/Report/End)
- Batch intermediate reports
- Send final update on completion

### 4. Deferred Execution
- Defer database-dependent work
- Ensure correct execution order
- Maintain consistency

### 5. Conditional Refresh
- Client capabilities checked before refresh requests
- State changes trigger targeted refreshes
- Avoid unnecessary work

## Error Handling

### Graceful Degradation
- Continue on non-fatal errors
- Report diagnostics for failures
- Maintain partial functionality

### Panic Detection
- Channel drops indicate panics
- Return error with diagnostic message
- Helps debugging background task failures

### Timeout Protection
- Loop duration monitoring
- Warns on long iterations
- Helps identify performance regressions

## Next Steps

See related documentation:
- `03-DATA-FLOW.md` - Data transformations and state updates
- `04-KEY-COMPONENTS.md` - Deep dive into major subsystems

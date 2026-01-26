# Incremental Indexing Architecture

> **Document ID**: D04
> **Status**: Design Complete
> **Last Updated**: 2026-01-26
> **Author**: Claude Code Analysis

---

## Executive Summary

This document provides a comprehensive architectural analysis of Parseltongue's incremental indexing system for real-time dependency graph updates. The current architecture is **fundamentally correct** but has **one critical stub** preventing live updates from working.

**Key Finding**: The `trigger_incremental_reindex_update()` function at `watcher_service.rs:292-313` is a stub that returns `Ok()` without performing any work.

---

## Table of Contents

1. [Current Architecture Overview](#current-architecture-overview)
2. [Component Deep Dive](#component-deep-dive)
3. [Control Flow Analysis](#control-flow-analysis)
4. [The Critical Gap](#the-critical-gap)
5. [Data Flow Diagrams](#data-flow-diagrams)
6. [ISGL1 v2: Stable Entity Identity](#isgl1-v2-stable-entity-identity)
7. [Entity Matching Algorithm](#entity-matching-algorithm)
8. [Simulation Scenarios](#simulation-scenarios)
9. [Implementation Status Matrix](#implementation-status-matrix)
10. [Three Alternative Approaches](#three-alternative-approaches)
11. [Recommendation](#recommendation)
12. [Critical Files Reference](#critical-files-reference)

---

## Current Architecture Overview

### High-Level System Architecture

```mermaid
flowchart TB
    subgraph External["External Triggers"]
        FS[("File System")]
        USER["User/AI Editor"]
    end

    subgraph FileWatcher["File Watcher Layer"]
        NOTIFY["notify v6.1<br/>Cross-Platform Watcher"]
        DEBOUNCE["Debouncer<br/>500ms Window"]
        FILTER["Path Filter<br/>Ignore patterns"]
    end

    subgraph Core["Parseltongue Core"]
        TRIGGER["trigger_incremental_reindex_update()<br/>⚠️ STUB"]
        PARSER["Tree-sitter Parser"]
        STORAGE["CozoDbStorage<br/>RocksDB Backend"]
        DIFF["Diff Engine"]
    end

    subgraph Output["Output Layer"]
        WS["WebSocket Server"]
        HTTP["HTTP API<br/>:7777"]
        REACT["React Frontend<br/>3D Graph"]
    end

    USER --> FS
    FS --> NOTIFY
    NOTIFY --> DEBOUNCE
    DEBOUNCE --> FILTER
    FILTER --> TRIGGER
    TRIGGER -.->|"NOT IMPLEMENTED"| PARSER
    PARSER -.-> STORAGE
    STORAGE -.-> DIFF
    DIFF -.-> WS
    WS --> REACT
    HTTP --> REACT

    style TRIGGER fill:#ff6b6b,stroke:#c92a2a,color:#fff
    style PARSER fill:#ffd43b,stroke:#fab005
    style STORAGE fill:#69db7c,stroke:#37b24d
    style WS fill:#69db7c,stroke:#37b24d
```

### Database Architecture (base.db vs live.db)

```mermaid
flowchart LR
    subgraph Snapshot["Initial Indexing"]
        FULL["Full Codebase Scan"]
        BASE[("base.db<br/>Baseline Snapshot")]
    end

    subgraph Live["Live Updates"]
        WATCH["File Watcher"]
        INCR["Incremental Parser"]
        LIVE[("live.db<br/>Current State")]
    end

    subgraph Diff["Diff Computation"]
        COMPARE["Compare<br/>base.db vs live.db"]
        RESULT["DiffResult<br/>Added/Removed/Modified"]
    end

    FULL --> BASE
    WATCH --> INCR
    INCR --> LIVE
    BASE --> COMPARE
    LIVE --> COMPARE
    COMPARE --> RESULT

    style INCR fill:#ff6b6b,stroke:#c92a2a,color:#fff
```

---

## Component Deep Dive

### 1. File Watcher Service

**Location**: `crates/pt08-http-code-query-server/src/file_watcher_service_module/`

```mermaid
flowchart TD
    subgraph WatcherService["watcher_service.rs"]
        CREATE["create_watcher_for_workspace()"]
        START["start_watching_workspace_directory()"]
        STOP["stop_watching_workspace_directory()"]
        TRIGGER["trigger_incremental_reindex_update()"]
        BROADCAST["broadcast_diff_to_subscribers()"]
        CONVERT["convert_notify_event_data()"]
    end

    subgraph Status["Implementation Status"]
        C_OK["✅ Lines 118-191"]
        S_OK["✅ Lines 205-251"]
        ST_OK["✅ Lines 265-290"]
        T_STUB["❌ Lines 292-313<br/>STUB - Returns Ok()"]
        B_OK["✅ Lines 326-342"]
        CV_OK["✅ Lines 351-376"]
    end

    CREATE --- C_OK
    START --- S_OK
    STOP --- ST_OK
    TRIGGER --- T_STUB
    BROADCAST --- B_OK
    CONVERT --- CV_OK

    style T_STUB fill:#ff6b6b,stroke:#c92a2a,color:#fff
```

### 2. Debouncer Module

**Location**: `crates/pt08-http-code-query-server/src/file_watcher_service_module/debouncer.rs`

```mermaid
sequenceDiagram
    participant FS as File System
    participant W as Watcher
    participant D as Debouncer
    participant T as Trigger

    Note over D: 500ms aggregation window

    FS->>W: File Change Event 1
    W->>D: notify::Event
    Note over D: Start timer

    FS->>W: File Change Event 2
    W->>D: notify::Event
    Note over D: Reset timer

    FS->>W: File Change Event 3
    W->>D: notify::Event
    Note over D: Reset timer

    Note over D: 500ms elapsed...

    D->>T: Aggregated [Event1, Event2, Event3]
    Note over T: Process batch of changes
```

### 3. Path Filter Module

**Location**: `crates/pt08-http-code-query-server/src/file_watcher_service_module/path_filter.rs`

```mermaid
flowchart TD
    INPUT["Incoming File Path"]

    subgraph Filters["Path Filters"]
        F1["target/"]
        F2["node_modules/"]
        F3[".git/"]
        F4["*.lock"]
        F5["*.db/"]
        F6[".DS_Store"]
    end

    DECISION{{"Should Process?"}}
    ACCEPT["✅ Process File"]
    REJECT["❌ Ignore File"]

    INPUT --> DECISION
    DECISION -->|"Not in ignore list"| ACCEPT
    DECISION -->|"Matches ignore pattern"| REJECT

    Filters --> DECISION
```

### 4. WebSocket Streaming Module

**Location**: `crates/pt08-http-code-query-server/src/websocket_streaming_module/`

```mermaid
flowchart LR
    subgraph Server["HTTP Server :7777"]
        WS_ENDPOINT["/websocket-diff-stream"]
        UPGRADE["WebSocket Upgrade"]
        HANDLER["handle_websocket_diff_stream_upgrade()"]
    end

    subgraph Streaming["Streaming Infrastructure"]
        SUBSCRIBERS[("Subscriber Map<br/>Arc&lt;RwLock&gt;")]
        BROADCAST["broadcast_diff_to_subscribers()"]
    end

    subgraph Clients["Connected Clients"]
        C1["React Frontend"]
        C2["Tauri Desktop"]
        C3["Other Consumers"]
    end

    WS_ENDPOINT --> UPGRADE
    UPGRADE --> HANDLER
    HANDLER --> SUBSCRIBERS
    BROADCAST --> SUBSCRIBERS
    SUBSCRIBERS --> C1
    SUBSCRIBERS --> C2
    SUBSCRIBERS --> C3
```

### 5. CozoDbStorage (Database Layer)

**Location**: `crates/parseltongue-core/src/storage/cozo_client.rs`

```mermaid
flowchart TB
    subgraph Operations["CRUD Operations"]
        INSERT["insert_entity()<br/>Lines 774-801"]
        DELETE["delete_entity()"]
        UPDATE["update_entity()"]
        GET["get_entity()"]
        CHANGED["get_changed_entities()<br/>Lines 884-911"]
    end

    subgraph Storage["RocksDB Backend"]
        ENTITIES[("entities<br/>relation")]
        EDGES[("edges<br/>relation")]
        META[("metadata<br/>relation")]
    end

    INSERT --> ENTITIES
    INSERT --> EDGES
    DELETE --> ENTITIES
    DELETE --> EDGES
    UPDATE --> ENTITIES
    GET --> ENTITIES
    CHANGED --> ENTITIES
```

---

## Control Flow Analysis

### Complete Data Flow (Current State)

```mermaid
flowchart TD
    subgraph Trigger["1. File Change Trigger"]
        A1["User saves file<br/>(e.g., src/auth.rs)"]
    end

    subgraph Watch["2. File Watching"]
        B1["notify v6.1 detects change"]
        B2["Debouncer aggregates<br/>(500ms window)"]
        B3["Path filter checks<br/>(not in ignore list)"]
    end

    subgraph Process["3. Processing ⚠️ GAP HERE"]
        C1["trigger_incremental_reindex_update()"]
        C2["❌ STUB: Returns Ok()"]
        C3["Should: Re-parse file"]
        C4["Should: Extract entities"]
        C5["Should: Compute delta"]
        C6["Should: Update live.db"]
    end

    subgraph Output["4. Output (Never Reached)"]
        D1["Compute diff<br/>(base.db vs live.db)"]
        D2["broadcast_diff_to_subscribers()"]
        D3["WebSocket push"]
        D4["React frontend updates"]
    end

    A1 --> B1
    B1 --> B2
    B2 --> B3
    B3 --> C1
    C1 --> C2
    C2 -.->|"NOT IMPLEMENTED"| C3
    C3 -.-> C4
    C4 -.-> C5
    C5 -.-> C6
    C6 -.-> D1
    D1 -.-> D2
    D2 -.-> D3
    D3 -.-> D4

    style C1 fill:#ff6b6b,stroke:#c92a2a,color:#fff
    style C2 fill:#ff6b6b,stroke:#c92a2a,color:#fff
    style C3 fill:#ffd43b,stroke:#fab005
    style C4 fill:#ffd43b,stroke:#fab005
    style C5 fill:#ffd43b,stroke:#fab005
    style C6 fill:#ffd43b,stroke:#fab005
```

### Expected Control Flow (After Implementation)

```mermaid
flowchart TD
    subgraph Input["Input"]
        IN1["File Change Event"]
        IN2["changed_files: Vec&lt;PathBuf&gt;"]
    end

    subgraph Parse["Re-Parse Phase"]
        P1["For each changed file"]
        P2["Read file content"]
        P3["tree-sitter parse"]
        P4["Extract entities + edges"]
    end

    subgraph Delta["Delta Computation"]
        D1["Query existing entities<br/>from this file"]
        D2["Compare old vs new"]
        D3["Identify: Added"]
        D4["Identify: Removed"]
        D5["Identify: Modified"]
    end

    subgraph Update["Database Update"]
        U1["DELETE removed entities"]
        U2["DELETE removed edges"]
        U3["INSERT new entities"]
        U4["INSERT new edges"]
        U5["UPDATE modified entities"]
    end

    subgraph Broadcast["Broadcast"]
        B1["Compute full diff<br/>(base.db vs live.db)"]
        B2["Build DiffNotification"]
        B3["WebSocket broadcast"]
    end

    IN1 --> IN2
    IN2 --> P1
    P1 --> P2
    P2 --> P3
    P3 --> P4

    P4 --> D1
    D1 --> D2
    D2 --> D3
    D2 --> D4
    D2 --> D5

    D3 --> U3
    D4 --> U1
    D4 --> U2
    D5 --> U5
    U3 --> U4

    U1 --> B1
    U2 --> B1
    U4 --> B1
    U5 --> B1

    B1 --> B2
    B2 --> B3

    style P3 fill:#69db7c,stroke:#37b24d
    style D2 fill:#ffd43b,stroke:#fab005
    style B3 fill:#69db7c,stroke:#37b24d
```

---

## The Critical Gap

### The Stub Function

**File**: `crates/pt08-http-code-query-server/src/file_watcher_service_module/watcher_service.rs`
**Lines**: 292-313

```rust
/// Trigger incremental reindex update
///
/// # 4-Word Name: trigger_incremental_reindex_update
///
/// Triggers an incremental reindex of the changed files and updates live.db.
///
/// ## Contract
/// - WHEN trigger_incremental_reindex_update is called
///   WITH changed_file_paths containing N files
/// - THEN SHALL update live.db with changed files
///   AND SHALL broadcast DiffAnalysisStartedNotification
///   AND SHALL return diff result on completion
pub async fn trigger_incremental_reindex_update(
    _state: &SharedApplicationStateContainer,
    _workspace_id: &str,
    _changed_files: &[PathBuf],
) -> Result<(), FileWatcherErrorType> {
    // TODO: Implement incremental reindex
    // This will be implemented in the GREEN phase
    // For now, this is a stub that returns Ok
    Ok(())
}
```

### What This Stub Should Do

```mermaid
flowchart TD
    subgraph Input["Function Input"]
        I1["state: SharedApplicationStateContainer"]
        I2["workspace_id: &str"]
        I3["changed_files: &[PathBuf]"]
    end

    subgraph Steps["Required Implementation Steps"]
        S1["1. Get live.db handle from state"]
        S2["2. For each file in changed_files:"]
        S3["   2a. Query existing entities by file_path"]
        S4["   2b. Re-parse file using tree-sitter"]
        S5["   2c. Generate new entities + edges"]
        S6["   2d. Compute delta (old vs new)"]
        S7["   2e. Apply delta to live.db"]
        S8["3. Compute diff (base.db vs live.db)"]
        S9["4. Broadcast diff via WebSocket"]
    end

    subgraph Output["Function Output"]
        O1["Result&lt;(), FileWatcherErrorType&gt;"]
    end

    I1 --> S1
    I2 --> S1
    I3 --> S2
    S1 --> S2
    S2 --> S3
    S3 --> S4
    S4 --> S5
    S5 --> S6
    S6 --> S7
    S7 --> S8
    S8 --> S9
    S9 --> O1
```

---

## Data Flow Diagrams

### Entity Lifecycle During Incremental Update

```mermaid
stateDiagram-v2
    [*] --> Indexed: Initial full index

    Indexed --> Watching: File watcher started

    Watching --> FileChanged: notify event

    FileChanged --> Debounced: After 500ms

    Debounced --> Filtered: Path filter passes

    Filtered --> Reparsed: Tree-sitter parse

    Reparsed --> DeltaComputed: Compare old vs new

    DeltaComputed --> DatabaseUpdated: Apply changes

    DatabaseUpdated --> DiffBroadcast: WebSocket push

    DiffBroadcast --> Watching: Ready for next change

    note right of Filtered
        Currently stuck here!
        trigger_incremental_reindex_update()
        is a stub
    end note
```

### ISGL1 Key Generation Flow

```mermaid
flowchart LR
    subgraph Input["Parsed Entity"]
        LANG["language: rust"]
        TYPE["entity_type: fn"]
        NAME["name: handle_auth"]
        FILE["file: src/auth.rs"]
        LINES["lines: 42-67"]
    end

    subgraph Generator["ISGL1 Generator"]
        NORM["Normalize path"]
        ENCODE["URL-safe encode"]
        CONCAT["Concatenate parts"]
    end

    subgraph Output["ISGL1 Key"]
        KEY["rust:fn:handle_auth:__src_auth_rs:42-67"]
    end

    LANG --> CONCAT
    TYPE --> CONCAT
    NAME --> CONCAT
    FILE --> NORM
    NORM --> ENCODE
    ENCODE --> CONCAT
    LINES --> CONCAT
    CONCAT --> KEY
```

### Dependency Edge Types

```mermaid
flowchart TD
    subgraph Entity["Source Entity"]
        SRC["rust:fn:process_request"]
    end

    subgraph EdgeTypes["Edge Types"]
        CALLS["Calls"]
        USES["Uses"]
        IMPORTS["Imports"]
        IMPLEMENTS["Implements"]
        EXTENDS["Extends"]
    end

    subgraph Targets["Target Entities"]
        T1["rust:fn:validate_input"]
        T2["rust:struct:Request"]
        T3["rust:mod:handlers"]
        T4["rust:trait:Handler"]
        T5["rust:struct:BaseHandler"]
    end

    SRC -->|"Calls"| T1
    SRC -->|"Uses"| T2
    SRC -->|"Imports"| T3
    SRC -->|"Implements"| T4
    SRC -->|"Extends"| T5
```

---

## ISGL1 v2: Stable Entity Identity

### The Problem with Line-Number Based Keys

The current ISGL1 key format includes line numbers:

```
rust:fn:handle_auth:__src_auth_rs:42-67
                                   ↑↑↑↑↑
                               LINE RANGE
```

**Critical Issue**: When you add lines to one function, ALL subsequent functions shift:

```mermaid
flowchart LR
    subgraph Before["BEFORE: Add 5 lines to handle_auth"]
        B1["fn handle_auth()<br/>lines 10-20<br/>KEY: ...rs:10-20"]
        B2["fn validate_token()<br/>lines 22-40<br/>KEY: ...rs:22-40"]
        B3["fn refresh_token()<br/>lines 42-60<br/>KEY: ...rs:42-60"]
    end

    subgraph After["AFTER: Lines shifted!"]
        A1["fn handle_auth()<br/>lines 10-25<br/>KEY: ...rs:10-25 ✗"]
        A2["fn validate_token()<br/>lines 27-45<br/>KEY: ...rs:27-45 ✗"]
        A3["fn refresh_token()<br/>lines 47-65<br/>KEY: ...rs:47-65 ✗"]
    end

    Before --> After

    style A1 fill:#ff6b6b,stroke:#c92a2a,color:#fff
    style A2 fill:#ff6b6b,stroke:#c92a2a,color:#fff
    style A3 fill:#ff6b6b,stroke:#c92a2a,color:#fff
```

**Consequences**:
1. All 3 ISGL1 keys change, even though only `handle_auth` was modified
2. All incoming edges break (they reference non-existent keys)
3. Diff shows phantom changes (3 "modified" when only 1 actually changed)

### The Solution: Birth Timestamp as Permanent Identity

**Primary Key = `semantic_path` × `birth_timestamp`**

```
rust:fn:process:Foo:__src_rs:T1_001
└──────────────┬───────────┘ └──┬──┘
          semantic_path      birth_ts
```

**Key Properties**:
1. **Unique**: semantic_path + timestamp is always unique
2. **Stable**: Once assigned, NEVER changes (even if content/lines change)
3. **Human-readable**: Still has the function name
4. **Ordered**: T1_001 came before T1_002

### ISGL1 v2 Key Format

```mermaid
flowchart LR
    subgraph Input["Parsed Entity"]
        LANG["language: rust"]
        TYPE["entity_type: fn"]
        NAME["name: handle_auth"]
        PARENT["parent: Foo"]
        FILE["file: src/auth.rs"]
        BIRTH["birth_ts: T1_001"]
    end

    subgraph Generator["ISGL1 v2 Generator"]
        BUILD["Build semantic path"]
        LOOKUP["Lookup or assign birth_ts"]
        CONCAT["Concatenate"]
    end

    subgraph Output["ISGL1 v2 Key"]
        KEY["rust:fn:handle_auth:Foo:__src_auth_rs:T1_001"]
    end

    LANG --> BUILD
    TYPE --> BUILD
    NAME --> BUILD
    PARENT --> BUILD
    FILE --> BUILD
    BUILD --> LOOKUP
    BIRTH --> LOOKUP
    LOOKUP --> CONCAT
    CONCAT --> KEY

    style KEY fill:#69db7c,stroke:#37b24d
```

### Entity Schema (v2)

```
Entity {
    // PRIMARY KEY (immutable after creation)
    key: "rust:fn:process:Foo:__src_rs:T1_001",

    // Semantic identity (for matching)
    semantic_path: "rust:fn:process:Foo:__src_rs",

    // Mutable metadata (can change without changing key)
    content_hash: "sha256_abc123",
    line_start: 42,
    line_end: 67,
    last_modified: "2026-01-26T09:30:00Z",
    birth_timestamp: "T1_001",
}
```

---

## Entity Matching Algorithm

### The Challenge: Matching Entities Across Re-Index

When a file is re-parsed, we must match newly parsed entities to existing ones in the database:

```mermaid
flowchart TD
    subgraph NewParse["Newly Parsed Entities"]
        N1["fn process() { A }"]
        N2["fn process() { B }"]
        N3["fn validate() { C }"]
    end

    subgraph Existing["Existing Entities in DB"]
        E1["T1_001: fn process() { A }"]
        E2["T1_002: fn process() { A }"]
        E3["T1_003: fn validate() { C }"]
    end

    subgraph Matching["Matching Algorithm"]
        M1["1. Match by content_hash"]
        M2["2. Match by position"]
        M3["3. Assign new timestamp"]
    end

    NewParse --> Matching
    Existing --> Matching

    style M1 fill:#69db7c,stroke:#37b24d
    style M2 fill:#ffd43b,stroke:#fab005
    style M3 fill:#4dabf7,stroke:#228be6
```

### Matching Priority

```
FOR each entity in newly parsed file:
    1. Find candidates by semantic_path (name + parent + file)

    2. IF candidate with matching content_hash exists:
       → MATCH (same entity, key unchanged)

    3. ELSE IF candidates exist but no hash match:
       → Match by closest line position
       → Mark as MODIFIED (content changed)

    4. ELSE (no candidates):
       → NEW entity, assign new birth timestamp

    5. Any old entity with no match → DELETED
```

---

## Simulation Scenarios

### Simulation 1: New Function Inserted BEFORE Duplicates

```
BEFORE:                              AFTER:
Line 10: fn process() { return 1 }   Line 5:  fn new_func() { ... }     ← NEW
         T1_001, hash=H1             Line 15: fn process() { return 1 } ← shifted
Line 20: fn process() { return 2 }            T1_001, hash=H1 ✓
         T1_002, hash=H2             Line 25: fn process() { return 2 } ← shifted
                                              T1_002, hash=H2 ✓
```

**Result**: Content hashes H1 and H2 are unique → **Easy match, keys stable**

---

### Simulation 2: Identical Duplicates, New Function Inserted

```
BEFORE:                              AFTER:
Line 10: fn process() { return 1 }   Line 5:  fn new_func() { ... }     ← NEW (T2_001)
         T1_001, hash=H1             Line 15: fn process() { return 1 } ← T1_001 or T1_002?
Line 20: fn process() { return 1 }            hash=H1
         T1_002, hash=H1 (SAME!)     Line 25: fn process() { return 1 } ← T1_001 or T1_002?
                                              hash=H1
```

**Challenge**: Both have same hash H1 → **Order-based matching**
- First `process` → T1_001
- Second `process` → T1_002

---

### Simulation 3: Identical Duplicates SWAPPED

```
BEFORE:                              AFTER (user swapped them):
Line 10: fn process() { A }  T1_001  Line 10: fn process() { A }  ← was T1_002?
Line 20: fn process() { A }  T1_002  Line 20: fn process() { A }  ← was T1_001?
```

**Reality**: **Undetectable**. But semantically, if they're byte-for-byte identical, does it matter which timestamp they have? They're functionally interchangeable.

---

### Simulation 4: One Duplicate Modified

```
BEFORE:                              AFTER:
Line 10: fn process() { A }  T1_001  Line 10: fn process() { A }  ← hash H1 → T1_001 ✓
         hash=H1                     Line 20: fn process() { B }  ← hash H2 (changed!)
Line 20: fn process() { A }  T1_002           T1_002 (matched by position)
         hash=H1                              marked as MODIFIED
```

**Result**: First matches by hash. Second has no hash match → **Position-based, mark MODIFIED**

---

### Simulation 5: New Duplicate Inserted BETWEEN

```
BEFORE:                              AFTER:
Line 10: fn process() { A }  T1_001  Line 10: fn process() { A }  ← hash → T1_001
         hash=H1                     Line 20: fn process() { C }  ← NEW (T2_001)
Line 30: fn process() { B }  T1_002           hash=H3 (no match)
         hash=H2                     Line 30: fn process() { B }  ← hash → T1_002
```

**Result**: Hash matching works perfectly. New entity gets new timestamp.

---

### Simulation 6: Middle Entity Deleted

```
BEFORE:                              AFTER:
Line 10: fn process() { A }  T1_001  Line 10: fn process() { A }  ← T1_001 ✓
         hash=H1                     Line 20: fn process() { C }  ← T1_003 ✓
Line 20: fn process() { B }  T1_002
         hash=H2                     T1_002 has no match → DELETED
Line 30: fn process() { C }  T1_003
         hash=H3
```

---

## Simulation Summary Matrix

```mermaid
flowchart TD
    subgraph Scenarios["Matching Scenarios"]
        S1["Different content hashes"]
        S2["Same hash, shifted"]
        S3["Same hash, swapped"]
        S4["Content modified"]
        S5["New entity"]
        S6["Entity deleted"]
    end

    subgraph Solutions["Resolution Strategy"]
        R1["Hash match ✓"]
        R2["Order-based match"]
        R3["Undetectable<br/>(semantically equivalent)"]
        R4["Position match + MODIFIED"]
        R5["New timestamp"]
        R6["Mark DELETED"]
    end

    S1 --> R1
    S2 --> R2
    S3 --> R3
    S4 --> R4
    S5 --> R5
    S6 --> R6

    style R1 fill:#69db7c,stroke:#37b24d
    style R2 fill:#ffd43b,stroke:#fab005
    style R3 fill:#868e96,stroke:#495057
    style R4 fill:#ffd43b,stroke:#fab005
    style R5 fill:#4dabf7,stroke:#228be6
    style R6 fill:#ff6b6b,stroke:#c92a2a
```

| Scenario | Content Hash | Solution | Key Stable? |
|----------|--------------|----------|-------------|
| Different content | Unique hashes | Match by hash | ✅ Yes |
| Same content, shifted | Same hash | Order-based match | ✅ Yes |
| Same content, swapped | Same hash | **Undetectable** (but equivalent) | ⚠️ N/A |
| Content modified | Hash changed | Position-based, mark MODIFIED | ✅ Yes |
| New entity | No hash match | New timestamp | ✅ Yes (new) |
| Deleted entity | No new match | Mark DELETED | ✅ Yes (gone) |

---

## Philosophical Acceptance

**The Identical Duplicate Limitation**:

If two functions are byte-for-byte identical:
- Same name
- Same content
- Same hash

...then their individual identity is **philosophically meaningless**. Swapping them changes nothing about the program's behavior. We accept this limitation.

**The Mental Model**:

```
OLD: "Track the same entity across changes"
     (impossible for identical duplicates)

NEW: "Track entities by semantic path, detect when content changed"
     (handles all practical cases)
```

---

## Implementation Status Matrix

| Component | Location | Status | Notes |
|-----------|----------|--------|-------|
| **File Watcher** | `watcher_service.rs:118-191` | ✅ Complete | Uses notify v6.1 |
| **Debouncer** | `debouncer.rs` | ✅ Complete | 500ms window |
| **Path Filter** | `path_filter.rs` | ✅ Complete | Ignores target/, .git/, etc. |
| **Start Watching** | `watcher_service.rs:205-251` | ✅ Complete | Recursive watching |
| **Stop Watching** | `watcher_service.rs:265-290` | ✅ Complete | Cleanup resources |
| **Incremental Reindex** | `watcher_service.rs:292-313` | ❌ **STUB** | Returns Ok() only |
| **Broadcast Diff** | `watcher_service.rs:326-342` | ✅ Complete | WebSocket push |
| **Event Conversion** | `watcher_service.rs:351-376` | ✅ Complete | notify → internal |
| **WebSocket Handler** | `websocket_streaming_module/` | ✅ Complete | Upgrade + streaming |
| **Database Insert** | `cozo_client.rs:774-801` | ✅ Complete | Entity insertion |
| **Database Delete** | `cozo_client.rs` | ✅ Complete | Entity deletion |
| **Get Changed Entities** | `cozo_client.rs:884-911` | ✅ Complete | For diff computation |
| **Diff Engine** | `parseltongue-core/src/diff/` | ✅ Complete | Full diff computation |
| **Blast Radius** | `blast_radius.rs` | ✅ Complete | Impact calculation |

---

## Three Alternative Approaches

Based on research into modern IDEs (rust-analyzer, VS Code), academic literature, and industrial-scale tools (Sourcegraph, Kythe), here are three distinct approaches for implementing incremental indexing:

### Approach 1: Simple But Reliable (File Hash + Coarse-Grained Invalidation)

```mermaid
flowchart TD
    subgraph Detection["Change Detection"]
        F1["File Change Event"]
        F2["Compute SHA-256 hash"]
        F3{{"Hash changed?"}}
        F4["Skip (no change)"]
    end

    subgraph Invalidation["Coarse Invalidation"]
        I1["Delete ALL entities from file"]
        I2["Delete ALL outgoing edges"]
    end

    subgraph Reparse["Full Reparse"]
        R1["Tree-sitter parse entire file"]
        R2["Extract all entities"]
        R3["Extract all edges"]
    end

    subgraph Update["Database Update"]
        U1["INSERT new entities"]
        U2["INSERT new edges"]
        U3["Update hash cache"]
    end

    F1 --> F2
    F2 --> F3
    F3 -->|"No"| F4
    F3 -->|"Yes"| I1
    I1 --> I2
    I2 --> R1
    R1 --> R2
    R2 --> R3
    R3 --> U1
    U1 --> U2
    U2 --> U3

    style F3 fill:#ffd43b,stroke:#fab005
```

**Characteristics**:
- **Latency**: 20-100ms per file change
- **Complexity**: Low (2-3 days implementation)
- **Granularity**: File-level (entire file invalidated)

**Pros**:
- Simple to understand and debug
- Predictable O(file_size) performance
- Reliable - no missed updates
- Works with existing codebase

**Cons**:
- Over-invalidation (one function change invalidates entire file)
- No incremental parsing
- High database churn

**Rust Crates**:
```toml
notify = "6.1"      # File watching (already in use)
sha2 = "0.10"       # SHA-256 hashing
dashmap = "6.0"     # Concurrent hash map
```

---

### Approach 2: High-Performance Optimized (Salsa + Tree-sitter Incremental)

```mermaid
flowchart TD
    subgraph Salsa["Salsa Query System"]
        Q1["file_content(path) → Arc&lt;String&gt;"]
        Q2["syntax_tree(path) → Arc&lt;Tree&gt;"]
        Q3["parsed_entities(path) → Arc&lt;Vec&gt;"]
        Q4["entity_dependencies(key) → Arc&lt;Vec&gt;"]
    end

    subgraph Incremental["Incremental Features"]
        I1["Automatic dependency tracking"]
        I2["Early cutoff optimization"]
        I3["Fine-grained invalidation"]
    end

    subgraph TreeSitter["Tree-sitter Incremental"]
        T1["edit() method"]
        T2["Reuse unchanged subtrees"]
        T3["Parse only affected regions"]
    end

    subgraph Delta["Entity-Level Delta"]
        D1["Diff old entities vs new"]
        D2["Identify added/removed/modified"]
        D3["Update only changed entities"]
    end

    Q1 --> Q2
    Q2 --> Q3
    Q3 --> Q4

    I1 --> Salsa
    I2 --> Salsa
    I3 --> Salsa

    T1 --> Q2
    T2 --> Q2
    T3 --> Q2

    Q3 --> D1
    D1 --> D2
    D2 --> D3

    style Salsa fill:#69db7c,stroke:#37b24d
    style TreeSitter fill:#4dabf7,stroke:#228be6
```

**Characteristics**:
- **Latency**: 10-40ms per file change
- **Complexity**: High (2-3 weeks implementation)
- **Granularity**: Entity-level

**Pros**:
- 2-5x faster than Approach 1
- Smart invalidation (only recomputes changed entities)
- Early cutoff (whitespace changes don't propagate)
- Proven architecture (used by rust-analyzer)

**Cons**:
- Steep learning curve for Salsa
- More code (4-5x more than Approach 1)
- Memory overhead (multiple caches)
- Debugging difficulty

**Rust Crates**:
```toml
salsa = "0.16"          # Incremental computation framework
tree-sitter = "0.20"    # Already in use
dashmap = "6.0"         # Thread-safe caching
parking_lot = "0.12"    # Better mutexes for Salsa
similar = "2.2"         # Entity diffing
```

---

### Approach 3: Bleeding-Edge Research (Reactive Incremental + Delta Encoding)

```mermaid
flowchart TD
    subgraph Reactive["Reactive Computation"]
        R1["Adapton Engine"]
        R2["Dynamic dependency graph"]
        R3["Automatic propagation"]
    end

    subgraph ContentAddressed["Content-Addressed Storage"]
        C1["Blake3 hash → AST node"]
        C2["Deduplicate across files"]
        C3["Structural sharing"]
    end

    subgraph Persistent["Persistent Graph Versions"]
        P1["Copy-on-write data structures"]
        P2["Multiple versions simultaneously"]
        P3["Instant rollback"]
    end

    subgraph Speculative["Speculative Indexing"]
        S1["ML predictor"]
        S2["Pre-compute likely next states"]
        S3["Warm cache proactively"]
    end

    subgraph CRDT["Concurrent Editing"]
        CR1["CRDT-based conflict resolution"]
        CR2["Multiple concurrent editors"]
        CR3["Automatic merge"]
    end

    R1 --> R2
    R2 --> R3

    C1 --> C2
    C2 --> C3

    P1 --> P2
    P2 --> P3

    S1 --> S2
    S2 --> S3

    CR1 --> CR2
    CR2 --> CR3

    style R1 fill:#e599f7,stroke:#be4bdb
    style S1 fill:#ffd43b,stroke:#fab005
    style CR1 fill:#ff6b6b,stroke:#c92a2a
```

**Characteristics**:
- **Latency**: 3-15ms (optimistic), 20-50ms (cache miss)
- **Complexity**: Very High (1-2 months implementation)
- **Granularity**: AST-node-level

**Pros**:
- Theoretical best performance (<5ms for cache hits)
- Maximum deduplication via content-addressing
- Time-travel (instant rollback to any version)
- Concurrent edit support (CRDTs)

**Cons**:
- Extreme complexity (10-20x more than Approach 1)
- Unproven at this scale
- High risk (many unknowns)
- Overkill for most use cases

**Rust Crates**:
```toml
adapton = "0.4"         # Reactive incremental computation
im = "15.1"             # Persistent data structures
blake3 = "1.5"          # Fast content hashing
automerge = "0.5"       # CRDT for collaborative editing
yrs = "0.16"            # Y-CRDT (faster alternative)
rayon = "1.7"           # Parallel processing
```

---

## Comparison Matrix

```mermaid
quadrantChart
    title Implementation Complexity vs Performance
    x-axis Low Complexity --> High Complexity
    y-axis Slow (100ms+) --> Fast (<10ms)
    quadrant-1 Ideal Zone
    quadrant-2 Over-Engineered
    quadrant-3 Acceptable
    quadrant-4 Avoid

    Approach 1: [0.2, 0.3]
    Approach 2: [0.6, 0.7]
    Approach 3: [0.9, 0.9]
```

| Criterion | Approach 1 | Approach 2 | Approach 3 |
|-----------|-----------|-----------|-----------|
| **Implementation Time** | 2-3 days | 2-3 weeks | 1-2 months |
| **Lines of Code** | ~200 | ~800-1200 | ~2000-3000 |
| **Typical Latency** | 20-100ms | 10-40ms | 3-15ms |
| **Complexity** | Low | High | Very High |
| **Memory Overhead** | ~10MB | ~50MB | ~200MB+ |
| **Granularity** | File-level | Entity-level | AST-node |
| **Incremental Parsing** | No | Yes | Yes + speculative |
| **Production Ready** | Yes | Yes | No (research) |
| **Risk Level** | Low | Medium | High |

---

## Recommendation

### Recommended Path: Start with Approach 1, Migrate to Approach 2

```mermaid
gantt
    title Implementation Roadmap
    dateFormat YYYY-MM-DD
    section Phase 1
    Approach 1 Basic Implementation   :a1, 2026-01-27, 3d
    Testing & Validation              :a2, after a1, 2d
    section Phase 2
    Salsa Framework Study             :b1, after a2, 5d
    Prototype Query Pipeline          :b2, after b1, 7d
    Performance Benchmarking          :b3, after b2, 3d
    section Phase 3
    Migration Decision                :c1, after b3, 2d
    Full Salsa Integration            :c2, after c1, 14d
```

**Phase 1 (Week 1-2)**: Implement Approach 1
- Add file hash tracking
- Integrate notify crate for file watching
- Implement basic incremental update handler
- Validate end-to-end flow works

**Phase 2 (Week 3-6)**: Evaluate Approach 2
- Study Salsa framework
- Prototype query-based parsing pipeline
- Benchmark performance improvements
- Decide if migration ROI is worth it

**Phase 3 (Optional)**: Full Salsa Migration
- Only if benchmarks show significant improvement
- Only if team has capacity for complexity

---

## Critical Files Reference

### Files to Implement

| File | Purpose | Priority |
|------|---------|----------|
| `watcher_service.rs:292-313` | The stub function | **P0 - Critical** |

### Files to Reuse

| File | Purpose | How to Use |
|------|---------|------------|
| `pt01-folder-to-cozodb-streamer/src/streamer.rs` | `stream_file()` method | Call for single-file parsing |
| `parseltongue-core/src/storage/cozo_client.rs` | `insert_entity()`, `delete_entity()` | Database operations |
| `parseltongue-core/src/diff/` | Diff computation | Compute base.db vs live.db |
| `websocket_streaming_module/handler.rs` | `broadcast_diff_to_subscribers()` | Real-time push |

### Configuration Files

| File | Purpose |
|------|---------|
| `Cargo.toml` (pt08) | Add new dependencies |
| `route_definition_builder_module.rs` | HTTP routes (no changes needed) |

---

## Appendix: Pseudocode for Implementation

### Basic Implementation (Approach 1)

```rust
pub async fn trigger_incremental_reindex_update(
    state: &SharedApplicationStateContainer,
    workspace_id: &str,
    changed_files: &[PathBuf],
) -> Result<(), FileWatcherErrorType> {
    let db = state.get_live_db(workspace_id)?;

    for file_path in changed_files {
        // 1. Compute file hash
        let content = tokio::fs::read_to_string(file_path).await?;
        let new_hash = sha256_hash(&content);

        // 2. Check if actually changed
        if let Some(old_hash) = db.get_file_hash(file_path).await? {
            if old_hash == new_hash {
                continue; // No actual change
            }
        }

        // 3. Delete old entities from this file
        let old_entities = db.get_entities_by_file(file_path).await?;
        for entity in &old_entities {
            db.delete_entity(&entity.key).await?;
            db.delete_outgoing_edges(&entity.key).await?;
        }

        // 4. Re-parse file (reuse existing streamer)
        let (new_entities, new_edges) = parse_file_to_entities(file_path, &content)?;

        // 5. Insert new entities and edges
        for entity in &new_entities {
            db.insert_entity(entity).await?;
        }
        for edge in &new_edges {
            db.insert_edge(edge).await?;
        }

        // 6. Update hash cache
        db.set_file_hash(file_path, new_hash).await?;
    }

    // 7. Compute diff and broadcast
    let base_db = state.get_base_db(workspace_id)?;
    let diff = compute_diff(&base_db, &db).await?;
    broadcast_diff_to_subscribers(&state.subscribers, workspace_id, diff).await?;

    Ok(())
}
```

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-26 | Claude Code Analysis | Initial comprehensive document |
| 1.1 | 2026-01-26 | Claude Code Analysis | Added ISGL1 v2 design with birth timestamp identity, entity matching algorithm, 6 simulation scenarios, philosophical acceptance of identical duplicate limitation |

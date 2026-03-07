# Rust-Analyzer Control Flow Diagrams

**Visual Guide to How Code Flows Through the System**

## Main Event Loop Control Flow

```mermaid
flowchart TD
    START([Program Start])
    START --> INIT[Initialize GlobalState<br/>- Config<br/>- Channels<br/>- Thread pools]

    INIT --> REGISTER[Register LSP Capabilities<br/>- didSave if dynamic<br/>- File watchers]

    REGISTER --> FETCH_INIT{Discovery<br/>config?}
    FETCH_INIT -->|No| FETCH_WS[Trigger initial<br/>workspace fetch]
    FETCH_INIT -->|Yes| LOOP

    FETCH_WS --> LOOP

    LOOP{Wait for<br/>Event}

    LOOP -->|LSP Message| LSP_EVENT[Event::Lsp]
    LOOP -->|Task Done| TASK_EVENT[Event::Task]
    LOOP -->|VFS Change| VFS_EVENT[Event::Vfs]
    LOOP -->|Flycheck| FLY_EVENT[Event::Flycheck]
    LOOP -->|Test Result| TEST_EVENT[Event::TestResult]
    LOOP -->|Project Discovery| DISC_EVENT[Event::DiscoverProject]
    LOOP -->|Workspace Fetch| FETCH_EVENT[Event::FetchWorkspaces]
    LOOP -->|Deferred Task| DEF_EVENT[Event::DeferredTask]

    LOOP -->|Exit notification| EXIT([Clean Exit])

    LSP_EVENT --> LSP_HANDLER[handle_lsp<br/>Request/Notification]
    TASK_EVENT --> TASK_HANDLER[handle_task<br/>+ Coalescing]
    VFS_EVENT --> VFS_HANDLER[handle_vfs_msg<br/>+ Coalescing]
    FLY_EVENT --> FLY_HANDLER[handle_flycheck<br/>+ Coalescing]
    TEST_EVENT --> TEST_HANDLER[handle_test_result<br/>+ Coalescing]
    DISC_EVENT --> DISC_HANDLER[handle_discover<br/>+ Coalescing]
    FETCH_EVENT --> FETCH_HANDLER[Queue workspace fetch]
    DEF_EVENT --> DEF_HANDLER[handle_deferred_task<br/>+ Coalescing]

    LSP_HANDLER --> POST
    TASK_HANDLER --> POST
    VFS_HANDLER --> POST
    FLY_HANDLER --> POST
    TEST_HANDLER --> POST
    DISC_HANDLER --> POST
    FETCH_HANDLER --> POST
    DEF_HANDLER --> POST

    POST[Post-Event Processing]
    POST --> VFS_DONE{vfs_done?}
    VFS_DONE -->|Yes| SWITCH{wants_to<br/>_switch?}
    VFS_DONE -->|No| SKIP_POST

    SWITCH -->|Yes| SWITCH_WS[switch_workspaces]
    SWITCH -->|No| PROCESS

    SWITCH_WS --> PROCESS[process_changes<br/>Apply VFS changes]

    PROCESS --> QUIESCE{is_quiescent?}

    QUIESCE -->|became_quiescent| QUIESCE_ACTIONS[Quiescence Actions<br/>- Trigger flycheck<br/>- Request prime_caches<br/>- Refresh tokens/lenses]

    QUIESCE -->|still quiescent| CLIENT_REFRESH{State changed?}
    QUIESCE -->|not quiescent| SKIP_QUIESCE

    QUIESCE_ACTIONS --> CLIENT_REFRESH

    CLIENT_REFRESH -->|Yes| REFRESHES[Send LSP Refreshes<br/>- SemanticTokens<br/>- CodeLens<br/>- InlayHints<br/>- Diagnostics]

    CLIENT_REFRESH -->|No| UPDATES

    REFRESHES --> UPDATES

    UPDATES{Project/docs<br/>changed?}
    UPDATES -->|Yes| UPDATE_ACTIONS[- update_diagnostics<br/>- update_tests]
    UPDATES -->|No| GC_CHECK

    UPDATE_ACTIONS --> GC_CHECK

    GC_CHECK{Idle &<br/>revision<br/>changed?}
    GC_CHECK -->|Yes| GC[trigger_garbage_collection]
    GC_CHECK -->|No| CLEANUP

    GC --> CLEANUP

    SKIP_QUIESCE --> CLEANUP
    SKIP_POST --> CLEANUP

    CLEANUP[Cleanup & Op Queue Checks]
    CLEANUP --> PUBLISH_DIAG{Diagnostics<br/>changed?}

    PUBLISH_DIAG -->|Yes| PUBLISH[publish_diagnostics<br/>to client]
    PUBLISH_DIAG -->|No| OP_QUEUE

    PUBLISH --> OP_QUEUE

    OP_QUEUE{Operations<br/>to start?}

    OP_QUEUE -->|fetch_workspaces| START_FETCH[Start fetch_workspaces]
    OP_QUEUE -->|fetch_build_data| START_BUILD[Start fetch_build_data]
    OP_QUEUE -->|fetch_proc_macros| START_PROC[Start fetch_proc_macros]
    OP_QUEUE -->|prime_caches| START_PRIME[Start prime_caches]
    OP_QUEUE -->|None| STATUS

    START_FETCH --> STATUS
    START_BUILD --> STATUS
    START_PROC --> STATUS
    START_PRIME --> STATUS

    STATUS[update_status_or_notify]
    STATUS --> PERF{Loop > 100ms<br/>& quiescent?}

    PERF -->|Yes| WARN[Log performance warning]
    PERF -->|No| LOOP

    WARN --> LOOP

    style START fill:#a8dadc
    style EXIT fill:#a8dadc
    style LOOP fill:#ffe66d
    style POST fill:#4ecdc4
    style QUIESCE_ACTIONS fill:#95e1d3
```

## LSP Request Handling Flow

```mermaid
flowchart TD
    LSP_MSG[LSP Message Received]

    LSP_MSG --> MSG_TYPE{Message Type}

    MSG_TYPE -->|Request| ON_REQ[on_new_request]
    MSG_TYPE -->|Notification| ON_NOTIF[on_notification]
    MSG_TYPE -->|Response| ON_RESP[complete_request]

    ON_REQ --> SHUTDOWN{Shutdown<br/>requested?}

    SHUTDOWN -->|Yes| REJECT[Respond with<br/>server error]
    SHUTDOWN -->|No| REG_REQ[Register request<br/>in req_queue]

    REG_REQ --> ROUTE[Route to handler<br/>on_request]

    ROUTE --> HANDLER_TYPE{Handler Type}

    HANDLER_TYPE -->|Immediate| SYNC_HANDLER[Synchronous Handler<br/>- On main thread<br/>- Quick operations]

    HANDLER_TYPE -->|Deferred| SPAWN_TASK[Spawn background task<br/>- Create snapshot<br/>- Send to task pool]

    HANDLER_TYPE -->|Special| SPECIAL_HANDLER[Special Handler<br/>- shutdown<br/>- cancelRequest]

    SYNC_HANDLER --> RESPOND[Send LSP Response]
    SPAWN_TASK --> BG_WORK[Background Work]
    SPECIAL_HANDLER --> SPECIAL_RESPOND[Handle specially]

    BG_WORK --> QUERY[Execute Salsa Queries<br/>on snapshot]
    QUERY --> RESULT[Compute Result]
    RESULT --> SEND_TASK[Send Task event<br/>with result]
    SEND_TASK --> TASK_LOOP[Processed in<br/>Event::Task branch]
    TASK_LOOP --> RESPOND

    RESPOND --> DONE([Done])
    SPECIAL_RESPOND --> DONE

    ON_NOTIF --> NOTIF_ROUTE[Route notification]

    NOTIF_ROUTE --> NOTIF_TYPE{Notification Type}

    NOTIF_TYPE -->|DidOpen| DID_OPEN[Add to MemDocs<br/>Override VFS]
    NOTIF_TYPE -->|DidChange| DID_CHANGE[Update MemDocs<br/>Apply changes]
    NOTIF_TYPE -->|DidSave| DID_SAVE[Trigger flycheck<br/>if configured]
    NOTIF_TYPE -->|DidClose| DID_CLOSE[Remove from MemDocs<br/>Reload from VFS]
    NOTIF_TYPE -->|DidChangeConfig| DID_CONFIG[Update config<br/>Maybe reload workspace]
    NOTIF_TYPE -->|Other| OTHER_NOTIF[Handle other]

    DID_OPEN --> NOTIF_DONE([Done])
    DID_CHANGE --> NOTIF_DONE
    DID_SAVE --> NOTIF_DONE
    DID_CLOSE --> NOTIF_DONE
    DID_CONFIG --> NOTIF_DONE
    OTHER_NOTIF --> NOTIF_DONE

    ON_RESP --> COMPLETE[Mark request complete<br/>in req_queue]
    COMPLETE --> RESP_DONE([Done])

    style LSP_MSG fill:#e1f5ff
    style SYNC_HANDLER fill:#95e1d3
    style SPAWN_TASK fill:#ffe66d
    style BG_WORK fill:#ffd97d
```

## Completion Request Flow

```mermaid
sequenceDiagram
    participant Client as LSP Client
    participant Main as Main Loop
    participant Handler as Completion Handler
    participant Snapshot as GlobalStateSnapshot
    participant IDE as ide::completions
    participant HIR as HIR Layer
    participant DB as Salsa DB

    Client->>Main: textDocument/completion
    Main->>Main: on_new_request
    Main->>Main: Register in req_queue
    Main->>Handler: Route to handle_completion

    Handler->>Snapshot: Create snapshot
    Snapshot-->>Handler: GlobalStateSnapshot

    Handler->>Handler: Spawn background task

    par Background Thread
        Handler->>IDE: completions(db, position)
        IDE->>DB: parse(file_id)
        DB-->>IDE: SyntaxTree

        IDE->>IDE: Find token at cursor
        IDE->>HIR: Resolve semantics
        HIR->>DB: infer(function)
        DB-->>HIR: Type info

        HIR-->>IDE: Semantic context

        IDE->>IDE: Determine completion kind
        IDE->>IDE: Run appropriate completer<br/>(dot, path, keyword, etc.)

        IDE->>IDE: Score and filter items
        IDE-->>Handler: Vec<CompletionItem>

        Handler->>Main: Send Task event
    end

    Main->>Main: handle_task(Task::Response)
    Main->>Main: to_proto::completion_item
    Main->>Client: LSP Response
```

## Workspace Loading Control Flow

```mermaid
stateDiagram-v2
    [*] --> RequestWorkspaces: User opens project

    RequestWorkspaces --> FetchMetadata: fetch_workspaces_queue.request_op()
    FetchMetadata --> LoadingMetadata: Spawn cargo metadata task

    LoadingMetadata --> MetadataComplete: CargoWorkspace ready
    MetadataComplete --> WantsToSwitch: wants_to_switch = Some(cause)

    WantsToSwitch --> SwitchWorkspaces: vfs_done = true
    SwitchWorkspaces --> BuildCrateGraph: Create initial CrateGraph

    BuildCrateGraph --> RequestBuildData: fetch_build_data_queue.request_op()
    RequestBuildData --> RunningBuildScripts: Spawn build.rs task

    RunningBuildScripts --> BuildDataComplete: Build data ready
    BuildDataComplete --> UpdateCrateGraph1: Add OUT_DIR, env, cfgs

    UpdateCrateGraph1 --> RequestProcMacros: fetch_proc_macros_queue.request_op()
    RequestProcMacros --> CompilingProcMacros: Compile proc macro dylibs

    CompilingProcMacros --> ProcMacrosComplete: Proc macro clients ready
    ProcMacrosComplete --> UpdateCrateGraph2: Attach proc macros

    UpdateCrateGraph2 --> Quiescent: became_quiescent = true

    Quiescent --> PrimeCaches: prime_caches_queue.request_op()
    PrimeCaches --> Indexing: Warm up Salsa caches

    Indexing --> Ready: [*]

    note right of FetchMetadata
        Fast: cargo metadata
        Returns quickly
    end note

    note right of RunningBuildScripts
        Slow: Runs build.rs
        May compile dependencies
    end note

    note right of CompilingProcMacros
        Slow: Compiles proc macros
        cargo build --message-format=json
    end note

    note right of Indexing
        Background: Precomputes
        - DefMaps
        - Type inference
        - Symbols
    end note
```

## VFS Change Processing

```mermaid
flowchart TD
    FS_CHANGE[File System Change]
    FS_CHANGE --> WATCHER[vfs-notify<br/>File Watcher]

    WATCHER --> LOADER_MSG[vfs::loader::Message]
    LOADER_MSG --> EVENT[Event::Vfs]

    EVENT --> HANDLE_VFS[handle_vfs_msg]

    HANDLE_VFS --> MSG_TYPE{Message Type}

    MSG_TYPE -->|Loaded| LOADED[Update VFS<br/>set_file_contents]
    MSG_TYPE -->|Progress| PROGRESS[Track progress]

    LOADED --> COALESCE{More VFS<br/>messages?}
    PROGRESS --> COALESCE

    COALESCE -->|Yes| DRAIN[try_recv next]
    COALESCE -->|No| REPORT_PROG

    DRAIN --> HANDLE_VFS

    REPORT_PROG[Report Progress<br/>"Roots Scanned"]
    REPORT_PROG --> POST_EVENT[Post-event processing]

    POST_EVENT --> PROCESS{vfs_done?}
    PROCESS -->|Yes| PROC_CHANGES[process_changes]
    PROCESS -->|No| SKIP

    PROC_CHANGES --> TAKE_CHANGES[vfs.take_changes]
    TAKE_CHANGES --> FOR_EACH[For each changed FileId]

    FOR_EACH --> INVALIDATE[Invalidate Salsa queries<br/>- file_text<br/>- parse<br/>- etc.]

    INVALIDATE --> APPLY[analysis_host.apply_change]

    APPLY --> DONE[State changed = true]
    DONE --> TRIGGER_UPDATE[Trigger diagnostics update<br/>on next quiescence]

    SKIP --> NO_CHANGE[State changed = false]

    style FS_CHANGE fill:#e8f5e9
    style INVALIDATE fill:#ff6b6b
    style TRIGGER_UPDATE fill:#4ecdc4
```

## Flycheck (Cargo Check) Flow

```mermaid
sequenceDiagram
    participant User
    participant Main as Main Loop
    participant Flycheck as FlycheckHandle
    participant Cargo as cargo check
    participant Queue as Message Queue

    User->>Main: Save file (DidSave)

    alt check_on_save enabled
        Main->>Flycheck: restart()
        Flycheck->>Cargo: Spawn cargo check
        Cargo-->>Queue: JSON diagnostic messages
    end

    loop While cargo running
        Queue->>Main: FlycheckMessage::AddDiagnostic
        Main->>Main: handle_flycheck_msg
        Main->>Main: Accumulate diagnostics
    end

    Cargo-->>Queue: FlycheckMessage::Finished
    Queue->>Main: Finished
    Main->>Main: Mark cargo_finished = true

    Main->>Main: Coalesce remaining messages
    Main->>Main: Send WorkspaceDiagnosticRefresh

    Main->>Main: publish_diagnostics()
    Main->>User: Show diagnostics in editor
```

## Macro Expansion Control Flow

```mermaid
flowchart TD
    MACRO_CALL[Encounter Macro Call<br/>in HIR lowering]

    MACRO_CALL --> RESOLVE[Resolve macro definition]

    RESOLVE --> KIND{Macro Kind}

    KIND -->|Built-in| BUILTIN[expand_builtin<br/>- include!<br/>- concat!<br/>- etc.]

    KIND -->|Declarative| MBE_EXPAND[mbe::expand<br/>Pattern matching]

    KIND -->|Proc Macro| PROC_EXPAND[proc_macro_expand]

    BUILTIN --> TOKENS[tt::TokenTree]
    MBE_EXPAND --> TOKENS
    PROC_EXPAND --> IPC

    IPC[IPC to proc-macro-srv]
    IPC --> SERVER[proc-macro-srv process]
    SERVER --> LOAD_DYLIB[Load proc macro dylib]
    LOAD_DYLIB --> CALL_EXPANDER[Call expander function]
    CALL_EXPANDER --> SERVER_RESULT[TokenStream result]
    SERVER_RESULT --> IPC_BACK[Serialize back]
    IPC_BACK --> TOKENS

    TOKENS --> PARSE[Parse expanded tokens]
    PARSE --> FIXUP[Apply span fixup]
    FIXUP --> INTEGRATE[Integrate into HIR]

    INTEGRATE --> DONE([Expansion Complete])

    style MACRO_CALL fill:#ffe66d
    style IPC fill:#ff6b6b
    style TOKENS fill:#95e1d3
```

## Goto Definition Flow

```mermaid
flowchart LR
    REQUEST[textDocument/definition<br/>at cursor position]

    REQUEST --> FIND_TOKEN[Find token at position]
    FIND_TOKEN --> TOKEN[SyntaxToken]

    TOKEN --> CLASSIFY{Token Classification}

    CLASSIFY -->|Name| NAME_RESOLVE[Resolve name<br/>using HIR]
    CLASSIFY -->|Keyword| NO_DEF[No definition]
    CLASSIFY -->|Literal| NO_DEF

    NAME_RESOLVE --> DEF_KIND{Definition Kind}

    DEF_KIND -->|Local| LOCAL_DEF[Find binding pattern]
    DEF_KIND -->|Field| FIELD_DEF[Find struct field]
    DEF_KIND -->|Function| FN_DEF[Find function definition]
    DEF_KIND -->|Type| TYPE_DEF[Find type definition]
    DEF_KIND -->|Macro| MACRO_DEF[Find macro definition]

    LOCAL_DEF --> NAV_TARGET[NavigationTarget]
    FIELD_DEF --> NAV_TARGET
    FN_DEF --> NAV_TARGET
    TYPE_DEF --> NAV_TARGET
    MACRO_DEF --> NAV_TARGET

    NAV_TARGET --> TO_LSP[Convert to LSP Location]
    TO_LSP --> RESPONSE[Send LSP Response]

    NO_DEF --> NULL_RESPONSE[Send null response]

    style REQUEST fill:#e1f5ff
    style NAV_TARGET fill:#95e1d3
    style RESPONSE fill:#4ecdc4
```

## Salsa Query Execution Flow

```mermaid
flowchart TD
    QUERY_START[Execute Query<br/>e.g., infer(function_id)]

    QUERY_START --> CHECK_MEMO{Result<br/>memoized?}

    CHECK_MEMO -->|Yes| CHECK_DEPS{Dependencies<br/>changed?}
    CHECK_MEMO -->|No| EXECUTE

    CHECK_DEPS -->|No| RETURN_MEMO[Return cached result]
    CHECK_DEPS -->|Yes| INVALIDATE[Invalidate cache]

    INVALIDATE --> EXECUTE[Execute query function]

    EXECUTE --> TRACK_DEPS[Track dependencies]
    TRACK_DEPS --> COMPUTE[Compute result]

    COMPUTE --> MAY_CALL{Query calls<br/>other queries?}

    MAY_CALL -->|Yes| NESTED_QUERY[Execute nested query]
    NESTED_QUERY --> TRACK_DEP[Track as dependency]
    TRACK_DEP --> COMPUTE

    MAY_CALL -->|No| MEMOIZE[Memoize result]

    MEMOIZE --> RETURN[Return result]

    RETURN_MEMO --> DONE([Done])
    RETURN --> DONE

    style CHECK_MEMO fill:#ffe66d
    style RETURN_MEMO fill:#95e1d3
    style EXECUTE fill:#ff6b6b
```

## Summary of Control Flow Patterns

### Event Loop Pattern
- **Central loop** waits for events from multiple sources
- **Event coalescing** batches similar events
- **Post-processing** applies changes and triggers follow-ups

### Request/Response Pattern
- **Synchronous**: Fast queries on main thread
- **Asynchronous**: Slow queries on background threads using snapshots
- **Queue tracking**: Requests tracked for cancellation

### State Machine Pattern
- **Workspace loading**: Multi-phase state transitions
- **Quiescence detection**: Idle state triggers optimizations
- **VFS synchronization**: Gates workspace switching

### Observer Pattern
- **File watching**: Filesystem → VFS → Analysis
- **Diagnostic updates**: Changes → Re-analysis → Publish
- **Progress reporting**: Long operations → Client notifications

### Pipeline Pattern
- **Syntax → Semantics → IDE**: Layered transformations
- **Incremental**: Only re-execute changed stages
- **Memoized**: Salsa caches intermediate results

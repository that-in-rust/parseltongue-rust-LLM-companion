# Control Flow Analysis

This document shows **how execution flows** through Parseltongue - where the program starts, what functions call what, and how the different parts work together.

## Top-Level Flow

```mermaid
flowchart TD
    START([User runs parseltongue command]) --> PARSE[Parse CLI arguments]
    PARSE --> ROUTE{Which tool?}

    ROUTE -->|pt01| PT01[Run ingestion tool]
    ROUTE -->|pt08| PT08[Run HTTP server]
    ROUTE -->|--help| HELP[Show help]
    ROUTE -->|--version| VER[Show version]

    PT01 --> PT01_DONE([Print database path & exit])
    PT08 --> PT08_RUNNING([Server running, await requests])
    HELP --> END([Exit])
    VER --> END

    style START fill:#e1f5ff
    style ROUTE fill:#fff4e1
    style PT01 fill:#99ccff
    style PT08 fill:#99ff99
```

## pt01 Ingestion Flow

```mermaid
flowchart TD
    START([pt01-folder-to-cozodb-streamer .]) --> WORKSPACE[Create timestamped workspace]
    WORKSPACE --> DB_INIT[Initialize CozoDB database]
    DB_INIT --> SCHEMA[Create database schema]

    SCHEMA --> WALK_START[Start directory walk]
    WALK_START --> WALK_LOOP{More files?}

    WALK_LOOP -->|Yes| CHECK_EXT{Supported extension?}
    WALK_LOOP -->|No| SUMMARY[Print ingestion summary]

    CHECK_EXT -->|No| WALK_LOOP
    CHECK_EXT -->|Yes| READ_FILE[Read file contents]

    READ_FILE --> DETECT_LANG[Detect language]
    DETECT_LANG --> CREATE_PARSER[Create Tree-sitter parser]
    CREATE_PARSER --> PARSE_FILE[Parse file into AST]

    PARSE_FILE --> EXTRACT{Extract entities}
    EXTRACT --> QUERY_TS[Run Tree-sitter queries]
    QUERY_TS --> BUILD_ENTITIES[Build CodeEntity objects]
    BUILD_ENTITIES --> BUILD_EDGES[Build DependencyEdge objects]

    BUILD_EDGES --> BATCH_INSERT[Batch insert to DB]
    BATCH_INSERT --> WALK_LOOP

    SUMMARY --> PRINT_STATS[Print statistics]
    PRINT_STATS --> DONE([Exit])

    style START fill:#e1f5ff
    style DONE fill:#c8e6c9
    style BATCH_INSERT fill:#fff4e1
```

### pt01 Detailed Function Flow

```mermaid
graph TB
    subgraph "Main Entry"
        A[pt01::run]
    end

    subgraph "Setup Phase"
        B[StreamerConfig::default]
        C[create_workspace]
        D[CozoDbStorage::new]
        E[create_schema]
    end

    subgraph "Processing Phase"
        F[walk_directory]
        G[process_file]
        H[QueryBasedExtractor::new]
        I[execute_query]
        J[execute_dependency_query]
    end

    subgraph "Storage Phase"
        K[insert_entities_batch]
        L[insert_edges_batch]
    end

    A --> B --> C --> D --> E --> F
    F --> G
    G --> H --> I
    I --> J
    J --> K --> L
    L --> F

    style A fill:#99ccff
    style K fill:#fff4e1
    style L fill:#fff4e1
```

## pt08 HTTP Server Flow

### Server Startup

```mermaid
flowchart TD
    START([pt08-http-code-query-server --db path]) --> PARSE_ARGS[Parse arguments]
    PARSE_ARGS --> CONNECT_DB[Connect to CozoDB]
    CONNECT_DB --> HEALTH_CHECK{Database connected?}

    HEALTH_CHECK -->|No| ERROR[Print error & exit]
    HEALTH_CHECK -->|Yes| SETUP_ROUTES[Setup Axum routes]

    SETUP_ROUTES --> BIND_PORT[Bind to port 7777]
    BIND_PORT --> START_SERVER[Start HTTP server]
    START_SERVER --> LISTEN([Listen for requests])

    ERROR --> EXIT([Exit])

    style START fill:#e1f5ff
    style LISTEN fill:#c8e6c9
    style ERROR fill:#ffcdd2
```

### HTTP Request Flow

```mermaid
sequenceDiagram
    participant Client
    participant Axum as Axum Router
    participant Handler as Endpoint Handler
    participant Core as parseltongue-core
    participant DB as CozoDB

    Client->>Axum: GET /code-entities-search-fuzzy?q=main
    Axum->>Axum: Route to handler
    Axum->>Handler: handle_code_entities_fuzzy_search()

    Handler->>Handler: Parse query params
    Handler->>Handler: Validate input

    Handler->>Core: search_entities_by_query_from_database()
    Core->>DB: Execute CozoQL query
    DB-->>Core: Query results (rows)
    Core->>Core: Parse JSON from rows
    Core-->>Handler: Vec<CodeEntity>

    Handler->>Handler: Build JSON response
    Handler-->>Axum: HTTP 200 + JSON
    Axum-->>Client: Response body
```

### Example: Reverse Callers Flow

```mermaid
flowchart TD
    START([GET /reverse-callers-query-graph?entity=rust:fn:main]) --> HANDLER[handle_reverse_callers_query_graph]

    HANDLER --> EXTRACT_KEY[extract_function_name_key]
    EXTRACT_KEY --> VALIDATE{Valid entity key?}

    VALIDATE -->|No| ERROR_400[Return HTTP 400]
    VALIDATE -->|Yes| QUERY[query_reverse_callers_direct_method]

    QUERY --> BUILD_COZO[Build CozoQL query string]
    BUILD_COZO --> EXECUTE[db.execute_query]
    EXECUTE --> DB[(CozoDB)]

    DB --> PARSE_RESULTS[Parse JSON rows]
    PARSE_RESULTS --> BUILD_GRAPH[Build graph structure]
    BUILD_GRAPH --> JSON_RESPONSE[Serialize to JSON]

    JSON_RESPONSE --> RETURN_200[Return HTTP 200]
    ERROR_400 --> END([Response sent])
    RETURN_200 --> END

    style START fill:#e1f5ff
    style DB fill:#fff4e1
    style END fill:#c8e6c9
    style ERROR_400 fill:#ffcdd2
```

## parseltongue-core Control Flow

### QueryBasedExtractor::new()

```mermaid
flowchart TD
    START([new file_path, content]) --> DETECT[get_ts_language]
    DETECT --> PARSER[init_parser]
    PARSER --> LOAD_QUERIES[Load .scm query files]

    LOAD_QUERIES --> ENTITY_Q[Load entity queries]
    LOAD_QUERIES --> DEP_Q[Load dependency queries]

    ENTITY_Q --> BUILD_EXTRACTOR[Create QueryBasedExtractor]
    DEP_Q --> BUILD_EXTRACTOR

    BUILD_EXTRACTOR --> RETURN([Return extractor])

    style START fill:#e1f5ff
    style RETURN fill:#c8e6c9
```

### execute_query() - Entity Extraction

```mermaid
flowchart TD
    START([execute_query]) --> TS_QUERY[Run Tree-sitter query on AST]
    TS_QUERY --> LOOP{More matches?}

    LOOP -->|No| DONE([Return entities])
    LOOP -->|Yes| EXTRACT_FIELDS[Extract captures: name, type, start, end]

    EXTRACT_FIELDS --> BUILD_KEY[Generate entity key]
    BUILD_KEY --> BUILD_ENTITY[Create CodeEntity]
    BUILD_ENTITY --> ADD_TO_VEC[Add to result vector]
    ADD_TO_VEC --> LOOP

    style START fill:#e1f5ff
    style DONE fill:#c8e6c9
```

### execute_dependency_query() - Edge Extraction

```mermaid
flowchart TD
    START([execute_dependency_query]) --> TS_QUERY[Run dependency query on AST]
    TS_QUERY --> LOOP{More matches?}

    LOOP -->|No| DONE([Return edges])
    LOOP -->|Yes| EXTRACT_CALLER[Extract caller capture]

    EXTRACT_CALLER --> EXTRACT_CALLEE[Extract callee capture]
    EXTRACT_CALLEE --> FIND_CONTAINING[find_containing_entity for caller]

    FIND_CONTAINING --> BUILD_EDGE[Create DependencyEdge]
    BUILD_EDGE --> ADD_TO_VEC[Add to result vector]
    ADD_TO_VEC --> LOOP

    style START fill:#e1f5ff
    style DONE fill:#c8e6c9
```

### CozoDbStorage::insert_entity()

```mermaid
flowchart TD
    START([insert_entity]) --> TO_PARAMS[entity_to_params]
    TO_PARAMS --> BUILD_QUERY[Build CozoQL INSERT statement]
    BUILD_QUERY --> EXECUTE[db.run_script]

    EXECUTE --> CHECK{Success?}
    CHECK -->|Yes| OK([Return Ok])
    CHECK -->|No| ERR([Return Err])

    style START fill:#e1f5ff
    style OK fill:#c8e6c9
    style ERR fill:#ffcdd2
```

## Advanced Analysis Flows

### Blast Radius Calculation

```mermaid
flowchart TD
    START([handle_blast_radius_impact_analysis]) --> PARSE[Parse entity + hops params]
    PARSE --> VALIDATE{Valid params?}

    VALIDATE -->|No| ERROR[HTTP 400]
    VALIDATE -->|Yes| COMPUTE[compute_blast_radius_by_hops]

    COMPUTE --> INIT_SET[Initialize visited set with root entity]
    INIT_SET --> LOOP{Current hop < max hops?}

    LOOP -->|No| BUILD_RESP[Build JSON response]
    LOOP -->|Yes| QUERY_DEPS[Query forward dependencies for current layer]

    QUERY_DEPS --> ADD_TO_SET[Add new entities to visited set]
    ADD_TO_SET --> INCREMENT[Increment hop counter]
    INCREMENT --> LOOP

    BUILD_RESP --> COUNT[Count total impact]
    COUNT --> RETURN[HTTP 200 with JSON]
    ERROR --> END([Response sent])
    RETURN --> END

    style START fill:#e1f5ff
    style END fill:#c8e6c9
    style ERROR fill:#ffcdd2
```

### Circular Dependency Detection

```mermaid
flowchart TD
    START([handle_circular_dependency_detection_scan]) --> FETCH_ALL[Get all entities]
    FETCH_ALL --> BUILD_GRAPH[Build adjacency list]

    BUILD_GRAPH --> INIT_DFS[Initialize DFS state]
    INIT_DFS --> LOOP{More entities?}

    LOOP -->|No| RETURN_CYCLES[Return found cycles]
    LOOP -->|Yes| CHECK_VISITED{Already visited?}

    CHECK_VISITED -->|Yes| LOOP
    CHECK_VISITED -->|No| DFS[dfs_find_cycles_recursive]

    DFS --> MARK_VISITING[Mark as in current path]
    MARK_VISITING --> GET_NEIGHBORS[Get forward dependencies]
    GET_NEIGHBORS --> NEIGHBOR_LOOP{More neighbors?}

    NEIGHBOR_LOOP -->|No| MARK_VISITED[Mark as fully visited]
    NEIGHBOR_LOOP -->|Yes| CHECK_IN_PATH{Neighbor in current path?}

    CHECK_IN_PATH -->|Yes| CYCLE_FOUND[Add cycle to results]
    CHECK_IN_PATH -->|No| RECURSE[Recurse on neighbor]

    CYCLE_FOUND --> NEIGHBOR_LOOP
    RECURSE --> NEIGHBOR_LOOP
    MARK_VISITED --> LOOP

    RETURN_CYCLES --> BUILD_JSON[Build JSON response]
    BUILD_JSON --> DONE([HTTP 200])

    style START fill:#e1f5ff
    style DONE fill:#c8e6c9
    style CYCLE_FOUND fill:#ffeb3b
```

## File Watching Flow (v1.4.2)

```mermaid
flowchart TD
    START([Server starts]) --> INIT_WATCHER[Initialize file watcher]
    INIT_WATCHER --> LISTEN([Listening for file changes])

    LISTEN --> EVENT{File event?}
    EVENT -->|Modify| COMPUTE_HASH[compute_file_content_hash]
    EVENT -->|Create| COMPUTE_HASH
    EVENT -->|Delete| DELETE_ENTITIES[delete_entities_batch_by_keys]

    COMPUTE_HASH --> GET_CACHED[get_cached_file_hash_value]
    GET_CACHED --> COMPARE{Hash changed?}

    COMPARE -->|No| LISTEN
    COMPARE -->|Yes| REINDEX[handle_incremental_reindex_file_request]

    REINDEX --> DELETE_OLD[delete_entities_batch_by_keys]
    DELETE_OLD --> DELETE_EDGES[delete_edges_by_from_keys]
    DELETE_EDGES --> REPARSE[Parse file with QueryBasedExtractor]
    REPARSE --> INSERT_NEW[insert_entities_batch + insert_edges_batch]
    INSERT_NEW --> UPDATE_HASH[Update file hash cache]
    UPDATE_HASH --> LISTEN

    DELETE_ENTITIES --> LISTEN

    style START fill:#e1f5ff
    style LISTEN fill:#c8e6c9
```

## Error Handling Flow

```mermaid
flowchart TD
    START([Any operation]) --> TRY{Operation result?}

    TRY -->|Ok| SUCCESS[Continue execution]
    TRY -->|Err| MATCH{Error type?}

    MATCH -->|ParseError| LOG_PARSE[Log parsing error]
    MATCH -->|DatabaseError| LOG_DB[Log database error]
    MATCH -->|NetworkError| LOG_NET[Log network error]
    MATCH -->|ValidationError| RETURN_400[HTTP 400 Bad Request]

    LOG_PARSE --> CONTINUE_OR_ABORT{Critical?}
    LOG_DB --> CONTINUE_OR_ABORT
    LOG_NET --> CONTINUE_OR_ABORT

    CONTINUE_OR_ABORT -->|Non-critical| SKIP[Skip this item]
    CONTINUE_OR_ABORT -->|Critical| ABORT[Abort operation]

    SKIP --> SUCCESS
    ABORT --> RETURN_500[HTTP 500 Internal Error]
    RETURN_400 --> END([Return to caller])
    RETURN_500 --> END
    SUCCESS --> END

    style START fill:#e1f5ff
    style SUCCESS fill:#c8e6c9
    style RETURN_400 fill:#ffcdd2
    style RETURN_500 fill:#ffcdd2
```

## Next: Data Flow

See [03-data-flow.md](03-data-flow.md) to understand how data is transformed through the system.

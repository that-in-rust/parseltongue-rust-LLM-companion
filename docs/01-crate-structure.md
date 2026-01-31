# Crate Structure Deep Dive

## Directory Layout

```
crates/
├── parseltongue/                   # Binary - CLI entry point
├── parseltongue-core/              # Library - shared functionality
├── pt01-folder-to-cozodb-streamer/ # Tool 1 - Ingestion
└── pt08-http-code-query-server/    # Tool 8 - HTTP API
```

## Dependency Graph

```mermaid
graph TD
    subgraph "Binary Layer"
        CLI[parseltongue<br/>main.rs]
    end

    subgraph "Tool Layer"
        PT01[pt01-folder-to-cozodb-streamer<br/>Ingest Tool]
        PT08[pt08-http-code-query-server<br/>HTTP Server]
    end

    subgraph "Core Layer"
        CORE[parseltongue-core]

        subgraph "Core Modules"
            ENT[entities.rs<br/>Types]
            STOR[storage/cozo_client.rs<br/>Database]
            EXT[query_extractor.rs<br/>Parser]
            INT[interfaces.rs<br/>Traits]
        end
    end

    subgraph "External Dependencies"
        TS[tree-sitter<br/>Parser]
        COZO[(CozoDB<br/>Database)]
        AXUM[Axum<br/>HTTP Framework]
    end

    CLI --> PT01
    CLI --> PT08
    PT01 --> CORE
    PT08 --> CORE
    CORE --> ENT
    CORE --> STOR
    CORE --> EXT
    CORE --> INT
    EXT --> TS
    STOR --> COZO
    PT08 --> AXUM

    style CLI fill:#ff9999
    style PT01 fill:#99ccff
    style PT08 fill:#99ff99
    style CORE fill:#ffcc99
```

## Crate Details

### 1. parseltongue (Binary Crate)

**Purpose**: CLI dispatcher that routes to tool crates

**Key Responsibilities**:
- Parse command-line arguments
- Route to pt01 or pt08
- Handle version/help commands

**Entry Point**:
```rust
// Simplified flow
fn main() {
    match args.tool {
        "pt01-folder-to-cozodb-streamer" => pt01::run(),
        "pt08-http-code-query-server" => pt08::run(),
        _ => show_help()
    }
}
```

### 2. parseltongue-core (Library Crate)

**Purpose**: Shared types, traits, and business logic

**Module Breakdown**:

```mermaid
graph TB
    subgraph "parseltongue-core Modules"
        A[entities.rs<br/>CodeEntity, DependencyEdge, EntityType]
        B[interfaces.rs<br/>GraphStorage trait]
        C[query_extractor.rs<br/>QueryBasedExtractor]
        D[storage/cozo_client.rs<br/>CozoDbStorage]
        E[query_json_graph_helpers.rs<br/>Graph traversal]
        F[query_json_graph_errors.rs<br/>Error types]
        G[serializers/<br/>JSON, TOON output]
        H[error.rs<br/>ParseltongError]
    end

    B --> D
    C --> A
    C --> B
    D --> A
    D --> B
    E --> A
    E --> F

    style A fill:#ffe6e6
    style B fill:#e6f2ff
    style C fill:#e6ffe6
    style D fill:#fff9e6
```

**Key Types**:

| Type | Purpose | Example |
|------|---------|---------|
| `CodeEntity` | Represents a code element (fn, struct, class) | `rust:fn:main:src_main_rs:1-50` |
| `DependencyEdge` | Represents a relationship between entities | Function A calls Function B |
| `EntityType` | Enum of entity kinds | Function, Struct, Class, Method |
| `GraphStorage` | Trait for database operations | `insert_entity()`, `get_dependencies()` |
| `QueryBasedExtractor` | Tree-sitter parser wrapper | Extracts entities from source code |

### 3. pt01-folder-to-cozodb-streamer (Tool Crate)

**Purpose**: Ingest source code into CozoDB

**Flow**:

```mermaid
sequenceDiagram
    participant CLI
    participant PT01
    participant Core as QueryBasedExtractor
    participant DB as CozoDbStorage

    CLI->>PT01: Run with folder path
    PT01->>PT01: Create timestamped workspace
    PT01->>PT01: Walk directory tree

    loop For each file
        PT01->>Core: Parse file
        Core->>Core: Tree-sitter parse
        Core-->>PT01: Entities + Edges
        PT01->>DB: insert_entities_batch()
        PT01->>DB: insert_edges_batch()
    end

    PT01-->>CLI: Print database path
```

**Key Functions**:

```mermaid
graph LR
    A[run] --> B[create_workspace]
    A --> C[walk_directory]
    C --> D[process_file]
    D --> E[QueryBasedExtractor::new]
    E --> F[extract_entities]
    F --> G[insert_to_db]

    style A fill:#99ccff
    style G fill:#ffcc99
```

**Output**:
- Creates `parseltongue{TIMESTAMP}/analysis.db`
- Prints statistics (files processed, entities created, errors)

### 4. pt08-http-code-query-server (Tool Crate)

**Purpose**: REST API server for querying the database

**Architecture**:

```mermaid
graph TB
    subgraph "HTTP Server"
        A[main.rs<br/>Axum setup]
        B[Router]
    end

    subgraph "Handler Modules"
        C[server_health_check_handler]
        D[codebase_statistics_overview_handler]
        E[code_entities_list_all_handler]
        F[code_entities_fuzzy_search_handler]
        G[code_entity_detail_view_handler]
        H[dependency_edges_list_handler]
        I[reverse_callers_query_graph_handler]
        J[forward_callees_query_graph_handler]
        K[blast_radius_impact_handler]
        L[circular_dependency_detection_handler]
        M[complexity_hotspots_ranking_handler]
        N[semantic_cluster_grouping_handler]
        O[smart_context_token_budget_handler]
        P[api_reference_documentation_handler]
    end

    A --> B
    B --> C
    B --> D
    B --> E
    B --> F
    B --> G
    B --> H
    B --> I
    B --> J
    B --> K
    B --> L
    B --> M
    B --> N
    B --> O
    B --> P

    style A fill:#99ff99
    style B fill:#ffff99
```

**Handler Organization** (4-word naming pattern):

| Category | Handler Function | Endpoint |
|----------|------------------|----------|
| Core | `handle_server_health_check_status` | `/server-health-check-status` |
| Core | `handle_codebase_statistics_overview_summary` | `/codebase-statistics-overview-summary` |
| Core | `handle_api_reference_documentation_help` | `/api-reference-documentation-help` |
| Entity | `handle_code_entities_list_all` | `/code-entities-list-all` |
| Entity | `handle_code_entity_detail_view` | `/code-entity-detail-view/{key}` |
| Entity | `handle_code_entities_fuzzy_search` | `/code-entities-search-fuzzy` |
| Edge | `handle_dependency_edges_list_all` | `/dependency-edges-list-all` |
| Edge | `handle_reverse_callers_query_graph` | `/reverse-callers-query-graph` |
| Edge | `handle_forward_callees_query_graph` | `/forward-callees-query-graph` |
| Analysis | `handle_blast_radius_impact_analysis` | `/blast-radius-impact-analysis` |
| Analysis | `handle_circular_dependency_detection_scan` | `/circular-dependency-detection-scan` |
| Analysis | `handle_complexity_hotspots_ranking_view` | `/complexity-hotspots-ranking-view` |
| Analysis | `handle_semantic_cluster_grouping_list` | `/semantic-cluster-grouping-list` |
| Advanced | `handle_smart_context_token_budget` | `/smart-context-token-budget` |

## Cross-Crate Communication

```mermaid
graph LR
    subgraph "User Space"
        U[User]
    end

    subgraph "CLI Layer"
        C[parseltongue binary]
    end

    subgraph "Tool Layer"
        T1[pt01]
        T2[pt08]
    end

    subgraph "Core Layer"
        CORE[parseltongue-core]
    end

    subgraph "Storage Layer"
        DB[(CozoDB)]
    end

    U -->|command| C
    C -->|dispatch| T1
    C -->|dispatch| T2
    T1 -->|use types| CORE
    T2 -->|use types| CORE
    CORE -->|read/write| DB
    T1 -->|write| DB
    T2 -->|read| DB

    style U fill:#e1f5ff
    style C fill:#ff9999
    style T1 fill:#99ccff
    style T2 fill:#99ff99
    style CORE fill:#ffcc99
    style DB fill:#fff4e1
```

## File Organization Per Crate

### parseltongue-core (Most Complex)

```
parseltongue-core/
├── src/
│   ├── entities.rs              # CodeEntity, DependencyEdge, EntityType, etc.
│   ├── interfaces.rs            # GraphStorage trait, ToolCapabilities
│   ├── error.rs                 # ParseltongError
│   ├── query_extractor.rs       # QueryBasedExtractor (Tree-sitter)
│   ├── query_json_graph_helpers.rs  # Graph traversal utilities
│   ├── query_json_graph_errors.rs   # JsonGraphQueryError
│   ├── output_path_resolver.rs  # Workspace path logic
│   ├── temporal.rs              # Temporal versioning
│   ├── entity_class_specifications.rs  # CODE vs TEST classification
│   ├── storage/
│   │   ├── mod.rs
│   │   └── cozo_client.rs       # CozoDbStorage implementation
│   └── serializers/
│       ├── mod.rs
│       ├── json.rs              # JSON output
│       └── toon.rs              # TOON format output
```

### pt01-folder-to-cozodb-streamer

```
pt01-folder-to-cozodb-streamer/
├── src/
│   ├── lib.rs                   # Main ingestion logic
│   └── errors.rs                # StreamerError
```

### pt08-http-code-query-server

```
pt08-http-code-query-server/
├── src/
│   ├── main.rs                  # Axum server setup
│   ├── structured_error_handling_types.rs  # HttpServerErrorTypes
│   └── http_endpoint_handler_modules/
│       ├── server_health_check_handler.rs
│       ├── codebase_statistics_overview_handler.rs
│       ├── api_reference_documentation_handler.rs
│       ├── code_entities_list_all_handler.rs
│       ├── code_entity_detail_view_handler.rs
│       ├── code_entities_fuzzy_search_handler.rs
│       ├── dependency_edges_list_handler.rs
│       ├── reverse_callers_query_graph_handler.rs
│       ├── forward_callees_query_graph_handler.rs
│       ├── blast_radius_impact_handler.rs
│       ├── circular_dependency_detection_handler.rs
│       ├── complexity_hotspots_ranking_handler.rs
│       ├── semantic_cluster_grouping_handler.rs
│       ├── smart_context_token_budget_handler.rs
│       └── incremental_reindex_file_handler.rs
```

## Next: Control Flow

See [02-control-flow.md](02-control-flow.md) for execution flow diagrams.

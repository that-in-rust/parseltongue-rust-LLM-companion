# Parseltongue Architecture Overview

## What is Parseltongue?

**Parseltongue** is like a librarian for your code. Instead of reading thousands of files one by one, Parseltongue:
1. Reads your entire codebase once
2. Organizes everything into a searchable database
3. Lets you ask questions super fast (31x faster than grep!)
4. Uses 99% fewer tokens for AI tools (2-5K vs 500K!)

## High-Level Architecture

```mermaid
graph TB
    subgraph "User Input"
        A[Your Codebase Folder]
    end

    subgraph "Ingestion Phase"
        B[pt01-folder-to-cozodb-streamer]
        C[parseltongue-core<br/>Tree-Sitter Parser]
        D[(CozoDB<br/>Graph Database)]
    end

    subgraph "Query Phase"
        E[pt08-http-code-query-server]
        F[REST API Endpoints]
    end

    subgraph "Client Applications"
        G[curl / HTTP Client]
        H[LLM Agents]
        I[Your IDE]
    end

    A --> B
    B --> C
    C --> D
    D --> E
    E --> F
    F --> G
    F --> H
    F --> I

    style A fill:#e1f5ff
    style D fill:#fff4e1
    style F fill:#e8f5e9
```

## Project Statistics

- **Total Code Entities**: 230
- **Dependency Edges**: 3,864
- **Languages Supported**: Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, Ruby, PHP, C#, Swift
- **Version**: 1.4.2

## Crate Structure

```mermaid
graph LR
    subgraph "Binary Crate"
        A[parseltongue<br/>CLI Dispatcher]
    end

    subgraph "Tool Crates"
        B[pt01-folder-to-cozodb-streamer<br/>Ingest Tool]
        C[pt08-http-code-query-server<br/>HTTP Server]
    end

    subgraph "Core Library"
        D[parseltongue-core<br/>Shared Types & Storage]
    end

    A --> B
    A --> C
    B --> D
    C --> D

    style A fill:#ff9999
    style B fill:#99ccff
    style C fill:#99ff99
    style D fill:#ffcc99
```

## How It Works (ELI5)

### Step 1: Ingestion (Building the Library)
Think of this like organizing a messy room:
- **Input**: Your codebase folder
- **Process**: pt01 reads every file, finds functions/classes/imports
- **Output**: A organized database (`analysis.db`)

### Step 2: Query (Finding Things Fast)
Think of this like asking a librarian questions:
- **Input**: HTTP requests (questions about your code)
- **Process**: pt08 searches the database
- **Output**: JSON answers in milliseconds

## Key Components

| Component | Role | Analogy |
|-----------|------|---------|
| **parseltongue-core** | Brain - parsing & storage logic | The librarian's training manual |
| **pt01-folder-to-cozodb-streamer** | Cataloger - builds database | The person organizing books |
| **pt08-http-code-query-server** | Reference Desk - answers questions | The help desk |
| **CozoDB** | Memory - stores graph data | The card catalog system |

## Workflow Example

```mermaid
sequenceDiagram
    participant User
    participant pt01 as pt01-streamer
    participant Core as parseltongue-core
    participant DB as CozoDB
    participant pt08 as pt08-server

    User->>pt01: parseltongue pt01 .
    pt01->>Core: Parse files with Tree-Sitter
    Core->>DB: Store entities & edges
    DB-->>User: ✓ Database created

    User->>pt08: Start server with --db
    pt08->>DB: Connect to database
    DB-->>pt08: ✓ Ready

    User->>pt08: GET /code-entities-search-fuzzy?q=main
    pt08->>DB: Query for "main"
    DB-->>pt08: Results (entities matching "main")
    pt08-->>User: JSON response
```

## Why Use Parseltongue?

1. **Speed**: 31x faster than grep
2. **Token Efficiency**: 99% reduction for LLM context
3. **Graph Queries**: Find who calls what, detect cycles, analyze impact
4. **Multi-Language**: Works with 12+ programming languages
5. **Always Fresh**: Built-in file watching (v1.4.2) auto-updates

## Next Steps

- **[01-crate-structure.md](01-crate-structure.md)** - Detailed breakdown of each crate
- **[02-control-flow.md](02-control-flow.md)** - How execution flows through the system
- **[03-data-flow.md](03-data-flow.md)** - How data is transformed
- **[04-api-guide.md](04-api-guide.md)** - Complete API reference with examples

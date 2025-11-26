# Parseltongue HTTP Server PRD

**Author**: Product
**Status**: Draft
**Version**: 1.2.0
**Last Updated**: 2025-11-27

---

## The One-Liner

Parseltongue ingests a codebase into a queryable database and spins up an HTTP server that any LLM agent can query.

---

## The Problem We're Solving

**Current pain**: Developers and LLM agents cannot easily understand codebases. They resort to grep, which:
- Returns raw text (no semantic understanding)
- Uses 100x more tokens than needed
- Misses relationships between code entities
- Requires re-parsing on every query

**The insight**: Code is a graph, not text. Parse it once, store it in a database, query it forever.

---

## What We're Building

A single binary that:
1. **Ingests** a folder into a database (entities + dependencies)
2. **Spins up** an HTTP server to query that database
3. **Returns** structured JSON that any LLM can understand

```bash
parseltongue serve-http-code-backend ./my-project
# Ingesting...done. 156 entities, 870 edges.
# Server running at http://localhost:3847
#
# Add this to your agent: http://localhost:3847
# Try: curl http://localhost:3847/stats
```

That's it. No JSON exports. No file management. Just ingest → serve → query.

---

## What We're NOT Building

- ❌ JSON file exports (removed - the HTTP server replaces this)
- ❌ GUI or web interface (terminal-first)
- ❌ Code editing/modification tools (analysis only)
- ❌ Cloud/hosted service (local backend only)
- ❌ MCP protocol (HTTP is more universal)

---

## Command Specification (4-Word Naming Convention)

Per project rules: ALL commands use EXACTLY 4 words.

### Primary Command

```bash
parseltongue serve-http-code-backend <DIRECTORY> [OPTIONS]
```

**4-Word Breakdown**: `serve` + `http` + `code` + `backend`

**Options**:
```bash
--port <PORT>        # HTTP port (default: auto-detect from 3333)
--reindex            # Force fresh ingestion even if DB exists
--daemon             # Run in background mode
--timeout <MINUTES>  # Auto-shutdown after idle period
--verbose            # Show query logs
--quiet              # Minimal output
```

**Examples**:
```bash
# Basic usage (current directory)
parseltongue serve-http-code-backend .

# Specific directory with custom port
parseltongue serve-http-code-backend /path/to/project --port 8080

# Force re-ingestion
parseltongue serve-http-code-backend . --reindex

# Background mode with auto-shutdown
parseltongue serve-http-code-backend . --daemon --timeout 60
```

---

## Languages Supported

**12 Languages via Tree-Sitter**:

| Language | File Extensions | Entity Types Extracted |
|----------|-----------------|----------------------|
| **Rust** | `.rs` | fn, struct, enum, trait, impl, mod |
| **Python** | `.py` | def, class, async def |
| **JavaScript** | `.js`, `.jsx` | function, class, arrow functions |
| **TypeScript** | `.ts`, `.tsx` | function, class, interface, type |
| **Go** | `.go` | func, type, struct, interface |
| **Java** | `.java` | class, interface, method, enum |
| **C** | `.c`, `.h` | function, struct, typedef |
| **C++** | `.cpp`, `.cc`, `.hpp` | function, class, struct, template |
| **Ruby** | `.rb` | def, class, module |
| **PHP** | `.php` | function, class, trait |
| **C#** | `.cs` | class, struct, interface, method |
| **Swift** | `.swift` | func, class, struct, protocol |

---

## User Jobs To Be Done

| Job | How Parseltongue Helps |
|-----|------------------------|
| "I want to understand this codebase" | Ingest → query `/stats`, `/hotspots` |
| "What calls this function?" | Query `/callers/{entity}` |
| "What breaks if I change X?" | Query `/blast/{entity}?hops=3` |
| "Are there circular dependencies?" | Query `/cycles` |
| "Find functions related to payment" | Query `/search?q=payment` |

---

## User Journey: Analyzing an Open Source Codebase

### Step 1: Run Parseltongue

```bash
$ cd ~/code/some-open-source-project
$ parseltongue serve-http-code-backend .
```

### Step 2: Watch Ingestion Happen

```
Parseltongue v1.1.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Ingesting: /Users/dev/code/some-open-source-project

Phase 1: Scanning
  ✓ 234 files found
  ✗ 12 skipped (target/, node_modules/, .git/, __pycache__/)

Phase 2: Parsing (12 languages supported)
  ✓ Rust:       89 files  →  456 entities
  ✓ Python:     45 files  →  123 entities
  ✓ TypeScript: 100 files →  234 entities

Phase 3: Dependency Extraction (CPG-Inspired Edge Types)
  ✓ 2,541 edges
    Calls: 1,890 | Uses: 345 | Implements: 106 | Extends: 45 | Contains: 155

Phase 4: Classification
  ✓ CODE entities: 591
  ✓ TEST entities: 222 (excluded from default queries)

Database saved to:
  ./parseltongue_20251126193045/analysis.db
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Step 3: Server Starts Automatically

```
HTTP Server running at: http://localhost:3847

┌─────────────────────────────────────────────────────────────────────┐
│  Add this to your LLM agent configuration:                          │
│                                                                     │
│  PARSELTONGUE_URL=http://localhost:3847                             │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘

Quick test:
  curl http://localhost:3847/stats

Available endpoints:
  GET /stats              - Codebase statistics
  GET /entities           - List all entities
  GET /callers/{name}     - Who calls this?
  GET /callees/{name}     - What does this call?
  GET /blast/{name}       - Impact analysis
  GET /cycles             - Circular dependencies
  GET /hotspots           - Complexity ranking
  GET /clusters           - Semantic module groupings (LPA)
  GET /search?q=pattern   - Search entities
  GET /help               - Full API reference

Press Ctrl+C to stop server
```

### Step 4: Agent Queries the Server

The LLM agent (Claude Code, GPT, Cursor, etc.) now knows to query `http://localhost:3847`:

```
User: "What are the most complex functions in this codebase?"

Agent internally runs:
  curl http://localhost:3847/hotspots?top=5

Server responds:
  {
    "hotspots": [
      {"name": "process_request", "deps": 34, "file": "src/handler.rs"},
      {"name": "parse_config", "deps": 28, "file": "src/config.rs"},
      ...
    ]
  }

Agent responds:
  "The most complex functions are process_request (34 dependencies)
   and parse_config (28 dependencies). Would you like me to analyze
   either of these?"
```

---

## User Journey: Returning to a Previously Analyzed Codebase

### Step 1: Run Parseltongue Again

```bash
$ cd ~/code/some-open-source-project
$ parseltongue serve-http-code-backend .
```

### Step 2: Existing Database Detected

```
Parseltongue v1.1.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Found existing database:
  ./parseltongue_20251126193045/analysis.db
  Created: 2 hours ago
  Entities: 591 | Edges: 2,341

Loading... done (0.3s)

HTTP Server running at: http://localhost:3847

(Use --reindex to force fresh ingestion)
```

No re-indexing. Instant startup. Same port management.

---

## Technical Architecture

### Folder Structure

```
./my-project/
├── src/
├── Cargo.toml
└── parseltongue_20251126193045/     ← Created by parseltongue
    ├── analysis.db                   ← RocksDB + CozoDB
    └── README.md                     ← Auto-generated API reference
```

**Why timestamped folder?**
- Multiple analysis snapshots possible
- Clear provenance (when was this indexed?)
- No conflicts with project files

### Database Schema (What Gets Ingested)

```
CodeGraph {
  ISGL1_key: String        → "rust:fn:process:src_handler_rs:45-89"
  entity_name: String      → "process"
  entity_type: String      → "function"
  file_path: String        → "src/handler.rs"
  line_number: Int         → 45
  interface_signature: String → "pub async fn process(req: Request) -> Response"
  entity_class: String     → "CODE" or "TEST"
  current_ind: Bool        → true (exists now)
  future_ind: Bool         → true (will exist)
}

DependencyEdges {
  from_key: String         → "rust:fn:process:..."
  to_key: String           → "rust:fn:validate:..."
  edge_type: String        → "Calls", "Uses", "Implements", "Extends", "Contains"
  direction: String        → "downward", "upward", "horizontal"
}

### Edge Types (CPG-Inspired)

| Edge Type | Direction | Meaning | Example |
|-----------|-----------|---------|---------|
| `Calls` | downward | Function invocation | `main` → `process` |
| `Uses` | downward | Type/constant reference | `handler` → `Config` |
| `Implements` | upward | Trait implementation | `MyStruct` → `Trait` |
| `Extends` | upward | Inheritance/extension | `Child` → `Parent` |
| `Contains` | downward | Structural containment | `module` → `function` |

**Direction Semantics**:
- `"downward"` - Caller→Callee, User→Provider, Container→Contained
- `"upward"` - Implementor→Trait, Child→Parent
- `"horizontal"` - Peer relationships (rare)
```

### Port Management

The server picks an available port automatically:

```rust
// Pseudocode
let port = find_available_port_number(starting_from: 3333);
println!("Server running at: http://localhost:{}", port);
```

**Why auto-detect port?**
- User might run multiple instances (different projects)
- No port conflicts
- Server prints exact URL to use

### The Port-Instance Association

Each `parseltongue serve-http-code-backend` session:
1. Creates/loads a database in `parseltongue_yymmddhhss/`
2. Binds to an available port
3. That port serves ONLY that database
4. Ctrl+C kills server, database persists

```
Terminal 1: parseltongue serve-http-code-backend ./project-a  → localhost:3333 → project-a DB
Terminal 2: parseltongue serve-http-code-backend ./project-b  → localhost:3334 → project-b DB
```

---

## HTTP API Specification

### Endpoint Handler Functions (4-Word Naming)

| Endpoint | Handler Function | Description |
|----------|-----------------|-------------|
| `GET /health` | `handle_health_check_request()` | Server status |
| `GET /stats` | `handle_stats_overview_request()` | Codebase statistics |
| `GET /entities` | `handle_entities_list_request()` | All entities |
| `GET /entities/{key}` | `handle_entity_detail_request()` | Single entity |
| `GET /edges` | `handle_edges_list_request()` | All dependency edges |
| `GET /callers/{entity}` | `handle_callers_query_request()` | Reverse dependencies |
| `GET /callees/{entity}` | `handle_callees_query_request()` | Forward dependencies |
| `GET /blast/{entity}` | `handle_blast_radius_request()` | Transitive impact |
| `GET /cycles` | `handle_cycles_detection_request()` | Circular dependencies |
| `GET /hotspots` | `handle_hotspots_analysis_request()` | Complexity ranking |
| `GET /search` | `handle_search_entities_request()` | Fuzzy search |
| `GET /clusters` | `handle_clusters_query_request()` | Semantic module groupings |
| `GET /help` | `handle_help_reference_request()` | API documentation |

### Core Endpoints

| Endpoint | Description | Example Response |
|----------|-------------|------------------|
| `GET /stats` | Codebase overview | `{"entities": 591, "edges": 2341, "functions": 456}` |
| `GET /entities` | All entities | `[{"key": "rust:fn:...", "name": "process", ...}]` |
| `GET /entities?name=X` | Filter by name | Entities matching pattern |
| `GET /entities?type=function` | Filter by type | Only functions |
| `GET /edges` | All dependency edges | `[{"from": "...", "to": "...", "type": "Calls", "direction": "downward"}]` |

### Graph Query Endpoints

| Endpoint | Description | Use Case |
|----------|-------------|----------|
| `GET /callers/{entity}` | Reverse dependencies | "What calls this?" |
| `GET /callees/{entity}` | Forward dependencies | "What does this call?" |
| `GET /blast/{entity}?hops=N` | Transitive impact | "What breaks if I change this?" |
| `GET /cycles` | Circular dependencies | "Are there architectural problems?" |
| `GET /hotspots?top=N` | Complexity ranking | "Where is the complexity?" |
| `GET /clusters` | Semantic module groupings | "What logical modules exist?" |
| `GET /clusters?entity=X` | Cluster for entity | "What module does X belong to?" |
| `GET /search?q=pattern` | Fuzzy search | "Find anything related to X" |

### Response Format

All responses are JSON with consistent structure:

```json
{
  "success": true,
  "endpoint": "/callers/process",
  "count": 5,
  "data": [...],
  "tokens": 234,
  "query_time_ms": 12
}
```

The `tokens` field helps LLMs understand context budget impact.

---

## What the README Should Contain

The README (shown after ingestion) teaches the LLM how to query:

```markdown
# Parseltongue API Quick Reference

Base URL: http://localhost:{PORT}

## Common Queries

### "What functions exist?"
curl http://localhost:{PORT}/entities?type=function

### "What calls X?"
curl http://localhost:{PORT}/callers/{function_name}

### "What does X depend on?"
curl http://localhost:{PORT}/callees/{function_name}

### "What breaks if I change X?"
curl http://localhost:{PORT}/blast/{function_name}?hops=3

### "Are there circular dependencies?"
curl http://localhost:{PORT}/cycles

### "Where is the complexity?"
curl http://localhost:{PORT}/hotspots?top=10

### "Find functions related to payment"
curl http://localhost:{PORT}/search?q=payment
```

This README can be:
- Printed to terminal after startup
- Saved to `parseltongue_yymmddhhss/README.md`
- Fetched via `GET /help` endpoint

---

## What Gets Removed

### Crates/Commands to Delete

| Crate | Current Command | Reason for Removal |
|-------|-----------------|-------------------|
| `pt02-llm-cozodb-to-context-writer` | `parseltongue pt02-llm-cozodb-to-context level00/01/02` | Replaced by HTTP endpoints |

### Features to Delete

| Feature | Files Affected | Reason |
|---------|---------------|--------|
| JSON export | `pt02/src/exporters/*.rs` | HTTP responses replace file exports |
| TOON export | `parseltongue-core/src/serializers/toon.rs` | No longer needed |
| `--output` flag | CLI argument parsing | No file output |
| `--include-code` flag | Export logic | Handled by endpoint selection |
| `--where-clause` flag | Query filtering | Replaced by query parameters |

### Crates to Keep (Modified)

| Crate | Modification |
|-------|--------------|
| `pt01-folder-to-cozodb-streamer` | Logic absorbed into `serve-http-code-backend` |
| `pt07-visual-analytics-terminal` | Keep as separate utility |
| `parseltongue-core` | Keep storage, entities; remove file serializers |

---

## Implementation Plan (TDD-First)

Per project rules: STUB → RED → GREEN → REFACTOR for every feature.

### Phase 1: Core Server (Week 1)

**Test-First Specifications**:

```rust
#[tokio::test]
async fn test_health_endpoint_returns_ok() {
    // GIVEN: Server running
    let server = create_test_server_instance().await;

    // WHEN: GET /health
    let response = server.get("/health").await;

    // THEN: Returns 200 with status "ok"
    assert_eq!(response.status(), 200);
    assert_eq!(response.json()["status"], "ok");
}

#[tokio::test]
async fn test_stats_returns_entity_count() {
    // GIVEN: Server with ingested codebase
    let server = create_test_server_with_data().await;

    // WHEN: GET /stats
    let response = server.get("/stats").await;

    // THEN: Returns entity and edge counts
    assert!(response.json()["data"]["entities"].as_u64().unwrap() > 0);
}
```

**Tasks**:
- [ ] HTTP server with axum (`create_http_server_instance()`)
- [ ] `/health` endpoint (`handle_health_check_request()`)
- [ ] `/stats` endpoint (`handle_stats_overview_request()`)
- [ ] Startup ingestion (integrate pt01 logic)
- [ ] Auto port detection (`find_available_port_number()`)

### Phase 2: Query Endpoints (Week 2)

**Test-First Specifications**:

```rust
#[tokio::test]
async fn test_callers_returns_reverse_deps() {
    // GIVEN: Entity "process" exists with callers
    let server = create_test_server_with_data().await;

    // WHEN: GET /callers/process
    let response = server.get("/callers/process").await;

    // THEN: Returns list of callers
    assert!(response.json()["success"].as_bool().unwrap());
    assert!(response.json()["count"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn test_blast_radius_bounded_by_hops() {
    // GIVEN: Entity with deep dependency tree
    let server = create_test_server_with_data().await;

    // WHEN: GET /blast/entity?hops=2
    let response = server.get("/blast/entity?hops=2").await;

    // THEN: No entity beyond depth 2
    for item in response.json()["data"].as_array().unwrap() {
        assert!(item["depth"].as_u64().unwrap() <= 2);
    }
}
```

**Tasks**:
- [ ] `/entities` endpoint (`handle_entities_list_request()`)
- [ ] `/edges` endpoint (`handle_edges_list_request()`)
- [ ] `/callers/{entity}` (`handle_callers_query_request()`)
- [ ] `/callees/{entity}` (`handle_callees_query_request()`)
- [ ] `/blast/{entity}` (`handle_blast_radius_request()`)
- [ ] `/cycles` (`handle_cycles_detection_request()`)
- [ ] `/hotspots` (`handle_hotspots_analysis_request()`)
- [ ] `/clusters` (`handle_clusters_query_request()`) - wraps pt08 LPA clustering
- [ ] `/search` (`handle_search_entities_request()`)

### Phase 3: Polish (Week 3)

- [ ] `--reindex` flag (`force_reindex_codebase_flag()`)
- [ ] `--daemon` mode (`run_background_daemon_mode()`)
- [ ] `--port` override (`override_default_port_number()`)
- [ ] `/help` endpoint with full API reference
- [ ] Error handling and edge cases
- [ ] README.md generation in database folder

### Phase 4: Remove Dead Code

- [ ] Delete `pt02-llm-cozodb-to-context-writer` crate
- [ ] Delete `serializers/toon.rs`
- [ ] Remove JSON export code from pt02
- [ ] Update all documentation

---

## Performance Contracts (Executable Specifications)

### Query Latency Contract

```rust
#[tokio::test]
async fn test_query_latency_under_100ms() {
    let server = create_server_with_1000_entities().await;

    let start = std::time::Instant::now();
    let _ = server.get("/entities").await;
    let elapsed = start.elapsed();

    assert!(elapsed < Duration::from_millis(100),
            "Query took {:?}, expected <100ms", elapsed);
}
```

### Startup Time Contract (Existing DB)

```rust
#[tokio::test]
async fn test_startup_existing_db_under_1s() {
    create_test_database_file().await;

    let start = std::time::Instant::now();
    let _ = start_server_existing_database().await;
    let elapsed = start.elapsed();

    assert!(elapsed < Duration::from_secs(1),
            "Startup took {:?}, expected <1s", elapsed);
}
```

---

## Success Metrics

| Metric | Target | Why It Matters |
|--------|--------|----------------|
| Time to first query | < 30 seconds | Fast enough to not interrupt flow |
| Query latency | < 100ms | Real-time feel |
| Restart time (existing DB) | < 1 second | No re-indexing penalty |
| Token efficiency vs grep | 50x reduction | LLM context budget |
| Agent compatibility | 100% HTTP clients | Universal access |

---

## Key Decisions

### Decision 1: No JSON Exports
**Rationale**: The HTTP server IS the export. Generating files that agents then read is an unnecessary step. Query directly.

### Decision 2: One Port Per Project
**Rationale**: Simple mental model. Each terminal running `parseltongue serve-http-code-backend` is one project, one port, one database.

### Decision 3: Auto-Detect Port
**Rationale**: User shouldn't think about ports. We pick one, we tell them what it is, they use it.

### Decision 4: Timestamped Folders Stay
**Rationale**: Clear provenance. User can have multiple snapshots. Doesn't pollute project root.

### Decision 5: HTTP Over MCP
**Rationale**: HTTP works with every agent (Claude Code, GPT, Cursor, scripts). MCP only works with Claude Desktop.

### Decision 6: 4-Word Naming Convention
**Rationale**: Per project rules - LLM tokenization optimized. `serve-http-code-backend` is self-documenting.

### Decision 7: TDD-First Implementation
**Rationale**: Per project rules - STUB → RED → GREEN → REFACTOR. Every feature has tests before code.

---

## Open Questions

1. **Multiple databases?** Should one server be able to query multiple project databases?
   - Current answer: No. One server, one project. Keep it simple.

2. **Incremental re-indexing?** When files change, update only changed entities?
   - Current answer: Future enhancement. V1 does full reindex with `--reindex`.

3. **Authentication?** Should the server require auth?
   - Current answer: No. Localhost only. If you have localhost access, you own the machine.

---

## Appendix: Migration from Current CLI

### What Gets Removed

```bash
# These commands go away:
parseltongue pt02-llm-cozodb-to-context level00 --output edges.json ...
parseltongue pt02-llm-cozodb-to-context level01 --output entities.json ...
parseltongue pt02-llm-cozodb-to-context level02 --output types.json ...

# These files are no longer created:
edges.json
edges_test.json
entities.json
entities_test.json
```

### What Replaces Them

```bash
# This replaces everything:
parseltongue serve-http-code-backend ./project

# Query via HTTP instead of reading files:
curl http://localhost:3847/edges      # replaces edges.json
curl http://localhost:3847/entities   # replaces entities.json
```

---

## Appendix: Full Endpoint Examples

### GET /stats

```bash
curl http://localhost:3847/stats
```

```json
{
  "success": true,
  "data": {
    "entities": 591,
    "edges": 2541,
    "by_entity_type": {"function": 456, "struct": 78, "trait": 23},
    "by_edge_type": {"Calls": 1890, "Uses": 345, "Implements": 106, "Extends": 45, "Contains": 155},
    "by_language": {"rust": 456, "typescript": 234, "python": 123},
    "db_path": "parseltongue_20251126/analysis.db"
  },
  "tokens": 89
}
```

### GET /callers/{entity}

```bash
curl http://localhost:3847/callers/process
```

```json
{
  "success": true,
  "entity": "process",
  "count": 2,
  "data": [
    {"from": "rust:fn:main:src_main_rs:10", "edge_type": "Calls", "direction": "downward"},
    {"from": "rust:fn:handle:src_api_rs:34", "edge_type": "Calls", "direction": "downward"}
  ],
  "tokens": 120
}
```

### GET /clusters

```bash
curl http://localhost:3847/clusters
```

```json
{
  "success": true,
  "endpoint": "/clusters",
  "count": 5,
  "data": [
    {
      "cluster_id": 1,
      "name": "core-processing",
      "members": ["rust:fn:process", "rust:fn:validate", "rust:fn:transform"],
      "cohesion_score": 0.87
    },
    {
      "cluster_id": 2,
      "name": "api-handlers",
      "members": ["rust:fn:handle", "rust:fn:route", "rust:fn:respond"],
      "cohesion_score": 0.72
    }
  ],
  "algorithm": "LPA",
  "tokens": 250
}
```

### GET /blast/{entity}?hops=N

```bash
curl "http://localhost:3847/blast/process?hops=3"
```

```json
{
  "success": true,
  "entity": "process",
  "max_hops": 3,
  "total_affected": 14,
  "data": [
    {"entity": "main", "depth": 1},
    {"entity": "handle", "depth": 1},
    {"entity": "cli", "depth": 2}
  ],
  "tokens": 200
}
```

### GET /cycles

```bash
curl http://localhost:3847/cycles
```

```json
{
  "success": true,
  "cycle_count": 2,
  "data": [
    {"cycle_id": 1, "entities": ["parser", "lexer", "parser"], "length": 2}
  ],
  "tokens": 150
}
```

### GET /hotspots?top=N

```bash
curl "http://localhost:3847/hotspots?top=5"
```

```json
{
  "success": true,
  "data": [
    {"name": "process", "file": "src/handler.rs", "deps": 34, "risk": "HIGH"},
    {"name": "parse", "file": "src/parser.rs", "deps": 28, "risk": "MEDIUM"}
  ],
  "tokens": 180
}
```

---

## Changelog

### v1.2.0 (2025-11-27) - CPG-Inspired Enhancements

**New Edge Types** (Code Property Graph inspired):
- `Extends` - Inheritance/trait extension (upward direction)
- `Contains` - Structural containment (downward direction)

**Direction Metadata**:
- All edges now include `direction` field: `"downward"`, `"upward"`, `"horizontal"`
- Enables semantic understanding of relationship flow

**New Endpoint**:
- `GET /clusters` - Semantic module groupings via LPA (Label Propagation Algorithm)
- Wraps existing pt08 clustering capability
- Returns cohesion scores for each cluster

### v1.1.0 (2025-11-26) - Initial HTTP Server Architecture

- HTTP server replaces CLI + JSON exports
- 4-word command naming convention
- 12 languages supported via tree-sitter
- TDD-first implementation approach

---

**End of PRD v1.2.0**

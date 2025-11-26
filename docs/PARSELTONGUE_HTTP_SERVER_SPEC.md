# Parseltongue HTTP Server Specification

**Status**: Proposed Architecture
**Date**: 2025-11-26
**Purpose**: Replace CLI + file exports with a local HTTP server for universal agent access

---

## Executive Summary

Parseltongue becomes a **local code intelligence backend**. One command starts an HTTP server that answers questions about your codebase. No JSON file exports. No path management. Just HTTP queries and JSON responses.

```bash
parseltongue serve ./my-project
# Server running at http://localhost:3333

curl http://localhost:3333/callers/export
# {"callers": ["main", "run_export"], "count": 2}
```

---

## Part I: The Problem with Current Architecture

### Current Flow (pt01 + pt02)

```
Step 1: parseltongue pt01-folder-to-cozodb-streamer ./project --db rocksdb:code.db
Step 2: parseltongue pt02-level00 --db rocksdb:code.db --output edges.json
Step 3: Agent reads edges.json
Step 4: Agent parses JSON
Step 5: Agent reasons and responds
```

**Problems**:
- 5+ steps to get an answer
- Database path copied multiple times
- JSON files accumulate (edges.json, entities.json, etc.)
- Agent must know file paths
- Stale exports (code changes, exports don't)

### Proposed Flow (HTTP Server)

```
Step 1: parseltongue serve ./project
Step 2: Agent: curl http://localhost:3333/callers/export
Step 3: Agent reads JSON response, reasons, responds
```

**Benefits**:
- 2 steps to get an answer
- No path management
- No JSON files
- Real-time queries against live database
- Universal access (any HTTP client)

---

## Part II: Storage Architecture

### Where Data Lives

The database location stays **exactly the same** as current pt01:

```
./my-project/
├── src/
├── Cargo.toml
└── parseltongue_20251126183000/    ← Timestamped folder (unchanged)
    └── analysis.db                  ← RocksDB database (unchanged)
```

### What Changes

| Aspect | Before (CLI) | After (HTTP Server) |
|--------|--------------|---------------------|
| Database location | `parseltongue_yymmddhhss/analysis.db` | **Same** |
| Database format | RocksDB + CozoDB | **Same** |
| JSON export files | `edges.json`, `entities.json`, `types.json` | **None** |
| Access method | Read exported files | HTTP queries |

### The Two-Layer Model

```
┌─────────────────────────────────────────────────┐
│  HTTP Server Process (ephemeral)                │
│  - Handles HTTP requests                        │
│  - Queries database                             │
│  - Returns JSON responses                       │
│  - Dies on shutdown                             │
└─────────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────┐
│  parseltongue_yymmddhhss/analysis.db            │
│  - RocksDB + CozoDB                             │
│  - Entities, edges, indexes                     │
│  - Survives server shutdown                     │
│  - Reloaded on next server start                │
└─────────────────────────────────────────────────┘
```

**Key insight**: The database persists. The server is just an HTTP interface to it.

---

## Part III: Server Lifecycle

### First Run (Index + Serve)

```bash
$ parseltongue serve ./my-project

Indexing ./my-project...
  Scanning files...
  Found 156 entities, 870 edges
  Saved to parseltongue_20251126183000/analysis.db

Server running at http://localhost:3333
Press Ctrl+C to stop
```

### Subsequent Runs (Load + Serve)

```bash
$ parseltongue serve ./my-project

Loading parseltongue_20251126183000/analysis.db...
  156 entities, 870 edges

Server running at http://localhost:3333
Press Ctrl+C to stop
```

### Server Shutdown

```
Ctrl+C or kill process
  → Server stops
  → Database remains on disk
  → Next `serve` command loads existing database instantly
```

### Re-indexing (Code Changed)

```bash
$ parseltongue serve ./my-project --reindex

Re-indexing ./my-project...
  Detected 3 changed files
  Updated 12 entities, 45 edges
  Saved to parseltongue_20251126183000/analysis.db

Server running at http://localhost:3333
```

---

## Part IV: HTTP API Specification

### Base URL

```
http://localhost:3333
```

### Endpoints

#### Health & Stats

```
GET /health
→ {"status": "ok", "uptime": "5m32s"}

GET /stats
→ {
    "entities": 156,
    "edges": 870,
    "functions": 120,
    "structs": 24,
    "traits": 8,
    "db_path": "parseltongue_20251126183000/analysis.db"
  }
```

#### Entity Queries

```
GET /entities
→ All entities (Level 1 equivalent)

GET /entities?name=export
→ Entities matching name pattern

GET /entities?type=function
→ Only functions

GET /entities?file=level1.rs
→ Entities in specific file

GET /entity/:key
→ Single entity by ISGL1 key
```

#### Edge Queries

```
GET /edges
→ All edges (Level 0 equivalent)

GET /edges?type=Calls
→ Only call edges
```

#### Graph Queries

```
GET /callers/:entity
→ Who calls this entity (reverse dependencies)

GET /callees/:entity
→ What does this entity call (forward dependencies)

GET /blast/:entity?hops=3
→ Transitive impact (blast radius)

GET /cycles
→ Circular dependencies

GET /hotspots?top=10
→ Most complex entities (by dependency count)
```

#### Search

```
GET /search?q=payment
→ Fuzzy search across entity names, signatures, file paths
```

### Response Format

All responses follow consistent structure:

```json
{
  "success": true,
  "query": "callers/export",
  "count": 2,
  "data": [
    {"from": "rust:fn:main:src_main_rs:45", "edge_type": "Calls"},
    {"from": "rust:fn:run_export:src_cli_rs:120", "edge_type": "Calls"}
  ],
  "tokens_estimate": 150
}
```

Error responses:

```json
{
  "success": false,
  "error": "Entity not found: unknown_function",
  "query": "callers/unknown_function"
}
```

---

## Part V: Agent Integration

### Universal Access

Any agent that can execute HTTP requests can query Parseltongue:

```bash
# Claude Code
curl -s http://localhost:3333/callers/export | jq .

# Python agent
requests.get("http://localhost:3333/callers/export").json()

# JavaScript agent
fetch("http://localhost:3333/callers/export").then(r => r.json())

# Any shell script
wget -qO- http://localhost:3333/callers/export
```

### Agent Workflow Example

```
User: "What functions call the export function in level1.rs?"

Agent (Claude Code):
1. Execute: curl -s http://localhost:3333/callers/export
2. Receive: {"callers": ["main", "run_export"], "count": 2, ...}
3. Format answer: "The export function is called by main() and run_export()"

No file paths. No database paths. No export commands.
```

### Why HTTP Beats MCP for Agent Access

| Aspect | HTTP Server | MCP |
|--------|-------------|-----|
| **Claude Code support** | ✅ curl works natively | ❌ No MCP in terminal |
| **GPT agent support** | ✅ HTTP calls | ❌ No MCP |
| **Cursor support** | ✅ HTTP | ❌ No MCP |
| **Any script** | ✅ curl, fetch, requests | ❌ Needs MCP SDK |
| **Browser access** | ✅ Just open URL | ❌ No |
| **Debugging** | ✅ curl, Postman | ❌ Specialized tools |
| **Implementation** | ~550 LOC | ~1300 LOC |

**HTTP is the universal protocol. Every agent speaks it.**

---

## Part VI: Implementation Plan

### Phase 1: Core Server (Week 1)

```
Component                    LOC
─────────────────────────────────
HTTP server (axum)           150
/stats, /health endpoints     50
/entities, /edges endpoints  100
Startup/indexing logic       100
─────────────────────────────────
Subtotal                     400
```

### Phase 2: Query Endpoints (Week 2)

```
Component                    LOC
─────────────────────────────────
/callers, /callees            80
/blast (blast radius)         60
/cycles                       40
/hotspots                     40
/search                       80
─────────────────────────────────
Subtotal                     300
```

### Phase 3: Polish (Week 3)

```
Component                    LOC
─────────────────────────────────
Error handling                50
Port auto-detection           30
--daemon mode                 50
--reindex flag                50
Documentation                  -
─────────────────────────────────
Subtotal                     180
```

**Total: ~880 LOC for complete HTTP server**

### Tech Stack

- **HTTP Framework**: axum (Rust, async, fast)
- **Database**: CozoDB + RocksDB (unchanged)
- **Serialization**: serde_json (unchanged)
- **Indexing**: pt01 logic (reused)

---

## Part VII: CLI Interface

### Commands

```bash
# Start server (index if needed, load if exists)
parseltongue serve <DIRECTORY> [OPTIONS]

Options:
  --port <PORT>      HTTP port (default: 3333, auto-detect if taken)
  --reindex          Force re-indexing even if database exists
  --daemon           Run in background
  --timeout <MINS>   Auto-shutdown after idle period
  --verbose          Show query logs
```

### Examples

```bash
# Basic usage
parseltongue serve ./my-project

# Custom port
parseltongue serve ./my-project --port 8080

# Background mode
parseltongue serve ./my-project --daemon

# Force fresh index
parseltongue serve ./my-project --reindex

# Auto-shutdown after 30 minutes idle
parseltongue serve ./my-project --timeout 30
```

---

## Part VIII: Migration Path

### What Stays

- `parseltongue_yymmddhhss/` folder structure
- `analysis.db` RocksDB database
- CozoDB schema (CodeGraph, DependencyEdges)
- ISGL1 key format
- Entity extraction logic (pt01)
- Query logic (Datalog patterns)

### What Goes Away

- pt02 CLI commands (replaced by HTTP endpoints)
- JSON file exports (edges.json, entities.json, types.json)
- `--output` flags
- File-based agent workflows

### What's New

- HTTP server with REST-ish API
- Real-time queries
- Universal agent access
- No intermediate files

---

## Part IX: Summary

### The Mental Model

**Parseltongue = Local code intelligence backend**

Like:
- PostgreSQL: Start server → query with any client
- Redis: Start server → query with any client
- **Parseltongue**: Start server → query with HTTP/curl/any agent

### The Value Proposition

```
BEFORE: Index → Export → Read file → Parse → Reason
AFTER:  Serve → Query → Reason

Steps:     5 → 3
Files:     3+ → 0
Latency:   Seconds → Milliseconds
Access:    Parseltongue CLI only → Any HTTP client
```

### The Bottom Line

One command. HTTP queries. JSON responses. Works with every agent.

```bash
parseltongue serve ./my-project
# That's it. Now any agent can ask questions about your code.
```

---

**Document Version**: 1.0
**Architecture Status**: Proposed
**Implementation Estimate**: 3 weeks, ~880 LOC

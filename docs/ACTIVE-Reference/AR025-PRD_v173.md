# PRD: Parseltongue v1.7.3 — Dual-Mode Graph Server + MCP + Desktop App

**Version**: 1.7.3
**Date**: 2026-02-14
**LNO Rating**: LEVERAGE (unblocks Windows + `/db/` and `/mem/` API routing + MCP for native LLM tool use + Tauri desktop companion)

---

## The Problem in One Sentence

CozoDB RocksDB fails on Windows because Windows Defender locks SST files during write-heavy ingestion. Six attempts (v1.6.7-v1.7.2) tried to fix this. All failed.

## The Solution in One Sentence

Two server modes — `/db/` (RocksDB, full data) and `/mem/` (MessagePack snapshot, graph + filesystem reads) — on auto-assigned ports with graceful shutdown. MCP for native LLM tool integration. And a Tauri desktop app that makes dependency graphs visible, always running, across every repo you care about.

---

## End-to-End User Journey

### Act 1: Index Your Codebase

You have a codebase. You want Parseltongue to understand it.

```bash
# RocksDB ingestion (Mac/Linux — full data, file watching)
parseltongue pt01-folder-to-cozodb-streamer ./my-project
```
```
Workspace: parseltongue20260213152200/
Database:  parseltongue20260213152200/analysis.db
Entities:  3,847
Edges:     2,519
Duration:  1.8s
```

```bash
# Snapshot generation (any OS — slim graph, portable file)
parseltongue pt02-folder-to-ram-snapshot ./my-project
```
```
Workspace: parseltongue20260213152230/
Snapshot:  parseltongue20260213152230/analysis.ptgraph (2.7 MB)
Entities:  3,847
Edges:     2,519
Duration:  2.1s
Format:    MessagePack (rmp-serde)
```

Two files exist. Each is independently useful. You don't need both.

### Act 2: Start a Server

Pick your mode. Each gets its own auto-assigned port.

```bash
# Option A: Full database mode
parseltongue pt08-http-code-query-server --db "rocksdb:parseltongue20260213152200/analysis.db"
```
```
Mode:      /db/
Port:      52341 (auto-assigned)
Endpoints: 24/24
Features:  source code in DB, live file watching, full diagnostics
Server:    http://localhost:52341/db/
```

```bash
# Option B: Snapshot mode
parseltongue pt08-http-code-query-server --mem "ptgraph:parseltongue20260213152230/analysis.ptgraph"
```
```
Mode:      /mem/
Port:      52387 (auto-assigned)
Endpoints: 23/24 (diagnostics-coverage unavailable)
Features:  pure RAM, fast startup, source code via filesystem reads, live file watching
Server:    http://localhost:52387/mem/
```

Both can run simultaneously. Different ports, no conflicts. Or run just one — your call.

### Act 3: Query

Every query uses the mode prefix: `/db/` or `/mem/`.

```bash
# Root endpoints — always work, no prefix
curl http://localhost:52341/server-health-check-status
curl http://localhost:52387/server-health-check-status

# /db/ mode — full data from RocksDB
curl http://localhost:52341/db/code-entities-list-all
curl http://localhost:52341/db/code-entity-detail-view?key=rust:fn:main:src_main_rs:1-50
curl http://localhost:52341/db/strongly-connected-components-analysis
curl http://localhost:52341/db/smart-context-token-budget?focus=X&tokens=4000

# /mem/ mode — graph from RAM, source code from filesystem
curl http://localhost:52387/mem/code-entities-list-all
curl http://localhost:52387/mem/code-entity-detail-view?key=rust:fn:main:src_main_rs:1-50
curl http://localhost:52387/mem/strongly-connected-components-analysis
curl http://localhost:52387/mem/blast-radius-impact-analysis?entity=X&hops=3
```

Wrong prefix? Clear error:
```bash
curl http://localhost:52341/mem/code-entities-list-all
# → 404 "This server runs in /db/ mode. Use /db/ prefix."
```

### Act 4: Shut Down

```bash
parseltongue shutdown --db     # stops the /db/ server
parseltongue shutdown --mem    # stops the /mem/ server
```

Discovery: server writes port to `/tmp/parseltongue-server-{mode}.port` on startup. Shutdown reads it, sends `POST /shutdown`, deletes the file.

### Act 5: LLM Integration

Copy this into your LLM agent's system prompt:

```text
PARSELTONGUE CODE ANALYSIS API

Server modes (check which is running):
  /db/*  = Full database (source code stored, live file watching, 24/24 endpoints)
  /mem/* = RAM snapshot (source code read from disk, live file watching, fast, 23/24 endpoints)

BASE URL: http://localhost:{PORT}/{MODE}
Example:  http://localhost:52341/db

ORIENT (no prefix needed):
  GET /server-health-check-status              → health + active mode + port
  GET /api-reference-documentation-help        → full API docs

FIND ENTITIES:
  GET /{mode}/code-entities-list-all
  GET /{mode}/code-entities-search-fuzzy?q=PATTERN
  GET /{mode}/code-entity-detail-view?key=ENTITY_KEY

TRACE DEPENDENCIES:
  GET /{mode}/dependency-edges-list-all
  GET /{mode}/reverse-callers-query-graph?entity=ENTITY_KEY
  GET /{mode}/forward-callees-query-graph?entity=ENTITY_KEY
  GET /{mode}/blast-radius-impact-analysis?entity=ENTITY_KEY&hops=N

ANALYZE ARCHITECTURE:
  GET /{mode}/circular-dependency-detection-scan
  GET /{mode}/complexity-hotspots-ranking-view?top=N
  GET /{mode}/semantic-cluster-grouping-list
  GET /{mode}/strongly-connected-components-analysis
  GET /{mode}/technical-debt-sqale-scoring
  GET /{mode}/kcore-decomposition-layering-analysis
  GET /{mode}/centrality-measures-entity-ranking?method=pagerank
  GET /{mode}/entropy-complexity-measurement-scores
  GET /{mode}/coupling-cohesion-metrics-suite
  GET /{mode}/leiden-community-detection-clusters

CONTEXT:
  GET /{mode}/smart-context-token-budget?focus=ENTITY_KEY&tokens=N

NAVIGATION:
  GET /{mode}/folder-structure-discovery-tree

DIAGNOSTICS (/db/ only for full report):
  GET /{mode}/ingestion-coverage-folder-report?depth=N
  GET /{mode}/ingestion-diagnostics-coverage-report    ← /db/ only, /mem/ returns 501
```

### Act 6: MCP — Native LLM Tool Integration

Instead of copying a system prompt and curling HTTP endpoints, just point your LLM at the MCP server.

```bash
# Start MCP server (stdio transport — Claude Desktop, Claude Code, Cursor, VS Code)
parseltongue pt09-mcp-code-query-server --db "rocksdb:parseltongue20260213152200/analysis.db"
parseltongue pt09-mcp-code-query-server --mem "ptgraph:parseltongue20260213152230/analysis.ptgraph"
```

Add to `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "parseltongue": {
      "command": "parseltongue",
      "args": [
        "pt09-mcp-code-query-server",
        "--db", "rocksdb:parseltongue20260213152200/analysis.db"
      ]
    }
  }
}
```

Now Claude (or any MCP-compatible LLM) can directly call:
```
code_entities_search_fuzzy(q="handle_request")
blast_radius_impact_analysis(entity="rust:fn:main", hops=3)
circular_dependency_detection_scan()
complexity_hotspots_ranking_view(top=10)
smart_context_token_budget(focus="rust:fn:main", tokens=4000)
```

No HTTP proxy. No system prompt. No curl. The LLM discovers available tools via `tools/list` and calls them with typed parameters.

#### MCP Architecture

```
SDK:        rmcp 0.15 (official Rust MCP SDK, modelcontextprotocol org)
Transport:  stdio (primary — local tools, IDE integrations)
Protocol:   JSON-RPC 2.0 over stdin/stdout
```

**Tools (18) — Model-driven, LLM chooses to invoke. All 4-word names.**

| MCP Tool Name | Maps to HTTP Endpoint |
|---------------|----------------------|
| `code_entities_search_fuzzy` | `/code-entities-search-fuzzy` |
| `code_entity_detail_view` | `/code-entity-detail-view` |
| `reverse_callers_query_graph` | `/reverse-callers-query-graph` |
| `forward_callees_query_graph` | `/forward-callees-query-graph` |
| `blast_radius_impact_analysis` | `/blast-radius-impact-analysis` |
| `circular_dependency_detection_scan` | `/circular-dependency-detection-scan` |
| `complexity_hotspots_ranking_view` | `/complexity-hotspots-ranking-view` |
| `semantic_cluster_grouping_list` | `/semantic-cluster-grouping-list` |
| `smart_context_token_budget` | `/smart-context-token-budget` |
| `strongly_connected_components_analysis` | `/strongly-connected-components-analysis` |
| `technical_debt_sqale_scoring` | `/technical-debt-sqale-scoring` |
| `kcore_decomposition_layering_analysis` | `/kcore-decomposition-layering-analysis` |
| `centrality_measures_entity_ranking` | `/centrality-measures-entity-ranking` |
| `entropy_complexity_measurement_scores` | `/entropy-complexity-measurement-scores` |
| `coupling_cohesion_metrics_suite` | `/coupling-cohesion-metrics-suite` |
| `leiden_community_detection_clusters` | `/leiden-community-detection-clusters` |
| `ingestion_coverage_folder_report` | `/ingestion-coverage-folder-report` |
| `ingestion_diagnostics_coverage_report` | `/ingestion-diagnostics-coverage-report` |

**Resources (5) — App-driven, provide passive context.**

| MCP Resource URI | Maps to HTTP Endpoint |
|------------------|----------------------|
| `parseltongue://codebase-statistics-overview-summary` | `/codebase-statistics-overview-summary` |
| `parseltongue://code-entities-list-all` | `/code-entities-list-all` |
| `parseltongue://dependency-edges-list-all` | `/dependency-edges-list-all` |
| `parseltongue://folder-structure-discovery-tree` | `/folder-structure-discovery-tree` |
| `parseltongue://api-reference-documentation-help` | `/api-reference-documentation-help` |

**Excluded from MCP** (infrastructure, not LLM-relevant):
`server-health-check-status`, `incremental-reindex-file-update`, `file-watcher-status-check`

#### Why MCP Matters

| Without MCP | With MCP |
|-------------|----------|
| Copy system prompt into every LLM session | LLM discovers tools automatically via `tools/list` |
| Parse HTTP JSON responses manually | Structured `CallToolResult` with typed content |
| User must know endpoint URLs | LLM reads tool descriptions + JSON Schema params |
| Works with curl/HTTP clients only | Works with Claude Desktop, Claude Code, Cursor, VS Code, any MCP client |
| Stateless per-request | Session-based with capability negotiation |

---

## `/db/` vs `/mem/` — Honest Comparison

### Same Codebase (3,847 entities, benchmarked)

| Endpoint Category | `/db/` (RocksDB) | `/mem/` (snapshot) | Difference |
|---|---|---|---|
| Health check | 0.3ms | 0.3ms | None |
| Entity list (full scan) | 5ms | 5ms | None at this scale |
| Fuzzy search | 7ms | 7ms | None at this scale |
| Detail view | 19ms (from DB) | 19ms (from filesystem) | Same — both cached |
| Graph algorithms | 24-220ms | 24-220ms | Algorithm dominates, not storage |
| Leiden clustering | 8-22s | 8-22s | Pure Rust compute, no DB |

### At 5 GB Scale (~960K entities, projected)

| Factor | `/db/` | `/mem/` | Winner |
|---|---|---|---|
| **Cold start** | 7-35s (RocksDB + first query) | 1.5-2.5s (load ptgraph) | `/mem/` by 10-15x |
| **Full scan query** | 1-5s (warm cache) | 100-300ms (slim data) | `/mem/` by 5-20x |
| **Graph algo total** | 6.5s (2s query + 4.5s algo) | 4.8s (300ms query + 4.5s algo) | `/mem/` by ~30% |
| **Key lookup** | 1-5ms | 1-3ms | Tie |
| **Concurrent queries** | RocksDB compaction contention | No contention | `/mem/` |
| **RAM** | ~256MB cache + OS cache | ~345MB fixed | Tie |
| **Live file watching** | Yes | Yes | Tie — CozoDB mem accepts same watcher updates |
| **Endpoint coverage** | 24/24 | 23/24 | `/db/` |

**Bottom line**: At small scale, identical. At large scale, `/mem/` wins on startup and full scans. Both support live file watching. `/db/` only advantage: the `ingestion-diagnostics-coverage-report` endpoint (needs tables not in snapshot).

---

## The Slim Model

### What `/mem/` Keeps (9 fields per entity)

| Field | Why |
|-------|-----|
| `isgl1_key` | Primary key — every query uses it |
| `file_path` | Read source code from disk at query time |
| `line_start`, `line_end` | Extract exact lines from file |
| `entity_type` | Function, struct, class, trait, etc. |
| `entity_class` | Standard, test, interface |
| `language` | Rust, Python, Go, etc. |
| `root_subfolder_l1`, `l2` | Scope filtering |

### Source Code in `/mem/` Mode

`/mem/` does NOT store source code in the database. When `/mem/code-entity-detail-view` is called:

```
1. Query CozoDB mem for entity metadata (file_path, line_start, line_end)
2. Read source file from disk: std::fs::read_to_string(source_dir + file_path)
3. Extract lines[line_start..line_end]
4. Return in response JSON
```

This means `/mem/` returns **live source code** (current file contents), while `/db/` returns **ingestion-time source code** (what was in the file when pt01 ran). If a file was edited after ingestion, `/mem/` shows the edit, `/db/` doesn't.

### Edge Model (3 fields, unchanged)

`from_key` → `to_key` with `edge_type`. All graph algorithms work identically in both modes.

---

## Endpoint Coverage

| Endpoint | `/db/` | `/mem/` | Notes |
|----------|:------:|:-------:|-------|
| 21 graph/entity/analysis endpoints | Works | Works | Identical behavior |
| `smart-context-token-budget` | Works | Works | Never needed code bodies (false alarm in original design) |
| `code-entity-detail-view` | Works (code from DB) | Works (code from filesystem) | `/mem/` reads live file |
| `ingestion-coverage-folder-report` | Works | Works | Uses CodeGraph + filesystem walk |
| `ingestion-diagnostics-coverage-report` | Works | **501** | Needs TestEntitiesExcluded, FileWordCoverage, IgnoredFiles tables — not in snapshot |

**23 of 24 endpoints work in `/mem/` mode. Only 1 truly cannot.**

---

## Coverage Reporting Bugs (Fix in v1.7.3)

The `ingestion-coverage-folder-report` endpoint is broken. Confirmed live on v1.7.2 against this codebase (1,620 entities, 215 files with entities).

### What's Wrong

```
Reported:  190 parsed / 2,758 eligible = 6.9% coverage
Actual:    215 files with entities, 1,334 test entities excluded across 111 files
Error log: 2,758 entries = EVERY eligible file marked [UNPARSED]
Folders:   ALL show 0% — even crates/ which has 40 files with entities
```

### 5 Bugs, 1 Root Cause

| Bug | Impact | Evidence |
|-----|--------|----------|
| **Path format mismatch** | Folder-level comparison always returns 0% | DB stores absolute (`/Users/.../crates/lib.rs`), handler walks relative (`./crates/lib.rs`). PathBuf comparison never matches. |
| **Error count = eligible count** | Every file is an "error" | 2,758 errors = 2,758 eligible. Zero DB paths match walked paths. Cascading from path mismatch. |
| **No test exclusion awareness** | Test files inflate error count | 1,334 test entities in `TestEntitiesExcluded` table (111 files). Coverage handler never queries it. |
| **No zero-entity category** | `__init__.py` falsely marked unparsed | 144 `__init__.py`/`__main__.py` files parsed successfully but yielded 0 entities. Same `[UNPARSED]` tag as real failures. |
| **Summary vs folder inconsistency** | Summary says 190, folders sum to 0 | Summary uses a different code path that partially works. Folder-level code is completely broken. |

### Fix Plan

1. **Normalize paths before comparison** — strip `./` prefix from walked paths, convert DB absolute paths to relative. Use `path_utils.rs:42-54` (`normalize_split_file_path`).
2. **Query `TestEntitiesExcluded`** — subtract those file paths from unparsed list. Pattern exists at `ingestion_diagnostics_coverage_handler.rs:357-387`.
3. **Add `[ZERO_ENTITIES]` tag** — files that parsed but yielded nothing. Separate from `[UNPARSED]` (actual parser failures).
4. **Add `[TEST_EXCLUDED]` tag** — intentionally excluded test files. Informational, not errors.
5. **Add response fields** — `test_excluded_count`, `zero_entity_count` in JSON summary.

### Corrected Coverage (This Codebase)

```
Before fix:  6.9% (190/2758) — all folders 0%
After fix:   ~55% of crates/ (40 code + ~38 test-excluded = 78 of 142)
             test-fixtures/: ~99% (144 of 145 have entities)
             competitor_research/: genuinely 0% (not ingested by pt01)
```

See: `docs/v173-bugs01-COVERAGE-REPORTING-BUG-ISSUES.md` for per-file analysis.

---

## Terminal Output Cleanup (Fix in v1.7.3)

pt01 ingestion dumps debug `eprintln!` noise to stderr that pollutes LLM context windows. Leftover from v1.5.1 edge investigation, never cleaned up.

| File | Lines | Tag | What It Prints |
|------|-------|-----|----------------|
| `streamer.rs` | 1036-1075 | `[DEBUG-INSERT]` | Per-file edge counts, sample edge keys, insert success/fail |
| `file_watcher.rs` | 475-533 | `[DEBOUNCER]` `[WATCHER]` `[EVENT_HANDLER]` | Watcher startup confirmations, event receipts, channel sends |

**Nothing is lost** — edges are in `DependencyEdges` table (queryable via API), insert failures already surface in the `errors` vec and final summary, watcher status is queryable via `/file-watcher-status-check`.

**Fix**: Delete all `[DEBUG-INSERT]` eprintln blocks in `streamer.rs`. Delete all `[DEBOUNCER]`/`[WATCHER]`/`[EVENT_HANDLER]` eprintln blocks in `file_watcher.rs`.

---

## Respect .gitignore During Ingestion (Fix in v1.7.3)

pt01 uses the `walkdir` crate which **does not respect `.gitignore`**. It has 8 hardcoded exclusions (`target`, `node_modules`, `.git`, `build`, `dist`, `__pycache__`, `.venv`, `venv`) but ignores everything else in `.gitignore`.

This means generated code, vendored dependencies, and any project-specific gitignored files get ingested into the graph — adding noise entities and false edges.

**Fix**: Replace `walkdir` with the `ignore` crate (same author, drop-in API). The `ignore` crate automatically reads `.gitignore`, `.git/info/exclude`, and global gitignore. The 8 hardcoded exclusions become unnecessary — `.gitignore` handles them.

| Before | After |
|--------|-------|
| `walkdir` crate | `ignore` crate |
| 8 hardcoded exclusions in `cli.rs` | `.gitignore` rules (+ keep hardcoded as fallback for non-git dirs) |
| Generated code ingested | Respects project's own exclusion rules |
| `Cargo.toml`: `walkdir = "2.0"` | `Cargo.toml`: `ignore = "0.4"` |

**Files changed**: `streamer.rs` (swap `WalkDir::new()` for `ignore::WalkBuilder::new()`), `cli.rs` (remove hardcoded defaults or keep as fallback), `Cargo.toml` (swap dependency).

**Stale entity cleanup**: The file watcher must watch `.gitignore` itself. When `.gitignore` changes, diff "files currently in graph" vs "files the `ignore` walker would now visit" — compute which entities and edges need to be added or removed, then update the graph accordingly. This prevents phantom dependencies to excluded code and ensures newly un-ignored files get ingested.

---

## XML-Tagged Context Categories (v1.7.3+)

**Insight from**: Zed editor (`agent/src/thread.rs`) — every piece of context sent to an LLM is wrapped in semantic XML tags so the model knows the provenance of each information block.

**Problem**: Parseltongue API responses return flat JSON arrays. An LLM receiving 200 entities in a list has no structural signal about what it's looking at — CoreCode entities, imports, tests, and edges are all mixed together.

**Solution**: Structure API responses with tagged categories so LLMs can filter and reason about entity types without parsing ISGL1 keys.

### Response Format (Current → Proposed)

**Current** (flat):
```json
{
  "entities": [
    {"isgl1_key": "rust|||fn|||login|||src/auth.rs|||5|||20", ...},
    {"isgl1_key": "rust|||fn|||test_login|||src/auth.rs|||81|||100", ...},
    {"isgl1_key": "rust|||import|||auth_imports|||src/auth.rs|||1|||3", ...}
  ]
}
```

**Proposed** (tagged):
```json
{
  "core_code": [
    {"isgl1_key": "rust|||fn|||login|||src/auth.rs|||5|||20", ...},
    {"isgl1_key": "rust|||fn|||register|||src/auth.rs|||23|||50", ...}
  ],
  "test_code": [
    {"isgl1_key": "rust|||test|||test_login|||src/auth.rs|||81|||100", ...}
  ],
  "import_blocks": [
    {"isgl1_key": "rust|||import|||auth_imports|||src/auth.rs|||1|||3", ...}
  ],
  "unparsed_constructs": [],
  "edges": {
    "calls": [...],
    "uses": [...],
    "implements": [...]
  },
  "meta": {
    "total_entities": 5,
    "file_coverage_pct": 100,
    "entity_class_counts": {"core_code": 2, "test_code": 1, "import_blocks": 1, "gap_fragments": 1}
  }
}
```

### Why This Matters

1. **LLM filtering without parsing**: An LLM that only wants production code reads `core_code` and ignores the rest. No regex on ISGL1 keys needed.
2. **Provenance awareness**: The LLM knows "these 3 entities are tests" vs "these 5 are production code" without inferring from names.
3. **Edge type separation**: `edges.calls` vs `edges.uses` vs `edges.implements` lets LLMs reason about relationship types directly.
4. **Coverage transparency**: `meta.entity_class_counts` tells the LLM exactly what's in the response and what was filtered.

### Applies To These Endpoints

All entity-returning endpoints adopt tagged categories:
- `/code-entities-list-all`
- `/code-entity-detail-view`
- `/code-entities-search-fuzzy`
- `/smart-context-token-budget`
- `/blast-radius-impact-analysis`
- `/forward-callees-query-graph`
- `/reverse-callers-query-graph`

Edge-only endpoints (`/dependency-edges-list-all`) group edges by type.

### Connects To

- **ISGL1 v3 entity taxonomy**: CoreCode, TestCode, ImportBlock, GapFragment, UnparsedConstruct map directly to response categories.
- **Exhaustive file coverage**: When every line maps to an entity class, the tagged response covers 100% of the file.
- **See**: `docs/RESEARCH-isgl1v3-exhaustive-graph-identity.md` for the full v3 design.

---

## Project Slug in URL Path (Fix in v1.7.3)

### The Problem

You're running 3 Parseltongue servers:
```
http://localhost:52341/db/code-entities-search-fuzzy?q=login
http://localhost:52387/mem/code-entities-search-fuzzy?q=login
http://localhost:52412/db/code-entities-search-fuzzy?q=login
```

Which project is which? Port numbers are meaningless. When an LLM or a human sees `localhost:52387`, there is zero signal about what codebase that server is analyzing.

### The Fix

Derive a project slug from the ingested folder name and mount all routes under it:

```
http://localhost:52341/parseltongue-dependency-graph/db/code-entities-search-fuzzy?q=login
http://localhost:52387/my-react-app/mem/code-entities-search-fuzzy?q=login
http://localhost:52412/competitor-api/db/code-entities-search-fuzzy?q=login
```

Now every URL is self-describing. An LLM reading the URL knows exactly which codebase it's querying.

### URL Structure

```
http://localhost:{port}/{project-slug}/{mode}/{endpoint}?{params}
```

### How It Works

```
pt01 ingestion:
  1. Folder name = last component of ingested path (e.g., "parseltongue-dependency-graph-generator")
  2. Slug = lowercase, truncate to reasonable length, replace spaces with hyphens
  3. Store slug in workspace config (analysis.db metadata or .ptgraph header)

pt08 startup:
  1. Read slug from database/snapshot metadata
  2. Mount all routes under /{slug}/{mode}/
  3. Print: "Server: http://localhost:{port}/{slug}/{mode}/"

Root endpoints (no slug needed):
  GET /server-health-check-status  →  returns { project: "parseltongue-dependency-graph", mode: "db", port: 52341 }
```

### Edge Cases

- **No slug stored** (old databases): fall back to current behavior (`/{mode}/` only)
- **Duplicate slugs** (two servers for same project name): ports are already different, slug is informational
- **Slug in port file**: `/tmp/parseltongue-server-{slug}-{mode}.port` — now shutdown knows which project to target

### LLM Benefit

When an LLM reads its system prompt, it sees:
```
Parseltongue server: http://localhost:52341/my-project/db/
```

Instead of:
```
Parseltongue server: http://localhost:52341/db/
```

The project name in the URL is context the LLM can use to reason about which codebase it's working with — especially when multiple servers are running.

---

## Port Management

### No `--port` flag. No conflicts. No confusion.

```
Server startup:
  1. Bind to port 0 → OS assigns available port
  2. Write port to /tmp/parseltongue-server-{slug}-{mode}.port
  3. Print: "Server: http://localhost:{port}/{slug}/{mode}/"

Server shutdown:
  1. parseltongue shutdown --{mode}
  2. Read /tmp/parseltongue-server-{slug}-{mode}.port
  3. POST http://localhost:{port}/shutdown
  4. Delete port file

Edge cases:
  - Server crashed? Port file stale → shutdown detects connection refused → deletes file
  - Start same mode twice? Old port file overwritten → old server orphaned (OS reclaims port)
```

---

## Acceptance Criteria

**Ship when all 21 pass.**

| # | Test | Pass Condition |
|---|------|---------------|
| 1 | `cargo build --release` | Zero errors |
| 2 | `cargo test --all` | All existing tests pass |
| 3 | `parseltongue pt02-folder-to-ram-snapshot .` | Produces `.ptgraph` file, exits 0 |
| 4 | `pt08 --db "rocksdb:analysis.db"` | Server starts, prints port, writes port file, `/db/` endpoints work |
| 5 | `pt08 --mem "ptgraph:analysis.ptgraph"` | Server starts, prints port, writes port file, `/mem/` endpoints work |
| 6 | `curl /{mode}/code-entity-detail-view?key=X` | Returns source code (from DB or filesystem) |
| 7 | `curl /{mode}/ingestion-diagnostics-coverage-report` | `/db/` returns data, `/mem/` returns 501 |
| 8 | `parseltongue shutdown --{mode}` | Server stops, port file deleted |
| 9 | `/mem/` file watcher detects edit | Edit a file → entity updated in CozoDB mem → query returns new data |
| 10 | `pt09 --db "rocksdb:analysis.db"` | MCP server starts on stdio, `tools/list` returns 18 tools |
| 11 | MCP `tools/call` round-trip | `code_entities_search_fuzzy(q="main")` returns results via JSON-RPC |
| 12 | Coverage excludes test files | `ingestion-coverage-folder-report` subtracts `TestEntitiesExcluded` — no test file false positives |
| 13 | Coverage separates zero-entity files | `__init__.py` files tagged `[ZERO_ENTITIES]`, not `[UNPARSED]` |
| 14 | Path normalization consistent | `./crates/foo.rs` and `crates/foo.rs` match correctly in coverage comparison |
| 15 | Error log has 3 categories | `ingestion-errors.txt` uses `[UNPARSED]`, `[TEST_EXCLUDED]`, `[ZERO_ENTITIES]` tags |
| 16 | No debug eprintln in pt01 | `grep -n "DEBUG-INSERT\|DEBOUNCER\|WATCHER\|EVENT_HANDLER" streamer.rs file_watcher.rs` returns 0 matches |
| 17 | API responses use tagged categories | `/code-entities-list-all` response has `core_code`, `test_code`, `import_blocks`, `edges` top-level keys (not flat array) |
| 18 | Ingestion respects .gitignore | Files in `.gitignore` are not ingested; `ignore` crate used instead of `walkdir` |
| 19 | .gitignore changes update graph | Edit `.gitignore` → file watcher detects it → newly ignored files' entities/edges removed, newly un-ignored files ingested |
| 20 | URL contains project slug | `curl http://localhost:{port}/{slug}/db/server-health-check-status` works; slug derived from ingested folder name |
| 21 | Entities have real token counts | `CodeGraph` relation has `token_count` column; `smart-context-token-budget` uses it instead of heuristic |

---

## Build Order

```
DONE  1. Slim types             → entities.rs           (+50 lines)
DONE  2. DB getter              → streamer.rs           (+5 lines)
DONE  3. Export + Import         → cozo_client.rs        (+150 lines)
DONE  5. pt08 snapshot loader    → snapshot_loader + startup (+145 lines)
DONE  6. Endpoint guards         → 3 handler files       (+15 lines)

TODO  4. pt02 crate             → new crate             (~130 lines)
TODO  7. Route prefix nesting    → startup runner        (~15 lines)
TODO  8. Auto port + port file   → startup runner        (~15 lines)
TODO  9. POST /shutdown endpoint → startup runner        (~15 lines)
TODO 10. shutdown CLI command    → main.rs               (~25 lines)
TODO 11. Filesystem source read  → detail view handler   (~20 lines)
TODO 12. Unblock smart-context   → remove false 501 guard (~5 lines)
TODO 13. CLI --db/--mem flags    → main.rs + Cargo.toml  (~40 lines)
TODO 14. /mem/ file watching      → startup runner        (~10 lines, remove snapshot guard on watcher)
TODO 15. pt09 MCP crate          → new crate             (~400 lines)
TODO 16. MCP tool definitions    → pt09 tool registry    (~350 lines)
TODO 17. MCP resource providers  → pt09 resource module  (~80 lines)
TODO 18. MCP CLI subcommand      → main.rs + Cargo.toml  (~30 lines)
TODO 19. Coverage: exclude tests  → coverage handler      (~20 lines, query TestEntitiesExcluded)
TODO 20. Coverage: zero-entity tag → coverage handler     (~15 lines, [ZERO_ENTITIES] vs [UNPARSED])
TODO 21. Coverage: path normalize → coverage handler      (~5 lines, strip ./ prefix)
TODO 22. Coverage: error log tags → coverage handler      (~15 lines, [TEST_EXCLUDED] + [ZERO_ENTITIES] tags)
TODO 23. Remove debug eprintln    → streamer.rs           (~-40 lines, delete [DEBUG-INSERT] blocks)
TODO 24. Remove watcher eprintln  → file_watcher.rs       (~-30 lines, delete [DEBOUNCER]/[WATCHER]/[EVENT_HANDLER] blocks)
TODO 25. XML-tagged responses     → all entity handlers   (~60 lines, group by entity class + edge type)
TODO 26. Swap walkdir → ignore   → streamer.rs + Cargo.toml (~20 lines, respect .gitignore)
TODO 27. Project slug in URL     → pt01 metadata + pt08 router (~30 lines, derive slug from folder name)
TODO 28. Slug in port file       → startup runner         (~5 lines, /tmp/parseltongue-server-{slug}-{mode}.port)
TODO 29. Token count at ingest   → streamer.rs + entities.rs + cozo_client.rs (~40 lines, bpe-openai count per entity)
TODO 30. Smart-context real tokens → smart_context handler  (~10 lines, read token_count column instead of heuristic)
TODO 31. README audit            → README.md             (verify)
TODO 32. Testing journal         → root .md file         (document)
```

**Done: ~365 lines. Remaining: ~1,275 lines. Total: ~1,640 lines.**

---

## Act 7: Desktop Companion — Parseltongue Central (Tauri, macOS)

### Moment Zero: You Find It

You're on GitHub. `github.com/that-in-rust/parseltongue/releases/v1.7.3`. You see two assets:

```
parseltongue-v1.7.3-aarch64-apple-darwin.tar.gz     (CLI — 12 MB)
Parseltongue.Central-v1.7.3-aarch64.dmg             (Desktop app — 3 MB)
```

You download both. The CLI goes in your `$PATH`. You double-click the `.dmg`.

### Moment One: First Launch

You drag `Parseltongue Central` to Applications. You open it.

**What you see**: Nothing happens in the foreground. No window. Instead, a small snake icon appears in your macOS menubar, next to WiFi and battery. It's gray — no servers running yet.

You click the icon. A small popup appears below it:

```
┌─────────────────────────────────┐
│  Parseltongue Central           │
│                                 │
│  No repos tracked yet.          │
│                                 │
│  [+ Add Repo]                   │
│                                 │
│  ─────────────────────────────  │
│  Quit                           │
└─────────────────────────────────┘
```

No onboarding wizard. No sign-in. No tutorial. One button.

### Moment Two: You Add Your First Repo

You click **[+ Add Repo]**. A native macOS folder picker opens. You navigate to `~/code/my-project` and click Open.

A small modal:

```
┌─────────────────────────────────────────────┐
│  Index: ~/code/my-project                   │
│                                             │
│  [/db/ Persistent]    [/mem/ In-Memory]     │
│                                             │
│  /db/ = RocksDB on disk, survives restart   │
│  /mem/ = CozoDB in RAM, fast, ephemeral     │
└─────────────────────────────────────────────┘
```

You click **/db/ Persistent**. The modal closes. In the tray popup, your repo appears with a spinner:

```
┌─────────────────────────────────┐
│  Parseltongue Central           │
│                                 │
│  ◌ my-project                   │
│    Indexing... 847 entities      │
│                                 │
│  [+ Add Repo]                   │
└─────────────────────────────────┘
```

Behind the scenes, the Tauri Rust backend:
1. Spawns `parseltongue pt01-folder-to-cozodb-streamer ~/code/my-project`
2. Captures the workspace path from stdout
3. Spawns `parseltongue pt08-http-code-query-server --db "rocksdb:workspace/analysis.db"`
4. Watches `/tmp/parseltongue-server-db.port` for the port file to appear
5. Health-checks the new server

For **/mem/ In-Memory**, the same flow but with `db_path = "mem"` — pt01 parses directly into CozoDB RAM, pt08 serves from that in-memory database. No disk writes. No `.ptgraph` file. Just pure RAM. Faster startup, no persistence across restarts.

After a few seconds, the spinner becomes a green dot. The tray icon turns green.

```
┌─────────────────────────────────┐
│  Parseltongue Central           │
│                                 │
│  ● my-project                   │
│    /db/ :7789 — 1,620 entities  │
│                                 │
│  [+ Add Repo]                   │
└─────────────────────────────────┘
```

### Moment Three: You Open The Graph

You click **● my-project** in the tray popup. The main window opens for the first time. Dark. Almost black. Inter font. Parseltongue green accents.

Three panels:

```
┌──────────────┬────────────────────────────────────────────────┐
│              │                                                │
│  REPOS       │         ARCHITECTURE OVERVIEW                  │
│              │                                                │
│  ● my-proj.. │    Nodes colored by language.                  │
│    /db :7789 │    Clusters drawn around Leiden communities.   │
│              │    Node size = coupling score.                 │
│              │                                                │
│              │    You see your codebase for the first time    │
│              │    as a living structure. The core modules      │
│              │    are dense clusters in the center. Utility    │
│              │    files orbit the edges. You notice a weird   │
│              │    bridge between two clusters you didn't       │
│              │    expect to be connected.                     │
│              │                                                │
│              ├────────────────────────────────────────────────│
│  [+ Add]     │  Click any node to explore                     │
└──────────────┴────────────────────────────────────────────────┘
```

The graph is force-directed (Cytoscape.js, fCoSE layout). Powered by `/dependency-edges-list-all` + `/leiden-community-detection-clusters`. Every node is an entity. Every edge is a dependency.

### Moment Four: You Click A Node

You click on a node labeled `rust:fn:handle_request`. The graph transitions to the **butterfly view** — callers on the left, entity in the center, callees on the right.

```
                  reverse callers                    forward callees
                  ────────────►   ┌─────────────┐   ────────────►
   ┌──────────┐                   │             │                  ┌───────────┐
   │ router   │──────────────────▶│  handle_    │─────────────────▶│ db_query  │
   └──────────┘                   │  request    │                  └───────────┘
   ┌──────────┐                   │             │                  ┌───────────┐
   │ main     │──────────────────▶│             │─────────────────▶│ serialize │
   └──────────┘                   └─────────────┘                  └───────────┘
```

Powered by `/reverse-callers-query-graph` + `/forward-callees-query-graph`.

The bottom panel shows entity detail — file path, line range, language, entity type. Powered by `/code-entity-detail-view`.

You click **db_query**. The graph re-centers. Now db_query is in the middle with its own callers and callees. You're navigating your architecture by clicking.

### Moment Five: You Add A Competitor Repo

You click **[+ Add]** in the sidebar. Folder picker. `~/code/competitor-repo`. Choose **/mem/ In-Memory** this time — you're just exploring, you don't need persistence.

Indexing happens. Green dot. Two repos in your sidebar:

```
│  ● my-project                   │
│    /db :7789 — 1,620 entities   │
│                                 │
│  ● competitor                   │
│    /mem :7801 — 4,200 entities  │
```

You click **competitor**. The graph re-renders with the competitor's architecture. Different structure. Different clusters. You can visually compare — are their modules tighter or looser than yours? Where are their hotspots?

### Moment Six: You Close The Window

Cmd+W. The window closes. Parseltongue Central is still in your menubar. Green dot. Servers still running. File watchers still watching.

You edit a file in your codebase. The file watcher picks it up. Entity re-indexed. Next time you open the graph, it reflects the change. You didn't do anything.

You close your laptop. Open it tomorrow. Tray icon is there. For `/db/` repos, servers respawn automatically from their RocksDB databases. For `/mem/` repos, the app re-ingests on launch (in-memory means re-parse — but it's fast).

### Moment Seven: You Right-Click

Right-click a repo in the sidebar:

```
┌─────────────────────────┐
│  Re-index               │
│  Stop Server            │
│  Open in Terminal       │
│  Copy Server URL        │
│  Remove                 │
└─────────────────────────┘
```

- **Re-index**: Stops server, re-runs pt01, restarts. Fresh entities.
- **Stop Server**: `POST /shutdown`. Dot goes gray. Tray icon updates.
- **Open in Terminal**: Opens the repo directory in Terminal.app.
- **Copy Server URL**: `http://localhost:7789/db/` → clipboard. Paste into Claude, Cursor, any LLM.
- **Remove**: Stops server, removes from sidebar. Workspace files stay on disk.

### What You Never Have To Do

- You never open a terminal to start a Parseltongue server.
- You never remember a port number.
- You never type a `curl` command.
- You never wonder "is my server still running?"
- You never manually kill a zombie process.

The CLI still exists. Power users can use it directly. But Parseltongue Central means you don't have to.

### Three Views

| View | Inspired By | Powered By Endpoints |
|------|-------------|---------------------|
| **Architecture overview** | Emerge (force-directed + community hulls) | `/dependency-edges-list-all` + `/leiden-community-detection-clusters` |
| **Entity explorer** | Sourcetrail (butterfly: callers ← entity → callees) | `/reverse-callers-query-graph` + `/forward-callees-query-graph` + `/code-entity-detail-view` |
| **Repo fingerprint** | repo-visualizer (circle-packing treemap) | `/ingestion-coverage-folder-report` + `/codebase-statistics-overview-summary` |

### Tech Stack

| Layer | Choice | Why |
|-------|--------|-----|
| Framework | Tauri v2 | ~3 MB bundle, sub-second startup, native macOS WebView |
| Frontend | Svelte 5 + TypeScript | ~10 KB, official Tauri template |
| Styling | Tailwind CSS (dark mode) | Linear-inspired: near-black bg, Inter font, green accent |
| Graph viz | Cytoscape.js (fCoSE layout) | Handles 5K nodes, compound nodes, multiple layouts |
| Graph viz (large) | Sigma.js + Graphology (upgrade path) | WebGL for 10K+ nodes if needed |
| HTTP | `reqwest` (Rust side) | Polls localhost Parseltongue servers, no CORS |
| File watching | `notify` crate (Rust side) | Watches `/tmp/` for port file changes |
| Child processes | `tokio::process::Command` | Spawns pt01/pt08 for ingestion + serving |
| Persistence | `~/.parseltongue/repos.json` | Remembers repos across app restarts |

### Design Principles

1. **Dark-first**: Near-black (#0D0D0D), high-contrast, Inter font
2. **Monochrome + one accent**: Grays for chrome, Parseltongue green for life
3. **Node colors**: Blue = function, Purple = class/struct, Orange = trait/interface, Gray = file
4. **Tray icon**: `icon_as_template(false)` — green = all up, red = any down, gray = none
5. **Click-to-explore**: Sourcetrail model — click any node, graph re-centers
6. **No onboarding**: One button. One action. Zero configuration.

### What It Does NOT Need

- No database of its own (reads from running Parseltongue servers)
- No settings page (nothing to configure)
- No log viewer (terminal exists)
- No Apple Developer account (ad-hoc signing for personal use)
- No Electron (Tauri is 3 MB, Electron is 150 MB)

---

## Structural Taint Analysis (New in v1.7.3)

> **Reference Documents**
>
> - **Thesis**: [`docs/THESIS-taint-analysis-for-parseltongue.md`](THESIS-taint-analysis-for-parseltongue.md) — 786-line technical thesis covering academic foundations (Denning 1976, Joern CPG, DOOP/Souffle), code-scalpel deep-dive (2,466-line taint_tracker.py), semgrep MCP analysis, proposed CozoDB Datalog architecture, CWE Top 25 (2025) mapping, false positive rate benchmarks, tooling gap analysis, and Shreyas Doshi LNO assessment. 25 references.
> - **Z3 Tradeoff**: [`docs/v173-z3-tradeoff-analysis.md`](v173-z3-tradeoff-analysis.md) — Architecture decision record for why Parseltongue uses CozoDB Datalog reachability instead of Z3 SMT solving. Covers three problems (C++ binary dependency, 12-language scaling, NP-hard vs polynomial category mismatch), quantified precision trade-off (~10% FP with Z3 vs ~20% without), and what we honestly lose (path feasibility pruning, constraint-based sanitizer verification).
> - **Competitor Research**: [`docs/CR-v173-03.md`](CR-v173-03.md) — 125KB competitor feature deep-dive with code-scalpel and semgrep taint implementation details from direct source reading.

### The Problem

Security teams need to know: "Can user input reach a SQL query without sanitization?" Today, only two tools answer this — code-scalpel (Python AST + Z3 symbolic solver) and semgrep (pattern-matching `mode: taint`). Both are Python-only or require external tooling. Neither integrates with a dependency graph database.

Parseltongue already has the graph. Every function call is an edge in CozoDB. The missing piece: **data-flow edges** — tracking how values move through assignments, parameters, and returns — and **source/sink classification** — labeling which entities produce dangerous data and which consume it unsafely.

### The Insight

Taint analysis has three layers:

| Layer | What It Tracks | code-scalpel | semgrep | Parseltongue (proposed) |
|-------|---------------|-------------|---------|------------------------|
| **Structural** | "A path exists from source entity to sink entity through call edges" | Yes | Yes | Yes — CozoDB Datalog reachability query |
| **Intra-procedural** | "Variable `x` assigned from `request.get()` flows to `cursor.execute(x)`" | Yes (Python AST walker) | Yes (pattern matching) | Yes — tree-sitter assignment/param extraction |
| **Symbolic** | "Does `html_escape(x)` actually neutralize SQL injection?" | Yes (Z3 solver) | No | **No** — curated registry instead |

Parseltongue targets Layers 1 + 2. Layer 3 (symbolic reasoning) is explicitly out of scope — we use a curated sanitizer registry instead of Z3. This means our taint analysis is **structural + intra-procedural** but not symbolic. The tradeoff: ~15% more false positives than code-scalpel, but works across 12 languages (not just Python), runs in milliseconds (not seconds), and leverages the existing graph.

### How It Works — End to End

#### Step 1: Tree-Sitter Extracts Data-Flow Edges (pt01 ingestion)

During ingestion, new tree-sitter queries extract three new edge types:

```
DataFlowEdge types:
  assign    — x = foo()         → edge from foo()'s return to x
  param     — bar(x)            → edge from x to bar()'s parameter
  return    — return x          → edge from x to bar()'s return value
```

These are intra-function edges. They connect to the existing `DependencyEdges` (call edges) to form a complete flow graph.

**Tree-sitter query example** (Rust — assignment extraction):
```scheme
;; Captures: let x = some_function_call();
(let_declaration
  pattern: (identifier) @assign_target
  value: (call_expression
    function: (_) @assign_source))
```

**Tree-sitter query example** (Python — parameter passing):
```scheme
;; Captures: some_function(tainted_var)
(call
  function: (_) @call_target
  arguments: (argument_list
    (_) @param_value))
```

Each language gets 3-5 new queries. The existing tree-sitter infrastructure in `parseltongue-core` handles this — same pattern as entity extraction.

#### Step 2: Source/Sink Classification (curated registry)

A Rust module defines known taint sources and sinks per language:

```rust
/// Taint source — where dangerous data enters the program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaintSourceKind {
    UserInput,        // request.get(), input(), stdin, argv
    FileContent,      // read(), open(), fs::read_to_string
    NetworkData,      // recv(), fetch(), http::get
    DatabaseQuery,    // cursor.fetchone(), query.row()
    EnvironmentVar,   // env::var(), os.environ
    CommandLineArg,   // sys.argv, std::env::args
    Deserialization,  // serde_json::from_str, pickle.loads
}

/// Security sink — where unsanitized data becomes a vulnerability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySinkKind {
    SqlQuery,         // execute(), query(), raw SQL — CWE-89
    ShellCommand,     // system(), exec(), Command::new — CWE-78
    HtmlOutput,       // innerHTML, render(), write_html — CWE-79
    FileWrite,        // write(), create(), open("w") — CWE-73
    NetworkSend,      // send(), post(), fetch — CWE-918
    Deserialization,  // from_str with untrusted data — CWE-502
    PathTraversal,    // open(user_path), read_dir — CWE-22
    LogInjection,     // log::info!("{}", user_data) — CWE-117
    RedirectUrl,      // redirect(url), Location header — CWE-601
    LdapQuery,        // ldap.search(filter) — CWE-90
    XpathQuery,       // xpath(expr) — CWE-643
    RegexInput,       // Regex::new(user_input) — CWE-1333
}
```

**Pattern matching** — sources and sinks are identified by matching entity names and signatures against curated patterns:

```rust
/// Source patterns per language
static TAINT_SOURCES: LazyLock<HashMap<&str, Vec<SourcePattern>>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("python", vec![
        SourcePattern::new("request.get", TaintSourceKind::UserInput),
        SourcePattern::new("request.form", TaintSourceKind::UserInput),
        SourcePattern::new("input", TaintSourceKind::UserInput),
        SourcePattern::new("sys.argv", TaintSourceKind::CommandLineArg),
        SourcePattern::new("os.environ", TaintSourceKind::EnvironmentVar),
        SourcePattern::new("open", TaintSourceKind::FileContent),
        // ... 20+ patterns per language
    ]);
    m.insert("rust", vec![
        SourcePattern::new("std::io::stdin", TaintSourceKind::UserInput),
        SourcePattern::new("std::env::args", TaintSourceKind::CommandLineArg),
        SourcePattern::new("std::env::var", TaintSourceKind::EnvironmentVar),
        SourcePattern::new("std::fs::read", TaintSourceKind::FileContent),
        SourcePattern::new("reqwest::get", TaintSourceKind::NetworkData),
        // ...
    ]);
    // JavaScript, TypeScript, Go, Java, C, C++, Ruby, PHP, C#, Swift
    m
});
```

During pt01 ingestion, entities matching source patterns get a `taint_source: Some(TaintSourceKind)` field. Entities matching sink patterns get a `taint_sink: Some(SecuritySinkKind)` field.

#### Step 3: Sanitizer Registry (curated, not symbolic)

```rust
/// Maps sanitizer function names to which sink types they neutralize
static SANITIZER_REGISTRY: LazyLock<HashMap<&str, Vec<SecuritySinkKind>>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    // SQL sanitizers
    m.insert("escape_sql", vec![SecuritySinkKind::SqlQuery]);
    m.insert("parameterize", vec![SecuritySinkKind::SqlQuery]);
    m.insert("sqlx::query", vec![SecuritySinkKind::SqlQuery]); // parameterized by design
    m.insert("prepared_statement", vec![SecuritySinkKind::SqlQuery]);

    // HTML/XSS sanitizers
    m.insert("html_escape", vec![SecuritySinkKind::HtmlOutput]);
    m.insert("sanitize_html", vec![SecuritySinkKind::HtmlOutput]);
    m.insert("encode_html", vec![SecuritySinkKind::HtmlOutput]);
    m.insert("bleach.clean", vec![SecuritySinkKind::HtmlOutput]);
    m.insert("DOMPurify.sanitize", vec![SecuritySinkKind::HtmlOutput]);

    // Shell sanitizers
    m.insert("shlex.quote", vec![SecuritySinkKind::ShellCommand]);
    m.insert("shell_escape", vec![SecuritySinkKind::ShellCommand]);

    // Path sanitizers
    m.insert("canonicalize", vec![SecuritySinkKind::PathTraversal]);
    m.insert("realpath", vec![SecuritySinkKind::PathTraversal]);

    // Multi-purpose
    m.insert("validate_input", vec![
        SecuritySinkKind::SqlQuery,
        SecuritySinkKind::ShellCommand,
        SecuritySinkKind::HtmlOutput,
    ]);

    // ... 60+ entries across all languages
    m
});
```

**Key difference from code-scalpel**: code-scalpel uses Z3 to symbolically reason "does `html_escape(x)` neutralize SQL injection?" (answer: no). We use a lookup table. This means if a sanitizer isn't in our registry, we can't reason about it. Tradeoff: faster, simpler, works across 12 languages, but less precise for custom sanitizers.

#### Step 4: CozoDB Relations (graph storage)

```
TaintSources {
  entity_key: String,         // ISGL1 key of source entity
  source_kind: String,        // UserInput, FileContent, etc.
  confidence: Float,          // 1.0 = exact match, 0.7 = heuristic
}

TaintSinks {
  entity_key: String,         // ISGL1 key of sink entity
  sink_kind: String,          // SqlQuery, ShellCommand, etc.
  cwe_id: String,             // CWE-89, CWE-78, etc.
  confidence: Float,
}

DataFlowEdges {
  from_key: String,           // Source entity
  to_key: String,             // Target entity
  flow_type: String,          // assign, param, return
  variable_name: String,      // The variable carrying the data
}

Sanitizers {
  entity_key: String,         // ISGL1 key of sanitizer entity
  neutralizes: [String],      // List of SecuritySinkKind values
}
```

#### Step 5: Taint Propagation via Datalog (the query)

The core taint query is a recursive CozoDB Datalog rule:

```datalog
# Find all taint flows: source → ... → sink (without sanitization)
tainted_path[source, sink, path] :=
  # Start from a known taint source
  *TaintSources[source, source_kind, _],
  # Find reachable sinks via data-flow + call edges
  *DataFlowEdges[source, next, _, _],
  reachable[next, sink, path],
  # Sink must be a known security sink
  *TaintSinks[sink, sink_kind, cwe, _],
  # No sanitizer on the path neutralizes this sink type
  not sanitized_for[path, sink_kind]

# Recursive reachability through data-flow and call edges
reachable[start, end, path] :=
  *DataFlowEdges[start, end, _, _],
  path = [start, end]
reachable[start, end, path] :=
  *DataFlowEdges[start, mid, _, _],
  reachable[mid, end, rest],
  path = [start | rest]
# Also traverse call edges (cross-function flow)
reachable[start, end, path] :=
  *DependencyEdges[start, mid, _],
  reachable[mid, end, rest],
  path = [start | rest]

# A path is sanitized if any entity on it is a sanitizer for the sink type
sanitized_for[path, sink_kind] :=
  *Sanitizers[entity, neutralizes],
  sink_kind in neutralizes,
  entity in path
```

This runs in milliseconds for typical codebases (< 50K entities). CozoDB's Datalog engine handles the recursion natively.

#### Step 6: HTTP Endpoints (2 new)

```
GET /{mode}/taint-flow-path-analysis?entity=X&hops=N
  → Returns all taint flows reachable from entity X within N hops
  → Response: { flows: [{ source, sink, path, cwe_id, confidence }] }

GET /{mode}/taint-source-sink-discovery
  → Returns all classified sources and sinks in the codebase
  → Response: { sources: [...], sinks: [...], sanitizers: [...],
                summary: { total_sources, total_sinks, unsanitized_flows } }
```

#### Step 7: MCP Tools (2 new)

```
taint_flow_path_analysis(entity="rust:fn:handle_request", hops=5)
taint_source_sink_discovery()
```

### CWE Mapping

| Sink Type | CWE | OWASP | Severity |
|-----------|-----|-------|----------|
| SqlQuery | CWE-89 | A03:2021 Injection | Critical |
| ShellCommand | CWE-78 | A03:2021 Injection | Critical |
| HtmlOutput | CWE-79 | A03:2021 Injection | High |
| FileWrite | CWE-73 | A01:2021 Broken Access | High |
| PathTraversal | CWE-22 | A01:2021 Broken Access | High |
| Deserialization | CWE-502 | A08:2021 Integrity | Critical |
| NetworkSend | CWE-918 | A10:2021 SSRF | High |
| LogInjection | CWE-117 | A09:2021 Logging | Medium |
| RedirectUrl | CWE-601 | A01:2021 Broken Access | Medium |
| LdapQuery | CWE-90 | A03:2021 Injection | High |
| XpathQuery | CWE-643 | A03:2021 Injection | High |
| RegexInput | CWE-1333 | A03:2021 Injection | Medium |

### What This Is NOT

- **Not a SAST scanner** — we don't replace semgrep or SonarQube. We provide taint flow *visibility* in a dependency graph.
- **Not symbolic** — we can't reason about whether `custom_sanitize(x)` actually works. We only know about sanitizers in our registry.
- **Not a vulnerability scanner** — we find *potential* taint flows. A human or LLM must assess whether each flow is a real vulnerability.
- **Not a replacement for code-scalpel** — code-scalpel's Z3 reasoning catches things we'll miss. Our advantage: 12 languages, millisecond queries, graph integration. See [`docs/v173-z3-tradeoff-analysis.md`](v173-z3-tradeoff-analysis.md) for the full Z3 vs Datalog decision record.

### Honest Accuracy Assessment

| Metric | code-scalpel | semgrep | Parseltongue (projected) |
|--------|-------------|---------|--------------------------|
| Languages | Python only | 30+ (but taint mode quality varies) | 12 (tree-sitter supported) |
| Intra-function flow | High (AST walker) | High (pattern engine) | Medium (tree-sitter assignment/param) |
| Cross-function flow | High (inter-procedural) | Low (intra-procedural) | High (CozoDB graph reachability) |
| Sanitizer reasoning | Z3 symbolic | Pattern match | Curated registry |
| False positive rate | ~10% | ~20% | ~25% (estimated) |
| Query speed | Seconds | Seconds | Milliseconds (pre-computed graph) |
| Integration | Standalone CLI | Standalone CLI / MCP | Native in dependency graph + MCP |

### Files Changed

- `parseltongue-core/src/taint/mod.rs` — NEW: TaintSourceKind, SecuritySinkKind, sanitizer registry, source/sink patterns (~400 lines)
- `parseltongue-core/src/taint/registry.rs` — NEW: TAINT_SOURCES, TAINT_SINKS, SANITIZER_REGISTRY static maps (~300 lines)
- `parseltongue-core/src/taint/tree_sitter_queries/` — NEW: data-flow extraction queries per language (~150 lines, 12 files)
- `parseltongue-core/src/storage/cozo_client.rs` — Add TaintSources, TaintSinks, DataFlowEdges, Sanitizers relations (~60 lines)
- `pt01/.../streamer.rs` — Extract data-flow edges during ingestion, classify sources/sinks (~80 lines)
- `pt08/.../taint_flow_path_analysis_handler.rs` — NEW: Datalog taint query endpoint (~100 lines)
- `pt08/.../taint_source_sink_discovery_handler.rs` — NEW: Source/sink listing endpoint (~60 lines)
- `pt08/.../startup_runner.rs` — Register 2 new routes (~4 lines)

**Total: ~1,150 lines of new Rust code. No new dependencies** (tree-sitter and CozoDB already in the stack).

### Build Order (appended to existing)

```
TODO 33. Taint types + enums        → parseltongue-core/src/taint/mod.rs     (~100 lines)
TODO 34. Source/sink registry        → parseltongue-core/src/taint/registry.rs (~300 lines)
TODO 35. Data-flow tree-sitter queries → parseltongue-core/src/taint/queries/  (~150 lines)
TODO 36. CozoDB taint relations      → cozo_client.rs                         (~60 lines)
TODO 37. pt01 taint extraction       → streamer.rs                            (~80 lines)
TODO 38. Taint flow endpoint         → taint_flow_path_analysis_handler.rs    (~100 lines)
TODO 39. Source/sink discovery endpoint → taint_source_sink_discovery_handler.rs (~60 lines)
TODO 40. MCP taint tools             → pt09 tool registry                     (~40 lines)
```

### Acceptance Criteria (appended)

| # | Test | Pass Condition |
|---|------|---------------|
| 22 | `cargo test -p parseltongue-core -- taint` | Taint registry tests pass — sources, sinks, sanitizers correctly classified |
| 23 | pt01 ingestion with taint | `TaintSources`, `TaintSinks`, `DataFlowEdges` relations populated after ingestion |
| 24 | `/taint-flow-path-analysis?entity=X` | Returns taint flows with source→path→sink, CWE IDs |
| 25 | `/taint-source-sink-discovery` | Returns classified sources, sinks, sanitizers, unsanitized flow count |
| 26 | Sanitizer interrupts flow | Path through known sanitizer NOT reported as unsanitized |
| 27 | MCP tools work | `taint_flow_path_analysis()` and `taint_source_sink_discovery()` return results via JSON-RPC |

---

## What We're NOT Building

| Temptation | Why Not |
|------------|---------|
| Dual-backend on single port | Loading both wastes RAM. `/mem/` is subset of `/db/`. Run them separately. |
| pt03 JSON exporter crate | It's a `--format` flag if ever needed. Not a crate. |
| `--port` flag | Auto-assign eliminates port conflicts. Port file enables shutdown discovery. |
| Runtime backend switching | Just start the server you want. `shutdown` + restart is fast enough. |
| Tiered endpoint system | `/mem/` serves 23/24. One 501 is not worth a tier system. |
| Schema versioning | v1 is the only version. |

---

## Risks (6)

| Risk | So What | Mitigation |
|------|---------|-----------|
| Peak RAM during pt02 (~3.2 GB at 400K entities) | Can't generate snapshot on 8GB machine for Linux kernel | One-time CLI. Run on bigger machine, copy .ptgraph. |
| `/mem/` filesystem read fails (file moved/deleted) | Detail view returns error instead of code | Return clear error: "Source file not found at {path}. Re-run pt02 if codebase moved." |
| CozoDB batch insert limits | Large codebases exceed query size | Chunk 5000 entities/batch (already implemented). |
| `rmcp` SDK is v0.15 (pre-1.0) | API may change in future versions | Pin exact version. stdio transport is stable in spec. Wrapper is thin (~400 lines) — easy to update. |
| Taint false positives (~25%) | Users may lose trust if too many flows are not real vulnerabilities | Confidence scores per flow. Label as "structural taint" not "vulnerability". Let LLM/human triage. |
| Curated sanitizer registry gaps | Custom sanitizers not in registry → false unsanitized warnings | Ship with 60+ known sanitizers. Expose `/taint-source-sink-discovery` so users see what's classified. Accept this limitation honestly in docs. |

---

## Real Token Counts Per Entity (Fix in v1.7.3)

### The Problem

`smart-context-token-budget` estimates tokens with `100 + key_length / 10`. A 5-line function and a 200-line function both estimate at ~110 tokens. The token budget is fiction.

### The Fix

During pt01 ingestion, count real tokens for every entity's source code and store it in the `CodeGraph` relation.

**Crate**: `bpe-openai = "0.3"` (by GitHub, pure Rust, 3.5x faster than tiktoken-rs)

```rust
use bpe_openai::Tokenizer;
use std::sync::LazyLock;

static TOKENIZER: LazyLock<&'static Tokenizer> = LazyLock::new(|| bpe_openai::cl100k_base());

fn count_entity_tokens_fast(source_code: &str) -> usize {
    TOKENIZER.count(source_code)
}
```

**Why `bpe-openai`**:
- **Dedicated `count()` method** — counts without allocating a token list. Critical for 50K+ entities.
- **3.5x faster than tiktoken-rs** with linear worst-case scaling (tiktoken-rs is quadratic).
- **Lightweight** — ~5 dependencies vs tiktoken-rs's 8.79 MB embedded vocab data.
- **Maintained by GitHub** (the company) — 93 stars, 13 contributors.
- **`cl100k_base` as universal proxy** — within ~10-20% of Claude's actual counts. Good enough for budgeting.

### CozoDB Schema Change

Add `token_count` column to `CodeGraph` relation:

```
CodeGraph {
  ISGL1_key: String,
  ...existing columns...,
  token_count: Int,       ← NEW: real BPE token count of source code
  word_count: Int,        ← NEW: split_whitespace().count() as cheap fallback
}
```

### What This Unlocks

1. **Real smart-context budgets**: `smart-context-token-budget` reads `token_count` from DB instead of guessing. The budget means something.
2. **Lookahead calculations**: "Sum all entity token counts in this module" → instant answer. Plan context allocation before fetching code.
3. **Cost estimation**: "This blast radius of 47 entities would cost 12,400 tokens" — the LLM can decide whether to expand or constrain the query.
4. **File-level totals**: Sum entity token counts per file → "this file costs 3,200 tokens to include fully."
5. **Surgical extraction**: When adding source code to smart-context responses, pack entities by real token count into the budget (greedy knapsack with accurate weights).

### Files Changed

- `pt01/.../streamer.rs`: After tree-sitter extraction, call `TOKENIZER.count(&code)` and store in entity
- `parseltongue-core/src/entities.rs`: Add `token_count: usize` and `word_count: usize` to entity structs
- `parseltongue-core/src/storage/cozo_client.rs`: Add columns to `CodeGraph` CREATE RELATION and INSERT queries
- `pt08/.../smart_context_token_budget_handler.rs`: Replace `estimate_entity_tokens()` heuristic with DB column lookup
- `Cargo.toml` (pt01 or parseltongue-core): Add `bpe-openai = "0.3"`

---

## Serialization: MessagePack

Format: **MessagePack** via `rmp-serde = "1.3"`. File extension: `.ptgraph`.

| Property | Value |
|----------|-------|
| Serde compatible | `#[derive(Serialize, Deserialize)]` — zero boilerplate |
| Size (this codebase) | ~2.7 MB (vs ~47 MB RocksDB) |
| Schema evolution | Forward-compatible (new fields ignored by old readers) |
| Speed | ~100 MB/s deserialize |
| Maintained | Yes (unlike bincode — dev doxxed) |

---

## MCP SDK: rmcp

| Property | Value |
|----------|-------|
| Crate | `rmcp = { version = "0.15", features = ["server", "transport-io", "macros"] }` |
| Also needs | `schemars = "0.8"` (JSON Schema generation for tool parameters) |
| Organization | `modelcontextprotocol` (official — same org as the spec) |
| Transport | stdio (stdin/stdout JSON-RPC 2.0) |
| Macros | `#[tool]`, `#[tool_router]`, `#[tool_handler]` — eliminates boilerplate |
| Tokio-native | Yes — fits existing async architecture |
| Why not others | `rust-mcp-sdk` (community), `mcp-protocol-sdk` (less mature), `pmcp` (less adoption) |

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

## Port Management

### No `--port` flag. No conflicts. No confusion.

```
Server startup:
  1. Bind to port 0 → OS assigns available port
  2. Write port to /tmp/parseltongue-server-{mode}.port
  3. Print: "Server: http://localhost:{port}/{mode}/"

Server shutdown:
  1. parseltongue shutdown --{mode}
  2. Read /tmp/parseltongue-server-{mode}.port
  3. POST http://localhost:{port}/shutdown
  4. Delete port file

Edge cases:
  - Server crashed? Port file stale → shutdown detects connection refused → deletes file
  - Start same mode twice? Old port file overwritten → old server orphaned (OS reclaims port)
```

---

## Acceptance Criteria

**Ship when all 15 pass.**

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
TODO 25. README audit            → README.md             (verify)
TODO 26. Testing journal         → root .md file         (document)
```

**Done: ~365 lines. Remaining: ~1,190 lines. Total: ~1,555 lines.**

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

## Risks (4)

| Risk | So What | Mitigation |
|------|---------|-----------|
| Peak RAM during pt02 (~3.2 GB at 400K entities) | Can't generate snapshot on 8GB machine for Linux kernel | One-time CLI. Run on bigger machine, copy .ptgraph. |
| `/mem/` filesystem read fails (file moved/deleted) | Detail view returns error instead of code | Return clear error: "Source file not found at {path}. Re-run pt02 if codebase moved." |
| CozoDB batch insert limits | Large codebases exceed query size | Chunk 5000 entities/batch (already implemented). |
| `rmcp` SDK is v0.15 (pre-1.0) | API may change in future versions | Pin exact version. stdio transport is stable in spec. Wrapper is thin (~400 lines) — easy to update. |

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

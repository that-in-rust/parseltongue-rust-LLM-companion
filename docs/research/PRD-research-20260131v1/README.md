# Parseltongue Research Documentation - v1.6 Planning

**Analysis Date**: 2026-01-31
**Database**: rocksdb:parseltongue20260131154912/analysis.db
**Current Version**: 1.4.2
**Target Version**: 1.6 (Agent-Native)

## Overview

This research documentation was generated using Parseltongue's own HTTP query server to analyze the codebase and extract PRD ideas for v1.6. The analysis focused on transforming Parseltongue from a standalone code analyzer into a **first-class agent memory layer**.

## Key Insight

> **Parseltongue has the RIGHT DATA MODEL but the WRONG INTERFACE MODEL for agent harness integration.**

The fix is **protocol, not architecture**—we need to expose existing capabilities through agent-native interfaces (MCP, Unix pipes, streaming) rather than rebuild the core.

## Methodology

1. **Codebase Analysis via Parseltongue HTTP API**
   - Queried `/codebase-statistics-overview-summary` (230 entities, 3,867 edges)
   - Analyzed `/complexity-hotspots-ranking-view` (coupling patterns)
   - Examined `/semantic-cluster-grouping-list` (50 clusters)
   - Traced control flow via `/reverse-callers-query-graph`

2. **Comparative Framework Analysis**
   - Aligned current architecture with agent harness principles (Arize, Cursor, Claude Code)
   - Identified gaps: composability, MCP protocol, tiered responses
   - Extracted actionable PRD ideas from comparative note

3. **PRD Synthesis**
   - Prioritized features by impact (P0/P1/P2/P3)
   - Estimated effort (weeks)
   - Mapped to existing crate structure
   - Ensured 4-word naming convention compliance

## Research Documents

### 01_ARCHITECTURE_OVERVIEW.md
**What it covers**:
- 3-tier layered architecture (CLI → Tools → Core)
- 4 crate responsibilities (parseltongue, pt01, pt08, parseltongue-core)
- 14 HTTP endpoints across 5 categories
- Data storage (CozoDB + RocksDB)
- ISGL1 entity key format
- Performance characteristics from complexity analysis

**Key takeaways**:
- Strong foundation: hierarchical memory, dynamic indexing, context efficiency
- Current architecture supports agent-native features without rewrites
- Most coupled entities: `new`, `unwrap`, `to_string` (stdlib usage patterns)

### 02_V16_PRD_IDEAS_EXTRACTED.md
**What it covers**:
- 8 concrete PRD ideas for v1.6
- Priority ranking (P0: MCP, P1: Unix piping/streaming/multi-workspace)
- Acceptance criteria for each feature
- Implementation notes (crates affected, new modules)
- Before/after usage examples

**Key PRDs**:
1. **P0**: MCP Server Core Implementation (2-3 weeks) - Foundation
2. **P1**: Unix Piping Output Format (1 week) - Composability
3. **P1**: Incremental Query Streaming API (2 weeks) - Responsiveness
4. **P1**: Workspace Context Multiplexing (1.5 weeks) - Monorepo support

**Recommended v1.6 Scope**: P0 + P1 features = **6.5-7.5 weeks** (one quarter)

### 03_V17_V19_ARIZE_PATTERNS_EXTRACTED.md
**What it covers**:
- 7 NEW PRD ideas for v1.7-v1.9 (builds on v1.6 foundation)
- Arize agent harness patterns applied to Parseltongue's ISG
- Focus: Agent memory patterns (preview/pointer, self-correction, budget management)
- Strategic differentiators vs. generic file systems

**Key PRDs by Version**:

**v1.7 - Agent Memory Foundation** (4.5 weeks):
1. **P1**: Entity Preview Signature Pointers (1.5 weeks) - 90% token reduction
2. **P1**: Query Token Budget Estimator (1 week) - Self-correction enablement
3. **P1**: Stateful Query Pagination Bookmarks (2 weeks) - Beyond SSE streaming

**v1.8 - Advanced Memory Patterns** (5.5 weeks):
4. **P2**: Subgraph Export Local Execution (2 weeks) - SQL vs. file system tradeoff
5. **P2**: Session Hot Path Cache (1.5 weeks) - 10-50× speedup on repeated queries
6. **P2**: ISG Query Composition Pipeline (2 weeks) - Composable graph operations

**v1.9 - Intelligent Budget Management** (2 weeks):
7. **P1**: Budget Aware Query Planner (2 weeks) - 200K context feels infinite

**Strategic Insight**: Cursor/Claude apply memory patterns to **files** (unstructured), Parseltongue applies them to **ISG** (structured graph). Better previews, estimation, caching, composition, and planning.

## Codebase Statistics

From Parseltongue HTTP API analysis:

```
Code Entities: 230
Test Entities: 3
Dependency Edges: 3,867
Languages: Rust (primary)
Database: rocksdb:parseltongue20260131154912/analysis.db

Crate Structure:
├── parseltongue (CLI binary) - Dispatcher
├── pt01-folder-to-cozodb-streamer - Ingestion tool
├── pt08-http-code-query-server - HTTP REST API (14 endpoints)
└── parseltongue-core - Shared library (parsing, storage, entities)

Top Complexity Hotspots:
1. rust:fn:new - 279 inbound dependencies
2. rust:fn:unwrap - 203 inbound dependencies
3. rust:fn:to_string - 147 inbound dependencies
4. rust:fn:test_complete_cycle_graph_state - 37 outbound dependencies
```

## Control Flow Patterns

**Ingestion Flow** (pt01):
1. `run_folder_to_cozodb_streamer` (CLI entry)
2. `stream_directory` (directory traversal)
3. `QueryBasedExtractor::parse_source` (tree-sitter parsing)
4. `CozoDbStorage::insert_entity` (entity storage)
5. File watcher monitors changes → incremental updates

**Query Flow** (pt08):
1. HTTP request → Axum router
2. Handler in `http_endpoint_handler_modules/` (e.g., `handle_code_entities_fuzzy_search`)
3. CozoDB query via `CozoDbStorage` methods
4. JSON response serialization
5. Return to agent/client

## Data Flow Patterns

**Entity Lifecycle**:
```
Source Code
    ↓ (tree-sitter parsing)
CodeEntity {key, file_path, entity_type, entity_class, language}
    ↓ (CozoDB insertion)
code_entities relation
    ↓ (HTTP query)
JSON response
    ↓ (future: MCP tool)
Agent context
```

**Dependency Tracking**:
```
Function Call in Source
    ↓ (tree-sitter dependency extraction)
DependencyEdge {from_key, to_key, edge_type, source_location}
    ↓ (CozoDB insertion)
dependency_edges relation
    ↓ (graph query: reverse-callers, forward-callees, blast-radius)
Dependency graph
```

## Agent-Native Transformation Roadmap

### The Gap
| Current State | Agent-Native Goal |
|---------------|------------------|
| HTTP-only API | MCP protocol integration |
| Verbose JSON | Unix pipeable output (NDJSON, TSV) |
| Blocking queries | Streaming responses (SSE) |
| Single workspace | Multi-workspace multiplexing |
| Static schema | Dynamic tool discovery |
| No agent memory | Hierarchical memory (L1/L2/L3) |

### The Solution
**v1.6 Scope** (P0 + P1):
- ✅ MCP server (`pt09-mcp-server-stdio-provider`) - Agent discovery/invocation
- ✅ Unix piping (`--format ndjson|tsv|csv`) - Composability
- ✅ SSE streaming (`/stream-*` endpoints) - Incremental feedback
- ✅ Multi-workspace (`?workspace=db1,db2`) - Monorepo support

**Future Scope** (P2/P3):
- Dynamic tool introspection
- Natural language prompt templates
- Agent memory persistence
- Graph query DSL

## Implementation Principles

All v1.6 features follow Parseltongue's established patterns:

1. **4-word naming convention**: `verb_constraint_target_qualifier()`
2. **TDD workflow**: STUB → RED → GREEN → REFACTOR
3. **Layered architecture**: L1 (core) → L2 (standard) → L3 (external)
4. **Single feature per version**: One complete feature from end to end
5. **Zero TODOs/stubs in commits**: Production-ready code only

## How to Use This Research

### For Product Planning
1. Review `02_V16_PRD_IDEAS_EXTRACTED.md` for feature prioritization
2. Use P0+P1 scope (6.5-7.5 weeks) for sprint planning
3. Each PRD has acceptance criteria for definition of done

### For Engineering
1. Start with `01_ARCHITECTURE_OVERVIEW.md` to understand current state
2. Reference PRD implementation notes for crate/module changes
3. Use before/after examples to guide interface design

### For Agent Integration
1. v1.6 will expose MCP tools via `npx @parseltongue/mcp-server`
2. Unix piping enables shell script integration
3. Streaming enables responsive agent workflows

## Analysis Tools Used

This research was generated using:
- Parseltongue HTTP server (v1.4.2) running on port 7777
- Database: `rocksdb:parseltongue20260131154912/analysis.db`
- Endpoints: `/codebase-statistics-overview-summary`, `/code-entities-list-all`, `/complexity-hotspots-ranking-view`, `/semantic-cluster-grouping-list`, `/reverse-callers-query-graph`
- Analysis agent: Claude Code general-purpose task agent

## Meta-Observation

This documentation is itself a demonstration of Parseltongue's value:
- **99% token reduction**: Architecture overview fits in ~2K tokens vs. 500K raw source
- **31x faster than grep**: Sub-millisecond queries vs. multi-second file searches
- **Deterministic facts**: No hallucinated struct fields or function signatures

The v1.6 roadmap makes these capabilities **agent-accessible** through MCP, completing Parseltongue's transformation from IDE tool to agent memory layer.

---

## Quick Start

To replicate this analysis:

```bash
# 1. Start existing HTTP server (if not running)
parseltongue pt08-http-code-query-server \
  --db "rocksdb:parseltongue20260131154912/analysis.db"

# 2. Query codebase statistics
curl http://localhost:7777/codebase-statistics-overview-summary

# 3. Explore architecture
curl http://localhost:7777/code-entities-list-all | jq '.data.entities[] | select(.entity_type=="fn") | .entity_key'

# 4. Analyze coupling
curl http://localhost:7777/complexity-hotspots-ranking-view?top=20

# 5. Trace dependencies
curl "http://localhost:7777/reverse-callers-query-graph?entity=rust:fn:stream_directory"
```

After v1.6:
```bash
# MCP integration
npx -y @parseltongue/mcp-server

# Unix piping
parseltongue pt08 --db "..." --format ndjson | grep 'fn' | wc -l

# Streaming queries
curl -N http://localhost:7777/stream-code-entities-list-all
```

---

**Generated**: 2026-01-31 using Parseltongue v1.4.2 self-analysis
**Next Steps**: Review PRD ideas, prioritize P0+P1 features, begin implementation sprint

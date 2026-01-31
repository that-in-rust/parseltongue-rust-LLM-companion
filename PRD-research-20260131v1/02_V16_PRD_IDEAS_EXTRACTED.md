# Parseltongue v1.6: Agent-Native PRD Extraction

**Analysis Date**: 2026-01-31
**Source**: Comparative analysis of Parseltongue vs. Agent Harness Principles
**Current Version**: v1.4.2
**Target Version**: v1.6

## Executive Summary

Based on the comparative Minto Pyramid analysis of Parseltongue against emerging agent memory architecture patterns (Arize, Cursor, Claude Code), we've identified **8 concrete PRD ideas** that transform Parseltongue from a standalone analyzer into a first-class agent memory layer.

**Key Insight**: Parseltongue has the **right data model** (hierarchical ISG, deterministic indexing, context efficiency) but the **wrong interface model** (not composable, no MCP, missing tiered responses) for agent harness integration. The fix is **protocol, not architecture**.

---

## The Strategic Gap

### Current State (v1.4.2)
- ✅ Hierarchical memory (Module → Struct/Trait/Function)
- ✅ Dynamic indexing (SigHash-based, 3-12ms updates)
- ✅ Context efficiency (~1% token footprint)
- ✅ Determinism (BLAKE3 hashes)
- ❌ **Not composable** (monolithic commands, no piping)
- ❌ **No MCP** (agents can't discover/invoke tools)
- ❌ **No tiered responses** (preview/pointer/full hierarchy)
- ❌ **No self-correction feedback** (result size warnings)

### Agent-Native Requirements (from analysis)
1. **Unix-style composability**: Pipeable primitives
2. **MCP protocol**: Dynamic tool discovery
3. **Tiered responses**: Preview → Pointer → Full
4. **Self-correction signals**: Context budget warnings
5. **Polyglot support**: Beyond Rust

---

## PRD Idea #1: MCP Server Core Implementation

**Priority**: P0 (Foundational)
**Effort Estimate**: 2-3 weeks
**Impact**: Unlocks agent-native interaction across all MCP-compatible IDEs (Cursor, VS Code, Claude Code, Windsurf)

### Current Limitation
Parseltongue exposes functionality only via HTTP REST API (port 7777). Agents cannot discover or invoke Parseltongue capabilities without manual HTTP curl commands. This prevents integration with the MCP ecosystem (5,800+ servers, 300+ clients, 97M+ monthly SDK downloads).

### Proposed Solution
Create new crate `pt09-mcp-server-stdio-provider` that implements MCP STDIO transport, exposing Parseltongue's 14 existing HTTP endpoints as MCP tools. Reuse existing CozoDB queries and business logic from `pt08-http-code-query-server`.

### Acceptance Criteria
- [ ] New crate `pt09-mcp-server-stdio-provider` created (4-word naming)
- [ ] STDIO transport implemented using `@modelcontextprotocol/sdk-rust`
- [ ] All 14 HTTP endpoints exposed as MCP tools (1:1 mapping)
- [ ] NPX installation: `npx -y @parseltongue/mcp-server`
- [ ] JSON configuration works in Cursor, VS Code, Windsurf, Claude Code
- [ ] Tool count stays under 40 (Cursor compatibility)
- [ ] All tests passing (TDD: RED → GREEN → REFACTOR)

### Implementation Notes
**Crates affected**: New `pt09-mcp-server-stdio-provider`, shared logic from `parseltongue-core`

**New modules needed**:
- `mcp_tool_registry_builder.rs` - Maps HTTP endpoints to MCP tools
- `stdio_transport_handler_module.rs` - STDIO communication
- `tool_invocation_router_dispatcher.rs` - Routes MCP requests to existing handlers

**Database schema changes**: None (reads existing RocksDB)

### Example Usage
```bash
# Before (v1.4.2) - Manual HTTP calls
curl http://localhost:7777/code-entities-search-fuzzy?q=handle

# After (v1.6) - Agent-native MCP
# In .cursor/mcp.json:
{
  "mcpServers": {
    "parseltongue": {
      "command": "npx",
      "args": ["-y", "@parseltongue/mcp-server"],
      "env": {
        "PARSELTONGUE_DB": "rocksdb:parseltongue20260131/analysis.db"
      }
    }
  }
}

# Agent automatically discovers and invokes Parseltongue tools
```

---

## PRD Idea #2: Unix Piping Output Format

**Priority**: P1 (High-value composability)
**Effort Estimate**: 1 week
**Impact**: Enables Unix-style composability for agent workflows and command-line scripting

### Current Limitation
Parseltongue outputs verbose JSON with nested structures, making it difficult to pipe into other Unix tools (grep, jq, awk, sort). Agents struggle to extract specific fields from responses.

### Proposed Solution
Add `--format` flag to all CLI tools with options: `json` (default), `ndjson` (newline-delimited JSON), `tsv` (tab-separated), `csv`. Implement `format_output_for_unix_piping()` function following 4-word naming.

### Acceptance Criteria
- [ ] All 14 endpoints support `--format=ndjson|tsv|csv` via CLI
- [ ] NDJSON outputs one entity/edge per line for streaming
- [ ] TSV format: `entity_key\tentity_type\tfile_path\tlines`
- [ ] Stdout-only output mode (no headers/metadata) for piping
- [ ] MCP tools return structured JSON by default
- [ ] Documentation updated with Unix piping examples
- [ ] All tests passing

### Implementation Notes
**Crates affected**: `parseltongue-core` (formatting module), `pt08-http-code-query-server` (output handlers)

**New modules needed**:
- `parseltongue-core/src/output_format_unix_piping.rs` - Format conversion
- `parseltongue-core/src/ndjson_streaming_serializer_module.rs` - NDJSON streaming

### Example Usage
```bash
# Before (v1.4.2) - Verbose JSON
curl http://localhost:7777/code-entities-list-all | jq -r '.data.entities[] | select(.entity_type=="fn") | .entity_key'

# After (v1.6) - Unix-style piping
parseltongue pt08-http-code-query-server --db "..." --format ndjson \
  | grep '"entity_type":"fn"' \
  | cut -f1 \
  | sort | uniq -c

# Tab-separated for awk
parseltongue pt08 --db "..." --format tsv \
  | awk -F'\t' '$2 == "fn" {print $1}' \
  | wc -l
```

---

## PRD Idea #3: Incremental Query Streaming API

**Priority**: P1 (Agent responsiveness)
**Effort Estimate**: 2 weeks
**Impact**: Enables sub-second agent feedback for large codebases (vs. 5-10s blocking waits)

### Current Limitation
Current HTTP endpoints return complete results (all 230 entities, all 3,867 edges) in one blocking response. For large codebases (10K+ entities), agents wait 5-10 seconds. No incremental feedback, pagination, or streaming.

### Proposed Solution
Implement Server-Sent Events (SSE) streaming for analysis endpoints. Add `--stream` flag that returns results progressively (10 entities per chunk). Reuse existing CozoDB cursor/iterator patterns.

### Acceptance Criteria
- [ ] SSE endpoint `/stream-code-entities-list-all` created
- [ ] SSE endpoint `/stream-dependency-edges-list-all` created
- [ ] SSE endpoint `/stream-blast-radius-impact-analysis` created
- [ ] Chunks emitted every 100ms or 10 entities (whichever first)
- [ ] MCP tools support streaming via SSE transport
- [ ] Backward compatibility: Non-streaming endpoints unchanged
- [ ] Memory usage stays constant (no buffering full results)
- [ ] All tests passing

### Implementation Notes
**Crates affected**: `pt08-http-code-query-server` (SSE endpoints), `parseltongue-core` (streaming iterators)

**New modules needed**:
- `pt08-http-code-query-server/src/sse_streaming_response_handler.rs` - SSE implementation
- `parseltongue-core/src/paginated_query_iterator_module.rs` - CozoDB pagination

### Example Usage
```bash
# Before (v1.4.2) - Blocking
curl http://localhost:7777/code-entities-list-all
# Waits 5 seconds, then dumps 230 entities

# After (v1.6) - Streaming
curl -N http://localhost:7777/stream-code-entities-list-all
# Output:
# data: {"entity_key": "rust:fn:main:src/main.rs:1-5", ...}
#
# data: {"entity_key": "rust:fn:parse_args:src/main.rs:7-12", ...}
#
# (continues streaming every 100ms)

# Agent receives first results in <100ms
```

---

## PRD Idea #4: Workspace Context Multiplexing

**Priority**: P1 (Multi-project agent support)
**Effort Estimate**: 1.5 weeks
**Impact**: Enables agents to work across multiple codebases simultaneously (monorepo support)

### Current Limitation
Parseltongue only supports one database per HTTP server instance. Agents working on monorepos must start multiple HTTP servers (ports 7777, 7778, 7779...) manually. No unified query across projects.

### Proposed Solution
Add `--workspace` parameter to all endpoints that accepts multiple database paths. Implement `workspace_multiplexer_router_module.rs` that queries multiple databases in parallel and merges results tagged with `workspace_id`.

### Acceptance Criteria
- [ ] All 14 endpoints accept `?workspace=db1,db2,db3` query parameter
- [ ] Results tagged with `"workspace": "parseltongue20260131"` field
- [ ] Parallel query execution (3 workspaces queried concurrently)
- [ ] Cross-workspace blast radius analysis
- [ ] MCP tool `analyze_multi_workspace_dependencies` added
- [ ] Backward compatibility: Single database works as before
- [ ] All tests passing

### Implementation Notes
**Crates affected**: `pt08-http-code-query-server` (workspace routing), `parseltongue-core` (multi-DB support)

**New modules needed**:
- `pt08-http-code-query-server/src/workspace_multiplexer_router_module.rs` - Multi-DB query
- `parseltongue-core/src/multi_database_connection_pool.rs` - Connection pooling

### Example Usage
```bash
# Before (v1.4.2) - One workspace per server
parseltongue pt08 --db "rocksdb:workspace1/analysis.db" --port 7777 &
parseltongue pt08 --db "rocksdb:workspace2/analysis.db" --port 7778 &

# After (v1.6) - Multi-workspace
parseltongue pt08 --db "rocksdb:workspace1/analysis.db,rocksdb:workspace2/analysis.db"

curl "http://localhost:7777/code-entities-search-fuzzy?q=handle&workspace=workspace1,workspace2"
# Returns entities tagged by workspace
```

---

## PRD Idea #5: Dynamic Tool Discovery Endpoint

**Priority**: P2 (Agent adaptability)
**Effort Estimate**: 1 week
**Impact**: Enables agents to discover available tools dynamically (vs. static schema)

### Current Limitation
MCP tools have static schemas defined at startup. When Parseltongue adds new endpoints, agents must restart to discover changes. No runtime introspection.

### Proposed Solution
Add MCP resource `parseltongue://capabilities` that returns current tool list with schemas. Add HTTP endpoint `/mcp-tools-schema-introspection-list`. Implement tool registry builder pattern.

### Acceptance Criteria
- [ ] MCP resource `parseltongue://capabilities` returns tool schemas
- [ ] HTTP endpoint `/mcp-tools-schema-introspection-list` created
- [ ] Tool schemas generated from existing handlers (no duplication)
- [ ] Schema includes: name, description, input schema, output schema
- [ ] MCP `tools/list` request returns dynamic tool list
- [ ] Documentation updated
- [ ] All tests passing

### Implementation Notes
**New modules needed**:
- `pt09-mcp-server-stdio-provider/src/tool_schema_registry_builder.rs`
- `pt09-mcp-server-stdio-provider/src/mcp_resource_provider_capabilities.rs`

### Example Usage
```bash
# After (v1.6) - Dynamic tool discovery
curl http://localhost:7777/mcp-tools-schema-introspection-list
# Returns: Current tool schemas

# MCP resource (from agent)
GET parseltongue://capabilities
# Returns: Live tool schemas for agent adaptation
```

---

## PRD Idea #6: Semantic Query Prompt Templates

**Priority**: P2 (Agent UX improvement)
**Effort Estimate**: 1.5 weeks
**Impact**: Enables natural language queries via MCP prompts (vs. structured parameters)

### Current Limitation
Current endpoints require structured parameters (`?entity=rust:fn:main&hops=2`). Agents must construct exact entity keys. No support for natural language queries like "show me what calls the main function".

### Proposed Solution
Implement MCP prompts feature with pre-defined templates: `analyze-blast-radius`, `find-circular-imports`, `show-callers`, `show-callees`. Prompts convert natural language to structured queries.

### Acceptance Criteria
- [ ] MCP prompt `parseltongue://prompts/analyze-blast-radius` created
- [ ] MCP prompt `parseltongue://prompts/find-circular-dependencies` created
- [ ] MCP prompt `parseltongue://prompts/show-function-callers` created
- [ ] MCP prompt `parseltongue://prompts/show-function-callees` created
- [ ] Prompts include parameter placeholders
- [ ] MCP `prompts/list` returns available prompts
- [ ] Works in Cursor, VS Code, Claude Code
- [ ] All tests passing

### Implementation Notes
**New modules needed**:
- `pt09-mcp-server-stdio-provider/src/mcp_prompt_template_registry.rs`
- `pt09-mcp-server-stdio-provider/src/prompt_parameter_parser_module.rs`

### Example Usage
```bash
# In Cursor/VS Code
@parseltongue Use the analyze-blast-radius prompt for function "main" with 2 hops

# MCP prompt template converts to structured query
```

---

## PRD Idea #7: Hierarchical Memory Persistence Layer

**Priority**: P2 (Agent memory architecture)
**Effort Estimate**: 2 weeks
**Impact**: Enables agent conversation context persistence (L1: working set, L2: session, L3: long-term)

### Current Limitation
Parseltongue stores code analysis but doesn't track agent queries, frequently accessed entities, or query patterns. Agents must re-query same information across sessions. No query result caching.

### Proposed Solution
Implement 3-tier memory hierarchy:
- **L1** (working set): Last 10 accessed entities
- **L2** (session): Current conversation context
- **L3** (long-term): Query history

Store in CozoDB relations: `AgentWorkingSet`, `AgentSession`, `AgentQueryHistory`.

### Acceptance Criteria
- [ ] `/agent-working-set-entities-list` returns last 10 accessed entities
- [ ] `/agent-session-context-retrieve` returns current session entities
- [ ] `/agent-query-history-list` returns past queries with timestamps
- [ ] MCP resource `parseltongue://memory/working-set` exposes L1
- [ ] MCP resource `parseltongue://memory/session` exposes L2
- [ ] Query result caching (10-minute TTL)
- [ ] Session ID parameter for multi-conversation tracking
- [ ] All tests passing

### Implementation Notes
**New modules needed**:
- `parseltongue-core/src/agent_memory_persistence_layer.rs`
- `pt08-http-code-query-server/src/http_endpoint_handler_modules/agent_working_set_handler.rs`
- `pt08-http-code-query-server/src/http_endpoint_handler_modules/agent_session_context_handler.rs`
- `pt08-http-code-query-server/src/http_endpoint_handler_modules/agent_query_history_handler.rs`

**Database schema changes**: Add 3 new CozoDB relations

### Example Usage
```bash
# After (v1.6) - Hierarchical memory
curl http://localhost:7777/code-entities-search-fuzzy?q=main&session=abc123
# Stored in L1 working set + L2 session

curl http://localhost:7777/agent-working-set-entities-list?session=abc123
# Returns: Recently accessed entities

curl http://localhost:7777/agent-query-history-list?session=abc123
# Returns: ["search: main", "callers: main", "blast-radius: main"]
```

---

## PRD Idea #8: Graph Query DSL Endpoint

**Priority**: P3 (Power users / advanced agents)
**Effort Estimate**: 2 weeks
**Impact**: Enables complex graph queries via declarative DSL (vs. fixed endpoints)

### Current Limitation
Current endpoints expose 14 fixed queries. Agents cannot compose custom graph traversals like "find all functions that call X AND are called by Y" or "show dependency path from A to B with max 5 hops avoiding module C".

### Proposed Solution
Create `/graph-query-dsl-execute` endpoint that accepts simple query DSL. DSL supports: `FIND entities WHERE`, `TRAVERSE edges DEPTH N`, `FILTER BY module`, `EXCLUDE patterns`. Compile DSL to CozoDB Datalog queries.

### Acceptance Criteria
- [ ] Endpoint `/graph-query-dsl-execute` accepts POST with DSL query
- [ ] DSL syntax: `FIND fn WHERE name LIKE "handle%" TRAVERSE callers DEPTH 2`
- [ ] DSL supports: FIND, WHERE, TRAVERSE (callers/callees), DEPTH, FILTER, EXCLUDE
- [ ] Query validation with clear error messages
- [ ] Query result format matches existing endpoints
- [ ] MCP tool `execute_graph_query_dsl` exposes DSL
- [ ] Documentation with 10+ DSL query examples
- [ ] All tests passing

### Implementation Notes
**New modules needed**:
- `parseltongue-core/src/graph_query_dsl_parser.rs` - DSL parsing (pest/nom)
- `parseltongue-core/src/dsl_to_datalog_compiler.rs` - CozoDB query generation
- `pt08-http-code-query-server/src/http_endpoint_handler_modules/graph_query_dsl_handler.rs`

### Example Usage
```bash
# After (v1.6) - DSL for custom queries
curl -X POST http://localhost:7777/graph-query-dsl-execute \
  -H "Content-Type: application/json" \
  -d '{
    "query": "FIND fn WHERE file LIKE \"src/auth/%\" AND visibility = \"public\" TRAVERSE callees DEPTH 1 FILTER name LIKE \"db_%\""
  }'

# Returns: All public auth functions that call database functions
```

---

## Summary: v1.6 Feature Prioritization

### P0 (Must-Have) - 1 feature
**Foundation for agent-native access**
1. MCP Server Core Implementation (2-3 weeks)

### P1 (High-Value) - 3 features
**Composability, Responsiveness, Monorepo support**
2. Unix Piping Output Format (1 week)
3. Incremental Query Streaming API (2 weeks)
4. Workspace Context Multiplexing (1.5 weeks)

### P2 (Agent UX) - 3 features
**Adaptability, Natural language, Context retention**
5. Dynamic Tool Discovery Endpoint (1 week)
6. Semantic Query Prompt Templates (1.5 weeks)
7. Hierarchical Memory Persistence Layer (2 weeks)

### P3 (Power Users) - 1 feature
**Advanced queries**
8. Graph Query DSL Endpoint (2 weeks)

---

## Recommended v1.6 Scope

**Total Effort**: 13-15 weeks for all features
**Recommended Scope**: **P0 + P1 features** = 6.5-7.5 weeks (one quarter)

This delivers:
- ✅ MCP protocol integration (agent discovery/invocation)
- ✅ Unix-style composability (piping, streaming)
- ✅ Monorepo support (multi-workspace queries)
- ✅ Production-ready for Cursor, VS Code, Claude Code

All features follow:
- ✅ 4-word naming convention
- ✅ TDD workflow (STUB → RED → GREEN → REFACTOR)
- ✅ Layered architecture (L1/L2/L3)
- ✅ Single complete feature per version increment
- ✅ Leverage existing CozoDB/tree-sitter infrastructure
- ✅ No rewrites, only protocol additions

---

## The Meta-Insight

> "Parseltongue is a sophisticated L2 cache that needs to be wired into the memory hierarchy properly. The cache works—it just needs the bus."

The fix isn't to rebuild Parseltongue; it's to **expose its internals as composable primitives**. The ISG is already the right data structure. The AIM daemon already provides the right update semantics. What's missing is the **interface** that lets agents treat Parseltongue operations like Unix commands they can chain together.

**In agent terms**: Parseltongue has deterministic architectural facts that eliminate hallucination—it just needs MCP to make those facts accessible to the emerging agent ecosystem.

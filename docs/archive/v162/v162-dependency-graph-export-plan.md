# Plan: Dependency Graph Export Endpoints (v1.6.2)

## Context

Parseltongue has 22 endpoints that return edges, entity lists, or derived analysis -- but no single endpoint returns a dependency graph (nodes + edges together). An LLM agent or visualization tool currently needs multiple round-trips to assemble a graph. Two new endpoints fix this:

1. **Subgraph** -- ego-graph around a focus entity within N hops
2. **Full export** -- the entire dependency graph

Both return JSON: `{nodes: [...], edges: [...]}`.

## Endpoint 1: `GET /dependency-subgraph-entity-export`

**Query params**: `entity=X&hops=N` (hops default: 2, max: 5)

**Algorithm**: Bidirectional BFS from focus entity (adapted from blast_radius_impact_handler.rs which only does reverse BFS). Each hop queries both directions:

```datalog
// Batch forward query for frontier
frontier[k] <- [["key1"], ["key2"], ...]
?[to_key] := frontier[from_key], *DependencyEdges{from_key, to_key}

// Batch reverse query for frontier
frontier[k] <- [["key1"], ["key2"], ...]
?[from_key] := frontier[to_key], *DependencyEdges{from_key, to_key}
```

After BFS produces a node set, two more queries fetch node metadata and edges between discovered nodes:

```datalog
// Nodes
node_set[k] <- [["key1"], ["key2"], ...]
?[key, file_path, entity_type, entity_class, language] :=
    *CodeGraph{ISGL1_key: key, file_path, entity_type, entity_class, language},
    node_set[key]

// Edges (only edges where BOTH endpoints are in the set)
node_set[k] <- [["key1"], ["key2"], ...]
?[from_key, to_key, edge_type, source_location] :=
    *DependencyEdges{from_key, to_key, edge_type, source_location},
    node_set[from_key], node_set[to_key]
```

**Safety**: `max_nodes` param (default 500). BFS stops when exceeded, response includes `truncated: true`.

**Response**:
```json
{
  "success": true,
  "endpoint": "/dependency-subgraph-entity-export",
  "data": {
    "focus_entity": "rust:fn:main:src_main_rs:1-50",
    "hops_requested": 2,
    "node_count": 12,
    "edge_count": 18,
    "truncated": false,
    "nodes": [
      {"key": "...", "file_path": "...", "entity_type": "function", "entity_class": "CODE", "language": "rust"}
    ],
    "edges": [
      {"from_key": "...", "to_key": "...", "edge_type": "Calls", "source_location": "..."}
    ]
  },
  "tokens": 650
}
```

## Endpoint 2: `GET /dependency-graph-export-full`

**Query params**: none required. Optional `max_nodes=10000&max_edges=50000` safety limits.

**Queries**: Two simple CozoDB queries (reuse patterns from code_entities_list_all_handler and dependency_edges_list_handler):

```datalog
// All nodes
?[key, file_path, entity_type, entity_class, language] :=
    *CodeGraph{ISGL1_key: key, file_path, entity_type, entity_class, language}

// All edges
?[from_key, to_key, edge_type, source_location] :=
    *DependencyEdges{from_key, to_key, edge_type, source_location}
```

**Response**: Same `{nodes, edges}` structure as endpoint 1, without `focus_entity` or `hops_requested`.

## Files to Create/Modify

### NEW: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/dependency_subgraph_entity_handler.rs`

Structs (4-word names):
- `SubgraphExportQueryParamsStruct` -- entity, hops (default 2), max_nodes (default 500)
- `SubgraphExportNodeDataItem` -- key, file_path, entity_type, entity_class, language
- `SubgraphExportEdgeDataItem` -- from_key, to_key, edge_type, source_location
- `SubgraphExportDataPayloadStruct` -- focus_entity, hops_requested, node_count, edge_count, truncated, nodes, edges
- `SubgraphExportResponsePayloadStruct` -- success, endpoint, data, tokens

Handler: `handle_dependency_subgraph_entity_export`

Key functions:
- Bidirectional BFS using batched CozoDB inline relations (adapted from blast_radius_impact_handler.rs BFS)
- Node metadata fetch via inline relation join
- Edge fetch between node set via inline relation join

### NEW: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/dependency_graph_export_handler.rs`

Structs (4-word names):
- `FullGraphExportQueryParamsStruct` -- max_nodes (default 10000), max_edges (default 50000)
- `FullGraphExportNodeDataItem` -- key, file_path, entity_type, entity_class, language
- `FullGraphExportEdgeDataItem` -- from_key, to_key, edge_type, source_location
- `FullGraphExportDataPayloadStruct` -- node_count, edge_count, truncated, nodes, edges
- `FullGraphExportResponsePayloadStruct` -- success, endpoint, data, tokens

Handler: `handle_dependency_graph_export_full`

### MODIFY: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/mod.rs`

Add:
```rust
pub mod dependency_subgraph_entity_handler;
pub mod dependency_graph_export_handler;
```

### MODIFY: `crates/pt08-http-code-query-server/src/route_definition_builder_module.rs`

Add imports and routes:
```rust
dependency_subgraph_entity_handler,
dependency_graph_export_handler,
```
```rust
.route("/dependency-subgraph-entity-export",
    get(dependency_subgraph_entity_handler::handle_dependency_subgraph_entity_export))
.route("/dependency-graph-export-full",
    get(dependency_graph_export_handler::handle_dependency_graph_export_full))
```

### MODIFY: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/api_reference_documentation_handler.rs`

Add endpoint 23 + 24 documentation. Update total_endpoints to 24.

### MODIFY: `CLAUDE.md`

Update endpoint count 22 -> 24, add rows for new endpoints.

### MODIFY: `README.md`

Add both endpoints to Jobs To Be Done table and HTTP API Reference.

## Reuse from Existing Code

| What | Source file | What to adapt |
|------|------------|---------------|
| BFS traversal | `blast_radius_impact_handler.rs` | Change from reverse-only to bidirectional |
| Edge struct | `dependency_edges_list_handler.rs` | Mirror `EdgeDataPayloadItem` |
| Node struct | `code_entities_list_all_handler.rs` | Mirror `EntitySummaryListItem` |
| DB access pattern | All handlers | Clone Arc from RwLock, release lock |
| `extract_string_value` | All handlers | Copy per convention |
| CozoDB inline relations | `blast_radius_impact_handler.rs` | Batch frontier queries |

## Verification

1. `cargo build --release` -- builds clean
2. `cargo test -p pt08-http-code-query-server` -- existing tests pass
3. Manual test:
   ```bash
   ./target/release/parseltongue pt01-folder-to-cozodb-streamer .
   ./target/release/parseltongue pt08-http-code-query-server --db "rocksdb:parseltongueTIMESTAMP/analysis.db"

   # Subgraph: ego-graph around an entity
   curl "http://localhost:7777/dependency-subgraph-entity-export?entity=rust:fn:main&hops=2" | jq '.data | {nodes: .node_count, edges: .edge_count}'

   # Full export: entire graph
   curl http://localhost:7777/dependency-graph-export-full | jq '.data | {nodes: .node_count, edges: .edge_count}'
   ```
4. Verify: subgraph nodes are subset of full export nodes
5. Verify: subgraph edges only connect nodes in the subgraph (no dangling edges)
6. Verify: full export node_count matches `/codebase-statistics-overview-summary` entity count
7. Verify: full export edge_count matches `/codebase-statistics-overview-summary` edge count
8. Verify: API reference shows total_endpoints >= 24

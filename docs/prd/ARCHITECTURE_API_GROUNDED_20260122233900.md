# Parseltongue Diff Visualization: API-Grounded Architecture

> Architecture validated against live Parseltongue API queries

---

## 1. API Inventory (Actually Tested)

The following endpoints were tested against a live Parseltongue server running on the parseltongue-dependency-graph-generator codebase itself:

| Endpoint | Response | Use Case |
|----------|----------|----------|
| `/codebase-statistics-overview-summary` | 215 entities, 790 external refs, 2880 edges | Dashboard summary |
| `/code-entities-list-all` | Full entity list with keys, types, locations | Graph nodes |
| `/dependency-edges-list-all` | All edges with from_key, to_key, edge_type | Graph edges |
| `/blast-radius-impact-analysis?entity=X&hops=N` | Affected entities within N hops | Impact visualization |
| `/semantic-cluster-grouping-list` | 46 clusters by module/namespace | Spatial grouping |
| `/reverse-callers-query-graph?entity=X` | Who calls X | Incoming edges |
| `/forward-callees-query-graph?entity=X` | What does X call | Outgoing edges |

---

## 2. Actual Data Structures

### Entity Format (from `/code-entities-list-all`)

**Response Envelope** (all endpoints use this pattern):
```json
{
  "success": true,
  "endpoint": "/code-entities-list-all",
  "data": { ... },
  "tokens": 50
}
```

**Entity Structure** (inside `data.entities[]`):
```json
{
  "key": "rust:fn:handle_blast_radius_impact_analysis:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_blast_radius_impact_handler_rs:116-171",
  "file_path": "./crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/blast_radius_impact_handler.rs",
  "entity_type": "fn",
  "entity_class": "CODE",
  "language": "rust"
}
```

**Key Pattern**: `{lang}:{entity_type}:{name}:{path_hash}:{start_line}-{end_line}`

**Note**: Path hash uses `__` prefix and replaces `/` with `_`, `-` preserved.

### Edge Format (from `/dependency-edges-list-all`)

**Edge Structure** (inside `data.edges[]`):
```json
{
  "from_key": "rust:file:__crates_parseltongue-core_src_entities_rs:1-1",
  "to_key": "rust:module:AccessModifier:0-0",
  "edge_type": "Uses",
  "source_location": "./crates/parseltongue-core/src/entities.rs:400"
}
```

**Edge Types Observed**: Uses, Calls, Implements, Contains

**Pagination Support**: This endpoint supports `limit` and `offset` query params.
Response includes: `total_count`, `returned_count`, `limit`, `offset`.

### Cluster Format (from `/semantic-cluster-grouping-list`)

**Cluster Structure** (inside `data.clusters[]`):
```json
{
  "cluster_id": 1,
  "entity_count": 336,
  "entities": ["rust:fn:filter_map:unknown:0-0", "rust:method:insert_edge:...", ...],
  "internal_edges": 942,
  "external_edges": 538
}
```

**Note**: Clusters use numeric IDs, not semantic names. Derive display names from entity patterns within each cluster.

### External References ("Unknown" Entities)

Many edges point to entities with `unknown:0-0` suffix:
```
rust:fn:is_empty:unknown:0-0
rust:fn:iter:unknown:0-0
rust:fn:map:unknown:0-0
```

These represent standard library functions, external crate functions, or functions not found in indexed source. The 790 "external references" in statistics are these entities. They should be rendered as "external" nodes without source locations.

---

## 3. Scale Metrics (Real Data)

From the actual Parseltongue codebase analysis:

| Metric | Value | Implication |
|--------|-------|-------------|
| **Total Entities** | 215 | Manageable for 3D rendering |
| **External References** | 790 | Could optionally show/hide (unknown:0-0 entities) |
| **Total Edges** | 2,880 | ~13 edges per entity average |
| **Semantic Clusters** | 46 | Natural spatial grouping |
| **Largest Cluster** | 336 entities | Contains stdlib/external refs, internal has 942 edges |

**Note**: A larger codebase (3000+ entities) would benefit from LOD and GPU instancing.

---

## 4. Architecture: Leverage Existing APIs

The diff visualization can be built using **80% existing Parseltongue APIs**.

### What We Already Have

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    EXISTING PARSELTONGUE APIs                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  GRAPH DATA (reuse directly)                                             │
│  ─────────────────────────────                                           │
│  /code-entities-list-all       → All nodes for graph                    │
│  /dependency-edges-list-all    → All edges for graph                    │
│  /semantic-cluster-grouping-list → Cluster-based positioning            │
│                                                                          │
│  ANALYSIS (reuse directly)                                               │
│  ─────────────────────────                                               │
│  /blast-radius-impact-analysis → Impact of changed nodes                │
│  /reverse-callers-query-graph  → Incoming dependencies                  │
│  /forward-callees-query-graph  → Outgoing dependencies                  │
│                                                                          │
│  SEARCH (reuse directly)                                                 │
│  ──────────────────────                                                  │
│  /code-entities-search-fuzzy   → Find entities by name                  │
│  /code-entity-detail-view/{key} → Entity details on click               │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### What We Need to Add

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      NEW ENDPOINTS NEEDED                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. /api/workspaces                                                      │
│     → List/create/delete workspaces                                     │
│     → Each workspace = one monitored folder                             │
│                                                                          │
│  2. /api/workspaces/{id}/diff                                            │
│     → Compare base.db vs live.db                                        │
│     → Returns: added/removed/modified entities and edges                │
│     → Calls existing APIs on both databases                             │
│                                                                          │
│  3. /api/workspaces/{id}/git                                             │
│     → Git status (clean/modified/staged/committed)                      │
│     → Branch info, commits ahead of base                                │
│                                                                          │
│  4. /api/workspaces/{id}/live (WebSocket)                                │
│     → Push updates when files change                                    │
│     → Debounced 500ms                                                   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 5. Diff Computation Strategy

### Algorithm Using Existing APIs

```
DIFF COMPUTATION (base.db vs live.db)
─────────────────────────────────────

Step 1: Load Both Graphs
  base_entities = GET /code-entities-list-all (from base.db)
  live_entities = GET /code-entities-list-all (from live.db)
  base_edges = GET /dependency-edges-list-all (from base.db)
  live_edges = GET /dependency-edges-list-all (from live.db)

Step 2: Entity Diff (by key)
  base_keys = set(base_entities.data.entities.map(e => e.key))
  live_keys = set(live_entities.data.entities.map(e => e.key))

  added = live_keys - base_keys
  removed = base_keys - live_keys
  common = base_keys ∩ live_keys

  modified = common.filter(key => {
    base = find(base_entities, key)
    live = find(live_entities, key)
    return base.line_start != live.line_start ||
           base.line_end != live.line_end ||
           base.file_path != live.file_path
  })

Step 3: Edge Diff
  edge_key = (from_key, to_key, edge_type)
  added_edges = live_edges - base_edges
  removed_edges = base_edges - live_edges

Step 4: Blast Radius (for each changed entity)
  for entity in (added ∪ modified):
    affected = GET /blast-radius-impact-analysis?entity={key}&hops=1
    mark affected entities as "neighbor"

Step 5: Return Diff
  {
    entities: { added: [...], removed: [...], modified: [...] },
    edges: { added: [...], removed: [...] },
    affected: [...],  // 1-hop neighbors
    clusters: GET /semantic-cluster-grouping-list (for layout)
  }
```

---

## 6. Frontend Data Flow

### From API to Three.js

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     FRONTEND ARCHITECTURE                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. INITIAL LOAD                                                         │
│     ────────────────                                                     │
│     GET /api/workspaces/{id}/diff                                       │
│       ↓                                                                  │
│     Parse response → categorize nodes by status                         │
│       ↓                                                                  │
│     Build Three.js scene:                                               │
│       - Focal nodes (changed): bright, glowing, pulsing                 │
│       - Neighbor nodes (1-hop): medium brightness                       │
│       - Ambient nodes (rest): dim, fog-like                             │
│                                                                          │
│  2. LIVE UPDATES (WebSocket)                                             │
│     ─────────────────────────                                            │
│     WS /api/workspaces/{id}/live                                        │
│       ↓                                                                  │
│     On message: { type: "diff_updated", diff: {...} }                   │
│       ↓                                                                  │
│     Animate transitions:                                                 │
│       - New focal nodes: scale up 0.5x → 1.5x, fade in glow            │
│       - Removed nodes: fade out, shrink                                 │
│       - Reclassified neighbors: opacity 0.15 → 0.7                      │
│                                                                          │
│  3. INTERACTION                                                          │
│     ─────────────                                                        │
│     Click node → GET /code-entity-detail-view/{key}                     │
│     Hover node → Show tooltip (name, file, lines)                       │
│     Search → GET /code-entities-search-fuzzy?q=...                      │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Node Classification Logic

```typescript
// Matches actual API response structure
interface ApiEntity {
  key: string;           // NOT entity_key
  file_path: string;
  entity_type: string;   // NOT kind
  entity_class: string;  // "CODE" | "TEST"
  language: string;
}

interface ApiCluster {
  cluster_id: number;    // NOT cluster_name (it's an integer)
  entity_count: number;  // NOT member_count
  entities: string[];
  internal_edges: number;
  external_edges: number;
}

interface Node {
  key: string;
  status: 'added' | 'removed' | 'modified' | 'neighbor' | 'ambient';
  position: { x: number; y: number; z: number };
  clusterId: number;
  isExternal: boolean;  // true if key contains "unknown:0-0"
}

function classifyNodes(diff: Diff, allEntities: ApiEntity[]): Node[] {
  const focal = new Set([
    ...diff.entities.added,
    ...diff.entities.removed,
    ...diff.entities.modified
  ]);

  const neighbors = new Set(diff.affected);

  return allEntities.map(entity => ({
    key: entity.key,
    status: focal.has(entity.key)
      ? getChangeStatus(entity, diff)
      : neighbors.has(entity.key)
        ? 'neighbor'
        : 'ambient',
    position: computePosition(entity, clusters),
    clusterId: findClusterForEntity(entity.key, clusters),
    isExternal: entity.key.includes('unknown:0-0')
  }));
}
```

---

## 7. Cluster-Based Spatial Layout

Using clusters from `/semantic-cluster-grouping-list`:

```
LAYOUT ALGORITHM
────────────────

1. Position clusters in 3D space (spherical arrangement)
   - 46 clusters → arrange on sphere surface
   - Radius based on entity_count (not member_count)

2. Position nodes within cluster
   - Force-directed within cluster bounds
   - Edges pull connected nodes together

3. Inter-cluster edges
   - Draw as curves between cluster centers
   - Less prominent than intra-cluster edges

Result:
                    ┌───────┐
                   /  HTTP   \
                  │  handlers │
                   \         /
        ┌───────┐   └───┬───┘   ┌───────┐
       /  Core   \      │      /  Tests  \
      │  types   │←─────┼─────→│         │
       \         /      │       \         /
        └───┬───┘   ┌───┴───┐    └───────┘
            │      /  Diff   \
            └─────→│  engine │
                   \         /
                    └───────┘
```

---

## 8. Performance Considerations

Based on actual scale metrics (215 entities, 2880 edges):

| Technique | When to Apply | Implementation |
|-----------|---------------|----------------|
| **Direct rendering** | < 500 nodes | Simple Three.js meshes |
| **GPU Instancing** | > 500 nodes | InstancedMesh for nodes |
| **Level of Detail** | > 1000 nodes | Simplify distant nodes |
| **Frustum culling** | Always | Built into Three.js |
| **Edge bundling** | > 5000 edges | Group parallel edges |

For MVP with 215 entities: **Direct rendering is sufficient**.

---

## 9. API Reuse Summary

| Existing API | Reuse For |
|--------------|-----------|
| `/code-entities-list-all` | All graph nodes |
| `/dependency-edges-list-all` | All graph edges |
| `/semantic-cluster-grouping-list` | Spatial layout grouping |
| `/blast-radius-impact-analysis` | Identify 1-hop neighbors |
| `/code-entity-detail-view/{key}` | Node click details |
| `/code-entities-search-fuzzy` | Search functionality |
| `/codebase-statistics-overview-summary` | Dashboard stats |

| New Endpoint | Purpose |
|--------------|---------|
| `/api/workspaces` | Workspace CRUD |
| `/api/workspaces/{id}/diff` | Diff base vs live |
| `/api/workspaces/{id}/git` | Git status |
| `/api/workspaces/{id}/live` | WebSocket updates |

**Result**: 80% API reuse, 4 new endpoints needed.

---

## 10. File Structure for Implementation

```
crates/
├── pt09-unified-web-server/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── workspace/
│   │   │   ├── mod.rs
│   │   │   ├── storage.rs         # ~/.parseltongue/ management
│   │   │   ├── manager.rs         # Workspace CRUD
│   │   │   └── watcher.rs         # File system watcher
│   │   ├── git/
│   │   │   ├── mod.rs
│   │   │   └── status.rs          # Git state detection
│   │   ├── diff/
│   │   │   ├── mod.rs
│   │   │   ├── entity_diff.rs     # Entity comparison
│   │   │   ├── edge_diff.rs       # Edge comparison
│   │   │   └── blast_radius.rs    # 1-hop calculation
│   │   ├── api/
│   │   │   ├── mod.rs
│   │   │   ├── routes.rs          # HTTP endpoints
│   │   │   └── websocket.rs       # Live updates
│   │   └── proxy.rs               # Forward to existing pt08 APIs
│   └── static/
│       ├── index.html
│       ├── app.js                 # Three.js + 3d-force-graph
│       └── styles.css
```

---

## 11. Key Implementation Insight

The diff visualization doesn't need to duplicate Parseltongue's analysis logic. Instead:

1. **Run two pt08 servers** (or access two databases):
   - One for `base.db` (snapshot at base commit)
   - One for `live.db` (current working state)

2. **Query both using existing APIs**:
   - Get entities/edges from both
   - Diff in memory
   - Forward detailed queries to appropriate db

3. **Add thin layer for**:
   - Workspace management
   - File watching → trigger re-index
   - Git status polling
   - WebSocket for push updates

This approach maximizes code reuse and leverages the battle-tested Parseltongue query engine.

---

*Architecture validated and corrected against live API queries on 2026-01-22*
*Field corrections applied from rubber duck debugging session*

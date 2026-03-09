# Rubber Duck Debug Report: API Validation

> Validating research documents against live Parseltongue API (2026-01-22)

---

## 1. Executive Summary

| Document | Issues Found | Severity |
|----------|--------------|----------|
| UNIFIED_SERVER_DESIGN_RESEARCH | 2 | Medium |
| VISUALIZATION_RESEARCH | 0 | N/A |
| ARCHITECTURE_API_GROUNDED | 4 | High |

**Overall Assessment**: Architecture doc has several inaccuracies in data structure assumptions.

---

## 2. Validated API Responses

### 2.1 Statistics Endpoint ✓
```json
GET /codebase-statistics-overview-summary

Response:
{
  "success": true,
  "endpoint": "/codebase-statistics-overview-summary",
  "data": {
    "code_entities_total_count": 215,
    "test_entities_total_count": 0,
    "dependency_edges_total_count": 2880,
    "languages_detected_list": ["rust"],
    "database_file_path": "rocksdb:..."
  },
  "tokens": 50
}
```

**Finding**: Response wrapped in `{success, endpoint, data, tokens}` envelope. Statistics fields use `_total_count` suffix.

### 2.2 Entity List Structure ✓
```json
GET /code-entities-list-all

Entity structure:
{
  "key": "rust:enum:EntityType:__crates_parseltongue-core_src_query_extractor_rs:47-59",
  "file_path": "./crates/parseltongue-core/src/query_extractor.rs",
  "entity_type": "enum",
  "entity_class": "CODE",
  "language": "rust"
}
```

**Finding**:
- Field is `key` not `entity_key` (doc error)
- Field is `entity_type` not `kind` (doc error)
- Has `entity_class` field (CODE/TEST)
- Has `language` field

### 2.3 Edge List Structure ✓
```json
GET /dependency-edges-list-all

Edge structure:
{
  "from_key": "rust:file:...",
  "to_key": "rust:module:...",
  "edge_type": "Uses",
  "source_location": "./crates/.../file.rs:400"
}
```

**Finding**: Edge structure matches docs. Has `source_location`.

### 2.4 Cluster Structure ⚠️
```json
GET /semantic-cluster-grouping-list

Actual response data keys:
- cluster_id (not cluster_name)
- entity_count (not member_count)
- entities (array)
- internal_edges
- external_edges
```

**Finding**: Cluster structure differs significantly:
- Uses `cluster_id` (integer) not `cluster_name` (string)
- Uses `entity_count` not `member_count`
- Includes `internal_edges` and `external_edges` counts

### 2.5 Blast Radius ⚠️
```json
GET /blast-radius-impact-analysis?entity=...&hops=1

Response (when entity not found or has no dependents):
{
  "success": false,
  "error": "No affected entities found for: ..."
}
```

**Finding**: Returns error when entity has no reverse dependencies. The blast radius requires entities that are CALLED BY others, not entities that CALL others.

### 2.6 Forward Callees ✓
```json
GET /forward-callees-query-graph?entity=...

Response:
{
  "success": true,
  "data": {
    "total_count": 10,
    "callees": [...]
  }
}
```

**Finding**: Works correctly. Returns what the entity calls.

### 2.7 Reverse Callers ⚠️
```json
GET /reverse-callers-query-graph?entity=...

Response (when no callers):
{
  "success": false,
  "error": "No callers found for entity: ..."
}
```

**Finding**: Many entities (especially handler functions) have no reverse callers in the static analysis because they're called via routing framework, not direct function calls.

---

## 3. Documentation Errors Found

### 3.1 ARCHITECTURE_API_GROUNDED_20260122233900.md

| Section | Error | Actual |
|---------|-------|--------|
| Entity Format | `entity_key` field | Field is `key` |
| Entity Format | `kind` field | Field is `entity_type` |
| Cluster Format | `cluster_name` | Field is `cluster_id` (integer) |
| Cluster Format | `member_count` | Field is `entity_count` |
| Scale Metrics | 35 clusters | Actual: 46 clusters (28 via one query, 46 via another - inconsistency?) |

### 3.2 UNIFIED_SERVER_DESIGN_RESEARCH_20260122233900.md

| Section | Error | Actual |
|---------|-------|--------|
| API not documented | Pagination support | `/dependency-edges-list-all` supports `limit` and `offset` |
| Token budget | Not mentioned | All responses include `tokens` field |

---

## 4. API Capabilities Discovered

### 4.1 Response Envelope Pattern
ALL endpoints return:
```json
{
  "success": boolean,
  "endpoint": string,      // Echo of called endpoint
  "data": object,          // Actual payload (when success=true)
  "error": string,         // Error message (when success=false)
  "tokens": number         // Token estimation for LLM context
}
```

### 4.2 Pagination Support
`/dependency-edges-list-all` response includes:
```json
{
  "total_count": 2880,
  "returned_count": N,
  "limit": N,
  "offset": N,
  "edges": [...]
}
```

### 4.3 Entity Key Pattern
```
{lang}:{type}:{name}:{path_hash}:{start_line}-{end_line}
```
Example: `rust:fn:main:__crates_parseltongue_src_main_rs:15-92`

**Note**: Path hash uses double underscores `__` and replaces `/` with `_`.

### 4.4 "Unknown" Entities
Many edges point to entities with `unknown:0-0` suffix:
```
rust:fn:is_empty:unknown:0-0
rust:fn:iter:unknown:0-0
```

These represent:
- Standard library functions
- External crate functions
- Functions not found in indexed source

**Implication for visualization**: ~790 "external references" will appear as nodes without source locations.

---

## 5. Corrected Data Structures

### 5.1 Correct Entity Structure
```typescript
interface Entity {
  key: string;           // NOT entity_key
  file_path: string;
  entity_type: string;   // NOT kind
  entity_class: string;  // "CODE" | "TEST"
  language: string;
}
```

### 5.2 Correct Cluster Structure
```typescript
interface Cluster {
  cluster_id: number;    // NOT cluster_name (string)
  entity_count: number;  // NOT member_count
  entities: string[];
  internal_edges: number;
  external_edges: number;
}
```

### 5.3 Correct Response Envelope
```typescript
interface ApiResponse<T> {
  success: boolean;
  endpoint: string;
  data?: T;
  error?: string;
  tokens: number;
}
```

---

## 6. Implications for Diff Visualization

### 6.1 Key-Based Diff is Valid
Entity keys ARE stable across re-indexing of the same code. The format:
```
{lang}:{type}:{name}:{path_hash}:{lines}
```
...means the same function at the same location will have the same key.

**However**: If line numbers change (e.g., code added above), the key changes too!

### 6.2 Blast Radius Limitation
The `/blast-radius-impact-analysis` endpoint only works for entities that have CALLERS. Many "leaf" functions (handlers, main, test functions) won't have blast radius data.

**Workaround**: Use `/forward-callees-query-graph` to find what the changed entity calls, then traverse those.

### 6.3 Cluster IDs are Integers
Clusters use numeric IDs, not semantic names. The visualization will need to:
1. Use cluster_id for grouping
2. Derive display names from file paths or entity patterns within cluster

---

## 7. Recommended Fixes

### 7.1 Update ARCHITECTURE_API_GROUNDED
```diff
- "entity_key": "rust:fn:..."
+ "key": "rust:fn:..."

- "kind": "fn",
+ "entity_type": "fn",

- "cluster_name": "pt08-http-code-query-server",
- "member_count": 23
+ "cluster_id": 1,
+ "entity_count": 23
```

### 7.2 Update UNIFIED_SERVER_DESIGN_RESEARCH
- Add note about pagination support
- Add note about token field in responses
- Document the "unknown" entity pattern for external references

### 7.3 Visualization Considerations
- Handle entities with `unknown:0-0` as "external" nodes
- Don't rely on blast radius for leaf nodes
- Use forward-callees + reverse-callers combination for complete graph

---

## 8. Verified Working Queries

```bash
# Statistics
curl http://localhost:7777/codebase-statistics-overview-summary

# All entities (wrapped in data.entities)
curl http://localhost:7777/code-entities-list-all

# All edges (wrapped in data.edges)
curl http://localhost:7777/dependency-edges-list-all

# Clusters (data.clusters array)
curl http://localhost:7777/semantic-cluster-grouping-list

# Forward dependencies (what X calls)
curl "http://localhost:7777/forward-callees-query-graph?entity=..."

# Fuzzy search
curl "http://localhost:7777/code-entities-search-fuzzy?q=blast"

# API documentation
curl http://localhost:7777/api-reference-documentation-help
```

---

## 9. Conclusion

The research documents contain **workable** architecture but have **field name inaccuracies** that would cause runtime errors. Key corrections:

1. `entity_key` → `key`
2. `kind` → `entity_type`
3. `cluster_name` → `cluster_id`
4. `member_count` → `entity_count`
5. Expect `unknown:0-0` entities for external references
6. All responses wrapped in `{success, endpoint, data, tokens}`

The core approach (diff by key, use existing APIs, add 4 new endpoints) remains valid.

---

*Validated against Parseltongue API v1.0.2 running on port 7777*

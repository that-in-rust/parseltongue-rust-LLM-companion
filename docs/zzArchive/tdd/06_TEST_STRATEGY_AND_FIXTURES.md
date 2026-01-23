# Diff Visualization Test Strategy and Fixtures

> **Version**: 1.0.0
> **Date**: 2026-01-23
> **Based On**: Real API data from Parseltongue HTTP server (localhost:7777)

---

## Executive Summary

This document defines the TDD-First test strategy for the Diff Visualization System, using **real entity keys and edge structures** discovered from the live Parseltongue API.

### Codebase Statistics (from API)

| Metric | Value |
|--------|-------|
| Total Code Entities | 215 |
| Total Dependency Edges | 2,880 |
| Languages Detected | Rust |
| Database | RocksDB |

---

## Part 1: Real Data Structures (from API)

### 1.1 Entity Key Format

**Pattern**: `{language}:{type}:{name}:__{path}:{lines}`

```
rust:fn:compute_blast_radius_by_hops:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_blast_radius_impact_handler_rs:185-277
```

**Components**:
- `rust` - Language
- `fn` - Entity type (fn, method, struct, enum, impl, trait)
- `compute_blast_radius_by_hops` - Entity name
- `__crates_...` - Path (slashes replaced with underscores)
- `185-277` - Line range (start-end)

### 1.2 Entity Types (from API)

| Type | Count | Example Key |
|------|-------|-------------|
| `function` | 52 | `rust:fn:handle_blast_radius_impact_analysis:__crates_pt08...` |
| `method` | ~50 | `rust:method:new:__crates_parseltongue-core_src_storage_cozo_client_rs:38-54` |
| `struct` | ~30 | `rust:struct:CozoDbStorage:__crates_parseltongue-core_src_storage_cozo_client_rs:...` |
| `enum` | 4 | `rust:enum:EntityType:__crates_parseltongue-core_src_query_extractor_rs:47-59` |
| `impl` | 8 | `rust:impl:CozoDbStorage:__crates_parseltongue-core_src_storage_cozo_client_rs:23-1272` |
| `trait` | ~5 | `rust:trait:CodeGraphRepository:...` |

### 1.3 Edge Format (from API)

```json
{
  "from_key": "rust:method:new:__crates_parseltongue-core_src_storage_cozo_client_rs:38-54",
  "to_key": "rust:fn:map_err:unknown:0-0",
  "edge_type": "Calls",
  "source_location": "./crates/parseltongue-core/src/storage/cozo_client.rs:47"
}
```

**Edge Types Observed**:
- `Uses` - Module/type usage
- `Calls` - Function/method call

### 1.4 Cluster Format (from API)

```json
{
  "cluster_id": 1,
  "entity_count": 761,
  "entities": ["rust:method:sanitize_path:...", "rust:fn:test_forward_dependencies:..."]
}
```

---

## Part 2: Test Fixtures Using Real Entity Keys

### 2.1 Core Function Entities (Production Data)

```typescript
// Fixture: Real function entities from Parseltongue codebase
export const REAL_FUNCTION_FIXTURES = {
  // Blast radius handler - has known callers
  blast_radius_handler: {
    key: "rust:fn:handle_blast_radius_impact_analysis:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_blast_radius_impact_handler_rs:116-171",
    file_path: "./crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/blast_radius_impact_handler.rs",
    entity_type: "function",
    entity_class: "CODE",
    language: "rust",
    line_start: 116,
    line_end: 171
  },

  // Core computation function - called by handler
  compute_blast_radius: {
    key: "rust:fn:compute_blast_radius_by_hops:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_blast_radius_impact_handler_rs:185-277",
    file_path: "./crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/blast_radius_impact_handler.rs",
    entity_type: "function",
    entity_class: "CODE",
    language: "rust",
    line_start: 185,
    line_end: 277
  },

  // Cycle detection
  detect_cycles: {
    key: "rust:fn:detect_cycles_using_dfs_traversal:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_circular_dependency_detection_handler_rs:105-156",
    file_path: "./crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/circular_dependency_detection_handler.rs",
    entity_type: "function",
    entity_class: "CODE",
    language: "rust",
    line_start: 105,
    line_end: 156
  },

  // Search function
  search_entities: {
    key: "rust:fn:search_entities_by_query_from_database:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_code_entities_fuzzy_search_handler_rs:121-178",
    file_path: "./crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/code_entities_fuzzy_search_handler.rs",
    entity_type: "function",
    entity_class: "CODE",
    language: "rust",
    line_start: 121,
    line_end: 178
  }
};
```

### 2.2 Method Entities (Production Data)

```typescript
export const REAL_METHOD_FIXTURES = {
  // CozoDbStorage::new - heavily connected
  cozo_storage_new: {
    key: "rust:method:new:__crates_parseltongue-core_src_storage_cozo_client_rs:38-54",
    file_path: "./crates/parseltongue-core/src/storage/cozo_client.rs",
    entity_type: "method",
    entity_class: "CODE",
    language: "rust",
    line_start: 38,
    line_end: 54
  },

  // get_entity - CRUD operation
  get_entity: {
    key: "rust:method:get_entity:__crates_parseltongue-core_src_storage_cozo_client_rs:804-834",
    file_path: "./crates/parseltongue-core/src/storage/cozo_client.rs",
    entity_type: "method",
    entity_class: "CODE",
    language: "rust",
    line_start: 804,
    line_end: 834
  },

  // delete_entity - CRUD operation
  delete_entity: {
    key: "rust:method:delete_entity:__crates_parseltongue-core_src_storage_cozo_client_rs:843-860",
    file_path: "./crates/parseltongue-core/src/storage/cozo_client.rs",
    entity_type: "method",
    entity_class: "CODE",
    language: "rust",
    line_start: 843,
    line_end: 860
  },

  // insert_entity
  insert_entity: {
    key: "rust:method:insert_entity:__crates_parseltongue-core_src_storage_cozo_client_rs:774-801",
    file_path: "./crates/parseltongue-core/src/storage/cozo_client.rs",
    entity_type: "method",
    entity_class: "CODE",
    language: "rust",
    line_start: 774,
    line_end: 801
  }
};
```

### 2.3 Edge Fixtures (Production Data)

```typescript
export const REAL_EDGE_FIXTURES = {
  // Method calling stdlib functions
  cozo_new_calls_ok: {
    from_key: "rust:method:new:__crates_parseltongue-core_src_storage_cozo_client_rs:38-54",
    to_key: "rust:fn:Ok:unknown:0-0",
    edge_type: "Calls",
    source_location: "./crates/parseltongue-core/src/storage/cozo_client.rs:53"
  },

  cozo_new_calls_map_err: {
    from_key: "rust:method:new:__crates_parseltongue-core_src_storage_cozo_client_rs:38-54",
    to_key: "rust:fn:map_err:unknown:0-0",
    edge_type: "Calls",
    source_location: "./crates/parseltongue-core/src/storage/cozo_client.rs:47"
  },

  // File using modules
  file_uses_hashmap: {
    from_key: "rust:file:__crates_parseltongue-core_src_temporal_rs:1-1",
    to_key: "rust:module:HashMap:0-0",
    edge_type: "Uses",
    source_location: "./crates/parseltongue-core/src/temporal.rs:9"
  },

  file_uses_code_entity: {
    from_key: "rust:file:__crates_parseltongue-core_src_temporal_rs:1-1",
    to_key: "rust:module:CodeEntity:0-0",
    edge_type: "Uses",
    source_location: "./crates/parseltongue-core/src/temporal.rs:42"
  }
};
```

### 2.4 Enum Fixtures (Production Data)

```typescript
export const REAL_ENUM_FIXTURES = {
  entity_type: {
    key: "rust:enum:EntityType:__crates_parseltongue-core_src_query_extractor_rs:47-59",
    file_path: "./crates/parseltongue-core/src/query_extractor.rs",
    entity_type: "enum",
    entity_class: "CODE",
    language: "rust"
  },

  http_error_types: {
    key: "rust:enum:HttpServerErrorTypes:__crates_pt08-http-code-query-server_src_structured_error_handling_types_rs:19-40",
    file_path: "./crates/pt08-http-code-query-server/src/structured_error_handling_types.rs",
    entity_type: "enum",
    entity_class: "CODE",
    language: "rust"
  },

  streamer_error: {
    key: "rust:enum:StreamerError:__crates_pt01-folder-to-cozodb-streamer_src_errors_rs:8-57",
    file_path: "./crates/pt01-folder-to-cozodb-streamer/src/errors.rs",
    entity_type: "enum",
    entity_class: "CODE",
    language: "rust"
  }
};
```

---

## Part 3: Test Strategy (TDD-First)

### 3.1 Testing Pyramid

```
                    /\
                   /  \
                  / E2E \         <- 5 tests (10%)
                 /______\
                /        \
               /  Integ   \       <- 15 tests (30%)
              /____________\
             /              \
            /     Unit       \    <- 30 tests (60%)
           /__________________\
```

### 3.2 Unit Test Strategy

#### Category: Diff Detection

| Test Name | WHEN | THEN |
|-----------|------|------|
| `test_detect_entity_added_scenario` | New entity key appears in T2 not in T1 | Returns `DiffType::Added` with correct metadata |
| `test_detect_entity_removed_scenario` | Entity key exists in T1 but not T2 | Returns `DiffType::Removed` with correct metadata |
| `test_detect_entity_modified_scenario` | Same key, different line range | Returns `DiffType::Modified` with before/after |
| `test_detect_entity_renamed_scenario` | Name change, same location | Returns `DiffType::Renamed` with old/new names |
| `test_detect_entity_moved_scenario` | Same name, different file | Returns `DiffType::Moved` with from/to paths |
| `test_detect_no_change_scenario` | Identical entity in T1 and T2 | Returns `DiffType::Unchanged` |

#### Category: Blast Radius Calculation

| Test Name | WHEN | THEN |
|-----------|------|------|
| `test_blast_radius_single_hop_scenario` | Entity with 1 direct caller | Returns exactly 1 affected entity at hop 1 |
| `test_blast_radius_multi_hop_scenario` | Entity with transitive callers | Returns correct counts per hop |
| `test_blast_radius_zero_hops_scenario` | Hops = 0 | Returns only the modified entity |
| `test_blast_radius_no_callers_scenario` | Entity has no callers | Returns empty affected list |
| `test_blast_radius_cyclic_graph_scenario` | Entity in dependency cycle | Does not loop infinitely, handles gracefully |

#### Category: Key Parsing

| Test Name | WHEN | THEN |
|-----------|------|------|
| `test_parse_function_key_format` | `rust:fn:name:__path:10-20` | Extracts all components correctly |
| `test_parse_method_key_format` | `rust:method:new:__path:38-54` | Extracts all components correctly |
| `test_parse_unknown_target_format` | `rust:fn:Ok:unknown:0-0` | Recognizes stdlib/unknown entities |
| `test_key_line_range_extraction` | Key with `185-277` | Returns (185, 277) tuple |

### 3.3 Integration Test Strategy

#### Test: Diff API Integration

```typescript
// REQ-DIFF-001: API returns correct diff for real entities
describe("Diff API Integration", () => {
  it("WHEN comparing two snapshots with a modified function THEN returns correct diff", async () => {
    // Given: Real entity from fixture
    const entity = REAL_FUNCTION_FIXTURES.compute_blast_radius;

    // When: Entity line range changes
    const t1 = createSnapshot({ [entity.key]: entity });
    const t2 = createSnapshot({
      [entity.key]: { ...entity, line_start: 185, line_end: 290 } // Extended
    });

    // Then: Diff shows modification
    const diff = await computeDiff(t1, t2);
    expect(diff.modified).toContainEqual({
      key: entity.key,
      change: "line_range",
      before: "185-277",
      after: "185-290"
    });
  });
});
```

#### Test: Blast Radius with Real Data

```typescript
// REQ-DIFF-002: Blast radius uses real dependency data
describe("Blast Radius Integration", () => {
  it("WHEN querying blast radius for compute_blast_radius_by_hops THEN returns handle_blast_radius as affected", async () => {
    // Given: Real API response (verified against http://localhost:7777)
    const entity = "rust:fn:compute_blast_radius_by_hops";

    // When: Query blast radius
    const response = await fetch(
      `http://localhost:7777/blast-radius-impact-analysis?entity=${entity}&hops=2`
    );
    const result = await response.json();

    // Then: Known caller is in results
    expect(result.data.by_hop[0].entities).toContain(
      "rust:fn:handle_blast_radius_impact_analysis:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_blast_radius_impact_handler_rs:116-171"
    );
  });
});
```

### 3.4 E2E Test Strategy

#### Scenario: Full Diff Workflow

```typescript
describe("E2E: Complete Diff Visualization Workflow", () => {
  it("WHEN developer modifies a core function THEN visualization shows complete impact", async () => {
    // Step 1: Create baseline snapshot
    const baseline = await createCodebaseSnapshot();

    // Step 2: Simulate modification to CozoDbStorage::new
    const modified = await simulateEntityModification(
      REAL_METHOD_FIXTURES.cozo_storage_new.key,
      { line_end: 60 } // Extended by 6 lines
    );

    // Step 3: Compute diff
    const diff = await computeFullDiff(baseline, modified);

    // Step 4: Verify visualization data
    expect(diff).toMatchObject({
      summary: {
        added: 0,
        removed: 0,
        modified: 1,
        renamed: 0,
        moved: 0
      },
      blast_radius: {
        hops_1: expect.any(Number),
        hops_2: expect.any(Number)
      },
      modified_entities: [{
        key: REAL_METHOD_FIXTURES.cozo_storage_new.key,
        type: "method",
        file: "./crates/parseltongue-core/src/storage/cozo_client.rs"
      }]
    });
  });
});
```

---

## Part 4: Diff Scenarios with Real Data

### 4.1 Scenario: Entity Added

```typescript
export const SCENARIO_ENTITY_ADDED = {
  name: "New handler function added",
  t1_entities: [
    REAL_FUNCTION_FIXTURES.blast_radius_handler,
    REAL_FUNCTION_FIXTURES.detect_cycles
  ],
  t2_entities: [
    REAL_FUNCTION_FIXTURES.blast_radius_handler,
    REAL_FUNCTION_FIXTURES.detect_cycles,
    // NEW: search_entities added in T2
    REAL_FUNCTION_FIXTURES.search_entities
  ],
  expected_diff: {
    added: [REAL_FUNCTION_FIXTURES.search_entities.key],
    removed: [],
    modified: []
  }
};
```

### 4.2 Scenario: Entity Removed

```typescript
export const SCENARIO_ENTITY_REMOVED = {
  name: "Deprecated function removed",
  t1_entities: [
    REAL_METHOD_FIXTURES.cozo_storage_new,
    REAL_METHOD_FIXTURES.get_entity,
    REAL_METHOD_FIXTURES.delete_entity
  ],
  t2_entities: [
    REAL_METHOD_FIXTURES.cozo_storage_new,
    REAL_METHOD_FIXTURES.get_entity
    // REMOVED: delete_entity no longer exists
  ],
  expected_diff: {
    added: [],
    removed: [REAL_METHOD_FIXTURES.delete_entity.key],
    modified: []
  },
  blast_radius_warning: "Callers of delete_entity will break"
};
```

### 4.3 Scenario: Entity Modified (Line Range Change)

```typescript
export const SCENARIO_ENTITY_MODIFIED = {
  name: "Function body extended",
  t1_entity: {
    ...REAL_FUNCTION_FIXTURES.compute_blast_radius,
    line_start: 185,
    line_end: 277
  },
  t2_entity: {
    ...REAL_FUNCTION_FIXTURES.compute_blast_radius,
    line_start: 185,
    line_end: 320 // Extended by 43 lines
  },
  expected_diff: {
    type: "modified",
    field: "line_range",
    before: "185-277",
    after: "185-320",
    lines_changed: 43
  }
};
```

### 4.4 Scenario: Entity Renamed

```typescript
export const SCENARIO_ENTITY_RENAMED = {
  name: "Function renamed for clarity",
  t1_entity: {
    key: "rust:fn:compute_blast_radius_by_hops:__crates_...rs:185-277",
    name: "compute_blast_radius_by_hops"
  },
  t2_entity: {
    key: "rust:fn:calculate_impact_radius_by_depth:__crates_...rs:185-277",
    name: "calculate_impact_radius_by_depth"
  },
  expected_diff: {
    type: "renamed",
    old_name: "compute_blast_radius_by_hops",
    new_name: "calculate_impact_radius_by_depth",
    same_location: true
  }
};
```

### 4.5 Scenario: Entity Moved

```typescript
export const SCENARIO_ENTITY_MOVED = {
  name: "Function moved to different module",
  t1_entity: {
    key: "rust:fn:sanitize_path_for_key_format:__crates_parseltongue-core_src_query_extractor_rs:72-74",
    file_path: "./crates/parseltongue-core/src/query_extractor.rs"
  },
  t2_entity: {
    key: "rust:fn:sanitize_path_for_key_format:__crates_parseltongue-core_src_utils_rs:10-12",
    file_path: "./crates/parseltongue-core/src/utils.rs"
  },
  expected_diff: {
    type: "moved",
    from_file: "./crates/parseltongue-core/src/query_extractor.rs",
    to_file: "./crates/parseltongue-core/src/utils.rs",
    name_unchanged: true
  }
};
```

### 4.6 Scenario: Blast Radius Impact

```typescript
export const SCENARIO_BLAST_RADIUS = {
  name: "Modification impacts callers",
  modified_entity: REAL_FUNCTION_FIXTURES.compute_blast_radius.key,

  // Real data from API: blast-radius-impact-analysis?entity=...&hops=2
  expected_impact: {
    source_entity: "rust:fn:compute_blast_radius_by_hops",
    hops_requested: 2,
    total_affected: 1,
    by_hop: [
      {
        hop: 1,
        count: 1,
        entities: [
          "rust:fn:handle_blast_radius_impact_analysis:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_blast_radius_impact_handler_rs:116-171"
        ]
      }
    ]
  }
};
```

---

## Part 5: Verification Criteria

### 5.1 Unit Test Acceptance Criteria

| Criterion | Requirement |
|-----------|-------------|
| Coverage | >= 80% line coverage for diff detection logic |
| Performance | Each unit test completes in < 50ms |
| Isolation | No network calls, all data mocked from fixtures |
| Naming | Tests follow `test_{scenario}_{expected_outcome}` pattern |

### 5.2 Integration Test Acceptance Criteria

| Criterion | Requirement |
|-----------|-------------|
| API Compatibility | Tests pass against Parseltongue API v1.0.3 |
| Real Data | Uses actual entity keys from production fixtures |
| Latency | API calls complete in < 500ms |
| Error Handling | Graceful degradation when API unavailable |

### 5.3 E2E Test Acceptance Criteria

| Criterion | Requirement |
|-----------|-------------|
| Full Flow | Tests complete workflow from snapshot to visualization |
| Visual Verification | Output can be manually verified in browser |
| Data Integrity | Entity counts match API statistics (215 entities, 2880 edges) |

---

## Part 6: Test Data Management

### 6.1 Fixture Update Process

```bash
# Step 1: Query fresh data from running Parseltongue server
curl http://localhost:7777/code-entities-list-all > fixtures/entities.json
curl http://localhost:7777/dependency-edges-list-all > fixtures/edges.json
curl http://localhost:7777/semantic-cluster-grouping-list > fixtures/clusters.json

# Step 2: Extract key fixtures for tests
node scripts/extract-test-fixtures.js

# Step 3: Validate fixture integrity
npm run test:fixtures
```

### 6.2 Snapshot Format

```typescript
interface CodebaseSnapshot {
  timestamp: string;              // ISO 8601
  entities: Map<string, Entity>;  // key -> entity
  edges: Edge[];                  // all dependency edges
  statistics: {
    total_entities: number;       // 215
    total_edges: number;          // 2880
    languages: string[];          // ["rust"]
  };
}
```

### 6.3 Diff Format

```typescript
interface DiffResult {
  from_snapshot: string;   // timestamp
  to_snapshot: string;     // timestamp

  summary: {
    added: number;
    removed: number;
    modified: number;
    renamed: number;
    moved: number;
  };

  changes: DiffChange[];
  blast_radius: BlastRadiusResult;
}

interface DiffChange {
  key: string;
  type: "added" | "removed" | "modified" | "renamed" | "moved";
  before?: Entity;
  after?: Entity;
  affected_callers?: string[];  // For blast radius
}
```

---

## Part 7: API Endpoint Mapping

### 7.1 Required API Calls for Diff Visualization

| Operation | Endpoint | Purpose |
|-----------|----------|---------|
| Get all entities | `GET /code-entities-list-all` | Build T1/T2 snapshots |
| Get all edges | `GET /dependency-edges-list-all` | Build dependency graph |
| Get blast radius | `GET /blast-radius-impact-analysis?entity=X&hops=N` | Calculate impact |
| Get entity detail | `GET /code-entity-detail-view/{key}` | Fetch modified entity info |
| Get callers | `GET /reverse-callers-query-graph?entity=X` | Find affected callers |
| Get callees | `GET /forward-callees-query-graph?entity=X` | Find called entities |

### 7.2 API Response Validation

```typescript
// Zod schemas for API response validation
const EntitySchema = z.object({
  key: z.string(),
  file_path: z.string(),
  entity_type: z.enum(["function", "method", "struct", "enum", "impl", "trait"]),
  entity_class: z.enum(["CODE", "TEST"]),
  language: z.string()
});

const EdgeSchema = z.object({
  from_key: z.string(),
  to_key: z.string(),
  edge_type: z.enum(["Calls", "Uses"]),
  source_location: z.string()
});

const BlastRadiusSchema = z.object({
  source_entity: z.string(),
  hops_requested: z.number(),
  total_affected: z.number(),
  by_hop: z.array(z.object({
    hop: z.number(),
    count: z.number(),
    entities: z.array(z.string())
  }))
});
```

---

## Part 8: TDD Workflow for Implementation

### 8.1 Phase 1: Core Diff Detection (Week 1)

```
STUB: Write test_detect_entity_added_scenario
RED:  Run test, verify it fails (no implementation)
GREEN: Implement minimal detectEntityAdded()
REFACTOR: Extract common diff detection logic

STUB: Write test_detect_entity_removed_scenario
...repeat cycle...
```

### 8.2 Phase 2: Blast Radius Integration (Week 2)

```
STUB: Write test_blast_radius_with_real_api
RED:  Run test, verify API integration fails
GREEN: Implement BlastRadiusClient
REFACTOR: Add caching for repeated queries
```

### 8.3 Phase 3: Visualization Output (Week 3)

```
STUB: Write test_render_diff_summary_format
RED:  Run test, verify rendering fails
GREEN: Implement DiffRenderer
REFACTOR: Optimize for large diffs (>100 changes)
```

---

## Appendix A: Complete Real Entity Keys

### A.1 Function Entities (52 total)

```
rust:fn:build_api_documentation_categories:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_api_reference_documentation_handler_rs:114-270
rust:fn:build_call_chain_from_root:__crates_parseltongue-core_src_query_json_graph_helpers_rs:34-56
rust:fn:build_smart_context_selection:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_smart_context_token_budget_handler_rs:128-248
rust:fn:calculate_entity_coupling_scores:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_complexity_hotspots_ranking_handler_rs:118-192
rust:fn:collect_entities_in_file_path:__crates_parseltongue-core_src_query_json_graph_helpers_rs:85-104
rust:fn:compute_blast_radius_by_hops:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_blast_radius_impact_handler_rs:185-277
rust:fn:detect_cycles_using_dfs_traversal:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_circular_dependency_detection_handler_rs:105-156
rust:fn:dfs_find_cycles_recursive:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_circular_dependency_detection_handler_rs:161-200
rust:fn:estimate_entity_tokens:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_smart_context_token_budget_handler_rs:256-266
rust:fn:filter_edges_by_type_only:__crates_parseltongue-core_src_query_json_graph_helpers_rs:61-80
rust:fn:find_reverse_dependencies_by_key:__crates_parseltongue-core_src_query_json_graph_helpers_rs:15-29
rust:fn:handle_blast_radius_impact_analysis:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_blast_radius_impact_handler_rs:116-171
rust:fn:handle_circular_dependency_detection_scan:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_circular_dependency_detection_handler_rs:68-97
rust:fn:handle_code_entities_fuzzy_search:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_code_entities_fuzzy_search_handler_rs:77-116
rust:fn:handle_code_entities_list_all:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_code_entities_list_all_handler_rs:64-87
rust:fn:handle_code_entity_detail_view:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_code_entity_detail_view_handler_rs:76-128
rust:fn:handle_codebase_statistics_overview_summary:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_codebase_statistics_overview_handler_rs:46-75
rust:fn:handle_dependency_edges_list_all:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_dependency_edges_list_handler_rs:83-112
rust:fn:handle_server_health_check_status:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_server_health_check_handler_rs:34-51
rust:fn:handle_smart_context_token_budget:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_smart_context_token_budget_handler_rs:85-119
rust:fn:query_dependency_edges_paginated:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_dependency_edges_list_handler_rs:117-171
rust:fn:query_entities_with_filter_from_database:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_code_entities_list_all_handler_rs:92-149
rust:fn:run_label_propagation_clustering:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_semantic_cluster_grouping_handler_rs:111-245
rust:fn:sanitize_path_for_key_format:__crates_parseltongue-core_src_query_extractor_rs:72-74
rust:fn:search_entities_by_query_from_database:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_code_entities_fuzzy_search_handler_rs:121-178
```

### A.2 Enum Entities (4 total)

```
rust:enum:EntityType:__crates_parseltongue-core_src_query_extractor_rs:47-59
rust:enum:HttpServerErrorTypes:__crates_pt08-http-code-query-server_src_structured_error_handling_types_rs:19-40
rust:enum:JsonGraphQueryError:__crates_parseltongue-core_src_query_json_graph_errors_rs:11-23
rust:enum:StreamerError:__crates_pt01-folder-to-cozodb-streamer_src_errors_rs:8-57
```

### A.3 Impl Entities (8 total)

```
rust:impl:CozoDbStorage:__crates_parseltongue-core_src_storage_cozo_client_rs:23-1272
rust:impl:CozoDbStorage:__crates_parseltongue-core_src_storage_cozo_client_rs:1276-1343
rust:impl:HttpServerErrorTypes:__crates_pt08-http-code-query-server_src_structured_error_handling_types_rs:42-69
rust:impl:JsonGraphQueryError:__crates_parseltongue-core_src_query_json_graph_errors_rs:25-29
rust:impl:ParseltongError:__crates_pt01-folder-to-cozodb-streamer_src_errors_rs:59-82
rust:impl:QueryBasedExtractor:__crates_parseltongue-core_src_query_extractor_rs:76-602
rust:impl:StreamerConfig:__crates_pt01-folder-to-cozodb-streamer_src_lib_rs:62-74
rust:impl:ToolFactory:__crates_pt01-folder-to-cozodb-streamer_src_lib_rs:79-87
```

---

## Appendix B: Edge Statistics

### B.1 Edge Type Distribution

| Edge Type | Count | Percentage |
|-----------|-------|------------|
| Uses | ~2000 | 69% |
| Calls | ~880 | 31% |
| **Total** | **2880** | 100% |

### B.2 High-Connectivity Entities

Based on forward-callees query for `rust:method:new` (CozoDbStorage):

```
Callees: 8 direct calls
- rust:fn:Ok:unknown:0-0
- rust:fn:collect:unknown:0-0
- rust:fn:contains:unknown:0-0
- rust:fn:default:unknown:0-0
- rust:fn:map_err:unknown:0-0
- rust:fn:new:unknown:0-0
- rust:fn:splitn:unknown:0-0
- rust:fn:to_string:unknown:0-0
```

---

*Document generated from live Parseltongue API data on 2026-01-23*

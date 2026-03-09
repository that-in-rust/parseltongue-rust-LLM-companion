# Executable Specifications: Diff Command Execution Module

> "DIFF IS THE PRODUCT" - Visualizing what changed in dependency graphs when AI agents edit code.

This document contains executable specifications for the `parseltongue diff` command in WHEN...THEN...SHALL contract format. Each specification is designed to be directly convertible to test cases.

---

## Table of Contents

1. [REQ-DIFF-001: execute_diff_analysis_command](#req-diff-001-execute_diff_analysis_command)
2. [REQ-DIFF-002: load_database_entity_snapshot](#req-diff-002-load_database_entity_snapshot)
3. [REQ-DIFF-003: load_database_edges_snapshot](#req-diff-003-load_database_edges_snapshot)
4. [REQ-DIFF-004: compute_combined_blast_radius](#req-diff-004-compute_combined_blast_radius)
5. [REQ-DIFF-005: format_diff_output_json](#req-diff-005-format_diff_output_json)
6. [REQ-DIFF-006: format_diff_output_human](#req-diff-006-format_diff_output_human)
7. [Edge Case Specifications](#edge-case-specifications)

---

## REQ-DIFF-001: execute_diff_analysis_command

### Problem Statement

AI agents and developers need to understand the impact of code changes on the dependency graph. The main entry point must orchestrate loading two database snapshots, computing diffs, calculating blast radius, and outputting results in the requested format. Without this, users cannot visualize what changed between code versions.

### Specification

```
WHEN I call execute_diff_analysis_command(args)
  WITH args.base_database_path_value = valid RocksDB path (format: "rocksdb:path/to/base.db")
  WITH args.live_database_path_value = valid RocksDB path (format: "rocksdb:path/to/live.db")
  WITH args.json_output_format_flag = boolean
  WITH args.max_hops_depth_limit = u32 in range [1, 10]
THEN SHALL load both databases successfully
  AND SHALL compute entity diff between base and live snapshots
  AND SHALL compute edge diff between base and live snapshots
  AND SHALL calculate blast radius for all significantly changed entities
  AND SHALL transform results to visualization format
  AND SHALL output to stdout in requested format (JSON or human-readable)
  AND SHALL return Ok(())
  AND SHALL NOT modify either database
  AND SHALL NOT panic on valid inputs
```

### Error Conditions

```
WHEN I call execute_diff_analysis_command(args)
  WITH args.base_database_path_value = non-existent path
THEN SHALL return Err with context "Failed to open base database: {path}"
  AND SHALL NOT attempt to open live database

WHEN I call execute_diff_analysis_command(args)
  WITH args.base_database_path_value = valid path
  WITH args.live_database_path_value = non-existent path
THEN SHALL return Err with context "Failed to open live database: {path}"

WHEN I call execute_diff_analysis_command(args)
  WITH args.base_database_path_value = valid path but corrupt database
THEN SHALL return Err with context containing "Failed to query entities"

WHEN I call execute_diff_analysis_command(args)
  WITH args.max_hops_depth_limit = 0
THEN SHALL proceed with zero blast radius depth (no hop traversal)
  AND SHALL return empty blast radius result
```

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Latency (1K entities) | < 500ms | End-to-end wall clock time |
| Latency (10K entities) | < 5s | End-to-end wall clock time |
| Memory (1K entities) | < 50MB | Peak RSS during execution |
| Memory (10K entities) | < 500MB | Peak RSS during execution |

### Verification Test Template

```rust
#[tokio::test]
async fn test_execute_diff_analysis_command_success() {
    // GIVEN two valid database paths with known entity differences
    let base_db = setup_test_database_with_entities(vec![
        ("rust:fn:foo:path:1-10", "src/lib.rs", 1, 10),
        ("rust:fn:bar:path:20-30", "src/lib.rs", 20, 30),
    ]).await;
    let live_db = setup_test_database_with_entities(vec![
        ("rust:fn:foo:path:1-15", "src/lib.rs", 1, 15), // Modified (grew)
        ("rust:fn:baz:path:40-50", "src/lib.rs", 40, 50), // Added
    ]).await;

    let args = DiffCommandArgsPayload {
        base_database_path_value: base_db.path(),
        live_database_path_value: live_db.path(),
        json_output_format_flag: true,
        max_hops_depth_limit: 2,
    };

    // WHEN executing the diff command
    let result = execute_diff_analysis_command(args).await;

    // THEN should succeed
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_execute_diff_analysis_command_invalid_base_path() {
    // GIVEN an invalid base database path
    let args = DiffCommandArgsPayload {
        base_database_path_value: "rocksdb:nonexistent/path.db".to_string(),
        live_database_path_value: "rocksdb:also/nonexistent.db".to_string(),
        json_output_format_flag: false,
        max_hops_depth_limit: 2,
    };

    // WHEN executing the diff command
    let result = execute_diff_analysis_command(args).await;

    // THEN should fail with appropriate error
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Failed to open base database"));
}
```

### Acceptance Criteria
- [ ] Successfully loads two separate CozoDB instances
- [ ] Computes correct entity change counts (added, removed, modified, relocated, unchanged)
- [ ] Computes correct edge change counts
- [ ] Calculates blast radius within specified hop limit
- [ ] Outputs valid JSON when json_output_format_flag is true
- [ ] Outputs human-readable text when json_output_format_flag is false
- [ ] Returns error with context for invalid database paths
- [ ] Does not modify source databases

---

## REQ-DIFF-002: load_database_entity_snapshot

### Problem Statement

To compute a diff, we need to extract all entities from a CozoDB database and transform them into a format suitable for comparison. The entity keys must be preserved for stable identity matching, and content hashes should be computed for modification detection.

### Specification

```
WHEN I call load_database_entity_snapshot(storage)
  WITH storage = valid CozoDbStorage instance
THEN SHALL query all entities from the CodeGraph table
  AND SHALL return HashMap<String, EntityDataPayload> keyed by ISGL1_key
  AND SHALL extract file_path from interface_signature.file_path
  AND SHALL extract line_range from interface_signature.line_range
  AND SHALL compute content_hash from current_code if present
  AND SHALL format entity_type as "{:?}" of interface_signature.entity_type
  AND SHALL NOT include external entity references (unknown:0-0 suffix)

WHEN I call load_database_entity_snapshot(storage)
  WITH storage = valid CozoDbStorage instance
  WITH database containing N entities
THEN SHALL return HashMap with exactly N entries
  AND SHALL have unique keys for all entries
```

### Error Conditions

```
WHEN I call load_database_entity_snapshot(storage)
  WITH storage = disconnected CozoDbStorage instance
THEN SHALL return Err with context "Failed to query entities from database"

WHEN I call load_database_entity_snapshot(storage)
  WITH storage = valid instance but missing CodeGraph table
THEN SHALL return Err with context "Failed to query entities from database"
```

### Data Mapping Contract

| Source (CodeEntity) | Target (EntityDataPayload) | Transformation |
|---------------------|---------------------------|----------------|
| `isgl1_key` | `key` | Direct copy |
| `interface_signature.file_path` | `file_path` | `to_string_lossy()` |
| `interface_signature.entity_type` | `entity_type` | `format!("{:?}", ...)` |
| `interface_signature.line_range.start` | `line_range.start` | Direct copy |
| `interface_signature.line_range.end` | `line_range.end` | Direct copy |
| `current_code` | `content_hash` | `Some(format!("{:x}", hash))` or `None` |

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Latency (1K entities) | < 100ms | Function execution time |
| Latency (10K entities) | < 1s | Function execution time |
| Memory per entity | < 1KB average | HashMap entry size |

### Verification Test Template

```rust
#[tokio::test]
async fn test_load_database_entity_snapshot_success() {
    // GIVEN a database with known entities
    let storage = setup_test_database_with_entities(vec![
        make_code_entity("rust:fn:main:__src_main_rs:1-50", "src/main.rs", 1, 50, Some("fn main() {}")),
        make_code_entity("rust:struct:Config:__src_lib_rs:10-20", "src/lib.rs", 10, 20, None),
    ]).await;

    // WHEN loading the entity snapshot
    let result = load_database_entity_snapshot(&storage).await;

    // THEN should return all entities
    assert!(result.is_ok());
    let entities = result.unwrap();
    assert_eq!(entities.len(), 2);

    // AND should have correct keys
    assert!(entities.contains_key("rust:fn:main:__src_main_rs:1-50"));
    assert!(entities.contains_key("rust:struct:Config:__src_lib_rs:10-20"));

    // AND should have computed content hash for entity with code
    let main_entity = entities.get("rust:fn:main:__src_main_rs:1-50").unwrap();
    assert!(main_entity.content_hash.is_some());

    // AND should have no content hash for entity without code
    let config_entity = entities.get("rust:struct:Config:__src_lib_rs:10-20").unwrap();
    assert!(config_entity.content_hash.is_none());
}

#[tokio::test]
async fn test_load_database_entity_snapshot_empty_database() {
    // GIVEN an empty database
    let storage = setup_empty_test_database().await;

    // WHEN loading the entity snapshot
    let result = load_database_entity_snapshot(&storage).await;

    // THEN should return empty HashMap
    assert!(result.is_ok());
    let entities = result.unwrap();
    assert!(entities.is_empty());
}
```

### Acceptance Criteria
- [ ] Returns all entities from database
- [ ] Keys match ISGL1_key format exactly
- [ ] File paths are correctly extracted
- [ ] Line ranges are correctly extracted
- [ ] Content hashes are computed when current_code is present
- [ ] Content hashes are None when current_code is absent
- [ ] Empty database returns empty HashMap
- [ ] Returns error for database connection issues

---

## REQ-DIFF-003: load_database_edges_snapshot

### Problem Statement

Dependencies (edges) between entities must be loaded to compute edge diffs and calculate blast radius. Each edge connects a "from" entity to a "to" entity with a relationship type.

### Specification

```
WHEN I call load_database_edges_snapshot(storage)
  WITH storage = valid CozoDbStorage instance
THEN SHALL query all dependencies from the DependencyEdges table
  AND SHALL return Vec<EdgeDataPayload>
  AND SHALL extract from_key from DependencyEdge.from_key
  AND SHALL extract to_key from DependencyEdge.to_key
  AND SHALL extract edge_type as string from DependencyEdge.edge_type
  AND SHALL extract source_location from DependencyEdge.source_location
```

### Error Conditions

```
WHEN I call load_database_edges_snapshot(storage)
  WITH storage = disconnected CozoDbStorage instance
THEN SHALL return Err with context "Failed to query dependencies from database"
```

### Data Mapping Contract

| Source (DependencyEdge) | Target (EdgeDataPayload) | Transformation |
|-------------------------|-------------------------|----------------|
| `from_key` | `from_key` | `.into_inner()` |
| `to_key` | `to_key` | `.into_inner()` |
| `edge_type` | `edge_type` | `.to_string()` |
| `source_location` | `source_location` | Direct copy (Option<String>) |

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Latency (10K edges) | < 200ms | Function execution time |
| Memory per edge | < 500 bytes average | Vec entry size |

### Verification Test Template

```rust
#[tokio::test]
async fn test_load_database_edges_snapshot_success() {
    // GIVEN a database with known edges
    let storage = setup_test_database_with_edges(vec![
        ("rust:fn:caller:path:1-10", "rust:fn:callee:path:20-30", "Calls"),
        ("rust:fn:user:path:40-50", "rust:struct:Config:path:60-70", "Uses"),
    ]).await;

    // WHEN loading the edges snapshot
    let result = load_database_edges_snapshot(&storage).await;

    // THEN should return all edges
    assert!(result.is_ok());
    let edges = result.unwrap();
    assert_eq!(edges.len(), 2);

    // AND should have correct edge types
    let calls_edge = edges.iter().find(|e| e.edge_type == "Calls").unwrap();
    assert_eq!(calls_edge.from_key, "rust:fn:caller:path:1-10");
    assert_eq!(calls_edge.to_key, "rust:fn:callee:path:20-30");
}

#[tokio::test]
async fn test_load_database_edges_snapshot_empty() {
    // GIVEN a database with no edges
    let storage = setup_empty_test_database().await;

    // WHEN loading the edges snapshot
    let result = load_database_edges_snapshot(&storage).await;

    // THEN should return empty Vec
    assert!(result.is_ok());
    let edges = result.unwrap();
    assert!(edges.is_empty());
}
```

### Acceptance Criteria
- [ ] Returns all edges from database
- [ ] from_key and to_key are correctly extracted
- [ ] edge_type is correctly converted to string
- [ ] source_location is preserved
- [ ] Empty database returns empty Vec
- [ ] Returns error for database connection issues

---

## REQ-DIFF-004: compute_combined_blast_radius

### Problem Statement

When entities change, other entities that depend on them may be affected. The blast radius calculation determines which entities are transitively impacted, grouped by distance (hops) from the changed entities. This is critical for understanding the ripple effects of code changes.

### Specification

```
WHEN I call compute_combined_blast_radius(changed_keys, edges, max_hops)
  WITH changed_keys = non-empty Vec<String> of entity keys
  WITH edges = Vec<EdgeDataPayload> representing dependency graph
  WITH max_hops = u32 > 0
THEN SHALL compute reverse dependencies (who depends on changed entities)
  AND SHALL traverse up to max_hops levels deep using BFS
  AND SHALL return BlastRadiusResultPayload with:
    - origin_entity = comma-joined changed_keys (normalized)
    - affected_by_distance = HashMap<u32, Vec<String>> (hop distance -> affected entities)
    - total_affected_count = sum of all affected entities (deduplicated)
    - max_depth_reached = maximum hop level with affected entities
  AND SHALL deduplicate entities affected by multiple changed entities
  AND SHALL NOT include the changed entities themselves in affected set
  AND SHALL normalize all keys to stable identity before comparison
```

### Edge Cases

```
WHEN I call compute_combined_blast_radius(changed_keys, edges, max_hops)
  WITH changed_keys = empty Vec
THEN SHALL return BlastRadiusResultPayload with:
    - origin_entity = ""
    - affected_by_distance = empty HashMap
    - total_affected_count = 0
    - max_depth_reached = 0

WHEN I call compute_combined_blast_radius(changed_keys, edges, max_hops)
  WITH edges = empty Vec
THEN SHALL return BlastRadiusResultPayload with:
    - total_affected_count = 0
    - affected_by_distance = empty HashMap

WHEN I call compute_combined_blast_radius(changed_keys, edges, max_hops)
  WITH max_hops = 0
THEN SHALL return BlastRadiusResultPayload with:
    - total_affected_count = 0
    - max_depth_reached = 0

WHEN I call compute_combined_blast_radius(changed_keys, edges, max_hops)
  WITH edges containing cycles (A -> B -> A)
THEN SHALL NOT enter infinite loop
  AND SHALL visit each entity at most once
  AND SHALL complete within reasonable time
```

### Algorithm Contract

```
1. For each changed_key in changed_keys:
   a. Normalize to stable identity
   b. Build reverse dependency graph (to -> [froms])
   c. BFS from changed entity, tracking distance
   d. Stop at max_hops depth

2. Merge results:
   a. Union all affected entities at each distance level
   b. Remove changed entities from affected set
   c. Deduplicate across distances (keep minimum distance)

3. Return combined result
```

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Latency (100 changed, 10K edges, 3 hops) | < 100ms | Function execution time |
| Memory | O(V + E) where V=vertices, E=edges | Peak memory during BFS |

### Verification Test Template

```rust
#[test]
fn test_compute_combined_blast_radius_chain() {
    // GIVEN a chain: A -> B -> C -> D (A calls B, B calls C, etc.)
    let edges = vec![
        create_edge("rust:fn:a:path:1-10", "rust:fn:b:path:20-30"),
        create_edge("rust:fn:b:path:20-30", "rust:fn:c:path:40-50"),
        create_edge("rust:fn:c:path:40-50", "rust:fn:d:path:60-70"),
    ];

    // WHEN D changes with max 2 hops
    let changed = vec!["rust:fn:d:path:60-70".to_string()];
    let result = compute_combined_blast_radius(&changed, &edges, 2);

    // THEN C should be at hop 1, B should be at hop 2
    assert_eq!(result.affected_by_distance.get(&1).map(|v| v.len()), Some(1)); // C
    assert_eq!(result.affected_by_distance.get(&2).map(|v| v.len()), Some(1)); // B
    assert_eq!(result.total_affected_count, 2);
    assert_eq!(result.max_depth_reached, 2);

    // AND A should NOT be included (beyond max hops)
    assert!(!result.all_affected_entities().iter().any(|e| e.contains("fn:a")));
}

#[test]
fn test_compute_combined_blast_radius_multiple_origins() {
    // GIVEN: C depends on both A and B
    let edges = vec![
        create_edge("rust:fn:c:path:1-10", "rust:fn:a:path:20-30"),
        create_edge("rust:fn:c:path:1-10", "rust:fn:b:path:40-50"),
    ];

    // WHEN both A and B change
    let changed = vec![
        "rust:fn:a:path:20-30".to_string(),
        "rust:fn:b:path:40-50".to_string(),
    ];
    let result = compute_combined_blast_radius(&changed, &edges, 2);

    // THEN C should appear only once (deduplicated)
    assert_eq!(result.total_affected_count, 1);
}

#[test]
fn test_compute_combined_blast_radius_handles_cycles() {
    // GIVEN a cycle: A -> B -> A
    let edges = vec![
        create_edge("rust:fn:a:path:1-10", "rust:fn:b:path:20-30"),
        create_edge("rust:fn:b:path:20-30", "rust:fn:a:path:1-10"),
    ];

    // WHEN A changes with large max hops
    let changed = vec!["rust:fn:a:path:1-10".to_string()];

    // THEN should complete without infinite loop
    let start = std::time::Instant::now();
    let result = compute_combined_blast_radius(&changed, &edges, 100);
    let duration = start.elapsed();

    assert!(duration < std::time::Duration::from_secs(1));
    assert!(result.total_affected_count <= 2);
}

#[test]
fn test_compute_combined_blast_radius_empty_inputs() {
    // GIVEN empty changed keys
    let edges = vec![create_edge("rust:fn:a:path:1-10", "rust:fn:b:path:20-30")];
    let result = compute_combined_blast_radius(&[], &edges, 2);

    // THEN should return empty result
    assert_eq!(result.total_affected_count, 0);
    assert!(result.affected_by_distance.is_empty());
}
```

### Acceptance Criteria
- [ ] Correctly identifies entities at each hop distance
- [ ] Respects max_hops limit
- [ ] Deduplicates entities affected by multiple changed entities
- [ ] Excludes changed entities from affected set
- [ ] Handles empty inputs gracefully
- [ ] Handles cycles without infinite loops
- [ ] Normalizes keys to stable identity
- [ ] Returns correct total_affected_count

---

## REQ-DIFF-005: format_diff_output_json

### Problem Statement

API consumers and downstream tools need structured JSON output of diff results for programmatic processing. The output must include the complete diff, blast radius, and visualization data in a well-defined schema.

### Specification

```
WHEN I call format_diff_output_json(diff_result, blast_radius, visualization)
  WITH diff_result = valid DiffResultDataPayload
  WITH blast_radius = valid BlastRadiusResultPayload
  WITH visualization = valid VisualizationGraphDataPayload
THEN SHALL serialize to JSON with structure:
    {
      "diff": DiffResultDataPayload,
      "blast_radius": BlastRadiusResultPayload,
      "visualization": VisualizationGraphDataPayload
    }
  AND SHALL use serde_json::to_string_pretty for formatting
  AND SHALL print to stdout
  AND SHALL return Ok(())
```

### Error Conditions

```
WHEN I call format_diff_output_json(diff_result, blast_radius, visualization)
  WITH any payload containing non-serializable data
THEN SHALL return Err with context "Failed to serialize diff result to JSON"
```

### Output Schema Contract

```json
{
  "diff": {
    "summary": {
      "total_before_count": number,
      "total_after_count": number,
      "added_entity_count": number,
      "removed_entity_count": number,
      "modified_entity_count": number,
      "unchanged_entity_count": number,
      "relocated_entity_count": number
    },
    "entity_changes": [
      {
        "stable_identity": string,
        "change_type": "AddedToCodebase" | "RemovedFromCodebase" | "ModifiedInCodebase" | "RelocatedInCodebase" | "UnchangedInCodebase",
        "before_entity": EntityDataPayload | null,
        "after_entity": EntityDataPayload | null
      }
    ],
    "edge_changes": [
      {
        "from_stable_identity": string,
        "to_stable_identity": string,
        "change_type": "AddedToGraph" | "RemovedFromGraph" | "ModifiedInGraph" | "UnchangedInGraph",
        "before_edge": EdgeDataPayload | null,
        "after_edge": EdgeDataPayload | null
      }
    ]
  },
  "blast_radius": {
    "origin_entity": string,
    "affected_by_distance": { [hop_number: string]: string[] },
    "total_affected_count": number,
    "max_depth_reached": number
  },
  "visualization": {
    "nodes": VisualizationNodeDataPayload[],
    "edges": VisualizationEdgeDataPayload[],
    "diff_summary": DiffSummaryDataPayload,
    "max_blast_radius_depth": number
  }
}
```

### Verification Test Template

```rust
#[test]
fn test_format_diff_output_json_structure() {
    // GIVEN valid diff data
    let diff_result = DiffResultDataPayload {
        summary: DiffSummaryDataPayload {
            total_before_count: 10,
            total_after_count: 12,
            added_entity_count: 3,
            removed_entity_count: 1,
            modified_entity_count: 2,
            unchanged_entity_count: 6,
            relocated_entity_count: 0,
        },
        entity_changes: vec![],
        edge_changes: vec![],
    };

    let blast_radius = BlastRadiusResultPayload {
        origin_entity: "test".to_string(),
        affected_by_distance: HashMap::new(),
        total_affected_count: 0,
        max_depth_reached: 0,
    };

    let visualization = VisualizationGraphDataPayload {
        nodes: vec![],
        edges: vec![],
        diff_summary: diff_result.summary.clone(),
        max_blast_radius_depth: 2,
    };

    // WHEN formatting as JSON (capture stdout)
    let mut output = Vec::new();
    let result = format_diff_output_json_to_writer(&diff_result, &blast_radius, &visualization, &mut output);

    // THEN should succeed
    assert!(result.is_ok());

    // AND output should be valid JSON
    let json_str = String::from_utf8(output).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // AND should have expected top-level keys
    assert!(parsed.get("diff").is_some());
    assert!(parsed.get("blast_radius").is_some());
    assert!(parsed.get("visualization").is_some());

    // AND summary values should match
    assert_eq!(parsed["diff"]["summary"]["added_entity_count"], 3);
}
```

### Acceptance Criteria
- [ ] Output is valid JSON
- [ ] Output is pretty-printed (human-readable indentation)
- [ ] Contains all three top-level keys: diff, blast_radius, visualization
- [ ] All enum values are serialized as strings (e.g., "AddedToCodebase")
- [ ] Null values are properly represented for optional fields
- [ ] Returns error for serialization failures

---

## REQ-DIFF-006: format_diff_output_human

### Problem Statement

Human operators need a readable, colorized summary of diff results in the terminal. The output should highlight important changes and provide actionable insights at a glance.

### Specification

```
WHEN I call format_diff_output_human(args, diff_result, blast_radius)
  WITH args = valid DiffCommandArgsPayload
  WITH diff_result = valid DiffResultDataPayload
  WITH blast_radius = valid BlastRadiusResultPayload
THEN SHALL print formatted output to stdout with sections:
    1. Header: "Diff Analysis: {base_name} vs {live_name}"
    2. Summary: counts for Added, Removed, Modified, Relocated, Unchanged
    3. Blast Radius: total affected + breakdown by hop distance
    4. Changes: up to 20 most significant changes with symbols ([+], [-], [~])
    5. Dependency Changes: summary of added/removed edges
  AND SHALL use console::style for colorization:
    - Added: green
    - Removed: red
    - Modified: yellow
    - Relocated: blue
    - Unchanged: dim
    - Headers: cyan bold
  AND SHALL extract database display names from paths
  AND SHALL return Ok(())
```

### Output Format Contract

```
Diff Analysis: base.db vs live.db
============================================================

Summary:
  Added: N entities
  Removed: N entities
  Modified: N entities
  Relocated: N entities
  Unchanged: N entities

Blast Radius: N affected entities (max M hops)
  Hop 1: N entities
  Hop 2: N entities

Changes:
  [+] type:name (Added)
  [-] type:name (Removed)
  [~] type:name (Modified, +N lines)
  ... and N more changes

Dependency Changes:
  +: N new dependencies
  -: N removed dependencies
```

### Display Name Extraction Contract

```
WHEN extracting database display name
  WITH path = "rocksdb:path/to/analysis.db"
THEN SHALL return "analysis.db"

WHEN extracting database display name
  WITH path = "rocksdb:analysis.db"
THEN SHALL return "analysis.db"

WHEN extracting database display name
  WITH path = "mem"
THEN SHALL return "mem"
```

### Entity Display Name Contract

```
WHEN extracting entity display name
  WITH stable_identity = "rust:fn:main:__crates_src_main_rs"
THEN SHALL return "fn:main"

WHEN extracting entity display name
  WITH stable_identity = "rust:struct:MyStruct:path"
THEN SHALL return "struct:MyStruct"
```

### Verification Test Template

```rust
#[test]
fn test_format_diff_output_human_structure() {
    // GIVEN diff data with various changes
    let args = DiffCommandArgsPayload {
        base_database_path_value: "rocksdb:path/to/base.db".to_string(),
        live_database_path_value: "rocksdb:path/to/live.db".to_string(),
        json_output_format_flag: false,
        max_hops_depth_limit: 2,
    };

    let diff_result = DiffResultDataPayload {
        summary: DiffSummaryDataPayload {
            total_before_count: 10,
            total_after_count: 12,
            added_entity_count: 3,
            removed_entity_count: 1,
            modified_entity_count: 2,
            unchanged_entity_count: 6,
            relocated_entity_count: 0,
        },
        entity_changes: vec![
            EntityChangeDataItem {
                stable_identity: "rust:fn:new_func:path".to_string(),
                change_type: EntityChangeTypeClassification::AddedToCodebase,
                before_entity: None,
                after_entity: Some(create_entity_payload("rust:fn:new_func:path:1-10")),
            },
        ],
        edge_changes: vec![],
    };

    let mut affected = HashMap::new();
    affected.insert(1, vec!["affected1".to_string()]);
    let blast_radius = BlastRadiusResultPayload {
        origin_entity: "test".to_string(),
        affected_by_distance: affected,
        total_affected_count: 1,
        max_depth_reached: 1,
    };

    // WHEN formatting as human-readable (capture stdout)
    let result = format_diff_output_human(&args, &diff_result, &blast_radius);

    // THEN should succeed
    assert!(result.is_ok());
}

#[test]
fn test_extract_database_display_name() {
    assert_eq!(extract_database_display_name("rocksdb:path/to/analysis.db"), "analysis.db");
    assert_eq!(extract_database_display_name("rocksdb:analysis.db"), "analysis.db");
    assert_eq!(extract_database_display_name("mem"), "mem");
}

#[test]
fn test_extract_entity_display_name() {
    assert_eq!(extract_entity_display_name("rust:fn:main:__crates_src_main_rs"), "fn:main");
    assert_eq!(extract_entity_display_name("rust:struct:Config:path"), "struct:Config");
}
```

### Acceptance Criteria
- [ ] Outputs readable summary to stdout
- [ ] Uses appropriate colors for each change type
- [ ] Extracts database names correctly
- [ ] Limits change output to 20 items
- [ ] Shows "... and N more changes" when truncated
- [ ] Shows blast radius breakdown by hop
- [ ] Shows dependency change summary
- [ ] Returns Ok(()) on success

---

## Edge Case Specifications

### REQ-DIFF-EDGE-001: Empty Base Database

```
WHEN I call execute_diff_analysis_command(args)
  WITH base database containing 0 entities
  WITH live database containing N > 0 entities
THEN SHALL report:
    - added_entity_count = N
    - removed_entity_count = 0
    - modified_entity_count = 0
    - unchanged_entity_count = 0
  AND SHALL show all live entities as "Added"
```

### REQ-DIFF-EDGE-002: Empty Live Database

```
WHEN I call execute_diff_analysis_command(args)
  WITH base database containing N > 0 entities
  WITH live database containing 0 entities
THEN SHALL report:
    - added_entity_count = 0
    - removed_entity_count = N
    - modified_entity_count = 0
    - unchanged_entity_count = 0
  AND SHALL show all base entities as "Removed"
  AND SHALL have blast_radius.total_affected_count = 0 (no live edges)
```

### REQ-DIFF-EDGE-003: Both Databases Empty

```
WHEN I call execute_diff_analysis_command(args)
  WITH base database containing 0 entities
  WITH live database containing 0 entities
THEN SHALL report:
    - all counts = 0
  AND SHALL produce valid output (not error)
  AND SHALL have empty entity_changes
  AND SHALL have empty edge_changes
```

### REQ-DIFF-EDGE-004: No Changes (Identical Databases)

```
WHEN I call execute_diff_analysis_command(args)
  WITH base database = exact copy of live database
THEN SHALL report:
    - added_entity_count = 0
    - removed_entity_count = 0
    - modified_entity_count = 0
    - unchanged_entity_count = N
  AND SHALL have blast_radius.total_affected_count = 0
```

### REQ-DIFF-EDGE-005: All Entities Removed and New Ones Added

```
WHEN I call execute_diff_analysis_command(args)
  WITH base database containing entities {A, B, C}
  WITH live database containing entities {X, Y, Z} (completely different)
THEN SHALL report:
    - added_entity_count = 3
    - removed_entity_count = 3
    - modified_entity_count = 0
    - unchanged_entity_count = 0
```

### REQ-DIFF-EDGE-006: Entity Relocated (Same Name, Different Lines)

```
WHEN I call execute_diff_analysis_command(args)
  WITH base entity key = "rust:fn:foo:path:1-10"
  WITH live entity key = "rust:fn:foo:path:50-60" (same stable identity)
  WITH identical content
THEN SHALL classify as RelocatedInCodebase
  AND SHALL NOT classify as Added + Removed
  AND SHALL NOT classify as Modified
```

### REQ-DIFF-EDGE-007: External Entity References

```
WHEN I call load_database_entity_snapshot(storage)
  WITH database containing external references (e.g., "rust:fn:HashMap:unknown:0-0")
THEN SHALL include external references in snapshot
  BUT compute_blast_radius SHALL skip external entities as change origins
  AND external entities SHALL NOT appear in blast radius results
```

### Edge Case Verification Test Template

```rust
#[tokio::test]
async fn test_diff_empty_base_database() {
    // GIVEN empty base, populated live
    let base_db = setup_empty_test_database().await;
    let live_db = setup_test_database_with_entities(vec![
        ("rust:fn:foo:path:1-10", "src/lib.rs", 1, 10),
        ("rust:fn:bar:path:20-30", "src/lib.rs", 20, 30),
    ]).await;

    // WHEN diffing
    let result = execute_diff_and_capture_result(&base_db, &live_db).await;

    // THEN all entities should be "Added"
    assert_eq!(result.summary.added_entity_count, 2);
    assert_eq!(result.summary.removed_entity_count, 0);
}

#[tokio::test]
async fn test_diff_empty_live_database() {
    // GIVEN populated base, empty live
    let base_db = setup_test_database_with_entities(vec![
        ("rust:fn:foo:path:1-10", "src/lib.rs", 1, 10),
    ]).await;
    let live_db = setup_empty_test_database().await;

    // WHEN diffing
    let result = execute_diff_and_capture_result(&base_db, &live_db).await;

    // THEN all entities should be "Removed"
    assert_eq!(result.summary.added_entity_count, 0);
    assert_eq!(result.summary.removed_entity_count, 1);
}

#[tokio::test]
async fn test_diff_identical_databases() {
    // GIVEN identical databases
    let entities = vec![("rust:fn:foo:path:1-10", "src/lib.rs", 1, 10)];
    let base_db = setup_test_database_with_entities(entities.clone()).await;
    let live_db = setup_test_database_with_entities(entities).await;

    // WHEN diffing
    let result = execute_diff_and_capture_result(&base_db, &live_db).await;

    // THEN no changes
    assert_eq!(result.summary.added_entity_count, 0);
    assert_eq!(result.summary.removed_entity_count, 0);
    assert_eq!(result.summary.modified_entity_count, 0);
    assert_eq!(result.summary.unchanged_entity_count, 1);
}

#[tokio::test]
async fn test_diff_relocated_entity() {
    // GIVEN entity that moved (same stable identity, different lines)
    let base_db = setup_test_database_with_entities(vec![
        ("rust:fn:foo:path:1-10", "src/lib.rs", 1, 10),
    ]).await;
    let live_db = setup_test_database_with_entities(vec![
        ("rust:fn:foo:path:50-59", "src/lib.rs", 50, 59), // Same size, different location
    ]).await;

    // WHEN diffing
    let result = execute_diff_and_capture_result(&base_db, &live_db).await;

    // THEN should be classified as relocated
    assert_eq!(result.summary.relocated_entity_count, 1);
    assert_eq!(result.summary.added_entity_count, 0);
    assert_eq!(result.summary.removed_entity_count, 0);
}
```

---

## Quality Checklist

Before implementing tests from these specifications, verify:

- [x] All quantities are specific and measurable
- [x] All behaviors are testable
- [x] Error conditions are specified
- [x] Performance boundaries are defined
- [x] Test templates are provided in Rust
- [x] Acceptance criteria are binary (pass/fail)
- [x] No ambiguous language remains
- [x] Edge cases are comprehensively covered
- [x] 4-Word Naming Convention is followed

---

## Implementation Priority

| Requirement | Priority | Rationale |
|------------|----------|-----------|
| REQ-DIFF-002 (load_database_entity_snapshot) | P0 | Foundation for all diff operations |
| REQ-DIFF-003 (load_database_edges_snapshot) | P0 | Foundation for blast radius |
| REQ-DIFF-004 (compute_combined_blast_radius) | P0 | Core value proposition |
| REQ-DIFF-001 (execute_diff_analysis_command) | P1 | Orchestration depends on above |
| REQ-DIFF-005 (format_diff_output_json) | P1 | Primary API output |
| REQ-DIFF-006 (format_diff_output_human) | P2 | Developer experience |
| Edge Cases | P1 | Robustness |

---

## Appendix: Type Definitions Reference

For convenience, here are the key types referenced in these specifications:

```rust
// From parseltongue-core/src/diff/types.rs

pub struct DiffCommandArgsPayload {
    pub base_database_path_value: String,
    pub live_database_path_value: String,
    pub json_output_format_flag: bool,
    pub max_hops_depth_limit: u32,
}

pub struct EntityDataPayload {
    pub key: String,
    pub file_path: String,
    pub entity_type: String,
    pub line_range: Option<LineRangeData>,
    pub content_hash: Option<String>,
}

pub struct EdgeDataPayload {
    pub from_key: String,
    pub to_key: String,
    pub edge_type: String,
    pub source_location: Option<String>,
}

pub struct DiffResultDataPayload {
    pub summary: DiffSummaryDataPayload,
    pub entity_changes: Vec<EntityChangeDataItem>,
    pub edge_changes: Vec<EdgeChangeDataItem>,
}

pub struct BlastRadiusResultPayload {
    pub origin_entity: String,
    pub affected_by_distance: HashMap<u32, Vec<String>>,
    pub total_affected_count: usize,
    pub max_depth_reached: u32,
}

pub enum EntityChangeTypeClassification {
    AddedToCodebase,
    RemovedFromCodebase,
    ModifiedInCodebase,
    RelocatedInCodebase,
    UnchangedInCodebase,
}
```

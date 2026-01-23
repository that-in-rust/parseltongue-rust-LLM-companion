# Executable Specifications: Diff Visualization System

> **Document Type**: Executable Specifications (WHEN...THEN...SHALL contracts)
> **Date**: 2026-01-23
> **Status**: Draft
> **Derived From**: API queries, ADR_001_KEY_NORMALIZATION.md, GAP_ANALYSIS_METHODOLOGY_EVOLUTION_20260122.md

---

## Executive Summary

This document defines testable specifications for the Parseltongue diff visualization system. Each specification uses the WHEN...THEN...SHALL format to create executable contracts that can be validated through automated tests.

**API-Grounded Insights** (from querying http://localhost:7777):
- Current codebase: 215 code entities, 2880 dependency edges
- 49 semantic clusters identified via label propagation
- Entity keys encode line numbers: `rust:fn:name:__path:start-end`
- External references tracked at `unknown:0-0`
- Blast radius API requires exact key matching

---

## Part 1: End-to-End User Scenario Specifications

### SPEC-E2E-001: User Adds Workspace and Sees Initial Graph

```
SPECIFICATION: Initial Workspace Creation

GIVEN a user has a code repository at path ./my-project
AND the repository contains Rust source files
AND no Parseltongue workspace exists

WHEN the user runs:
  $ parseltongue pt01-folder-to-cozodb-streamer ./my-project

THEN the system SHALL:
  1. Create a timestamped workspace folder: parseltongue{YYYYMMDDHHMMSS}/
  2. Generate analysis.db with all code entities
  3. Print to stdout: "Workspace: parseltongue{timestamp}"
  4. Print to stdout: "Database: rocksdb:parseltongue{timestamp}/analysis.db"

AND the database SHALL contain:
  - All functions, structs, enums, impl blocks as entities
  - Entity keys in format: {lang}:{type}:{name}:{path_hash}:{start_line}-{end_line}
  - Dependency edges between entities

VERIFICATION:
  - Query /codebase-statistics-overview-summary returns non-zero entity count
  - Query /code-entities-list-all returns entities with valid keys

ACCEPTANCE CRITERIA:
  - [ ] Workspace folder created with timestamp
  - [ ] Database file exists and is queryable
  - [ ] Entity count > 0
  - [ ] Edge count > 0
```

### SPEC-E2E-002: Agent Modifies File, Diff Shows Changes

```
SPECIFICATION: File Modification Detection

GIVEN a Parseltongue workspace exists with base.db at commit ABC123
AND a pt09 unified server is running with dual database handles
AND the user has WebSocket connection to /ws/workspace/{id}

WHEN an AI agent (Claude Code) modifies ./src/auth.rs:
  - Adds new function: `validate_token_expiry()`
  - Modifies existing function: `check_permissions()` (adds 5 lines)
  - Deletes function: `deprecated_auth()`

THEN the system SHALL:
  1. Detect file change within 500ms via notify crate
  2. Re-index only ./src/auth.rs (incremental update)
  3. Compute diff between base.db and live.db
  4. Push WebSocket message within 1000ms of file save:

  {
    "type": "diff_updated",
    "timestamp": "2026-01-23T14:30:00Z",
    "changes": {
      "added": [
        {
          "key": "rust:fn:validate_token_expiry:__crates_src_auth_rs:45-60",
          "stable_id": "rust:fn:validate_token_expiry:__crates_src_auth_rs",
          "file_path": "./src/auth.rs",
          "entity_type": "function"
        }
      ],
      "removed": [
        {
          "key": "rust:fn:deprecated_auth:__crates_src_auth_rs:100-120",
          "stable_id": "rust:fn:deprecated_auth:__crates_src_auth_rs",
          "file_path": "./src/auth.rs",
          "entity_type": "function"
        }
      ],
      "modified": [
        {
          "base_key": "rust:fn:check_permissions:__crates_src_auth_rs:20-35",
          "live_key": "rust:fn:check_permissions:__crates_src_auth_rs:20-40",
          "stable_id": "rust:fn:check_permissions:__crates_src_auth_rs",
          "change_type": "lines_changed"
        }
      ],
      "moved": [],
      "neighbors": ["rust:fn:authenticate:__crates_src_auth_rs:5-18"]
    },
    "summary": {
      "added_count": 1,
      "removed_count": 1,
      "modified_count": 1,
      "moved_count": 0,
      "neighbor_count": 1
    }
  }

VERIFICATION:
  - WebSocket client receives message
  - Visual graph updates to show green (added), red (removed), yellow (modified)
  - 1-hop neighbors highlighted in orange

ACCEPTANCE CRITERIA:
  - [ ] File change detected < 500ms
  - [ ] Diff computed < 500ms
  - [ ] WebSocket push < 1000ms total latency
  - [ ] All change types correctly classified
```

### SPEC-E2E-003: Diff Visualized in 3D Graph

```
SPECIFICATION: 3D Graph Diff Rendering

GIVEN a diff result with:
  - 2 added entities
  - 1 removed entity
  - 1 modified entity
  - 4 neighbor entities (1-hop)

WHEN the Three.js visualization receives the diff data

THEN the system SHALL render:

  | Status   | Color   | Size  | Animation     | Label  |
  |----------|---------|-------|---------------|--------|
  | added    | #00ff88 | 1.5x  | Pulse in      | [+]    |
  | removed  | #ff4444 | 1.0x  | Fade out      | [-]    |
  | modified | #ffcc00 | 1.2x  | Subtle pulse  | [~]    |
  | neighbor | #ffa94d | 1.0x  | Glow          |        |
  | ambient  | #888888 | 0.5x  | None          |        |

AND the camera SHALL:
  - Auto-orbit to center on changed entities
  - Zoom to fit all changed + neighbor entities in view

AND edges between changed entities SHALL:
  - Highlight in same color as source node
  - Animate flow direction (particles or gradient)

VERIFICATION:
  - Visual regression test with snapshot comparison
  - FPS remains > 30 during animation
  - All labeled nodes are readable

ACCEPTANCE CRITERIA:
  - [ ] Each change type has distinct visual treatment
  - [ ] Neighbors are visually distinct from changes
  - [ ] Camera positions to show changes
  - [ ] Performance: 60 FPS target, 30 FPS minimum
```

---

## Part 2: CLI Command Specifications

### SPEC-CLI-001: Diff Between Two Databases

```
SPECIFICATION: CLI Diff Command

GIVEN two Parseltongue databases exist:
  - base.db: Indexed at commit ABC123 (5 days ago)
  - live.db: Indexed at current working directory (uncommitted changes)

WHEN the user runs:
  $ parseltongue diff base.db live.db

THEN the system SHALL output:

  Parseltongue Diff Analysis
  ==========================
  Base: rocksdb:base.db (commit: ABC123)
  Live: rocksdb:live.db (working directory)

  Summary:
    Added:    3 entities
    Removed:  1 entity
    Modified: 5 entities
    Moved:    2 entities
    Total:    11 changes

  Added Entities:
    [+] rust:fn:new_handler          ./src/api/handlers.rs:45-60
    [+] rust:struct:NewConfig        ./src/config.rs:10-25
    [+] rust:impl:NewConfig          ./src/config.rs:27-50

  Removed Entities:
    [-] rust:fn:deprecated_func      ./src/legacy.rs:100-150

  Modified Entities:
    [~] rust:fn:process_request      ./src/api/handlers.rs:100-150 -> 100-165
    [~] rust:fn:validate_input       ./src/validation.rs:20-40 -> 20-55
    ... (3 more)

  Moved Entities (line numbers only):
    [->] rust:fn:helper_func         :50-60 -> :55-65 (same file)
    [->] rust:struct:OldStruct       :10-30 -> :15-35 (same file)

  Blast Radius (1-hop neighbors of changes):
    12 entities may be affected by these changes

AND the exit code SHALL be:
  - 0 if no changes
  - 1 if changes detected
  - 2 if error (database not found, parse error, etc.)

AND optional flags SHALL support:
  --json           Output as JSON
  --hops N         Include N-hop neighbors (default: 1)
  --filter TYPE    Filter by entity type (fn, struct, impl, etc.)
  --file PATH      Filter by file path pattern

VERIFICATION:
  $ parseltongue diff base.db live.db --json | jq '.summary.added_count'
  3

ACCEPTANCE CRITERIA:
  - [ ] Command parses both database paths
  - [ ] Diff computed using stable identity matching
  - [ ] Output is human-readable by default
  - [ ] JSON output is valid and parseable
  - [ ] Exit codes are correct
```

### SPEC-CLI-002: Interactive Diff Mode

```
SPECIFICATION: Interactive Watch Mode

GIVEN a Parseltongue workspace exists

WHEN the user runs:
  $ parseltongue diff --watch base.db

THEN the system SHALL:
  1. Start file watcher on current directory
  2. On each file change:
     a. Re-index changed files only (incremental)
     b. Compute diff against base.db
     c. Print changes to stdout (or update in place)
  3. Continue watching until Ctrl+C

AND the output SHALL update in place:
  [Watching ./] (12 entities changed)
  Last change: ./src/auth.rs at 14:30:05

  Recent Changes:
    [+] rust:fn:new_func         ./src/auth.rs:45
    [~] rust:fn:existing_func    ./src/auth.rs:20

  Press Ctrl+C to exit, 'r' to refresh, 'c' to clear base

ACCEPTANCE CRITERIA:
  - [ ] File watcher initializes successfully
  - [ ] Incremental re-indexing works
  - [ ] Diff updates on each file save
  - [ ] Clean exit on Ctrl+C
```

---

## Part 3: Key Normalization Specifications

### SPEC-KEY-001: Stable Identity Extraction

```
SPECIFICATION: Extract Stable Identity from Entity Key

GIVEN an entity key in format:
  {language}:{entity_type}:{name}:{path_hash}:{start_line}-{end_line}

WHEN extracting stable identity

THEN the system SHALL strip the line number suffix:

  Input:  "rust:fn:handle_auth:__crates_path_src_auth_rs:10-50"
  Output: "rust:fn:handle_auth:__crates_path_src_auth_rs"

  Input:  "rust:method:new:__crates_parseltongue-core_src_storage_cozo_client_rs:38-54"
  Output: "rust:method:new:__crates_parseltongue-core_src_storage_cozo_client_rs"

  Input:  "rust:fn:map:unknown:0-0"
  Output: "rust:fn:map:unknown"

ALGORITHM:
  fn extract_stable_identity(key: &str) -> &str {
      // Find last colon
      if let Some(last_colon) = key.rfind(':') {
          // Find second-to-last colon
          if let Some(second_last) = key[..last_colon].rfind(':') {
              let suffix = &key[second_last + 1..];
              // Verify suffix is line numbers (digits, hyphen, colon only)
              if suffix.chars().all(|c| c.is_ascii_digit() || c == '-' || c == ':') {
                  return &key[..second_last];
              }
          }
      }
      key // Fallback: return full key
  }

TEST CASES:
  | Input Key | Expected Stable ID |
  |-----------|-------------------|
  | rust:fn:main:__path:10-50 | rust:fn:main:__path |
  | rust:struct:Config:__path:1-20 | rust:struct:Config:__path |
  | rust:fn:helper:unknown:0-0 | rust:fn:helper:unknown |
  | rust:impl:MyStruct:__path:100-200 | rust:impl:MyStruct:__path |
  | invalid_key_format | invalid_key_format |

ACCEPTANCE CRITERIA:
  - [ ] All standard key formats correctly parsed
  - [ ] External references (0-0) handled correctly
  - [ ] Invalid keys return unchanged
  - [ ] O(1) time complexity (single string scan)
```

### SPEC-KEY-002: Moved vs Added/Removed Detection

```
SPECIFICATION: Distinguish Moved Entities from Added/Removed

GIVEN two entity lists from base.db and live.db
AND a function exists in both but with different line numbers:
  Base: rust:fn:helper:__path:20-30
  Live: rust:fn:helper:__path:25-35

WHEN computing entity diff

THEN the system SHALL classify as MOVED (not added+removed):

  Classification Logic:
  1. Extract stable_id from both keys
     Base stable_id: rust:fn:helper:__path
     Live stable_id: rust:fn:helper:__path

  2. stable_id matches -> entity exists in both

  3. Compare full keys:
     - Same key: UNCHANGED
     - Different lines only, same file: MOVED
     - Different file: RELOCATED
     - Only in base: REMOVED
     - Only in live: ADDED

TRUTH TABLE:
  | In Base | In Live | Keys Match | File Same | Classification |
  |---------|---------|------------|-----------|----------------|
  | Yes     | Yes     | Yes        | -         | UNCHANGED      |
  | Yes     | Yes     | No         | Yes       | MOVED          |
  | Yes     | Yes     | No         | No        | RELOCATED      |
  | Yes     | No      | -          | -         | REMOVED        |
  | No      | Yes     | -          | -         | ADDED          |

VERIFICATION TEST:
  fn test_moved_not_added_removed() {
      let base = vec![entity("rust:fn:main:__path:10-50")];
      let live = vec![entity("rust:fn:main:__path:15-55")];

      let diff = compute_entity_diff(base, live);

      assert!(diff.added.is_empty(), "Should not show as added");
      assert!(diff.removed.is_empty(), "Should not show as removed");
      assert_eq!(diff.moved.len(), 1, "Should show as moved");
  }

ACCEPTANCE CRITERIA:
  - [ ] Same stable_id + different lines = MOVED
  - [ ] Same stable_id + different file = RELOCATED
  - [ ] No false positives from line number shifts
  - [ ] Adding blank lines doesn't cause spurious diffs
```

### SPEC-KEY-003: Collision Handling

```
SPECIFICATION: Handle Key Collisions

GIVEN two different functions with the same name in different scopes:

  rust:fn:new:__crates_module_a_rs:10-20
  rust:fn:new:__crates_module_b_rs:50-60

WHEN extracting stable identities

THEN they SHALL have different stable_ids:
  - rust:fn:new:__crates_module_a_rs
  - rust:fn:new:__crates_module_b_rs

AND if the same function name exists twice in one file (overloaded):
  - rust:fn:process:__path:10-20  (first)
  - rust:fn:process:__path:50-80  (second)

THEN the system SHALL:
  1. Include additional context in stable_id (if available)
  2. OR treat as potential collision and compare content
  3. OR log warning and treat as distinct entities

COLLISION RESOLUTION STRATEGY (for MVP):
  - Path hash already distinguishes files
  - Same name in same file: use line numbers as tiebreaker
  - Log: "Warning: Potential collision for stable_id {id}"

ACCEPTANCE CRITERIA:
  - [ ] Different files = different stable_ids
  - [ ] Same file, same name = distinguished somehow
  - [ ] Collisions logged but don't crash
```

---

## Part 4: Diff Algorithm Specifications

### SPEC-DIFF-001: Entity Diff Computation

```
SPECIFICATION: Compute Entity Diff Between Databases

GIVEN:
  - base_entities: Vec<Entity> from base.db
  - live_entities: Vec<Entity> from live.db

WHEN calling compute_entity_diff(base, live)

THEN the system SHALL return:

  pub struct EntityDiff {
      pub added: Vec<Entity>,        // Only in live
      pub removed: Vec<Entity>,      // Only in base
      pub modified: Vec<ModifiedEntity>,  // Same stable_id, content changed
      pub moved: Vec<MovedEntity>,   // Same stable_id, line numbers changed
      pub relocated: Vec<RelocatedEntity>, // Same stable_id, file changed
      pub unchanged_count: usize,    // For metrics
  }

ALGORITHM:
  1. Build HashMap<stable_id, NormalizedEntity> for base
  2. Build HashMap<stable_id, NormalizedEntity> for live
  3. Collect all unique stable_ids from both
  4. For each stable_id:
     - Classify change type
     - Add to appropriate diff category
  5. Return diff struct

PERFORMANCE REQUIREMENTS:
  - O(n) time where n = max(base.len(), live.len())
  - < 100ms for 10,000 entities
  - < 10ms for 1,000 entities (typical codebase)

VERIFICATION:
  #[test]
  fn test_diff_performance_10k_entities() {
      let base = generate_entities(10_000);
      let live = mutate_entities(&base, 100); // 100 changes

      let start = Instant::now();
      let diff = compute_entity_diff(base, live);
      let elapsed = start.elapsed();

      assert!(elapsed < Duration::from_millis(100));
      assert_eq!(diff.added.len() + diff.removed.len() + diff.modified.len(), 100);
  }

ACCEPTANCE CRITERIA:
  - [ ] All change types correctly classified
  - [ ] No false positives from line shifts
  - [ ] Performance meets SLO
  - [ ] Empty inputs handled gracefully
```

### SPEC-DIFF-002: Edge Diff Computation

```
SPECIFICATION: Compute Dependency Edge Diff

GIVEN:
  - base_edges: Vec<Edge> from base.db (2880 edges typical)
  - live_edges: Vec<Edge> from live.db

WHEN computing edge diff

THEN the system SHALL return:

  pub struct EdgeDiff {
      pub added: Vec<Edge>,    // New dependencies
      pub removed: Vec<Edge>,  // Broken dependencies
  }

EDGE IDENTITY:
  An edge is uniquely identified by: (from_stable_id, to_stable_id, edge_type)

  Note: from_key and to_key include line numbers, but edge identity
  should use stable_ids for matching.

EXAMPLE:
  Base edge: A:10-20 -> B:30-40 (Calls)
  Live edge: A:15-25 -> B:35-45 (Calls)

  These are the SAME edge (stable_ids match), not added+removed.

ALGORITHM:
  1. Normalize edge keys to stable_ids
  2. Create edge identity: (from_stable_id, to_stable_id, edge_type)
  3. Compare sets
  4. Edges only in live = added
  5. Edges only in base = removed

ACCEPTANCE CRITERIA:
  - [ ] Edge identity uses stable_ids
  - [ ] Edge type is part of identity (A->B Calls vs A->B Implements are different)
  - [ ] Line number changes don't create spurious edge diffs
```

### SPEC-DIFF-003: Incremental Update on File Change

```
SPECIFICATION: Incremental Re-indexing

GIVEN:
  - A file ./src/auth.rs is modified
  - live.db contains entities from previous index

WHEN the file watcher detects the change

THEN the system SHALL:

  1. DELETE from live.db WHERE file_path = "./src/auth.rs"
     - Remove all entities with that file path
     - Remove all edges FROM or TO those entities

  2. RE-INDEX ./src/auth.rs only
     - Parse with tree-sitter
     - Extract entities
     - Extract dependencies

  3. INSERT new entities and edges into live.db

  4. RECOMPUTE cross-file edges
     - Edges FROM new entities TO existing entities
     - Edges FROM existing entities TO new entities
     - Note: This may require scanning other files' dependency lists

OPTIMIZATION:
  - Cache AST of unchanged files
  - Only recompute edges involving changed file
  - Use inverted index: entity_name -> files_that_reference_it

PERFORMANCE REQUIREMENTS:
  - Single file re-index: < 500ms
  - Diff computation after re-index: < 100ms

ACCEPTANCE CRITERIA:
  - [ ] Old entities removed
  - [ ] New entities inserted
  - [ ] Edges correctly updated
  - [ ] No orphaned edges
  - [ ] Performance SLO met
```

---

## Part 5: Blast Radius Specifications

### SPEC-BLAST-001: 1-Hop Neighbor Identification

```
SPECIFICATION: Identify 1-Hop Neighbors of Changed Entities

GIVEN a set of changed entities (added, removed, modified)
AND a dependency graph with edges

WHEN computing blast radius with hops=1

THEN the system SHALL return all entities that:
  1. CALL any changed entity (reverse dependencies)
  2. ARE CALLED BY any changed entity (forward dependencies)

EXAMPLE:
  Changed entity: rust:fn:authenticate

  Reverse callers (who calls authenticate):
    - rust:fn:handle_login (calls authenticate)
    - rust:fn:api_middleware (calls authenticate)

  Forward callees (what does authenticate call):
    - rust:fn:validate_credentials (called by authenticate)
    - rust:fn:log_access (called by authenticate)

  1-hop blast radius = {handle_login, api_middleware, validate_credentials, log_access}

API INTEGRATION:
  Use existing endpoints:
  - GET /reverse-callers-query-graph?entity={key}
  - GET /forward-callees-query-graph?entity={key}

  Combine results for all changed entities, deduplicate.

ALGORITHM:
  fn compute_blast_radius(changed: &[Entity], hops: usize) -> Vec<Entity> {
      let mut frontier: HashSet<String> = changed.iter()
          .map(|e| e.stable_id.clone())
          .collect();

      let mut visited: HashSet<String> = frontier.clone();
      let mut result: Vec<Entity> = Vec::new();

      for _ in 0..hops {
          let mut next_frontier = HashSet::new();

          for stable_id in &frontier {
              // Get callers
              for caller in get_reverse_callers(stable_id) {
                  if !visited.contains(&caller.stable_id) {
                      visited.insert(caller.stable_id.clone());
                      next_frontier.insert(caller.stable_id.clone());
                      result.push(caller);
                  }
              }

              // Get callees
              for callee in get_forward_callees(stable_id) {
                  if !visited.contains(&callee.stable_id) {
                      visited.insert(callee.stable_id.clone());
                      next_frontier.insert(callee.stable_id.clone());
                      result.push(callee);
                  }
              }
          }

          frontier = next_frontier;
      }

      result
  }

ACCEPTANCE CRITERIA:
  - [ ] All direct callers included
  - [ ] All direct callees included
  - [ ] No duplicates in result
  - [ ] Changed entities not included (they're already in diff)
  - [ ] Handles leaf nodes (no neighbors) gracefully
```

### SPEC-BLAST-002: Multi-Hop Expansion

```
SPECIFICATION: N-Hop Blast Radius

GIVEN a set of changed entities and N hops

WHEN computing blast radius

THEN the system SHALL:
  1. Start with changed entities as "wave 0"
  2. For each wave 1..N:
     - Find all neighbors of previous wave
     - Exclude already-visited entities
     - Add to result with wave number

RESULT FORMAT:
  {
    "blast_radius": {
      "hop_1": [
        {"entity": "rust:fn:caller1", "relation": "calls_changed"},
        {"entity": "rust:fn:callee1", "relation": "called_by_changed"}
      ],
      "hop_2": [
        {"entity": "rust:fn:indirect1", "relation": "calls_hop1"}
      ]
    },
    "total_affected": 15,
    "by_hop": [8, 5, 2]  // hop_1: 8, hop_2: 5, hop_3: 2
  }

VISUALIZATION GUIDANCE:
  - hop_1: Orange glow (most relevant)
  - hop_2: Dimmer orange
  - hop_3+: Subtle highlight only

PERFORMANCE:
  - Typical codebase: hop_1 < 50ms, hop_2 < 200ms, hop_3 < 500ms
  - Exponential growth expected; cap at 1000 entities per hop

ACCEPTANCE CRITERIA:
  - [ ] Correctly expands N hops
  - [ ] No cycles (visited set)
  - [ ] Hop distance tracked per entity
  - [ ] Performance degrades gracefully
```

### SPEC-BLAST-003: External Reference Handling

```
SPECIFICATION: Handle External Dependencies in Blast Radius

GIVEN external references like:
  rust:fn:map:unknown:0-0
  rust:fn:clone:unknown:0-0

WHEN computing blast radius

THEN the system SHALL:
  1. INCLUDE edges TO external entities (our code calls std::map)
  2. EXCLUDE external entities themselves from result
  3. NOT traverse THROUGH external entities (no blast radius of std lib)

RATIONALE:
  - We care that our code uses HashMap
  - We don't care what HashMap internally calls
  - External entities are graph boundaries

EXAMPLE:
  Changed: rust:fn:process_data
  Edges:
    - process_data -> HashMap::new (external)
    - process_data -> our_helper (internal)

  Blast radius should include:
    - our_helper (internal callee)

  Should NOT include:
    - HashMap::new (external - stop traversal)
    - HashMap's internal methods (never reach)

IDENTIFICATION:
  fn is_external(entity: &Entity) -> bool {
      entity.key.contains("unknown:0-0") ||
      entity.file_path == "unknown" ||
      !entity.file_path.starts_with("./")
  }

ACCEPTANCE CRITERIA:
  - [ ] External entities excluded from result
  - [ ] Traversal stops at external boundary
  - [ ] Edges TO externals still tracked (for completeness)
```

---

## Part 6: Integration Test Specifications

### SPEC-INT-001: Full Pipeline Test

```
SPECIFICATION: End-to-End Pipeline Integration Test

SETUP:
  1. Create temp directory with sample Rust project
  2. Initialize git repository
  3. Run pt01 to create base.db
  4. Commit as baseline

TEST SEQUENCE:
  1. Start pt09 unified server with base.db

  2. Modify source file:
     - Add function: `fn new_feature() { }`
     - Modify function: add 3 lines to existing function
     - Delete function: remove `fn old_code()`

  3. Connect WebSocket client

  4. Trigger file watcher (save file)

  5. Assert WebSocket receives diff within 2 seconds:
     - added.len() == 1
     - modified.len() == 1
     - removed.len() == 1
     - neighbors exist

  6. Query HTTP endpoint /diff to verify same result

CLEANUP:
  - Stop server
  - Remove temp directory

ACCEPTANCE CRITERIA:
  - [ ] Full pipeline works end-to-end
  - [ ] WebSocket delivers correct diff
  - [ ] HTTP endpoint agrees with WebSocket
  - [ ] < 2 second total latency
```

### SPEC-INT-002: Large Codebase Performance Test

```
SPECIFICATION: Performance at Scale

SETUP:
  Generate synthetic codebase:
  - 5,000 entities
  - 50,000 edges
  - 100 files

TEST:
  1. Modify 10 files simultaneously
  2. Measure:
     - Time to detect changes
     - Time to re-index
     - Time to compute diff
     - Time to compute blast radius (hops=2)
     - Memory usage peak

REQUIREMENTS:
  | Metric | Target | Acceptable |
  |--------|--------|------------|
  | Detection | < 100ms | < 500ms |
  | Re-index (10 files) | < 2s | < 5s |
  | Diff computation | < 500ms | < 1s |
  | Blast radius (2 hop) | < 1s | < 3s |
  | Memory peak | < 500MB | < 1GB |

ACCEPTANCE CRITERIA:
  - [ ] All targets met
  - [ ] OR acceptable thresholds met with warning logged
  - [ ] No OOM errors
  - [ ] No infinite loops in blast radius
```

---

## Part 7: Data Structure Specifications

### SPEC-DATA-001: Diff Result Structure

```
SPECIFICATION: Diff Result Data Types

Rust Structures:

/// Result of comparing two database snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    /// Entities present only in live database
    pub added: Vec<DiffEntity>,

    /// Entities present only in base database
    pub removed: Vec<DiffEntity>,

    /// Entities with same stable_id but different content/lines
    pub modified: Vec<ModifiedPair>,

    /// Entities that moved within same file (line numbers changed)
    pub moved: Vec<MovedPair>,

    /// Entities that moved to different file
    pub relocated: Vec<RelocatedPair>,

    /// Summary statistics
    pub summary: DiffSummary,

    /// Timestamp of computation
    pub computed_at: DateTime<Utc>,

    /// Base database info
    pub base_info: DatabaseInfo,

    /// Live database info
    pub live_info: DatabaseInfo,
}

/// Entity with both full key and stable identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEntity {
    /// Full key with line numbers
    pub key: String,

    /// Stable identity without line numbers
    pub stable_id: String,

    /// File path
    pub file_path: String,

    /// Entity type (fn, struct, impl, etc.)
    pub entity_type: String,

    /// Language
    pub language: String,

    /// Line range
    pub line_start: u32,
    pub line_end: u32,
}

/// A pair of entities representing a modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifiedPair {
    pub stable_id: String,
    pub base: DiffEntity,
    pub live: DiffEntity,
    pub change_details: ChangeDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeDetails {
    LinesChanged { added: u32, removed: u32 },
    ContentChanged { signature_changed: bool },
    Unknown,
}

TypeScript Interfaces (for frontend):

interface DiffResult {
  added: DiffEntity[];
  removed: DiffEntity[];
  modified: ModifiedPair[];
  moved: MovedPair[];
  relocated: RelocatedPair[];
  summary: DiffSummary;
  computedAt: string;
  baseInfo: DatabaseInfo;
  liveInfo: DatabaseInfo;
}

interface DiffEntity {
  key: string;
  stableId: string;
  filePath: string;
  entityType: string;
  language: string;
  lineStart: number;
  lineEnd: number;
}

ACCEPTANCE CRITERIA:
  - [ ] Rust structs compile
  - [ ] TypeScript interfaces compile
  - [ ] JSON serialization round-trips correctly
  - [ ] All fields documented
```

---

## Appendix A: API Queries Used for Specification

The following API queries were used to ground these specifications:

```bash
# Codebase statistics
GET /codebase-statistics-overview-summary
Response: 215 entities, 2880 edges, 1 language (Rust)

# Entity key format examples
GET /code-entities-list-all
Response: Keys like "rust:fn:name:__path:start-end"

# Cluster structure (for visualization layout)
GET /semantic-cluster-grouping-list
Response: 49 clusters, largest with 163 entities

# Blast radius API behavior
GET /blast-radius-impact-analysis?entity={key}&hops=1
Note: Requires exact key match including line numbers

# Reverse callers (for neighbor detection)
GET /reverse-callers-query-graph?entity={key}
Response: Shows caller -> callee edges
```

---

## Appendix B: Referenced Documents

1. **ADR_001_KEY_NORMALIZATION.md**: Defines stable identity extraction algorithm
2. **GAP_ANALYSIS_METHODOLOGY_EVOLUTION_20260122.md**: Identifies 12 gaps including key instability
3. **CLAUDE.md**: Project build commands and naming conventions

---

*Document Version: 1.0*
*Created: 2026-01-23*
*Specifications Count: 18*
*Test Cases Implied: 50+*

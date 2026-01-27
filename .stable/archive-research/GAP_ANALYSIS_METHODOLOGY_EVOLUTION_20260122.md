# Gap Analysis and Methodology Evolution
> Systematic review of diff visualization documentation (2026-01-22)

---

## Executive Summary

After reviewing all 4 design documents, I identified **12 gaps**, **8 inconsistencies**, and **15 opportunities for strengthening** the design-to-implementation pathway. The documentation shows strong conceptual thinking but lacks critical implementation details that would enable a developer to build without ambiguity.

**Overall Assessment**: 70% complete - excellent vision, needs executable specificity.

---

## 1. Gaps Identified

### Gap 1: Key Instability Problem Not Fully Solved

**Location**: UNIFIED_SERVER_DESIGN_RESEARCH (lines 22-54), RUBBER_DUCK_DEBUG_REPORT (lines 249-257)

**The Problem**: The design acknowledges that entity keys change when line numbers change:
```
rust:fn:handle_auth:__path:10-50  -->  rust:fn:handle_auth:__path:12-52
```

But the solution ("skip rename detection for MVP, just show added/removed") creates a **false positive explosion**:
- Any code added ABOVE a function shifts ALL line numbers below
- This makes EVERY function appear as "removed + added" even if unchanged
- A single blank line added at line 1 could show 215 entities as "changed"

**Proposed Solution**:
```
KEY NORMALIZATION STRATEGY
--------------------------
1. Parse key: {lang}:{type}:{name}:{path_hash}:{lines}
2. Extract "stable identity": {lang}:{type}:{name}:{path_hash}
3. Use stable identity for MATCHING (finding the same entity)
4. Use full key (with lines) for CHANGE DETECTION
5. If stable identity matches but lines differ:
   - Check if only lines changed (same file) -> MOVED, not CHANGED
   - Check if body hash differs -> MODIFIED
   - Check if path_hash differs -> RELOCATED
```

**Impact**: Without this, the MVP will be unusable for any real codebase.

---

### Gap 2: No Content Hashing Mechanism

**Location**: UNIFIED_SERVER_DESIGN_RESEARCH (lines 56-82)

**The Problem**: The schema shows `signature_hash` and `body_hash` fields:
```sql
signature_hash TEXT,         -- SHA256 of function signature
body_hash TEXT               -- SHA256 of function body
```

But the Parseltongue API **does not return these fields**. The API returns:
```json
{
  "key": "...",
  "file_path": "...",
  "entity_type": "fn",
  "entity_class": "CODE",
  "language": "rust"
}
```

**No hash fields are available.**

**Proposed Solution**:
```
Option A: Compute hashes during diff (expensive)
  - Read file from disk
  - Parse to extract entity span
  - Compute hash of source text
  - Store in diff-specific cache

Option B: Add hash fields to pt01 indexer (better)
  - Modify parseltongue-core to compute hashes during parse
  - Store in CozoDB entity relations
  - Extend /code-entities-list-all to return hashes

Option C: Line-based change detection (MVP pragmatic)
  - If line numbers changed significantly (>5 lines), mark as POTENTIALLY_MODIFIED
  - Let user inspect to confirm
  - Document limitation
```

**Recommendation**: Option C for MVP, Option B for v1.1.

---

### Gap 3: Two-Database Architecture Not Specified

**Location**: ARCHITECTURE_API_GROUNDED (lines 427-446)

**The Problem**: The design says "run two pt08 servers" but doesn't specify:
1. How to spawn/manage two server instances
2. Port allocation strategy (7777 for base, 7778 for live?)
3. How the unified server proxies to both
4. Database lifecycle (when to create/destroy live.db)

**Proposed Architecture**:
```
DUAL DATABASE MANAGEMENT
------------------------

pt09-unified-web-server
  |
  +-- WorkspaceManager
  |     |
  |     +-- base.db (CozoDB handle, read-only)
  |     +-- live.db (CozoDB handle, read-write)
  |
  +-- No separate pt08 servers needed!
      |
      +-- Import pt08's query functions directly
      +-- Execute against base.db or live.db as needed

File: workspace/dual_db.rs
```rust
pub struct DualDatabaseHandle {
    base: CozoDb,  // Snapshot at base commit
    live: CozoDb,  // Current working state
    base_commit: String,
}

impl DualDatabaseHandle {
    /// Query both databases and return diff
    pub fn compute_diff(&self) -> Result<DiffResult> {
        let base_entities = self.base.query(ENTITIES_QUERY)?;
        let live_entities = self.live.query(ENTITIES_QUERY)?;
        // ... diff logic
    }
}
```
```

---

### Gap 4: File Watcher -> Re-index Pipeline Missing

**Location**: UNIFIED_SERVER_DESIGN_RESEARCH (lines 477-481)

**The Problem**: Phase 1 lists "File watcher using `notify` crate" but doesn't specify:
1. What happens when a file changes?
2. How to re-index ONLY the changed file (not entire codebase)?
3. How to update live.db incrementally?
4. Debounce strategy for rapid saves

**Proposed Pipeline**:
```
FILE CHANGE PIPELINE
--------------------

1. notify crate detects change: src/auth.rs modified
   |
2. Debounce: wait 500ms, collect all changes
   |
3. For each changed file:
   |  a. Remove old entities from live.db WHERE file_path = changed_file
   |  b. Parse changed file with tree-sitter
   |  c. Insert new entities into live.db
   |  d. Re-compute edges (tricky - may need full re-index)
   |
4. Compute diff against base.db
   |
5. Push diff via WebSocket
```

**Edge Re-computation Problem**:
Removing a function may break edges FROM other files. Options:
- Full re-index on any change (slow but correct)
- Lazy edge repair on query (fast but inconsistent)
- Mark edges as "potentially stale" (compromise)

**Recommendation**: Full re-index for MVP (< 2 seconds for 215 entities).

---

### Gap 5: No Error Recovery Strategy

**Location**: All documents

**The Problem**: No documentation of:
- What happens if base.db is corrupted?
- What if git status check fails?
- What if WebSocket disconnects?
- What if re-indexing fails mid-way?

**Proposed Error States**:
```typescript
type WorkspaceState =
  | { status: 'healthy', watching: boolean }
  | { status: 'base_corrupt', message: string, action: 'delete_and_recreate' }
  | { status: 'live_stale', lastGoodTimestamp: Date, action: 'reindex' }
  | { status: 'git_unavailable', message: string, action: 'continue_without_git' }
  | { status: 'index_failed', file: string, error: string, action: 'skip_file' }
```

---

### Gap 6: External Entity Rendering Strategy Incomplete

**Location**: VISUALIZATION_RESEARCH (line 70), RUBBER_DUCK_DEBUG_REPORT (lines 193-205)

**The Problem**: 790 external references (e.g., `rust:fn:map:unknown:0-0`) are documented but:
- No visual design for how they appear in 3D graph
- No UX for filtering them on/off
- No cluster assignment strategy (they all end up in one mega-cluster?)

**Proposed Strategy**:
```
EXTERNAL ENTITY HANDLING
------------------------

1. IDENTIFICATION:
   isExternal = key.includes('unknown:0-0') || key.includes(':0-0')

2. VISUAL TREATMENT:
   - Size: 0.3x (smaller than ambient)
   - Color: #333333 (dark gray, almost invisible)
   - Opacity: 0.08 (barely visible unless zoomed)
   - Shape: Small cube (different from spheres for internal)
   - No label unless hovered

3. SPATIAL LAYOUT:
   - Create synthetic "External" cluster
   - Position at graph periphery (outer ring)
   - Collapse by default (single node representing all 790)
   - Expand on click to show individual externals

4. UI CONTROLS:
   [ ] Show external dependencies (default: off)
   When on: externals fade in at graph edge
```

---

### Gap 7: No Performance Benchmarks or SLOs

**Location**: VISUALIZATION_RESEARCH (lines 234-240)

**The Problem**: Targets listed but no measurement strategy:
```
| Metric | Target |
|--------|--------|
| Initial load (3000 nodes) | < 2 seconds |
| File change -> visual update | < 500ms |
```

No specification of:
- How to measure
- What to do if targets aren't met
- Degradation strategy for larger codebases

**Proposed SLOs**:
```
PERFORMANCE SLOs WITH MEASUREMENT
---------------------------------

| Metric | Target | Measurement | Fallback |
|--------|--------|-------------|----------|
| Initial load | < 2s | performance.now() on graph ready | Progressive load |
| Live update | < 500ms | WebSocket msg to render complete | Show "updating..." |
| 60 FPS | 16.6ms frame | requestAnimationFrame timing | Reduce node count |
| Memory | < 200MB | performance.memory | LOD simplification |

Degradation tiers:
- Tier 1 (< 500 nodes): Full quality
- Tier 2 (500-2000 nodes): Disable glow, reduce labels
- Tier 3 (2000-5000 nodes): GPU instancing, LOD, no labels
- Tier 4 (> 5000 nodes): Cluster aggregation only
```

---

### Gap 8: WebSocket Reconnection Not Specified

**Location**: UNIFIED_SERVER_DESIGN_RESEARCH (lines 325-343)

**The Problem**: WebSocket protocol defined but no handling for:
- Server restart (loses connection)
- Network blip
- Tab backgrounded (browser may throttle)

**Proposed Protocol**:
```javascript
// WebSocket reconnection strategy
const WS_RECONNECT = {
  initialDelay: 1000,
  maxDelay: 30000,
  backoffMultiplier: 2,
  maxRetries: 10
};

class ResilientWebSocket {
  connect() {
    this.ws = new WebSocket(url);
    this.ws.onclose = () => this.scheduleReconnect();
    this.ws.onopen = () => this.requestFullState(); // Re-sync after reconnect
  }

  requestFullState() {
    // After reconnect, request current diff state
    // Don't assume incremental updates are valid
    this.ws.send(JSON.stringify({ type: 'request_full_diff' }));
  }
}
```

---

### Gap 9: No "Base Update" Workflow Details

**Location**: UNIFIED_SERVER_DESIGN_RESEARCH (lines 264-267)

**The Problem**: "Click Update Base" is mentioned but:
- What exactly happens?
- Is live.db copied to base.db? Or re-indexed from HEAD?
- What if there are uncommitted changes?
- What if we're on a different branch?

**Proposed Workflow**:
```
UPDATE BASE WORKFLOW
--------------------

PRE-CONDITIONS:
- Git working directory is clean (no uncommitted changes)
- OR user confirms "Update base with uncommitted changes?"

STEPS:
1. Check git status
   - If dirty: Prompt "Include uncommitted changes in new base?"

2. If user confirms OR clean:
   a. current_commit = git rev-parse HEAD
   b. Delete old base.db
   c. Run full pt01 indexing -> new base.db
   d. Update workspace meta.json: base_commit = current_commit
   e. Diff now shows 0 changes (base == live)

3. Push WebSocket: { type: 'base_updated', new_commit: current_commit }
```

---

### Gap 10: Cluster Display Names Missing

**Location**: RUBBER_DUCK_DEBUG_REPORT (lines 262-266)

**The Problem**: Clusters use numeric IDs (`cluster_id: 1`) not semantic names. The design says "derive display names from entity patterns" but doesn't specify how.

**Proposed Algorithm**:
```rust
fn derive_cluster_display_name(cluster: &Cluster) -> String {
    // Strategy: Find common path prefix among entities
    let paths: Vec<_> = cluster.entities.iter()
        .filter_map(|key| extract_path_from_key(key))
        .collect();

    if paths.is_empty() {
        return format!("Cluster {}", cluster.cluster_id);
    }

    // Find longest common prefix
    let common_prefix = longest_common_prefix(&paths);

    // Extract meaningful name
    // e.g., "./crates/pt08-http-code-query-server/src/handlers/"
    //    -> "pt08 handlers"

    let segments: Vec<_> = common_prefix.split('/').collect();
    if let Some(crate_name) = segments.iter().find(|s| s.starts_with("pt")) {
        let module = segments.last().unwrap_or(&"");
        format!("{} {}", crate_name, module)
    } else {
        segments.last().unwrap_or(&"unknown").to_string()
    }
}
```

---

### Gap 11: No Accessibility Considerations

**Location**: VISUALIZATION_RESEARCH

**The Problem**: Color-only differentiation (green=added, red=removed) is inaccessible for colorblind users.

**Proposed Solution**:
```
ACCESSIBILITY-FIRST VISUAL DESIGN
---------------------------------

Use SHAPE + COLOR + MOTION:

| Status   | Color   | Shape    | Motion        | Label |
|----------|---------|----------|---------------|-------|
| Added    | #00ff88 | Sphere   | Pulse         | [+]   |
| Removed  | #ff4444 | Cube     | Fade out      | [-]   |
| Modified | #ffcc00 | Diamond  | Subtle pulse  | [~]   |
| Neighbor | #ffa94d | Sphere   | None          |       |
| Ambient  | #888888 | Point    | None          |       |

Additionally:
- High contrast mode toggle
- Shape legend always visible
- Screen reader: List changed entities in sidebar
```

---

### Gap 12: No Testing Strategy for Diff Engine

**Location**: ARCHITECTURE_API_GROUNDED

**The Problem**: No specification of how to test the diff algorithm. What are the test cases?

**Proposed Test Suite**:
```rust
#[cfg(test)]
mod diff_tests {
    // Test: Empty base, entities added
    #[test]
    fn test_diff_empty_base_to_populated() {
        let base = vec![];
        let live = vec![entity("rust:fn:main:path:1-10")];
        let diff = compute_diff(&base, &live);
        assert_eq!(diff.added.len(), 1);
        assert!(diff.removed.is_empty());
    }

    // Test: Same entities, different line numbers (MOVED)
    #[test]
    fn test_diff_entity_moved() {
        let base = vec![entity("rust:fn:main:path:1-10")];
        let live = vec![entity("rust:fn:main:path:5-14")]; // shifted 4 lines
        let diff = compute_diff(&base, &live);
        assert_eq!(diff.moved.len(), 1);
        assert!(diff.added.is_empty());
    }

    // Test: Entity genuinely removed
    #[test]
    fn test_diff_entity_removed() {
        let base = vec![entity("rust:fn:helper:path:20-30")];
        let live = vec![];
        let diff = compute_diff(&base, &live);
        assert_eq!(diff.removed.len(), 1);
    }

    // Test: External entities unchanged
    #[test]
    fn test_diff_externals_ignored() {
        let base = vec![entity("rust:fn:map:unknown:0-0")];
        let live = vec![entity("rust:fn:map:unknown:0-0")];
        let diff = compute_diff(&base, &live);
        assert!(diff.is_empty());
    }

    // Test: Edge changes
    #[test]
    fn test_diff_edge_added() {
        let base_edges = vec![];
        let live_edges = vec![edge("A", "B", "Calls")];
        let diff = compute_edge_diff(&base_edges, &live_edges);
        assert_eq!(diff.added.len(), 1);
    }
}
```

---

## 2. Inconsistencies Found

| # | Document A | Document B | Inconsistency |
|---|-----------|-----------|---------------|
| 1 | UNIFIED (line 430) | ARCHITECTURE (line 397) | Different crate structure (pt10 in one, nested in pt09 in other) |
| 2 | UNIFIED (line 171) | ARCHITECTURE (line 82) | Cluster count: meta.json says nothing about clusters, but needed for layout |
| 3 | RUBBER_DUCK (line 149) | UNIFIED (line 83) | Statistics endpoint: "external references" vs "external_references_total_count" |
| 4 | VISUALIZATION (line 53) | ARCHITECTURE (line 289) | External color: #4a4a4a vs "dark gray" |
| 5 | UNIFIED (line 352) | ARCHITECTURE (line 270) | Node status field: has "change" subfield in one, not the other |
| 6 | RUBBER_DUCK (line 105) | ARCHITECTURE (line 211) | Blast radius: says it fails for leaf nodes but architecture relies on it |
| 7 | UNIFIED (line 326) | ARCHITECTURE (line 250) | WebSocket message type: "diff_updated" vs assumed incremental |
| 8 | All docs | - | No agreed-upon TypeScript interface definitions |

---

## 3. Recommended Document Structure Evolution

The current 4-document structure has overlap and gaps. I recommend evolving to:

```
docs/
├── 01_REQUIREMENTS.md           # What we're building (user stories, acceptance criteria)
├── 02_ARCHITECTURE.md           # How it fits together (single source of truth)
├── 03_API_SPECIFICATION.md      # Exact endpoints, request/response shapes
├── 04_DATA_STRUCTURES.md        # TypeScript interfaces, Rust structs
├── 05_IMPLEMENTATION_GUIDE.md   # Step-by-step build order
├── 06_VISUAL_DESIGN.md          # Colors, animations, layouts
├── 07_TESTING_STRATEGY.md       # Test cases, fixtures, coverage targets
└── 08_VALIDATION_LOG.md         # Rubber duck sessions, API tests
```

**Key Principle**: Each document answers ONE question:
- 01: WHAT are we building?
- 02: HOW does it fit together?
- 03: WHAT is the API contract?
- 04: WHAT are the data shapes?
- 05: IN WHAT ORDER do we build?
- 06: HOW does it look/feel?
- 07: HOW do we verify correctness?
- 08: WHAT did we learn from validation?

---

## 4. Immediate Action Items

### Priority 1: Fix Key Instability (Blocks MVP)
```
Create: docs/ADR_001_KEY_NORMALIZATION.md
- Document the stable identity extraction algorithm
- Add test cases for edge cases (rename, move, split)
- Update ARCHITECTURE to reference ADR
```

### Priority 2: Define Exact Data Structures
```
Create: docs/04_DATA_STRUCTURES.md
- Single TypeScript interface file
- Matching Rust structs with serde
- Examples for every type
```

### Priority 3: Specify File Watcher Pipeline
```
Update: UNIFIED_SERVER_DESIGN_RESEARCH.md
- Add "File Change Pipeline" section
- Document incremental vs full re-index decision
- Add sequence diagram
```

### Priority 4: Add Test Cases
```
Create: docs/07_TESTING_STRATEGY.md
- Unit tests for diff algorithm
- Integration tests for watch pipeline
- Visual regression tests for 3D graph
```

---

## 5. New Insights and Methodology Refinements

### Insight 1: "Diff is the Product" Needs Refinement

The core thesis "DIFF IS THE PRODUCT" is correct, but the diff must be **semantically meaningful**, not just syntactic. Showing 215 entities as "changed" when only 1 function was modified defeats the purpose.

**Refinement**: "MEANINGFUL DIFF IS THE PRODUCT"
- Filter out line-number-only changes
- Group related changes (function + its tests)
- Highlight actual semantic changes

### Insight 2: Start with CLI Diff, Not WebSocket

The design jumps to live WebSocket updates, but a simpler MVP:
```bash
parseltongue diff --base HEAD~1 --live .
```
This CLI-first approach:
- Tests diff algorithm without WebSocket complexity
- Provides immediate value
- Can be wrapped in watch mode later

### Insight 3: External Entities Are Graph Periphery, Not First-Class

The 790 external entities should be treated as "graph periphery":
- Not counted in entity totals
- Not shown by default
- Treated as edge targets only

### Insight 4: Use Git Commit as Natural "Base" Boundary

Instead of arbitrary "update base" button, consider:
- Base = last commit on current branch
- Auto-update base when git detects new commit
- Show "uncommitted changes" as live diff

This aligns with actual developer mental model.

### Insight 5: Cluster-Based Spatial Layout is Key Differentiator

The cluster visualization is where Parseltongue can truly differentiate:
- CodeCity shows files/folders
- Gource shows time
- **Parseltongue shows semantic structure**

Double down on cluster visualization:
- Cluster = 3D region with boundary
- Cross-cluster edges are "long-distance" visually
- Cluster selection zooms into subgraph

---

## 6. Summary of Deliverables from This Analysis

| File | Status | Purpose |
|------|--------|---------|
| `GAP_ANALYSIS_METHODOLOGY_EVOLUTION_20260122.md` | Created | This document |
| `ADR_001_KEY_NORMALIZATION.md` | Recommended | Solve key instability |
| `04_DATA_STRUCTURES.md` | Recommended | Single source of truth for types |
| `07_TESTING_STRATEGY.md` | Recommended | Test cases for diff |

---

## 7. Conclusion

The documentation represents strong conceptual work but needs:
1. **Key normalization** to prevent false positive explosion
2. **Exact data structures** in TypeScript and Rust
3. **Test cases** for the diff algorithm
4. **Error handling** specification
5. **Accessibility** considerations

The core approach (diff by key, use existing APIs, add 4 new endpoints) remains valid. The refinements above will make it buildable.

---

*Analysis completed: 2026-01-22*
*Documents reviewed: 4*
*Gaps identified: 12*
*Inconsistencies found: 8*
*Recommendations: 15*

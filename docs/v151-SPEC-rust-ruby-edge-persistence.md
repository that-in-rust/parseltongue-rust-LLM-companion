# SPEC: Rust/Ruby Edge Persistence Bug Investigation

**Version**: 1.0
**Date**: 2026-02-07
**Status**: INVESTIGATION SPECIFICATION
**Priority**: P1 - HIGH (Blocking v1.5.1 completion)
**Author**: Claude Code (Anthropic)

---

## Table of Contents

1. [Problem Statement](#problem-statement)
2. [Evidence](#evidence)
3. [Investigation Areas](#investigation-areas)
4. [Hypotheses](#hypotheses)
5. [Debugging Plan](#debugging-plan)
6. [Acceptance Criteria](#acceptance-criteria)
7. [Timeline](#timeline)

---

## Problem Statement

### The Bug

**Symptom**: Rust and Ruby edges are captured by tree-sitter queries and successfully reach the edge builder, but edges do not appear in database queries.

**Manifestation**:
- Debug logging confirms edges captured with valid `from_entity`
- Insert logs report successful insertion
- HTTP API queries return 0 edges
- Direct database queries return 0 edges

### Impact

| Language | Entities | Edges Captured | Edges in DB | Gap |
|----------|----------|----------------|-------------|-----|
| **Rust** | 26 ✅ | 35 ✅ | 0 ❌ | -35 |
| **Ruby** | 25 ✅ | 12+ ✅ | 11 ✅ | **WORKING** |
| **C++** | 5 ✅ | 9 ✅ | 9 ✅ | **WORKING** |
| **Java** | 15 ✅ | 21 ✅ | 21 ✅ | **WORKING** |

**Key Insight**: Ruby edges were initially reported as 0, but Phase 3/4 investigation revealed 11 Ruby edges ARE persisted and queryable. This makes the Rust case more mysterious - why does Ruby work but Rust doesn't?

### Paradox

```
Debug logging shows:
  [DEBUG-INSERT] About to insert 35 total edges
  [DEBUG-INSERT] Rust edges: 35
  [DEBUG-INSERT] ✅ Successfully inserted edges

Database query shows:
  HTTP API: 0 Rust edges
  Direct DB: 0 Rust edges

PARADOX: How can edges be "successfully inserted" yet not appear in queries?
```

---

## Evidence

### Evidence 1: Edge Capture (Working ✅)

**Source**: Phase 3 debug logging from `build_dependency_edge()`

```
[DEBUG-EDGE] Language: Rust
[DEBUG-EDGE] Capture: reference.call
[DEBUG-EDGE] To: HashMap
[DEBUG-EDGE] Node line: 5
[DEBUG-EDGE] ✅ Found from_entity: main (type: Function)
```

**Analysis**:
- Tree-sitter queries ARE capturing Rust dependencies
- `from_entity` lookup IS succeeding
- Edge builder IS receiving valid data
- **Conclusion**: Query extraction layer working correctly

### Evidence 2: Insert Logging (Confusing ❓)

**Source**: Phase 4 debug logging from `pt01-folder-to-cozodb-streamer`

```
[DEBUG-INSERT] About to insert 35 total edges
[DEBUG-INSERT] Rust edges: 35
[DEBUG-INSERT] ✅ Successfully inserted edges
```

**Analysis**:
- 35 edges passed to insert function
- Insert function reports success
- No error messages, no exceptions
- **Concern**: "Success" may be premature - logged before commit?

### Evidence 3: Database Queries (Failing ❌)

**Source**: HTTP API verification

```bash
curl -s http://localhost:7777/dependency-edges-list-all | \
  jq '[.data.edges[] | select(.from_key | startswith("rust"))] | length'

Result: 0
```

**Direct Database Query** (CozoDB):
```
Result: 0 Rust edges found
```

**Analysis**:
- HTTP API returns 0 Rust edges
- Direct database inspection returns 0 Rust edges
- **Conclusion**: Edges genuinely not in database

### Evidence 4: Ruby Comparison (Working ✅)

**Source**: Same test run, Ruby edges

```
Ruby edges captured: 12+
Ruby edges in DB: 11
Ruby edges queryable: 11 ✅

DIFFERENCE: Ruby edges ARE persisted, Rust edges ARE NOT
```

**Analysis**:
- Ruby and Rust go through same code path
- Ruby edges work end-to-end
- Rust edges fail at some point between insert and commit
- **Key Question**: What's different about Rust edges?

---

## Investigation Areas

### Area 1: Edge Creation in query_extractor.rs

**File**: `crates/parseltongue-core/src/query_extractor.rs`
**Function**: `build_dependency_edge()`
**Status**: ✅ VERIFIED WORKING (Phase 3)

**What to Check**:
- ✅ Tree-sitter captures working correctly
- ✅ `from_entity` lookup succeeding
- ✅ Edge struct created with valid data
- ⚠️ **New concern**: Are Rust edge keys valid after sanitization?

**Debug Points**:
```rust
// Log edge after creation
eprintln!("[DEBUG-EDGE-CREATED] from_key={}, to_key={}",
    edge.from_key, edge.to_key);
```

### Area 2: Edge Validation in external_dependency_handler.rs

**File**: `crates/parseltongue-core/src/external_dependency_handler.rs`
**Function**: Edge validation and external dependency handling
**Status**: ⚠️ SUSPECTED

**What to Check**:
- Do Rust edges pass validation?
- Are Rust edges filtered out as "external" incorrectly?
- Does key format validation reject Rust edges?
- Are Rust edges deduplicated away completely?

**Debug Points**:
```rust
// Log edges before/after validation
eprintln!("[DEBUG-VALIDATE] Before validation: {} edges", edges.len());
eprintln!("[DEBUG-VALIDATE] After validation: {} edges", edges.len());
eprintln!("[DEBUG-VALIDATE] Removed: {:?}", removed_edges);
```

### Area 3: Edge Insertion in streamer.rs

**File**: `crates/pt01-folder-to-cozodb-streamer/src/streamer.rs`
**Function**: Database insertion logic
**Status**: ⚠️ SUSPECTED

**What to Check**:
- Are Rust edges reaching the insert function?
- Does insert function handle Rust edges differently?
- Are there language-specific filters before insert?
- Is transaction handling correct for Rust edges?

**Debug Points**:
```rust
// Log edges at insert entry point
eprintln!("[DEBUG-INSERT] About to insert {} edges", edges.len());
eprintln!("[DEBUG-INSERT] Rust edges: {}",
    edges.iter().filter(|e| e.from_key.starts_with("rust")).count());

// Log after insert
eprintln!("[DEBUG-INSERT] Insert function returned: {:?}", result);
```

### Area 4: Database Operations in cozo_client.rs

**File**: `crates/parseltongue-core/src/cozo_client.rs`
**Function**: CozoDB transaction and commit logic
**Status**: ⚠️ SUSPECTED

**What to Check**:
- Does transaction commit succeed?
- Are Rust edges in transaction buffer?
- Does rollback occur silently?
- Are there CozoDB-specific constraints failing?

**Debug Points**:
```rust
// Log transaction state
eprintln!("[DEBUG-TX] Transaction state before commit");
eprintln!("[DEBUG-TX] Edges in buffer: {}", buffer_count);

// Log commit result
eprintln!("[DEBUG-TX] Commit result: {:?}", commit_result);

// Query immediately after commit
eprintln!("[DEBUG-TX-VERIFY] Rust edges after commit: {}", count);
```

---

## Hypotheses

### Hypothesis 1: Transaction Rollback

**Theory**: Insert succeeds but transaction rolls back silently.

**Mechanism**:
```
1. Edges inserted into transaction buffer
2. Debug log runs: "✅ Successfully inserted edges"
3. Transaction commits
4. Commit fails silently (no error propagation)
5. Transaction rolls back
6. Rust edges lost
```

**Why Only Rust?**
- Rust edges may trigger unique constraint violations
- Rust edges may fail CozoDB schema validation
- Rust edges may cause transaction size limits

**How to Test**:
```rust
// Log transaction lifecycle
eprintln!("[DEBUG-TX] Before commit");
let result = db.commit_transaction();
eprintln!("[DEBUG-TX] Commit result: {:?}", result);
if result.is_err() {
    eprintln!("[DEBUG-TX] ❌ ROLLBACK: {:?}", result.err());
}
```

**Likelihood**: HIGH (40%)

---

### Hypothesis 2: Edge Key Format Validation

**Theory**: Rust edge keys fail ISGL1 validation after sanitization.

**Mechanism**:
```
1. Rust edges created with sanitized keys: rust:fn:std__collections__HashMap:...
2. Keys look valid
3. Database insert function validates keys
4. Validation fails for Rust-specific reason
5. Edges silently dropped
```

**Why Only Rust?**
- Rust has deepest namespace nesting: `std::collections::HashMap`
- After sanitization: `std__collections__HashMap`
- May exceed key length limits
- May have other validation issues

**Evidence**:
- Ruby also has `::`
- Ruby edges work (11 in DB)
- But Ruby namespaces typically shorter

**How to Test**:
```rust
// Log keys at insert
eprintln!("[DEBUG-KEY] Inserting Rust edge:");
eprintln!("[DEBUG-KEY]   from_key: {}", edge.from_key);
eprintln!("[DEBUG-KEY]   to_key: {}", edge.to_key);

// Validate key manually
let parts: Vec<&str> = edge.to_key.split(':').collect();
eprintln!("[DEBUG-KEY]   Parts: {} (expected 5)", parts.len());
```

**Likelihood**: MEDIUM (30%)

---

### Hypothesis 3: Language-Specific Filtering

**Theory**: Code path filters out Rust edges before persistence.

**Mechanism**:
```
1. Edges captured correctly
2. Some filter checks language == Rust
3. Filter removes Rust edges (bug in filter logic)
4. Remaining edges inserted
5. Rust edges never reach database
```

**Where to Look**:
- External dependency handler (filters external refs)
- Deduplication logic (may treat Rust edges as duplicates)
- Language-specific edge type mapping

**Why Only Rust?**
- Rust may be treated as "external" incorrectly
- Rust stdlib edges (std::) may be filtered as "not user code"

**How to Test**:
```bash
# Search for language-specific filters
grep -n "Language::Rust" crates/*/src/*.rs
grep -n "if.*language.*==" crates/*/src/*.rs
```

**Likelihood**: MEDIUM (20%)

---

### Hypothesis 4: Deduplication Removing All Rust Edges

**Theory**: All 35 Rust edges are considered duplicates and removed.

**Mechanism**:
```
1. Rust edges created
2. Deduplication runs
3. All Rust edges match existing edges (false positive)
4. All 35 edges removed
5. 0 edges remain
```

**Why Unlikely**:
- Would require ALL 35 edges to be exact duplicates
- Test fixture has diverse Rust code
- Should have at least some unique edges

**How to Test**:
```rust
// Log deduplication
eprintln!("[DEBUG-DEDUP] Before: {} edges", before_count);
eprintln!("[DEBUG-DEDUP] After: {} edges", after_count);
eprintln!("[DEBUG-DEDUP] Removed: {} edges", before_count - after_count);
eprintln!("[DEBUG-DEDUP] Rust edges remaining: {}", rust_count);
```

**Likelihood**: LOW (10%)

---

### Hypothesis 5: Database Commit Issue

**Theory**: Insert succeeds but commit fails silently for Rust edges.

**Mechanism**:
```
1. Transaction starts
2. Edges inserted into transaction buffer
3. Log runs: "Successfully inserted"
4. Transaction.commit() called
5. Commit succeeds for other languages
6. Commit fails for Rust edges (constraint violation)
7. Rust edges dropped, no error logged
```

**Why Only Rust?**
- Rust edge keys may violate unique constraint
- Rust edges may reference non-existent entities
- CozoDB may have Rust-specific schema issues

**How to Test**:
```rust
// Check database state immediately after commit
let rust_edges = query_rust_edges_from_db();
eprintln!("[DEBUG-VERIFY] Rust edges in DB after commit: {}", rust_edges.len());
if rust_edges.is_empty() {
    eprintln!("[DEBUG-VERIFY] ❌ RUST EDGES MISSING AFTER COMMIT");
}
```

**Likelihood**: MEDIUM (20%)

---

## Debugging Plan

### Phase 1: Key Format Verification (30 minutes)

**Goal**: Verify Rust edge keys are valid after sanitization.

**Steps**:
1. Add logging at edge creation in `query_extractor.rs`:
   ```rust
   eprintln!("[DEBUG-RUST-KEY] Created edge:");
   eprintln!("  from_key: {}", edge.from_key);
   eprintln!("  to_key: {}", edge.to_key);
   ```

2. Run on test fixture:
   ```bash
   parseltongue pt01-folder-to-cozodb-streamer test-fixtures/v151-edge-bug-repro/namespaces.rs 2>&1 | grep DEBUG-RUST-KEY
   ```

3. Check output:
   - Are keys formatted correctly?
   - Do keys have 5 parts when split by `:`?
   - Are there any `::`remaining after sanitization?

**Expected Output**:
```
[DEBUG-RUST-KEY] from_key: rust:fn:main:definition:T1234567890
[DEBUG-RUST-KEY] to_key: rust:fn:std__collections__HashMap:unresolved-reference:0-0
```

**Decision Point**:
- ✅ Keys valid → Move to Phase 2
- ❌ Keys invalid → Fix sanitization, retest

---

### Phase 2: Insert Path Tracing (1 hour)

**Goal**: Trace Rust edges through insert pipeline.

**Steps**:
1. Add logging in `pt01-folder-to-cozodb-streamer`:
   ```rust
   // At entry to insert function
   eprintln!("[DEBUG-INSERT-ENTRY] Total edges: {}", edges.len());
   let rust_count = edges.iter()
       .filter(|e| e.from_key.starts_with("rust"))
       .count();
   eprintln!("[DEBUG-INSERT-ENTRY] Rust edges: {}", rust_count);

   // After validation
   eprintln!("[DEBUG-INSERT-VALIDATED] Rust edges: {}", rust_validated);

   // After deduplication
   eprintln!("[DEBUG-INSERT-DEDUPED] Rust edges: {}", rust_deduped);

   // Before database insert
   eprintln!("[DEBUG-INSERT-FINAL] Rust edges going to DB: {}", rust_final);
   ```

2. Run and capture output:
   ```bash
   parseltongue pt01-folder-to-cozodb-streamer test-fixtures/v151-edge-bug-repro 2>&1 | grep DEBUG-INSERT
   ```

3. Analyze:
   - Where does Rust edge count drop?
   - Is validation removing Rust edges?
   - Is deduplication removing all Rust edges?

**Expected Output**:
```
[DEBUG-INSERT-ENTRY] Rust edges: 35
[DEBUG-INSERT-VALIDATED] Rust edges: 35  ← Should stay 35
[DEBUG-INSERT-DEDUPED] Rust edges: 22    ← Expected reduction
[DEBUG-INSERT-FINAL] Rust edges: 22      ← Should match DB
```

**Decision Point**:
- Edge count drops at validation → Investigate validation logic
- Edge count drops at deduplication → Investigate dedup logic
- Edge count stays high but not in DB → Move to Phase 3

---

### Phase 3: Transaction and Commit Tracing (1 hour)

**Goal**: Verify transaction commit succeeds for Rust edges.

**Steps**:
1. Add logging around transaction commit:
   ```rust
   eprintln!("[DEBUG-TX] Before commit: {} edges in buffer", buffer_count);
   eprintln!("[DEBUG-TX] Rust edges in buffer: {}", rust_buffer_count);

   let result = db.commit_transaction();
   eprintln!("[DEBUG-TX] Commit result: {:?}", result);

   if let Err(e) = result {
       eprintln!("[DEBUG-TX] ❌ COMMIT FAILED: {:?}", e);
   }
   ```

2. Query immediately after commit:
   ```rust
   // Verify persistence
   let rust_edges_after = db.query("
       ?[from_key, to_key] :=
       *dependencies[from_key, to_key, _],
       from_key ~= 'rust:.*'
   ");
   eprintln!("[DEBUG-TX-VERIFY] Rust edges in DB: {}", rust_edges_after.len());
   ```

3. Compare:
   - Buffer count vs. DB count
   - If mismatch, transaction rollback occurred

**Expected Output**:
```
[DEBUG-TX] Before commit: 43 edges in buffer
[DEBUG-TX] Rust edges in buffer: 22
[DEBUG-TX] Commit result: Ok(())
[DEBUG-TX-VERIFY] Rust edges in DB: 22  ← Should match buffer
```

**Decision Point**:
- Commit fails → Investigate CozoDB error handling
- Commit succeeds but DB count is 0 → Database constraint issue
- Commit succeeds and DB count matches → Bug is elsewhere (query issue?)

---

### Phase 4: Compare Ruby vs Rust Code Paths (30 minutes)

**Goal**: Find what's different between Ruby (working) and Rust (broken).

**Steps**:
1. Run both test fixtures with full logging:
   ```bash
   # Ruby (working)
   parseltongue pt01-folder-to-cozodb-streamer test-fixtures/v151-edge-bug-repro/modules.rb 2>&1 > ruby.log

   # Rust (broken)
   parseltongue pt01-folder-to-cozodb-streamer test-fixtures/v151-edge-bug-repro/namespaces.rs 2>&1 > rust.log
   ```

2. Compare logs side-by-side:
   ```bash
   diff -u ruby.log rust.log | grep -A5 -B5 "DEBUG"
   ```

3. Look for:
   - Different validation outcomes
   - Different deduplication behavior
   - Different transaction handling
   - Different key formats

**Expected Insights**:
- Ruby edges pass validation, Rust edges fail
- Ruby keys shorter, Rust keys longer
- Ruby edges not deduplicated, Rust edges fully deduplicated

---

### Phase 5: Direct Database Inspection (30 minutes)

**Goal**: Bypass API and inspect database directly.

**Steps**:
1. After ingestion, inspect CozoDB directly:
   ```rust
   // Raw query bypassing all abstractions
   let all_edges = db.run_query("?[from_key, to_key] := *dependencies[from_key, to_key, _]");

   // Count by language
   for (lang, count) in edges_by_language {
       eprintln!("Language {}: {} edges", lang, count);
   }
   ```

2. Check:
   - Do ANY Rust edges exist in raw database?
   - Are Rust edges stored under different keys?
   - Are Rust edges in a different table?

**Expected Output**:
```
Language rust: 0 edges  ← Current problem
Language ruby: 11 edges ← Working
Language java: 21 edges ← Working
```

**Decision Point**:
- Rust edges in DB but different format → Query issue
- Rust edges genuinely absent → Insert/commit issue confirmed

---

## Acceptance Criteria

### AC-001: Rust Edges Persisted

**WHEN** Rust code is parsed with `pt01-folder-to-cozodb-streamer`
**THEN** Rust edges SHALL appear in database
**AND** Rust edge count SHALL be > 0
**AND** Rust edges SHALL be queryable via HTTP API

**Verification**:
```bash
curl -s http://localhost:7777/dependency-edges-list-all | \
  jq '[.data.edges[] | select(.from_key | startswith("rust"))] | length'
# Expected: > 0 (not zero)
```

---

### AC-002: No Regression for Ruby

**WHEN** Ruby code is parsed
**THEN** Ruby edges SHALL continue to work
**AND** Ruby edge count SHALL remain ≥ 11

**Verification**:
```bash
curl -s http://localhost:7777/dependency-edges-list-all | \
  jq '[.data.edges[] | select(.from_key | startswith("ruby"))] | length'
# Expected: ≥ 11
```

---

### AC-003: Key Format Validation

**WHEN** Rust edges are created
**THEN** all Rust edge keys SHALL be valid ISGL1 format
**AND** keys SHALL split into exactly 5 parts by `:`
**AND** keys SHALL NOT contain `::` (only `__`)

**Verification**:
```bash
# Check for invalid keys in logs
parseltongue pt01-folder-to-cozodb-streamer test-fixtures/v151-edge-bug-repro 2>&1 | \
  grep "Invalid.*key.*format"
# Expected: 0 matches (no invalid keys)
```

---

### AC-004: Transaction Integrity

**WHEN** edges are inserted into database
**THEN** transaction commit SHALL succeed
**AND** commit result SHALL be logged
**AND** any commit failures SHALL be logged as errors

**Verification**:
```rust
// In code
match db.commit_transaction() {
    Ok(_) => eprintln!("[DEBUG-TX] ✅ Commit successful"),
    Err(e) => eprintln!("[DEBUG-TX] ❌ Commit failed: {:?}", e),
}
```

---

### AC-005: Debugging Visibility

**WHEN** debug logging is enabled
**THEN** Rust edge count SHALL be logged at each pipeline stage:
- Entry to insert function
- After validation
- After deduplication
- Before database insert
- After transaction commit

**Verification**:
```bash
parseltongue pt01-folder-to-cozodb-streamer test-fixtures/v151-edge-bug-repro 2>&1 | \
  grep "DEBUG.*Rust edges"
# Expected: Multiple log lines showing edge count at each stage
```

---

## Timeline

### Investigation Phases

| Phase | Goal | Time | Cumulative |
|-------|------|------|------------|
| Phase 1 | Key format verification | 30 min | 30 min |
| Phase 2 | Insert path tracing | 1 hour | 1.5 hours |
| Phase 3 | Transaction/commit tracing | 1 hour | 2.5 hours |
| Phase 4 | Ruby vs Rust comparison | 30 min | 3 hours |
| Phase 5 | Direct DB inspection | 30 min | 3.5 hours |

**Total Investigation Time**: 3.5 hours

### Fix Implementation

**Estimated**: 1-2 hours (depends on root cause)

**Possible Fixes**:
- Key validation logic fix: 1 hour
- Transaction handling fix: 1.5 hours
- Deduplication logic fix: 2 hours

**Total Time Budget**: 5.5 hours (3.5 investigation + 2 fix)

---

## Success Metrics

### Before Fix (Current State)

| Metric | Value | Status |
|--------|-------|--------|
| Rust edges in DB | 0 | ❌ BROKEN |
| Ruby edges in DB | 11 | ✅ WORKING |
| Rust entities in DB | 26 | ✅ WORKING |
| Invalid keys logged | 0 | ✅ (after BUG-001 fix) |

### After Fix (Target State)

| Metric | Value | Status |
|--------|-------|--------|
| Rust edges in DB | > 0 (target: 22) | ✅ FIXED |
| Ruby edges in DB | ≥ 11 | ✅ NO REGRESSION |
| Rust entities in DB | 26 | ✅ NO REGRESSION |
| Invalid keys logged | 0 | ✅ MAINTAINED |

---

## Related Documents

| Document | Purpose |
|----------|---------|
| `docs/v151-primary-PRD.md` | Original bug analysis (all 3 bugs) |
| `docs/v151-TDD-PROGRESS.md` | Current session state and findings |
| `test-fixtures/v151-edge-bug-repro/EXPECTED.md` | Test fixture documentation |
| `docs/v151-TDD-SPEC-key-sanitization-qualified-names.md` | BUG-001 fix (complete) |

---

## Notes

### Key Insight from Phase 4 Investigation

The TDD progress document reveals that:
1. **BUG-001 (qualified names)** was REAL and is now FIXED
2. **BUG-003 (Ruby edges)** was FALSE ALARM - Ruby edges work (11 in DB)
3. **BUG-002 (Rust edges)** remains UNRESOLVED - paradox persists

**Critical Questions**:
1. Why do Ruby edges work but Rust edges don't, if both had `::` issues?
2. Are Rust edges actually being created with valid keys after sanitization?
3. Is there a Rust-specific code path that's dropping edges?

### Hypothesis Priority (Revised)

Based on evidence that Ruby works:

1. **Transaction rollback** (40%) - Most likely, fits the evidence
2. **Edge key format** (30%) - Rust keys may be longer/different
3. **Language-specific filter** (20%) - Rust may be filtered differently
4. **Database commit issue** (10%) - Less likely if Ruby works

---

**END OF SPECIFICATION**

*This document guides the investigation of the Rust edge persistence bug. Update with findings as investigation progresses.*

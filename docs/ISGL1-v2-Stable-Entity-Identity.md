# ISGL1 v2: Stable Entity Identity Architecture

> **Status**: Architecture Design Complete
> **Version**: 2.0
> **Date**: 2026-02-02
> **Author**: Research synthesis from ADR-001, D04, and implementation analysis

---

## Executive Summary (Minto Pyramid Top)

**The Answer**: ISGL1 v2 must replace line-number-based keys with birth-timestamp-based keys to enable reliable incremental indexing. Implementation requires 3-4 days using TDD approach with 23 comprehensive tests.

**Critical Decision**: This is Priority #1 foundational work, blocking all feature workflows until stable entity identity is achieved.

**Format Change**:
- **ISGL1 v1** (unstable): `rust:fn:handle_auth:__src_auth_rs:10-50`
- **ISGL1 v2** (stable): `rust:fn:handle_auth:__src_auth_rs:T1706284800`

**Migration**: Breaking change requiring full re-index (acceptable trade-off for stability).

---

## Level 1: Supporting Arguments

### 1. WHY: The Fundamental Problem

**Observation**: Adding/removing code above an entity shifts ALL subsequent line numbers, causing cascading key changes across the entire file.

**Impact**:
- Every entity below the change gets a NEW key
- ALL dependency edges pointing to shifted entities break
- Diff shows false positives (3 "removed" + 3 "added" when only 1 actually changed)
- Incremental indexing produces unreliable results

**Consequence**: Current ISGL1 v1 makes incremental indexing **fundamentally broken** for production use.

### 2. WHAT: The Solution Architecture

**Core Insight**: Entity identity must be **immutable at birth**, not dependent on **mutable position**.

**Three-Component Solution**:
1. **Birth Timestamp** (Unix epoch seconds): Permanent unique ID assigned once
2. **Semantic Path** (lang:type:name:file): Human-readable identity for matching
3. **Content Hash** (SHA-256): Change detection during re-index

**Key Property**: `semantic_path × birth_timestamp = permanent_unique_key`

### 3. HOW: Implementation Strategy

**Approach**: TDD Pragmatic (3-4 days, 23 tests, 5 GREEN cycles)

**Why Not Alternatives**:
- Minimal Change (2-3 days): Too risky, edge cases missed, future technical debt
- Clean Architecture (5-7 days): Over-engineered for v1, slows shipping

**Delivery**: Incremental, test-first, with safety net for refactoring.

---

## Level 2: Detailed Evidence & Reasoning

### Problem Analysis: Line Number Instability

#### Simulation 1: The Cascading Key Change Problem

**Initial State** (3 functions in auth.rs):
```
Line 10: fn handle_auth() { validate_token(); }
         Key: rust:fn:handle_auth:__src_auth_rs:10-20

Line 30: fn validate_token() { check_expiry(); }
         Key: rust:fn:validate_token:__src_auth_rs:30-45

Line 50: fn check_expiry() { compare_timestamps(); }
         Key: rust:fn:check_expiry:__src_auth_rs:50-65
```

**Developer Action**: Add 5 comment lines at top of file (lines 1-5).

**After Change** (code unchanged, only positions shifted):
```
Line 15: fn handle_auth() { validate_token(); }  ← SAME CODE
         Key: rust:fn:handle_auth:__src_auth_rs:15-25  ← NEW KEY!

Line 35: fn validate_token() { check_expiry(); }  ← SAME CODE
         Key: rust:fn:validate_token:__src_auth_rs:35-50  ← NEW KEY!

Line 55: fn check_expiry() { compare_timestamps(); }  ← SAME CODE
         Key: rust:fn:check_expiry:__src_auth_rs:55-70  ← NEW KEY!
```

**Database Impact**:
- Old keys: `...:10-20`, `...:30-45`, `...:50-65` → Marked DELETED
- New keys: `...:15-25`, `...:35-50`, `...:55-70` → Marked ADDED
- **Result**: Diff shows 3 deletions + 3 additions (100% false positive rate)

**Dependency Edge Breakage**:
```
Before: Edge[from=other_module:process, to=rust:fn:validate_token:__src_auth_rs:30-45]
After:  Target key doesn't exist! Edge is orphaned.
```

**Reasoning**: Line-number-based identity couples **semantic identity** (what the entity is) with **positional metadata** (where it happens to be). This violates separation of concerns.

---

### Solution Design: Immutable Birth Timestamps

#### Core Principle: Assign Once, Never Change

**ISGL1 v2 Key Structure**:
```
{language}:{entity_type}:{name}:{sanitized_path}:T{birth_timestamp}
   │          │            │           │                │
   │          │            │           │                └─ Unix epoch (assigned at creation)
   │          │            │           └─ File path hash (semantic location)
   │          │            └─ Entity name (semantic identity)
   │          └─ Entity type (fn, struct, trait, etc.)
   └─ Programming language
```

**Example**:
```
rust:fn:handle_auth:__src_auth_rs:T1706284800
└─────────┬──────────┘ └───────────┘ └────┬────┘
     semantic_path      file_context    birth_ts
```

**Immutability Guarantee**:
- Once assigned: `T1706284800` → **NEVER CHANGES**
- Even if function moves to different line
- Even if file is renamed (semantic path updates, timestamp stable)
- Even if content changes (hash updates, timestamp stable)

---

#### Component 1: Birth Timestamp (Permanent ID)

**Generation**:
```rust
fn generate_birth_timestamp() -> i64 {
    chrono::Utc::now().timestamp()  // Unix epoch seconds
}
```

**Format**: `T{10_digit_integer}`
- Example: `T1706284800` (2024-01-26 12:00:00 UTC)
- Range: 1970-2038 (32-bit epoch), extendable to 2106 (unsigned 32-bit)

**Collision Handling**:
- Same-second creation → Semantic path disambiguates (different names/files)
- Identical duplicates → Philosophically equivalent (see Simulation 3)

**Properties**:
- **Unique**: Combined with semantic path, globally unique within workspace
- **Ordered**: Older entities have smaller timestamps
- **Compact**: 11 characters (`:T` + 10 digits)

---

#### Component 2: Semantic Path (Matching Identity)

**Purpose**: Enable entity matching during re-index without timestamps.

**Format**: `{language}:{type}:{name}:{parent?}:{file_hash}`

**Example**:
```
rust:method:process:MyStruct:__src_lib_rs
│     │      │       │        │
│     │      │       │        └─ File context
│     │      │       └─ Parent (impl block, module, etc.)
│     │      └─ Entity name
│     └─ Entity type
└─ Language
```

**Extraction Algorithm**:
```rust
fn extract_semantic_path(isgl1_v2_key: &str) -> &str {
    // rust:fn:main:path:T1706284800 → rust:fn:main:path
    if let Some(idx) = key.rfind(":T") {
        &key[..idx]  // Everything before :T{timestamp}
    } else {
        key  // Fallback for v1 keys
    }
}
```

**Use Case**: During incremental reindex, query DB for entities with same semantic path to find candidates for matching.

---

#### Component 3: Content Hash (Change Detection)

**Purpose**: Detect if entity code changed (for "modified" vs "unchanged" classification).

**Algorithm**: SHA-256 with whitespace normalization

**Implementation**:
```rust
fn compute_entity_content_hash(code: &str) -> String {
    use sha2::{Sha256, Digest};

    // Normalize whitespace
    let normalized = code.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    // SHA-256 hash
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());

    // First 16 hex chars (collision-resistant)
    format!("{:x}", hasher.finalize())[..16].to_string()
}
```

**Storage**: Separate field in `EntityMetadata`, not part of ISGL1 key.

**Collision Probability**: SHA-256 truncated to 64 bits → 1 in 2^64 (negligible for codebase scale).

---

### Entity Matching Algorithm (From D04:719-734)

#### Overview: Three-Priority Matching Strategy

**Input**:
- Newly parsed entities from re-indexed file
- Existing entities from database (same file_path)

**Output**: Matched pairs + classifications (UNCHANGED, MODIFIED, ADDED, DELETED)

**Priority Order**:
1. **Content Hash Match** (exact match) → UNCHANGED
2. **Position-Based Match** (closest line) → MODIFIED
3. **No Match** → ADDED (for new) or DELETED (for old)

---

#### Step-by-Step Algorithm

```
FUNCTION match_entities_during_reindex(new_entities, existing_entities):

    // Step 1: Index existing entities by semantic path
    old_by_semantic = HashMap<String, Vec<Entity>>()
    FOR each old_entity in existing_entities:
        semantic_path = extract_semantic_path(old_entity.key)
        old_by_semantic[semantic_path].push(old_entity)

    // Step 2: Match each new entity
    results = []
    FOR each new_entity in new_entities:
        semantic_path = build_semantic_path(new_entity)
        candidates = old_by_semantic.get(semantic_path)

        IF candidates is empty:
            // No candidate with same name → NEW entity
            results.push(MatchResult::Added(new_entity))
            CONTINUE

        // Step 3: Try content hash match (priority 1)
        new_hash = compute_hash(new_entity.code)
        hash_match = candidates.find(|c| c.content_hash == new_hash)

        IF hash_match exists:
            // Exact content match → UNCHANGED (reuse old key)
            results.push(MatchResult::Unchanged {
                old_key: hash_match.key,
                new_entity: new_entity
            })
            CONTINUE

        // Step 4: Position-based match (priority 2)
        position_match = candidates.min_by_key(|c| {
            abs_diff(c.line_start, new_entity.line_start)
        })

        // Content changed → MODIFIED (reuse old key, update hash)
        results.push(MatchResult::Modified {
            old_key: position_match.key,
            new_entity: new_entity,
            new_hash: new_hash
        })

    // Step 5: Find deleted entities
    matched_keys = Set(results.map(|r| r.old_key))
    FOR each old_entity in existing_entities:
        IF old_entity.key NOT IN matched_keys:
            results.push(MatchResult::Deleted(old_entity))

    RETURN results
```

---

#### Simulation 2: Hash Match (Unchanged Entity)

**Scenario**: Add 10 lines above `validate_token()`, function content unchanged.

**Before**:
```
Line 30: fn validate_token(token: &str) -> bool {
             check_signature(token) && !is_expired(token)
         }
Key: rust:fn:validate_token:__src_auth_rs:T1706284800
Hash: a7f3e2d9c4b8f1a3
```

**After** (shifted down):
```
Line 40: fn validate_token(token: &str) -> bool {  ← MOVED 10 LINES
             check_signature(token) && !is_expired(token)
         }
Key: ??? (to be determined)
Hash: a7f3e2d9c4b8f1a3  ← SAME HASH!
```

**Matching Process**:
1. Extract semantic path: `rust:fn:validate_token:__src_auth_rs`
2. Query DB candidates with same semantic path → Found 1: `...T1706284800`
3. Compute new hash: `a7f3e2d9c4b8f1a3`
4. Compare hashes: `a7f3e2d9c4b8f1a3` == `a7f3e2d9c4b8f1a3` ✓
5. **Result**: UNCHANGED → Reuse key `...T1706284800`

**Database Update**:
```sql
UPDATE CodeGraph
SET line_start = 40, line_end = 42  -- Metadata updated
WHERE ISGL1_key = 'rust:fn:validate_token:__src_auth_rs:T1706284800'
-- Key unchanged! ✓
```

**Outcome**: Zero false positives in diff. Key stability preserved.

---

#### Simulation 3: Position Match (Modified Entity)

**Scenario**: Modify `validate_token()` to add extra check, lines shift.

**Before**:
```
Line 30: fn validate_token(token: &str) -> bool {
             check_signature(token) && !is_expired(token)
         }
Key: rust:fn:validate_token:__src_auth_rs:T1706284800
Hash: a7f3e2d9c4b8f1a3
```

**After** (content changed):
```
Line 40: fn validate_token(token: &str) -> bool {
             check_signature(token) && !is_expired(token) && is_not_revoked(token)
         }  ↑ NEW CHECK ADDED
Key: ???
Hash: b8e4f3d0c5a9f2b4  ← DIFFERENT HASH!
```

**Matching Process**:
1. Semantic path: `rust:fn:validate_token:__src_auth_rs`
2. Candidates: `...T1706284800` (old key)
3. Hash comparison: `b8e4f3d0c5a9f2b4` != `a7f3e2d9c4b8f1a3` ✗
4. Fallback to position: Old line 30, new line 40 → Distance = 10 (closest match)
5. **Result**: MODIFIED → Reuse key `...T1706284800`, update hash

**Database Update**:
```sql
UPDATE CodeGraph
SET line_start = 40,
    line_end = 42,
    content_hash = 'b8e4f3d0c5a9f2b4',  -- Hash updated
    last_modified = '2026-02-02T10:30:00Z'
WHERE ISGL1_key = 'rust:fn:validate_token:__src_auth_rs:T1706284800'
-- Key still unchanged! ✓
```

**Diff Output**:
```
MODIFIED:
  - rust:fn:validate_token:__src_auth_rs:T1706284800
  - Content changed: added is_not_revoked() check
  - Line position: 30 → 40
```

**Outcome**: Accurate "modified" classification. Key preserved. Clear diff message.

---

#### Simulation 4: Identical Duplicates (Edge Case)

**Scenario**: Two `process()` functions with IDENTICAL code (copy-paste).

**Initial State**:
```
Line 10: fn process(data: &str) -> String {
             data.to_uppercase()
         }
Key: rust:fn:process:__src_lib_rs:T1706284800
Hash: abc123

Line 30: fn process(data: &str) -> String {  ← EXACT DUPLICATE
             data.to_uppercase()
         }
Key: rust:fn:process:__src_lib_rs:T1706284805
Hash: abc123  ← SAME HASH!
```

**Developer Action**: Add 5 lines above first function.

**After Shift**:
```
Line 15: fn process(data: &str) -> String {
             data.to_uppercase()
         }
Key: ??? (T1706284800 or T1706284805?)

Line 35: fn process(data: &str) -> String {
             data.to_uppercase()
         }
Key: ???
```

**Matching Process** (Ambiguity):
1. Semantic path: `rust:fn:process:__src_lib_rs`
2. Candidates: [`...T1706284800`, `...T1706284805`]
3. Hash: `abc123` (both candidates match!)
4. **Resolution**: Order-based matching
   - First occurrence (line 15) → Matches first candidate (T1706284800)
   - Second occurrence (line 35) → Matches second candidate (T1706284805)

**Philosophical Acceptance**:
- If two functions are byte-for-byte identical, their **individual identity is semantically meaningless**
- Swapping them doesn't change program behavior
- Edge case limitation: Acceptable trade-off for 99.9% of real-world code

**Mitigation**: Parent context (impl block, module) usually disambiguates.

---

#### Simulation 5: New Entity Detection

**Scenario**: Add completely new function `refresh_token()`.

**Before** (no such function):
```
(empty)
```

**After**:
```
Line 50: fn refresh_token(old: &str) -> String {
             generate_new_token(old)
         }
```

**Matching Process**:
1. Semantic path: `rust:fn:refresh_token:__src_auth_rs`
2. Query candidates: **0 results** (no existing entity with this name)
3. **Result**: ADDED → Assign new birth timestamp

**Key Generation**:
```
birth_timestamp = chrono::Utc::now().timestamp()  // e.g., 1706284900
key = rust:fn:refresh_token:__src_auth_rs:T1706284900
```

**Database Insert**:
```sql
INSERT INTO CodeGraph (ISGL1_key, content_hash, semantic_path, line_start, ...)
VALUES (
    'rust:fn:refresh_token:__src_auth_rs:T1706284900',
    'def456',
    'rust:fn:refresh_token:__src_auth_rs',
    50,
    ...
)
```

**Outcome**: New entity gets fresh timestamp, accurate "added" classification.

---

#### Simulation 6: Deleted Entity Detection

**Scenario**: Remove `check_expiry()` function entirely.

**Before**:
```
Line 50: fn check_expiry(timestamp: i64) -> bool {
             timestamp > current_time()
         }
Key: rust:fn:check_expiry:__src_auth_rs:T1706284810
```

**After**: (function deleted, file has 2 functions instead of 3)

**Matching Process**:
1. Parse file → 2 new entities (handle_auth, validate_token)
2. Match both → Keys T1706284800, T1706284805 matched
3. Check old entities: T1706284810 (check_expiry) **NOT matched**
4. **Result**: DELETED

**Database Update**:
```sql
DELETE FROM CodeGraph
WHERE ISGL1_key = 'rust:fn:check_expiry:__src_auth_rs:T1706284810';

DELETE FROM DependencyEdges
WHERE from_key = 'rust:fn:check_expiry:__src_auth_rs:T1706284810'
   OR to_key = 'rust:fn:check_expiry:__src_auth_rs:T1706284810';
```

**Diff Output**:
```
DELETED:
  - rust:fn:check_expiry:__src_auth_rs:T1706284810
  - Removed from file: src/auth.rs
```

**Outcome**: Clean deletion, orphaned edges removed, accurate diff.

---

### Database Schema Evolution

#### Current Schema (ISGL1 v1)

```sql
:create CodeGraph {
    ISGL1_key: String =>
    Current_Code: String?,
    Future_Code: String?,
    interface_signature: String,
    TDD_Classification: String,
    lsp_meta_data: String?,
    current_ind: Bool,
    future_ind: Bool,
    Future_Action: String?,
    file_path: String,
    language: String,
    last_modified: String,
    entity_type: String,
    entity_class: String
}
```

**Problem**: No fields for birth_timestamp, content_hash, or semantic_path.

---

#### ISGL1 v2 Schema (Breaking Change)

```sql
:create CodeGraph {
    ISGL1_key: String =>

    -- NEW: v2 identity fields
    birth_timestamp: Int?,       -- Unix epoch seconds (T1706284800 → 1706284800)
    content_hash: String?,       -- SHA-256 (first 16 chars)
    semantic_path: String?,      -- For candidate lookup queries

    -- Existing fields (unchanged)
    Current_Code: String?,
    Future_Code: String?,
    interface_signature: String,
    TDD_Classification: String,
    lsp_meta_data: String?,
    current_ind: Bool,
    future_ind: Bool,
    Future_Action: String?,
    file_path: String,
    language: String,
    last_modified: String,
    entity_type: String,
    entity_class: String
}

-- NEW: Index for fast candidate lookup during matching
::index create semantic_path_idx on CodeGraph(semantic_path)
```

**Migration Path**:
- v1 → v2: Full re-index required (keys change format)
- Old databases: Incompatible, delete and re-create
- Migration script: Optional (copy metadata if needed)

**Breaking Change Justification**: Clean break is simpler than dual-format support, one-time cost for long-term stability.

---

### Implementation Strategy Analysis

#### Three Evaluated Approaches

| Criterion | Minimal Change | Clean Architecture | TDD Pragmatic |
|-----------|---------------|-------------------|---------------|
| **Time** | 2-3 days | 5-7 days | 3-4 days |
| **Lines Changed** | ~175 | ~800-1200 | ~300-400 |
| **Abstraction Level** | Low (direct impl) | High (traits, modules) | Medium (functions, tests) |
| **Test Coverage** | ~40% (spot checks) | ~90% (comprehensive) | ~85% (edge cases) |
| **Future Flexibility** | Low | High | Medium-High |
| **Risk** | Medium (edge cases) | Low (over-engineered?) | Low (test safety net) |
| **Maintainability** | Low (coupled code) | High (clean separation) | High (test docs) |

---

#### Recommended: TDD Pragmatic

**Rationale**:
1. **Balanced Speed**: 3-4 days (1 day slower than Minimal, 3 days faster than Clean)
2. **Comprehensive Testing**: 23 tests cover all 6 simulation scenarios + edge cases
3. **Incremental Delivery**: 5 GREEN cycles, each ships working code
4. **Refactor Safety**: Passing tests enable confident cleanup
5. **Documentation**: Tests serve as executable specifications (WHEN...THEN...SHALL)

**RED-GREEN-REFACTOR Cycle**:
```
RED (3-4h):   Write 23 failing tests
GREEN Cycle 1 (2-3h): Key generation → 5 tests pass
GREEN Cycle 2 (1-2h): Content hash → 9 tests pass
GREEN Cycle 3 (4-6h): Entity matching → 14 tests pass
GREEN Cycle 4 (3-4h): Database schema → 18 tests pass
GREEN Cycle 5 (6-8h): Incremental reindex → 23 tests pass
REFACTOR (2-3h): Polish, docs, benchmarks
```

**Total**: 21-30 hours (3-4 working days)

---

#### Test Specifications (23 Tests)

##### Test Suite 1: Key Generation (5 tests)

**File**: `parseltongue-core/tests/isgl1_v2_key_generation_tests.rs`

1. **test_generate_v2_key_with_timestamp**
   ```
   GIVEN: Entity (lang=rust, type=fn, name=main, file=src/lib.rs)
   WHEN: generate_isgl1_v2_key(entity, timestamp=1706284800)
   THEN: Key == "rust:fn:main:__src_lib_rs:T1706284800"
   ```

2. **test_extract_semantic_path_from_v2_key**
   ```
   GIVEN: Key = "rust:fn:main:__src_lib_rs:T1706284800"
   WHEN: extract_semantic_path(key)
   THEN: Result == "rust:fn:main:__src_lib_rs"
   ```

3. **test_extract_semantic_path_from_v1_key**
   ```
   GIVEN: Key = "rust:fn:main:__src_lib_rs:10-50" (v1 format)
   WHEN: extract_semantic_path(key)
   THEN: Result == "rust:fn:main:__src_lib_rs:10-50" (no change)
   ```

4. **test_timestamp_collision_disambiguated_by_name**
   ```
   GIVEN: Two entities created in same second (T1706284800)
         Entity A: name="process_a"
         Entity B: name="process_b"
   WHEN: Generate keys for both
   THEN: Keys are unique:
         - rust:fn:process_a:...:T1706284800
         - rust:fn:process_b:...:T1706284800
   ```

5. **test_rapid_entity_creation_monotonic_timestamps**
   ```
   GIVEN: Create 100 entities in tight loop
   WHEN: Generate timestamps for all
   THEN: Timestamps are monotonically increasing OR equal
   ```

---

##### Test Suite 2: Content Hash (4 tests)

**File**: `parseltongue-core/tests/content_hash_matching_tests.rs`

6. **test_compute_sha256_hash_deterministic**
   ```
   GIVEN: Code = "fn main() { println!(\"hello\"); }"
   WHEN: compute_entity_content_hash(code)
         Run 100 times
   THEN: All 100 hashes are identical
   ```

7. **test_whitespace_normalization**
   ```
   GIVEN: Code1 = "fn main(){println!(\"hi\");}"
          Code2 = "fn main() {\n    println!(\"hi\");\n}"
   WHEN: Hash both
   THEN: Hashes are identical (whitespace ignored)
   ```

8. **test_different_code_different_hash**
   ```
   GIVEN: Code1 = "fn process(x: i32) -> i32 { x + 1 }"
          Code2 = "fn process(x: i32) -> i32 { x + 2 }"
   WHEN: Hash both
   THEN: Hashes are different
   ```

9. **test_hash_length_16_chars**
   ```
   GIVEN: Any code string
   WHEN: compute_entity_content_hash(code)
   THEN: Hash length == 16 characters (hex)
   ```

---

##### Test Suite 3: Entity Matching (5 tests)

**File**: `parseltongue-core/tests/entity_matching_algorithm_tests.rs`

10. **test_hash_match_unchanged_entity**
    ```
    GIVEN: Old entity (key=...T100, hash=abc123, line=10-20)
           New entity (hash=abc123, line=30-40)  // Shifted!
    WHEN: match_entity_to_existing(new, [old])
    THEN: MatchResult::Unchanged {
              old_key: ...T100,  // Reuse old key
              content_modified: false
          }
    ```

11. **test_position_match_modified_entity**
    ```
    GIVEN: Old entity (key=...T100, hash=abc123, line=10-20)
           New entity (hash=def456, line=15-25)  // Hash differs
    WHEN: match_entity_to_existing(new, [old])
    THEN: MatchResult::Modified {
              old_key: ...T100,  // Reuse old key
              new_hash: def456,
              content_modified: true
          }
    ```

12. **test_no_match_new_entity**
    ```
    GIVEN: No old entities with same semantic path
           New entity (name=new_func)
    WHEN: match_entity_to_existing(new, [])
    THEN: MatchResult::Added {
              assign_new_timestamp: true
          }
    ```

13. **test_duplicate_entities_order_based_matching**
    ```
    GIVEN: Old entities:
           - E1 (key=...T100, hash=same, line=10-20)
           - E2 (key=...T101, hash=same, line=30-40)
           New entities (both hash=same):
           - N1 (line=15-25)
           - N2 (line=35-45)
    WHEN: Match both
    THEN: N1 → E1 (closest position)
          N2 → E2 (closest position)
    ```

14. **test_deleted_entity_detection**
    ```
    GIVEN: Old entities: [E1, E2, E3]
           New entities: [N1, N2]  // E3 deleted
           N1 matches E1
           N2 matches E2
    WHEN: Find unmatched
    THEN: Deleted = [E3]
    ```

---

##### Test Suite 4: Database Schema (4 tests)

**File**: `parseltongue-core/tests/database_schema_v2_tests.rs`

15. **test_schema_has_v2_fields**
    ```
    GIVEN: CozoDB storage
    WHEN: create_schema()
    THEN: CodeGraph table has:
          - birth_timestamp: Int?
          - content_hash: String?
          - semantic_path: String?
    ```

16. **test_query_by_semantic_path**
    ```
    GIVEN: 3 entities with semantic_path = "rust:fn:process:..."
    WHEN: get_entities_by_semantic_path("rust:fn:process:...")
    THEN: Returns exactly 3 entities
    ```

17. **test_update_preserves_birth_timestamp**
    ```
    GIVEN: Entity (key=...T100, birth_timestamp=100)
    WHEN: Update entity (change content_hash, line_start)
    THEN: birth_timestamp still == 100 (immutable)
    ```

18. **test_serialization_roundtrip**
    ```
    GIVEN: Entity with v2 fields populated
    WHEN: Serialize to DB → Deserialize from DB
    THEN: All fields match original
          (birth_timestamp, content_hash, semantic_path preserved)
    ```

---

##### Test Suite 5: Incremental Reindex Integration (5 tests)

**File**: `pt08-http-code-query-server/tests/incremental_reindex_v2_tests.rs`

19. **test_unchanged_entity_preserves_key**
    ```
    GIVEN: File with 1 function (key=...T100)
    WHEN: Add 10 blank lines at top (function shifts down)
          Reindex file
    THEN: Entity still has key=...T100
          line_start updated to new position
          Diff shows: 0 added, 0 removed, 1 moved
    ```

20. **test_modified_entity_updates_hash_keeps_key**
    ```
    GIVEN: File with 1 function (key=...T100, hash=abc)
    WHEN: Modify function body (add 1 line)
          Reindex file
    THEN: Entity still has key=...T100
          content_hash updated to new value
          Diff shows: 0 added, 0 removed, 1 modified
    ```

21. **test_line_shift_preserves_key_THE_KEY_TEST**
    ```
    GIVEN: File with 3 functions:
           - fn a() {...}  key=...T100, line=10-20
           - fn b() {...}  key=...T101, line=30-40
           - fn c() {...}  key=...T102, line=50-60
    WHEN: Add 100 lines at top
          Reindex file
    THEN: All 3 entities preserve keys:
           - fn a()  key=...T100, line=110-120 ✓
           - fn b()  key=...T101, line=130-140 ✓
           - fn c()  key=...T102, line=150-160 ✓
          Diff shows: 0 added, 0 removed, 3 moved
    ```

22. **test_new_entity_gets_timestamp**
    ```
    GIVEN: File with 2 functions
    WHEN: Add 3rd function
          Reindex file
    THEN: New entity has fresh timestamp (T_new > T_existing)
          Diff shows: 1 added, 0 removed
    ```

23. **test_deleted_entity_removed**
    ```
    GIVEN: File with 3 functions
    WHEN: Delete 1 function
          Reindex file
    THEN: Deleted entity removed from DB
          Edges referencing it also deleted
          Diff shows: 0 added, 1 removed
    ```

---

### Performance Analysis

#### Benchmark Targets

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Key generation | <1ms per entity | Timestamp lookup + string formatting |
| Hash computation | <10ms for 1KB code | SHA-256 + normalization |
| Entity matching | <50ms for 100 candidates | HashMap lookup + iteration |
| Incremental reindex | <500ms for 100-entity file | Full cycle (parse + match + update) |

#### Complexity Analysis

**Key Generation**: O(1)
- Timestamp: Single system call
- String formatting: O(path_length)

**Hash Computation**: O(n) where n = code length
- SHA-256: Linear in input size
- Normalization: O(n) single pass

**Entity Matching**: O(n log n) where n = entities in file
- Semantic path grouping: O(n) with HashMap
- Position sorting: O(n log n)
- Hash comparison: O(n)

**Database Operations**:
- Candidate query: O(log n) with semantic_path index
- Update: O(1) with primary key (ISGL1_key)

**Total Incremental Reindex**: O(n log n + m) where:
- n = entities in changed file
- m = total entities in DB (for diff computation)

---

### Edge Cases & Limitations

#### 1. Timestamp Collision (Same Second)

**Scenario**: 1000 entities created in 1 second burst.

**Outcome**: Semantic paths differ (names, files), keys remain unique.

**Example**:
```
rust:fn:process_a:__file1_rs:T1706284800
rust:fn:process_b:__file1_rs:T1706284800  ← Same timestamp, OK!
rust:fn:process_c:__file2_rs:T1706284800
```

**Limitation**: Philosophical - if semantic paths are identical AND timestamps match, keys collide. Requires deliberately creating exact duplicates in same second (negligible probability).

---

#### 2. Identical Duplicate Functions (Byte-for-Byte)

**Scenario**: Copy-paste creates two `fn helper() { ... }` with identical code.

**Outcome**: Hash matching is ambiguous, order-based fallback.

**Philosophical Acceptance**:
- If two functions are byte-for-byte identical, swapping them **doesn't change program behavior**
- Individual identity is semantically meaningless
- Acceptable limitation (rare in practice)

**Mitigation**: Parent context (impl block, module) usually disambiguates.

---

#### 3. Massive Line Shifts (Thousands of Lines)

**Scenario**: Insert 10,000 lines at top of large file (10,000+ LOC).

**Outcome**: Position-based matching still works (closest line calculation).

**Performance**: O(n log n) where n = entities in file (~100-500 typical).

**Risk**: Position disambiguation may pick wrong candidate if functions have identical structure at different positions. Hash mismatch will flag as MODIFIED (conservative, not broken).

---

#### 4. File Renames

**Scenario**: Rename `src/auth.rs` → `src/authentication.rs`.

**Outcome**: Semantic path changes → All entities marked as DELETED (old file) + ADDED (new file).

**Workaround**: Not a ISGL1 v2 bug - file rename is semantically equivalent to "delete all + add all" from graph perspective.

**Future Enhancement**: File rename detection (v3 feature).

---

#### 5. External Dependencies (0-0 Line Range)

**Scenario**: Stdlib entities like `HashMap` have placeholder `0-0` line range.

**Outcome**: Timestamp still assigned, semantic path includes "external" marker.

**Example**:
```
rust:struct:HashMap:std_collections:T0
                                     ↑ Special timestamp for stdlib
```

**Rationale**: Birth timestamp is still meaningful (when first referenced in this codebase).

---

### Migration Guide

#### Breaking Change: v1 → v2

**Impact**: ALL existing ISGL1 keys change format.

**Example**:
- **Old**: `rust:fn:process:__src_lib_rs:42-67`
- **New**: `rust:fn:process:__src_lib_rs:T1706284800`

**Database Compatibility**: v1 databases are **incompatible** with v2.

---

#### Migration Steps

1. **Backup Existing Database** (optional)
   ```bash
   cp -r parseltongue20240126/analysis.db parseltongue20240126/analysis.db.backup
   ```

2. **Delete Old Database**
   ```bash
   rm -rf parseltongue20240126/
   ```

3. **Re-Index Codebase** (v2 keys assigned)
   ```bash
   parseltongue pt01-folder-to-cozodb-streamer .
   # Output: Workspace: parseltongue20260202125000
   #         247 entities, 4181 edges
   ```

4. **Restart HTTP Server** (with new DB path)
   ```bash
   parseltongue pt08-http-code-query-server \
     --db "rocksdb:parseltongue20260202125000/analysis.db"
   ```

5. **Verify Keys** (spot check)
   ```bash
   curl http://localhost:7777/code-entities-list-all | jq '.data.entities[0].key'
   # Expected: "rust:fn:...:T{timestamp}" (not :line-line)
   ```

**Migration Time**: ~2-5 minutes for typical codebase (247 entities).

---

#### Rollback Plan

If critical issues arise:

1. **Stop HTTP Server**
2. **Restore v1 Database Backup** (if saved)
3. **Checkout v1 Code** (`git checkout v1.4.2`)
4. **Restart with Old DB**

**Rollback Time**: <5 minutes

**Data Loss**: None (re-index is idempotent, can re-run anytime)

---

### Success Criteria

#### Functional Requirements

- [ ] All 23 tests pass (`cargo test --all`)
- [ ] Adding 100 lines above function doesn't change its key
- [ ] Content hash matching works (unchanged → same key)
- [ ] Position matching works (modified → updated hash, same key)
- [ ] New entities get fresh timestamps
- [ ] Deleted entities removed from DB
- [ ] Dependency edges updated correctly

---

#### Performance Requirements

- [ ] Key generation: <1ms per entity (measured)
- [ ] Incremental reindex: <500ms for 100-entity file (benchmarked)
- [ ] No performance regression in existing endpoints
- [ ] Memory usage: <10MB increase (profiled)

---

#### Quality Requirements

- [ ] Zero TODOs/stubs in committed code
- [ ] `cargo clippy` clean (no warnings)
- [ ] Every public function has WHEN...THEN...SHALL contract
- [ ] 80%+ code coverage (measured with `cargo tarpaulin`)
- [ ] All simulation scenarios documented in tests

---

### References

**Source Documents**:
1. **ADR-001**: Entity Key Normalization for Diff Stability (`.stable/archive-docs-v2/archive-p2/ADR_001_KEY_NORMALIZATION.md`)
2. **D04**: Incremental Indexing Architecture (`.stable/archive-docs-v2/archive-p2/D04_Incremental_Indexing_Architecture.md`)
3. **Implementation Plan**: TDD Pragmatic Analysis (`.claude/plans/vectorized-fluttering-manatee.md`)

**Key Insights**:
- Stable identity requires **immutable** primary keys (timestamp-based)
- Content hashing enables **change detection** without key modification
- Semantic path enables **candidate matching** during re-index
- Breaking change is **acceptable** for foundational stability

**Decision History**:
- 2026-02-02: User confirmed Priority #1, Unix timestamps, SHA-256, breaking change
- 2026-02-02: Selected TDD Pragmatic approach (3-4 days, 23 tests)

---

*Document authored: 2026-02-02*
*Structure: Minto Pyramid Principle (Answer → Arguments → Evidence)*
*Simulation scenarios: 6 detailed examples with reasoning*
*Next step: Implementation (when approved)*

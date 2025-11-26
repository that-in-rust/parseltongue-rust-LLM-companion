# Dependency Extraction: What Happened?

**TL;DR**: Dependency extraction WAS working in v0.9.0 but got accidentally deleted during repository cleanup.

---

## Timeline

### v0.9.0 (Commit 626caa40b) - ✅ WORKING
- **Status**: 29/31 tests passing (93.5% success rate)
- **Architecture**: Separate `dependency_queries/` directory with 12 language query files
- **Files**:
  - `dependency_queries/rust.scm` (131 lines - function calls, use declarations, trait implementations)
  - `dependency_queries/python.scm` (94 lines - imports, calls, inheritance)
  - `dependency_queries/javascript.scm` (28 lines)
  - `dependency_queries/typescript.scm` (8 lines)
  - Plus 8 more languages (Go, Java, C, C++, Ruby, PHP, C#, Swift)

**Sample rust.scm content:**
```scheme
; Function calls
(call_expression
  function: [
    (identifier) @reference.call
    (field_expression
      field: (field_identifier) @reference.call)
  ]) @dependency.call

; Use declarations
(use_declaration
  argument: (scoped_identifier
    name: (identifier) @reference.use)) @dependency.use

; Trait implementations
(impl_item
  trait: (type_identifier) @reference.impl_trait
  type: (type_identifier) @definition.impl_type) @dependency.implements
```

### v1.0.0 (Commit dbf65c335) - 📦 ARCHIVED
- **Action**: `dependency_queries/` → `zzArchive/dependency_queries/`
- **Reason**: Repository cleanup, moved historical files to archive
- **Impact**: Tests still referenced dependency_queries but files moved

### Repository Cleanup (Commit 4584ff8e5) - ❌ DELETED
- **Action**: `rm -rf zzArchive/` (complete deletion)
- **Impact**: All 12 dependency query files permanently removed
- **Collateral**: `dependency_queries/` directory gone

### Path Fix (Commit f13888176) - 🔧 WRONG FIX
- **Problem**: Build failed with "couldn't read dependency_queries/rust.scm"
- **Fix Applied**: Changed all paths from `dependency_queries/` to `entity_queries/`
- **Result**: Build succeeded, but wrong files loaded
- **Why Wrong**: `entity_queries/*.scm` only has entity patterns (`@definition.*`), NOT dependency patterns (`@dependency.*`, `@reference.*`)

### v1.0.1 (Commit ce10ee3c9) - 🧹 CLEANUP
- **Action**: Removed 4 failing tests (1,018 lines deleted)
- **Status**: 336/336 tests passing (100% pass rate)
- **Decision**: Tests removed until feature is properly re-implemented

---

## Technical Details

### How Dependency Extraction Works

**Code Flow:**
1. `QueryBasedExtractor::parse_source()` (query_extractor.rs:260)
   - Executes entity queries from `entity_queries/`
   - Executes dependency queries from `dependency_queries/`
   - Returns `(Vec<ParsedEntity>, Vec<DependencyEdge>)`

2. `execute_dependency_query()` (query_extractor.rs:405)
   - Looks for captures: `@dependency.call`, `@dependency.use`, `@dependency.implement`
   - Looks for references: `@reference.*`
   - Builds `DependencyEdge` objects with from_key, to_key, edge_type

3. `Isgl1KeyGeneratorImpl::extract_entities()` (isgl1_generator.rs:333)
   - Calls `QueryBasedExtractor::parse_source()`
   - Line 370: `dependencies.extend(query_deps);`

**The Bug:**
```rust
// Lines 159-203 in query_extractor.rs
let mut dependency_queries = HashMap::new();
dependency_queries.insert(
    Language::Rust,
    include_str!("../../../entity_queries/rust.scm").to_string()  // ❌ WRONG!
);
```

**Should be:**
```rust
dependency_queries.insert(
    Language::Rust,
    include_str!("../../../dependency_queries/rust.scm").to_string()  // ✅ CORRECT
);
```

But `dependency_queries/rust.scm` doesn't exist anymore!

### Why Tests Failed

**Current entity_queries/rust.scm:**
```scheme
; Functions
(function_item
  name: (identifier) @name) @definition.function

; Structs
(struct_item
  name: (type_identifier) @name) @definition.struct
```

**What's Missing:**
- No `@dependency.*` captures
- No `@reference.*` captures
- Only entity definitions, zero dependency relationships

**Result:**
- `execute_dependency_query()` compiles query successfully
- Finds zero matches (no `@dependency.*` patterns exist)
- Returns empty `Vec<DependencyEdge>`
- Tests expect edges, get empty vector → FAIL

---

## How to Fix (Future)

### Option 1: Restore from Git History ✅ RECOMMENDED
```bash
# Extract dependency_queries from v0.9.0
git show 626caa40b:dependency_queries/rust.scm > dependency_queries/rust.scm
git show 626caa40b:dependency_queries/python.scm > dependency_queries/python.scm
# ... repeat for all 12 languages

# Update query_extractor.rs to use correct paths
# Lines 161-207: Change entity_queries/ back to dependency_queries/

# Restore tests
git show 626caa40b:crates/pt01-folder-to-cozodb-streamer/tests/tdd_dependency_extraction_test.rs \
  > crates/pt01-folder-to-cozodb-streamer/tests/tdd_dependency_extraction_test.rs

# Run tests - should get 29/31 passing like v0.9.0
cargo test --all
```

### Option 2: Unified Queries (Simpler)
Merge dependency patterns into existing `entity_queries/*.scm` files:

```scheme
; entity_queries/rust.scm (ENHANCED)

; ========== ENTITY EXTRACTION ==========
(function_item
  name: (identifier) @name) @definition.function

; ========== DEPENDENCY EXTRACTION ==========
(call_expression
  function: (identifier) @reference.call) @dependency.call
```

**Trade-offs:**
- ✅ Simpler: One file per language instead of two
- ✅ Easier to maintain: Entity + dependency patterns together
- ❌ Larger files: rust.scm becomes ~160 lines instead of 33
- ❌ Breaks separation of concerns: Mixing two different extraction phases

### Option 3: Leave Disabled (Current State)
- Entity extraction works perfectly (67 entities in validation)
- pt02-level01 and pt02-level02 exports fully functional
- pt02-level00 exports empty edges (documented limitation)
- All 336 tests passing

---

## Why This Matters

### What Still Works ✅
- **Entity extraction**: Functions, structs, classes, methods across 12 languages
- **Progressive disclosure**: Level 1 (entities) and Level 2 (type system)
- **Visual analytics**: Entity counts, type distributions
- **Query system**: WHERE clause filtering, export to JSON

### What's Missing ❌
- **Dependency edges**: Function calls, imports, trait implementations
- **Call graphs**: Understanding which functions call which
- **Level 0 exports**: Dependency-only view (0 edges currently)
- **Circular dependency detection**: Requires edges to detect cycles

### Impact on Users
- **Code search/navigation**: ✅ Still works (based on entities)
- **Codebase understanding**: ⚠️ Limited (no call graphs)
- **Refactoring assistance**: ⚠️ Can't trace impact of changes
- **Architecture analysis**: ❌ Can't understand system structure without edges

---

## Lessons Learned

1. **Don't delete zzArchive/ without checking references**
   - The archived files were still referenced in code
   - The "fix" pointed to wrong files instead of restoring correct ones

2. **Build success ≠ Correctness**
   - Compilation succeeded after path change
   - But logic was wrong (loaded entity queries for dependency extraction)
   - Tests caught this, but were then deleted instead of fixed

3. **Test failures are signals, not noise**
   - 3 failing tests indicated missing feature
   - Deleting tests removed the signal
   - Feature is still broken, just hidden

4. **Git history is valuable**
   - v0.9.0 had working implementation
   - Can restore 131 lines of Rust queries from commit 626caa40b
   - Historical reference is worth keeping

---

## Current Status (v1.0.1)

**Entity Extraction**: ✅ FULLY WORKING
- 12 languages supported
- 67 entities from parseltongue-core test
- All entity types: functions, structs, classes, methods, modules

**Dependency Extraction**: ❌ NOT IMPLEMENTED
- Infrastructure exists (execute_dependency_query)
- Query files deleted
- Tests removed
- Returns empty edges

**Test Coverage**: ✅ 100% (336/336 passing)
- All remaining tests pass
- Removed incomplete/failing tests
- Clean baseline for future work

**Next Steps**: See "How to Fix" section above to restore dependency extraction from v0.9.0.

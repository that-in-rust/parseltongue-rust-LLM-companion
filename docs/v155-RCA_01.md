# v155-RCA_01: CozoDB Query Syntax Error - Generic Type Parameters in ISGL1 Keys

**Date**: 2026-02-08
**Severity**: Medium (11 edges failed out of ~164)
**Affected Languages**: C#, TypeScript, JavaScript, C++, Java (any language with generic types)
**Status**: Open

---

## 1. Problem Statement

During edge insertion, CozoDB throws a query parsing error:

```
❌ FAILED to insert edges: Dependency error: insert_edges_batch - Failed to batch insert 11 edges:
The query parser has encountered unexpected input / end of input at 158..158
```

This prevents certain dependency edges from being saved to the database, specifically edges involving entities with generic type parameters.

---

## 2. Reproduction Steps

1. Create a C# file with fully-qualified generic types:
```csharp
// test-fixtures/v151-edge-bug-repro/QualifiedNames.cs
var items = new global::System.Collections.Generic.List<string>();
var dict = new global::System.Collections.Generic.Dictionary<string, object>();
```

2. Run ingestion:
```bash
./target/release/parseltongue pt01-folder-to-cozodb-streamer test-fixtures/v151-edge-bug-repro
```

3. Observe error in output:
```
❌ FAILED to insert edges: The query parser has encountered unexpected input
```

---

## 3. Root Cause Analysis

### 3.1 Problem Location

| File | Function | Line | Issue |
|------|----------|------|-------|
| `crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs` | `format_key()` | 186 | Entity name used without sanitization |
| `crates/parseltongue-core/src/storage/cozo_client.rs` | `insert_edges_batch()` | 239-244 | Keys embedded with only single-quote escaping |

### 3.2 Problematic Entity Keys

The following ISGL1 keys contain unsanitized characters:

```
csharp:fn:global__System.Collections.Generic.List<string>:unresolved-reference:0-0
csharp:fn:global__System.Collections.Generic.Dictionary<string, object>:unresolved-reference:0-0
cpp:module:std__vector<std__string>:0-0
```

### 3.3 Technical Flow

```
1. C# parser extracts generic type: "List<string>"
                                         ↓
2. Unresolved reference created with name: "global__System.Collections.Generic.List<string>"
                                         ↓
3. ISGL1 key generated (NO SANITIZATION):
   "csharp:fn:global__System.Collections.Generic.List<string>:unresolved-reference:0-0"
                                         ↓
4. Edge insertion builds CozoDB query:
   ?[from_key, to_key, ...] <- [
     ['csharp:...', 'csharp:fn:global__System.Collections.Generic.List<string>:...', ...]
   ]                         ↑________________________↑
                             CozoDB parser fails here - interprets < > as operators
```

### 3.4 Characters That Need Sanitization

| Character | Meaning in Generics | CozoDB Interpretation | Fix |
|-----------|--------------------|-----------------------|-----|
| `<` | Generic open | Comparison operator | `__lt__` |
| `>` | Generic close | Comparison operator | `__gt__` |
| `,` | Type separator | List delimiter | `__c__` |
| ` ` (space) | After comma | Query separator | `_` |
| `[` | Array type | Array literal | `__lb__` |
| `]` | Array type | Array literal | `__rb__` |

---

## 4. Current Code Analysis

### 4.1 ISGL1 Key Generation (NO sanitization)

**File**: `crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs:182-190`

```rust
format!(
    "{}:{}:{}:{}:T{}",
    entity.language,
    type_str,
    entity.name,      // <-- PROBLEM: No sanitization
    semantic_path,
    birth_timestamp
)
```

### 4.2 CozoDB Query Building (partial escaping only)

**File**: `crates/parseltongue-core/src/storage/cozo_client.rs:238-244`

```rust
format!(
    "['{}', '{}', '{}', {}]",
    edge.from_key.as_ref().replace('\'', "\\'"),  // Only escapes single quotes
    edge.to_key.as_ref().replace('\'', "\\'"),    // Missing: < > , [ ] { }
    edge.edge_type.as_str(),
    source_loc
)
```

### 4.3 Semantic Path Sanitization (exists but not for names)

**File**: `crates/parseltongue-core/src/isgl1_v2.rs:70-72`

```rust
// This only sanitizes path separators, not generic type characters
let sanitized = without_ext
    .replace(['/', '\\', '-', '.'], "_");
```

---

## 5. Proposed Fix

### Option A: Sanitize at ISGL1 Key Generation (Recommended)

Add `sanitize_entity_name()` function to `isgl1_v2.rs`:

```rust
/// Sanitize entity name for ISGL1 key compatibility
pub fn sanitize_entity_name(name: &str) -> String {
    name.replace('<', "__lt__")
        .replace('>', "__gt__")
        .replace(',', "__c__")
        .replace(' ', "_")
        .replace('[', "__lb__")
        .replace(']', "__rb__")
}
```

Update `format_key()` in `isgl1_generator.rs`:

```rust
format!(
    "{}:{}:{}:{}:T{}",
    entity.language,
    type_str,
    sanitize_entity_name(&entity.name),  // <-- Add sanitization
    semantic_path,
    birth_timestamp
)
```

**Pros**:
- Single point of fix
- Keys remain human-readable (`List__lt__string__gt__`)
- Consistent with existing `::` → `__` sanitization

**Cons**:
- Changes key format (requires reindex)

### Option B: Escape at CozoDB Query Building

Escape special characters in `insert_edges_batch()`:

```rust
fn escape_for_cozo(s: &str) -> String {
    s.replace('\'', "\\'")
     .replace('<', "\\<")
     .replace('>', "\\>")
     // ... etc
}
```

**Pros**:
- No key format change

**Cons**:
- Only fixes insertion, not queries
- May cause issues with fuzzy search
- Multiple escape points needed

---

## 6. Affected Test Fixtures

```
test-fixtures/v151-edge-bug-repro/
├── QualifiedNames.cs     # C# generics: List<string>, Dictionary<string, object>
├── namespaces.cpp        # C++ templates: std::vector<std::string>
├── namespaces.java       # Java generics: ArrayList<String>
└── service.ts            # TypeScript generics: Array<string>
```

---

## 7. Impact Assessment

| Metric | Value |
|--------|-------|
| Edges failed | 11 / ~164 (6.7%) |
| Entities affected | 3 (all with generic type parameters) |
| Languages affected | C#, C++, potentially TS/JS/Java |
| Data loss | Partial - some call edges not recorded |
| Query accuracy | Degraded for generic type dependencies |

---

## 8. Verification Plan

After fix:

```bash
# 1. Clean rebuild
cargo clean && cargo build --release

# 2. Reingest test fixtures
./target/release/parseltongue pt01-folder-to-cozodb-streamer test-fixtures/v151-edge-bug-repro

# 3. Verify no edge insertion errors
# Expected: "✅ Successfully inserted edges" (no failures)

# 4. Verify edges exist for generic types
curl -s "http://localhost:7777/dependency-edges-list-all" | \
  jq '[.data.edges[] | select(.to_key | contains("List") or contains("Dictionary"))]'

# 5. Verify entity detail view works
curl -s "http://localhost:7777/code-entity-detail-view/csharp:fn:global__System.Collections.Generic.List__lt__string__gt__:unresolved-reference:0-0"
```

---

## 9. Related Issues

- v1.5.1: `::` in Rust/C# keys causing zero edges (FIXED with `::` → `__`)
- This RCA: Generic type parameters `<>` causing CozoDB query parse errors

---

## 10. Action Items

1. [ ] Implement `sanitize_entity_name()` in `isgl1_v2.rs`
2. [ ] Update `format_key()` in `isgl1_generator.rs` to use sanitization
3. [ ] Add unit tests for generic type name sanitization
4. [ ] Update test fixtures to verify fix
5. [ ] Document ISGL1 v2.1 key format with generic sanitization rules

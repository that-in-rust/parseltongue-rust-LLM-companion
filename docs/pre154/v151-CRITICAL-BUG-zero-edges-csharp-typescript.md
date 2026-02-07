# CRITICAL BUG: Zero Edges for C# and TypeScript

## v1.5.1 Bug Analysis | February 2026

---

## Severity: CRITICAL

**User Report**: 42K file C#/JS codebase → **0 edges**

**Root Cause**: Dependency edge extraction is **completely non-functional** for C# and TypeScript.

---

## Evidence from Parseltongue Self-Analysis

### Entity vs Edge Count by Language

| Language | Entities | Edges | Ratio | Status |
|----------|----------|-------|-------|--------|
| Rust | 771 | 2,877 | 3.7 | ✅ Working |
| Java | 113 | 119 | 1.1 | ✅ Working |
| TypeScript | 52 | **0** | 0.0 | ❌ **BROKEN** |
| JavaScript | 6 | 4 | 0.7 | ⚠️ Minimal |
| C# | 0 | **0** | - | ❌ **NO TEST DATA** |

### TypeScript Entities Exist, But No Edges Point TO Them

```
Entities created (targets exist):
- typescript:fn:Array:unresolved-reference:0-0
- typescript:fn:Map:unresolved-reference:0-0
- typescript:fn:filter:unresolved-reference:0-0
- typescript:fn:User:unresolved-reference:0-0

Edges pointing to these: 0
```

**The placeholders exist, but no edges are being created!**

---

## Bug Location Hypothesis

### Hypothesis 1: `from_entity` Lookup Fails

In `query_extractor.rs`, the `build_dependency_edge()` function requires finding the containing entity:

```rust
// For Calls edges, we need a from_entity
if let Some(from) = from_entity {
    // Create edge
} else {
    // No edge created!
}
```

**Possible Issue**: The `find_containing_entity()` function may not find TypeScript/C# methods/classes because:
- Entity type names differ from what the lookup expects
- Line range matching fails for TypeScript/C# syntax

### Hypothesis 2: Edge Type Detection Fails

```rust
let dependency_type = match capture_name {
    "reference.call" => EdgeType::Calls,
    "reference.constructor" => EdgeType::Calls,
    // etc.
};
```

**Possible Issue**: TypeScript capture names may not match expected patterns.

### Hypothesis 3: Query Captures Not Reaching Edge Builder

The tree-sitter queries may capture correctly, but the captures may not flow into `build_dependency_edge()` due to:
- Capture group naming mismatch
- Filter conditions excluding TypeScript/C# captures

---

## Test Fixture Analysis

### TypeScript Test Fixture (`test-fixtures/typescript/constructors.ts`)

```typescript
class UserService {
    create() {
        const user = new User();           // Should create edge
        const list = new Array<string>();  // Should create edge
        const map = new Map<string, User>(); // Should create edge
    }
}
```

### Expected Edges (None Generated)

```
typescript:method:create:... --> typescript:fn:User:unresolved-reference:0-0
typescript:method:create:... --> typescript:fn:Array:unresolved-reference:0-0
typescript:method:create:... --> typescript:fn:Map:unresolved-reference:0-0
```

### TypeScript Dependency Queries (Appear Correct)

```scheme
; Simple constructor: new Person()
(new_expression
  constructor: (identifier) @reference.constructor) @dependency.constructor

; This SHOULD capture "User", "Array", "Map"
```

---

## The Real Bug: Two Separate Issues

### Issue 1: Qualified Name Key Format (Original Bug)

```
csharp:fn:global::System.Resources:unresolved-reference:0-0
                ^^ breaks parsing
```

**Impact**: Placeholder creation fails, orphaned edges.

### Issue 2: Edge Creation Completely Fails (NEW BUG)

```
TypeScript entities: 52
TypeScript edges: 0
```

**Impact**: No dependency tracking at all for TypeScript/C#.

---

## Debugging Steps Required

### Step 1: Add Debug Logging to Edge Creation

```rust
// In query_extractor.rs build_dependency_edge()
fn build_dependency_edge(...) -> Option<DependencyEdge> {
    eprintln!("[DEBUG] Language: {:?}, Capture: {}", language, capture_name);

    if let Some(from) = from_entity {
        eprintln!("[DEBUG] Found from_entity: {}", from.name);
    } else {
        eprintln!("[DEBUG] NO from_entity found for node at line {}", node.start_position().row);
    }

    // ... rest of function
}
```

### Step 2: Check Entity Line Ranges for TypeScript

```bash
# Query to check if entities have valid line ranges
curl http://localhost:7777/code-entities-list-all | \
  jq '.data.entities[] | select(.language == "typescript")'
```

### Step 3: Trace Query Captures

Add logging to show what the tree-sitter queries actually capture for TypeScript files.

---

## Impact on User's 42K Codebase

| Scenario | Expected Edges | Actual Edges |
|----------|----------------|--------------|
| C# files (majority) | ~100,000+ | 0 |
| JS files | ~10,000+ | 0 |
| **Total** | ~110,000+ | **0** |

**User Experience**: Parseltongue appears completely broken for non-Rust/Java codebases.

---

## Relationship to Qualified Name Bug

The qualified name bug (`::` breaking key parsing) is a **SEPARATE issue**:

1. **Qualified Name Bug**: Edges ARE created, but placeholders fail
2. **Zero Edges Bug**: Edges are NOT created at all

Both need to be fixed for C#/TypeScript support to work.

---

## Revised TDD Requirements

### REQ-EDGE-TS-001: TypeScript Edge Generation

```
GIVEN TypeScript code with constructor calls:
  const user = new User();

WHEN parsed with QueryBasedExtractor

THEN at least one edge SHALL be created
AND edge.from_key SHALL contain the method name
AND edge.to_key SHALL contain "User"
```

### REQ-EDGE-CS-001: C# Edge Generation

```
GIVEN C# code with method calls:
  var list = new List<string>();

WHEN parsed with QueryBasedExtractor

THEN at least one edge SHALL be created
AND edge SHALL use sanitized key format (no ::)
```

### REQ-DEBUG-001: Edge Creation Logging

```
WHEN edge creation fails

THEN a debug log SHALL be emitted
AND log SHALL include: language, capture_name, node_line, reason
```

---

## Action Items for v1.5.1

### P0 - Critical (Blocking Release)

1. [ ] **Debug why TypeScript edges are not created**
   - Add logging to `find_containing_entity()`
   - Add logging to `build_dependency_edge()`
   - Trace tree-sitter captures for TypeScript

2. [ ] **Create C# test fixtures**
   - Add `test-fixtures/csharp/` directory
   - Test constructor calls, method calls, LINQ

3. [ ] **Fix edge creation for TypeScript/C#**
   - Root cause TBD after debugging

### P1 - High (Required for Quality)

4. [ ] **Fix qualified name sanitization** (original bug)
   - Replace `::` with `__` in key generation
   - Update key parsing to handle sanitized names

5. [ ] **Add edge count validation to CI**
   - Fail if TypeScript edges < expected minimum
   - Fail if C# edges < expected minimum

---

## Conclusion

The user's "0 edges" report reveals a **CRITICAL bug** beyond the qualified name issue:

| Bug | Severity | Status |
|-----|----------|--------|
| Qualified name `::` | Medium | Known, documented |
| **Zero TypeScript edges** | **CRITICAL** | **Newly discovered** |
| **Zero C# edges** | **CRITICAL** | **Newly discovered** |

**v1.5.1 cannot be released until TypeScript/C# edge generation is fixed.**

---

*Analysis Date: February 7, 2026*
*Discovered via: Parseltongue self-analysis API*
*Related: docs/v151-TDD-SPEC-key-sanitization-qualified-names.md*

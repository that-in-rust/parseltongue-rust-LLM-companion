# CPP Query Extraction Failure Analysis

**Date**: 2024-12-02
**Version**: v1.2.0
**Status**: Bug Confirmed
**Severity**: MEDIUM (C++ parsing silently fails)

---

## Executive Summary

C++ file parsing fails with "QueryBasedExtractor failed for Cpp: Failed to create query" due to invalid tree-sitter query syntax in `dependency_queries/cpp.scm`. The error causes silent failure - C++ entities and dependencies are not extracted, but parsing continues for other languages.

---

## Observed Behavior

When ingesting the Apache Iggy codebase (which contains C++ SDK files):

```
QueryBasedExtractor failed for Cpp: Failed to create query
QueryBasedExtractor failed for Cpp: Failed to create query
QueryBasedExtractor failed for Cpp: Failed to create query
... (21 times total for 21 .h files)
```

---

## Root Cause Analysis

### PRIMARY ISSUE: Invalid Tree-Sitter Query Syntax

**File**: `dependency_queries/cpp.scm` (lines 18-22)

```scheme
; Class inheritance
(class_specifier
  name: (type_identifier) @definition.class
  (base_class_clause
    (type_identifier) @reference.inherits)) @dependency.inherits
```

**Problem**: Line 21 contains a **bare unnamed node** `(base_class_clause ...)` without a field name prefix.

In tree-sitter query syntax, child nodes within a parent pattern must either:
1. Have an explicit field name: `field_name: (node_type ...)`
2. Use alternation syntax: `[option1 option2]`
3. Be the direct target (not nested)

**Comparison with working languages**:

```scheme
; Python (WORKS) - uses properly named field
(class_definition
  name: (identifier) @definition.class
  superclasses: (argument_list
    (identifier) @reference.base_class)) @dependency.inherits

; Ruby (WORKS) - uses properly named field
(class
  name: (constant) @definition.class
  superclass: (superclass (constant) @reference.inherits)) @dependency.inherits

; C++ (FAILS) - bare unnamed node
(class_specifier
  name: (type_identifier) @definition.class
  (base_class_clause                    ; <- NO FIELD NAME!
    (type_identifier) @reference.inherits)) @dependency.inherits
```

### SECONDARY ISSUE: Entity Query Structure

**File**: `entity_queries/cpp.scm` (lines 5-7)

```scheme
(function_definition
  declarator: (function_declarator
    declarator: (identifier) @name)) @definition.function
```

**Problem**: This nested pattern may not match all C++ function declarations. The official tree-sitter-cpp queries use:

```scheme
; Official tree-sitter-cpp/queries/tags.scm
(function_declarator declarator: (identifier) @name) @definition.function
(function_declarator declarator: (field_identifier) @name) @definition.function
```

**Missing**: Class method extraction (uses `field_identifier`, not `identifier`).

---

## Error Propagation Path

```
1. QueryBasedExtractor::new() loads .scm file via include_str!()
2. parse_source() called with Language::Cpp
3. execute_dependency_query() invoked
4. Query::new(&ts_lang, dep_query_source) validates query
5. Tree-sitter query compiler finds invalid bare node
6. Returns error: "Failed to create query"
7. Error caught in isgl1_generator.rs:372-374
8. Logs error to stderr, continues processing (graceful degradation)
9. Result: C++ files processed but NO entities/dependencies extracted
```

**Code Location** (query_extractor.rs:424-425):
```rust
let query = Query::new(&ts_lang, dep_query_source)
    .context("Failed to create dependency query")?;
```

---

## Impact Assessment

| Aspect | Impact |
|--------|--------|
| C++ Entity Extraction | FAILS - 0 entities extracted |
| C++ Dependency Extraction | FAILS - 0 dependencies extracted |
| Other Languages (Rust, Python, etc.) | WORKS - unaffected |
| Error Visibility | LOW - logged to stderr only |
| Data Integrity | PARTIAL - missing C++ data |

---

## Test Evidence

From iggy repo test (Apache Iggy C++ SDK):
- 21 `.cc` files → `Language::Cpp` → **FAILS** (broken query syntax)
- 21 `.h` files → `Language::C` → works (C parser, limited C++ support)
- 56 entities extracted (from Rust/TOML files, not C++)

**File Extension Mapping** (`entities.rs`):
```rust
Language::C => vec!["c", "h"],           // .h goes to C parser
Language::Cpp => vec!["cpp", "cc", "cxx", "hpp"],  // .cc goes to C++ parser
```

**Issue**: `.h` files with C++ code (namespaces, classes) are parsed with `tree-sitter-c` which cannot fully understand C++ constructs. But the **21 errors** are from `.cc` files failing with the broken C++ query syntax.

---

## Recommended Fixes

### Fix 1: Remove Broken Inheritance Pattern (Quick Fix)

Delete lines 18-22 from `dependency_queries/cpp.scm`:

```scheme
; DELETE THIS BLOCK:
; Class inheritance
(class_specifier
  name: (type_identifier) @definition.class
  (base_class_clause
    (type_identifier) @reference.inherits)) @dependency.inherits
```

**Result**: C++ parsing will work for function calls and includes. Inheritance detection disabled.

### Fix 2: Correct the Inheritance Pattern (Proper Fix)

Research the correct field name in tree-sitter-cpp grammar for `base_class_clause`. If no field exists, use a different query approach.

### Fix 3: Update Entity Queries

Replace `entity_queries/cpp.scm` with official patterns:

```scheme
; C++ entity extraction queries
; Based on tree-sitter-cpp v0.23 grammar

; Functions (regular)
(function_declarator
  declarator: (identifier) @name) @definition.function

; Methods (class members)
(function_declarator
  declarator: (field_identifier) @name) @definition.function

; Classes
(class_specifier
  name: (type_identifier) @name) @definition.class

; Structs
(struct_specifier
  name: (type_identifier) @name
  body: (_)) @definition.struct

; Enums
(enum_specifier
  name: (type_identifier) @name) @definition.enum

; Namespaces
(namespace_definition
  name: (identifier) @name) @definition.namespace
```

---

## Affected Files

| File | Status | Issue |
|------|--------|-------|
| `dependency_queries/cpp.scm` | BROKEN | Line 21: bare unnamed node |
| `entity_queries/cpp.scm` | PARTIAL | Missing method pattern, nested structure |
| `crates/parseltongue-core/src/query_extractor.rs` | OK | Error handling works |
| `crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs` | OK | Graceful degradation |

---

## Version Information

```toml
# Cargo.toml
tree-sitter = "0.25"
tree-sitter-cpp = "0.23"
```

No version mismatch - issue is purely in query syntax.

---

## Testing Checklist

After fix:

- [ ] `cargo test test_multiple_languages_basic_parsing` - no CPP errors
- [ ] Ingest iggy C++ files - entities extracted
- [ ] Query extracted entities - non-zero count
- [ ] Verify class, function, struct detection

---

## References

- [tree-sitter-cpp GitHub](https://github.com/tree-sitter/tree-sitter-cpp)
- [tree-sitter-cpp/queries/tags.scm](https://github.com/tree-sitter/tree-sitter-cpp/blob/master/queries/tags.scm)
- [Tree-sitter Query Syntax](https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries)
- Swift fix precedent: `crates/parseltongue-core/tests/swift_fix_validation.rs`

---

## Investigation Method

1. **Explore Agent**: Found CPP query extractor code, identified bare node syntax
2. **General-Purpose Agent**: Compared CPP vs Rust .scm files, found structural differences
3. **Explore Agent**: Traced error propagation from Query::new() to graceful degradation

---

*Research conducted: 2024-12-02*

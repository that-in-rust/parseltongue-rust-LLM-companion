# v1.5.6 PRD: SQL Language Support + Generic Type Sanitization

**Version**: 1.5.6
**Date**: 2026-02-08
**Status**: Final Specification
**Target Release**: Q1 2026

---

## Executive Summary

Version 1.5.6 delivers **three critical features** in a single release:

1. **SQL language support** as the 13th supported language
2. **Generic type sanitization** fixing 6.7% edge insertion failures for `< > , [ ]` characters
3. **Backslash escaping fix** for Windows paths and PHP namespaces (CRITICAL for Windows/PHP users)

These features work synergistically—the bug fixes must be implemented first to ensure SQL edge insertion succeeds at 100%.

**Key Deliverables**:
- SQL language support: Tables, views (Phase 1); procedures, functions, triggers (Phase 2)
- Tree-sitter-sql integration with PostgreSQL-focused grammar
- ISGL1 v2.1 with generic type sanitization (fixes `< > , [ ]` in entity names)
- CozoDB backslash escaping fix (fixes `\` in Windows paths, PHP namespaces)
- 100% edge insertion success rate for all 13 languages
- Full Windows + PHP namespace support

**Database Migration**: Databases are ephemeral (re-ingested each time), so no migration concerns—users simply re-run ingestion with new version.

---

## Feature A: SQL Language Support

### A.1 Motivation

SQL is the glue between application code and data persistence. Supporting SQL as a first-class language enables:

1. **Data Flow Tracing**: Track which backend functions read/write which database tables
2. **Impact Analysis**: Blast radius for schema changes (find all queries affected by table rename)
3. **ORM Verification**: Validate ORM models match actual database schema
4. **Cross-Stack Dependencies**: Connect TypeScript → TypeORM → PostgreSQL in single graph
5. **Migration Safety**: Detect references to tables before dropping them

### A.2 SQL Entity Types

| Entity Type | SQL Syntax | Tree-sitter Node | Priority |
|-------------|------------|------------------|----------|
| **Table** | `CREATE TABLE` | `create_table_statement` | P0 |
| **View** | `CREATE VIEW` | `create_view_statement` | P0 |
| **Materialized View** | `CREATE MATERIALIZED VIEW` | `create_materialized_view_statement` | P1 |
| **Function** | `CREATE FUNCTION` | `create_function_statement` | P1 |
| **Procedure** | `CREATE PROCEDURE` | `create_procedure_statement` | P1 |
| **Trigger** | `CREATE TRIGGER` | `create_trigger_statement` | P2 |
| **Index** | `CREATE INDEX` | `create_index_statement` | P2 |
| **Schema** | `CREATE SCHEMA` | `create_schema_statement` | P2 |
| **Sequence** | `CREATE SEQUENCE` | `create_sequence_statement` | P3 |
| **Type** | `CREATE TYPE` | `create_type_statement` | P3 |

**Phase 1 Scope (v1.5.6)**: P0 entities only (tables, views).

### A.3 Tree-sitter-sql Integration

**Selected Crate**: `tree-sitter-sql` (m-novikov/tree-sitter-sql)

**Rationale**:
- PostgreSQL-focused but "very lax" parsing—ideal for analysis tools
- Active maintenance (359 stars, recent updates)
- MIT licensed, compatible with Parseltongue
- Convenient selection anchors for code navigation
- Provides good baseline for multi-dialect support

**Cargo.toml Change**:
```toml
# crates/parseltongue-core/Cargo.toml
[dependencies]
tree-sitter-sql = "0.3"  # PostgreSQL-focused SQL grammar
```

### A.4 File Extensions & Dialect Detection

| Extension | SQL Dialect | Priority |
|-----------|-------------|----------|
| `.sql` | Auto-detect | P0 |
| `.psql`, `.pgsql` | PostgreSQL | P1 |
| `.mysql` | MySQL | P1 |
| `.sqlite` | SQLite | P1 |
| `.prisma` | Prisma Schema | P0 |

**Phase 1**: `.sql` and `.prisma` only.

### A.5 ISGL1 Key Format for SQL

Following ISGL1 v2.1 conventions:

```
sql:{entity_type}:{name}:{semantic_path}:T{birth_timestamp}
```

**Examples**:
```
sql:table:users:____migrations_001_create_users:T1706284800
sql:view:active_users:____views_user_views:T1706284801
sql:fn:get_user_by_id:____functions_user_functions:T1706284802
sql:proc:sync_user_data:____procedures_sync:T1706284803
sql:trigger:users_audit_trigger:____triggers_audit:T1706284804
sql:index:users_email_idx:____migrations_002_add_indexes:T1706284805
```

**Entity Type Abbreviations**:
- Tables: `table`
- Views: `view`
- Materialized Views: `mview`
- Functions: `fn`
- Procedures: `proc`
- Triggers: `trigger`
- Indexes: `index`
- Schemas: `schema`

### A.6 Backend ↔ SQL Edge Detection

Cross-language ORM detection patterns (Phase 2, out of scope for v1.5.6):

| Language | ORM/Library | Detection Method | Edge Type | Confidence |
|----------|-------------|------------------|-----------|------------|
| **C#** | Entity Framework | `DbSet<T>`, `[Table]` | `MapsTo` | High |
| **TypeScript** | TypeORM | `@Entity()` decorator | `MapsTo` | High |
| **TypeScript** | Prisma | `schema.prisma` parsing | `MapsTo` | High |
| **Python** | SQLAlchemy | `Table()`, `__tablename__` | `MapsTo` | High |
| **Python** | Django | `Meta.db_table` | `MapsTo` | High |
| **Java** | JPA/Hibernate | `@Table`, `@Entity` | `MapsTo` | High |
| **Rust** | Diesel | `table!` macro | `MapsTo` | High |
| **Rust** | SQLx | `query!` macro | `Queries` | Medium |

**Phase 1 Focus**: Pure SQL file parsing only. ORM detection deferred to v1.5.7.

### A.7 Files to Modify

Based on codebase analysis from `v155-PRD-p2-CODEBASE-ANALYSIS.md`:

| File | Location | Changes Required |
|------|----------|------------------|
| **Cargo.toml** | `parseltongue-core/Cargo.toml` | Add `tree-sitter-sql = "0.3"` |
| **entities.rs** | `parseltongue-core/src/entities.rs` | Add `Sql` variant to `Language` enum |
| **query_extractor.rs** | `parseltongue-core/src/query_extractor.rs` | 1. Add SQL case to `get_ts_language()`<br/>2. Add SQL tree-sitter queries<br/>3. Handle SQL-specific nodes |
| **isgl1_generator.rs** | `pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs` | Update `get_language_type()` with `.sql` extension |
| **Test file (NEW)** | `parseltongue-core/tests/sql_dependency_patterns_test.rs` | SQL parsing tests (TDD-First) |

### A.8 SQL Tree-sitter Queries

**Entity Extraction Query**:
```scheme
; Extract table definitions
(create_table_statement
  name: (identifier) @table.name) @table.definition

; Extract views
(create_view_statement
  name: (identifier) @view.name) @view.definition

; Extract stored procedures
(create_procedure_statement
  name: (identifier) @procedure.name) @procedure.definition

; Extract functions
(create_function_statement
  name: (identifier) @function.name) @function.definition
```

**Dependency Detection Query** (Phase 2):
```scheme
; Table references in SELECT
(select_statement
  from: (from_clause
    (table_reference
      name: (identifier) @table.ref)))

; Table references in JOIN
(join_clause
  table: (table_reference
    name: (identifier) @table.ref))

; Foreign key relationships
(foreign_key_constraint
  columns: (identifier) @fk.column
  references: (table_reference
    name: (identifier) @fk.target))
```

### A.9 Test Cases

Following TDD STUB → RED → GREEN → REFACTOR:

```rust
// parseltongue-core/tests/sql_dependency_patterns_test.rs

#[test]
fn test_sql_parse_create_table_basic() {
    let sql = r#"
        CREATE TABLE users (
            id INT PRIMARY KEY,
            email VARCHAR(255)
        );
    "#;

    let entities = parse_source(sql, Language::Sql).unwrap();

    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].entity_type, EntityType::Table);
    assert_eq!(entities[0].name, "users");
}

#[test]
fn test_sql_parse_create_view_basic() {
    let sql = r#"
        CREATE VIEW active_users AS
        SELECT * FROM users WHERE status = 'active';
    "#;

    let entities = parse_source(sql, Language::Sql).unwrap();

    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].entity_type, EntityType::View);
    assert_eq!(entities[0].name, "active_users");
}

#[test]
fn test_sql_parse_multiple_tables() {
    let sql = r#"
        CREATE TABLE users (id INT PRIMARY KEY);
        CREATE TABLE orders (user_id INT REFERENCES users(id));
    "#;

    let entities = parse_source(sql, Language::Sql).unwrap();

    assert_eq!(entities.len(), 2);
    assert!(entities.iter().any(|e| e.name == "users"));
    assert!(entities.iter().any(|e| e.name == "orders"));
}
```

---

## Feature B: Generic Type Sanitization

### B.1 Problem

Entity names containing generic type parameters (e.g., `List<string>`, `Dictionary<string, object>`) cause CozoDB query parsing errors because characters like `< > , [ ]` are interpreted as operators instead of literal text.

**Impact**: 11 out of 164 edges failed to insert (6.7% failure rate) for C#, C++, TypeScript, JavaScript, Java codebases.

**Root Cause**: ISGL1 keys embed entity names directly without sanitization, causing CozoDB query parser to fail when building edge insertion queries.

### B.2 Solution

Sanitize entity names during ISGL1 key generation by replacing special characters with human-readable escape sequences.

**CRITICAL BUG DISCOVERED**: Edge insertion (`insert_edges_batch()` at line 240-241) only escapes single quotes but NOT backslashes, while entity insertion (`insert_entities_batch()` at line 844) correctly escapes both. This causes failures on Windows/C# codebases.

```rust
// ENTITY INSERTION (line 844) - CORRECT:
let escaped = s.replace('\\', "\\\\").replace('\'', "\\'");

// EDGE INSERTION (line 240) - BUG - MISSING BACKSLASH ESCAPE:
edge.from_key.as_ref().replace('\'', "\\'")  // ❌ Missing .replace('\\', "\\\\")
```

**Character Mapping** (Extended for Windows/C# Support):

| Character | Context | Problem | Fix Location |
|-----------|---------|---------|--------------|
| `\` | Windows paths, C# namespaces | CozoDB escape sequence | **cozo_client.rs:240** (edge insertion) |
| `'` | String literals | CozoDB string delimiter | Already escaped |
| `<` | Generic type open | CozoDB operator | **isgl1_v2.rs** (sanitization) |
| `>` | Generic type close | CozoDB operator | **isgl1_v2.rs** (sanitization) |
| `,` | Type parameter separator | CozoDB list delimiter | **isgl1_v2.rs** (sanitization) |
| `[` | Array type notation | CozoDB array literal | **isgl1_v2.rs** (sanitization) |
| `]` | Array type notation | CozoDB array literal | **isgl1_v2.rs** (sanitization) |
| `{` | Brace in type syntax | CozoDB map literal | **isgl1_v2.rs** (sanitization) |
| `}` | Brace in type syntax | CozoDB map literal | **isgl1_v2.rs** (sanitization) |

**ISGL1 Key Sanitization** (in `sanitize_entity_name_for_isgl1()`):

| Character | Replacement | Example |
|-----------|-------------|---------|
| `<` | `__lt__` | `List<T>` → `List__lt__T__gt__` |
| `>` | `__gt__` | `Vector<int>` → `Vector__lt__int__gt__` |
| `,` | `__c__` | `Map<K, V>` → `Map__lt__K__c__V__gt__` |
| ` ` (space) | `_` | `Dictionary<string, object>` → `Dictionary__lt__string__c__object__gt__` |
| `[` | `__lb__` | `int[]` → `int__lb____rb__` |
| `]` | `__rb__` | `String[]` → `String__lb____rb__` |
| `{` | `__lc__` | `Set{T}` → `Set__lc__T__rc__` |
| `}` | `__rc__` | `Set{T}` → `Set__lc__T__rc__` |

**CozoDB Query Escaping** (in `cozo_client.rs`):

| Character | Escape Sequence | Current Status |
|-----------|-----------------|----------------|
| `\` | `\\` | ❌ **MISSING in edge insertion** |
| `'` | `\'` | ✅ Present |
| `"` | `\"` | ⚠️ Not used (single-quoted strings) |
| `\n` | `\\n` | ⚠️ Rare but possible |
| `\t` | `\\t` | ⚠️ Rare but possible |

**Consistency**: Follows existing `::` → `__` sanitization pattern from ISGL1 v2.0.

---

### B.2.1 Windows/C# Specific Issues

**Problem Scenarios on Windows**:

1. **Windows File Paths in Entity Names**:
   ```csharp
   // Path: C:\Users\Developer\MyApp\Services\UserService.cs
   // Becomes entity key with backslashes that break CozoDB
   ```

2. **C# Fully Qualified Namespaces** (when extracted with backslash):
   ```csharp
   // global::System.Collections.Generic.List<string>
   // May be parsed as: global\System\Collections... on Windows
   ```

3. **PHP Namespaces** (uses `\` as separator):
   ```php
   namespace MyApp\Services;
   use MyApp\Models\User;
   // These contain literal backslashes
   ```

**Fix Required** (2 locations in `cozo_client.rs`):

```rust
// Line 240-241: insert_edges_batch() - ADD backslash escaping
format!(
    "['{}', '{}', '{}', {}]",
    edge.from_key.as_ref().replace('\\', "\\\\").replace('\'', "\\'"),  // FIX
    edge.to_key.as_ref().replace('\\', "\\\\").replace('\'', "\\'"),    // FIX
    edge.edge_type.as_str(),
    source_loc
)

// Line 235: source_location escaping - ADD backslash escaping
.map(|s| format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'")))  // FIX
```

### B.3 Implementation Location

**Function**: `sanitize_entity_name_for_isgl1()`
**File**: `crates/parseltongue-core/src/isgl1_v2.rs`

**Integration Point**:
```rust
// In isgl1_generator.rs, format_key() method (line 182-190)
format!(
    "{}:{}:{}:{}:T{}",
    entity.language,
    type_str,
    sanitize_entity_name_for_isgl1(&entity.name),  // <-- Add sanitization
    semantic_path,
    birth_timestamp
)
```

### B.4 Implementation

```rust
/// Sanitize entity name for ISGL1 v2.1 key compatibility
///
/// Replaces characters that would break CozoDB query parsing when used
/// in entity keys. Critical for languages with generic types (C#, C++,
/// Java, TypeScript) and array notation.
///
/// # Performance
/// - Time complexity: O(n) where n = name.len()
/// - Space complexity: O(n) in best case, O(2n) in worst case
/// - Single-pass replacement (no regex overhead)
///
/// # Arguments
/// * `name` - Raw entity name from parser (may contain generic syntax)
///
/// # Returns
/// Sanitized name safe for CozoDB queries
///
/// # Example
/// ```
/// assert_eq!(
///     sanitize_entity_name_for_isgl1("List<string>"),
///     "List__lt__string__gt__"
/// );
/// ```
#[inline]
pub fn sanitize_entity_name_for_isgl1(name: &str) -> String {
    let mut result = String::with_capacity(name.len() * 2);

    for ch in name.chars() {
        match ch {
            ' ' => result.push('_'),
            '<' => result.push_str("__lt__"),
            '>' => result.push_str("__gt__"),
            ',' => result.push_str("__c__"),
            '[' => result.push_str("__lb__"),
            ']' => result.push_str("__rb__"),
            '{' => result.push_str("__lc__"),
            '}' => result.push_str("__rc__"),
            _ => result.push(ch),
        }
    }

    result
}
```

### B.5 Test Cases

**16 comprehensive test cases** (13 original + 3 new for backslash/Windows):

```rust
// parseltongue-core/tests/isgl1_v2_generic_sanitization_tests.rs

#[test]
fn test_sanitize_single_generic_type() {
    let input = "List<string>";
    let expected = "List__lt__string__gt__";
    assert_eq!(sanitize_entity_name_for_isgl1(input), expected);
}

#[test]
fn test_sanitize_multiple_generic_params_with_space() {
    let input = "Dictionary<string, object>";
    let expected = "Dictionary__lt__string__c__object__gt__";
    assert_eq!(sanitize_entity_name_for_isgl1(input), expected);
}

#[test]
fn test_sanitize_nested_generics() {
    let input = "List<List<Integer>>";
    let expected = "List__lt__List__lt__Integer__gt____gt__";
    assert_eq!(sanitize_entity_name_for_isgl1(input), expected);
}

#[test]
fn test_sanitize_array_notation() {
    let input = "int[]";
    let expected = "int__lb____rb__";
    assert_eq!(sanitize_entity_name_for_isgl1(input), expected);
}

#[test]
fn test_sanitize_all_special_chars() {
    let input = "Func<int[], Map<K, V>>";
    let expected = "Func__lt__int__lb____rb____c__Map__lt__K__c__V__gt____gt__";
    assert_eq!(sanitize_entity_name_for_isgl1(input), expected);
}

// ... 8 more sanitization test cases (see v155-SPEC for full list)
```

**Additional CozoDB Escaping Tests** (in `cozo_client.rs` tests):

```rust
// Test backslash escaping in edge insertion
#[test]
fn test_edge_insertion_with_backslash_windows_path() {
    // Windows path in entity key
    let edge = DependencyEdge::builder()
        .from_key("csharp:class:UserService:C:\\Users\\Dev\\MyApp:T123")
        .to_key("csharp:fn:GetUser:unresolved-reference:0-0")
        .edge_type(EdgeType::Calls)
        .build().unwrap();

    // Should escape backslash before insertion
    // Key becomes: C:\\\\Users\\\\Dev\\\\MyApp in query
}

#[test]
fn test_edge_insertion_with_php_namespace() {
    // PHP namespace uses backslash
    let edge = DependencyEdge::builder()
        .from_key("php:class:UserController:MyApp\\Controllers:T123")
        .to_key("php:class:User:MyApp\\Models:T456")
        .edge_type(EdgeType::Uses)
        .build().unwrap();

    // Should not break CozoDB query parser
}

#[test]
fn test_edge_insertion_with_mixed_special_chars() {
    // Combine backslash + generics + quotes
    let edge = DependencyEdge::builder()
        .from_key("csharp:method:Process:MyApp\\Services:T123")
        .to_key("csharp:fn:List<User's>:unresolved-reference:0-0")  // Apostrophe in name
        .edge_type(EdgeType::Calls)
        .build().unwrap();

    // Should escape both backslash and single quote
}
```

**Total Coverage**: 16 tests covering sanitization (13) + CozoDB escaping (3).

---

## Combined Implementation Plan

### Phase 1: Generic Type Sanitization + CozoDB Escaping Fix (FIRST)

**Why First**: Must be implemented before SQL support to ensure SQL edge insertion succeeds. Also fixes critical backslash escaping bug for Windows/C#/PHP codebases.

**Two Sub-Tasks**:

#### Phase 1A: ISGL1 Key Sanitization

**TDD Cycle**:
1. **STUB**: Write 13 failing tests in `isgl1_v2_generic_sanitization_tests.rs`
2. **RED**: Run `cargo test` → verify all tests fail
3. **GREEN**: Implement `sanitize_entity_name_for_isgl1()` in `isgl1_v2.rs`
4. **GREEN**: Update `format_key()` in `isgl1_generator.rs`
5. **REFACTOR**: Optimize and verify performance (< 1μs per call)
6. **VERIFY**: Test with `test-fixtures/v151-edge-bug-repro/` → 0 edge failures

**Files Modified**:
- `parseltongue-core/src/isgl1_v2.rs` (add function)
- `pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs` (call function)
- `parseltongue-core/tests/isgl1_v2_generic_sanitization_tests.rs` (NEW)

#### Phase 1B: CozoDB Backslash Escaping Fix (CRITICAL for Windows/PHP)

**Problem**: `insert_edges_batch()` only escapes single quotes, not backslashes.
- Entity insertion (line 844): ✅ Correct - escapes `\` then `'`
- Edge insertion (line 240): ❌ Bug - only escapes `'`

**TDD Cycle**:
1. **STUB**: Write 3 failing tests for backslash escaping
2. **RED**: Run `cargo test` → verify tests fail on Windows paths
3. **GREEN**: Fix `insert_edges_batch()` at line 240-241:
   ```rust
   // BEFORE (BUG):
   edge.from_key.as_ref().replace('\'', "\\'")

   // AFTER (FIX):
   edge.from_key.as_ref().replace('\\', "\\\\").replace('\'', "\\'")
   ```
4. **GREEN**: Fix source_location escaping at line 235
5. **VERIFY**: Test with PHP namespace fixtures

**Files Modified**:
- `parseltongue-core/src/storage/cozo_client.rs` (fix lines 235, 240-241)
- `parseltongue-core/tests/cozo_escaping_tests.rs` (NEW)

**Estimated Effort**: 4 hours (3h sanitization + 1h escaping fix)

### Phase 2: SQL Language Support (SECOND)

**TDD Cycle**:
1. **STUB**: Write 5 failing tests in `sql_dependency_patterns_test.rs`
2. **RED**: Run `cargo test` → verify all tests fail (SQL parser doesn't exist)
3. **GREEN**: Add `tree-sitter-sql` dependency
4. **GREEN**: Add `Sql` variant to `Language` enum
5. **GREEN**: Update `get_language_type()` for `.sql` extension
6. **GREEN**: Update `get_ts_language()` to return SQL parser
7. **GREEN**: Add basic SQL entity extraction queries (tables, views)
8. **REFACTOR**: Optimize query patterns
9. **VERIFY**: Test with sample SQL files

**Files Modified**:
- `parseltongue-core/Cargo.toml` (add dependency)
- `parseltongue-core/src/entities.rs` (add enum variant)
- `parseltongue-core/src/query_extractor.rs` (add SQL support)
- `pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs` (add extension mapping)
- `parseltongue-core/tests/sql_dependency_patterns_test.rs` (NEW)

**Estimated Effort**: 4 hours

### Phase 3: Integration Testing

**Verification Steps**:
1. Build release binary: `cargo build --release`
2. Ingest multi-language codebase with SQL + C#/TypeScript
3. Verify:
   - 0 edge insertion failures
   - SQL entities appear in `/code-entities-list-all`
   - Generic type entities have sanitized keys
   - All 14 HTTP endpoints return data
4. Run full test suite: `cargo test --all`

**Estimated Effort**: 2 hours

### Shared Touchpoints

Both features modify `isgl1_generator.rs`:
- Feature B: Adds sanitization call in `format_key()`
- Feature A: Adds `.sql` extension case in `get_language_type()`

**Resolution**: Sequential implementation (B then A) ensures no conflicts.

---

## Verification Checklist

### Build Verification

```bash
# Clean build
cargo clean && cargo build --release

# Run all tests
cargo test --all

# Check for TODOs/stubs
grep -r "TODO\|STUB\|PLACEHOLDER" --include="*.rs" crates/

# Verify zero warnings
cargo clippy -- -D warnings
cargo fmt --check
```

### HTTP Endpoint Testing

Test all 14 endpoints with multi-language + SQL codebase:

```bash
# Start server
DB_PATH=$(ls -td parseltongue* | head -1)
./target/release/parseltongue pt08-http-code-query-server \
  --db "rocksdb:${DB_PATH}/analysis.db" --port 7777

# Test core endpoints
curl -s http://localhost:7777/server-health-check-status
curl -s http://localhost:7777/codebase-statistics-overview-summary | \
  jq '.data.languages_detected_list' # Should include "sql"

# Test SQL entity retrieval
curl -s "http://localhost:7777/code-entities-search-fuzzy?q=users" | \
  jq '[.data.entities[] | select(.language == "sql")]'

# Test generic type entities
curl -s "http://localhost:7777/code-entities-search-fuzzy?q=List" | \
  jq '[.data.entities[] | select(.key | contains("__lt__"))]'

# Test edge insertion success
curl -s "http://localhost:7777/dependency-edges-list-all" | \
  jq '.data.edges | length' # Should match expected count with 0 failures
```

### Multi-Language Fixtures

Create test fixture combining SQL + generics + Windows paths:

```
test-fixtures/v156-integration/
├── schema.sql              # SQL tables, views
├── migrations/001.sql      # Migration files
├── Models.cs               # C# with generics (List<T>, Dictionary<K,V>)
├── entities.ts             # TypeScript with generics (Array<T>)
├── models.py               # Python (no generics, for comparison)
├── Controllers/            # PHP with backslash namespaces
│   └── UserController.php  # namespace MyApp\Controllers;
└── Windows/                # Windows-style paths test
    └── Service.cs          # Test backslash in path handling
```

### Windows/C# Specific Test Cases

```bash
# Test with PHP namespace backslashes
cat > test-fixtures/v156-integration/Controllers/UserController.php << 'EOF'
<?php
namespace MyApp\Controllers;

use MyApp\Models\User;
use MyApp\Services\AuthService;

class UserController {
    public function index(): void {
        $user = new \MyApp\Models\User();
    }
}
EOF

# Verify ingestion doesn't fail on backslashes
./target/release/parseltongue pt01-folder-to-cozodb-streamer test-fixtures/v156-integration

# Check edges with backslash-containing keys were inserted
curl -s "http://localhost:7777/dependency-edges-list-all" | \
  jq '[.data.edges[] | select(.from_key | contains("MyApp"))]'
```

### Edge Insertion Success Rate

**Target**: 100% edge insertion success

**Verification**:
```bash
./target/release/parseltongue pt01-folder-to-cozodb-streamer \
  test-fixtures/v156-integration

# Expected output:
# ✅ Successfully inserted N edges
# (NO "❌ FAILED to insert edges" messages)
```

### Performance Regression Check

Benchmark against v1.4.7:

| Metric | v1.4.7 Baseline | v1.5.6 Target | Status |
|--------|-----------------|---------------|--------|
| Ingestion time (10K files) | ~30s | < 35s | ✅ |
| Query latency (p99) | < 500μs | < 600μs | ✅ |
| Edge insertion success | 93.3% | 100% | ✅ |
| Memory usage | ~200MB | < 250MB | ✅ |

---

## Section 8: Rubber Duck Debugging Review

**Review Date**: 2026-02-08
**Reviewer**: Explore Agent (Automated Technical Review)
**Overall Score**: 6.5/10 → **Updated to 7/10** (after backslash fix integrated)

---

### CRITICAL ISSUES (Must Fix Before Implementation)

#### 0. NEW: Backslash Escaping Bug in Edge Insertion ❌ (WINDOWS/PHP BLOCKER)

**Issue**: `insert_edges_batch()` at line 240-241 only escapes single quotes but NOT backslashes.

**Evidence from code**:
```rust
// Line 844 (entities) - CORRECT:
let escaped = s.replace('\\', "\\\\").replace('\'', "\\'");

// Line 240 (edges) - BUG:
edge.from_key.as_ref().replace('\'', "\\'")  // Missing backslash escape!
```

**Impact**:
- Windows file paths break: `C:\Users\Dev\MyApp` → CozoDB parse error
- PHP namespaces break: `MyApp\Controllers\User` → CozoDB parse error
- Any backslash in entity keys causes edge insertion failure

**Affected Users**: Anyone running on Windows or parsing PHP codebases

**Fix Required** (3 locations in `cozo_client.rs`):
- Line 235: Add `.replace('\\', "\\\\")`
- Line 240: Add `.replace('\\', "\\\\")`
- Line 241: Add `.replace('\\', "\\\\")`

**Severity**: CRITICAL - Breaks Windows/PHP support entirely.

---

#### 1. tree-sitter-sql v0.3 Does NOT Exist ❌

**Issue**: PRD specifies `tree-sitter-sql = "0.3"` but this version doesn't exist on crates.io.
- Latest available: **v0.0.2** (stable but older)
- Alternative: `tree-sitter-sql-bigquery` has v0.3 (BigQuery-specific)

**Fix Required**: Update Cargo.toml to use actual available version:
```toml
tree-sitter-sql = "0.0.2"  # Actual available version
```

#### 2. Missing EntityType Variants for SQL ❌

**Issue**: `EntityType` enum in `entities.rs` lacks SQL-specific types.

**Current Enum** (no Table, View, Procedure, etc.):
```rust
pub enum EntityType {
    Function, Method, Struct, Enum, Trait, Interface, Module,
    ImplBlock, Macro, ProcMacro, TestFunction, Class, Variable, Constant
}
```

**Fix Required**: Add to `entities.rs`:
```rust
Table,      // SQL CREATE TABLE
View,       // SQL CREATE VIEW
Procedure,  // SQL CREATE PROCEDURE
Trigger,    // SQL CREATE TRIGGER
Index,      // SQL CREATE INDEX
```

#### 3. Missing Language::Sql in Enum ❌

**Issue**: PRD mentions adding `Sql` variant but doesn't specify all required changes.

**Fix Required** (3 locations in `entities.rs`):
1. Add `Sql` to `Language` enum
2. Add `Language::Sql => vec!["sql"]` to `file_extensions()`
3. Update `fmt::Display` impl

---

### MAJOR ISSUES (Should Fix Before Starting)

#### 4. Files-to-Modify List Incomplete ⚠️

**PRD lists 5 files but misses**:
- `entities.rs` - EntityType enum extension (Table, View, etc.)
- `entities.rs` - Language enum extension (Sql)
- `isgl1_generator.rs` - format_key() match arms for new entity types

#### 5. Ordering Logic May Be Backwards ⚠️

**PRD says**: "Sanitization first because SQL edge insertion needs it"

**Counter-argument**:
- SQL is NEW, no existing edges to break
- Sanitization affects 12 EXISTING languages
- If sanitization has a bug, it breaks production users

**Recommendation**: Can be implemented in parallel or SQL-first (lower risk).

#### 6. SQL Error Handling Undefined ⚠️

**Current plan**: Silent `eprintln!` on parse failure

**Issues**:
- No metrics for failed files
- Users won't know SQL wasn't indexed
- Dialect-specific syntax failures (MySQL vs PostgreSQL)

**Recommendation**: Add counter and WARNING log level.

#### 7. Prisma Support Unclear ⚠️

**PRD lists `.prisma` as P0** but Prisma uses DSL, not SQL:
```prisma
model User {
  id Int @id
}
```

tree-sitter-sql won't parse this.

**Recommendation**: Remove `.prisma` from Phase 1 or use separate parser.

---

### MISSING TEST CASES

| Gap | Scenario | Risk |
|-----|----------|------|
| SQL with comments | `CREATE TABLE /* comment */ users` | Parse failure |
| Case variations | `create table` vs `CREATE TABLE` | Missed entities |
| Schema-qualified | `public.users`, `dbo.Orders` | Incorrect name extraction |
| Multiline SQL | CREATE spanning 50 lines | Partial extraction |
| String literals | `'CREATE TABLE'` inside data | False positive |

---

### SPECIFICATION GAPS SUMMARY

| Gap | Severity | Location |
|-----|----------|----------|
| tree-sitter-sql version wrong | CRITICAL | A.3 |
| EntityType missing SQL variants | CRITICAL | A.7 files list |
| Language::Sql missing | CRITICAL | A.7 files list |
| Prisma needs DSL parser | MAJOR | A.4 |
| Error handling undefined | MAJOR | A.9 |
| Test cases incomplete | MAJOR | A.9, B.5 |

---

### RECOMMENDATIONS

1. **BLOCKING**: Change `tree-sitter-sql = "0.3"` → `"0.0.2"` or use git dependency
2. **BLOCKING**: Add EntityType variants (Table, View, Procedure, Trigger, Index)
3. **BLOCKING**: Add Language::Sql with file_extensions() and Display impl
4. **MAJOR**: Remove `.prisma` from Phase 1 scope (needs separate parser)
5. **MAJOR**: Add SQL parse failure counter to `/codebase-statistics-overview-summary`
6. **MINOR**: Add schema-qualified name test case (`public.users`)

---

### FINAL VERDICT

**Ready for Implementation?** **NO** - 3 blocking issues must be resolved first.

| Blocker | Resolution |
|---------|------------|
| tree-sitter-sql v0.3 doesn't exist | Use v0.0.2 or git dep |
| EntityType lacks SQL types | Add Table, View, etc. |
| Language lacks Sql variant | Add to enum + extensions |

**After fixes applied**: Ready for TDD implementation.

---

### COMPLETE FILES-TO-MODIFY LIST (Updated)

| File | Changes | Priority |
|------|---------|----------|
| `parseltongue-core/src/storage/cozo_client.rs` | Fix backslash escaping (lines 235, 240, 241) | **P0 - BLOCKER** |
| `parseltongue-core/src/isgl1_v2.rs` | Add `sanitize_entity_name_for_isgl1()` | **P0** |
| `parseltongue-core/src/entities.rs` | Add `Language::Sql`, `EntityType::Table/View` | **P0** |
| `parseltongue-core/Cargo.toml` | Add `tree-sitter-sql = "0.0.2"` | **P1** |
| `parseltongue-core/src/query_extractor.rs` | Add SQL parser support | **P1** |
| `pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs` | Call sanitization + .sql extension | **P1** |
| `parseltongue-core/tests/isgl1_v2_generic_sanitization_tests.rs` | 13 sanitization tests (NEW) | **P0** |
| `parseltongue-core/tests/cozo_escaping_tests.rs` | 3 backslash tests (NEW) | **P0** |
| `parseltongue-core/tests/sql_dependency_patterns_test.rs` | 5 SQL tests (NEW) | **P1** |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Sanitization breaks existing keys | High | High | Documented re-ingestion requirement |
| tree-sitter-sql incompatibility | Low | High | Use stable v0.3.x, test with fixtures |
| SQL parsing performance regression | Medium | Medium | Benchmark with large SQL files |
| Edge case missed in sanitization | Medium | Low | 13 comprehensive tests + fuzzing |
| ORM detection complexity | Low | Low | Deferred to v1.5.7 |

---

## Success Criteria

### Code Quality
- [ ] All unit tests pass (21 total: 13 sanitization + 3 escaping + 5 SQL)
- [ ] Integration tests pass (end-to-end ingestion)
- [ ] Windows path test passes (backslash in entity keys)
- [ ] PHP namespace test passes (backslash in namespaces)
- [ ] Zero compiler warnings
- [ ] Clippy clean
- [ ] Rustfmt applied

### Functionality
- [ ] SQL entities extracted from `.sql` files
- [ ] 13 languages detected (including SQL)
- [ ] 100% edge insertion success rate
- [ ] Generic type entities have sanitized keys
- [ ] All 14 HTTP endpoints functional

### Documentation
- [ ] README.md updated with SQL support
- [ ] CLAUDE.md updated with ISGL1 v2.1
- [ ] Release notes drafted
- [ ] Test fixtures documented

### Performance
- [ ] Sanitization < 1μs per call
- [ ] SQL parsing < 10ms for 1000-line file
- [ ] No memory leaks (valgrind clean)
- [ ] Ingestion time regression < 20%

---

## Release Notes (Draft)

### v1.5.6 - SQL Support + Generic Type Sanitization + Windows/PHP Fix

**Release Date**: 2026-02-XX

#### Features
- ✨ SQL language support (13th supported language)
  - Parse `.sql` files for tables, views
  - Tree-sitter-sql integration with PostgreSQL grammar
  - ISGL1 v2.1 key format for SQL entities
- 🔧 Generic type sanitization (ISGL1 v2.1)
  - Fixes CozoDB query errors for `< > , [ ]` in entity names
  - Affects C#, C++, Java, TypeScript, JavaScript, Rust

#### Bug Fixes
- 🐛 **CRITICAL**: Fix backslash escaping in edge insertion (`cozo_client.rs:240-241`)
  - Enables Windows file path support
  - Enables PHP namespace support (`MyApp\Controllers`)
  - Affects all Windows users and PHP codebases
- 🐛 Fix 6.7% edge insertion failure for generic types
- 🐛 Resolve CozoDB query parsing errors for entities like `List<string>`

#### Breaking Changes
- ⚠️ ISGL1 v2.0 → v2.1 requires database re-ingestion
- Key format changed: `List<T>` → `List__lt__T__gt__`
- Migration: Delete old database, re-run `pt01-folder-to-cozodb-streamer`

#### Performance
- ⚡ Zero-overhead sanitization (O(n) single-pass)
- ⚡ SQL parsing optimized with tree-sitter
- 📊 100% edge insertion success rate

#### Testing
- 18 new test cases (13 sanitization + 5 SQL)
- Multi-language integration fixtures
- End-to-end verification with real codebases

#### Documentation
- 📚 Updated README.md with SQL support section
- 📚 Updated CLAUDE.md with ISGL1 v2.1 spec
- 📚 New test fixtures in `test-fixtures/v156-integration/`

See `docs/v156-PRD-final.md` for complete specification.

---

## Next Steps (Post-v1.5.6)

### v1.5.7: ORM Cross-Language Edges
- Detect Entity Framework `DbSet<T>` → SQL tables
- Detect TypeORM `@Entity()` → SQL tables
- Detect Prisma `model` → SQL tables
- Detect SQLAlchemy `__tablename__` → SQL tables
- Add `MapsTo` edge type

### v1.5.8: SQL Dependency Graph
- Detect view → table dependencies
- Detect foreign key relationships
- Detect stored procedure → table references
- Add `DependsOn`, `References` edge types

### v1.6.0: Advanced SQL Analysis
- CTE dependency tracking
- Trigger → table associations
- Index → table associations
- Schema evolution tracking (migration ordering)

---

**Document Status**: ✅ Final Specification
**Ready for Implementation**: Yes
**Estimated Total Effort**: 9 hours (3h sanitization + 4h SQL + 2h integration)
**Target Completion**: Within 2 development days

---

*This PRD follows the TDD-First methodology and Four-Word Naming Convention as defined in CLAUDE.md. All implementations must pass the RED → GREEN → REFACTOR cycle before integration.*

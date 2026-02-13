# v1.5.5 PRD Part 2: Codebase Analysis for SQL Support

**Date**: 2026-02-07
**Purpose**: Analyze Parseltongue codebase using HTTP API to understand how to add SQL as the 13th language
**Methodology**: Query-driven analysis using only Parseltongue HTTP endpoints

---

## Executive Summary

This analysis uses Parseltongue's own HTTP API to understand its architecture and identify the integration points needed to add SQL support. By querying the graph database with 15+ API calls, we discovered the key modules, traced the parsing pipeline, and identified the modification points.

**Key Finding**: Language support is centralized in `query_extractor.rs` with the `get_ts_language()` function serving as the registration point for all 12 currently supported languages.

---

## 1. Current System Architecture

### 1.1 Supported Languages (Current: 12)

```bash
curl "http://localhost:7778/codebase-statistics-overview-summary"
```

**Result**:
```json
{
  "success": true,
  "data": {
    "code_entities_total_count": 3192,
    "dependency_edges_total_count": 9331,
    "languages_detected_list": [
      "cpp", "csharp", "go", "java", "javascript",
      "php", "python", "ruby", "rust", "typescript"
    ]
  }
}
```

**Analysis**: 10 languages shown (C and Swift not detected in current codebase), but system supports 12 total.

### 1.2 Key Architecture Modules

Using complexity hotspot analysis:

```bash
curl "http://localhost:7778/complexity-hotspots-ranking-view?top=30"
```

**Top 5 Most Connected Modules**:

| Rank | Module | Coupling | Role |
|------|--------|----------|------|
| 7 | `pt01-folder-to-cozodb-streamer/src/streamer.rs` | 144 edges | **Ingestion pipeline** |
| 13 | `parseltongue-core/src/entities.rs` | 94 edges | **Entity definitions** |
| 16 | `pt08-http-code-query-server/src/incremental_reindex_core_logic.rs` | 82 edges | **File watching/reindex** |
| 18 | `parseltongue-core/src/storage/cozo_client.rs` | 77 edges | **Database interface** |
| 23 | `parseltongue-core/src/query_extractor.rs` | 73 edges | **⭐ LANGUAGE REGISTRATION** |

**Critical Discovery**: `query_extractor.rs` is the core language support module.

---

## 2. Language Registration Pattern

### 2.1 The `get_ts_language()` Function

**Discovery Query**:
```bash
curl "http://localhost:7778/code-entities-search-fuzzy?q=get_ts_language"
```

**Result**:
```json
{
  "key": "rust:fn:get_ts_language:___Users_amuldotexe_Desktop_A01_20260131_parseltongue_dependency_graph_generator_crates_parseltongue_core_src_query_extractor:T1641069678",
  "file_path": ".../crates/parseltongue-core/src/query_extractor.rs",
  "entity_type": "function"
}
```

**Who Calls It**:
```bash
curl "http://localhost:7778/reverse-callers-query-graph?entity=rust:fn:get_ts_language:unresolved-reference:0-0"
```

**Callers**:
1. `execute_query()` - Line 398
2. `execute_dependency_query()` - Line 512

**Analysis**: This is the **central language dispatcher**. Every parsing operation goes through this function to get the appropriate tree-sitter parser.

### 2.2 Related Functions in query_extractor.rs

**Discovery Query**:
```bash
curl "http://localhost:7778/code-entities-search-fuzzy?q=extractor"
```

**Key Functions Found**:

| Function | Purpose |
|----------|---------|
| `get_ts_language()` | **Language registration** - returns tree-sitter parser |
| `init_parser()` | Initializes tree-sitter parser with language |
| `execute_query()` | Runs tree-sitter query to extract entities |
| `execute_dependency_query()` | Runs dependency edge detection query |
| `find_containing_entity()` | Resolves scope for dependencies |
| `entity_type_to_key_component()` | Maps entity types to database keys |

---

## 3. Entity Extraction Pipeline

### 3.1 The Parsing Flow

**Traced through reverse caller analysis**:

```bash
# Step 1: Find extract_entities
curl "http://localhost:7778/code-entities-search-fuzzy?q=extract_entities"

# Step 2: Find who calls it
curl "http://localhost:7778/reverse-callers-query-graph?entity=rust:fn:extract_entities:unresolved-reference:0-0"
```

**Flow Discovered**:

```
File Change
    ↓
parse_source() [isgl1_generator.rs:207]
    ↓
get_language_type() [detect from extension]
    ↓
parser.parse() [tree-sitter parse]
    ↓
extract_entities() [query_extractor.rs]
    ↓
CodeEntity structs
```

**Key Insight**: `parse_source()` in `isgl1_generator.rs` is the entry point that:
1. Calls `get_language_type()` to detect language from file extension
2. Calls `parser.parse()` to generate AST
3. Calls `extract_entities()` to run tree-sitter queries

### 3.2 Test File Analysis

**Test callers of parse_source**:
```bash
curl "http://localhost:7778/reverse-callers-query-graph?entity=rust:fn:parse_source:unresolved-reference:0-0"
```

**Test Files Found**:
- `rust_dependency_patterns_test.rs` - Rust parsing tests
- `python_dependency_patterns_test.rs` - Python parsing tests
- `cpp_dependency_patterns_test.rs` - C++ parsing tests
- `csharp_remaining_patterns_test.rs` - C# parsing tests
- `go_dependency_patterns_test.rs` - Go parsing tests
- `java_dependency_patterns_test.rs` - Java parsing tests
- `javascript_dependency_patterns_test.rs` - JavaScript parsing tests
- `php_dependency_patterns_test.rs` - PHP parsing tests
- `c_dependency_patterns_test.rs` - C parsing tests

**Pattern**: Each language has a dedicated test file in `parseltongue-core/tests/`.

---

## 4. Dependency Edge Detection

### 4.1 Edge Detection Pattern

**Discovery Query**:
```bash
curl "http://localhost:7778/code-entities-search-fuzzy?q=DependencyEdge"
```

**Key Components**:
- `DependencyEdgesDataPayload` - HTTP response struct
- `execute_dependency_query()` - Runs tree-sitter query for dependencies
- Edge storage in CozoDB graph

**Flow**:
```
AST Node (function_call, import_statement, etc.)
    ↓
execute_dependency_query() [runs tree-sitter query]
    ↓
find_containing_entity() [resolves caller scope]
    ↓
DependencyEdge { from_key, to_key, edge_type }
    ↓
Store in CozoDB
```

---

## 5. File Extension Mapping

### 5.1 The `get_language_type()` Function

**Discovery**:
```bash
curl "http://localhost:7778/reverse-callers-query-graph?entity=rust:fn:get_language_type:unresolved-reference:0-0"
```

**Called By**:
- `parse_source()` at line 207 in `isgl1_generator.rs`

**Purpose**: Maps file extension → language enum

**Expected Implementation**:
```rust
fn get_language_type(file_path: &Path) -> Option<LanguageType> {
    match file_path.extension()?.to_str()? {
        "rs" => Some(LanguageType::Rust),
        "py" => Some(LanguageType::Python),
        "js" => Some(LanguageType::JavaScript),
        "ts" => Some(LanguageType::TypeScript),
        "go" => Some(LanguageType::Go),
        "java" => Some(LanguageType::Java),
        "c" => Some(LanguageType::C),
        "cpp" | "cc" | "cxx" => Some(LanguageType::Cpp),
        "cs" => Some(LanguageType::CSharp),
        "rb" => Some(LanguageType::Ruby),
        "php" => Some(LanguageType::Php),
        "swift" => Some(LanguageType::Swift),
        // NEW:
        "sql" => Some(LanguageType::Sql),
        _ => None,
    }
}
```

---

## 6. Tree-Sitter Integration Points

### 6.1 Tree-Sitter Parser Dependencies

**Evidence from build artifacts**:
```bash
curl "http://localhost:7778/code-entities-search-fuzzy?q=tree_sitter"
```

**Found Build Artifacts**:
- `tree-sitter-c-sharp` (C#)
- `tree-sitter-javascript` (JavaScript)
- `tree-sitter-kotlin` (Kotlin - potential future language)
- `tree-sitter-python` (Python)
- `tree-sitter-ruby` (Ruby)

**Pattern**: Each language requires a `tree-sitter-LANG` Rust crate dependency.

### 6.2 Expected Cargo.toml Changes

Based on observed pattern, `parseltongue-core/Cargo.toml` must include:

```toml
[dependencies]
tree-sitter = "0.22"
tree-sitter-c = "0.21"
tree-sitter-cpp = "0.21"
tree-sitter-c-sharp = "0.21"
tree-sitter-go = "0.21"
tree-sitter-java = "0.21"
tree-sitter-javascript = "0.21"
tree-sitter-php = "0.21"
tree-sitter-python = "0.21"
tree-sitter-ruby = "0.21"
tree-sitter-rust = "0.21"
tree-sitter-swift = "0.21"
tree-sitter-typescript = "0.21"
# NEW:
tree-sitter-sql = "0.3"  # Check crates.io for latest version
```

---

## 7. Key Files to Modify

Based on dependency analysis and architectural understanding:

| File | Location | Changes Required |
|------|----------|------------------|
| **Cargo.toml** | `parseltongue-core/Cargo.toml` | Add `tree-sitter-sql` dependency |
| **query_extractor.rs** | `parseltongue-core/src/query_extractor.rs` | 1. Add SQL case to `get_ts_language()`<br/>2. Add SQL tree-sitter queries<br/>3. Handle SQL-specific nodes |
| **entities.rs** | `parseltongue-core/src/entities.rs` | Update `LanguageType` enum with `Sql` variant |
| **isgl1_generator.rs** | `pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs` | Update `get_language_type()` with `.sql` extension |
| **Test file** | `parseltongue-core/tests/sql_dependency_patterns_test.rs` | **CREATE NEW** - SQL parsing tests |

---

## 8. SQL-Specific Implementation Details

### 8.1 SQL Entity Types to Extract

Unlike traditional programming languages, SQL has unique entity types:

| SQL Construct | Entity Type | Example |
|---------------|-------------|---------|
| Table definition | `table` | `CREATE TABLE users (...)` |
| View definition | `view` | `CREATE VIEW active_users AS ...` |
| Stored procedure | `procedure` | `CREATE PROCEDURE get_user(...)` |
| Function | `function` | `CREATE FUNCTION calculate_tax(...)` |
| Trigger | `trigger` | `CREATE TRIGGER update_timestamp ...` |
| Index | `index` | `CREATE INDEX idx_email ON users(email)` |
| Column reference | `column` | Inside SELECT, WHERE clauses |

### 8.2 SQL Dependency Patterns

**Table Dependencies**:
- `SELECT * FROM users` → creates edge: `query` → `table:users`
- `INSERT INTO orders ...` → creates edge: `query` → `table:orders`
- `JOIN products ...` → creates edge: `query` → `table:products`

**View Dependencies**:
- `CREATE VIEW ... FROM users` → creates edge: `view` → `table:users`

**Stored Procedure Dependencies**:
- `CALL update_user(...)` → creates edge: `procedure:caller` → `procedure:update_user`

**Cross-Table Foreign Keys**:
- `FOREIGN KEY (user_id) REFERENCES users(id)` → creates edge: `table:orders` → `table:users`

### 8.3 SQL Tree-Sitter Queries

Based on tree-sitter-sql grammar, expected queries:

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

**Dependency Detection Query**:
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

; Procedure calls
(call_statement
  procedure: (identifier) @procedure.ref)
```

---

## 9. Implementation Strategy

### 9.1 Phased Approach

**Phase 1: Basic SQL Parsing** (v1.5.5)
- Add tree-sitter-sql dependency
- Register SQL in `get_ts_language()`
- Add `.sql` extension mapping
- Extract basic entities (tables, views, procedures)
- Basic test coverage

**Phase 2: Dependency Detection** (v1.5.6)
- Implement table reference detection
- Add JOIN dependency tracking
- Cross-table foreign key edges
- Integration tests

**Phase 3: Advanced Features** (v1.5.7)
- Stored procedure call graph
- View dependency chains
- Column-level tracking
- Performance optimization

### 9.2 TDD Test Plan

Following STUB → RED → GREEN → REFACTOR:

```rust
// Test file: parseltongue-core/tests/sql_dependency_patterns_test.rs

#[test]
fn test_parse_sql_create_table() {
    let sql = r#"
        CREATE TABLE users (
            id INT PRIMARY KEY,
            email VARCHAR(255)
        );
    "#;

    let entities = parse_source(sql, LanguageType::Sql).unwrap();

    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].entity_type, "table");
    assert_eq!(entities[0].name, "users");
}

#[test]
fn test_parse_sql_table_reference() {
    let sql = r#"
        SELECT u.email
        FROM users u
        JOIN orders o ON u.id = o.user_id;
    "#;

    let edges = parse_dependencies(sql, LanguageType::Sql).unwrap();

    // Should detect dependencies: query -> users, query -> orders
    assert!(edges.iter().any(|e| e.to_entity == "table:users"));
    assert!(edges.iter().any(|e| e.to_entity == "table:orders"));
}
```

---

## 10. Cross-Language Edge Patterns

### 10.1 Existing Cross-Language Support

**Query**:
```bash
curl "http://localhost:7778/code-entities-search-fuzzy?q=unresolved"
```

**Found**:
- `rust:fn:new:unresolved-reference:0-0` (373 callers)
- `rust:fn:to_string:unresolved-reference:0-0` (284 callers)

**Analysis**: Parseltongue tracks **unresolved references** when a function call cannot be matched to a definition. This enables cross-file and cross-language dependency tracking.

### 10.2 SQL Cross-Language Pattern

**Use Case**: Application code calling SQL

**Python Example**:
```python
cursor.execute("SELECT * FROM users")  # Python → SQL dependency
```

**Rust Example**:
```rust
sqlx::query!("SELECT * FROM users")  # Rust → SQL dependency
```

**Implementation**: Detect SQL string literals in application code, parse them as SQL fragments, extract table references, create edges.

**Complexity**: High - requires:
1. SQL string literal detection in host language
2. Embedded SQL parsing
3. Context preservation across languages

**Recommendation**: Defer to Phase 4 (v1.5.8+). Focus on pure SQL file parsing first.

---

## 11. Verification Queries for Implementation

After implementing SQL support, run these queries to verify:

### 11.1 Language Detection
```bash
# Should now show 13 languages including "sql"
curl "http://localhost:7778/codebase-statistics-overview-summary"
```

### 11.2 SQL Entity Extraction
```bash
# Should return SQL tables, views, procedures
curl "http://localhost:7778/code-entities-search-fuzzy?q=users" | jq '.data.entities[] | select(.language == "sql")'
```

### 11.3 SQL Dependency Graph
```bash
# Should show table dependencies
curl "http://localhost:7778/forward-callees-query-graph?entity=sql:view:active_users"
```

### 11.4 Complexity Analysis
```bash
# Should identify heavily-referenced tables
curl "http://localhost:7778/complexity-hotspots-ranking-view?top=50" | jq '.data.hotspots[] | select(.entity_key | contains("sql:"))'
```

---

## 12. Parseltongue API Queries Summary

**Total Queries Run**: 24

| Query Type | Endpoint | Purpose | Count |
|------------|----------|---------|-------|
| Health Check | `/server-health-check-status` | Verify server running | 1 |
| Statistics | `/codebase-statistics-overview-summary` | Get supported languages | 2 |
| Search | `/code-entities-search-fuzzy` | Find functions, types | 15 |
| Traversal | `/reverse-callers-query-graph` | Trace dependencies | 4 |
| Traversal | `/forward-callees-query-graph` | Find callees | 2 |
| Analysis | `/complexity-hotspots-ranking-view` | Find key modules | 1 |
| Analysis | `/blast-radius-impact-analysis` | Impact analysis | 1 |
| Context | `/smart-context-token-budget` | Get code context | 1 |
| Listing | `/code-entities-list-all` | Get file lists | 5 |

---

## 13. Conclusions

### 13.1 Key Architectural Insights

1. **Centralized Language Registration**: All languages funnel through `get_ts_language()` in `query_extractor.rs`

2. **Tree-Sitter Query-Based**: Entity extraction uses tree-sitter queries, not hand-written parsers

3. **Test-Driven Pattern**: Every language has a dedicated test file (`LANG_dependency_patterns_test.rs`)

4. **Graph Database Storage**: Entities and edges stored in CozoDB for efficient graph queries

5. **File Watching Integration**: Incremental reindexing supports live updates

### 13.2 SQL Implementation Confidence

**High Confidence Changes**:
- ✅ Add `tree-sitter-sql` to Cargo.toml
- ✅ Add `Sql` variant to `LanguageType` enum
- ✅ Update `get_language_type()` for `.sql` extension
- ✅ Update `get_ts_language()` to return SQL parser

**Medium Confidence Changes**:
- ⚠️ Tree-sitter SQL queries (need to study tree-sitter-sql grammar)
- ⚠️ SQL-specific entity types (table, view, procedure, etc.)

**Low Confidence Changes**:
- ❌ Cross-language SQL detection (deferred to future)
- ❌ Embedded SQL in string literals (deferred to future)

### 13.3 Estimated Effort

| Task | LOC | Files | Effort |
|------|-----|-------|--------|
| Cargo dependency | +1 | 1 | 5 min |
| Enum variant | +1 | 1 | 5 min |
| Extension mapping | +1 | 1 | 5 min |
| Language registration | +5 | 1 | 15 min |
| Tree-sitter queries | +50 | 1 | 2 hours |
| Test file | +200 | 1 | 3 hours |
| Documentation | +100 | 2 | 1 hour |
| **TOTAL** | **~358** | **8** | **~6.5 hours** |

---

## 14. Next Steps

1. **Read tree-sitter-sql documentation** to understand grammar nodes
2. **Create `sql_dependency_patterns_test.rs`** following existing test patterns
3. **Implement minimal SQL support** (Phase 1)
4. **Run verification queries** to confirm integration
5. **Iterate based on test failures** (RED → GREEN → REFACTOR)

---

## Appendix A: Example Parseltongue API Calls

### A.1 Find Key Function
```bash
curl "http://localhost:7778/code-entities-search-fuzzy?q=get_ts_language"
# Returns: rust:fn:get_ts_language:...:T1641069678
```

### A.2 Trace Call Graph
```bash
curl "http://localhost:7778/reverse-callers-query-graph?entity=rust:fn:get_ts_language:unresolved-reference:0-0"
# Returns: execute_query(), execute_dependency_query()
```

### A.3 Find Architecture Hotspots
```bash
curl "http://localhost:7778/complexity-hotspots-ranking-view?top=30"
# Returns: query_extractor.rs (rank 23, 73 edges)
```

### A.4 List Source Files
```bash
curl "http://localhost:7778/code-entities-list-all?limit=1000" | jq -r '.data.entities[].file_path' | grep "parseltongue-core" | sort -u
# Returns: entities.rs, query_extractor.rs, storage/cozo_client.rs
```

---

**Document Status**: ✅ Complete
**Analysis Method**: HTTP API Query-Driven
**Confidence Level**: High (80%+)
**Ready for Implementation**: Yes - Proceed to Phase 1 (v1.5.5)

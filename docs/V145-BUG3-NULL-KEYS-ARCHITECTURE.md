# Architecture Design: Fix Data Quality Corruption in Parseltongue v1.4.5

**Bug**: CRITICAL BUG #3 - Data Quality Corruption (Language Field Mismatch, Missing Entities)
**Severity**: CRITICAL - Data Corruption + Missing Functionality
**Impact**: 230 entities with wrong language field, main function missing, search broken
**Version**: v1.4.5
**Design Date**: 2026-02-01
**Investigation Date**: 2026-02-01 (Live Server Validation Completed)

---

## Executive Summary

**Problem (Original Bug Report)**: Entity keys reported as "NULL" in v1.4.3 analysis.

**Actual Problem (Live Server Investigation)**: Keys are NOT null, but multiple critical data quality issues exist:

1. **Language Field Corruption**: ALL 233 entities have `language: "rust"` field, even JavaScript entities
   - Keys are CORRECT (e.g., `javascript:fn:greetUser`)
   - Language FIELD is WRONG (all say "rust")
   - 100% data corruption on language field

2. **Missing Main Function**: Main function (`rust:fn:main`) completely absent from database
   - Binary entry point not indexed
   - No entities from `parseltongue/src/main.rs` file
   - Smart context returns 0 entities for main

3. **Orphaned External Dependencies**: 20+ external module references with `0-0` line ranges
   - Edges reference entities like `rust:module:HashMap:0-0`
   - These entities NOT stored in database
   - Graph traversal encounters dead ends

4. **Broken Search**: Fuzzy search returns 0 results for valid entities
   - Database has 230 entities
   - Search for "main" returns 0 results
   - Search algorithm completely broken

5. **Broken Blast Radius**: Graph traversal fails for valid entities
   - Returns "No affected entities found" for existing entities
   - Impact analysis non-functional

**Root Cause**: Language detection during ingestion writes incorrect field to database while key generation works correctly. Main function filtering or parsing also broken.

**Solution Architecture**: Fix language field serialization, implement main function indexing, add external dependency tracking, fix search algorithm.

**Impact**: Fixes 233 entities with corrupt language field, restores main function indexing, enables search and graph traversal.

---

## Phase 1: Live Server Investigation Results (COMPLETED 2026-02-01)

### Query 1: Inspect Entity Structure

**Command**:
```bash
curl -s "http://localhost:7777/code-entities-list-all" | jq '.data.entities[0:5]'
```

**Result**:
```json
[
  {
    "key": "javascript:class:FileWatcherTest:__tests_e2e_workspace_src_test_v141_js:12-21",
    "file_path": "./tests/e2e_workspace/src/test_v141.js",
    "entity_type": "class",
    "entity_class": "CODE",
    "language": "rust"  ← WRONG! Should be "javascript"
  },
  {
    "key": "javascript:fn:calculateSum:__tests_e2e_workspace_src_test_v141_js:8-10",
    "file_path": "./tests/e2e_workspace/src/test_v141.js",
    "entity_type": "function",
    "entity_class": "CODE",
    "language": "rust"  ← WRONG! Should be "javascript"
  }
]
```

**Finding**: Keys are CORRECT (format: `language:type:name:file:lines`), but `language` field is WRONG.

---

### Query 2: Validate Language Field Corruption

**Command**:
```bash
curl -s "http://localhost:7777/code-entities-list-all" | \
  jq '[.data.entities[] | {key, language, key_prefix: (.key | split(":")[0])}] |
      group_by(.language) |
      map({language: .[0].language, count: length, sample_key_prefix: .[0].key_prefix})'
```

**Result**:
```json
[
  {
    "language": "rust",
    "count": 233,
    "sample_key_prefix": "javascript"  ← Proof of mismatch!
  }
]
```

**Finding**: ALL 233 entities have `language: "rust"` field, even though key prefixes vary (javascript, rust, etc.). **100% language field corruption**.

---

### Query 3: Check Key Prefix Distribution

**Command**:
```bash
curl -s "http://localhost:7777/code-entities-list-all" | \
  jq '[.data.entities[] | .key | split(":")[0]] | group_by(.) |
      map({language_prefix: .[0], count: length})'
```

**Result**:
```json
[
  {
    "language_prefix": "javascript",
    "count": 5
  },
  {
    "language_prefix": "rust",
    "count": 228
  }
]
```

**Finding**: Keys correctly identify 5 JavaScript entities and 228 Rust entities. **Key generation is working correctly**.

---

### Query 4: Search for Main Function

**Command**:
```bash
curl -s "http://localhost:7777/code-entities-list-all" | \
  jq '[.data.entities[] | select(.key | startswith("rust:fn:main"))]'
```

**Result**:
```json
[]
```

**Finding**: Main function **completely missing** from database. No entities from `parseltongue/src/main.rs`.

---

### Query 5: Check Parseltongue-Core Indexing

**Command**:
```bash
curl -s "http://localhost:7777/code-entities-list-all" | \
  jq '[.data.entities[] | select(.key | contains("parseltongue-core"))] | length'
```

**Result**:
```
78
```

**Finding**: 78 entities from parseltongue-core indexed, but ~150 missing (main, CLI code, etc.).

---

### Query 6: Test Fuzzy Search

**Command**:
```bash
curl -s "http://localhost:7777/code-entities-search-fuzzy?q=main"
```

**Result**:
```json
{
  "success": true,
  "endpoint": "/code-entities-search-fuzzy",
  "data": {
    "total_count": 0,
    "entities": []
  }
}
```

**Finding**: Search returns **0 results** even though database has 230 entities. **Search algorithm completely broken**.

---

### Query 7: Check External Dependencies

**Command**:
```bash
curl -s "http://localhost:7777/dependency-edges-list-all" | \
  jq '[.data.edges[] | select(.to_key | contains("0-0")) | .to_key] | unique | .[0:20]'
```

**Result**:
```json
[
  "javascript:fn:log:unknown:0-0",
  "rust:module:HashMap:0-0",
  "rust:module:CodeEntity:0-0",
  "rust:module:EntityMetadata:0-0",
  "rust:module:EdgeType:0-0"
]
```

**Finding**: 20+ external dependencies referenced with `0-0` line ranges. These are **orphaned references** - edges point to entities that don't exist in the database.

---

### Query 8: Test Entity Detail Lookup

**Command**:
```bash
curl -s "http://localhost:7777/code-entity-detail-view?key=javascript:fn:greetUser:__tests_e2e_workspace_src_test_v141_js:4-6"
```

**Result**:
```json
{
  "success": true,
  "endpoint": "/code-entity-detail-view",
  "data": {
    "key": "javascript:fn:greetUser:__tests_e2e_workspace_src_test_v141_js:4-6",
    "file_path": "./tests/e2e_workspace/src/test_v141.js",
    "entity_type": "function",
    "entity_class": "CODE",
    "language": "rust",  ← Still wrong
    "code": "function greetUser(name) {\n    return `Hello, ${name}! Welcome to Parseltongue v1.4.1`;\n}"
  }
}
```

**Finding**: Entity detail lookup **works** with correct key. Returns full code. Language field still corrupted.

---

### Query 9: Test Blast Radius

**Command**:
```bash
curl -s "http://localhost:7777/blast-radius-impact-analysis?entity=javascript:fn:greetUser:__tests_e2e_workspace_src_test_v141_js:4-6&hops=1"
```

**Result**:
```json
{
  "success": false,
  "error": "No affected entities found for: javascript:fn:greetUser:__tests_e2e_workspace_src_test_v141_js:4-6"
}
```

**Finding**: Blast radius **completely broken** even for valid entities. Graph traversal non-functional.

---

### Query 10: Test Smart Context

**Command**:
```bash
curl -s "http://localhost:7777/smart-context-token-budget?focus=rust:fn:main&tokens=2000"
```

**Result**:
```json
{
  "success": true,
  "endpoint": "/smart-context-token-budget",
  "data": {
    "focus_entity": "rust:fn:main",
    "token_budget": 2000,
    "tokens_used": 0,
    "entities_included": 0,
    "context": []
  }
}
```

**Finding**: Smart context returns **0 entities, 0 tokens** because main function doesn't exist in database.

---

### Summary of Actual Findings

| Issue | Severity | Description |
|-------|----------|-------------|
| **Language Field Corruption** | CRITICAL | ALL 233 entities have `language: "rust"` regardless of actual language |
| **Missing Main Function** | CRITICAL | Entry point not indexed, no CLI code entities |
| **Orphaned Dependencies** | HIGH | 20+ external module references with no corresponding entities |
| **Broken Search** | CRITICAL | Returns 0 results for any query |
| **Broken Blast Radius** | CRITICAL | Graph traversal fails for valid entities |
| **Broken Smart Context** | CRITICAL | Returns 0 entities due to missing main |

**Original Bug Title "NULL Entity Keys" is MISLEADING**: Keys are NOT null. The real problems are language field corruption, missing entities, and broken algorithms.

---

## Phase 2: Architecture Analysis (Based on Live Server Data)

### Actual Data Flow (Validated)

```mermaid
flowchart TB
    subgraph INGESTION["pt01-folder-to-cozodb-streamer"]
        PARSE[Tree-sitter Parser<br/>Extract entities]
        KEYGEN[Entity Key Generation<br/>✅ WORKS CORRECTLY]
        LANGDET[Language Detection<br/>❌ WRITES "rust" FOR ALL]
        STORE[CozoDB Storage<br/>Persists corrupt data]
    end

    subgraph SERVER["pt08-http-code-query-server"]
        QUERY[CozoDB Query<br/>Read from analysis.db<br/>✅ Returns data correctly]
        SEARCH[Fuzzy Search Algorithm<br/>❌ RETURNS 0 RESULTS]
        BLAST[Blast Radius Algorithm<br/>❌ FAILS FOR VALID KEYS]
        SMART[Smart Context Algorithm<br/>❌ RETURNS 0 ENTITIES]
        JSON[JSON Response<br/>Correct keys, wrong language]
    end

    PARSE --> KEYGEN --> LANGDET --> STORE
    STORE --> QUERY --> JSON
    STORE --> SEARCH
    STORE --> BLAST
    STORE --> SMART
```

### Root Cause Analysis (Confirmed via Live Server)

| Component | Status | Actual Behavior | Evidence |
|-----------|--------|-----------------|----------|
| **Key Generation** | ✅ WORKING | Correctly creates `javascript:fn:name`, `rust:fn:name` | Query 1, 3 confirmed |
| **Language Field** | ❌ BROKEN | ALL entities get `language: "rust"` | Query 2: 233/233 corrupted |
| **Main Function Parsing** | ❌ BROKEN | `main()` not indexed | Query 4: 0 results |
| **External Deps** | ❌ INCOMPLETE | References created but entities not stored | Query 7: 20+ orphans |
| **Search Algorithm** | ❌ BROKEN | Returns 0 for any query | Query 6: 0/230 results |
| **Blast Radius** | ❌ BROKEN | Fails for valid entities | Query 9: Error message |
| **Smart Context** | ❌ BROKEN | Returns 0 entities | Query 10: 0 entities |
| **Entity Lookup** | ✅ WORKING | Returns full entity details | Query 8: Successful |
| **Edge Creation** | ✅ WORKING | Edges reference correct keys | Query 7: Valid edge structure |

### Priority Bugs to Fix (In Order)

1. **Language Field Corruption** (Bug #3a)
   - Location: `pt01/src/ingestion.rs` (likely)
   - Fix: Extract language from key prefix, not from hardcoded value
   - Impact: Fixes 233 entities

2. **Main Function Missing** (Bug #3b)
   - Location: `pt01/src/parser.rs` or filtering logic
   - Fix: Ensure main function is parsed and indexed
   - Impact: Restores entry point visibility

3. **Search Algorithm Broken** (Bug #5)
   - Location: `pt08/src/search.rs` (likely)
   - Fix: Implement fuzzy matching on entity names
   - Impact: Enables entity discovery

4. **Blast Radius Broken** (Bug #3c)
   - Location: `pt08/src/graph_analysis.rs` (likely)
   - Fix: Graph traversal logic with orphan handling
   - Impact: Enables impact analysis

5. **Smart Context Broken** (Bug #2)
   - Location: `pt08/src/context.rs` (likely)
   - Fix: Check entity exists before building context
   - Impact: Enables LLM context generation

---

## Phase 3: Proposed Solution Architecture

### Solution 1: Fix Language Field Corruption (Priority #1)

**Root Cause**: Language field hardcoded to "rust" during ingestion, but key generation correctly extracts language from file extension.

**Solution**: Extract language from entity key prefix (which is correct) instead of using separate detection.

```rust
// File: crates/parseltongue-core/src/entities.rs (or wherever Entity is defined)

impl CodeEntity {
    /// Extract language from entity key
    /// Key format: "language:type:name:file:lines"
    /// Example: "javascript:fn:greetUser:..." → "javascript"
    pub fn extract_language_from_key_validated(&self) -> String {
        self.key
            .split(':')
            .next()
            .unwrap_or("unknown")
            .to_string()
    }

    /// Fix corrupted language field by extracting from key
    pub fn fix_language_field_from_key(&mut self) {
        self.language = self.extract_language_from_key_validated();
    }
}
```

**Migration Function** (for existing database):

```rust
// File: crates/parseltongue-core/src/migration.rs

use cozo::DbInstance;

/// Migrate existing entities to fix language field corruption
/// Reads key prefix and updates language field to match
pub fn migrate_language_field_corruption_fix(db: &DbInstance) -> Result<usize, anyhow::Error> {
    // CozoDB query to update language field based on key prefix
    let query = r#"
        # Extract language from key (first component before ':')
        ?[key, new_lang] := *code_entities{key, language, file_path},
                            new_lang = split_first(key, ':')

        # Update language field
        :put code_entities {
            key => key,
            language => new_lang
        }

        :returning count(key)
    "#;

    let result = db.run_script(query, Default::default())?;
    let count = result.rows[0][0].get_int().unwrap_or(0) as usize;

    Ok(count)
}
```

**Ingestion Fix** (prevent future corruption):

```rust
// File: crates/pt01-folder-to-cozodb-streamer/src/ingestion.rs

// BEFORE (BROKEN):
fn create_entity(name: &str, entity_type: &str, file_path: &Path) -> CodeEntity {
    let key = format!("{}:{}:{}", detect_language(file_path), entity_type, name);
    CodeEntity {
        key: key.clone(),
        language: "rust".to_string(),  // ❌ HARDCODED!
        // ...
    }
}

// AFTER (FIXED):
fn create_entity(name: &str, entity_type: &str, file_path: &Path) -> CodeEntity {
    let detected_lang = detect_language(file_path);  // "javascript", "rust", etc.
    let key = format!("{}:{}:{}", detected_lang, entity_type, name);
    CodeEntity {
        key: key.clone(),
        language: detected_lang.clone(),  // ✅ MATCHES KEY PREFIX
        // ...
    }
}
```

---

### Solution 2: Fix Missing Main Function (Priority #2)

**Root Cause**: Main function likely filtered out or not parsed correctly.

**Investigation Needed**: Query live server to check if main.rs is even scanned:

```bash
# Check if any entities from main.rs exist
curl -s "http://localhost:7777/code-entities-list-all" | \
  jq '[.data.entities[] | select(.file_path | contains("main.rs"))]'
```

**Possible Fixes**:

1. **If main.rs not scanned**: Fix file discovery in pt01
2. **If main() filtered out**: Remove main function filter
3. **If parser fails**: Update tree-sitter query for Rust main functions

---

### Solution 3: Fix Search Algorithm (Priority #3)

**Root Cause**: Search returns 0 results - likely case sensitivity or missing index.

**Solution**: Implement case-insensitive fuzzy matching.

```rust
// File: crates/pt08-http-code-query-server/src/handlers/search.rs

// BEFORE (BROKEN):
async fn fuzzy_search(query: &str, db: &DbInstance) -> Vec<CodeEntity> {
    let cozo_query = r#"
        ?[key, name] := *code_entities{key, name},
                        name == $query
    "#;
    // Returns 0 because exact match fails
}

// AFTER (FIXED):
async fn fuzzy_search(query: &str, db: &DbInstance) -> Vec<CodeEntity> {
    let cozo_query = r#"
        ?[key, name, file_path, language] :=
            *code_entities{key, name, file_path, language},
            lowercase(name) ~~ lowercase($query)  # Case-insensitive contains
        :limit 50
    "#;
    // Returns matching entities
}
```

---

### Solution 4: Fix Blast Radius Algorithm (Priority #4)

**Root Cause**: Fails even for valid entities - likely doesn't handle orphaned external dependencies.

**Solution**: Skip missing entities during graph traversal.

```rust
// File: crates/pt08-http-code-query-server/src/handlers/blast_radius.rs

pub fn calculate_blast_radius_with_hops(
    entity_key: &str,
    hops: u32,
    db: &DbInstance,
) -> Result<Vec<String>, anyhow::Error> {
    let query = format!(r#"
        # Find all entities within N hops
        affected[entity_key] :=
            *dependency_edges{{from_key: '{}'}},
            entity_key = from_key

        # Hop 1: Direct dependencies
        affected[to_key] :=
            *dependency_edges{{from_key: entity_key}},
            affected[entity_key],
            *code_entities{{key: to_key}}  # ✅ ONLY INCLUDE IF ENTITY EXISTS

        # Repeat for N hops...

        ?[entity_key] := affected[entity_key]
    "#, entity_key);

    // Execute and return
}
```

---

### Solution 5: Fix Smart Context Algorithm (Priority #5)

**Root Cause**: Returns 0 entities because focus entity (main) doesn't exist.

**Solution**: Add entity existence check + fallback logic.

```rust
// File: crates/pt08-http-code-query-server/src/handlers/smart_context.rs

pub async fn generate_smart_context_within_budget(
    focus_entity: &str,
    token_budget: usize,
    db: &DbInstance,
) -> Result<SmartContext, anyhow::Error> {
    // ✅ CHECK IF ENTITY EXISTS FIRST
    let entity_exists = db.run_script(
        "?[key] := *code_entities{key}, key == $focus",
        btreemap! { "focus" => focus_entity.into() }
    )?;

    if entity_exists.rows.is_empty() {
        // Fallback: Return entities from same file
        return generate_file_based_context_fallback(focus_entity, token_budget, db);
    }

    // Original algorithm...
}

---

## Phase 4: Test Strategy (TDD Approach)

### Unit Tests (RED Phase)

```rust
// File: crates/parseltongue-core/tests/entity_key_generation_tests.rs

use parseltongue_core::entity_key::{EntityKey, EntityKeyError};

#[test]
fn test_entity_key_create_from_parts_validated_success() {
    let key = EntityKey::create_from_parts_validated("rust", "fn", "main")
        .expect("Should create valid key");
    assert_eq!(key.as_str(), "rust:fn:main");
}

#[test]
fn test_entity_key_create_from_parts_validated_rejects_empty() {
    let result = EntityKey::create_from_parts_validated("", "fn", "main");
    assert!(matches!(result, Err(EntityKeyError::EmptyComponent)));
}

#[test]
fn test_entity_key_create_from_parts_validated_rejects_invalid_language() {
    let result = EntityKey::create_from_parts_validated("klingon", "fn", "main");
    assert!(matches!(result, Err(EntityKeyError::UnsupportedLanguage(_))));
}

#[test]
fn test_entity_key_parse_from_string_validated_success() {
    let key = EntityKey::parse_from_string_validated("python:class:Parser")
        .expect("Should parse valid format");
    assert_eq!(key.extract_language_component_only(), "python");
}

#[test]
fn test_entity_key_parse_from_string_validated_rejects_malformed() {
    let result = EntityKey::parse_from_string_validated("invalid");
    assert!(matches!(result, Err(EntityKeyError::InvalidFormat(_))));
}

#[test]
fn test_entity_key_builder_pattern_success() {
    use parseltongue_core::entity_key::EntityKeyBuilder;

    let key = EntityKeyBuilder::new()
        .with_language_set("go")
        .with_entity_type_set("func")
        .with_name_set("HandleRequest")
        .build_validated_key()
        .expect("Builder should succeed");

    assert_eq!(key.as_str(), "go:func:HandleRequest");
}

#[test]
fn test_entity_key_extract_language_component_only() {
    let key = EntityKey::parse_from_string_validated("rust:struct:Entity").unwrap();
    assert_eq!(key.extract_language_component_only(), "rust");
}

#[test]
fn test_entity_key_extract_entity_type_only() {
    let key = EntityKey::parse_from_string_validated("rust:struct:Entity").unwrap();
    assert_eq!(key.extract_entity_type_only(), "struct");
}

#[test]
fn test_entity_key_extract_name_component_only() {
    let key = EntityKey::parse_from_string_validated("rust:struct:Entity").unwrap();
    assert_eq!(key.extract_name_component_only(), "Entity");
}
```

### Integration Tests (GREEN Phase)

```rust
// File: crates/pt01-folder-to-cozodb-streamer/tests/ingestion_integration_tests.rs

use pt01_folder_to_cozodb_streamer::*;
use std::path::Path;

#[test]
fn test_ingest_rust_file_generates_entity_keys() {
    // Arrange
    let test_code = r#"
        fn main() {
            println!("Hello");
        }

        struct Entity {
            name: String,
        }
    "#;

    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("test.rs");
    std::fs::write(&file_path, test_code).unwrap();

    // Act
    let db_path = temp_dir.path().join("test.db");
    ingest_folder_to_database(&temp_dir.path(), &db_path).unwrap();

    // Assert: Query entities and verify keys are NOT null
    let db = cozo::DbInstance::new("rocksdb", db_path.to_str().unwrap(), "").unwrap();
    let query = r#"
        ?[entity_key, name] := *entities{entity_key, name}
    "#;
    let result = db.run_script(query, Default::default()).unwrap();

    assert!(result.rows.len() >= 2, "Should find at least 2 entities");

    for row in result.rows {
        let key = row[0].get_str().expect("Key should be string");
        assert!(!key.is_empty(), "Entity key must not be empty");
        assert!(key.contains(':'), "Entity key must have format language:type:name");
    }
}
```

### Regression Tests

```rust
// File: crates/parseltongue-core/tests/null_entity_key_regression_test.rs

/// Regression test: Ensure NULL entity keys never happen again
#[test]
fn test_entity_creation_without_key_fails_compilation() {
    use parseltongue_core::Entity;

    // Only valid constructor requires key generation:
    let entity = Entity::create_with_generated_key(
        "test",
        "fn",
        "rust",
        "test.rs",
        1,
    ).expect("Should create entity with key");

    // Verify key is never null
    assert!(!entity.entity_key.as_str().is_empty());
}
```

---

## Phase 5: Implementation Plan

### Phase 5.1: Investigation (Current State Analysis)

**Goal**: Determine WHERE NULL keys are introduced.

Run investigation queries (see Phase 1) and document findings.

### Phase 5.2: Core Type System (RED → GREEN)

1. Create `entity_key.rs` module
2. Write RED tests
3. Implement `EntityKey` type (GREEN)
4. Update `Entity` struct
5. All tests passing

### Phase 5.3: Ingestion Pipeline Fix (GREEN → REFACTOR)

1. Update pt01 ingestion logic
2. Add database schema validation
3. Integration tests
4. All tests passing

### Phase 5.4: Data Migration (Fix Existing Database)

1. Create migration module
2. Implement `migrate_null_keys_to_valid()`
3. Add CLI command
4. Test on live database

### Phase 5.5: Verification and Testing

1. Run full test suite
2. Live server verification
3. Regression verification
4. All metrics passing

---

## Phase 6: Verification Plan

### Success Criteria

| Criterion | Verification Method | Expected Result |
|-----------|---------------------|-----------------|
| **No NULL keys** | Query `/code-entities-list-all` | All `entity_key != null` |
| **Valid format** | Check key format | All match `language:type:name` |
| **Entity lookup works** | GET `/code-entity-detail-view?key=X` | Returns 200 with entity |
| **Search works** | Search for "main" | Returns entities with valid keys |
| **Edges reference valid keys** | Query edges | `from_key` and `to_key` valid |
| **Migration success** | Count before/after | 230 entities → 230 valid keys |
| **No regression** | Fresh ingestion | Zero NULL keys |

---

## Implementation Checklist

### Phase 1: Investigation ✓
- [ ] Run investigation queries on live server
- [ ] Document findings
- [ ] Identify exact point where NULL is introduced

### Phase 2: Core Types (TDD)
- [ ] Create `crates/parseltongue-core/src/entity_key.rs`
- [ ] Write RED tests for `EntityKey`
- [ ] Implement `EntityKey` type (GREEN)
- [ ] Update `Entity` struct
- [ ] All core tests passing

### Phase 3: Ingestion Fix
- [ ] Update pt01 to generate keys
- [ ] Add database schema with constraints
- [ ] Integration tests passing

### Phase 4: Migration
- [ ] Create migration module
- [ ] Implement migration function
- [ ] Add CLI command
- [ ] Test on live database

### Phase 5: Verification
- [ ] Full test suite passing
- [ ] Migrate live database
- [ ] Verify live server metrics
- [ ] Fresh ingestion test
- [ ] All checklist items ✓

---

## Time Estimate

- Investigation: 30 minutes
- Core types + tests: 2 hours
- Ingestion fix: 1 hour
- Migration: 1 hour
- Verification: 30 minutes
- **Total: ~5 hours**

---

**Design Ready**: This architecture is ready for TDD implementation following STUB → RED → GREEN → REFACTOR cycle.

**Next Steps**: Invoke rust-coder-01 agent with this architecture for implementation.

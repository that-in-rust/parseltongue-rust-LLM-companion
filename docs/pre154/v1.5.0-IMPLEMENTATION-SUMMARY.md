# v1.5.0 Implementation Summary

**Quick Reference for rust-coder-01 Agent**

---

## The Task

Implement batch entity insertion to achieve 10x+ ingestion speedup.

**Current bottleneck** (line 624 in `streamer.rs`):
```rust
// ❌ N database calls
self.db.insert_entity(&code_entity).await?;
```

**Solution** (copy `insert_edges_batch` pattern):
```rust
// ✅ 1 database call
self.db.insert_entities_batch(&code_entities).await?;
```

---

## Files to Modify

### 1. Add Method: `crates/parseltongue-core/src/storage/cozo_client.rs`

**Location**: After `insert_entity()` method (around line 805)

**Pattern to copy**: `insert_edges_batch()` (lines 207-251)

```rust
/// Insert multiple entities in a single batch operation
///
/// # Performance Contract
/// - 10,000 entities: < 500ms
/// - 50,000 entities: < 2s
///
pub async fn insert_entities_batch(&self, entities: &[CodeEntity]) -> Result<()> {
    if entities.is_empty() {
        return Ok(());
    }

    let query = format!(
        r#"
        ?[ISGL1_key, Current_Code, Future_Code, interface_signature, TDD_Classification,
          lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
          last_modified, entity_type, entity_class, birth_timestamp, content_hash, semantic_path] <- [{}]

        :put CodeGraph {{
            ISGL1_key =>
            Current_Code, Future_Code, interface_signature, TDD_Classification,
            lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
            last_modified, entity_type, entity_class, birth_timestamp, content_hash, semantic_path
        }}
        "#,
        entities
            .iter()
            .map(|entity| {
                let params = self.entity_to_params(entity).unwrap();
                // Convert params to inline array format
                format!("[{}, {}, {}, ...]",
                    quote_str(params.get("ISGL1_key")),
                    quote_str(params.get("Current_Code")),
                    // ... all 17 fields
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    );

    self.db.run_script(&query, Default::default(), ScriptMutability::Mutable)?;
    Ok(())
}

// Helper to quote strings for CozoDB
fn quote_str(val: &DataValue) -> String {
    match val {
        DataValue::Str(s) => format!("'{}'", s.replace('\'', "\\'")),
        DataValue::Null => "null".to_string(),
        // ... handle other types
    }
}
```

---

### 2. Modify: `crates/pt01-folder-to-cozodb-streamer/src/streamer.rs`

**Location**: Lines 600-640 in `stream_file()` method

**Change**:
```rust
// BEFORE: Per-entity insertion
for parsed_entity in parsed_entities {
    let isgl1_key = self.key_generator.generate_key(&parsed_entity)?;
    let lsp_metadata = self.fetch_lsp_metadata_for_entity(&parsed_entity, file_path).await;

    match self.parsed_entity_to_code_entity(&parsed_entity, &isgl1_key, &content, file_path) {
        Ok(mut code_entity) => {
            if let Some(metadata) = lsp_metadata {
                code_entity.lsp_metadata = Some(metadata);
            }

            // Skip test entities
            if matches!(code_entity.entity_class, EntityClass::TestImplementation) {
                test_count += 1;
                continue;
            }

            // ❌ BOTTLENECK HERE
            match self.db.insert_entity(&code_entity).await {
                Ok(_) => { entities_created += 1; code_count += 1; }
                Err(e) => errors.push(format!("Failed to insert: {}", e)),
            }
        }
        Err(e) => errors.push(format!("Failed to convert: {}", e)),
    }
}

// AFTER: Batch insertion
let mut code_entities = Vec::new();
for parsed_entity in parsed_entities {
    let isgl1_key = self.key_generator.generate_key(&parsed_entity)?;
    let lsp_metadata = self.fetch_lsp_metadata_for_entity(&parsed_entity, file_path).await;

    match self.parsed_entity_to_code_entity(&parsed_entity, &isgl1_key, &content, file_path) {
        Ok(mut code_entity) => {
            if let Some(metadata) = lsp_metadata {
                code_entity.lsp_metadata = Some(metadata);
            }

            // Skip test entities
            if matches!(code_entity.entity_class, EntityClass::TestImplementation) {
                test_count += 1;
                continue;
            }

            // Collect for batch insert
            code_entities.push(code_entity);
        }
        Err(e) => errors.push(format!("Failed to convert: {}", e)),
    }
}

// ✅ BATCH INSERT HERE
match self.db.insert_entities_batch(&code_entities).await {
    Ok(_) => {
        entities_created = code_entities.len();
        code_count = code_entities.len();
    }
    Err(e) => errors.push(format!("Failed batch insert: {}", e)),
}
```

---

### 3. Add Tests: `crates/parseltongue-core/tests/storage_batch_insert_performance_tests.rs`

**Create new file** with 14 test specifications from TDD-SPEC document:
- REQ-v1.5.0-001 through REQ-v1.5.0-014
- Helper function `create_test_code_entity_simple()`

---

## TDD Workflow

### STUB Phase
```bash
# Create test file with all 14 tests
cargo test insert_entities_batch
# Expected: Compile error - method not found
```

### RED Phase
```bash
# Add empty method stub
cargo test insert_entities_batch
# Expected: All tests fail
```

### GREEN Phase
```bash
# Implement method
cargo test insert_entities_batch
# Expected: All tests pass

# Run performance contract test
cargo test test_insert_entities_batch_very_large --ignored -- --nocapture
# Expected: < 500ms for 10K entities
```

### REFACTOR Phase
```bash
cargo clippy
cargo fmt
cargo test --all
```

---

## Performance Contract (Must Pass)

| Test | Contract | Location |
|------|----------|----------|
| Empty batch | < 1ms | REQ-v1.5.0-001 |
| Single entity | < 10ms | REQ-v1.5.0-002 |
| 10 entities | < 50ms | REQ-v1.5.0-003 |
| 100 entities | < 100ms | REQ-v1.5.0-004 |
| 1,000 entities | < 200ms | REQ-v1.5.0-005 |
| **10,000 entities** | **< 500ms** | **REQ-v1.5.0-006** ⭐ |
| Speedup | >= 10x | REQ-v1.5.0-007 |

---

## Definition of Done

- [ ] `insert_entities_batch()` implemented
- [ ] `stream_file()` uses batch insertion
- [ ] All 14 tests pass
- [ ] Performance: 10K entities in < 500ms
- [ ] Speedup: >= 10x vs sequential
- [ ] Zero clippy warnings
- [ ] Four-word naming verified
- [ ] No TODO/STUB comments

---

## Quick Commands

```bash
# Run unit tests
cargo test insert_entities_batch --lib

# Run performance test
cargo test test_insert_entities_batch_very_large --ignored -- --nocapture

# Run benchmark
cargo test benchmark_batch_insert_throughput_measurement --ignored -- --nocapture

# Full validation
./docs/validate-v1.5.0-performance.sh
```

---

## Key Insight

The pattern already exists in the codebase:
- `insert_edges_batch()` (lines 207-251) does exactly what we need
- Copy the inline array format: `?[col1, col2] <- [[val1, val2], [val3, val4]]`
- Single `run_script()` call instead of N calls

Expected result: **10-60x speedup** with minimal code changes.

---

*Ready for implementation by rust-coder-01 via TDD-First cycle.*

# TDD Specification: v1.5.0 Ingestion Performance Optimization

**Version**: 1.5.0
**Date**: 2026-02-06
**Target**: Batch entity insertion for 10x speedup
**Specification Type**: Executable acceptance criteria (TDD-First)

---

## Executive Summary

This specification defines the TDD test suite for parseltongue v1.5.0, which implements **batch entity insertion** to achieve **10-50x ingestion speedup**. The current architecture inserts entities one-at-a-time (N database round-trips for N entities). The solution already exists in the codebase: `insert_edges_batch()` demonstrates the pattern—we simply need to extend it to entities.

**Current Bottleneck** (lines 600-640 in `streamer.rs`):
```rust
// ❌ CURRENT: N database calls
for parsed_entity in parsed_entities {
    let isgl1_key = self.key_generator.generate_key(&parsed_entity)?;
    let code_entity = self.parsed_entity_to_code_entity(...)?;
    self.db.insert_entity(&code_entity).await?;  // DB round-trip per entity!
}
```

**Proposed Solution**:
```rust
// ✅ PROPOSED: 1 database call
let mut code_entities = Vec::new();
for parsed_entity in parsed_entities {
    let isgl1_key = self.key_generator.generate_key(&parsed_entity)?;
    let code_entity = self.parsed_entity_to_code_entity(...)?;
    code_entities.push(code_entity);
}
self.db.insert_entities_batch(&code_entities).await?;  // Single DB call
```

**Performance Contract**:
- 10,000 entities: < 500ms (vs current ~30s)
- 50,000 entities: < 2s (vs current ~150s)
- Linear scaling with entity count (not quadratic)

---

## Performance Baseline Measurements

### Current Performance (Measured via Thesis Analysis)

| Metric | Value | Source |
|--------|-------|--------|
| **Per-entity DB round-trip** | 2-5ms | CozoDB insert latency |
| **10,000 entity file** | ~30-50s | 10K × 3ms = 30s |
| **50,000 entity codebase** | ~150s | 50K × 3ms = 2.5min |
| **Parseltongue itself (933 entities)** | ~3s | Current ingestion time |

### Target Performance (Post-Optimization)

| Metric | Target | Speedup |
|--------|--------|---------|
| **10,000 entities batch** | < 500ms | **60x faster** |
| **50,000 entities batch** | < 2s | **75x faster** |
| **Parseltongue (933 entities)** | < 100ms | **30x faster** |

---

## Version Roadmap

### v1.5.0: Batch Entity Insertion (THIS SPEC)
**Effort**: Low (2-4 hours) | **Impact**: Very High (10-50x speedup)

Core deliverable:
- `insert_entities_batch()` method in `CozoDbStorage`
- Batched insertion in `stream_file()` method
- Performance benchmarks proving 10x+ speedup

**Why this first?** Quick win that requires minimal code changes—the pattern already exists for edges.

### v1.5.1: Parallel File Processing
**Effort**: Medium (1-2 days) | **Impact**: High (4-8x additional speedup)

- Add `rayon` dependency for parallel iterators
- Thread-safe entity buffering with `crossbeam` channels
- Parallel `WalkDir` iteration

### v1.5.2: LSP Optimization
**Effort**: Medium (2-3 days) | **Impact**: High (5x additional speedup)

- `--skip-lsp` flag for fast initial scan
- Background LSP enrichment queue
- Batch hover requests per file

### v1.5.3: Two-Phase Architecture
**Effort**: High (1 week) | **Impact**: Transformative (20-50x total)

- Separate SCAN and COMMIT phases
- Memory-bounded streaming processing
- Progress reporting per phase

---

## Module Architecture

### New Method Location

**File**: `crates/parseltongue-core/src/storage/cozo_client.rs`

**Pattern to Copy**: `insert_edges_batch()` (lines 207-251)

```rust
// Existing pattern for edges (lines 207-251):
pub async fn insert_edges_batch(&self, edges: &[DependencyEdge]) -> Result<()> {
    if edges.is_empty() {
        return Ok(());
    }

    let query = format!(
        r#"
        ?[from_key, to_key, edge_type, source_location] <- [{}]

        :put DependencyEdges {{
            from_key, to_key, edge_type =>
            source_location
        }}
        "#,
        edges.iter().map(|edge| { /* ... */ }).collect::<Vec<_>>().join(", ")
    );

    self.db.run_script(&query, Default::default(), ScriptMutability::Mutable)?;
    Ok(())
}
```

**New Method to Implement**:
```rust
/// Insert multiple entities in a single batch operation
///
/// # Performance Contract
/// - 10,000 entities: < 500ms (v1.5.0 requirement)
/// - 50,000 entities: < 2s (v1.5.0 requirement)
/// - Linear scaling: O(n) where n = entity count
///
/// # Example
/// ```ignore
/// let entities = vec![entity1, entity2, ...];
/// storage.insert_entities_batch(&entities).await?;
/// ```
pub async fn insert_entities_batch(&self, entities: &[CodeEntity]) -> Result<()> {
    // Implementation follows insert_edges_batch pattern
}
```

### Integration Point

**File**: `crates/pt01-folder-to-cozodb-streamer/src/streamer.rs`

**Current Code** (lines 600-640):
```rust
// Inside stream_file() method
for parsed_entity in parsed_entities {
    let isgl1_key = self.key_generator.generate_key(&parsed_entity)?;
    // ... LSP metadata fetch ...
    let code_entity = self.parsed_entity_to_code_entity(...)?;

    // ❌ BOTTLENECK: Per-entity DB insertion
    match self.db.insert_entity(&code_entity).await {
        Ok(_) => entities_created += 1,
        Err(e) => errors.push(...),
    }
}
```

**New Code** (v1.5.0):
```rust
// Collect entities first
let mut code_entities = Vec::new();
for parsed_entity in parsed_entities {
    let isgl1_key = self.key_generator.generate_key(&parsed_entity)?;
    // ... LSP metadata fetch ...
    match self.parsed_entity_to_code_entity(...) {
        Ok(code_entity) => code_entities.push(code_entity),
        Err(e) => errors.push(...),
    }
}

// ✅ OPTIMIZED: Batch insert all entities
match self.db.insert_entities_batch(&code_entities).await {
    Ok(_) => entities_created = code_entities.len(),
    Err(e) => errors.push(...),
}
```

---

## TDD Test Specifications

### Test Suite Organization

**File**: `crates/parseltongue-core/tests/storage_batch_insert_performance_tests.rs`

All tests follow four-word naming convention: `verb_constraint_target_qualifier()`

---

## PHASE 1: Core Functionality Tests

### REQ-v1.5.0-001: Empty Batch Handling

**WHEN** I call `insert_entities_batch()` with empty vec
**THEN** the system SHALL return `Ok(())`
**AND** SHALL perform zero database operations
**AND** SHALL complete in < 1ms

```rust
#[tokio::test]
async fn test_insert_entities_batch_empty() {
    // Arrange
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    let entities: Vec<CodeEntity> = vec![];

    // Act
    let start = Instant::now();
    let result = storage.insert_entities_batch(&entities).await;
    let elapsed = start.elapsed();

    // Assert
    assert!(result.is_ok());
    assert!(elapsed < Duration::from_millis(1));
}
```

---

### REQ-v1.5.0-002: Single Entity Batch

**WHEN** I call `insert_entities_batch()` with one entity
**THEN** the system SHALL insert the entity successfully
**AND** SHALL be retrievable via `get_entity()`
**AND** SHALL complete in < 10ms

```rust
#[tokio::test]
async fn test_insert_entities_batch_single() {
    // Arrange
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();

    let entity = create_test_code_entity_simple(
        "rust:fn:test_function:test_rs:1-10",
        "fn test() {}",
        EntityClass::CodeImplementation,
    );
    let entities = vec![entity.clone()];

    // Act
    let start = Instant::now();
    storage.insert_entities_batch(&entities).await.unwrap();
    let elapsed = start.elapsed();

    // Assert
    let retrieved = storage.get_entity(&entity.isgl1_key).await.unwrap();
    assert_eq!(retrieved.isgl1_key, entity.isgl1_key);
    assert_eq!(retrieved.current_code, entity.current_code);
    assert!(elapsed < Duration::from_millis(10));
}
```

---

### REQ-v1.5.0-003: Small Batch Insert (10 entities)

**WHEN** I call `insert_entities_batch()` with 10 entities
**THEN** the system SHALL insert all entities
**AND** SHALL be retrievable via `get_entity()`
**AND** SHALL complete in < 50ms

```rust
#[tokio::test]
async fn test_insert_entities_batch_small() {
    // Arrange
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();

    let entities: Vec<CodeEntity> = (0..10)
        .map(|i| create_test_code_entity_simple(
            &format!("rust:fn:test_function_{}:test_rs:{}-{}", i, i*10, i*10+5),
            &format!("fn test_{}() {{}}", i),
            EntityClass::CodeImplementation,
        ))
        .collect();

    // Act
    let start = Instant::now();
    storage.insert_entities_batch(&entities).await.unwrap();
    let elapsed = start.elapsed();

    // Assert - verify all entities inserted
    for entity in &entities {
        let retrieved = storage.get_entity(&entity.isgl1_key).await;
        assert!(retrieved.is_ok(), "Failed to retrieve entity {}", entity.isgl1_key);
    }

    assert!(elapsed < Duration::from_millis(50));
}
```

---

### REQ-v1.5.0-004: Medium Batch Insert (100 entities)

**WHEN** I call `insert_entities_batch()` with 100 entities
**THEN** the system SHALL insert all entities
**AND** SHALL complete in < 100ms
**AND** SHALL use single database transaction

```rust
#[tokio::test]
async fn test_insert_entities_batch_medium() {
    // Arrange
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();

    let entities: Vec<CodeEntity> = (0..100)
        .map(|i| create_test_code_entity_simple(
            &format!("rust:fn:function_{}:test_rs:{}-{}", i, i*10, i*10+5),
            &format!("fn func_{}() {{ println!(\"test\"); }}", i),
            EntityClass::CodeImplementation,
        ))
        .collect();

    // Act
    let start = Instant::now();
    storage.insert_entities_batch(&entities).await.unwrap();
    let elapsed = start.elapsed();

    // Assert
    assert!(elapsed < Duration::from_millis(100),
        "Expected < 100ms, got {:?}", elapsed);

    // Spot check: verify first, middle, last entities
    storage.get_entity(&entities[0].isgl1_key).await.unwrap();
    storage.get_entity(&entities[50].isgl1_key).await.unwrap();
    storage.get_entity(&entities[99].isgl1_key).await.unwrap();
}
```

---

## PHASE 2: Performance Contract Tests

### REQ-v1.5.0-005: Large Batch Insert (1,000 entities)

**WHEN** I call `insert_entities_batch()` with 1,000 entities
**THEN** the system SHALL complete in < 200ms
**AND** SHALL demonstrate 10x+ speedup vs individual inserts

```rust
#[tokio::test]
async fn test_insert_entities_batch_large() {
    // Arrange
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();

    let entities: Vec<CodeEntity> = (0..1000)
        .map(|i| create_test_code_entity_simple(
            &format!("rust:fn:large_function_{}:large_rs:{}-{}", i, i*10, i*10+5),
            &format!("fn large_{}() {{ /* implementation */ }}", i),
            EntityClass::CodeImplementation,
        ))
        .collect();

    // Act
    let start = Instant::now();
    storage.insert_entities_batch(&entities).await.unwrap();
    let elapsed = start.elapsed();

    // Assert
    assert!(elapsed < Duration::from_millis(200),
        "Expected < 200ms for 1K entities, got {:?}", elapsed);

    println!("✅ 1,000 entities inserted in {:?}", elapsed);
}
```

---

### REQ-v1.5.0-006: Very Large Batch (10,000 entities) - PRIMARY CONTRACT

**WHEN** I call `insert_entities_batch()` with 10,000 entities
**THEN** the system SHALL complete in < 500ms
**AND** SHALL be at least 50x faster than sequential inserts
**AND** SHALL scale linearly with entity count

```rust
#[tokio::test]
#[ignore] // Expensive test - run explicitly with: cargo test --ignored
async fn test_insert_entities_batch_very_large() {
    // Arrange
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();

    let entities: Vec<CodeEntity> = (0..10_000)
        .map(|i| create_test_code_entity_simple(
            &format!("rust:fn:very_large_func_{}:huge_rs:{}-{}", i, i*10, i*10+5),
            &format!("fn func_{}() {{ /* code */ }}", i),
            EntityClass::CodeImplementation,
        ))
        .collect();

    // Act - Batch insert
    let start = Instant::now();
    storage.insert_entities_batch(&entities).await.unwrap();
    let batch_elapsed = start.elapsed();

    // Assert - Performance contract
    assert!(batch_elapsed < Duration::from_millis(500),
        "Expected < 500ms for 10K entities, got {:?}", batch_elapsed);

    println!("✅ v1.5.0 PERFORMANCE CONTRACT MET:");
    println!("   10,000 entities inserted in {:?}", batch_elapsed);
    println!("   Average: {:?} per entity", batch_elapsed / 10_000);
}
```

---

### REQ-v1.5.0-007: Baseline Comparison (Sequential vs Batch)

**WHEN** I compare sequential inserts vs batch inserts for 100 entities
**THEN** batch SHALL be at least 10x faster
**AND** results SHALL demonstrate O(1) batch vs O(n) sequential

```rust
#[tokio::test]
async fn test_batch_vs_sequential_speedup_comparison() {
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();

    let entities: Vec<CodeEntity> = (0..100)
        .map(|i| create_test_code_entity_simple(
            &format!("rust:fn:compare_func_{}:cmp_rs:{}-{}", i, i*10, i*10+5),
            &format!("fn compare_{}() {{}}", i),
            EntityClass::CodeImplementation,
        ))
        .collect();

    // Measure sequential inserts
    let start_seq = Instant::now();
    for entity in &entities {
        storage.insert_entity(entity).await.unwrap();
    }
    let sequential_time = start_seq.elapsed();

    // Clear database for batch test
    let storage_batch = CozoDbStorage::new("mem").await.unwrap();
    storage_batch.create_schema().await.unwrap();

    // Measure batch insert
    let start_batch = Instant::now();
    storage_batch.insert_entities_batch(&entities).await.unwrap();
    let batch_time = start_batch.elapsed();

    // Calculate speedup
    let speedup = sequential_time.as_micros() as f64 / batch_time.as_micros() as f64;

    // Assert
    assert!(speedup >= 10.0,
        "Expected at least 10x speedup, got {:.2}x (seq: {:?}, batch: {:?})",
        speedup, sequential_time, batch_time);

    println!("✅ SPEEDUP VERIFICATION:");
    println!("   Sequential: {:?} for 100 entities", sequential_time);
    println!("   Batch:      {:?} for 100 entities", batch_time);
    println!("   Speedup:    {:.2}x", speedup);
}
```

---

## PHASE 3: Edge Case Tests

### REQ-v1.5.0-008: Duplicate Key Handling

**WHEN** I insert batch with duplicate ISGL1 keys
**THEN** the system SHALL overwrite with latest value (CozoDB :put semantics)
**AND** SHALL NOT fail the entire batch

```rust
#[tokio::test]
async fn test_insert_entities_batch_duplicate_keys() {
    // Arrange
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();

    let key = "rust:fn:duplicate:test_rs:1-10";
    let entities = vec![
        create_test_code_entity_simple(key, "fn duplicate() { v1 }", EntityClass::CodeImplementation),
        create_test_code_entity_simple(key, "fn duplicate() { v2 }", EntityClass::CodeImplementation),
    ];

    // Act
    let result = storage.insert_entities_batch(&entities).await;

    // Assert
    assert!(result.is_ok());
    let retrieved = storage.get_entity(key).await.unwrap();
    assert!(retrieved.current_code.as_ref().unwrap().contains("v2")); // Latest wins
}
```

---

### REQ-v1.5.0-009: Special Characters Escaping

**WHEN** I insert entities with SQL-sensitive characters (quotes, backslashes)
**THEN** the system SHALL escape correctly
**AND** SHALL retrieve exact original content

```rust
#[tokio::test]
async fn test_insert_entities_batch_special_characters() {
    // Arrange
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();

    let entities = vec![
        create_test_code_entity_simple(
            "rust:fn:with_quotes:test_rs:1-5",
            r#"fn test() { println!("it's \"quoted\""); }"#,
            EntityClass::CodeImplementation,
        ),
        create_test_code_entity_simple(
            "rust:fn:with_backslash:test_rs:6-10",
            r"fn test() { let path = r\"C:\Users\test\"; }",
            EntityClass::CodeImplementation,
        ),
    ];

    // Act
    storage.insert_entities_batch(&entities).await.unwrap();

    // Assert - verify exact content matches
    for entity in &entities {
        let retrieved = storage.get_entity(&entity.isgl1_key).await.unwrap();
        assert_eq!(retrieved.current_code, entity.current_code);
    }
}
```

---

### REQ-v1.5.0-010: Large Entity Content (100KB)

**WHEN** I insert entities with large code content (100KB each)
**THEN** the system SHALL handle successfully
**AND** SHALL not degrade performance below contract

```rust
#[tokio::test]
async fn test_insert_entities_batch_large_content() {
    // Arrange
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();

    // Create 10 entities with 100KB content each
    let large_code = "fn large() {\n".to_string() + &"    println!(\"line\");\n".repeat(5000) + "}";
    let entities: Vec<CodeEntity> = (0..10)
        .map(|i| create_test_code_entity_simple(
            &format!("rust:fn:large_content_{}:test_rs:{}-{}", i, i*1000, i*1000+5000),
            &large_code,
            EntityClass::CodeImplementation,
        ))
        .collect();

    // Act
    let start = Instant::now();
    storage.insert_entities_batch(&entities).await.unwrap();
    let elapsed = start.elapsed();

    // Assert
    assert!(elapsed < Duration::from_millis(200));
    println!("✅ 10 entities × 100KB each inserted in {:?}", elapsed);
}
```

---

## PHASE 4: Integration Tests

### REQ-v1.5.0-011: End-to-End File Streaming Integration

**WHEN** I stream a Rust file with 50 entities using new batched method
**THEN** the system SHALL batch insert all entities
**AND** SHALL be faster than v1.4.7 baseline
**AND** SHALL maintain backward compatibility

```rust
#[tokio::test]
async fn test_stream_file_with_batch_insertion() {
    // Arrange
    let temp_dir = tempfile::tempdir().unwrap();
    let test_file = temp_dir.path().join("test.rs");

    // Generate file with 50 functions
    let mut content = String::new();
    for i in 0..50 {
        content.push_str(&format!("fn test_function_{}() {{ println!(\"test\"); }}\n", i));
    }
    std::fs::write(&test_file, content).unwrap();

    let config = StreamerConfig {
        source_path: temp_dir.path().to_path_buf(),
        db_path: "mem".to_string(),
        ..Default::default()
    };

    let key_generator = Arc::new(Isgl1KeyGeneratorImpl::new());
    let test_detector = Arc::new(RustTestDetectorImpl);
    let streamer = FileStreamerImpl::new(config, key_generator, test_detector).await.unwrap();

    // Act
    let start = Instant::now();
    let result = streamer.stream_file(&test_file).await.unwrap();
    let elapsed = start.elapsed();

    // Assert
    assert!(result.success);
    assert_eq!(result.entities_created, 50);
    assert!(elapsed < Duration::from_millis(500)); // Should be fast with batching

    println!("✅ Streamed 50 entities in {:?} (batch mode)", elapsed);
}
```

---

### REQ-v1.5.0-012: Database Consistency After Batch Insert

**WHEN** I insert entities in batch and immediately query
**THEN** ALL entities SHALL be queryable
**AND** dependency edges SHALL reference valid entity keys

```rust
#[tokio::test]
async fn test_batch_insert_database_consistency_check() {
    // Arrange
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    let entities: Vec<CodeEntity> = (0..100)
        .map(|i| create_test_code_entity_simple(
            &format!("rust:fn:func_{}:test_rs:{}-{}", i, i*10, i*10+5),
            &format!("fn func_{}() {{}}", i),
            EntityClass::CodeImplementation,
        ))
        .collect();

    // Act - Insert entities batch
    storage.insert_entities_batch(&entities).await.unwrap();

    // Act - Insert edges referencing these entities
    let edges: Vec<DependencyEdge> = (0..99)
        .map(|i| DependencyEdge::builder()
            .from_key(format!("rust:fn:func_{}:test_rs:{}-{}", i, i*10, i*10+5))
            .to_key(format!("rust:fn:func_{}:test_rs:{}-{}", i+1, (i+1)*10, (i+1)*10+5))
            .edge_type(EdgeType::Calls)
            .build().unwrap())
        .collect();
    storage.insert_edges_batch(&edges).await.unwrap();

    // Assert - Verify all entities and edges exist
    for entity in &entities {
        let retrieved = storage.get_entity(&entity.isgl1_key).await;
        assert!(retrieved.is_ok(), "Entity {} not found", entity.isgl1_key);
    }

    let all_edges = storage.get_all_dependencies().await.unwrap();
    assert_eq!(all_edges.len(), 99);

    println!("✅ Database consistency verified: 100 entities + 99 edges");
}
```

---

## PHASE 5: Benchmark Suite

### REQ-v1.5.0-013: Throughput Benchmark (entities/sec)

**WHEN** I measure throughput for various batch sizes
**THEN** system SHALL demonstrate linear scaling
**AND** SHALL achieve >20,000 entities/sec for large batches

```rust
#[tokio::test]
#[ignore] // Benchmark - run with: cargo test --ignored bench
async fn benchmark_batch_insert_throughput_measurement() {
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();

    let batch_sizes = vec![10, 100, 1000, 5000, 10_000];

    println!("\n📊 BATCH INSERT THROUGHPUT BENCHMARK");
    println!("{:<10} | {:<12} | {:<15} | {:<12}", "Batch Size", "Time", "Entities/sec", "μs/entity");
    println!("{}", "-".repeat(60));

    for size in batch_sizes {
        let entities: Vec<CodeEntity> = (0..size)
            .map(|i| create_test_code_entity_simple(
                &format!("rust:fn:bench_func_{}:bench_rs:{}-{}", i, i*10, i*10+5),
                &format!("fn bench_{}() {{}}", i),
                EntityClass::CodeImplementation,
            ))
            .collect();

        let start = Instant::now();
        storage.insert_entities_batch(&entities).await.unwrap();
        let elapsed = start.elapsed();

        let throughput = (size as f64 / elapsed.as_secs_f64()) as u64;
        let micros_per_entity = elapsed.as_micros() / size as u128;

        println!("{:<10} | {:>10?} | {:>13} | {:>10}μs",
            size, elapsed, throughput, micros_per_entity);

        // Clear for next iteration (recreate storage)
        drop(storage);
        let storage = CozoDbStorage::new("mem").await.unwrap();
        storage.create_schema().await.unwrap();
    }

    println!("\n✅ Throughput benchmark complete");
}
```

---

### REQ-v1.5.0-014: Memory Usage Benchmark

**WHEN** I insert 10,000 entities in batch
**THEN** peak memory SHALL be < 50MB
**AND** memory SHALL be released after operation

```rust
#[tokio::test]
#[ignore] // Memory test - requires manual verification
async fn benchmark_batch_insert_memory_usage() {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Simple memory tracker (production code should use jemalloc/mimalloc profiling)
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();

    let entities: Vec<CodeEntity> = (0..10_000)
        .map(|i| create_test_code_entity_simple(
            &format!("rust:fn:mem_func_{}:mem_rs:{}-{}", i, i*10, i*10+5),
            &format!("fn mem_{}() {{ /* avg 500 bytes */ }}", i),
            EntityClass::CodeImplementation,
        ))
        .collect();

    // Estimate entity vec size
    let estimated_size = entities.len() * std::mem::size_of::<CodeEntity>();
    println!("📊 Estimated entity vec size: ~{} MB", estimated_size / 1_024 / 1_024);

    // Act
    storage.insert_entities_batch(&entities).await.unwrap();

    // Assert
    assert!(estimated_size < 50 * 1_024 * 1_024, "Memory exceeds 50MB");

    drop(entities); // Memory should be freed
    println!("✅ Memory usage acceptable");
}
```

---

## PHASE 6: Helper Functions (Test Utilities)

```rust
use parseltongue_core::entities::*;
use parseltongue_core::storage::CozoDbStorage;
use std::time::{Duration, Instant};

/// Create simple test entity for benchmarks
fn create_test_code_entity_simple(
    key: &str,
    code: &str,
    entity_class: EntityClass,
) -> CodeEntity {
    CodeEntity {
        isgl1_key: EntityKey::new(key.to_string()),
        current_code: Some(code.to_string()),
        future_code: None,
        interface_signature: InterfaceSignature {
            signature_text: format!("signature for {}", key),
            signature_hash: "hash123".to_string(),
        },
        tdd_classification: TddClassification::Stub,
        lsp_metadata: None,
        current_ind: true,
        future_ind: false,
        future_action: None,
        file_path: "test.rs".to_string(),
        language: Language::Rust,
        last_modified: chrono::Utc::now(),
        entity_type: EntityType::Function,
        entity_class,
        birth_timestamp: Some(chrono::Utc::now().timestamp()),
        content_hash: Some("content_hash_123".to_string()),
        semantic_path: Some("test::module::function".to_string()),
    }
}
```

---

## Implementation Checklist (TDD Workflow)

### STUB Phase
- [ ] Write all test stubs in `storage_batch_insert_performance_tests.rs`
- [ ] Add helper function `create_test_code_entity_simple()`
- [ ] Verify tests compile but fail with "method not found"

### RED Phase
- [ ] Run `cargo test insert_entities_batch` → verify all tests fail
- [ ] Confirm failures are due to missing method (not logic errors)

### GREEN Phase
- [ ] Implement `insert_entities_batch()` in `cozo_client.rs` (copy edge pattern)
- [ ] Modify `stream_file()` to buffer entities and batch insert
- [ ] Run tests → verify REQ-001 through REQ-010 pass
- [ ] Run benchmarks → verify performance contracts met

### REFACTOR Phase
- [ ] Extract query building logic to helper method
- [ ] Add inline comments explaining CozoDB syntax
- [ ] Run `cargo clippy` → fix warnings
- [ ] Run `cargo fmt` → format code

---

## Performance Validation Script

```bash
#!/bin/bash
# validate-v1.5.0-performance.sh

echo "🧪 Running v1.5.0 Performance Validation"
echo "========================================"

# Phase 1: Unit tests
echo "Phase 1: Core functionality tests..."
cargo test insert_entities_batch --lib

# Phase 2: Performance contracts
echo "Phase 2: Performance contract verification..."
cargo test test_insert_entities_batch_very_large --ignored -- --nocapture

# Phase 3: Baseline comparison
echo "Phase 3: Speedup verification..."
cargo test test_batch_vs_sequential_speedup_comparison -- --nocapture

# Phase 4: Integration tests
echo "Phase 4: Integration tests..."
cargo test test_stream_file_with_batch_insertion -- --nocapture

# Phase 5: Benchmarks
echo "Phase 5: Throughput benchmarks..."
cargo test benchmark_batch_insert_throughput_measurement --ignored -- --nocapture

echo ""
echo "✅ v1.5.0 VALIDATION COMPLETE"
echo "Expected: All tests pass, 10x+ speedup demonstrated"
```

---

## Acceptance Criteria (Definition of Done)

### Code Complete
- [ ] `insert_entities_batch()` implemented in `cozo_client.rs`
- [ ] `stream_file()` modified to use batch insertion
- [ ] All 14 test specs implemented and passing
- [ ] Benchmarks demonstrate 10x+ speedup

### Performance Met
- [ ] 10,000 entities insert in < 500ms (REQ-v1.5.0-006)
- [ ] Speedup >= 10x vs sequential (REQ-v1.5.0-007)
- [ ] Linear scaling verified (REQ-v1.5.0-013)

### Quality Gates
- [ ] Zero TODO/STUB comments in production code
- [ ] `cargo clippy` passes with zero warnings
- [ ] `cargo test --all` passes
- [ ] `cargo fmt --check` passes
- [ ] Four-word naming verified for new functions

### Documentation
- [ ] Performance numbers added to README.md
- [ ] CHANGELOG.md updated with v1.5.0 entry
- [ ] Inline code comments explain batch query construction

---

## Future Work (Out of Scope for v1.5.0)

### v1.5.1: Parallel File Processing
```rust
// Not in v1.5.0 - future optimization
use rayon::prelude::*;

WalkDir::new(dir)
    .into_iter()
    .par_bridge()  // Parallel iteration
    .for_each(|entry| {
        self.stream_file(&entry.path());
    });
```

### v1.5.2: LSP Optimization
```rust
// Not in v1.5.0 - defer or skip LSP
let lsp_metadata = if config.skip_lsp {
    None
} else {
    self.fetch_lsp_metadata_for_entity(&entity, file_path).await
};
```

### v1.5.3: Two-Phase Architecture
```
SCAN Phase (parallel) → COMMIT Phase (batched)
- Separate concerns
- Memory-bounded streaming
- Progress reporting
```

---

## References

### Source Documents
1. **THESIS-ingestion-speed-optimization-v1.5.0.md** - Problem analysis
2. **crates/parseltongue-core/src/storage/cozo_client.rs** - Implementation target
3. **crates/pt01-folder-to-cozodb-streamer/src/streamer.rs** - Integration point

### Existing Patterns
- `insert_edges_batch()` (lines 207-251) - Copy this pattern!
- `insert_entity()` (lines 778-805) - Current single-insert method
- `stream_file()` (lines 600-640) - Where bottleneck occurs

### Performance Measurements
- Per-entity insert: 2-5ms (CozoDB latency)
- 10,000 entities sequential: ~30-50s
- Target batch insert: < 500ms (60x faster)

---

**TDD Contract**: Every requirement SHALL have a failing test BEFORE implementation.

**Version Increment Rule**: v1.5.0 delivers EXACTLY ONE feature end-to-end: batch entity insertion with 10x+ speedup, fully tested and benchmarked.

---

*Specification follows parseltongue TDD-First principles and four-word naming convention.*
*Ready for rust-coder-01 agent to implement via STUB → RED → GREEN → REFACTOR cycle.*

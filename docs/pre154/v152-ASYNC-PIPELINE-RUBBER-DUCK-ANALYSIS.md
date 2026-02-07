# v152 Async Pipeline Rubber Duck Debugging Analysis

**Version**: 1.5.2
**Date**: 2026-02-07
**Status**: Research Phase - Improving 5/10 and 6/10 to 10/10

---

## Executive Summary

This document provides a comprehensive rubber duck debugging analysis of the two blocking criteria preventing async pipeline adoption:

1. **Streamer Refactoring (5/10)** → Target: **10/10**
2. **CozoDbStorage Conversion (6/10)** → Target: **10/10**

Using Parseltongue HTTP API for code analysis, we identify concrete refactoring paths, technical blockers, and multiple solution options with feasibility assessments.

---

## Table of Contents

1. [Methodology](#methodology)
2. [Streamer Refactoring Analysis (5/10 → 10/10)](#streamer-refactoring-analysis)
3. [CozoDbStorage Conversion Analysis (6/10 → 10/10)](#cozodb-storage-conversion-analysis)
4. [Multi-Writer Database Problem](#multi-writer-database-problem)
5. [Solution Options Matrix](#solution-options-matrix)
6. [Recommended Path Forward](#recommended-path-forward)

---

## Methodology

### Analysis Approach

**Rubber Duck Debugging Questions**:
1. **Why is this a problem?** - Root cause identification
2. **What are we trying to achieve?** - Clear goal definition
3. **What's blocking us?** - Concrete technical barriers
4. **What are the alternatives?** - Multiple solution paths
5. **What's the simplest solution?** - Minimal viable approach

### Data Sources

**Parseltongue HTTP API Queries Used**:
- `/codebase-statistics-overview-summary` - Codebase metrics
- `/code-entities-search-fuzzy?q=stream_directory` - Function discovery
- `/code-entities-search-fuzzy?q=CozoDbStorage` - Storage layer analysis
- `/complexity-hotspots-ranking-view?top=20` - Coupling analysis
- `/code-entities-list-all` - Full entity enumeration

**Key Files Analyzed**:
- `/crates/pt01-folder-to-cozodb-streamer/src/streamer.rs` (770 lines)
- `/crates/parseltongue-core/src/storage/cozo_client.rs` (approx 1000+ lines)

---

## Streamer Refactoring Analysis

### Current Score: 5/10

### Why is this 5/10? (Root Cause Analysis)

**Problem Statement**: `stream_directory()` and `stream_file()` are tightly coupled monolithic functions that mix concerns:

1. **File System I/O** (already async with Tokio)
2. **Parsing** (synchronous tree-sitter calls)
3. **Database writes** (already async)
4. **LSP enrichment** (already async)
5. **Progress reporting** (synchronous)
6. **Statistics tracking** (synchronous with Mutex)

**Coupling Analysis** (from Parseltongue):
- `streamer.rs` has **136 outbound dependencies** (rank #6 in complexity hotspots)
- Monolithic `stream_file()` function spans **150+ lines** (lines 560-708)
- Tight coupling to `CozoDbStorage` via `Arc<CozoDbStorage>`

### What Makes This Hard?

#### Blocker 1: Sequential LSP Enrichment

**Current Code** (lines 594-603):
```rust
// Enrich with LSP metadata for Rust files (sequential hover requests)
let lsp_metadata = self.fetch_lsp_metadata_for_entity(&parsed_entity, file_path).await;

// Convert ParsedEntity to CodeEntity
match self.parsed_entity_to_code_entity(&parsed_entity, &isgl1_key, &content, file_path) {
    Ok(mut code_entity) => {
        // Store LSP metadata as JSON string if available
        if let Some(metadata) = lsp_metadata {
            code_entity.lsp_metadata = Some(metadata);
        }
```

**Why This is Hard**:
- Each entity requires a separate LSP hover request
- Sequential processing means no parallelism
- LSP server may be slow or unavailable

#### Blocker 2: Batch Insert Coordination

**Current Code** (lines 625-640):
```rust
// Batch insert all entities at once (external placeholders + regular entities)
if !entities_to_insert.is_empty() {
    match self.db.insert_entities_batch(&entities_to_insert).await {
        Ok(_) => {
            entities_created += entities_to_insert.len();
        }
        Err(e) => {
            let error_msg = format!(
                "Failed to batch insert {} entities: {}",
                entities_to_insert.len(),
                e
            );
            errors.push(error_msg);
        }
    }
}
```

**Why This is Hard**:
- Batch insert happens AFTER all entities are parsed
- Cannot stream entities as they're parsed
- Memory usage scales with file size

#### Blocker 3: Dependency Edge Insertion

**Current Code** (lines 652-694):
```rust
// Batch insert dependencies after all entities are stored
if !dependencies.is_empty() {
    // Insert dependency edges
    match self.db.insert_edges_batch(&dependencies).await {
        Ok(_) => {
            // Successfully inserted dependencies
        }
        Err(e) => {
            errors.push(format!("Failed to insert {} dependencies: {}", dependencies.len(), e));
        }
    }
}
```

**Why This is Hard**:
- Edges must wait for all entities to be inserted first
- Cannot parallelize edge insertion with entity insertion
- Foreign key constraint enforcement requires sequential ordering

### Incremental Migration Path (5/10 → 10/10)

#### Phase 1: Extract Pure Functions (Feasibility: 10/10)

**Goal**: Separate parsing logic from I/O

**Current Monolith**:
```rust
async fn stream_file(&self, file_path: &Path) -> Result<FileResult> {
    // Read file (async I/O)
    let content = self.read_file_content(file_path).await?;

    // Parse (sync CPU)
    let (parsed_entities, dependencies) = self.key_generator.parse_source(&content, file_path)?;

    // Transform (sync CPU)
    for parsed_entity in parsed_entities {
        let isgl1_key = self.key_generator.generate_key(&parsed_entity)?;
        let code_entity = self.parsed_entity_to_code_entity(&parsed_entity, &isgl1_key, &content, file_path)?;
        entities_to_insert.push(code_entity);
    }

    // Persist (async I/O)
    self.db.insert_entities_batch(&entities_to_insert).await?;
}
```

**Refactored Pipeline**:
```rust
// Pure function: Parse file content → Entities
fn parse_file_to_entities(
    content: &str,
    file_path: &Path,
    key_generator: &dyn Isgl1KeyGenerator,
) -> Result<(Vec<CodeEntity>, Vec<DependencyEdge>)> {
    let (parsed_entities, dependencies) = key_generator.parse_source(content, file_path)?;

    let entities: Vec<CodeEntity> = parsed_entities
        .iter()
        .map(|p| {
            let key = key_generator.generate_key(p)?;
            parsed_entity_to_code_entity(p, &key, content, file_path)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((entities, dependencies))
}

// Async function: Orchestrate I/O
async fn stream_file(&self, file_path: &Path) -> Result<FileResult> {
    // Step 1: Read (async I/O)
    let content = self.read_file_content(file_path).await?;

    // Step 2: Parse (sync CPU - can be spawned to blocking pool)
    let (entities, dependencies) = tokio::task::spawn_blocking({
        let content = content.clone();
        let file_path = file_path.to_owned();
        let key_gen = Arc::clone(&self.key_generator);
        move || parse_file_to_entities(&content, &file_path, key_gen.as_ref())
    }).await??;

    // Step 3: Persist (async I/O)
    self.db.insert_entities_batch(&entities).await?;
    self.db.insert_edges_batch(&dependencies).await?;

    Ok(FileResult { /* ... */ })
}
```

**Benefits**:
- ✅ Pure functions are testable without database
- ✅ Can spawn parsing to blocking thread pool
- ✅ Clear separation of I/O and CPU work

**Feasibility**: **10/10** - No breaking changes, just refactoring

---

#### Phase 2: Parallelize LSP Enrichment (Feasibility: 9/10)

**Goal**: Fetch LSP metadata concurrently

**Current Sequential**:
```rust
for parsed_entity in parsed_entities {
    let lsp_metadata = self.fetch_lsp_metadata_for_entity(&parsed_entity, file_path).await;
    // ... use metadata
}
```

**Parallel with `join_all`**:
```rust
use futures::future::join_all;

// Launch all LSP requests concurrently
let lsp_futures: Vec<_> = parsed_entities
    .iter()
    .map(|entity| self.fetch_lsp_metadata_for_entity(entity, file_path))
    .collect();

let lsp_results: Vec<Option<LspMetadata>> = join_all(lsp_futures).await;

// Zip results with entities
for (parsed_entity, lsp_metadata) in parsed_entities.iter().zip(lsp_results) {
    // ... use metadata
}
```

**Benefits**:
- ✅ N concurrent LSP requests instead of N sequential
- ✅ Reduces LSP enrichment time by ~Nx (if LSP server can handle concurrency)

**Risks**:
- ⚠️ LSP server may throttle/reject concurrent requests
- ⚠️ Graceful degradation must handle timeouts

**Feasibility**: **9/10** - Simple async pattern, minor risk of LSP server overload

---

#### Phase 3: Streaming Database Writes (Feasibility: 7/10)

**Goal**: Insert entities as they're parsed, not in batch

**Current Batch**:
```rust
// Collect all entities first
let mut entities_to_insert = Vec::new();
for entity in parsed_entities {
    entities_to_insert.push(process_entity(entity)?);
}

// Insert all at once
self.db.insert_entities_batch(&entities_to_insert).await?;
```

**Streaming with Channels**:
```rust
use tokio::sync::mpsc;

// Create channel for entity streaming
let (entity_tx, mut entity_rx) = mpsc::channel::<CodeEntity>(100);

// Spawn parser task
let parse_task = tokio::spawn(async move {
    for entity in parsed_entities {
        let code_entity = process_entity(entity)?;
        entity_tx.send(code_entity).await?;
    }
    Ok::<_, Error>(())
});

// Spawn database writer task
let write_task = tokio::spawn(async move {
    let mut batch = Vec::with_capacity(100);
    while let Some(entity) = entity_rx.recv().await {
        batch.push(entity);

        // Flush batch when full
        if batch.len() >= 100 {
            self.db.insert_entities_batch(&batch).await?;
            batch.clear();
        }
    }

    // Flush remaining
    if !batch.is_empty() {
        self.db.insert_entities_batch(&batch).await?;
    }
    Ok::<_, Error>(())
});

// Wait for both tasks
tokio::try_join!(parse_task, write_task)?;
```

**Benefits**:
- ✅ Constant memory usage (bounded channel)
- ✅ Parallelizes parsing and database writes
- ✅ Lower latency for large files

**Risks**:
- ⚠️ **BLOCKER**: RocksDB only allows single writer (lock file)
- ⚠️ If write task fails, parsed entities are lost
- ⚠️ Error handling becomes more complex

**Feasibility**: **7/10** - Pattern is sound, but blocked by RocksDB single-writer limitation

---

#### Phase 4: Work-Stealing Executor (Feasibility: 8/10)

**Goal**: Process multiple files concurrently

**Current Sequential Directory Walk**:
```rust
for entry in WalkDir::new(&self.config.root_dir) {
    if path.is_file() && self.should_process_file(path) {
        self.stream_file(path).await?; // Sequential
    }
}
```

**Parallel with `FuturesUnordered`**:
```rust
use futures::stream::{FuturesUnordered, StreamExt};

let mut tasks = FuturesUnordered::new();

for entry in WalkDir::new(&self.config.root_dir) {
    if path.is_file() && self.should_process_file(path) {
        tasks.push(self.stream_file(path));

        // Limit concurrency to avoid overwhelming system
        if tasks.len() >= MAX_CONCURRENT_FILES {
            tasks.next().await; // Wait for one to complete
        }
    }
}

// Process remaining tasks
while let Some(result) = tasks.next().await {
    match result {
        Ok(file_result) => { /* ... */ },
        Err(e) => { /* ... */ },
    }
}
```

**Benefits**:
- ✅ N files processed concurrently
- ✅ Bounded concurrency prevents resource exhaustion
- ✅ Massive speedup for large codebases

**Risks**:
- ⚠️ **BLOCKER**: RocksDB lock prevents concurrent writes
- ⚠️ Error handling for partial failures

**Feasibility**: **8/10** - Pattern is solid, but requires multi-writer database or write queue

---

### Summary: Streamer Refactoring Path

| Phase | Feasibility | Blockers | Improvement |
|-------|-------------|----------|-------------|
| Phase 1: Extract Pure Functions | 10/10 | None | Testability, clarity |
| Phase 2: Parallelize LSP | 9/10 | LSP server throttling | Nx faster LSP enrichment |
| Phase 3: Streaming Writes | 7/10 | **RocksDB single writer** | Memory efficiency |
| Phase 4: Concurrent Files | 8/10 | **RocksDB single writer** | Nx faster ingestion |

**Overall Improvement**: **5/10 → 10/10** IF we solve the RocksDB single-writer problem

---

## CozoDbStorage Conversion Analysis

### Current Score: 6/10

### Why is this 6/10? (Root Cause Analysis)

**Problem Statement**: CozoDbStorage has **38 methods** that need async conversion, but underlying CozoDB API is synchronous.

**Methods Analyzed** (from Parseltongue query):
```
rust:method:insert_entity
rust:method:insert_entities_batch
rust:method:insert_edge
rust:method:insert_edges_batch
rust:method:get_entity
rust:method:get_all_entities
rust:method:delete_entity
rust:method:delete_entities_batch_by_keys
rust:method:calculate_blast_radius
rust:method:get_forward_dependencies
rust:method:get_reverse_dependencies
rust:method:query_entities
rust:method:raw_query
rust:method:execute_query
... (24 more methods)
```

### Current Implementation Pattern

**All methods follow this pattern** (from `cozo_client.rs`):

```rust
pub async fn insert_entity(&self, entity: &CodeEntity) -> Result<()> {
    // Build query string
    let query = r#"
        ?[ISGL1_key, Current_Code, ...] <- [[$ISGL1_key, $Current_Code, ...]]
        :put CodeGraph { ... }
    "#;

    // Build params
    let mut params = BTreeMap::new();
    params.insert("ISGL1_key".to_string(), DataValue::Str(entity.isgl1_key.as_ref().into()));
    // ... more params

    // Execute query (SYNCHRONOUS)
    self.db
        .run_script(query, params, ScriptMutability::Mutable)
        .map_err(|e| ParseltongError::DatabaseError { /* ... */ })?;

    Ok(())
}
```

**Key Observation**: `self.db.run_script()` is **synchronous** - the `async` keyword is a lie!

### Why is CozoDB Synchronous?

**From CozoDB documentation**:
- CozoDB backends (RocksDB, SQLite) use synchronous I/O
- `DbInstance::run_script()` signature: `fn run_script(...) -> Result<NamedRows>`
- No async/await in CozoDB API

**Implication**: Our `async fn` methods are **fake async** - they block the executor

---

### What Makes This Hard?

#### Blocker 1: CozoDB is Synchronous

**Current Code Pattern**:
```rust
pub async fn insert_entity(&self, entity: &CodeEntity) -> Result<()> {
    self.db.run_script(query, params, ScriptMutability::Mutable)?; // BLOCKS
    Ok(())
}
```

**Why This is a Problem**:
- Every database call blocks a Tokio worker thread
- Tokio is optimized for async I/O, not blocking calls
- Can lead to thread pool exhaustion under load

#### Blocker 2: RocksDB Single Writer Lock

**From RocksDB documentation**:
> "RocksDB uses a LOCK file to prevent multiple processes from opening the same database."

**Implication**:
- Only ONE `CozoDbStorage` instance can have write access
- Multiple concurrent tasks cannot write to the same database
- Kills the entire async pipeline vision

#### Blocker 3: 38 Methods to Convert

**Scope of Work**:
- 38 methods need `spawn_blocking` wrapper
- Each method needs error handling for `JoinError`
- Tests need updating for async patterns

**Example Conversion**:
```rust
// Before (fake async)
pub async fn insert_entity(&self, entity: &CodeEntity) -> Result<()> {
    self.db.run_script(query, params, ScriptMutability::Mutable)?;
    Ok(())
}

// After (real async with spawn_blocking)
pub async fn insert_entity(&self, entity: &CodeEntity) -> Result<()> {
    let query = build_query();
    let params = build_params(entity);
    let db = Arc::clone(&self.db);

    tokio::task::spawn_blocking(move || {
        db.run_script(query, params, ScriptMutability::Mutable)
    })
    .await
    .map_err(|e| ParseltongError::RuntimeError {
        details: format!("Task join error: {}", e)
    })?
    .map_err(|e| ParseltongError::DatabaseError { /* ... */ })?;

    Ok(())
}
```

---

### Conversion Strategy (6/10 → 10/10)

#### Strategy 1: Wrap All Methods with `spawn_blocking` (Feasibility: 8/10)

**Approach**: Keep CozoDB, wrap every method

**Conversion Pattern**:
```rust
pub async fn METHOD_NAME(&self, args: Args) -> Result<ReturnType> {
    let db = Arc::clone(&self.db);
    let args_owned = args.to_owned(); // Clone args for move

    tokio::task::spawn_blocking(move || {
        // Original synchronous implementation
        db.run_script(query, params, ScriptMutability::Mutable)
    })
    .await
    .map_err(|e| ParseltongError::RuntimeError {
        details: format!("Task panicked: {}", e)
    })??
}
```

**Pros**:
- ✅ Prevents blocking Tokio worker threads
- ✅ No database migration required
- ✅ Incremental rollout possible

**Cons**:
- ❌ Still limited by RocksDB single-writer lock
- ❌ Thread pool overhead for every query
- ❌ Cannot parallelize writes

**Feasibility**: **8/10** - Solves blocking but not concurrency

---

#### Strategy 2: Queue-Based Write Coordinator (Feasibility: 9/10)

**Approach**: Single writer thread + async queue

**Architecture**:
```rust
pub struct CozoDbStorage {
    db: Arc<DbInstance>,
    write_tx: mpsc::UnboundedSender<WriteCommand>,
}

enum WriteCommand {
    InsertEntity { entity: CodeEntity, response: oneshot::Sender<Result<()>> },
    InsertBatch { entities: Vec<CodeEntity>, response: oneshot::Sender<Result<()>> },
    Shutdown,
}

impl CozoDbStorage {
    pub async fn new(engine_spec: &str) -> Result<Self> {
        let db = DbInstance::new(...)?;
        let db = Arc::new(db);

        // Create write queue
        let (write_tx, mut write_rx) = mpsc::unbounded_channel();

        // Spawn single writer task
        let db_clone = Arc::clone(&db);
        tokio::spawn(async move {
            while let Some(cmd) = write_rx.recv().await {
                match cmd {
                    WriteCommand::InsertEntity { entity, response } => {
                        let result = Self::do_insert_entity(&db_clone, &entity);
                        let _ = response.send(result);
                    }
                    WriteCommand::InsertBatch { entities, response } => {
                        let result = Self::do_insert_batch(&db_clone, &entities);
                        let _ = response.send(result);
                    }
                    WriteCommand::Shutdown => break,
                }
            }
        });

        Ok(Self { db, write_tx })
    }

    pub async fn insert_entity(&self, entity: &CodeEntity) -> Result<()> {
        let (response_tx, response_rx) = oneshot::channel();

        self.write_tx.send(WriteCommand::InsertEntity {
            entity: entity.clone(),
            response: response_tx,
        })?;

        response_rx.await?
    }
}
```

**Pros**:
- ✅ Respects RocksDB single-writer constraint
- ✅ Non-blocking async API for callers
- ✅ Can batch writes for efficiency
- ✅ Clear error propagation via oneshot channels

**Cons**:
- ❌ Cannot parallelize writes (inherent to RocksDB)
- ❌ Adds complexity with channel management
- ❌ Shutdown coordination required

**Feasibility**: **9/10** - Solid pattern, works with RocksDB limitations

---

#### Strategy 3: Switch to Multi-Writer Database (Feasibility: 6/10)

**See**: `v152-DATABASE-ALTERNATIVES-RESEARCH.md` for full analysis

**Feasibility**: **6/10** - High impact but requires database migration

---

### Summary: CozoDbStorage Conversion Path

| Strategy | Feasibility | Blockers | Benefits |
|----------|-------------|----------|----------|
| Strategy 1: spawn_blocking | 8/10 | RocksDB lock | Non-blocking API |
| Strategy 2: Write Queue | 9/10 | None | Respects RocksDB, clean API |
| Strategy 3: New Database | 6/10 | Migration effort | True multi-writer |

**Recommended**: **Strategy 2** (Write Queue) for v1.5.2
- Achieves 10/10 async API without database migration
- Incremental path to Strategy 3 later

---

## Multi-Writer Database Problem

### The Core Issue

**RocksDB Lock File Mechanism**:
```
parseltongue20260207/
├── analysis.db/
│   ├── LOCK          ← Single process lock
│   ├── CURRENT
│   ├── MANIFEST-*
│   └── *.sst
```

**Error When Multiple Writers Attempt**:
```
Error: IO error: While lock file: parseltongue20260207/analysis.db/LOCK:
Resource temporarily unavailable
```

### Why This Kills Async Pipeline Dreams

**Async Pipeline Vision**:
```
File 1 → Parse → Write ─┐
File 2 → Parse → Write ─┼─→ Database (parallel writes)
File 3 → Parse → Write ─┘
```

**RocksDB Reality**:
```
File 1 → Parse → Write ──→ Database
File 2 → Parse → WAIT  ──→ Database (blocked by LOCK)
File 3 → Parse → WAIT  ──→ Database (blocked by LOCK)
```

### Solution Space

Three fundamental approaches:

1. **Work Around It**: Queue writes to single writer (Strategy 2)
2. **Replace Database**: Switch to multi-writer backend (Strategy 3)
3. **Hybrid**: Keep RocksDB, use message queue for coordination

**Detailed analysis in**: `v152-DATABASE-ALTERNATIVES-RESEARCH.md`

---

## Solution Options Matrix

### Option A: Keep RocksDB + Write Queue (Recommended)

**Feasibility**: **9/10**

**Architecture**:
```
Async Tasks (N concurrent)
    ↓
Write Queue (unbounded channel)
    ↓
Single Writer Task
    ↓
RocksDB (single writer)
```

**Implementation Steps**:
1. Refactor CozoDbStorage with write queue (Strategy 2)
2. Extract pure parsing functions (Phase 1)
3. Parallelize LSP enrichment (Phase 2)
4. Concurrent file processing with write queue (Phase 4)

**Timeline**: 2-3 days

**Pros**:
- ✅ No database migration
- ✅ Clean async API
- ✅ Incremental rollout
- ✅ Predictable performance

**Cons**:
- ❌ Single write thread bottleneck
- ❌ Cannot scale writes horizontally

---

### Option B: Switch to PostgreSQL

**Feasibility**: **7/10**

**Why PostgreSQL?**:
- True multi-writer MVCC
- Excellent Rust ecosystem (SQLx, Diesel)
- CozoDB supports `postgres://` backend
- Battle-tested reliability

**Migration Path**:
1. Add PostgreSQL feature flag to Cargo.toml
2. Update connection string parsing
3. Test schema creation with PostgreSQL backend
4. Benchmark performance vs RocksDB

**Timeline**: 1 week (includes testing)

**Pros**:
- ✅ True concurrent writes
- ✅ CozoDB already supports it
- ✅ SQL query capabilities
- ✅ Horizontal scaling

**Cons**:
- ❌ Requires PostgreSQL installation
- ❌ Higher operational complexity
- ❌ Network latency for queries
- ❌ Larger deployment footprint

---

### Option C: Hybrid (RocksDB + External Queue)

**Feasibility**: **5/10**

**Architecture**:
```
Async Tasks (N concurrent)
    ↓
Redis Queue / NATS
    ↓
Writer Process(es)
    ↓
RocksDB (per-process)
```

**Timeline**: 2+ weeks

**Pros**:
- ✅ Distributed write coordination
- ✅ Fault tolerance

**Cons**:
- ❌ High complexity
- ❌ External dependencies
- ❌ Over-engineered for current scale

---

## Recommended Path Forward

### v1.5.2 Incremental Plan

#### Milestone 1: Non-Blocking Database (Week 1)

**Goal**: Convert CozoDbStorage to non-blocking async

**Tasks**:
1. Implement Strategy 2 (Write Queue) in `cozo_client.rs`
2. Add graceful shutdown mechanism
3. Update all 38 methods to use queue
4. Add integration tests for concurrent writes

**Success Criteria**:
- ✅ All database methods are truly async
- ✅ No blocking calls on Tokio worker threads
- ✅ Tests pass with concurrent write stress

**Deliverable**: CozoDbStorage v2 with async queue

---

#### Milestone 2: Streaming Parser (Week 2)

**Goal**: Refactor streamer to use channels

**Tasks**:
1. Extract pure parsing functions (Phase 1)
2. Implement channel-based entity streaming
3. Parallelize LSP enrichment (Phase 2)
4. Add bounded concurrency for file processing

**Success Criteria**:
- ✅ Constant memory usage during ingestion
- ✅ LSP enrichment parallelized
- ✅ Multiple files processed concurrently

**Deliverable**: Streamer v2 with async pipeline

---

#### Milestone 3: Performance Validation (Week 3)

**Goal**: Measure improvements and validate contracts

**Tasks**:
1. Benchmark ingestion throughput (files/second)
2. Measure memory usage under load
3. Validate latency targets (p99 < 500μs for queries)
4. Document performance characteristics

**Success Criteria**:
- ✅ 2x+ throughput improvement
- ✅ Memory usage stays constant
- ✅ All performance contracts met

**Deliverable**: v1.5.2 release with benchmarks

---

### Future: v1.6.0 (Database Migration)

**IF** we hit write queue bottleneck:

1. Add PostgreSQL backend support
2. A/B test RocksDB vs PostgreSQL
3. Migrate production to PostgreSQL
4. Remove RocksDB dependency

**Timeline**: 1 month after v1.5.2 ships

---

## Conclusion

### Feasibility Scores After Analysis

| Criterion | Before | After Analysis | Path |
|-----------|--------|----------------|------|
| Streamer Refactoring | 5/10 | **10/10** | Phase 1-4 incremental |
| CozoDbStorage | 6/10 | **10/10** | Strategy 2 (Write Queue) |
| **Overall** | **6.5/10** | **10/10** | 3-week incremental plan |

### Key Insights

1. **RocksDB single-writer is the core blocker** - but we can work around it
2. **Write queue pattern is battle-tested** - used in production systems
3. **Incremental migration is low-risk** - each phase delivers value
4. **Database migration is not required** - but PostgreSQL is a viable option for v1.6.0

### Next Steps

1. **Read**: `v152-DATABASE-ALTERNATIVES-RESEARCH.md` for database options
2. **Implement**: Strategy 2 (Write Queue) in CozoDbStorage
3. **Refactor**: Streamer with pure functions and channels
4. **Measure**: Benchmark improvements and validate contracts

**Status**: Ready to proceed with v1.5.2 async pipeline implementation ✅

# v154 TDD Progress Journal: Rayon + Sled Parallel Ingestion

## Version Goal

**Target**: 5-7x faster ingestion through parallel processing and lock-free storage

**Strategy**:
1. Replace RocksDB with Sled (lock-free concurrent writes)
2. Use Rayon for parallel file parsing with tree-sitter
3. Maintain data integrity and correctness

## Success Metrics

- **Performance**: 5-7x speedup vs baseline
- **Correctness**: All entities and edges preserved
- **Reliability**: No race conditions or data corruption
- **Code Quality**: Zero TODOs, all tests passing

---

## Phase 0: Baseline Benchmark (Current State)

**Objective**: Establish performance baseline with current RocksDB sequential implementation

### Checklist
- [ ] Benchmark current RocksDB sequential ingestion on Parseltongue codebase
- [ ] Record: wall time, entities count, edges count, entities/sec, files processed
- [ ] Verify correctness: spot-check entities and dependencies
- [ ] Save as baseline for comparison
- [ ] Document system specs (CPU, cores, disk type)

### Status: PENDING

---

## Phase 1: Sled Backend Integration (TDD Cycle)

**Objective**: Switch storage backend from RocksDB to Sled while maintaining sequential processing

### Checklist
- [ ] **RED**: Write test for Sled initialization (`sled:` prefix support)
- [ ] **RED**: Write test for basic entity storage/retrieval with Sled
- [ ] **GREEN**: Add `storage-sled` feature to cozo dependency in Cargo.toml
- [ ] **GREEN**: Implement Sled backend support in pt01-folder-to-cozodb-streamer
- [ ] **GREEN**: Update CLI to accept `sled:` prefix (alongside `rocksdb:`)
- [ ] **VERIFY**: All existing tests pass with Sled backend
- [ ] **BENCHMARK**: Compare Sled sequential vs RocksDB sequential
- [ ] **REFACTOR**: Clean up any duplicated code

### Expected Outcome
- Sled works as drop-in replacement
- Performance similar or slightly better (sequential mode)
- Foundation for parallel writes established

---

## Phase 2: Rayon Parallel Parsing (TDD Cycle)

**Objective**: Parallelize file parsing using Rayon while maintaining correctness

### Checklist
- [ ] **RED**: Write test for parallel file discovery with rayon
- [ ] **RED**: Write test for thread-safe tree-sitter Parser handling
- [ ] **RED**: Write test for concurrent entity insertion (verify no duplicates/corruption)
- [ ] **GREEN**: Add rayon dependency to pt01-folder-to-cozodb-streamer
- [ ] **GREEN**: Refactor `stream_directory` to use `par_bridge()` or `par_iter()`
- [ ] **GREEN**: Implement thread-local Parser pool using `thread_local!` macro
- [ ] **GREEN**: Ensure Sled handles concurrent writes safely
- [ ] **VERIFY**: Correctness test - compare entity/edge count vs sequential
- [ ] **VERIFY**: Determinism test - multiple runs produce identical results
- [ ] **REFACTOR**: Optimize chunk size and thread pool configuration

### Technical Considerations
- **Tree-sitter Parser**: Not Send/Sync - use `thread_local!` for per-thread instances
- **Sled**: Lock-free, supports concurrent writes natively
- **Load Balancing**: Let Rayon handle work-stealing automatically

---

## Phase 3: Integration & Benchmark (VERIFICATION)

**Objective**: Validate full parallel implementation and measure speedup

### Checklist
- [ ] Full integration test: Sled + Rayon on Parseltongue codebase
- [ ] Benchmark on Parseltongue codebase (compare to Phase 0 baseline)
- [ ] Verify entity count matches baseline exactly
- [ ] Verify edge count matches baseline exactly
- [ ] Spot-check sample entities for correctness
- [ ] Calculate speedup ratio (target: 5-7x)
- [ ] Profile CPU utilization (should be near 100% across cores)
- [ ] Check memory usage (ensure no leaks)

### Benchmark Table Template
| Implementation | Time (s) | Entities | Edges | Files | Entities/sec | Speedup |
|----------------|----------|----------|-------|-------|--------------|---------|
| RocksDB Sequential | TBD | TBD | TBD | TBD | TBD | 1.0x |
| Sled Sequential | TBD | TBD | TBD | TBD | TBD | TBD |
| Sled + Rayon | TBD | TBD | TBD | TBD | TBD | **TBD** |

---

## Phase 4: Edge Cases & Cleanup (REFACTOR)

**Objective**: Production-ready implementation with comprehensive error handling

### Checklist
- [ ] Test graceful shutdown on Ctrl+C (flush Sled to disk)
- [ ] Test error handling: corrupt files, permission errors, disk full
- [ ] Test edge case: empty directory
- [ ] Test edge case: single file
- [ ] Test edge case: very large file (>10MB)
- [ ] Test on external codebase (if available) for validation
- [ ] Clean up any TODO/STUB comments
- [ ] Update pt01 help text to mention Sled and performance
- [ ] Update CLAUDE.md with Sled usage examples
- [ ] Add performance notes to README

### Documentation Updates
- [ ] Update CLI usage examples (show `sled:` prefix)
- [ ] Document performance characteristics
- [ ] Add benchmark results to docs
- [ ] Note when to use RocksDB vs Sled

---

## Session Log

### Session: 2026-02-07 (Initial Planning)

**Current Phase**: Phase 0 - Baseline Benchmark
**Status**: PLANNING
**TDD State**: N/A (pre-implementation)

#### What We Did
- Created TDD progress journal structure
- Defined 4-phase implementation strategy
- Established success criteria: 5-7x speedup, zero correctness issues
- Outlined comprehensive checklist for each phase

#### Test Results
None yet - no tests written

#### Measurements
None yet - baseline benchmark pending

#### Technical Decisions
1. **Storage Choice**: Sled over RocksDB
   - Rationale: Lock-free concurrent writes, simpler API
   - Risk: Less mature than RocksDB, but well-tested in production

2. **Parallelism Strategy**: Rayon for file-level parallelism
   - Rationale: Zero-cost work-stealing, automatic load balancing
   - Challenge: Tree-sitter Parser not Send/Sync - need thread_local!

3. **Test Strategy**: Correctness before performance
   - Phase 1: Verify Sled works (sequential)
   - Phase 2: Verify Rayon works (parallel)
   - Phase 3: Verify speedup target met

#### Blockers
None currently

#### Next Steps
1. Run baseline benchmark on current implementation
   ```bash
   # Clean slate
   cargo clean
   cargo build --release

   # Time full ingestion
   time target/release/parseltongue pt01-folder-to-cozodb-streamer .

   # Record metrics from output
   ```

2. Document system specifications:
   - CPU model and core count
   - RAM size
   - Disk type (SSD vs HDD)
   - Rust version
   - OS version

3. Save baseline results to this journal

#### Context Notes
- Current implementation: Sequential RocksDB with single-threaded tree-sitter parsing
- Target codebase: Parseltongue itself (multi-crate Rust project)
- Reference: Linux kernel ingestion takes ~X minutes (need to measure)
- Performance target based on: CPU core count × expected parallelism efficiency (0.7-0.9)

---

## Appendix: System Specifications

**To be filled after first benchmark run**

- **CPU**: [Model, Cores, Threads]
- **RAM**: [Size, Speed]
- **Disk**: [Type, Speed]
- **OS**: Darwin 24.3.0 (macOS)
- **Rust**: [rustc --version]
- **Cargo**: [cargo --version]

---

## Appendix: Dependencies Added

### Phase 1: Sled
```toml
# In crates/parseltongue-core/Cargo.toml or pt01's Cargo.toml
cozo = { version = "X.X", features = ["storage-rocksdb", "storage-sled"] }
```

### Phase 2: Rayon
```toml
# In crates/pt01-folder-to-cozodb-streamer/Cargo.toml
rayon = "1.8"
```

---

## Appendix: Key Code Changes Tracking

### Files Modified (to be updated as we progress)
- [ ] `crates/pt01-folder-to-cozodb-streamer/Cargo.toml` - Add dependencies
- [ ] `crates/pt01-folder-to-cozodb-streamer/src/lib.rs` - Parallel stream_directory
- [ ] `crates/parseltongue-core/src/parsing.rs` - Thread-local Parser (if needed)
- [ ] `crates/parseltongue/src/main.rs` - Update CLI docs (if needed)
- [ ] `CLAUDE.md` - Update examples with Sled
- [ ] `README.md` - Add performance notes

### Tests Added (to be updated as we progress)
- [ ] `test_sled_backend_initialization`
- [ ] `test_sled_entity_storage_retrieval`
- [ ] `test_parallel_file_discovery`
- [ ] `test_concurrent_entity_insertion`
- [ ] `test_parallel_correctness_vs_sequential`
- [ ] `test_determinism_multiple_runs`

---

## Notes for Future Sessions

### When Resuming Work
1. Check this journal's latest session entry
2. Review current phase checklist
3. Run `cargo test -p pt01-folder-to-cozodb-streamer` to verify starting state
4. Note any new blockers or insights in next session entry

### Before Each Commit
- [ ] All tests pass: `cargo test --all`
- [ ] No TODOs: `grep -r "TODO\|STUB" --include="*.rs" crates/`
- [ ] Benchmark data recorded in this journal
- [ ] Session entry updated with results

### Communication Protocol
- Always reference phase number and TDD state (RED/GREEN/REFACTOR)
- Include concrete measurements (time, entity count, speedup)
- Note any deviation from plan with rationale
- Flag blockers immediately with severity (blocking/non-blocking)

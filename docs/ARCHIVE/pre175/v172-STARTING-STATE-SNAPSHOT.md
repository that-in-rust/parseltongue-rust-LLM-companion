# v1.7.2 Starting State Snapshot

**Snapshot Timestamp**: 2026-02-12
**Purpose**: Capture exact starting state before v1.7.2 implementation begins

---

## File 1: cozo_client.rs (Current State)

**Path**: `crates/parseltongue-core/src/storage/cozo_client.rs`

### Lines 44-83: write_rocksdb_options_file_tuned() - TO BE DELETED
```rust
/// Write tuned RocksDB options file to prevent Windows 75MB write stall
///
/// Creates an options file in the RocksDB database directory with tuned settings
/// that prevent write stalls on Windows when ingesting large codebases (>75MB).
/// The tuning increases buffer sizes and background jobs to handle burst writes.
///
/// If the options file already exists, this function does nothing (preserves user customizations).
/// If writing fails, the error is ignored - the database will still work with defaults.
///
/// # Arguments
/// * `path` - RocksDB database directory path (e.g., "parseltongue20251201/analysis.db")
fn write_rocksdb_options_file_tuned(path: &str) {
    use std::fs;
    use std::io::Write;
    use std::path::Path;

    let options_path = Path::new(path).join("options");

    // Don't overwrite existing options file (preserve user customizations)
    if options_path.exists() {
        return;
    }

    // Create directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(path) {
        eprintln!("Warning: failed to create RocksDB directory {}: {}", path, e);
        return;
    }

    // Write tuned OPTIONS file
    // Buffer tuning from v1.6.6 (keep), direct IO from v1.6.7 (removed in v1.6.9)
    let options_content = r#"[DBOptions]
max_background_jobs=4
create_if_missing=true

[CFOptions "default"]
write_buffer_size=134217728
max_write_buffer_number=4
level0_slowdown_writes_trigger=40
level0_stop_writes_trigger=56
target_file_size_base=67108864
max_bytes_for_level_base=268435456
"#;

    if let Err(e) = fs::File::create(&options_path)
        .and_then(|mut f| f.write_all(options_content.as_bytes()))
    {
        eprintln!("Warning: failed to write RocksDB options file: {}", e);
        // Continue anyway - database will use defaults
    }
}
```

### Lines 123-126: Call to write_rocksdb_options_file_tuned() - TO BE DELETED
```rust
// Write tuned RocksDB options file if using rocksdb engine (fixes Windows 75MB write stall)
if engine == "rocksdb" && !path.is_empty() {
    write_rocksdb_options_file_tuned(path);
}
```

### Lines 104: Performance notes comment - TO BE UPDATED
```rust
/// # Performance Notes (v1.7.0)
/// - RocksDB: Fastest for Cozo's workload, recommended for Mac/Linux
/// - SQLite: Rock-solid, CozoDB-recommended for Windows (replaced abandoned Sled)
```

**TO BE REPLACED WITH**:
```rust
/// # Performance Notes (v1.7.2)
/// - RocksDB: Fastest for Cozo's workload, recommended for Mac/Linux
/// - mem→SQLite backup: Recommended for Windows (in-memory ingestion, atomic backup)
```

### NEW METHOD TO BE ADDED (after line 135)
```rust
    /// Backup database to a SQLite file using CozoDB's backup_db()
    ///
    /// Used for Windows mem→SQLite workflow: ingest into memory, then atomically
    /// save to SQLite file. The backup file IS a working SQLite database that
    /// can be opened directly with `sqlite:path`.
    ///
    /// # 4-Word Name: backup_to_sqlite_file
    ///
    /// # Arguments
    /// * `path` - Target SQLite file path (WITHOUT "sqlite:" prefix)
    ///
    /// # Example
    /// ```ignore
    /// // After ingesting into mem backend
    /// storage.backup_to_sqlite_file("parseltongue20251201/analysis.db").await?;
    /// // Now open with: CozoDbStorage::new("sqlite:parseltongue20251201/analysis.db")
    /// ```
    pub async fn backup_to_sqlite_file(&self, path: &str) -> Result<()> {
        self.db.backup_db(path).map_err(|e| ParseltongError::DatabaseError {
            operation: "backup_to_sqlite_file".to_string(),
            details: format!("Backup failed: {:?}", e),
        })
    }
```

---

## File 2: streamer.rs (Current State)

**Path**: `crates/pt01-folder-to-cozodb-streamer/src/streamer.rs`

### Lines 776-839: Platform-aware batch insert - TO BE REPLACED

**Current Code** (lines 776-839):
```rust
        // v1.7.1: Platform-aware batch insert strategy
        // - Mac/Linux (RocksDB): concurrent via tokio::join! (RocksDB handles concurrent writes natively)
        // - Windows (SQLite): sequential (SQLite is single-writer; concurrent writes cause silent SQLITE_BUSY failures)

        #[cfg(not(target_os = "windows"))]
        let (
            result_entities,
            result_edges,
            result_excluded_tests,
            result_word_coverage,
            result_ignored_files,
        ) = tokio::join!(
            // Task 1: Insert entities
            async {
                if all_entities.is_empty() {
                    Ok(())
                } else {
                    self.db.insert_entities_batch(&all_entities).await
                }
            },
            // Task 2: Insert dependency edges
            async {
                if all_dependencies.is_empty() {
                    Ok(())
                } else {
                    self.db.insert_edges_batch(&all_dependencies).await
                }
            },
            // Task 3: Insert excluded test entities
            async {
                if all_excluded_tests.is_empty() {
                    Ok(())
                } else {
                    self.db.insert_test_entities_excluded_batch(&all_excluded_tests).await
                }
            },
            // Task 4: Insert file word coverage
            async {
                if all_word_coverages.is_empty() {
                    Ok(())
                } else {
                    self.db.insert_file_word_coverage_batch(&all_word_coverages).await
                }
            },
            // Task 5: Insert ignored files
            async {
                if ignored_files.is_empty() {
                    Ok(())
                } else {
                    self.db.insert_ignored_files_batch(&ignored_files).await
                }
            },
        );

        #[cfg(target_os = "windows")]
        let (result_entities, result_edges, result_excluded_tests, result_word_coverage, result_ignored_files) = {
            // SQLite single-writer: serialize all batch inserts to avoid SQLITE_BUSY silent failures
            let result_entities = if all_entities.is_empty() { Ok(()) } else { self.db.insert_entities_batch(&all_entities).await };
            let result_edges = if all_dependencies.is_empty() { Ok(()) } else { self.db.insert_edges_batch(&all_dependencies).await };
            let result_excluded_tests = if all_excluded_tests.is_empty() { Ok(()) } else { self.db.insert_test_entities_excluded_batch(&all_excluded_tests).await };
            let result_word_coverage = if all_word_coverages.is_empty() { Ok(()) } else { self.db.insert_file_word_coverage_batch(&all_word_coverages).await };
            let result_ignored_files = if ignored_files.is_empty() { Ok(()) } else { self.db.insert_ignored_files_batch(&ignored_files).await };
            (result_entities, result_edges, result_excluded_tests, result_word_coverage, result_ignored_files)
        };
```

**TO BE REPLACED WITH** (lines 776-828 only):
```rust
        // v1.7.2: Concurrent batch inserts work for both mem and rocksdb backends
        // Windows uses mem backend during ingestion (backup_db() handled in main.rs)
        let (
            result_entities,
            result_edges,
            result_excluded_tests,
            result_word_coverage,
            result_ignored_files,
        ) = tokio::join!(
            // Task 1: Insert entities
            async {
                if all_entities.is_empty() {
                    Ok(())
                } else {
                    self.db.insert_entities_batch(&all_entities).await
                }
            },
            // Task 2: Insert dependency edges
            async {
                if all_dependencies.is_empty() {
                    Ok(())
                } else {
                    self.db.insert_edges_batch(&all_dependencies).await
                }
            },
            // Task 3: Insert excluded test entities
            async {
                if all_excluded_tests.is_empty() {
                    Ok(())
                } else {
                    self.db.insert_test_entities_excluded_batch(&all_excluded_tests).await
                }
            },
            // Task 4: Insert file word coverage
            async {
                if all_word_coverages.is_empty() {
                    Ok(())
                } else {
                    self.db.insert_file_word_coverage_batch(&all_word_coverages).await
                }
            },
            // Task 5: Insert ignored files
            async {
                if ignored_files.is_empty() {
                    Ok(())
                } else {
                    self.db.insert_ignored_files_batch(&ignored_files).await
                }
            },
        );
```

### NEW METHOD TO BE ADDED (around line 1432, before #[cfg(test)])
```rust
    /// Backup in-memory database to SQLite file (Windows mem→SQLite workflow)
    ///
    /// # 4-Word Name: backup_to_sqlite
    pub async fn backup_to_sqlite(&self, path: &str) -> Result<()> {
        self.db.backup_to_sqlite_file(path).await
    }
```

---

## File 3: main.rs (Current State)

**Path**: `crates/parseltongue/src/main.rs`

### Lines 127-142: Current engine selection - TO BE REPLACED

**Current Code**:
```rust
    // Construct database path within workspace
    let workspace_db_path = if db == "mem" {
        "mem".to_string()
    } else {
        // v1.7.0: Windows uses SQLite (CozoDB-recommended, stable, no data loss)
        // Mac/Linux uses RocksDB (fastest)
        #[cfg(target_os = "windows")]
        {
            println!("  Engine: {} (optimized for Windows)", style("SQLite").green());
            format!("sqlite:{}/analysis.db", workspace_dir)
        }
        #[cfg(not(target_os = "windows"))]
        {
            format!("rocksdb:{}/analysis.db", workspace_dir)
        }
    };
```

**TO BE REPLACED WITH**:
```rust
    // Windows: ingest into memory, then backup to SQLite
    #[cfg(target_os = "windows")]
    let (workspace_db_path, backup_needed) = if db == "mem" {
        ("mem".to_string(), false)
    } else {
        println!("  Engine: {} (in-memory ingestion → SQLite backup)", style("mem→SQLite").green());
        ("mem".to_string(), true)
    };
    #[cfg(target_os = "windows")]
    let backup_target = format!("{}/analysis.db", workspace_dir);

    // Mac/Linux: direct RocksDB (unchanged)
    #[cfg(not(target_os = "windows"))]
    let workspace_db_path = if db == "mem" {
        "mem".to_string()
    } else {
        format!("rocksdb:{}/analysis.db", workspace_dir)
    };
    #[cfg(not(target_os = "windows"))]
    let backup_needed = false;
```

### NEW CODE TO BE INSERTED (after line 173, before error log section at line 176)
```rust
    // Windows: backup mem database to SQLite file
    #[cfg(target_os = "windows")]
    if backup_needed {
        println!("  {} Saving to disk...", style("↓").cyan());
        streamer.backup_to_sqlite(&backup_target)?;
        println!("  Saved: {}", style(&backup_target).yellow());
    }
```

### Lines 207-209: Display path logic - TO BE UPDATED

**Current Code**:
```rust
        println!("{}", style("Next step:").cyan());
        println!("  parseltongue pt08-http-code-query-server \\");
        println!("    --db \"{}\"", workspace_db_path);
```

**TO BE REPLACED WITH**:
```rust
        println!("{}", style("Next step:").cyan());
        #[cfg(target_os = "windows")]
        let display_db_path = if backup_needed {
            format!("sqlite:{}", backup_target)
        } else {
            workspace_db_path.clone()
        };
        #[cfg(not(target_os = "windows"))]
        let display_db_path = workspace_db_path.clone();

        println!("  parseltongue pt08-http-code-query-server \\");
        println!("    --db \"{}\"", display_db_path);
```

---

## File 4: ingestion_coverage_folder_handler.rs (Current State)

**Path**: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/ingestion_coverage_folder_handler.rs`

### Lines 254-258: derive_workspace_directory_from_database() - TO BE UPDATED

**Current Code**:
```rust
fn derive_workspace_directory_from_database(db_path: &str) -> PathBuf {
    // Strip engine prefix (rocksdb:, sled:, or sqlite:) if present
    let path_str = db_path.strip_prefix("rocksdb:")
        .or_else(|| db_path.strip_prefix("sled:"))
        .or_else(|| db_path.strip_prefix("sqlite:"))
        .unwrap_or(db_path);
```

**TO BE REPLACED WITH**:
```rust
fn derive_workspace_directory_from_database(db_path: &str) -> PathBuf {
    // Strip engine prefix (rocksdb: or sqlite:) if present
    let path_str = db_path.strip_prefix("rocksdb:")
        .or_else(|| db_path.strip_prefix("sqlite:"))
        .unwrap_or(db_path);
```

---

## Git Status (Before Changes)

```
Current branch: main

Status:
D test-fixtures/v151-edge-bug-repro/.gitignore
 D test-fixtures/v151-edge-bug-repro/EXPECTED.md
 D test-fixtures/v151-edge-bug-repro/Program.cs
 D test-fixtures/v151-edge-bug-repro/QualifiedNames.cs
 D test-fixtures/v151-edge-bug-repro/app.js
 D test-fixtures/v151-edge-bug-repro/modules.rb
 D test-fixtures/v151-edge-bug-repro/namespaces.cpp
 D test-fixtures/v151-edge-bug-repro/namespaces.go
 D test-fixtures/v151-edge-bug-repro/namespaces.java
 D test-fixtures/v151-edge-bug-repro/namespaces.php
 D test-fixtures/v151-edge-bug-repro/namespaces.py
 D test-fixtures/v151-edge-bug-repro/namespaces.rs
 D test-fixtures/v151-edge-bug-repro/service.ts
?? FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md
?? MASTER_FEATURE_EXTRACTION_TABLE.md
?? test-fixtures/v156-sql-test/

Recent commits:
a9fca5f62 Merge pull request #6 from that-in-rust/v155
a0c2b5e94 feat(v1.5.6): generic type sanitization + Windows/PHP backslash fix + SQL infrastructure
31013ea08 Merge pull request #5 from that-in-rust/main
ae7f082b2 Merge branch 'that-in-rust:main' into main
b415e1534 fix: resolve RwLock deadlock in HTTP endpoints (v1.5.4)
```

---

## Implementation Order (Critical)

1. **FIRST**: `cozo_client.rs` - Defines `backup_to_sqlite_file()` method
2. **SECOND**: `streamer.rs` - Depends on cozo_client.rs method
3. **THIRD**: `main.rs` - Depends on streamer.rs method
4. **FOURTH**: `ingestion_coverage_folder_handler.rs` - Independent, can be done anytime

---

## Verification After Implementation

### Build Commands
```bash
cargo build --release
cargo test --all
cargo clean && cargo build --release  # Full rebuild verification
```

### Smoke Test (Mac)
```bash
parseltongue pt01-folder-to-cozodb-streamer .
# Expected output: rocksdb:parseltongueXXXX/analysis.db
# Expected entities: ~1599 for self-ingestion

parseltongue pt08-http-code-query-server --db "rocksdb:parseltongueXXXX/analysis.db"
curl http://localhost:7777/codebase-statistics-overview-summary
# Should show ~1599 entities
```

### Expected Windows Output (Manual Test)
```
Running Tool 1: folder-to-cozodb-streamer
  Workspace: parseltongue20260212HHMMSS
  Engine: mem→SQLite (in-memory ingestion → SQLite backup)

Starting PARALLEL directory streaming (v1.5.4 Rayon)...
[processing output]

  ↓ Saving to disk...
  Saved: parseltongue20260212HHMMSS/analysis.db

✓ Indexing completed
  Files processed: XXX
  Entities created: XXX

Next step:
  parseltongue pt08-http-code-query-server \
    --db "sqlite:parseltongue20260212HHMMSS/analysis.db"
```

---

**Snapshot Complete**: Ready for rust-coder-01 agent to begin implementation

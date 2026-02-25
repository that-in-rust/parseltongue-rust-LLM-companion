# Windows-Specific Code to Remove for v1.7.2

This document lists ALL Windows-specific code added in v1.6.7 through v1.7.1 that needs to be undone for the v1.7.2 mem→SQLite backup strategy.

## Summary of Changes Needed

| File | Lines to Remove/Change | Reason |
|------|----------------------|---------|
| `main.rs` | 131-142 | Remove Windows SQLite engine selection |
| `streamer.rs` | 780-839 | Remove Windows sequential insert code |
| `cozo_client.rs` | 44-83 | Remove RocksDB OPTIONS file tuning |
| `ingestion_coverage_folder_handler.rs` | 256 | Remove sled: prefix stripping (keep sqlite:) |

---

## File 1: `crates/parseltongue/src/main.rs`

### Location: Lines 131-142 (run_folder_to_cozodb_streamer function)

**Current Code (Windows-specific engine selection):**
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

**What to Replace With (v1.7.2 mem→SQLite backup):**
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

### NEW: After ingestion completes (insert after line 173)

**Add this BEFORE the ingestion error log section:**
```rust
// Windows: backup mem database to SQLite file
#[cfg(target_os = "windows")]
if backup_needed {
    println!("  {} Saving to disk...", style("↓").cyan());
    streamer.backup_to_sqlite(&backup_target)?;
    println!("  Saved: {}", style(&backup_target).yellow());
}
```

### NEW: Update display_db_path for Windows (replace lines 208-209)

**Current Code:**
```rust
println!("  parseltongue pt08-http-code-query-server \\");
println!("    --db \"{}\"", workspace_db_path);
```

**Replace With:**
```rust
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

## File 2: `crates/pt01-folder-to-cozodb-streamer/src/streamer.rs`

### Location: Lines 780-839 (Windows sequential insert code)

**Current Code (Windows-specific sequential inserts):**
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

**Replace With (v1.7.2 - ONLY tokio::join!, works for both mem and rocksdb):**
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

### NEW: Add backup_to_sqlite method (add near end of impl FileStreamerImpl)

**Add this new method (around line 1432, before the #[cfg(test)] section):**
```rust
    /// Backup in-memory database to SQLite file (Windows mem→SQLite workflow)
    ///
    /// # 4-Word Name: backup_to_sqlite
    pub async fn backup_to_sqlite(&self, path: &str) -> Result<()> {
        self.db.backup_to_sqlite_file(path).await
    }
```

---

## File 3: `crates/parseltongue-core/src/storage/cozo_client.rs`

### Location 1: Lines 44-83 (write_rocksdb_options_file_tuned function)

**Current Code (Windows RocksDB tuning - REMOVE ENTIRE FUNCTION):**
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

**Action:** DELETE ENTIRE FUNCTION (lines 44-83)

### Location 2: Lines 123-126 (call to write_rocksdb_options_file_tuned)

**Current Code:**
```rust
// Write tuned RocksDB options file if using rocksdb engine (fixes Windows 75MB write stall)
if engine == "rocksdb" && !path.is_empty() {
    write_rocksdb_options_file_tuned(path);
}
```

**Action:** DELETE these 3 lines

### Location 3: Update comment in `new()` method (around line 104)

**Current Code:**
```rust
/// # Performance Notes (v1.7.0)
/// - RocksDB: Fastest for Cozo's workload, recommended for Mac/Linux
/// - SQLite: Rock-solid, CozoDB-recommended for Windows (replaced abandoned Sled)
```

**Update To:**
```rust
/// # Performance Notes (v1.7.2)
/// - RocksDB: Fastest for Cozo's workload, recommended for Mac/Linux
/// - mem→SQLite backup: Recommended for Windows (in-memory ingestion, atomic backup)
```

### NEW: Add backup_to_sqlite_file method

**Add this new method to impl CozoDbStorage (around line 135, after the `is_connected` method):**
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

## File 4: `crates/pt08-http-code-query-server/.../ingestion_coverage_folder_handler.rs`

### Location: Lines 254-256 (derive_workspace_directory_from_database function)

**Current Code:**
```rust
// Strip engine prefix (rocksdb:, sled:, or sqlite:) if present
let path_str = db_path.strip_prefix("rocksdb:")
    .or_else(|| db_path.strip_prefix("sled:"))
    .or_else(|| db_path.strip_prefix("sqlite:"))
    .unwrap_or(db_path);
```

**Replace With (remove sled:, keep sqlite:):**
```rust
// Strip engine prefix (rocksdb: or sqlite:) if present
let path_str = db_path.strip_prefix("rocksdb:")
    .or_else(|| db_path.strip_prefix("sqlite:"))
    .unwrap_or(db_path);
```

---

## Verification Checklist

After making changes, verify:

- [ ] `cargo build --release` succeeds
- [ ] `cargo test --all` passes
- [ ] No TODOs/STUBs in modified code
- [ ] All function names still follow 4WNC
- [ ] Windows-specific code ONLY in `main.rs` and `streamer.rs` (backup logic)
- [ ] Mac/Linux behavior unchanged (RocksDB, tokio::join!)
- [ ] HTTP server unchanged (opens sqlite: or rocksdb: based on --db flag)

---

## Summary of Strategy

### What REMOVES Windows-specific complexity:
1. **No RocksDB OPTIONS file tuning** - not needed, mem backend has no files
2. **No sequential inserts** - mem backend is pure in-memory BTreeMap, no write contention
3. **No Sled references** - abandoned project, fully replaced

### What ADDS (Windows-only):
1. **mem backend for ingestion** - zero filesystem interaction during writes
2. **backup_db() after ingestion** - atomic SQLite file creation
3. **Display correct `sqlite:` prefix** - for HTTP server startup command

### What STAYS unchanged:
- Mac/Linux: RocksDB + tokio::join! (no changes)
- HTTP server: opens sqlite: or rocksdb: based on --db flag (no changes)
- Test suite: runs on Mac/Linux with mem or rocksdb (no changes)
- All 22 endpoints: no changes

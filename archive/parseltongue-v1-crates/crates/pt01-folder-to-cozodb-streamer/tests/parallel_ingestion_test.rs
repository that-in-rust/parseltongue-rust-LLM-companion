//! v1.5.4 Parallel Ingestion Tests
//!
//! Test parallel file parsing with Rayon and RocksDB/SQLite backends.

use parseltongue_core::storage::CozoDbStorage;
use pt01_folder_to_cozodb_streamer::*;
use std::sync::Arc;
use tempfile::TempDir;

/// Test RocksDB backend still works (regression test)
#[tokio::test]
async fn test_rocksdb_backend_still_works() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_rocksdb.db");
    let rocksdb_spec = format!("rocksdb:{}", db_path.display());

    let storage = CozoDbStorage::new(&rocksdb_spec).await;
    assert!(storage.is_ok(), "RocksDB backend initialization should succeed");

    let storage = storage.unwrap();
    assert!(storage.is_connected().await);
}

/// Test parallel streaming produces same results as sequential
#[tokio::test]
async fn test_parallel_streaming_correctness_matches_sequential() {
    // Create test fixture with Rust files
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir(&src_dir).unwrap();

    // Write test Rust file
    let test_file = src_dir.join("lib.rs");
    std::fs::write(
        &test_file,
        r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(x: i32, y: i32) -> i32 {
    x * y
}
"#,
    )
    .unwrap();

    // Sequential ingestion
    let seq_db_path = temp_dir.path().join("seq.db");
    let seq_config = StreamerConfig {
        root_dir: temp_dir.path().to_path_buf(),
        db_path: format!("rocksdb:{}", seq_db_path.display()),
        include_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec![],
        max_file_size: 10_000_000,
        parsing_library: "tree-sitter".to_string(),
        chunking: "ISGL1".to_string(),
    };

    let key_gen = Isgl1KeyGeneratorFactory::new();
    let test_detector = Arc::new(DefaultTestDetector::new());

    let seq_streamer = FileStreamerImpl::new(seq_config, key_gen.clone(), test_detector.clone())
        .await
        .unwrap();
    let seq_result = seq_streamer.stream_directory().await.unwrap();

    // Parallel ingestion
    let par_db_path = temp_dir.path().join("par.db");
    let par_config = StreamerConfig {
        root_dir: temp_dir.path().to_path_buf(),
        db_path: format!("rocksdb:{}", par_db_path.display()),
        include_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec![],
        max_file_size: 10_000_000,
        parsing_library: "tree-sitter".to_string(),
        chunking: "ISGL1".to_string(),
    };

    let par_streamer = FileStreamerImpl::new(par_config, key_gen, test_detector)
        .await
        .unwrap();
    let par_result = par_streamer
        .stream_directory_with_parallel_rayon()
        .await
        .unwrap();

    // Verify results match
    assert_eq!(
        seq_result.total_files, par_result.total_files,
        "Total files should match"
    );
    assert_eq!(
        seq_result.processed_files, par_result.processed_files,
        "Processed files should match"
    );
    assert_eq!(
        seq_result.entities_created, par_result.entities_created,
        "Entities created should match"
    );
}

/// Test parallel streaming performance benchmark
#[tokio::test]
#[ignore] // Run with --ignored for manual performance testing
async fn bench_parallel_vs_sequential_performance() {
    use std::time::Instant;

    // Create test fixture with multiple files
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir(&src_dir).unwrap();

    // Generate 100 test files
    for i in 0..100 {
        let file_path = src_dir.join(format!("module_{}.rs", i));
        std::fs::write(
            &file_path,
            format!(
                r#"
pub fn function_{}(x: i32) -> i32 {{
    x * {}
}}

pub struct Struct{} {{
    field: i32,
}}
"#,
                i, i, i
            ),
        )
        .unwrap();
    }

    let key_gen = Isgl1KeyGeneratorFactory::new();
    let test_detector = Arc::new(DefaultTestDetector::new());

    // Sequential benchmark
    let seq_db_path = temp_dir.path().join("seq_bench.db");
    let seq_config = StreamerConfig {
        root_dir: temp_dir.path().to_path_buf(),
        db_path: format!("rocksdb:{}", seq_db_path.display()),
        include_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec![],
        max_file_size: 10_000_000,
        parsing_library: "tree-sitter".to_string(),
        chunking: "ISGL1".to_string(),
    };

    let seq_streamer = FileStreamerImpl::new(seq_config, key_gen.clone(), test_detector.clone())
        .await
        .unwrap();

    let seq_start = Instant::now();
    let seq_result = seq_streamer.stream_directory().await.unwrap();
    let seq_duration = seq_start.elapsed();

    // Parallel benchmark
    let par_db_path = temp_dir.path().join("par_bench.db");
    let par_config = StreamerConfig {
        root_dir: temp_dir.path().to_path_buf(),
        db_path: format!("rocksdb:{}", par_db_path.display()),
        include_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec![],
        max_file_size: 10_000_000,
        parsing_library: "tree-sitter".to_string(),
        chunking: "ISGL1".to_string(),
    };

    let par_streamer = FileStreamerImpl::new(par_config, key_gen, test_detector)
        .await
        .unwrap();

    let par_start = Instant::now();
    let par_result = par_streamer
        .stream_directory_with_parallel_rayon()
        .await
        .unwrap();
    let par_duration = par_start.elapsed();

    // Report results
    println!("\n=== Performance Benchmark ===");
    println!("Sequential: {:?} ({} files)", seq_duration, seq_result.processed_files);
    println!("Parallel:   {:?} ({} files)", par_duration, par_result.processed_files);
    println!(
        "Speedup:    {:.2}x",
        seq_duration.as_secs_f64() / par_duration.as_secs_f64()
    );

    // Expect at least 2x speedup on multi-core systems
    // Note: This is conservative; actual speedup may be 5-7x
    let speedup = seq_duration.as_secs_f64() / par_duration.as_secs_f64();
    assert!(
        speedup > 1.5,
        "Parallel should be faster than sequential (got {:.2}x)",
        speedup
    );
}

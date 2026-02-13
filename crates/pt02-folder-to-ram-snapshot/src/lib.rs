//! Parseltongue Tool 02: folder-to-ram-snapshot
//!
//! Generates MessagePack (.ptgraph) snapshots of dependency graphs.
//! Snapshots contain slim entity/edge data (no code bodies) and load
//! into CozoDB `mem` for zero-disk-write operation on Windows.
//!
//! ## CLI Examples
//!
//! ```bash
//! # Generate snapshot from current directory
//! parseltongue pt02-folder-to-ram-snapshot .
//!
//! # Output: parseltongue20260213120000/analysis.ptgraph
//! ```
//!
//! ## How it Works
//!
//! 1. Uses pt01 to parse codebase into CozoDB `mem` (RAM-only)
//! 2. Exports slim entities (9 fields) and edges (3 fields)
//! 3. Serializes to MessagePack format
//! 4. Writes `.ptgraph` file to timestamped workspace
//!
//! ## Windows Support
//!
//! This is the **primary ingestion tool for Windows**. RocksDB fails on Windows
//! due to Windows Defender locking SST files during write-heavy operations.
//! The .ptgraph format bypasses this entirely by using in-memory storage.

#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
#![allow(missing_docs)]

use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use console::style;
use parseltongue_core::entities::PtGraphSnapshotContainer;
use pt01_folder_to_cozodb_streamer::{FileStreamer, StreamerConfig, ToolFactory};

/// Generate a .ptgraph snapshot file from a source directory.
///
/// # 4-Word Name: generate_ptgraph_snapshot_file
///
/// # Arguments
/// * `source_directory` - Path to the codebase to analyze
/// * `workspace_directory` - Path to the timestamped workspace (where .ptgraph will be written)
///
/// # Returns
/// Path to the generated .ptgraph file
///
/// # Performance Contract
/// - Parsing + Export: Same as pt01 (~5s for 400K entities)
/// - Serialization: ~500ms for 400K entities
/// - File write: ~200ms for typical snapshot (2-8 MB)
///
/// # Example
/// ```no_run
/// use pt02_folder_to_ram_snapshot::generate_ptgraph_snapshot_file;
/// # async fn example() -> anyhow::Result<()> {
/// let output_path = generate_ptgraph_snapshot_file(
///     "/path/to/codebase",
///     "parseltongue20260213120000"
/// ).await?;
/// println!("Snapshot written to: {}", output_path.display());
/// # Ok(())
/// # }
/// ```
pub async fn generate_ptgraph_snapshot_file(
    source_directory: impl AsRef<Path>,
    workspace_directory: impl AsRef<Path>,
) -> Result<PathBuf> {
    let source_path = source_directory.as_ref();
    let workspace_path = workspace_directory.as_ref();

    // Validate inputs
    if !source_path.exists() {
        anyhow::bail!("Source directory does not exist: {}", source_path.display());
    }

    // Step 1: Create StreamerConfig with db_path = "mem"
    let config = StreamerConfig {
        root_dir: source_path.to_path_buf(),
        db_path: "mem".to_string(), // Critical: in-memory storage only
        max_file_size: 100 * 1024 * 1024, // 100MB
        include_patterns: vec!["*".to_string()], // All files - tree-sitter handles filtering
        exclude_patterns: vec![
            "target".to_string(),
            "node_modules".to_string(),
            ".git".to_string(),
            "build".to_string(),
            "dist".to_string(),
            "__pycache__".to_string(),
            ".venv".to_string(),
            "venv".to_string(),
        ],
        parsing_library: "tree-sitter".to_string(),
        chunking: "ISGL1".to_string(),
    };

    println!("{}", style("Step 1/5: Parsing codebase with pt01 (RAM-only)...").cyan());

    // Step 2: Create streamer and parse the directory
    let streamer = ToolFactory::create_streamer(config)
        .await
        .context("Failed to create file streamer")?;

    // Step 3: Stream directory with parallel Rayon (same as pt01)
    let result = streamer
        .stream_directory_with_parallel_rayon()
        .await
        .context("Failed to parse source directory")?;

    println!(
        "  {} entities parsed from {} files",
        style(result.entities_created).yellow(),
        style(result.processed_files).yellow()
    );

    println!("{}", style("Step 2/5: Exporting slim entities from database...").cyan());

    // Step 4: Get database handle
    let db = streamer.get_database_storage_reference();

    // CRITICAL: Ensure DependencyEdges schema exists before export
    // The parallel streaming path (stream_directory_with_parallel_rayon) only creates
    // this schema if there are dependencies. We need it to exist even for empty graphs
    // so the export query doesn't fail.
    let _ = db.create_dependency_edges_schema().await;

    // Step 5: Export slim entities
    let entities = db
        .export_slim_entities_from_database()
        .await
        .context("Failed to export slim entities from database")?;

    println!("  {} entities exported", style(entities.len()).yellow());

    println!("{}", style("Step 3/5: Exporting slim edges from database...").cyan());

    // Step 6: Export slim edges
    let edges = db
        .export_slim_edges_from_database()
        .await
        .context("Failed to export slim edges from database")?;

    println!("  {} edges exported", style(edges.len()).yellow());

    println!("{}", style("Step 4/5: Building snapshot container...").cyan());

    // Step 7: Build PtGraphSnapshotContainer
    let container = PtGraphSnapshotContainer {
        version: "1.7.3".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        source_directory: source_path.to_string_lossy().to_string(),
        entity_count: entities.len(),
        edge_count: edges.len(),
        entities,
        edges,
    };

    println!("{}", style("Step 5/5: Serializing to MessagePack...").cyan());

    // Step 8: Serialize to MessagePack
    let bytes = rmp_serde::to_vec(&container)
        .context("Failed to serialize snapshot to MessagePack")?;

    let file_size_kb = bytes.len() / 1024;
    println!("  Serialized {} KB", style(file_size_kb).yellow());

    // Step 9: Write to .ptgraph file
    let output_path = workspace_path.join("analysis.ptgraph");
    std::fs::write(&output_path, &bytes)
        .with_context(|| format!("Failed to write snapshot to {}", output_path.display()))?;

    // Print summary
    println!();
    println!("{}", style("✓ Snapshot generation completed").green().bold());
    println!("  Entities: {}", style(container.entity_count).yellow());
    println!("  Edges: {}", style(container.edge_count).yellow());
    println!("  File size: {} KB", style(file_size_kb).yellow());
    println!("  Output: {}", style(output_path.display()).yellow().bold());
    println!();

    Ok(output_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_snapshot_with_empty_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let workspace_dir = tempfile::tempdir().unwrap();

        let result = generate_ptgraph_snapshot_file(
            temp_dir.path(),
            workspace_dir.path()
        ).await;

        // Should succeed even with empty directory
        match &result {
            Ok(_) => {},
            Err(e) => eprintln!("Error: {:#?}", e),
        }
        assert!(result.is_ok());

        let output_path = result.unwrap();
        assert!(output_path.exists());
        assert_eq!(output_path.file_name().unwrap(), "analysis.ptgraph");
    }
}

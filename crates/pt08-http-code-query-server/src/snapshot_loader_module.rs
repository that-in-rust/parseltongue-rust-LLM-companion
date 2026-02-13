//! Snapshot loader for .ptgraph files (v1.7.3)
//!
//! Loads MessagePack-serialized slim graph snapshots into CozoDB mem databases.

use parseltongue_core::{CozoDbStorage, PtGraphSnapshotContainer};
use anyhow::{Context, Result};
use console::style;

/// Load a .ptgraph snapshot file into a CozoDB mem database.
///
/// Reads the file, deserializes from MessagePack, creates a fresh CozoDB mem
/// instance, creates schemas, and bulk-inserts all entities and edges.
///
/// # 4-Word Name: load_ptgraph_snapshot_database
pub async fn load_ptgraph_snapshot_database(ptgraph_path: &str) -> Result<CozoDbStorage> {
    // 1. Read file
    let bytes = std::fs::read(ptgraph_path)
        .with_context(|| format!("Failed to read snapshot file: {}", ptgraph_path))?;

    // 2. Deserialize
    let container: PtGraphSnapshotContainer = rmp_serde::from_slice(&bytes)
        .with_context(|| format!("Failed to deserialize snapshot: {}", ptgraph_path))?;

    println!("{} Loading snapshot: {} entities, {} edges from {}",
        style("▸").cyan(),
        container.entity_count,
        container.edge_count,
        container.source_directory
    );

    // 3. Create CozoDB mem
    let db = CozoDbStorage::new("mem").await
        .with_context(|| "Failed to create CozoDB mem database")?;

    // 4. Create schemas
    db.create_schema().await
        .with_context(|| "Failed to create CodeGraph schema")?;
    db.create_dependency_edges_schema().await
        .with_context(|| "Failed to create DependencyEdges schema")?;

    // 5. Bulk insert entities
    db.insert_slim_entities_batch_directly(&container.entities).await
        .with_context(|| "Failed to insert slim entities")?;

    // 6. Bulk insert edges
    db.insert_slim_edges_batch_directly(&container.edges).await
        .with_context(|| "Failed to insert slim edges")?;

    println!("{} Snapshot loaded successfully: {} entities, {} edges",
        style("✓").green(),
        container.entity_count,
        container.edge_count
    );

    Ok(db)
}

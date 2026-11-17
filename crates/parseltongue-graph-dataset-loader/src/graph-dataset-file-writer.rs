//! Writes the checked rows into one DuckDB file and reads such a file back (REQ-RUST-067.0,
//! A2 to A5). The only code of this crate that touches DuckDB or the file system.

use crate::GraphDatasetLoadError;
use crate::graph_export_row_checks::{
    GraphDatasetEdgeRecord, GraphDatasetInfoRecord, GraphDatasetNodeRecord, GraphDatasetRowTables,
    build_graph_dataset_tables,
};
use duckdb::{AccessMode, Config, Connection, params};
use std::path::Path;

const DATASET_TABLE_SCHEMA_TEXT: &str = "
CREATE TABLE full_graph_dataset_info (
    run_generation_digest_value VARCHAR NOT NULL,
    node_row_count_value BIGINT NOT NULL,
    edge_row_count_value BIGINT NOT NULL,
    node_file_digest_sha256 VARCHAR NOT NULL,
    edge_file_digest_sha256 VARCHAR NOT NULL
);
CREATE TABLE full_graph_node_rows (
    node_identity_key_text VARCHAR PRIMARY KEY,
    node_qualified_path_text VARCHAR,
    node_input_category VARCHAR,
    node_item_kind_value VARCHAR,
    node_owner_kind_value VARCHAR,
    node_source_relative_path VARCHAR,
    node_function_body_ancestry_flag BOOLEAN,
    node_effective_visibility_kind VARCHAR,
    node_reachable_outside_crate_flag BOOLEAN,
    node_signature_text VARCHAR,
    node_payload_json_text VARCHAR NOT NULL
);
CREATE TABLE full_graph_edge_rows (
    edge_identity_key_text VARCHAR PRIMARY KEY,
    edge_relation_kind VARCHAR NOT NULL,
    source_node_identity_key VARCHAR NOT NULL,
    target_node_identity_key VARCHAR,
    edge_resolution_kind_value VARCHAR NOT NULL,
    edge_unresolved_reason_value VARCHAR,
    site_source_file_path VARCHAR,
    site_byte_start_value BIGINT,
    site_byte_end_value BIGINT,
    edge_dispatch_metadata_text VARCHAR,
    edge_payload_json_text VARCHAR NOT NULL
);";

fn build_store_step_error(step: &'static str) -> impl Fn(duckdb::Error) -> GraphDatasetLoadError {
    move |error| GraphDatasetLoadError::DatasetStoreStepFailed {
        dataset_store_step_name: step,
        error_context_detail_text: error.to_string(),
    }
}

fn build_file_access_error(path: &Path) -> impl Fn(std::io::Error) -> GraphDatasetLoadError + '_ {
    move |error| GraphDatasetLoadError::DatasetFileAccessFailed {
        dataset_file_path_value: path.to_path_buf(),
        error_context_detail_text: error.to_string(),
    }
}

/// Opens a DuckDB file with no extension installed or loaded on demand.
fn open_dataset_store_connection(
    path: &Path,
    mode: AccessMode,
) -> Result<Connection, GraphDatasetLoadError> {
    let failed = build_store_step_error("open");
    let config = Config::default()
        .access_mode(mode)
        .and_then(|config| config.with("autoinstall_known_extensions", "false"))
        .and_then(|config| config.with("autoload_known_extensions", "false"))
        .and_then(|config| config.with("allow_community_extensions", "false"))
        .map_err(&failed)?;
    Connection::open_with_flags(path, config).map_err(&failed)
}

fn insert_dataset_node_rows(
    connection: &Connection,
    rows: &[GraphDatasetNodeRecord],
) -> Result<(), GraphDatasetLoadError> {
    let failed = build_store_step_error("insert node rows");
    let mut statement = connection
        .prepare("INSERT INTO full_graph_node_rows VALUES (?,?,?,?,?,?,?,?,?,?,?)")
        .map_err(&failed)?;
    rows.iter().try_for_each(|row| {
        statement
            .execute(params![
                row.node_identity_key_text,
                row.node_qualified_path_text,
                row.node_input_category,
                row.node_item_kind_value,
                row.node_owner_kind_value,
                row.node_source_relative_path,
                row.node_function_body_ancestry_flag,
                row.node_effective_visibility_kind,
                row.node_reachable_outside_crate_flag,
                row.node_signature_text,
                row.node_payload_json_text,
            ])
            .map(drop)
            .map_err(&failed)
    })
}

fn insert_dataset_edge_rows(
    connection: &Connection,
    rows: &[GraphDatasetEdgeRecord],
) -> Result<(), GraphDatasetLoadError> {
    let failed = build_store_step_error("insert edge rows");
    let mut statement = connection
        .prepare("INSERT INTO full_graph_edge_rows VALUES (?,?,?,?,?,?,?,?,?,?,?)")
        .map_err(&failed)?;
    rows.iter().try_for_each(|row| {
        statement
            .execute(params![
                row.edge_identity_key_text,
                row.edge_relation_kind,
                row.source_node_identity_key,
                row.target_node_identity_key,
                row.edge_resolution_kind_value,
                row.edge_unresolved_reason_value,
                row.site_source_file_path,
                row.site_byte_start_value,
                row.site_byte_end_value,
                row.edge_dispatch_metadata_text,
                row.edge_payload_json_text,
            ])
            .map(drop)
            .map_err(&failed)
    })
}

/// Creates the file at `path` with the three tables in one transaction, then closes it.
fn write_dataset_table_rows(
    path: &Path,
    tables: &GraphDatasetRowTables,
) -> Result<(), GraphDatasetLoadError> {
    let connection = open_dataset_store_connection(path, AccessMode::ReadWrite)?;
    let info = &tables.dataset_info_record_value;
    connection
        .execute_batch(&format!("BEGIN;{DATASET_TABLE_SCHEMA_TEXT}"))
        .map_err(build_store_step_error("create tables"))?;
    connection
        .execute(
            "INSERT INTO full_graph_dataset_info VALUES (?,?,?,?,?)",
            params![
                info.run_generation_digest_value,
                info.node_row_count_value,
                info.edge_row_count_value,
                info.node_file_digest_sha256,
                info.edge_file_digest_sha256,
            ],
        )
        .map_err(build_store_step_error("insert info row"))?;
    insert_dataset_node_rows(&connection, &tables.dataset_node_record_values)?;
    insert_dataset_edge_rows(&connection, &tables.dataset_edge_record_values)?;
    connection
        .execute_batch("COMMIT;CHECKPOINT;")
        .map_err(build_store_step_error("commit"))?;
    connection
        .close()
        .map_err(|(_, error)| build_store_step_error("close")(error))
}

/// Reads the three tables of a dataset file, node and edge rows ordered by key.
pub fn read_graph_dataset_file(
    path: &Path,
) -> Result<GraphDatasetRowTables, GraphDatasetLoadError> {
    let connection = open_dataset_store_connection(path, AccessMode::ReadOnly)?;
    let failed = build_store_step_error("read back");
    let info = connection
        .query_row("SELECT * FROM full_graph_dataset_info", [], |row| {
            Ok(GraphDatasetInfoRecord {
                run_generation_digest_value: row.get(0)?,
                node_row_count_value: row.get(1)?,
                edge_row_count_value: row.get(2)?,
                node_file_digest_sha256: row.get(3)?,
                edge_file_digest_sha256: row.get(4)?,
            })
        })
        .map_err(&failed)?;
    let nodes = connection
        .prepare("SELECT * FROM full_graph_node_rows ORDER BY node_identity_key_text")
        .map_err(&failed)?
        .query_map([], |row| {
            Ok(GraphDatasetNodeRecord {
                node_identity_key_text: row.get(0)?,
                node_qualified_path_text: row.get(1)?,
                node_input_category: row.get(2)?,
                node_item_kind_value: row.get(3)?,
                node_owner_kind_value: row.get(4)?,
                node_source_relative_path: row.get(5)?,
                node_function_body_ancestry_flag: row.get(6)?,
                node_effective_visibility_kind: row.get(7)?,
                node_reachable_outside_crate_flag: row.get(8)?,
                node_signature_text: row.get(9)?,
                node_payload_json_text: row.get(10)?,
            })
        })
        .and_then(Iterator::collect::<Result<Vec<_>, _>>)
        .map_err(&failed)?;
    let edges = connection
        .prepare("SELECT * FROM full_graph_edge_rows ORDER BY edge_identity_key_text")
        .map_err(&failed)?
        .query_map([], |row| {
            Ok(GraphDatasetEdgeRecord {
                edge_identity_key_text: row.get(0)?,
                edge_relation_kind: row.get(1)?,
                source_node_identity_key: row.get(2)?,
                target_node_identity_key: row.get(3)?,
                edge_resolution_kind_value: row.get(4)?,
                edge_unresolved_reason_value: row.get(5)?,
                site_source_file_path: row.get(6)?,
                site_byte_start_value: row.get(7)?,
                site_byte_end_value: row.get(8)?,
                edge_dispatch_metadata_text: row.get(9)?,
                edge_payload_json_text: row.get(10)?,
            })
        })
        .and_then(Iterator::collect::<Result<Vec<_>, _>>)
        .map_err(&failed)?;
    Ok(GraphDatasetRowTables {
        dataset_info_record_value: info,
        dataset_node_record_values: nodes,
        dataset_edge_record_values: edges,
    })
}

fn check_output_path_free(output: &Path) -> Result<(), GraphDatasetLoadError> {
    match output.symlink_metadata() {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(build_file_access_error(output)(error)),
        Ok(_) => Err(GraphDatasetLoadError::DatasetOutputPathExists {
            dataset_output_file_path: output.to_path_buf(),
        }),
    }
}

/// Writes `tables` to a new DuckDB file at `output`. The file is built in a temporary
/// directory beside `output`, read back, and linked to `output` only when the rows read
/// back equal `tables`. An `output` that exists is an error and is left as it was.
pub fn write_graph_dataset_file(
    tables: &GraphDatasetRowTables,
    output: &Path,
) -> Result<(), GraphDatasetLoadError> {
    check_output_path_free(output)?;
    let parent = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let work = tempfile::Builder::new()
        .prefix(".graph-dataset-work-")
        .tempdir_in(parent)
        .map_err(build_file_access_error(parent))?;
    let draft = work.path().join("graph-dataset-draft.duckdb");
    write_dataset_table_rows(&draft, tables)?;
    (read_graph_dataset_file(&draft)? == *tables)
        .then_some(())
        .ok_or(GraphDatasetLoadError::DatasetReadbackRowsDiffer)?;
    std::fs::hard_link(&draft, output).map_err(|error| match error.kind() {
        std::io::ErrorKind::AlreadyExists => GraphDatasetLoadError::DatasetOutputPathExists {
            dataset_output_file_path: output.to_path_buf(),
        },
        _ => build_file_access_error(output)(error),
    })
}

/// Reads the two exported files, checks them (C1 to C7) and writes the DuckDB file. Returns
/// the row of `full_graph_dataset_info`. Nothing is created when a check fails.
pub fn load_graph_export_files(
    node_export: &Path,
    edge_export: &Path,
    output: &Path,
) -> Result<GraphDatasetInfoRecord, GraphDatasetLoadError> {
    check_output_path_free(output)?;
    let node_text =
        std::fs::read_to_string(node_export).map_err(build_file_access_error(node_export))?;
    let edge_text =
        std::fs::read_to_string(edge_export).map_err(build_file_access_error(edge_export))?;
    let tables = build_graph_dataset_tables(&node_text, &edge_text)?;
    write_graph_dataset_file(&tables, output)?;
    Ok(tables.dataset_info_record_value)
}

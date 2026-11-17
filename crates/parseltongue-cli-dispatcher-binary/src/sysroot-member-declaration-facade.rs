//! REQ-RUST-054.0 D6 to D9: reads the `core` source files of the sysroot and stores the
//! methods they declare. The only place that touches the file system for this table.
use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StageTableRowRecord, StoreBackendFailureError,
};
use parseltongue_syntax_span_extractor::extract_member_declaration_rows;
use parseltongue_table_store_traits::StageTableStorePort;
use std::path::{Path, PathBuf};

#[cfg(test)]
#[path = "sysroot-member-declaration-tests.rs"]
mod sysroot_member_declaration_tests;

pub(crate) const SYSROOT_MEMBER_TABLE_NAME: &str = "sysroot-member-declaration-records";
pub(crate) const SYSROOT_MEMBER_STAGE_NAME: &str = "sysroot-member-declaration-stage";
const SCANNED_CRATE_SOURCE_PATH: &str = "core/src";

fn build_sysroot_read_error(
    path: &Path,
    error: impl std::fmt::Display,
) -> StoreBackendFailureError {
    super::build_pipeline_stage_error(format!("sysroot source {}: {error}", path.display()))
}

/// Every `.rs` file under `directory`, in no particular order.
fn list_rust_source_files(directory: &Path) -> Result<Vec<PathBuf>, StoreBackendFailureError> {
    std::fs::read_dir(directory)
        .map_err(|error| build_sysroot_read_error(directory, error))?
        .try_fold(Vec::new(), |mut files, entry| {
            let path = entry
                .map_err(|error| build_sysroot_read_error(directory, error))?
                .path();
            if path.is_dir() {
                files.extend(list_rust_source_files(&path)?);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
            Ok(files)
        })
}

fn read_library_relative_path(library_directory: &Path, file: &Path) -> Option<String> {
    file.strip_prefix(library_directory).ok().map(|relative| {
        relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/")
    })
}

/// REQ-RUST-054.0 D6, D9: the rows of every `.rs` file under `<library>/core/src`, files
/// in path order.
pub(crate) fn collect_sysroot_member_rows(
    library_directory: &Path,
) -> Result<Vec<StageTableRowRecord>, StoreBackendFailureError> {
    let mut files = list_rust_source_files(&library_directory.join(SCANNED_CRATE_SOURCE_PATH))?
        .into_iter()
        .map(|file| {
            read_library_relative_path(library_directory, &file)
                .map(|relative| (relative, file.clone()))
                .ok_or_else(|| build_sysroot_read_error(&file, "outside the library directory"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    files.sort();
    files
        .iter()
        .map(|(relative, file)| {
            let text = std::fs::read_to_string(file)
                .map_err(|error| build_sysroot_read_error(Path::new(relative), error))?;
            Ok(extract_member_declaration_rows(relative, &text)
                .into_iter()
                .map(|row| StageTableRowRecord {
                    stage_owned_table_name: SYSROOT_MEMBER_TABLE_NAME.into(),
                    stage_row_record_key: build_declaration_row_key(relative, &row),
                    stage_row_payload_text: row.to_string(),
                })
                .collect::<Vec<_>>())
        })
        .collect::<Result<Vec<_>, StoreBackendFailureError>>()
        .map(|rows| rows.into_iter().flatten().collect())
}

/// REQ-RUST-054.0 D6: stores the rows for the sysroot named in the workspace row.
pub(crate) fn persist_sysroot_member_rows(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    cancel: &RunCancelFlagValue,
) -> Result<usize, StoreBackendFailureError> {
    let config = super::read_stage_json_records(store, scope, "workspace-config-state-records")?;
    let sysroot = config
        .first()
        .ok_or_else(|| super::build_pipeline_stage_error("workspace configuration missing"))
        .and_then(|row| {
            super::read_required_json_string(row, "workspace_sysroot_directory_path")
        })?;
    let rows =
        collect_sysroot_member_rows(&Path::new(sysroot).join("lib/rustlib/src/rust/library"))?;
    super::write_pipeline_stage_unit(
        store,
        scope,
        SYSROOT_MEMBER_STAGE_NAME,
        SCANNED_CRATE_SOURCE_PATH,
        &rows,
        cancel,
    )?;
    Ok(rows.len())
}

/// The key of a declaration row: its file and the offset of its name. A file scope row
/// (REQ-RUST-062.0 C9) shares that offset with the declaration row of the same name, so
/// its key also says its kind.
fn build_declaration_row_key(relative: &str, row: &serde_json::Value) -> String {
    let offset = &row["member_name_start_offset"];
    match row["declaration_owner_kind_value"].as_str() {
        Some(kind @ "file_scope_name_binding") => format!("{relative}#{offset}#{kind}"),
        _ => format!("{relative}#{offset}"),
    }
}

//! REQ-RUST-054.0 D6 to D9: reads the `core` source files of the sysroot and stores the
//! methods they declare. The only place that touches the file system for this table.
use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StageTableRowRecord, StoreBackendFailureError,
};
use parseltongue_syntax_span_extractor::extract_member_declaration_rows;
use parseltongue_table_store_traits::StageTableStorePort;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[cfg(test)]
#[path = "sysroot-member-declaration-tests.rs"]
mod sysroot_member_declaration_tests;

pub(crate) const SYSROOT_MEMBER_TABLE_NAME: &str = "sysroot-member-declaration-records";
pub(crate) const SYSROOT_MEMBER_STAGE_NAME: &str = "sysroot-member-declaration-stage";
pub(crate) const SYSROOT_SOURCE_RECORD_NAME: &str = "workspace_sysroot_source_record";
const SCANNED_CRATE_SOURCE_PATH: &str = "core/src";

/// The directory of a sysroot that holds the source of `core`.
pub(crate) fn build_sysroot_library_path(sysroot: &str) -> PathBuf {
    Path::new(sysroot).join("lib/rustlib/src/rust/library")
}

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

/// REQ-RUST-065.0 D1, D5: every `.rs` file under `<library>/core/src` as its path relative
/// to the library directory and its bytes, in byte order of that path.
fn read_sysroot_source_files(
    library_directory: &Path,
) -> Result<Vec<(String, Vec<u8>)>, StoreBackendFailureError> {
    let mut files = list_rust_source_files(&library_directory.join(SCANNED_CRATE_SOURCE_PATH))?
        .into_iter()
        .map(|file| {
            let relative = read_library_relative_path(library_directory, &file)
                .ok_or_else(|| build_sysroot_read_error(&file, "outside the library directory"))?;
            let bytes = std::fs::read(&file)
                .map_err(|error| build_sysroot_read_error(Path::new(&relative), error))?;
            Ok((relative, bytes))
        })
        .collect::<Result<Vec<_>, StoreBackendFailureError>>()?;
    files.sort();
    Ok(files)
}

fn hash_bytes_sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// REQ-RUST-065.0 D1: which bytes of `core` a run read. The digest is the SHA-256 of one
/// `sha256sum` line per file, the value the SOP's S1 command prints.
fn build_source_listing_record(files: &[(String, Vec<u8>)]) -> Value {
    let listing = files
        .iter()
        .map(|(relative, bytes)| format!("{}  {relative}\n", hash_bytes_sha256_hex(bytes)))
        .collect::<String>();
    json!({
        "sysroot_source_directory_path": SCANNED_CRATE_SOURCE_PATH,
        "sysroot_source_file_count": files.len(),
        "sysroot_source_byte_count": files.iter().map(|(_, bytes)| bytes.len()).sum::<usize>(),
        "sysroot_listing_digest_sha256": hash_bytes_sha256_hex(listing.as_bytes()),
    })
}

/// REQ-RUST-065.0 D1: the record the workspace row stores before the run id is made.
pub(crate) fn read_sysroot_source_record(
    library_directory: &Path,
) -> Result<Value, StoreBackendFailureError> {
    read_sysroot_source_files(library_directory).map(|files| build_source_listing_record(&files))
}

/// REQ-RUST-065.0 D4: the bytes that were parsed are the bytes the run id covers.
pub(crate) fn check_sysroot_source_record(
    config: &Value,
    record: &Value,
) -> Result<(), StoreBackendFailureError> {
    (config.get(SYSROOT_SOURCE_RECORD_NAME) == Some(record))
        .then_some(())
        .ok_or_else(|| {
            build_sysroot_read_error(
                Path::new(SCANNED_CRATE_SOURCE_PATH),
                "the files differ from the record the run id covers",
            )
        })
}

/// REQ-RUST-054.0 D6, D9 and REQ-RUST-065.0 D4: the rows of every `.rs` file under
/// `<library>/core/src`, files in path order, with the record of the bytes parsed.
pub(crate) fn collect_sysroot_member_rows(
    library_directory: &Path,
) -> Result<(Value, Vec<StageTableRowRecord>), StoreBackendFailureError> {
    let files = read_sysroot_source_files(library_directory)?;
    files
        .iter()
        .map(|(relative, bytes)| {
            let text = std::str::from_utf8(bytes)
                .map_err(|error| build_sysroot_read_error(Path::new(relative), error))?;
            Ok(extract_member_declaration_rows(relative, text)
                .into_iter()
                .map(|row| StageTableRowRecord {
                    stage_owned_table_name: SYSROOT_MEMBER_TABLE_NAME.into(),
                    stage_row_record_key: build_declaration_row_key(relative, &row),
                    stage_row_payload_text: row.to_string(),
                })
                .collect::<Vec<_>>())
        })
        .collect::<Result<Vec<_>, StoreBackendFailureError>>()
        .map(|rows| {
            (
                build_source_listing_record(&files),
                rows.into_iter().flatten().collect(),
            )
        })
}

/// REQ-RUST-054.0 D6: stores the rows for the sysroot named in the workspace row.
pub(crate) fn persist_sysroot_member_rows(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    cancel: &RunCancelFlagValue,
) -> Result<usize, StoreBackendFailureError> {
    let config = super::read_stage_json_records(store, scope, "workspace-config-state-records")?;
    let config = config
        .first()
        .ok_or_else(|| super::build_pipeline_stage_error("workspace configuration missing"))?;
    let sysroot = super::read_required_json_string(config, "workspace_sysroot_directory_path")?;
    let (record, rows) = collect_sysroot_member_rows(&build_sysroot_library_path(sysroot))?;
    check_sysroot_source_record(config, &record)?;
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
/// (REQ-RUST-062.0 C9) shares that offset with the declaration row of the same name, and
/// a macro call in an impl (REQ-RUST-063.0 M1) with the row of the names its file's
/// calls of that macro write, so the key of either also says its kind.
fn build_declaration_row_key(relative: &str, row: &Value) -> String {
    let offset = &row["member_name_start_offset"];
    match row["declaration_owner_kind_value"].as_str() {
        Some(kind @ ("file_scope_name_binding" | "impl_macro_call")) => {
            format!("{relative}#{offset}#{kind}")
        }
        _ => format!("{relative}#{offset}"),
    }
}

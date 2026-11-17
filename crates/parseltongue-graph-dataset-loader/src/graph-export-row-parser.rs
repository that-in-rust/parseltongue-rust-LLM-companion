//! Reads the text of one exported table into rows (REQ-RUST-067.0, C1, C2 and C7).

use crate::{GraphDatasetLoadError, GraphExportFileKind};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// One row line of an exported table.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphExportRowRecord {
    pub export_line_number_value: usize,
    pub export_row_identity_text: String,
    pub export_row_payload_value: Value,
}

/// One exported table: its header facts and its rows, in file order.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphExportFileRows {
    pub export_file_kind_value: GraphExportFileKind,
    pub run_generation_digest_value: String,
    pub export_file_digest_sha256: String,
    pub export_row_record_values: Vec<GraphExportRowRecord>,
}

/// Compact JSON text with object keys sorted, so equal values give equal text.
pub fn write_canonical_json_text(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut entries = map.iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(right.0));
            let body = entries
                .iter()
                .map(|(key, item)| {
                    format!(
                        "{}:{}",
                        Value::String((*key).clone()),
                        write_canonical_json_text(item)
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{body}}}")
        }
        Value::Array(items) => format!(
            "[{}]",
            items
                .iter()
                .map(write_canonical_json_text)
                .collect::<Vec<_>>()
                .join(",")
        ),
        scalar => scalar.to_string(),
    }
}

fn hash_text_sha256_hex(text: &str) -> String {
    Sha256::digest(text.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn parse_export_line_value(
    kind: GraphExportFileKind,
    line_number: usize,
    line: &str,
) -> Result<Value, GraphDatasetLoadError> {
    serde_json::from_str(line).map_err(|error| GraphDatasetLoadError::ExportLineNotJson {
        export_file_kind_value: kind,
        export_line_number_value: line_number,
        error_context_detail_text: error.to_string(),
    })
}

fn read_export_header_digest(
    kind: GraphExportFileKind,
    header: &Value,
) -> Result<String, GraphDatasetLoadError> {
    let missing = || GraphDatasetLoadError::ExportHeaderMissing {
        export_file_kind_value: kind,
    };
    (header["export_record_kind_value"] == "header")
        .then_some(())
        .ok_or_else(missing)?;
    let table = header["export_owned_table_name"]
        .as_str()
        .ok_or_else(missing)?;
    (table == kind.read_export_table_name())
        .then_some(())
        .ok_or_else(|| GraphDatasetLoadError::ExportTableNameMismatch {
            export_file_kind_value: kind,
            export_owned_table_name: table.to_owned(),
        })?;
    header["export_scope_identity_record"]["run_generation_digest_value"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(missing)
}

fn build_export_row_record(
    kind: GraphExportFileKind,
    generation: &str,
    line_number: usize,
    value: Value,
) -> Result<GraphExportRowRecord, GraphDatasetLoadError> {
    let invalid = || GraphDatasetLoadError::ExportRowShapeInvalid {
        export_file_kind_value: kind,
        export_line_number_value: line_number,
    };
    (value["export_record_kind_value"] == "row")
        .then_some(())
        .ok_or_else(invalid)?;
    let identity = value["export_row_identity_value"]
        .as_str()
        .ok_or_else(invalid)?
        .to_owned();
    let payload = value
        .get("export_row_payload_value")
        .filter(|payload| payload.is_object())
        .cloned()
        .ok_or_else(invalid)?;
    let found = payload["run_generation_digest_full"].as_str().ok_or(
        GraphDatasetLoadError::PayloadFieldShapeInvalid {
            export_file_kind_value: kind,
            export_line_number_value: line_number,
            payload_field_name_text: "run_generation_digest_full",
        },
    )?;
    (found == generation).then_some(()).ok_or_else(|| {
        GraphDatasetLoadError::RunGenerationMismatch {
            export_file_kind_value: kind,
            export_line_number_value: line_number,
            run_generation_digest_value: found.to_owned(),
        }
    })?;
    Ok(GraphExportRowRecord {
        export_line_number_value: line_number,
        export_row_identity_text: identity,
        export_row_payload_value: payload,
    })
}

/// Parses one exported table. The first line is the header, every other line is a row of the
/// header's run generation, and there is at least one row.
pub fn parse_graph_export_text(
    kind: GraphExportFileKind,
    text: &str,
) -> Result<GraphExportFileRows, GraphDatasetLoadError> {
    let mut lines = text
        .lines()
        .enumerate()
        .map(|(index, line)| (index + 1, line))
        .filter(|(_, line)| !line.trim().is_empty());
    let (header_line, header_text) =
        lines
            .next()
            .ok_or(GraphDatasetLoadError::ExportHeaderMissing {
                export_file_kind_value: kind,
            })?;
    let generation = read_export_header_digest(
        kind,
        &parse_export_line_value(kind, header_line, header_text)?,
    )?;
    let rows = lines
        .map(|(line_number, line)| {
            parse_export_line_value(kind, line_number, line)
                .and_then(|value| build_export_row_record(kind, &generation, line_number, value))
        })
        .collect::<Result<Vec<_>, _>>()?;
    (!rows.is_empty())
        .then_some(())
        .ok_or(GraphDatasetLoadError::ExportFileEmpty {
            export_file_kind_value: kind,
        })?;
    Ok(GraphExportFileRows {
        export_file_kind_value: kind,
        run_generation_digest_value: generation,
        export_file_digest_sha256: hash_text_sha256_hex(text),
        export_row_record_values: rows,
    })
}

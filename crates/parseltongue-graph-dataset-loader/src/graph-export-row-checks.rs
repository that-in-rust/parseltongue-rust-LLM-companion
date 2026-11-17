//! Checks the rows of the two exported tables against each other and builds the rows of the
//! three DuckDB tables (REQ-RUST-067.0, C2 to C6). Pure: no file and no database.

use crate::graph_export_row_parser::{
    GraphExportFileRows, GraphExportRowRecord, parse_graph_export_text, write_canonical_json_text,
};
use crate::{GraphDatasetLoadError, GraphExportFileKind};
use serde_json::Value;
use std::collections::BTreeSet;

/// The one row of `full_graph_dataset_info`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphDatasetInfoRecord {
    pub run_generation_digest_value: String,
    pub node_row_count_value: i64,
    pub edge_row_count_value: i64,
    pub node_file_digest_sha256: String,
    pub edge_file_digest_sha256: String,
}

/// One row of `full_graph_node_rows`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GraphDatasetNodeRecord {
    pub node_identity_key_text: String,
    pub node_qualified_path_text: Option<String>,
    pub node_input_category: Option<String>,
    pub node_item_kind_value: Option<String>,
    pub node_owner_kind_value: Option<String>,
    pub node_source_relative_path: Option<String>,
    pub node_function_body_ancestry_flag: Option<bool>,
    pub node_effective_visibility_kind: Option<String>,
    pub node_reachable_outside_crate_flag: Option<bool>,
    pub node_signature_text: Option<String>,
    pub node_payload_json_text: String,
}

/// One row of `full_graph_edge_rows`: one source site.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GraphDatasetEdgeRecord {
    pub edge_identity_key_text: String,
    pub edge_relation_kind: String,
    pub source_node_identity_key: String,
    pub target_node_identity_key: Option<String>,
    pub edge_resolution_kind_value: String,
    pub edge_unresolved_reason_value: Option<String>,
    pub site_source_file_path: Option<String>,
    pub site_byte_start_value: Option<i64>,
    pub site_byte_end_value: Option<i64>,
    pub edge_dispatch_metadata_text: Option<String>,
    pub edge_payload_json_text: String,
}

/// The rows of the three tables, node and edge rows ordered by key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphDatasetRowTables {
    pub dataset_info_record_value: GraphDatasetInfoRecord,
    pub dataset_node_record_values: Vec<GraphDatasetNodeRecord>,
    pub dataset_edge_record_values: Vec<GraphDatasetEdgeRecord>,
}

/// A string field of a payload, or a scalar written as its JSON text; `None` for null or absent.
fn read_optional_text_field(payload: &Value, name: &str) -> Option<String> {
    payload.get(name).and_then(|value| match value {
        Value::Null => None,
        Value::String(text) => Some(text.clone()),
        other => Some(write_canonical_json_text(other)),
    })
}

fn build_field_shape_error(
    kind: GraphExportFileKind,
    row: &GraphExportRowRecord,
    name: &'static str,
) -> GraphDatasetLoadError {
    GraphDatasetLoadError::PayloadFieldShapeInvalid {
        export_file_kind_value: kind,
        export_line_number_value: row.export_line_number_value,
        payload_field_name_text: name,
    }
}

fn read_required_text_field(
    kind: GraphExportFileKind,
    row: &GraphExportRowRecord,
    owner: &Value,
    name: &'static str,
) -> Result<String, GraphDatasetLoadError> {
    owner[name]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| build_field_shape_error(kind, row, name))
}

fn read_required_object_field<'a>(
    kind: GraphExportFileKind,
    row: &'a GraphExportRowRecord,
    name: &'static str,
) -> Result<&'a Value, GraphDatasetLoadError> {
    row.export_row_payload_value
        .get(name)
        .filter(|value| value.is_object())
        .ok_or_else(|| build_field_shape_error(kind, row, name))
}

/// C3: the identity text of a node row is JSON equal to its `node_target_identity`.
fn build_node_dataset_record(
    row: &GraphExportRowRecord,
) -> Result<GraphDatasetNodeRecord, GraphDatasetLoadError> {
    let payload = &row.export_row_payload_value;
    let identity =
        read_required_object_field(GraphExportFileKind::NodeRows, row, "node_target_identity")?;
    (serde_json::from_str::<Value>(&row.export_row_identity_text)
        .ok()
        .as_ref()
        == Some(identity))
    .then_some(())
    .ok_or_else(|| GraphDatasetLoadError::NodeIdentityTextMismatch {
        export_line_number_value: row.export_line_number_value,
        export_row_identity_text: row.export_row_identity_text.clone(),
    })?;
    Ok(GraphDatasetNodeRecord {
        node_identity_key_text: write_canonical_json_text(identity),
        node_qualified_path_text: read_optional_text_field(payload, "node_qualified_path_text"),
        node_input_category: read_optional_text_field(payload, "node_input_category"),
        node_item_kind_value: read_optional_text_field(payload, "node_item_kind_value"),
        node_owner_kind_value: read_optional_text_field(payload, "node_owner_kind_value"),
        node_source_relative_path: read_optional_text_field(payload, "node_source_relative_path"),
        node_function_body_ancestry_flag: payload["node_function_body_ancestry_flag"].as_bool(),
        node_effective_visibility_kind: read_optional_text_field(
            payload,
            "node_effective_visibility_kind",
        ),
        node_reachable_outside_crate_flag: payload["node_reachable_outside_crate_flag"].as_bool(),
        node_signature_text: read_optional_text_field(payload, "node_signature_text"),
        node_payload_json_text: write_canonical_json_text(payload),
    })
}

/// C6: one target that is a node, or no target and a reason.
fn read_edge_target_columns(
    row: &GraphExportRowRecord,
    resolution: &Value,
    nodes: &BTreeSet<&str>,
) -> Result<(Option<String>, Option<String>), GraphDatasetLoadError> {
    match (resolution.get("target"), resolution.get("reason")) {
        (Some(target), None) if target.is_object() => {
            let key = write_canonical_json_text(target);
            if nodes.contains(key.as_str()) {
                Ok((Some(key), None))
            } else {
                Err(GraphDatasetLoadError::EdgeTargetNotNode {
                    export_line_number_value: row.export_line_number_value,
                    node_identity_key_text: key,
                })
            }
        }
        (None, Some(Value::String(reason))) => Ok((None, Some(reason.clone()))),
        _ => Err(GraphDatasetLoadError::EdgeResolutionShapeInvalid {
            export_line_number_value: row.export_line_number_value,
        }),
    }
}

/// C5 and C6 for one edge row, then its table row.
fn build_edge_dataset_record(
    row: &GraphExportRowRecord,
    nodes: &BTreeSet<&str>,
) -> Result<GraphDatasetEdgeRecord, GraphDatasetLoadError> {
    let kind = GraphExportFileKind::EdgeRows;
    let payload = &row.export_row_payload_value;
    let source = write_canonical_json_text(read_required_object_field(
        kind,
        row,
        "edge_source_target_identity",
    )?);
    nodes
        .contains(source.as_str())
        .then_some(())
        .ok_or_else(|| GraphDatasetLoadError::EdgeSourceNotNode {
            export_line_number_value: row.export_line_number_value,
            node_identity_key_text: source.clone(),
        })?;
    let resolution = read_required_object_field(kind, row, "edge_resolution")?;
    let (target, reason) = read_edge_target_columns(row, resolution, nodes)?;
    let site = &payload["edge_evidence_identity"]["site_occurrence_identity"];
    Ok(GraphDatasetEdgeRecord {
        edge_identity_key_text: row.export_row_identity_text.clone(),
        edge_relation_kind: read_required_text_field(kind, row, payload, "edge_relation_kind")?,
        source_node_identity_key: source,
        target_node_identity_key: target,
        edge_resolution_kind_value: read_required_text_field(kind, row, resolution, "kind")?,
        edge_unresolved_reason_value: reason,
        site_source_file_path: read_optional_text_field(site, "source_file_path"),
        site_byte_start_value: site["byte_range"][0].as_i64(),
        site_byte_end_value: site["byte_range"][1].as_i64(),
        edge_dispatch_metadata_text: read_optional_text_field(
            payload,
            "edge_dispatch_metadata_text",
        ),
        edge_payload_json_text: write_canonical_json_text(payload),
    })
}

/// C4: the first row, in file order, whose key an earlier row has.
fn check_row_keys_distinct<'a>(
    kind: GraphExportFileKind,
    keys: impl Iterator<Item = (usize, &'a str)>,
) -> Result<(), GraphDatasetLoadError> {
    let mut ordered = keys.collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.1.cmp(right.1).then(left.0.cmp(&right.0)));
    ordered
        .windows(2)
        .filter(|pair| pair[0].1 == pair[1].1)
        .map(|pair| pair[1])
        .min()
        .map_or(Ok(()), |(line, key)| {
            Err(GraphDatasetLoadError::RowIdentityDuplicated {
                export_file_kind_value: kind,
                export_line_number_value: line,
                export_row_identity_text: key.to_owned(),
            })
        })
}

/// A row count as DuckDB stores it. A `Vec` holds at most `isize::MAX` rows, so the
/// fallback is never taken.
fn count_dataset_table_rows(count: usize) -> i64 {
    i64::try_from(count).unwrap_or(i64::MAX)
}

/// Checks C2 to C6 over the parsed rows of both files and returns the rows of the three
/// tables, node and edge rows ordered by key.
pub fn check_graph_export_rows(
    nodes: &GraphExportFileRows,
    edges: &GraphExportFileRows,
) -> Result<GraphDatasetRowTables, GraphDatasetLoadError> {
    (nodes.run_generation_digest_value == edges.run_generation_digest_value)
        .then_some(())
        .ok_or_else(|| GraphDatasetLoadError::RunGenerationMismatch {
            export_file_kind_value: GraphExportFileKind::EdgeRows,
            export_line_number_value: 1,
            run_generation_digest_value: edges.run_generation_digest_value.clone(),
        })?;
    let mut node_records = nodes
        .export_row_record_values
        .iter()
        .map(build_node_dataset_record)
        .collect::<Result<Vec<_>, _>>()?;
    check_row_keys_distinct(
        GraphExportFileKind::NodeRows,
        nodes
            .export_row_record_values
            .iter()
            .zip(&node_records)
            .map(|(row, record)| {
                (
                    row.export_line_number_value,
                    record.node_identity_key_text.as_str(),
                )
            }),
    )?;
    check_row_keys_distinct(
        GraphExportFileKind::EdgeRows,
        edges.export_row_record_values.iter().map(|row| {
            (
                row.export_line_number_value,
                row.export_row_identity_text.as_str(),
            )
        }),
    )?;
    let node_keys = node_records
        .iter()
        .map(|record| record.node_identity_key_text.as_str())
        .collect::<BTreeSet<_>>();
    let mut edge_records = edges
        .export_row_record_values
        .iter()
        .map(|row| build_edge_dataset_record(row, &node_keys))
        .collect::<Result<Vec<_>, _>>()?;
    edge_records.sort();
    node_records.sort();
    Ok(GraphDatasetRowTables {
        dataset_info_record_value: GraphDatasetInfoRecord {
            run_generation_digest_value: nodes.run_generation_digest_value.clone(),
            node_row_count_value: count_dataset_table_rows(node_records.len()),
            edge_row_count_value: count_dataset_table_rows(edge_records.len()),
            node_file_digest_sha256: nodes.export_file_digest_sha256.clone(),
            edge_file_digest_sha256: edges.export_file_digest_sha256.clone(),
        },
        dataset_node_record_values: node_records,
        dataset_edge_record_values: edge_records,
    })
}

/// Parses and checks the text of the two exported files (C1 to C7).
pub fn build_graph_dataset_tables(
    node_export_text: &str,
    edge_export_text: &str,
) -> Result<GraphDatasetRowTables, GraphDatasetLoadError> {
    let nodes = parse_graph_export_text(GraphExportFileKind::NodeRows, node_export_text)?;
    let edges = parse_graph_export_text(GraphExportFileKind::EdgeRows, edge_export_text)?;
    check_graph_export_rows(&nodes, &edges)
}

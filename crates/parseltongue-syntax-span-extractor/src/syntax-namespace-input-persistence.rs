use super::{
    syntax_namespace_input_extraction::{CrateSyntaxProjectionState, MacroProjectionRowOffsets},
    syntax_namespace_record_codec::*,
};
use parseltongue_core_row_types::*;
use parseltongue_table_store_traits::StageTableStorePort;
use serde_json::{Value, json};

const INPUT_STAGE: &str = "syntax-namespace-input-records";
const INPUT_BATCH_BYTES: usize = 4 * 1024 * 1024;

fn read_namespace_input_values(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    table: &str,
) -> Result<Vec<Value>, StoreBackendFailureError> {
    let mut values = Vec::new();
    store.visit_stage_table_rows(scope, table, &mut |row| {
        values.push(
            serde_json::from_str(&row.stage_row_payload_text)
                .map_err(build_name_projection_error)?,
        );
        Ok(())
    })?;
    Ok(values)
}

fn build_namespace_input_record(table: &str, value: Value) -> StageTableRowRecord {
    let text = value.to_string();
    StageTableRowRecord {
        stage_owned_table_name: table.into(),
        stage_row_record_key: blake3::hash(text.as_bytes()).to_hex().to_string(),
        stage_row_payload_text: text,
    }
}

#[derive(Default)]
struct NamespaceInputPendingRows {
    records: Vec<StageTableRowRecord>,
    payload_bytes: usize,
    written_records: usize,
    chunk_sequence: usize,
}

fn flush_namespace_input_rows(
    mut pending: NamespaceInputPendingRows,
    prefix: &str,
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    cancel: &RunCancelFlagValue,
) -> Result<NamespaceInputPendingRows, StoreBackendFailureError> {
    if pending.records.is_empty() {
        return Ok(pending);
    }
    store.write_stage_table_unit(
        &StageUnitWriteRequest {
            stage_run_scope_value: scope,
            stage_owner_name_value: INPUT_STAGE,
            stage_unit_work_key: &format!("{prefix}/pending/{:08}", pending.chunk_sequence),
            stage_input_stage_names: &["syntax-spans"],
            stage_unit_output_rows: &pending.records,
        },
        cancel,
    )?;
    pending.records.clear();
    pending.chunk_sequence += 1;
    pending.payload_bytes = 0;
    Ok(pending)
}

pub(super) fn namespace_input_row_records<'a>(
    projection: &'a CrateSyntaxProjectionState<'_>,
    offsets: &MacroProjectionRowOffsets,
) -> impl Iterator<Item = Result<StageTableRowRecord, StoreBackendFailureError>> + 'a {
    let input = &projection.projection_resolution_input_rows;
    let scopes = input
        .name_module_scope_rows
        .iter()
        .skip(offsets.module_scope_row_count)
        .map(move |row| {
            let mut value = encode_scope_record_value(row);
            value["scope_cargo_unit_key"] =
                projection.projection_unit_record_value["unit_compilation_identity_key"].clone();
            Ok(build_namespace_input_record(
                "syntax-namespace-input-scopes",
                value,
            ))
        });
    let items = input
        .name_symbol_binding_rows
        .iter()
        .skip(offsets.symbol_binding_row_count)
        .map(move |row| {
            let metadata = projection
                .projection_target_metadata_rows
                .get(&row.binding_target_identity_key)
                .ok_or_else(|| build_name_projection_error("declared target metadata absent"))?;
            Ok(build_namespace_input_record(
                "syntax-namespace-input-items",
                encode_binding_record_value(row, metadata.clone()),
            ))
        });
    let imports = input.name_import_request_rows.iter().skip(offsets.import_request_row_count).map(|row| Ok(
        build_namespace_input_record("syntax-namespace-input-imports", json!({
            "import_site_occurrence_key":encode_occurrence_identity_value(&row.import_site_occurrence_key),
            "import_lexical_scope_key":encode_resolution_scope_value(&row.import_lexical_scope_key),
            "import_path_segment_rows":encode_path_segment_values(&row.import_path_segment_rows),
            "import_binding_alias_text":row.import_binding_alias_text,
            "import_glob_expansion_flag":row.import_glob_expansion_flag,
            "import_visibility_scope_kind":encode_visibility_scope_value(&row.import_visibility_scope_kind),
            "import_resolution_source_kind":format!("{:?}",row.import_resolution_source_kind),
        })),
    ));
    let paths = input.name_signature_path_rows.iter().skip(offsets.signature_path_row_count).map(move |row| {
        let metadata = projection.projection_path_metadata_rows.get(&row.path_site_occurrence_key)
            .ok_or_else(|| build_name_projection_error("syntax path metadata absent"))?;
        Ok(build_namespace_input_record("syntax-namespace-input-paths", json!({
            "path_site_occurrence_key":encode_occurrence_identity_value(&row.path_site_occurrence_key),
            "path_lexical_scope_key":encode_resolution_scope_value(&row.path_lexical_scope_key),
            "path_source_segment_rows":encode_path_segment_values(&row.path_source_segment_rows),
            "path_requested_namespace_kind":format!("{:?}",row.path_requested_namespace_kind),
            "path_resolution_source_kind":format!("{:?}",row.path_resolution_source_kind),
            "path_unavailable_reason_kind":row.path_unavailable_reason_kind.map(encode_unresolved_reason_value),
            "path_source_metadata_value":metadata,
        })))
    });
    let macros = input.name_macro_invocation_rows.iter().skip(offsets.macro_invocation_row_count).map(|row| Ok(
        build_namespace_input_record("syntax-namespace-input-macros", json!({
            "invocation_site_occurrence_key":encode_occurrence_identity_value(&row.invocation_site_occurrence_key),
            "invocation_lexical_scope_key":encode_resolution_scope_value(&row.invocation_lexical_scope_key),
            "invocation_macro_path_segments":encode_path_segment_values(&row.invocation_macro_path_segments),
            "invocation_argument_source_text":row.invocation_argument_source_text,
        })),
    ));
    let externs = projection
        .projection_extern_declaration_rows
        .iter()
        .skip(offsets.extern_declaration_row_count)
        .map(|row| {
            Ok(build_namespace_input_record(
                "syntax-namespace-input-externs",
                row.clone(),
            ))
        });
    let lang = projection
        .projection_lang_definition_rows
        .iter()
        .skip(offsets.lang_definition_row_count)
        .cloned()
        .map(Ok);
    let legacy = projection
        .projection_legacy_source_event_rows
        .iter()
        .skip(offsets.legacy_source_event_row_count)
        .map(|row| {
            Ok(build_namespace_input_record(
                "legacy-macro-source-events",
                row.clone(),
            ))
        });
    scopes
        .chain(items)
        .chain(imports)
        .chain(paths)
        .chain(macros)
        .chain(externs)
        .chain(lang)
        .chain(legacy)
}

fn persist_projection_input_rows(
    projection: &mut CrateSyntaxProjectionState<'_>,
    scope: &StageRunScopeValue,
    store: &dyn StageTableStorePort,
    cancel: &RunCancelFlagValue,
) -> Result<usize, StoreBackendFailureError> {
    let legacy = projection.read_legacy_lookup_updates()?;
    let input = &projection.projection_resolution_input_rows;
    let prefix = format!(
        "crate/{}",
        blake3::hash(
            &encode_compilation_unit_value(&input.unit_compilation_identity_key)
                .to_string()
                .into_bytes(),
        )
        .to_hex()
    );
    let pending = namespace_input_row_records(projection, &MacroProjectionRowOffsets::default())
        .chain(legacy.into_iter().map(Ok))
        .try_fold(
            NamespaceInputPendingRows::default(),
            |mut pending, record| {
                let record = record?;
                if cancel.check_stage_work_cancelled() {
                    return Err(build_name_projection_error(
                        "namespace input extraction cancelled",
                    ));
                }
                if pending.payload_bytes + record.stage_row_payload_text.len() > INPUT_BATCH_BYTES {
                    pending = flush_namespace_input_rows(pending, &prefix, store, scope, cancel)?;
                }
                pending.payload_bytes += record.stage_row_payload_text.len();
                pending.written_records += 1;
                pending.records.push(record);
                Ok(pending)
            },
        )?;
    let pending = flush_namespace_input_rows(pending, &prefix, store, scope, cancel)?;
    store.write_stage_table_unit(
        &StageUnitWriteRequest {
            stage_run_scope_value: scope,
            stage_owner_name_value: INPUT_STAGE,
            stage_unit_work_key: &prefix,
            stage_input_stage_names: &["syntax-spans"],
            stage_unit_output_rows: &[],
        },
        cancel,
    )?;
    Ok(pending.written_records)
}

pub fn persist_namespace_syntax_inputs(
    scope: &StageRunScopeValue,
    store: &dyn StageTableStorePort,
    cancel: &RunCancelFlagValue,
) -> Result<usize, StoreBackendFailureError> {
    let workspace = read_namespace_input_values(store, scope, "workspace-config-state-records")?
        .into_iter()
        .next()
        .ok_or_else(|| build_name_projection_error("workspace configuration absent"))?;
    let count = read_namespace_input_values(store, scope, "crate-dependency-graph-table")?
        .into_iter()
        .filter(|unit| {
            unit[MAPPED_TARGET_FLAG_FIELD] == true || unit["unit_sysroot_context_flag"] == true
        })
        .try_fold(0, |count, unit| {
            let mut projection = CrateSyntaxProjectionState::build_crate_projection_state(
                unit, &workspace, store, scope, cancel,
            )?;
            projection.project_unit_source_root()?;
            persist_projection_input_rows(&mut projection, scope, store, cancel)
                .map(|rows| count + rows)
        })?;
    store.write_stage_table_unit(
        &StageUnitWriteRequest {
            stage_run_scope_value: scope,
            stage_owner_name_value: INPUT_STAGE,
            stage_unit_work_key: "__complete__",
            stage_input_stage_names: &["syntax-spans"],
            stage_unit_output_rows: &[],
        },
        cancel,
    )?;
    Ok(count)
}

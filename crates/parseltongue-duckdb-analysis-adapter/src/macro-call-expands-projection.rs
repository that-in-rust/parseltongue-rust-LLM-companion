use super::*;
use parseltongue_core_row_types::HldUnresolvedReasonKind;

const LOOKUP_TABLE: &str = "legacy-macro-scope-records";
const EXPANSION_TABLE: &str = "macro-expanded-item-table";

/// REQ-RUST-051.0 E1-E8: one `expands` row per source-written macro call node, from the
/// call to the macro definition its path names in textual scope, sited at the call.
pub(super) fn build_macro_expands_records(
    connection: &duckdb::Connection,
    source: &dyn StageTableStorePort,
    source_scope: &StageRunScopeValue,
    workspace_root: &Path,
    generation: RunGenerationDigestValue,
    cancel: &RunCancelFlagValue,
) -> Result<Vec<HldEdgeInputRecord>, OriginalSourceProjectionError> {
    connection
        .execute_batch(include_str!("macro-call-expands-projection.sql"))
        .map_err(|error| projection_backend_error(error.to_string()))?;
    let rows = read_projection_rows::<13>(
        connection,
        "SELECT unit_payload,root_scope_payload,request_payload,file_payload,span_payload,macro_name,target_kind,definition_unit_payload,definition_root_scope_payload,definition_request_payload,definition_file_payload,definition_span_payload,expansion_payload FROM \"macro-call-expands-candidates\" WHERE generation_key=? AND attempt_key=? ORDER BY unit_payload,request_payload,span_payload",
        source_scope,
    )?;
    // E4: the stored lookups of one call must agree, so one call is one candidate row.
    if rows.windows(2).any(|pair| pair[0][..5] == pair[1][..5]) {
        return Err(invalid_source_row(
            LOOKUP_TABLE,
            "macro call has more than one definition in scope or more than one expansion row"
                .into(),
        ));
    }
    rows.iter()
        .map(|row| {
            check_projection_cancel(cancel)?;
            build_macro_expands_edge(source, row, workspace_root, generation)
        })
        .collect()
}

fn build_macro_expands_edge(
    source: &dyn StageTableStorePort,
    row: &[String; 13],
    workspace_root: &Path,
    generation: RunGenerationDigestValue,
) -> Result<HldEdgeInputRecord, OriginalSourceProjectionError> {
    let area = read_original_graph_source(source, &row[..4], workspace_root)?;
    let call_span: Value = decode_json(&row[4], "span-reference-text-table")?;
    let call_occurrence = build_original_item_identity(&area, &call_span)?;
    let call =
        HldTargetIdentityValue::from(NameSymbolTargetKey::Definition(call_occurrence.clone()));
    let definition_span: Value = decode_json(&row[11], "span-reference-text-table")?;
    let expansion: Value = decode_json(&row[12], EXPANSION_TABLE)?;
    let expanded = expansion["macro_resolution_state_kind"] == "expanded";
    let (resolution, metadata) = match (row[6].as_str(), definition_span.is_null()) {
        ("definition", false) => {
            let definition_area = read_original_graph_source(source, &row[7..11], workspace_root)?;
            validate_macro_token_origins(&expansion, &definition_span)?;
            let target = HldTargetIdentityValue::from(NameSymbolTargetKey::Definition(
                build_original_item_identity(&definition_area, &definition_span)?,
            ));
            let arm = expanded
                .then(|| expansion["macro_matched_arm_index"].clone())
                .filter(|index| !index.is_null());
            let metadata = arm.map_or_else(
                || serde_json::json!({"macro_name_text": row[5]}),
                |index| serde_json::json!({"macro_name_text": row[5], "macro_matched_arm_index": index}),
            );
            (HldEdgeResolutionValue::Extracted { target }, metadata)
        }
        // E7: an expansion ran, so a definition was found some other way than textual scope.
        _ if expanded => {
            return Err(invalid_source_row(
                LOOKUP_TABLE,
                "expanded macro call has no definition node in textual scope".into(),
            ));
        }
        ("", _) => {
            let reason = expansion["macro_unresolved_reason_kind"].as_str();
            let metadata = serde_json::json!({
                "macro_name_text": Some(row[5].as_str()).filter(|name| !name.is_empty()),
                "macro_unresolved_reason_kind": reason,
                "macro_unresolved_detail_text": expansion["macro_unresolved_detail_text"],
            });
            (
                HldEdgeResolutionValue::Unresolved {
                    reason: match reason {
                        Some("external_source_missing") => {
                            HldUnresolvedReasonKind::ExternalSourceMissing
                        }
                        Some(_) => HldUnresolvedReasonKind::Other,
                        None => HldUnresolvedReasonKind::MissingTarget,
                    },
                },
                metadata,
            )
        }
        _ => {
            return Err(invalid_source_row(
                LOOKUP_TABLE,
                "macro definition in scope is not a graph node".into(),
            ));
        }
    };
    let target = match &resolution {
        HldEdgeResolutionValue::Extracted { target } => Some(target.clone()),
        _ => None,
    };
    Ok(HldEdgeInputRecord {
        run_generation_digest_value: generation,
        edge_source_reference: call.clone(),
        edge_relation_kind_value: HldRelationKindValue::Expands,
        edge_resolution_value: resolution,
        edge_evidence_row_key: HldEvidenceRowKey {
            evidence_site_occurrence_key: call_occurrence,
            evidence_source_occurrence_key: call,
            evidence_relation_kind_value: HldRelationKindValue::Expands,
            evidence_target_reference_value: target,
        },
        edge_dispatch_metadata_text: Some(metadata.to_string()),
    })
}

/// E5: every output token the expansion copied from a definition lies inside the target
/// definition, in the same file.
fn validate_macro_token_origins(
    expansion: &Value,
    definition_span: &Value,
) -> Result<(), OriginalSourceProjectionError> {
    let path = &definition_span["file_source_absolute_path"];
    let read_span_offset = |key: &str| definition_span[key].as_u64();
    let (start, end) = (
        read_span_offset("span_byte_start_offset"),
        read_span_offset("span_byte_end_offset"),
    );
    expansion["macro_token_origin_rows"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|origin| origin["token_source_origin_kind"] == "definition")
        .all(|origin| {
            let range = &origin["token_original_byte_range"];
            origin["token_source_absolute_path"] == *path
                && range[0].as_u64() >= start
                && range[1].as_u64() <= end
                && range[0].as_u64().is_some()
                && start.is_some()
        })
        .then_some(())
        .ok_or_else(|| {
            invalid_source_row(
                EXPANSION_TABLE,
                "expansion copied a token from outside the macro definition in scope".into(),
            )
        })
}

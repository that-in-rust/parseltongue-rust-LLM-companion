use super::*;

const SPAN_TABLE: &str = "span-reference-text-table";

/// REQ-RUST-046.0 R1-R8: items written by a macro expansion become nodes, each held by
/// the file area or module of the outermost source-written macro call and each with one
/// `generates` row from the macro call that wrote it.
pub(super) fn build_generated_item_records(
    connection: &duckdb::Connection,
    source: &dyn StageTableStorePort,
    source_scope: &StageRunScopeValue,
    workspace_root: &Path,
    generation: RunGenerationDigestValue,
    cancel: &RunCancelFlagValue,
) -> Result<(Vec<HldNodeInputRecord>, Vec<HldEdgeInputRecord>), OriginalSourceProjectionError> {
    connection
        .execute_batch(include_str!("generated-item-graph-projection.sql"))
        .map_err(|error| projection_backend_error(error.to_string()))?;
    let rows = read_projection_rows::<14>(
        connection,
        "SELECT unit_payload,root_scope_payload,request_payload,file_payload,span_payload,owner_payload,container_payload,lexical_names,impl_bridges,expanded_file_payload,call_payload,call_container_payload,call_lexical_names,call_span_payload FROM \"generated-item-graph-candidates\" WHERE generation_key=? AND attempt_key=? ORDER BY unit_payload,request_payload,span_payload",
        source_scope,
    )?;
    let inventory = read_projection_rows::<2>(
        connection,
        "SELECT projection_state, count(*)::VARCHAR FROM \"generated-item-source-inventory\" WHERE generation_key=? AND attempt_key=? GROUP BY projection_state ORDER BY projection_state",
        source_scope,
    )?;
    eprintln!("stage=generated-item-source-inventory counts={inventory:?}");
    let included = inventory
        .iter()
        .find(|row| row[0] == "included")
        .map_or("0", |row| row[1].as_str());
    if included != rows.len().to_string() {
        return Err(invalid_source_row(
            "macro-expanded-item-table",
            "generated item needs one macro call, one expanded file and one call file area".into(),
        ));
    }
    rows.iter()
        .map(|row| {
            check_projection_cancel(cancel)?;
            build_generated_item_pair(source, row, workspace_root, generation)
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|pairs| {
            let (nodes, edges): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
            (nodes, edges.into_iter().flatten().collect())
        })
}

fn build_generated_item_pair(
    source: &dyn StageTableStorePort,
    row: &[String; 14],
    workspace_root: &Path,
    generation: RunGenerationDigestValue,
) -> Result<(HldNodeInputRecord, [HldEdgeInputRecord; 2]), OriginalSourceProjectionError> {
    let area = read_original_graph_source(source, &row[..4], workspace_root)?;
    let span: Value = decode_json(&row[4], SPAN_TABLE)?;
    let owner: Value = decode_json(&row[5], SPAN_TABLE)?;
    let container: Value = decode_json(&row[6], SPAN_TABLE)?;
    let call_container: Value = decode_json(&row[11], SPAN_TABLE)?;
    let call_span: Value = decode_json(&row[13], SPAN_TABLE)?;
    if call_span.is_null() {
        return Err(invalid_source_row(
            "macro-expanded-item-table",
            "generating macro call is not a source-written macro call span".into(),
        ));
    }
    let call_occurrence = build_original_item_identity(&area, &call_span)?;
    let call =
        HldTargetIdentityValue::from(NameSymbolTargetKey::Definition(call_occurrence.clone()));
    let names = decode_lexical_name_list(&row[12])?
        .into_iter()
        .chain(decode_lexical_name_list(&row[7])?)
        .collect::<Vec<_>>();
    let bridges: Vec<Value> =
        serde_json::from_str(&row[8]).map_err(|error: serde_json::Error| {
            invalid_source_row("impl-header-index-table", error.to_string())
        })?;
    let expanded = read_generated_source_context(source, &area, &row[0], &row[9], &span)?;
    let occurrence = build_original_item_identity(&expanded, &span)?;
    let lexical_owner = if owner.is_null() {
        &call_container
    } else {
        &owner
    };
    let (item_kind, owner_kind) =
        decode_original_item_kinds(&span, lexical_owner, area.file_is_crate_root)?;
    if owner_kind == HldOwnerKindValue::Impl {
        validate_original_impl_bridge(&expanded, &span, &owner, &bridges)?;
    }
    let target = HldTargetIdentityValue::from(NameSymbolTargetKey::Definition(occurrence.clone()));
    let holder = if !container.is_null() {
        HldTargetIdentityValue::from(NameSymbolTargetKey::Definition(
            build_original_item_identity(&expanded, &container)?,
        ))
    } else if !call_container.is_null() {
        HldTargetIdentityValue::from(NameSymbolTargetKey::Definition(
            build_original_item_identity(&area, &call_container)?,
        ))
    } else {
        build_original_area_identity(&area)?
    };
    let kind = required_string(&span, "span_syntax_kind_value", SPAN_TABLE)?;
    let name = span["span_declared_name_value"]
        .as_str()
        .filter(|name| !name.is_empty())
        .unwrap_or(if kind == "CONST" { "_" } else { kind });
    let label = [
        area.unit.unit_cargo_package_identifier.as_str(),
        area.path.as_str(),
    ]
    .into_iter()
    .chain(names.iter().map(String::as_str))
    .chain(std::iter::once(name.trim_start_matches("r#")))
    .collect::<Vec<_>>()
    .join("::");
    let node = HldNodeInputRecord {
        run_generation_digest_value: generation,
        node_target_reference: target.clone(),
        node_input_category: HldNodeInputCategory::LocalItem,
        node_item_kind_value: Some(item_kind),
        node_owner_kind_value: Some(owner_kind),
        node_function_body_ancestry_flag: false,
        node_effective_visibility_kind: Some(HldEffectiveVisibilityKind::Unknown),
        node_reachable_outside_crate_flag: None,
        node_source_relative_path: Some(area.path),
        node_qualified_path_text: label,
        node_signature_text: None,
        node_generating_call_reference: Some(call.clone()),
    };
    let generates = HldEdgeInputRecord {
        run_generation_digest_value: generation,
        edge_source_reference: call.clone(),
        edge_relation_kind_value: HldRelationKindValue::Generates,
        edge_resolution_value: HldEdgeResolutionValue::Extracted {
            target: target.clone(),
        },
        edge_evidence_row_key: HldEvidenceRowKey {
            evidence_site_occurrence_key: call_occurrence,
            evidence_source_occurrence_key: call,
            evidence_relation_kind_value: HldRelationKindValue::Generates,
            evidence_target_reference_value: Some(target.clone()),
        },
        edge_dispatch_metadata_text: None,
    };
    let edge = HldEdgeInputRecord {
        run_generation_digest_value: generation,
        edge_source_reference: holder.clone(),
        edge_relation_kind_value: HldRelationKindValue::Contains,
        edge_resolution_value: HldEdgeResolutionValue::Extracted {
            target: target.clone(),
        },
        edge_evidence_row_key: HldEvidenceRowKey {
            evidence_site_occurrence_key: occurrence,
            evidence_source_occurrence_key: holder,
            evidence_relation_kind_value: HldRelationKindValue::Contains,
            evidence_target_reference_value: Some(target),
        },
        edge_dispatch_metadata_text: None,
    };
    Ok((node, [edge, generates]))
}

fn decode_lexical_name_list(text: &str) -> Result<Vec<String>, OriginalSourceProjectionError> {
    serde_json::from_str(text)
        .map_err(|error: serde_json::Error| invalid_source_row(SPAN_TABLE, error.to_string()))
}

/// R6: the expanded text is read back from the store and checked against its digest.
fn read_generated_source_context(
    source: &dyn StageTableStorePort,
    area: &OriginalGraphSourceContext,
    unit_payload: &str,
    file_payload: &str,
    span: &Value,
) -> Result<OriginalGraphSourceContext, OriginalSourceProjectionError> {
    let unit: Value = decode_json(unit_payload, "crate-dependency-graph-table")?;
    let file: Value = decode_json(file_payload, "file-source-text-table")?;
    let file_digest = required_string(&file, "file_content_digest_full", "file-source-text-table")?;
    let bytes = source.read_file_content_bytes(file_digest)?;
    validate_original_file_digest(&bytes, file_digest)?;
    let path = Path::new(required_string(
        span,
        "file_source_absolute_path",
        SPAN_TABLE,
    )?)
    .strip_prefix(required_string(
        &unit,
        "unit_package_root_path",
        "crate-dependency-graph-table",
    )?)
    .map_err(|_| invalid_source_row(SPAN_TABLE, "expanded source outside package root".into()))?
    .to_string_lossy()
    .replace('\\', "/");
    let chain_digests = span["span_expansion_provenance_chain"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|digest| {
            digest.as_str().map(str::to_owned).ok_or_else(|| {
                invalid_source_row(SPAN_TABLE, "expansion chain digest is not text".into())
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(OriginalGraphSourceContext {
        unit: area.unit.clone(),
        canonical_unit: area.canonical_unit.clone(),
        path,
        bytes,
        file_digest: file_digest.into(),
        file_is_crate_root: false,
        chain_digests,
    })
}

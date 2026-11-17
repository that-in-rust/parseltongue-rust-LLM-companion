use super::*;

/// REQ-RUST-070.0 A2, A3: one `contains` row per active `mod name;` declaration outside a
/// function body whose file the run read, from the module to the file area of that file,
/// sited at the declaration. An inline module has no request row, so it gets no such row.
pub(super) fn build_module_area_records(
    connection: &duckdb::Connection,
    source: &dyn StageTableStorePort,
    source_scope: &StageRunScopeValue,
    workspace_root: &Path,
    generation: RunGenerationDigestValue,
    cancel: &RunCancelFlagValue,
) -> Result<Vec<HldEdgeInputRecord>, OriginalSourceProjectionError> {
    connection
        .execute_batch(include_str!("module-file-area-projection.sql"))
        .map_err(|error| projection_backend_error(error.to_string()))?;
    let rows = read_projection_rows::<9>(
        connection,
        "SELECT unit_payload,root_scope_payload,request_payload,file_payload,span_payload,target_unit_payload,target_root_scope_payload,target_request_payload,target_file_payload FROM \"original-module-file-links\" WHERE generation_key=? AND attempt_key=? ORDER BY unit_payload,request_payload,span_payload,target_request_payload",
        source_scope,
    )?;
    // One declaration names one file: two read candidates would be a guess between them.
    if rows.windows(2).any(|pair| pair[0][..5] == pair[1][..5]) {
        return Err(invalid_source_row(
            MODULE_FILE_LINK_TABLE,
            "module declaration names more than one read file".into(),
        ));
    }
    rows.iter()
        .map(|row| {
            check_projection_cancel(cancel)?;
            let declaring = read_original_graph_source(source, &row[..4], workspace_root)?;
            let span: Value = decode_json(&row[4], "span-reference-text-table")?;
            let site = build_original_item_identity(&declaring, &span)?;
            let target = read_original_graph_source(source, &row[5..9], workspace_root)?;
            Ok(build_area_contains_edge(
                generation,
                HldTargetIdentityValue::from(NameSymbolTargetKey::Definition(site.clone())),
                site,
                build_original_area_identity(&target)?,
            ))
        })
        .collect()
}

/// REQ-RUST-070.0 A1: the crate root contains the file area of the unit's root file. The
/// link is Cargo's root path of the target, so the site is the whole file.
pub(super) fn build_root_area_edge(
    context: &OriginalGraphSourceContext,
    generation: RunGenerationDigestValue,
) -> Result<HldEdgeInputRecord, OriginalSourceProjectionError> {
    let site = SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
        &context.unit,
        &context.path,
        SpanByteRangeValue::try_from((0, context.bytes.len() as u64))
            .map_err(|error| invalid_source_row("file-source-text-table", error.to_string()))?,
        SpanDigestIdValue::from(*blake3::hash(&context.bytes).as_bytes()),
        &[],
    )
    .map_err(|error| invalid_source_row("file-source-text-table", error.to_string()))?;
    Ok(build_area_contains_edge(
        generation,
        HldTargetIdentityValue::from(NameSymbolTargetKey::CrateRoot(context.unit.clone())),
        site,
        build_original_area_identity(context)?,
    ))
}

fn build_area_contains_edge(
    generation: RunGenerationDigestValue,
    module: HldTargetIdentityValue,
    site: SemanticOccurrenceKeyValue,
    area: HldTargetIdentityValue,
) -> HldEdgeInputRecord {
    HldEdgeInputRecord {
        run_generation_digest_value: generation,
        edge_source_reference: module.clone(),
        edge_relation_kind_value: HldRelationKindValue::Contains,
        edge_resolution_value: HldEdgeResolutionValue::Extracted {
            target: area.clone(),
        },
        edge_evidence_row_key: HldEvidenceRowKey {
            evidence_site_occurrence_key: site,
            evidence_source_occurrence_key: module,
            evidence_relation_kind_value: HldRelationKindValue::Contains,
            evidence_target_reference_value: Some(area),
        },
        edge_dispatch_metadata_text: None,
    }
}

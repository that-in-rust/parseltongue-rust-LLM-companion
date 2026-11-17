use super::*;

#[cfg(test)]
#[path = "original-item-projection-tests.rs"]
mod original_item_projection_tests;

#[path = "generated-item-graph-projection.rs"]
mod generated_item_graph_projection;
#[path = "macro-call-expands-projection.rs"]
mod macro_call_expands_projection;

#[path = "item-node-label-text.rs"]
mod item_node_label_text;
#[path = "module-file-area-projection.rs"]
mod module_file_area_projection;

const MODULE_FILE_LINK_TABLE: &str = "module-source-closure-requests";

impl DuckdbAnalysisStoreAdapter {
    pub fn derive_original_graph_input(
        &self,
        source: &dyn StageTableStorePort,
        source_scope: &StageRunScopeValue,
        destination_base_scope: &StageRunScopeValue,
        workspace_root: &Path,
        cancel: &RunCancelFlagValue,
    ) -> Result<HldMaterializationInputRecord, OriginalSourceProjectionError> {
        check_projection_cancel(cancel)?;
        if source_scope.run_generation_digest_value
            != destination_base_scope.run_generation_digest_value
        {
            return Err(invalid_source_row(
                "original-item-graph-input",
                "source/destination generation mismatch".into(),
            ));
        }
        let tables = ORIGINAL_SOURCE_PROJECTION_TABLE_NAMES
            .iter()
            .copied()
            .chain([
                "impl-header-index-table",
                "macro-expanded-item-table",
                "legacy-macro-scope-records",
                MODULE_FILE_LINK_TABLE,
            ])
            .collect::<Vec<_>>();
        self.copy_published_stage_rows(source, source_scope, &tables, cancel)?;
        let connection = self
            ._connection
            .lock()
            .map_err(|error| projection_backend_error(error.to_string()))?;
        connection
            .execute_batch(include_str!("hld-original-source-projection.sql"))
            .and_then(|()| {
                connection.execute_batch(include_str!("original-item-graph-projection.sql"))
            })
            .map_err(|error| projection_backend_error(error.to_string()))?;
        validate_original_graph_inventory(&connection, source_scope)?;
        let generation = decode_generation(&source_scope.run_generation_digest_value)?;
        let areas = read_projection_rows::<4>(
            &connection,
            "SELECT unit_payload,root_scope_payload,request_payload,file_payload FROM \"original-source-area-candidates\" WHERE generation_key=? AND attempt_key=? ORDER BY unit_payload,request_payload",
            source_scope,
        )?;
        let mut units = BTreeSet::new();
        let mut nodes = BTreeSet::new();
        let mut edges = BTreeSet::new();
        for row in &areas {
            check_projection_cancel(cancel)?;
            let context = read_original_graph_source(source, row, workspace_root)?;
            units.insert(HldUnitProvenanceRecord {
                unit_compilation_identity_key: context.unit.clone(),
                unit_origin_kind: HldUnitOriginKind::Workspace,
            });
            if context.file_is_crate_root {
                nodes.insert(build_crate_root_node(&context, generation));
                edges.insert(module_file_area_projection::build_root_area_edge(
                    &context, generation,
                )?);
            }
            nodes.insert(HldNodeInputRecord {
                run_generation_digest_value: generation,
                node_target_reference: build_original_area_identity(&context)?,
                node_input_category: HldNodeInputCategory::FilePublicArea,
                node_item_kind_value: Some(HldItemKindValue::FilePublicArea),
                node_owner_kind_value: None,
                node_function_body_ancestry_flag: false,
                node_effective_visibility_kind: None,
                node_reachable_outside_crate_flag: None,
                node_source_relative_path: Some(context.path.clone()),
                node_qualified_path_text: item_node_label_text::join_node_label_segments(
                    &context.unit,
                    &context.path,
                    &[],
                    None,
                    "<file-area>",
                ),
                node_signature_text: None,
                node_generating_call_reference: None,
            });
        }
        let items = read_projection_rows::<9>(
            &connection,
            "SELECT unit_payload,root_scope_payload,request_payload,file_payload,span_payload,owner_payload,container_payload,lexical_names,impl_bridges FROM \"original-item-graph-candidates\" WHERE generation_key=? AND attempt_key=? ORDER BY unit_payload,request_payload,span_payload",
            source_scope,
        )?;
        for row in &items {
            check_projection_cancel(cancel)?;
            let context = read_original_graph_source(source, &row[..4], workspace_root)?;
            let span: Value = decode_json(&row[4], "span-reference-text-table")?;
            let owner: Value = decode_json(&row[5], "span-reference-text-table")?;
            let container: Value = decode_json(&row[6], "span-reference-text-table")?;
            let names: Vec<String> =
                serde_json::from_str(&row[7]).map_err(|error: serde_json::Error| {
                    invalid_source_row("original-item-source-ancestors", error.to_string())
                })?;
            let bridges: Vec<Value> =
                serde_json::from_str(&row[8]).map_err(|error: serde_json::Error| {
                    invalid_source_row("impl-header-index-table", error.to_string())
                })?;
            let occurrence = build_original_item_identity(&context, &span)?;
            let (item_kind, owner_kind) =
                decode_original_item_kinds(&span, &owner, context.file_is_crate_root)?;
            if owner_kind == HldOwnerKindValue::Impl {
                validate_original_impl_bridge(&context, &span, &owner, &bridges)?;
            }
            let target =
                HldTargetIdentityValue::from(NameSymbolTargetKey::Definition(occurrence.clone()));
            let source = if container.is_null() {
                build_original_area_identity(&context)?
            } else {
                HldTargetIdentityValue::from(NameSymbolTargetKey::Definition(
                    build_original_item_identity(&context, &container)?,
                ))
            };
            required_string(&span, "span_syntax_kind_value", "span-reference-text-table")?;
            let label = item_node_label_text::join_node_label_segments(
                &context.unit,
                &context.path,
                &names,
                bridges
                    .first()
                    .filter(|_| owner_kind == HldOwnerKindValue::Impl)
                    .map(|bridge| &bridge["header"]),
                &item_node_label_text::build_item_name_segment(&span, None),
            );
            nodes.insert(HldNodeInputRecord {
                run_generation_digest_value: generation,
                node_target_reference: target.clone(),
                node_input_category: HldNodeInputCategory::LocalItem,
                node_item_kind_value: Some(item_kind),
                node_owner_kind_value: Some(owner_kind),
                node_function_body_ancestry_flag: false,
                node_effective_visibility_kind: Some(HldEffectiveVisibilityKind::Unknown),
                node_reachable_outside_crate_flag: None,
                node_source_relative_path: Some(context.path),
                node_qualified_path_text: label,
                node_signature_text: None,
                node_generating_call_reference: None,
            });
            edges.insert(HldEdgeInputRecord {
                run_generation_digest_value: generation,
                edge_source_reference: source.clone(),
                edge_relation_kind_value: HldRelationKindValue::Contains,
                edge_resolution_value: HldEdgeResolutionValue::Extracted {
                    target: target.clone(),
                },
                edge_evidence_row_key: HldEvidenceRowKey {
                    evidence_site_occurrence_key: occurrence,
                    evidence_source_occurrence_key: source,
                    evidence_relation_kind_value: HldRelationKindValue::Contains,
                    evidence_target_reference_value: Some(target),
                },
                edge_dispatch_metadata_text: None,
            });
        }
        let (generated_nodes, generated_edges) =
            generated_item_graph_projection::build_generated_item_records(
                &connection,
                source,
                source_scope,
                workspace_root,
                generation,
                cancel,
            )?;
        nodes.extend(generated_nodes);
        edges.extend(generated_edges);
        edges.extend(macro_call_expands_projection::build_macro_expands_records(
            &connection,
            source,
            source_scope,
            workspace_root,
            generation,
            cancel,
        )?);
        edges.extend(module_file_area_projection::build_module_area_records(
            &connection,
            source,
            source_scope,
            workspace_root,
            generation,
            cancel,
        )?);
        let inventory = read_projection_rows::<2>(
            &connection,
            "SELECT projection_state, count(*)::VARCHAR FROM \"original-item-source-inventory\" WHERE generation_key=? AND attempt_key=? GROUP BY projection_state ORDER BY projection_state",
            source_scope,
        )?;
        eprintln!("stage=original-item-source-inventory counts={inventory:?}");
        let input = HldMaterializationInputRecord {
            run_generation_digest_value: generation,
            stage_attempt_sequence_value: destination_base_scope
                .stage_attempt_sequence_value
                .clone(),
            semantic_input_completeness_kind: HldInputCompletenessKind::IncompleteTypedInput,
            source_coverage_kind: HldSourceCoverageKind::FullSemantic,
            input_unit_provenance_records: units.into_iter().collect(),
            input_node_records: nodes.into_iter().collect(),
            input_edge_records: edges.into_iter().collect(),
        };
        parseltongue_core_row_types::validate_full_graph_input(&input)
            .map_err(|error| invalid_source_row("original-item-graph-input", error.to_string()))?;
        check_projection_cancel(cancel)?;
        Ok(input)
    }
}

fn validate_original_graph_inventory(
    connection: &duckdb::Connection,
    scope: &StageRunScopeValue,
) -> Result<(), OriginalSourceProjectionError> {
    let missing = read_projection_rows::<1>(
        connection,
        "SELECT unit.payload FROM \"frozen-generation-source-records\" unit WHERE unit.generation_key=? AND unit.attempt_key=? AND unit.table_name='crate-dependency-graph-table' AND json_extract_string(unit.payload,'$.unit_mapped_target_flag')='true' AND NOT EXISTS (SELECT 1 FROM \"frozen-generation-source-records\" root WHERE root.generation_key=unit.generation_key AND root.attempt_key=unit.attempt_key AND root.table_name='syntax-namespace-input-scopes' AND json_extract_string(root.payload,'$.scope_cargo_unit_key')=json_extract_string(unit.payload,'$.unit_compilation_identity_key') AND json_array_length(json_extract(root.payload,'$.scope_module_identity_key.scope_semantic_path_parts'))=0)",
        scope,
    )?;
    if let Some(row) = missing.first() {
        let unit: Value = decode_json(&row[0], "crate-dependency-graph-table")?;
        return Err(OriginalSourceProjectionError::MissingRootMapping {
            raw_unit_key: required_string(
                &unit,
                "unit_compilation_identity_key",
                "crate-dependency-graph-table",
            )?
            .into(),
        });
    }
    let missing = read_projection_rows::<1>(
        connection,
        "SELECT request_payload FROM \"original-source-missing-files\" WHERE generation_key=? AND attempt_key=?",
        scope,
    )?;
    if let Some(row) = missing.first() {
        let request: Value = decode_json(&row[0], "syntax-file-work-requests")?;
        return Err(OriginalSourceProjectionError::MissingFileEvidence {
            file_path: required_string(
                &request,
                "file_source_absolute_path",
                "syntax-file-work-requests",
            )?
            .into(),
        });
    }
    if !read_projection_rows::<1>(connection,
        "SELECT span_payload FROM \"original-source-unowned-items\" WHERE generation_key=? AND attempt_key=?", scope)?.is_empty() {
        return Err(OriginalSourceProjectionError::MissingSpanOwnership);
    }
    if !read_projection_rows::<1>(connection,
        "SELECT unit_key FROM \"original-item-source-spans\" WHERE generation_key=? AND attempt_key=? GROUP BY unit_key,file_path,start_byte,end_byte HAVING count(DISTINCT payload)>1", scope)?.is_empty() {
        return Err(invalid_source_row("span-reference-text-table", "conflicting original source occurrence".into()));
    }
    Ok(())
}

struct OriginalGraphSourceContext {
    unit: CrateCompilationUnitKey,
    canonical_unit: Value,
    path: String,
    bytes: Vec<u8>,
    file_digest: String,
    file_is_crate_root: bool,
    chain_digests: Vec<String>,
}

fn read_original_graph_source(
    source: &dyn StageTableStorePort,
    row: &[String],
    root: &Path,
) -> Result<OriginalGraphSourceContext, OriginalSourceProjectionError> {
    let unit: Value = decode_json(&row[0], "crate-dependency-graph-table")?;
    let namespace: Value = decode_json(&row[1], "syntax-namespace-input-scopes")?;
    let request: Value = decode_json(&row[2], "syntax-file-work-requests")?;
    let file: Value = decode_json(&row[3], "file-source-text-table")?;
    let file_digest = required_string(&file, "file_content_digest_full", "file-source-text-table")?;
    let bytes = source.read_file_content_bytes(file_digest)?;
    validate_original_file_digest(&bytes, file_digest)?;
    Ok(OriginalGraphSourceContext {
        unit: canonical_unit(&unit, &namespace)?,
        canonical_unit: namespace["scope_module_identity_key"]["scope_compilation_unit_key"]
            .clone(),
        path: source_relative_path(&unit, &request, root)?,
        bytes,
        file_digest: file_digest.into(),
        file_is_crate_root: Path::new(required_string(
            &unit,
            "unit_source_root_path",
            "crate-dependency-graph-table",
        )?) == Path::new(required_string(
            &request,
            "file_source_absolute_path",
            "syntax-file-work-requests",
        )?),
        chain_digests: Vec::new(),
    })
}

/// REQ-RUST-043.1 C1: the node of a unit's root module.
fn build_crate_root_node(
    context: &OriginalGraphSourceContext,
    generation: RunGenerationDigestValue,
) -> HldNodeInputRecord {
    HldNodeInputRecord {
        run_generation_digest_value: generation,
        node_target_reference: HldTargetIdentityValue::from(NameSymbolTargetKey::CrateRoot(
            context.unit.clone(),
        )),
        node_input_category: HldNodeInputCategory::CrateRoot,
        node_item_kind_value: None,
        node_owner_kind_value: None,
        node_function_body_ancestry_flag: false,
        node_effective_visibility_kind: None,
        node_reachable_outside_crate_flag: None,
        node_source_relative_path: None,
        node_qualified_path_text: context.unit.unit_cargo_target_name.clone(),
        node_signature_text: None,
        node_generating_call_reference: None,
    }
}

fn build_original_area_identity(
    context: &OriginalGraphSourceContext,
) -> Result<HldTargetIdentityValue, OriginalSourceProjectionError> {
    FileAreaIdentityKey::build_file_area_identity(&context.unit, &context.path)
        .map(HldTargetIdentityValue::FilePublicArea)
        .map_err(|error| invalid_source_row("syntax-file-work-requests", error.to_string()))
}

fn build_original_item_identity(
    context: &OriginalGraphSourceContext,
    span: &Value,
) -> Result<SemanticOccurrenceKeyValue, OriginalSourceProjectionError> {
    let start = required_u64(span, "span_byte_start_offset", "span-reference-text-table")?;
    let end = required_u64(span, "span_byte_end_offset", "span-reference-text-table")?;
    let text = required_string(span, "span_source_code_text", "span-reference-text-table")?;
    let digest = required_string(
        span,
        "span_content_digest_full",
        "span-reference-text-table",
    )?;
    validate_original_source_evidence(
        &context.bytes,
        &context.file_digest,
        start,
        end,
        text,
        digest,
    )?;
    SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
        &context.unit,
        &context.path,
        SpanByteRangeValue::try_from((start, end))
            .map_err(|error| invalid_source_row("span-reference-text-table", error.to_string()))?,
        SpanDigestIdValue::from(decode_digest(digest)?),
        &context
            .chain_digests
            .iter()
            .map(|digest| decode_digest(digest))
            .collect::<Result<Vec<_>, _>>()?,
    )
    .map_err(|error| invalid_source_row("span-reference-text-table", error.to_string()))
}

fn encode_original_bridge_identity(context: &OriginalGraphSourceContext, span: &Value) -> Value {
    serde_json::json!({
        "unit_compilation_identity_key": context.canonical_unit,
        "span_source_file_path": context.path,
        "span_byte_range_value": [span["span_byte_start_offset"], span["span_byte_end_offset"]],
        "span_content_digest_full": span["span_content_digest_full"],
        "span_expansion_provenance_chain": context.chain_digests,
    })
}

fn validate_original_impl_bridge(
    context: &OriginalGraphSourceContext,
    span: &Value,
    owner: &Value,
    bridges: &[Value],
) -> Result<(), OriginalSourceProjectionError> {
    build_original_item_identity(context, owner)?;
    if bridges.len() != 1
        || bridges[0]["header"]["impl_function_body_ancestry_flag"] != false
        || bridges[0]["header"]["impl_semantic_occurrence_key"]
            != encode_original_bridge_identity(context, owner)
        || bridges[0]["member"]["member_semantic_occurrence_key"]
            != encode_original_bridge_identity(context, span)
    {
        return Err(invalid_source_row(
            "impl-header-index-table",
            "original member needs one matching canonical impl bridge".into(),
        ));
    }
    Ok(())
}

fn decode_original_item_kinds(
    span: &Value,
    owner: &Value,
    file_is_crate_root: bool,
) -> Result<(HldItemKindValue, HldOwnerKindValue), OriginalSourceProjectionError> {
    let owner_kind = if owner.is_null() {
        if file_is_crate_root {
            HldOwnerKindValue::CrateRoot
        } else {
            HldOwnerKindValue::Module
        }
    } else {
        match required_string(owner, "span_syntax_kind_value", "span-reference-text-table")? {
            "MODULE" => HldOwnerKindValue::Module,
            "TRAIT" => HldOwnerKindValue::Trait,
            "IMPL" => HldOwnerKindValue::Impl,
            "EXTERN_BLOCK" => HldOwnerKindValue::ExternBlock,
            other => {
                return Err(OriginalSourceProjectionError::UnsupportedActiveSyntax {
                    syntax_kind: other.into(),
                });
            }
        }
    };
    let associated = matches!(
        owner_kind,
        HldOwnerKindValue::Trait | HldOwnerKindValue::Impl
    );
    let kind = match required_string(span, "span_syntax_kind_value", "span-reference-text-table")? {
        "FN" if associated => HldItemKindValue::AssociatedFunction,
        "FN" => HldItemKindValue::Function,
        "CONST" if associated => HldItemKindValue::AssociatedConstant,
        "CONST" => HldItemKindValue::Constant,
        "TYPE_ALIAS" if associated => HldItemKindValue::AssociatedType,
        "TYPE_ALIAS" => HldItemKindValue::TypeAlias,
        "STATIC" => HldItemKindValue::Static,
        "STRUCT" => HldItemKindValue::Struct,
        "ENUM" => HldItemKindValue::Enum,
        "TRAIT" => HldItemKindValue::Trait,
        "MODULE" => HldItemKindValue::Module,
        "USE" => HldItemKindValue::UseStatement,
        "UNION" => HldItemKindValue::Union,
        "MACRO_RULES" | "MACRO_DEF" => HldItemKindValue::MacroDefinition,
        "MACRO_CALL" => HldItemKindValue::MacroInvocation,
        "EXTERN_BLOCK" => HldItemKindValue::ExternBlock,
        "EXTERN_CRATE" => HldItemKindValue::ExternCrate,
        "ASM_EXPR" => HldItemKindValue::InlineAssembly,
        other => {
            return Err(OriginalSourceProjectionError::UnsupportedActiveSyntax {
                syntax_kind: other.into(),
            });
        }
    };
    Ok((kind, owner_kind))
}

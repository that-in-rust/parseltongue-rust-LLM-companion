#![allow(clippy::unwrap_used)]

use super::*;
use parseltongue_core_row_types::{
    CompilationUnitRoleKind, CrateCompilationUnitKey, HldDependencyOriginKind, HldEdgeInputRecord,
    HldEdgeResolutionValue, HldEffectiveVisibilityKind, HldEndpointRollupBucket,
    HldEvidencePathRecord, HldEvidenceRowKey, HldFileRollupRecord, HldFolderEndpointBucket,
    HldFolderRollupRecord, HldGraphEdgeKey, HldInputCompletenessKind, HldItemKindValue,
    HldMaterializationInputRecord, HldMaterializationValidationError, HldNodeInputCategory,
    HldNodeInputRecord, HldOutputCompletenessKind, HldOutputEdgeStatus, HldOwnerKindValue,
    HldRelationKindValue, HldSourceCoverageKind, HldTargetIdentityValue, HldUnitOriginKind,
    HldUnitProvenanceRecord, HldUnresolvedReasonKind, HldViaKindValue, NameSymbolTargetKey,
    RunCancelFlagValue, RunGenerationDigestValue, SemanticOccurrenceKeyValue, SpanByteRangeValue,
    SpanDigestIdValue, StageRunScopeValue, StageUnitWriteRequest,
    validate_hld_materialization_input,
};
use parseltongue_table_store_traits::{
    HldGraphAnalysisPort, HldGraphPublicationPort, StageTableStorePort,
};
use parseltongue_turso_store_adapter::TursoWorkspaceStoreAdapter;
use std::collections::{BTreeMap, BTreeSet};

fn build_hld_fixture_unit(package_name: &str, context_byte: u8) -> CrateCompilationUnitKey {
    CrateCompilationUnitKey {
        unit_cargo_package_identifier: format!(
            "registry+https://example.invalid#{package_name}@1.0.0"
        ),
        unit_cargo_target_name: package_name.to_owned(),
        unit_cargo_target_kind: "lib".into(),
        unit_compilation_role_kind: CompilationUnitRoleKind::Target,
        unit_configuration_digest_full: [context_byte; 32],
    }
}

fn build_hld_fixture_occurrence(
    unit_identity: &CrateCompilationUnitKey,
    file_path: &str,
    start_offset: u64,
    digest_byte: u8,
) -> SemanticOccurrenceKeyValue {
    SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
        unit_identity,
        file_path,
        SpanByteRangeValue::try_from((start_offset, start_offset + 1)).unwrap(),
        SpanDigestIdValue::from([digest_byte; 32]),
        &[],
    )
    .unwrap()
}

#[allow(clippy::too_many_arguments)]
fn build_hld_fixture_node(
    generation: RunGenerationDigestValue,
    target: NameSymbolTargetKey,
    category: HldNodeInputCategory,
    item_kind: Option<HldItemKindValue>,
    owner_kind: Option<HldOwnerKindValue>,
    in_function_body: bool,
    visibility: Option<HldEffectiveVisibilityKind>,
    reachable_outside_crate: bool,
    source_path: Option<&str>,
    qualified_path: &str,
) -> HldNodeInputRecord {
    HldNodeInputRecord {
        run_generation_digest_value: generation,
        node_target_reference: target.into(),
        node_input_category: category,
        node_item_kind_value: item_kind,
        node_owner_kind_value: owner_kind,
        node_function_body_ancestry_flag: in_function_body,
        node_effective_visibility_kind: visibility,
        node_reachable_outside_crate_flag: (category == HldNodeInputCategory::LocalItem)
            .then_some(reachable_outside_crate),
        node_source_relative_path: source_path.map(str::to_owned),
        node_qualified_path_text: qualified_path.to_owned(),
        node_signature_text: None,
    }
}

fn build_hld_fixture_edge<Source, Target>(
    generation: RunGenerationDigestValue,
    source: Source,
    target: Target,
    relation: HldRelationKindValue,
    evidence: SemanticOccurrenceKeyValue,
) -> HldEdgeInputRecord
where
    Source: Into<HldTargetIdentityValue>,
    Target: Into<HldTargetIdentityValue>,
{
    let source = source.into();
    let target = target.into();
    HldEdgeInputRecord {
        run_generation_digest_value: generation,
        edge_source_reference: source.clone(),
        edge_relation_kind_value: relation,
        edge_resolution_value: HldEdgeResolutionValue::Extracted {
            target: target.clone(),
        },
        edge_evidence_row_key: HldEvidenceRowKey {
            evidence_site_occurrence_key: evidence,
            evidence_source_occurrence_key: source,
            evidence_relation_kind_value: relation,
            evidence_target_reference_value: Some(target),
        },
        edge_dispatch_metadata_text: None,
    }
}

fn build_hld_fixture_input(
    generation: RunGenerationDigestValue,
    provenance: Vec<HldUnitProvenanceRecord>,
    nodes: Vec<HldNodeInputRecord>,
    edges: Vec<HldEdgeInputRecord>,
) -> HldMaterializationInputRecord {
    HldMaterializationInputRecord {
        run_generation_digest_value: generation,
        stage_attempt_sequence_value: "attempt-a".into(),
        semantic_input_completeness_kind: HldInputCompletenessKind::CompleteTypedInput,
        source_coverage_kind: HldSourceCoverageKind::FullSemantic,
        input_unit_provenance_records: provenance,
        input_node_records: nodes,
        input_edge_records: edges,
    }
}

fn build_hld_fixture_store() -> DuckdbAnalysisStoreAdapter {
    DuckdbAnalysisStoreAdapter {
        _connection: std::sync::Mutex::new(duckdb::Connection::open_in_memory().unwrap()),
    }
}

fn materialize_hld_fixture_graph(
    input: &HldMaterializationInputRecord,
) -> parseltongue_core_row_types::HldMaterializationOutputRecord {
    let store = build_hld_fixture_store();
    let output = store
        .materialize_hld_graph_output(input, &RunCancelFlagValue::default())
        .unwrap();
    eprintln!(
        "HLD_TEST_DUCKDB_MEMORY={}",
        store.read_namespace_memory_usage().unwrap()
    );
    output
}

fn build_hld_path_node(
    generation: RunGenerationDigestValue,
    unit_identity: &CrateCompilationUnitKey,
    file_name: &str,
    offset: u8,
    visibility: HldEffectiveVisibilityKind,
    qualified_path: &str,
) -> HldNodeInputRecord {
    let source_path = format!("src/{file_name}.rs");
    build_hld_fixture_node(
        generation,
        NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
            unit_identity,
            &source_path,
            u64::from(offset),
            offset,
        )),
        HldNodeInputCategory::LocalItem,
        Some(HldItemKindValue::Function),
        Some(HldOwnerKindValue::Module),
        false,
        Some(visibility),
        false,
        Some(&source_path),
        qualified_path,
    )
}

fn build_hld_path_edge<Source, Target>(
    generation: RunGenerationDigestValue,
    unit_identity: &CrateCompilationUnitKey,
    source: &Source,
    target: &Target,
    relation: HldRelationKindValue,
    site_file_name: &str,
    site_offset: u8,
) -> HldEdgeInputRecord
where
    Source: Clone + Into<HldTargetIdentityValue>,
    Target: Clone + Into<HldTargetIdentityValue>,
{
    let site_path = format!("src/{site_file_name}.rs");
    build_hld_fixture_edge(
        generation,
        source.clone().into(),
        target.clone().into(),
        relation,
        build_hld_fixture_occurrence(
            unit_identity,
            &site_path,
            u64::from(site_offset),
            site_offset,
        ),
    )
}

fn expected_hld_edge_key<Source, Target>(
    generation: RunGenerationDigestValue,
    source: Source,
    target: Target,
    relation: HldRelationKindValue,
    via: HldViaKindValue,
) -> HldGraphEdgeKey
where
    Source: Into<HldTargetIdentityValue>,
    Target: Into<HldTargetIdentityValue>,
{
    HldGraphEdgeKey {
        run_generation_digest_value: generation,
        edge_source_occurrence_key: source.into(),
        edge_target_reference_value: target.into(),
        edge_relation_kind_value: relation,
        edge_collapse_via_kind: via,
    }
}

fn build_hld_bucket_node(
    generation: RunGenerationDigestValue,
    unit_identity: &CrateCompilationUnitKey,
    source_relative_path: &str,
    offset: u8,
    qualified_path: &str,
) -> HldNodeInputRecord {
    let target_reference = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        unit_identity,
        source_relative_path,
        u64::from(offset),
        offset,
    ));
    build_hld_fixture_node(
        generation,
        target_reference,
        HldNodeInputCategory::LocalItem,
        Some(HldItemKindValue::Function),
        Some(HldOwnerKindValue::Module),
        false,
        Some(HldEffectiveVisibilityKind::Public),
        false,
        Some(source_relative_path),
        qualified_path,
    )
}

fn build_hld_file_rollup(
    source_endpoint_bucket: HldEndpointRollupBucket,
    target_endpoint_bucket: HldEndpointRollupBucket,
    relation: HldRelationKindValue,
    via: HldViaKindValue,
    edge_count: usize,
) -> HldFileRollupRecord {
    HldFileRollupRecord {
        source_endpoint_bucket,
        target_endpoint_bucket,
        edge_relation_kind_value: relation,
        edge_collapse_via_kind: via,
        edge_row_count: edge_count,
    }
}

fn build_hld_folder_rollup(
    source_folder_bucket: HldFolderEndpointBucket,
    target_folder_bucket: HldFolderEndpointBucket,
    relation: HldRelationKindValue,
    via: HldViaKindValue,
    edge_count: usize,
) -> HldFolderRollupRecord {
    HldFolderRollupRecord {
        source_folder_bucket,
        target_folder_bucket,
        edge_relation_kind_value: relation,
        edge_collapse_via_kind: via,
        edge_row_count: edge_count,
    }
}

fn sum_hld_file_rollups(records: &[HldFileRollupRecord]) -> usize {
    records.iter().map(|record| record.edge_row_count).sum()
}

fn hld_source_file_bucket(
    unit_identity: CrateCompilationUnitKey,
    file_source_relative_path: &str,
) -> HldEndpointRollupBucket {
    HldEndpointRollupBucket::SourceFile {
        unit_compilation_identity_key: unit_identity,
        file_source_relative_path: file_source_relative_path.into(),
    }
}

fn hld_source_folder_bucket(
    unit_identity: CrateCompilationUnitKey,
    folder_source_relative_path: &str,
) -> HldFolderEndpointBucket {
    HldFolderEndpointBucket::SourceFolder {
        unit_compilation_identity_key: unit_identity,
        folder_source_relative_path: folder_source_relative_path.into(),
    }
}

fn sum_hld_folder_rollups(records: &[HldFolderRollupRecord]) -> usize {
    records.iter().map(|record| record.edge_row_count).sum()
}

fn build_hld_path_record<Source, Target, Endpoint>(
    generation: RunGenerationDigestValue,
    source: Source,
    target: Target,
    relation: HldRelationKindValue,
    via: HldViaKindValue,
    endpoints: Vec<Endpoint>,
    evidence: Vec<HldEvidenceRowKey>,
) -> HldEvidencePathRecord
where
    Source: Into<HldTargetIdentityValue>,
    Target: Into<HldTargetIdentityValue>,
    Endpoint: Into<HldTargetIdentityValue>,
{
    let sites = evidence
        .iter()
        .map(|row| row.evidence_site_occurrence_key.clone())
        .collect();
    HldEvidencePathRecord {
        edge_key: expected_hld_edge_key(generation, source, target, relation, via),
        ordered_endpoint_path: endpoints.into_iter().map(Into::into).collect(),
        ordered_evidence_row_keys: evidence,
        ordered_site_occurrence_keys: sites,
    }
}

fn build_hld_path_input(
    generation: RunGenerationDigestValue,
    unit_identity: &CrateCompilationUnitKey,
    nodes: Vec<HldNodeInputRecord>,
    edges: Vec<HldEdgeInputRecord>,
) -> HldMaterializationInputRecord {
    build_hld_fixture_input(
        generation,
        vec![HldUnitProvenanceRecord {
            unit_compilation_identity_key: unit_identity.clone(),
            unit_origin_kind: HldUnitOriginKind::Workspace,
        }],
        nodes,
        edges,
    )
}

fn collect_hld_edge_counts(
    output: &parseltongue_core_row_types::HldMaterializationOutputRecord,
) -> BTreeSet<(HldGraphEdgeKey, usize)> {
    output
        .materialized_edge_records
        .iter()
        .map(|edge| (edge.edge_key.clone(), edge.edge_evidence_row_count))
        .collect()
}

/// REQ-RUST-040.0
#[test]
fn test_hld_symbol_codec_unchanged() {
    let target_identity = HldTargetIdentityValue::from(NameSymbolTargetKey::Builtin("u8".into()));
    let encoded_identity =
        crate::hld_materialization_duckdb_store::encode_hld_target_identity(&target_identity)
            .unwrap();

    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&encoded_identity).unwrap(),
        serde_json::json!(["builtin", "u8"])
    );
}

fn build_hld_area_input(
    evidence_unit: &CrateCompilationUnitKey,
    evidence_path: &str,
    relation: HldRelationKindValue,
) -> HldMaterializationInputRecord {
    let generation = RunGenerationDigestValue([41; 32]);
    let unit = build_hld_fixture_unit("structural-area", 41);
    let path = "src/lib.rs";
    let area = HldTargetIdentityValue::FilePublicArea(
        parseltongue_core_row_types::FileAreaIdentityKey::build_file_area_identity(&unit, path)
            .unwrap(),
    );
    let definition = build_hld_fixture_occurrence(&unit, path, 20, 42);
    let item = HldTargetIdentityValue::from(NameSymbolTargetKey::Definition(definition.clone()));
    let site = build_hld_fixture_occurrence(evidence_unit, evidence_path, 20, 42);
    let area_node = HldNodeInputRecord {
        run_generation_digest_value: generation,
        node_target_reference: area.clone(),
        node_input_category: HldNodeInputCategory::FilePublicArea,
        node_item_kind_value: Some(HldItemKindValue::FilePublicArea),
        node_owner_kind_value: None,
        node_function_body_ancestry_flag: false,
        node_effective_visibility_kind: None,
        node_reachable_outside_crate_flag: None,
        node_source_relative_path: Some(path.into()),
        node_qualified_path_text: "file:src/lib.rs".into(),
        node_signature_text: None,
    };
    let mut item_node = build_hld_fixture_node(
        generation,
        NameSymbolTargetKey::Definition(definition),
        HldNodeInputCategory::LocalItem,
        Some(HldItemKindValue::Struct),
        Some(HldOwnerKindValue::Module),
        false,
        Some(HldEffectiveVisibilityKind::Public),
        false,
        Some(path),
        "crate::Item",
    );
    item_node.node_reachable_outside_crate_flag = None;
    let mut provenance = vec![HldUnitProvenanceRecord {
        unit_compilation_identity_key: unit.clone(),
        unit_origin_kind: HldUnitOriginKind::Workspace,
    }];
    if evidence_unit != &unit {
        provenance.push(HldUnitProvenanceRecord {
            unit_compilation_identity_key: evidence_unit.clone(),
            unit_origin_kind: HldUnitOriginKind::Workspace,
        });
    }
    let mut input = build_hld_fixture_input(
        generation,
        provenance,
        vec![area_node, item_node],
        vec![build_hld_fixture_edge(
            generation, area, item, relation, site,
        )],
    );
    input.source_coverage_kind = HldSourceCoverageKind::OriginalTopLevelContainment;
    input
}

/// REQ-RUST-040.0
#[test]
fn test_area_edge_materialization_roundtrip() {
    let evidence_unit = build_hld_fixture_unit("structural-area", 41);
    let input = build_hld_area_input(&evidence_unit, "src/lib.rs", HldRelationKindValue::Contains);
    let output = materialize_hld_fixture_graph(&input);
    let area = input.input_node_records[0].node_target_reference.clone();
    let item = input.input_node_records[1].node_target_reference.clone();
    let edge = output.materialized_edge_records.first().unwrap();

    assert_eq!(edge.edge_key.edge_source_occurrence_key, area);
    assert_eq!(edge.edge_key.edge_target_reference_value, item);
    assert_eq!(
        edge.edge_key.edge_relation_kind_value,
        HldRelationKindValue::Contains
    );
    assert_eq!(
        edge.edge_key.edge_collapse_via_kind,
        HldViaKindValue::Direct
    );
    assert!(
        output
            .selected_node_records
            .iter()
            .any(|node| node.node_target_reference == edge.edge_key.edge_source_occurrence_key)
    );
    assert!(
        output
            .selected_node_records
            .iter()
            .any(|node| node.node_target_reference == edge.edge_key.edge_target_reference_value)
    );
    assert!(output.evidence_path_records.iter().any(|path| {
        path.ordered_endpoint_path
            == vec![
                edge.edge_key.edge_source_occurrence_key.clone(),
                item.clone(),
            ]
            && path.ordered_evidence_row_keys
                == vec![input.input_edge_records[0].edge_evidence_row_key.clone()]
    }));
    assert_eq!(edge.edge_evidence_row_count, 1);
    let file_rollup = &output.file_rollup_records.as_ref().unwrap()[0];
    let file_bucket = HldEndpointRollupBucket::SourceFile {
        unit_compilation_identity_key: evidence_unit.clone(),
        file_source_relative_path: "src/lib.rs".into(),
    };
    assert_eq!(file_rollup.source_endpoint_bucket, file_bucket);
    assert_eq!(file_rollup.target_endpoint_bucket, file_bucket);
    assert_eq!(
        file_rollup.edge_relation_kind_value,
        HldRelationKindValue::Contains
    );
    assert_eq!(file_rollup.edge_collapse_via_kind, HldViaKindValue::Direct);
    assert_eq!(file_rollup.edge_row_count, 1);
    let folder_rollup = &output.folder_rollup_records.as_ref().unwrap()[0];
    let folder_bucket = HldFolderEndpointBucket::SourceFolder {
        unit_compilation_identity_key: evidence_unit,
        folder_source_relative_path: "src".into(),
    };
    assert_eq!(folder_rollup.source_folder_bucket, folder_bucket);
    assert_eq!(folder_rollup.target_folder_bucket, folder_bucket);
    assert_eq!(
        folder_rollup.edge_relation_kind_value,
        HldRelationKindValue::Contains
    );
    assert_eq!(
        folder_rollup.edge_collapse_via_kind,
        HldViaKindValue::Direct
    );
    assert_eq!(folder_rollup.edge_row_count, 1);
}

/// REQ-RUST-040.0
#[test]
fn test_area_evidence_path_rejected() {
    let unit = build_hld_fixture_unit("area-path", 42);
    let input = build_hld_area_input(&unit, "src/other.rs", HldRelationKindValue::Contains);
    assert_eq!(
        validate_hld_materialization_input(&input),
        Err(HldMaterializationValidationError::InvalidEvidenceIdentity)
    );
}

/// REQ-RUST-040.0
#[test]
fn test_area_evidence_unit_rejected() {
    let other_unit = build_hld_fixture_unit("area-other-unit", 43);
    let input = build_hld_area_input(&other_unit, "src/lib.rs", HldRelationKindValue::Contains);
    assert_eq!(
        validate_hld_materialization_input(&input),
        Err(HldMaterializationValidationError::InvalidEvidenceIdentity)
    );
}

/// REQ-RUST-040.0
#[test]
fn test_structural_imports_edge_rejected() {
    let unit = build_hld_fixture_unit("structural-imports", 44);
    let input = build_hld_area_input(&unit, "src/lib.rs", HldRelationKindValue::Imports);
    assert_eq!(
        validate_hld_materialization_input(&input),
        Err(HldMaterializationValidationError::StructuralScopeAllowsContainsOnly)
    );
}

/// REQ-RUST-040.0
#[test]
fn test_structural_local_source_rejected() {
    let unit = build_hld_fixture_unit("structural-local", 45);
    let mut input = build_hld_area_input(&unit, "src/lib.rs", HldRelationKindValue::Contains);
    let item = input.input_node_records[1].node_target_reference.clone();
    input.input_edge_records[0].edge_source_reference = item.clone();
    input.input_edge_records[0]
        .edge_evidence_row_key
        .evidence_source_occurrence_key = item;
    assert_eq!(
        validate_hld_materialization_input(&input),
        Err(HldMaterializationValidationError::InvalidEdgeRecord)
    );
}

/// REQ-RUST-040.0
#[test]
fn test_structural_expanded_node_rejected() {
    let unit = build_hld_fixture_unit("structural-expanded", 49);
    let mut input = build_hld_area_input(&unit, "src/lib.rs", HldRelationKindValue::Contains);
    let expanded = SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
        &unit,
        "src/lib.rs",
        SpanByteRangeValue::try_from((20, 21)).unwrap(),
        SpanDigestIdValue::from([49; 32]),
        &[[49; 32]],
    )
    .unwrap();
    let target = HldTargetIdentityValue::from(NameSymbolTargetKey::Definition(expanded.clone()));
    input.input_node_records[1].node_target_reference = target.clone();
    input.input_edge_records[0].edge_resolution_value = HldEdgeResolutionValue::Extracted {
        target: target.clone(),
    };
    input.input_edge_records[0]
        .edge_evidence_row_key
        .evidence_site_occurrence_key = expanded.clone();
    input.input_edge_records[0]
        .edge_evidence_row_key
        .evidence_target_reference_value = Some(target);
    assert_eq!(
        validate_hld_materialization_input(&input),
        Err(HldMaterializationValidationError::InvalidNodeCategory)
    );
}

/// REQ-RUST-040.0
#[test]
fn test_semantic_unknown_reachability_rejected() {
    let (mut input, _, _, _) = build_hld_validation_input();
    input.input_node_records[1].node_reachable_outside_crate_flag = None;
    assert_eq!(
        validate_hld_materialization_input(&input),
        Err(HldMaterializationValidationError::UnknownReachability)
    );
}

/// REQ-RUST-040.0
#[test]
fn test_area_signature_rejected() {
    let unit = build_hld_fixture_unit("area-signature", 46);
    let mut input = build_hld_area_input(&unit, "src/lib.rs", HldRelationKindValue::Contains);
    input.input_node_records[0].node_signature_text = Some("fake".into());
    assert_eq!(
        validate_hld_materialization_input(&input),
        Err(HldMaterializationValidationError::InvalidNodeCategory)
    );
}

/// REQ-RUST-040.0
#[test]
fn test_local_area_kind_rejected() {
    let unit = build_hld_fixture_unit("local-area-kind", 47);
    let mut input = build_hld_area_input(&unit, "src/lib.rs", HldRelationKindValue::Contains);
    input.input_node_records[1].node_item_kind_value = Some(HldItemKindValue::FilePublicArea);
    assert_eq!(
        validate_hld_materialization_input(&input),
        Err(HldMaterializationValidationError::InvalidNodeCategory)
    );
}

/// REQ-RUST-030.0 / REQ-RUST-019.1
#[test]
fn test_hld_node_visibility_rules() {
    let generation = RunGenerationDigestValue([7; 32]);
    let unit_identity = build_hld_fixture_unit("visibility-fixture", 3);
    let cases = [
        (
            "public in private parent",
            build_hld_fixture_node(
                generation,
                NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
                    &unit_identity,
                    "src/private.rs",
                    1,
                    1,
                )),
                HldNodeInputCategory::LocalItem,
                Some(HldItemKindValue::Function),
                Some(HldOwnerKindValue::Module),
                false,
                Some(HldEffectiveVisibilityKind::Public),
                false,
                Some("src/private.rs"),
                "crate::private::visible",
            ),
            true,
        ),
        (
            "crate restricted",
            build_hld_fixture_node(
                generation,
                NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
                    &unit_identity,
                    "src/module.rs",
                    2,
                    2,
                )),
                HldNodeInputCategory::LocalItem,
                Some(HldItemKindValue::Function),
                Some(HldOwnerKindValue::Module),
                false,
                Some(HldEffectiveVisibilityKind::RestrictedNonPrivate),
                false,
                Some("src/module.rs"),
                "crate::module::internal_api",
            ),
            true,
        ),
        (
            "trait inherited member",
            build_hld_fixture_node(
                generation,
                NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
                    &unit_identity,
                    "src/trait.rs",
                    3,
                    3,
                )),
                HldNodeInputCategory::LocalItem,
                Some(HldItemKindValue::AssociatedFunction),
                Some(HldOwnerKindValue::Trait),
                false,
                Some(HldEffectiveVisibilityKind::Public),
                false,
                Some("src/trait.rs"),
                "crate::HiddenTrait::member",
            ),
            true,
        ),
        (
            "private equivalent",
            build_hld_fixture_node(
                generation,
                NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
                    &unit_identity,
                    "src/module.rs",
                    4,
                    4,
                )),
                HldNodeInputCategory::LocalItem,
                Some(HldItemKindValue::Function),
                Some(HldOwnerKindValue::Module),
                false,
                Some(HldEffectiveVisibilityKind::PrivateEquivalent),
                false,
                Some("src/module.rs"),
                "crate::module::private_item",
            ),
            false,
        ),
        (
            "private use statement",
            build_hld_fixture_node(
                generation,
                NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
                    &unit_identity,
                    "src/use.rs",
                    5,
                    5,
                )),
                HldNodeInputCategory::LocalItem,
                Some(HldItemKindValue::UseStatement),
                Some(HldOwnerKindValue::Module),
                false,
                Some(HldEffectiveVisibilityKind::PrivateEquivalent),
                false,
                Some("src/use.rs"),
                "crate::private_use",
            ),
            true,
        ),
        (
            "function body item",
            build_hld_fixture_node(
                generation,
                NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
                    &unit_identity,
                    "src/body.rs",
                    5,
                    5,
                )),
                HldNodeInputCategory::LocalItem,
                Some(HldItemKindValue::Function),
                Some(HldOwnerKindValue::FunctionBody),
                true,
                Some(HldEffectiveVisibilityKind::Public),
                false,
                Some("src/body.rs"),
                "crate::function::local_item",
            ),
            false,
        ),
        (
            "impl container",
            build_hld_fixture_node(
                generation,
                NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
                    &unit_identity,
                    "src/impl.rs",
                    6,
                    6,
                )),
                HldNodeInputCategory::LocalItem,
                Some(HldItemKindValue::ImplContainer),
                Some(HldOwnerKindValue::Module),
                false,
                Some(HldEffectiveVisibilityKind::Public),
                false,
                Some("src/impl.rs"),
                "crate::Implementation",
            ),
            false,
        ),
        (
            "degree zero selected",
            build_hld_fixture_node(
                generation,
                NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
                    &unit_identity,
                    "src/unused.rs",
                    7,
                    7,
                )),
                HldNodeInputCategory::LocalItem,
                Some(HldItemKindValue::Struct),
                Some(HldOwnerKindValue::Module),
                false,
                Some(HldEffectiveVisibilityKind::Public),
                true,
                Some("src/unused.rs"),
                "crate::UnusedPublicType",
            ),
            true,
        ),
    ];
    let input = build_hld_fixture_input(
        generation,
        vec![HldUnitProvenanceRecord {
            unit_compilation_identity_key: unit_identity,
            unit_origin_kind: HldUnitOriginKind::Workspace,
        }],
        cases.iter().map(|(_, node, _)| node.clone()).collect(),
        Vec::new(),
    );

    let output = materialize_hld_fixture_graph(&input);
    let selected = output
        .selected_node_records
        .iter()
        .map(|node| node.node_target_reference.clone())
        .collect::<BTreeSet<_>>();
    let expected_reachability = cases
        .iter()
        .filter(|(_, _, selected)| *selected)
        .map(|(_, node, _)| {
            (
                node.node_target_reference.clone(),
                node.node_reachable_outside_crate_flag,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let actual_reachability = output
        .selected_node_records
        .iter()
        .map(|node| {
            (
                node.node_target_reference.clone(),
                node.node_reachable_outside_crate_flag,
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(selected, expected_reachability.keys().cloned().collect());
    assert_eq!(actual_reachability, expected_reachability);
}

/// REQ-RUST-030.0
#[test]
fn test_hld_direct_edge_projection() {
    let generation = RunGenerationDigestValue([9; 32]);
    let source_unit = build_hld_fixture_unit("source-fixture", 5);
    let dependency_unit = CrateCompilationUnitKey {
        unit_cargo_package_identifier: "registry+https://github.com/rust-lang/rust#core@0.0.0"
            .into(),
        unit_cargo_target_name: "core".into(),
        unit_cargo_target_kind: "lib".into(),
        unit_compilation_role_kind: CompilationUnitRoleKind::Target,
        unit_configuration_digest_full: [11; 32],
    };
    let standard_unit = CrateCompilationUnitKey {
        unit_cargo_package_identifier: "registry+https://example.invalid#alloc@1.0.0".into(),
        unit_cargo_target_name: "alloc".into(),
        unit_cargo_target_kind: "lib".into(),
        unit_compilation_role_kind: CompilationUnitRoleKind::Target,
        unit_configuration_digest_full: [12; 32],
    };
    let source_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &source_unit,
        "src/source.rs",
        1,
        1,
    ));
    let declaration_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &source_unit,
        "src/interface.rs",
        2,
        2,
    ));
    let implementation_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &source_unit,
        "src/implementation.rs",
        3,
        3,
    ));
    let root_key = NameSymbolTargetKey::CrateRoot(source_unit.clone());
    let builtin_key = NameSymbolTargetKey::Builtin("u8".into());
    let external_definition_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &dependency_unit,
        "src/mem/mod.rs",
        4,
        4,
    ));
    let standard_definition_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &standard_unit,
        "src/vec/mod.rs",
        5,
        5,
    ));
    assert_ne!(declaration_key, implementation_key);

    let mut nodes = vec![
        build_hld_fixture_node(
            generation,
            source_key.clone(),
            HldNodeInputCategory::LocalItem,
            Some(HldItemKindValue::Function),
            Some(HldOwnerKindValue::Module),
            false,
            Some(HldEffectiveVisibilityKind::Public),
            false,
            Some("src/source.rs"),
            "crate::source",
        ),
        build_hld_fixture_node(
            generation,
            declaration_key.clone(),
            HldNodeInputCategory::LocalItem,
            Some(HldItemKindValue::AssociatedFunction),
            Some(HldOwnerKindValue::Trait),
            false,
            Some(HldEffectiveVisibilityKind::Public),
            false,
            Some("src/interface.rs"),
            "crate::Interface::format",
        ),
        build_hld_fixture_node(
            generation,
            implementation_key.clone(),
            HldNodeInputCategory::LocalItem,
            Some(HldItemKindValue::AssociatedFunction),
            Some(HldOwnerKindValue::Impl),
            false,
            Some(HldEffectiveVisibilityKind::Public),
            false,
            Some("src/implementation.rs"),
            "crate::ConcreteImplementation::format",
        ),
        build_hld_fixture_node(
            generation,
            root_key.clone(),
            HldNodeInputCategory::CrateRoot,
            None,
            None,
            false,
            None,
            true,
            None,
            "crate",
        ),
        build_hld_fixture_node(
            generation,
            builtin_key.clone(),
            HldNodeInputCategory::Builtin,
            None,
            None,
            false,
            None,
            true,
            None,
            "u8",
        ),
        build_hld_fixture_node(
            generation,
            external_definition_key.clone(),
            HldNodeInputCategory::ExternalDefinition,
            None,
            None,
            false,
            None,
            true,
            Some("src/mem/mod.rs"),
            "core::mem::MaybeUninit::uninit",
        ),
        build_hld_fixture_node(
            generation,
            standard_definition_key.clone(),
            HldNodeInputCategory::ExternalDefinition,
            None,
            None,
            false,
            None,
            true,
            Some("src/vec/mod.rs"),
            "alloc::vec::Vec",
        ),
    ];
    nodes[0].node_signature_text = Some("pub fn source() -> u8".into());
    nodes[1].node_signature_text = Some("fn format(&self)".into());
    nodes[5].node_signature_text = Some("pub unsafe fn uninit() -> MaybeUninit<T>".into());
    let edges = vec![
        build_hld_fixture_edge(
            generation,
            source_key.clone(),
            declaration_key.clone(),
            HldRelationKindValue::Calls,
            build_hld_fixture_occurrence(&source_unit, "src/source.rs", 20, 20),
        ),
        build_hld_fixture_edge(
            generation,
            root_key.clone(),
            source_key.clone(),
            HldRelationKindValue::Contains,
            build_hld_fixture_occurrence(&source_unit, "src/source.rs", 21, 21),
        ),
        build_hld_fixture_edge(
            generation,
            source_key.clone(),
            builtin_key.clone(),
            HldRelationKindValue::UsesType,
            build_hld_fixture_occurrence(&source_unit, "src/source.rs", 22, 22),
        ),
        build_hld_fixture_edge(
            generation,
            source_key.clone(),
            external_definition_key.clone(),
            HldRelationKindValue::Calls,
            build_hld_fixture_occurrence(&source_unit, "src/source.rs", 23, 23),
        ),
        build_hld_fixture_edge(
            generation,
            source_key.clone(),
            standard_definition_key.clone(),
            HldRelationKindValue::Calls,
            build_hld_fixture_occurrence(&source_unit, "src/source.rs", 24, 24),
        ),
    ];
    let input = build_hld_fixture_input(
        generation,
        vec![
            HldUnitProvenanceRecord {
                unit_compilation_identity_key: source_unit.clone(),
                unit_origin_kind: HldUnitOriginKind::Workspace,
            },
            HldUnitProvenanceRecord {
                unit_compilation_identity_key: dependency_unit,
                unit_origin_kind: HldUnitOriginKind::ExternalCrate,
            },
            HldUnitProvenanceRecord {
                unit_compilation_identity_key: standard_unit,
                unit_origin_kind: HldUnitOriginKind::StandardLibrary,
            },
        ],
        nodes,
        edges,
    );
    let output = materialize_hld_fixture_graph(&input);

    let expected_edge_keys = [
        HldGraphEdgeKey {
            run_generation_digest_value: generation,
            edge_source_occurrence_key: source_key.clone().into(),
            edge_target_reference_value: declaration_key.clone().into(),
            edge_relation_kind_value: HldRelationKindValue::Calls,
            edge_collapse_via_kind: HldViaKindValue::Direct,
        },
        HldGraphEdgeKey {
            run_generation_digest_value: generation,
            edge_source_occurrence_key: root_key.clone().into(),
            edge_target_reference_value: source_key.clone().into(),
            edge_relation_kind_value: HldRelationKindValue::Contains,
            edge_collapse_via_kind: HldViaKindValue::Direct,
        },
        HldGraphEdgeKey {
            run_generation_digest_value: generation,
            edge_source_occurrence_key: source_key.clone().into(),
            edge_target_reference_value: builtin_key.clone().into(),
            edge_relation_kind_value: HldRelationKindValue::UsesType,
            edge_collapse_via_kind: HldViaKindValue::Direct,
        },
        HldGraphEdgeKey {
            run_generation_digest_value: generation,
            edge_source_occurrence_key: source_key.clone().into(),
            edge_target_reference_value: external_definition_key.clone().into(),
            edge_relation_kind_value: HldRelationKindValue::Calls,
            edge_collapse_via_kind: HldViaKindValue::Direct,
        },
        HldGraphEdgeKey {
            run_generation_digest_value: generation,
            edge_source_occurrence_key: source_key.clone().into(),
            edge_target_reference_value: standard_definition_key.clone().into(),
            edge_relation_kind_value: HldRelationKindValue::Calls,
            edge_collapse_via_kind: HldViaKindValue::Direct,
        },
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let actual_edge_keys = output
        .materialized_edge_records
        .iter()
        .map(|edge| edge.edge_key.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(output.materialized_edge_records.len(), 5);
    assert_eq!(actual_edge_keys, expected_edge_keys);
    assert!(output.materialized_edge_records.iter().all(|edge| {
        edge.edge_resolution_status_kind == HldOutputEdgeStatus::Extracted
            && edge.edge_evidence_row_count == 1
    }));
    assert_eq!(output.evidence_path_records.len(), 5);
    assert!(output.evidence_path_records.iter().all(|path| {
        path.ordered_evidence_row_keys.len() == 1 && path.ordered_site_occurrence_keys.len() == 1
    }));
    let path_edge_keys = output
        .evidence_path_records
        .iter()
        .map(|path| path.edge_key.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(path_edge_keys, expected_edge_keys);
    assert_eq!(
        output.output_completeness_kind,
        HldOutputCompletenessKind::CompleteGraph
    );
    assert!(
        output
            .selected_node_records
            .iter()
            .any(|node| node.node_target_reference == implementation_key)
    );
    assert!(!output.selected_node_records.iter().any(|node| {
        node.node_target_reference == external_definition_key
            || node.node_target_reference == standard_definition_key
            || node.node_target_reference == builtin_key
    }));
    assert!(
        !actual_edge_keys
            .iter()
            .any(|key| { key.edge_target_reference_value == implementation_key })
    );

    let edge_origins = output
        .materialized_edge_records
        .iter()
        .map(|edge| {
            (
                edge.edge_key.edge_target_reference_value.clone(),
                edge.edge_dependency_origin_kind,
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        edge_origins,
        [
            (declaration_key.clone(), HldDependencyOriginKind::SameCrate),
            (source_key.clone(), HldDependencyOriginKind::SameCrate),
            (
                builtin_key.clone(),
                HldDependencyOriginKind::StandardLibrary
            ),
            (
                external_definition_key.clone(),
                HldDependencyOriginKind::ExternalCrate,
            ),
            (
                standard_definition_key.clone(),
                HldDependencyOriginKind::StandardLibrary,
            ),
        ]
        .into_iter()
        .map(|(target, origin)| (target.into(), origin))
        .collect()
    );

    let external_call = output
        .materialized_edge_records
        .iter()
        .find(|edge| edge.edge_key.edge_target_reference_value == external_definition_key)
        .unwrap();
    assert_eq!(external_call.edge_source_qualified_path, "crate::source");
    assert_eq!(
        external_call.edge_target_qualified_path,
        "core::mem::MaybeUninit::uninit"
    );
    assert_eq!(
        external_call.edge_source_signature_text.as_deref(),
        Some("pub fn source() -> u8")
    );
    assert_eq!(
        external_call.edge_target_signature_text.as_deref(),
        Some("pub unsafe fn uninit() -> MaybeUninit<T>")
    );
    assert_eq!(
        external_call.edge_dependency_origin_kind,
        HldDependencyOriginKind::ExternalCrate
    );

    let standard_call = output
        .materialized_edge_records
        .iter()
        .find(|edge| edge.edge_key.edge_target_reference_value == standard_definition_key)
        .unwrap();
    assert_eq!(
        standard_call.edge_dependency_origin_kind,
        HldDependencyOriginKind::StandardLibrary
    );
}

/// REQ-RUST-030.0
#[test]
fn test_hld_private_path_materialized() {
    let generation = RunGenerationDigestValue([13; 32]);
    let unit_identity = build_hld_fixture_unit("private-path-fixture", 13);
    let source_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &unit_identity,
        "src/source.rs",
        1,
        1,
    ));
    let private_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &unit_identity,
        "src/private.rs",
        2,
        2,
    ));
    let target_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &unit_identity,
        "src/target.rs",
        3,
        3,
    ));
    let nodes = vec![
        build_hld_fixture_node(
            generation,
            source_key.clone(),
            HldNodeInputCategory::LocalItem,
            Some(HldItemKindValue::Function),
            Some(HldOwnerKindValue::Module),
            false,
            Some(HldEffectiveVisibilityKind::Public),
            false,
            Some("src/source.rs"),
            "crate::source",
        ),
        build_hld_fixture_node(
            generation,
            private_key.clone(),
            HldNodeInputCategory::LocalItem,
            Some(HldItemKindValue::Function),
            Some(HldOwnerKindValue::Module),
            false,
            Some(HldEffectiveVisibilityKind::PrivateEquivalent),
            false,
            Some("src/private.rs"),
            "crate::private::helper",
        ),
        build_hld_fixture_node(
            generation,
            target_key.clone(),
            HldNodeInputCategory::LocalItem,
            Some(HldItemKindValue::Function),
            Some(HldOwnerKindValue::Module),
            false,
            Some(HldEffectiveVisibilityKind::Public),
            false,
            Some("src/target.rs"),
            "crate::target",
        ),
    ];
    let mut input = build_hld_fixture_input(
        generation,
        vec![HldUnitProvenanceRecord {
            unit_compilation_identity_key: unit_identity.clone(),
            unit_origin_kind: HldUnitOriginKind::Workspace,
        }],
        nodes,
        vec![
            build_hld_fixture_edge(
                generation,
                source_key.clone(),
                private_key.clone(),
                HldRelationKindValue::Calls,
                build_hld_fixture_occurrence(&unit_identity, "src/source.rs", 20, 20),
            ),
            build_hld_fixture_edge(
                generation,
                private_key,
                target_key,
                HldRelationKindValue::UsesType,
                build_hld_fixture_occurrence(&unit_identity, "src/private.rs", 21, 21),
            ),
        ],
    );
    input.input_node_records[0].node_signature_text = Some("fn source()".into());
    input.input_node_records[2].node_signature_text = Some("fn target()".into());
    let output = materialize_hld_fixture_graph(&input);
    let source_key = input.input_node_records[0].node_target_reference.clone();
    let private_key = input.input_node_records[1].node_target_reference.clone();
    let target_key = input.input_node_records[2].node_target_reference.clone();
    let expected_key = expected_hld_edge_key(
        generation,
        source_key.clone(),
        target_key.clone(),
        HldRelationKindValue::UsesType,
        HldViaKindValue::Collapsed,
    );
    assert_eq!(
        output.output_completeness_kind,
        HldOutputCompletenessKind::CompleteGraph
    );
    assert_eq!(output.materialized_edge_records.len(), 1);
    let edge = &output.materialized_edge_records[0];
    assert_eq!(edge.edge_key, expected_key);
    assert_eq!(edge.edge_evidence_row_count, 2);
    assert_eq!(edge.edge_source_qualified_path, "crate::source");
    assert_eq!(edge.edge_target_qualified_path, "crate::target");
    assert_eq!(
        edge.edge_source_signature_text.as_deref(),
        Some("fn source()")
    );
    assert_eq!(
        edge.edge_target_signature_text.as_deref(),
        Some("fn target()")
    );
    assert_eq!(output.evidence_path_records.len(), 1);
    assert_eq!(
        output.evidence_path_records[0],
        build_hld_path_record(
            generation,
            source_key,
            target_key,
            HldRelationKindValue::UsesType,
            HldViaKindValue::Collapsed,
            vec![
                input.input_node_records[0].node_target_reference.clone(),
                private_key,
                input.input_node_records[2].node_target_reference.clone(),
            ],
            input
                .input_edge_records
                .iter()
                .map(|edge| edge.edge_evidence_row_key.clone())
                .collect(),
        ),
    );
}

fn build_hld_validation_input() -> (
    HldMaterializationInputRecord,
    CrateCompilationUnitKey,
    NameSymbolTargetKey,
    NameSymbolTargetKey,
) {
    let generation = RunGenerationDigestValue([21; 32]);
    let unit_identity = build_hld_fixture_unit("validation-fixture", 21);
    let root_key = NameSymbolTargetKey::CrateRoot(unit_identity.clone());
    let source_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &unit_identity,
        "src/source.rs",
        1,
        1,
    ));
    let target_key = NameSymbolTargetKey::Builtin("u8".into());
    let input = build_hld_fixture_input(
        generation,
        vec![HldUnitProvenanceRecord {
            unit_compilation_identity_key: unit_identity.clone(),
            unit_origin_kind: HldUnitOriginKind::Workspace,
        }],
        vec![
            build_hld_fixture_node(
                generation,
                root_key,
                HldNodeInputCategory::CrateRoot,
                None,
                None,
                false,
                None,
                false,
                None,
                "crate",
            ),
            build_hld_fixture_node(
                generation,
                source_key.clone(),
                HldNodeInputCategory::LocalItem,
                Some(HldItemKindValue::Function),
                Some(HldOwnerKindValue::Module),
                false,
                Some(HldEffectiveVisibilityKind::Public),
                false,
                Some("src/source.rs"),
                "crate::source",
            ),
            build_hld_fixture_node(
                generation,
                target_key.clone(),
                HldNodeInputCategory::Builtin,
                None,
                None,
                false,
                None,
                false,
                None,
                "u8",
            ),
        ],
        vec![build_hld_fixture_edge(
            generation,
            source_key.clone(),
            target_key.clone(),
            HldRelationKindValue::Calls,
            build_hld_fixture_occurrence(&unit_identity, "src/callsite.rs", 8, 8),
        )],
    );
    (input, unit_identity, source_key, target_key)
}

fn set_hld_builtin_target(input: &mut HldMaterializationInputRecord, name: &str) {
    let target = NameSymbolTargetKey::Builtin(name.into());
    input.input_node_records[2].node_target_reference = target.clone().into();
    input.input_node_records[2].node_qualified_path_text = name.into();
    input.input_edge_records[0].edge_resolution_value = HldEdgeResolutionValue::Extracted {
        target: target.clone().into(),
    };
    input.input_edge_records[0]
        .edge_evidence_row_key
        .evidence_target_reference_value = Some(target.into());
}

fn collect_hld_validation_failures(
    cases: Vec<(
        &str,
        HldMaterializationInputRecord,
        HldMaterializationValidationError,
    )>,
) -> Vec<String> {
    cases
        .into_iter()
        .filter_map(|(case_name, input, expected_error)| {
            let result = validate_hld_materialization_input(&input);
            (result != Err(expected_error)).then(|| format!("{case_name}: {result:?}"))
        })
        .collect()
}

/// REQ-RUST-030.0
#[test]
fn test_hld_identity_validation_rules() {
    let (base_input, unit_identity, _, _) = build_hld_validation_input();
    let supported_builtins = [
        "bool", "char", "str", "usize", "isize", "u8", "u16", "u32", "u64", "u128", "i8", "i16",
        "i32", "i64", "i128", "f16", "f32", "f64", "f128",
    ];
    for builtin_name in supported_builtins {
        let mut input = base_input.clone();
        set_hld_builtin_target(&mut input, builtin_name);
        assert_eq!(validate_hld_materialization_input(&input), Ok(()));
    }

    let mut invalid_cases = Vec::new();
    for builtin_name in ["not_a_primitive", "u8 ", "MaybeUninit", "uninit"] {
        let mut input = base_input.clone();
        set_hld_builtin_target(&mut input, builtin_name);
        invalid_cases.push((
            "invalid builtin identity",
            input,
            HldMaterializationValidationError::MalformedBuiltinIdentity,
        ));
    }
    let mut namespaced_builtin = base_input.clone();
    set_hld_builtin_target(&mut namespaced_builtin, "core::mem::MaybeUninit::uninit");
    invalid_cases.push((
        "ordinary core definition as builtin",
        namespaced_builtin,
        HldMaterializationValidationError::MalformedBuiltinIdentity,
    ));

    let external_same_unit = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &unit_identity,
        "src/external.rs",
        22,
        22,
    ));
    let mut external_masquerade = base_input.clone();
    external_masquerade
        .input_node_records
        .push(build_hld_fixture_node(
            external_masquerade.run_generation_digest_value,
            external_same_unit,
            HldNodeInputCategory::ExternalDefinition,
            None,
            None,
            false,
            None,
            false,
            None,
            "external::same_unit",
        ));
    invalid_cases.push((
        "external definition in local unit",
        external_masquerade,
        HldMaterializationValidationError::InvalidNodeCategory,
    ));

    let mut function_body_owner = base_input.clone();
    function_body_owner.input_node_records[1].node_owner_kind_value =
        Some(HldOwnerKindValue::FunctionBody);
    invalid_cases.push((
        "function body owner without ancestry",
        function_body_owner,
        HldMaterializationValidationError::InvalidNodeCategory,
    ));
    let mut unknown_owner = base_input.clone();
    unknown_owner.input_node_records[1].node_owner_kind_value = Some(HldOwnerKindValue::Other);
    invalid_cases.push((
        "unknown owner kind",
        unknown_owner,
        HldMaterializationValidationError::InvalidNodeCategory,
    ));
    for item_kind in [HldItemKindValue::Field, HldItemKindValue::EnumVariant] {
        let mut unsupported_owner = base_input.clone();
        unsupported_owner.input_node_records[1].node_item_kind_value = Some(item_kind);
        invalid_cases.push((
            "unsupported item owner semantics",
            unsupported_owner,
            HldMaterializationValidationError::InvalidNodeCategory,
        ));
    }
    for (node_index, expected_error) in [
        (0, HldMaterializationValidationError::InvalidNodeCategory),
        (
            2,
            HldMaterializationValidationError::MalformedBuiltinIdentity,
        ),
    ] {
        let mut invalid_ancestry = base_input.clone();
        invalid_ancestry.input_node_records[node_index].node_function_body_ancestry_flag = true;
        invalid_cases.push((
            "nonlocal endpoint has body ancestry",
            invalid_ancestry,
            expected_error,
        ));
    }
    let external_unit = build_hld_fixture_unit("other-workspace", 22);
    assert_ne!(external_unit, unit_identity);
    let mut external_workspace_input = base_input.clone();
    external_workspace_input
        .input_unit_provenance_records
        .push(HldUnitProvenanceRecord {
            unit_compilation_identity_key: external_unit.clone(),
            unit_origin_kind: HldUnitOriginKind::Workspace,
        });
    external_workspace_input
        .input_node_records
        .push(build_hld_fixture_node(
            external_workspace_input.run_generation_digest_value,
            NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
                &external_unit,
                "src/dependency.rs",
                23,
                23,
            )),
            HldNodeInputCategory::ExternalDefinition,
            None,
            None,
            false,
            None,
            false,
            None,
            "dependency::external",
        ));
    assert_eq!(
        validate_hld_materialization_input(&external_workspace_input),
        Ok(())
    );
    let mut external_body_ancestry = external_workspace_input.clone();
    external_body_ancestry.input_node_records[3].node_function_body_ancestry_flag = true;
    invalid_cases.push((
        "external definition has body ancestry",
        external_body_ancestry,
        HldMaterializationValidationError::InvalidNodeCategory,
    ));

    let mut nested_module = base_input.clone();
    nested_module.input_edge_records.clear();
    nested_module.input_node_records[1].node_item_kind_value = Some(HldItemKindValue::Module);
    nested_module.input_node_records[1].node_function_body_ancestry_flag = true;
    assert_eq!(validate_hld_materialization_input(&nested_module), Ok(()));
    let nested_output = materialize_hld_fixture_graph(&nested_module);
    assert!(
        !nested_output
            .selected_node_records
            .iter()
            .any(|node| node.node_target_reference
                == nested_module.input_node_records[1].node_target_reference)
    );

    let failures = collect_hld_validation_failures(invalid_cases);
    assert!(failures.is_empty(), "{failures:?}");
}

/// REQ-RUST-030.0
#[test]
fn test_hld_invalid_input_rejection() {
    let (base_input, unit_identity, source_key, target_key) = build_hld_validation_input();
    let mut cases = Vec::new();
    let mut incomplete = base_input.clone();
    incomplete.semantic_input_completeness_kind = HldInputCompletenessKind::IncompleteTypedInput;
    cases.push((
        "incomplete input",
        incomplete,
        HldMaterializationValidationError::IncompleteTypedInput,
    ));

    let mut generation_mismatch = base_input.clone();
    generation_mismatch.input_node_records[1].run_generation_digest_value =
        RunGenerationDigestValue([99; 32]);
    cases.push((
        "mismatched generation",
        generation_mismatch,
        HldMaterializationValidationError::GenerationMismatch,
    ));

    let missing_endpoint = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &unit_identity,
        "src/missing.rs",
        30,
        30,
    ));
    let mut missing_target = base_input.clone();
    missing_target.input_edge_records[0].edge_resolution_value =
        HldEdgeResolutionValue::Extracted {
            target: missing_endpoint.clone().into(),
        };
    missing_target.input_edge_records[0]
        .edge_evidence_row_key
        .evidence_target_reference_value = Some(missing_endpoint.into());
    cases.push((
        "missing endpoint",
        missing_target,
        HldMaterializationValidationError::MissingLocalEndpoint,
    ));

    let mut missing_provenance = base_input.clone();
    missing_provenance.input_unit_provenance_records.clear();
    cases.push((
        "missing unit provenance",
        missing_provenance,
        HldMaterializationValidationError::MissingUnitProvenance,
    ));

    let mut invalid_evidence = base_input.clone();
    invalid_evidence.input_edge_records[0]
        .edge_evidence_row_key
        .evidence_relation_kind_value = HldRelationKindValue::Derives;
    cases.push((
        "invalid evidence identity",
        invalid_evidence,
        HldMaterializationValidationError::InvalidEvidenceIdentity,
    ));

    let other_unit = build_hld_fixture_unit("other-evidence-unit", 24);
    let mut cross_unit_site = base_input.clone();
    cross_unit_site
        .input_unit_provenance_records
        .push(HldUnitProvenanceRecord {
            unit_compilation_identity_key: other_unit.clone(),
            unit_origin_kind: HldUnitOriginKind::Workspace,
        });
    cross_unit_site.input_edge_records[0]
        .edge_evidence_row_key
        .evidence_site_occurrence_key =
        build_hld_fixture_occurrence(&other_unit, "src/other.rs", 24, 24);
    cases.push((
        "cross-unit evidence site",
        cross_unit_site,
        HldMaterializationValidationError::InvalidEvidenceIdentity,
    ));

    let failures = collect_hld_validation_failures(cases);
    assert!(failures.is_empty(), "{failures:?}");
    assert_eq!(
        source_key,
        base_input.input_edge_records[0].edge_source_reference
    );

    for (resolution, evidence_target) in [
        (
            HldEdgeResolutionValue::Inferred {
                candidate_target: Some(target_key.clone().into()),
            },
            Some(target_key.clone()),
        ),
        (
            HldEdgeResolutionValue::Proposed {
                candidate_target: Some(target_key.clone().into()),
            },
            Some(target_key.clone()),
        ),
        (
            HldEdgeResolutionValue::Unresolved {
                reason: HldUnresolvedReasonKind::TraitSolverNeeded,
            },
            None,
        ),
    ] {
        let mut input = base_input.clone();
        input.input_edge_records[0].edge_resolution_value = resolution.clone();
        input.input_edge_records[0]
            .edge_evidence_row_key
            .evidence_target_reference_value = evidence_target.map(Into::into);
        let output = materialize_hld_fixture_graph(&input);
        assert!(output.materialized_edge_records.is_empty());
        assert_eq!(
            input.input_edge_records[0].edge_resolution_value,
            resolution
        );
    }
}

/// REQ-RUST-030.0
#[test]
fn test_hld_duplicate_conflict_rejection() {
    let (base_input, _, _, _) = build_hld_validation_input();
    let mut identical_duplicates = base_input.clone();
    identical_duplicates
        .input_unit_provenance_records
        .push(identical_duplicates.input_unit_provenance_records[0].clone());
    identical_duplicates
        .input_node_records
        .push(identical_duplicates.input_node_records[1].clone());
    identical_duplicates
        .input_edge_records
        .push(identical_duplicates.input_edge_records[0].clone());
    assert_eq!(
        validate_hld_materialization_input(&identical_duplicates),
        Ok(())
    );

    let mut conflicting_unit = base_input.clone();
    conflicting_unit
        .input_unit_provenance_records
        .push(HldUnitProvenanceRecord {
            unit_compilation_identity_key: conflicting_unit.input_unit_provenance_records[0]
                .unit_compilation_identity_key
                .clone(),
            unit_origin_kind: HldUnitOriginKind::ExternalCrate,
        });
    assert_eq!(
        validate_hld_materialization_input(&conflicting_unit),
        Err(HldMaterializationValidationError::ConflictingUnitProvenance)
    );

    let mut conflicting_node = base_input.clone();
    let mut node_conflict = conflicting_node.input_node_records[1].clone();
    node_conflict.node_signature_text = Some("different signature".into());
    conflicting_node.input_node_records.push(node_conflict);
    assert_eq!(
        validate_hld_materialization_input(&conflicting_node),
        Err(HldMaterializationValidationError::ConflictingNodeRecord)
    );

    let mut conflicting_evidence = base_input.clone();
    let mut evidence_conflict = conflicting_evidence.input_edge_records[0].clone();
    evidence_conflict.edge_dispatch_metadata_text = Some("different dispatch".into());
    conflicting_evidence
        .input_edge_records
        .push(evidence_conflict);
    assert_eq!(
        validate_hld_materialization_input(&conflicting_evidence),
        Err(HldMaterializationValidationError::ConflictingEvidenceRecord)
    );

    let mut same_site_distinct_relation = base_input;
    let mut second_relation = same_site_distinct_relation.input_edge_records[0].clone();
    second_relation.edge_relation_kind_value = HldRelationKindValue::UsesType;
    second_relation
        .edge_evidence_row_key
        .evidence_relation_kind_value = HldRelationKindValue::UsesType;
    same_site_distinct_relation
        .input_edge_records
        .push(second_relation);
    assert_eq!(
        validate_hld_materialization_input(&same_site_distinct_relation),
        Ok(())
    );
}

/// REQ-RUST-030.0
#[test]
fn test_hld_private_terminal_kind() {
    let generation = RunGenerationDigestValue([31; 32]);
    let unit_identity = build_hld_fixture_unit("private-terminal-fixture", 31);
    let source = build_hld_path_node(
        generation,
        &unit_identity,
        "source",
        1,
        HldEffectiveVisibilityKind::Public,
        "crate::source",
    );
    let private = build_hld_path_node(
        generation,
        &unit_identity,
        "private",
        2,
        HldEffectiveVisibilityKind::PrivateEquivalent,
        "crate::private::helper",
    );
    let target = build_hld_path_node(
        generation,
        &unit_identity,
        "target",
        3,
        HldEffectiveVisibilityKind::Public,
        "crate::target",
    );
    let source_key = source.node_target_reference.clone();
    let private_key = private.node_target_reference.clone();
    let target_key = target.node_target_reference.clone();
    let input = build_hld_path_input(
        generation,
        &unit_identity,
        vec![source, private, target],
        vec![
            build_hld_path_edge(
                generation,
                &unit_identity,
                &source_key,
                &private_key,
                HldRelationKindValue::Calls,
                "callsite-a",
                31,
            ),
            build_hld_path_edge(
                generation,
                &unit_identity,
                &private_key,
                &target_key,
                HldRelationKindValue::UsesType,
                "callsite-private",
                32,
            ),
        ],
    );
    let output = materialize_hld_fixture_graph(&input);
    let expected_key = expected_hld_edge_key(
        generation,
        source_key.clone(),
        target_key.clone(),
        HldRelationKindValue::UsesType,
        HldViaKindValue::Collapsed,
    );
    assert_eq!(
        collect_hld_edge_counts(&output),
        BTreeSet::from([(expected_key, 2)]),
    );
    assert_eq!(
        output.materialized_edge_records[0].edge_dependency_origin_kind,
        HldDependencyOriginKind::SameCrate,
    );
    assert_eq!(
        output.evidence_path_records,
        vec![build_hld_path_record(
            generation,
            source_key.clone(),
            target_key.clone(),
            HldRelationKindValue::UsesType,
            HldViaKindValue::Collapsed,
            vec![source_key, private_key, target_key],
            input
                .input_edge_records
                .iter()
                .map(|edge| edge.edge_evidence_row_key.clone())
                .collect(),
        )],
    );
}

/// REQ-RUST-030.0
#[test]
fn test_hld_selected_target_stop() {
    let generation = RunGenerationDigestValue([32; 32]);
    let unit_identity = build_hld_fixture_unit("selected-stop-fixture", 32);
    let source = build_hld_path_node(
        generation,
        &unit_identity,
        "a",
        1,
        HldEffectiveVisibilityKind::Public,
        "crate::a",
    );
    let private = build_hld_path_node(
        generation,
        &unit_identity,
        "p",
        2,
        HldEffectiveVisibilityKind::PrivateEquivalent,
        "crate::private::p",
    );
    let selected_b = build_hld_path_node(
        generation,
        &unit_identity,
        "b",
        3,
        HldEffectiveVisibilityKind::Public,
        "crate::b",
    );
    let selected_c = build_hld_path_node(
        generation,
        &unit_identity,
        "c",
        4,
        HldEffectiveVisibilityKind::Public,
        "crate::c",
    );
    let source_key = source.node_target_reference.clone();
    let private_key = private.node_target_reference.clone();
    let b_key = selected_b.node_target_reference.clone();
    let c_key = selected_c.node_target_reference.clone();
    let input = build_hld_path_input(
        generation,
        &unit_identity,
        vec![source, private, selected_b, selected_c],
        vec![
            build_hld_path_edge(
                generation,
                &unit_identity,
                &source_key,
                &private_key,
                HldRelationKindValue::Calls,
                "a-p",
                41,
            ),
            build_hld_path_edge(
                generation,
                &unit_identity,
                &private_key,
                &b_key,
                HldRelationKindValue::UsesType,
                "p-b",
                42,
            ),
            build_hld_path_edge(
                generation,
                &unit_identity,
                &b_key,
                &c_key,
                HldRelationKindValue::Implements,
                "b-c",
                43,
            ),
        ],
    );
    let output = materialize_hld_fixture_graph(&input);
    let expected_edges = BTreeSet::from([
        (
            expected_hld_edge_key(
                generation,
                source_key.clone(),
                b_key.clone(),
                HldRelationKindValue::UsesType,
                HldViaKindValue::Collapsed,
            ),
            2,
        ),
        (
            expected_hld_edge_key(
                generation,
                b_key.clone(),
                c_key.clone(),
                HldRelationKindValue::Implements,
                HldViaKindValue::Direct,
            ),
            1,
        ),
    ]);
    assert_eq!(collect_hld_edge_counts(&output), expected_edges);
    assert!(!output.materialized_edge_records.iter().any(|edge| {
        edge.edge_key.edge_source_occurrence_key == source_key
            && edge.edge_key.edge_target_reference_value == c_key
    }));
    let mut expected_paths = vec![
        build_hld_path_record(
            generation,
            source_key.clone(),
            b_key.clone(),
            HldRelationKindValue::UsesType,
            HldViaKindValue::Collapsed,
            vec![source_key.clone(), private_key, b_key.clone()],
            input.input_edge_records[..2]
                .iter()
                .map(|edge| edge.edge_evidence_row_key.clone())
                .collect(),
        ),
        build_hld_path_record(
            generation,
            b_key.clone(),
            c_key.clone(),
            HldRelationKindValue::Implements,
            HldViaKindValue::Direct,
            vec![b_key, c_key],
            vec![input.input_edge_records[2].edge_evidence_row_key.clone()],
        ),
    ];
    expected_paths.sort();
    assert_eq!(output.evidence_path_records, expected_paths);
}

/// REQ-RUST-030.0 / REQ-RUST-009.2
#[test]
fn test_hld_diamond_evidence_union() {
    let generation = RunGenerationDigestValue([33; 32]);
    let unit_identity = build_hld_fixture_unit("diamond-fixture", 33);
    let a = build_hld_path_node(
        generation,
        &unit_identity,
        "a",
        1,
        HldEffectiveVisibilityKind::Public,
        "crate::a",
    );
    let p = build_hld_path_node(
        generation,
        &unit_identity,
        "p",
        2,
        HldEffectiveVisibilityKind::PrivateEquivalent,
        "crate::private::p",
    );
    let q = build_hld_path_node(
        generation,
        &unit_identity,
        "q",
        3,
        HldEffectiveVisibilityKind::PrivateEquivalent,
        "crate::private::q",
    );
    let r = build_hld_path_node(
        generation,
        &unit_identity,
        "r",
        4,
        HldEffectiveVisibilityKind::PrivateEquivalent,
        "crate::private::r",
    );
    let b = build_hld_path_node(
        generation,
        &unit_identity,
        "b",
        5,
        HldEffectiveVisibilityKind::Public,
        "crate::b",
    );
    let a_key = a.node_target_reference.clone();
    let p_key = p.node_target_reference.clone();
    let q_key = q.node_target_reference.clone();
    let r_key = r.node_target_reference.clone();
    let b_key = b.node_target_reference.clone();
    let edges = vec![
        build_hld_path_edge(
            generation,
            &unit_identity,
            &a_key,
            &p_key,
            HldRelationKindValue::Calls,
            "a-p",
            51,
        ),
        build_hld_path_edge(
            generation,
            &unit_identity,
            &p_key,
            &q_key,
            HldRelationKindValue::Calls,
            "p-q",
            52,
        ),
        build_hld_path_edge(
            generation,
            &unit_identity,
            &q_key,
            &b_key,
            HldRelationKindValue::UsesType,
            "q-b",
            53,
        ),
        build_hld_path_edge(
            generation,
            &unit_identity,
            &p_key,
            &r_key,
            HldRelationKindValue::Calls,
            "p-r",
            54,
        ),
        build_hld_path_edge(
            generation,
            &unit_identity,
            &r_key,
            &b_key,
            HldRelationKindValue::UsesType,
            "r-b",
            55,
        ),
        build_hld_path_edge(
            generation,
            &unit_identity,
            &a_key,
            &b_key,
            HldRelationKindValue::UsesType,
            "direct-one",
            56,
        ),
        build_hld_path_edge(
            generation,
            &unit_identity,
            &a_key,
            &b_key,
            HldRelationKindValue::UsesType,
            "direct-two",
            57,
        ),
        build_hld_path_edge(
            generation,
            &unit_identity,
            &p_key,
            &q_key,
            HldRelationKindValue::Calls,
            "p-q-second-proof",
            58,
        ),
    ];
    let input = build_hld_path_input(generation, &unit_identity, vec![a, p, q, r, b], edges);
    let output = materialize_hld_fixture_graph(&input);
    let expected_edges = BTreeSet::from([
        (
            expected_hld_edge_key(
                generation,
                a_key.clone(),
                b_key.clone(),
                HldRelationKindValue::UsesType,
                HldViaKindValue::Collapsed,
            ),
            6,
        ),
        (
            expected_hld_edge_key(
                generation,
                a_key.clone(),
                b_key.clone(),
                HldRelationKindValue::UsesType,
                HldViaKindValue::Direct,
            ),
            2,
        ),
    ]);
    assert_eq!(collect_hld_edge_counts(&output), expected_edges);
    let mut expected_paths = vec![
        build_hld_path_record(
            generation,
            a_key.clone(),
            b_key.clone(),
            HldRelationKindValue::UsesType,
            HldViaKindValue::Collapsed,
            vec![a_key.clone(), p_key.clone(), q_key.clone(), b_key.clone()],
            vec![
                input.input_edge_records[0].edge_evidence_row_key.clone(),
                input.input_edge_records[1].edge_evidence_row_key.clone(),
                input.input_edge_records[2].edge_evidence_row_key.clone(),
            ],
        ),
        build_hld_path_record(
            generation,
            a_key.clone(),
            b_key.clone(),
            HldRelationKindValue::UsesType,
            HldViaKindValue::Collapsed,
            vec![a_key.clone(), p_key.clone(), q_key.clone(), b_key.clone()],
            vec![
                input.input_edge_records[0].edge_evidence_row_key.clone(),
                input.input_edge_records[7].edge_evidence_row_key.clone(),
                input.input_edge_records[2].edge_evidence_row_key.clone(),
            ],
        ),
        build_hld_path_record(
            generation,
            a_key.clone(),
            b_key.clone(),
            HldRelationKindValue::UsesType,
            HldViaKindValue::Collapsed,
            vec![a_key.clone(), p_key.clone(), r_key.clone(), b_key.clone()],
            vec![
                input.input_edge_records[0].edge_evidence_row_key.clone(),
                input.input_edge_records[3].edge_evidence_row_key.clone(),
                input.input_edge_records[4].edge_evidence_row_key.clone(),
            ],
        ),
        build_hld_path_record(
            generation,
            a_key.clone(),
            b_key.clone(),
            HldRelationKindValue::UsesType,
            HldViaKindValue::Direct,
            vec![a_key.clone(), b_key.clone()],
            vec![input.input_edge_records[5].edge_evidence_row_key.clone()],
        ),
        build_hld_path_record(
            generation,
            a_key.clone(),
            b_key.clone(),
            HldRelationKindValue::UsesType,
            HldViaKindValue::Direct,
            vec![a_key.clone(), b_key.clone()],
            vec![input.input_edge_records[6].edge_evidence_row_key.clone()],
        ),
    ];
    expected_paths.sort();
    assert_eq!(output.evidence_path_records, expected_paths);
    let direct_paths = output
        .evidence_path_records
        .iter()
        .filter(|path| path.edge_key.edge_collapse_via_kind == HldViaKindValue::Direct)
        .collect::<Vec<_>>();
    assert_eq!(direct_paths.len(), 2);
    assert_eq!(
        direct_paths[0].ordered_endpoint_path,
        direct_paths[1].ordered_endpoint_path
    );
    assert_ne!(
        direct_paths[0].ordered_evidence_row_keys,
        direct_paths[1].ordered_evidence_row_keys
    );
    let collapsed_same_endpoint_paths = output
        .evidence_path_records
        .iter()
        .filter(|path| {
            path.edge_key.edge_collapse_via_kind == HldViaKindValue::Collapsed
                && path.ordered_endpoint_path
                    == vec![a_key.clone(), p_key.clone(), q_key.clone(), b_key.clone()]
        })
        .collect::<Vec<_>>();
    assert_eq!(collapsed_same_endpoint_paths.len(), 2);
    assert_ne!(
        collapsed_same_endpoint_paths[0].ordered_evidence_row_keys,
        collapsed_same_endpoint_paths[1].ordered_evidence_row_keys
    );
}

/// REQ-RUST-040.0
#[test]
fn test_hld_recursive_edge_guards() {
    let generation = RunGenerationDigestValue([48; 32]);
    let unit = build_hld_fixture_unit("recursive-guards", 48);
    let nodes = [
        ("contains-source", 1, HldEffectiveVisibilityKind::Public),
        (
            "contains-private",
            2,
            HldEffectiveVisibilityKind::PrivateEquivalent,
        ),
        ("contains-target", 3, HldEffectiveVisibilityKind::Public),
        ("imports-source", 4, HldEffectiveVisibilityKind::Public),
        (
            "imports-private",
            5,
            HldEffectiveVisibilityKind::PrivateEquivalent,
        ),
        ("imports-target", 6, HldEffectiveVisibilityKind::Public),
        ("contains-hop-source", 7, HldEffectiveVisibilityKind::Public),
        (
            "contains-hop-private",
            8,
            HldEffectiveVisibilityKind::PrivateEquivalent,
        ),
        (
            "contains-hop-next",
            9,
            HldEffectiveVisibilityKind::PrivateEquivalent,
        ),
        (
            "contains-hop-target",
            10,
            HldEffectiveVisibilityKind::Public,
        ),
        ("imports-hop-source", 11, HldEffectiveVisibilityKind::Public),
        (
            "imports-hop-private",
            12,
            HldEffectiveVisibilityKind::PrivateEquivalent,
        ),
        (
            "imports-hop-next",
            13,
            HldEffectiveVisibilityKind::PrivateEquivalent,
        ),
        ("imports-hop-target", 14, HldEffectiveVisibilityKind::Public),
        ("reads-source", 15, HldEffectiveVisibilityKind::Public),
        (
            "reads-private",
            16,
            HldEffectiveVisibilityKind::PrivateEquivalent,
        ),
        ("reads-target", 17, HldEffectiveVisibilityKind::Public),
        (
            "direct-contains-source",
            18,
            HldEffectiveVisibilityKind::Public,
        ),
        (
            "direct-contains-target",
            19,
            HldEffectiveVisibilityKind::Public,
        ),
        (
            "direct-imports-source",
            20,
            HldEffectiveVisibilityKind::Public,
        ),
        (
            "direct-imports-target",
            21,
            HldEffectiveVisibilityKind::Public,
        ),
    ]
    .into_iter()
    .map(|(name, offset, visibility)| {
        build_hld_path_node(generation, &unit, name, offset, visibility, name)
    })
    .collect::<Vec<_>>();
    let keys = nodes
        .iter()
        .map(|node| node.node_target_reference.clone())
        .collect::<Vec<_>>();
    let edge = |source, target, relation, site: &str, offset| {
        build_hld_path_edge(
            generation,
            &unit,
            &keys[source],
            &keys[target],
            relation,
            site,
            offset,
        )
    };
    let input = build_hld_path_input(
        generation,
        &unit,
        nodes,
        vec![
            edge(0, 1, HldRelationKindValue::Contains, "contains-first", 22),
            edge(1, 2, HldRelationKindValue::Calls, "contains-next", 23),
            edge(3, 4, HldRelationKindValue::Imports, "imports-first", 24),
            edge(4, 5, HldRelationKindValue::Calls, "imports-next", 25),
            edge(6, 7, HldRelationKindValue::Calls, "contains-hop-first", 26),
            edge(7, 8, HldRelationKindValue::Contains, "contains-hop", 27),
            edge(8, 9, HldRelationKindValue::Calls, "contains-hop-last", 28),
            edge(10, 11, HldRelationKindValue::Calls, "imports-hop-first", 29),
            edge(11, 12, HldRelationKindValue::Imports, "imports-hop", 30),
            edge(12, 13, HldRelationKindValue::Calls, "imports-hop-last", 31),
            edge(14, 15, HldRelationKindValue::Reads, "reads-first", 32),
            edge(15, 16, HldRelationKindValue::Reads, "reads-last", 33),
            edge(
                17,
                18,
                HldRelationKindValue::Contains,
                "direct-contains",
                34,
            ),
            edge(19, 20, HldRelationKindValue::Imports, "direct-imports", 35),
        ],
    );
    let output = materialize_hld_fixture_graph(&input);
    let has_edge = |source, target, relation, via| {
        output.materialized_edge_records.iter().any(|edge| {
            edge.edge_key
                == expected_hld_edge_key(generation, &keys[source], &keys[target], relation, via)
        })
    };

    assert!(!has_edge(
        0,
        2,
        HldRelationKindValue::Contains,
        HldViaKindValue::Collapsed
    ));
    assert!(!has_edge(
        3,
        5,
        HldRelationKindValue::Imports,
        HldViaKindValue::Collapsed
    ));
    assert!(!has_edge(
        6,
        9,
        HldRelationKindValue::Calls,
        HldViaKindValue::Collapsed
    ));
    assert!(!has_edge(
        10,
        13,
        HldRelationKindValue::Calls,
        HldViaKindValue::Collapsed
    ));
    assert!(has_edge(
        14,
        16,
        HldRelationKindValue::Reads,
        HldViaKindValue::Collapsed
    ));
    assert!(has_edge(
        17,
        18,
        HldRelationKindValue::Contains,
        HldViaKindValue::Direct
    ));
    assert!(has_edge(
        19,
        20,
        HldRelationKindValue::Imports,
        HldViaKindValue::Direct
    ));
}

/// REQ-RUST-030.0
#[test]
fn test_hld_cycle_self_edges() {
    let generation = RunGenerationDigestValue([34; 32]);
    let unit_identity = build_hld_fixture_unit("cycle-fixture", 34);
    let a = build_hld_path_node(
        generation,
        &unit_identity,
        "a",
        1,
        HldEffectiveVisibilityKind::Public,
        "crate::a",
    );
    let p = build_hld_path_node(
        generation,
        &unit_identity,
        "p",
        2,
        HldEffectiveVisibilityKind::PrivateEquivalent,
        "crate::private::p",
    );
    let q = build_hld_path_node(
        generation,
        &unit_identity,
        "q",
        3,
        HldEffectiveVisibilityKind::PrivateEquivalent,
        "crate::private::q",
    );
    let a_key = a.node_target_reference.clone();
    let p_key = p.node_target_reference.clone();
    let q_key = q.node_target_reference.clone();
    let input = build_hld_path_input(
        generation,
        &unit_identity,
        vec![a, p, q],
        vec![
            build_hld_path_edge(
                generation,
                &unit_identity,
                &a_key,
                &p_key,
                HldRelationKindValue::Calls,
                "a-p",
                61,
            ),
            build_hld_path_edge(
                generation,
                &unit_identity,
                &p_key,
                &q_key,
                HldRelationKindValue::Calls,
                "p-q",
                62,
            ),
            build_hld_path_edge(
                generation,
                &unit_identity,
                &q_key,
                &p_key,
                HldRelationKindValue::Calls,
                "q-p",
                63,
            ),
            build_hld_path_edge(
                generation,
                &unit_identity,
                &p_key,
                &a_key,
                HldRelationKindValue::UsesType,
                "p-a",
                64,
            ),
            build_hld_path_edge(
                generation,
                &unit_identity,
                &a_key,
                &a_key,
                HldRelationKindValue::Implements,
                "a-a",
                65,
            ),
        ],
    );
    let output = materialize_hld_fixture_graph(&input);
    let expected_edges = BTreeSet::from([
        (
            expected_hld_edge_key(
                generation,
                a_key.clone(),
                a_key.clone(),
                HldRelationKindValue::UsesType,
                HldViaKindValue::Collapsed,
            ),
            2,
        ),
        (
            expected_hld_edge_key(
                generation,
                a_key.clone(),
                a_key.clone(),
                HldRelationKindValue::Implements,
                HldViaKindValue::Direct,
            ),
            1,
        ),
    ]);
    assert_eq!(collect_hld_edge_counts(&output), expected_edges);
    let mut expected_paths = vec![
        build_hld_path_record(
            generation,
            a_key.clone(),
            a_key.clone(),
            HldRelationKindValue::UsesType,
            HldViaKindValue::Collapsed,
            vec![a_key.clone(), p_key.clone(), a_key.clone()],
            vec![
                input.input_edge_records[0].edge_evidence_row_key.clone(),
                input.input_edge_records[3].edge_evidence_row_key.clone(),
            ],
        ),
        build_hld_path_record(
            generation,
            a_key.clone(),
            a_key.clone(),
            HldRelationKindValue::Implements,
            HldViaKindValue::Direct,
            vec![a_key.clone(), a_key.clone()],
            vec![input.input_edge_records[4].edge_evidence_row_key.clone()],
        ),
    ];
    expected_paths.sort();
    assert_eq!(output.evidence_path_records, expected_paths);
    let mut private_cycle_input = input.clone();
    private_cycle_input.input_edge_records.truncate(3);
    let private_cycle_output = materialize_hld_fixture_graph(&private_cycle_input);
    assert!(private_cycle_output.materialized_edge_records.is_empty());
    assert!(private_cycle_output.evidence_path_records.is_empty());
}

/// REQ-RUST-030.0
#[test]
fn test_hld_unresolved_path_stop() {
    let generation = RunGenerationDigestValue([35; 32]);
    let unit_identity = build_hld_fixture_unit("unresolved-path-fixture", 35);
    let a = build_hld_path_node(
        generation,
        &unit_identity,
        "a",
        1,
        HldEffectiveVisibilityKind::Public,
        "crate::a",
    );
    let p = build_hld_path_node(
        generation,
        &unit_identity,
        "p",
        2,
        HldEffectiveVisibilityKind::PrivateEquivalent,
        "crate::private::p",
    );
    let b = build_hld_path_node(
        generation,
        &unit_identity,
        "b",
        3,
        HldEffectiveVisibilityKind::Public,
        "crate::b",
    );
    let r = build_hld_path_node(
        generation,
        &unit_identity,
        "r",
        4,
        HldEffectiveVisibilityKind::PrivateEquivalent,
        "crate::private::r",
    );
    let c = build_hld_path_node(
        generation,
        &unit_identity,
        "c",
        5,
        HldEffectiveVisibilityKind::Public,
        "crate::c",
    );
    let s = build_hld_path_node(
        generation,
        &unit_identity,
        "s",
        6,
        HldEffectiveVisibilityKind::PrivateEquivalent,
        "crate::private::s",
    );
    let d = build_hld_path_node(
        generation,
        &unit_identity,
        "d",
        7,
        HldEffectiveVisibilityKind::Public,
        "crate::d",
    );
    let q = build_hld_path_node(
        generation,
        &unit_identity,
        "q",
        8,
        HldEffectiveVisibilityKind::PrivateEquivalent,
        "crate::private::q",
    );
    let e = build_hld_path_node(
        generation,
        &unit_identity,
        "e",
        9,
        HldEffectiveVisibilityKind::Public,
        "crate::e",
    );
    let f = build_hld_path_node(
        generation,
        &unit_identity,
        "f",
        10,
        HldEffectiveVisibilityKind::Public,
        "crate::f",
    );
    let a_key = a.node_target_reference.clone();
    let p_key = p.node_target_reference.clone();
    let b_key = b.node_target_reference.clone();
    let r_key = r.node_target_reference.clone();
    let c_key = c.node_target_reference.clone();
    let s_key = s.node_target_reference.clone();
    let d_key = d.node_target_reference.clone();
    let q_key = q.node_target_reference.clone();
    let e_key = e.node_target_reference.clone();
    let f_key = f.node_target_reference.clone();
    let mut unresolved = build_hld_path_edge(
        generation,
        &unit_identity,
        &p_key,
        &b_key,
        HldRelationKindValue::Calls,
        "p-b-unresolved",
        71,
    );
    unresolved.edge_resolution_value = HldEdgeResolutionValue::Unresolved {
        reason: HldUnresolvedReasonKind::MissingTarget,
    };
    unresolved
        .edge_evidence_row_key
        .evidence_target_reference_value = None;
    let mut proposed = build_hld_path_edge(
        generation,
        &unit_identity,
        &r_key,
        &c_key,
        HldRelationKindValue::Calls,
        "r-c-proposed",
        72,
    );
    proposed.edge_resolution_value = HldEdgeResolutionValue::Proposed {
        candidate_target: Some(c_key.clone()),
    };
    let mut inferred = build_hld_path_edge(
        generation,
        &unit_identity,
        &s_key,
        &d_key,
        HldRelationKindValue::Calls,
        "s-d-inferred",
        73,
    );
    inferred.edge_resolution_value = HldEdgeResolutionValue::Inferred {
        candidate_target: Some(d_key.clone()),
    };
    let input = build_hld_path_input(
        generation,
        &unit_identity,
        vec![a, p, b, r, c, s, d, q, e, f],
        vec![
            build_hld_path_edge(
                generation,
                &unit_identity,
                &a_key,
                &p_key,
                HldRelationKindValue::Calls,
                "a-p",
                74,
            ),
            unresolved,
            build_hld_path_edge(
                generation,
                &unit_identity,
                &a_key,
                &r_key,
                HldRelationKindValue::Calls,
                "a-r",
                75,
            ),
            proposed,
            build_hld_path_edge(
                generation,
                &unit_identity,
                &a_key,
                &s_key,
                HldRelationKindValue::Calls,
                "a-s",
                76,
            ),
            inferred,
            build_hld_path_edge(
                generation,
                &unit_identity,
                &a_key,
                &q_key,
                HldRelationKindValue::Calls,
                "a-q",
                77,
            ),
            build_hld_path_edge(
                generation,
                &unit_identity,
                &q_key,
                &e_key,
                HldRelationKindValue::UsesType,
                "q-e",
                78,
            ),
            build_hld_path_edge(
                generation,
                &unit_identity,
                &a_key,
                &f_key,
                HldRelationKindValue::Calls,
                "a-f",
                79,
            ),
        ],
    );
    let output = materialize_hld_fixture_graph(&input);
    let expected_edges = BTreeSet::from([
        (
            expected_hld_edge_key(
                generation,
                a_key.clone(),
                e_key.clone(),
                HldRelationKindValue::UsesType,
                HldViaKindValue::Collapsed,
            ),
            2,
        ),
        (
            expected_hld_edge_key(
                generation,
                a_key.clone(),
                f_key.clone(),
                HldRelationKindValue::Calls,
                HldViaKindValue::Direct,
            ),
            1,
        ),
    ]);
    assert_eq!(collect_hld_edge_counts(&output), expected_edges);
    let unresolved_targets = BTreeSet::from([b_key, c_key, d_key]);
    assert!(
        !output.materialized_edge_records.iter().any(|edge| {
            unresolved_targets.contains(&edge.edge_key.edge_target_reference_value)
        })
    );
    let mut expected_paths = vec![
        build_hld_path_record(
            generation,
            a_key.clone(),
            e_key.clone(),
            HldRelationKindValue::UsesType,
            HldViaKindValue::Collapsed,
            vec![a_key.clone(), q_key.clone(), e_key.clone()],
            vec![
                input.input_edge_records[6].edge_evidence_row_key.clone(),
                input.input_edge_records[7].edge_evidence_row_key.clone(),
            ],
        ),
        build_hld_path_record(
            generation,
            a_key.clone(),
            f_key.clone(),
            HldRelationKindValue::Calls,
            HldViaKindValue::Direct,
            vec![a_key.clone(), f_key.clone()],
            vec![input.input_edge_records[8].edge_evidence_row_key.clone()],
        ),
    ];
    expected_paths.sort();
    assert_eq!(output.evidence_path_records, expected_paths);
}

/// REQ-RUST-031.0 / REQ-RUST-009.2
#[test]
fn test_hld_rollup_edge_counts() {
    let generation = RunGenerationDigestValue([42; 32]);
    let unit = build_hld_fixture_unit("rollup-counts-fixture", 42);
    let a = build_hld_path_node(
        generation,
        &unit,
        "group/a",
        1,
        HldEffectiveVisibilityKind::Public,
        "crate::a",
    );
    let p = build_hld_path_node(
        generation,
        &unit,
        "private/p",
        2,
        HldEffectiveVisibilityKind::PrivateEquivalent,
        "crate::private::p",
    );
    let q = build_hld_path_node(
        generation,
        &unit,
        "private/q",
        3,
        HldEffectiveVisibilityKind::PrivateEquivalent,
        "crate::private::q",
    );
    let b = build_hld_path_node(
        generation,
        &unit,
        "target/b",
        4,
        HldEffectiveVisibilityKind::Public,
        "crate::b",
    );
    let c = build_hld_path_node(
        generation,
        &unit,
        "group/c",
        5,
        HldEffectiveVisibilityKind::Public,
        "crate::c",
    );
    let d = build_hld_path_node(
        generation,
        &unit,
        "target/d",
        6,
        HldEffectiveVisibilityKind::Public,
        "crate::d",
    );
    let e = build_hld_path_node(
        generation,
        &unit,
        "group/e",
        7,
        HldEffectiveVisibilityKind::Public,
        "crate::e",
    );
    let f = build_hld_path_node(
        generation,
        &unit,
        "target/f",
        8,
        HldEffectiveVisibilityKind::Public,
        "crate::f",
    );
    let (a_key, p_key, q_key, b_key, c_key, d_key, e_key, f_key) = (
        a.node_target_reference.clone(),
        p.node_target_reference.clone(),
        q.node_target_reference.clone(),
        b.node_target_reference.clone(),
        c.node_target_reference.clone(),
        d.node_target_reference.clone(),
        e.node_target_reference.clone(),
        f.node_target_reference.clone(),
    );
    let input = build_hld_path_input(
        generation,
        &unit,
        vec![a, p, q, b, c, d, e, f],
        vec![
            build_hld_path_edge(
                generation,
                &unit,
                &a_key,
                &p_key,
                HldRelationKindValue::Calls,
                "a-p",
                101,
            ),
            build_hld_path_edge(
                generation,
                &unit,
                &p_key,
                &b_key,
                HldRelationKindValue::UsesType,
                "p-b",
                102,
            ),
            build_hld_path_edge(
                generation,
                &unit,
                &a_key,
                &q_key,
                HldRelationKindValue::Calls,
                "a-q",
                103,
            ),
            build_hld_path_edge(
                generation,
                &unit,
                &q_key,
                &b_key,
                HldRelationKindValue::UsesType,
                "q-b",
                104,
            ),
            build_hld_path_edge(
                generation,
                &unit,
                &a_key,
                &b_key,
                HldRelationKindValue::UsesType,
                "a-b",
                105,
            ),
            build_hld_path_edge(
                generation,
                &unit,
                &c_key,
                &d_key,
                HldRelationKindValue::Calls,
                "c-d",
                106,
            ),
            build_hld_path_edge(
                generation,
                &unit,
                &e_key,
                &f_key,
                HldRelationKindValue::Calls,
                "e-f",
                107,
            ),
        ],
    );
    let output = materialize_hld_fixture_graph(&input);
    let collapsed_key = expected_hld_edge_key(
        generation,
        a_key.clone(),
        b_key.clone(),
        HldRelationKindValue::UsesType,
        HldViaKindValue::Collapsed,
    );
    assert_eq!(output.materialized_edge_records.len(), 4);
    assert_eq!(
        output
            .materialized_edge_records
            .iter()
            .filter(|edge| edge.edge_key == collapsed_key)
            .count(),
        1
    );
    assert_eq!(
        output
            .materialized_edge_records
            .iter()
            .find(|edge| edge.edge_key == collapsed_key)
            .unwrap()
            .edge_evidence_row_count,
        4
    );
    assert_eq!(
        output
            .evidence_path_records
            .iter()
            .filter(|path| path.edge_key == collapsed_key)
            .count(),
        2
    );
    let a_file = hld_source_file_bucket(unit.clone(), "src/group/a.rs");
    let b_file = hld_source_file_bucket(unit.clone(), "src/target/b.rs");
    let c_file = hld_source_file_bucket(unit.clone(), "src/group/c.rs");
    let d_file = hld_source_file_bucket(unit.clone(), "src/target/d.rs");
    let e_file = hld_source_file_bucket(unit.clone(), "src/group/e.rs");
    let f_file = hld_source_file_bucket(unit.clone(), "src/target/f.rs");
    let mut expected_files = vec![
        build_hld_file_rollup(
            a_file.clone(),
            b_file.clone(),
            HldRelationKindValue::UsesType,
            HldViaKindValue::Collapsed,
            1,
        ),
        build_hld_file_rollup(
            a_file.clone(),
            b_file.clone(),
            HldRelationKindValue::UsesType,
            HldViaKindValue::Direct,
            1,
        ),
        build_hld_file_rollup(
            c_file,
            d_file,
            HldRelationKindValue::Calls,
            HldViaKindValue::Direct,
            1,
        ),
        build_hld_file_rollup(
            e_file,
            f_file,
            HldRelationKindValue::Calls,
            HldViaKindValue::Direct,
            1,
        ),
    ];
    expected_files.sort();
    assert_eq!(output.file_rollup_records, Some(expected_files));
    let group_folder = hld_source_folder_bucket(unit.clone(), "src/group");
    let target_folder = hld_source_folder_bucket(unit.clone(), "src/target");
    let mut expected_folders = vec![
        build_hld_folder_rollup(
            group_folder.clone(),
            target_folder.clone(),
            HldRelationKindValue::UsesType,
            HldViaKindValue::Collapsed,
            1,
        ),
        build_hld_folder_rollup(
            group_folder.clone(),
            target_folder.clone(),
            HldRelationKindValue::UsesType,
            HldViaKindValue::Direct,
            1,
        ),
        build_hld_folder_rollup(
            group_folder,
            target_folder,
            HldRelationKindValue::Calls,
            HldViaKindValue::Direct,
            2,
        ),
    ];
    expected_folders.sort();
    assert_eq!(output.folder_rollup_records, Some(expected_folders));
    assert_eq!(
        output.output_completeness_kind,
        HldOutputCompletenessKind::CompleteGraph
    );
    assert_eq!(
        sum_hld_file_rollups(output.file_rollup_records.as_deref().unwrap()),
        output.materialized_edge_records.len()
    );
    assert_eq!(
        sum_hld_folder_rollups(output.folder_rollup_records.as_deref().unwrap()),
        output.materialized_edge_records.len()
    );
}

/// REQ-RUST-031.0 / REQ-RUST-009.2
#[test]
fn test_hld_rollup_bucket_context() {
    let generation = RunGenerationDigestValue([43; 32]);
    let first_unit = build_hld_fixture_unit("bucket-first", 43);
    let second_unit = build_hld_fixture_unit("bucket-second", 44);
    let known_unit = build_hld_fixture_unit("core", 45);
    let missing_unit = build_hld_fixture_unit("remote", 46);
    let root_key = NameSymbolTargetKey::CrateRoot(first_unit.clone());
    let root_file = build_hld_bucket_node(generation, &first_unit, "lib.rs", 1, "crate::root_file");
    let first_shared = build_hld_bucket_node(
        generation,
        &first_unit,
        "src/shared.rs",
        2,
        "crate::first_shared",
    );
    let second_shared = build_hld_bucket_node(
        generation,
        &second_unit,
        "src/shared.rs",
        3,
        "crate::second_shared",
    );
    let main = build_hld_bucket_node(generation, &first_unit, "src/main.rs", 4, "crate::main");
    let root_file_key = root_file.node_target_reference.clone();
    let first_shared_key = first_shared.node_target_reference.clone();
    let second_shared_key = second_shared.node_target_reference.clone();
    let main_key = main.node_target_reference.clone();
    let known_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &known_unit,
        "vendor/known.rs",
        5,
        5,
    ));
    let missing_key = NameSymbolTargetKey::Definition(build_hld_fixture_occurrence(
        &missing_unit,
        "vendor/missing.rs",
        6,
        6,
    ));
    let builtin_key = NameSymbolTargetKey::Builtin("u8".into());
    let nodes = vec![
        build_hld_fixture_node(
            generation,
            root_key.clone(),
            HldNodeInputCategory::CrateRoot,
            None,
            None,
            false,
            None,
            true,
            None,
            "crate",
        ),
        root_file,
        first_shared,
        second_shared,
        main,
        build_hld_fixture_node(
            generation,
            known_key.clone(),
            HldNodeInputCategory::ExternalDefinition,
            None,
            None,
            false,
            None,
            true,
            Some("vendor/known.rs"),
            "core::known",
        ),
        build_hld_fixture_node(
            generation,
            missing_key.clone(),
            HldNodeInputCategory::ExternalDefinition,
            None,
            None,
            false,
            None,
            true,
            None,
            "remote::missing",
        ),
        build_hld_fixture_node(
            generation,
            builtin_key.clone(),
            HldNodeInputCategory::Builtin,
            None,
            None,
            false,
            None,
            true,
            None,
            "u8",
        ),
    ];
    let input = build_hld_fixture_input(
        generation,
        vec![
            HldUnitProvenanceRecord {
                unit_compilation_identity_key: first_unit.clone(),
                unit_origin_kind: HldUnitOriginKind::Workspace,
            },
            HldUnitProvenanceRecord {
                unit_compilation_identity_key: second_unit.clone(),
                unit_origin_kind: HldUnitOriginKind::Workspace,
            },
            HldUnitProvenanceRecord {
                unit_compilation_identity_key: known_unit.clone(),
                unit_origin_kind: HldUnitOriginKind::ExternalCrate,
            },
            HldUnitProvenanceRecord {
                unit_compilation_identity_key: missing_unit.clone(),
                unit_origin_kind: HldUnitOriginKind::ExternalCrate,
            },
        ],
        nodes,
        vec![
            build_hld_path_edge(
                generation,
                &first_unit,
                &root_key,
                &root_file_key,
                HldRelationKindValue::Contains,
                "root-contains",
                31,
            ),
            build_hld_path_edge(
                generation,
                &first_unit,
                &first_shared_key,
                &known_key,
                HldRelationKindValue::Calls,
                "first-known",
                32,
            ),
            build_hld_path_edge(
                generation,
                &second_unit,
                &second_shared_key,
                &known_key,
                HldRelationKindValue::Calls,
                "second-known",
                33,
            ),
            build_hld_path_edge(
                generation,
                &first_unit,
                &main_key,
                &missing_key,
                HldRelationKindValue::UsesType,
                "missing-file",
                34,
            ),
            build_hld_path_edge(
                generation,
                &first_unit,
                &main_key,
                &builtin_key,
                HldRelationKindValue::Implements,
                "builtin",
                35,
            ),
        ],
    );
    let output = materialize_hld_fixture_graph(&input);
    assert_eq!(output.materialized_edge_records.len(), 5);
    let first_file = hld_source_file_bucket(first_unit.clone(), "src/shared.rs");
    let second_file = hld_source_file_bucket(second_unit.clone(), "src/shared.rs");
    let known_file = hld_source_file_bucket(known_unit.clone(), "vendor/known.rs");
    let main_file = hld_source_file_bucket(first_unit.clone(), "src/main.rs");
    let mut expected_files = vec![
        build_hld_file_rollup(
            HldEndpointRollupBucket::CrateRoot {
                unit_compilation_identity_key: first_unit.clone(),
            },
            hld_source_file_bucket(first_unit.clone(), "lib.rs"),
            HldRelationKindValue::Contains,
            HldViaKindValue::Direct,
            1,
        ),
        build_hld_file_rollup(
            first_file.clone(),
            known_file.clone(),
            HldRelationKindValue::Calls,
            HldViaKindValue::Direct,
            1,
        ),
        build_hld_file_rollup(
            second_file.clone(),
            known_file,
            HldRelationKindValue::Calls,
            HldViaKindValue::Direct,
            1,
        ),
        build_hld_file_rollup(
            main_file.clone(),
            HldEndpointRollupBucket::CanonicalExternalIdentity {
                endpoint_target_reference: missing_key.clone().into(),
            },
            HldRelationKindValue::UsesType,
            HldViaKindValue::Direct,
            1,
        ),
        build_hld_file_rollup(
            main_file.clone(),
            HldEndpointRollupBucket::CanonicalExternalIdentity {
                endpoint_target_reference: builtin_key.clone().into(),
            },
            HldRelationKindValue::Implements,
            HldViaKindValue::Direct,
            1,
        ),
    ];
    expected_files.sort();
    assert_eq!(output.file_rollup_records, Some(expected_files));
    assert_ne!(first_file, second_file);
    let mut expected_folders = vec![
        build_hld_folder_rollup(
            HldFolderEndpointBucket::CrateRoot {
                unit_compilation_identity_key: first_unit.clone(),
            },
            hld_source_folder_bucket(first_unit.clone(), "."),
            HldRelationKindValue::Contains,
            HldViaKindValue::Direct,
            1,
        ),
        build_hld_folder_rollup(
            hld_source_folder_bucket(first_unit.clone(), "src"),
            hld_source_folder_bucket(known_unit.clone(), "vendor"),
            HldRelationKindValue::Calls,
            HldViaKindValue::Direct,
            1,
        ),
        build_hld_folder_rollup(
            hld_source_folder_bucket(second_unit.clone(), "src"),
            hld_source_folder_bucket(known_unit, "vendor"),
            HldRelationKindValue::Calls,
            HldViaKindValue::Direct,
            1,
        ),
        build_hld_folder_rollup(
            hld_source_folder_bucket(first_unit.clone(), "src"),
            HldFolderEndpointBucket::CanonicalExternalIdentity {
                endpoint_target_reference: missing_key.into(),
            },
            HldRelationKindValue::UsesType,
            HldViaKindValue::Direct,
            1,
        ),
        build_hld_folder_rollup(
            hld_source_folder_bucket(first_unit, "src"),
            HldFolderEndpointBucket::CanonicalExternalIdentity {
                endpoint_target_reference: builtin_key.into(),
            },
            HldRelationKindValue::Implements,
            HldViaKindValue::Direct,
            1,
        ),
    ];
    expected_folders.sort();
    assert_eq!(output.folder_rollup_records, Some(expected_folders));
    assert_eq!(
        output.output_completeness_kind,
        HldOutputCompletenessKind::CompleteGraph
    );
    assert_eq!(
        sum_hld_file_rollups(output.file_rollup_records.as_deref().unwrap()),
        output.materialized_edge_records.len()
    );
    assert_eq!(
        sum_hld_folder_rollups(output.folder_rollup_records.as_deref().unwrap()),
        output.materialized_edge_records.len()
    );
}

/// REQ-RUST-031.0
#[test]
fn test_hld_empty_input_rollups() {
    let generation = RunGenerationDigestValue([44; 32]);
    let input = build_hld_fixture_input(generation, Vec::new(), Vec::new(), Vec::new());
    let output = materialize_hld_fixture_graph(&input);
    assert_eq!(output.file_rollup_records, Some(Vec::new()));
    assert_eq!(output.folder_rollup_records, Some(Vec::new()));
    assert_eq!(
        output.output_completeness_kind,
        HldOutputCompletenessKind::CompleteGraph
    );
}

/// REQ-RUST-030.0 / REQ-RUST-031.0
#[test]
fn test_hld_order_attempt_determinism() {
    let generation = RunGenerationDigestValue([36; 32]);
    let unit_identity = build_hld_fixture_unit("order-attempt-fixture", 36);
    let a = build_hld_path_node(
        generation,
        &unit_identity,
        "a",
        1,
        HldEffectiveVisibilityKind::Public,
        "crate::a",
    );
    let p = build_hld_path_node(
        generation,
        &unit_identity,
        "p",
        2,
        HldEffectiveVisibilityKind::PrivateEquivalent,
        "crate::private::p",
    );
    let b = build_hld_path_node(
        generation,
        &unit_identity,
        "b",
        3,
        HldEffectiveVisibilityKind::Public,
        "crate::b",
    );
    let a_key = a.node_target_reference.clone();
    let p_key = p.node_target_reference.clone();
    let b_key = b.node_target_reference.clone();
    let input = build_hld_path_input(
        generation,
        &unit_identity,
        vec![a, p, b],
        vec![
            build_hld_path_edge(
                generation,
                &unit_identity,
                &a_key,
                &p_key,
                HldRelationKindValue::Calls,
                "a-p",
                81,
            ),
            build_hld_path_edge(
                generation,
                &unit_identity,
                &p_key,
                &b_key,
                HldRelationKindValue::UsesType,
                "p-b",
                82,
            ),
            build_hld_path_edge(
                generation,
                &unit_identity,
                &a_key,
                &b_key,
                HldRelationKindValue::UsesType,
                "a-b",
                83,
            ),
        ],
    );
    let store = build_hld_fixture_store();
    let expected = store
        .materialize_hld_graph_output(&input, &RunCancelFlagValue::default())
        .unwrap();
    assert!(!expected.materialized_edge_records.is_empty());
    assert!(
        expected
            .file_rollup_records
            .as_ref()
            .is_some_and(|rows| !rows.is_empty())
    );
    assert!(
        expected
            .folder_rollup_records
            .as_ref()
            .is_some_and(|rows| !rows.is_empty())
    );
    let mut reordered = input.clone();
    reordered.input_unit_provenance_records.reverse();
    reordered.input_node_records.reverse();
    reordered.input_edge_records.reverse();
    reordered
        .input_unit_provenance_records
        .extend(input.input_unit_provenance_records.clone());
    reordered
        .input_node_records
        .extend(input.input_node_records.clone());
    reordered
        .input_edge_records
        .extend(input.input_edge_records.clone());
    reordered.stage_attempt_sequence_value = "attempt-b".into();
    let reordered_output = store
        .materialize_hld_graph_output(&reordered, &RunCancelFlagValue::default())
        .unwrap();
    assert_eq!(reordered_output, expected);
    let empty_input = build_hld_fixture_input(generation, Vec::new(), Vec::new(), Vec::new());
    let empty_output = store
        .materialize_hld_graph_output(&empty_input, &RunCancelFlagValue::default())
        .unwrap();
    assert_eq!(
        empty_output.output_completeness_kind,
        HldOutputCompletenessKind::CompleteGraph
    );
    assert!(empty_output.selected_node_records.is_empty());
    assert!(empty_output.materialized_edge_records.is_empty());
    assert!(empty_output.evidence_path_records.is_empty());
    assert_eq!(empty_output.file_rollup_records, Some(Vec::new()));
    assert_eq!(empty_output.folder_rollup_records, Some(Vec::new()));
}

fn encode_fixture_generation_text(generation: RunGenerationDigestValue) -> String {
    generation
        .0
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn build_hld_publication_scope(
    generation: RunGenerationDigestValue,
    attempt: &str,
) -> StageRunScopeValue {
    StageRunScopeValue {
        run_generation_digest_value: encode_fixture_generation_text(generation),
        stage_attempt_sequence_value: attempt.into(),
    }
}

fn write_hld_publication_marker(
    store: &TursoWorkspaceStoreAdapter,
    scope: &StageRunScopeValue,
    stage: &str,
) {
    store
        .write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: scope,
                stage_owner_name_value: stage,
                stage_unit_work_key: "__complete__",
                stage_input_stage_names: &[],
                stage_unit_output_rows: &[],
            },
            &RunCancelFlagValue::default(),
        )
        .unwrap();
}

fn complete_hld_publication_stages(store: &TursoWorkspaceStoreAdapter, scope: &StageRunScopeValue) {
    for stage in [
        "crate-graph",
        "source-ingest",
        "syntax-spans",
        "syntax-namespace-input-records",
        "name-resolution",
        "signature-type-lowering-stage",
        "item-effective-visibility-stage",
        "semantic-edge-evidence-stage",
    ] {
        write_hld_publication_marker(store, scope, stage);
    }
}

fn seal_hld_publication_snapshot(
    store: &TursoWorkspaceStoreAdapter,
    scope: &StageRunScopeValue,
) -> StageRunScopeValue {
    store
        .publish_workspace_generation_manifest(
            scope,
            &scope.run_generation_digest_value,
            &RunCancelFlagValue::default(),
        )
        .unwrap()
}

#[test]
/// REQ-RUST-031.0
fn test_hld_publication_round_trip() {
    let (input, unit_identity, _, _) = build_hld_validation_input();
    let output = materialize_hld_fixture_graph(&input);
    assert_eq!(
        output.output_completeness_kind,
        HldOutputCompletenessKind::CompleteGraph
    );
    assert!(!output.selected_node_records.is_empty());
    assert!(!output.materialized_edge_records.is_empty());
    let store = TursoWorkspaceStoreAdapter::open_turso_workspace_store(":memory:").unwrap();
    store.initialize_workspace_stage_schema().unwrap();
    let old_generation = input.run_generation_digest_value;
    let old_scope = build_hld_publication_scope(old_generation, "old-attempt");
    complete_hld_publication_stages(&store, &old_scope);
    let old_frozen = seal_hld_publication_snapshot(&store, &old_scope);
    let old_output = output.clone();
    let old_published = store
        .publish_hld_graph_materialization(
            &old_frozen,
            &old_scope,
            &old_output,
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    let new_scope = build_hld_publication_scope(input.run_generation_digest_value, "new-attempt");
    complete_hld_publication_stages(&store, &new_scope);
    let new_frozen = seal_hld_publication_snapshot(&store, &new_scope);
    let published = store
        .publish_hld_graph_materialization(
            &new_frozen,
            &new_scope,
            &output,
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    let table_counts = [
        (
            "hld-current-interface-nodes",
            output.selected_node_records.len(),
        ),
        (
            "hld-current-dependency-edges",
            output.materialized_edge_records.len(),
        ),
        (
            "hld-edge-evidence-paths",
            output.evidence_path_records.len(),
        ),
        (
            "hld-file-level-rollup",
            output.file_rollup_records.as_ref().unwrap().len(),
        ),
        (
            "hld-folder-level-rollup",
            output.folder_rollup_records.as_ref().unwrap().len(),
        ),
    ];
    for (table, expected_count) in table_counts {
        let new_rows = store.read_stage_table_rows(&published, table).unwrap();
        let old_rows = store.read_stage_table_rows(&old_published, table).unwrap();
        assert_eq!(new_rows.len(), expected_count, "{table}");
        assert_eq!(old_rows.len(), expected_count, "{table}");
        assert!(new_rows.iter().all(|row| {
            serde_json::from_str::<serde_json::Value>(&row.stage_row_payload_text).is_ok()
        }));
        let mut new_semantic_rows = new_rows
            .into_iter()
            .map(|row| (row.stage_row_record_key, row.stage_row_payload_text))
            .collect::<Vec<_>>();
        let mut old_semantic_rows = old_rows
            .into_iter()
            .map(|row| (row.stage_row_record_key, row.stage_row_payload_text))
            .collect::<Vec<_>>();
        new_semantic_rows.sort_unstable();
        old_semantic_rows.sort_unstable();
        assert_eq!(new_semantic_rows, old_semantic_rows, "{table}");
    }
    let unit_identity_json = serde_json::json!({
        "cargo_package_identifier": unit_identity.unit_cargo_package_identifier,
        "cargo_target_name": unit_identity.unit_cargo_target_name,
        "cargo_target_kind": unit_identity.unit_cargo_target_kind,
        "compilation_role_kind": "Target",
        "configuration_digest_full": vec![21_u8; 32]
    });
    let source_occurrence_json = serde_json::json!({
        "unit_compilation_identity": unit_identity_json,
        "source_file_path": "src/source.rs",
        "byte_range": [1, 2],
        "content_digest_full": vec![1_u8; 32],
        "expansion_provenance_chain": []
    });
    let source_identity_json = serde_json::json!({
        "kind": "Definition",
        "occurrence_identity": source_occurrence_json
    });
    let builtin_identity_json = serde_json::json!({
        "kind": "Builtin",
        "builtin_name": "u8"
    });
    let expected_edge_key = serde_json::json!({
        "generation_digest_full": vec![21_u8; 32],
        "source_target_identity": source_identity_json,
        "target_identity": builtin_identity_json,
        "relation_kind": "Calls",
        "via_kind": "Direct"
    });
    let expected_site_occurrence_json = serde_json::json!({
        "unit_compilation_identity": unit_identity_json,
        "source_file_path": "src/callsite.rs",
        "byte_range": [8, 9],
        "content_digest_full": vec![8_u8; 32],
        "expansion_provenance_chain": []
    });
    let expected_evidence_identity = serde_json::json!({
        "site_occurrence_identity": expected_site_occurrence_json,
        "source_target_identity": source_identity_json,
        "relation_kind": "Calls",
        "target_identity": builtin_identity_json
    });
    let new_nodes = store
        .read_stage_table_rows(&published, "hld-current-interface-nodes")
        .unwrap();
    let source_node = new_nodes
        .iter()
        .find(|row| {
            serde_json::from_str::<serde_json::Value>(&row.stage_row_payload_text)
                .is_ok_and(|payload| payload["node_qualified_path_text"] == "crate::source")
        })
        .unwrap();
    let expected_node_key = serde_json::json!({
        "generation_digest_full": vec![21_u8; 32],
        "node_target_identity": source_identity_json
    });
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&source_node.stage_row_record_key).unwrap(),
        expected_node_key
    );
    let source_node_payload: serde_json::Value =
        serde_json::from_str(&source_node.stage_row_payload_text).unwrap();
    assert_eq!(
        source_node_payload["node_target_identity"],
        source_identity_json
    );
    assert_eq!(
        source_node_payload["node_source_relative_path"],
        "src/source.rs"
    );
    assert_eq!(
        source_node_payload["node_effective_visibility_kind"],
        "Public"
    );
    assert_eq!(
        source_node_payload["node_signature_text"],
        serde_json::Value::Null
    );

    let new_edges = store
        .read_stage_table_rows(&published, "hld-current-dependency-edges")
        .unwrap();
    let edge_payload: serde_json::Value =
        serde_json::from_str(&new_edges[0].stage_row_payload_text).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&new_edges[0].stage_row_record_key).unwrap(),
        expected_edge_key
    );
    assert_eq!(edge_payload["edge_key"], expected_edge_key);
    assert_eq!(edge_payload["edge_source_qualified_path"], "crate::source");
    assert_eq!(edge_payload["edge_target_qualified_path"], "u8");
    assert_eq!(
        edge_payload["edge_source_signature_text"],
        serde_json::Value::Null
    );
    assert_eq!(
        edge_payload["edge_target_signature_text"],
        serde_json::Value::Null
    );
    assert_eq!(edge_payload["edge_resolution_status_kind"], "Extracted");
    assert_eq!(edge_payload["edge_evidence_row_count"], 1);
    assert_eq!(
        edge_payload["edge_dependency_origin_kind"],
        "StandardLibrary"
    );

    let paths = store
        .read_stage_table_rows(&published, "hld-edge-evidence-paths")
        .unwrap();
    let path_key: serde_json::Value = serde_json::from_str(&paths[0].stage_row_record_key).unwrap();
    assert_eq!(path_key["edge_key"], expected_edge_key);
    assert_eq!(
        path_key["ordered_endpoint_identities"],
        serde_json::json!([source_identity_json, builtin_identity_json])
    );
    assert_eq!(
        path_key["ordered_proof_identities"],
        serde_json::json!([expected_evidence_identity])
    );
    let path_payload: serde_json::Value =
        serde_json::from_str(&paths[0].stage_row_payload_text).unwrap();
    assert_eq!(
        path_payload["ordered_endpoint_path"],
        serde_json::json!([source_identity_json, builtin_identity_json])
    );
    assert_eq!(
        path_payload["ordered_evidence_row_keys"],
        serde_json::json!([expected_evidence_identity])
    );
    assert_eq!(
        path_payload["ordered_site_occurrence_keys"],
        serde_json::json!([expected_site_occurrence_json])
    );

    let file_rollups = store
        .read_stage_table_rows(&published, "hld-file-level-rollup")
        .unwrap();
    let file_payload: serde_json::Value =
        serde_json::from_str(&file_rollups[0].stage_row_payload_text).unwrap();
    assert_eq!(file_payload["source_endpoint_bucket"]["kind"], "SourceFile");
    assert_eq!(
        file_payload["source_endpoint_bucket"]["unit_compilation_identity"],
        unit_identity_json
    );
    assert_eq!(
        file_payload["source_endpoint_bucket"]["file_source_relative_path"],
        "src/source.rs"
    );
    assert_eq!(
        file_payload["target_endpoint_bucket"]["kind"],
        "CanonicalExternalIdentity"
    );
    assert_eq!(
        file_payload["target_endpoint_bucket"]["endpoint_target_identity"],
        builtin_identity_json
    );
    assert_eq!(file_payload["relation_kind"], "Calls");
    assert_eq!(file_payload["via_kind"], "Direct");
    assert_eq!(file_payload["edge_row_count"], 1);

    let folder_rollups = store
        .read_stage_table_rows(&published, "hld-folder-level-rollup")
        .unwrap();
    let folder_payload: serde_json::Value =
        serde_json::from_str(&folder_rollups[0].stage_row_payload_text).unwrap();
    assert_eq!(
        folder_payload["source_folder_bucket"]["kind"],
        "SourceFolder"
    );
    assert_eq!(
        folder_payload["source_folder_bucket"]["unit_compilation_identity"],
        unit_identity_json
    );
    assert_eq!(
        folder_payload["source_folder_bucket"]["folder_source_relative_path"],
        "src"
    );
    assert_eq!(
        folder_payload["target_folder_bucket"]["kind"],
        "CanonicalExternalIdentity"
    );
    assert_eq!(
        folder_payload["target_folder_bucket"]["endpoint_target_identity"],
        builtin_identity_json
    );
    assert_eq!(folder_payload["relation_kind"], "Calls");
    assert_eq!(folder_payload["via_kind"], "Direct");
    assert_eq!(folder_payload["edge_row_count"], 1);
    let markers = store
        .read_stage_table_rows(&published, "stage-run-generation-records")
        .unwrap();
    let global = markers
        .iter()
        .find(|row| {
            row.stage_row_record_key == "__marker__/hld-current-materialization-stage/__complete__"
        })
        .unwrap();
    let metadata: serde_json::Value = serde_json::from_str(&global.stage_row_payload_text).unwrap();
    assert_eq!(
        metadata["input_snapshot_attempt_key"],
        new_frozen.stage_attempt_sequence_value
    );
    assert_eq!(
        store.read_workspace_publication_scope().unwrap(),
        Some(published)
    );
}

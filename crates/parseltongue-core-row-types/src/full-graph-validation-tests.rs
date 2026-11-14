#![allow(clippy::unwrap_used)]
use super::*;
use crate::{CompilationUnitRoleKind, SpanByteRangeValue, SpanDigestIdValue};

fn build_full_graph_fixture() -> HldMaterializationInputRecord {
    let unit = CrateCompilationUnitKey {
        unit_cargo_package_identifier: "fixture".into(),
        unit_cargo_target_name: "fixture".into(),
        unit_cargo_target_kind: "lib".into(),
        unit_compilation_role_kind: CompilationUnitRoleKind::Target,
        unit_configuration_digest_full: [4; 32],
    };
    let occurrence = SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
        &unit,
        "src/lib.rs",
        SpanByteRangeValue::try_from((0, 6)).unwrap(),
        SpanDigestIdValue::from([6; 32]),
        &[],
    )
    .unwrap();
    let target =
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(occurrence.clone()));
    HldMaterializationInputRecord {
        run_generation_digest_value: RunGenerationDigestValue([0; 32]),
        stage_attempt_sequence_value: "attempt".into(),
        semantic_input_completeness_kind: HldInputCompletenessKind::IncompleteTypedInput,
        source_coverage_kind: HldSourceCoverageKind::FullSemantic,
        input_unit_provenance_records: vec![HldUnitProvenanceRecord {
            unit_compilation_identity_key: unit,
            unit_origin_kind: HldUnitOriginKind::Workspace,
        }],
        input_node_records: vec![HldNodeInputRecord {
            run_generation_digest_value: RunGenerationDigestValue([0; 32]),
            node_target_reference: target.clone(),
            node_input_category: HldNodeInputCategory::LocalItem,
            node_item_kind_value: Some(HldItemKindValue::Function),
            node_owner_kind_value: Some(HldOwnerKindValue::Module),
            node_function_body_ancestry_flag: false,
            node_effective_visibility_kind: Some(HldEffectiveVisibilityKind::Unknown),
            node_reachable_outside_crate_flag: None,
            node_source_relative_path: Some("src/lib.rs".into()),
            node_qualified_path_text: "fixture::function".into(),
            node_signature_text: None,
        }],
        input_edge_records: vec![HldEdgeInputRecord {
            run_generation_digest_value: RunGenerationDigestValue([0; 32]),
            edge_source_reference: target.clone(),
            edge_relation_kind_value: HldRelationKindValue::Calls,
            edge_resolution_value: HldEdgeResolutionValue::Extracted {
                target: target.clone(),
            },
            edge_evidence_row_key: HldEvidenceRowKey {
                evidence_site_occurrence_key: occurrence,
                evidence_source_occurrence_key: target.clone(),
                evidence_relation_kind_value: HldRelationKindValue::Calls,
                evidence_target_reference_value: Some(target),
            },
            edge_dispatch_metadata_text: Some("direct".into()),
        }],
    }
}

/// REQ-RUST-045.0
#[test]
fn test_full_incomplete_visibility_preserved() {
    let mut input = build_full_graph_fixture();
    assert_eq!(validate_full_graph_input(&input), Ok(()));
    assert_eq!(
        validate_hld_materialization_input(&input),
        Err(HldMaterializationValidationError::IncompleteTypedInput)
    );
    input.input_node_records[0].node_effective_visibility_kind =
        Some(HldEffectiveVisibilityKind::Public);
    input.input_node_records[0].node_reachable_outside_crate_flag = Some(true);
    assert_eq!(validate_full_graph_input(&input), Ok(()));
    assert_eq!(
        validate_hld_materialization_input(&input),
        Err(HldMaterializationValidationError::IncompleteTypedInput)
    );
}

/// REQ-RUST-045.0
#[test]
fn test_full_complete_visibility_required() {
    let mut input = build_full_graph_fixture();
    input.semantic_input_completeness_kind = HldInputCompletenessKind::CompleteTypedInput;
    assert_eq!(
        validate_full_graph_input(&input),
        Err(HldMaterializationValidationError::UnknownEffectiveVisibility)
    );
    assert_eq!(
        validate_hld_materialization_input(&input),
        Err(HldMaterializationValidationError::UnknownEffectiveVisibility)
    );
    input.input_node_records[0].node_effective_visibility_kind =
        Some(HldEffectiveVisibilityKind::Public);
    assert_eq!(
        validate_full_graph_input(&input),
        Err(HldMaterializationValidationError::UnknownReachability)
    );
    assert_eq!(
        validate_hld_materialization_input(&input),
        Err(HldMaterializationValidationError::UnknownReachability)
    );
    input.input_node_records[0].node_reachable_outside_crate_flag = Some(true);
    assert_eq!(validate_full_graph_input(&input), Ok(()));
    assert_eq!(validate_hld_materialization_input(&input), Ok(()));
}

/// REQ-RUST-044.0, REQ-RUST-045.0
#[test]
fn test_full_graph_invariants_retained() {
    let input = build_full_graph_fixture();
    let mut invalid = input.clone();
    invalid.input_node_records[0].node_owner_kind_value = Some(HldOwnerKindValue::Other);
    assert_eq!(
        validate_full_graph_input(&invalid),
        Err(HldMaterializationValidationError::InvalidNodeCategory)
    );
    invalid = input.clone();
    invalid.input_node_records[0].node_owner_kind_value = Some(HldOwnerKindValue::FunctionBody);
    assert_eq!(
        validate_full_graph_input(&invalid),
        Err(HldMaterializationValidationError::InvalidNodeCategory)
    );
    invalid = input.clone();
    invalid.input_node_records[0].node_source_relative_path = Some("src/other.rs".into());
    assert_eq!(
        validate_full_graph_input(&invalid),
        Err(HldMaterializationValidationError::InvalidNodeCategory)
    );
    invalid = input.clone();
    invalid.input_node_records[0].run_generation_digest_value = RunGenerationDigestValue([1; 32]);
    assert_eq!(
        validate_full_graph_input(&invalid),
        Err(HldMaterializationValidationError::GenerationMismatch)
    );
    invalid = input.clone();
    invalid.input_unit_provenance_records.clear();
    assert_eq!(
        validate_full_graph_input(&invalid),
        Err(HldMaterializationValidationError::MissingUnitProvenance)
    );
    invalid = input.clone();
    invalid.input_edge_records[0].edge_resolution_value = HldEdgeResolutionValue::Extracted {
        target: NameSymbolTargetKey::Builtin("u8".into()).into(),
    };
    assert_eq!(
        validate_full_graph_input(&invalid),
        Err(HldMaterializationValidationError::MissingLocalEndpoint)
    );
    invalid = input;
    invalid.input_edge_records[0]
        .edge_evidence_row_key
        .evidence_relation_kind_value = HldRelationKindValue::UsesType;
    assert_eq!(
        validate_full_graph_input(&invalid),
        Err(HldMaterializationValidationError::InvalidEvidenceIdentity)
    );
}

/// REQ-RUST-045.0
#[test]
fn test_full_original_kinds_accepted() {
    for kind in [
        HldItemKindValue::Union,
        HldItemKindValue::MacroDefinition,
        HldItemKindValue::MacroInvocation,
        HldItemKindValue::ExternBlock,
        HldItemKindValue::ExternCrate,
        HldItemKindValue::InlineAssembly,
    ] {
        let mut input = build_full_graph_fixture();
        input.input_node_records[0].node_item_kind_value = Some(kind);
        assert_eq!(validate_full_graph_input(&input), Ok(()), "{kind:?}");
        assert_eq!(
            validate_hld_materialization_input(&input),
            Err(HldMaterializationValidationError::IncompleteTypedInput)
        );
    }
}

/// REQ-RUST-045.0
#[test]
fn test_full_foreign_ownership_retained() {
    for kind in [
        HldItemKindValue::Function,
        HldItemKindValue::Static,
        HldItemKindValue::TypeAlias,
    ] {
        let mut input = build_full_graph_fixture();
        input.input_node_records[0].node_item_kind_value = Some(kind);
        input.input_node_records[0].node_owner_kind_value = Some(HldOwnerKindValue::ExternBlock);
        assert_eq!(validate_full_graph_input(&input), Ok(()));
    }
}

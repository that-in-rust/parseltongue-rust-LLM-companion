#![allow(clippy::unwrap_used)]
use super::*;
use crate::{CompilationUnitRoleKind, SpanByteRangeValue, SpanDigestIdValue};

const EXPANDED: &str = ".parseltongue/macro-source/09/expanded.rs";

/// One file area for `src/lib.rs` that contains one item written by a macro expansion.
fn build_generated_node_fixture() -> HldMaterializationInputRecord {
    let unit = CrateCompilationUnitKey {
        unit_cargo_package_identifier: "fixture".into(),
        unit_cargo_target_name: "fixture".into(),
        unit_cargo_target_kind: "lib".into(),
        unit_compilation_role_kind: CompilationUnitRoleKind::Target,
        unit_configuration_digest_full: [4; 32],
    };
    let occurrence = SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
        &unit,
        EXPANDED,
        SpanByteRangeValue::try_from((0, 6)).unwrap(),
        SpanDigestIdValue::from([6; 32]),
        &[[9; 32]],
    )
    .unwrap();
    let item = HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(occurrence.clone()));
    let area = HldTargetIdentityValue::FilePublicArea(
        FileAreaIdentityKey::build_file_area_identity(&unit, "src/lib.rs").unwrap(),
    );
    let generation = RunGenerationDigestValue([0; 32]);
    HldMaterializationInputRecord {
        run_generation_digest_value: generation,
        stage_attempt_sequence_value: "attempt".into(),
        semantic_input_completeness_kind: HldInputCompletenessKind::IncompleteTypedInput,
        source_coverage_kind: HldSourceCoverageKind::FullSemantic,
        input_unit_provenance_records: vec![HldUnitProvenanceRecord {
            unit_compilation_identity_key: unit,
            unit_origin_kind: HldUnitOriginKind::Workspace,
        }],
        input_node_records: vec![
            HldNodeInputRecord {
                run_generation_digest_value: generation,
                node_target_reference: area.clone(),
                node_input_category: HldNodeInputCategory::FilePublicArea,
                node_item_kind_value: Some(HldItemKindValue::FilePublicArea),
                node_owner_kind_value: None,
                node_function_body_ancestry_flag: false,
                node_effective_visibility_kind: None,
                node_reachable_outside_crate_flag: None,
                node_source_relative_path: Some("src/lib.rs".into()),
                node_qualified_path_text: "fixture::src/lib.rs".into(),
                node_signature_text: None,
            },
            HldNodeInputRecord {
                run_generation_digest_value: generation,
                node_target_reference: item.clone(),
                node_input_category: HldNodeInputCategory::LocalItem,
                node_item_kind_value: Some(HldItemKindValue::Constant),
                node_owner_kind_value: Some(HldOwnerKindValue::CrateRoot),
                node_function_body_ancestry_flag: false,
                node_effective_visibility_kind: Some(HldEffectiveVisibilityKind::Unknown),
                node_reachable_outside_crate_flag: None,
                node_source_relative_path: Some("src/lib.rs".into()),
                node_qualified_path_text: "fixture::src/lib.rs::_".into(),
                node_signature_text: None,
            },
        ],
        input_edge_records: vec![HldEdgeInputRecord {
            run_generation_digest_value: generation,
            edge_source_reference: area.clone(),
            edge_relation_kind_value: HldRelationKindValue::Contains,
            edge_resolution_value: HldEdgeResolutionValue::Extracted {
                target: item.clone(),
            },
            edge_evidence_row_key: HldEvidenceRowKey {
                evidence_site_occurrence_key: occurrence,
                evidence_source_occurrence_key: area,
                evidence_relation_kind_value: HldRelationKindValue::Contains,
                evidence_target_reference_value: Some(item),
            },
            edge_dispatch_metadata_text: None,
        }],
    }
}

/// REQ-RUST-046.0 R3: a generated node names the file of its macro call, and that file's
/// area contains it, although the node's own text is in the expanded source.
#[test]
fn test_generated_node_call_file() {
    assert_eq!(
        validate_full_graph_input(&build_generated_node_fixture()),
        Ok(())
    );
}

/// REQ-RUST-046.0 R3: a generated node without a file is refused.
#[test]
fn test_generated_node_path_required() {
    let mut input = build_generated_node_fixture();
    input.input_node_records[1].node_source_relative_path = None;
    assert_eq!(
        validate_full_graph_input(&input),
        Err(HldMaterializationValidationError::InvalidNodeCategory)
    );
}

/// REQ-RUST-046.0 R3: the relaxation is for generated text only. An item in an original
/// file still has to name that file, and a file area still holds only sites in its file.
#[test]
fn test_original_node_path_exact() {
    let mut input = build_generated_node_fixture();
    let HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(generated)) =
        input.input_node_records[1].node_target_reference.clone()
    else {
        unreachable!()
    };
    let parts = generated.read_semantic_occurrence_parts();
    let original = SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
        parts.unit_compilation_identity_key,
        "src/other.rs",
        *parts.span_byte_range_value,
        *parts.span_content_digest_full,
        &[],
    )
    .unwrap();
    let item = HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(original.clone()));
    input.input_node_records[1].node_target_reference = item.clone();
    let edge = &mut input.input_edge_records[0];
    edge.edge_resolution_value = HldEdgeResolutionValue::Extracted {
        target: item.clone(),
    };
    edge.edge_evidence_row_key.evidence_site_occurrence_key = original;
    edge.edge_evidence_row_key.evidence_target_reference_value = Some(item);
    assert_eq!(
        validate_full_graph_input(&input),
        Err(HldMaterializationValidationError::InvalidNodeCategory)
    );
    input.input_node_records[1].node_source_relative_path = Some("src/other.rs".into());
    assert_eq!(
        validate_full_graph_input(&input),
        Err(HldMaterializationValidationError::InvalidEvidenceIdentity)
    );
}

use crate::{
    CrateCompilationUnitKey, NameSymbolTargetKey, RunGenerationDigestValue,
    SemanticOccurrenceKeyValue,
};
use std::collections::{BTreeMap, BTreeSet};

const HLD_BUILTIN_TYPE_NAMES: [&str; 19] = [
    "bool", "char", "str", "usize", "isize", "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32",
    "i64", "i128", "f16", "f32", "f64", "f128",
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldInputCompletenessKind {
    CompleteTypedInput,
    IncompleteTypedInput,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldNodeInputCategory {
    LocalItem,
    CrateRoot,
    ExternalDefinition,
    Builtin,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldUnitOriginKind {
    Workspace,
    StandardLibrary,
    ExternalCrate,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HldUnitProvenanceRecord {
    pub unit_compilation_identity_key: CrateCompilationUnitKey,
    pub unit_origin_kind: HldUnitOriginKind,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldItemKindValue {
    Module,
    Function,
    Struct,
    Enum,
    Trait,
    TypeAlias,
    Constant,
    Static,
    AssociatedFunction,
    AssociatedConstant,
    AssociatedType,
    EnumVariant,
    ImplContainer,
    UseStatement,
    Field,
    FunctionBodyItem,
    GenericParameter,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldOwnerKindValue {
    CrateRoot,
    Module,
    Trait,
    Impl,
    FunctionBody,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldEffectiveVisibilityKind {
    Public,
    RestrictedNonPrivate,
    PrivateEquivalent,
    Unknown,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HldNodeInputRecord {
    pub run_generation_digest_value: RunGenerationDigestValue,
    pub node_target_reference: NameSymbolTargetKey,
    pub node_input_category: HldNodeInputCategory,
    pub node_item_kind_value: Option<HldItemKindValue>,
    pub node_owner_kind_value: Option<HldOwnerKindValue>,
    pub node_function_body_ancestry_flag: bool,
    pub node_effective_visibility_kind: Option<HldEffectiveVisibilityKind>,
    pub node_reachable_outside_crate_flag: bool,
    pub node_source_relative_path: Option<String>,
    pub node_qualified_path_text: String,
    pub node_signature_text: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldRelationKindValue {
    Calls,
    UsesType,
    Implements,
    Reexports,
    Contains,
    Derives,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HldEvidenceRowKey {
    pub evidence_site_occurrence_key: SemanticOccurrenceKeyValue,
    pub evidence_source_occurrence_key: NameSymbolTargetKey,
    pub evidence_relation_kind_value: HldRelationKindValue,
    pub evidence_target_reference_value: Option<NameSymbolTargetKey>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldUnresolvedReasonKind {
    MissingTarget,
    AmbiguousTarget,
    CfgInactive,
    ExternalSourceMissing,
    TraitSolverNeeded,
    Other,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldEdgeResolutionValue {
    Extracted {
        target: NameSymbolTargetKey,
    },
    Unresolved {
        reason: HldUnresolvedReasonKind,
    },
    Inferred {
        candidate_target: Option<NameSymbolTargetKey>,
    },
    Proposed {
        candidate_target: Option<NameSymbolTargetKey>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HldEdgeInputRecord {
    pub run_generation_digest_value: RunGenerationDigestValue,
    pub edge_source_reference: NameSymbolTargetKey,
    pub edge_relation_kind_value: HldRelationKindValue,
    pub edge_resolution_value: HldEdgeResolutionValue,
    pub edge_evidence_row_key: HldEvidenceRowKey,
    pub edge_dispatch_metadata_text: Option<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HldMaterializationInputRecord {
    pub run_generation_digest_value: RunGenerationDigestValue,
    pub stage_attempt_sequence_value: String,
    pub semantic_input_completeness_kind: HldInputCompletenessKind,
    pub input_unit_provenance_records: Vec<HldUnitProvenanceRecord>,
    pub input_node_records: Vec<HldNodeInputRecord>,
    pub input_edge_records: Vec<HldEdgeInputRecord>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldViaKindValue {
    Direct,
    Collapsed,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldOutputEdgeStatus {
    Extracted,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldOutputCompletenessKind {
    DirectProjectionOnly,
    CompleteGraph,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldDependencyOriginKind {
    SameCrate,
    WorkspaceCrate,
    StandardLibrary,
    ExternalCrate,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HldSelectedNodeRecord {
    pub run_generation_digest_value: RunGenerationDigestValue,
    pub node_target_reference: NameSymbolTargetKey,
    pub node_qualified_path_text: String,
    pub node_source_relative_path: Option<String>,
    pub node_effective_visibility_kind: Option<HldEffectiveVisibilityKind>,
    pub node_reachable_outside_crate_flag: bool,
    pub node_signature_text: Option<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HldGraphEdgeKey {
    pub run_generation_digest_value: RunGenerationDigestValue,
    pub edge_source_occurrence_key: NameSymbolTargetKey,
    pub edge_target_reference_value: NameSymbolTargetKey,
    pub edge_relation_kind_value: HldRelationKindValue,
    pub edge_collapse_via_kind: HldViaKindValue,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HldMaterializedEdgeRecord {
    pub edge_key: HldGraphEdgeKey,
    pub edge_source_qualified_path: String,
    pub edge_target_qualified_path: String,
    pub edge_source_signature_text: Option<String>,
    pub edge_target_signature_text: Option<String>,
    pub edge_resolution_status_kind: HldOutputEdgeStatus,
    pub edge_evidence_row_count: usize,
    pub edge_dependency_origin_kind: HldDependencyOriginKind,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HldEvidencePathRecord {
    pub edge_key: HldGraphEdgeKey,
    pub ordered_endpoint_path: Vec<NameSymbolTargetKey>,
    pub ordered_evidence_row_keys: Vec<HldEvidenceRowKey>,
    pub ordered_site_occurrence_keys: Vec<SemanticOccurrenceKeyValue>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldEndpointRollupBucket {
    SourceFile {
        unit_compilation_identity_key: crate::CrateCompilationUnitKey,
        file_source_relative_path: String,
    },
    CrateRoot {
        unit_compilation_identity_key: crate::CrateCompilationUnitKey,
    },
    CanonicalExternalIdentity {
        endpoint_target_reference: NameSymbolTargetKey,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HldFileRollupRecord {
    pub source_endpoint_bucket: HldEndpointRollupBucket,
    pub target_endpoint_bucket: HldEndpointRollupBucket,
    pub edge_relation_kind_value: HldRelationKindValue,
    pub edge_collapse_via_kind: HldViaKindValue,
    pub edge_row_count: usize,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HldFolderRollupRecord {
    pub source_folder_bucket: HldEndpointRollupBucket,
    pub target_folder_bucket: HldEndpointRollupBucket,
    pub edge_relation_kind_value: HldRelationKindValue,
    pub edge_collapse_via_kind: HldViaKindValue,
    pub edge_row_count: usize,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HldMaterializationOutputRecord {
    pub run_generation_digest_value: RunGenerationDigestValue,
    pub output_completeness_kind: HldOutputCompletenessKind,
    pub selected_node_records: Vec<HldSelectedNodeRecord>,
    pub materialized_edge_records: Vec<HldMaterializedEdgeRecord>,
    pub evidence_path_records: Vec<HldEvidencePathRecord>,
    pub file_rollup_records: Option<Vec<HldFileRollupRecord>>,
    pub folder_rollup_records: Option<Vec<HldFolderRollupRecord>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HldMaterializationValidationError {
    IncompleteTypedInput,
    GenerationMismatch,
    MalformedCompilationUnitIdentity,
    MalformedSemanticOccurrenceIdentity,
    MalformedBuiltinIdentity,
    InvalidSourceRelativePath,
    InvalidNodeCategory,
    UnknownEffectiveVisibility,
    MissingLocalEndpoint,
    ConflictingNodeRecord,
    ConflictingEvidenceRecord,
    InvalidEdgeRecord,
    MissingUnitProvenance,
    ConflictingUnitProvenance,
    InvalidEvidenceIdentity,
}

impl std::fmt::Display for HldMaterializationValidationError {
    fn fmt(&self, error_display_format_sink: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_display_format_sink.write_str(match self {
            Self::IncompleteTypedInput => "typed semantic input is incomplete",
            Self::GenerationMismatch => "semantic input generation keys differ",
            Self::MalformedCompilationUnitIdentity => "compilation unit identity is malformed",
            Self::MalformedSemanticOccurrenceIdentity => {
                "semantic occurrence identity is malformed"
            }
            Self::MalformedBuiltinIdentity => "builtin identity is malformed",
            Self::InvalidSourceRelativePath => "source-relative path is invalid",
            Self::InvalidNodeCategory => "node category does not match its typed identity",
            Self::UnknownEffectiveVisibility => "effective visibility is unknown",
            Self::MissingLocalEndpoint => "resolved edge endpoint is missing",
            Self::ConflictingNodeRecord => "conflicting duplicate node record",
            Self::ConflictingEvidenceRecord => "conflicting duplicate evidence record",
            Self::InvalidEdgeRecord => "edge record is malformed",
            Self::MissingUnitProvenance => "compilation unit provenance is missing",
            Self::ConflictingUnitProvenance => "compilation unit provenance conflicts",
            Self::InvalidEvidenceIdentity => "evidence-row identity does not match its relation",
        })
    }
}

impl std::error::Error for HldMaterializationValidationError {}

pub fn validate_hld_materialization_input(
    input_record: &HldMaterializationInputRecord,
) -> Result<(), HldMaterializationValidationError> {
    if input_record.semantic_input_completeness_kind != HldInputCompletenessKind::CompleteTypedInput
    {
        return Err(HldMaterializationValidationError::IncompleteTypedInput);
    }

    let mut unit_origins = BTreeMap::new();
    for provenance_record in &input_record.input_unit_provenance_records {
        provenance_record
            .unit_compilation_identity_key
            .validate_crate_unit_identity()
            .map_err(|_| HldMaterializationValidationError::MalformedCompilationUnitIdentity)?;
        if unit_origins
            .insert(
                provenance_record.unit_compilation_identity_key.clone(),
                provenance_record.unit_origin_kind,
            )
            .is_some_and(|previous_origin| previous_origin != provenance_record.unit_origin_kind)
        {
            return Err(HldMaterializationValidationError::ConflictingUnitProvenance);
        }
    }

    let local_unit_identities = input_record.input_node_records.iter().try_fold(
        BTreeSet::new(),
        |mut unit_identities, node_record| {
            if matches!(
                node_record.node_input_category,
                HldNodeInputCategory::LocalItem | HldNodeInputCategory::CrateRoot
            ) {
                validate_hld_node_input(node_record)?;
                let unit_identity = read_hld_target_unit(&node_record.node_target_reference)?
                    .ok_or(HldMaterializationValidationError::InvalidNodeCategory)?;
                unit_identities.insert(unit_identity);
            }
            Ok::<_, HldMaterializationValidationError>(unit_identities)
        },
    )?;

    let mut node_records = BTreeMap::new();
    for node_record in &input_record.input_node_records {
        if node_record.run_generation_digest_value != input_record.run_generation_digest_value {
            return Err(HldMaterializationValidationError::GenerationMismatch);
        }
        validate_hld_node_input(node_record)?;
        if node_record.node_input_category == HldNodeInputCategory::ExternalDefinition
            && read_hld_target_unit(&node_record.node_target_reference)?
                .is_some_and(|unit_identity| local_unit_identities.contains(&unit_identity))
        {
            return Err(HldMaterializationValidationError::InvalidNodeCategory);
        }
        if let Some(unit_identity) = read_hld_target_unit(&node_record.node_target_reference)? {
            match unit_origins.get(&unit_identity) {
                None => return Err(HldMaterializationValidationError::MissingUnitProvenance),
                Some(HldUnitOriginKind::Workspace)
                | Some(HldUnitOriginKind::StandardLibrary)
                | Some(HldUnitOriginKind::ExternalCrate)
                    if node_record.node_input_category
                        == HldNodeInputCategory::ExternalDefinition => {}
                Some(HldUnitOriginKind::Workspace) => {}
                Some(_) => return Err(HldMaterializationValidationError::InvalidNodeCategory),
            }
        }
        if node_records
            .insert(node_record.node_target_reference.clone(), node_record)
            .is_some_and(|previous_record| previous_record != node_record)
        {
            return Err(HldMaterializationValidationError::ConflictingNodeRecord);
        }
    }

    let mut evidence_records = BTreeMap::new();
    for edge_record in &input_record.input_edge_records {
        if edge_record.run_generation_digest_value != input_record.run_generation_digest_value {
            return Err(HldMaterializationValidationError::GenerationMismatch);
        }
        validate_hld_edge_input(edge_record)?;
        let source_record = node_records
            .get(&edge_record.edge_source_reference)
            .ok_or(HldMaterializationValidationError::MissingLocalEndpoint)?;
        if !matches!(
            source_record.node_input_category,
            HldNodeInputCategory::LocalItem | HldNodeInputCategory::CrateRoot
        ) {
            return Err(HldMaterializationValidationError::InvalidEdgeRecord);
        }
        let target_reference = match &edge_record.edge_resolution_value {
            HldEdgeResolutionValue::Extracted { target } => {
                if !node_records.contains_key(target) {
                    return Err(HldMaterializationValidationError::MissingLocalEndpoint);
                }
                Some(target)
            }
            HldEdgeResolutionValue::Unresolved { .. } => None,
            HldEdgeResolutionValue::Inferred { candidate_target }
            | HldEdgeResolutionValue::Proposed { candidate_target } => candidate_target.as_ref(),
        };
        if edge_record
            .edge_evidence_row_key
            .evidence_source_occurrence_key
            != edge_record.edge_source_reference
            || edge_record
                .edge_evidence_row_key
                .evidence_relation_kind_value
                != edge_record.edge_relation_kind_value
            || edge_record
                .edge_evidence_row_key
                .evidence_target_reference_value
                .as_ref()
                != target_reference
        {
            return Err(HldMaterializationValidationError::InvalidEvidenceIdentity);
        }
        if let Some(target_reference) = target_reference {
            validate_hld_target_key(target_reference)?;
            if read_hld_target_unit(target_reference)?
                .is_some_and(|unit_identity| !unit_origins.contains_key(&unit_identity))
            {
                return Err(HldMaterializationValidationError::MissingUnitProvenance);
            }
        }
        validate_hld_occurrence_identity(
            &edge_record
                .edge_evidence_row_key
                .evidence_site_occurrence_key,
        )?;
        let site_unit_identity = edge_record
            .edge_evidence_row_key
            .evidence_site_occurrence_key
            .read_semantic_occurrence_parts()
            .unit_compilation_identity_key;
        if read_hld_target_unit(&edge_record.edge_source_reference)?.as_ref()
            != Some(site_unit_identity)
        {
            return Err(HldMaterializationValidationError::InvalidEvidenceIdentity);
        }
        if !unit_origins.contains_key(site_unit_identity) {
            return Err(HldMaterializationValidationError::MissingUnitProvenance);
        }
        if evidence_records
            .insert(&edge_record.edge_evidence_row_key, edge_record)
            .is_some_and(|previous_record| previous_record != edge_record)
        {
            return Err(HldMaterializationValidationError::ConflictingEvidenceRecord);
        }
    }
    Ok(())
}

fn validate_hld_node_input(
    node_record: &HldNodeInputRecord,
) -> Result<(), HldMaterializationValidationError> {
    if node_record.node_qualified_path_text.trim().is_empty() {
        return Err(HldMaterializationValidationError::InvalidNodeCategory);
    }
    match (
        &node_record.node_input_category,
        &node_record.node_target_reference,
    ) {
        (HldNodeInputCategory::LocalItem, NameSymbolTargetKey::Definition(occurrence_key)) => {
            validate_hld_occurrence_identity(occurrence_key)?;
            if node_record.node_source_relative_path.as_deref()
                != Some(
                    occurrence_key
                        .read_semantic_occurrence_parts()
                        .span_source_file_path,
                )
                || node_record.node_item_kind_value.is_none()
                || node_record.node_item_kind_value == Some(HldItemKindValue::Unknown)
                || node_record.node_owner_kind_value.is_none()
            {
                return Err(HldMaterializationValidationError::InvalidNodeCategory);
            }
            if matches!(
                node_record.node_item_kind_value,
                Some(HldItemKindValue::Field | HldItemKindValue::EnumVariant)
            ) || node_record.node_owner_kind_value == Some(HldOwnerKindValue::Other)
                || (node_record.node_owner_kind_value == Some(HldOwnerKindValue::FunctionBody)
                    && !node_record.node_function_body_ancestry_flag)
            {
                return Err(HldMaterializationValidationError::InvalidNodeCategory);
            }
            match node_record.node_effective_visibility_kind {
                Some(HldEffectiveVisibilityKind::Unknown) | None => {
                    Err(HldMaterializationValidationError::UnknownEffectiveVisibility)
                }
                Some(_) => Ok(()),
            }
        }
        (HldNodeInputCategory::CrateRoot, NameSymbolTargetKey::CrateRoot(unit_identity)) => {
            unit_identity
                .validate_crate_unit_identity()
                .map_err(|_| HldMaterializationValidationError::MalformedCompilationUnitIdentity)?;
            if node_record.node_item_kind_value.is_some()
                || node_record.node_owner_kind_value.is_some()
                || node_record.node_function_body_ancestry_flag
                || node_record.node_effective_visibility_kind.is_some()
                || node_record.node_source_relative_path.is_some()
            {
                return Err(HldMaterializationValidationError::InvalidNodeCategory);
            }
            Ok(())
        }
        (
            HldNodeInputCategory::ExternalDefinition,
            NameSymbolTargetKey::Definition(occurrence_key),
        ) => {
            validate_hld_occurrence_identity(occurrence_key)?;
            if node_record.node_item_kind_value.is_some()
                || node_record.node_owner_kind_value.is_some()
                || node_record.node_function_body_ancestry_flag
                || node_record.node_effective_visibility_kind.is_some()
                || node_record
                    .node_source_relative_path
                    .as_deref()
                    .is_some_and(|path| {
                        path != occurrence_key
                            .read_semantic_occurrence_parts()
                            .span_source_file_path
                    })
            {
                return Err(HldMaterializationValidationError::InvalidNodeCategory);
            }
            Ok(())
        }
        (HldNodeInputCategory::Builtin, NameSymbolTargetKey::Builtin(name)) => {
            if !is_hld_builtin_type(name)
                || node_record.node_item_kind_value.is_some()
                || node_record.node_owner_kind_value.is_some()
                || node_record.node_function_body_ancestry_flag
                || node_record.node_effective_visibility_kind.is_some()
                || node_record.node_source_relative_path.is_some()
            {
                return Err(HldMaterializationValidationError::MalformedBuiltinIdentity);
            }
            Ok(())
        }
        _ => Err(HldMaterializationValidationError::InvalidNodeCategory),
    }
}

fn validate_hld_edge_input(
    edge_record: &HldEdgeInputRecord,
) -> Result<(), HldMaterializationValidationError> {
    validate_hld_target_key(&edge_record.edge_source_reference)?;
    if edge_record
        .edge_dispatch_metadata_text
        .as_ref()
        .is_some_and(|text| text.trim().is_empty())
    {
        return Err(HldMaterializationValidationError::InvalidEdgeRecord);
    }
    validate_hld_occurrence_identity(
        &edge_record
            .edge_evidence_row_key
            .evidence_site_occurrence_key,
    )
}

fn validate_hld_target_key(
    target_reference: &NameSymbolTargetKey,
) -> Result<(), HldMaterializationValidationError> {
    match target_reference {
        NameSymbolTargetKey::Definition(occurrence_key) => {
            validate_hld_occurrence_identity(occurrence_key)
        }
        NameSymbolTargetKey::GenericParameter(occurrence_key, parameter_name) => {
            if parameter_name.trim().is_empty() {
                return Err(HldMaterializationValidationError::MalformedSemanticOccurrenceIdentity);
            }
            validate_hld_occurrence_identity(occurrence_key)
        }
        NameSymbolTargetKey::Builtin(name) => {
            if !is_hld_builtin_type(name) {
                Err(HldMaterializationValidationError::MalformedBuiltinIdentity)
            } else {
                Ok(())
            }
        }
        NameSymbolTargetKey::CrateRoot(unit_identity) => unit_identity
            .validate_crate_unit_identity()
            .map_err(|_| HldMaterializationValidationError::MalformedCompilationUnitIdentity),
    }
}

fn is_hld_builtin_type(name: &str) -> bool {
    HLD_BUILTIN_TYPE_NAMES.contains(&name)
}

fn validate_hld_occurrence_identity(
    occurrence_key: &SemanticOccurrenceKeyValue,
) -> Result<(), HldMaterializationValidationError> {
    let occurrence_parts = occurrence_key.read_semantic_occurrence_parts();
    SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
        occurrence_parts.unit_compilation_identity_key,
        occurrence_parts.span_source_file_path,
        *occurrence_parts.span_byte_range_value,
        *occurrence_parts.span_content_digest_full,
        occurrence_parts.span_expansion_provenance_chain,
    )
    .map(|_| ())
    .map_err(|_| HldMaterializationValidationError::MalformedSemanticOccurrenceIdentity)
}

fn read_hld_target_unit(
    target_reference: &NameSymbolTargetKey,
) -> Result<Option<CrateCompilationUnitKey>, HldMaterializationValidationError> {
    let unit_identity = match target_reference {
        NameSymbolTargetKey::Definition(occurrence_key)
        | NameSymbolTargetKey::GenericParameter(occurrence_key, _) => occurrence_key
            .read_semantic_occurrence_parts()
            .unit_compilation_identity_key
            .clone(),
        NameSymbolTargetKey::CrateRoot(unit_identity) => unit_identity.clone(),
        NameSymbolTargetKey::Builtin(_) => return Ok(None),
    };
    unit_identity
        .validate_crate_unit_identity()
        .map_err(|_| HldMaterializationValidationError::MalformedCompilationUnitIdentity)?;
    Ok(Some(unit_identity))
}

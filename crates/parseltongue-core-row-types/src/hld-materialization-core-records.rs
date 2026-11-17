use crate::{
    CrateCompilationUnitKey, NameSymbolTargetKey, RunGenerationDigestValue,
    SemanticOccurrenceKeyValue,
};
use std::collections::{BTreeMap, BTreeSet};

const HLD_BUILTIN_TYPE_NAMES: [&str; 19] = [
    "bool", "char", "str", "usize", "isize", "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32",
    "i64", "i128", "f16", "f32", "f64", "f128",
];

#[cfg(test)]
#[path = "full-graph-validation-tests.rs"]
mod full_graph_validation_tests;

#[cfg(test)]
#[path = "generated-node-validation-tests.rs"]
mod generated_node_validation_tests;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldInputCompletenessKind {
    CompleteTypedInput,
    IncompleteTypedInput,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FileAreaIdentityKey {
    unit_compilation_identity_key: CrateCompilationUnitKey,
    file_source_relative_path: String,
}

impl FileAreaIdentityKey {
    pub fn build_file_area_identity(
        unit_compilation_identity_key: &CrateCompilationUnitKey,
        file_source_relative_path: &str,
    ) -> Result<Self, FileAreaIdentityError> {
        unit_compilation_identity_key
            .validate_crate_unit_identity()
            .map_err(|_| FileAreaIdentityError::InvalidCompilationUnitIdentity)?;
        Ok(Self {
            unit_compilation_identity_key: unit_compilation_identity_key.clone(),
            file_source_relative_path: normalize_file_area_path(file_source_relative_path)?,
        })
    }

    pub fn read_unit_identity_value(&self) -> &CrateCompilationUnitKey {
        &self.unit_compilation_identity_key
    }

    pub fn read_source_relative_path(&self) -> &str {
        &self.file_source_relative_path
    }
}

fn normalize_file_area_path(
    file_source_relative_path: &str,
) -> Result<String, FileAreaIdentityError> {
    if file_source_relative_path.starts_with('/')
        || file_source_relative_path.contains(['\\', ':', '\0'])
    {
        return Err(FileAreaIdentityError::InvalidSourceRelativePath);
    }
    let path_components = file_source_relative_path
        .split('/')
        .filter(|component| !component.is_empty() && *component != ".")
        .collect::<Vec<_>>();
    if path_components.is_empty() || path_components.contains(&"..") {
        return Err(FileAreaIdentityError::InvalidSourceRelativePath);
    }
    Ok(path_components.join("/"))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileAreaIdentityError {
    InvalidCompilationUnitIdentity,
    InvalidSourceRelativePath,
}

impl std::fmt::Display for FileAreaIdentityError {
    fn fmt(&self, error_display_format_sink: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_display_format_sink.write_str(match self {
            Self::InvalidCompilationUnitIdentity => "file area unit identity is invalid",
            Self::InvalidSourceRelativePath => "file area source path is invalid",
        })
    }
}

impl std::error::Error for FileAreaIdentityError {}

/// REQ-RUST-053.0 N3: what in read source proves the name of an external item.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldExternalNameKind {
    /// A path that starts at the name of a crate with no unit in the run.
    CrateRootedPath,
    /// One name that only the implicit prelude can bind.
    PreludeName,
    /// A member reached through a builtin type that no read source declares.
    BuiltinMember,
    /// REQ-RUST-056.0: a member that the read declarations of `core` prove: the self
    /// type as `core` writes it, `::`, the member's name.
    CoreDeclaredMember,
}

/// REQ-RUST-053.0 N1: an item outside the read source, known by the path the source writes.
/// It claims no definition location and no span.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HldExternalNamedKey {
    pub external_name_kind_value: HldExternalNameKind,
    pub external_crate_name_value: Option<String>,
    pub external_item_path_text: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldTargetIdentityValue {
    Symbol(NameSymbolTargetKey),
    FilePublicArea(FileAreaIdentityKey),
    ExternalNamed(HldExternalNamedKey),
}

impl From<NameSymbolTargetKey> for HldTargetIdentityValue {
    fn from(symbol_identity: NameSymbolTargetKey) -> Self {
        Self::Symbol(symbol_identity)
    }
}

impl From<&NameSymbolTargetKey> for HldTargetIdentityValue {
    fn from(symbol_identity: &NameSymbolTargetKey) -> Self {
        Self::Symbol(symbol_identity.clone())
    }
}

impl From<&HldTargetIdentityValue> for HldTargetIdentityValue {
    fn from(target_identity: &HldTargetIdentityValue) -> Self {
        target_identity.clone()
    }
}

impl PartialEq<NameSymbolTargetKey> for HldTargetIdentityValue {
    fn eq(&self, symbol_identity: &NameSymbolTargetKey) -> bool {
        matches!(self, Self::Symbol(target_symbol_identity) if target_symbol_identity == symbol_identity)
    }
}

impl PartialEq<HldTargetIdentityValue> for NameSymbolTargetKey {
    fn eq(&self, target_identity: &HldTargetIdentityValue) -> bool {
        target_identity == self
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldSourceCoverageKind {
    FullSemantic,
    OriginalTopLevelContainment,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldNodeInputCategory {
    LocalItem,
    FilePublicArea,
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
    Union,
    MacroDefinition,
    MacroInvocation,
    ExternBlock,
    ExternCrate,
    InlineAssembly,
    FilePublicArea,
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
    ExternBlock,
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
    /// REQ-RUST-069.0 V5: a macro call has no visibility to declare.
    NotApplicable,
    Unknown,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HldNodeInputRecord {
    pub run_generation_digest_value: RunGenerationDigestValue,
    pub node_target_reference: HldTargetIdentityValue,
    pub node_input_category: HldNodeInputCategory,
    pub node_item_kind_value: Option<HldItemKindValue>,
    pub node_owner_kind_value: Option<HldOwnerKindValue>,
    pub node_function_body_ancestry_flag: bool,
    pub node_effective_visibility_kind: Option<HldEffectiveVisibilityKind>,
    pub node_reachable_outside_crate_flag: Option<bool>,
    pub node_source_relative_path: Option<String>,
    pub node_qualified_path_text: String,
    pub node_signature_text: Option<String>,
    /// The macro call whose expansion wrote this item; `None` for source-written items.
    pub node_generating_call_reference: Option<HldTargetIdentityValue>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldRelationKindValue {
    Calls,
    UsesType,
    Imports,
    Reads,
    Implements,
    Reexports,
    Contains,
    Derives,
    /// A macro call wrote this item: macro call node to generated item, sited at the call.
    Generates,
    /// A macro call expands this definition: macro call node to macro definition node,
    /// sited at the call.
    Expands,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HldEvidenceRowKey {
    pub evidence_site_occurrence_key: SemanticOccurrenceKeyValue,
    pub evidence_source_occurrence_key: HldTargetIdentityValue,
    pub evidence_relation_kind_value: HldRelationKindValue,
    pub evidence_target_reference_value: Option<HldTargetIdentityValue>,
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
        target: HldTargetIdentityValue,
    },
    Unresolved {
        reason: HldUnresolvedReasonKind,
    },
    Inferred {
        candidate_target: Option<HldTargetIdentityValue>,
    },
    Proposed {
        candidate_target: Option<HldTargetIdentityValue>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HldEdgeInputRecord {
    pub run_generation_digest_value: RunGenerationDigestValue,
    pub edge_source_reference: HldTargetIdentityValue,
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
    pub source_coverage_kind: HldSourceCoverageKind,
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
    GraphPathsOnly,
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
    pub node_target_reference: HldTargetIdentityValue,
    pub node_qualified_path_text: String,
    pub node_source_relative_path: Option<String>,
    pub node_effective_visibility_kind: Option<HldEffectiveVisibilityKind>,
    pub node_reachable_outside_crate_flag: Option<bool>,
    pub node_signature_text: Option<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HldGraphEdgeKey {
    pub run_generation_digest_value: RunGenerationDigestValue,
    pub edge_source_occurrence_key: HldTargetIdentityValue,
    pub edge_target_reference_value: HldTargetIdentityValue,
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
    pub ordered_endpoint_path: Vec<HldTargetIdentityValue>,
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
        endpoint_target_reference: HldTargetIdentityValue,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum HldFolderEndpointBucket {
    SourceFolder {
        unit_compilation_identity_key: crate::CrateCompilationUnitKey,
        folder_source_relative_path: String,
    },
    CrateRoot {
        unit_compilation_identity_key: crate::CrateCompilationUnitKey,
    },
    CanonicalExternalIdentity {
        endpoint_target_reference: HldTargetIdentityValue,
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
    pub source_folder_bucket: HldFolderEndpointBucket,
    pub target_folder_bucket: HldFolderEndpointBucket,
    pub edge_relation_kind_value: HldRelationKindValue,
    pub edge_collapse_via_kind: HldViaKindValue,
    pub edge_row_count: usize,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HldMaterializationOutputRecord {
    pub run_generation_digest_value: RunGenerationDigestValue,
    pub output_completeness_kind: HldOutputCompletenessKind,
    pub source_coverage_kind: HldSourceCoverageKind,
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
    MalformedExternalIdentity,
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
    StructuralScopeAllowsContainsOnly,
    UnknownReachability,
    InvalidGeneratingCall,
    InvalidExpandsEdge,
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
            Self::MalformedExternalIdentity => "named external identity is malformed",
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
            Self::InvalidGeneratingCall => {
                "a generated node must name a macro call node; a source-written node names none"
            }
            Self::InvalidExpandsEdge => {
                "an expands row goes from a macro call node to a macro definition node and is sited at the call"
            }
            Self::StructuralScopeAllowsContainsOnly => {
                "original top-level scope allows only extracted contains edges"
            }
            Self::UnknownReachability => "full semantic local-item reachability is unknown",
        })
    }
}

impl std::error::Error for HldMaterializationValidationError {}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::items_after_test_module)]
mod file_area_identity_tests {
    use super::*;

    fn build_test_unit_identity(digest: u8) -> CrateCompilationUnitKey {
        CrateCompilationUnitKey {
            unit_cargo_package_identifier: "fixture-package".into(),
            unit_cargo_target_name: "fixture-package".into(),
            unit_cargo_target_kind: "lib".into(),
            unit_compilation_role_kind: crate::CompilationUnitRoleKind::Target,
            unit_configuration_digest_full: [digest; 32],
        }
    }

    /// REQ-RUST-040.0
    #[test]
    fn test_file_area_path_normalizes() {
        let identity = FileAreaIdentityKey::build_file_area_identity(
            &build_test_unit_identity(1),
            "src//./lib.rs",
        )
        .unwrap();
        assert_eq!(identity.read_source_relative_path(), "src/lib.rs");
        assert_eq!(
            FileAreaIdentityKey::build_file_area_identity(
                &build_test_unit_identity(1),
                "../src/lib.rs",
            ),
            Err(FileAreaIdentityError::InvalidSourceRelativePath)
        );
    }

    /// REQ-RUST-040.0
    #[test]
    fn test_file_area_units_differ() {
        let first = FileAreaIdentityKey::build_file_area_identity(
            &build_test_unit_identity(1),
            "src/lib.rs",
        )
        .unwrap();
        let second = FileAreaIdentityKey::build_file_area_identity(
            &build_test_unit_identity(2),
            "src/lib.rs",
        )
        .unwrap();
        assert_ne!(first, second);
    }

    /// REQ-RUST-040.0
    #[test]
    fn test_file_area_invalid_unit_rejected() {
        let mut unit = build_test_unit_identity(1);
        unit.unit_cargo_package_identifier.clear();
        assert_eq!(
            FileAreaIdentityKey::build_file_area_identity(&unit, "src/lib.rs"),
            Err(FileAreaIdentityError::InvalidCompilationUnitIdentity)
        );
    }
}

/// Validate canonical full-map facts without requiring HLD qualification.
pub fn validate_full_graph_input(
    input_record: &HldMaterializationInputRecord,
) -> Result<(), HldMaterializationValidationError> {
    validate_graph_input_records(input_record)
}

pub fn validate_hld_materialization_input(
    input_record: &HldMaterializationInputRecord,
) -> Result<(), HldMaterializationValidationError> {
    if input_record.semantic_input_completeness_kind != HldInputCompletenessKind::CompleteTypedInput
    {
        return Err(HldMaterializationValidationError::IncompleteTypedInput);
    }
    validate_graph_input_records(input_record)
}

fn validate_graph_input_records(
    input_record: &HldMaterializationInputRecord,
) -> Result<(), HldMaterializationValidationError> {
    if input_record.source_coverage_kind == HldSourceCoverageKind::OriginalTopLevelContainment
        && input_record.input_edge_records.iter().any(|edge| {
            edge.edge_relation_kind_value != HldRelationKindValue::Contains
                || !matches!(
                    edge.edge_resolution_value,
                    HldEdgeResolutionValue::Extracted { .. }
                )
        })
    {
        return Err(HldMaterializationValidationError::StructuralScopeAllowsContainsOnly);
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
                HldNodeInputCategory::LocalItem
                    | HldNodeInputCategory::FilePublicArea
                    | HldNodeInputCategory::CrateRoot
            ) {
                validate_hld_node_input(
                    node_record,
                    input_record.source_coverage_kind,
                    input_record.semantic_input_completeness_kind,
                )?;
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
        validate_hld_node_input(
            node_record,
            input_record.source_coverage_kind,
            input_record.semantic_input_completeness_kind,
        )?;
        if input_record.source_coverage_kind == HldSourceCoverageKind::OriginalTopLevelContainment
            && !matches!(
                node_record.node_input_category,
                HldNodeInputCategory::FilePublicArea | HldNodeInputCategory::LocalItem
            )
        {
            return Err(HldMaterializationValidationError::InvalidNodeCategory);
        }
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

    // A node written by a macro expansion names its macro call, and that call is a node.
    if node_records.values().any(|node_record| {
        let generated = matches!(
            &node_record.node_target_reference,
            HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(key))
                if !key
                    .read_semantic_occurrence_parts()
                    .span_expansion_provenance_chain
                    .is_empty()
        );
        match &node_record.node_generating_call_reference {
            Some(call) => {
                !generated
                    || node_records.get(call).is_none_or(|call_record| {
                        call_record.node_item_kind_value != Some(HldItemKindValue::MacroInvocation)
                    })
            }
            None => generated,
        }
    }) {
        return Err(HldMaterializationValidationError::InvalidGeneratingCall);
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
        // An impl block is not a node, so `impl Trait for u8` is sourced at the builtin.
        let builtin_source = source_record.node_input_category == HldNodeInputCategory::Builtin
            && edge_record.edge_relation_kind_value == HldRelationKindValue::Implements;
        if !builtin_source
            && !matches!(
                source_record.node_input_category,
                HldNodeInputCategory::LocalItem
                    | HldNodeInputCategory::FilePublicArea
                    | HldNodeInputCategory::CrateRoot
            )
        {
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
        // A `generates` row goes from the call its target names, and is sited at that call.
        if edge_record.edge_relation_kind_value == HldRelationKindValue::Generates
            && (!matches!(
                &edge_record.edge_resolution_value,
                HldEdgeResolutionValue::Extracted { target }
                    if node_records.get(target).is_some_and(|target_record| {
                        target_record.node_generating_call_reference.as_ref()
                            == Some(&edge_record.edge_source_reference)
                    })
            ) || !matches!(
                &edge_record.edge_source_reference,
                HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(call))
                    if *call == edge_record.edge_evidence_row_key.evidence_site_occurrence_key
            ))
        {
            return Err(HldMaterializationValidationError::InvalidGeneratingCall);
        }
        if edge_record.edge_relation_kind_value == HldRelationKindValue::Expands
            && !check_expands_edge_record(edge_record, source_record, &node_records)
        {
            return Err(HldMaterializationValidationError::InvalidExpandsEdge);
        }
        if input_record.source_coverage_kind == HldSourceCoverageKind::OriginalTopLevelContainment {
            let source_area = match &edge_record.edge_source_reference {
                HldTargetIdentityValue::FilePublicArea(file_area) => file_area,
                _ => return Err(HldMaterializationValidationError::InvalidEdgeRecord),
            };
            let target_occurrence = match target_reference {
                Some(HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(
                    occurrence,
                ))) => occurrence,
                _ => return Err(HldMaterializationValidationError::InvalidEvidenceIdentity),
            };
            let target_parts = target_occurrence.read_semantic_occurrence_parts();
            let site_occurrence = &edge_record
                .edge_evidence_row_key
                .evidence_site_occurrence_key;
            if target_parts.unit_compilation_identity_key != source_area.read_unit_identity_value()
                || target_parts.span_source_file_path != source_area.read_source_relative_path()
                || !target_parts.span_expansion_provenance_chain.is_empty()
                || target_occurrence != site_occurrence
            {
                return Err(HldMaterializationValidationError::InvalidEvidenceIdentity);
            }
        }
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
        if read_hld_target_unit(&edge_record.edge_source_reference)?
            .map_or(!builtin_source, |unit| &unit != site_unit_identity)
        {
            return Err(HldMaterializationValidationError::InvalidEvidenceIdentity);
        }
        if !unit_origins.contains_key(site_unit_identity) {
            return Err(HldMaterializationValidationError::MissingUnitProvenance);
        }
        let site_parts = edge_record
            .edge_evidence_row_key
            .evidence_site_occurrence_key
            .read_semantic_occurrence_parts();
        // A file area holds sites in its own file, and items its macro calls expand to.
        if matches!(
            edge_record.edge_source_reference,
            HldTargetIdentityValue::FilePublicArea(_)
        ) && site_parts.span_expansion_provenance_chain.is_empty()
            && read_hld_target_path(&edge_record.edge_source_reference)
                .is_some_and(|source_path| source_path != site_parts.span_source_file_path)
        {
            return Err(HldMaterializationValidationError::InvalidEvidenceIdentity);
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
    source_coverage_kind: HldSourceCoverageKind,
    input_completeness_kind: HldInputCompletenessKind,
) -> Result<(), HldMaterializationValidationError> {
    if node_record.node_qualified_path_text.trim().is_empty() {
        return Err(HldMaterializationValidationError::InvalidNodeCategory);
    }
    match (
        &node_record.node_input_category,
        &node_record.node_target_reference,
    ) {
        (
            HldNodeInputCategory::LocalItem,
            HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(occurrence_key)),
        ) => {
            validate_hld_occurrence_identity(occurrence_key)?;
            let occurrence_parts = occurrence_key.read_semantic_occurrence_parts();
            // An item written by a macro expansion names the file of its macro call; its own
            // text is in the expanded source named by its identity.
            let source_path_valid = match node_record.node_source_relative_path.as_deref() {
                Some(_) if !occurrence_parts.span_expansion_provenance_chain.is_empty() => true,
                source_path => source_path == Some(occurrence_parts.span_source_file_path),
            };
            if !source_path_valid
                || node_record.node_item_kind_value.is_none()
                || node_record.node_item_kind_value == Some(HldItemKindValue::Unknown)
                || node_record.node_owner_kind_value.is_none()
                || node_record.node_item_kind_value == Some(HldItemKindValue::FilePublicArea)
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
            if source_coverage_kind == HldSourceCoverageKind::OriginalTopLevelContainment
                && (node_record.node_function_body_ancestry_flag
                    || !occurrence_key
                        .read_semantic_occurrence_parts()
                        .span_expansion_provenance_chain
                        .is_empty()
                    || !matches!(
                        node_record.node_owner_kind_value,
                        Some(HldOwnerKindValue::CrateRoot | HldOwnerKindValue::Module)
                    )
                    || matches!(
                        node_record.node_item_kind_value,
                        Some(
                            HldItemKindValue::AssociatedFunction
                                | HldItemKindValue::AssociatedConstant
                                | HldItemKindValue::AssociatedType
                                | HldItemKindValue::ImplContainer
                                | HldItemKindValue::FunctionBodyItem
                                | HldItemKindValue::GenericParameter
                        )
                    ))
            {
                return Err(HldMaterializationValidationError::InvalidNodeCategory);
            }
            match node_record.node_effective_visibility_kind {
                Some(_)
                    if input_completeness_kind
                        == HldInputCompletenessKind::IncompleteTypedInput =>
                {
                    Ok(())
                }
                Some(HldEffectiveVisibilityKind::Unknown) | None => {
                    Err(HldMaterializationValidationError::UnknownEffectiveVisibility)
                }
                Some(_)
                    if source_coverage_kind == HldSourceCoverageKind::FullSemantic
                        && node_record.node_reachable_outside_crate_flag.is_none() =>
                {
                    Err(HldMaterializationValidationError::UnknownReachability)
                }
                Some(_) => Ok(()),
            }
        }
        (
            HldNodeInputCategory::FilePublicArea,
            HldTargetIdentityValue::FilePublicArea(file_area_identity),
        ) => {
            if node_record.node_item_kind_value != Some(HldItemKindValue::FilePublicArea)
                || node_record.node_owner_kind_value.is_some()
                || node_record.node_function_body_ancestry_flag
                || node_record.node_effective_visibility_kind.is_some()
                || node_record.node_reachable_outside_crate_flag.is_some()
                || node_record.node_signature_text.is_some()
                || node_record.node_source_relative_path.as_deref()
                    != Some(file_area_identity.read_source_relative_path())
            {
                return Err(HldMaterializationValidationError::InvalidNodeCategory);
            }
            Ok(())
        }
        (
            HldNodeInputCategory::CrateRoot,
            HldTargetIdentityValue::Symbol(NameSymbolTargetKey::CrateRoot(unit_identity)),
        ) => {
            unit_identity
                .validate_crate_unit_identity()
                .map_err(|_| HldMaterializationValidationError::MalformedCompilationUnitIdentity)?;
            if node_record.node_item_kind_value.is_some()
                || node_record.node_owner_kind_value.is_some()
                || node_record.node_function_body_ancestry_flag
                || node_record.node_effective_visibility_kind.is_some()
                || node_record.node_reachable_outside_crate_flag.is_some()
                || node_record.node_source_relative_path.is_some()
            {
                return Err(HldMaterializationValidationError::InvalidNodeCategory);
            }
            Ok(())
        }
        (
            HldNodeInputCategory::ExternalDefinition,
            HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(occurrence_key)),
        ) => {
            validate_hld_occurrence_identity(occurrence_key)?;
            if node_record.node_item_kind_value.is_some()
                || node_record.node_owner_kind_value.is_some()
                || node_record.node_function_body_ancestry_flag
                || node_record.node_effective_visibility_kind.is_some()
                || node_record.node_reachable_outside_crate_flag.is_some()
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
        (
            HldNodeInputCategory::ExternalDefinition,
            HldTargetIdentityValue::ExternalNamed(external_key),
        ) => {
            validate_hld_external_key(external_key)?;
            if node_record.node_item_kind_value.is_some()
                || node_record.node_owner_kind_value.is_some()
                || node_record.node_function_body_ancestry_flag
                || node_record.node_effective_visibility_kind.is_some()
                || node_record.node_reachable_outside_crate_flag.is_some()
                || node_record.node_source_relative_path.is_some()
                || node_record.node_signature_text.is_some()
                || node_record.node_generating_call_reference.is_some()
                || node_record.node_qualified_path_text != external_key.external_item_path_text
            {
                return Err(HldMaterializationValidationError::MalformedExternalIdentity);
            }
            Ok(())
        }
        (
            HldNodeInputCategory::Builtin,
            HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Builtin(name)),
        ) => {
            if !is_hld_builtin_type(name)
                || node_record.node_item_kind_value.is_some()
                || node_record.node_owner_kind_value.is_some()
                || node_record.node_function_body_ancestry_flag
                || node_record.node_effective_visibility_kind.is_some()
                || node_record.node_reachable_outside_crate_flag.is_some()
                || node_record.node_source_relative_path.is_some()
            {
                return Err(HldMaterializationValidationError::MalformedBuiltinIdentity);
            }
            Ok(())
        }
        _ => Err(HldMaterializationValidationError::InvalidNodeCategory),
    }
}

/// REQ-RUST-053.0 N1, N3: the shape each kind of external name must have.
fn validate_hld_external_key(
    external_key: &HldExternalNamedKey,
) -> Result<(), HldMaterializationValidationError> {
    let segments = external_key
        .external_item_path_text
        .split("::")
        .collect::<Vec<_>>();
    let well_formed = segments.iter().all(|segment| {
        !segment.is_empty()
            && !segment.contains(|letter: char| letter.is_whitespace() || letter == ':')
    });
    let crate_name = external_key.external_crate_name_value.as_deref();
    // The self type of a `core` member is written text and may hold spaces: `*mut T`.
    let well_formed = well_formed
        || (external_key.external_name_kind_value == HldExternalNameKind::CoreDeclaredMember
            && external_key
                .external_item_path_text
                .rsplit_once("::")
                .is_some_and(|(owner, member)| {
                    !owner.trim().is_empty()
                        && !member.is_empty()
                        && !member.contains(|letter: char| letter.is_whitespace() || letter == ':')
                }));
    let shape_holds = match external_key.external_name_kind_value {
        HldExternalNameKind::CoreDeclaredMember => crate_name == Some("core"),
        HldExternalNameKind::CrateRootedPath => crate_name == segments.first().copied(),
        HldExternalNameKind::PreludeName => crate_name.is_none() && segments.len() == 1,
        HldExternalNameKind::BuiltinMember => {
            crate_name.is_none()
                && segments.len() == 2
                && segments
                    .first()
                    .is_some_and(|name| is_hld_builtin_type(name))
        }
    };
    if well_formed && shape_holds {
        Ok(())
    } else {
        Err(HldMaterializationValidationError::MalformedExternalIdentity)
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
    target_reference: &HldTargetIdentityValue,
) -> Result<(), HldMaterializationValidationError> {
    match target_reference {
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(occurrence_key)) => {
            validate_hld_occurrence_identity(occurrence_key)
        }
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::GenericParameter(
            occurrence_key,
            parameter_name,
        )) => {
            if parameter_name.trim().is_empty() {
                return Err(HldMaterializationValidationError::MalformedSemanticOccurrenceIdentity);
            }
            validate_hld_occurrence_identity(occurrence_key)
        }
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Builtin(name)) => {
            if !is_hld_builtin_type(name) {
                Err(HldMaterializationValidationError::MalformedBuiltinIdentity)
            } else {
                Ok(())
            }
        }
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::CrateRoot(unit_identity)) => {
            unit_identity
                .validate_crate_unit_identity()
                .map_err(|_| HldMaterializationValidationError::MalformedCompilationUnitIdentity)
        }
        HldTargetIdentityValue::ExternalNamed(external_key) => {
            validate_hld_external_key(external_key)
        }
        HldTargetIdentityValue::FilePublicArea(file_area_identity) => {
            FileAreaIdentityKey::build_file_area_identity(
                file_area_identity.read_unit_identity_value(),
                file_area_identity.read_source_relative_path(),
            )
            .map(|_| ())
            .map_err(|_| HldMaterializationValidationError::InvalidSourceRelativePath)
        }
    }
}

/// Whether `name` is a builtin type the graph has a node for.
pub fn is_hld_builtin_type(name: &str) -> bool {
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
    target_reference: &HldTargetIdentityValue,
) -> Result<Option<CrateCompilationUnitKey>, HldMaterializationValidationError> {
    let unit_identity = match target_reference {
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(occurrence_key))
        | HldTargetIdentityValue::Symbol(NameSymbolTargetKey::GenericParameter(
            occurrence_key,
            _,
        )) => occurrence_key
            .read_semantic_occurrence_parts()
            .unit_compilation_identity_key
            .clone(),
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::CrateRoot(unit_identity)) => {
            unit_identity.clone()
        }
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Builtin(_))
        | HldTargetIdentityValue::ExternalNamed(_) => return Ok(None),
        HldTargetIdentityValue::FilePublicArea(file_area_identity) => {
            file_area_identity.read_unit_identity_value().clone()
        }
    };
    unit_identity
        .validate_crate_unit_identity()
        .map_err(|_| HldMaterializationValidationError::MalformedCompilationUnitIdentity)?;
    Ok(Some(unit_identity))
}

fn read_hld_target_path(target_reference: &HldTargetIdentityValue) -> Option<&str> {
    match target_reference {
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(occurrence))
        | HldTargetIdentityValue::Symbol(NameSymbolTargetKey::GenericParameter(occurrence, _)) => {
            Some(
                occurrence
                    .read_semantic_occurrence_parts()
                    .span_source_file_path,
            )
        }
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::CrateRoot(_))
        | HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Builtin(_))
        | HldTargetIdentityValue::ExternalNamed(_) => None,
        HldTargetIdentityValue::FilePublicArea(file_area_identity) => {
            Some(file_area_identity.read_source_relative_path())
        }
    }
}

/// REQ-RUST-051.0 E9: an `expands` row starts at a macro call node, is sited at that call,
/// and a target, when it has one, is a macro definition node.
fn check_expands_edge_record(
    edge_record: &HldEdgeInputRecord,
    source_record: &HldNodeInputRecord,
    node_records: &BTreeMap<HldTargetIdentityValue, &HldNodeInputRecord>,
) -> bool {
    let source_is_call =
        source_record.node_item_kind_value == Some(HldItemKindValue::MacroInvocation);
    let sited_at_call = matches!(
        &edge_record.edge_source_reference,
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(call))
            if *call == edge_record.edge_evidence_row_key.evidence_site_occurrence_key
    );
    let target_is_definition = match &edge_record.edge_resolution_value {
        HldEdgeResolutionValue::Extracted { target } => {
            node_records.get(target).is_some_and(|target_record| {
                target_record.node_item_kind_value == Some(HldItemKindValue::MacroDefinition)
            })
        }
        HldEdgeResolutionValue::Unresolved { .. } => true,
        HldEdgeResolutionValue::Inferred { .. } | HldEdgeResolutionValue::Proposed { .. } => false,
    };
    source_is_call && sited_at_call && target_is_definition
}

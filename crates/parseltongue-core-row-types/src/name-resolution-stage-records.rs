use crate::{
    CrateCompilationUnitKey, NameResolutionStuckError, SemanticOccurrenceKeyValue,
    SiteUnresolvedReasonKind, StageInputMissingError, StageTableRowRecord,
    WorkspaceNotCompilingError,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum NamespaceBindingKindValue {
    Type,
    Value,
    Macro,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum NameNamespaceFilterKind {
    Any,
    Type,
    Value,
    Macro,
    TypeOrValue,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RustSourceEditionKind {
    Edition2015,
    Edition2018,
    Edition2021,
    Edition2024,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NameResolutionScopeKey {
    pub scope_compilation_unit_key: CrateCompilationUnitKey,
    pub scope_semantic_path_parts: Vec<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NameScopeRecordRow {
    pub scope_module_identity_key: NameResolutionScopeKey,
    pub scope_parent_identity_key: Option<NameResolutionScopeKey>,
    pub scope_parent_lookup_flag: bool,
    pub scope_module_boundary_flag: bool,
    pub scope_implicit_prelude_key: Option<NameResolutionScopeKey>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum NameSymbolTargetKey {
    Definition(SemanticOccurrenceKeyValue),
    GenericParameter(SemanticOccurrenceKeyValue, String),
    Builtin(String),
    CrateRoot(CrateCompilationUnitKey),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum NameBindingVisibilityKind {
    Invisible,
    Restricted(NameResolutionScopeKey),
    Public,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum NameBindingPriorityKind {
    Builtin,
    Prelude,
    Glob,
    Explicit,
    Declaration,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NameResolvedBindingRow {
    pub binding_lexical_scope_key: NameResolutionScopeKey,
    pub binding_resolved_name_text: String,
    pub binding_namespace_kind_value: NamespaceBindingKindValue,
    pub binding_target_identity_key: NameSymbolTargetKey,
    pub binding_target_child_scope: Option<NameResolutionScopeKey>,
    pub binding_visibility_scope_kind: NameBindingVisibilityKind,
    pub binding_selection_priority_kind: NameBindingPriorityKind,
    pub binding_import_evidence_keys: Vec<SemanticOccurrenceKeyValue>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum NamePathSegmentKind {
    CrateRoot,
    SelfModule,
    ParentModule,
    GlobalRoot,
    DefiningCrate(CrateCompilationUnitKey),
    Name(String),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NameImportRequestRow {
    pub import_site_occurrence_key: SemanticOccurrenceKeyValue,
    pub import_lexical_scope_key: NameResolutionScopeKey,
    pub import_path_segment_rows: Vec<NamePathSegmentKind>,
    pub import_binding_alias_text: Option<String>,
    pub import_glob_expansion_flag: bool,
    pub import_visibility_scope_kind: NameBindingVisibilityKind,
    pub import_resolution_source_kind: NameResolutionPathKind,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum NameResolutionPathKind {
    Import,
    Signature,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamePathRequestRow {
    pub path_site_occurrence_key: SemanticOccurrenceKeyValue,
    pub path_lexical_scope_key: NameResolutionScopeKey,
    pub path_source_segment_rows: Vec<NamePathSegmentKind>,
    pub path_requested_namespace_kind: NameNamespaceFilterKind,
    pub path_resolution_source_kind: NameResolutionPathKind,
    pub path_unavailable_reason_kind: Option<SiteUnresolvedReasonKind>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NamePathAnswerKind {
    Resolved,
    Unresolved,
    Ambiguous,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamePathAnswerRow {
    pub path_site_occurrence_key: SemanticOccurrenceKeyValue,
    pub path_resolution_state_kind: NamePathAnswerKind,
    pub path_resolved_target_rows: Vec<NameResolvedBindingRow>,
    pub path_candidate_target_rows: Vec<NameResolvedBindingRow>,
    pub path_unresolved_reason_kind: Option<SiteUnresolvedReasonKind>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NameMacroInvocationRow {
    pub invocation_site_occurrence_key: SemanticOccurrenceKeyValue,
    pub invocation_lexical_scope_key: NameResolutionScopeKey,
    pub invocation_macro_path_segments: Vec<NamePathSegmentKind>,
    pub invocation_argument_source_text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NameResolutionInputRows {
    pub unit_compilation_identity_key: CrateCompilationUnitKey,
    pub unit_rust_edition_kind: RustSourceEditionKind,
    pub unit_extern_prelude_key: Option<NameResolutionScopeKey>,
    pub name_shared_namespace_rows: Vec<std::sync::Arc<NameSharedNamespaceRows>>,
    pub name_module_scope_rows: Vec<NameScopeRecordRow>,
    pub name_symbol_binding_rows: Vec<NameResolvedBindingRow>,
    pub name_import_request_rows: Vec<NameImportRequestRow>,
    pub name_signature_path_rows: Vec<NamePathRequestRow>,
    pub name_macro_invocation_rows: Vec<NameMacroInvocationRow>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NameSharedNamespaceRows {
    pub name_module_scope_rows: Vec<NameScopeRecordRow>,
    pub name_symbol_binding_rows: Vec<NameResolvedBindingRow>,
}

impl NameResolutionInputRows {
    pub fn all_name_scope_rows(&self) -> impl Iterator<Item = &NameScopeRecordRow> {
        self.name_module_scope_rows.iter().chain(
            self.name_shared_namespace_rows
                .iter()
                .flat_map(|rows| rows.name_module_scope_rows.iter()),
        )
    }
    pub fn all_name_binding_rows(&self) -> impl Iterator<Item = &NameResolvedBindingRow> {
        self.name_symbol_binding_rows.iter().chain(
            self.name_shared_namespace_rows
                .iter()
                .flat_map(|rows| rows.name_symbol_binding_rows.iter()),
        )
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NameExpansionRoundRows {
    pub expansion_module_scope_rows: Vec<NameScopeRecordRow>,
    pub expansion_symbol_binding_rows: Vec<NameResolvedBindingRow>,
    pub expansion_import_request_rows: Vec<NameImportRequestRow>,
    pub expansion_signature_path_rows: Vec<NamePathRequestRow>,
    pub expansion_macro_invocation_rows: Vec<NameMacroInvocationRow>,
    pub expansion_completed_invocation_keys: Vec<SemanticOccurrenceKeyValue>,
    pub expansion_unavailable_reason_rows:
        Vec<(SemanticOccurrenceKeyValue, SiteUnresolvedReasonKind)>,
    pub expansion_semantic_output_rows: Vec<StageTableRowRecord>,
}

pub struct NameMacroExpansionQuery<'a> {
    pub expansion_input_record_rows: &'a NameResolutionInputRows,
    pub expansion_known_binding_rows: &'a [NameResolvedBindingRow],
    pub expansion_pending_invocation_rows: &'a [NameMacroInvocationRow],
    pub expansion_resolved_macro_rows: Vec<(SemanticOccurrenceKeyValue, NameResolvedBindingRow)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NameResolutionOutputRows {
    pub resolved_module_scope_rows: Vec<NameScopeRecordRow>,
    pub resolved_symbol_binding_rows: Vec<NameResolvedBindingRow>,
    pub resolved_signature_path_rows: Vec<NamePathAnswerRow>,
    pub resolved_completed_invocation_keys: Vec<SemanticOccurrenceKeyValue>,
    pub resolved_unavailable_reason_rows:
        Vec<(SemanticOccurrenceKeyValue, SiteUnresolvedReasonKind)>,
    pub resolved_semantic_output_rows: Vec<StageTableRowRecord>,
    pub resolved_changing_macro_rounds: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NameResolutionStageError {
    Input(StageInputMissingError),
    Expansion(WorkspaceNotCompilingError),
    Stuck(NameResolutionStuckError),
    Cancelled,
}

impl std::fmt::Display for NameResolutionStageError {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input(error) => error.fmt(output),
            Self::Expansion(error) => error.fmt(output),
            Self::Stuck(error) => error.fmt(output),
            Self::Cancelled => output.write_str("NameResolutionCancelledError"),
        }
    }
}

impl std::error::Error for NameResolutionStageError {}

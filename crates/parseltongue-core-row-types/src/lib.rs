//! Dependency-free semantic identities and typed stage boundaries.

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SpanDigestIdValue([u8; 32]);

impl From<[u8; 32]> for SpanDigestIdValue {
    fn from(_span_content_digest_full: [u8; 32]) -> Self {
        todo!()
    }
}

impl SpanDigestIdValue {
    pub fn read_full_digest_bytes(&self) -> &[u8; 32] {
        todo!()
    }

    pub fn read_truncated_digest_bytes(&self) -> [u8; 16] {
        todo!()
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SpanByteRangeValue {
    span_byte_start_offset: u64,
    span_byte_end_offset: u64,
}

impl TryFrom<(u64, u64)> for SpanByteRangeValue {
    type Error = CoreRowValidationError;

    fn try_from(_span_byte_range_pair: (u64, u64)) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CrateCompilationUnitKey {
    pub unit_cargo_package_identifier: String,
    pub unit_cargo_target_name: String,
    pub unit_cargo_target_kind: String,
    pub unit_compilation_role_kind: CompilationUnitRoleKind,
    pub unit_configuration_digest_full: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CompilationUnitRoleKind {
    Host,
    Target,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SemanticOccurrenceKeyValue {
    unit_compilation_identity_key: CrateCompilationUnitKey,
    span_source_file_path: String,
    span_byte_range_value: SpanByteRangeValue,
    span_content_digest_full: SpanDigestIdValue,
    span_expansion_provenance_chain: Vec<[u8; 32]>,
}

impl SemanticOccurrenceKeyValue {
    pub fn build_semantic_occurrence_key(
        _unit_compilation_identity_key: &CrateCompilationUnitKey,
        _span_source_file_path: &str,
        _span_byte_range_value: SpanByteRangeValue,
        _span_content_digest_full: SpanDigestIdValue,
        _span_expansion_provenance_chain: &[[u8; 32]],
    ) -> Result<Self, CoreRowValidationError> {
        todo!()
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RunGenerationDigestValue(pub [u8; 32]);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CrateDenseIndexValue(pub u32);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoreRowValidationError {
    ReversedRange,
    InvalidPath,
    EmptyUnitIdentity,
}

impl std::fmt::Display for CoreRowValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for CoreRowValidationError {}

macro_rules! define_context_error_types {
    ($($error_type:ident),+ $(,)?) => {
        $(
            #[derive(Clone, Debug, Eq, PartialEq)]
            pub struct $error_type {
                pub error_context_detail_text: String,
            }

            impl std::fmt::Display for $error_type {
                fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    formatter.write_str(&self.error_context_detail_text)
                }
            }

            impl std::error::Error for $error_type {}
        )+
    };
}

define_context_error_types!(
    CrateGraphLoadError,
    WorkspaceNotCompilingError,
    NameResolutionStuckError,
    StageInputMissingError,
    StoreWriteConflictError,
    StoreBackendFailureError,
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SiteUnresolvedReasonKind {
    CfgInactive,
    BuildScript,
    ProcMacro,
    ExternalSourceMissing,
    UnknownReceiver,
    TraitSolverNeeded,
}

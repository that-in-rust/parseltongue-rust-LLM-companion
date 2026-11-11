//! Dependency-free semantic identities and typed stage boundaries.

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SpanDigestIdValue([u8; 32]);

impl From<[u8; 32]> for SpanDigestIdValue {
    fn from(span_content_digest_full: [u8; 32]) -> Self {
        Self(span_content_digest_full)
    }
}

impl SpanDigestIdValue {
    pub fn read_full_digest_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn read_truncated_digest_bytes(&self) -> [u8; 16] {
        std::array::from_fn(|digest_byte_position_index| self.0[digest_byte_position_index])
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SpanByteRangeValue {
    span_byte_start_offset: u64,
    span_byte_end_offset: u64,
}

impl TryFrom<(u64, u64)> for SpanByteRangeValue {
    type Error = CoreRowValidationError;

    fn try_from(span_byte_range_pair: (u64, u64)) -> Result<Self, Self::Error> {
        let (span_byte_start_offset, span_byte_end_offset) = span_byte_range_pair;
        if span_byte_end_offset < span_byte_start_offset {
            Err(CoreRowValidationError::ReversedRange)
        } else {
            Ok(Self {
                span_byte_start_offset,
                span_byte_end_offset,
            })
        }
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
        unit_compilation_identity_key: &CrateCompilationUnitKey,
        span_source_file_path: &str,
        span_byte_range_value: SpanByteRangeValue,
        span_content_digest_full: SpanDigestIdValue,
        span_expansion_provenance_chain: &[[u8; 32]],
    ) -> Result<Self, CoreRowValidationError> {
        validate_source_relative_path(span_source_file_path)?;
        validate_compilation_unit_identity(unit_compilation_identity_key)?;
        Ok(Self {
            unit_compilation_identity_key: unit_compilation_identity_key.clone(),
            span_source_file_path: span_source_file_path.to_owned(),
            span_byte_range_value,
            span_content_digest_full,
            span_expansion_provenance_chain: span_expansion_provenance_chain.to_owned(),
        })
    }
}

fn validate_source_relative_path(
    span_source_file_path: &str,
) -> Result<(), CoreRowValidationError> {
    if span_source_file_path.contains(['\\', ':', '\0'])
        || span_source_file_path
            .split('/')
            .any(|source_path_component_text| matches!(source_path_component_text, "" | "." | ".."))
    {
        Err(CoreRowValidationError::InvalidPath)
    } else {
        Ok(())
    }
}

fn validate_compilation_unit_identity(
    unit_compilation_identity_key: &CrateCompilationUnitKey,
) -> Result<(), CoreRowValidationError> {
    [
        &unit_compilation_identity_key.unit_cargo_package_identifier,
        &unit_compilation_identity_key.unit_cargo_target_name,
        &unit_compilation_identity_key.unit_cargo_target_kind,
    ]
    .iter()
    .all(|unit_identity_component_text| !unit_identity_component_text.trim().is_empty())
    .then_some(())
    .ok_or(CoreRowValidationError::EmptyUnitIdentity)
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
    fn fmt(&self, error_display_format_sink: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_display_format_sink.write_str(match self {
            Self::ReversedRange => "span byte end precedes start",
            Self::InvalidPath => "source path must be normalized, relative and without NUL",
            Self::EmptyUnitIdentity => {
                "Cargo package identifier, target name and target kind must be nonempty"
            }
        })
    }
}

impl std::error::Error for CoreRowValidationError {}

macro_rules! define_context_error_types {
    ($($core_context_error_type:ident),+ $(,)?) => {
        $(
            #[derive(Clone, Debug, Eq, PartialEq)]
            pub struct $core_context_error_type {
                pub error_context_detail_text: String,
            }

            impl std::fmt::Display for $core_context_error_type {
                fn fmt(&self, error_display_format_sink: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    error_display_format_sink.write_str(&self.error_context_detail_text)
                }
            }

            impl std::error::Error for $core_context_error_type {}
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

#[cfg(test)]
mod core_identity_contract_tests {
    use super::*;

    fn build_fixture_compilation_unit() -> CrateCompilationUnitKey {
        CrateCompilationUnitKey {
            unit_cargo_package_identifier: "registry+https://example.invalid#sample@1.0.0".into(),
            unit_cargo_target_name: "sample".into(),
            unit_cargo_target_kind: "lib".into(),
            unit_compilation_role_kind: CompilationUnitRoleKind::Target,
            unit_configuration_digest_full: [0; 32],
        }
    }

    #[test]
    fn test_semantic_identity_collision_safety() -> Result<(), Box<dyn std::error::Error>> {
        let first_full_digest_bytes = [0; 32];
        let second_full_digest_bytes = std::array::from_fn(|index| u8::from(index == 31));
        let first_span_digest_value = SpanDigestIdValue::from(first_full_digest_bytes);
        let second_span_digest_value = SpanDigestIdValue::from(second_full_digest_bytes);
        assert_eq!(
            first_span_digest_value.read_truncated_digest_bytes(),
            second_span_digest_value.read_truncated_digest_bytes()
        );
        assert_ne!(first_span_digest_value, second_span_digest_value);
        assert_eq!(
            first_span_digest_value.read_full_digest_bytes(),
            &first_full_digest_bytes
        );
        assert_eq!(
            second_span_digest_value.read_full_digest_bytes(),
            &second_full_digest_bytes
        );
        let source_byte_range_value = SpanByteRangeValue::try_from((0, 8))?;
        let initial_compilation_unit_key = build_fixture_compilation_unit();
        let initial_semantic_occurrence_key =
            SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
                &initial_compilation_unit_key,
                "src/lib.rs",
                source_byte_range_value,
                first_span_digest_value,
                &[],
            )?;
        let alternate_compilation_unit_keys = [
            CrateCompilationUnitKey {
                unit_cargo_target_name: "sample-bin".into(),
                unit_cargo_target_kind: "bin".into(),
                ..initial_compilation_unit_key.clone()
            },
            CrateCompilationUnitKey {
                unit_cargo_package_identifier: "registry+https://other.invalid#sample@1.0.0".into(),
                ..initial_compilation_unit_key.clone()
            },
            CrateCompilationUnitKey {
                unit_cargo_package_identifier: "registry+https://example.invalid#sample@2.0.0"
                    .into(),
                ..initial_compilation_unit_key.clone()
            },
            CrateCompilationUnitKey {
                unit_configuration_digest_full: [1; 32],
                ..initial_compilation_unit_key.clone()
            },
            CrateCompilationUnitKey {
                unit_compilation_role_kind: CompilationUnitRoleKind::Host,
                ..initial_compilation_unit_key.clone()
            },
        ];
        for alternate_compilation_unit_key in &alternate_compilation_unit_keys {
            assert_ne!(
                initial_semantic_occurrence_key,
                SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
                    alternate_compilation_unit_key,
                    "src/lib.rs",
                    source_byte_range_value,
                    first_span_digest_value,
                    &[]
                )?
            );
        }
        for (
            alternate_source_file_path,
            alternate_byte_range_value,
            alternate_content_digest_full,
            alternate_expansion_provenance_chain,
        ) in [
            (
                "src/other.rs",
                source_byte_range_value,
                first_span_digest_value,
                vec![],
            ),
            (
                "src/lib.rs",
                SpanByteRangeValue::try_from((1, 8))?,
                first_span_digest_value,
                vec![],
            ),
            (
                "src/lib.rs",
                source_byte_range_value,
                second_span_digest_value,
                vec![],
            ),
            (
                "src/lib.rs",
                source_byte_range_value,
                first_span_digest_value,
                vec![[1; 32]],
            ),
            (
                "src/lib.rs",
                source_byte_range_value,
                first_span_digest_value,
                vec![[1; 32], [2; 32]],
            ),
        ] {
            assert_ne!(
                initial_semantic_occurrence_key,
                SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
                    &initial_compilation_unit_key,
                    alternate_source_file_path,
                    alternate_byte_range_value,
                    alternate_content_digest_full,
                    &alternate_expansion_provenance_chain
                )?
            );
        }
        assert_eq!(
            initial_semantic_occurrence_key,
            SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
                &initial_compilation_unit_key,
                "src/lib.rs",
                source_byte_range_value,
                first_span_digest_value,
                &[]
            )?
        );
        let digest_collision_record_set = (0..=255)
            .map(|suffix| {
                SpanDigestIdValue::from(std::array::from_fn(
                    |index| if index == 31 { suffix } else { 0 },
                ))
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(digest_collision_record_set.len(), 256);
        Ok(())
    }

    #[test]
    fn test_span_range_rejects_reversal() {
        assert!(SpanByteRangeValue::try_from((0, 0)).is_ok());
        assert!(SpanByteRangeValue::try_from((u64::MAX, u64::MAX)).is_ok());
        assert_eq!(
            SpanByteRangeValue::try_from((9, 8)),
            Err(CoreRowValidationError::ReversedRange)
        );
    }

    #[test]
    fn test_occurrence_rejects_invalid_paths() -> Result<(), Box<dyn std::error::Error>> {
        let source_byte_range_value = SpanByteRangeValue::try_from((0, 0))?;
        let initial_compilation_unit_key = build_fixture_compilation_unit();
        for invalid_source_file_path in [
            "",
            "/src/lib.rs",
            "../lib.rs",
            "src/./lib.rs",
            "src//lib.rs",
            "src/",
            "C:\\lib.rs",
            "src/\0lib.rs",
        ] {
            assert_eq!(
                SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
                    &initial_compilation_unit_key,
                    invalid_source_file_path,
                    source_byte_range_value,
                    SpanDigestIdValue::from([0; 32]),
                    &[]
                ),
                Err(CoreRowValidationError::InvalidPath)
            );
        }
        let empty_compilation_unit_key = CrateCompilationUnitKey {
            unit_cargo_target_name: String::new(),
            ..initial_compilation_unit_key
        };
        assert_eq!(
            SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
                &empty_compilation_unit_key,
                "lib.rs",
                source_byte_range_value,
                SpanDigestIdValue::from([0; 32]),
                &[]
            ),
            Err(CoreRowValidationError::EmptyUnitIdentity)
        );
        Ok(())
    }
}

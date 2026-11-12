use crate::{RustSourceEditionKind, SpanByteRangeValue};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarativeMacroExpansionInput {
    pub macro_declaration_style_flag: bool,
    pub macro_definition_source_text: String,
    pub macro_pattern_source_text: Option<String>,
    pub macro_argument_source_text: String,
    pub macro_definition_edition_kind: RustSourceEditionKind,
    pub macro_invocation_edition_kind: RustSourceEditionKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MacroSourceOriginKind {
    Definition,
    Invocation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MacroTokenOriginRecord {
    pub token_output_byte_range: SpanByteRangeValue,
    pub token_original_byte_range: SpanByteRangeValue,
    pub token_source_origin_kind: MacroSourceOriginKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarativeMacroExpansionOutput {
    pub macro_expanded_source_text: String,
    pub macro_token_origin_rows: Vec<MacroTokenOriginRecord>,
    pub macro_matched_arm_index: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MacroExpansionFailureKind {
    InvalidDefinition(String),
    InvalidInvocation(String),
    NoMatchingRule,
    Cancelled,
    Expansion(String),
}

impl std::fmt::Display for MacroExpansionFailureKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDefinition(detail) => {
                write!(formatter, "InvalidMacroDefinitionError: {detail}")
            }
            Self::InvalidInvocation(detail) => {
                write!(formatter, "InvalidMacroInvocationError: {detail}")
            }
            Self::NoMatchingRule => formatter.write_str("MacroMatchingRuleMissingError"),
            Self::Cancelled => formatter.write_str("MacroExpansionCancelledError"),
            Self::Expansion(detail) => {
                write!(formatter, "MacroItemExpansionFailureError: {detail}")
            }
        }
    }
}

impl std::error::Error for MacroExpansionFailureKind {}

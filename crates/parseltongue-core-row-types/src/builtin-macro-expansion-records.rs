use crate::{RustSourceEditionKind, SiteUnresolvedReasonKind};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuiltinMacroExpansionInput {
    pub macro_builtin_identifier_text: String,
    pub macro_argument_source_text: String,
    pub macro_invocation_edition_kind: RustSourceEditionKind,
    pub macro_compilation_cfg_values: BTreeSet<String>,
    pub macro_metadata_environment_rows: BTreeMap<String, Option<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BuiltinMacroExpansionOutput {
    Include(String),
    Expression(String),
    Unavailable(SiteUnresolvedReasonKind, String),
}

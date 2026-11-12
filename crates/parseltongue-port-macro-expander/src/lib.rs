#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

extern crate ra_ap_rustc_lexer as rustc_lexer;

#[cfg(test)]
use parseltongue_core_row_types::{
    DeclarativeMacroExpansionInput, DeclarativeMacroExpansionOutput, MacroExpansionFailureKind,
    RunCancelFlagValue,
};

#[path = "declarative-expansion-source-facade.rs"]
mod declarative_expansion_source_facade;
#[allow(dead_code)]
#[path = "macro-expansion-rule-engine.rs"]
mod macro_expansion_rule_engine;
#[path = "macro-token-parser-input.rs"]
mod macro_token_parser_input;
pub use declarative_expansion_source_facade::expand_declarative_macro_items;
#[path = "builtin-expansion-argument-rules.rs"]
mod builtin_expansion_argument_rules;
pub use builtin_expansion_argument_rules::{BuiltinMacroLookupPort, expand_builtin_macro_items};

#[cfg(test)]
#[path = "builtin-expansion-contract-tests.rs"]
mod builtin_expansion_contract_tests;
#[cfg(test)]
#[path = "declarative-expansion-contract-tests.rs"]
mod declarative_expansion_contract_tests;

#![allow(clippy::unwrap_used)]

use super::*;
use parseltongue_core_row_types::{MacroSourceOriginKind, RustSourceEditionKind};
use ra_ap_syntax::{Edition, SourceFile};
use std::process::Command;

fn build_macro_expansion_fixture(
    definition: &str,
    arguments: &str,
) -> DeclarativeMacroExpansionInput {
    DeclarativeMacroExpansionInput {
        macro_definition_source_text: definition.into(),
        macro_pattern_source_text: None,
        macro_argument_source_text: arguments.into(),
        macro_definition_edition_kind: RustSourceEditionKind::Edition2024,
        macro_invocation_edition_kind: RustSourceEditionKind::Edition2024,
    }
}

fn read_macro_expansion_fixture(
    definition: &str,
    arguments: &str,
) -> DeclarativeMacroExpansionOutput {
    let result = expand_declarative_macro_items(
        &build_macro_expansion_fixture(definition, arguments),
        &RunCancelFlagValue::default(),
    )
    .unwrap();
    assert!(
        SourceFile::parse(&result.macro_expanded_source_text, Edition::Edition2024)
            .errors()
            .is_empty(),
        "{}",
        result.macro_expanded_source_text
    );
    result
}

fn normalize_macro_output_text(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn test_macro_item_expansion() {
    let output = read_macro_expansion_fixture(
        "{(first)=>{pub struct First;};($name:ident)=>{pub struct $name;};}",
        "(Chosen)",
    );
    assert_eq!(output.macro_matched_arm_index, 1);
    assert_eq!(
        normalize_macro_output_text(&output.macro_expanded_source_text),
        "pubstructChosen;"
    );
    let mut input = build_macro_expansion_fixture("{pub struct $name;}", "(Second)");
    input.macro_pattern_source_text = Some("($name:ident)".into());
    let second = expand_declarative_macro_items(&input, &RunCancelFlagValue::default()).unwrap();
    assert_eq!(
        normalize_macro_output_text(&second.macro_expanded_source_text),
        "pubstructSecond;"
    );
}

#[test]
fn test_macro_repetition_expansion() {
    let output = read_macro_expansion_fixture(
        "{($($name:ident:[$($ty:ty),*]),* $(,)?)=>{$(pub struct $name($(pub $ty),*);)*};}",
        "(First:[u8,u16],Second:[u32],)",
    );
    assert_eq!(
        normalize_macro_output_text(&output.macro_expanded_source_text),
        "pubstructFirst(pubu8,pubu16);pubstructSecond(pubu32);"
    );
}

#[test]
fn test_macro_fragment_matching() {
    let output = read_macro_expansion_fixture(
        "{($ty:ty,$value:expr,$bound:path,$item:item)=>{pub type Chosen=$ty;pub const VALUE:$ty=$value;$item pub fn sample<T:$bound>(){}};}",
        "(Option<(u8,u16)>,Some((1,2)),std::fmt::Debug,pub struct Input;)",
    );
    let normalized = normalize_macro_output_text(&output.macro_expanded_source_text);
    assert!(normalized.contains("pubtypeChosen=Option<(u8,u16)>;"));
    assert!(normalized.contains("pubconstVALUE:Option<(u8,u16)>=Some((1,2));"));
    assert!(normalized.contains("pubstructInput;"));
    assert!(normalized.contains("pubfnsample<T:std::fmt::Debug>(){}"));
}

#[test]
fn test_macro_empty_expansion() {
    let output = read_macro_expansion_fixture("{()=>{};}", "()");
    assert!(output.macro_expanded_source_text.trim().is_empty());
    assert!(output.macro_token_origin_rows.is_empty());
}

#[test]
fn test_macro_invalid_inputs() {
    let cancel = RunCancelFlagValue::default();
    assert!(matches!(
        expand_declarative_macro_items(&build_macro_expansion_fixture("{$broken}", "()"), &cancel),
        Err(MacroExpansionFailureKind::InvalidDefinition(_))
    ));
    assert!(matches!(
        expand_declarative_macro_items(&build_macro_expansion_fixture("{()=>{};}", "("), &cancel),
        Err(MacroExpansionFailureKind::InvalidInvocation(_))
    ));
    assert!(matches!(
        expand_declarative_macro_items(
            &build_macro_expansion_fixture("{(first)=>{};}", "(second)"),
            &cancel
        ),
        Err(MacroExpansionFailureKind::NoMatchingRule)
    ));
}

#[test]
fn test_macro_origin_provenance() {
    let input = build_macro_expansion_fixture("{($name:ident)=>{pub struct $name;};}", "(Chosen)");
    let output = expand_declarative_macro_items(&input, &RunCancelFlagValue::default()).unwrap();
    assert!(!output.macro_token_origin_rows.is_empty());
    let captured = output
        .macro_token_origin_rows
        .iter()
        .find(|row| {
            let (start, end) = row.token_output_byte_range.read_source_range_pair();
            &output.macro_expanded_source_text[start as usize..end as usize] == "Chosen"
        })
        .unwrap();
    assert_eq!(
        captured.token_source_origin_kind,
        MacroSourceOriginKind::Invocation
    );
    let (start, end) = captured.token_original_byte_range.read_source_range_pair();
    assert_eq!(
        &input.macro_argument_source_text[start as usize..end as usize],
        "Chosen"
    );
    assert!(
        output
            .macro_token_origin_rows
            .iter()
            .any(|row| row.token_source_origin_kind == MacroSourceOriginKind::Definition)
    );
}

#[test]
fn test_macro_context_editions() {
    let mut input = build_macro_expansion_fixture(
        "{($value:expr)=>{pub const VALUE:u8=$value;};}",
        "(const {1})",
    );
    input.macro_definition_edition_kind = RustSourceEditionKind::Edition2021;
    assert!(matches!(
        expand_declarative_macro_items(&input, &RunCancelFlagValue::default()),
        Err(MacroExpansionFailureKind::NoMatchingRule)
    ));
    input.macro_definition_edition_kind = RustSourceEditionKind::Edition2024;
    input.macro_invocation_edition_kind = RustSourceEditionKind::Edition2021;
    assert!(expand_declarative_macro_items(&input, &RunCancelFlagValue::default()).is_ok());
}

#[test]
fn test_macro_cancellation_errors() {
    let cancel = RunCancelFlagValue::default();
    cancel.request_stage_work_cancellation();
    assert_eq!(
        expand_declarative_macro_items(&build_macro_expansion_fixture("{()=>{};}", "()"), &cancel),
        Err(MacroExpansionFailureKind::Cancelled)
    );
}

#[test]
fn test_macro_dependencies_exclude_salsa() {
    let output = Command::new(env!("CARGO"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args([
            "tree",
            "--offline",
            "--locked",
            "-p",
            "parseltongue-port-macro-expander",
            "--prefix",
            "none",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let tree = String::from_utf8(output.stdout).unwrap();
    assert!(
        !tree
            .lines()
            .filter_map(|line| line.split_whitespace().next())
            .any(|name| name == "salsa"
                || name.starts_with("ra_ap_hir")
                || name.starts_with("ra_ap_ide")
                || name == "ra_ap_base_db"),
        "{tree}"
    );
}

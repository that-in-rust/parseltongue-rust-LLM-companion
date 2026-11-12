#![allow(clippy::unwrap_used)]

use super::{BuiltinMacroLookupPort, expand_builtin_macro_items};
use parseltongue_core_row_types::*;

struct FixtureBuiltinMacroLookup;
impl BuiltinMacroLookupPort for FixtureBuiltinMacroLookup {
    fn read_builtin_macro_name(&self, path: &[NamePathSegmentKind]) -> Option<String> {
        match path {
            [NamePathSegmentKind::Name(name)] => Some(name.clone()),
            _ => None,
        }
    }
}

fn build_builtin_expansion_input(name: &str, args: &str) -> BuiltinMacroExpansionInput {
    BuiltinMacroExpansionInput {
        macro_builtin_identifier_text: name.into(),
        macro_argument_source_text: args.into(),
        macro_invocation_edition_kind: RustSourceEditionKind::Edition2024,
        macro_compilation_cfg_values: ["unix".into(), "feature=\"enabled\"".into()]
            .into_iter()
            .collect(),
        macro_metadata_environment_rows: [
            ("CARGO_PKG_NAME".into(), Some("fixture".into())),
            ("CARGO_PKG_LICENSE".into(), Some("".into())),
            ("CARGO_BIN_NAME".into(), None),
        ]
        .into_iter()
        .collect(),
    }
}

fn expand_builtin_fixture(
    name: &str,
    args: &str,
) -> Result<BuiltinMacroExpansionOutput, MacroExpansionFailureKind> {
    expand_builtin_macro_items(
        &build_builtin_expansion_input(name, args),
        &FixtureBuiltinMacroLookup,
        &RunCancelFlagValue::default(),
    )
}

#[test]
fn test_builtin_cfg_arguments() {
    assert_eq!(
        expand_builtin_fixture("cfg", "(all(unix,not(test),feature=\"enabled\"))").unwrap(),
        BuiltinMacroExpansionOutput::Expression("true".into())
    );
    assert_eq!(
        expand_builtin_fixture("cfg", "(test)").unwrap(),
        BuiltinMacroExpansionOutput::Expression("false".into())
    );
}

#[test]
fn test_builtin_include_arguments() {
    assert_eq!(
        expand_builtin_fixture(
            "include",
            "(concat!(env!(\"CARGO_PKG_NAME\"),\"/\",r#\"src,part.rs\"#))"
        )
        .unwrap(),
        BuiltinMacroExpansionOutput::Include("fixture/src,part.rs".into())
    );
    assert_eq!(
        expand_builtin_fixture("concat", "(1_000u16, '-', true, \"é\")").unwrap(),
        BuiltinMacroExpansionOutput::Expression("\"1000-trueé\"".into())
    );
}

#[test]
fn test_builtin_stringify_arguments() {
    assert_eq!(
        expand_builtin_fixture("stringify", "(alpha + beta)").unwrap(),
        BuiltinMacroExpansionOutput::Expression("\"alpha + beta\"".into())
    );
    assert_eq!(
        expand_builtin_fixture("stringify", "()").unwrap(),
        BuiltinMacroExpansionOutput::Expression("\"\"".into())
    );
    assert_eq!(
        expand_builtin_fixture("concat", "()").unwrap(),
        BuiltinMacroExpansionOutput::Expression("\"\"".into())
    );
}

#[test]
fn test_builtin_environment_boundaries() {
    assert_eq!(
        expand_builtin_fixture("env", "(\"CARGO_PKG_NAME\")").unwrap(),
        BuiltinMacroExpansionOutput::Expression("\"fixture\"".into())
    );
    assert_eq!(
        expand_builtin_fixture("option_env", "(\"CARGO_PKG_LICENSE\")").unwrap(),
        BuiltinMacroExpansionOutput::Expression("Some(\"\")".into())
    );
    assert_eq!(
        expand_builtin_fixture("option_env", "(\"CARGO_BIN_NAME\")").unwrap(),
        BuiltinMacroExpansionOutput::Expression("None".into())
    );
    for name in ["env", "option_env"] {
        assert!(matches!(
            expand_builtin_fixture(name, "(\"HOME\")").unwrap(),
            BuiltinMacroExpansionOutput::Unavailable(
                SiteUnresolvedReasonKind::ExternalSourceMissing,
                _
            )
        ));
        assert!(matches!(
            expand_builtin_fixture(name, "(\"OUT_DIR\")").unwrap(),
            BuiltinMacroExpansionOutput::Unavailable(SiteUnresolvedReasonKind::BuildScript, _)
        ));
    }
}

#[test]
fn test_builtin_shadowed_arguments() {
    assert!(matches!(
        expand_builtin_fixture("include", "(shadow::concat!(\"src/lib.rs\"))").unwrap(),
        BuiltinMacroExpansionOutput::Unavailable(
            SiteUnresolvedReasonKind::ExternalSourceMissing,
            _
        )
    ));
    assert!(matches!(
        expand_builtin_fixture("format_args", "(\"hello\")").unwrap(),
        BuiltinMacroExpansionOutput::Unavailable(
            SiteUnresolvedReasonKind::ExternalSourceMissing,
            _
        )
    ));
}

#[test]
fn test_builtin_invalid_arguments() {
    for (name, args) in [
        ("include", "()"),
        ("env", "()"),
        ("env", "(1)"),
        ("concat", "(\"a\"]"),
        ("cfg", "(all(unix),test)"),
    ] {
        assert!(
            matches!(
                expand_builtin_fixture(name, args),
                Err(MacroExpansionFailureKind::InvalidInvocation(_))
            ),
            "{name} {args}"
        );
    }
    let cancelled = RunCancelFlagValue::default();
    cancelled.request_stage_work_cancellation();
    assert_eq!(
        expand_builtin_macro_items(
            &build_builtin_expansion_input("concat", "()"),
            &FixtureBuiltinMacroLookup,
            &cancelled
        ),
        Err(MacroExpansionFailureKind::Cancelled)
    );
}

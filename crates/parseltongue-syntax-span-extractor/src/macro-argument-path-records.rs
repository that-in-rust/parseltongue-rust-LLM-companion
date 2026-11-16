//! Value paths written in the arguments of a standard assertion macro
//! (REQ-RUST-052.0 G15). The arguments of `assert!(A::N < 0, "..")` are a
//! token tree, not expressions, so the paths in them are read by parsing the
//! argument text again as a list of expressions.
use crate::body_value_read_records::{
    check_body_value_path, check_body_value_prefix, check_value_name_bound,
};
use ra_ap_syntax::{
    AstNode, Edition, SourceFile, SyntaxKind,
    ast::{self, HasArgList},
};

/// The text the arguments are parsed in: one call inside one item.
const ARGUMENT_WRAP_PREFIX_TEXT: &str = "const _: () = { __f(";

#[cfg(test)]
#[path = "macro-argument-path-tests.rs"]
mod macro_argument_path_tests;

/// The macros whose arguments are a comma-separated list of expressions.
pub(super) const ASSERTION_MACRO_NAME_LIST: [&str; 6] = [
    "assert",
    "debug_assert",
    "assert_eq",
    "assert_ne",
    "debug_assert_eq",
    "debug_assert_ne",
];

/// One path in the arguments of an assertion macro call, with its byte range
/// in the parsed text of the file the call is written in.
#[derive(Clone, Debug)]
pub(super) struct MacroArgumentPathValue {
    pub(super) argument_path_syntax_node: ast::Path,
    pub(super) argument_path_start_offset: usize,
    pub(super) argument_path_end_offset: usize,
    pub(super) argument_path_context_kind: &'static str,
}

/// The value paths, and their one-name type prefixes, in the arguments of
/// `call`. Empty unless the call names an assertion macro by one name, is in
/// expression position and its arguments parse as expressions without error.
pub(super) fn read_macro_argument_paths(call: &ast::MacroCall) -> Vec<MacroArgumentPathValue> {
    let name = call
        .path()
        .filter(|path| path.coloncolon_token().is_none())
        .and_then(|path| path.segment())
        .and_then(|segment| segment.name_ref())
        .map(|name| name.text().to_string())
        .filter(|name| ASSERTION_MACRO_NAME_LIST.contains(&name.as_str()));
    let Some(((tree, site), name)) = call
        .token_tree()
        .zip(call.syntax().parent())
        .zip(name)
        .filter(|_| !crate::check_syntax_item_node(call.syntax()))
    else {
        return vec![];
    };
    // The expressions the macro compares; the message and its arguments follow.
    let compared = if name.ends_with("_eq") || name.ends_with("_ne") {
        2
    } else {
        1
    };
    let text = tree.syntax().text().to_string();
    let Some(inner) = text.get(1..text.len().saturating_sub(1)) else {
        return vec![];
    };
    let parsed = SourceFile::parse(
        &format!("{ARGUMENT_WRAP_PREFIX_TEXT}{inner}\n); }};"),
        Edition::Edition2024,
    );
    let arguments = parsed
        .tree()
        .syntax()
        .descendants()
        .find_map(ast::CallExpr::cast)
        .and_then(|call| call.arg_list())
        .map(|list| list.args().collect::<Vec<_>>())
        .unwrap_or_default();
    if !parsed.errors().is_empty() || arguments.len() < compared {
        return vec![];
    }
    // `name = value` after the message names a format argument, not a path.
    let labels = arguments
        .iter()
        .skip(compared)
        .filter_map(|argument| match argument {
            ast::Expr::BinExpr(pair)
                if pair
                    .op_token()
                    .is_some_and(|token| token.kind() == SyntaxKind::EQ) =>
            {
                pair.lhs().map(|label| label.syntax().text_range())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let inner_start = usize::from(tree.syntax().text_range().start()) + 1;
    // A whole value path that is not rooted at `$crate` and whose single name is
    // not bound where the call is written.
    let admitted = |path: &ast::Path| {
        check_body_value_path(path)
            && !labels
                .iter()
                .any(|label| label.contains_range(path.syntax().text_range()))
            && path
                .first_segment()
                .is_none_or(|segment| segment.syntax().text() != "_crate")
            && !(path.qualifier().is_none()
                && path.segment().is_some_and(|segment| {
                    check_value_name_bound(&segment.syntax().text().to_string(), &site)
                }))
    };
    parsed
        .tree()
        .syntax()
        .descendants()
        .filter_map(ast::Path::cast)
        .filter_map(|path| {
            let kind = if admitted(&path) {
                "body_value_path"
            } else if check_body_value_prefix(&path)
                && path
                    .syntax()
                    .parent()
                    .and_then(ast::Path::cast)
                    .is_some_and(|outer| admitted(&outer))
            {
                "body_value_type_prefix"
            } else {
                return None;
            };
            let range = path.syntax().text_range();
            let shift = |offset: ra_ap_syntax::TextSize| {
                usize::from(offset).checked_sub(ARGUMENT_WRAP_PREFIX_TEXT.len())
            };
            Some(MacroArgumentPathValue {
                argument_path_start_offset: shift(range.start())? + inner_start,
                argument_path_end_offset: shift(range.end())? + inner_start,
                argument_path_context_kind: kind,
                argument_path_syntax_node: path,
            })
        })
        .collect()
}

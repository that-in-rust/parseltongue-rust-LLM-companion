//! REQ-RUST-069.0 S1, S2: the signature of an item, printed from its own tokens, and
//! V1, V7: the visibility text the item declares.

use ra_ap_syntax::{NodeOrToken, SyntaxKind, SyntaxKind::*, SyntaxNode, SyntaxToken};

#[cfg(test)]
#[path = "item-signature-text-tests.rs"]
mod item_signature_text_tests;

/// The item's tokens without attributes, comments, body, field list, value and final
/// `;`, joined by the spacing rule S2. Source text and expanded text print the same.
pub(crate) fn print_item_signature_text(item: &SyntaxNode) -> String {
    let tokens = collect_item_signature_tokens(item);
    tokens
        .iter()
        .enumerate()
        .map(|(index, token)| {
            let spaced = index > 0 && check_token_pair_spaced(&tokens[index - 1], token);
            format!("{}{}", if spaced { " " } else { "" }, token.text())
        })
        .collect()
}

/// S1: the tokens the signature keeps, in source order.
fn collect_item_signature_tokens(item: &SyntaxNode) -> Vec<SyntaxToken> {
    // REQ-RUST-071.0 F3: the discriminant of a variant is a value too.
    let value_cut = matches!(item.kind(), CONST | STATIC | VARIANT);
    let tokens = item
        .children_with_tokens()
        .take_while(|child| !(value_cut && child.kind() == EQ))
        // A doc comment is a node of the item in this parser, an attribute in all but form.
        .filter(|child| !check_signature_child_cut(item.kind(), child.kind()))
        .flat_map(|child| match child {
            NodeOrToken::Token(token) => vec![token],
            NodeOrToken::Node(node) => node
                .descendants_with_tokens()
                .filter_map(NodeOrToken::into_token)
                .collect(),
        })
        .filter(|token| !matches!(token.kind(), WHITESPACE | COMMENT))
        .collect::<Vec<_>>();
    let kept =
        tokens.len() - usize::from(tokens.last().is_some_and(|token| token.kind() == SEMICOLON));
    tokens.into_iter().take(kept).collect()
}

/// S1: a direct child of the item that is no part of its signature.
fn check_signature_child_cut(item: SyntaxKind, child: SyntaxKind) -> bool {
    matches!(
        child,
        ATTR | DOC_COMMENT
            | OUTER_DOC_COMMENT
            | INNER_DOC_COMMENT
            | ASSOC_ITEM_LIST
            | ITEM_LIST
            | RECORD_FIELD_LIST
            | TUPLE_FIELD_LIST
            | VARIANT_LIST
    ) || (child == BLOCK_EXPR && item == FN)
        || (child == TOKEN_TREE && matches!(item, MACRO_RULES | MACRO_DEF))
}

/// S2: whether one space is written between two neighbouring tokens.
fn check_token_pair_spaced(previous: &SyntaxToken, next: &SyntaxToken) -> bool {
    let parent_kind = |token: &SyntaxToken| token.parent().map(|parent| parent.kind());
    // `<` `>` `&` `*` are type punctuation unless they are written in a binary expression.
    let type_mark = |token: &SyntaxToken| {
        matches!(token.kind(), L_ANGLE | R_ANGLE | AMP | STAR)
            && parent_kind(token) != Some(BIN_EXPR)
    };
    let (before, after) = (previous.kind(), next.kind());
    let joined_after = matches!(
        before,
        L_PAREN | L_BRACK | L_CURLY | COLON2 | POUND | DOT | QUESTION | DOT2 | DOT2EQ
    ) || (matches!(before, L_ANGLE | AMP | STAR) && type_mark(previous))
        || parent_kind(previous) == Some(PREFIX_EXPR);
    let joined_before = match after {
        R_PAREN | R_BRACK | R_CURLY | COMMA | SEMICOLON | COLON | COLON2 | DOT | DOT2 | DOT2EQ => {
            true
        }
        R_ANGLE => type_mark(next),
        BANG => matches!(before, IDENT | MACRO_RULES_KW),
        L_ANGLE => type_mark(next) && matches!(before, IDENT | SELF_TYPE_KW | IMPL_KW | FOR_KW),
        L_PAREN => {
            matches!(
                before,
                IDENT
                    | SELF_KW
                    | SELF_TYPE_KW
                    | SUPER_KW
                    | CRATE_KW
                    | PUB_KW
                    | FN_KW
                    | R_PAREN
                    | R_BRACK
                    | BANG
            ) || (before == R_ANGLE && type_mark(previous))
        }
        L_BRACK => matches!(before, IDENT | R_PAREN | R_BRACK | POUND | BANG),
        _ => false,
    };
    !(joined_after || joined_before)
}

/// V1, V7: the visibility an item writes: its `pub` form, `private` when it writes none,
/// and `macro_export` for a `macro_rules!` definition that carries that attribute.
pub(crate) fn read_item_visibility_text(item: &SyntaxNode) -> String {
    let exported = item.kind() == MACRO_RULES
        && item
            .children()
            .filter(|child| child.kind() == ATTR)
            .map(|attribute| {
                attribute
                    .text()
                    .to_string()
                    .split_whitespace()
                    .collect::<String>()
            })
            .any(|text| text == "#[macro_export]" || text.starts_with("#[macro_export("));
    if exported {
        return "macro_export".into();
    }
    item.children()
        .find(|child| child.kind() == VISIBILITY)
        .map_or_else(|| "private".into(), |written| written.text().to_string())
}

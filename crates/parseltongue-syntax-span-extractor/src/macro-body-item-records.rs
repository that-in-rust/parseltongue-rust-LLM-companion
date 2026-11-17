//! REQ-RUST-063.0 M2, M3: where an item written as tokens sits in a `macro_rules!` body,
//! from syntax only.
use ra_ap_syntax::{Direction, NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};
use serde_json::{Value, json};

fn check_tree_opener_kind(node: &SyntaxNode, opener: SyntaxKind) -> bool {
    node.kind() == SyntaxKind::TOKEN_TREE
        && node
            .first_token()
            .is_some_and(|token| token.kind() == opener)
}

/// The elements before `token` in its brackets, nearest first. A doc comment is a token
/// here and is passed over like a comment: it holds no `cfg`.
fn list_previous_code_elements(token: &SyntaxToken) -> impl Iterator<Item = SyntaxElement> {
    token
        .siblings_with_tokens(Direction::Prev)
        .skip(1)
        .filter(|element| {
            !element.kind().is_trivia()
                && !matches!(
                    element.kind(),
                    SyntaxKind::OUTER_DOC_COMMENT | SyntaxKind::INNER_DOC_COMMENT
                )
        })
}

/// The token trees of the arms of a `macro_rules!`: pattern, body, pattern, body.
fn list_macro_arm_trees(definition: &SyntaxNode) -> Vec<SyntaxNode> {
    definition
        .children()
        .find(|node| node.kind() == SyntaxKind::TOKEN_TREE)
        .map(|rules| {
            rules
                .children()
                .filter(|node| node.kind() == SyntaxKind::TOKEN_TREE)
                .collect()
        })
        .unwrap_or_default()
}

/// M2: the number of arms of a `macro_rules!`. `None` when its token trees are not pairs
/// of a pattern and a body.
pub(crate) fn count_macro_rule_arms(definition: &SyntaxNode) -> Option<usize> {
    let trees = list_macro_arm_trees(definition);
    (definition.kind() == SyntaxKind::MACRO_RULES && trees.len().is_multiple_of(2))
        .then_some(trees.len() / 2)
}

/// Whether `tree` is the parentheses of a repetition: `$( .. )`.
fn check_repetition_tree_form(tree: &SyntaxNode) -> bool {
    check_tree_opener_kind(tree, SyntaxKind::L_PAREN)
        && tree
            .siblings_with_tokens(Direction::Prev)
            .skip(1)
            .find(|element| !element.kind().is_trivia())
            .is_some_and(|element| element.kind() == SyntaxKind::DOLLAR)
}

/// M3: the arm of the `macro_rules!` `definition` that holds `keyword`, the bracket pairs
/// between that arm's body and `keyword`, and whether one of them is a repetition. All
/// null for any other macro form.
pub(crate) fn read_macro_body_position(keyword: &SyntaxToken, definition: &SyntaxNode) -> Value {
    let arms = list_macro_arm_trees(definition);
    let inside = keyword
        .parent_ancestors()
        .take_while(|node| node != definition)
        .filter(|node| node.kind() == SyntaxKind::TOKEN_TREE)
        .collect::<Vec<_>>();
    // The last two trees around the keyword are the arm's tree and the macro's braces.
    let found = match inside.as_slice() {
        [nested @ .., arm, _] if definition.kind() == SyntaxKind::MACRO_RULES => arms
            .iter()
            .position(|tree| tree == arm)
            .map(|at| (at, nested)),
        _ => None,
    };
    match found {
        Some((at, nested)) => json!({
            "macro_arm_ordinal_value":at / 2,
            "macro_body_nesting_depth":(at % 2 == 1).then_some(nested.len()),
            "macro_body_repetition_flag":nested.iter().any(check_repetition_tree_form),
        }),
        None => json!({
            "macro_arm_ordinal_value":null,
            "macro_body_nesting_depth":null,
            "macro_body_repetition_flag":null,
        }),
    }
}

fn check_item_modifier_element(element: &SyntaxElement) -> bool {
    match element {
        NodeOrToken::Token(token) => matches!(
            token.kind(),
            SyntaxKind::PUB_KW
                | SyntaxKind::CONST_KW
                | SyntaxKind::UNSAFE_KW
                | SyntaxKind::ASYNC_KW
                | SyntaxKind::EXTERN_KW
                | SyntaxKind::STRING
        ),
        // `pub(crate)`
        NodeOrToken::Node(node) => check_tree_opener_kind(node, SyntaxKind::L_PAREN),
    }
}

/// M3: the `#[..]` written directly before the item whose keyword is `keyword`, in
/// source order, and whether the tokens before them end at the start of the brackets, a
/// `;` or a `}`. When they do not, something else may add an attribute to the item.
pub(crate) fn read_item_attribute_texts(keyword: &SyntaxToken) -> (Vec<String>, bool) {
    let before = list_previous_code_elements(keyword)
        .skip_while(check_item_modifier_element)
        .collect::<Vec<_>>();
    let pairs = before
        .chunks(2)
        .take_while(|pair| match pair {
            [NodeOrToken::Node(tree), NodeOrToken::Token(pound)] => {
                check_tree_opener_kind(tree, SyntaxKind::L_BRACK)
                    && pound.kind() == SyntaxKind::POUND
            }
            _ => false,
        })
        .count();
    let boundary = match before.get(pairs * 2) {
        None => true,
        // The opener of the brackets that hold the item, or the end of the item before.
        Some(NodeOrToken::Token(token)) => matches!(
            token.kind(),
            SyntaxKind::SEMICOLON | SyntaxKind::L_CURLY | SyntaxKind::L_PAREN | SyntaxKind::L_BRACK
        ),
        Some(NodeOrToken::Node(node)) => check_tree_opener_kind(node, SyntaxKind::L_CURLY),
    };
    let texts = before[..pairs * 2]
        .chunks(2)
        .rev()
        .map(|pair| format!("#{}", pair[0]))
        .collect();
    (texts, boundary)
}

/// M3: the type written after the `:` of a `const` item, up to its `=` or `;`.
pub(crate) fn read_const_type_text(colon: &SyntaxToken) -> String {
    colon
        .siblings_with_tokens(Direction::Next)
        .skip(1)
        .take_while(|element| !matches!(element.kind(), SyntaxKind::EQ | SyntaxKind::SEMICOLON))
        .map(|element| element.to_string())
        .collect::<String>()
        .trim()
        .to_owned()
}

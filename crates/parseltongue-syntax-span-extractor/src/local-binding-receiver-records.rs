//! REQ-RUST-064.0 L1: the one `let` that binds a receiver name to a method call. A pure
//! read of the syntax of one function; anything else gives `None`.
use ra_ap_syntax::{
    AstNode, SyntaxKind, SyntaxToken,
    ast::{self, HasName},
};

/// The `let` that binds a local variable, with what the lookup needs of it.
pub(crate) struct LocalBindingSyntaxValue {
    pub(crate) binding_let_statement: ast::LetStmt,
    pub(crate) binding_initializer_call: ast::MethodCallExpr,
    pub(crate) binding_name_token_count: usize,
    pub(crate) binding_mutable_flag: bool,
}

/// Whether `token` is a whole path expression of one segment: a use of a variable.
fn check_plain_path_token(token: &SyntaxToken) -> bool {
    token
        .parent()
        .filter(|node| node.kind() == SyntaxKind::NAME_REF)
        .and_then(|node| node.parent())
        .and_then(ast::PathSegment::cast)
        .is_some_and(|segment| {
            segment.syntax().children().count() == 1
                && segment
                    .syntax()
                    .parent()
                    .and_then(ast::Path::cast)
                    .is_some_and(|path| {
                        path.qualifier().is_none()
                            && path
                                .syntax()
                                .parent()
                                .is_some_and(|node| node.kind() == SyntaxKind::PATH_EXPR)
                    })
        })
}

/// L1 a to d: the `let` of `function` that gives `name` its type at `call`.
pub(crate) fn find_local_binding_statement(
    function: &ast::Fn,
    call: &ast::MethodCallExpr,
    name: &str,
) -> Option<LocalBindingSyntaxValue> {
    let tokens = function
        .syntax()
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| {
            token.kind() == SyntaxKind::IDENT && token.text().trim_start_matches("r#") == name
        })
        .collect::<Vec<_>>();
    // a: one identifier pattern; every other token is a use of the variable.
    let patterns = tokens
        .iter()
        .filter_map(|token| {
            ast::Name::cast(token.parent()?)?
                .syntax()
                .parent()
                .and_then(ast::IdentPat::cast)
        })
        .collect::<Vec<_>>();
    let [pattern] = patterns.as_slice() else {
        return None;
    };
    let uses = tokens
        .iter()
        .filter(|token| check_plain_path_token(token))
        .count();
    // b: the whole pattern of a plain `let`.
    let statement = pattern.syntax().parent().and_then(ast::LetStmt::cast)?;
    let plain = pattern.name().is_some()
        && pattern.ref_token().is_none()
        && pattern.at_token().is_none()
        && pattern.pat().is_none()
        && statement.colon_token().is_none()
        && statement.ty().is_none()
        && statement.let_else().is_none();
    // c: in scope at the call, in the same function.
    let block = statement.syntax().parent()?;
    let own_item = |node: &ra_ap_syntax::SyntaxNode| {
        node.ancestors()
            .find_map(ast::Item::cast)
            .is_some_and(|item| item.syntax() == function.syntax())
    };
    let scoped = block.kind() == SyntaxKind::STMT_LIST
        && call.syntax().ancestors().any(|node| node == block)
        && statement.syntax().text_range().end() <= call.syntax().text_range().start()
        && own_item(statement.syntax())
        && own_item(call.syntax());
    // d: the initializer is a method call, through parentheses only.
    let initializer = std::iter::successors(statement.initializer(), |found| match found {
        ast::Expr::ParenExpr(wrapped) => wrapped.expr(),
        _ => None,
    })
    .last()
    .and_then(|found| match found {
        ast::Expr::MethodCallExpr(inner) => Some(inner),
        _ => None,
    })?;
    (uses + 1 == tokens.len() && plain && scoped).then(|| LocalBindingSyntaxValue {
        binding_let_statement: statement,
        binding_initializer_call: initializer,
        binding_name_token_count: tokens.len(),
        binding_mutable_flag: pattern.mut_token().is_some(),
    })
}

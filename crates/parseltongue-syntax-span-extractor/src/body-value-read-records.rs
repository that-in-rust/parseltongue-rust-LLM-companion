use ra_ap_syntax::{AstNode, SyntaxKind, SyntaxNode, ast, ast::HasLoopBody};

#[cfg(test)]
#[path = "body-value-read-tests.rs"]
mod body_value_read_tests;

/// Whether `path` is a whole path expression in a function body that may name
/// a `static` or `const`: it is not a call's callee and its single name is not
/// bound by a parameter or pattern in scope. Items and generic parameters are
/// left to name resolution, which knows the block scopes.
pub(super) fn check_body_value_path(path: &ast::Path) -> bool {
    let Some(expression) = path.syntax().parent().and_then(ast::PathExpr::cast) else {
        return false;
    };
    let site = expression.syntax();
    let in_body = site
        .ancestors()
        .find_map(ast::Fn::cast)
        .and_then(|function| function.body())
        .is_some_and(|body| body.syntax().text_range().contains_range(site.text_range()));
    let callee = site
        .parent()
        .and_then(ast::CallExpr::cast)
        .and_then(|call| call.expr())
        .is_some_and(|callee| callee.syntax() == site);
    in_body && !callee && !check_local_value_binding(path)
}

/// Whether the single name of `path` is `self` or is bound by a pattern whose
/// scope covers the path. A pattern of another function never covers it.
fn check_local_value_binding(path: &ast::Path) -> bool {
    let name = match (path.qualifier(), path.segment()) {
        (None, Some(segment)) => segment.syntax().text().to_string(),
        _ => return false,
    };
    let name = name.trim_start_matches("r#");
    let site = path.syntax();
    let function = site.ancestors().find(|node| node.kind() == SyntaxKind::FN);
    name == "self"
        || function.iter().any(|function| {
            function
                .descendants()
                .filter(|pattern| pattern.kind() == SyntaxKind::IDENT_PAT)
                .filter(|pattern| {
                    pattern
                        .children()
                        .filter_map(ast::Name::cast)
                        .any(|bound| bound.text().trim_start_matches("r#") == name)
                })
                .any(|pattern| check_pattern_scope_covers(&pattern, site))
        })
}

/// Whether the names `pattern` binds are in scope at `site`: after a `let`
/// to the end of its block, inside a match arm, inside the taken branch of an
/// `if let`, inside a loop body, or anywhere in a function or closure that
/// takes it as a parameter.
fn check_pattern_scope_covers(pattern: &SyntaxNode, site: &SyntaxNode) -> bool {
    let site = site.text_range();
    let within = |scope: Option<SyntaxNode>| {
        scope.is_some_and(|scope| scope.text_range().contains_range(site))
    };
    let Some(binder) = pattern.ancestors().find(|node| {
        matches!(
            node.kind(),
            SyntaxKind::LET_STMT
                | SyntaxKind::LET_EXPR
                | SyntaxKind::MATCH_ARM
                | SyntaxKind::FOR_EXPR
                | SyntaxKind::PARAM
        )
    }) else {
        return false;
    };
    let after = site.start() >= binder.text_range().end();
    match binder.kind() {
        SyntaxKind::LET_STMT => after && within(binder.parent()),
        SyntaxKind::MATCH_ARM => site.start() >= pattern.text_range().end() && within(Some(binder)),
        SyntaxKind::FOR_EXPR => within(
            ast::ForExpr::cast(binder)
                .and_then(|each| each.loop_body())
                .map(|body| body.syntax().clone()),
        ),
        SyntaxKind::PARAM => within(
            binder
                .ancestors()
                .find(|node| matches!(node.kind(), SyntaxKind::FN | SyntaxKind::CLOSURE_EXPR)),
        ),
        _ => {
            let scope = binder.ancestors().find(|node| {
                matches!(
                    node.kind(),
                    SyntaxKind::IF_EXPR | SyntaxKind::WHILE_EXPR | SyntaxKind::MATCH_ARM
                )
            });
            let taken = scope
                .clone()
                .and_then(ast::IfExpr::cast)
                .and_then(|branch| branch.then_branch())
                .is_none_or(|then| site.end() <= then.syntax().text_range().end());
            after && taken && within(scope)
        }
    }
}

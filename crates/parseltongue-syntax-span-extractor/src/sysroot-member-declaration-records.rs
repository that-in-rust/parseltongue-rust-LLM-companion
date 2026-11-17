//! REQ-RUST-054.0: the methods a sysroot source file declares, from syntax only.
use crate::macro_body_item_records::{
    count_macro_rule_arms, read_const_type_text, read_item_attribute_texts,
    read_macro_body_position,
};
use ra_ap_syntax::{
    AstNode, Direction, Edition, NodeOrToken, SourceFile, SyntaxKind, SyntaxNode, SyntaxToken,
    TextRange,
    ast::{self, HasAttrs, HasGenericParams, HasName, HasTypeBounds, HasVisibility},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
#[path = "sysroot-member-declaration-tests.rs"]
mod sysroot_member_declaration_tests;

/// REQ-RUST-054.0 D1 to D5: one row per `fn` written directly in an impl, in a trait
/// declaration, or in the body of a macro definition, ordered by the byte offset of its
/// name. Nothing is resolved.
pub fn extract_member_declaration_rows(
    source_relative_path: &str,
    source_text: &str,
) -> Vec<Value> {
    let tree = SourceFile::parse(source_text, Edition::Edition2021).tree();
    let mut rows = tree
        .syntax()
        .descendants()
        .flat_map(|node| match node.kind() {
            SyntaxKind::IMPL => ast::Impl::cast(node)
                .map(|owner| {
                    let mut rows = list_impl_member_rows(source_relative_path, &owner);
                    rows.extend(list_impl_macro_rows(source_relative_path, &owner));
                    rows
                })
                .unwrap_or_default(),
            // REQ-RUST-073.0 P1: the trait's own row, then the rows of its members.
            SyntaxKind::TRAIT => build_type_name_row(source_relative_path, &node)
                .into_iter()
                .chain(
                    ast::Trait::cast(node)
                        .map(|owner| list_trait_member_rows(source_relative_path, &owner))
                        .unwrap_or_default(),
                )
                .collect(),
            SyntaxKind::MACRO_RULES | SyntaxKind::MACRO_DEF => {
                let name = node
                    .children()
                    .find_map(ast::Name::cast)
                    .map(|name| name.text().to_string());
                list_macro_token_rows(source_relative_path, &node, "macro_definition_body", name)
            }
            SyntaxKind::MACRO_CALL => {
                let name = ast::MacroCall::cast(node.clone())
                    .and_then(|call| call.path())
                    .map(|path| read_syntax_node_text(path.syntax()));
                list_macro_token_rows(source_relative_path, &node, "macro_call_argument", name)
            }
            SyntaxKind::STRUCT
            | SyntaxKind::ENUM
            | SyntaxKind::UNION
            | SyntaxKind::TYPE_ALIAS
            | SyntaxKind::RENAME => build_type_name_row(source_relative_path, &node)
                .into_iter()
                .collect(),
            SyntaxKind::USE_TREE if source_relative_path.starts_with("core/src/prelude/") => {
                build_prelude_leaf_row(source_relative_path, &node)
                    .into_iter()
                    .collect()
            }
            _ => Vec::new(),
        })
        .chain(list_macro_call_rows(source_relative_path, tree.syntax()))
        .chain(list_file_scope_rows(source_relative_path, tree.syntax()))
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row["member_name_start_offset"].as_u64());
    rows
}

/// The name a `use` leaf binds, with the range that writes it, and the full path the leaf
/// names. `self` binds the name before it; `as _` binds nothing.
fn read_use_leaf_binding(tree: &ast::UseTree) -> (Option<(String, TextRange)>, String) {
    let mut written = tree
        .syntax()
        .ancestors()
        .filter_map(ast::UseTree::cast)
        .filter_map(|holder| holder.path())
        .map(|path| read_syntax_node_text(path.syntax()).replace(char::is_whitespace, ""))
        .collect::<Vec<_>>();
    written.reverse();
    let joined = written.join("::");
    let path = joined.strip_suffix("::self").unwrap_or(&joined).to_owned();
    let bound = match tree.rename() {
        Some(rename) => rename
            .name()
            .map(|name| (name.text().to_string(), name.syntax().text_range())),
        None => tree
            .path()
            .and_then(|path| path.segment())
            .and_then(|segment| segment.name_ref())
            .zip(path.rsplit("::").next().map(str::to_owned))
            .map(|(written, name)| (name, written.syntax().text_range())),
    };
    (bound, path)
}

/// REQ-RUST-062.0 C9: the rows of one direct child of a source file: what it may bind in
/// the type namespace of the file's module.
fn list_scope_binding_rows(path: &str, node: &SyntaxNode) -> Vec<Value> {
    let attributes = ast::AnyHasAttrs::cast(node.clone())
        .map(|owner| list_attribute_text_values(&owner))
        .unwrap_or_default();
    let row = |form: &str,
               kind: Option<&str>,
               bound: Option<(String, TextRange)>,
               leaf: Option<String>| {
        let range = bound
            .as_ref()
            .map_or_else(|| node.text_range(), |(_, range)| *range);
        json!({
            "declaration_owner_kind_value":"file_scope_name_binding",
            "declaration_source_relative_path":path,
            "member_name_start_offset":usize::from(range.start()),
            "member_name_end_offset":usize::from(range.end()),
            "member_declared_name_value":bound.map(|(name, _)| name).filter(|name| !name.is_empty()),
            "scope_binding_form_kind":form,
            "type_declaration_kind_value":kind,
            "use_leaf_path_text":leaf,
            "member_attribute_text_values":attributes,
        })
    };
    let named = || {
        node.children()
            .find_map(ast::Name::cast)
            .map(|name| (name.text().to_string(), name.syntax().text_range()))
    };
    let item = |kind| named().map(|bound| row("item", Some(kind), Some(bound), None));
    match node.kind() {
        SyntaxKind::STRUCT => item("struct").into_iter().collect(),
        SyntaxKind::ENUM => item("enum").into_iter().collect(),
        SyntaxKind::UNION => item("union").into_iter().collect(),
        SyntaxKind::TRAIT => item("trait").into_iter().collect(),
        SyntaxKind::TYPE_ALIAS => item("type_alias").into_iter().collect(),
        SyntaxKind::MODULE => {
            let kind = match ast::Module::cast(node.clone()).and_then(|module| module.item_list()) {
                Some(_) => "module_inline",
                None => "module_file",
            };
            named()
                .map(|bound| row("module", Some(kind), Some(bound), None))
                .into_iter()
                .collect()
        }
        SyntaxKind::EXTERN_CRATE => ast::ExternCrate::cast(node.clone())
            .and_then(|found| match found.rename() {
                Some(rename) => rename
                    .name()
                    .map(|name| (name.text().to_string(), name.syntax().text_range())),
                None => found
                    .name_ref()
                    .map(|name| (name.text().to_string(), name.syntax().text_range())),
            })
            .map(|bound| row("extern_crate", None, Some(bound), None))
            .into_iter()
            .collect(),
        SyntaxKind::USE => node
            .descendants()
            .filter_map(ast::UseTree::cast)
            .filter(|tree| tree.use_tree_list().is_none())
            .filter_map(|tree| {
                let (bound, leaf) = read_use_leaf_binding(&tree);
                match (tree.star_token(), bound) {
                    // A glob has no name: its row is at its `*`.
                    (Some(star), _) => Some(row(
                        "use_glob",
                        None,
                        Some((String::new(), star.text_range())),
                        Some(leaf),
                    )),
                    (None, Some(bound)) => Some(row("use_leaf", None, Some(bound), Some(leaf))),
                    (None, None) => None,
                }
            })
            .collect(),
        SyntaxKind::MACRO_CALL | SyntaxKind::EXTERN_BLOCK => {
            vec![row("unread_form", None, None, None)]
        }
        // REQ-RUST-063.0 M2
        SyntaxKind::MACRO_RULES => named()
            .map(|bound| {
                let mut found = row("macro_definition", None, Some(bound), None);
                found["macro_arm_count_value"] = json!(count_macro_rule_arms(node));
                found
            })
            .into_iter()
            .collect(),
        _ => Vec::new(),
    }
}

/// REQ-RUST-062.0 C9: one row per direct child of a source file that may bind a name in
/// the type namespace of the file's module.
fn list_file_scope_rows(path: &str, root: &SyntaxNode) -> Vec<Value> {
    root.children()
        .flat_map(|node| list_scope_binding_rows(path, &node))
        .collect()
}

/// Whether `node`, a trait or an impl, is a direct child of its source file.
fn check_file_scope_parent(node: &SyntaxNode) -> bool {
    node.parent()
        .is_some_and(|parent| parent.kind() == SyntaxKind::SOURCE_FILE)
}

/// What a member row says about the item that holds the member.
struct MemberOwnerSyntaxValue {
    owner_kind: &'static str,
    self_type: Option<String>,
    generic_parameters: Option<String>,
    generic_names: Option<Vec<String>>,
    bound_restriction: Option<bool>,
    /// REQ-RUST-073.0 P2, for an impl: each bound written on its type parameters, and
    /// whether it has a where clause.
    bound_texts: Option<Vec<String>>,
    where_clause: Option<bool>,
    trait_path: Option<String>,
    attributes: Vec<String>,
    function_body: bool,
    /// REQ-RUST-062.0 C9: the owner is a direct child of its source file.
    file_scope: bool,
    /// REQ-RUST-062.0 C3, for a trait declaration: the names of its generic parameters,
    /// of the associated types it declares, and those of both `?Sized` is written for.
    trait_generic_names: Option<Vec<String>>,
    trait_associated_names: Option<Vec<String>>,
    unsized_relaxed_names: Vec<String>,
}

/// The names of the type and const parameters of a generic parameter list.
fn list_parameter_list_names(list: Option<ast::GenericParamList>) -> Vec<String> {
    list.into_iter()
        .flat_map(|list| list.type_or_const_params())
        .filter_map(|parameter| match parameter {
            ast::TypeOrConstParam::Type(found) => found.name(),
            ast::TypeOrConstParam::Const(found) => found.name(),
        })
        .map(|name| name.text().to_string())
        .collect()
}

/// REQ-RUST-062.0 C3: the names of `candidates` that a bound taking the implicit `Sized`
/// away is written for: in the bounds of the name's own declaration, or anywhere in a
/// where clause of the owner, which then counts for every name.
fn list_unsized_relaxed_names(
    candidates: &[String],
    declarations: &[SyntaxNode],
    where_clause: Option<ast::WhereClause>,
) -> Vec<String> {
    // Each of these, written as a bound, takes the implicit `Sized` away.
    let relaxed = |node: &SyntaxNode| {
        let text = node.text().to_string();
        ["?Sized", "PointeeSized", "MetaSized"]
            .iter()
            .any(|bound| text.contains(bound))
    };
    if where_clause.is_some_and(|clause| relaxed(clause.syntax())) {
        return candidates.to_vec();
    }
    declarations
        .iter()
        .filter(|node| relaxed(node))
        .filter_map(|node| node.children().find_map(ast::Name::cast))
        .map(|name| name.text().to_string())
        .filter(|name| candidates.contains(name))
        .collect()
}

fn read_syntax_node_text(node: &SyntaxNode) -> String {
    node.text().to_string()
}

fn list_attribute_text_values(owner: &impl HasAttrs) -> Vec<String> {
    owner
        .attrs()
        .map(|attribute| read_syntax_node_text(attribute.syntax()))
        .collect()
}

fn check_function_body_ancestry(node: &SyntaxNode) -> bool {
    node.ancestors()
        .skip(1)
        .any(|ancestor| ancestor.kind() == SyntaxKind::BLOCK_EXPR)
}

fn list_impl_member_rows(path: &str, owner: &ast::Impl) -> Vec<Value> {
    let trait_path = owner.trait_().map(|ty| read_syntax_node_text(ty.syntax()));
    let value = MemberOwnerSyntaxValue {
        owner_kind: if trait_path.is_some() {
            "trait_impl"
        } else {
            "inherent_impl"
        },
        self_type: owner.self_ty().map(|ty| read_syntax_node_text(ty.syntax())),
        generic_parameters: owner
            .generic_param_list()
            .map(|list| read_syntax_node_text(list.syntax())),
        generic_names: Some(list_generic_parameter_names(owner)),
        bound_restriction: Some(check_impl_bound_restriction(owner)),
        bound_texts: Some(list_impl_bound_texts(owner)),
        where_clause: Some(owner.where_clause().is_some()),
        trait_path,
        attributes: list_attribute_text_values(owner),
        function_body: check_function_body_ancestry(owner.syntax()),
        file_scope: check_file_scope_parent(owner.syntax()),
        trait_generic_names: None,
        trait_associated_names: None,
        unsized_relaxed_names: list_unsized_relaxed_names(
            &list_generic_parameter_names(owner),
            &list_generic_declaration_nodes(owner.generic_param_list()),
            owner.where_clause(),
        ),
    };
    list_owned_function_rows(path, owner.assoc_item_list(), &value)
}

/// REQ-RUST-063.0 M1: one row per macro call written as a direct item of an impl that
/// is a direct child of its source file.
fn list_impl_macro_rows(path: &str, owner: &ast::Impl) -> Vec<Value> {
    owner
        .assoc_item_list()
        .filter(|_| check_file_scope_parent(owner.syntax()))
        .into_iter()
        .flat_map(|list| list.assoc_items())
        .filter_map(|item| match item {
            ast::AssocItem::MacroCall(call) => call.path().map(|name| (call, name)),
            _ => None,
        })
        .map(|(call, name)| {
            let range = name.syntax().text_range();
            json!({
                "declaration_source_relative_path":path,
                "member_name_start_offset":usize::from(range.start()),
                "member_name_end_offset":usize::from(range.end()),
                "member_declared_name_value":null,
                "member_name_parameter_flag":false,
                "declaration_owner_kind_value":"impl_macro_call",
                "owner_macro_name_value":read_syntax_node_text(name.syntax()),
                "impl_self_type_text":owner.self_ty().map(|ty| read_syntax_node_text(ty.syntax())),
                "owner_trait_path_text":owner.trait_().map(|ty| read_syntax_node_text(ty.syntax())),
                "impl_generic_parameter_text":owner
                    .generic_param_list()
                    .map(|list| read_syntax_node_text(list.syntax())),
                "owner_attribute_text_values":list_attribute_text_values(owner),
                "member_attribute_text_values":list_attribute_text_values(&call),
            })
        })
        .collect()
}

fn list_generic_declaration_nodes(list: Option<ast::GenericParamList>) -> Vec<SyntaxNode> {
    list.into_iter()
        .flat_map(|list| list.generic_params())
        .map(|parameter| parameter.syntax().clone())
        .collect()
}

fn list_trait_member_rows(path: &str, owner: &ast::Trait) -> Vec<Value> {
    let generics = list_parameter_list_names(owner.generic_param_list());
    let associated = owner
        .assoc_item_list()
        .into_iter()
        .flat_map(|list| list.assoc_items())
        .filter_map(|item| match item {
            ast::AssocItem::TypeAlias(found) => Some(found),
            _ => None,
        })
        .collect::<Vec<_>>();
    let associated_names = associated
        .iter()
        .filter_map(|found| found.name())
        .map(|name| name.text().to_string())
        .collect::<Vec<_>>();
    let names = generics
        .iter()
        .chain(&associated_names)
        .cloned()
        .collect::<Vec<_>>();
    let declarations = list_generic_declaration_nodes(owner.generic_param_list())
        .into_iter()
        .chain(associated.iter().map(|found| found.syntax().clone()))
        .collect::<Vec<_>>();
    let value = MemberOwnerSyntaxValue {
        owner_kind: "trait_declaration",
        self_type: None,
        generic_parameters: None,
        generic_names: None,
        bound_restriction: None,
        bound_texts: None,
        where_clause: None,
        trait_path: owner.name().map(|name| name.text().to_string()),
        attributes: list_attribute_text_values(owner),
        function_body: check_function_body_ancestry(owner.syntax()),
        file_scope: check_file_scope_parent(owner.syntax()),
        unsized_relaxed_names: list_unsized_relaxed_names(
            &names,
            &declarations,
            owner.where_clause(),
        ),
        trait_generic_names: Some(generics),
        trait_associated_names: Some(associated_names),
    };
    list_owned_function_rows(path, owner.assoc_item_list(), &value)
}

fn list_owned_function_rows(
    path: &str,
    members: Option<ast::AssocItemList>,
    owner: &MemberOwnerSyntaxValue,
) -> Vec<Value> {
    members
        .into_iter()
        .flat_map(|list| list.assoc_items())
        .filter_map(|item| match item {
            ast::AssocItem::Fn(function) => Some(function),
            _ => None,
        })
        .filter_map(|function| build_owned_function_row(path, &function, owner))
        .collect()
}

fn read_self_parameter_kind(parameter: &ast::SelfParam) -> &'static str {
    match (
        parameter.ty().is_some(),
        parameter.amp_token().is_some(),
        parameter.mut_token().is_some(),
    ) {
        (true, _, _) => "typed",
        (false, true, true) => "mutable_reference",
        (false, true, false) => "reference",
        (false, false, _) => "value",
    }
}

fn build_owned_function_row(
    path: &str,
    function: &ast::Fn,
    owner: &MemberOwnerSyntaxValue,
) -> Option<Value> {
    let name = function.name()?;
    let range = name.syntax().text_range();
    let parameter = function.param_list().and_then(|list| list.self_param());
    Some(json!({
        "declaration_source_relative_path":path,
        "member_name_start_offset":usize::from(range.start()),
        "member_name_end_offset":usize::from(range.end()),
        "member_declared_name_value":name.text().trim_start_matches("r#"),
        "member_name_parameter_flag":false,
        "declaration_owner_kind_value":owner.owner_kind,
        "impl_self_type_text":owner.self_type,
        "impl_generic_parameter_text":owner.generic_parameters,
        "impl_generic_parameter_names":owner.generic_names,
        "impl_bound_restriction_flag":owner.bound_restriction,
        "impl_bound_text_values":owner.bound_texts,
        "impl_where_clause_flag":owner.where_clause,
        "macro_self_parameter_flag":null,
        "macro_owner_shape_kind":null,
        "type_declaration_kind_value":null,
        "owner_trait_path_text":owner.trait_path,
        "owner_macro_name_value":null,
        "member_self_parameter_kind":parameter.as_ref().map_or("none", read_self_parameter_kind),
        "member_self_parameter_text":parameter.as_ref().map(|parameter| read_syntax_node_text(parameter.syntax())),
        "member_return_type_text":function.ret_type().and_then(|ret| ret.ty()).map(|ty| read_syntax_node_text(ty.syntax())),
        "member_attribute_text_values":list_attribute_text_values(function),
        "owner_attribute_text_values":owner.attributes,
        "owner_function_body_flag":owner.function_body,
        "owner_file_scope_flag":owner.file_scope,
        "trait_generic_parameter_names":owner.trait_generic_names,
        "trait_associated_type_names":owner.trait_associated_names,
        "owner_unsized_relaxed_names":owner.unsized_relaxed_names,
        "member_generic_parameter_names":list_parameter_list_names(function.generic_param_list()),
    }))
}

/// REQ-RUST-054.0 D6: the names of an impl's type and const parameters.
fn list_generic_parameter_names(owner: &ast::Impl) -> Vec<String> {
    owner
        .generic_param_list()
        .into_iter()
        .flat_map(|list| list.type_or_const_params())
        .filter_map(|parameter| match parameter {
            ast::TypeOrConstParam::Type(found) => found.name(),
            ast::TypeOrConstParam::Const(found) => found.name(),
        })
        .map(|name| name.text().to_string())
        .collect()
}

/// REQ-RUST-073.0 P2: the text of each bound written on a type parameter of the impl,
/// in source order.
fn list_impl_bound_texts(owner: &ast::Impl) -> Vec<String> {
    owner
        .generic_param_list()
        .into_iter()
        .flat_map(|list| list.type_or_const_params())
        .filter_map(|parameter| match parameter {
            ast::TypeOrConstParam::Type(found) => found.type_bound_list(),
            ast::TypeOrConstParam::Const(_) => None,
        })
        .flat_map(|list| list.bounds())
        .map(|bound| read_syntax_node_text(bound.syntax()).trim().to_owned())
        .collect()
}

/// REQ-RUST-054.0 D6: whether the impl may not apply to every type its self type
/// matches: a where clause, or a bound on a type parameter other than `?Sized`.
fn check_impl_bound_restriction(owner: &ast::Impl) -> bool {
    owner.where_clause().is_some()
        || list_impl_bound_texts(owner)
            .iter()
            .any(|bound| bound != "?Sized")
}

fn check_token_tree_opener(node: &SyntaxNode, opener: SyntaxKind) -> bool {
    node.kind() == SyntaxKind::TOKEN_TREE
        && node
            .first_token()
            .is_some_and(|token| token.kind() == opener)
}

/// REQ-RUST-054.0 D7: whether the parameter list after a `fn` name written as tokens
/// holds `self`.
fn check_macro_self_parameter(name: &SyntaxToken) -> bool {
    name.siblings_with_tokens(Direction::Next)
        .filter_map(NodeOrToken::into_node)
        .find(|node| node.kind() == SyntaxKind::TOKEN_TREE)
        .filter(|node| check_token_tree_opener(node, SyntaxKind::L_PAREN))
        .is_some_and(|parameters| {
            parameters
                .children_with_tokens()
                .filter_map(NodeOrToken::into_token)
                .any(|token| token.kind() == SyntaxKind::SELF_KW)
        })
}

/// REQ-RUST-054.0 D7: what the tokens before the braces that hold a `fn` say the braces
/// belong to. `unknown` when the `fn` is not written directly in braces after `impl` or
/// `trait`.
fn read_macro_owner_shape(function: &SyntaxToken) -> &'static str {
    let Some(body) = function
        .parent_ancestors()
        .find(|node| !check_token_tree_opener(node, SyntaxKind::L_PAREN))
        .filter(|node| check_token_tree_opener(node, SyntaxKind::L_CURLY))
    else {
        return "unknown";
    };
    let mut header = body
        .siblings_with_tokens(Direction::Prev)
        .skip(1)
        .take_while(|element| match element {
            NodeOrToken::Node(node) => !check_token_tree_opener(node, SyntaxKind::L_CURLY),
            NodeOrToken::Token(token) => token.kind() != SyntaxKind::SEMICOLON,
        })
        .filter_map(NodeOrToken::into_token)
        .map(|token| token.kind())
        .filter(|kind| !kind.is_trivia())
        .collect::<Vec<_>>();
    header.reverse();
    // (angle depth, the last keyword seen outside angle brackets, the token before)
    let (_, found, _) = header
        .iter()
        .take_while(|kind| **kind != SyntaxKind::WHERE_KW)
        .fold(
            (0i32, "unknown", SyntaxKind::WHITESPACE),
            |(depth, found, previous), kind| match kind {
                SyntaxKind::L_ANGLE => (depth + 1, found, *kind),
                SyntaxKind::R_ANGLE if previous != SyntaxKind::MINUS => (depth - 1, found, *kind),
                SyntaxKind::TRAIT_KW if depth == 0 && found == "unknown" => {
                    (depth, "trait_declaration", *kind)
                }
                SyntaxKind::IMPL_KW if depth == 0 && found == "unknown" => {
                    (depth, "inherent_impl", *kind)
                }
                SyntaxKind::FOR_KW if depth == 0 && found == "inherent_impl" => {
                    (depth, "trait_impl", *kind)
                }
                _ => (depth, found, *kind),
            },
        );
    found
}

/// The last name of the path a use tree writes. `prefix::{self}` writes the last name of
/// the prefix; `super`, `crate` and a bare `self` write none.
fn read_use_tree_name(tree: &ast::UseTree) -> Option<String> {
    let written = |tree: &ast::UseTree| {
        tree.path()?
            .segment()?
            .name_ref()
            .map(|name| name.text().to_string())
            .filter(|name| !matches!(name.as_str(), "self" | "super" | "crate"))
    };
    written(tree).or_else(|| {
        let list = tree.syntax().parent().and_then(ast::UseTreeList::cast)?;
        written(&list.syntax().parent().and_then(ast::UseTree::cast)?)
    })
}

fn read_previous_code_token(token: &SyntaxToken) -> Option<SyntaxToken> {
    std::iter::successors(token.prev_token(), SyntaxToken::prev_token)
        .find(|token| !token.kind().is_trivia())
}

/// REQ-RUST-054.0 D12: per macro name the file calls, in an item or inside tokens, the
/// names written in those calls and whether one passes a macro variable on. A call of a
/// macro variable (`$m!(..)`) is a row with no macro name.
fn list_macro_call_rows(path: &str, root: &SyntaxNode) -> Vec<Value> {
    root.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| token.kind() == SyntaxKind::BANG)
        .filter_map(|bang| {
            let arguments = bang
                .siblings_with_tokens(Direction::Next)
                .skip(1)
                .find(|element| !element.kind().is_trivia())?
                .into_node()
                .filter(|node| node.kind() == SyntaxKind::TOKEN_TREE)?;
            let name = read_previous_code_token(&bang)
                .filter(|token| token.kind() == SyntaxKind::IDENT)?;
            let known = read_previous_code_token(&name)
                .is_none_or(|token| token.kind() != SyntaxKind::DOLLAR);
            Some((name, known, arguments))
        })
        .fold(
            BTreeMap::<Option<String>, (SyntaxToken, BTreeSet<String>, bool)>::new(),
            |mut found, (name, known, arguments)| {
                let key = known.then(|| name.text().to_owned());
                let (_, names, passed) = found
                    .entry(key)
                    .or_insert_with(|| (name, BTreeSet::new(), false));
                arguments
                    .descendants_with_tokens()
                    .filter_map(|element| element.into_token())
                    .for_each(|token| match token.kind() {
                        SyntaxKind::IDENT => {
                            names.insert(token.text().to_owned());
                        }
                        SyntaxKind::DOLLAR => *passed = true,
                        _ => {}
                    });
                found
            },
        )
        .into_iter()
        .map(|(name, (first, names, passed))| {
            let range = first.text_range();
            json!({
                "declaration_source_relative_path":path,
                "member_name_start_offset":usize::from(range.start()),
                "member_name_end_offset":usize::from(range.end()),
                "member_declared_name_value":null,
                "member_name_parameter_flag":false,
                "declaration_owner_kind_value":"macro_call_name_tokens",
                "owner_macro_name_value":name,
                "macro_call_identifier_values":names,
                "macro_call_metavariable_flag":passed,
            })
        })
        .collect()
}

/// REQ-RUST-054.0 D9: the name a struct, enum, union, type alias outside an impl or
/// trait, or `use .. as NAME` gives to a type. The alias and the rename are rows because
/// either gives a second name to a type.
fn build_type_name_row(path: &str, node: &SyntaxNode) -> Option<Value> {
    let kind = match node.kind() {
        SyntaxKind::TRAIT => "trait",
        SyntaxKind::STRUCT => "struct",
        SyntaxKind::ENUM => "enum",
        SyntaxKind::UNION => "union",
        SyntaxKind::TYPE_ALIAS
            if node
                .parent()
                .is_none_or(|parent| parent.kind() != SyntaxKind::ASSOC_ITEM_LIST) =>
        {
            "type_alias"
        }
        SyntaxKind::RENAME
            if node
                .parent()
                .is_some_and(|parent| parent.kind() == SyntaxKind::EXTERN_CRATE) =>
        {
            "extern_crate_rename"
        }
        SyntaxKind::RENAME => "use_rename",
        _ => return None,
    };
    let name = node.children().find_map(ast::Name::cast)?;
    let range = name.syntax().text_range();
    let renamed = (kind == "use_rename")
        .then(|| node.parent().and_then(ast::UseTree::cast))
        .flatten()
        .and_then(|tree| read_use_tree_name(&tree));
    Some(json!({
        "rename_source_name_value":renamed,
        "declaration_source_relative_path":path,
        "member_name_start_offset":usize::from(range.start()),
        "member_name_end_offset":usize::from(range.end()),
        "member_declared_name_value":name.text().to_string(),
        "member_name_parameter_flag":false,
        // REQ-RUST-073.0 P1: a trait is not a type name; it has its own row kind.
        "declaration_owner_kind_value":if kind == "trait" {
            "trait_name_declaration"
        } else {
            "type_name_declaration"
        },
        "impl_self_type_text":null,
        "impl_generic_parameter_text":null,
        "impl_generic_parameter_names":null,
        "impl_bound_restriction_flag":null,
        "macro_self_parameter_flag":null,
        "macro_owner_shape_kind":null,
        "type_declaration_kind_value":kind,
        "owner_trait_path_text":null,
        "owner_macro_name_value":null,
        "member_self_parameter_kind":null,
        "member_self_parameter_text":null,
        "member_return_type_text":null,
        "member_attribute_text_values":[],
        "owner_attribute_text_values":ast::Trait::cast(node.clone())
            .map(|owner| list_attribute_text_values(&owner))
            .unwrap_or_default(),
        "owner_function_body_flag":check_function_body_ancestry(node),
    }))
}

/// REQ-RUST-054.0 D10: one name a prelude file re-exports. A glob is a row named `*`
/// that keeps the path it globs. A `use` inside a private inline module re-exports
/// nothing from the prelude, so it gives no row.
fn build_prelude_leaf_row(path: &str, node: &SyntaxNode) -> Option<Value> {
    let tree = ast::UseTree::cast(node.clone())?;
    if tree.use_tree_list().is_some()
        || node
            .ancestors()
            .filter_map(ast::Module::cast)
            .any(|module| module.visibility().is_none())
    {
        return None;
    }
    let (range, name, glob_path) = match tree.star_token() {
        Some(star) => {
            let mut prefixes = node
                .ancestors()
                .filter_map(ast::UseTree::cast)
                .filter_map(|holder| holder.path())
                .map(|prefix| read_syntax_node_text(prefix.syntax()))
                .collect::<Vec<_>>();
            prefixes.reverse();
            (star.text_range(), "*".to_owned(), Some(prefixes.join("::")))
        }
        None => {
            let name = tree.path()?.segment()?.name_ref()?;
            (name.syntax().text_range(), name.text().to_string(), None)
        }
    };
    // REQ-RUST-062.0 C9: the name the leaf binds and the full path it names.
    let (bound, leaf_path) = read_use_leaf_binding(&tree);
    Some(json!({
        "use_leaf_bound_name_value":bound.filter(|_| glob_path.is_none()).map(|(name, _)| name),
        "use_leaf_path_text":leaf_path,
        "declaration_source_relative_path":path,
        "member_name_start_offset":usize::from(range.start()),
        "member_name_end_offset":usize::from(range.end()),
        "member_declared_name_value":name,
        "member_name_parameter_flag":false,
        "declaration_owner_kind_value":"prelude_use_leaf",
        "impl_self_type_text":glob_path,
        "impl_generic_parameter_text":null,
        "impl_generic_parameter_names":null,
        "impl_bound_restriction_flag":null,
        "macro_self_parameter_flag":null,
        "macro_owner_shape_kind":null,
        "type_declaration_kind_value":null,
        "owner_trait_path_text":null,
        "owner_macro_name_value":null,
        "member_self_parameter_kind":null,
        "member_self_parameter_text":null,
        "member_return_type_text":null,
        "member_attribute_text_values":[],
        "owner_attribute_text_values":[],
        "owner_function_body_flag":check_function_body_ancestry(node),
    }))
}

/// One item written as tokens: its keyword, the tokens of its name, and what it is.
struct MacroTokenItemValue<'a> {
    item_kind: &'static str,
    declared_name: String,
    keyword_token: &'a SyntaxToken,
    name_first_token: &'a SyntaxToken,
    name_last_token: &'a SyntaxToken,
    name_parameter_flag: bool,
    written_type_text: Option<String>,
}

/// The item whose keyword is the token at `at`: `fn NAME`, `fn $name`, or
/// `const NAME :` (REQ-RUST-063.0 M3) that is not a const generic parameter.
fn read_macro_token_item(tokens: &[SyntaxToken], at: usize) -> Option<MacroTokenItemValue<'_>> {
    let [keyword, first, second] = tokens.get(at..at + 3)? else {
        return None;
    };
    let item = |item_kind, declared_name, last, parameter, written| MacroTokenItemValue {
        item_kind,
        declared_name,
        keyword_token: keyword,
        name_first_token: first,
        name_last_token: last,
        name_parameter_flag: parameter,
        written_type_text: written,
    };
    match (keyword.kind(), first.kind(), second.kind()) {
        (SyntaxKind::FN_KW, SyntaxKind::IDENT, _) => {
            Some(item("fn", first.text().to_owned(), first, false, None))
        }
        (SyntaxKind::FN_KW, SyntaxKind::DOLLAR, SyntaxKind::IDENT) => Some(item(
            "fn",
            format!("${}", second.text()),
            second,
            true,
            None,
        )),
        (SyntaxKind::CONST_KW, SyntaxKind::IDENT, SyntaxKind::COLON)
            if !at
                .checked_sub(1)
                .and_then(|before| tokens.get(before))
                .is_some_and(|before| {
                    matches!(before.kind(), SyntaxKind::L_ANGLE | SyntaxKind::COMMA)
                }) =>
        {
            Some(item(
                "const",
                first.text().to_owned(),
                first,
                false,
                Some(read_const_type_text(second)),
            ))
        }
        _ => None,
    }
}

/// REQ-RUST-054.0 D3, D8: `fn NAME` and `fn $name` written as tokens in a macro
/// definition or in the arguments of a macro call. REQ-RUST-063.0 M3: `const NAME`
/// too, and for a `macro_rules!` where in its body each item is.
fn list_macro_token_rows(
    path: &str,
    definition: &SyntaxNode,
    owner_kind: &'static str,
    macro_name: Option<String>,
) -> Vec<Value> {
    let tokens = definition
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| !token.kind().is_trivia())
        .collect::<Vec<SyntaxToken>>();
    let name_offset = definition
        .children()
        .find_map(ast::Name::cast)
        .map(|name| usize::from(name.syntax().text_range().start()));
    (0..tokens.len())
        .filter_map(|at| read_macro_token_item(&tokens, at))
        .map(|item| {
            let (attributes, boundary) = read_item_attribute_texts(item.keyword_token);
            let position = read_macro_body_position(item.keyword_token, definition);
            json!({
                "declaration_source_relative_path":path,
                "member_name_start_offset":usize::from(item.name_first_token.text_range().start()),
                "member_name_end_offset":usize::from(item.name_last_token.text_range().end()),
                "member_declared_name_value":item.declared_name,
                "member_name_parameter_flag":item.name_parameter_flag,
                "declaration_owner_kind_value":owner_kind,
                "impl_self_type_text":null,
                "impl_generic_parameter_text":null,
                "impl_generic_parameter_names":null,
                "impl_bound_restriction_flag":null,
                "macro_self_parameter_flag":item.item_kind == "fn"
                    && check_macro_self_parameter(item.name_last_token),
                "macro_owner_shape_kind":read_macro_owner_shape(item.keyword_token),
                "member_item_kind_value":item.item_kind,
                "member_written_type_text":item.written_type_text,
                "macro_arm_ordinal_value":position["macro_arm_ordinal_value"],
                "macro_body_nesting_depth":position["macro_body_nesting_depth"],
                "macro_body_repetition_flag":position["macro_body_repetition_flag"],
                "macro_item_boundary_flag":boundary,
                "type_declaration_kind_value":null,
                "owner_trait_path_text":null,
                "owner_macro_name_value":macro_name,
                "owner_macro_name_offset":name_offset,
                "member_self_parameter_kind":null,
                "member_self_parameter_text":null,
                "member_return_type_text":null,
                "member_attribute_text_values":attributes,
                "owner_attribute_text_values":[],
                "owner_function_body_flag":check_function_body_ancestry(definition),
            })
        })
        .collect()
}

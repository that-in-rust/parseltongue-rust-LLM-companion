//! REQ-RUST-054.0: the methods a sysroot source file declares, from syntax only.
use ra_ap_syntax::{
    AstNode, Direction, Edition, NodeOrToken, SourceFile, SyntaxKind, SyntaxNode, SyntaxToken,
    ast::{self, HasAttrs, HasGenericParams, HasName, HasTypeBounds},
};
use serde_json::{Value, json};

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
                .map(|owner| list_impl_member_rows(source_relative_path, &owner))
                .unwrap_or_default(),
            SyntaxKind::TRAIT => ast::Trait::cast(node)
                .map(|owner| list_trait_member_rows(source_relative_path, &owner))
                .unwrap_or_default(),
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
            _ => Vec::new(),
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row["member_name_start_offset"].as_u64());
    rows
}

/// What a member row says about the item that holds the member.
struct MemberOwnerSyntaxValue {
    owner_kind: &'static str,
    self_type: Option<String>,
    generic_parameters: Option<String>,
    generic_names: Option<Vec<String>>,
    bound_restriction: Option<bool>,
    trait_path: Option<String>,
    attributes: Vec<String>,
    function_body: bool,
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
        trait_path,
        attributes: list_attribute_text_values(owner),
        function_body: check_function_body_ancestry(owner.syntax()),
    };
    list_owned_function_rows(path, owner.assoc_item_list(), &value)
}

fn list_trait_member_rows(path: &str, owner: &ast::Trait) -> Vec<Value> {
    let value = MemberOwnerSyntaxValue {
        owner_kind: "trait_declaration",
        self_type: None,
        generic_parameters: None,
        generic_names: None,
        bound_restriction: None,
        trait_path: owner.name().map(|name| name.text().to_string()),
        attributes: list_attribute_text_values(owner),
        function_body: check_function_body_ancestry(owner.syntax()),
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

/// REQ-RUST-054.0 D6: whether the impl may not apply to every type its self type
/// matches: a where clause, or a bound on a type parameter other than `?Sized`.
fn check_impl_bound_restriction(owner: &ast::Impl) -> bool {
    owner.where_clause().is_some()
        || owner
            .generic_param_list()
            .into_iter()
            .flat_map(|list| list.type_or_const_params())
            .filter_map(|parameter| match parameter {
                ast::TypeOrConstParam::Type(found) => found.type_bound_list(),
                ast::TypeOrConstParam::Const(_) => None,
            })
            .flat_map(|list| list.bounds())
            .any(|bound| read_syntax_node_text(bound.syntax()).trim() != "?Sized")
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

/// REQ-RUST-054.0 D9: the name a struct, enum, union, type alias outside an impl or
/// trait, or `use .. as NAME` gives to a type. The alias and the rename are rows because
/// either gives a second name to a type.
fn build_type_name_row(path: &str, node: &SyntaxNode) -> Option<Value> {
    let kind = match node.kind() {
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
        SyntaxKind::RENAME => "use_rename",
        _ => return None,
    };
    let name = node.children().find_map(ast::Name::cast)?;
    let range = name.syntax().text_range();
    Some(json!({
        "declaration_source_relative_path":path,
        "member_name_start_offset":usize::from(range.start()),
        "member_name_end_offset":usize::from(range.end()),
        "member_declared_name_value":name.text().to_string(),
        "member_name_parameter_flag":false,
        "declaration_owner_kind_value":"type_name_declaration",
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
        "owner_attribute_text_values":[],
        "owner_function_body_flag":check_function_body_ancestry(node),
    }))
}

/// REQ-RUST-054.0 D3, D8: `fn NAME` and `fn $name` written as tokens in a macro
/// definition or in the arguments of a macro call.
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
    tokens
        .windows(3)
        .filter(|window| window[0].kind() == SyntaxKind::FN_KW)
        .filter_map(|window| match (window[1].kind(), window[2].kind()) {
            (SyntaxKind::IDENT, _) => {
                Some((window[1].text().to_owned(), &window[1], &window[1], false))
            }
            (SyntaxKind::DOLLAR, SyntaxKind::IDENT) => Some((
                format!("${}", window[2].text()),
                &window[1],
                &window[2],
                true,
            )),
            _ => None,
        })
        .map(|(name, first, last, parameter)| {
            json!({
                "declaration_source_relative_path":path,
                "member_name_start_offset":usize::from(first.text_range().start()),
                "member_name_end_offset":usize::from(last.text_range().end()),
                "member_declared_name_value":name,
                "member_name_parameter_flag":parameter,
                "declaration_owner_kind_value":owner_kind,
                "impl_self_type_text":null,
                "impl_generic_parameter_text":null,
                "impl_generic_parameter_names":null,
                "impl_bound_restriction_flag":null,
                "macro_self_parameter_flag":check_macro_self_parameter(last),
                "macro_owner_shape_kind":read_macro_owner_shape(first),
                "type_declaration_kind_value":null,
                "owner_trait_path_text":null,
                "owner_macro_name_value":macro_name,
                "member_self_parameter_kind":null,
                "member_self_parameter_text":null,
                "member_return_type_text":null,
                "member_attribute_text_values":[],
                "owner_attribute_text_values":[],
                "owner_function_body_flag":check_function_body_ancestry(definition),
            })
        })
        .collect()
}

//! REQ-RUST-054.0: the methods a sysroot source file declares, from syntax only.
use ra_ap_syntax::{
    AstNode, Edition, SourceFile, SyntaxKind, SyntaxNode, SyntaxToken,
    ast::{self, HasAttrs, HasGenericParams, HasName},
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
                list_macro_body_rows(source_relative_path, &node)
            }
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

/// REQ-RUST-054.0 D3: `fn NAME` and `fn $name` written as tokens in a macro definition.
fn list_macro_body_rows(path: &str, definition: &SyntaxNode) -> Vec<Value> {
    let macro_name = definition
        .children()
        .find_map(ast::Name::cast)
        .map(|name| name.text().to_string());
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
                "declaration_owner_kind_value":"macro_definition_body",
                "impl_self_type_text":null,
                "impl_generic_parameter_text":null,
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

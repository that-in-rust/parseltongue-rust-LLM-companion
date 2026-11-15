#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;

fn resolved_builtin_type(name: &str) -> Value {
    json!({
        "resolution_state_kind":"resolved",
        "target_identity":{"target_symbol_kind_value":"Builtin","target_builtin_name_value":name},
        "type_source_evidence":{"type_source_text":name}
    })
}

fn resolved_trait_type(name: &str) -> Value {
    json!({
        "resolution_state_kind":"resolved",
        "target_identity":{"target_symbol_kind_value":"Trait","target_name":name},
        "type_source_evidence":{"type_source_text":name}
    })
}

/// One stored impl header: `impl <trait_name> for <self_type>` with members
/// given as (name, syntax kind).
fn impl_header_fixture(
    self_type: Value,
    trait_name: Option<&str>,
    members: &[(&str, &str)],
) -> Value {
    json!({
        "impl_semantic_occurrence_key":format!("header {self_type} {trait_name:?}"),
        "impl_self_type_resolution":self_type,
        "impl_trait_type_resolution":trait_name.map(resolved_trait_type),
        "impl_generic_parameters_flag":false,
        "impl_negative_implementation_flag":false,
        "impl_member_occurrence_rows":members.iter().map(|(name, kind)| json!({
            "member_declared_name_value":name,
            "member_syntax_kind_value":kind,
            "member_source_occurrence_key":format!("source {trait_name:?} {name}"),
            "member_semantic_occurrence_key":format!("semantic {trait_name:?} {name}")
        })).collect::<Vec<_>>()
    })
}

fn find_fixture_const_target(headers: Vec<Value>) -> Option<(Value, Value)> {
    let index = ImplMemberLookupIndex {
        impl_header_value_rows: headers,
    };
    index
        .find_type_member_target("u8", &resolved_builtin_type("u8"), "N", "CONST")
        .and_then(|found| {
            found
                .member_target_occurrence
                .map(|member| (member.clone(), found.member_lookup_evidence.clone()))
        })
}

/// The outcome kind and the number of candidates the lookup records for
/// `u8::N` over `headers`.
fn read_fixture_lookup_outcome(headers: Vec<Value>) -> (String, usize) {
    let index = ImplMemberLookupIndex {
        impl_header_value_rows: headers,
    };
    let found = index
        .find_type_member_target("u8", &resolved_builtin_type("u8"), "N", "CONST")
        .unwrap();
    assert_eq!(
        found.member_lookup_evidence["sources_not_read"],
        json!([
            "inherent items of builtin type u8",
            "trait impls for u8 in crates whose source was not read"
        ])
    );
    (
        found.member_lookup_evidence["member_lookup_outcome_kind"]
            .as_str()
            .unwrap()
            .to_owned(),
        found.member_lookup_evidence["member_candidate_impl_rows"]
            .as_array()
            .unwrap()
            .len(),
    )
}

/// REQ-RUST-052.0 G5: a row without a target still lists what was looked at.
#[test]
fn test_blocked_lookup_keeps_candidates() {
    let header = |trait_name, members: &[(&str, &str)]| {
        impl_header_fixture(resolved_builtin_type("u8"), trait_name, members)
    };
    let outcome = |kind: &str, rows| (kind.to_owned(), rows);
    assert_eq!(
        read_fixture_lookup_outcome(vec![header(Some("A"), &[("other", "CONST")])]),
        outcome("no_stored_impl_member", 0)
    );
    assert_eq!(
        read_fixture_lookup_outcome(vec![header(Some("A"), &[("N", "CONST")])]),
        outcome("single_trait_impl_member", 1)
    );
    assert_eq!(
        read_fixture_lookup_outcome(vec![header(None, &[("N", "CONST")])]),
        outcome("single_member_unproven", 1)
    );
    assert_eq!(
        read_fixture_lookup_outcome(vec![
            header(Some("A"), &[("N", "CONST")]),
            header(None, &[("N", "FN")])
        ]),
        outcome("several_stored_impl_members", 2)
    );
    let index = ImplMemberLookupIndex {
        impl_header_value_rows: vec![impl_header_fixture(
            json!({"resolution_state_kind":"resolved","target_identity":{"target_symbol_kind_value":"Struct"}}),
            Some("A"),
            &[("N", "CONST")],
        )],
    };
    let found = index
        .find_type_member_target(
            "S",
            &json!({"resolution_state_kind":"resolved","target_identity":{"target_symbol_kind_value":"Struct"}}),
            "N",
            "CONST",
        )
        .unwrap();
    assert_eq!(
        found.member_lookup_evidence["sources_not_read"],
        json!(["blanket trait impls in crates whose source was not read"]),
        "a type of this crate has no unread inherent items"
    );
}

/// REQ-RUST-052.0 G5
#[test]
fn test_single_trait_impl_member() {
    let (member, proof) = find_fixture_const_target(vec![
        impl_header_fixture(resolved_builtin_type("u8"), Some("A"), &[("N", "CONST")]),
        impl_header_fixture(resolved_builtin_type("u8"), Some("B"), &[("other", "FN")]),
        impl_header_fixture(resolved_builtin_type("u16"), Some("A"), &[("N", "CONST")]),
    ])
    .unwrap();
    assert_eq!(member, "semantic Some(\"A\") N");
    assert_eq!(proof["self_type_source_text"], "u8");
    assert_eq!(proof["self_type_builtin_name"], "u8");
    assert_eq!(proof["impl_trait_source_text"], "A");
    assert_eq!(
        proof["member_lookup_outcome_kind"],
        "single_trait_impl_member"
    );
    assert!(proof["impl_header_occurrence"].is_string());
}

/// REQ-RUST-052.0 G5: an inherent member of the type shadows a trait member.
#[test]
fn test_inherent_member_blocks_target() {
    assert_eq!(
        find_fixture_const_target(vec![
            impl_header_fixture(resolved_builtin_type("u8"), Some("A"), &[("N", "CONST")]),
            impl_header_fixture(resolved_builtin_type("u8"), None, &[("N", "CONST")]),
        ]),
        None
    );
    assert_eq!(
        find_fixture_const_target(vec![impl_header_fixture(
            resolved_builtin_type("u8"),
            None,
            &[("N", "CONST")]
        )]),
        None,
        "an inherent member alone is not this rule's target"
    );
}

/// REQ-RUST-052.0 G5: two trait impls of the type declare the name.
#[test]
fn test_two_trait_impls_untargeted() {
    assert_eq!(
        find_fixture_const_target(vec![
            impl_header_fixture(resolved_builtin_type("u8"), Some("A"), &[("N", "CONST")]),
            impl_header_fixture(resolved_builtin_type("u8"), Some("B"), &[("N", "CONST")]),
        ]),
        None
    );
}

/// REQ-RUST-052.0 G5: an impl whose self type has no stored target may be
/// an impl of this type, so its member of the same name leaves no proof.
#[test]
fn test_unknown_self_impl_blocks() {
    let unknown = json!({"resolution_state_kind":"unresolved"});
    assert_eq!(
        find_fixture_const_target(vec![
            impl_header_fixture(resolved_builtin_type("u8"), Some("A"), &[("N", "CONST")]),
            impl_header_fixture(unknown.clone(), Some("B"), &[("N", "CONST")]),
        ]),
        None
    );
    assert!(
        find_fixture_const_target(vec![
            impl_header_fixture(resolved_builtin_type("u8"), Some("A"), &[("N", "CONST")]),
            impl_header_fixture(unknown, Some("B"), &[("other", "CONST")]),
        ])
        .is_some()
    );
}

/// REQ-RUST-052.0 G5: a generic impl, a negative impl, an impl whose trait
/// has no stored target, and a member of another kind prove nothing.
#[test]
fn test_unproven_impl_shapes_untargeted() {
    let plain = || impl_header_fixture(resolved_builtin_type("u8"), Some("A"), &[("N", "CONST")]);
    let change = |field: &str, value: Value| {
        let mut header = plain();
        header[field] = value;
        find_fixture_const_target(vec![header])
    };
    assert!(find_fixture_const_target(vec![plain()]).is_some());
    assert_eq!(change("impl_generic_parameters_flag", json!(true)), None);
    assert_eq!(
        change("impl_negative_implementation_flag", json!(true)),
        None
    );
    assert_eq!(
        change(
            "impl_trait_type_resolution",
            json!({"resolution_state_kind":"unresolved"})
        ),
        None
    );
    assert_eq!(
        find_fixture_const_target(vec![impl_header_fixture(
            resolved_builtin_type("u8"),
            Some("A"),
            &[("N", "FN")]
        )]),
        None
    );
    let index = ImplMemberLookupIndex {
        impl_header_value_rows: vec![plain()],
    };
    assert_eq!(
        index.find_type_member_target(
            "u8",
            &json!({"resolution_state_kind":"unresolved"}),
            "N",
            "CONST"
        ),
        None,
        "a prefix without a stored target has no record at all"
    );
}

/// REQ-RUST-052.0 G5
#[test]
fn test_owner_impl_self_type() {
    let index = ImplMemberLookupIndex {
        impl_header_value_rows: vec![
            impl_header_fixture(resolved_builtin_type("u8"), Some("A"), &[("N", "CONST")]),
            impl_header_fixture(resolved_builtin_type("u16"), Some("B"), &[("f", "FN")]),
        ],
    };
    assert_eq!(
        index.read_owner_self_type("source Some(\"B\") f"),
        Some(&resolved_builtin_type("u16"))
    );
    assert_eq!(
        index.read_owner_self_type("source of a free function"),
        None
    );
}

/// REQ-RUST-052.0 G5, Q20: expanded text spaces its tokens.
#[test]
fn test_type_relative_path_parts() {
    assert_eq!(
        read_type_relative_parts("Self ::MAX_STR_LEN"),
        Some(("Self", "MAX_STR_LEN"))
    );
    assert_eq!(read_type_relative_parts("u8::N"), Some(("u8", "N")));
    assert_eq!(read_type_relative_parts("N"), None);
    assert_eq!(read_type_relative_parts("a::b::N"), None);
    assert_eq!(read_type_relative_parts("<T as A>::N"), None);
    assert_eq!(read_type_relative_parts("Vec<u8>::N"), None);
}

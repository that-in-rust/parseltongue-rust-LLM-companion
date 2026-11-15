#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::original_call_resolution_tests::{
    OriginalCallFixtureStore, make_call_fixture_graph, make_call_fixture_occurrence,
    make_call_fixture_row, make_call_fixture_scope, make_call_fixture_unit,
};
use super::*;
use crate::syntax_namespace_record_codec::{
    encode_compilation_unit_value, encode_symbol_target_value,
};
use ra_ap_syntax::{Edition, SourceFile};
use serde_json::{Value, json};

/// For each call in `source`, in order: whether its first argument is the own
/// by-value `self` of the method it is written in.
fn read_own_self_flags(source: &str) -> Vec<bool> {
    SourceFile::parse(source, Edition::Edition2021)
        .tree()
        .syntax()
        .descendants()
        .filter_map(ast::CallExpr::cast)
        .map(|call| {
            ast::HasArgList::arg_list(&call)
                .and_then(|list| list.args().next())
                .is_some_and(|argument| check_own_self_argument(&argument))
        })
        .collect()
}

/// REQ-RUST-052.0 G12: the first argument is the method's own `self` only when it is
/// the bare path `self` and the nearest function declares `self` by value, untyped,
/// outside any block.
#[test]
fn test_own_self_argument_flag() {
    let method = |parameters: &str, call: &str| {
        read_own_self_flags(&format!(
            "impl A for B {{ fn w({parameters}) {{ {call}; }} }}"
        ))
    };
    assert_eq!(method("self", "T::f(self, 1)"), [true]);
    assert_eq!(method("mut self", "T::f(self)"), [true]);
    assert_eq!(method("self", "let c = || T::f(self)"), [true]);
    assert_eq!(method("&self", "T::f(self)"), [false]);
    assert_eq!(method("&mut self", "T::f(self)"), [false]);
    assert_eq!(method("self: Box<Self>", "T::f(self)"), [false]);
    assert_eq!(method("self", "T::f(1, self)"), [false]);
    assert_eq!(method("self", "T::f(self.0)"), [false]);
    assert_eq!(method("self", "T::f(&self)"), [false]);
    assert_eq!(method("self", "T::f(self::g)"), [false]);
    assert_eq!(method("self", "T::f()"), [false]);
    assert_eq!(method("this: B", "T::f(this)"), [false]);
    // The `self` of a method of an impl written inside a body is not the owner's.
    assert_eq!(
        method(
            "self",
            "struct X; impl X { fn m(self) { T::f(self); } } T::f(self)"
        ),
        [false, true]
    );
    assert_eq!(read_own_self_flags("fn f() { g(self); }"), [false]);
}

/// REQ-RUST-052.0 G12: the member is chosen only when the calling method and the
/// member take `self` by value in plain impls of one stored type, and exactly one
/// stored impl of the trait for that type declares the function.
#[test]
fn test_self_argument_member_rules() {
    let of = |identity: &str| {
        json!({"resolution_state_kind":"resolved", "target_identity":identity,
            "type_source_evidence":{"type_simple_path_flag":true}})
    };
    let member = |name: &str, kind: &str, receiver: &str| {
        json!({"member_declared_name_value":name, "member_syntax_kind_value":kind,
            "member_self_parameter_kind":receiver,
            "member_source_occurrence_key":format!("source {name}"),
            "member_semantic_occurrence_key":format!("semantic {name}")})
    };
    let header = |trait_target: Value, self_type: Value, members: Value| {
        json!({"impl_trait_type_resolution":trait_target, "impl_self_type_resolution":self_type,
            "impl_generic_parameters_flag":false, "impl_negative_implementation_flag":false,
            "impl_function_body_ancestry_flag":false, "impl_member_occurrence_rows":members})
    };
    let owner = || {
        header(
            Value::Null,
            of("u8"),
            json!([member("write", "FN", "value")]),
        )
    };
    let target = || header(of("T"), of("u8"), json!([member("fmt", "FN", "value")]));
    let site = json!({"site_owner_occurrence_key":"source write",
        "call_syntax_evidence":{"call_first_argument_self_flag":true,
            "member_declared_name_value":"fmt"}});
    let found = |headers: &[Value], site: &Value| {
        find_self_argument_member(headers, &json!("T"), site)
            .map(|(member, owner)| {
                format!(
                    "{} for {}",
                    member["member_semantic_occurrence_key"].as_str().unwrap(),
                    owner["impl_self_type_resolution"]["target_identity"]
                        .as_str()
                        .unwrap()
                )
            })
            .unwrap_or_else(|| "-".to_owned())
    };
    let edit = |mut value: Value, pointer: &str, new: Value| {
        *value.pointer_mut(pointer).unwrap() = new;
        value
    };
    assert_eq!(found(&[owner(), target()], &site), "semantic fmt for u8");
    // Impls of the trait for other types, and of other traits, do not count.
    let others = [
        header(of("T"), of("u16"), json!([member("fmt", "FN", "value")])),
        header(of("Other"), of("u8"), json!([member("fmt", "FN", "value")])),
        header(of("T"), of("u8"), json!([member("other", "FN", "value")])),
    ];
    let mut all = vec![owner(), target()];
    all.extend(others);
    assert_eq!(found(&all, &site), "semantic fmt for u8");
    // The call: its first argument is not the own `self`, or it has no owner.
    for (pointer, new) in [
        (
            "/call_syntax_evidence/call_first_argument_self_flag",
            json!(false),
        ),
        (
            "/call_syntax_evidence/call_first_argument_self_flag",
            Value::Null,
        ),
        (
            "/call_syntax_evidence/member_declared_name_value",
            json!("other"),
        ),
        ("/site_owner_occurrence_key", json!("source absent")),
        ("/site_owner_occurrence_key", Value::Null),
    ] {
        let site = edit(site.clone(), pointer, new.clone());
        assert_eq!(found(&[owner(), target()], &site), "-", "{pointer} {new}");
    }
    // The impl that holds the calling method, then the impl of the trait.
    let header_edits = [
        ("/impl_generic_parameters_flag", json!(true)),
        ("/impl_negative_implementation_flag", json!(true)),
        ("/impl_function_body_ancestry_flag", json!(true)),
        (
            "/impl_self_type_resolution/resolution_state_kind",
            json!("unresolved"),
        ),
        (
            "/impl_self_type_resolution/type_source_evidence/type_simple_path_flag",
            json!(false),
        ),
        ("/impl_self_type_resolution/target_identity", json!("u16")),
        (
            "/impl_member_occurrence_rows/0/member_self_parameter_kind",
            json!("reference"),
        ),
        (
            "/impl_member_occurrence_rows/0/member_self_parameter_kind",
            json!("typed"),
        ),
        (
            "/impl_member_occurrence_rows/0/member_self_parameter_kind",
            Value::Null,
        ),
        (
            "/impl_member_occurrence_rows/0/member_syntax_kind_value",
            json!("CONST"),
        ),
    ];
    for (pointer, new) in &header_edits {
        assert_eq!(
            found(&[edit(owner(), pointer, new.clone()), target()], &site),
            "-",
            "owner {pointer} {new}"
        );
        assert_eq!(
            found(&[owner(), edit(target(), pointer, new.clone())], &site),
            "-",
            "target {pointer} {new}"
        );
    }
    for (pointer, new) in [
        (
            "/impl_trait_type_resolution/resolution_state_kind",
            json!("unresolved"),
        ),
        (
            "/impl_trait_type_resolution/target_identity",
            json!("Other"),
        ),
    ] {
        assert_eq!(
            found(&[owner(), edit(target(), pointer, new.clone())], &site),
            "-",
            "target {pointer} {new}"
        );
    }
    // Two stored impls of the trait for the type: neither is chosen.
    assert_eq!(found(&[owner(), target(), target()], &site), "-");
    // An impl whose trait or self type has no stored target may be a second impl of
    // this trait for this type (`impl T<u16> for u8`, `impl<X> T<u16> for X`).
    let unstored = |pointer: &str| edit(target(), pointer, json!("unresolved"));
    for pointer in [
        "/impl_trait_type_resolution/resolution_state_kind",
        "/impl_self_type_resolution/resolution_state_kind",
    ] {
        assert_eq!(
            found(&[owner(), target(), unstored(pointer)], &site),
            "-",
            "{pointer}"
        );
    }
    // An inherent function of the same name is not the trait's.
    let inherent = header(Value::Null, of("u8"), json!([member("fmt", "FN", "value")]));
    assert_eq!(
        found(&[owner(), target(), inherent], &site),
        "semantic fmt for u8"
    );
    // A member written as a raw identifier is the same name.
    let raw = edit(
        target(),
        "/impl_member_occurrence_rows/0/member_declared_name_value",
        json!("r#fmt"),
    );
    assert_eq!(found(&[owner(), raw], &site), "semantic fmt for u8");
}

const BY_SELF: &str = "struct Buffer; trait T { fn helper(self); } \
    impl T for Buffer { fn helper(self) {} } \
    impl Buffer { fn caller(self, other: Buffer) { T::helper(self); } }";

/// A call fixture graph of `source` (a variant of `BY_SELF`) with the stored name
/// lookups of the trait `T`, of the callee `T::helper` and of the impl's trait, and
/// the graph targets of the trait's declaration of `helper` and of the impl's.
fn make_self_fixture_graph(
    source: &str,
) -> (
    OriginalCallFixtureStore,
    HldMaterializationInputRecord,
    [HldTargetIdentityValue; 2],
) {
    let (store, input) = make_call_fixture_graph(source);
    let unit = encode_compilation_unit_value(&make_call_fixture_unit());
    let item = |text: &str, kind: &str| {
        let start = source.find(text).unwrap();
        let end = start + text.len();
        let target = NameSymbolTargetKey::Definition(make_call_fixture_occurrence(
            start as u64,
            end as u64,
            text,
        ));
        let metadata = json!({"target_symbol_kind_value":kind,
            "target_source_file_path":"/fixture/src/lib.rs",
            "target_source_range_start":start, "target_source_range_end":end,
            "target_compilation_scope_record":{"unit_semantic_identity_record":unit}});
        (target, metadata)
    };
    let (trait_target, trait_metadata) = item("trait T { fn helper(self); }", "Trait");
    let (declared, function) = item("fn helper(self);", "Function");
    let (member, _) = item("fn helper(self) {}", "Function");
    let path = |text: &str, context: &str, target: &Value| {
        let start = source.rfind("T::helper").unwrap();
        make_call_fixture_row(
            "path-resolution-site-records",
            json!({"site_compilation_scope_record":{"unit_semantic_identity_record":unit},
                "site_source_absolute_path":"/fixture/src/lib.rs",
                "site_source_range_start":start, "site_source_range_end":start + text.len(),
                "site_expansion_provenance_chain":[],
                "site_path_context_kind":context,
                "path_site_source_text":text,
                "path_resolution_state_kind":"resolved",
                "path_unresolved_reason_kind":Value::Null,
                "path_target_symbol_rows":[target]}),
        )
    };
    let binding = |target: &NameSymbolTargetKey, metadata: &Value| {
        make_call_fixture_row(
            "syntax-namespace-input-items",
            json!({"binding_target_identity_key":encode_symbol_target_value(target),
                "binding_target_metadata_value":metadata}),
        )
    };
    let extra = vec![
        binding(&trait_target, &trait_metadata),
        binding(&declared, &function),
        path("T", "body_call_type_prefix", &trait_metadata),
        path("T::helper", "body_call_callee_path", &function),
    ];
    let mut rows = store.rows.lock().unwrap();
    // The impl header stage ran before the trait's lookup was stored: store its outcome.
    let mut stored = 0;
    for row in rows
        .iter_mut()
        .filter(|row| row.stage_owned_table_name == "impl-header-index-table")
    {
        let mut header: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
        let resolution = &mut header["impl_trait_type_resolution"];
        if resolution["type_source_evidence"]["type_source_text"] == "T" {
            resolution["resolution_state_kind"] = json!("resolved");
            resolution["target_identity"] = json!(encode_symbol_target_value(&trait_target));
            row.stage_row_payload_text = header.to_string();
            stored += 1;
        }
    }
    assert_eq!(stored, 1, "one impl of T in the fixture");
    rows.extend(extra);
    drop(rows);
    (store, input, [declared.into(), member.into()])
}

/// The one call row of the fixture: its target, dispatch kind and trait outcome.
fn read_self_fixture_row(
    store: &OriginalCallFixtureStore,
    input: &HldMaterializationInputRecord,
) -> (HldEdgeResolutionValue, String, String) {
    let edges = resolve_original_call_edges(
        store,
        &make_call_fixture_scope(),
        input,
        &RunCancelFlagValue::default(),
    )
    .unwrap();
    let [edge] = edges.as_slice() else {
        panic!("one call expected, found {}", edges.len());
    };
    let metadata: Value =
        serde_json::from_str(edge.edge_dispatch_metadata_text.as_deref().unwrap()).unwrap();
    let record = &metadata["trait_method_lookup_evidence"];
    assert_eq!(
        record["trait_member_candidate_rows"]
            .as_array()
            .unwrap()
            .len(),
        1,
        "{metadata}"
    );
    assert_eq!(
        record["receiver_type_stored_flag"],
        !record["receiver_self_type_identity"].is_null(),
        "{metadata}"
    );
    for field in [
        "trait_method_declaration_identity",
        "selected_member_occurrence_key",
        "receiver_impl_header_occurrence",
        "receiver_self_type_source_text",
    ] {
        assert_eq!(
            record[field].is_null(),
            record["receiver_self_type_identity"].is_null(),
            "{field} {metadata}"
        );
    }
    (
        edge.edge_resolution_value.clone(),
        metadata["dispatch_kind"].as_str().unwrap().to_owned(),
        record["trait_member_outcome_kind"]
            .as_str()
            .unwrap()
            .to_owned(),
    )
}

/// REQ-RUST-052.0 G12: `T::helper(self)` in a by-value method of `Buffer` has the
/// `helper` of `impl T for Buffer` as target. With another first argument, or when
/// that member is not a graph node, the target is the trait's declaration.
#[test]
fn test_trait_call_self_member() {
    let row = |target: &HldTargetIdentityValue, kind: &str, outcome: &str| {
        (
            HldEdgeResolutionValue::Extracted {
                target: target.clone(),
            },
            kind.to_owned(),
            outcome.to_owned(),
        )
    };
    let by_trait = |declared| row(declared, "trait_path_method", "trait_method_declaration");
    let (store, input, [declared, member]) = make_self_fixture_graph(BY_SELF);
    assert_ne!(declared, member);
    assert_eq!(
        read_self_fixture_row(&store, &input),
        row(
            &member,
            "trait_path_impl_method",
            "single_impl_member_by_self"
        )
    );
    let other = BY_SELF.replace("T::helper(self)", "T::helper(other)");
    let (store, input, [declared, _]) = make_self_fixture_graph(&other);
    assert_eq!(read_self_fixture_row(&store, &input), by_trait(&declared));
    let (store, mut input, [declared, member]) = make_self_fixture_graph(BY_SELF);
    input
        .input_node_records
        .retain(|node| node.node_target_reference != member);
    assert_eq!(read_self_fixture_row(&store, &input), by_trait(&declared));
}

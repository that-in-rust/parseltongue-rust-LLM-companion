#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::original_call_resolution_tests::{
    make_call_fixture_graph, make_call_fixture_occurrence, make_call_fixture_scope,
    read_method_receiver_evidence,
};
use super::*;
use crate::syntax_namespace_record_codec::encode_symbol_target_value;
use ra_ap_syntax::{Edition, SourceFile};
use serde_json::{Value, json};

/// REQ-RUST-052.0 G13: a method call's receiver evidence says the receiver is a cast,
/// gives the cast's type with its source range, and says whether the call is written
/// directly in the body of a function that is not inside a block.
#[test]
fn test_cast_receiver_syntax_evidence() {
    let read = |source: &str, call: &str| {
        let evidence = read_method_receiver_evidence(source, call);
        let cast = &evidence["receiver_cast_type_evidence"];
        let text = cast["span_byte_start_offset"]
            .as_u64()
            .zip(cast["span_byte_end_offset"].as_u64())
            .map(|(start, end)| &source[start as usize..end as usize]);
        assert_eq!(text, cast["type_source_text"].as_str(), "{evidence}");
        (
            evidence["receiver_shape_kind"].as_str().unwrap().to_owned(),
            text.map(str::to_owned),
            cast["type_simple_path_flag"].as_bool(),
            evidence["receiver_member_body_flag"].as_bool().unwrap(),
        )
    };
    let cast = |text: &str, simple: bool, body: bool| {
        (
            "cast_expression".to_owned(),
            Some(text.to_owned()),
            Some(simple),
            body,
        )
    };
    assert_eq!(
        read(
            "impl A for B { fn w(self) { (self as u64).go() } }",
            "(self as u64).go()"
        ),
        cast("u64", true, true)
    );
    assert_eq!(
        read("fn f(x: u8) { ((x as m::T)).go() }", "((x as m::T)).go()"),
        cast("m::T", true, true)
    );
    assert_eq!(
        read("fn f(x: u8) { (x as Vec<u8>).go() }", "(x as Vec<u8>).go()"),
        cast("Vec<u8>", false, true)
    );
    assert_eq!(
        read(
            "fn f(x: u8) { (x as *const u8).go() }",
            "(x as *const u8).go()"
        ),
        cast("*const u8", false, true)
    );
    assert_eq!(
        read("fn f() { let c = || (1 as u8).go(); }", "(1 as u8).go()"),
        cast("u8", true, true)
    );
    // A function or a const written inside a body is not the member the call's owner is.
    assert_eq!(
        read("fn f() { fn g() { (1 as u8).go() } }", "(1 as u8).go()"),
        cast("u8", true, false)
    );
    assert_eq!(
        read("fn f() { const C: u8 = (1 as u8).go(); }", "(1 as u8).go()"),
        cast("u8", true, false)
    );
    // A call that is not in a function at all.
    assert_eq!(
        read("const C: u8 = (1 as u8).go();", "(1 as u8).go()"),
        cast("u8", true, false)
    );
    assert_eq!(
        read("fn f(x: u8) { (x).go() }", "(x).go()"),
        ("other".to_owned(), None, None, true)
    );
    assert_eq!(
        read("fn f(x: u8) { x.go() }", "x.go()"),
        ("parameter_identifier".to_owned(), None, None, true)
    );
}

/// REQ-RUST-052.0 G13: only the whole simple path of the type a receiver is cast to
/// is sent to name lookup.
#[test]
fn test_cast_receiver_path_request() {
    let requested = |source: &str| {
        SourceFile::parse(source, Edition::Edition2021)
            .tree()
            .syntax()
            .descendants()
            .filter_map(ast::Path::cast)
            .filter(check_cast_receiver_type)
            .map(|path| path.syntax().text().to_string())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        requested(
            "fn f(x: u8) { (x as m::T).go(); y.h(x as u8); (x as Vec<Q>).k(); \
             let z = x as u16; ((x as u32)).go(); (x as u64).a().b(); }"
        ),
        ["m::T", "u32", "u64"]
    );
}

/// REQ-RUST-052.0 G13: the member is chosen only when one stored impl member has the
/// name among the impls that may be of the cast's type, it is a method of a trait
/// impl, and that trait is the trait of the impl the call is written directly in.
#[test]
fn test_cast_receiver_member_rules() {
    let of = |identity: &str| {
        json!({"resolution_state_kind":"resolved", "target_identity":identity,
            "type_source_evidence":{"type_source_text":identity}})
    };
    let unstored = || json!({"resolution_state_kind":"unresolved", "target_identity":null});
    let member = |name: &str, kind: &str, receiver: Value, owner: &str| {
        json!({"member_declared_name_value":name, "member_syntax_kind_value":kind,
            "member_self_parameter_kind":receiver,
            "member_source_occurrence_key":format!("source {owner} {name}"),
            "member_semantic_occurrence_key":format!("semantic {owner} {name}")})
    };
    let header = |trait_target: Value, self_type: Value, members: Value| {
        json!({"impl_trait_type_resolution":trait_target, "impl_self_type_resolution":self_type,
            "impl_generic_parameters_flag":false, "impl_negative_implementation_flag":false,
            "impl_function_body_ancestry_flag":false, "impl_member_occurrence_rows":members,
            "impl_semantic_occurrence_key":"impl"})
    };
    let write = |receiver: &str, owner: &str| member("write", "FN", json!(receiver), owner);
    let enclosing = || header(of("T"), of("isize"), json!([write("value", "isize")]));
    let target = || header(of("T"), of("i64"), json!([write("value", "i64")]));
    let site = json!({"site_owner_occurrence_key":"source isize write"});
    let receiver = json!({"receiver_cast_type_evidence":{"type_source_text":"i64"},
        "method_declared_name_value":"write", "receiver_member_body_flag":true});
    let found = |headers: Vec<Value>, site: &Value, receiver: &Value, cast_type: Value| {
        let index = ImplMemberLookupIndex {
            impl_header_value_rows: headers,
        };
        let (member, stopped, evidence) =
            find_cast_receiver_member(site, receiver, &cast_type, &index);
        assert_eq!(
            evidence["trait_scope_proof_kind"] == "enclosing_impl_trait",
            member.is_some(),
            "{evidence}"
        );
        assert_eq!(member.is_some(), stopped.is_empty(), "{evidence}");
        let held = site["site_owner_occurrence_key"] == "source isize write";
        for field in [
            "enclosing_impl_header_occurrence",
            "enclosing_impl_self_type_identity",
            "enclosing_impl_self_type_source_text",
        ] {
            assert_eq!(
                evidence[field].is_null(),
                !held || stopped == "cast_type_not_stored",
                "{field} {evidence}"
            );
        }
        if member.is_some() {
            assert_eq!(evidence["enclosing_impl_trait_identity"], "T", "{evidence}");
            assert_eq!(evidence["enclosing_impl_trait_source_text"], "T");
            assert_eq!(evidence["enclosing_impl_self_type_source_text"], "isize");
        }
        (
            member.map(|member| member["member_semantic_occurrence_key"].clone()),
            stopped,
            evidence["member_lookup_outcome_kind"]
                .as_str()
                .unwrap()
                .to_owned(),
        )
    };
    let plain = |headers: Vec<Value>| found(headers, &site, &receiver, of("i64"));
    let chosen = |kind: &str| (Some(json!("semantic i64 write")), "", kind.to_owned());
    let stopped = |stopped: &'static str, kind: &str| (None, stopped, kind.to_owned());
    let single = "single_trait_impl_member";
    assert_eq!(plain(vec![enclosing(), target()]), chosen(single));
    let by_reference = header(of("T"), of("i64"), json!([write("reference", "i64")]));
    assert_eq!(plain(vec![enclosing(), by_reference]), chosen(single));
    let raw = header(
        of("T"),
        of("i64"),
        json!([member("r#write", "FN", json!("value"), "i64")]),
    );
    assert_eq!(
        plain(vec![enclosing(), raw]),
        (Some(json!("semantic i64 r#write")), "", single.to_owned())
    );
    // The member is not a method.
    for receiver_kind in [json!("typed"), Value::Null] {
        let other = header(
            of("T"),
            of("i64"),
            json!([member("write", "FN", receiver_kind, "i64")]),
        );
        assert_eq!(
            plain(vec![enclosing(), other]),
            stopped("cast_member_not_method", single)
        );
    }
    // The cast's type has no stored target.
    assert_eq!(
        found(vec![enclosing(), target()], &site, &receiver, unstored()),
        stopped("cast_type_not_stored", "cast_type_not_stored")
    );
    // No member, a member of another kind, or more than one impl that may apply.
    assert_eq!(
        plain(vec![enclosing()]),
        stopped("cast_member_unproven", "no_stored_impl_member")
    );
    let constant = header(
        of("T"),
        of("i64"),
        json!([member("write", "CONST", Value::Null, "i64")]),
    );
    assert_eq!(
        plain(vec![enclosing(), constant]),
        stopped("cast_member_unproven", "single_member_unproven")
    );
    let second = header(of("U"), of("i64"), json!([write("value", "i64 second")]));
    assert_eq!(
        plain(vec![enclosing(), target(), second]),
        stopped("cast_member_unproven", "several_stored_impl_members")
    );
    let blanket = header(of("T"), unstored(), json!([write("value", "blanket")]));
    assert_eq!(
        plain(vec![enclosing(), target(), blanket]),
        stopped("cast_member_unproven", "several_stored_impl_members")
    );
    // The one member is in an inherent impl or in an impl with generic parameters.
    let inherent = header(Value::Null, of("i64"), json!([write("value", "i64")]));
    assert_eq!(
        plain(vec![enclosing(), inherent]),
        stopped("cast_member_unproven", "single_member_unproven")
    );
    for flag in [
        "impl_generic_parameters_flag",
        "impl_negative_implementation_flag",
    ] {
        let mut other = target();
        other[flag] = json!(true);
        assert_eq!(
            plain(vec![enclosing(), other]),
            stopped("cast_member_unproven", "single_member_unproven"),
            "{flag}"
        );
    }
    // The call names the method as a raw identifier.
    let mut raw_call = receiver.clone();
    raw_call["method_declared_name_value"] = json!("r#write");
    assert_eq!(
        found(vec![enclosing(), target()], &site, &raw_call, of("i64")),
        chosen(single)
    );
    // The trait is not proven to be in scope at the call.
    let scope = |headers: Vec<Value>, site: &Value, receiver: &Value| {
        assert_eq!(
            found(headers, site, receiver, of("i64")),
            stopped("cast_trait_scope_unproven", single)
        );
    };
    let other_trait = header(of("U"), of("isize"), json!([write("value", "isize")]));
    scope(vec![other_trait, target()], &site, &receiver);
    let no_trait = header(Value::Null, of("isize"), json!([write("value", "isize")]));
    scope(vec![no_trait, target()], &site, &receiver);
    let unstored_trait = header(unstored(), of("isize"), json!([write("value", "isize")]));
    scope(vec![unstored_trait, target()], &site, &receiver);
    let mut in_body = enclosing();
    in_body["impl_function_body_ancestry_flag"] = json!(true);
    scope(vec![in_body, target()], &site, &receiver);
    let free = json!({"site_owner_occurrence_key":"source free function"});
    scope(vec![enclosing(), target()], &free, &receiver);
    let mut nested = receiver.clone();
    nested["receiver_member_body_flag"] = json!(false);
    scope(vec![enclosing(), target()], &site, &nested);
}

const BY_CAST: &str = "struct Buffer; trait T { fn helper(self); fn caller(self); } \
    impl T for Buffer { fn helper(self) {} fn caller(self) { (self as Buffer).helper(); } }";

/// REQ-RUST-052.0 G13: the call producer, run on source text, gives
/// `(self as Buffer).helper()` the `helper` of `impl T for Buffer` as target. When
/// that member is not a graph node the row has no target and says so.
#[test]
fn test_cast_receiver_call_member() {
    let run = |keep_member: bool| {
        let (store, mut input) = make_call_fixture_graph(BY_CAST);
        let occurrence = |text: &str| {
            let start = BY_CAST.find(text).unwrap();
            NameSymbolTargetKey::Definition(make_call_fixture_occurrence(
                start as u64,
                (start + text.len()) as u64,
                text,
            ))
        };
        let trait_target = occurrence("trait T { fn helper(self); fn caller(self); }");
        let member = HldTargetIdentityValue::from(occurrence("fn helper(self) {}"));
        // The impl header stage ran before the trait's lookup was stored: store its outcome.
        let mut stored = 0;
        for row in store
            .rows
            .lock()
            .unwrap()
            .iter_mut()
            .filter(|row| row.stage_owned_table_name == "impl-header-index-table")
        {
            let mut header: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
            let resolution = &mut header["impl_trait_type_resolution"];
            resolution["resolution_state_kind"] = json!("resolved");
            resolution["target_identity"] = json!(encode_symbol_target_value(&trait_target));
            row.stage_row_payload_text = header.to_string();
            stored += 1;
        }
        assert_eq!(stored, 1, "one impl in the fixture");
        assert!(
            input
                .input_node_records
                .iter()
                .any(|node| node.node_target_reference == member)
        );
        if !keep_member {
            input
                .input_node_records
                .retain(|node| node.node_target_reference != member);
        }
        let edges = resolve_original_call_edges(
            &store,
            &make_call_fixture_scope(),
            &input,
            &RunCancelFlagValue::default(),
        )
        .unwrap();
        let [edge] = edges.as_slice() else {
            panic!("one call expected, found {}", edges.len());
        };
        let metadata: Value =
            serde_json::from_str(edge.edge_dispatch_metadata_text.as_deref().unwrap()).unwrap();
        let record = &metadata["cast_receiver_lookup_evidence"];
        assert_eq!(
            record["member_lookup_outcome_kind"], "single_trait_impl_member",
            "{metadata}"
        );
        assert_eq!(record["trait_scope_proof_kind"], "enclosing_impl_trait");
        assert_eq!(record["enclosing_impl_self_type_source_text"], "Buffer");
        (
            matches!(&edge.edge_resolution_value,
                HldEdgeResolutionValue::Extracted { target } if *target == member),
            metadata["dispatch_kind"].as_str().unwrap().to_owned(),
            metadata["unresolved_detail_kind"].clone(),
        )
    };
    assert_eq!(
        run(true),
        (true, "cast_receiver_impl_method".to_owned(), Value::Null)
    );
    assert_eq!(
        run(false),
        (
            false,
            "undetermined".to_owned(),
            json!("cast_member_not_node")
        )
    );
}

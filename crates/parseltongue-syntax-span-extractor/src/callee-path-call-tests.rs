#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::original_call_resolution_tests::{
    OriginalCallFixtureStore, make_call_fixture_graph, make_call_fixture_occurrence,
    make_call_fixture_row, make_call_fixture_scope, make_call_fixture_unit,
};
use super::*;
use crate::syntax_namespace_record_codec::{
    encode_compilation_unit_value, encode_symbol_target_value,
};
use serde_json::{Value, json};

/// A call fixture graph of `source` (which must declare `struct Buffer` and
/// `fn helper`) with one stored path row per entry of `paths`: (path text at
/// its last occurrence, context kind, state, target symbol kind or stored
/// reason). A target kind of `Function` names `helper`; any other kind is a
/// target row of that kind only. Name lookup outcomes are supplied here, at the
/// stage seam; the resolver under test only joins them.
fn make_callee_fixture_graph(
    source: &str,
    paths: &[(&str, &str, &str, &str)],
) -> (OriginalCallFixtureStore, HldMaterializationInputRecord) {
    let (store, input) = make_call_fixture_graph(source);
    let unit = encode_compilation_unit_value(&make_call_fixture_unit());
    let helper = store
        .rows
        .lock()
        .unwrap()
        .iter()
        .filter(|row| row.stage_owned_table_name == "span-reference-text-table")
        .map(|row| serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap())
        .find(|span| span["span_declared_name_value"] == "helper")
        .unwrap();
    let function = json!({"target_symbol_kind_value":"Function",
        "target_source_file_path":"/fixture/src/lib.rs",
        "target_source_range_start":helper["span_byte_start_offset"],
        "target_source_range_end":helper["span_byte_end_offset"],
        "target_compilation_scope_record":{"unit_semantic_identity_record":unit}});
    let target = NameSymbolTargetKey::Definition(make_call_fixture_occurrence(
        helper["span_byte_start_offset"].as_u64().unwrap(),
        helper["span_byte_end_offset"].as_u64().unwrap(),
        helper["span_source_code_text"].as_str().unwrap(),
    ));
    let mut extra = vec![make_call_fixture_row(
        "syntax-namespace-input-items",
        json!({"binding_target_identity_key":encode_symbol_target_value(&target),
            "binding_target_metadata_value":function}),
    )];
    for (text, context, state, kind) in paths {
        let start = source.rfind(text).unwrap();
        let (targets, reason) = match (*state, *kind) {
            ("resolved", "Function") => (json!([function]), Value::Null),
            ("resolved", "Function+Function") => (json!([function, function]), Value::Null),
            // A struct bound in the Value namespace, standing where `helper` is.
            ("resolved", "Struct") => {
                let mut constructor = function.clone();
                constructor["target_symbol_kind_value"] = json!("Struct");
                extra.push(make_call_fixture_row(
                    "syntax-namespace-input-items",
                    json!({"binding_target_identity_key":encode_symbol_target_value(&target),
                        "binding_target_metadata_value":constructor}),
                ));
                (json!([constructor]), Value::Null)
            }
            ("resolved", "Builtin") => (
                json!([{"target_symbol_kind_value":"Builtin", "target_builtin_name_value":"str"}]),
                Value::Null,
            ),
            ("resolved", kind) => (json!([{"target_symbol_kind_value":kind}]), Value::Null),
            (_, "") => (json!([]), Value::Null),
            (_, reason) => (json!([]), json!(reason)),
        };
        extra.push(make_call_fixture_row(
            "path-resolution-site-records",
            json!({"site_compilation_scope_record":{"unit_semantic_identity_record":unit},
                "site_source_absolute_path":"/fixture/src/lib.rs",
                "site_source_range_start":start, "site_source_range_end":start + text.len(),
                "site_expansion_provenance_chain":[],
                "site_path_context_kind":context,
                "path_site_source_text":text,
                "path_resolution_state_kind":state,
                "path_unresolved_reason_kind":reason,
                "path_target_symbol_rows":targets}),
        ));
    }
    store.rows.lock().unwrap().extend(extra);
    (store, input)
}

/// The one call row of `caller`: its target's name or `unresolved:<reason>`,
/// its dispatch kind, and what stopped the lookup.
fn read_callee_fixture_row(
    store: &OriginalCallFixtureStore,
    input: &HldMaterializationInputRecord,
) -> (String, String, String) {
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
    let target = match &edge.edge_resolution_value {
        HldEdgeResolutionValue::Extracted {
            target: HldTargetIdentityValue::ExternalNamed(key),
        } => format!("external:{}", key.external_item_path_text),
        HldEdgeResolutionValue::Extracted { target } => input
            .input_node_records
            .iter()
            .find(|node| &node.node_target_reference == target)
            .unwrap()
            .node_qualified_path_text
            .clone(),
        HldEdgeResolutionValue::Unresolved { reason } => format!("unresolved:{reason:?}"),
        other => panic!("neither extracted nor unresolved: {other:?}"),
    };
    let outcome = metadata["callee_path_lookup_evidence"]["callee_path_outcome_kind"]
        .as_str()
        .unwrap_or("-")
        .to_owned();
    let detail = metadata["unresolved_detail_kind"]
        .as_str()
        .unwrap_or("-")
        .to_owned();
    // A row the callee path decided repeats the outcome as what stopped it.
    assert!(
        !detail.starts_with("callee_")
            || detail == outcome
            || [
                "callee_path_not_requested",
                "callee_prefix_lexically_shadowed"
            ]
            .contains(&detail.as_str()),
        "{metadata}"
    );
    (
        target,
        metadata["dispatch_kind"].as_str().unwrap().to_owned(),
        format!("{detail} {outcome}"),
    )
}

/// Expected row: target, dispatch kind, "<what stopped it> <callee path outcome>".
fn row(target: &str, kind: &str, detail: &str) -> (String, String, String) {
    (target.to_owned(), kind.to_owned(), detail.to_owned())
}

const BARE: &str = "struct Buffer; fn helper() {} fn caller() { helper(); }";
const CALLEE: &str = "body_call_callee_path";
const NONE: &str = "undetermined";

/// REQ-RUST-052.0 G9
#[test]
fn test_bare_callee_names_function() {
    let (store, input) =
        make_callee_fixture_graph(BARE, &[("helper", CALLEE, "resolved", "Function")]);
    assert_eq!(
        read_callee_fixture_row(&store, &input),
        row("helper", "callee_path_function", "- single_function_node")
    );
    let generic = BARE.replace("helper();", "helper::<u8>();");
    let (store, input) = make_callee_fixture_graph(
        &generic,
        &[("helper::<u8>", CALLEE, "resolved", "Function")],
    );
    assert_eq!(
        read_callee_fixture_row(&store, &input),
        row("helper", "callee_path_function", "- single_function_node"),
        "generic arguments on the callee do not change the function named"
    );
}

/// REQ-RUST-052.0 G9: a bare callee without a function target keeps one row and says why.
#[test]
fn test_bare_callee_untargeted_reasons() {
    let outcome = |paths: &[(&str, &str, &str, &str)]| {
        let (store, input) = make_callee_fixture_graph(BARE, paths);
        read_callee_fixture_row(&store, &input)
    };
    let cases = [
        // No stored lookup for the callee: the row is as it was before rule G9.
        (vec![], "TraitSolverNeeded", "callee_path_not_requested -"),
        (
            vec![("helper", CALLEE, "resolved", "Const")],
            "TraitSolverNeeded",
            "callee_is_not_function callee_is_not_function",
        ),
        (
            vec![("helper", CALLEE, "unresolved", "external_source_missing")],
            "ExternalSourceMissing",
            "callee_path_unresolved callee_path_unresolved",
        ),
        (
            vec![("helper", CALLEE, "unresolved", "")],
            "TraitSolverNeeded",
            "callee_path_unresolved callee_path_unresolved",
        ),
        (
            vec![("helper", CALLEE, "ambiguous", "")],
            "AmbiguousTarget",
            "callee_path_ambiguous callee_path_ambiguous",
        ),
        // Stored as resolved, with two targets: never one of them picked.
        (
            vec![("helper", CALLEE, "resolved", "Function+Function")],
            "AmbiguousTarget",
            "callee_path_ambiguous callee_path_ambiguous",
        ),
    ];
    for (paths, reason, detail) in cases {
        assert_eq!(
            outcome(&paths),
            row(&format!("unresolved:{reason}"), NONE, detail),
            "{paths:?}"
        );
    }
    let (store, mut input) =
        make_callee_fixture_graph(BARE, &[("helper", CALLEE, "resolved", "Function")]);
    input
        .input_node_records
        .retain(|node| node.node_qualified_path_text != "helper");
    assert_eq!(
        read_callee_fixture_row(&store, &input),
        row(
            "unresolved:MissingTarget",
            NONE,
            "callee_target_not_node callee_target_not_node"
        ),
        "a function that is not a graph node, such as one declared in a block"
    );
}

/// REQ-RUST-052.0 G9, G10, G11: a qualified callee is decided by its whole path only
/// when its qualifier is stored as a module or a trait; otherwise the row keeps the
/// record of the whole path and goes to the type-prefix lookup.
#[test]
fn test_qualified_callee_needs_module() {
    let source = "struct Buffer; fn helper() {} fn caller() { m::helper(); }";
    let outcome = |prefix: (&str, &str), callee: (&str, &str)| {
        let (store, input) = make_callee_fixture_graph(
            source,
            &[
                ("m", "body_call_type_prefix", prefix.0, prefix.1),
                ("m::helper", CALLEE, callee.0, callee.1),
            ],
        );
        read_callee_fixture_row(&store, &input)
    };
    let missing = "external_source_missing";
    let cases = [
        (
            ("resolved", "Module"),
            ("resolved", "Function"),
            "helper",
            "callee_path_function",
            "- single_function_node",
        ),
        // A trait's method named through the trait is not this rule's.
        (
            ("resolved", "Trait"),
            ("resolved", "Function"),
            "helper",
            "trait_path_method",
            "- single_trait_method",
        ),
        // Through a trait, only a function is a target.
        (
            ("resolved", "Trait"),
            ("resolved", "Struct"),
            "unresolved:TraitSolverNeeded",
            NONE,
            "type_prefix_is_trait callee_qualifier_not_module",
        ),
        (
            ("resolved", "Trait"),
            ("resolved", "Const"),
            "unresolved:TraitSolverNeeded",
            NONE,
            "type_prefix_is_trait callee_is_not_function",
        ),
        // A function named through a type is a member: the path does not decide it,
        // the member lookup of the type does.
        (
            ("resolved", "Struct"),
            ("resolved", "Function"),
            "unresolved:MissingTarget",
            NONE,
            "member_lookup_unproven callee_qualifier_not_module",
        ),
        // A module qualifier whose whole path has no target: the older outcome stands.
        (
            ("resolved", "Module"),
            ("unresolved", ""),
            "unresolved:TraitSolverNeeded",
            NONE,
            "type_prefix_is_module callee_path_unresolved",
        ),
        (
            ("resolved", "Module"),
            ("resolved", "Struct"),
            "helper",
            "tuple_struct_constructor",
            "- single_struct_node",
        ),
        (
            ("resolved", "Module"),
            ("resolved", "Const"),
            "unresolved:TraitSolverNeeded",
            NONE,
            "type_prefix_is_module callee_is_not_function",
        ),
        (
            ("resolved", "Module"),
            ("ambiguous", ""),
            "unresolved:TraitSolverNeeded",
            NONE,
            "type_prefix_is_module callee_path_ambiguous",
        ),
        // The whole path's stored reason wins over a qualifier stored as a builtin type.
        (
            ("resolved", "Builtin"),
            ("unresolved", missing),
            "unresolved:ExternalSourceMissing",
            NONE,
            "callee_path_unresolved callee_path_unresolved",
        ),
        // A builtin qualifier whose whole path has no stored reason goes to member lookup,
        // which names the member of the builtin type (REQ-RUST-053.0 N3). The fixture
        // stores the qualifier `m` as the builtin `str`.
        (
            ("resolved", "Builtin"),
            ("unresolved", ""),
            "external:str::helper",
            "external_builtin_member",
            "- callee_path_unresolved",
        ),
        // A qualifier without a target keeps its own outcome.
        (
            ("unresolved", missing),
            ("unresolved", missing),
            "unresolved:ExternalSourceMissing",
            NONE,
            "type_prefix_unresolved callee_path_unresolved",
        ),
    ];
    for (prefix, callee, target, kind, detail) in cases {
        assert_eq!(
            outcome(prefix, callee),
            row(target, kind, detail),
            "{prefix:?} {callee:?}"
        );
    }
}

/// REQ-RUST-052.0 G9: the whole path's stored reason does not displace the member
/// lookup of a struct qualifier, the only lookup that can give such a call a target.
#[test]
fn test_struct_qualifier_keeps_member() {
    let source = "struct Buffer; impl Buffer { fn new() {} } fn helper() {} \
        fn caller() { Buffer::new(); }";
    let (store, input) = make_callee_fixture_graph(
        source,
        &[(
            "Buffer::new",
            CALLEE,
            "unresolved",
            "external_source_missing",
        )],
    );
    assert_eq!(
        read_callee_fixture_row(&store, &input),
        row(
            "new",
            "type_path_associated_function",
            "- callee_path_unresolved"
        )
    );
}

/// REQ-RUST-052.0 G9: stored lookups for a qualifier that a name in the function
/// shadows are not used, as for a type prefix (REQ-RUST-034.0). No record is kept.
#[test]
fn test_shadowed_qualifier_blocks_callee() {
    let source = "struct Buffer; fn helper() {} fn caller(m: u8) { m::helper(); }";
    let (store, input) = make_callee_fixture_graph(
        source,
        &[
            ("m", "body_call_type_prefix", "resolved", "Module"),
            ("m::helper", CALLEE, "resolved", "Function"),
        ],
    );
    let (target, kind, detail) = read_callee_fixture_row(&store, &input);
    assert!(target.starts_with("unresolved:"), "{target}");
    assert_eq!(
        (kind.as_str(), detail.as_str()),
        (NONE, "callee_prefix_lexically_shadowed -")
    );
}

/// REQ-RUST-052.0 G11: a struct found for a bare callee is a tuple struct's
/// constructor and is the call's target.
#[test]
fn test_tuple_struct_callee_target() {
    let (store, input) =
        make_callee_fixture_graph(BARE, &[("helper", CALLEE, "resolved", "Struct")]);
    assert_eq!(
        read_callee_fixture_row(&store, &input),
        row("helper", "tuple_struct_constructor", "- single_struct_node")
    );
    let (store, mut input) =
        make_callee_fixture_graph(BARE, &[("helper", CALLEE, "resolved", "Struct")]);
    input
        .input_node_records
        .retain(|node| node.node_qualified_path_text != "helper");
    assert_eq!(
        read_callee_fixture_row(&store, &input),
        row(
            "unresolved:MissingTarget",
            NONE,
            "callee_target_not_node callee_target_not_node"
        )
    );
}

/// REQ-RUST-052.0 G10: the candidates of a trait's method named through the trait are
/// the functions of that name in the stored impls of that trait, and nothing else.
#[test]
fn test_trait_method_candidate_rows() {
    let header = |trait_target: Value, self_text: &str, members: Value| {
        json!({"impl_semantic_occurrence_key":format!("impl for {self_text}"),
            "impl_trait_type_resolution":trait_target,
            "impl_self_type_resolution":{"resolution_state_kind":"resolved",
                "type_source_evidence":{"type_source_text":self_text}},
            "impl_generic_parameters_flag":false,
            "impl_member_occurrence_rows":members})
    };
    let of =
        |identity: &str| json!({"resolution_state_kind":"resolved", "target_identity":identity});
    let member = |name: &str, kind: &str| {
        json!({"member_declared_name_value":name, "member_syntax_kind_value":kind,
            "member_semantic_occurrence_key":format!("{kind} {name}")})
    };
    let headers = [
        header(
            of("T"),
            "u8",
            json!([member("other", "FN"), member("fmt", "FN")]),
        ),
        header(of("T"), "u16", json!([member("fmt", "CONST")])),
        header(of("T"), "u32", json!([member("other", "FN")])),
        header(of("Other"), "u64", json!([member("fmt", "FN")])),
        header(Value::Null, "u128", json!([member("fmt", "FN")])),
        header(
            json!({"resolution_state_kind":"unresolved", "target_identity":"T"}),
            "i8",
            json!([member("fmt", "FN")]),
        ),
    ];
    let evidence = list_trait_method_candidates(&headers, &json!("T"), &json!("fmt"));
    assert_eq!(
        evidence["trait_member_candidate_rows"],
        json!([{"impl_header_occurrence":"impl for u8", "impl_self_type_source_text":"u8",
            "impl_self_type_stored_flag":true, "impl_generic_parameters_flag":false,
            "member_semantic_occurrence_key":"FN fmt"}])
    );
    assert_eq!(
        evidence["trait_member_outcome_kind"],
        "trait_method_declaration"
    );
    assert_eq!(evidence["receiver_type_stored_flag"], false);
    assert_eq!(evidence["sources_not_read"].as_array().unwrap().len(), 1);
}

/// REQ-RUST-053.0 N1, N4, N5: a callee path whose stored lookup proves an external name
/// targets that name and keeps how the site reached it. No name: without a stored proof,
/// or when the path is qualified and a trait in read source declares its last name.
#[test]
fn test_external_callee_names_target() {
    let missing = "external_source_missing";
    let import = json!({"span_byte_range_value":[0, 9]});
    let outcome_stored = |source: &str, callee: &str, record: Value, stored: &str| {
        let qualified = callee.contains("::");
        let prefix = [("m", "body_call_type_prefix", "unresolved", missing)];
        let mut paths = vec![(callee, CALLEE, "unresolved", stored)];
        paths.extend(prefix.iter().filter(|_| qualified));
        let (store, mut input) = make_callee_fixture_graph(source, &paths);
        // The fixture graph holds no trait member; a source that declares one gets its node.
        if source.contains("trait Shape") {
            let member = HldNodeInputRecord {
                node_owner_kind_value: Some(HldOwnerKindValue::Trait),
                node_qualified_path_text: "Shape::r#size_of".into(),
                ..input.input_node_records[0].clone()
            };
            input.input_node_records.push(member);
        }
        for row in store.rows.lock().unwrap().iter_mut() {
            let mut path: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
            if path["path_site_source_text"] == callee {
                path["path_external_name_record"] = record.clone();
                row.stage_row_payload_text = path.to_string();
            }
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
        (edge.edge_resolution_value.clone(), metadata)
    };
    let outcome =
        |source: &str, callee: &str, record: Value| outcome_stored(source, callee, record, missing);
    let record = |kind: &str, crate_name: Value, path: &str, proof: &str| {
        json!({"external_name_kind":kind, "external_crate_name":crate_name,
            "external_item_path_text":path, "external_site_proof_kind":proof,
            "external_import_site_key":import})
    };
    let named = |kind, crate_name: Option<&str>, path: &str| HldEdgeResolutionValue::Extracted {
        target: HldTargetIdentityValue::ExternalNamed(HldExternalNamedKey {
            external_name_kind_value: kind,
            external_crate_name_value: crate_name.map(str::to_owned),
            external_item_path_text: path.to_owned(),
        }),
    };
    let unresolved = HldEdgeResolutionValue::Unresolved {
        reason: HldUnresolvedReasonKind::ExternalSourceMissing,
    };
    let plain = "struct Buffer; fn helper() {} fn caller() { m::size_of(); }";
    let composed = record(
        "crate_rooted_path",
        json!("core"),
        "core::mem::size_of",
        "use_import_names_path",
    );

    // The qualified path, composed through a use item.
    let (resolution, metadata) = outcome(plain, "m::size_of", composed.clone());
    assert_eq!(
        resolution,
        named(
            HldExternalNameKind::CrateRootedPath,
            Some("core"),
            "core::mem::size_of"
        )
    );
    assert_eq!(metadata["dispatch_kind"], "external_written_path");
    assert_eq!(
        metadata["callee_path_lookup_evidence"]["callee_path_outcome_kind"],
        "external_name_proved"
    );
    assert!(metadata.get("unresolved_detail_kind").is_none());
    assert_eq!(
        metadata["external_target_proof_record"],
        json!({"external_name_kind":"crate_rooted_path",
            "external_site_proof_kind":"use_import_names_path",
            "external_import_site_key":import,
            "external_source_read_flag":false,
            "external_definition_path_claimed_flag":false})
    );

    // No stored proof: the row is as before.
    let (resolution, metadata) = outcome(plain, "m::size_of", Value::Null);
    assert_eq!(resolution, unresolved);
    assert_eq!(metadata["dispatch_kind"], "undetermined");
    assert!(metadata.get("external_target_proof_record").is_none());

    // A proof beside any other stored reason is not used.
    let (resolution, metadata) =
        outcome_stored(plain, "m::size_of", composed.clone(), "private_item");
    assert!(
        matches!(resolution, HldEdgeResolutionValue::Unresolved { .. }),
        "{resolution:?}"
    );
    assert!(metadata.get("external_target_proof_record").is_none());

    // A parameter named as the qualifier shadows it: no name, whatever is stored.
    let shadowed = "struct Buffer; fn helper() {} fn caller(m: u8) { m::size_of(); }";
    let (resolution, metadata) = outcome(shadowed, "m::size_of", composed.clone());
    assert!(
        matches!(resolution, HldEdgeResolutionValue::Unresolved { .. }),
        "{resolution:?}"
    );
    assert_eq!(
        metadata["unresolved_detail_kind"],
        "callee_prefix_lexically_shadowed"
    );
    assert!(metadata.get("external_target_proof_record").is_none());

    // A trait in read source declares `size_of`: a qualified path gets no name ...
    let local = "struct Buffer; fn helper() {} trait Shape { fn size_of(); } \
        fn caller() { m::size_of(); }";
    let (resolution, metadata) = outcome(local, "m::size_of", composed.clone());
    assert_eq!(resolution, unresolved);
    assert_eq!(
        metadata["unresolved_detail_kind"],
        "external_member_name_in_local_trait"
    );
    assert!(metadata.get("external_target_proof_record").is_none());

    // ... and a bare name does: a trait's member is never named by one segment.
    let bare = "struct Buffer; fn helper() {} trait Shape { fn size_of(); } \
        fn caller() { size_of(); }";
    let (resolution, _) = outcome(bare, "size_of", composed.clone());
    assert_eq!(
        resolution,
        named(
            HldExternalNameKind::CrateRootedPath,
            Some("core"),
            "core::mem::size_of"
        )
    );

    // An inherent member of that name is of a type of this crate, which the proved
    // qualifier is not: the path keeps its name.
    let inherent = "struct Buffer; impl Buffer { fn size_of() {} } fn helper() {} \
        fn caller() { m::size_of(); }";
    let (resolution, _) = outcome(inherent, "m::size_of", composed.clone());
    assert_eq!(
        resolution,
        named(
            HldExternalNameKind::CrateRootedPath,
            Some("core"),
            "core::mem::size_of"
        )
    );

    // A prelude name.
    let prelude = "struct Buffer; fn helper() {} fn caller() { drop(); }";
    let (resolution, metadata) = outcome(
        prelude,
        "drop",
        record("prelude_name", Value::Null, "drop", "implicit_prelude_name"),
    );
    assert_eq!(
        resolution,
        named(HldExternalNameKind::PreludeName, None, "drop")
    );
    assert_eq!(
        metadata["external_target_proof_record"]["external_site_proof_kind"],
        "implicit_prelude_name"
    );

    // A stored record that cannot be read is a typed error, not a guess.
    let (store, input) =
        make_callee_fixture_graph(plain, &[("m::size_of", CALLEE, "unresolved", missing)]);
    for row in store.rows.lock().unwrap().iter_mut() {
        let mut path: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
        if path["path_site_source_text"] == "m::size_of" {
            path["path_external_name_record"] = json!({"external_name_kind":"guess"});
            row.stage_row_payload_text = path.to_string();
        }
    }
    assert!(matches!(
        resolve_original_call_edges(
            &store,
            &make_call_fixture_scope(),
            &input,
            &RunCancelFlagValue::default()
        ),
        Err(OriginalCallResolutionError::InvalidEvidence {
            field: "external_name_kind"
        })
    ));
}

/// REQ-RUST-053.0 N3, N4, N5: a call `T::name(..)` through a builtin type that no stored
/// impl declares targets the written name. A trait in read source that declares the name
/// withholds it.
#[test]
fn test_builtin_member_call_target() {
    let outcome = |source: &str| {
        let (store, mut input) = make_callee_fixture_graph(
            source,
            &[
                ("str", "body_call_type_prefix", "resolved", "Builtin"),
                ("str::make", CALLEE, "unresolved", ""),
            ],
        );
        if source.contains("trait Shape") {
            let member = HldNodeInputRecord {
                node_owner_kind_value: Some(HldOwnerKindValue::Trait),
                node_qualified_path_text: "Shape::make".into(),
                ..input.input_node_records[0].clone()
            };
            input.input_node_records.push(member);
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
        (edge.clone(), metadata)
    };
    let (edge, metadata) = outcome("struct Buffer; fn helper() {} fn caller() { str::make(); }");
    let target = HldTargetIdentityValue::ExternalNamed(HldExternalNamedKey {
        external_name_kind_value: HldExternalNameKind::BuiltinMember,
        external_crate_name_value: None,
        external_item_path_text: "str::make".to_owned(),
    });
    assert_eq!(
        edge.edge_resolution_value,
        HldEdgeResolutionValue::Extracted {
            target: target.clone()
        }
    );
    assert_eq!(
        edge.edge_evidence_row_key.evidence_target_reference_value,
        Some(target)
    );
    assert_eq!(metadata["dispatch_kind"], "external_builtin_member");
    assert!(metadata["unresolved_detail_kind"].is_null(), "{metadata}");
    assert_eq!(
        metadata["external_target_proof_record"],
        json!({"external_name_kind":"builtin_member",
            "external_site_proof_kind":"builtin_type_member",
            "external_import_site_key":null,
            "external_source_read_flag":false,
            "external_definition_path_claimed_flag":false})
    );
    let lookup = &metadata["type_relative_member_evidence"];
    assert_eq!(
        lookup["member_lookup_outcome_kind"],
        "no_stored_impl_member"
    );
    assert_eq!(lookup["self_type_builtin_name"], "str");

    let (edge, metadata) = outcome(
        "struct Buffer; trait Shape { fn make(); } fn helper() {} fn caller() { str::make(); }",
    );
    assert_eq!(
        edge.edge_resolution_value,
        HldEdgeResolutionValue::Unresolved {
            reason: HldUnresolvedReasonKind::MissingTarget
        }
    );
    assert_eq!(metadata["dispatch_kind"], NONE);
    assert_eq!(
        metadata["unresolved_detail_kind"],
        "external_member_name_in_local_trait"
    );
    assert!(metadata["external_target_proof_record"].is_null());
    assert_eq!(
        metadata["type_relative_member_evidence"]["member_lookup_outcome_kind"],
        "no_stored_impl_member"
    );
}

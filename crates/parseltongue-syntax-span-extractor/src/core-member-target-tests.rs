#![allow(clippy::unwrap_used, clippy::expect_used)]
use crate::extract_member_declaration_rows;
use crate::original_call_resolution_records::original_call_resolution_tests::{
    make_call_fixture_graph, make_call_fixture_row, make_call_fixture_scope, make_call_fixture_unit,
};
use crate::original_call_resolution_records::resolve_original_call_edges;
use crate::syntax_namespace_record_codec::encode_compilation_unit_value;
use parseltongue_core_row_types::*;
use serde_json::{Value, json};

const CORE_FIXTURE_FILES: [(&str, &str); 3] = [
    (
        "core/src/mem/maybe_uninit.rs",
        "pub union MaybeUninit<T> { value: T }\n\
         impl<T> MaybeUninit<T> { pub fn write(&mut self, val: T) -> &mut T { loop {} } }\n",
    ),
    (
        "core/src/slice/mod.rs",
        "impl<T> [T] { pub fn len(&self) -> usize { 0 }\n\
         pub fn first(&self) -> usize { 0 }\n\
         pub unsafe fn get_unchecked<I>(&self, index: I) -> &I::Output { loop {} } }\n\
         pub trait ExactSizeIterator { fn len(&self) -> usize; }\n\
         pub trait SliceIndex<T> { fn get_unchecked(self, slice: *const T); }\n\
         pub trait Hidden { fn first(&self) -> usize; }\n\
         pub use self::Hidden as Shown;\n",
    ),
    (
        "core/src/prelude/v1.rs",
        "pub use crate::iter::ExactSizeIterator;\n",
    ),
];

/// The one call edge of `source`, resolved over a store that holds the fixture's `core`
/// declarations and a path lookup for every written `MaybeUninit<u8>`, `u8` and `usize`.
/// `no_std` is the unit's stored flag; `core_name` says whether `MaybeUninit` was looked
/// up as the `core` name or has no stored outcome.
fn read_core_fixture_edge(
    source: &str,
    no_std: bool,
    core_name: bool,
    stray: &str,
) -> (HldEdgeInputRecord, Value) {
    let (store, input) = make_call_fixture_graph(source);
    let unit = encode_compilation_unit_value(&make_call_fixture_unit());
    let path_row = |text: &str, start: usize, outcome: Value| {
        let mut row = json!({
            "site_compilation_scope_record":{"unit_semantic_identity_record":unit},
            "site_source_absolute_path":"/fixture/src/lib.rs",
            "site_source_range_start":start, "site_source_range_end":start + text.len(),
            "site_expansion_provenance_chain":[]
        });
        for (key, value) in outcome.as_object().unwrap() {
            row[key] = value.clone();
        }
        make_call_fixture_row("path-resolution-site-records", row)
    };
    let builtin = |name: &str| {
        json!({"path_resolution_state_kind":"resolved", "path_target_symbol_rows":[{
            "target_symbol_kind_value":"Builtin", "target_builtin_name_value":name,
            "target_namespace_kind_values":["type"]}]})
    };
    let mut extra = Vec::new();
    for (start, _) in source.match_indices("MaybeUninit<u8>") {
        if core_name {
            extra.push(path_row(
                "MaybeUninit<u8>",
                start,
                json!({
                    "path_resolution_state_kind":"unresolved",
                    "path_unresolved_reason_kind":"external_source_missing",
                    "path_target_symbol_rows":[],
                    "path_external_name_record":{"external_crate_name":"core",
                        "external_item_path_text":"core::mem::MaybeUninit",
                        "external_name_kind":"crate_rooted_path",
                        "external_site_proof_kind":"use_import_names_path",
                        "external_import_site_key":null}
                }),
            ));
        }
        extra.push(path_row("u8", start + "MaybeUninit<".len(), builtin("u8")));
    }
    for (start, _) in source.match_indices("usize") {
        extra.push(path_row("usize", start, builtin("usize")));
    }
    extra.push(make_call_fixture_row(
        "file-source-text-table",
        json!({"file_source_absolute_path":"/fixture/src/lib.rs",
            "file_content_digest_full":"fixture-source-digest", "file_source_byte_count":source.len()}),
    ));
    extra.push(StageTableRowRecord {
        stage_owned_table_name: "fixture-file-content-bytes".into(),
        stage_row_record_key: "fixture-source-digest".into(),
        stage_row_payload_text: source.to_owned(),
    });
    // A file of another target: stored, with no span of a mapped unit.
    extra.push(make_call_fixture_row(
        "file-source-text-table",
        json!({"file_source_absolute_path":"/fixture/benches/bench.rs",
            "file_content_digest_full":"fixture-stray-digest", "file_source_byte_count":stray.len()}),
    ));
    extra.push(StageTableRowRecord {
        stage_owned_table_name: "fixture-file-content-bytes".into(),
        stage_row_record_key: "fixture-stray-digest".into(),
        stage_row_payload_text: stray.to_owned(),
    });
    for (path, text) in CORE_FIXTURE_FILES {
        extra.extend(
            extract_member_declaration_rows(path, text)
                .into_iter()
                .map(|row| make_call_fixture_row("sysroot-member-declaration-records", row)),
        );
    }
    {
        let mut rows = store.rows.lock().unwrap();
        for row in rows
            .iter_mut()
            .filter(|row| row.stage_owned_table_name == "crate-dependency-graph-table")
        {
            let mut stored: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
            stored["unit_no_std_flag"] = json!(no_std);
            stored["unit_normal_dependency_rows"] = json!([]);
            row.stage_row_payload_text = stored.to_string();
        }
        rows.extend(extra);
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
}

fn make_core_member_target(path: &str) -> HldEdgeResolutionValue {
    HldEdgeResolutionValue::Extracted {
        target: HldTargetIdentityValue::ExternalNamed(HldExternalNamedKey {
            external_name_kind_value: HldExternalNameKind::CoreDeclaredMember,
            external_crate_name_value: Some("core".to_owned()),
            external_item_path_text: path.to_owned(),
        }),
    }
}

/// REQ-RUST-056.0 T1, T2, T4: a parameter with a written type whose names are proven to
/// be `core`'s gives the one declaration the probe finds; a prelude trait that declares
/// the name, a unit that is not `no_std` and a type name with no stored proof each
/// withhold it.
#[test]
fn test_parameter_receiver_core_member() {
    let element = "struct Buffer; fn f(buf: &mut [MaybeUninit<u8>], i: usize) { buf[i].write(1); }";
    let (edge, metadata) = read_core_fixture_edge(element, true, true, "");
    assert_eq!(
        make_core_member_target("MaybeUninit<T>::write"),
        edge.edge_resolution_value,
        "{metadata}"
    );
    assert_eq!("core_inherent_member", metadata["dispatch_kind"]);
    let lookup = &metadata["core_member_lookup_evidence"];
    assert_eq!("MaybeUninit<u8>", lookup["receiver_type_text"]);
    assert_eq!(
        "core/src/mem/maybe_uninit.rs",
        lookup["declaration_source_relative_path"]
    );
    assert_eq!(
        "step 0 by mutable reference",
        lookup["method_probe_step_text"]
    );
    let (start, end) = (
        lookup["member_name_start_offset"].as_u64().unwrap() as usize,
        lookup["member_name_end_offset"].as_u64().unwrap() as usize,
    );
    assert_eq!("write", &CORE_FIXTURE_FILES[0].1[start..end]);
    assert_eq!(
        false,
        metadata["external_target_proof_record"]["external_source_read_flag"]
    );

    let slice =
        "struct Buffer; fn f(buf: &[MaybeUninit<u8>], i: usize) { buf.get_unchecked(i..); }";
    let (edge, metadata) = read_core_fixture_edge(slice, true, true, "");
    assert_eq!(
        make_core_member_target("[T]::get_unchecked"),
        edge.edge_resolution_value,
        "{metadata}"
    );

    let withheld = |source: &str, no_std: bool, core_name: bool| {
        let (edge, metadata) = read_core_fixture_edge(source, no_std, core_name, "");
        assert!(
            matches!(
                edge.edge_resolution_value,
                HldEdgeResolutionValue::Unresolved { .. }
            ),
            "{metadata}"
        );
        assert_eq!("undetermined", metadata["dispatch_kind"]);
        metadata["unresolved_detail_kind"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    // An inherent method found at the receiver's own type comes before every trait.
    let direct = "struct Buffer; fn f(buf: &[MaybeUninit<u8>], i: usize) { buf.len(); }";
    let (edge, metadata) = read_core_fixture_edge(direct, true, true, "");
    assert_eq!(
        make_core_member_target("[T]::len"),
        edge.edge_resolution_value,
        "{metadata}"
    );
    // On an array the slice's method is a later step: a prelude trait may fit before it.
    let length = "struct Buffer; fn f(buf: [MaybeUninit<u8>; 4], i: usize) { buf.len(); }";
    assert_eq!("scope_trait_may_fit", withheld(length, true, true));
    // A range written as the index gives the slice.
    let ranged = "struct Buffer; fn f(buf: &[MaybeUninit<u8>], i: usize) { buf[i..].len(); }";
    let (edge, metadata) = read_core_fixture_edge(ranged, true, true, "");
    assert_eq!(
        make_core_member_target("[T]::len"),
        edge.edge_resolution_value,
        "{metadata}"
    );
    // A trait of `core` that is neither in the prelude nor written in the workspace is
    // out of scope; the name `core` renames it to brings it back.
    let first = "struct Buffer; fn f(buf: [MaybeUninit<u8>; 4], i: usize) { buf.first(); }";
    let (edge, metadata) = read_core_fixture_edge(first, true, true, "");
    assert_eq!(
        make_core_member_target("[T]::first"),
        edge.edge_resolution_value,
        "{metadata}"
    );
    let renamed = "struct Buffer; use core::slice::Shown; \
        fn f(buf: [MaybeUninit<u8>; 4], i: usize) { buf.first(); }";
    assert_eq!("scope_trait_may_fit", withheld(renamed, true, true));
    let glob = "struct Buffer; use core::slice::*; \
        fn f(buf: [MaybeUninit<u8>; 4], i: usize) { buf.first(); }";
    assert_eq!("scope_glob_import_unread", withheld(glob, true, true));
    // What a file with no span names is not in a mapped unit's scope, but its
    // `extern crate` could be.
    let (edge, metadata) = read_core_fixture_edge(element, true, true, "use std::fmt::Write;");
    assert_eq!(
        make_core_member_target("MaybeUninit<T>::write"),
        edge.edge_resolution_value,
        "{metadata}"
    );
    let (edge, metadata) = read_core_fixture_edge(element, true, true, "extern crate alloc;");
    assert!(
        matches!(
            edge.edge_resolution_value,
            HldEdgeResolutionValue::Unresolved { .. }
        ),
        "{metadata}"
    );
    assert_eq!(
        "unit_file_membership_unproven",
        metadata["unresolved_detail_kind"]
    );
    let named = "struct Buffer; extern crate alloc; \
        fn f(buf: &mut [MaybeUninit<u8>], i: usize) { buf[i].write(1); }";
    assert_eq!("alloc_or_std_named", withheld(named, true, true));
    assert_eq!("std_prelude_not_read", withheld(element, false, true));
    assert_eq!(
        "receiver_type_name_unproven",
        withheld(element, true, false)
    );
}

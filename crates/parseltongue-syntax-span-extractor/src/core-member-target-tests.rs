#![allow(clippy::unwrap_used, clippy::expect_used)]
use crate::extract_member_declaration_rows;
use crate::original_call_resolution_records::original_call_resolution_tests::{
    OriginalCallFixtureStore, make_call_fixture_graph, make_call_fixture_occurrence,
    make_call_fixture_row, make_call_fixture_scope, make_call_fixture_unit,
};
use crate::original_call_resolution_records::resolve_original_call_edges;
use crate::persist_impl_header_rows;
use crate::syntax_namespace_record_codec::encode_compilation_unit_value;
use crate::syntax_namespace_record_codec::encode_symbol_target_value;
use parseltongue_core_row_types::*;
use serde_json::{Value, json};

const CORE_FIXTURE_FILES: [(&str, &str); 9] = [
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
        "pub use crate::iter::ExactSizeIterator;\npub use crate::convert::TryInto;\n\
         pub use crate::result::Result::{self, Err, Ok};\n",
    ),
    (
        "core/src/convert/mod.rs",
        "pub trait TryInto<T>: Sized { type Error; fn try_into(self) -> Result<T, Self::Error>; }\n\
         impl<T, U> TryInto<U> for T { type Error = (); fn try_into(self) -> Result<U, ()> { loop {} } }\n\
         pub trait Lone { fn lone_call(self); }\n\
         pub trait Loose<T: ?Sized> { fn loose_call(self) -> Result<T, ()>; }\n",
    ),
    (
        "core/src/result.rs",
        "pub enum Result<T, E> { Ok(T), Err(E) }\n\
         impl<T, E> Result<T, E> { pub fn expect(self, msg: &str) -> T { loop {} }\n\
         pub fn unwrap(self) -> T { loop {} }\n\
         pub fn shadowed(self) {} }\n",
    ),
    // REQ-RUST-062.0 C8: `core` gives the name `Result` to an alias as well.
    (
        "core/src/fmt/mod.rs",
        "pub struct Error;\n\
         pub type Result = crate::result::Result<(), Error>;\n\
         impl Result { pub fn shadowed(self) {} }\n",
    ),
    (
        "core/src/lib.rs",
        "#[prelude_import]\nuse prelude::rust_2024::*;\n\
         pub mod prelude; pub mod result; pub mod fmt; pub mod convert;\n",
    ),
    // REQ-RUST-063.0: a macro of a `#[macro_use]` module writes the members of `u8`.
    (
        "core/src/num/mod.rs",
        "#[macro_use]\nmod uint_macros;\nimpl u8 { uint_impl! {} }\n",
    ),
    (
        "core/src/num/uint_macros.rs",
        "macro_rules! uint_impl { () => {\n pub const MAX: Self = !0;\n\
         pub const BITS: u32 = 8;\n\
         pub const fn ilog10(self) -> u32 { 0 }\n\
         $(#[$m])* pub const fn gated(self) -> u32 { 0 }\n}; }\n",
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
    read_core_fixture_rows(source, no_std, core_name, stray, false)
}

/// `read_core_fixture_edge`; with `impls`, every trait name, every `u8` and every
/// `Buffer<..>` also has a stored path lookup, and the impl headers are stored again
/// over those lookups.
fn read_core_fixture_rows(
    source: &str,
    no_std: bool,
    core_name: bool,
    stray: &str,
    impls: bool,
) -> (HldEdgeInputRecord, Value) {
    let edges = list_core_fixture_edges(source, no_std, core_name, stray, impls);
    let [edge] = edges.as_slice() else {
        panic!("one call expected, found {}", edges.len());
    };
    edge.clone()
}

/// `read_core_fixture_rows` for a source with any number of calls: every call edge with
/// its dispatch metadata, in stored order.
fn list_core_fixture_edges(
    source: &str,
    no_std: bool,
    core_name: bool,
    stray: &str,
    impls: bool,
) -> Vec<(HldEdgeInputRecord, Value)> {
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
    if impls {
        let read = |table: &str| {
            store
                .rows
                .lock()
                .unwrap()
                .iter()
                .filter(|row| row.stage_owned_table_name == table)
                .map(|row| serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap())
                .collect::<Vec<_>>()
        };
        for found in read("span-reference-text-table")
            .iter()
            .filter(|span| span["span_syntax_kind_value"] == "TRAIT")
        {
            let name = found["span_declared_name_value"].as_str().unwrap();
            let (start, end) = (
                found["span_byte_start_offset"].as_u64().unwrap(),
                found["span_byte_end_offset"].as_u64().unwrap(),
            );
            let occurrence = make_call_fixture_occurrence(
                start,
                end,
                found["span_source_code_text"].as_str().unwrap(),
            );
            let metadata = json!({"target_symbol_kind_value":"Trait",
                "target_source_file_path":"/fixture/src/lib.rs",
                "target_source_range_start":start, "target_source_range_end":end,
                "target_compilation_scope_record":{"unit_semantic_identity_record":unit}});
            extra.push(make_call_fixture_row(
                "syntax-namespace-input-items",
                json!({"binding_target_identity_key":encode_symbol_target_value(
                    &NameSymbolTargetKey::Definition(occurrence)),
                    "binding_target_metadata_value":metadata}),
            ));
            for (at, _) in source.match_indices(name) {
                extra.push(path_row(
                    name,
                    at,
                    json!({"path_resolution_state_kind":"resolved",
                        "path_target_symbol_rows":[metadata]}),
                ));
            }
        }
        for (at, _) in source.match_indices("u8") {
            if !source[..at].ends_with("MaybeUninit<") {
                extra.push(path_row("u8", at, builtin("u8")));
            }
        }
        let buffer = read("path-resolution-site-records")
            .into_iter()
            .find(|row| row["path_target_symbol_rows"][0]["target_symbol_kind_value"] == "Struct")
            .unwrap();
        for (at, _) in source.match_indices("Buffer<") {
            let end = at + source[at..].find('>').unwrap() + 1;
            extra.push(path_row(
                &source[at..end],
                at,
                json!({"path_resolution_state_kind":"resolved",
                    "path_target_symbol_rows":buffer["path_target_symbol_rows"]}),
            ));
        }
    }
    // REQ-RUST-059.0: each `static` is a namespace item, and each place its name is
    // written has a stored value-path lookup that names it.
    for found in read_fixture_table_values(&store, "span-reference-text-table")
        .iter()
        .filter(|span| span["span_syntax_kind_value"] == "STATIC")
    {
        let name = found["span_declared_name_value"].as_str().unwrap();
        let (start, end) = (
            found["span_byte_start_offset"].as_u64().unwrap(),
            found["span_byte_end_offset"].as_u64().unwrap(),
        );
        let occurrence = make_call_fixture_occurrence(
            start,
            end,
            found["span_source_code_text"].as_str().unwrap(),
        );
        let metadata = json!({"target_symbol_kind_value":"Static",
            "target_source_file_path":"/fixture/src/lib.rs",
            "target_source_range_start":start, "target_source_range_end":end,
            "target_compilation_scope_record":{"unit_semantic_identity_record":unit}});
        extra.push(make_call_fixture_row(
            "syntax-namespace-input-items",
            json!({"binding_target_identity_key":encode_symbol_target_value(
                &NameSymbolTargetKey::Definition(occurrence)),
                "binding_target_metadata_value":metadata}),
        ));
        extra.extend(source.match_indices(name).map(|(at, _)| {
            path_row(
                name,
                at,
                json!({"path_resolution_state_kind":"resolved",
                    "site_path_context_kind":"body_value_path",
                    "path_target_symbol_rows":[metadata]}),
            )
        }));
    }
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
        if impls {
            rows.retain(|row| row.stage_owned_table_name != "impl-header-index-table");
        }
    }
    if impls {
        persist_impl_header_rows(
            &store,
            &make_call_fixture_scope(),
            std::path::Path::new("/fixture"),
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    }
    let edges = resolve_original_call_edges(
        &store,
        &make_call_fixture_scope(),
        &input,
        &RunCancelFlagValue::default(),
    )
    .unwrap();
    edges
        .into_iter()
        .map(|edge| {
            let metadata: Value =
                serde_json::from_str(edge.edge_dispatch_metadata_text.as_deref().unwrap()).unwrap();
            (edge, metadata)
        })
        .collect()
}

fn read_fixture_table_values(store: &OriginalCallFixtureStore, table: &str) -> Vec<Value> {
    store
        .rows
        .lock()
        .unwrap()
        .iter()
        .filter(|row| row.stage_owned_table_name == table)
        .map(|row| serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap())
        .collect()
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
    assert_eq!(
        json!(["core_macro_expansions_import_no_trait"]),
        metadata["core_member_lookup_evidence"]["lookup_assumption_values"]
    );
    // REQ-RUST-060.0: on an array the slice's method is a later step, and a prelude trait
    // of `core` declares the name. The target is named and the row says what it assumes.
    let length = "struct Buffer; fn f(buf: [MaybeUninit<u8>; 4], i: usize) { buf.len(); }";
    let (edge, metadata) = read_core_fixture_edge(length, true, true, "");
    assert_eq!(
        make_core_member_target("[T]::len"),
        edge.edge_resolution_value,
        "{metadata}"
    );
    let lookup = &metadata["core_member_lookup_evidence"];
    assert_eq!(
        json!([
            "core_macro_expansions_import_no_trait",
            "prelude_trait_impl_unrefuted"
        ]),
        lookup["lookup_assumption_values"]
    );
    assert_eq!(
        json!(["ExactSizeIterator"]),
        lookup["prelude_trait_assumed_names"]
    );
    // The trait's name written in the workspace: an impl of it may be in the workspace.
    let imported = "struct Buffer; use core::iter::ExactSizeIterator; \
        fn f(buf: [MaybeUninit<u8>; 4], i: usize) { buf.len(); }";
    assert_eq!("scope_trait_may_fit", withheld(imported, true, true));
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

/// REQ-RUST-057.0 A2 to A9: a parameter typed `Self::NAME` takes the type the one stored
/// impl of the self type writes for `NAME`, when that impl is the function's own or of a
/// supertrait of its trait; every other case keeps the call without a target.
#[test]
fn test_alias_parameter_core_member() {
    let named = |source: &str, written: &str| {
        let (edge, metadata) = read_core_fixture_rows(source, true, true, "", true);
        assert_eq!(
            make_core_member_target("MaybeUninit<T>::write"),
            edge.edge_resolution_value,
            "{metadata}"
        );
        let lookup = &metadata["core_member_lookup_evidence"];
        assert_eq!(
            "parameter_associated_type_alias",
            lookup["receiver_type_proof_kind"]
        );
        assert_eq!("[MaybeUninit<u8>; 3]", lookup["associated_type_alias_text"]);
        assert_eq!(written, lookup["receiver_written_type_text"]);
        assert_eq!("MaybeUninit<u8>", lookup["receiver_type_text"]);
        assert!(
            lookup["lookup_assumption_values"]
                .as_array()
                .unwrap()
                .contains(&json!("source_compiles_one_associated_type"))
        );
        for key in [
            "associated_type_member_occurrence",
            "associated_type_impl_occurrence",
        ] {
            assert!(lookup[key].is_object(), "{key}: {lookup}");
        }
    };
    let withheld = |source: &str| {
        let (edge, metadata) = read_core_fixture_rows(source, true, true, "", true);
        assert!(
            matches!(
                edge.edge_resolution_value,
                HldEdgeResolutionValue::Unresolved { .. }
            ),
            "{metadata}"
        );
        metadata["unresolved_detail_kind"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    // A2: the function's own impl writes the type.
    named(
        "struct Buffer; trait Sealed { type Store; fn put(self, buf: &mut Self::Store); } \
         impl Sealed for u8 { type Store = [MaybeUninit<u8>; 3]; \
         fn put(self, buf: &mut Self::Store) { buf[0].write(1); } }",
        "&mut Self::Store",
    );
    // A2: white space between the tokens, as a macro expansion writes them, changes nothing.
    named(
        "struct Buffer; trait Sealed { type Store; fn put(self, buf: &mut Self ::Store); } \
         impl Sealed for u8 { type Store = [MaybeUninit<u8>; 3]; \
         fn put(self, buf: & mut Self :: Store) { buf[0].write(1); } }",
        "& mut Self :: Store",
    );
    // A5: an impl of a supertrait of the function's impl's trait writes it.
    named(
        "struct Buffer; trait Sealed { type Store; } trait Middle: Sealed {} \
         trait Unsigned: Middle { fn fmt(self, buf: &mut Self::Store); } \
         impl Sealed for u8 { type Store = [MaybeUninit<u8>; 3]; } \
         impl Unsigned for u8 { fn fmt(self, buf: &mut Self::Store) { buf[0].write(1); } }",
        "&mut Self::Store",
    );
    // A5: no stored supertrait bound leads from the function's trait to the type's trait.
    assert_eq!(
        "associated_type_trait_unproven",
        withheld(
            "struct Buffer; trait Sealed { type Store; } \
             trait Unsigned { fn fmt(self, buf: &mut Self::Store); } \
             impl Sealed for u8 { type Store = [MaybeUninit<u8>; 3]; } \
             impl Unsigned for u8 { fn fmt(self, buf: &mut Self::Store) { buf[0].write(1); } }"
        )
    );
    // A4: two stored impls of the self type write a type of that name.
    assert_eq!(
        "associated_type_impl_unproven",
        withheld(
            "struct Buffer; trait Sealed { type Store; fn put(self, buf: &mut Self::Store); } \
             trait Other { type Store; } impl Other for u8 { type Store = [u8; 2]; } \
             impl Sealed for u8 { type Store = [MaybeUninit<u8>; 3]; \
             fn put(self, buf: &mut Self::Store) { buf[0].write(1); } }"
        )
    );
    // A6: the impl that writes the type has another written self type.
    assert_eq!(
        "associated_type_impl_unproven",
        withheld(
            "struct Buffer; trait Sealed { type Store; } \
             trait Unsigned: Sealed { fn fmt(self, buf: &mut Self::Store); } \
             impl Sealed for Buffer<u8> { type Store = [MaybeUninit<u8>; 3]; } \
             impl Unsigned for Buffer<i8> { fn fmt(self, buf: &mut Self::Store) { buf[0].write(1); } }"
        )
    );
    // A7: the written type is a path through another impl.
    assert_eq!(
        "receiver_type_name_unproven",
        withheld(
            "struct Buffer; trait Sealed { type Store; fn put(self, buf: &mut Self::Store); } \
             impl Sealed for u8 { type Store = <u8 as Sealed>::Store; \
             fn put(self, buf: &mut Self::Store) { buf[0].write(1); } }"
        )
    );
}

/// REQ-RUST-059.0 S1 to S5: a receiver that is a `static` of the workspace, then fields of
/// workspace structs, has the type the last field declares. A stored field with an
/// attribute, a type that is not one plain struct name, and a name with no stored proof
/// each withhold it.
#[test]
fn test_static_field_receiver_member() {
    let named = |source: &str, label: &str, kind: &str| {
        let (edge, metadata) = read_core_fixture_edge(source, true, true, "");
        assert_eq!(
            make_core_member_target(label),
            edge.edge_resolution_value,
            "{metadata}"
        );
        let lookup = &metadata["core_member_lookup_evidence"];
        assert_eq!(kind, lookup["receiver_type_proof_kind"], "{metadata}");
        lookup.clone()
    };
    let detail = |source: &str| {
        let (edge, metadata) = read_core_fixture_edge(source, true, true, "");
        assert!(
            matches!(
                edge.edge_resolution_value,
                HldEdgeResolutionValue::Unresolved { .. }
            ),
            "{metadata}"
        );
        metadata["unresolved_detail_kind"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    // A tuple field, by its position.
    let tuple = "struct Buffer(usize, [MaybeUninit<u8>; 4]); \
        static PAIRS: Buffer = Buffer(0, [MaybeUninit::uninit(); 4]); \
        fn f(i: usize) { PAIRS.1.get_unchecked(i); }";
    let lookup = named(tuple, "[T]::get_unchecked", "static_declared_type_field");
    assert_eq!("Buffer", lookup["receiver_written_type_text"]);
    assert_eq!("[MaybeUninit<u8>; _]", lookup["receiver_type_text"]);
    assert_eq!(
        json!(["1", "[MaybeUninit<u8>; 4]"]),
        json!([
            lookup["receiver_field_step_rows"][0]["field_declared_name_value"],
            lookup["receiver_field_step_rows"][0]["field_type_source_text"]
        ])
    );
    // A named field, behind a visibility and a reference to the struct.
    let record = "struct Buffer { len: usize, pub(crate) bytes: [MaybeUninit<u8>; 4] } \
        static PAIRS: &Buffer = &Buffer { len: 0, bytes: [MaybeUninit::uninit(); 4] }; \
        fn f(i: usize) { PAIRS.bytes.get_unchecked(i); }";
    named(record, "[T]::get_unchecked", "static_declared_type_field");
    // No field step: the type the `static` declares.
    let plain = "struct Buffer; static PAIRS: [MaybeUninit<u8>; 4] = [MaybeUninit::uninit(); 4]; \
        fn f(i: usize) { PAIRS.get_unchecked(i); }";
    named(plain, "[T]::get_unchecked", "static_declared_type");
    // An index after the field.
    let indexed = "struct Buffer([MaybeUninit<u8>; 4]); \
        static mut PAIRS: Buffer = Buffer([MaybeUninit::uninit(); 4]); \
        fn f(i: usize) { unsafe { PAIRS.0[i].write(1); } }";
    named(
        indexed,
        "MaybeUninit<T>::write",
        "static_declared_type_field",
    );
    // A field that a `cfg` removes is not stored, so it takes no position.
    let removed = "struct Buffer(#[cfg(any())] usize, [MaybeUninit<u8>; 4]); \
        static PAIRS: Buffer = Buffer([MaybeUninit::uninit(); 4]); \
        fn f(i: usize) { PAIRS.0.get_unchecked(i); }";
    named(removed, "[T]::get_unchecked", "static_declared_type_field");
    // A stored field with an attribute is not read: the attribute may remove it.
    let gated = "struct Buffer(#[cfg(all())] usize, [MaybeUninit<u8>; 4]); \
        static PAIRS: Buffer = Buffer(0, [MaybeUninit::uninit(); 4]); \
        fn f(i: usize) { PAIRS.1.get_unchecked(i); }";
    assert_eq!("receiver_field_type_unproven", detail(gated));
    // A field of a type that is not one plain struct name.
    let nested = "struct Buffer([MaybeUninit<u8>; 4]); \
        static PAIRS: (Buffer, usize) = (Buffer([MaybeUninit::uninit(); 4]), 0); \
        fn f(i: usize) { PAIRS.0.0.get_unchecked(i); }";
    assert_eq!("receiver_field_type_unproven", detail(nested));
    // A field type with a name that has no stored proof.
    let unproven = "struct Buffer([Other; 4]); struct Other; \
        static PAIRS: Buffer = Buffer([Other; 4]); \
        fn f(i: usize) { PAIRS.0.get_unchecked(i); }";
    assert_eq!("receiver_type_name_unproven", detail(unproven));
}

/// REQ-RUST-061.0 M1 to M4: a method name that exactly one trait of `core` declares, and
/// nothing else in `core` or the workspace, names that trait's declaration whatever the
/// receiver is.
#[test]
fn test_sole_trait_method_name() {
    let literal = "struct Buffer; fn f(i: usize) { 999.try_into(); }";
    let (edge, metadata) = read_core_fixture_edge(literal, true, true, "");
    assert_eq!(
        make_core_member_target("TryInto::try_into"),
        edge.edge_resolution_value,
        "{metadata}"
    );
    assert_eq!("core_trait_declared_member", metadata["dispatch_kind"]);
    let lookup = &metadata["core_member_lookup_evidence"];
    assert_eq!(
        json!([
            "method_name_sole_declaration",
            "trait_declaration",
            "TryInto"
        ]),
        json!([
            lookup["receiver_type_proof_kind"],
            lookup["target_owner_kind_value"],
            lookup["owner_trait_name_value"]
        ])
    );
    assert_eq!(
        json!(["source_compiles_method_resolves"]),
        lookup["lookup_assumption_values"]
    );
    assert_eq!(
        "core/src/convert/mod.rs",
        lookup["declaration_source_relative_path"]
    );
    let (start, end) = (
        lookup["member_name_start_offset"].as_u64().unwrap() as usize,
        lookup["member_name_end_offset"].as_u64().unwrap() as usize,
    );
    let declared = CORE_FIXTURE_FILES[3].1;
    assert_eq!("try_into", &declared[start..end]);
    assert_eq!(declared.find("try_into"), Some(start));

    let unnamed = |source: &str, no_std: bool| {
        let (edge, metadata) = read_core_fixture_edge(source, no_std, true, "");
        assert!(
            matches!(
                edge.edge_resolution_value,
                HldEdgeResolutionValue::Unresolved { .. }
            ),
            "{metadata}"
        );
    };
    // A function of the workspace has the name.
    unnamed(
        "struct Buffer; fn try_into(i: usize) {} fn f(i: usize) { 999.try_into(); }",
        true,
    );
    // An inherent impl of `core` declares the name too.
    unnamed("struct Buffer; fn f(i: usize) { 999.len(); }", true);
    // The one trait that declares the name is not in scope.
    unnamed("struct Buffer; fn f(i: usize) { 999.lone_call(); }", true);
    // The std prelude is not read.
    unnamed(literal, false);
    // Its name written in the workspace brings it into scope.
    let imported = "struct Buffer; use core::convert::Lone; fn f(i: usize) { 999.lone_call(); }";
    let (edge, metadata) = read_core_fixture_edge(imported, true, true, "");
    assert_eq!(
        make_core_member_target("Lone::lone_call"),
        edge.edge_resolution_value,
        "{metadata}"
    );
}

/// REQ-RUST-059.0 S7: `self` then a field has the type the field of the impl's self type
/// declares. A `self` with a written type, and a trait's own method, give no target.
#[test]
fn test_self_field_receiver_member() {
    let method = |receiver: &str| {
        format!(
            "struct Buffer {{ bytes: [MaybeUninit<u8>; 4] }} \
             impl Buffer {{ fn f({receiver}, i: usize) {{ self.bytes.get_unchecked(i); }} }}"
        )
    };
    for (receiver, written) in [
        ("&mut self", "&mut Buffer"),
        ("&self", "&Buffer"),
        ("self", "Buffer"),
    ] {
        let (edge, metadata) = read_core_fixture_rows(&method(receiver), true, true, "", true);
        assert_eq!(
            make_core_member_target("[T]::get_unchecked"),
            edge.edge_resolution_value,
            "{metadata}"
        );
        let lookup = &metadata["core_member_lookup_evidence"];
        assert_eq!(
            json!(["self_declared_type_field", written, receiver]),
            json!([
                lookup["receiver_type_proof_kind"],
                lookup["receiver_written_type_text"],
                lookup["receiver_self_parameter_text"]
            ]),
            "{metadata}"
        );
        assert_eq!(
            "bytes",
            lookup["receiver_field_step_rows"][0]["field_declared_name_value"]
        );
    }
    let unnamed = |source: &str| {
        let (edge, metadata) = read_core_fixture_rows(source, true, true, "", true);
        assert!(
            matches!(
                edge.edge_resolution_value,
                HldEdgeResolutionValue::Unresolved { .. }
            ),
            "{metadata}"
        );
    };
    // `self` with a written type may be a smart pointer: its field is not read.
    unnamed(&method("self: &Self"));
    // A method a trait declares has no one self type.
    unnamed(
        "struct Buffer { bytes: [MaybeUninit<u8>; 4] } \
         trait Reader { fn f(&self, i: usize) { self.bytes.get_unchecked(i); } }",
    );
}

/// REQ-RUST-062.0 C1, C2, C6: a receiver that is a method call named by a declaration of
/// `core` has the type that declaration returns; the lookup record cites the inner call.
#[test]
fn test_call_result_receiver_member() {
    // The edge of the call of `method` in `source`, which may hold other calls.
    let find = |source: &str, method: &str| {
        list_core_fixture_edges(source, true, true, "", false)
            .into_iter()
            .find(|(_, metadata)| metadata["callee_source_text"] == method)
            .unwrap()
    };
    let (edge, metadata) = find(
        "struct Buffer; fn f(i: usize) { 999.try_into().expect(\"x\"); }",
        "expect",
    );
    assert_eq!(
        make_core_member_target("Result<T, E>::expect"),
        edge.edge_resolution_value,
        "{metadata}"
    );
    assert_eq!("core_inherent_member", metadata["dispatch_kind"]);
    let lookup = &metadata["core_member_lookup_evidence"];
    let inner = &lookup["receiver_inner_call_lookup"];
    assert_eq!(
        json!([
            "call_result_declared_type",
            "TryInto::try_into",
            "Result<T, Self::Error>",
            "Result<_, _>",
            "step 0 by value",
            "core/src/result.rs",
            ["source_compiles_method_resolves"]
        ]),
        json!([
            lookup["receiver_type_proof_kind"],
            lookup["receiver_inner_call_target_label"],
            lookup["receiver_return_type_text"],
            lookup["receiver_type_text"],
            lookup["method_probe_step_text"],
            lookup["declaration_source_relative_path"],
            lookup["lookup_assumption_values"]
        ])
    );
    // The inner call's own lookup record: where `try_into` is declared.
    let declared = CORE_FIXTURE_FILES[3].1;
    assert_eq!(
        json!(["core/src/convert/mod.rs", declared.find("try_into")]),
        json!([
            inner["declaration_source_relative_path"],
            inner["member_name_start_offset"]
        ])
    );
    let (start, end) = (
        lookup["member_name_start_offset"].as_u64().unwrap() as usize,
        lookup["member_name_end_offset"].as_u64().unwrap() as usize,
    );
    assert_eq!("expect", &CORE_FIXTURE_FILES[4].1[start..end]);
    // C13: which `Result` the return type of `try_into` names, and what proves it.
    let leaf = CORE_FIXTURE_FILES[2].1.find("self").unwrap();
    assert_eq!(
        json!([{
            "type_name_value":"Result",
            "name_written_source_path":"core/src/convert/mod.rs",
            "type_name_proof_kind":"prelude_import",
            "type_declaration_site_value":{
                "declaration_source_relative_path":"core/src/result.rs",
                "member_name_start_offset":9,
                "member_name_end_offset":15
            },
            "prelude_import_site_value":{
                "declaration_source_relative_path":"core/src/lib.rs",
                "member_name_start_offset":42,
                "member_name_end_offset":43
            },
            "prelude_leaf_site_value":{
                "declaration_source_relative_path":"core/src/prelude/v1.rs",
                "member_name_start_offset":leaf,
                "member_name_end_offset":leaf + 4
            },
            "prelude_leaf_path_text":"crate::result::Result"
        }]),
        lookup["receiver_type_name_proofs"]
    );

    // Through parentheses.
    let (edge, metadata) = find(
        "struct Buffer; fn f(i: usize) { (999.try_into()).unwrap(); }",
        "unwrap",
    );
    assert_eq!(
        make_core_member_target("Result<T, E>::unwrap"),
        edge.edge_resolution_value,
        "{metadata}"
    );

    let unnamed = |source: &str, method: &str, detail: &str| {
        let (edge, metadata) = find(source, method);
        assert!(
            matches!(
                edge.edge_resolution_value,
                HldEdgeResolutionValue::Unresolved { .. }
            ),
            "{metadata}"
        );
        assert_eq!(detail, metadata["unresolved_detail_kind"], "{metadata}");
    };
    // The inner call is not named: its trait is not in scope.
    unnamed(
        "struct Buffer; fn f(i: usize) { 999.lone_call().expect(\"x\"); }",
        "expect",
        "receiver_type_not_determined",
    );
    // The inner call is named, and its declaration writes no return type.
    unnamed(
        "struct Buffer; use core::convert::Lone; fn f(i: usize) { 999.lone_call().expect(\"x\"); }",
        "expect",
        "receiver_type_not_determined",
    );
    // `?Sized` is written for the trait's parameter: it is not proven `Sized`.
    unnamed(
        "struct Buffer; use core::convert::Loose; fn f(i: usize) { 999.loose_call().expect(\"x\"); }",
        "expect",
        "inherent_member_unproven",
    );
    // C12: an impl in the file of the alias declares `shadowed` on its own `Result`.
    unnamed(
        "struct Buffer; fn f(i: usize) { 999.try_into().shadowed(); }",
        "shadowed",
        "receiver_type_name_unproven",
    );
    // `expect` returns a bare `T`: a type the model does not read.
    unnamed(
        "struct Buffer; fn f(i: usize) { 999.try_into().expect(\"x\").first(); }",
        "first",
        "inherent_member_unproven",
    );
}

/// REQ-RUST-063.0 M6, M7: a method called on a const of a builtin type, both written
/// into the type's impl by a macro of `core`.
#[test]
fn test_builtin_constant_receiver_member() {
    let read = |source: &str| {
        read_core_fixture_rows(&format!("struct Buffer; {source}"), true, true, "", true)
    };
    let (edge, metadata) = read("fn f() -> u32 { u8::MAX.ilog10() }");
    assert_eq!(
        make_core_member_target("u8::ilog10"),
        edge.edge_resolution_value,
        "{metadata}"
    );
    assert_eq!("core_inherent_member", metadata["dispatch_kind"]);
    let lookup = &metadata["core_member_lookup_evidence"];
    let macros = CORE_FIXTURE_FILES[8].1;
    let call = CORE_FIXTURE_FILES[7].1.find("uint_impl").unwrap();
    assert_eq!(
        json!([
            "builtin_constant_declared_type",
            "u8",
            "u8",
            [],
            "core/src/num/uint_macros.rs",
            macros.find("ilog10").unwrap(),
            "Self",
            macros.find("MAX").unwrap(),
            ["core/src/num/mod.rs", call, "macro_use_module", 13],
            "inherent_item_declared_once"
        ]),
        json!([
            lookup["receiver_type_proof_kind"],
            lookup["receiver_type_text"],
            lookup["receiver_prefix_member_evidence"]["self_type_builtin_name"],
            lookup["lookup_assumption_values"],
            lookup["declaration_source_relative_path"],
            lookup["member_name_start_offset"],
            lookup["receiver_constant_member_proof"]["member_written_type_text"],
            lookup["receiver_constant_member_proof"]["macro_body_item_site_value"]["member_name_start_offset"],
            [
                lookup["method_macro_member_proof"]["impl_macro_call_site_value"]["declaration_source_relative_path"],
                lookup["method_macro_member_proof"]["impl_macro_call_site_value"]["member_name_start_offset"],
                lookup["method_macro_member_proof"]["macro_scope_proof_value"]["macro_scope_proof_kind"],
                lookup["method_macro_member_proof"]["macro_scope_proof_value"]["macro_definition_site_value"]
                    ["member_name_start_offset"]
            ],
            lookup["method_macro_member_proof"]["inherent_lookup_rule_kind"]
        ]),
        "{metadata}"
    );
    // Withheld: a const whose type is not the type itself; a method the call may give
    // an attribute; a type with no macro call; a path with a field after it.
    let withheld = |source: &str| {
        let (edge, metadata) = read(source);
        assert!(
            matches!(
                edge.edge_resolution_value,
                HldEdgeResolutionValue::Unresolved { .. }
            ),
            "{metadata}"
        );
        metadata["unresolved_detail_kind"].clone()
    };
    assert_eq!(
        json!([
            "builtin_macro_member_unproven",
            "builtin_macro_member_unproven",
            "builtin_macro_member_unproven",
            "receiver_type_not_determined"
        ]),
        json!([
            withheld("fn f() -> u32 { u8::BITS.ilog10() }"),
            withheld("fn f() -> u32 { u8::MAX.gated() }"),
            withheld("fn f() -> u32 { usize::MAX.ilog10() }"),
            withheld("fn f() -> u32 { u8::MAX.0.ilog10() }")
        ])
    );
}

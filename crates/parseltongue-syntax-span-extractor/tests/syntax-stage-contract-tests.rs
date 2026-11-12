#![allow(clippy::unwrap_used)]

use parseltongue_syntax_span_extractor::*;
use serde_json::Value;
use std::collections::BTreeSet;

fn make_syntax_file_input(source: &[u8]) -> SyntaxFileInputData {
    SyntaxFileInputData {
        file_source_absolute_path: "/workspace/src/lib.rs".into(),
        file_source_content_bytes: source.to_vec(),
        unit_compilation_identity_key: "package/lib/target/cfg".into(),
        unit_rust_edition_value: "2024".into(),
        unit_target_cfg_values: BTreeSet::from(["unix".into(), "target_os=\"linux\"".into()]),
        file_module_directory_path: "/workspace/src".into(),
        unit_metadata_environment_values: Default::default(),
        file_parse_context_kind: "items".into(),
        file_expansion_provenance_chain: Vec::new(),
    }
}

#[test]
fn test_module_raw_identifier_paths() {
    let input = make_syntax_file_input(b"mod r#impl; mod r#type { mod r#match; }");
    let result = extract_file_syntax_rows(&input).unwrap();
    let candidates: Vec<_> = result
        .source_closure_file_requests
        .iter()
        .flat_map(|request| {
            request
                .source_request_path_candidates
                .iter()
                .map(String::as_str)
        })
        .collect();
    assert!(candidates.contains(&"/workspace/src/impl.rs"));
    assert!(candidates.contains(&"/workspace/src/type/match.rs"));
}

#[test]
fn test_sysroot_prelude_context_flags() {
    let normal = read_unit_prelude_configuration(&make_syntax_file_input(b"")).unwrap();
    assert!(
        !normal.unit_no_std_flag && !normal.unit_no_core_flag && normal.unit_implicit_prelude_flag
    );
    let source = b"#![cfg_attr(unix,no_std)] #![no_implicit_prelude] pub struct Local;";
    let no_std = read_unit_prelude_configuration(&make_syntax_file_input(source)).unwrap();
    assert!(
        no_std.unit_no_std_flag && !no_std.unit_no_core_flag && !no_std.unit_implicit_prelude_flag
    );
    let no_core = read_unit_prelude_configuration(&make_syntax_file_input(b"#![no_core]")).unwrap();
    assert!(no_core.unit_no_core_flag && no_core.unit_no_std_flag);
    assert!(read_unit_prelude_configuration(&make_syntax_file_input(&[0xff])).is_err());
}

#[test]
fn test_sysroot_dependency_demand_detection() {
    assert!(!check_sysroot_context_needed(
        "pub struct Empty; // std Vec\n const TEXT:&str=\"Option core\";"
    ));
    assert!(check_sysroot_context_needed(
        "pub fn take(_:Option<Vec<u8>>){}"
    ));
    assert!(check_sysroot_context_needed("use core::ops::Range;"));
    for source in [
        "pub trait Borrowed: AsRef<u8> {}",
        "pub trait Borrowed: AsMut<u8> {}",
        "pub fn dispose(value: u8) { drop(value); }",
        "pub fn measure() { size_of::<u8>(); size_of_val(&0); align_of::<u8>(); align_of_val(&0); }",
    ] {
        assert!(check_sysroot_context_needed(source), "{source}");
    }
}

#[test]
fn test_lang_item_recognition_rules() {
    for name in [
        "sized",
        "copy",
        "fn",
        "owned_box",
        "panic_impl",
        "Option",
        "RangeCopy",
    ] {
        assert!(check_recognized_lang_item(name), "{name}");
    }
    for name in ["", "not_a_lang_item", "fn_"] {
        assert!(!check_recognized_lang_item(name), "{name}");
    }
}

#[test]
fn test_sysroot_final_trait_methods() {
    assert!(
        extract_file_syntax_rows(&make_syntax_file_input(
            b"pub trait Share { fn share(&self) -> &Self { self } }"
        ))
        .is_ok()
    );
    let direct = ra_ap_syntax::SourceFile::parse(
        "pub trait Share { final fn share(&self) -> &Self { self } }",
        ra_ap_syntax::Edition::Edition2024,
    );
    assert!(
        direct.errors().is_empty(),
        "pinned parser: {:?}",
        direct.errors()
    );
    extract_file_syntax_rows(&make_syntax_file_input(
        b"pub trait Share { final fn share(&self) -> &Self { self } }",
    ))
    .unwrap();
}

#[test]
fn test_sysroot_const_closure_support() {
    let binding = ra_ap_syntax::SourceFile::parse(
        "fn f() { let _ = const move |a| a; }",
        ra_ap_syntax::Edition::Edition2024,
    );
    assert!(binding.errors().is_empty(), "{:?}", binding.errors());
    let control = ra_ap_syntax::SourceFile::parse(
        "fn f() { move |a| a }",
        ra_ap_syntax::Edition::Edition2024,
    );
    assert!(control.errors().is_empty(), "{:?}", control.errors());
    let source = ra_ap_syntax::SourceFile::parse(
        "fn f() { const move |a| a }",
        ra_ap_syntax::Edition::Edition2024,
    );
    assert!(
        source.errors().is_empty(),
        "pinned parser: {:?}",
        source.errors()
    );
}

#[test]
fn test_cfg_comments_keep_semantics() {
    let enabled = BTreeSet::from(["unix".into(), "feature=\"selected\"".into()]);
    for source in [
        "all(/* nested /* inner */ outer */ unix, // comment with (, \"\n feature = \"selected\")",
        "all(unix, feature = r#\"selected\"#)",
        "all(true, not(false), r#unix)",
    ] {
        assert!(
            evaluate_cfg_expression_truth(source, &enabled).unwrap(),
            "{source}"
        );
    }
    assert!(!evaluate_cfg_expression_truth("all(unix, /* comment */ windows)", &enabled).unwrap());
    assert!(evaluate_cfg_expression_truth("all(unix, /* unterminated", &enabled).is_err());
    let mut input = make_syntax_file_input(
        b"#[cfg_attr(/* ( \" */ unix, cfg(feature = r#\"selected\"#))] fn selected() {}",
    );
    input.unit_target_cfg_values = enabled;
    assert!(extract_file_syntax_rows(&input).is_ok());
}

#[test]
fn test_source_crlf_tiling_exact() {
    let source = b"#![doc = \"\r\nDocs\r\n\"]\r\nfn active() { let text = \"x\r\ny\"; }\r\n";
    let result = extract_file_syntax_rows(&make_syntax_file_input(source)).unwrap();
    let spans: Vec<Value> = result
        .stage_unit_output_rows
        .iter()
        .filter(|r| r.stage_owned_table_name == "span-reference-text-table")
        .map(|r| serde_json::from_str(&r.stage_row_payload_text).unwrap())
        .collect();
    for span in spans {
        let start = span["span_byte_start_offset"].as_u64().unwrap() as usize;
        let end = span["span_byte_end_offset"].as_u64().unwrap() as usize;
        assert_eq!(
            span["span_source_code_text"],
            std::str::from_utf8(&source[start..end]).unwrap()
        );
        assert_eq!(
            span["span_content_digest_full"],
            blake3::hash(&source[start..end]).to_hex().to_string()
        );
    }
    let ranges: Vec<Value> = result
        .stage_unit_output_rows
        .iter()
        .filter(|r| r.stage_owned_table_name == "span-byte-ownership-ranges")
        .map(|r| serde_json::from_str(&r.stage_row_payload_text).unwrap())
        .collect();
    assert_eq!(
        ranges
            .iter()
            .map(|r| r["span_byte_end_offset"].as_u64().unwrap()
                - r["span_byte_start_offset"].as_u64().unwrap())
            .sum::<u64>(),
        source.len() as u64
    );
}

#[test]
fn test_module_path_parent_relative() {
    let mut input=make_syntax_file_input(b"#[path=\"discouraged.rs\"] mod discouraged; mod normal; mod inner { #[path=\"nested.rs\"] mod nested; }");
    input.file_source_absolute_path = "/workspace/src/parser.rs".into();
    input.file_module_directory_path = "/workspace/src/parser".into();
    let result = extract_file_syntax_rows(&input).unwrap();
    let paths: Vec<_> = result
        .source_closure_file_requests
        .iter()
        .flat_map(|r| r.source_request_path_candidates.iter())
        .collect();
    for expected in [
        "/workspace/src/discouraged.rs",
        "/workspace/src/parser/normal.rs",
        "/workspace/src/parser/inner/nested.rs",
    ] {
        assert!(paths.iter().any(|p| p.as_str() == expected), "{paths:?}");
    }
}

#[test]
fn test_contextual_macro_names_parse() {
    let input = make_syntax_file_input(
        b"macro_rules! safe {() => {}} safe! {} macro_rules! default {() => {}} default! {}",
    );
    let result = extract_file_syntax_rows(&input).unwrap();
    let sites: Vec<Value> = result
        .stage_unit_output_rows
        .iter()
        .filter(|r| r.stage_owned_table_name == "body-site-reference-table")
        .map(|r| serde_json::from_str(&r.stage_row_payload_text).unwrap())
        .collect();
    assert!(
        sites
            .iter()
            .any(|r| r["site_syntax_source_text"] == "safe! {}")
    );
    assert!(
        sites
            .iter()
            .any(|r| r["site_syntax_source_text"] == "default! {}")
    );
}

#[test]
fn test_span_gap_tiling_identity() {
    for source in [
        b"".as_slice(),
        b"// leading\n pub struct A { pub x: u8 }\n fn f(x: A) -> u8 { g(x.x) }\n// end\n"
            .as_slice(),
    ] {
        let input = make_syntax_file_input(source);
        let result = extract_file_syntax_rows(&input).unwrap();
        let ranges: Vec<serde_json::Value> = result
            .stage_unit_output_rows
            .iter()
            .filter(|r| r.stage_owned_table_name == "span-byte-ownership-ranges")
            .map(|r| serde_json::from_str(&r.stage_row_payload_text).unwrap())
            .collect();
        let end = ranges.iter().fold(0, |end, row| {
            assert_eq!(row["span_byte_start_offset"].as_u64().unwrap(), end);
            assert_ne!(row["span_gap_classification_kind"], "other");
            row["span_byte_end_offset"].as_u64().unwrap()
        });
        assert_eq!(end, source.len() as u64);
        if !source.is_empty() {
            assert!(
                result
                    .stage_unit_output_rows
                    .iter()
                    .any(|r| r.stage_owned_table_name == "field-signature-detail-table")
            );
            assert!(
                result
                    .stage_unit_output_rows
                    .iter()
                    .any(|r| r.stage_owned_table_name == "body-site-reference-table")
            );
        }
    }
}

#[test]
fn test_parse_error_stops_run() {
    let error = extract_file_syntax_rows(&make_syntax_file_input(b"fn broken( {")).unwrap_err();
    assert!(error.to_string().contains("WorkspaceNotCompilingError"));
    assert!(error.to_string().contains("/workspace/src/lib.rs"));
}

#[test]
fn test_non_utf8_stops_run() {
    let error = extract_file_syntax_rows(&make_syntax_file_input(&[b' ', 255])).unwrap_err();
    assert!(error.to_string().contains("offset 1"));
}

#[test]
fn test_inactive_cfg_items_counted() {
    let input = make_syntax_file_input(
        b"#[cfg(test)] mod absent; #[cfg(not(unix))] fn off() {} fn on() {}",
    );
    let result = extract_file_syntax_rows(&input).unwrap();
    assert!(result.source_closure_file_requests.is_empty());
    let inactive: Vec<_> = result
        .stage_unit_output_rows
        .iter()
        .filter(|r| {
            r.stage_owned_table_name == "span-reference-text-table"
                && r.stage_row_payload_text.contains("cfg_inactive")
        })
        .collect();
    assert_eq!(inactive.len(), 2);
    assert!(
        evaluate_cfg_expression_truth(
            "all(unix, not(test), target_os=\"linux\")",
            &input.unit_target_cfg_values
        )
        .unwrap()
    );
    assert!(!evaluate_cfg_expression_truth("any()", &input.unit_target_cfg_values).unwrap());
    assert!(evaluate_cfg_expression_truth("all()", &input.unit_target_cfg_values).unwrap());
    assert!(
        evaluate_cfg_expression_truth("not(unix, test)", &input.unit_target_cfg_values).is_err()
    );
}

#[test]
fn test_out_dir_include_unresolved() {
    let input = make_syntax_file_input(b"include!(concat!(env!(\"OUT_DIR\"), \"/generated.rs\"));");
    let result = extract_file_syntax_rows(&input).unwrap();
    assert!(result.source_closure_file_requests.is_empty());
    assert!(
        result
            .stage_unit_output_rows
            .iter()
            .any(|r| r.stage_row_payload_text.contains("build_script"))
    );
}

#[test]
fn test_module_path_context_rules() {
    let input = make_syntax_file_input(
        b"mod child; mod nested { #[path=\"alternate.rs\"] mod leaf; } include!(\"extra.rs\");",
    );
    let result = extract_file_syntax_rows(&input).unwrap();
    let requests = &result.source_closure_file_requests;
    assert!(requests.iter().any(|r| r.source_request_path_candidates
        == ["/workspace/src/child.rs", "/workspace/src/child/mod.rs"]));
    assert!(
        requests
            .iter()
            .any(|r| r.source_request_path_candidates == ["/workspace/src/nested/alternate.rs"])
    );
    assert!(
        requests
            .iter()
            .any(|r| r.source_request_path_candidates == ["/workspace/src/extra.rs"])
    );
}

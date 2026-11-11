use std::collections::BTreeSet;
use parseltongue_syntax_span_extractor::*;

fn make_syntax_file_input(source: &[u8]) -> SyntaxFileInputData {
    SyntaxFileInputData {
        file_source_absolute_path: "/workspace/src/lib.rs".into(),
        file_source_content_bytes: source.to_vec(),
        unit_compilation_identity_key: "package/lib/target/cfg".into(),
        unit_rust_edition_value: "2024".into(),
        unit_target_cfg_values: BTreeSet::from(["unix".into(), "target_os=\"linux\"".into()]),
        file_module_directory_path: "/workspace/src".into(),
    }
}

#[test]
fn test_span_gap_tiling_identity() {
    for source in [b"".as_slice(), b"// leading\n pub struct A { pub x: u8 }\n fn f(x: A) -> u8 { g(x.x) }\n// end\n".as_slice()] {
        let input = make_syntax_file_input(source);
        let result = extract_file_syntax_rows(&input).unwrap();
        let ranges: Vec<serde_json::Value> = result.stage_unit_output_rows.iter()
            .filter(|r| r.stage_owned_table_name == "span-byte-ownership-ranges")
            .map(|r| serde_json::from_str(&r.stage_row_payload_text).unwrap()).collect();
        let end = ranges.iter().fold(0, |end, row| {
            assert_eq!(row["span_byte_start_offset"].as_u64().unwrap(), end);
            assert_ne!(row["span_gap_classification_kind"], "other");
            row["span_byte_end_offset"].as_u64().unwrap()
        });
        assert_eq!(end, source.len() as u64);
        if !source.is_empty() {
            assert!(result.stage_unit_output_rows.iter().any(|r| r.stage_owned_table_name == "field-signature-detail-table"));
            assert!(result.stage_unit_output_rows.iter().any(|r| r.stage_owned_table_name == "body-site-reference-table"));
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
    let input = make_syntax_file_input(b"#[cfg(test)] mod absent; #[cfg(not(unix))] fn off() {} fn on() {}");
    let result = extract_file_syntax_rows(&input).unwrap();
    assert!(result.source_closure_file_requests.is_empty());
    let inactive: Vec<_> = result.stage_unit_output_rows.iter().filter(|r|
        r.stage_owned_table_name == "span-reference-text-table" && r.stage_row_payload_text.contains("cfg_inactive")).collect();
    assert_eq!(inactive.len(), 2);
    assert!(evaluate_cfg_expression_truth("all(unix, not(test), target_os=\"linux\")", &input.unit_target_cfg_values).unwrap());
    assert!(!evaluate_cfg_expression_truth("any()", &input.unit_target_cfg_values).unwrap());
    assert!(evaluate_cfg_expression_truth("all()", &input.unit_target_cfg_values).unwrap());
    assert!(evaluate_cfg_expression_truth("not(unix, test)", &input.unit_target_cfg_values).is_err());
}

#[test]
fn test_out_dir_include_unresolved() {
    let input = make_syntax_file_input(b"include!(concat!(env!(\"OUT_DIR\"), \"/generated.rs\"));");
    let result = extract_file_syntax_rows(&input).unwrap();
    assert!(result.source_closure_file_requests.is_empty());
    assert!(result.stage_unit_output_rows.iter().any(|r| r.stage_row_payload_text.contains("build_script")));
}

#[test]
fn test_module_path_context_rules() {
    let input = make_syntax_file_input(b"mod child; mod nested { #[path=\"alternate.rs\"] mod leaf; } include!(\"extra.rs\");");
    let result = extract_file_syntax_rows(&input).unwrap();
    let requests = &result.source_closure_file_requests;
    assert!(requests.iter().any(|r| r.source_request_path_candidates == ["/workspace/src/child.rs", "/workspace/src/child/mod.rs"]));
    assert!(requests.iter().any(|r| r.source_request_path_candidates == ["/workspace/src/nested/alternate.rs"]));
    assert!(requests.iter().any(|r| r.source_request_path_candidates == ["/workspace/src/extra.rs"]));
}

#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::original_call_resolution_records::original_call_resolution_tests::make_call_syntax_input;

const FIELD_FIXTURE_SOURCE: &str = "pub struct S { pub a: u8, #[cfg(feature = \"never\")] gone: u8, b: [u8; N] }\n\
    struct T(u8, #[cfg(feature = \"never\")] u16, pub u32);\n\
    enum E { A, B(u8), C { x: u8 } = 3 }\n";

/// The payloads of `table` that extraction writes for the fixture, in stored order.
fn read_field_fixture_rows(table: &str) -> Vec<Value> {
    extract_file_syntax_rows(&make_call_syntax_input(FIELD_FIXTURE_SOURCE))
        .unwrap()
        .stage_unit_output_rows
        .iter()
        .filter(|row| row.stage_owned_table_name == table)
        .map(|row| serde_json::from_str(&row.stage_row_payload_text).unwrap())
        .collect()
}

/// REQ-RUST-071.0 N1, N2: a field and a variant have a span row. A tuple field is named by
/// its position among the tuple fields the configuration keeps; a removed one has no name.
#[test]
fn test_field_span_rows_written() {
    let rows = read_field_fixture_rows("span-reference-text-table")
        .iter()
        .map(|span| {
            format!(
                "{} {} {}",
                span["span_syntax_kind_value"].as_str().unwrap(),
                span["span_declared_name_value"].as_str().unwrap_or("-"),
                span["span_configuration_active_flag"]
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        vec![
            "STRUCT S true",
            "RECORD_FIELD a true",
            "RECORD_FIELD gone false",
            "RECORD_FIELD b true",
            "STRUCT T true",
            "TUPLE_FIELD 0 true",
            "TUPLE_FIELD - false",
            "TUPLE_FIELD 1 true",
            "ENUM E true",
            "VARIANT A true",
            "VARIANT B true",
            "TUPLE_FIELD 0 true",
            "VARIANT C true",
            "RECORD_FIELD x true",
        ],
        rows
    );
}

/// REQ-RUST-071.0 E1: a field owns the row of its own type and each site written inside
/// it; its struct or variant does not.
#[test]
fn test_field_site_owner_rows() {
    let spans = read_field_fixture_rows("span-reference-text-table");
    let name_span_owner = |site: &Value| {
        spans
            .iter()
            .find(|span| span["span_semantic_occurrence_key"] == site["site_owner_occurrence_key"])
            .map(|span| {
                format!(
                    "{} {}",
                    span["span_syntax_kind_value"].as_str().unwrap(),
                    span["span_declared_name_value"].as_str().unwrap_or("-")
                )
            })
            .unwrap_or_else(|| "no span".into())
    };
    let owners = read_field_fixture_rows("field-signature-detail-table")
        .iter()
        .chain(&read_field_fixture_rows("body-site-reference-table"))
        .filter(|site| {
            matches!(
                site["site_syntax_kind_value"].as_str(),
                Some("RECORD_FIELD" | "TUPLE_FIELD" | "PATH_EXPR")
            )
        })
        .map(|site| {
            format!(
                "{} => {}",
                site["site_syntax_source_text"].as_str().unwrap(),
                name_span_owner(site)
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        vec![
            "pub a: u8 => RECORD_FIELD a",
            "b: [u8; N] => RECORD_FIELD b",
            "u8 => TUPLE_FIELD 0",
            "pub u32 => TUPLE_FIELD 1",
            "u8 => TUPLE_FIELD 0",
            "x: u8 => RECORD_FIELD x",
            "N => RECORD_FIELD b",
        ],
        owners
    );
}

/// REQ-RUST-071.0 N3: a field inside a function body is no node and binds no name
/// there, so it neither shadows a called path nor rebinds a receiver of its name.
#[test]
fn test_body_field_binds_nothing() {
    let source = "fn run(buf: Buffer) -> usize {\n    struct Local { buf: u8, Buffer: u8 }\n    \
        let _ = Buffer::new();\n    buf.len()\n}\n";
    let sites = extract_file_syntax_rows(&make_call_syntax_input(source))
        .unwrap()
        .stage_unit_output_rows
        .iter()
        .filter(|row| row.stage_owned_table_name == "body-site-reference-table")
        .map(|row| serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap())
        .collect::<Vec<_>>();
    let find = |kind: &str| {
        sites
            .iter()
            .find(|site| site["site_syntax_kind_value"] == kind)
            .unwrap()
    };
    assert_eq!(
        (
            &find("CALL_EXPR")["call_syntax_evidence"]["call_lexical_shadowing_flag"],
            &find("METHOD_CALL_EXPR")["call_syntax_evidence"]["method_receiver_evidence"]["receiver_rebinding_flag"]
        ),
        (&json!(false), &json!(false))
    );
}

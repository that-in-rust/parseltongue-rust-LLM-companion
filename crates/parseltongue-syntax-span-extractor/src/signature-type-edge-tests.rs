#![allow(clippy::unwrap_used, clippy::expect_used)]
use crate::extract_file_syntax_rows;
use crate::original_call_resolution_records::original_call_resolution_tests::make_call_syntax_input;
use serde_json::Value;

/// Every signature row of `source` as (row kind, row text, listed type paths).
/// A row that carries no list at all reports `None`.
fn read_signature_path_lists(source: &str) -> Vec<(String, String, Option<Vec<String>>)> {
    extract_file_syntax_rows(&make_call_syntax_input(source))
        .unwrap()
        .stage_unit_output_rows
        .iter()
        .filter(|row| row.stage_owned_table_name == "field-signature-detail-table")
        .map(|row| serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap())
        .filter(|row| {
            !matches!(
                row["site_syntax_kind_value"].as_str(),
                Some("ATTR" | "IMPL")
            )
        })
        .map(|row| {
            let paths = row["type_path_source_rows"].as_array().map(|paths| {
                paths
                    .iter()
                    .map(|path| {
                        let start = path["span_byte_start_offset"].as_u64().unwrap() as usize;
                        let end = path["span_byte_end_offset"].as_u64().unwrap() as usize;
                        assert_eq!(path["type_source_text"], source[start..end]);
                        assert_eq!(
                            path["type_path_self_flag"],
                            &source[start..end] == "Self",
                            "only a bare Self is flagged"
                        );
                        source[start..end].to_owned()
                    })
                    .collect()
            });
            (
                row["site_syntax_kind_value"].as_str().unwrap().to_owned(),
                row["site_syntax_source_text"].as_str().unwrap().to_owned(),
                paths,
            )
        })
        .collect()
}

fn row(kind: &str, text: &str, paths: &[&str]) -> (String, String, Option<Vec<String>>) {
    (
        kind.to_owned(),
        text.to_owned(),
        Some(paths.iter().map(|path| (*path).to_owned()).collect()),
    )
}

/// REQ-RUST-018.2
#[test]
fn test_signature_paths_listed_once() {
    let source = "struct S { a: m::A<B>, b: &'static [C] }\nstruct T(D);\n\
        impl S { fn run<I: E>(&self, x: Self, y: (F, F)) -> Option<Self::G> where I: H { loop {} } }\n";
    assert_eq!(
        read_signature_path_lists(source),
        vec![
            row("RECORD_FIELD", "a: m::A<B>", &["m::A<B>", "B"]),
            row("RECORD_FIELD", "b: &'static [C]", &["C"]),
            row("TUPLE_FIELD", "D", &["D"]),
            row("TYPE_PARAM", "I: E", &[]),
            row("TYPE_BOUND", "E", &["E"]),
            row("SELF_PARAM", "&self", &[]),
            row("PARAM", "x: Self", &["Self"]),
            row("PARAM", "y: (F, F)", &["F", "F"]),
            row(
                "RET_TYPE",
                "-> Option<Self::G>",
                &["Option<Self::G>", "Self::G"]
            ),
            row("WHERE_PRED", "I: H", &["I"]),
            row("TYPE_BOUND", "H", &["H"]),
        ],
        "each type path is listed once, on the nearest signature row that holds it"
    );
}

/// REQ-RUST-018.2
#[test]
fn test_expression_paths_not_types() {
    let source = "struct S { a: [u8; N::LEN], b: W<{ LEN }>, c: [u8; size::<X>()] }\n";
    assert_eq!(
        read_signature_path_lists(source),
        vec![
            row("RECORD_FIELD", "a: [u8; N::LEN]", &["u8"]),
            row("RECORD_FIELD", "b: W<{ LEN }>", &["W<{ LEN }>"]),
            row("RECORD_FIELD", "c: [u8; size::<X>()]", &["u8"]),
        ],
        "a path under an expression is not a signature type path"
    );
}

/// REQ-RUST-018.2
#[test]
fn test_body_position_rows_unlisted() {
    let source = "fn run(a: A) { let f = |b: B| -> C { loop {} }; }\n";
    let rows = read_signature_path_lists(source);
    assert_eq!(rows[0], row("PARAM", "a: A", &["A"]));
    assert!(
        rows[1..].iter().all(|(_, _, paths)| paths.is_none()),
        "closure parameters and return types are body sites: {rows:?}"
    );
    assert_eq!(rows.len(), 3);
}

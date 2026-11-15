use super::*;
use parseltongue_core_row_types::HldUnresolvedReasonKind;

const ITEM: (&str, &str, &str, bool) = ("struct Made;", "STRUCT", "Made", true);

fn build_expands_fixture_store() -> OriginalGraphFixtureStore {
    build_generated_fixture_store(ITEM.0, &[ITEM]).0
}

fn read_fixture_expands_rows(input: &HldMaterializationInputRecord) -> Vec<&HldEdgeInputRecord> {
    input
        .input_edge_records
        .iter()
        .filter(|edge| edge.edge_relation_kind_value == HldRelationKindValue::Expands)
        .collect()
}

fn find_fixture_span_record(source: &OriginalGraphFixtureStore, text: &str) -> Value {
    source.rows["span-reference-text-table"]
        .iter()
        .map(|row| serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap())
        .find(|span| span["span_source_code_text"] == text)
        .unwrap()
}

/// Rewrites the one stored expansion row of the fixture.
fn rewrite_fixture_expansion_row(
    source: &mut OriginalGraphFixtureStore,
    rewrite: impl Fn(&mut Value),
) {
    let row = source.rows.remove("macro-expanded-item-table").unwrap();
    let mut value: Value = serde_json::from_str(&row[0].stage_row_payload_text).unwrap();
    rewrite(&mut value);
    source.insert_source_fixture_row("macro-expanded-item-table", value);
}

fn check_fixture_row_rejected(source: &OriginalGraphFixtureStore, table: &str, reason: &str) {
    let result = derive_fixture_graph_input(source);
    assert!(
        matches!(
            &result,
            Err(OriginalSourceProjectionError::InvalidSourceRow { table: found, reason: text })
                if found == table && text.contains(reason)
        ),
        "{result:?}"
    );
}

/// REQ-RUST-051.0 E1-E3, E8: a macro call written in a source file has one `expands` row
/// to the definition its path names, sited at the call, with the matched arm.
#[test]
fn test_expands_call_definition_row() {
    let input = derive_fixture_graph_input(&build_expands_fixture_store()).unwrap();
    let rows = read_fixture_expands_rows(&input);
    let [edge] = rows[..] else {
        panic!("one expands row expected, found {}", rows.len());
    };
    let call = find_fixture_named_node(&input, "make!()");
    let definition = find_fixture_named_node(&input, "make");
    assert_eq!(edge.edge_source_reference, call.node_target_reference);
    assert_eq!(
        edge.edge_resolution_value,
        HldEdgeResolutionValue::Extracted {
            target: definition.node_target_reference.clone()
        }
    );
    assert_eq!(
        HldTargetIdentityValue::from(NameSymbolTargetKey::Definition(
            edge.edge_evidence_row_key
                .evidence_site_occurrence_key
                .clone()
        )),
        call.node_target_reference
    );
    assert_eq!(
        serde_json::from_str::<Value>(edge.edge_dispatch_metadata_text.as_deref().unwrap())
            .unwrap(),
        json!({"macro_name_text": "make", "macro_matched_arm_index": 0})
    );
}

/// REQ-RUST-051.0 E4: the latest scope revision of a call's lookup is the one in force;
/// two definitions at that revision are a typed error.
#[test]
fn test_expands_latest_scope_revision() {
    let mut source = build_expands_fixture_store();
    let call = find_fixture_span_record(&source, CALL_TEXT);
    let other = find_fixture_span_record(&source, "struct Record;");
    source.rows.remove("legacy-macro-scope-records");
    source.insert_fixture_scope_lookup(&call, None, 0);
    source.insert_fixture_scope_lookup(
        &call,
        Some(&find_fixture_span_record(&source, DEFINITION_TEXT)),
        1,
    );
    let input = derive_fixture_graph_input(&source).unwrap();
    assert_eq!(
        read_fixture_expands_rows(&input)[0].edge_resolution_value,
        HldEdgeResolutionValue::Extracted {
            target: find_fixture_named_node(&input, "make")
                .node_target_reference
                .clone()
        }
    );
    // A definition in scope that is not a macro definition node is not a target.
    source.insert_fixture_scope_lookup(&call, Some(&other), 1);
    check_fixture_row_rejected(&source, "legacy-macro-scope-records", "more than one");
    source.rows.remove("legacy-macro-scope-records");
    source.insert_fixture_scope_lookup(&call, Some(&other), 0);
    check_fixture_row_rejected(&source, "legacy-macro-scope-records", "no definition node");
}

/// REQ-RUST-051.0 E6: a call with no definition in scope and no expansion has one
/// unresolved row with the stored reason; with no expansion row the target is missing.
#[test]
fn test_expands_unresolved_call_row() {
    let mut source = build_expands_fixture_store();
    source.rows.remove("legacy-macro-scope-records");
    source
        .rows
        .get_mut("span-reference-text-table")
        .unwrap()
        .retain(|row| !row.stage_row_payload_text.contains("macro-source"));
    let mut missing = OriginalGraphFixtureStore {
        rows: source.rows.clone(),
        bytes: source.bytes.clone(),
        expanded: BTreeMap::new(),
    };
    rewrite_fixture_expansion_row(&mut source, |value| {
        *value = json!({"macro_resolution_state_kind":"unresolved", "macro_unresolved_reason_kind":"external_source_missing", "macro_unresolved_detail_text":"barrier", "macro_invocation_identity_record":value["macro_invocation_identity_record"]});
    });
    missing.rows.remove("macro-expanded-item-table");
    for (source, reason, stored) in [
        (
            &source,
            HldUnresolvedReasonKind::ExternalSourceMissing,
            json!("external_source_missing"),
        ),
        (
            &missing,
            HldUnresolvedReasonKind::MissingTarget,
            Value::Null,
        ),
    ] {
        let input = derive_fixture_graph_input(source).unwrap();
        let rows = read_fixture_expands_rows(&input);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].edge_resolution_value,
            HldEdgeResolutionValue::Unresolved { reason }
        );
        assert_eq!(
            rows[0]
                .edge_evidence_row_key
                .evidence_target_reference_value,
            None
        );
        let metadata: Value =
            serde_json::from_str(rows[0].edge_dispatch_metadata_text.as_deref().unwrap()).unwrap();
        assert_eq!(metadata["macro_unresolved_reason_kind"], stored);
    }
}

/// REQ-RUST-051.0 E7, not built: a call that expanded with no definition in textual scope
/// (a macro named by path) stops with a typed error.
#[test]
fn test_expands_pathless_expansion_rejected() {
    let mut source = build_expands_fixture_store();
    source.rows.remove("legacy-macro-scope-records");
    check_fixture_row_rejected(&source, "legacy-macro-scope-records", "no definition node");
}

/// REQ-RUST-051.0 E5: an expansion that copied a token from outside the definition in
/// scope contradicts the lookup and is a typed error.
#[test]
fn test_expands_token_origin_mismatch() {
    let mut source = build_expands_fixture_store();
    let definition = find_fixture_span_record(&source, DEFINITION_TEXT);
    let start = definition["span_byte_start_offset"].as_u64().unwrap();
    let build_origin_rows = |from: u64| {
        json!([
            {"token_source_origin_kind":"definition", "token_source_absolute_path":"/fixture/src/lib.rs", "token_original_byte_range":[from, from + 2], "token_output_byte_range":[0, 2]},
            {"token_source_origin_kind":"invocation", "token_source_absolute_path":"/fixture/src/lib.rs", "token_original_byte_range":[0, 2], "token_output_byte_range":[3, 5]},
        ])
    };
    rewrite_fixture_expansion_row(&mut source, |value| {
        value["macro_token_origin_rows"] = build_origin_rows(start + 1);
    });
    assert_eq!(
        read_fixture_expands_rows(&derive_fixture_graph_input(&source).unwrap()).len(),
        1
    );
    rewrite_fixture_expansion_row(&mut source, |value| {
        value["macro_token_origin_rows"] = build_origin_rows(start - 1);
    });
    check_fixture_row_rejected(
        &source,
        "macro-expanded-item-table",
        "outside the macro definition",
    );
}

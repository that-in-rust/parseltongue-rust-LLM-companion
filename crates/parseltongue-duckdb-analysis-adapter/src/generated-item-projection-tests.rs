use super::*;

#[path = "macro-call-expands-projection-tests.rs"]
mod macro_call_expands_projection_tests;

const CALL_TEXT: &str = "make!();";
const DEFINITION_TEXT: &str = "macro_rules! make { () => {} }";

fn build_fixture_digest_text(text: &str) -> String {
    blake3::hash(text.as_bytes()).to_hex().to_string()
}

fn build_expanded_fixture_path(chain: &[String]) -> String {
    format!(
        "/fixture/.parseltongue/macro-source/{}/expanded.rs",
        chain.join("-")
    )
}

impl OriginalGraphFixtureStore {
    /// Stores one expansion: its macro call row, its expanded text and one span per item.
    /// `call` is the file and byte range where the macro call is written.
    fn insert_fixture_macro_expansion(
        &mut self,
        chain: &[String],
        call: (&str, usize, usize),
        text: &str,
        items: &[(&str, &str, &str, bool)],
    ) -> BTreeMap<String, Value> {
        let path = build_expanded_fixture_path(chain);
        let digest = build_fixture_digest_text(text);
        self.expanded
            .insert(digest.clone(), text.as_bytes().to_vec());
        self.insert_source_fixture_row("macro-expanded-item-table", json!({"macro_resolution_state_kind":"expanded", "macro_virtual_source_path":path, "macro_invocation_provenance_chain":chain, "macro_matched_arm_index":0, "macro_invocation_identity_record":{"unit_compilation_identity_key":build_original_fixture_unit(), "span_source_file_path":call.0, "span_byte_range_value":[call.1, call.2], "span_expansion_provenance_chain":chain[..chain.len() - 1]}}));
        self.insert_source_fixture_row(
            "file-source-text-table",
            json!({"file_source_absolute_path":path, "file_content_digest_full":digest}),
        );
        items
            .iter()
            .map(|(item, kind, name, active)| {
                let start = text.find(item).unwrap();
                let end = start + item.len();
                let span = json!({"unit_compilation_identity_key":"raw-unit", "file_source_absolute_path":path, "span_byte_start_offset":start, "span_byte_end_offset":end, "span_content_digest_full":build_fixture_digest_text(item), "span_syntax_kind_value":kind, "span_declared_name_value":name, "span_visibility_source_text":"private", "span_source_code_text":item, "span_configuration_active_flag":active, "span_expansion_provenance_chain":chain, "span_semantic_occurrence_key":format!("expanded-{start}-{end}")});
                self.insert_source_fixture_row("span-reference-text-table", span.clone());
                ((*item).to_owned(), span)
            })
            .collect()
    }
}

impl OriginalGraphFixtureStore {
    /// Stores the textual-scope lookup of the macro path of `call` at one scope revision:
    /// the definition in scope, or none.
    fn insert_fixture_scope_lookup(
        &mut self,
        call: &Value,
        definition: Option<&Value>,
        revision: u64,
    ) {
        let start = call["span_byte_start_offset"].as_u64().unwrap();
        let site = json!({"unit_compilation_identity_key":build_original_fixture_unit(), "span_source_file_path":"src/lib.rs", "span_byte_range_value":[start, start + 4], "span_content_digest_full":build_fixture_digest_text("make"), "span_expansion_provenance_chain":[]});
        let target = definition.map(|span| json!({"target_identity_variant_kind":"definition", "target_semantic_occurrence_key":encode_fixture_occurrence_value(span)}));
        self.insert_source_fixture_row("legacy-macro-scope-records", json!({"legacy_lookup_identity_key":"lookup-make", "legacy_source_site_key":site, "legacy_macro_name_text":"make", "legacy_target_identity_key":target, "legacy_scope_revision_value":revision, "legacy_source_order_parts":[start, 3, 0]}));
    }
}

/// The base fixture with `macro_rules! make` and `make!();` written at the end of
/// `src/lib.rs`, the call's path looked up to that definition, and the call expanded to
/// `text`. Returns the store, the chain of the expansion and the generated spans by text.
fn build_generated_fixture_store(
    text: &str,
    items: &[(&str, &str, &str, bool)],
) -> (
    OriginalGraphFixtureStore,
    Vec<String>,
    BTreeMap<String, Value>,
) {
    let mut source = build_original_fixture_store();
    for text in [DEFINITION_TEXT, CALL_TEXT] {
        source.bytes.extend_from_slice(text.as_bytes());
        source.bytes.extend_from_slice(b"\r\n");
    }
    let file_digest = blake3::hash(&source.bytes).to_hex().to_string();
    source.rows.remove("file-source-text-table");
    source.insert_source_fixture_row("file-source-text-table", json!({"file_source_absolute_path":"/fixture/src/lib.rs", "file_content_digest_full":file_digest}));
    let definition =
        source.insert_fixture_span_record(DEFINITION_TEXT, "MACRO_RULES", "make", true);
    let call = source.insert_fixture_span_record(CALL_TEXT, "MACRO_CALL", "", true);
    for span in [&definition, &call] {
        source.insert_source_fixture_row("span-byte-ownership-ranges", json!({"unit_compilation_identity_key":"raw-unit", "file_source_absolute_path":"/fixture/src/lib.rs", "span_byte_start_offset":span["span_byte_start_offset"], "span_byte_end_offset":span["span_byte_end_offset"], "span_gap_classification_kind":"item", "span_expansion_provenance_chain":[]}));
    }
    source.insert_fixture_scope_lookup(&call, Some(&definition), 0);
    let (start, end) = (
        call["span_byte_start_offset"].as_u64().unwrap() as usize,
        call["span_byte_end_offset"].as_u64().unwrap() as usize,
    );
    let chain = vec!["33".repeat(32)];
    let spans =
        source.insert_fixture_macro_expansion(&chain, ("src/lib.rs", start, end), text, items);
    (source, chain, spans)
}

fn read_fixture_generated_nodes(input: &HldMaterializationInputRecord) -> Vec<&HldNodeInputRecord> {
    input
        .input_node_records
        .iter()
        .filter(|node| node.node_generating_call_reference.is_some())
        .collect()
}

fn read_fixture_relation_sources<'a>(
    input: &'a HldMaterializationInputRecord,
    node: &HldNodeInputRecord,
    relation: HldRelationKindValue,
) -> Vec<&'a HldTargetIdentityValue> {
    input
        .input_edge_records
        .iter()
        .filter(|edge| {
            edge.edge_relation_kind_value == relation
                && edge.edge_resolution_value
                    == HldEdgeResolutionValue::Extracted {
                        target: node.node_target_reference.clone(),
                    }
        })
        .map(|edge| &edge.edge_source_reference)
        .collect()
}

/// REQ-RUST-046.0 R1, R3, R5, R8: a generated struct and a member of a generated impl are
/// nodes, held by the file of the macro call, each with one `generates` row from the call.
#[test]
fn test_expanded_item_projected_node() {
    let text = "struct Made; impl Made { fn member() {} }";
    let (mut source, chain, spans) = build_generated_fixture_store(
        text,
        &[
            ("struct Made;", "STRUCT", "Made", true),
            ("impl Made { fn member() {} }", "IMPL", "", true),
            ("fn member() {}", "FN", "member", true),
        ],
    );
    let bridge = |span: &Value| json!({"unit_compilation_identity_key":build_original_fixture_unit(), "span_source_file_path":build_expanded_fixture_path(&chain).trim_start_matches("/fixture/"), "span_byte_range_value":[span["span_byte_start_offset"],span["span_byte_end_offset"]], "span_content_digest_full":span["span_content_digest_full"], "span_expansion_provenance_chain":chain});
    let (implementation, member) = (
        &spans["impl Made { fn member() {} }"],
        &spans["fn member() {}"],
    );
    source.insert_source_fixture_row("impl-header-index-table", json!({"impl_source_occurrence_key":implementation["span_semantic_occurrence_key"], "impl_semantic_occurrence_key":bridge(implementation), "impl_member_occurrence_rows":[{"member_source_occurrence_key":member["span_semantic_occurrence_key"], "member_semantic_occurrence_key":bridge(member)}], "impl_function_body_ancestry_flag":false, "impl_source_container":{"container_kind_value":"file_area"}}));
    let input = derive_fixture_graph_input(&source).unwrap();
    let generated = read_fixture_generated_nodes(&input);
    assert_eq!(generated.len(), 2, "the impl block itself is not a node");
    for (name, owner) in [
        ("Made", HldOwnerKindValue::CrateRoot),
        ("member", HldOwnerKindValue::Impl),
    ] {
        let node = find_fixture_named_node(&input, name);
        assert_eq!(node.node_owner_kind_value, Some(owner));
        assert_eq!(
            node.node_source_relative_path.as_deref(),
            Some("src/lib.rs")
        );
        let holders = read_fixture_relation_sources(&input, node, HldRelationKindValue::Contains);
        assert!(matches!(
            holders[..],
            [HldTargetIdentityValue::FilePublicArea(_)]
        ));
        let call = node.node_generating_call_reference.as_ref().unwrap();
        assert_eq!(
            read_fixture_relation_sources(&input, node, HldRelationKindValue::Generates),
            [call]
        );
        let call_node = input
            .input_node_records
            .iter()
            .find(|candidate| &candidate.node_target_reference == call)
            .unwrap();
        assert_eq!(
            call_node.node_item_kind_value,
            Some(HldItemKindValue::MacroInvocation)
        );
    }
}

/// REQ-RUST-046.0 R4, not built: an item written by a macro call that is itself inside
/// expanded text is rejected with a typed error. Pinned itoa has no such call.
#[test]
fn test_nested_expansion_call_rejected() {
    let (mut source, chain, _) = build_generated_fixture_store("inner!();", &[]);
    let nested = [chain[0].clone(), "44".repeat(32)];
    let outer_path = build_expanded_fixture_path(&chain);
    source.insert_fixture_macro_expansion(
        &nested,
        (outer_path.trim_start_matches("/fixture/"), 0, 9),
        "struct Deep;",
        &[("struct Deep;", "STRUCT", "Deep", true)],
    );
    assert!(matches!(
        derive_fixture_graph_input(&source),
        Err(OriginalSourceProjectionError::InvalidSourceRow { table, reason })
            if table == "macro-expanded-item-table" && reason.contains("source-written")
    ));
}

/// REQ-RUST-046.0 R6: expanded text that no longer matches its stored digest is a typed
/// error, not a skipped row.
#[test]
fn test_expanded_text_digest_mismatch() {
    let text = "struct Made;";
    let (mut source, _, _) = build_generated_fixture_store(text, &[(text, "STRUCT", "Made", true)]);
    source
        .expanded
        .insert(build_fixture_digest_text(text), b"struct Mado;".to_vec());
    assert!(matches!(
        derive_fixture_graph_input(&source),
        Err(OriginalSourceProjectionError::InvalidEvidence(_))
    ));
}

/// REQ-RUST-046.0 R7: a generated item whose `#[cfg]` is off gives no node and no row.
#[test]
fn test_inactive_expanded_item_skipped() {
    let text = "#[cfg(any())] struct Made;";
    let (source, _, _) = build_generated_fixture_store(text, &[(text, "STRUCT", "Made", false)]);
    let input = derive_fixture_graph_input(&source).unwrap();
    assert!(read_fixture_generated_nodes(&input).is_empty());
    assert!(
        input
            .input_edge_records
            .iter()
            .all(|edge| edge.edge_relation_kind_value != HldRelationKindValue::Generates)
    );
}

/// REQ-RUST-046.0 R1: an item declared inside a generated function body is body evidence,
/// not a node; the function is a node.
#[test]
fn test_expanded_body_item_evidence() {
    let text = "fn made() { struct Hidden; }";
    let (source, _, _) = build_generated_fixture_store(
        text,
        &[
            (text, "FN", "made", true),
            ("struct Hidden;", "STRUCT", "Hidden", true),
        ],
    );
    let input = derive_fixture_graph_input(&source).unwrap();
    let generated = read_fixture_generated_nodes(&input);
    assert_eq!(generated.len(), 1);
    assert!(generated[0].node_qualified_path_text.ends_with("::made"));
}

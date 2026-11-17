#![allow(clippy::unwrap_used)]

use super::*;
use parseltongue_core_row_types::{StageTableRowRecord, StageUnitWriteRequest};
use serde_json::json;
use std::collections::BTreeMap;

#[path = "generated-item-projection-tests.rs"]
mod generated_item_projection_tests;

struct OriginalGraphFixtureStore {
    rows: BTreeMap<String, Vec<StageTableRowRecord>>,
    bytes: Vec<u8>,
    /// Expanded macro texts by digest; any other digest reads the one source file.
    expanded: BTreeMap<String, Vec<u8>>,
}

impl StageTableStorePort for OriginalGraphFixtureStore {
    fn initialize_workspace_stage_schema(&self) -> Result<(), StoreBackendFailureError> {
        Ok(())
    }
    fn write_stage_table_unit(
        &self,
        _: &StageUnitWriteRequest<'_>,
        _: &RunCancelFlagValue,
    ) -> Result<usize, StoreBackendFailureError> {
        unreachable!()
    }
    fn read_stage_table_rows(
        &self,
        scope: &StageRunScopeValue,
        table: &str,
    ) -> Result<Vec<StageTableRowRecord>, StoreBackendFailureError> {
        assert_eq!(scope, &build_original_fixture_scope());
        Ok(self.rows.get(table).cloned().unwrap_or_default())
    }
    fn write_file_content_bytes(
        &self,
        _: &str,
        _: &[u8],
    ) -> Result<bool, StoreBackendFailureError> {
        unreachable!()
    }
    fn read_file_content_bytes(&self, digest: &str) -> Result<Vec<u8>, StoreBackendFailureError> {
        Ok(self.expanded.get(digest).unwrap_or(&self.bytes).clone())
    }
    fn publish_workspace_generation_manifest(
        &self,
        _: &StageRunScopeValue,
        _: &str,
        _: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, StoreBackendFailureError> {
        unreachable!()
    }
    fn read_workspace_publication_scope(
        &self,
    ) -> Result<Option<StageRunScopeValue>, StoreBackendFailureError> {
        Ok(Some(build_original_fixture_scope()))
    }
}

fn build_original_fixture_scope() -> StageRunScopeValue {
    StageRunScopeValue {
        run_generation_digest_value: "11".repeat(32),
        stage_attempt_sequence_value: "frozen-input".into(),
    }
}

fn build_original_fixture_unit() -> Value {
    json!({"unit_cargo_package_identifier":"fixture", "unit_cargo_target_name":"fixture", "unit_cargo_target_kind":"lib", "unit_compilation_role_kind":"target", "unit_configuration_digest_full":"22".repeat(32)})
}

fn encode_fixture_occurrence_value(span: &Value) -> Value {
    json!({"unit_compilation_identity_key":build_original_fixture_unit(), "span_source_file_path":"src/lib.rs", "span_byte_range_value":[span["span_byte_start_offset"],span["span_byte_end_offset"]], "span_content_digest_full":span["span_content_digest_full"], "span_expansion_provenance_chain":[]})
}

impl OriginalGraphFixtureStore {
    fn insert_source_fixture_row(&mut self, table: &str, value: Value) {
        let payload = value.to_string();
        let rows = self.rows.entry(table.into()).or_default();
        rows.push(StageTableRowRecord {
            stage_owned_table_name: table.into(),
            stage_row_record_key: blake3::hash(payload.as_bytes()).to_hex().to_string(),
            stage_row_payload_text: payload,
        });
        rows.sort_by(|a, b| a.stage_row_record_key.cmp(&b.stage_row_record_key));
    }

    fn insert_fixture_span_record(
        &mut self,
        text: &str,
        kind: &str,
        name: &str,
        active: bool,
    ) -> Value {
        let source = std::str::from_utf8(&self.bytes).unwrap();
        let start = if kind == "VARIANT" {
            source.rfind(text)
        } else {
            source.find(text)
        }
        .unwrap();
        let end = start + text.len();
        let span = json!({"unit_compilation_identity_key":"raw-unit", "file_source_absolute_path":"/fixture/src/lib.rs", "span_byte_start_offset":start, "span_byte_end_offset":end, "span_content_digest_full":blake3::hash(text.as_bytes()).to_hex().to_string(), "span_syntax_kind_value":kind, "span_declared_name_value":name, "span_visibility_source_text":"private", "span_signature_source_text":"fixture signature", "span_source_code_text":text, "span_configuration_active_flag":active, "span_expansion_provenance_chain":[], "span_semantic_occurrence_key":format!("source-{start}-{end}")});
        self.insert_source_fixture_row("span-reference-text-table", span.clone());
        span
    }

    fn insert_fixture_impl_header(&mut self, implementation: &Value, member: &Value, body: bool) {
        self.insert_source_fixture_row("impl-header-index-table", json!({"impl_source_occurrence_key":implementation["span_semantic_occurrence_key"], "impl_semantic_occurrence_key":encode_fixture_occurrence_value(implementation), "impl_member_occurrence_rows":[{"member_source_occurrence_key":member["span_semantic_occurrence_key"], "member_semantic_occurrence_key":encode_fixture_occurrence_value(member)}], "impl_function_body_ancestry_flag":body, "impl_source_container":{"container_kind_value":"file_area"}}));
    }
}

fn build_original_fixture_store() -> OriginalGraphFixtureStore {
    let text = "use crate::Record;\r\nstruct Record;\r\ntrait Trait { fn required(); const VALUE: u8; type Output; }\r\nimpl Record { fn same() {} }\r\nimpl Trait for Record { fn same() { } }\r\nmod nested { mod inner { impl Record { fn deep() {} } use super::Record; } }\r\nfn outer() { impl Record { fn hidden() {} } mod local { fn hidden_too() {} } }\r\nconst VALUE: u8 = { fn const_hidden() {} 1 };\r\n#[cfg(any())] mod disabled { fn off() {} }\r\nfn café() {}\r\nenum Choice { A }\r\n";
    let mut source = OriginalGraphFixtureStore {
        rows: BTreeMap::new(),
        bytes: text.as_bytes().to_vec(),
        expanded: BTreeMap::new(),
    };
    source.insert_source_fixture_row("crate-dependency-graph-table", json!({"unit_compilation_identity_key":"raw-unit", "unit_package_root_path":"/fixture", "unit_source_root_path":"/fixture/src/lib.rs", "unit_mapped_target_flag":true}));
    source.insert_source_fixture_row("syntax-namespace-input-scopes", json!({"scope_cargo_unit_key":"raw-unit", "scope_module_identity_key":{"scope_compilation_unit_key":build_original_fixture_unit(), "scope_semantic_path_parts":[]}}));
    source.insert_source_fixture_row("syntax-file-work-requests", json!({"unit_compilation_identity_key":"raw-unit", "file_source_absolute_path":"/fixture/src/lib.rs", "file_parse_context_kind":"items", "file_expansion_provenance_chain":[]}));
    source.insert_source_fixture_row("file-source-text-table", json!({"file_source_absolute_path":"/fixture/src/lib.rs", "file_content_digest_full":blake3::hash(text.as_bytes()).to_hex().to_string()}));
    let mut spans = BTreeMap::new();
    for (text, kind, name, active) in [
        ("use crate::Record;", "USE", "", true),
        ("struct Record;", "STRUCT", "Record", true),
        (
            "trait Trait { fn required(); const VALUE: u8; type Output; }",
            "TRAIT",
            "Trait",
            true,
        ),
        ("fn required();", "FN", "required", true),
        ("const VALUE: u8;", "CONST", "VALUE", true),
        ("type Output;", "TYPE_ALIAS", "Output", true),
        ("impl Record { fn same() {} }", "IMPL", "", true),
        ("fn same() {}", "FN", "same", true),
        ("impl Trait for Record { fn same() { } }", "IMPL", "", true),
        ("fn same() { }", "FN", "same", true),
        (
            "mod nested { mod inner { impl Record { fn deep() {} } use super::Record; } }",
            "MODULE",
            "nested",
            true,
        ),
        (
            "mod inner { impl Record { fn deep() {} } use super::Record; }",
            "MODULE",
            "inner",
            true,
        ),
        ("impl Record { fn deep() {} }", "IMPL", "", true),
        ("fn deep() {}", "FN", "deep", true),
        ("use super::Record;", "USE", "", true),
        (
            "fn outer() { impl Record { fn hidden() {} } mod local { fn hidden_too() {} } }",
            "FN",
            "outer",
            true,
        ),
        ("impl Record { fn hidden() {} }", "IMPL", "", true),
        ("fn hidden() {}", "FN", "hidden", true),
        ("mod local { fn hidden_too() {} }", "MODULE", "local", true),
        ("fn hidden_too() {}", "FN", "hidden_too", true),
        (
            "const VALUE: u8 = { fn const_hidden() {} 1 };",
            "CONST",
            "VALUE",
            true,
        ),
        ("fn const_hidden() {}", "FN", "const_hidden", true),
        (
            "#[cfg(any())] mod disabled { fn off() {} }",
            "MODULE",
            "disabled",
            false,
        ),
        ("fn off() {}", "FN", "off", false),
        ("fn café() {}", "FN", "café", true),
        ("enum Choice { A }", "ENUM", "Choice", true),
        ("A", "VARIANT", "A", true),
    ] {
        spans.insert(
            text,
            source.insert_fixture_span_record(text, kind, name, active),
        );
    }
    for (implementation, member, body) in [
        ("impl Record { fn same() {} }", "fn same() {}", false),
        (
            "impl Trait for Record { fn same() { } }",
            "fn same() { }",
            false,
        ),
        ("impl Record { fn deep() {} }", "fn deep() {}", false),
        ("impl Record { fn hidden() {} }", "fn hidden() {}", true),
    ] {
        source.insert_fixture_impl_header(&spans[implementation], &spans[member], body);
    }
    for span in spans.values() {
        let start = span["span_byte_start_offset"].as_u64().unwrap();
        let end = span["span_byte_end_offset"].as_u64().unwrap();
        if !spans.values().any(|other| {
            other["span_byte_start_offset"].as_u64().unwrap() < start
                && other["span_byte_end_offset"].as_u64().unwrap() > end
        }) {
            source.insert_source_fixture_row("span-byte-ownership-ranges", json!({"unit_compilation_identity_key":"raw-unit", "file_source_absolute_path":"/fixture/src/lib.rs", "span_byte_start_offset":start, "span_byte_end_offset":end, "span_gap_classification_kind":"item", "span_expansion_provenance_chain":[]}));
        }
    }
    source
}

fn derive_fixture_graph_input(
    source: &OriginalGraphFixtureStore,
) -> Result<HldMaterializationInputRecord, OriginalSourceProjectionError> {
    let directory = tempfile::tempdir().unwrap();
    let adapter = DuckdbAnalysisStoreAdapter::open_source_projection_store(
        &directory.path().join("analysis.duckdb"),
        directory.path(),
    )
    .unwrap();
    adapter.derive_original_graph_input(
        source,
        &build_original_fixture_scope(),
        &StageRunScopeValue {
            run_generation_digest_value: "11".repeat(32),
            stage_attempt_sequence_value: "destination".into(),
        },
        Path::new("/fixture"),
        &RunCancelFlagValue::default(),
    )
}

fn find_fixture_named_node<'a>(
    input: &'a HldMaterializationInputRecord,
    name: &str,
) -> &'a HldNodeInputRecord {
    input
        .input_node_records
        .iter()
        .find(|node| {
            node.node_qualified_path_text
                .ends_with(&format!("::{name}"))
        })
        .unwrap()
}

fn find_fixture_contains_source<'a>(
    input: &'a HldMaterializationInputRecord,
    node: &HldNodeInputRecord,
) -> &'a HldTargetIdentityValue {
    let parents = input
        .input_edge_records
        .iter()
        .filter(|edge| {
            edge.edge_resolution_value
                == HldEdgeResolutionValue::Extracted {
                    target: node.node_target_reference.clone(),
                }
        })
        .collect::<Vec<_>>();
    assert_eq!(parents.len(), 1);
    &parents[0].edge_source_reference
}

/// REQ-RUST-032.0, REQ-RUST-043.0, REQ-RUST-045.0
#[test]
fn test_impl_source_container_identity() {
    let source = build_original_fixture_store();
    let input = derive_fixture_graph_input(&source).unwrap();
    let area = input
        .input_node_records
        .iter()
        .find(|node| node.node_input_category == HldNodeInputCategory::FilePublicArea)
        .unwrap();
    let methods = input
        .input_node_records
        .iter()
        .filter(|node| node.node_qualified_path_text.ends_with("::same"))
        .collect::<Vec<_>>();
    assert_eq!(methods.len(), 2);
    assert_ne!(
        methods[0].node_target_reference,
        methods[1].node_target_reference
    );
    for method in methods {
        assert_eq!(
            method.node_item_kind_value,
            Some(HldItemKindValue::AssociatedFunction)
        );
        assert_eq!(method.node_owner_kind_value, Some(HldOwnerKindValue::Impl));
        assert_eq!(
            find_fixture_contains_source(&input, method),
            &area.node_target_reference
        );
    }
    assert!(
        !input
            .input_node_records
            .iter()
            .any(|node| node.node_item_kind_value == Some(HldItemKindValue::ImplContainer))
    );
    assert_eq!(
        input.semantic_input_completeness_kind,
        HldInputCompletenessKind::IncompleteTypedInput
    );
    assert_eq!(
        input.source_coverage_kind,
        HldSourceCoverageKind::FullSemantic
    );
    assert_eq!(input.stage_attempt_sequence_value, "destination");
    assert!(parseltongue_core_row_types::validate_hld_materialization_input(&input).is_err());
}

/// REQ-RUST-043.0
#[test]
fn test_impl_nested_container_identity() {
    let input = derive_fixture_graph_input(&build_original_fixture_store()).unwrap();
    let inner = find_fixture_named_node(&input, "inner");
    let deep = find_fixture_named_node(&input, "deep");
    assert_eq!(
        find_fixture_contains_source(&input, deep),
        &inner.node_target_reference
    );
    assert_eq!(
        find_fixture_contains_source(&input, inner),
        &find_fixture_named_node(&input, "nested").node_target_reference
    );
    let required = find_fixture_named_node(&input, "required");
    assert_eq!(
        required.node_owner_kind_value,
        Some(HldOwnerKindValue::Trait)
    );
    assert_eq!(
        find_fixture_contains_source(&input, required),
        &find_fixture_named_node(&input, "Trait").node_target_reference
    );
    assert_eq!(
        find_fixture_named_node(&input, "Output").node_item_kind_value,
        Some(HldItemKindValue::AssociatedType)
    );
    assert!(
        input
            .input_node_records
            .iter()
            .any(|node| node.node_item_kind_value == Some(HldItemKindValue::AssociatedConstant))
    );
    for node in input
        .input_node_records
        .iter()
        .filter(|node| node.node_input_category == HldNodeInputCategory::LocalItem)
    {
        find_fixture_contains_source(&input, node);
    }
}

/// REQ-RUST-032.0, REQ-RUST-043.0, REQ-RUST-045.0
#[test]
fn test_impl_body_members_excluded() {
    let source = build_original_fixture_store();
    let before = source.rows.clone();
    let input = derive_fixture_graph_input(&source).unwrap();
    assert_eq!(input.input_node_records.len(), 18);
    // 16 rows to items, and the crate root's row to its file area (REQ-RUST-070.0 A1).
    assert_eq!(input.input_edge_records.len(), 17);
    for name in [
        "hidden",
        "hidden_too",
        "const_hidden",
        "local",
        "off",
        "disabled",
        "A",
    ] {
        assert!(
            !input.input_node_records.iter().any(|node| node
                .node_qualified_path_text
                .ends_with(&format!("::{name}"))),
            "{name}"
        );
    }
    assert_eq!(source.rows, before);
    assert_eq!(
        input
            .input_node_records
            .iter()
            .filter(|node| node.node_item_kind_value == Some(HldItemKindValue::UseStatement))
            .count(),
        2
    );
    for node in input
        .input_node_records
        .iter()
        .filter(|node| node.node_input_category == HldNodeInputCategory::LocalItem)
    {
        // REQ-RUST-069.0 V1, S1: the projection copies the declared visibility and the
        // signature from the span row. Reachability is set after the edges are resolved.
        assert_eq!(
            node.node_effective_visibility_kind,
            Some(HldEffectiveVisibilityKind::PrivateEquivalent)
        );
        assert_eq!(node.node_reachable_outside_crate_flag, None);
        assert_eq!(
            node.node_signature_text.as_deref(),
            Some("fixture signature")
        );
    }
}

/// REQ-RUST-045.0
#[test]
fn test_original_source_evidence_exact() {
    let source = build_original_fixture_store();
    let input = derive_fixture_graph_input(&source).unwrap();
    let node = find_fixture_named_node(&input, "café");
    let HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(key)) =
        &node.node_target_reference
    else {
        panic!("item identity expected");
    };
    let parts = key.read_semantic_occurrence_parts();
    let (start, end) = parts.span_byte_range_value.read_source_range_pair();
    assert_eq!(
        &source.bytes[start as usize..end as usize],
        "fn café() {}".as_bytes()
    );
    assert_eq!(
        parts.span_content_digest_full.read_full_digest_bytes(),
        blake3::hash("fn café() {}".as_bytes()).as_bytes()
    );
    assert_eq!(parts.span_source_file_path, "src/lib.rs");
    assert_eq!(
        parts
            .unit_compilation_identity_key
            .unit_configuration_digest_full,
        [0x22; 32]
    );
}

/// REQ-RUST-043.0, REQ-RUST-045.0
#[test]
fn test_child_file_owner_identity() {
    let mut source = build_original_fixture_store();
    let row = &mut source.rows.get_mut("crate-dependency-graph-table").unwrap()[0];
    let mut unit: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
    unit["unit_source_root_path"] = json!("/fixture/src/entry.rs");
    row.stage_row_payload_text = unit.to_string();
    let input = derive_fixture_graph_input(&source).unwrap();
    for name in ["Record", "café"] {
        let node = find_fixture_named_node(&input, name);
        assert_eq!(node.node_owner_kind_value, Some(HldOwnerKindValue::Module));
        assert!(matches!(
            find_fixture_contains_source(&input, node),
            HldTargetIdentityValue::FilePublicArea(_)
        ));
    }
}

/// REQ-RUST-043.1 C1, C2: the unit's root module is one node, labelled by the target
/// name, identified by the unit, with no file, kind, owner or visibility and no edge.
#[test]
fn test_crate_root_node_stored() {
    let input = derive_fixture_graph_input(&build_original_fixture_store()).unwrap();
    let roots = input
        .input_node_records
        .iter()
        .filter(|node| node.node_input_category == HldNodeInputCategory::CrateRoot)
        .collect::<Vec<_>>();
    assert_eq!(roots.len(), 1);
    let root = roots[0];
    assert_eq!(root.node_qualified_path_text, "fixture");
    let unit = &input.input_unit_provenance_records[0].unit_compilation_identity_key;
    assert_eq!(
        root.node_target_reference,
        HldTargetIdentityValue::from(NameSymbolTargetKey::CrateRoot(unit.clone()))
    );
    assert_eq!(
        (
            root.node_item_kind_value,
            root.node_owner_kind_value,
            root.node_effective_visibility_kind,
            root.node_reachable_outside_crate_flag,
            root.node_source_relative_path.as_deref(),
            root.node_function_body_ancestry_flag,
        ),
        (None, None, None, None, None, false)
    );
    let touching = input
        .input_edge_records
        .iter()
        .filter(|edge| {
            edge.edge_source_reference == root.node_target_reference
                || edge.edge_resolution_value
                    == HldEdgeResolutionValue::Extracted {
                        target: root.node_target_reference.clone(),
                    }
        })
        .count();
    // REQ-RUST-070.0 A1: its one row is `contains` to the file area of the root file.
    assert_eq!(touching, 1);
}

/// REQ-RUST-070.0 A2, A3: a `mod child;` declaration whose file was read gives one
/// `contains` row from the module to that file's area, sited at the declaration; the
/// inline module `nested` gives none; a second read candidate is refused, not chosen.
#[test]
fn test_module_declaration_area_edge() {
    let mut source = build_original_fixture_store();
    source.bytes.extend_from_slice(b"mod child;\r\n");
    let digest = blake3::hash(&source.bytes).to_hex().to_string();
    source.rows.remove("file-source-text-table");
    for path in ["/fixture/src/lib.rs", "/fixture/src/child.rs"] {
        source.insert_source_fixture_row(
            "file-source-text-table",
            json!({"file_source_absolute_path":path, "file_content_digest_full":digest}),
        );
    }
    source.insert_source_fixture_row("syntax-file-work-requests", json!({"unit_compilation_identity_key":"raw-unit", "file_source_absolute_path":"/fixture/src/child.rs", "file_parse_context_kind":"items", "file_expansion_provenance_chain":[]}));
    let span = source.insert_fixture_span_record("mod child;", "MODULE", "child", true);
    source.insert_source_fixture_row("span-byte-ownership-ranges", json!({"unit_compilation_identity_key":"raw-unit", "file_source_absolute_path":"/fixture/src/lib.rs", "span_byte_start_offset":span["span_byte_start_offset"], "span_byte_end_offset":span["span_byte_end_offset"], "span_gap_classification_kind":"item", "span_expansion_provenance_chain":[]}));
    let request = |candidates: &[&str]| json!({"source_request_path_candidates":candidates, "source_request_context_kind":"items", "source_request_origin_digest":span["span_semantic_occurrence_key"], "source_request_origin_kind":"module_declaration", "unit_compilation_identity_key":"raw-unit", "file_expansion_provenance_chain":[]});
    source.insert_source_fixture_row(
        MODULE_FILE_LINK_TABLE,
        request(&["/fixture/src/child.rs", "/fixture/src/child/mod.rs"]),
    );
    let input = derive_fixture_graph_input(&source).unwrap();
    let area_rows = input
        .input_edge_records
        .iter()
        .filter_map(|edge| match &edge.edge_resolution_value {
            HldEdgeResolutionValue::Extracted {
                target: HldTargetIdentityValue::FilePublicArea(area),
            } => Some((edge, area)),
            _ => None,
        })
        .collect::<Vec<_>>();
    let child = find_fixture_named_node(&input, "child");
    let nested = find_fixture_named_node(&input, "nested");
    assert_eq!(
        area_rows.len(),
        2,
        "the crate root's row and the declaration's"
    );
    assert!(
        area_rows
            .iter()
            .all(|(edge, _)| edge.edge_source_reference != nested.node_target_reference)
    );
    let (edge, _) = area_rows
        .iter()
        .find(|(edge, _)| edge.edge_source_reference == child.node_target_reference)
        .unwrap();
    assert_eq!(
        edge.edge_relation_kind_value,
        HldRelationKindValue::Contains
    );
    let site = edge
        .edge_evidence_row_key
        .evidence_site_occurrence_key
        .read_semantic_occurrence_parts();
    assert_eq!(site.span_source_file_path, "src/lib.rs");
    assert_eq!(
        site.span_byte_range_value.read_source_range_pair(),
        (
            span["span_byte_start_offset"].as_u64().unwrap(),
            span["span_byte_end_offset"].as_u64().unwrap()
        )
    );
    assert_eq!(
        site.span_content_digest_full.read_full_digest_bytes(),
        blake3::hash(b"mod child;").as_bytes()
    );

    source.insert_source_fixture_row(MODULE_FILE_LINK_TABLE, request(&["/fixture/src/lib.rs"]));
    assert!(matches!(
        derive_fixture_graph_input(&source),
        Err(OriginalSourceProjectionError::InvalidSourceRow { .. })
    ));
}

/// REQ-RUST-043.1 C3: a unit whose root file is not read has no crate-root node.
#[test]
fn test_crate_root_node_absent() {
    let mut source = build_original_fixture_store();
    let row = &mut source.rows.get_mut("crate-dependency-graph-table").unwrap()[0];
    let mut unit: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
    unit["unit_source_root_path"] = json!("/fixture/src/entry.rs");
    row.stage_row_payload_text = unit.to_string();
    let input = derive_fixture_graph_input(&source).unwrap();
    assert!(
        input
            .input_node_records
            .iter()
            .all(|node| node.node_input_category != HldNodeInputCategory::CrateRoot)
    );
}

/// REQ-RUST-046.0 R6: a span that names a macro expansion with no stored macro call is
/// rejected; it is neither read as original text nor dropped.
#[test]
fn test_reject_orphan_generated_span() {
    let mut source = build_original_fixture_store();
    let mut expanded = source.rows["span-reference-text-table"]
        .iter()
        .map(|row| serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap())
        .find(|value| value["span_declared_name_value"] == "café")
        .unwrap();
    expanded["span_expansion_provenance_chain"] = json!(["33".repeat(32)]);
    expanded["span_semantic_occurrence_key"] = json!("expanded");
    expanded["span_declared_name_value"] = json!("generated");
    source.insert_source_fixture_row("span-reference-text-table", expanded);
    assert!(matches!(
        derive_fixture_graph_input(&source),
        Err(OriginalSourceProjectionError::InvalidSourceRow { table, .. })
            if table == "macro-expanded-item-table"
    ));
}

/// REQ-RUST-045.0
#[test]
fn test_reject_corrupt_source_evidence() {
    let mut source = build_original_fixture_store();
    let row = source
        .rows
        .get_mut("span-reference-text-table")
        .unwrap()
        .iter_mut()
        .find(|row| {
            let value: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
            value["span_declared_name_value"] == "café"
        })
        .unwrap();
    let mut value: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
    value["span_source_code_text"] = json!("fn wrong() {}");
    row.stage_row_payload_text = value.to_string();
    assert!(matches!(
        derive_fixture_graph_input(&source),
        Err(OriginalSourceProjectionError::InvalidEvidence(_))
    ));
}

/// REQ-RUST-043.0, REQ-RUST-045.0
#[test]
fn test_reject_mismatched_member_identity() {
    let mut source = build_original_fixture_store();
    let row = source
        .rows
        .get_mut("impl-header-index-table")
        .unwrap()
        .iter_mut()
        .find(|row| {
            let value: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
            value["impl_function_body_ancestry_flag"] == false
        })
        .unwrap();
    let mut value: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
    value["impl_member_occurrence_rows"][0]["member_semantic_occurrence_key"]["span_content_digest_full"] =
        json!("00".repeat(32));
    row.stage_row_payload_text = value.to_string();
    assert!(derive_fixture_graph_input(&source).is_err());
}

/// REQ-RUST-043.0, REQ-RUST-045.0
#[test]
fn test_reject_missing_impl_evidence() {
    let mut source = build_original_fixture_store();
    source.rows.remove("impl-header-index-table");
    assert!(derive_fixture_graph_input(&source).is_err());
}

/// REQ-RUST-020.1, REQ-RUST-045.0
#[test]
fn test_reject_cancelled_original_projection() {
    let source = build_original_fixture_store();
    let directory = tempfile::tempdir().unwrap();
    let adapter = DuckdbAnalysisStoreAdapter::open_source_projection_store(
        &directory.path().join("analysis.duckdb"),
        directory.path(),
    )
    .unwrap();
    let cancel = RunCancelFlagValue::default();
    cancel.request_stage_work_cancellation();
    assert!(
        adapter
            .derive_original_graph_input(
                &source,
                &build_original_fixture_scope(),
                &build_original_fixture_scope(),
                Path::new("/fixture"),
                &cancel
            )
            .is_err()
    );
}

/// REQ-RUST-032.0, REQ-RUST-043.0, REQ-RUST-045.0
#[test]
fn test_preserve_known_syntax_kinds() {
    let mut source = build_original_fixture_store();
    let text = "union U { x: u8 }\nmacro_rules! m { () => {} }\nmacro m2() {}\nm!();\nextern \"C\" { fn foreign(); static FOREIGN: u8; type Opaque; }\nextern crate core;\nasm!(\"nop\");\n";
    source.bytes = text.as_bytes().to_vec();
    for table in [
        "span-reference-text-table",
        "span-byte-ownership-ranges",
        "impl-header-index-table",
        "file-source-text-table",
    ] {
        source.rows.remove(table);
    }
    source.insert_source_fixture_row("file-source-text-table", json!({"file_source_absolute_path":"/fixture/src/lib.rs", "file_content_digest_full":blake3::hash(text.as_bytes()).to_hex().to_string()}));
    let spans = [
        ("union U { x: u8 }", "UNION", "U", "Union"),
        (
            "macro_rules! m { () => {} }",
            "MACRO_RULES",
            "m",
            "MacroDefinition",
        ),
        ("macro m2() {}", "MACRO_DEF", "m2", "MacroDefinition"),
        ("m!();", "MACRO_CALL", "", "MacroInvocation"),
        (
            "extern \"C\" { fn foreign(); static FOREIGN: u8; type Opaque; }",
            "EXTERN_BLOCK",
            "extern",
            "ExternBlock",
        ),
        ("extern crate core;", "EXTERN_CRATE", "core", "ExternCrate"),
        ("asm!(\"nop\");", "ASM_EXPR", "assembly", "InlineAssembly"),
    ];
    for (text, kind, name, _) in spans {
        let span = source.insert_fixture_span_record(text, kind, name, true);
        source.insert_source_fixture_row("span-byte-ownership-ranges", json!({"unit_compilation_identity_key":"raw-unit", "file_source_absolute_path":"/fixture/src/lib.rs", "span_byte_start_offset":span["span_byte_start_offset"], "span_byte_end_offset":span["span_byte_end_offset"], "span_gap_classification_kind":"item", "span_expansion_provenance_chain":[]}));
    }
    for (text, kind, name) in [
        ("fn foreign();", "FN", "foreign"),
        ("static FOREIGN: u8;", "STATIC", "FOREIGN"),
        ("type Opaque;", "TYPE_ALIAS", "Opaque"),
    ] {
        source.insert_fixture_span_record(text, kind, name, true);
    }
    let input = derive_fixture_graph_input(&source).unwrap();
    assert_eq!(input.input_node_records.len(), 12);
    for (_, syntax_kind, name, kind) in spans {
        // REQ-RUST-050.0 L3: a macro call is labelled with its own source text.
        let name = if syntax_kind == "MACRO_CALL" {
            "m!()"
        } else {
            name
        };
        assert_eq!(
            format!(
                "{:?}",
                find_fixture_named_node(&input, name)
                    .node_item_kind_value
                    .unwrap()
            ),
            kind
        );
    }
    for name in ["foreign", "FOREIGN", "Opaque"] {
        let node = find_fixture_named_node(&input, name);
        assert_eq!(
            format!("{:?}", node.node_owner_kind_value.unwrap()),
            "ExternBlock"
        );
        assert!(matches!(
            find_fixture_contains_source(&input, node),
            HldTargetIdentityValue::FilePublicArea(_)
        ));
    }
    assert_eq!(
        find_fixture_named_node(&input, "foreign").node_item_kind_value,
        Some(HldItemKindValue::Function)
    );
}

/// REQ-RUST-045.0
#[test]
fn test_reject_unknown_syntax_kind() {
    let mut source = build_original_fixture_store();
    for row in source.rows.get_mut("span-reference-text-table").unwrap() {
        let mut value: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
        if value["span_declared_name_value"] == "café" {
            value["span_syntax_kind_value"] = json!("UNRECOGNIZED_ITEM");
            row.stage_row_payload_text = value.to_string();
        }
    }
    assert!(matches!(
        derive_fixture_graph_input(&source),
        Err(OriginalSourceProjectionError::UnsupportedActiveSyntax { .. })
    ));
}

/// REQ-RUST-020.1, REQ-RUST-045.0
#[test]
fn test_reject_mismatched_generation_scope() {
    let source = build_original_fixture_store();
    let directory = tempfile::tempdir().unwrap();
    let adapter = DuckdbAnalysisStoreAdapter::open_source_projection_store(
        &directory.path().join("analysis.duckdb"),
        directory.path(),
    )
    .unwrap();
    let mut destination = build_original_fixture_scope();
    destination.run_generation_digest_value = "44".repeat(32);
    assert!(
        adapter
            .derive_original_graph_input(
                &source,
                &build_original_fixture_scope(),
                &destination,
                Path::new("/fixture"),
                &RunCancelFlagValue::default()
            )
            .is_err()
    );
}

/// REQ-RUST-045.0
#[test]
fn test_reject_missing_file_evidence() {
    let mut source = build_original_fixture_store();
    source.rows.remove("file-source-text-table");
    assert!(matches!(
        derive_fixture_graph_input(&source),
        Err(OriginalSourceProjectionError::MissingFileEvidence { .. })
    ));
}

/// REQ-RUST-045.0
#[test]
fn test_reject_missing_canonical_mapping() {
    let mut source = build_original_fixture_store();
    source.rows.remove("syntax-namespace-input-scopes");
    assert!(matches!(
        derive_fixture_graph_input(&source),
        Err(OriginalSourceProjectionError::MissingRootMapping { .. })
    ));
}

/// REQ-RUST-043.0, REQ-RUST-045.0
#[test]
fn test_reject_conflicting_span_identity() {
    let mut source = build_original_fixture_store();
    let row = source.rows["span-reference-text-table"]
        .iter()
        .find(|row| {
            let value: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
            value["span_declared_name_value"] == "café"
        })
        .unwrap();
    let mut value: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
    value["span_declared_name_value"] = json!("conflicting_name");
    source.insert_source_fixture_row("span-reference-text-table", value);
    assert!(derive_fixture_graph_input(&source).is_err());
}

/// REQ-RUST-043.0, REQ-RUST-045.0
#[test]
fn test_reject_unowned_source_item() {
    let mut source = build_original_fixture_store();
    source.rows.remove("span-byte-ownership-ranges");
    assert!(matches!(
        derive_fixture_graph_input(&source),
        Err(OriginalSourceProjectionError::MissingSpanOwnership)
    ));
}

/// REQ-RUST-045.0
#[test]
fn test_reject_corrupt_file_digest() {
    let mut source = build_original_fixture_store();
    source.bytes[0] = b'!';
    assert!(matches!(
        derive_fixture_graph_input(&source),
        Err(OriginalSourceProjectionError::InvalidEvidence(_))
    ));
}

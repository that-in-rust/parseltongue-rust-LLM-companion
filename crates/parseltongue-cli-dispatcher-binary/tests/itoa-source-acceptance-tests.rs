//! Phase 1 real-source acceptance tests on pinned itoa (SOP §1.1).
//!
//! The pinned source goes through the production CLI entry point. Tests read only rows of the
//! published generation; expected rows live in assertions here, never in production input.
#![allow(clippy::unwrap_used)]

use parseltongue_core_row_types::{
    HldEdgeResolutionValue, HldEffectiveVisibilityKind, HldInputCompletenessKind, HldItemKindValue,
    HldMaterializationInputRecord, HldNodeInputCategory, HldNodeInputRecord, HldOwnerKindValue,
    HldRelationKindValue, HldTargetIdentityValue, NameSymbolTargetKey, RunCancelFlagValue,
    validate_hld_materialization_input,
};
use parseltongue_table_store_traits::{
    FullGraphInputStorePort, HldGraphInputStorePort, StageTableStorePort,
};
use parseltongue_turso_store_adapter::TursoWorkspaceStoreAdapter;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const PINNED_ITOA_TREE_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/pinned-itoa-source-tree");
const PHASE_ONE_PIN_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/phase-20261111/phase-one-benchmark-pin.json"
);

fn hash_file_sha256_hex(path: &Path) -> String {
    Sha256::digest(fs::read(path).unwrap())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn canonical_unit_identity_from_root_scope(database: &Path, unit: &Value) -> Value {
    let unit_key = unit["unit_compilation_identity_key"].as_str().unwrap();
    let root_scope = read_published_stage_rows(database, "syntax-namespace-input-scopes")
        .into_iter()
        .find(|row| {
            row["scope_cargo_unit_key"] == unit_key
                && row["scope_module_identity_key"]["scope_semantic_path_parts"]
                    .as_array()
                    .is_some_and(Vec::is_empty)
        })
        .unwrap();
    let canonical_unit = &root_scope["scope_module_identity_key"]["scope_compilation_unit_key"];
    for field in [
        "unit_cargo_package_identifier",
        "unit_cargo_target_name",
        "unit_cargo_target_kind",
    ] {
        assert_eq!(canonical_unit[field], unit[field]);
    }
    assert_eq!(
        root_scope["scope_cargo_unit_key"],
        unit["unit_compilation_identity_key"]
    );
    let role = match canonical_unit["unit_compilation_role_kind"]
        .as_str()
        .unwrap()
    {
        "target" | "Target" => "Target",
        "host" | "Host" => "Host",
        other => panic!("unknown canonical compilation role {other}"),
    };
    assert_eq!(
        role.to_ascii_lowercase(),
        unit["unit_compilation_role_kind"]
    );
    let digest_hex = canonical_unit["unit_configuration_digest_full"]
        .as_str()
        .unwrap();
    assert_eq!(digest_hex.len(), 64);
    let digest_bytes = digest_hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(digest_bytes.len(), 32);
    json!({
        "cargo_package_identifier": canonical_unit["unit_cargo_package_identifier"],
        "cargo_target_name": canonical_unit["unit_cargo_target_name"],
        "cargo_target_kind": canonical_unit["unit_cargo_target_kind"],
        "compilation_role_kind": role,
        "configuration_digest_full": digest_bytes
    })
}

/// Copies the vendored pin into a fresh directory, checks every digest the pin manifest
/// records, and provisions the Cargo cache the offline production loader needs.
fn prepare_pinned_itoa_workspace(parent: &Path) -> PathBuf {
    let pin: Value = serde_json::from_slice(&fs::read(PHASE_ONE_PIN_PATH).unwrap()).unwrap();
    let tree = Path::new(PINNED_ITOA_TREE_PATH);
    let root = parent.join("itoa");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("benches")).unwrap();
    [
        ("upstream-cargo-manifest-file.toml", "Cargo.toml"),
        ("qualification-cargo-lock-file.toml", "Cargo.lock"),
        ("src/lib.rs", "src/lib.rs"),
        ("src/u128_ext.rs", "src/u128_ext.rs"),
        ("benches/bench.rs", "benches/bench.rs"),
    ]
    .iter()
    .for_each(|(from, to)| {
        fs::copy(tree.join(from), root.join(to)).unwrap();
    });
    assert_eq!(
        hash_file_sha256_hex(&root.join("Cargo.toml")),
        pin["benchmark_manifest_digest_sha256"].as_str().unwrap(),
        "vendored manifest differs from the pin"
    );
    let inventory = pin["benchmark_source_inventory_records"]
        .as_array()
        .unwrap();
    assert_eq!(inventory.len(), 2, "the pin lists the whole library");
    inventory.iter().for_each(|record| {
        let path = record["path"].as_str().unwrap();
        assert_eq!(
            hash_file_sha256_hex(&root.join(path)),
            record["sha256"].as_str().unwrap(),
            "vendored {path} differs from the pin"
        );
    });
    let fetch = Command::new(env!("CARGO"))
        .current_dir(&root)
        .args(["fetch", "--locked"])
        .output()
        .unwrap();
    assert!(
        fetch.status.success(),
        "BLOCKED: Cargo cache for the pinned lockfile is missing and could not be fetched: {}",
        String::from_utf8_lossy(&fetch.stderr)
    );
    root
}

fn run_itoa_mapping_command(root: &Path, database: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_parseltongue"))
        .args([
            "map-workspace-full-run",
            "--workspace-root-path-value",
            root.to_str().unwrap(),
            "--workspace-store-file-path",
            database.to_str().unwrap(),
            "--worker-thread-count-value",
            "2",
        ])
        .args(extra)
        .output()
        .unwrap()
}

fn read_published_stage_rows(database: &Path, table: &str) -> Vec<Value> {
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let scope = store
        .read_workspace_publication_scope()
        .unwrap()
        .unwrap_or_else(|| panic!("no published generation in {}", database.display()));
    store
        .read_stage_table_rows(&scope, table)
        .unwrap()
        .iter()
        .map(|row| serde_json::from_str(&row.stage_row_payload_text).unwrap())
        .collect()
}

fn run_original_graph_fixture() -> (
    tempfile::TempDir,
    PathBuf,
    PathBuf,
    HldMaterializationInputRecord,
) {
    let directory = tempfile::tempdir().unwrap();
    let root = prepare_pinned_itoa_workspace(directory.path());
    let database = directory.path().join("original-full-graph.db");
    let output = run_itoa_mapping_command(
        &root,
        &database,
        &[
            "--sysroot-source-scope-kind",
            "excluded",
            "--mapping-final-stage-kind",
            "full-item-dependency-graph",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let scope = store.read_workspace_publication_scope().unwrap().unwrap();
    let cancel = RunCancelFlagValue::default();
    let input = store.assemble_full_graph_input(&scope, &cancel).unwrap();
    assert_eq!(
        input.semantic_input_completeness_kind,
        HldInputCompletenessKind::IncompleteTypedInput
    );
    assert!(store.assemble_hld_graph_input(&scope, &cancel).is_err());
    assert!(validate_hld_materialization_input(&input).is_err());
    for table in [
        "hld-current-interface-nodes",
        "hld-current-dependency-edges",
        "hld-edge-evidence-paths",
        "hld-file-level-rollup",
        "hld-folder-level-rollup",
    ] {
        assert!(
            store
                .read_stage_table_rows(&scope, table)
                .unwrap()
                .is_empty(),
            "HLD output must stay absent: {table}"
        );
    }
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    (directory, root, database, input)
}

fn locate_original_member_node<'a>(
    input: &'a HldMaterializationInputRecord,
    spans: &[Value],
    name: &str,
) -> &'a HldNodeInputRecord {
    let spans = spans
        .iter()
        .filter(|span| {
            span["span_expansion_provenance_chain"] == json!([])
                && span["span_declared_name_value"] == name
                && span["span_syntax_kind_value"] == "FN"
        })
        .collect::<Vec<_>>();
    assert_eq!(spans.len(), 1, "pin must have one original {name} method");
    input
        .input_node_records
        .iter()
        .find(|node| {
            let HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(key)) =
                &node.node_target_reference
            else {
                return false;
            };
            let parts = key.read_semantic_occurrence_parts();
            parts.span_source_file_path == "src/lib.rs"
                && parts.span_byte_range_value.read_source_range_pair()
                    == (
                        spans[0]["span_byte_start_offset"].as_u64().unwrap(),
                        spans[0]["span_byte_end_offset"].as_u64().unwrap(),
                    )
        })
        .unwrap_or_else(|| panic!("missing canonical method node: {name}"))
}

/// REQ-RUST-032.0, REQ-RUST-043.0, REQ-RUST-045.0
#[test]
fn test_original_member_graph_roundtrip() {
    let (_directory, root, database, input) = run_original_graph_fixture();
    let spans = read_published_stage_rows(&database, "span-reference-text-table");
    let headers = read_published_stage_rows(&database, "impl-header-index-table");
    let local = input
        .input_node_records
        .iter()
        .filter(|node| node.node_input_category == HldNodeInputCategory::LocalItem)
        .collect::<Vec<_>>();
    assert!(!local.is_empty());
    assert!(
        local
            .iter()
            .any(
                |node| node.node_source_relative_path.as_deref() == Some("src/u128_ext.rs")
                    && node.node_item_kind_value == Some(HldItemKindValue::Function)
            )
    );
    assert_eq!(
        input
            .input_node_records
            .iter()
            .filter(|node| node.node_input_category == HldNodeInputCategory::FilePublicArea)
            .count(),
        2
    );
    for node in &local {
        assert!(!node.node_function_body_ancestry_flag);
        assert_ne!(
            node.node_item_kind_value,
            Some(HldItemKindValue::ImplContainer)
        );
        assert_eq!(
            node.node_effective_visibility_kind,
            Some(HldEffectiveVisibilityKind::Unknown)
        );
        assert_eq!(node.node_reachable_outside_crate_flag, None);
        if node.node_source_relative_path.as_deref() == Some("src/u128_ext.rs")
            && node.node_item_kind_value == Some(HldItemKindValue::Function)
        {
            assert_eq!(node.node_owner_kind_value, Some(HldOwnerKindValue::Module));
        }
        let HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(key)) =
            &node.node_target_reference
        else {
            panic!("original item identity required");
        };
        let parts = key.read_semantic_occurrence_parts();
        let (start, end) = parts.span_byte_range_value.read_source_range_pair();
        assert!(parts.span_expansion_provenance_chain.is_empty());
        let bytes = fs::read(root.join(parts.span_source_file_path)).unwrap();
        assert_eq!(
            blake3::hash(&bytes[start as usize..end as usize]).as_bytes(),
            parts.span_content_digest_full.read_full_digest_bytes()
        );
        assert!(
            !spans
                .iter()
                .any(|span| span["span_syntax_kind_value"] == "FN"
                    && span["span_expansion_provenance_chain"] == json!([])
                    && span["file_source_absolute_path"].as_str()
                        == root.join(parts.span_source_file_path).to_str()
                    && span["span_byte_start_offset"].as_u64().unwrap() < start
                    && span["span_byte_end_offset"].as_u64().unwrap() > end),
            "body descendant became node"
        );
        let parents = input
            .input_edge_records
            .iter()
            .filter(|edge| {
                edge.edge_relation_kind_value == HldRelationKindValue::Contains
                    && edge.edge_resolution_value
                        == HldEdgeResolutionValue::Extracted {
                            target: node.node_target_reference.clone(),
                        }
            })
            .collect::<Vec<_>>();
        assert_eq!(
            parents.len(),
            1,
            "every local item needs one lexical container"
        );
    }
    let default = locate_original_member_node(&input, &spans, "default");
    let new = locate_original_member_node(&input, &spans, "new");
    assert_ne!(default.node_target_reference, new.node_target_reference);
    for node in [default, new] {
        assert_eq!(
            node.node_item_kind_value,
            Some(HldItemKindValue::AssociatedFunction)
        );
        assert_eq!(node.node_owner_kind_value, Some(HldOwnerKindValue::Impl));
        let HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(key)) =
            &node.node_target_reference
        else {
            unreachable!()
        };
        let parts = key.read_semantic_occurrence_parts();
        let range = parts.span_byte_range_value.read_source_range_pair();
        let bridges = headers
            .iter()
            .flat_map(|header| header["impl_member_occurrence_rows"].as_array().unwrap())
            .filter(|member| {
                member["member_semantic_occurrence_key"]["span_source_file_path"]
                    == parts.span_source_file_path
                    && member["member_semantic_occurrence_key"]["span_byte_range_value"]
                        == json!([range.0, range.1])
                    && member["member_semantic_occurrence_key"]["span_expansion_provenance_chain"]
                        == json!([])
            })
            .collect::<Vec<_>>();
        assert_eq!(
            bridges.len(),
            1,
            "member needs one separate impl evidence bridge"
        );
        assert_eq!(
            bridges[0]["member_semantic_occurrence_key"]["span_content_digest_full"],
            blake3::Hash::from(*parts.span_content_digest_full.read_full_digest_bytes())
                .to_hex()
                .to_string()
        );
        let parent = input
            .input_edge_records
            .iter()
            .find(|edge| {
                edge.edge_relation_kind_value == HldRelationKindValue::Contains
                    && edge.edge_resolution_value
                        == HldEdgeResolutionValue::Extracted {
                            target: node.node_target_reference.clone(),
                        }
            })
            .unwrap();
        assert!(
            matches!(
                parent.edge_source_reference,
                HldTargetIdentityValue::FilePublicArea(_)
            ),
            "self type and trait are not lexical containers"
        );
    }
}

/// REQ-RUST-034.0, REQ-RUST-035.0, REQ-RUST-045.0
#[test]
fn test_default_new_call_roundtrip() {
    let (_directory, root, database, input) = run_original_graph_fixture();
    let spans = read_published_stage_rows(&database, "span-reference-text-table");
    let default = locate_original_member_node(&input, &spans, "default");
    let new = locate_original_member_node(&input, &spans, "new");
    let calls = input
        .input_edge_records
        .iter()
        .filter(|edge| {
            edge.edge_source_reference == default.node_target_reference
                && edge.edge_relation_kind_value == HldRelationKindValue::Calls
        })
        .collect::<Vec<_>>();
    assert_eq!(
        calls.len(),
        1,
        "exactly one call site in pinned Default::default"
    );
    let call = calls[0];
    assert_eq!(
        call.edge_resolution_value,
        HldEdgeResolutionValue::Extracted {
            target: new.node_target_reference.clone()
        }
    );
    assert_eq!(
        call.edge_evidence_row_key.evidence_source_occurrence_key,
        default.node_target_reference
    );
    assert_eq!(
        call.edge_evidence_row_key.evidence_target_reference_value,
        Some(new.node_target_reference.clone())
    );
    let site = call
        .edge_evidence_row_key
        .evidence_site_occurrence_key
        .read_semantic_occurrence_parts();
    let (start, end) = site.span_byte_range_value.read_source_range_pair();
    let bytes = fs::read(root.join(site.span_source_file_path)).unwrap();
    assert_eq!(&bytes[start as usize..end as usize], b"Buffer::new()");
    assert_eq!(
        site.span_content_digest_full.read_full_digest_bytes(),
        blake3::hash(b"Buffer::new()").as_bytes()
    );
    let body_sites = read_published_stage_rows(&database, "body-site-reference-table");
    assert_eq!(
        body_sites
            .iter()
            .filter(|row| row["span_byte_start_offset"] == start
                && row["span_byte_end_offset"] == end
                && row["span_expansion_provenance_chain"] == json!([])
                && row["site_syntax_source_text"] == "Buffer::new()")
            .count(),
        1
    );
    let prefixes = read_published_stage_rows(&database, "path-resolution-site-records");
    assert!(
        prefixes
            .iter()
            .any(|row| row["site_source_range_start"] == start
                && row["path_resolution_state_kind"] == "resolved"
                && row["site_path_context_kind"] == "body_call_type_prefix"),
        "call must use existing namespace type-prefix facts"
    );
}

/// REQ-RUST-042.0
#[test]
fn test_format_sealed_write_call() {
    let (_directory, root, database, input) = run_original_graph_fixture();
    let spans = read_published_stage_rows(&database, "span-reference-text-table");
    let format = locate_original_member_node(&input, &spans, "format");
    let write = locate_original_member_node(&input, &spans, "write");
    let source = fs::read(root.join("src/lib.rs")).unwrap();
    let expected_site = b"i.write(unsafe { &mut *buf_ptr })";
    let calls = input
        .input_edge_records
        .iter()
        .filter(|edge| {
            let site = edge
                .edge_evidence_row_key
                .evidence_site_occurrence_key
                .read_semantic_occurrence_parts();
            let (start, end) = site.span_byte_range_value.read_source_range_pair();
            edge.edge_source_reference == format.node_target_reference
                && edge.edge_relation_kind_value == HldRelationKindValue::Calls
                && source[start as usize..end as usize] == expected_site[..]
        })
        .collect::<Vec<_>>();
    assert_eq!(
        calls.len(),
        1,
        "exactly one stored call for the i.write site in Buffer::format"
    );
    let call = calls[0];
    assert_eq!(
        call.edge_resolution_value,
        HldEdgeResolutionValue::Extracted {
            target: write.node_target_reference.clone()
        },
        "the generic receiver call must target the trait method private::Sealed::write"
    );
    let dispatch = call.edge_dispatch_metadata_text.as_deref().unwrap_or("");
    assert!(
        dispatch.contains("Integer") && dispatch.contains("Sealed"),
        "dispatch metadata must carry the bound chain, got {dispatch:?}"
    );
    let metadata: Value = serde_json::from_str(dispatch).unwrap();
    assert_eq!(
        metadata["dispatch_kind"], "generic_bound_trait_method",
        "the edge says how its target was chosen"
    );
    assert_eq!(
        metadata["bound_dispatch_evidence"],
        json!({"bound_chain":["I", "Integer", "private::Sealed"],"missing_source_traits":["Copy"]}),
        "sysroot source is excluded in this run, so Copy is recorded as not inspected"
    );
    let write_targets = input
        .input_edge_records
        .iter()
        .filter(|edge| {
            edge.edge_source_reference == format.node_target_reference
                && edge.edge_relation_kind_value == HldRelationKindValue::Calls
                && matches!(
                    edge.edge_resolution_value,
                    HldEdgeResolutionValue::Extracted { .. }
                )
        })
        .count();
    assert_eq!(
        write_targets, 1,
        "Buffer::format has one resolved call: no fan-out to implementations"
    );
}

/// REQ-RUST-042.0
#[test]
fn test_generic_bound_rows_written() {
    let (_directory, _root, database, _input) = run_original_graph_fixture();
    let bounds = read_published_stage_rows(&database, "field-signature-detail-table")
        .into_iter()
        .filter(|row| {
            row["site_syntax_kind_value"] == "TYPE_BOUND"
                && row["span_expansion_provenance_chain"] == json!([])
        })
        .collect::<Vec<_>>();
    for (text, subject, name) in [
        ("Integer", "type_parameter", json!("I")),
        ("private::Sealed", "supertrait", Value::Null),
        ("Copy", "supertrait", Value::Null),
    ] {
        let found = bounds
            .iter()
            .filter(|row| {
                let bound = &row["bound_syntax_evidence"];
                bound["trait_type_source"]["type_source_text"] == text
                    && bound["bound_subject_kind"] == subject
                    && bound["bound_subject_name_value"] == name
                    && bound["bound_form_kind"] == "trait"
                    && bound["trait_type_source"]["type_simple_path_flag"] == true
            })
            .count();
        assert_eq!(found, 1, "one original bound row for {text} as {subject}");
    }
}

/// REQ-RUST-033.0 / REQ-RUST-035.0 / REQ-RUST-043.0
#[test]
fn test_impl_header_row_contents() {
    let directory = tempfile::tempdir().unwrap();
    let root = prepare_pinned_itoa_workspace(directory.path());
    let database = directory.path().join("impl-headers.db");
    let output = run_itoa_mapping_command(
        &root,
        &database,
        &["--sysroot-source-scope-kind", "excluded"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let spans = read_published_stage_rows(&database, "span-reference-text-table");
    let active_impls = spans
        .iter()
        .filter(|span| {
            span["span_syntax_kind_value"] == "IMPL"
                && span["span_configuration_active_flag"] == true
        })
        .collect::<Vec<_>>();
    assert!(
        !active_impls.is_empty(),
        "pinned source must reach impl extraction"
    );
    let headers = read_published_stage_rows(&database, "impl-header-index-table");
    assert_eq!(
        headers.len(),
        active_impls.len(),
        "one header per active source impl, including expansions"
    );
    for span in &active_impls {
        let rows = headers
            .iter()
            .filter(|header| {
                header["impl_source_occurrence_key"] == span["span_semantic_occurrence_key"]
            })
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 1, "missing or duplicate impl header: {span}");
        let header = rows[0];
        let occurrence = &header["impl_semantic_occurrence_key"];
        assert_eq!(
            occurrence["span_byte_range_value"],
            json!([span["span_byte_start_offset"], span["span_byte_end_offset"]])
        );
        assert_eq!(
            occurrence["span_content_digest_full"],
            span["span_content_digest_full"]
        );
        assert_eq!(
            occurrence["span_expansion_provenance_chain"],
            span["span_expansion_provenance_chain"]
        );
    }
    let default_span = active_impls
        .iter()
        .find(|span| {
            span["span_expansion_provenance_chain"] == json!([])
                && span["span_source_code_text"]
                    .as_str()
                    .unwrap()
                    .starts_with("impl Default for Buffer")
        })
        .unwrap();
    let default_header = headers
        .iter()
        .find(|header| {
            header["impl_source_occurrence_key"] == default_span["span_semantic_occurrence_key"]
        })
        .unwrap();
    let buffer_span = spans
        .iter()
        .find(|span| {
            span["span_syntax_kind_value"] == "STRUCT"
                && span["span_declared_name_value"] == "Buffer"
        })
        .unwrap();
    let self_type = &default_header["impl_self_type_resolution"];
    assert_eq!(self_type["resolution_state_kind"], "resolved");
    assert_eq!(
        self_type["target_identity"]["target_semantic_occurrence_key"]["span_content_digest_full"],
        buffer_span["span_content_digest_full"]
    );
    let trait_type = &default_header["impl_trait_type_resolution"];
    assert_eq!(trait_type["resolution_state_kind"], "unresolved");
    assert_eq!(
        trait_type["unresolved_reason_kind"],
        "external_source_missing"
    );
    assert!(
        trait_type["target_identity"].is_null(),
        "missing core trait must not become a fabricated target"
    );
    assert_eq!(default_header["impl_function_body_ancestry_flag"], false);
    assert_eq!(
        default_header["impl_source_container"]["container_kind_value"],
        "file_area"
    );
    assert_eq!(
        default_header["impl_source_container"]["source_file_relative_path"],
        "src/lib.rs"
    );
    let members = default_header["impl_member_occurrence_rows"]
        .as_array()
        .unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0]["member_declared_name_value"], "default");
    let member_span = spans
        .iter()
        .find(|span| {
            span["span_semantic_occurrence_key"] == members[0]["member_source_occurrence_key"]
        })
        .unwrap();
    let bytes = fs::read(root.join("src/lib.rs")).unwrap();
    let start = member_span["span_byte_start_offset"].as_u64().unwrap() as usize;
    let end = member_span["span_byte_end_offset"].as_u64().unwrap() as usize;
    assert_eq!(
        member_span["span_source_code_text"],
        std::str::from_utf8(&bytes[start..end]).unwrap()
    );
    assert!(
        member_span["span_source_code_text"]
            .as_str()
            .unwrap()
            .contains("Buffer::new()")
    );
}

fn target_file_location_value(identity: &Value) -> Option<(Value, String)> {
    match identity["kind"].as_str()? {
        "FilePublicArea" => Some((
            identity["unit_compilation_identity"].clone(),
            identity["file_source_relative_path"].as_str()?.to_owned(),
        )),
        "Definition" => Some((
            identity["occurrence_identity"]["unit_compilation_identity"].clone(),
            identity["occurrence_identity"]["source_file_path"]
                .as_str()?
                .to_owned(),
        )),
        _ => None,
    }
}

fn target_folder_location_value(identity: &Value) -> Option<(Value, String)> {
    let (unit_identity, file_path) = target_file_location_value(identity)?;
    Some((
        unit_identity,
        Path::new(&file_path)
            .parent()?
            .to_string_lossy()
            .into_owned(),
    ))
}

fn prepare_source_scope_fixture(parent: &Path, exposed_name: &str) -> PathBuf {
    let root = parent.join("source-scope-fixture");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname=\"source-scope-fixture\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
    )
    .unwrap();
    fs::write(
        root.join("src/lib.rs"),
        format!(
            "pub struct {exposed_name};\npub(crate) struct Restricted;\nfn private_helper() {{}}\nuse crate::{exposed_name} as Alias;\npub mod inline {{ pub fn nested_public() {{}} }}\nfn body_owner() {{ struct BodyLocal; }}\n#[cfg(any())] pub struct Disabled;\nmod child;\nmod empty;\ninclude!(\"included.rs\");\n"
        ),
    )
    .unwrap();
    fs::write(
        root.join("src/child.rs"),
        "pub struct Child;\nfn child_private() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("src/empty.rs"),
        "// reached source file with no items\n",
    )
    .unwrap();
    fs::write(root.join("src/included.rs"), "pub struct MacroIncluded;\n").unwrap();
    let lock = Command::new(env!("CARGO"))
        .current_dir(&root)
        .args(["generate-lockfile", "--offline"])
        .output()
        .unwrap();
    assert!(
        lock.status.success(),
        "{}",
        String::from_utf8_lossy(&lock.stderr)
    );
    root
}

fn run_source_scope_mapping(root: &Path, database: &Path, final_stage: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_parseltongue"))
        .args([
            "map-workspace-full-run",
            "--workspace-root-path-value",
            root.to_str().unwrap(),
            "--workspace-store-file-path",
            database.to_str().unwrap(),
            "--worker-thread-count-value",
            "2",
            "--sysroot-source-scope-kind",
            "excluded",
            "--mapping-final-stage-kind",
            final_stage,
        ])
        .output()
        .unwrap()
}

/// REQ-RUST-033.0
/// REQ-RUST-038.0
#[test]
fn test_itoa_pipeline_reads_published() {
    let directory = tempfile::tempdir().unwrap();
    let root = prepare_pinned_itoa_workspace(directory.path());
    let database = directory.path().join("itoa-store.db");
    let output = run_itoa_mapping_command(
        &root,
        &database,
        &["--sysroot-source-scope-kind", "excluded"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let banner: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        banner["stage_implementation_status_values"]["name-resolution"],
        "complete"
    );
    let config = read_published_stage_rows(&database, "workspace-config-state-records");
    assert_eq!(config.len(), 1);
    assert_eq!(config[0]["workspace_sysroot_scope_kind"], "excluded");
    let units = read_published_stage_rows(&database, "crate-dependency-graph-table");
    assert!(
        units
            .iter()
            .all(|unit| unit["unit_sysroot_context_flag"] != true),
        "a sysroot unit was mapped"
    );
    let mapped = units
        .iter()
        .filter(|unit| unit["unit_mapped_target_flag"] == true)
        .collect::<Vec<_>>();
    assert_eq!(mapped.len(), 1, "one mapped unit: the itoa library");
    assert_eq!(mapped[0]["unit_cargo_target_kind"], "lib");
    assert_eq!(mapped[0]["unit_rust_edition_value"], "2021");
    let spans = read_published_stage_rows(&database, "span-reference-text-table");
    assert!(!spans.is_empty(), "syntax stage published no span rows");
    let paths = read_published_stage_rows(&database, "path-resolution-site-records");
    assert!(
        !paths.is_empty(),
        "name resolution published no path-site rows"
    );
}

/// REQ-RUST-040.0
#[test]
fn test_itoa_top_level_containment() {
    let directory = tempfile::tempdir().unwrap();
    let root = prepare_pinned_itoa_workspace(directory.path());
    let database = directory.path().join("itoa-store.db");
    let output = run_itoa_mapping_command(
        &root,
        &database,
        &[
            "--sysroot-source-scope-kind",
            "excluded",
            "--mapping-final-stage-kind",
            "original-top-level-containment",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let config = read_published_stage_rows(&database, "workspace-config-state-records");
    assert_eq!(config.len(), 1);
    assert_eq!(config[0]["workspace_sysroot_scope_kind"], "excluded");
    assert_eq!(
        config[0]["workspace_mapping_scope_kind"],
        "original-top-level-containment"
    );
    let units = read_published_stage_rows(&database, "crate-dependency-graph-table");
    let unit = units
        .iter()
        .find(|row| row["unit_mapped_target_flag"] == true)
        .unwrap();
    let unit_identity_key = unit["unit_compilation_identity_key"].as_str().unwrap();
    let unit_identity = canonical_unit_identity_from_root_scope(&database, unit);
    let source_root = root.canonicalize().unwrap();
    let source_path = source_root.join("src/lib.rs");
    let source_path_text = source_path.to_string_lossy().into_owned();
    let source_bytes = fs::read(&source_path).unwrap();
    let files = read_published_stage_rows(&database, "file-source-text-table");
    let file = files
        .iter()
        .find(|row| row["file_source_absolute_path"] == source_path_text)
        .unwrap();
    assert_eq!(
        file["file_content_digest_full"],
        blake3::hash(&source_bytes).to_hex().to_string()
    );

    let spans = read_published_stage_rows(&database, "span-reference-text-table");
    let buffer_span = spans
        .iter()
        .find(|row| {
            row["unit_compilation_identity_key"] == unit_identity_key
                && row["file_source_absolute_path"] == source_path_text
                && row["span_declared_name_value"] == "Buffer"
                && row["span_syntax_kind_value"] == "STRUCT"
        })
        .unwrap();
    assert_eq!(buffer_span["span_configuration_active_flag"], true);
    assert_eq!(buffer_span["span_visibility_source_text"], "pub");
    assert_eq!(buffer_span["span_expansion_provenance_chain"], json!([]));
    let start = buffer_span["span_byte_start_offset"].as_u64().unwrap() as usize;
    let end = buffer_span["span_byte_end_offset"].as_u64().unwrap() as usize;
    let buffer_source = std::str::from_utf8(&source_bytes[start..end]).unwrap();
    assert!(buffer_source.contains("pub struct Buffer"));
    assert!(buffer_source.trim_end().ends_with('}'));
    assert_eq!(buffer_span["span_source_code_text"], buffer_source);
    assert_eq!(
        buffer_span["span_content_digest_full"],
        blake3::hash(&source_bytes[start..end]).to_hex().to_string()
    );
    let ownership_ranges = read_published_stage_rows(&database, "span-byte-ownership-ranges");
    assert!(
        ownership_ranges.iter().any(|row| {
            row["unit_compilation_identity_key"] == unit_identity_key
                && row["file_source_absolute_path"] == source_path_text
                && row["span_byte_start_offset"] == buffer_span["span_byte_start_offset"]
                && row["span_byte_end_offset"] == buffer_span["span_byte_end_offset"]
                && row["span_gap_classification_kind"] == "item"
        }),
        "Buffer's original file range is not top-level owned"
    );

    let relative_path = source_path
        .strip_prefix(&source_root)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/");
    let file_area_identity = json!({
        "kind": "FilePublicArea",
        "unit_compilation_identity": unit_identity,
        "file_source_relative_path": relative_path
    });
    let buffer_identity = json!({
        "kind": "Definition",
        "occurrence_identity": {
            "unit_compilation_identity": unit_identity,
            "source_file_path": relative_path,
            "byte_range": [start, end],
            "content_digest_full": blake3::hash(&source_bytes[start..end]).as_bytes().to_vec(),
            "expansion_provenance_chain": []
        }
    });
    let namespace_buffer = read_published_stage_rows(&database, "syntax-namespace-input-items")
        .into_iter()
        .find(|row| {
            row["binding_resolved_name_text"] == "Buffer"
                && row["binding_target_identity_key"]["target_identity_variant_kind"]
                    == "definition"
                && row["binding_target_identity_key"]["target_semantic_occurrence_key"]["span_byte_range_value"]
                    == json!([start, end])
        })
        .unwrap();
    let namespace_occurrence =
        &namespace_buffer["binding_target_identity_key"]["target_semantic_occurrence_key"];
    let namespace_digest = namespace_occurrence["span_content_digest_full"]
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect::<Vec<_>>();
    let namespace_definition_identity = json!({
        "kind": "Definition",
        "occurrence_identity": {
            "unit_compilation_identity": unit_identity,
            "source_file_path": namespace_occurrence["span_source_file_path"],
            "byte_range": namespace_occurrence["span_byte_range_value"],
            "content_digest_full": namespace_digest,
            "expansion_provenance_chain": []
        }
    });
    assert_eq!(namespace_definition_identity, buffer_identity);
    let fullgraph_nodes = read_published_stage_rows(&database, "item-effective-visibility-records");
    let fullgraph_edges = read_published_stage_rows(&database, "edge-evidence-record-table");
    let interface_nodes = read_published_stage_rows(&database, "hld-current-interface-nodes");
    let dependency_edges = read_published_stage_rows(&database, "hld-current-dependency-edges");
    let evidence_paths = read_published_stage_rows(&database, "hld-edge-evidence-paths");
    let file_rollups = read_published_stage_rows(&database, "hld-file-level-rollup");
    let folder_rollups = read_published_stage_rows(&database, "hld-folder-level-rollup");
    assert!(
        !interface_nodes.is_empty(),
        "missing required scoped HLD interface nodes; fullgraph nodes={}, edges={}",
        fullgraph_nodes.len(),
        fullgraph_edges.len()
    );

    let file_area_node = interface_nodes
        .iter()
        .find(|row| row["node_target_identity"] == file_area_identity)
        .unwrap();
    let buffer_node = interface_nodes
        .iter()
        .find(|row| row["node_target_identity"] == buffer_identity)
        .unwrap();
    let fullgraph_buffer_node = fullgraph_nodes
        .iter()
        .find(|row| row["node_target_identity"] == buffer_identity)
        .unwrap();
    assert_eq!(
        buffer_node["node_target_identity"],
        fullgraph_buffer_node["node_target_identity"]
    );
    assert_eq!(
        fullgraph_buffer_node["node_effective_visibility_kind"],
        "Public"
    );
    assert_eq!(
        fullgraph_buffer_node["node_reachable_outside_crate_flag"],
        true
    );

    let direct_contains_edges = dependency_edges
        .iter()
        .filter(|row| {
            row["edge_key"]["source_target_identity"] == file_area_node["node_target_identity"]
                && row["edge_key"]["target_identity"] == buffer_node["node_target_identity"]
                && row["edge_key"]["relation_kind"] == "Contains"
                && row["edge_key"]["via_kind"] == "Direct"
        })
        .collect::<Vec<_>>();
    assert_eq!(direct_contains_edges.len(), 1);
    let edge = direct_contains_edges[0];
    let fullgraph_edge = fullgraph_edges
        .iter()
        .find(|row| {
            row["edge_source_target_identity"] == file_area_identity
                && row["edge_relation_kind"] == "Contains"
                && row["edge_evidence_identity"]["source_target_identity"] == file_area_identity
                && row["edge_evidence_identity"]["relation_kind"] == "Contains"
                && row["edge_evidence_identity"]["target_identity"] == buffer_identity
                && row["edge_evidence_identity"]["site_occurrence_identity"]
                    == buffer_identity["occurrence_identity"]
        })
        .unwrap();
    let evidence = evidence_paths
        .iter()
        .find(|row| row["edge_key"] == edge["edge_key"])
        .unwrap();
    assert_eq!(
        evidence["ordered_endpoint_path"],
        json!([file_area_identity, buffer_identity])
    );
    assert!(
        evidence["ordered_evidence_row_keys"]
            .as_array()
            .unwrap()
            .iter()
            .any(|proof| {
                proof["source_target_identity"] == file_area_identity
                    && proof["target_identity"] == buffer_identity
                    && proof["relation_kind"] == "Contains"
                    && proof["site_occurrence_identity"] == buffer_identity["occurrence_identity"]
                    && proof["source_target_identity"]
                        == fullgraph_edge["edge_evidence_identity"]["source_target_identity"]
                    && proof["target_identity"]
                        == fullgraph_edge["edge_evidence_identity"]["target_identity"]
                    && proof["relation_kind"]
                        == fullgraph_edge["edge_evidence_identity"]["relation_kind"]
                    && proof["site_occurrence_identity"]
                        == fullgraph_edge["edge_evidence_identity"]["site_occurrence_identity"]
            })
    );

    let file_rollup = file_rollups
        .iter()
        .find(|row| {
            row["source_endpoint_bucket"]["kind"] == "SourceFile"
                && row["source_endpoint_bucket"]["unit_compilation_identity"] == unit_identity
                && row["source_endpoint_bucket"]["file_source_relative_path"] == relative_path
                && row["target_endpoint_bucket"]["unit_compilation_identity"] == unit_identity
                && row["target_endpoint_bucket"]["file_source_relative_path"] == relative_path
                && row["relation_kind"] == "Contains"
                && row["via_kind"] == "Direct"
        })
        .unwrap();
    let file_bucket_edges = dependency_edges
        .iter()
        .filter(|row| {
            let edge_key = &row["edge_key"];
            edge_key["relation_kind"] == "Contains"
                && edge_key["via_kind"] == "Direct"
                && target_file_location_value(&edge_key["source_target_identity"])
                    == Some((unit_identity.clone(), relative_path.clone()))
                && target_file_location_value(&edge_key["target_identity"])
                    == Some((unit_identity.clone(), relative_path.clone()))
        })
        .collect::<Vec<_>>();
    assert!(file_bucket_edges.len() > 1);
    assert!(
        file_bucket_edges
            .iter()
            .any(|row| row["edge_key"] == edge["edge_key"])
    );
    assert_eq!(
        file_rollup["edge_row_count"].as_u64(),
        Some(file_bucket_edges.len() as u64)
    );
    let folder_rollup = folder_rollups
        .iter()
        .find(|row| {
            row["source_folder_bucket"]["kind"] == "SourceFolder"
                && row["source_folder_bucket"]["unit_compilation_identity"] == unit_identity
                && row["source_folder_bucket"]["folder_source_relative_path"] == "src"
                && row["target_folder_bucket"]["unit_compilation_identity"] == unit_identity
                && row["target_folder_bucket"]["folder_source_relative_path"] == "src"
                && row["relation_kind"] == "Contains"
                && row["via_kind"] == "Direct"
        })
        .unwrap();
    let folder_bucket_edges = dependency_edges
        .iter()
        .filter(|row| {
            let edge_key = &row["edge_key"];
            edge_key["relation_kind"] == "Contains"
                && edge_key["via_kind"] == "Direct"
                && target_folder_location_value(&edge_key["source_target_identity"])
                    == Some((unit_identity.clone(), "src".into()))
                && target_folder_location_value(&edge_key["target_identity"])
                    == Some((unit_identity.clone(), "src".into()))
        })
        .collect::<Vec<_>>();
    assert!(
        folder_bucket_edges
            .iter()
            .any(|row| row["edge_key"] == edge["edge_key"])
    );
    assert_eq!(
        folder_rollup["edge_row_count"].as_u64(),
        Some(folder_bucket_edges.len() as u64)
    );
    let markers = read_published_stage_rows(&database, "stage-run-generation-records");
    let hld_marker = markers
        .iter()
        .find(|row| {
            row.get("input_snapshot_revision").is_some()
                && row.get("unknown_reachability_diagnostic_count").is_some()
        })
        .unwrap();
    assert_eq!(
        hld_marker["mapping_scope_kind"],
        "original-top-level-containment"
    );
    assert_eq!(
        hld_marker["source_coverage_kind"],
        "original-file-top-level-only"
    );
    assert!(
        hld_marker["unknown_reachability_diagnostic_count"]
            .as_u64()
            .is_some()
    );
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .filter(|line| {
            line.starts_with("stage=original-top-level-")
                || line.starts_with("duckdb-source-projection-memory-limit=")
        })
        .for_each(|line| eprintln!("{line}"));
    eprintln!(
        "observed-hld-rows interface_nodes={} dependency_edges={} evidence_paths={} file_rollups={} folder_rollups={}",
        interface_nodes.len(),
        dependency_edges.len(),
        evidence_paths.len(),
        file_rollups.len(),
        folder_rollups.len()
    );
}

fn verify_source_fixture_compiles(root: &Path) {
    let output = Command::new(env!("CARGO"))
        .current_dir(root)
        .args(["check", "--offline", "--locked", "--lib"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// REQ-RUST-040.0
#[test]
fn test_source_visibility_scope_equivalence() {
    let directory = tempfile::tempdir().unwrap();
    let root = prepare_source_scope_fixture(directory.path(), "Exposed");
    fs::write(
        root.join("src/lib.rs"),
        "mod child;\npub(crate) struct RootRestricted;\n",
    )
    .unwrap();
    fs::write(
        root.join("src/child.rs"),
        concat!(
            "pub struct Public;\n",
            "struct Private;\n",
            "pub(self) struct SelfOnly;\n",
            "pub(in self) struct InSelf;\n",
            "pub(in crate::child) struct SameModule;\n",
            "pub(crate) struct CrateScope;\n",
            "pub(in crate) struct CrateEquivalent;\n",
            "pub(super) struct ParentScope;\n",
            "pub /* comment */ (crate) struct CommentedCrate;\n",
        ),
    )
    .unwrap();
    verify_source_fixture_compiles(&root);
    let database = directory.path().join("visibility.db");
    let output = run_source_scope_mapping(&root, &database, "original-top-level-containment");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let nodes = read_published_stage_rows(&database, "item-effective-visibility-records");
    let selected = read_published_stage_rows(&database, "hld-current-interface-nodes");
    for (name, visibility, included) in [
        ("SelfOnly", "PrivateEquivalent", false),
        ("InSelf", "PrivateEquivalent", false),
        ("SameModule", "PrivateEquivalent", false),
        ("Private", "PrivateEquivalent", false),
        ("RootRestricted", "PrivateEquivalent", false),
        ("CrateScope", "RestrictedNonPrivate", true),
        ("CrateEquivalent", "RestrictedNonPrivate", true),
        ("ParentScope", "RestrictedNonPrivate", true),
        ("CommentedCrate", "RestrictedNonPrivate", true),
        ("Public", "Public", true),
    ] {
        let matches = nodes
            .iter()
            .filter(|node| {
                node["node_qualified_path_text"]
                    .as_str()
                    .is_some_and(|path| path.ends_with(&format!("::{name}")))
            })
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1, "full graph must retain {name}");
        let node = matches[0];
        assert_eq!(
            node["node_effective_visibility_kind"], visibility,
            "visibility of {name}"
        );
        assert_eq!(
            selected
                .iter()
                .any(|row| row["node_target_identity"] == node["node_target_identity"]),
            included,
            "interface selection of {name}"
        );
        if visibility != "Public" {
            assert_eq!(node["node_reachable_outside_crate_flag"], false);
        }
    }
}

/// REQ-RUST-040.0
#[test]
fn test_source_module_owner_paths() {
    let directory = tempfile::tempdir().unwrap();
    let root = prepare_source_scope_fixture(directory.path(), "Shared");
    fs::write(
        root.join("src/lib.rs"),
        "pub struct Shared;\nmod left;\n#[path=\"different.rs\"] mod right;\n",
    )
    .unwrap();
    for file in ["src/left.rs", "src/different.rs"] {
        fs::write(root.join(file), "pub struct Shared;\n").unwrap();
    }
    verify_source_fixture_compiles(&root);
    let database = directory.path().join("ownership.db");
    let output = run_source_scope_mapping(&root, &database, "original-top-level-containment");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let nodes = read_published_stage_rows(&database, "item-effective-visibility-records");
    let unit = read_published_stage_rows(&database, "crate-dependency-graph-table")
        .into_iter()
        .find(|row| row["unit_mapped_target_flag"] == true)
        .unwrap();
    let package = unit["unit_cargo_package_identifier"].as_str().unwrap();
    let mut identities = std::collections::BTreeSet::new();
    for (file, suffix, owner) in [
        ("src/lib.rs", "Shared", "CrateRoot"),
        ("src/left.rs", "left::Shared", "Module"),
        ("src/different.rs", "right::Shared", "Module"),
    ] {
        let candidates = nodes
            .iter()
            .filter(|node| {
                node["node_source_relative_path"] == file
                    && node["node_item_kind_value"] == "Struct"
            })
            .collect::<Vec<_>>();
        assert_eq!(candidates.len(), 1, "one struct in {file}");
        let node = candidates[0];
        assert_eq!(node["node_owner_kind_value"], owner, "owner of {file}");
        assert_eq!(
            node["node_qualified_path_text"],
            format!("{package}::{suffix}")
        );
        assert!(
            identities.insert(node["node_target_identity"].to_string()),
            "same-name declarations cannot share identity"
        );
    }
    assert_eq!(identities.len(), 3);
}

/// REQ-RUST-040.0
#[test]
fn test_source_fixture_scope_rules() {
    let directory = tempfile::tempdir().unwrap();
    let root = prepare_source_scope_fixture(directory.path(), "Exposed");
    let database = directory.path().join("source-scope.db");
    let output = run_source_scope_mapping(&root, &database, "original-top-level-containment");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let root_path = root.canonicalize().unwrap();
    let source_rows = read_published_stage_rows(&database, "file-source-text-table");
    [
        "src/lib.rs",
        "src/child.rs",
        "src/empty.rs",
        "src/included.rs",
    ]
    .iter()
    .map(|path| root_path.join(path).to_string_lossy().into_owned())
    .for_each(|expected| {
        assert!(
            source_rows
                .iter()
                .any(|row| row["file_source_absolute_path"] == expected)
        );
    });
    let child_path = root_path
        .join("src/child.rs")
        .to_string_lossy()
        .into_owned();
    let empty_path = root_path
        .join("src/empty.rs")
        .to_string_lossy()
        .into_owned();
    let included_path = root_path
        .join("src/included.rs")
        .to_string_lossy()
        .into_owned();
    let closure_requests = read_published_stage_rows(&database, "module-source-closure-requests");
    let find_closure_source_request = |path: &str| {
        closure_requests.iter().find(|request| {
            request["source_request_path_candidates"]
                .as_array()
                .is_some_and(|candidates| {
                    candidates
                        .iter()
                        .any(|candidate| candidate.as_str() == Some(path))
                })
        })
    };
    assert_eq!(
        find_closure_source_request(&child_path).unwrap()["source_request_origin_kind"],
        "module_declaration"
    );
    assert_eq!(
        find_closure_source_request(&included_path).unwrap()["source_request_origin_kind"],
        "macro_include"
    );
    let syntax_requests = read_published_stage_rows(&database, "syntax-file-work-requests");
    [child_path.as_str(), empty_path.as_str()]
        .iter()
        .for_each(|path| {
            let request = syntax_requests
                .iter()
                .find(|request| request["file_source_absolute_path"] == *path)
                .unwrap();
            assert_eq!(request["file_expansion_provenance_chain"], json!([]));
        });
    let included_request = syntax_requests
        .iter()
        .find(|request| request["file_source_absolute_path"] == included_path)
        .unwrap();
    assert!(
        included_request["file_expansion_provenance_chain"]
            .as_array()
            .is_some_and(|chain| !chain.is_empty())
    );
    let spans = read_published_stage_rows(&database, "span-reference-text-table");
    let exposed_span = spans
        .iter()
        .find(|row| {
            row["file_source_absolute_path"]
                == root_path.join("src/lib.rs").to_string_lossy().as_ref()
                && row["span_declared_name_value"] == "Exposed"
                && row["span_syntax_kind_value"] == "STRUCT"
        })
        .unwrap();
    assert_eq!(exposed_span["span_configuration_active_flag"], true);
    assert!(spans.iter().any(|row| {
        row["span_declared_name_value"] == "Disabled"
            && row["span_configuration_active_flag"] == false
    }));

    let fullgraph_nodes = read_published_stage_rows(&database, "item-effective-visibility-records");
    assert!(
        !fullgraph_nodes.is_empty(),
        "scoped source graph did not publish nodes from the renamed fixture"
    );
    let hld_nodes = read_published_stage_rows(&database, "hld-current-interface-nodes");
    let unit = read_published_stage_rows(&database, "crate-dependency-graph-table")
        .into_iter()
        .find(|row| row["unit_mapped_target_flag"] == true)
        .unwrap();
    let unit_identity = canonical_unit_identity_from_root_scope(&database, &unit);
    ["src/lib.rs", "src/child.rs", "src/empty.rs"]
        .iter()
        .map(|path| {
            json!({
                "kind": "FilePublicArea",
                "unit_compilation_identity": unit_identity,
                "file_source_relative_path": path
            })
        })
        .for_each(|identity| {
            assert!(
                hld_nodes
                    .iter()
                    .any(|row| row["node_target_identity"] == identity),
                "missing file area {identity}; available areas={:?}, requests={:?}",
                hld_nodes
                    .iter()
                    .filter(|row| row["node_target_identity"]["kind"] == "FilePublicArea")
                    .map(|row| row["node_target_identity"].clone())
                    .collect::<Vec<_>>(),
                read_published_stage_rows(&database, "syntax-file-work-requests")
            );
        });

    let excluded_names = ["BodyLocal", "nested_public", "Disabled", "MacroIncluded"];
    assert!(fullgraph_nodes.iter().all(|node| {
        !excluded_names.iter().any(|name| {
            node["node_qualified_path_text"]
                .as_str()
                .is_some_and(|path| path.ends_with(name))
        })
    }));
    assert!(
        fullgraph_nodes
            .iter()
            .all(|node| { node["node_source_relative_path"] != "src/included.rs" })
    );
    let private_function = fullgraph_nodes
        .iter()
        .find(|node| {
            node["node_qualified_path_text"]
                .as_str()
                .is_some_and(|path| path.ends_with("private_helper"))
        })
        .unwrap();
    assert!(
        !hld_nodes.iter().any(|node| {
            node["node_target_identity"] == private_function["node_target_identity"]
        })
    );
    let private_use = fullgraph_nodes
        .iter()
        .find(|node| {
            node["node_item_kind_value"]
                .as_str()
                .is_some_and(|kind| matches!(kind, "UseStatement" | "use_statement"))
                && node["node_source_relative_path"] == "src/lib.rs"
        })
        .unwrap();
    assert!(
        hld_nodes
            .iter()
            .any(|node| { node["node_target_identity"] == private_use["node_target_identity"] })
    );
    let child_public = fullgraph_nodes
        .iter()
        .find(|node| {
            node["node_qualified_path_text"]
                .as_str()
                .is_some_and(|path| path.ends_with("Child"))
        })
        .unwrap();
    assert_eq!(
        child_public["node_reachable_outside_crate_flag"],
        Value::Null
    );
}

/// REQ-RUST-040.0
#[test]
fn test_source_mutation_identity_changes() {
    let directory = tempfile::tempdir().unwrap();
    let before_root = prepare_source_scope_fixture(directory.path(), "Exposed");
    let before_database = directory.path().join("before.db");
    let before_output = run_source_scope_mapping(
        &before_root,
        &before_database,
        "original-top-level-containment",
    );
    assert!(before_output.status.success());
    let before_nodes =
        read_published_stage_rows(&before_database, "item-effective-visibility-records");
    let before_edges = read_published_stage_rows(&before_database, "edge-evidence-record-table");
    assert!(
        !before_nodes.is_empty(),
        "source-derived node rows are missing"
    );
    assert!(
        !before_edges.is_empty(),
        "source-derived contains edges are missing"
    );
    let before_structs = before_nodes
        .iter()
        .filter(|row| row["node_item_kind_value"] == "Struct")
        .map(|row| serde_json::to_string(&row["node_target_identity"]).unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    let before_targets = before_edges
        .iter()
        .filter(|row| row["edge_relation_kind"] == "Contains")
        .map(|row| {
            serde_json::to_string(&row["edge_evidence_identity"]["target_identity"]).unwrap()
        })
        .collect::<std::collections::BTreeSet<_>>();

    let after_root = prepare_source_scope_fixture(directory.path(), "Renamed");
    let after_database = directory.path().join("after.db");
    let after_output = run_source_scope_mapping(
        &after_root,
        &after_database,
        "original-top-level-containment",
    );
    assert!(after_output.status.success());
    let after_nodes =
        read_published_stage_rows(&after_database, "item-effective-visibility-records");
    let after_edges = read_published_stage_rows(&after_database, "edge-evidence-record-table");
    let after_structs = after_nodes
        .iter()
        .filter(|row| row["node_item_kind_value"] == "Struct")
        .map(|row| serde_json::to_string(&row["node_target_identity"]).unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    let after_targets = after_edges
        .iter()
        .filter(|row| row["edge_relation_kind"] == "Contains")
        .map(|row| {
            serde_json::to_string(&row["edge_evidence_identity"]["target_identity"]).unwrap()
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_ne!(before_structs, after_structs);
    assert_ne!(before_targets, after_targets);
    let after_nodes = after_nodes
        .iter()
        .map(|row| serde_json::to_string(&row["node_target_identity"]).unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        after_targets
            .iter()
            .all(|target| after_nodes.contains(target))
    );
}

/// REQ-RUST-038.0
#[test]
fn test_sysroot_scope_excluded_units() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname=\"fixture\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
    )
    .unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "use core::ops::Range; pub struct Local; pub fn take(_: Range<u8>, _: Local) {}",
    )
    .unwrap();
    assert!(
        Command::new(env!("CARGO"))
            .current_dir(&root)
            .args(["generate-lockfile", "--offline"])
            .output()
            .unwrap()
            .status
            .success()
    );
    let database = directory.path().join("store.db");
    let unknown = run_itoa_mapping_command(
        &root,
        &database,
        &["--sysroot-source-scope-kind", "partial"],
    );
    assert!(!unknown.status.success());
    assert!(
        String::from_utf8_lossy(&unknown.stderr).contains("unknown sysroot source scope partial")
    );
    let output = run_itoa_mapping_command(
        &root,
        &database,
        &["--sysroot-source-scope-kind", "excluded"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let config = read_published_stage_rows(&database, "workspace-config-state-records");
    assert_eq!(config[0]["workspace_sysroot_scope_kind"], "excluded");
    let units = read_published_stage_rows(&database, "crate-dependency-graph-table");
    assert!(
        units
            .iter()
            .all(|unit| unit["unit_sysroot_context_flag"] != true)
    );
    let paths = read_published_stage_rows(&database, "path-resolution-site-records");
    let read_path_state = |text: &str| {
        paths
            .iter()
            .find(|row| row["path_site_source_text"] == text)
            .unwrap_or_else(|| panic!("no path-site row for {text}"))
    };
    assert_eq!(
        read_path_state("Local")["path_resolution_state_kind"],
        "resolved"
    );
    let external = read_path_state("core::ops::Range");
    assert_eq!(external["path_resolution_state_kind"], "unresolved");
    assert!(
        external["path_unresolved_reason_kind"]
            .as_str()
            .is_some_and(|reason| !reason.is_empty()),
        "{external}"
    );
}

/// REQ-RUST-038.1, REQ-RUST-045.0
#[test]
fn test_sysroot_reached_names_reasoned() {
    let directory = tempfile::tempdir().unwrap();
    let root = prepare_pinned_itoa_workspace(directory.path());
    let database = directory.path().join("itoa-store.db");
    let output = run_itoa_mapping_command(
        &root,
        &database,
        &["--sysroot-source-scope-kind", "excluded"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let paths = read_published_stage_rows(&database, "path-resolution-site-records");
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let sources = read_published_stage_rows(&database, "file-source-text-table")
        .into_iter()
        .map(|row| {
            (
                row["file_source_absolute_path"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
                store
                    .read_file_content_bytes(row["file_content_digest_full"].as_str().unwrap())
                    .unwrap(),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    for row in &paths {
        let source = &sources[row["site_source_absolute_path"].as_str().unwrap()];
        let start = row["site_source_range_start"].as_u64().unwrap() as usize;
        let end = row["site_source_range_end"].as_u64().unwrap() as usize;
        assert_eq!(
            std::str::from_utf8(&source[start..end]).unwrap(),
            row["path_site_source_text"].as_str().unwrap()
        );
    }
    let (body_prefixes, paths): (Vec<_>, Vec<_>) = paths
        .into_iter()
        .partition(|row| row["site_path_context_kind"] == "body_call_type_prefix");
    assert_eq!(body_prefixes.len(), 29);
    assert!(
        body_prefixes
            .iter()
            .any(|row| row["path_site_source_text"] == "Buffer"
                && row["path_resolution_state_kind"] == "resolved")
    );
    let count_sites = |text: &str, reason: Value| {
        paths
            .iter()
            .filter(|row| {
                row["path_site_source_text"]
                    .as_str()
                    .is_some_and(|source| source.split_whitespace().collect::<String>() == text)
                    && row["path_resolution_state_kind"] == "unresolved"
                    && row["path_unresolved_reason_kind"] == reason
            })
            .count()
    };
    let missing = || Value::from("external_source_missing");
    assert_eq!(count_sites("MaybeUninit<u8>", missing()), 13);
    assert_eq!(count_sites("Copy", missing()), 2);
    assert_eq!(count_sites("Default", missing()), 1);
    assert_eq!(count_sites("Clone", missing()), 1);
    assert_eq!(count_sites("Self::MAX_STR_LEN", Value::Null), 10);
    assert_eq!(count_sites("Self::Buffer", Value::Null), 5);
    assert_eq!(count_sites("i128::MAX_STR_LEN", Value::Null), 1);
    let count_state = |state: &str| {
        paths
            .iter()
            .filter(|row| row["path_resolution_state_kind"] == state)
            .count()
    };
    assert_eq!(paths.len(), 202);
    assert_eq!(count_state("resolved"), 164);
    assert_eq!(count_state("ambiguous"), 0);
    let reasoned = paths
        .iter()
        .filter(|row| row["path_unresolved_reason_kind"] == "external_source_missing")
        .count();
    assert_eq!(reasoned, 22);
}

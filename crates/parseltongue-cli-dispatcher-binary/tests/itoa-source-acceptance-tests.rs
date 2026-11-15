//! Phase 1 real-source acceptance tests on pinned itoa (SOP §1.1).
//!
//! The pinned source goes through the production CLI entry point. Tests read only rows of the
//! published generation; expected rows live in assertions here, never in production input.
#![allow(clippy::unwrap_used)]

use parseltongue_core_row_types::{
    HldEdgeInputRecord, HldEdgeResolutionValue, HldEffectiveVisibilityKind,
    HldInputCompletenessKind, HldItemKindValue, HldMaterializationInputRecord,
    HldNodeInputCategory, HldNodeInputRecord, HldOwnerKindValue, HldRelationKindValue,
    HldTargetIdentityValue, HldUnresolvedReasonKind, NameSymbolTargetKey, RunCancelFlagValue,
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
    locate_original_kind_node(input, spans, name, "FN")
}

fn locate_original_kind_node<'a>(
    input: &'a HldMaterializationInputRecord,
    spans: &[Value],
    name: &str,
    kind: &str,
) -> &'a HldNodeInputRecord {
    let spans = spans
        .iter()
        .filter(|span| {
            span["span_expansion_provenance_chain"] == json!([])
                && span["span_declared_name_value"] == name
                && span["span_syntax_kind_value"] == kind
        })
        .collect::<Vec<_>>();
    assert_eq!(spans.len(), 1, "pin must have one original {kind} {name}");
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
    // Items written by a macro expansion are covered by test_generated_item_node_counts.
    let local = input
        .input_node_records
        .iter()
        .filter(|node| {
            matches!(
                &node.node_target_reference,
                HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(key))
                    if node.node_input_category == HldNodeInputCategory::LocalItem
                        && key
                            .read_semantic_occurrence_parts()
                            .span_expansion_provenance_chain
                            .is_empty()
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(local.len(), 43, "original items, impl members included");
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

/// The `uses_type` edges of one original function, and the byte range of
/// `path` inside `signature` in src/lib.rs.
fn read_signature_type_uses(
    owner: &str,
    signature: &str,
    path: &str,
) -> (
    HldMaterializationInputRecord,
    Vec<HldEdgeInputRecord>,
    HldNodeInputRecord,
    (u64, u64),
) {
    let (_directory, root, database, input) = run_original_graph_fixture();
    let spans = read_published_stage_rows(&database, "span-reference-text-table");
    let owner = locate_original_member_node(&input, &spans, owner).clone();
    let buffer = locate_original_kind_node(&input, &spans, "Buffer", "STRUCT").clone();
    let source = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    let start = source.find(signature).unwrap() + signature.rfind(path).unwrap();
    let uses = input
        .input_edge_records
        .iter()
        .filter(|edge| {
            edge.edge_source_reference == owner.node_target_reference
                && edge.edge_relation_kind_value == HldRelationKindValue::UsesType
        })
        .cloned()
        .collect();
    (
        input,
        uses,
        buffer,
        (start as u64, (start + path.len()) as u64),
    )
}

/// The byte range of an edge's site in its source file. `None` for a site in expanded
/// text: its range is a range of that expanded text, not of any source file.
fn read_edge_site_range(edge: &HldEdgeInputRecord) -> Option<(u64, u64)> {
    let site = edge
        .edge_evidence_row_key
        .evidence_site_occurrence_key
        .read_semantic_occurrence_parts();
    site.span_expansion_provenance_chain
        .is_empty()
        .then(|| site.span_byte_range_value.read_source_range_pair())
}

/// REQ-RUST-018.2
#[test]
fn test_new_return_type_edge() {
    let (_input, uses, buffer, expected_site) =
        read_signature_type_uses("new", "pub fn new() -> Buffer {", "Buffer");
    assert_eq!(
        uses.len(),
        1,
        "Buffer::new names one type in its signature: its return type"
    );
    assert_eq!(
        uses[0].edge_resolution_value,
        HldEdgeResolutionValue::Extracted {
            target: buffer.node_target_reference.clone()
        },
        "the return type resolves to the struct Buffer"
    );
    assert_eq!(
        read_edge_site_range(&uses[0]),
        Some(expected_site),
        "the site is the type path itself, not the whole signature"
    );
}

/// REQ-RUST-018.2
#[test]
fn test_write_builtin_type_edge() {
    let (input, uses, _buffer, expected_site) = read_signature_type_uses(
        "write",
        "fn write(self, buf: &mut Self::Buffer) -> &str;",
        "str",
    );
    let target: HldTargetIdentityValue = NameSymbolTargetKey::Builtin("str".to_owned()).into();
    let to_str = uses
        .iter()
        .filter(|edge| {
            edge.edge_resolution_value
                == HldEdgeResolutionValue::Extracted {
                    target: target.clone(),
                }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        to_str.len(),
        1,
        "Sealed::write names str once in its signature: its return type"
    );
    assert_eq!(read_edge_site_range(to_str[0]), Some(expected_site));
    let nodes = input
        .input_node_records
        .iter()
        .filter(|node| node.node_target_reference == target)
        .collect::<Vec<_>>();
    assert_eq!(nodes.len(), 1, "one node stands for the builtin str");
    assert_eq!(nodes[0].node_input_category, HldNodeInputCategory::Builtin);
}

/// REQ-RUST-018.2
#[test]
fn test_signature_external_type_unresolved() {
    let (_input, uses, _buffer, expected_site) = read_signature_type_uses(
        "enc_16lsd",
        "fn enc_16lsd<const OFFSET: usize>(buf: &mut [MaybeUninit<u8>], n: u64) {",
        "MaybeUninit<u8>",
    );
    let at_site = uses
        .iter()
        .filter(|edge| read_edge_site_range(edge) == Some(expected_site))
        .collect::<Vec<_>>();
    assert_eq!(
        at_site.len(),
        1,
        "the MaybeUninit path is stored once, though its source is not in this run"
    );
    assert_eq!(
        at_site[0].edge_resolution_value,
        HldEdgeResolutionValue::Unresolved {
            reason: HldUnresolvedReasonKind::ExternalSourceMissing
        },
        "core is excluded from this run, so the reason is missing source"
    );
}

/// REQ-RUST-018.2
#[test]
fn test_clone_self_type_edge() {
    let (_input, uses, buffer, expected_site) =
        read_signature_type_uses("clone", "fn clone(&self) -> Self {", "Self");
    assert_eq!(
        uses.len(),
        1,
        "clone names one type in its signature: Self, its return type"
    );
    assert_eq!(
        uses[0].edge_resolution_value,
        HldEdgeResolutionValue::Extracted {
            target: buffer.node_target_reference.clone()
        },
        "Self is the impl's self type, the struct Buffer"
    );
    assert_eq!(read_edge_site_range(&uses[0]), Some(expected_site));
}

/// REQ-RUST-048.0
#[test]
fn test_decimal_pairs_static_read() {
    let (_directory, root, database, input) = run_original_graph_fixture();
    let spans = read_published_stage_rows(&database, "span-reference-text-table");
    let owner = locate_original_member_node(&input, &spans, "enc_16lsd");
    let target = locate_original_kind_node(&input, &spans, "DECIMAL_PAIRS", "STATIC");
    let source = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    let body = source.find("fn enc_16lsd").unwrap()..source.find("fn div_rem_1e16").unwrap();
    let name = "DECIMAL_PAIRS";
    let expected = source[body.clone()]
        .match_indices(name)
        .map(|(offset, _)| {
            let start = (body.start + offset) as u64;
            (start, start + name.len() as u64)
        })
        .collect::<Vec<_>>();
    assert_eq!(expected.len(), 8, "the pin reads the static 8 times here");
    let reads = input
        .input_edge_records
        .iter()
        .filter(|edge| {
            edge.edge_source_reference == owner.node_target_reference
                && edge.edge_relation_kind_value == HldRelationKindValue::Reads
        })
        .collect::<Vec<_>>();
    let mut sites = reads
        .iter()
        .map(|edge| read_edge_site_range(edge).unwrap())
        .collect::<Vec<_>>();
    sites.sort_unstable();
    assert_eq!(
        sites, expected,
        "one reads row per path expression, sited at the path itself"
    );
    assert!(
        reads.iter().all(|edge| edge.edge_resolution_value
            == HldEdgeResolutionValue::Extracted {
                target: target.node_target_reference.clone()
            }),
        "every row targets the static DECIMAL_PAIRS"
    );
}

/// REQ-RUST-047.0
#[test]
fn test_buffer_clone_trait_unresolved() {
    let (_directory, root, database, input) = run_original_graph_fixture();
    let spans = read_published_stage_rows(&database, "span-reference-text-table");
    let buffer = locate_original_kind_node(&input, &spans, "Buffer", "STRUCT").clone();
    let text = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    let mut rows = input
        .input_edge_records
        .iter()
        .filter(|edge| {
            edge.edge_relation_kind_value == HldRelationKindValue::Implements
                && edge.edge_source_reference == buffer.node_target_reference
        })
        .map(|edge| {
            let (start, end) = read_edge_site_range(edge).unwrap();
            (
                text[start as usize..end as usize].to_owned(),
                text[..start as usize].ends_with("impl "),
                edge.edge_resolution_value.clone(),
            )
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.0.cmp(&right.0));
    let external = HldEdgeResolutionValue::Unresolved {
        reason: HldUnresolvedReasonKind::ExternalSourceMissing,
    };
    assert_eq!(
        rows,
        ["Clone", "Copy", "Default"]
            .map(|name| (name.to_owned(), true, external.clone()))
            .to_vec(),
        "Buffer implements three core traits; core source is excluded from this run"
    );
}

/// The `implements` rows of one itoa run as (builtin source name if any,
/// target node name if extracted, whether the site is in expanded text).
fn read_itoa_implements_rows(
    input: &HldMaterializationInputRecord,
) -> Vec<(Option<String>, Option<String>, bool)> {
    input
        .input_edge_records
        .iter()
        .filter(|edge| edge.edge_relation_kind_value == HldRelationKindValue::Implements)
        .map(|edge| {
            let source = match &edge.edge_source_reference {
                HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Builtin(name)) => {
                    Some(name.clone())
                }
                _ => None,
            };
            let target = match &edge.edge_resolution_value {
                HldEdgeResolutionValue::Extracted { target } => input
                    .input_node_records
                    .iter()
                    .find(|node| &node.node_target_reference == target)
                    .map(|node| node.node_qualified_path_text.clone()),
                _ => None,
            };
            let expanded = !edge
                .edge_evidence_row_key
                .evidence_site_occurrence_key
                .read_semantic_occurrence_parts()
                .span_expansion_provenance_chain
                .is_empty();
            (source, target, expanded)
        })
        .collect()
}

/// REQ-RUST-047.0
#[test]
fn test_itoa_implements_row_counts() {
    let (_directory, _root, _database, input) = run_original_graph_fixture();
    let rows = read_itoa_implements_rows(&input);
    let count = |name: &str| {
        rows.iter()
            .filter(|(_, target, _)| target.as_deref().is_some_and(|path| path.ends_with(name)))
            .count()
    };
    assert_eq!(rows.len(), 32, "32 of itoa's 33 impl headers name a trait");
    assert_eq!(
        (count("Integer"), count("Sealed"), count("Unsigned")),
        (12, 12, 5)
    );
    assert_eq!(
        rows.iter()
            .filter(|(_, target, _)| target.is_none())
            .count(),
        3,
        "Buffer's three core traits"
    );
    let sources = rows
        .iter()
        .filter_map(|(source, _, _)| source.as_deref())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(sources.len(), 12, "the twelve builtin integer types");
    for name in sources {
        let nodes = input
            .input_node_records
            .iter()
            .filter(|node| {
                node.node_input_category == HldNodeInputCategory::Builtin
                    && node.node_qualified_path_text == name
            })
            .count();
        assert_eq!(nodes, 1, "{name}");
    }
}

/// REQ-RUST-047.0
#[test]
fn test_implements_rows_read_twice() {
    let (_directory, _root, database, input) = run_original_graph_fixture();
    let read = |graph: &HldMaterializationInputRecord| {
        graph
            .input_edge_records
            .iter()
            .filter(|edge| edge.edge_relation_kind_value == HldRelationKindValue::Implements)
            .cloned()
            .collect::<Vec<_>>()
    };
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let scope = store.read_workspace_publication_scope().unwrap().unwrap();
    let again = store
        .assemble_full_graph_input(&scope, &RunCancelFlagValue::default())
        .unwrap();
    let rows = read(&input);
    assert_eq!(rows.len(), 32);
    assert_eq!(rows, read(&again), "same rows in the same order");
    assert_eq!(
        rows.iter().collect::<std::collections::BTreeSet<_>>().len(),
        32,
        "no row is stored twice"
    );
}

/// REQ-RUST-047.0
#[test]
fn test_macro_header_edge_provenance() {
    let (_directory, _root, database, input) = run_original_graph_fixture();
    let rows = read_itoa_implements_rows(&input);
    let expanded = rows.iter().filter(|row| row.2).collect::<Vec<_>>();
    assert_eq!(
        expanded.len(),
        28,
        "28 impl headers come from macro expansion"
    );
    assert!(
        expanded
            .iter()
            .all(|(source, target, _)| source.is_some() && target.is_some()),
        "each joins a builtin to a trait of this crate"
    );
    let spans = read_published_stage_rows(&database, "span-reference-text-table");
    let mut sites = input
        .input_edge_records
        .iter()
        .filter(|edge| edge.edge_relation_kind_value == HldRelationKindValue::Implements)
        .filter_map(|edge| {
            let site = edge
                .edge_evidence_row_key
                .evidence_site_occurrence_key
                .read_semantic_occurrence_parts();
            let (start, end) = site.span_byte_range_value.read_source_range_pair();
            let chain = site
                .span_expansion_provenance_chain
                .iter()
                .map(|digest| digest.iter().map(|byte| format!("{byte:02x}")).collect())
                .collect::<Vec<String>>();
            (!chain.is_empty()).then(|| {
                let headers = spans
                    .iter()
                    .filter(|span| {
                        span["span_syntax_kind_value"] == "IMPL"
                            && span["span_expansion_provenance_chain"] == json!(chain)
                            && span["span_byte_start_offset"].as_u64().unwrap() <= start
                            && end <= span["span_byte_end_offset"].as_u64().unwrap()
                    })
                    .collect::<Vec<_>>();
                assert_eq!(headers.len(), 1, "one expanded impl holds the site");
                let offset = headers[0]["span_byte_start_offset"].as_u64().unwrap();
                let text = headers[0]["span_source_code_text"].as_str().unwrap();
                assert!(text[..(start - offset) as usize].ends_with("impl "));
                assert!(
                    site.span_source_file_path
                        .ends_with(&format!("macro-source/{}/expanded.rs", chain[0]))
                );
                text[(start - offset) as usize..(end - offset) as usize].replace(' ', "")
            })
        })
        .collect::<Vec<_>>();
    sites.sort();
    sites.dedup_by(|left, right| left == right);
    assert_eq!(
        sites,
        ["Integer", "Unsigned", "private::Sealed"],
        "each site is the trait path in the expanded impl header"
    );
}

/// REQ-RUST-047.0
#[test]
fn test_unsigned_impl_builtin_source() {
    let (_directory, root, database, input) = run_original_graph_fixture();
    let spans = read_published_stage_rows(&database, "span-reference-text-table");
    let unsigned = locate_original_kind_node(&input, &spans, "Unsigned", "TRAIT").clone();
    let source: HldTargetIdentityValue = NameSymbolTargetKey::Builtin("u128".to_owned()).into();
    let text = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    let start = (text.find("impl Unsigned for u128 {").unwrap() + "impl ".len()) as u64;
    let rows = input
        .input_edge_records
        .iter()
        .filter(|edge| {
            edge.edge_relation_kind_value == HldRelationKindValue::Implements
                && edge.edge_source_reference == source
                && edge
                    .edge_evidence_row_key
                    .evidence_site_occurrence_key
                    .read_semantic_occurrence_parts()
                    .span_expansion_provenance_chain
                    .is_empty()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        rows.len(),
        1,
        "itoa has one original trait impl header on u128: impl Unsigned for u128"
    );
    assert_eq!(
        rows[0].edge_resolution_value,
        HldEdgeResolutionValue::Extracted {
            target: unsigned.node_target_reference.clone()
        }
    );
    assert_eq!(
        read_edge_site_range(rows[0]),
        Some((start, start + "Unsigned".len() as u64))
    );
    let nodes = input
        .input_node_records
        .iter()
        .filter(|node| node.node_target_reference == source)
        .collect::<Vec<_>>();
    assert_eq!(nodes.len(), 1, "one node stands for the builtin u128");
    assert_eq!(nodes[0].node_input_category, HldNodeInputCategory::Builtin);
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
    // REQ-RUST-052.0 G5: the one-name qualifier of a two-segment body value path
    // (`u128::MAX_STR_LEN`, `I::MAX_STR_LEN`) is its own Type request; `Self` is not requested.
    let (value_prefixes, paths): (Vec<_>, Vec<_>) = paths
        .into_iter()
        .partition(|row| row["site_path_context_kind"] == "body_value_type_prefix");
    let mut prefix_kinds = std::collections::BTreeMap::<(String, String), usize>::new();
    for row in &value_prefixes {
        assert_eq!(row["path_resolution_state_kind"], "resolved");
        let targets = row["path_target_symbol_rows"].as_array().unwrap();
        assert_eq!(targets.len(), 1);
        *prefix_kinds
            .entry((
                row["path_site_source_text"].as_str().unwrap().to_owned(),
                targets[0]["target_symbol_kind_value"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
            ))
            .or_default() += 1;
    }
    let kind = |text: &str, kind: &str, rows| ((text.to_owned(), kind.to_owned()), rows);
    assert_eq!(
        prefix_kinds,
        std::collections::BTreeMap::from([
            kind("I", "TypeParam", 1),
            kind("i128", "Builtin", 1),
            // 5 in the source-written `fmt` for u128, 1 in the generated `write` for i128
            kind("u128", "Builtin", 6),
            kind("u16", "Builtin", 1),
            kind("u32", "Builtin", 1),
            kind("u64", "Builtin", 1),
            kind("u8", "Builtin", 1),
        ])
    );
    // REQ-RUST-052.0 G9: the whole callee path of a call is its own request kind.
    let (callee_paths, paths): (Vec<_>, Vec<_>) = paths
        .into_iter()
        .partition(|row| row["site_path_context_kind"] == "body_call_callee_path");
    let mut callee_kinds = std::collections::BTreeMap::<(String, String), usize>::new();
    for row in &callee_paths {
        let text = row["path_site_source_text"].as_str().unwrap();
        let text = text.split("::<").next().unwrap().replace(' ', "");
        let targets = row["path_target_symbol_rows"].as_array().unwrap();
        let outcome = match row["path_resolution_state_kind"].as_str().unwrap() {
            "resolved" => {
                assert_eq!(targets.len(), 1, "{row}");
                targets[0]["target_symbol_kind_value"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            }
            state => format!(
                "{state}:{}",
                row["path_unresolved_reason_kind"].as_str().unwrap_or("-")
            ),
        };
        *callee_kinds.entry((text, outcome)).or_default() += 1;
    }
    assert_eq!(
        callee_kinds,
        std::collections::BTreeMap::from([
            // A member reached through a type: name lookup stores no target and no reason.
            kind("Buffer::new", "unresolved:-", 2),
            kind("u128::from", "unresolved:-", 9),
            // A tuple struct's constructor.
            kind("DecimalPairs", "Struct", 1),
            // A trait's method, named through the trait.
            kind("Unsigned::fmt", "Function", 10),
            kind("div_rem_1e16", "Function", 2),
            kind("divmod100", "Function", 12),
            kind("enc_16lsd", "Function", 2),
            kind("slice_buffer_to_str", "Function", 10),
            kind("u128_ext::mulhi", "Function", 1),
            // In `core`, which is not read. `str` is the module imported by `use core::str`.
            kind("MaybeUninit", "unresolved:external_source_missing", 1),
            kind(
                "hint::unreachable_unchecked",
                "unresolved:external_source_missing",
                1
            ),
            kind("mem::size_of", "unresolved:external_source_missing", 4),
            kind(
                "str::from_utf8_unchecked",
                "unresolved:external_source_missing",
                1
            ),
        ]),
        "56 callee paths: (path, what name lookup stored for it)"
    );
    // REQ-RUST-048.0: value paths in bodies are their own request kind, kept out of the
    // signature and import totals below.
    let (body_values, paths): (Vec<_>, Vec<_>) = paths
        .into_iter()
        .partition(|row| row["site_path_context_kind"] == "body_value_path");
    let statics = body_values
        .iter()
        .filter(|row| row["path_site_source_text"] == "DECIMAL_PAIRS")
        .collect::<Vec<_>>();
    assert_eq!(statics.len(), 38, "14 in original bodies, 24 in expansions");
    assert!(
        statics
            .iter()
            .all(|row| row["path_resolution_state_kind"] == "resolved")
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
    assert_eq!(paths.len(), 204);
    assert_eq!(count_state("resolved"), 166);
    assert_eq!(count_state("ambiguous"), 0);
    let reasoned = paths
        .iter()
        .filter(|row| row["path_unresolved_reason_kind"] == "external_source_missing")
        .count();
    assert_eq!(reasoned, 22);
}

/// REQ-RUST-048.0 R4
#[test]
fn test_generic_const_bound_read() {
    let (_directory, root, database, input) = run_original_graph_fixture();
    let spans = read_published_stage_rows(&database, "span-reference-text-table");
    let format = locate_original_member_node(&input, &spans, "format");
    let target = locate_original_kind_node(&input, &spans, "MAX_STR_LEN", "CONST");
    let source = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    let start = source.find("I::MAX_STR_LEN").unwrap() as u64;
    let reads = input
        .input_edge_records
        .iter()
        .filter(|edge| {
            edge.edge_source_reference == format.node_target_reference
                && edge.edge_relation_kind_value == HldRelationKindValue::Reads
        })
        .collect::<Vec<_>>();
    assert_eq!(reads.len(), 1, "Buffer::format reads one const, once");
    let read = reads[0];
    assert_eq!(
        read_edge_site_range(read),
        Some((start, start + "I::MAX_STR_LEN".len() as u64))
    );
    assert_eq!(
        read.edge_resolution_value,
        HldEdgeResolutionValue::Extracted {
            target: target.node_target_reference.clone()
        },
        "the type parameter path targets the trait const Integer::MAX_STR_LEN"
    );
    let metadata: Value = serde_json::from_str(
        read.edge_dispatch_metadata_text
            .as_deref()
            .unwrap_or("null"),
    )
    .unwrap();
    assert_eq!(
        metadata["bound_read_evidence"],
        json!({"bound_chain":["I", "Integer", "private::Sealed"],"missing_source_traits":["Copy"]}),
        "the row names the bounds walked and the trait whose source was not read"
    );
}

/// REQ-RUST-048.0 R5 R7
#[test]
fn test_itoa_reads_row_counts() {
    let (_directory, root, database, input) = run_original_graph_fixture();
    let spans = read_published_stage_rows(&database, "span-reference-text-table");
    let new = locate_original_member_node(&input, &spans, "new");
    let source = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    let rows = input
        .input_edge_records
        .iter()
        .filter(|edge| {
            edge.edge_relation_kind_value == HldRelationKindValue::Reads
                && edge
                    .edge_evidence_row_key
                    .evidence_site_occurrence_key
                    .read_semantic_occurrence_parts()
                    .span_expansion_provenance_chain
                    .is_empty()
        })
        .map(|edge| {
            let (start, end) = read_edge_site_range(edge).unwrap();
            (edge, &source[start as usize..end as usize])
        })
        .collect::<Vec<_>>();
    let count = |text: &str, extracted: bool| {
        rows.iter()
            .filter(|(edge, site)| {
                *site == text
                    && extracted
                        == matches!(
                            edge.edge_resolution_value,
                            HldEdgeResolutionValue::Extracted { .. }
                        )
            })
            .count()
    };
    assert_eq!(rows.len(), 21, "sited in original text");
    assert_eq!(
        (count("DECIMAL_PAIRS", true), count("I::MAX_STR_LEN", true)),
        (14, 1)
    );
    assert_eq!(
        (
            count("u128::MAX_STR_LEN", true),
            count("i128::MAX_STR_LEN", true)
        ),
        (5, 1),
        "REQ-RUST-052.0 G5: the const of the one stored trait impl of the builtin type"
    );
    assert_eq!(rows.len(), 20 + 1, "all 21 have a target");
    for (edge, site) in &rows {
        if let HldEdgeResolutionValue::Unresolved { reason } = &edge.edge_resolution_value {
            assert_eq!(
                *reason,
                HldUnresolvedReasonKind::TraitSolverNeeded,
                "no reason was stored for {site}"
            );
        }
        if *site == "i128::MAX_STR_LEN" {
            assert_eq!(edge.edge_source_reference, new.node_target_reference);
        }
    }
}

/// REQ-RUST-049.0 R6 R7 R11
#[test]
fn test_itoa_imports_row_counts() {
    let (_directory, root, database, input) = run_original_graph_fixture();
    let source = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    let outcomes = read_published_stage_rows(&database, "import-resolution-site-records");
    assert_eq!(
        outcomes.len(),
        4,
        "R1: one stored outcome per use import; the 4 bound imports get none"
    );
    assert!(outcomes.iter().all(|row| {
        row["import_resolution_state_kind"] == "unresolved"
            && row["import_unresolved_reason_kind"] == "external_source_missing"
    }));
    let use_nodes = input
        .input_node_records
        .iter()
        .filter(|node| node.node_item_kind_value == Some(HldItemKindValue::UseStatement))
        .map(|node| &node.node_target_reference)
        .collect::<Vec<_>>();
    assert_eq!(
        use_nodes.len(),
        3,
        "the two cfg-inactive uses are not nodes"
    );
    let mut rows = input
        .input_edge_records
        .iter()
        .filter(|edge| edge.edge_relation_kind_value == HldRelationKindValue::Imports)
        .map(|edge| {
            let (start, end) = read_edge_site_range(edge).unwrap();
            let metadata = edge
                .edge_dispatch_metadata_text
                .as_deref()
                .and_then(|text| serde_json::from_str::<Value>(text).ok())
                .unwrap_or(Value::Null);
            assert!(use_nodes.contains(&&edge.edge_source_reference));
            assert_eq!(
                edge.edge_resolution_value,
                HldEdgeResolutionValue::Unresolved {
                    reason: HldUnresolvedReasonKind::ExternalSourceMissing
                },
                "the core source is not read in this run"
            );
            (
                source[start as usize..end as usize].to_owned(),
                metadata["import_path_text"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
            )
        })
        .collect::<Vec<_>>();
    rows.sort();
    let expected = [
        ("MaybeUninit", "core::mem::MaybeUninit"),
        ("core::hint", "core::hint"),
        ("core::str", "core::str"),
        ("self", "core::mem"),
    ]
    .map(|(site, path)| (site.to_owned(), path.to_owned()));
    assert_eq!(rows, expected);
}

/// Reads the stored graph a second time and checks the rows of one relation: the expected
/// number, the same rows in the same order on both reads, and no row stored twice.
fn check_relation_rows_twice(relation: HldRelationKindValue, expected: usize) {
    let (_directory, _root, database, input) = run_original_graph_fixture();
    let read = |graph: &HldMaterializationInputRecord| {
        graph
            .input_edge_records
            .iter()
            .filter(|edge| edge.edge_relation_kind_value == relation)
            .cloned()
            .collect::<Vec<_>>()
    };
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let scope = store.read_workspace_publication_scope().unwrap().unwrap();
    let again = store
        .assemble_full_graph_input(&scope, &RunCancelFlagValue::default())
        .unwrap();
    let rows = read(&input);
    assert_eq!(rows.len(), expected);
    assert_eq!(rows, read(&again), "same rows in the same order");
    assert_eq!(
        rows.iter().collect::<std::collections::BTreeSet<_>>().len(),
        expected,
        "no row is stored twice"
    );
}

/// REQ-RUST-048.0 R8
#[test]
fn test_reads_rows_read_twice() {
    check_relation_rows_twice(HldRelationKindValue::Reads, 55);
}

/// REQ-RUST-049.0 R12
#[test]
fn test_imports_rows_read_twice() {
    check_relation_rows_twice(HldRelationKindValue::Imports, 4);
}

/// REQ-RUST-046.0 R1, R2, R3: the `const _` written by `impl_Integer!(i8, u8)` is a node,
/// identified by its call and its place in the expanded text, held by the file of the call.
#[test]
fn test_generated_const_node_contained() {
    let (_directory, root, database, input) = run_original_graph_fixture();
    let source = fs::read(root.join("src/lib.rs")).unwrap();
    let macros = read_published_stage_rows(&database, "macro-expanded-item-table");
    let calls = macros
        .iter()
        .filter(|row| {
            let call = &row["macro_invocation_identity_record"];
            let range = &call["span_byte_range_value"];
            call["span_source_file_path"] == "src/lib.rs"
                && source
                    .get(range[0].as_u64().unwrap() as usize..range[1].as_u64().unwrap() as usize)
                    == Some(b"impl_Integer!(i8, u8);".as_slice())
        })
        .collect::<Vec<_>>();
    assert_eq!(calls.len(), 1, "pin has one impl_Integer!(i8, u8) call");
    let chain = &calls[0]["macro_invocation_provenance_chain"];
    assert_eq!(
        chain.as_array().unwrap().len(),
        1,
        "the call is source-written"
    );
    let spans = read_published_stage_rows(&database, "span-reference-text-table");
    let consts = spans
        .iter()
        .filter(|span| {
            span["span_expansion_provenance_chain"] == *chain
                && span["span_syntax_kind_value"] == "CONST"
                && span["span_declared_name_value"].is_null()
        })
        .collect::<Vec<_>>();
    assert_eq!(consts.len(), 1, "the call expands to one const _");
    assert_eq!(consts[0]["span_configuration_active_flag"], true);
    let range = (
        consts[0]["span_byte_start_offset"].as_u64().unwrap(),
        consts[0]["span_byte_end_offset"].as_u64().unwrap(),
    );
    let expanded_path = format!(
        ".parseltongue/macro-source/{}/expanded.rs",
        chain[0].as_str().unwrap()
    );
    let nodes = input
        .input_node_records
        .iter()
        .filter(|node| {
            let HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(key)) =
                &node.node_target_reference
            else {
                return false;
            };
            let parts = key.read_semantic_occurrence_parts();
            let digests = parts
                .span_expansion_provenance_chain
                .iter()
                .map(|digest| digest.iter().map(|byte| format!("{byte:02x}")).collect())
                .collect::<Vec<String>>();
            json!(digests) == *chain
                && parts.span_source_file_path == expanded_path
                && parts.span_byte_range_value.read_source_range_pair() == range
        })
        .collect::<Vec<_>>();
    assert_eq!(nodes.len(), 1, "the generated const _ is one node");
    let node = nodes[0];
    assert_eq!(node.node_input_category, HldNodeInputCategory::LocalItem);
    assert_eq!(node.node_item_kind_value, Some(HldItemKindValue::Constant));
    assert_eq!(
        node.node_owner_kind_value,
        Some(HldOwnerKindValue::CrateRoot),
        "the call is written at the top of the crate root file"
    );
    assert_eq!(
        node.node_source_relative_path.as_deref(),
        Some("src/lib.rs"),
        "a generated node names the file of its macro call"
    );
    assert_eq!(
        node.node_qualified_path_text,
        "itoa::src/lib.rs::const _ #impl_Integer!(i8, u8)"
    );
    let HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(key)) =
        &node.node_target_reference
    else {
        unreachable!()
    };
    assert_eq!(
        blake3::Hash::from(
            *key.read_semantic_occurrence_parts()
                .span_content_digest_full
                .read_full_digest_bytes()
        )
        .to_hex()
        .to_string(),
        consts[0]["span_content_digest_full"].as_str().unwrap()
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
    assert_eq!(parents.len(), 1, "one lexical container");
    let area = input
        .input_node_records
        .iter()
        .find(|area| {
            area.node_input_category == HldNodeInputCategory::FilePublicArea
                && area.node_source_relative_path.as_deref() == Some("src/lib.rs")
        })
        .unwrap();
    assert_eq!(
        parents[0].edge_source_reference, area.node_target_reference,
        "the file area of the call holds the item; the macro call node does not"
    );
}

/// REQ-RUST-046.0 R1, R3, R4, R7: every active item written by a macro expansion is one
/// node with one container; inactive generated spans and impl blocks are not nodes.
#[test]
fn test_generated_item_node_counts() {
    let (_directory, _root, _database, input) = run_original_graph_fixture();
    let generated = input
        .input_node_records
        .iter()
        .filter(|node| {
            matches!(
                &node.node_target_reference,
                HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(key))
                    if !key
                        .read_semantic_occurrence_parts()
                        .span_expansion_provenance_chain
                        .is_empty()
            )
        })
        .collect::<Vec<_>>();
    let count = |kind: HldItemKindValue, name: &str| {
        generated
            .iter()
            .filter(|node| {
                node.node_item_kind_value == Some(kind)
                    && (node
                        .node_qualified_path_text
                        .ends_with(&format!("::{name}"))
                        || node
                            .node_qualified_path_text
                            .contains(&format!("::{name} #")))
            })
            .count()
    };
    assert_eq!(
        [
            count(HldItemKindValue::Constant, "const _"),
            count(HldItemKindValue::AssociatedConstant, "MAX_STR_LEN"),
            count(HldItemKindValue::AssociatedType, "Buffer"),
            count(HldItemKindValue::AssociatedFunction, "write"),
            count(HldItemKindValue::AssociatedFunction, "fmt"),
        ],
        [5, 12, 12, 12, 4]
    );
    assert_eq!(generated.len(), 45);
    assert_eq!(input.input_node_records.len(), 103);
    let area = input
        .input_node_records
        .iter()
        .find(|area| {
            area.node_input_category == HldNodeInputCategory::FilePublicArea
                && area.node_source_relative_path.as_deref() == Some("src/lib.rs")
        })
        .unwrap();
    generated.iter().for_each(|node| {
        assert_eq!(
            node.node_source_relative_path.as_deref(),
            Some("src/lib.rs")
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
        assert_eq!(parents.len(), 1, "{}", node.node_qualified_path_text);
        assert_eq!(parents[0].edge_source_reference, area.node_target_reference);
    });
    assert_eq!(
        input
            .input_edge_records
            .iter()
            .filter(|edge| edge.edge_relation_kind_value == HldRelationKindValue::Contains)
            .count(),
        88
    );
}

/// REQ-RUST-046.0 R5: a generated node names the macro call that wrote it, and that call
/// is a node; a source-written node names no call.
#[test]
fn test_generated_node_names_call() {
    let (_directory, root, database, input) = run_original_graph_fixture();
    let source = fs::read(root.join("src/lib.rs")).unwrap();
    let call_ranges = read_published_stage_rows(&database, "macro-expanded-item-table")
        .iter()
        .filter(|row| row["macro_resolution_state_kind"] == "expanded")
        .map(|row| {
            let range = &row["macro_invocation_identity_record"]["span_byte_range_value"];
            (
                row["macro_invocation_provenance_chain"][0]
                    .as_str()
                    .unwrap()
                    .to_owned(),
                (range[0].as_u64().unwrap(), range[1].as_u64().unwrap()),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut named = 0;
    let mut first_call_text = Vec::new();
    for node in &input.input_node_records {
        let HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(key)) =
            &node.node_target_reference
        else {
            assert_eq!(node.node_generating_call_reference, None);
            continue;
        };
        let parts = key.read_semantic_occurrence_parts();
        let Some(digest) = parts.span_expansion_provenance_chain.first() else {
            assert_eq!(
                node.node_generating_call_reference, None,
                "source-written node names no call: {}",
                node.node_qualified_path_text
            );
            continue;
        };
        let digest = digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let call = node
            .node_generating_call_reference
            .as_ref()
            .unwrap_or_else(|| panic!("no call named: {}", node.node_qualified_path_text));
        let call_node = input
            .input_node_records
            .iter()
            .find(|other| other.node_target_reference == *call)
            .unwrap_or_else(|| panic!("the named call is not a node"));
        assert_eq!(
            call_node.node_item_kind_value,
            Some(HldItemKindValue::MacroInvocation)
        );
        let HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(call_key)) = call else {
            panic!("call identity expected");
        };
        let call_parts = call_key.read_semantic_occurrence_parts();
        let range = call_parts.span_byte_range_value.read_source_range_pair();
        assert_eq!(call_parts.span_source_file_path, "src/lib.rs");
        assert!(call_parts.span_expansion_provenance_chain.is_empty());
        assert_eq!(range, call_ranges[&digest], "the call of the node's chain");
        if node.node_item_kind_value == Some(HldItemKindValue::Constant)
            && first_call_text.is_empty()
        {
            first_call_text = source[range.0 as usize..range.1 as usize].to_vec();
        }
        named += 1;
    }
    assert_eq!(named, 45);
    assert!(first_call_text.starts_with(b"impl_Integer!("));
}

/// REQ-RUST-046.0 R5: each generated node has exactly one `generates` row, from the macro
/// call it names and sited at that call; itoa has 45.
#[test]
fn test_generates_edge_row_counts() {
    let (_directory, _root, _database, input) = run_original_graph_fixture();
    let rows = input
        .input_edge_records
        .iter()
        .filter(|edge| edge.edge_relation_kind_value == HldRelationKindValue::Generates)
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 45);
    let mut rows_per_call = std::collections::BTreeMap::new();
    for node in &input.input_node_records {
        let Some(call) = &node.node_generating_call_reference else {
            continue;
        };
        let HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(call_key)) = call else {
            panic!("call identity expected");
        };
        let matching = rows
            .iter()
            .filter(|edge| {
                edge.edge_resolution_value
                    == HldEdgeResolutionValue::Extracted {
                        target: node.node_target_reference.clone(),
                    }
            })
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1, "{}", node.node_qualified_path_text);
        assert_eq!(matching[0].edge_source_reference, *call);
        assert_eq!(
            matching[0]
                .edge_evidence_row_key
                .evidence_site_occurrence_key,
            *call_key
        );
        let range = call_key
            .read_semantic_occurrence_parts()
            .span_byte_range_value
            .read_source_range_pair();
        *rows_per_call.entry(range).or_insert(0_usize) += 1;
    }
    // 11 of the 15 calls write an active item: five `impl_Integer!` (7 each), the two
    // `impl_Integer_size!` for 64-bit pointers (3 each) and four `impl_Unsigned!` (1 each).
    // The other four `impl_Integer_size!` calls write only items whose `#[cfg]` is off.
    let mut counts = rows_per_call.values().copied().collect::<Vec<_>>();
    counts.sort_unstable();
    assert_eq!(counts, [1, 1, 1, 1, 3, 3, 7, 7, 7, 7, 7]);
    assert_eq!(input.input_edge_records.len(), 502);
}

/// REQ-RUST-051.0 E1-E5, E8: each of itoa's 15 macro call nodes has exactly one `expands`
/// row to the `macro_rules!` definition its path names, sited at the call, with the arm
/// the expansion matched.
#[test]
fn test_expands_edge_row_counts() {
    let (_directory, _root, _database, input) = run_original_graph_fixture();
    let find_node_record = |reference: &HldTargetIdentityValue| {
        input
            .input_node_records
            .iter()
            .find(|node| node.node_target_reference == *reference)
            .unwrap()
    };
    let rows = input
        .input_edge_records
        .iter()
        .filter(|edge| edge.edge_relation_kind_value == HldRelationKindValue::Expands)
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 15);
    let mut rows_per_definition = std::collections::BTreeMap::new();
    for edge in &rows {
        let source = find_node_record(&edge.edge_source_reference);
        assert_eq!(
            source.node_item_kind_value,
            Some(HldItemKindValue::MacroInvocation)
        );
        let HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(call_key)) =
            &edge.edge_source_reference
        else {
            panic!("call identity expected");
        };
        assert_eq!(
            edge.edge_evidence_row_key.evidence_site_occurrence_key,
            *call_key
        );
        let HldEdgeResolutionValue::Extracted { target } = &edge.edge_resolution_value else {
            panic!("{} has no target", source.node_qualified_path_text);
        };
        let definition = find_node_record(target);
        assert_eq!(
            definition.node_item_kind_value,
            Some(HldItemKindValue::MacroDefinition)
        );
        let name = definition
            .node_qualified_path_text
            .rsplit("::")
            .next()
            .unwrap();
        // The call's own text starts with the name of the definition it expands.
        assert!(
            source
                .node_qualified_path_text
                .contains(&format!("::{name}!(")),
            "{} -> {}",
            source.node_qualified_path_text,
            definition.node_qualified_path_text
        );
        let metadata: Value =
            serde_json::from_str(edge.edge_dispatch_metadata_text.as_deref().unwrap()).unwrap();
        assert_eq!(
            metadata,
            json!({"macro_name_text": name, "macro_matched_arm_index": 0})
        );
        *rows_per_definition
            .entry(definition.node_qualified_path_text.as_str())
            .or_insert(0_usize) += 1;
    }
    assert_eq!(
        rows_per_definition.into_iter().collect::<Vec<_>>(),
        [
            ("itoa::src/lib.rs::impl_Integer", 5),
            ("itoa::src/lib.rs::impl_Integer_size", 6),
            ("itoa::src/lib.rs::impl_Unsigned", 4),
        ]
    );
    let calls = input
        .input_node_records
        .iter()
        .filter(|node| node.node_item_kind_value == Some(HldItemKindValue::MacroInvocation))
        .count();
    assert_eq!(calls, 15);
    let sources = rows
        .iter()
        .map(|edge| &edge.edge_source_reference)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(sources.len(), 15);
}

/// REQ-RUST-046.0 R11: a second read of the store gives the same 45 generated nodes, each
/// naming the same macro call, and the same `generates` and `contains` rows.
#[test]
fn test_generated_nodes_read_twice() {
    let (_directory, _root, database, input) = run_original_graph_fixture();
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let scope = store.read_workspace_publication_scope().unwrap().unwrap();
    let again = store
        .assemble_full_graph_input(&scope, &RunCancelFlagValue::default())
        .unwrap();
    let read_nodes = |graph: &HldMaterializationInputRecord| {
        graph
            .input_node_records
            .iter()
            .filter(|node| node.node_generating_call_reference.is_some())
            .cloned()
            .collect::<Vec<_>>()
    };
    let nodes = read_nodes(&input);
    assert_eq!(nodes.len(), 45);
    assert_eq!(nodes, read_nodes(&again), "same nodes in the same order");
    assert_eq!(input.input_node_records, again.input_node_records);
    assert_eq!(input.input_edge_records, again.input_edge_records);
    check_relation_rows_twice(HldRelationKindValue::Generates, 45);
    check_relation_rows_twice(HldRelationKindValue::Contains, 88);
}

/// REQ-RUST-050.0 L1, L3, L4: every node label is distinct on itoa, none carries the
/// checkout path, and a macro call or use item reads as its own source text.
#[test]
fn test_node_labels_all_distinct() {
    let (_directory, root, _database, input) = run_original_graph_fixture();
    let source = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    let labels = input
        .input_node_records
        .iter()
        .map(|node| node.node_qualified_path_text.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(input.input_node_records.len(), 103);
    assert_eq!(labels.len(), 103, "every label is distinct");
    assert!(
        labels.iter().all(|label| !label.contains("file:///")),
        "no label carries the checkout path"
    );
    let (mut calls, mut uses) = (0, 0);
    for node in &input.input_node_records {
        match node.node_item_kind_value {
            Some(HldItemKindValue::MacroInvocation) => calls += 1,
            Some(HldItemKindValue::UseStatement) => uses += 1,
            _ => continue,
        }
        let HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(key)) =
            &node.node_target_reference
        else {
            panic!("item identity expected");
        };
        let range = key
            .read_semantic_occurrence_parts()
            .span_byte_range_value
            .read_source_range_pair();
        let text = source[range.0 as usize..range.1 as usize].trim_end_matches(';');
        assert_eq!(
            node.node_qualified_path_text,
            format!("itoa::src/lib.rs::{text}")
        );
    }
    assert_eq!((calls, uses), (15, 3));
}

/// REQ-RUST-050.0 L2, L5: an impl member names its impl header, original or generated,
/// and a generated `const _` names the call that wrote it.
#[test]
fn test_impl_member_label_segment() {
    let (_directory, _root, _database, input) = run_original_graph_fixture();
    let read_label_node_count = |label: &str| {
        input
            .input_node_records
            .iter()
            .filter(|node| node.node_qualified_path_text == label)
            .count()
    };
    for label in [
        "itoa::src/lib.rs::<file-area>",
        "itoa::src/lib.rs::Buffer::new",
        "itoa::src/lib.rs::Buffer::format",
        "itoa::src/lib.rs::<Buffer as Default>::default",
        "itoa::src/lib.rs::<Buffer as Clone>::clone",
        "itoa::src/lib.rs::<u128 as Unsigned>::fmt",
        "itoa::src/lib.rs::<u8 as Unsigned>::fmt",
        "itoa::src/lib.rs::<u8 as private::Sealed>::write",
        "itoa::src/lib.rs::<u8 as Integer>::MAX_STR_LEN",
        "itoa::src/lib.rs::<usize as private::Sealed>::Buffer",
        "itoa::src/lib.rs::private::Sealed::write",
        "itoa::src/lib.rs::Integer::MAX_STR_LEN",
        "itoa::src/lib.rs::impl_Integer",
        "itoa::src/lib.rs::const _ #impl_Integer!(i8, u8)",
        "itoa::src/u128_ext.rs::mulhi",
    ] {
        assert_eq!(read_label_node_count(label), 1, "{label}");
    }
    let unnamed = input
        .input_node_records
        .iter()
        .filter(|node| {
            node.node_item_kind_value == Some(HldItemKindValue::Constant)
                && node.node_generating_call_reference.is_some()
        })
        .collect::<Vec<_>>();
    assert_eq!(unnamed.len(), 5);
    for node in unnamed {
        let call = input
            .input_node_records
            .iter()
            .find(|other| {
                Some(&other.node_target_reference) == node.node_generating_call_reference.as_ref()
            })
            .unwrap_or_else(|| panic!("the named call is not a node"));
        let segment = call
            .node_qualified_path_text
            .strip_prefix("itoa::src/lib.rs::")
            .unwrap();
        assert_eq!(
            node.node_qualified_path_text,
            format!("itoa::src/lib.rs::const _ #{segment}")
        );
    }
}

/// REQ-RUST-052.0 G1, G2: a function written by a macro expansion is a source of `reads`
/// rows. Each of the four generated `<uN as Unsigned>::fmt` reads the static
/// `DECIMAL_PAIRS` six times; every row is sited in the expanded text of its own node.
#[test]
fn test_generated_fmt_static_reads() {
    let (_directory, _root, database, input) = run_original_graph_fixture();
    let spans = read_published_stage_rows(&database, "span-reference-text-table");
    let target = locate_original_kind_node(&input, &spans, "DECIMAL_PAIRS", "STATIC");
    let writers = input
        .input_node_records
        .iter()
        .filter(|node| {
            node.node_generating_call_reference.is_some()
                && node
                    .node_qualified_path_text
                    .ends_with(" as Unsigned>::fmt")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        writers.len(),
        4,
        "u8, u16, u32 and u64; the one for u128 is source-written"
    );
    for node in writers {
        let HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(key)) =
            &node.node_target_reference
        else {
            panic!("definition identity expected");
        };
        let owner = key.read_semantic_occurrence_parts();
        let (owner_start, owner_end) = owner.span_byte_range_value.read_source_range_pair();
        assert_eq!(owner.span_expansion_provenance_chain.len(), 1);
        let text = spans
            .iter()
            .find(|span| {
                span["span_declared_name_value"] == "fmt"
                    && span["span_byte_start_offset"] == owner_start
                    && span["span_byte_end_offset"] == owner_end
                    && span["file_source_absolute_path"]
                        .as_str()
                        .is_some_and(|path| path.ends_with(owner.span_source_file_path))
            })
            .and_then(|span| span["span_source_code_text"].as_str())
            .unwrap();
        let rows = input
            .input_edge_records
            .iter()
            .filter(|edge| {
                edge.edge_relation_kind_value == HldRelationKindValue::Reads
                    && edge.edge_source_reference == node.node_target_reference
            })
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 6, "{}", node.node_qualified_path_text);
        for edge in rows {
            assert_eq!(
                edge.edge_resolution_value,
                HldEdgeResolutionValue::Extracted {
                    target: target.node_target_reference.clone()
                }
            );
            let site = edge
                .edge_evidence_row_key
                .evidence_site_occurrence_key
                .read_semantic_occurrence_parts();
            let (start, end) = site.span_byte_range_value.read_source_range_pair();
            assert_eq!(site.span_source_file_path, owner.span_source_file_path);
            assert_eq!(
                site.span_expansion_provenance_chain,
                owner.span_expansion_provenance_chain
            );
            assert!(owner_start <= start && end <= owner_end);
            assert_eq!(
                &text[(start - owner_start) as usize..(end - owner_start) as usize],
                "DECIMAL_PAIRS"
            );
        }
    }
    let stored = |relation| {
        input
            .input_edge_records
            .iter()
            .filter(|edge| edge.edge_relation_kind_value == relation)
            .count()
    };
    assert_eq!(
        stored(HldRelationKindValue::Reads),
        55,
        "21 original and 34 generated"
    );
    // The other 10 generated rows are `Self::MAX_STR_LEN` and `uN::MAX_STR_LEN` in generated
    // `write` bodies. Name lookup alone does not find a trait const through a builtin type;
    // G5 gives each its target (`test_type_relative_member_targets`), so none is left.
    let pending = input
        .input_edge_records
        .iter()
        .filter(|edge| {
            edge.edge_relation_kind_value == HldRelationKindValue::Reads
                && !edge
                    .edge_evidence_row_key
                    .evidence_site_occurrence_key
                    .read_semantic_occurrence_parts()
                    .span_expansion_provenance_chain
                    .is_empty()
                && edge.edge_resolution_value
                    == HldEdgeResolutionValue::Unresolved {
                        reason: HldUnresolvedReasonKind::TraitSolverNeeded,
                    }
        })
        .count();
    assert_eq!(pending, 0);
}

/// REQ-RUST-052.0 G3: a `reads` row sited in expanded text names where its first and its
/// last token were copied from in `src/lib.rs`, apart: the macro definition, or the call.
/// A row sited in original text has no such record.
#[test]
fn test_generated_read_source_origins() {
    let (_directory, root, _database, input) = run_original_graph_fixture();
    let source = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    let mut shapes = std::collections::BTreeMap::<(String, String), usize>::new();
    for edge in input
        .input_edge_records
        .iter()
        .filter(|edge| edge.edge_relation_kind_value == HldRelationKindValue::Reads)
    {
        let metadata: Value =
            serde_json::from_str(edge.edge_dispatch_metadata_text.as_deref().unwrap()).unwrap();
        let expanded = !edge
            .edge_evidence_row_key
            .evidence_site_occurrence_key
            .read_semantic_occurrence_parts()
            .span_expansion_provenance_chain
            .is_empty();
        let origin = &metadata["expansion_source_origin"];
        if !expanded {
            assert!(origin.is_null(), "{metadata}");
            continue;
        }
        let read = |token: &Value| {
            assert_eq!(token["token_source_relative_path"], "src/lib.rs");
            let range = &token["token_source_byte_range"];
            (
                token["token_source_origin_kind"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
                source[range[0].as_u64().unwrap() as usize..range[1].as_u64().unwrap() as usize]
                    .to_owned(),
            )
        };
        let first = read(&origin["site_first_token_origin"]);
        let last = read(&origin["site_last_token_origin"]);
        let text = metadata["read_source_text"].as_str().unwrap();
        assert!(
            text.starts_with(&first.1) && text.ends_with(&last.1),
            "{text}"
        );
        assert_eq!(last.0, "definition", "{text}");
        *shapes
            .entry((format!("{} {}", first.0, first.1), last.1))
            .or_default() += 1;
    }
    let count = |first: &str, last: &str| {
        shapes
            .iter()
            .filter(|((start, end), _)| start == first && end == last)
            .map(|(_, count)| *count)
            .sum::<usize>()
    };
    assert_eq!(count("definition DECIMAL_PAIRS", "DECIMAL_PAIRS"), 24);
    assert_eq!(shapes.values().sum::<usize>(), 34);
    // `Self::MAX_STR_LEN` is written in the macro definition; in `$Unsigned::MAX_STR_LEN` the
    // type name is copied from the call and the const name from the definition.
    let from_call = shapes
        .iter()
        .filter(|((first, last), _)| first.starts_with("invocation u") && last == "MAX_STR_LEN")
        .map(|(_, count)| *count)
        .sum::<usize>();
    assert_eq!(
        (count("definition Self", "MAX_STR_LEN"), from_call),
        (10 - from_call, from_call)
    );
    assert!(from_call > 0, "{shapes:?}");
}

/// REQ-RUST-052.0 G1, G2, G3: the signature of a function written by a macro expansion is a
/// source of `uses_type` rows, sited in the expanded text of its own node and naming where
/// the site was copied from. Each of the 12 generated `write` names its impl's own `Buffer`
/// and `str`; each of the 4 generated `fmt` names `usize` and, without a target until G5,
/// `Self::Buffer` of the supertrait.
#[test]
fn test_generated_signature_type_uses() {
    let (_directory, root, _database, input) = run_original_graph_fixture();
    let source = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    let label = |reference: &HldTargetIdentityValue| {
        input
            .input_node_records
            .iter()
            .find(|node| &node.node_target_reference == reference)
            .map(|node| node.node_qualified_path_text.clone())
            .unwrap()
    };
    let mut shapes = std::collections::BTreeMap::<(String, String, String), usize>::new();
    for node in input
        .input_node_records
        .iter()
        .filter(|node| node.node_generating_call_reference.is_some())
    {
        let HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(key)) =
            &node.node_target_reference
        else {
            panic!("definition identity expected");
        };
        let owner = key.read_semantic_occurrence_parts();
        let (owner_start, owner_end) = owner.span_byte_range_value.read_source_range_pair();
        for edge in input.input_edge_records.iter().filter(|edge| {
            edge.edge_relation_kind_value == HldRelationKindValue::UsesType
                && edge.edge_source_reference == node.node_target_reference
        }) {
            let site = edge
                .edge_evidence_row_key
                .evidence_site_occurrence_key
                .read_semantic_occurrence_parts();
            let (start, end) = site.span_byte_range_value.read_source_range_pair();
            assert_eq!(site.span_source_file_path, owner.span_source_file_path);
            assert_eq!(
                site.span_expansion_provenance_chain,
                owner.span_expansion_provenance_chain
            );
            assert!(owner_start <= start && end <= owner_end);
            let metadata: Value =
                serde_json::from_str(edge.edge_dispatch_metadata_text.as_deref().unwrap()).unwrap();
            let text = metadata["type_source_text"].as_str().unwrap();
            let origin = &metadata["expansion_source_origin"];
            let copied = |token: &Value| {
                assert_eq!(token["token_source_relative_path"], "src/lib.rs");
                assert_eq!(token["token_source_origin_kind"], "definition");
                let range = &token["token_source_byte_range"];
                &source[range[0].as_u64().unwrap() as usize..range[1].as_u64().unwrap() as usize]
            };
            assert!(text.starts_with(copied(&origin["site_first_token_origin"])));
            assert!(text.ends_with(copied(&origin["site_last_token_origin"])));
            let target = match &edge.edge_resolution_value {
                HldEdgeResolutionValue::Extracted { target } => match target {
                    HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Builtin(name)) => {
                        format!("builtin {name}")
                    }
                    target => label(target),
                },
                HldEdgeResolutionValue::Unresolved { reason } => format!("{reason:?}"),
                other => panic!("{other:?}"),
            };
            let member = node
                .node_qualified_path_text
                .rsplit("::")
                .next()
                .unwrap()
                .to_owned();
            let own = node
                .node_qualified_path_text
                .strip_suffix("write")
                .is_some_and(|header| target == format!("{header}Buffer"));
            let sibling = node
                .node_qualified_path_text
                .strip_suffix(" as Unsigned>::fmt")
                .is_some_and(|header| target == format!("{header} as private::Sealed>::Buffer"));
            let target = if own {
                "own Buffer".to_owned()
            } else if sibling {
                "Buffer of the type's Sealed impl".to_owned()
            } else {
                target
            };
            *shapes
                .entry((member, text.replace(' ', ""), target))
                .or_default() += 1;
        }
    }
    let shape = |member: &str, text: &str, target: &str| {
        (
            (member.to_owned(), text.to_owned(), target.to_owned()),
            match member {
                "write" => 12,
                _ => 4,
            },
        )
    };
    assert_eq!(
        shapes,
        std::collections::BTreeMap::from([
            shape("fmt", "Self::Buffer", "Buffer of the type's Sealed impl"),
            shape("fmt", "usize", "builtin usize"),
            shape("write", "Self::Buffer", "own Buffer"),
            shape("write", "str", "builtin str"),
        ])
    );
    assert_eq!(
        input
            .input_edge_records
            .iter()
            .filter(|edge| edge.edge_relation_kind_value == HldRelationKindValue::UsesType)
            .count(),
        66,
        "34 from original signatures and 32 from generated ones"
    );
}

/// REQ-RUST-052.0 G1, G2, G3, G6: the body of an item written by a macro expansion is a
/// source of `calls` rows. Every `calls` row, original or generated, carries the call's
/// text, the callee's text, the call's shape and how its target was chosen; a row without a
/// target says what stopped the lookup. A generated row is sited in the expanded text of
/// its own node and names where its first and last token were copied from.
#[test]
fn test_generated_body_call_rows() {
    let (_directory, _root, _database, input) = run_original_graph_fixture();
    let generated_nodes = input
        .input_node_records
        .iter()
        .filter(|node| node.node_generating_call_reference.is_some())
        .map(|node| &node.node_target_reference)
        .collect::<std::collections::BTreeSet<_>>();
    let calls = input
        .input_edge_records
        .iter()
        .filter(|edge| edge.edge_relation_kind_value == HldRelationKindValue::Calls)
        .collect::<Vec<_>>();
    let mut shapes = std::collections::BTreeMap::<String, usize>::new();
    for edge in &calls {
        let site = edge
            .edge_evidence_row_key
            .evidence_site_occurrence_key
            .read_semantic_occurrence_parts();
        let HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(owner)) =
            &edge.edge_source_reference
        else {
            panic!("definition identity expected");
        };
        let owner = owner.read_semantic_occurrence_parts();
        assert_eq!(site.span_source_file_path, owner.span_source_file_path);
        assert_eq!(
            site.span_expansion_provenance_chain,
            owner.span_expansion_provenance_chain
        );
        let expanded = !site.span_expansion_provenance_chain.is_empty();
        assert_eq!(
            expanded,
            generated_nodes.contains(&edge.edge_source_reference)
        );
        let metadata: Value =
            serde_json::from_str(edge.edge_dispatch_metadata_text.as_deref().unwrap()).unwrap();
        let text = metadata["call_source_text"].as_str().unwrap();
        let callee = metadata["callee_source_text"].as_str().unwrap();
        assert!(!callee.is_empty() && text.contains(callee), "{metadata}");
        let shape = metadata["call_shape_kind"].as_str().unwrap();
        let kind = metadata["dispatch_kind"].as_str().unwrap();
        let detail = metadata["unresolved_detail_kind"].as_str();
        match &edge.edge_resolution_value {
            HldEdgeResolutionValue::Extracted { .. } => {
                assert!(detail.is_none(), "{metadata}");
                assert!(
                    [
                        "generic_bound_trait_method",
                        "type_path_associated_function",
                        "callee_path_function",
                        "trait_path_method",
                        "trait_path_impl_method",
                        "cast_receiver_impl_method",
                        "tuple_struct_constructor"
                    ]
                    .contains(&kind)
                );
            }
            _ => {
                assert_eq!(kind, "undetermined", "{metadata}");
                assert!(detail.is_some(), "{metadata}");
            }
        }
        assert_eq!(
            metadata["expansion_source_origin"].is_object(),
            expanded,
            "{metadata}"
        );
        if expanded {
            let name = match shape {
                "method_call" => "method call".to_owned(),
                _ => metadata["call_syntax_evidence"]["member_declared_name_value"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
            };
            *shapes
                .entry(format!("{shape} {name} {}", detail.unwrap_or("-")))
                .or_default() += 1;
        }
    }
    assert_eq!(
        shapes,
        std::collections::BTreeMap::from([
            // Rule G9: the callee path names one function of this crate.
            ("bare_name_call divmod100 -".to_owned(), 8),
            ("bare_name_call slice_buffer_to_str -".to_owned(), 10),
            // Rule G13: the receiver is a cast to a stored type.
            ("method_call method call -".to_owned(), 2),
            (
                "method_call method call receiver_type_not_determined".to_owned(),
                102
            ),
            // Rule G10: the trait's method declaration, named through the trait.
            ("qualified_path_call fmt -".to_owned(), 10),
            (
                "qualified_path_call size_of type_prefix_unresolved".to_owned(),
                4
            ),
        ])
    );
    assert_eq!(calls.len(), 197, "61 in original bodies, 136 in generated");
}

/// REQ-RUST-052.0 G5: a member reached through a type (`Self::NAME`, `u8::NAME`) has the
/// member of the one stored trait impl of that type as its target, and the row says which
/// impl proved it and what was not read.
#[test]
fn test_type_relative_member_targets() {
    let (_directory, _root, _database, input) = run_original_graph_fixture();
    let label = |reference: &HldTargetIdentityValue| {
        input
            .input_node_records
            .iter()
            .find(|node| &node.node_target_reference == reference)
            .map(|node| node.node_qualified_path_text.clone())
            .unwrap()
    };
    // The type a member impl is for, read from its label `..::<T as Trait>::name`.
    let self_type = |text: &str| {
        text.rsplit_once("::<")
            .and_then(|(_, rest)| rest.split_once(" as "))
            .map(|(name, _)| name.to_owned())
            .unwrap()
    };
    let check = |relation, text_field: &str, member: &str, trait_text: &str, owner: &str| {
        input
            .input_edge_records
            .iter()
            .filter(|edge| edge.edge_relation_kind_value == relation)
            .filter_map(|edge| {
                let metadata: Value =
                    serde_json::from_str(edge.edge_dispatch_metadata_text.as_deref()?).ok()?;
                let text = metadata[text_field].as_str()?.replace(' ', "");
                let (prefix, name) = text.split_once("::")?;
                (name == member && prefix != "I")
                    .then(|| (edge, metadata.clone(), prefix.to_owned()))
            })
            .map(|(edge, metadata, prefix)| {
                (label(&edge.edge_source_reference), edge, metadata, prefix)
            })
            .filter(|(source, ..)| source.contains(owner))
            .map(|(source, edge, metadata, prefix)| {
                let named = if prefix == "Self" {
                    self_type(&source)
                } else {
                    prefix.clone()
                };
                let HldEdgeResolutionValue::Extracted { target } = &edge.edge_resolution_value
                else {
                    panic!(
                        "{source} {prefix}::{member} has no target: {:?}",
                        edge.edge_resolution_value
                    );
                };
                assert_eq!(
                    label(target),
                    format!("itoa::src/lib.rs::<{named} as {trait_text}>::{member}"),
                    "{source}"
                );
                let proof = &metadata["type_relative_member_evidence"];
                assert_eq!(proof["self_type_source_text"], prefix.as_str(), "{source}");
                assert_eq!(
                    proof["member_lookup_outcome_kind"], "single_trait_impl_member",
                    "{source}"
                );
                assert_eq!(
                    proof["member_candidate_impl_rows"].as_array().map(Vec::len),
                    Some(1),
                    "{source}"
                );
                assert_eq!(proof["self_type_builtin_name"], named.as_str(), "{source}");
                assert_eq!(
                    proof["impl_trait_source_text"]
                        .as_str()
                        .map(|text| text.replace(' ', ""))
                        .as_deref(),
                    Some(trait_text),
                    "{source}"
                );
                assert_eq!(
                    proof["sources_not_read"],
                    json!([
                        format!("inherent items of builtin type {named}"),
                        format!("trait impls for {named} in crates whose source was not read")
                    ]),
                    "{source}"
                );
                (
                    prefix == "Self",
                    source.contains('#')
                        || !edge
                            .edge_evidence_row_key
                            .evidence_site_occurrence_key
                            .read_semantic_occurrence_parts()
                            .span_expansion_provenance_chain
                            .is_empty(),
                )
            })
            .fold(
                std::collections::BTreeMap::<(bool, bool), usize>::new(),
                |mut counts, key| {
                    *counts.entry(key).or_default() += 1;
                    counts
                },
            )
    };
    // (through Self, sited in expanded text) -> rows
    assert_eq!(
        check(
            HldRelationKindValue::Reads,
            "read_source_text",
            "MAX_STR_LEN",
            "Integer",
            ""
        ),
        std::collections::BTreeMap::from([
            ((true, true), 5),
            ((false, true), 5),
            ((false, false), 6)
        ])
    );
    // `Self::Buffer` in `impl Unsigned for uN`: the alias is a member of another impl of uN.
    assert_eq!(
        check(
            HldRelationKindValue::UsesType,
            "type_source_text",
            "Buffer",
            "private::Sealed",
            " as Unsigned>::fmt"
        ),
        std::collections::BTreeMap::from([((true, true), 4), ((true, false), 1)])
    );
}

/// REQ-RUST-052.0 G10, G11, G12: a trait's method called through the trait
/// (`Unsigned::fmt(x, ..)`) has the trait's method declaration as target and lists the
/// stored impl members that could run; when its first argument is the calling method's
/// own by-value `self`, the member of the impl for that method's type is the target. A
/// tuple struct called as a function (`DecimalPairs(..)`) has the struct as target.
#[test]
fn test_trait_path_call_targets() {
    let (_directory, _root, _database, input) = run_original_graph_fixture();
    let label = |reference: &HldTargetIdentityValue| {
        input
            .input_node_records
            .iter()
            .find(|node| &node.node_target_reference == reference)
            .map(|node| node.node_qualified_path_text.clone())
            .unwrap()
            .replace("itoa::src/lib.rs::", "")
    };
    let mut found = std::collections::BTreeSet::<String>::new();
    for edge in input
        .input_edge_records
        .iter()
        .filter(|edge| edge.edge_relation_kind_value == HldRelationKindValue::Calls)
    {
        let metadata: Value =
            serde_json::from_str(edge.edge_dispatch_metadata_text.as_deref().unwrap()).unwrap();
        let callee = metadata["callee_source_text"]
            .as_str()
            .unwrap()
            .replace(' ', "");
        if !["Unsigned::fmt", "DecimalPairs"].contains(&callee.as_str()) {
            continue;
        }
        let caller = label(&edge.edge_source_reference);
        let HldEdgeResolutionValue::Extracted { target } = &edge.edge_resolution_value else {
            panic!("{caller} calls {callee} without a target: {metadata}");
        };
        assert!(metadata["unresolved_detail_kind"].is_null(), "{metadata}");
        let record = &metadata["trait_method_lookup_evidence"];
        if callee == "Unsigned::fmt" {
            // Every row lists the same five impl members that could run.
            let candidates = record["trait_member_candidate_rows"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| row["impl_self_type_source_text"].as_str().unwrap())
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(
                candidates.into_iter().collect::<Vec<_>>().join(","),
                "u128,u16,u32,u64,u8",
                "{metadata}"
            );
        } else {
            assert!(record.is_null(), "{metadata}");
        }
        assert!(
            found.insert(format!(
                "{caller} | {} | {} {} {} {}",
                label(target),
                metadata["dispatch_kind"].as_str().unwrap(),
                record["trait_member_outcome_kind"].as_str().unwrap_or("-"),
                record["receiver_type_stored_flag"],
                record["receiver_self_type_source_text"]
                    .as_str()
                    .unwrap_or("-")
            )),
            "{metadata}"
        );
    }
    let by_self = |ty: &str| {
        format!(
            "<{ty} as private::Sealed>::write | <{ty} as Unsigned>::fmt | \
             trait_path_impl_method single_impl_member_by_self true {ty}"
        )
    };
    let by_trait = |ty: &str| {
        format!(
            "<{ty} as private::Sealed>::write | Unsigned::fmt | \
             trait_path_method trait_method_declaration false -"
        )
    };
    assert_eq!(
        found,
        ["u8", "u16", "u32", "u64", "u128"]
            .into_iter()
            .map(by_self)
            .chain(
                ["i8", "i16", "i32", "i64", "i128"]
                    .into_iter()
                    .map(by_trait)
            )
            .chain(["DECIMAL_PAIRS | DecimalPairs | tuple_struct_constructor - null -".to_owned()])
            .collect::<std::collections::BTreeSet<_>>()
    );
}

/// REQ-RUST-052.0 G13: a method call whose receiver is a cast to a stored type
/// (`(self as u64).write(buf)`) has the one stored impl member of that type as target,
/// when it is a member of the trait whose impl holds the call. The row keeps the cast's
/// type, the candidates and what was not read.
#[test]
fn test_cast_receiver_call_targets() {
    let (_directory, _root, _database, input) = run_original_graph_fixture();
    let label = |reference: &HldTargetIdentityValue| {
        input
            .input_node_records
            .iter()
            .find(|node| &node.node_target_reference == reference)
            .map(|node| node.node_qualified_path_text.clone())
            .unwrap()
            .replace("itoa::src/lib.rs::", "")
    };
    let mut found = std::collections::BTreeSet::<String>::new();
    for edge in input
        .input_edge_records
        .iter()
        .filter(|edge| edge.edge_relation_kind_value == HldRelationKindValue::Calls)
    {
        let metadata: Value =
            serde_json::from_str(edge.edge_dispatch_metadata_text.as_deref().unwrap()).unwrap();
        let receiver = &metadata["call_syntax_evidence"]["method_receiver_evidence"];
        if receiver["receiver_shape_kind"] != "cast_expression" {
            continue;
        }
        let caller = label(&edge.edge_source_reference);
        let HldEdgeResolutionValue::Extracted { target } = &edge.edge_resolution_value else {
            panic!("{caller} has a cast receiver call without a target: {metadata}");
        };
        let record = &metadata["cast_receiver_lookup_evidence"];
        assert_eq!(
            record["sources_not_read"].as_array().map(Vec::len),
            Some(2),
            "{metadata}"
        );
        assert!(
            found.insert(format!(
                "{caller} | {} | {} {} {} {}",
                label(target),
                metadata["dispatch_kind"].as_str().unwrap(),
                receiver["receiver_cast_type_evidence"]["type_source_text"]
                    .as_str()
                    .unwrap(),
                record["member_lookup_outcome_kind"].as_str().unwrap(),
                // Expanded text is written with a space before `::`.
                record["impl_trait_source_text"]
                    .as_str()
                    .unwrap()
                    .replace(' ', "")
            )),
            "{metadata}"
        );
    }
    let row = |from: &str, to: &str| {
        format!(
            "<{from} as private::Sealed>::write | <{to} as private::Sealed>::write | \
             cast_receiver_impl_method {to} single_trait_impl_member private::Sealed"
        )
    };
    assert_eq!(
        found,
        [row("isize", "i64"), row("usize", "u64")]
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
    );
}

/// REQ-RUST-052.0 G9: a call whose callee is a path that name lookup resolves to one
/// function of this crate has that function as its target: a bare name
/// (`divmod100(..)`), a bare name with generic arguments (`enc_16lsd::<..>(..)`), and
/// a module path (`u128_ext::mulhi(..)`), in original and in generated bodies alike.
#[test]
fn test_callee_path_function_targets() {
    let (_directory, _root, _database, input) = run_original_graph_fixture();
    let label = |reference: &HldTargetIdentityValue| {
        input
            .input_node_records
            .iter()
            .find(|node| &node.node_target_reference == reference)
            .map(|node| node.node_qualified_path_text.clone())
            .unwrap()
    };
    // Every call whose callee path was looked up keeps the record of that lookup, with
    // or without a target: (callee, outcome, symbol kinds found, qualifier's kind).
    let mut records = std::collections::BTreeMap::<String, usize>::new();
    for edge in input
        .input_edge_records
        .iter()
        .filter(|edge| edge.edge_relation_kind_value == HldRelationKindValue::Calls)
    {
        let metadata: Value =
            serde_json::from_str(edge.edge_dispatch_metadata_text.as_deref().unwrap()).unwrap();
        let record = &metadata["callee_path_lookup_evidence"];
        if record.is_null() {
            assert_eq!(metadata["call_shape_kind"], "method_call", "{metadata}");
            continue;
        }
        let callee = metadata["callee_source_text"].as_str().unwrap();
        *records
            .entry(format!(
                "{} {} {} {}",
                callee.split("::<").next().unwrap().replace(' ', ""),
                record["callee_path_outcome_kind"].as_str().unwrap(),
                record["callee_target_symbol_kinds"],
                record["callee_qualifier_symbol_kind"]
            ))
            .or_default() += 1;
    }
    assert_eq!(
        records,
        std::collections::BTreeMap::from([
            (
                "Buffer::new callee_path_unresolved [] \"Struct\"".to_owned(),
                2
            ),
            (
                "DecimalPairs single_struct_node [\"Struct\"] null".to_owned(),
                1
            ),
            ("MaybeUninit callee_path_unresolved [] null".to_owned(), 1),
            (
                "Unsigned::fmt single_trait_method [\"Function\"] \"Trait\"".to_owned(),
                10
            ),
            (
                "div_rem_1e16 single_function_node [\"Function\"] null".to_owned(),
                2
            ),
            (
                "divmod100 single_function_node [\"Function\"] null".to_owned(),
                12
            ),
            (
                "enc_16lsd single_function_node [\"Function\"] null".to_owned(),
                2
            ),
            (
                "hint::unreachable_unchecked callee_path_unresolved [] null".to_owned(),
                1
            ),
            ("mem::size_of callee_path_unresolved [] null".to_owned(), 4),
            (
                "slice_buffer_to_str single_function_node [\"Function\"] null".to_owned(),
                10
            ),
            (
                "str::from_utf8_unchecked callee_path_unresolved [] \"Builtin\"".to_owned(),
                1
            ),
            (
                "u128::from callee_path_unresolved [] \"Builtin\"".to_owned(),
                9
            ),
            (
                "u128_ext::mulhi single_function_node [\"Function\"] \"Module\"".to_owned(),
                1
            ),
        ]),
        "56 calls with a looked-up callee path"
    );
    let mut found = std::collections::BTreeMap::<(String, String, bool), usize>::new();
    for edge in input
        .input_edge_records
        .iter()
        .filter(|edge| edge.edge_relation_kind_value == HldRelationKindValue::Calls)
    {
        let metadata: Value =
            serde_json::from_str(edge.edge_dispatch_metadata_text.as_deref().unwrap()).unwrap();
        let callee = metadata["callee_source_text"].as_str().unwrap();
        let name = callee.split("::<").next().unwrap().replace(' ', "");
        if ![
            "divmod100",
            "slice_buffer_to_str",
            "div_rem_1e16",
            "enc_16lsd",
            "u128_ext::mulhi",
        ]
        .contains(&name.as_str())
        {
            continue;
        }
        let HldEdgeResolutionValue::Extracted { target } = &edge.edge_resolution_value else {
            panic!(
                "{} calls {callee} without a target: {metadata}",
                label(&edge.edge_source_reference)
            );
        };
        assert_eq!(
            metadata["dispatch_kind"], "callee_path_function",
            "{metadata}"
        );
        assert!(metadata["unresolved_detail_kind"].is_null(), "{metadata}");
        let expanded = metadata["expansion_source_origin"].is_object();
        *found.entry((name, label(target), expanded)).or_default() += 1;
    }
    let row = |name: &str, target: &str, expanded| (name.to_owned(), target.to_owned(), expanded);
    assert_eq!(
        found,
        std::collections::BTreeMap::from([
            (
                row("div_rem_1e16", "itoa::src/lib.rs::div_rem_1e16", false),
                2
            ),
            (row("divmod100", "itoa::src/lib.rs::divmod100", false), 4),
            (row("divmod100", "itoa::src/lib.rs::divmod100", true), 8),
            (row("enc_16lsd", "itoa::src/lib.rs::enc_16lsd", false), 2),
            (
                row(
                    "slice_buffer_to_str",
                    "itoa::src/lib.rs::slice_buffer_to_str",
                    true
                ),
                10
            ),
            (
                row("u128_ext::mulhi", "itoa::src/u128_ext.rs::mulhi", false),
                1
            ),
        ]),
        "27 calls: (callee, target, written in an expansion)"
    );
}

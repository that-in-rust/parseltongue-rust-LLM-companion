#![allow(clippy::unwrap_used)]

use parseltongue_core_row_types::{RunCancelFlagValue, StageRunScopeValue, StageUnitWriteRequest};
use parseltongue_table_store_traits::StageTableStorePort;
use parseltongue_turso_store_adapter::TursoWorkspaceStoreAdapter;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::{fs, path::Path, process::Command};

fn create_workspace_source_fixture(root: &Path) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("Cargo.toml"), "[package]\nname=\"fixture\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[features]\ndefault=[\"enabled\"]\nenabled=[]\n").unwrap();
    fs::write(
        root.join("Cargo.lock"),
        "version = 4\n[[package]]\nname = \"fixture\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(root.join("src/lib.rs"), "mod child; #[cfg(test)] mod absent; pub struct A { pub field: u8 } pub fn f(x: u8) -> u8 { child::g(x) }").unwrap();
    fs::write(root.join("src/child.rs"), "pub fn g(x: u8) -> u8 { x }").unwrap();
    fs::write(root.join("unrelated.rs"), [255, 0]).unwrap();
    fs::write(
        root.join("build.rs"),
        "compile_error!(\"never execute me\");",
    )
    .unwrap();
    let lock = Command::new(env!("CARGO"))
        .current_dir(root)
        .args(["generate-lockfile", "--offline"])
        .output()
        .unwrap();
    assert!(
        lock.status.success(),
        "{}",
        String::from_utf8_lossy(&lock.stderr)
    );
}
fn run_workspace_mapping_fixture(
    root: &Path,
    database: &Path,
    workers: &str,
) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_parseltongue"))
        .args([
            "map-workspace-full-run",
            "--workspace-root-path-value",
            root.to_str().unwrap(),
            "--workspace-store-file-path",
            database.to_str().unwrap(),
            "--worker-thread-count-value",
            workers,
        ])
        .output()
        .unwrap()
}

fn run_workspace_stage_fixture(root: &Path, database: &Path, stage: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_parseltongue"))
        .args([
            "map-workspace-full-run",
            "--workspace-root-path-value",
            root.to_str().unwrap(),
            "--workspace-store-file-path",
            database.to_str().unwrap(),
            "--worker-thread-count-value",
            "2",
            "--mapping-final-stage-kind",
            stage,
        ])
        .output()
        .unwrap()
}

fn write_stale_completion_marker(
    store: &TursoWorkspaceStoreAdapter,
    scope: &StageRunScopeValue,
    stage: &str,
    inputs: &[&str],
) {
    store
        .write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: scope,
                stage_owner_name_value: stage,
                stage_unit_work_key: "__complete__",
                stage_input_stage_names: inputs,
                stage_unit_output_rows: &[],
            },
            &RunCancelFlagValue::default(),
        )
        .unwrap();
}
fn read_workspace_banner_fixture(database: &Path) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_parseltongue"))
        .args([
            "status-coverage-banner-show",
            "--json",
            "--workspace-store-file-path",
            database.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn read_published_table_fixture(database: &Path, table: &str) -> Vec<Value> {
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let scope = store.read_workspace_publication_scope().unwrap().unwrap();
    store
        .read_stage_table_rows(&scope, table)
        .unwrap()
        .iter()
        .map(|row| serde_json::from_str(&row.stage_row_payload_text).unwrap())
        .collect()
}

fn fixture_compilation_unit_identity(database: &Path) -> (String, Value) {
    let unit = read_published_table_fixture(database, "crate-dependency-graph-table")
        .into_iter()
        .find(|unit| {
            unit["unit_cargo_target_kind"] == "lib"
                && unit["unit_compilation_role_kind"] == "target"
                && unit["unit_cargo_target_name"] == "fixture"
        })
        .unwrap();
    let configuration = json!([
        unit["unit_target_cfg_values"],
        unit["unit_enabled_feature_values"],
        unit["unit_rust_edition_value"],
        unit["unit_compilation_role_kind"],
    ]);
    let identity = json!({
        "unit_cargo_package_identifier":unit["unit_cargo_package_identifier"],
        "unit_cargo_target_name":unit["unit_cargo_target_name"],
        "unit_cargo_target_kind":unit["unit_cargo_target_kind"],
        "unit_compilation_role_kind":unit["unit_compilation_role_kind"],
        "unit_configuration_digest_full":blake3::hash(configuration.to_string().as_bytes()).to_hex().to_string(),
    });
    (
        unit["unit_compilation_identity_key"]
            .as_str()
            .unwrap()
            .to_owned(),
        identity,
    )
}

fn expected_macro_invocation_identity(database: &Path, source: &str, invocation: &str) -> Value {
    let (_, unit_identity) = fixture_compilation_unit_identity(database);
    let start = source.find(invocation).unwrap();
    json!({
        "unit_compilation_identity_key":unit_identity,
        "span_source_file_path":"src/lib.rs",
        "span_byte_range_value":[start, start + invocation.len()],
        "span_content_digest_full":blake3::hash(invocation.as_bytes()).to_hex().to_string(),
        "span_expansion_provenance_chain":[],
    })
}

fn expected_syntax_item_owner(
    database: &Path,
    root: &Path,
    source: &str,
    item_source: &str,
    syntax_kind: &str,
    provenance: Value,
) -> String {
    let (unit_compilation_identity_key, _) = fixture_compilation_unit_identity(database);
    let start = source.find(item_source).unwrap();
    let end = start + item_source.len();
    let path = root.join("src/lib.rs").canonicalize().unwrap();
    let canonical = json!([
        unit_compilation_identity_key,
        path.to_string_lossy(),
        start,
        end,
        syntax_kind,
        provenance,
    ]);
    blake3::hash(canonical.to_string().as_bytes())
        .to_hex()
        .to_string()
}

fn expected_virtual_occurrence_owner(
    database: &Path,
    source_path: &Path,
    source: &str,
    item_source: &str,
    syntax_kind: &str,
    provenance: Value,
) -> String {
    let (unit_identity, _) = fixture_compilation_unit_identity(database);
    let start = source.find(item_source).unwrap();
    let end = start + item_source.len();
    let canonical = json!([
        unit_identity,
        source_path.to_string_lossy(),
        start,
        end,
        syntax_kind,
        provenance,
    ]);
    blake3::hash(canonical.to_string().as_bytes())
        .to_hex()
        .to_string()
}

fn expected_local_block_scope(
    database: &Path,
    source_path: &Path,
    source: &str,
    function_source: &str,
    block_source: &str,
) -> Value {
    let (_, unit_identity) = fixture_compilation_unit_identity(database);
    let scope_part = |label: &str, item_source: &str| {
        let start = source.find(item_source).unwrap();
        let end = start + item_source.len();
        let digest = blake3::hash(
            json!([source_path.to_string_lossy(), start, end])
                .to_string()
                .as_bytes(),
        )
        .to_hex()
        .to_string();
        format!("@{label}:{digest}")
    };
    json!({
        "scope_compilation_unit_key":unit_identity,
        "scope_semantic_path_parts":[
            scope_part("FN", function_source),
            scope_part("BLOCK_EXPR", block_source),
        ],
    })
}

fn expected_macro_provenance_chain(invocation_identity: &Value) -> Value {
    json!([blake3::hash(invocation_identity.to_string().as_bytes())
        .to_hex()
        .to_string()])
}

fn expected_nested_macro_identity(
    database: &Path,
    source_path: &str,
    source: &str,
    invocation: &str,
    provenance: &Value,
) -> Value {
    let (_, unit_identity) = fixture_compilation_unit_identity(database);
    let start = source.find(invocation).unwrap();
    json!({
        "unit_compilation_identity_key":unit_identity,
        "span_source_file_path":source_path,
        "span_byte_range_value":[start, start + invocation.len()],
        "span_content_digest_full":blake3::hash(invocation.as_bytes()).to_hex().to_string(),
        "span_expansion_provenance_chain":provenance,
    })
}

fn expected_macro_virtual_path(root: &Path, invocation_identity: &Value) -> std::path::PathBuf {
    let digest = blake3::hash(invocation_identity.to_string().as_bytes())
        .to_hex()
        .to_string();
    root.join(".parseltongue/macro-source")
        .join(digest)
        .join("expanded.rs")
}

fn extend_macro_provenance_chain(parent: &Value, invocation_identity: &Value) -> Value {
    let mut chain = parent.as_array().unwrap().clone();
    chain.push(json!(
        blake3::hash(invocation_identity.to_string().as_bytes())
            .to_hex()
            .to_string()
    ));
    Value::Array(chain)
}

fn read_rust_token_ranges(source: &str) -> Vec<(usize, usize)> {
    let bytes = source.as_bytes();
    let mut ranges = Vec::new();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
            continue;
        }
        let start = cursor;
        if bytes[cursor].is_ascii_alphanumeric() || bytes[cursor] == b'_' {
            cursor += 1;
            while cursor < bytes.len()
                && (bytes[cursor].is_ascii_alphanumeric() || bytes[cursor] == b'_')
            {
                cursor += 1;
            }
        } else {
            cursor += 1;
        }
        ranges.push((start, cursor));
    }
    ranges
}

fn expected_macro_token_origins(
    source: &str,
    definition_template: &str,
    expansion_source: &str,
    definition_path: &Path,
) -> Vec<Value> {
    let template_start = source.find(definition_template).unwrap();
    let template_ranges = read_rust_token_ranges(definition_template);
    let expansion_ranges = read_rust_token_ranges(expansion_source);
    assert_eq!(template_ranges.len(), expansion_ranges.len());
    template_ranges
        .iter()
        .zip(expansion_ranges)
        .map(|((source_start, source_end), (output_start, output_end))| {
            assert_eq!(
                &definition_template[*source_start..*source_end],
                &expansion_source[output_start..output_end]
            );
            json!({
                "token_output_byte_range":[output_start, output_end],
                "token_original_byte_range":[
                    template_start + source_start,
                    template_start + source_end,
                ],
                "token_source_origin_kind":"definition",
                "token_source_absolute_path":definition_path.to_string_lossy(),
            })
        })
        .collect()
}

fn expected_macro_body_site(
    source_path: &Path,
    expansion_source: &str,
    site_text: &str,
    kind: &str,
    owner: &str,
    invocation_identity: &Value,
    provenance: &Value,
) -> Value {
    let start = expansion_source.find(site_text).unwrap();
    json!({
        "kind":kind,
        "range":[start, start + site_text.len()],
        "text":site_text,
        "source_path":source_path.to_string_lossy(),
        "owner":owner,
        "invocation":invocation_identity,
        "chain":provenance,
        "resolution":"pending",
        "unresolved":null,
    })
}

fn read_sorted_published_records(database: &Path, table: &str) -> Vec<Value> {
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let scope = store.read_workspace_publication_scope().unwrap().unwrap();
    let mut records = store
        .read_stage_table_rows(&scope, table)
        .unwrap()
        .into_iter()
        .map(|row| {
            json!({
                "row_key":row.stage_row_record_key,
                "payload":serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap(),
            })
        })
        .collect::<Vec<_>>();
    records.sort_by_key(Value::to_string);
    records
}

fn expected_definition_token_origin(
    path: &Path,
    output_start: usize,
    output_end: usize,
    source_start: usize,
    source_end: usize,
) -> Value {
    json!({
        "token_output_byte_range":[output_start, output_end],
        "token_original_byte_range":[source_start, source_end],
        "token_source_origin_kind":"definition",
        "token_source_absolute_path":path.to_string_lossy(),
    })
}

fn assert_receipt_product_rows(
    store: &TursoWorkspaceStoreAdapter,
    scope: &StageRunScopeValue,
    receipt: &Value,
) -> u64 {
    let expected_tables = [
        "name-resolution-scope-records",
        "module-scope-binding-table",
        "path-resolution-site-records",
    ];
    let proofs = receipt["analysis_product_content_proofs"]
        .as_object()
        .unwrap();
    let actual_tables = proofs.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected_table_set = expected_tables.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(actual_tables, expected_table_set);
    let mut total = 0;
    for table in expected_tables {
        let mut rows = store.read_stage_table_rows(scope, table).unwrap();
        rows.sort_unstable_by(|left, right| {
            left.stage_row_record_key.cmp(&right.stage_row_record_key)
        });
        let row_count = rows.len() as u64;
        assert_eq!(
            proofs[table]["row_count"].as_u64(),
            Some(row_count),
            "{table} row count"
        );
        assert!(
            rows.windows(2)
                .all(|pair| { pair[0].stage_row_record_key != pair[1].stage_row_record_key }),
            "{table} product keys must be unique"
        );
        let mut hasher = blake3::Hasher::new();
        rows.iter().for_each(|row| {
            hasher.update(&hash_published_row_leaf(
                table,
                &row.stage_row_record_key,
                &row.stage_row_payload_text,
            ));
        });
        let digest = hasher.finalize().to_hex().to_string();
        assert_eq!(
            proofs[table]["content_digest"].as_str(),
            Some(digest.as_str()),
            "{table} ordered digest"
        );
        total += row_count;
    }
    assert_eq!(
        receipt["analysis_product_record_count"].as_u64(),
        Some(total)
    );
    total
}

fn hash_published_row_leaf(table: &str, key: &str, payload: &str) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    [table, key, payload].into_iter().for_each(|value| {
        hasher.update(&(value.len() as u64).to_le_bytes());
        hasher.update(value.as_bytes());
    });
    *hasher.finalize().as_bytes()
}

#[test]
fn test_macro_items_publish_atomically() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(
        root.join("src/lib.rs"),
        r#"
macro_rules! outer { ($name:ident) => {
    macro_rules! inner { () => { pub struct $name { pub field:u8 } }; }
    inner!();
}; }
outer!(Generated);
pub fn take(_:Generated){}
"#,
    )
    .unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let macros = read_published_table_fixture(&database, "macro-expanded-item-table");
    assert_eq!(macros.len(), 2);
    let nested = macros
        .iter()
        .find(|row| {
            row["macro_invocation_provenance_chain"]
                .as_array()
                .unwrap()
                .len()
                == 2
        })
        .unwrap();
    assert!(
        nested["macro_expanded_source_text"]
            .as_str()
            .unwrap()
            .contains("Generated")
    );
    let path = read_published_table_fixture(&database, "path-resolution-site-records")
        .into_iter()
        .find(|row| row["path_site_source_text"] == "Generated")
        .unwrap();
    assert_eq!(path["path_resolution_state_kind"], "resolved");
    let targets = path["path_target_symbol_rows"].as_array().unwrap();
    assert_eq!(targets.len(), 1);
    assert!(
        targets[0]["target_source_file_path"]
            .as_str()
            .unwrap()
            .contains("macro-source")
    );
    assert!(
        read_published_table_fixture(&database, "span-reference-text-table")
            .iter()
            .any(|row| !row["span_expansion_provenance_chain"]
                .as_array()
                .is_none_or(|chain| chain.is_empty()))
    );
    assert!(
        read_published_table_fixture(&database, "field-signature-detail-table")
            .iter()
            .any(|row| !row["span_expansion_provenance_chain"]
                .as_array()
                .is_none_or(|chain| chain.is_empty()))
    );

    let before = read_workspace_banner_fixture(&database);
    fs::write(
        root.join("src/lib.rs"),
        "macro_rules! make {($name:ident)=>{pub struct $name;};} make!(Completed); make!(1);",
    )
    .unwrap();
    let failed = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(!failed.status.success());
    assert_eq!(read_workspace_banner_fixture(&database), before);
    assert_eq!(
        read_published_table_fixture(&database, "macro-expanded-item-table"),
        macros
    );
}

#[test]
fn test_macro_defining_crate_hygiene() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::create_dir_all(root.join("defs/src")).unwrap();
    fs::write(
        root.join("defs/Cargo.toml"),
        "[package]\nname=\"defs\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
    )
    .unwrap();
    fs::write(root.join("defs/src/lib.rs"),"pub struct Original; #[macro_export] macro_rules! build {($name:ident)=>{pub struct $name {pub value:$crate::Original}};}").unwrap();
    let manifest = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        format!("{manifest}\n[dependencies]\ndefs={{path=\"defs\"}}\n"),
    )
    .unwrap();
    fs::write(root.join("src/lib.rs"),"pub struct Original; defs::build!(First); defs::build!(Second); pub fn take(_:First,_:Second){}").unwrap();
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
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let macros = read_published_table_fixture(&database, "macro-expanded-item-table");
    assert_eq!(macros.len(), 2);
    assert_ne!(
        macros[0]["macro_virtual_source_path"],
        macros[1]["macro_virtual_source_path"]
    );
    let paths = read_published_table_fixture(&database, "path-resolution-site-records");
    let defining = paths
        .iter()
        .filter(|row| {
            row["path_site_source_text"].as_str().is_some_and(|text| {
                text.chars()
                    .filter(|ch| !ch.is_whitespace())
                    .collect::<String>()
                    == "$crate::Original"
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(defining.len(), 2);
    for path in defining {
        assert_eq!(path["path_resolution_state_kind"], "resolved");
        assert!(
            path["path_target_symbol_rows"][0]["target_source_file_path"]
                .as_str()
                .unwrap()
                .ends_with("/defs/src/lib.rs")
        );
    }
}

#[test]
fn test_macro_empty_virtual_file() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(
        root.join("src/lib.rs"),
        "macro_rules! empty {()=>{};} empty!();",
    )
    .unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows = read_published_table_fixture(&database, "macro-expanded-item-table");
    assert_eq!(rows.len(), 1);
    assert!(
        rows[0]["macro_expanded_source_text"]
            .as_str()
            .unwrap()
            .trim()
            .is_empty()
    );
    let files = read_published_table_fixture(&database, "file-source-text-table");
    assert!(files.iter().any(|row| row["file_source_absolute_path"]
        == rows[0]["macro_virtual_source_path"]
        && row["file_source_byte_count"] == 0));
}

#[test]
fn test_macro_declarations_publish_rows() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(root.join("src/lib.rs"),
        "#![feature(decl_macro)] macro make { (first)=>{pub struct First;}, (second)=>{pub struct Second;}, } make!(second); pub fn take(_:Second){}").unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        read_published_table_fixture(&database, "path-resolution-site-records")
            .iter()
            .any(|row| row["path_site_source_text"] == "Second"
                && row["path_resolution_state_kind"] == "resolved")
    );
}

#[test]
fn test_flags_without_sysroot_support() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::create_dir_all(root.join("defs/src")).unwrap();
    fs::write(
        root.join("defs/Cargo.toml"),
        "[package]\nname=\"defs\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
    )
    .unwrap();
    fs::write(root.join("defs/src/lib.rs"), "#![no_std] pub struct Item;").unwrap();
    let manifest = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        format!("{manifest}\n[dependencies]\ndep={{path=\"defs\",package=\"defs\"}}\n"),
    )
    .unwrap();
    fs::write(root.join("src/lib.rs"), "#![no_std] #![no_implicit_prelude] extern crate dep as local; pub fn take(_:local::Item,_:u8){}").unwrap();
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
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let units = read_published_table_fixture(&database, "crate-dependency-graph-table");
    assert!(
        units
            .iter()
            .all(|unit| unit["unit_sysroot_context_flag"] != true)
    );
    let unit = units
        .iter()
        .find(|unit| {
            unit["unit_package_root_path"] == root.to_str().unwrap()
                && unit["unit_cargo_target_kind"] == "lib"
                && unit["unit_compilation_role_kind"] == "target"
        })
        .unwrap();
    assert!(units.iter().any(
        |unit| unit["unit_package_root_path"] == root.to_str().unwrap()
            && unit["unit_cargo_target_kind"] == "custom-build"
            && unit["unit_mapped_target_flag"] == false
    ));
    assert_eq!(unit["unit_no_std_flag"], true);
    assert_eq!(unit["unit_implicit_prelude_flag"], false);
    let paths = read_published_table_fixture(&database, "path-resolution-site-records");
    for name in ["local::Item", "u8"] {
        assert!(
            paths.iter().any(|row| row["path_site_source_text"] == name
                && row["path_resolution_state_kind"] == "resolved"),
            "{name}"
        );
    }
}

#[test]
fn test_grouped_self_import_rules() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(root.join("src/lib.rs"), "pub mod defs {pub enum Choice {First}} use defs::Choice::{self as Alias, First}; pub fn take(_:Alias,_:defs::Choice){}").unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let paths = read_published_table_fixture(&database, "path-resolution-site-records");
    for name in ["defs::Choice", "Alias"] {
        assert!(
            paths.iter().any(|row| row["path_site_source_text"] == name
                && row["path_resolution_state_kind"] == "resolved"),
            "{name}"
        );
    }
}

#[test]
fn test_nested_macro_crate_provenance() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::create_dir_all(root.join("defs/src")).unwrap();
    fs::write(
        root.join("defs/Cargo.toml"),
        "[package]\nname=\"defs\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
    )
    .unwrap();
    fs::write(root.join("defs/src/lib.rs"),"pub struct Original; #[macro_export] macro_rules! outer {($name:ident)=>{macro_rules! inner {()=>{pub struct $name {pub field:$crate::Original}};} inner!();};}").unwrap();
    let manifest = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        format!("{manifest}\n[dependencies]\ndefs={{path=\"defs\"}}\n"),
    )
    .unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub struct Original; defs::outer!(Éclair); pub fn take(_:Éclair){}",
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
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let macros = read_published_table_fixture(&database, "macro-expanded-item-table");
    assert_eq!(macros.len(), 2);
    assert!(macros.iter().any(|row| {
        row["macro_invocation_provenance_chain"]
            .as_array()
            .unwrap()
            .len()
            == 2
    }));
    let paths = read_published_table_fixture(&database, "path-resolution-site-records");
    let path = paths
        .iter()
        .find(|row| {
            row["path_site_source_text"].as_str().is_some_and(|text| {
                text.chars()
                    .filter(|ch| !ch.is_whitespace())
                    .collect::<String>()
                    == "$crate::Original"
            })
        })
        .unwrap();
    assert_eq!(path["path_resolution_state_kind"], "resolved");
    assert!(
        path["path_target_symbol_rows"][0]["target_source_file_path"]
            .as_str()
            .unwrap()
            .ends_with("/defs/src/lib.rs")
    );
    assert_eq!(
        path["site_expansion_provenance_chain"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let generated = paths
        .iter()
        .find(|row| row["path_site_source_text"] == "Éclair")
        .unwrap();
    assert_eq!(generated["path_resolution_state_kind"], "resolved");
}

/// REQ-RUST-024.10
/// REQ-RUST-024.11
#[test]
fn test_persisted_resolution_path_answers() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    let source = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    fs::write(root.join("src/lib.rs"),format!("{source}\npub use child::T as PublicT; mod alias {{ pub use crate::PublicT as Again; pub fn make(x:Again)->Again{{x}} }}")).unwrap();
    fs::write(
        root.join("src/child.rs"),
        "pub struct T; pub fn g(x:u8)->u8{x}",
    )
    .unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let banner: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        banner["stage_publication_status_values"],
        serde_json::json!({
            "crate-graph": "complete",
            "source-ingest": "complete",
            "syntax-spans": "complete",
            "name-resolution": "complete"
        })
    );
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let scope = store.read_workspace_publication_scope().unwrap().unwrap();
    let rows: Vec<Value> = store
        .read_stage_table_rows(&scope, "path-resolution-site-records")
        .unwrap()
        .iter()
        .map(|row| serde_json::from_str(&row.stage_row_payload_text).unwrap())
        .collect();
    let answers: Vec<_> = rows
        .iter()
        .filter(|row| row["path_site_source_text"] == "Again")
        .collect();
    assert_eq!(answers.len(), 2);
    for answer in answers {
        assert_eq!(
            answer["site_compilation_scope_record"]["unit_module_path_values"],
            serde_json::json!(["alias"])
        );
        let targets = answer["path_target_symbol_rows"].as_array().unwrap();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0]["target_symbol_kind_value"], "Struct");
        assert!(
            targets[0]["target_source_file_path"]
                .as_str()
                .unwrap()
                .ends_with("/src/child.rs")
        );
    }
    assert!(
        !store
            .read_stage_table_rows(&scope, "module-scope-binding-table")
            .unwrap()
            .is_empty()
    );
    let receipt_rows = store
        .read_stage_table_rows(&scope, "analysis-resolution-receipt-records")
        .unwrap();
    assert_eq!(receipt_rows.len(), 1);
    let receipt: Value = serde_json::from_str(&receipt_rows[0].stage_row_payload_text).unwrap();
    let product_count = assert_receipt_product_rows(&store, &scope, &receipt);
    assert!(product_count > 0);
    drop(store);
    let banner = read_workspace_banner_fixture(&database);
    assert_eq!(
        banner["stage_implementation_status_values"]["name-resolution"],
        "complete"
    );
}

#[test]
fn test_builtin_include_name_resolution() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(root.join("src/lib.rs"),r#"include!(concat!(env!("CARGO_MANIFEST_DIR"),"/src/fixture.rs")); pub fn take(_:Included){}"#).unwrap();
    fs::write(root.join("src/fixture.rs"),"#[cfg_attr(not(test),cfg(feature=\"enabled\"))] pub struct Included {pub field:u8} #[cfg(test)] pub struct Excluded;").unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows = read_published_table_fixture(&database, "path-resolution-site-records");
    let included = rows
        .iter()
        .find(|row| row["path_site_source_text"] == "Included")
        .unwrap();
    assert_eq!(included["path_resolution_state_kind"], "resolved");
    assert!(
        !read_published_table_fixture(&database, "module-scope-binding-table")
            .iter()
            .any(|row| row["binding_resolved_name_text"] == "Excluded")
    );
}

#[test]
fn test_builtin_missing_source_reasons() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(
        root.join("src/lib.rs"),
        "include!(concat!(env!(\"OUT_DIR\"),\"/generated.rs\"));",
    )
    .unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows = read_published_table_fixture(&database, "macro-expanded-item-table");
    assert!(
        rows.iter()
            .any(|row| row["macro_unresolved_reason_kind"] == "build_script"
                && row["macro_resolution_state_kind"] == "unresolved")
    );
}

#[test]
fn test_builtin_include_callsite_base() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(root.join("src/lib.rs"), "mod defs; pub mod child;").unwrap();
    fs::write(
        root.join("src/defs.rs"),
        "#[macro_export] macro_rules! emit {()=>{include!(\"chosen.rs\");};}",
    )
    .unwrap();
    fs::write(
        root.join("src/child.rs"),
        "crate::emit!(); pub fn take(_:Chosen){}",
    )
    .unwrap();
    fs::write(root.join("src/chosen.rs"), "pub struct Chosen;").unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let paths = read_published_table_fixture(&database, "path-resolution-site-records");
    let chosen = paths
        .iter()
        .find(|row| row["path_site_source_text"] == "Chosen")
        .unwrap();
    assert_eq!(chosen["path_resolution_state_kind"], "resolved");
    assert!(
        chosen["path_target_symbol_rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["target_source_file_path"]
                .as_str()
                .is_some_and(|path| path.ends_with("/src/chosen.rs")))
    );
    assert!(
        read_published_table_fixture(&database, "macro-expanded-item-table")
            .iter()
            .any(|row| row["macro_invocation_provenance_chain"]
                .as_array()
                .is_some_and(|chain| chain.len() == 2))
    );
}

#[test]
fn test_builtin_nested_macro_shadowing() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(root.join("src/lib.rs"),r#"macro_rules! concat {($($tt:tt)*)=>{"other.rs"}} include!(concat!("right.rs")); pub fn take(_:Other){}"#).unwrap();
    fs::write(root.join("src/right.rs"), "pub struct Right;").unwrap();
    fs::write(root.join("src/other.rs"), "pub struct Other;").unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let macros = read_published_table_fixture(&database, "macro-expanded-item-table");
    assert!(macros.iter().any(|row| {
        row["macro_unresolved_reason_kind"] == "external_source_missing"
            && row["macro_unresolved_detail_text"]
                .as_str()
                .is_some_and(|detail| detail.contains("nested macro expansion unavailable"))
    }));
    assert!(
        !read_published_table_fixture(&database, "module-scope-binding-table")
            .iter()
            .any(|row| matches!(
                row["binding_resolved_name_text"].as_str(),
                Some("Right" | "Other")
            ))
    );
}

#[test]
fn test_lang_items_in_sysroot() {
    let retained = std::env::var_os("PARSELTONGUE_NATIVE_FIXTURE_DIRECTORY");
    let temporary = retained
        .as_ref()
        .map_or_else(tempfile::tempdir, tempfile::tempdir_in)
        .unwrap();
    let directory = temporary.path().to_path_buf();
    let _guard = if retained.is_some() {
        let _ = temporary.keep();
        None
    } else {
        Some(temporary)
    };
    eprintln!("native_fixture_directory={}", directory.display());
    let root = directory.join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(root.join("src/lib.rs"),"pub fn build(_:Option<Vec<u8>>,_:core::ops::Range<u8>)->std::string::String{String::new()}").unwrap();
    let database = directory.join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let paths = read_published_table_fixture(&database, "path-resolution-site-records");
    for name in [
        "Option<Vec<u8>>",
        "Vec<u8>",
        "core::ops::Range<u8>",
        "std::string::String",
    ] {
        let row = paths.iter().find(|row| {
            row["path_site_source_text"] == name
                && row["site_compilation_scope_record"]["unit_cargo_target_name"] == "fixture"
        });
        assert!(
            row.is_some_and(|row| row["path_resolution_state_kind"] == "resolved"),
            "{name}: {row:?}"
        );
    }
    let langs = read_published_table_fixture(&database, "lang-item-definition-table");
    for name in [
        "sized",
        "copy",
        "owned_box",
        "panic_in_cleanup",
        "coerce_pointee_validated",
    ] {
        assert!(
            langs.iter().any(|row| row["lang_item_source_name"] == name),
            "{name}"
        );
    }
    assert!(
        langs
            .iter()
            .all(|row| row["lang_definition_provenance_chain"].is_array())
    );
}

#[test]
fn test_path_attributes_follow_source() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(
        root.join("src/lib.rs"),
        "mod parent; pub fn take(_:parent::Selected){}",
    )
    .unwrap();
    fs::write(
        root.join("src/parent.rs"),
        "#[path=\"selected.rs\"] mod selected; pub use selected::Selected;",
    )
    .unwrap();
    fs::write(root.join("src/selected.rs"), "pub struct Selected;").unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let paths = read_published_table_fixture(&database, "path-resolution-site-records");
    assert!(
        paths
            .iter()
            .any(|row| row["path_site_source_text"] == "parent::Selected"
                && row["path_resolution_state_kind"] == "resolved")
    );
}

#[test]
fn test_mapping_stage_selection_contract() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    let database = directory.path().join("store.db");
    let run = |stage: &str| {
        Command::new(env!("CARGO_BIN_EXE_parseltongue"))
            .args([
                "map-workspace-full-run",
                "--workspace-root-path-value",
                root.to_str().unwrap(),
                "--workspace-store-file-path",
                database.to_str().unwrap(),
                "--mapping-final-stage-kind",
                stage,
            ])
            .output()
            .unwrap()
    };
    let output = run("syntax-spans");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let banner: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        banner["stage_implementation_status_values"]["syntax-spans"],
        "complete"
    );
    assert_eq!(
        banner["stage_implementation_status_values"]["name-resolution"],
        "not yet implemented"
    );
    assert!(read_published_table_fixture(&database, "module-scope-binding-table").is_empty());
    let output = run("");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown mapping final stage"));
}

/// REQ-RUST-024.11
#[test]
fn test_syntax_publication_status_complete() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    let database = directory.path().join("store.db");
    let output = run_workspace_stage_fixture(&root, &database, "syntax-spans");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let banner: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        banner["stage_implementation_status_values"]["syntax-spans"],
        "complete"
    );
    assert_eq!(
        banner["stage_publication_status_values"],
        serde_json::json!({
            "crate-graph": "complete",
            "source-ingest": "complete",
            "syntax-spans": "complete",
            "name-resolution": "pending"
        })
    );
}

/// REQ-RUST-024.11
#[test]
fn test_stale_publication_marker_pending() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    let database = directory.path().join("store.db");
    let output = run_workspace_stage_fixture(&root, &database, "syntax-spans");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let published_scope = store.read_workspace_publication_scope().unwrap().unwrap();
    let stale_scope = StageRunScopeValue {
        run_generation_digest_value: format!(
            "stale-{}",
            published_scope.run_generation_digest_value
        ),
        stage_attempt_sequence_value: "stale-attempt".into(),
    };
    write_stale_completion_marker(&store, &stale_scope, "name-resolution", &[]);
    assert!(
        store
            .read_stage_table_rows(&stale_scope, "stage-run-generation-records")
            .unwrap()
            .iter()
            .any(|row| row.stage_row_record_key == "__marker__/name-resolution/__complete__")
    );
    assert_eq!(
        store.read_workspace_publication_scope().unwrap(),
        Some(published_scope)
    );
    drop(store);

    let banner = read_workspace_banner_fixture(&database);
    assert_eq!(
        banner["stage_publication_status_values"]["name-resolution"],
        "pending"
    );
}

/// REQ-RUST-024.11
#[test]
fn test_publication_status_without_generation() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("store.db");
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    store.initialize_workspace_stage_schema().unwrap();
    drop(store);

    let banner = read_workspace_banner_fixture(&database);
    assert_eq!(
        banner["stage_publication_status_values"],
        serde_json::json!({
            "crate-graph": "pending",
            "source-ingest": "pending",
            "syntax-spans": "pending",
            "name-resolution": "pending"
        })
    );
}

#[test]
fn test_type_macros_remain_deferred() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(
        root.join("src/lib.rs"),
        "macro_rules! ty {()=>{u8};} pub fn take(_:ty!()){}",
    )
    .unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(read_published_table_fixture(&database, "macro-expanded-item-table").is_empty());
}

#[test]
/// REQ-RUST-006.2
fn test_macro_rules_items_match() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    let source = r#"
macro_rules! emit_item { () => { pub struct MadeItem; }; }
emit_item!();
macro_rules! emit_body { () => { target_call() }; }
pub fn run() { emit_body!(); }
"#;
    fs::write(root.join("src/lib.rs"), source).unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_macro_sysroot_disabled(&database);

    let rows = read_published_table_fixture(&database, "macro-expanded-item-table");
    let mut actual = rows
        .iter()
        .map(|row| {
            json!({
                "fragment":row["macro_expansion_fragment_kind"],
                "source":row["macro_expanded_source_text"],
                "invocation":row["macro_invocation_identity_record"],
                "owner":row["macro_enclosing_body_owner_occurrence_key"],
                "chain":row["macro_invocation_provenance_chain"],
                "origins":row["macro_token_origin_rows"],
            })
        })
        .collect::<Vec<_>>();
    actual.sort_by_key(Value::to_string);

    let item_invocation = expected_macro_invocation_identity(&database, source, "emit_item!();");
    let body_invocation = expected_macro_invocation_identity(&database, source, "emit_body!()");
    let item_chain = expected_macro_provenance_chain(&item_invocation);
    let body_chain = expected_macro_provenance_chain(&body_invocation);
    let body_owner = expected_syntax_item_owner(
        &database,
        &root,
        source,
        "pub fn run() { emit_body!(); }",
        "FN",
        json!([]),
    );
    let source_path = root.join("src/lib.rs").canonicalize().unwrap();
    let item_start = source.find("pub struct MadeItem;").unwrap();
    let item_origins = vec![
        expected_definition_token_origin(&source_path, 0, 3, item_start, item_start + 3),
        expected_definition_token_origin(&source_path, 4, 10, item_start + 4, item_start + 10),
        expected_definition_token_origin(&source_path, 11, 19, item_start + 11, item_start + 19),
        expected_definition_token_origin(&source_path, 20, 21, item_start + 19, item_start + 20),
    ];
    let body_start = source.find("target_call()").unwrap();
    let body_origins = vec![
        expected_definition_token_origin(&source_path, 0, 11, body_start, body_start + 11),
        expected_definition_token_origin(&source_path, 12, 13, body_start + 11, body_start + 12),
        expected_definition_token_origin(&source_path, 13, 14, body_start + 12, body_start + 13),
    ];
    let mut expected = vec![
        json!({
            "fragment":"items",
            "source":"pub struct MadeItem ; ",
            "invocation":item_invocation,
            "owner":null,
            "chain":item_chain,
            "origins":item_origins,
        }),
        json!({
            "fragment":"statements",
            "source":"target_call () ",
            "invocation":body_invocation,
            "owner":body_owner,
            "chain":body_chain,
            "origins":body_origins,
        }),
    ];
    expected.sort_by_key(Value::to_string);
    assert_eq!(actual, expected);
    let (_, target_unit) = fixture_compilation_unit_identity(&database);
    let generated_items = read_published_table_fixture(&database, "syntax-namespace-input-items")
        .into_iter()
        .filter(|row| {
            row["binding_resolved_name_text"] == "MadeItem"
                && row["binding_lexical_scope_key"]["scope_compilation_unit_key"] == target_unit
        })
        .collect::<Vec<_>>();
    let mut namespaces = generated_items
        .iter()
        .map(|row| row["binding_namespace_kind_value"].as_str().unwrap())
        .collect::<Vec<_>>();
    namespaces.sort_unstable();
    assert_eq!(namespaces, ["type", "value"]);
    let type_binding = generated_items
        .iter()
        .find(|row| row["binding_namespace_kind_value"] == "type")
        .unwrap();
    let value_binding = generated_items
        .iter()
        .find(|row| row["binding_namespace_kind_value"] == "value")
        .unwrap();
    assert_eq!(type_binding["binding_resolved_name_text"], "MadeItem");
    assert_eq!(value_binding["binding_resolved_name_text"], "MadeItem");
    assert_eq!(
        type_binding["binding_lexical_scope_key"],
        value_binding["binding_lexical_scope_key"]
    );
    assert_eq!(
        type_binding["binding_target_identity_key"],
        value_binding["binding_target_identity_key"]
    );
    assert_eq!(
        type_binding["binding_target_identity_key"]["target_semantic_occurrence_key"]["span_expansion_provenance_chain"],
        item_chain
    );
    assert_eq!(
        type_binding["binding_target_metadata_value"]["target_symbol_kind_value"],
        "Struct"
    );
    assert_eq!(
        type_binding["binding_target_metadata_value"]["target_source_range_start"],
        0
    );
    assert_eq!(
        type_binding["binding_target_metadata_value"]["target_source_range_end"],
        21
    );
    assert_eq!(
        type_binding["binding_target_child_scope"]["scope_compilation_unit_key"],
        target_unit
    );
    let child_path = type_binding["binding_target_child_scope"]["scope_semantic_path_parts"]
        .as_array()
        .unwrap();
    assert_eq!(child_path.len(), 1);
    assert!(child_path[0].as_str().unwrap().starts_with("@STRUCT:"));
    assert!(value_binding["binding_target_child_scope"].is_null());
}

#[test]
/// REQ-RUST-006.2
fn test_body_macro_sites_match() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    let source = r#"
macro_rules! statement_leaf { () => { let statement_value = statement_target(); }; }
macro_rules! statement_child { () => { statement_leaf!(); }; }
macro_rules! expression_child { () => { target_call().finish(argument) }; }
macro_rules! body_root { () => { statement_child!(); let value = expression_child!(); }; }
pub fn run() { body_root!(); }
"#;
    fs::write(root.join("src/lib.rs"), source).unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_macro_sysroot_disabled(&database);

    let package_root = root.canonicalize().unwrap();
    let source_path = package_root.join("src/lib.rs");
    let body_owner = expected_syntax_item_owner(
        &database,
        &package_root,
        source,
        "pub fn run() { body_root!(); }",
        "FN",
        json!([]),
    );
    let body_root_invocation =
        expected_macro_invocation_identity(&database, source, "body_root!()");
    let body_root_chain = expected_macro_provenance_chain(&body_root_invocation);
    let body_root_virtual_path = expected_macro_virtual_path(&package_root, &body_root_invocation);
    let body_root_virtual_source_path = body_root_virtual_path
        .strip_prefix(&package_root)
        .unwrap()
        .to_string_lossy()
        .into_owned();

    let body_root_source = "statement_child !() ; let value = expression_child !() ; ";
    let statement_child_source = "statement_leaf !() ; ";
    let statement_leaf_source = "let statement_value = statement_target () ; ";
    let expression_child_source = "target_call () .finish (argument ) ";
    let statement_child_invocation = expected_nested_macro_identity(
        &database,
        &body_root_virtual_source_path,
        body_root_source,
        "statement_child !()",
        &body_root_chain,
    );
    let expression_child_invocation = expected_nested_macro_identity(
        &database,
        &body_root_virtual_source_path,
        body_root_source,
        "expression_child !()",
        &body_root_chain,
    );
    let statement_child_chain =
        extend_macro_provenance_chain(&body_root_chain, &statement_child_invocation);
    let statement_child_virtual_path =
        expected_macro_virtual_path(&package_root, &statement_child_invocation);
    let statement_child_virtual_source_path = statement_child_virtual_path
        .strip_prefix(&package_root)
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let statement_leaf_invocation = expected_nested_macro_identity(
        &database,
        &statement_child_virtual_source_path,
        statement_child_source,
        "statement_leaf !()",
        &statement_child_chain,
    );
    let statement_leaf_chain =
        extend_macro_provenance_chain(&statement_child_chain, &statement_leaf_invocation);
    let statement_leaf_virtual_path =
        expected_macro_virtual_path(&package_root, &statement_leaf_invocation);
    let expression_child_chain =
        extend_macro_provenance_chain(&body_root_chain, &expression_child_invocation);
    let expression_child_virtual_path =
        expected_macro_virtual_path(&package_root, &expression_child_invocation);

    let mut actual_expansions =
        read_published_table_fixture(&database, "macro-expanded-item-table")
            .iter()
            .map(|row| {
                json!({
                    "fragment":row["macro_expansion_fragment_kind"],
                    "path":row["macro_virtual_source_path"],
                    "source":row["macro_expanded_source_text"],
                    "invocation":row["macro_invocation_identity_record"],
                    "owner":row["macro_enclosing_body_owner_occurrence_key"],
                    "chain":row["macro_invocation_provenance_chain"],
                    "origins":row["macro_token_origin_rows"],
                    "arm":row["macro_matched_arm_index"],
                    "resolution":row["macro_resolution_state_kind"],
                })
            })
            .collect::<Vec<_>>();
    actual_expansions.sort_by_key(Value::to_string);
    let mut expected_expansions = vec![
        json!({
            "fragment":"statements",
            "path":body_root_virtual_path.to_string_lossy(),
            "source":body_root_source,
            "invocation":&body_root_invocation,
            "owner":body_owner,
            "chain":&body_root_chain,
            "origins":expected_macro_token_origins(
                source,
                "statement_child!(); let value = expression_child!();",
                body_root_source,
                &source_path,
            ),
            "arm":0,
            "resolution":"expanded",
        }),
        json!({
            "fragment":"statements",
            "path":statement_child_virtual_path.to_string_lossy(),
            "source":statement_child_source,
            "invocation":&statement_child_invocation,
            "owner":body_owner,
            "chain":&statement_child_chain,
            "origins":expected_macro_token_origins(
                source,
                "statement_leaf!();",
                statement_child_source,
                &source_path,
            ),
            "arm":0,
            "resolution":"expanded",
        }),
        json!({
            "fragment":"statements",
            "path":statement_leaf_virtual_path.to_string_lossy(),
            "source":statement_leaf_source,
            "invocation":&statement_leaf_invocation,
            "owner":body_owner,
            "chain":&statement_leaf_chain,
            "origins":expected_macro_token_origins(
                source,
                "let statement_value = statement_target();",
                statement_leaf_source,
                &source_path,
            ),
            "arm":0,
            "resolution":"expanded",
        }),
        json!({
            "fragment":"expression",
            "path":expression_child_virtual_path.to_string_lossy(),
            "source":expression_child_source,
            "invocation":&expression_child_invocation,
            "owner":body_owner,
            "chain":&expression_child_chain,
            "origins":expected_macro_token_origins(
                source,
                "target_call().finish(argument)",
                expression_child_source,
                &source_path,
            ),
            "arm":0,
            "resolution":"expanded",
        }),
    ];
    expected_expansions.sort_by_key(Value::to_string);
    assert_eq!(
        actual_expansions, expected_expansions,
        "macro expansion known-answer rows"
    );

    let mut actual_sites = read_published_table_fixture(&database, "body-site-reference-table")
        .iter()
        .filter(|row| row["site_originating_macro_invocation_identity"].is_object())
        .map(|row| {
            json!({
                "kind":row["site_syntax_kind_value"],
                "range":[row["span_byte_start_offset"],row["span_byte_end_offset"]],
                "text":row["site_syntax_source_text"],
                "source_path":row["file_source_absolute_path"],
                "owner":row["site_owner_occurrence_key"],
                "invocation":row["site_originating_macro_invocation_identity"],
                "chain":row["span_expansion_provenance_chain"],
                "resolution":row["site_resolution_status_kind"],
                "unresolved":row["site_unresolved_reason_kind"],
            })
        })
        .collect::<Vec<_>>();
    actual_sites.sort_by_key(Value::to_string);
    let mut expected_sites = vec![
        expected_macro_body_site(
            &body_root_virtual_path,
            body_root_source,
            "statement_child !()",
            "MACRO_CALL",
            &body_owner,
            &body_root_invocation,
            &body_root_chain,
        ),
        expected_macro_body_site(
            &body_root_virtual_path,
            body_root_source,
            "expression_child !()",
            "MACRO_CALL",
            &body_owner,
            &body_root_invocation,
            &body_root_chain,
        ),
        expected_macro_body_site(
            &statement_child_virtual_path,
            statement_child_source,
            "statement_leaf !()",
            "MACRO_CALL",
            &body_owner,
            &statement_child_invocation,
            &statement_child_chain,
        ),
        expected_macro_body_site(
            &statement_leaf_virtual_path,
            statement_leaf_source,
            "statement_target ()",
            "CALL_EXPR",
            &body_owner,
            &statement_leaf_invocation,
            &statement_leaf_chain,
        ),
        expected_macro_body_site(
            &statement_leaf_virtual_path,
            statement_leaf_source,
            "statement_target",
            "PATH_EXPR",
            &body_owner,
            &statement_leaf_invocation,
            &statement_leaf_chain,
        ),
        expected_macro_body_site(
            &expression_child_virtual_path,
            expression_child_source,
            "target_call () .finish (argument ) ",
            "METHOD_CALL_EXPR",
            &body_owner,
            &expression_child_invocation,
            &expression_child_chain,
        ),
        expected_macro_body_site(
            &expression_child_virtual_path,
            expression_child_source,
            "target_call ()",
            "CALL_EXPR",
            &body_owner,
            &expression_child_invocation,
            &expression_child_chain,
        ),
        expected_macro_body_site(
            &expression_child_virtual_path,
            expression_child_source,
            "target_call",
            "PATH_EXPR",
            &body_owner,
            &expression_child_invocation,
            &expression_child_chain,
        ),
        expected_macro_body_site(
            &expression_child_virtual_path,
            expression_child_source,
            "argument",
            "PATH_EXPR",
            &body_owner,
            &expression_child_invocation,
            &expression_child_chain,
        ),
    ];
    expected_sites.sort_by_key(Value::to_string);
    assert_eq!(
        actual_sites, expected_sites,
        "expanded body-site known-answer rows"
    );

    let first_expansions = read_sorted_published_records(&database, "macro-expanded-item-table");
    let first_body_sites = read_sorted_published_records(&database, "body-site-reference-table");
    let rerun = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        rerun.status.success(),
        "{}",
        String::from_utf8_lossy(&rerun.stderr)
    );
    assert_macro_sysroot_disabled(&database);
    assert_eq!(
        read_sorted_published_records(&database, "macro-expanded-item-table"),
        first_expansions
    );
    assert_eq!(
        read_sorted_published_records(&database, "body-site-reference-table"),
        first_body_sites
    );
}

fn build_body_macro_fixture(
    source: &str,
) -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(root.join("src/lib.rs"), source).unwrap();
    let database = directory.path().join("store.db");
    (directory, root, database)
}

fn assert_macro_sysroot_disabled(database: &Path) {
    let units = read_published_table_fixture(database, "crate-dependency-graph-table");
    assert!(!units.is_empty());
    assert!(
        units
            .iter()
            .all(|unit| unit["unit_sysroot_context_flag"] != true)
    );
}

fn read_sorted_published_rows(database: &Path, table: &str) -> Vec<Value> {
    let mut rows = read_published_table_fixture(database, table);
    rows.sort_by_key(Value::to_string);
    rows
}

#[test]
/// REQ-RUST-006.2
fn test_body_fragment_context_routing() {
    let source = r#"
macro_rules! statement { () => { let local = target_call(); }; }
macro_rules! initializer { () => { target_call() }; }
macro_rules! tail { () => { target_call() }; }
pub fn run() { statement!(); let _value = initializer!(); tail!() }
"#;
    let (_directory, root, database) = build_body_macro_fixture(source);
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_macro_sysroot_disabled(&database);
    let mut actual = read_published_table_fixture(&database, "macro-expanded-item-table")
        .iter()
        .map(|row| {
            json!({
                "fragment":row["macro_expansion_fragment_kind"],
                "invocation":row["macro_invocation_identity_record"],
            })
        })
        .collect::<Vec<_>>();
    actual.sort_by_key(Value::to_string);
    let mut expected = vec![
        json!({
            "fragment":"statements",
            "invocation":expected_macro_invocation_identity(&database, source, "statement!()"),
        }),
        json!({
            "fragment":"expression",
            "invocation":expected_macro_invocation_identity(&database, source, "initializer!()"),
        }),
        json!({
            "fragment":"expression",
            "invocation":expected_macro_invocation_identity(&database, source, "tail!()"),
        }),
    ];
    expected.sort_by_key(Value::to_string);
    assert_eq!(actual, expected);
}

#[test]
/// REQ-RUST-006.2
fn test_generated_function_owner_identity() {
    let source = r#"
macro_rules! inner { () => { target_call() }; }
macro_rules! generate { () => { fn generated() { inner!(); } }; }
generate!();
"#;
    let (_directory, root, database) = build_body_macro_fixture(source);
    let package_root = root.canonicalize().unwrap();
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_macro_sysroot_disabled(&database);
    let expansions = read_published_table_fixture(&database, "macro-expanded-item-table");
    assert_eq!(expansions.len(), 2);
    let outer_invocation = expected_macro_invocation_identity(&database, source, "generate!();");
    let outer_chain = expected_macro_provenance_chain(&outer_invocation);
    let outer_virtual_path = expected_macro_virtual_path(&package_root, &outer_invocation);
    let outer_source = "fn generated () {inner !() ; } ";
    let function_source = "fn generated () {inner !() ; }";
    assert_eq!(function_source.len(), 30);
    let generated_owner = expected_virtual_occurrence_owner(
        &database,
        &outer_virtual_path,
        outer_source,
        function_source,
        "FN",
        outer_chain.clone(),
    );
    let outer = expansions
        .iter()
        .find(|row| row["macro_invocation_identity_record"] == outer_invocation)
        .unwrap();
    assert_eq!(outer["macro_expansion_fragment_kind"], "items");
    assert_eq!(
        outer["macro_virtual_source_path"],
        outer_virtual_path.to_string_lossy().as_ref()
    );
    assert_eq!(outer["macro_expanded_source_text"], outer_source);
    assert_eq!(
        outer["macro_enclosing_body_owner_occurrence_key"],
        Value::Null
    );
    assert_eq!(outer["macro_invocation_provenance_chain"], outer_chain);
    let outer_virtual_source_path = outer_virtual_path
        .strip_prefix(&package_root)
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let nested_invocation = expected_nested_macro_identity(
        &database,
        &outer_virtual_source_path,
        outer_source,
        "inner !()",
        &expected_macro_provenance_chain(&outer_invocation),
    );
    let nested_chain = extend_macro_provenance_chain(&outer_chain, &nested_invocation);
    let nested = expansions
        .iter()
        .find(|row| row["macro_invocation_identity_record"] == nested_invocation)
        .unwrap();
    assert_eq!(nested["macro_expansion_fragment_kind"], "statements");
    assert_eq!(nested["macro_expanded_source_text"], "target_call () ");
    assert_eq!(nested["macro_invocation_provenance_chain"], nested_chain);
    assert_eq!(
        nested["macro_enclosing_body_owner_occurrence_key"],
        generated_owner
    );
    let sites = read_published_table_fixture(&database, "body-site-reference-table");
    let generated_sites = sites
        .iter()
        .filter(|row| row["site_originating_macro_invocation_identity"] == nested_invocation)
        .collect::<Vec<_>>();
    assert!(!generated_sites.is_empty());
    for generated_site in generated_sites {
        assert_eq!(generated_site["site_owner_occurrence_key"], generated_owner);
        assert_eq!(
            generated_site["span_expansion_provenance_chain"],
            nested_chain
        );
    }
    assert!(sites.iter().any(|row| {
        row["site_originating_macro_invocation_identity"] == nested_invocation
            && row["site_syntax_kind_value"] == "PATH_EXPR"
            && row["site_syntax_source_text"] == "target_call"
    }));
}

#[test]
/// REQ-RUST-006.2
fn test_local_scope_binding_isolation() {
    let source = r#"
pub struct Marker;
macro_rules! local { () => { use crate::Marker as LocalMarker; fn local_function() {} }; }
pub fn inside() { local!(); }
"#;
    let (_directory, root, database) = build_body_macro_fixture(source);
    let source_path = root.join("src/lib.rs").canonicalize().unwrap();
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_macro_sysroot_disabled(&database);
    let expected_block_scope = expected_local_block_scope(
        &database,
        &source_path,
        source,
        "pub fn inside() { local!(); }",
        "{ local!(); }",
    );
    let (_, unit_identity) = fixture_compilation_unit_identity(&database);
    let module_scope = json!({
        "scope_compilation_unit_key":unit_identity,
        "scope_semantic_path_parts":[],
    });
    let items = read_published_table_fixture(&database, "syntax-namespace-input-items");
    let local_function_items = items
        .iter()
        .filter(|row| row["binding_resolved_name_text"] == "local_function")
        .collect::<Vec<_>>();
    assert!(!local_function_items.is_empty());
    let local_marker_items = items
        .iter()
        .filter(|row| row["binding_resolved_name_text"] == "LocalMarker")
        .collect::<Vec<_>>();
    for item in local_function_items.iter().chain(local_marker_items.iter()) {
        assert_eq!(item["binding_lexical_scope_key"], expected_block_scope);
        assert_ne!(item["binding_lexical_scope_key"], module_scope);
    }
    let imports = read_published_table_fixture(&database, "syntax-namespace-input-imports");
    let local_imports = imports
        .iter()
        .filter(|row| {
            row["import_binding_alias_text"] == "LocalMarker"
                || row["import_binding_alias_text"] == "local_function"
        })
        .collect::<Vec<_>>();
    let local_marker_imports = local_imports
        .iter()
        .filter(|row| row["import_binding_alias_text"] == "LocalMarker")
        .collect::<Vec<_>>();
    assert!(!local_marker_imports.is_empty());
    for import in local_imports {
        assert_eq!(import["import_lexical_scope_key"], expected_block_scope);
        assert_ne!(import["import_lexical_scope_key"], module_scope);
    }
}

#[test]
/// REQ-RUST-006.2
fn test_nested_macro_token_origins() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::create_dir_all(root.join("defs/src")).unwrap();
    fs::write(
        root.join("defs/Cargo.toml"),
        "[package]\nname=\"defs\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
    )
    .unwrap();
    let macro_definition_body_source =
        "{ ($argument:ident) => { $crate::target_call().finish($argument) }; }";
    let definition_source = [
        "pub struct Target; impl Target { pub fn finish(self, _: i32) {} } ",
        "pub fn target_call() -> Target { Target } ",
        "#[macro_export] macro_rules! expression_child ",
        macro_definition_body_source,
    ]
    .concat();
    fs::write(root.join("defs/src/lib.rs"), &definition_source).unwrap();
    let manifest = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        format!("{manifest}\n[dependencies]\ndefs={{path=\"defs\"}}\n"),
    )
    .unwrap();
    let source = r#"
macro_rules! outer { () => { let value = defs::expression_child!(argument); }; }
pub fn run() { let argument = 1; outer!(); }
"#;
    fs::write(root.join("src/lib.rs"), source).unwrap();
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
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_macro_sysroot_disabled(&database);

    let package_root = root.canonicalize().unwrap();
    let source_path = root.join("defs/src/lib.rs").canonicalize().unwrap();
    let expansions = read_published_table_fixture(&database, "macro-expanded-item-table");
    assert_eq!(expansions.len(), 2);
    let outer_invocation = expected_macro_invocation_identity(&database, source, "outer!()");
    let outer_chain = expected_macro_provenance_chain(&outer_invocation);
    let outer_virtual_path = expected_macro_virtual_path(&package_root, &outer_invocation);
    let outer_source = "let value = defs ::expression_child !(argument ) ; ";
    let outer = expansions
        .iter()
        .find(|row| row["macro_invocation_identity_record"] == outer_invocation)
        .unwrap();
    assert_eq!(outer["macro_expansion_fragment_kind"], "statements");
    assert_eq!(
        outer["macro_virtual_source_path"],
        outer_virtual_path.to_string_lossy().as_ref()
    );
    assert_eq!(outer["macro_expanded_source_text"], outer_source);
    assert_eq!(outer["macro_invocation_provenance_chain"], outer_chain);

    let outer_virtual_source_path = outer_virtual_path
        .strip_prefix(&package_root)
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let nested_invocation_source = "defs ::expression_child !(argument )";
    let nested_invocation = expected_nested_macro_identity(
        &database,
        &outer_virtual_source_path,
        outer_source,
        nested_invocation_source,
        &outer_chain,
    );
    let nested_start = outer_source.find(nested_invocation_source).unwrap();
    assert_eq!(
        nested_invocation["span_byte_range_value"],
        json!([nested_start, nested_start + nested_invocation_source.len()])
    );
    assert_eq!(
        &outer_source[nested_start..nested_start + nested_invocation_source.len()],
        nested_invocation_source
    );
    let nested_chain = extend_macro_provenance_chain(&outer_chain, &nested_invocation);
    let nested = expansions
        .iter()
        .find(|row| row["macro_invocation_identity_record"] == nested_invocation)
        .unwrap();
    let nested_source = "$crate ::target_call () .finish (argument ) ";
    let nested_virtual_path = expected_macro_virtual_path(&package_root, &nested_invocation);
    assert_eq!(nested["macro_expansion_fragment_kind"], "expression");
    assert_eq!(
        nested["macro_virtual_source_path"],
        nested_virtual_path.to_string_lossy().as_ref()
    );
    assert_eq!(nested["macro_expanded_source_text"], nested_source);
    assert_eq!(nested["macro_invocation_provenance_chain"], nested_chain);

    let nested_origins = nested["macro_token_origin_rows"].as_array().unwrap();
    let crate_output_start = nested_source.find("$crate").unwrap();
    let crate_source_start = definition_source.find("$crate").unwrap();
    let expected_crate_origin = expected_definition_token_origin(
        &source_path,
        crate_output_start,
        crate_output_start + "$crate".len(),
        crate_source_start + 1,
        crate_source_start + "$crate".len(),
    );
    assert!(
        nested_origins.contains(&expected_crate_origin),
        "expected definition origin {expected_crate_origin:#?}; actual origins {nested_origins:#?}"
    );
    let outer_argument_start = outer_source.find("argument").unwrap();
    let nested_argument_start = nested_source.find("argument").unwrap();
    let argument_length = "argument".len();
    assert_eq!(
        &outer_source[outer_argument_start..outer_argument_start + argument_length],
        "argument"
    );
    let expected_argument_origin = json!({
        "token_output_byte_range":[nested_argument_start,nested_argument_start+argument_length],
        "token_original_byte_range":[outer_argument_start,outer_argument_start+argument_length],
        "token_source_origin_kind":"invocation",
        "token_source_absolute_path":outer_virtual_path.to_string_lossy(),
    });
    assert!(nested_origins.contains(&expected_argument_origin));

    let namespace_items = read_published_table_fixture(&database, "syntax-namespace-input-items");
    let expression_child = namespace_items
        .iter()
        .find(|row| row["binding_resolved_name_text"] == "expression_child")
        .unwrap();
    let definition =
        &expression_child["binding_target_metadata_value"]["target_macro_definition_record"];
    assert_eq!(
        definition["macro_source_absolute_path"],
        source_path.to_string_lossy().as_ref()
    );
    assert_eq!(
        definition["macro_definition_source_text"],
        macro_definition_body_source
    );
    let definition_unit = definition["macro_definition_unit_key"].clone();
    let (_, invocation_unit) = fixture_compilation_unit_identity(&database);
    assert_ne!(definition_unit, invocation_unit);
    assert_eq!(
        definition["macro_definition_crate_rows"],
        json!([{
            "macro_source_byte_offset":macro_definition_body_source.find("$crate").unwrap(),
            "macro_definition_unit_key":definition_unit,
        }])
    );

    let definition_scope_unit = expression_child["binding_target_metadata_value"]
        ["target_compilation_scope_record"]["unit_semantic_identity_record"]
        .clone();
    assert_eq!(
        definition_scope_unit,
        definition["macro_definition_unit_key"]
    );
    assert_eq!(
        definition["macro_definition_crate_rows"][0]["macro_definition_unit_key"],
        definition_scope_unit
    );
}

#[test]
/// REQ-RUST-006.2
fn test_legacy_macro_shadow_restoration() {
    let source = r#"
scope_macro!();
macro_rules! scope_macro { () => { pub fn outer_generated() {} }; }
pub fn nested() { macro_rules! scope_macro { () => { fn shadow_generated() {} }; } scope_macro!(); }
scope_macro!();
"#;
    let (_directory, root, database) = build_body_macro_fixture(source);
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_macro_sysroot_disabled(&database);
    let expansions = read_published_table_fixture(&database, "macro-expanded-item-table");
    let expanded = expansions
        .iter()
        .filter(|row| row["macro_resolution_state_kind"] == "expanded")
        .collect::<Vec<_>>();
    assert_eq!(expanded.len(), 2);
    let invocation_ranges = source
        .match_indices("scope_macro!()")
        .map(|(start, invocation)| [start, start + invocation.len()])
        .collect::<Vec<_>>();
    assert_eq!(invocation_ranges.len(), 3);
    let mut expanded_by_range = expanded
        .iter()
        .map(|row| {
            let range = row["macro_invocation_identity_record"]["span_byte_range_value"]
                .as_array()
                .unwrap();
            (
                [
                    range[0].as_u64().unwrap() as usize,
                    range[1].as_u64().unwrap() as usize,
                ],
                *row,
            )
        })
        .collect::<Vec<_>>();
    expanded_by_range.sort_by_key(|(range, _)| range[0]);
    let actual_ranges = expanded_by_range
        .iter()
        .map(|(range, _)| *range)
        .collect::<Vec<_>>();
    let mut expected_ranges = invocation_ranges[1..].to_vec();
    expected_ranges[1][1] += 1;
    assert_eq!(actual_ranges, expected_ranges);
    assert!(
        expanded_by_range[0].1["macro_expanded_source_text"]
            .as_str()
            .unwrap()
            .contains("shadow_generated")
    );
    assert!(
        expanded_by_range[1].1["macro_expanded_source_text"]
            .as_str()
            .unwrap()
            .contains("outer_generated")
    );
}

#[test]
/// REQ-RUST-006.2
fn test_macro_output_rerun_determinism() {
    let source =
        "macro_rules! call { () => { target_call() }; } pub fn run() { let _value = call!(); }";
    let (_directory, root, database) = build_body_macro_fixture(source);
    let first = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_macro_sysroot_disabled(&database);
    let first_macros = read_sorted_published_rows(&database, "macro-expanded-item-table");
    let first_sites = read_sorted_published_rows(&database, "body-site-reference-table");
    let second = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_macro_sysroot_disabled(&database);
    assert_eq!(
        read_sorted_published_rows(&database, "macro-expanded-item-table"),
        first_macros
    );
    assert_eq!(
        read_sorted_published_rows(&database, "body-site-reference-table"),
        first_sites
    );
}

#[test]
/// REQ-RUST-006.2
fn test_malformed_fragment_publication_rollback() {
    let source =
        "macro_rules! body { () => { target_call() }; } pub fn run() { let _value = body!(); }";
    let (_directory, root, database) = build_body_macro_fixture(source);
    let first = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_macro_sysroot_disabled(&database);
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let previous_scope = store.read_workspace_publication_scope().unwrap().unwrap();
    let previous_completion_keys = store
        .read_stage_table_rows(&previous_scope, "stage-run-generation-records")
        .unwrap()
        .into_iter()
        .map(|row| row.stage_row_record_key)
        .collect::<BTreeSet<_>>();
    assert!(previous_completion_keys.contains("__marker__/name-resolution/__complete__"));
    drop(store);
    let previous_macros = read_sorted_published_rows(&database, "macro-expanded-item-table");
    let previous_sites = read_sorted_published_rows(&database, "body-site-reference-table");
    fs::write(
        root.join("src/lib.rs"),
        "macro_rules! body { () => { let value = ; }; } pub fn run() { body!(); }",
    )
    .unwrap();
    let failed = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        !failed.status.success(),
        "malformed fragment unexpectedly published: {}",
        String::from_utf8_lossy(&failed.stderr)
    );
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    assert_eq!(
        store.read_workspace_publication_scope().unwrap(),
        Some(previous_scope.clone())
    );
    let completion_keys = store
        .read_stage_table_rows(&previous_scope, "stage-run-generation-records")
        .unwrap()
        .into_iter()
        .map(|row| row.stage_row_record_key)
        .collect::<BTreeSet<_>>();
    assert_eq!(completion_keys, previous_completion_keys);
    drop(store);
    assert_eq!(
        read_sorted_published_rows(&database, "macro-expanded-item-table"),
        previous_macros
    );
    assert_eq!(
        read_sorted_published_rows(&database, "body-site-reference-table"),
        previous_sites
    );
}

#[test]
/// REQ-RUST-006.2
fn test_deferred_macro_position_retention() {
    let source = r#"
macro_rules! type_macro { () => { u8 }; }
macro_rules! pattern_macro { () => { _ }; }
pub fn deferred(value: type_macro!()) { let pattern_macro!() = value; }
"#;
    let (_directory, root, database) = build_body_macro_fixture(source);
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_macro_sysroot_disabled(&database);
    assert!(read_published_table_fixture(&database, "macro-expanded-item-table").is_empty());
    let sites = read_published_table_fixture(&database, "body-site-reference-table");
    let mut deferred = sites
        .iter()
        .filter(|row| row["site_syntax_kind_value"] == "MACRO_CALL")
        .map(|row| {
            (
                row["site_syntax_source_text"].as_str().unwrap().to_owned(),
                row["site_resolution_status_kind"].clone(),
            )
        })
        .collect::<Vec<_>>();
    deferred.sort_by_key(|(source_text, _)| source_text.clone());
    assert_eq!(
        deferred,
        vec![
            ("pattern_macro!()".to_owned(), json!("pending")),
            ("type_macro!()".to_owned(), json!("pending")),
        ]
    );
}

/// REQ-RUST-024.10
#[test]
fn test_empty_resolution_stage_rows() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(root.join("src/lib.rs"), "").unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let scope = store.read_workspace_publication_scope().unwrap().unwrap();
    assert_eq!(
        store
            .count_stage_table_rows(&scope, "path-resolution-site-records")
            .unwrap(),
        0
    );
    assert_eq!(
        store
            .count_stage_table_rows(&scope, "module-scope-binding-table")
            .unwrap(),
        19
    );
    let receipt_rows = store
        .read_stage_table_rows(&scope, "analysis-resolution-receipt-records")
        .unwrap();
    assert_eq!(receipt_rows.len(), 1);
    let receipt: Value = serde_json::from_str(&receipt_rows[0].stage_row_payload_text).unwrap();
    assert_eq!(
        receipt["analysis_product_digest_kind"],
        "blake3-ordered-row-leaves"
    );
    let proofs = receipt["analysis_product_content_proofs"]
        .as_object()
        .unwrap();
    let proved_count = proofs
        .values()
        .map(|proof| proof["row_count"].as_u64().unwrap())
        .sum::<u64>();
    assert_eq!(
        receipt["analysis_product_record_count"].as_u64(),
        Some(proved_count)
    );
    assert!(proofs.values().all(|proof| {
        proof["content_digest"]
            .as_str()
            .is_some_and(|digest| digest.len() == 64)
    }));
    assert_eq!(
        assert_receipt_product_rows(&store, &scope, &receipt),
        proved_count
    );
    assert_eq!(
        proofs["path-resolution-site-records"]["row_count"].as_u64(),
        Some(0)
    );
    assert!(Path::new(receipt["analysis_database_absolute_path"].as_str().unwrap()).is_file());
    assert!(
        store
            .read_stage_table_rows(&scope, "stage-run-generation-records")
            .unwrap()
            .iter()
            .any(|row| row.stage_row_record_key == "__marker__/name-resolution/__complete__")
    );
}

#[test]
fn test_fixture_projects_remain_unmapped() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    let manifest = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    fs::write(root.join("Cargo.toml"), format!("{manifest}\n[[example]]\nname=\"unused_lib\"\npath=\"examples/unused_lib.rs\"\ncrate-type=[\"rlib\"]\n")).unwrap();
    let library = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        format!("{library}\nmod empty; #[cfg(any())] mod dormant;"),
    )
    .unwrap();
    fs::write(root.join("src/empty.rs"), []).unwrap();
    fs::write(root.join("src/dormant.rs"), [255, 0]).unwrap();
    for path in ["tests/fixtures/project/src", "examples", "benches"] {
        fs::create_dir_all(root.join(path)).unwrap();
    }
    fs::write(
        root.join("tests/fixtures/project/Cargo.toml"),
        "[package]\nname=\"nested-fixture\"\nversion=\"0.1.0\"\n[workspace]\n",
    )
    .unwrap();
    for path in [
        "tests/fixtures/project/src/lib.rs",
        "tests/integration.rs",
        "examples/unused_lib.rs",
        "benches/unused.rs",
    ] {
        fs::write(root.join(path), [255, 0]).unwrap();
    }
    let database = directory.path().join("store.db");
    let mapped = run_workspace_mapping_fixture(&root, &database, "4");
    assert!(
        mapped.status.success(),
        "{}",
        String::from_utf8_lossy(&mapped.stderr)
    );
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let scope = store.read_workspace_publication_scope().unwrap().unwrap();
    let expected: BTreeSet<_> = ["src/lib.rs", "src/child.rs", "src/empty.rs"]
        .into_iter()
        .map(|path| root.join(path).to_str().unwrap().to_owned())
        .collect();
    let requests = store
        .read_stage_table_rows(&scope, "syntax-file-work-requests")
        .unwrap();
    let actual: BTreeSet<_> = requests
        .iter()
        .map(|row| {
            let value: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
            value["file_source_absolute_path"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(requests.len(), 3);
    assert_eq!(
        store
            .count_stage_table_rows(&scope, "file-source-text-table")
            .unwrap(),
        10
    );
    store
        .visit_stage_table_rows(&scope, "span-reference-text-table", &mut |row| {
            let value: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
            assert!(expected.contains(value["file_source_absolute_path"].as_str().unwrap()));
            Ok(())
        })
        .unwrap();
    drop(store);
    assert_eq!(
        read_workspace_banner_fixture(&database)["source_tiling_identity_holds"],
        true
    );
}
/// REQ-RUST-024.10
#[test]
fn test_stage_rerun_is_identical() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "1");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let first = read_workspace_banner_fixture(&database);
    assert_eq!(first["source_file_record_count"], 4);
    assert!(first["syntax_span_record_count"].as_u64().unwrap() > 0);
    assert!(first["signature_field_record_count"].as_u64().unwrap() > 0);
    assert!(first["body_site_record_count"].as_u64().unwrap() > 0);
    assert_eq!(first["source_tiling_identity_holds"], true);
    assert_eq!(
        first["stage_implementation_status_values"]["name-resolution"],
        "complete"
    );
    let first_receipts =
        read_published_table_fixture(&database, "analysis-resolution-receipt-records");
    assert_eq!(first_receipts.len(), 1);
    let first_receipt = first_receipts[0].clone();
    let first_store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let first_scope = first_store
        .read_workspace_publication_scope()
        .unwrap()
        .unwrap();
    let first_product_count =
        assert_receipt_product_rows(&first_store, &first_scope, &first_receipt);
    assert!(first_product_count > 0);
    drop(first_store);
    let output = run_workspace_mapping_fixture(&root, &database, "4");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let second = read_workspace_banner_fixture(&database);
    assert_eq!(
        first["run_generation_digest_value"],
        second["run_generation_digest_value"]
    );
    assert_eq!(
        first["syntax_span_record_count"],
        second["syntax_span_record_count"]
    );
    let second_receipts =
        read_published_table_fixture(&database, "analysis-resolution-receipt-records");
    assert_eq!(second_receipts.len(), 1);
    let second_receipt = second_receipts[0].clone();
    let second_store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let second_scope = second_store
        .read_workspace_publication_scope()
        .unwrap()
        .unwrap();
    let second_product_count =
        assert_receipt_product_rows(&second_store, &second_scope, &second_receipt);
    assert_eq!(first_product_count, second_product_count);
    assert_eq!(
        first_receipt["analysis_product_content_proofs"],
        second_receipt["analysis_product_content_proofs"]
    );
    drop(second_store);
    fs::write(root.join("src/child.rs"), "fn broken(").unwrap();
    assert!(
        !run_workspace_mapping_fixture(&root, &database, "4")
            .status
            .success()
    );
    let retained = read_workspace_banner_fixture(&database);
    assert_eq!(
        second["run_generation_digest_value"],
        retained["run_generation_digest_value"]
    );
}

#[test]
fn test_reason_counts_use_metadata() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(
        root.join("src/child.rs"),
        "pub fn g(x:u8)->u8 { let build_script=x; build_script }",
    )
    .unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "4");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let banner = read_workspace_banner_fixture(&database);
    assert_eq!(banner["build_script_unresolved_count"], 0);
    fs::write(root.join("src/child.rs"),"pub fn g(x:u8)->u8 { let build_script=x; let _ = include!(concat!(env!(\"OUT_DIR\"),\"/generated.rs\")); build_script }").unwrap();
    let output = run_workspace_mapping_fixture(&root, &database, "4");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let banner = read_workspace_banner_fixture(&database);
    assert_eq!(banner["build_script_unresolved_count"], 1);
    assert_eq!(
        banner["crate_unresolved_reason_counts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["unit_unresolved_reason_counts"]["build_script"]
                .as_u64()
                .unwrap())
            .sum::<u64>(),
        1
    );
}

#[test]
fn test_include_closure_digest_complete() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(
        root.join("src/child.rs"),
        "pub fn g(x:u8)->u8 { x + include!(\"value.inc\") }",
    )
    .unwrap();
    fs::write(root.join("src/value.inc"), "7").unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "4");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let first = read_workspace_banner_fixture(&database);
    assert_eq!(first["source_file_record_count"], 5);
    assert_eq!(first["source_tiling_identity_holds"], true);
    fs::write(root.join("src/value.inc"), "9").unwrap();
    let output = run_workspace_mapping_fixture(&root, &database, "4");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let second = read_workspace_banner_fixture(&database);
    assert_ne!(
        first["run_generation_digest_value"],
        second["run_generation_digest_value"]
    );
}

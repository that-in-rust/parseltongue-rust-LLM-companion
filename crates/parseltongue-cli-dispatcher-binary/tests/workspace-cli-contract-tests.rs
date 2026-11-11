#![allow(clippy::unwrap_used)]

use parseltongue_table_store_traits::StageTableStorePort;
use parseltongue_turso_store_adapter::TursoWorkspaceStoreAdapter;
use serde_json::Value;
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
        "not yet implemented"
    );
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

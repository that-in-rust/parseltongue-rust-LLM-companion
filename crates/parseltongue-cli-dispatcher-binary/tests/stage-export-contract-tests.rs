#![allow(clippy::unwrap_used)]

use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StageTableRowRecord, StageUnitWriteRequest,
};
use parseltongue_table_store_traits::StageTableStorePort;
use parseltongue_turso_store_adapter::TursoWorkspaceStoreAdapter;
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn run_export_fixture_command(database: &Path, command: &str, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_parseltongue"))
        .args([
            command,
            "--workspace-store-file-path",
            database.to_str().unwrap(),
        ])
        .args(extra)
        .output()
        .unwrap()
}

fn create_export_workspace_fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    let dependency = directory.path().join("dependency");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(dependency.join("src")).unwrap();
    fs::write(root.join("Cargo.toml"), "[package]\nname=\"exports\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[dependencies]\ndefs={path=\"../dependency\"}\n").unwrap();
    fs::write(
        dependency.join("Cargo.toml"),
        "[package]\nname=\"defs\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
    )
    .unwrap();
    fs::write(dependency.join("src/lib.rs"), "pub struct Foreign;").unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub struct Wanted(pub defs::Foreign); #[cfg(test)] mod absent;",
    )
    .unwrap();
    fs::write(root.join("src/unused.rs"), "pub struct Forbidden;").unwrap();
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
    let database = directory.path().join("store.db");
    let mapped = Command::new(env!("CARGO_BIN_EXE_parseltongue"))
        .args([
            "map-workspace-full-run",
            "--workspace-root-path-value",
            root.to_str().unwrap(),
            "--workspace-store-file-path",
            database.to_str().unwrap(),
            "--worker-thread-count-value",
            "2",
        ])
        .output()
        .unwrap();
    assert!(
        mapped.status.success(),
        "{}",
        String::from_utf8_lossy(&mapped.stderr)
    );
    (directory, database)
}

fn decode_export_fixture_rows(output: &Output) -> Vec<Value> {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).unwrap())
        .collect()
}

#[test]
fn test_published_stage_export_rows() {
    let (_directory, database) = create_export_workspace_fixture();
    let output = run_export_fixture_command(
        &database,
        "export-published-stage-rows",
        &["--stage-owned-table-name", "crate-dependency-graph-table"],
    );
    let rows = decode_export_fixture_rows(&output);
    assert_eq!(rows[0]["export_record_kind_value"], "header");
    assert_eq!(
        rows[0]["export_owned_table_name"],
        "crate-dependency-graph-table"
    );
    assert_eq!(
        rows[0]["export_scope_identity_record"]["run_generation_digest_value"]
            .as_str()
            .unwrap()
            .len(),
        64
    );
    assert!(
        rows[0]["export_scope_identity_record"]["stage_attempt_sequence_value"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
    assert_eq!(rows.len(), 3);
    assert!(
        rows[1..]
            .iter()
            .all(|row| row["export_record_kind_value"] == "row"
                && row["export_row_identity_value"].is_string()
                && row["export_row_payload_value"]["unit_mapped_target_flag"] == true)
    );
}

#[test]
fn test_empty_exports_preserve_scope() {
    let (_directory, database) = create_export_workspace_fixture();
    let output = run_export_fixture_command(
        &database,
        "export-published-stage-rows",
        &[
            "--stage-owned-table-name",
            "crate-dependency-graph-table",
            "--stage-row-filter-value",
            "never-a-binding",
        ],
    );
    let rows = decode_export_fixture_rows(&output);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["export_record_kind_value"], "header");
    assert!(rows[0]["export_scope_identity_record"].is_object());
}

#[test]
fn test_exact_manifest_selected_closure() {
    let (directory, database) = create_export_workspace_fixture();
    let output = run_export_fixture_command(&database, "export-mapped-unit-manifest", &[]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let manifest: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(manifest["manifest_format_version_value"], 1);
    assert_eq!(manifest["manifest_target_context_kind"], "compiler_default");
    assert_eq!(
        manifest["manifest_mapped_target_policy"]["policy_test_targets_flag"],
        false
    );
    assert_eq!(
        manifest["manifest_mapped_target_policy"]["policy_target_kind_values"],
        json!(["lib", "bin"])
    );
    assert_eq!(
        manifest["manifest_mapped_target_policy"]["policy_dependency_kind_values"],
        json!(["normal"])
    );
    assert_eq!(
        manifest["manifest_mapped_target_policy"]["policy_support_mapping_flag"],
        false
    );
    assert!(
        manifest["manifest_compiler_version_text"]
            .as_str()
            .unwrap()
            .starts_with("rustc ")
    );
    let units = manifest["manifest_unit_context_rows"].as_array().unwrap();
    assert_eq!(units.len(), 2);
    assert!(units.iter().all(|unit| {
        unit["unit_mapped_target_flag"] == true
            && unit["unit_compilation_role_kind"] == "target"
            && unit["unit_rust_edition_value"] == "2024"
            && unit["unit_target_cfg_values"].is_array()
            && unit["unit_enabled_feature_values"].is_array()
            && unit["unit_semantic_identity_record"]["unit_configuration_digest_full"]
                .as_str()
                .is_some_and(|value| value.len() == 64)
    }));
    let files = units
        .iter()
        .flat_map(|unit| unit["unit_source_closure_rows"].as_array().unwrap())
        .map(|row| row["file_source_absolute_path"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(files.len(), 2);
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("/dependency/src/lib.rs"))
    );
    assert!(
        files
            .iter()
            .all(|path| !path.ends_with("unused.rs") && !path.ends_with("absent.rs"))
    );
    assert_eq!(
        manifest["manifest_configuration_state_record"]["workspace_source_root_path"],
        directory.path().join("workspace").to_str().unwrap()
    );
    let exported = decode_export_fixture_rows(&run_export_fixture_command(
        &database,
        "export-published-stage-rows",
        &["--stage-owned-table-name", "crate-dependency-graph-table"],
    ));
    assert_eq!(
        manifest["manifest_scope_identity_record"],
        exported[0]["export_scope_identity_record"]
    );
    let sites = decode_export_fixture_rows(&run_export_fixture_command(
        &database,
        "export-published-stage-rows",
        &["--stage-owned-table-name", "path-resolution-site-records"],
    ));
    assert!(sites.len() > 1);
    for row in &sites[1..] {
        let payload = &row["export_row_payload_value"];
        let source = &payload["site_compilation_scope_record"];
        let unit = units
            .iter()
            .find(|unit| {
                unit["unit_cargo_package_identifier"] == source["unit_cargo_package_identifier"]
            })
            .unwrap();
        assert_eq!(
            source["unit_semantic_identity_record"],
            unit["unit_semantic_identity_record"]
        );
        assert_eq!(payload["site_path_context_kind"], "signature");
    }
}

#[test]
fn test_empty_manifest_unit_contexts() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("store.db");
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    store.initialize_workspace_stage_schema().unwrap();
    let scope = StageRunScopeValue {
        run_generation_digest_value: "empty".into(),
        stage_attempt_sequence_value: "one".into(),
    };
    let cancel = RunCancelFlagValue::default();
    let config = StageTableRowRecord {
        stage_owned_table_name: "workspace-config-state-records".into(),
        stage_row_record_key: "workspace".into(),
        stage_row_payload_text: json!({
            "workspace_source_root_path":directory.path(),"workspace_target_cfg_values":[],
            "workspace_compiler_version_text":"rustc fixture",
            "workspace_parser_source_record":{"parser_source_commit_hash":"fixture"},
        })
        .to_string(),
    };
    store
        .write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: &scope,
                stage_owner_name_value: "crate-graph",
                stage_unit_work_key: "workspace",
                stage_input_stage_names: &[],
                stage_unit_output_rows: &[config],
            },
            &cancel,
        )
        .unwrap();
    for stage in ["crate-graph", "source-ingest", "syntax-spans"] {
        store
            .write_stage_table_unit(
                &StageUnitWriteRequest {
                    stage_run_scope_value: &scope,
                    stage_owner_name_value: stage,
                    stage_unit_work_key: "__complete__",
                    stage_input_stage_names: &[],
                    stage_unit_output_rows: &[],
                },
                &cancel,
            )
            .unwrap();
    }
    store
        .publish_workspace_generation_manifest(&scope, "empty", &cancel)
        .unwrap();
    drop(store);
    let output = run_export_fixture_command(&database, "export-mapped-unit-manifest", &[]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let manifest: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(manifest["manifest_unit_context_rows"], json!([]));
    assert_eq!(
        manifest["manifest_scope_identity_record"]["run_generation_digest_value"],
        "empty"
    );
}

#[test]
fn test_export_errors_remain_explicit() {
    let (_directory, database) = create_export_workspace_fixture();
    let unknown = run_export_fixture_command(
        &database,
        "export-published-stage-rows",
        &["--stage-owned-table-name", "not-an-owned-table"],
    );
    assert!(!unknown.status.success());
    let missing = database.with_file_name("missing.db");
    let output = run_export_fixture_command(&missing, "export-mapped-unit-manifest", &[]);
    assert!(!output.status.success());
    assert!(!missing.exists());
    let unpublished = database.with_file_name("unpublished.db");
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(unpublished.to_str().unwrap())
            .unwrap();
    store.initialize_workspace_stage_schema().unwrap();
    drop(store);
    let output = run_export_fixture_command(&unpublished, "export-mapped-unit-manifest", &[]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("StagePublicationMissingError"));
}

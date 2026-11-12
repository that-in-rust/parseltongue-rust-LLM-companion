#![allow(clippy::unwrap_used)]

use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StageTableRowRecord, StageUnitWriteRequest,
};
use parseltongue_duckdb_analysis_adapter::{
    OracleComparisonFailureError, compare_oracle_path_sites,
};
use parseltongue_table_store_traits::StageTableStorePort;
use parseltongue_turso_store_adapter::TursoWorkspaceStoreAdapter;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

struct OracleComparisonTestFixture {
    _directory: tempfile::TempDir,
    source: PathBuf,
    primary: PathBuf,
    secondary: PathBuf,
    agreement: PathBuf,
    primary_value: Value,
}

fn run_comparison_cli_command(database: &Path, command: &str, extra: &[&str]) -> Output {
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

fn build_oracle_comparison_fixture() -> OracleComparisonTestFixture {
    let directory = tempfile::tempdir().unwrap();
    fs::create_dir(directory.path().join("src")).unwrap();
    fs::write(
        directory.path().join("src/lib.rs"),
        "pub struct Leaf; pub struct Uses(pub Leaf);",
    )
    .unwrap();
    let source = directory.path().join("source.db");
    let package_root = directory.path().to_str().unwrap().to_owned();
    let source_root = directory
        .path()
        .join("src/lib.rs")
        .to_str()
        .unwrap()
        .to_owned();
    let package_identifier = format!("path+file://{package_root}#comparison@0.1.0");
    let configuration = json!([[], [], "2024"]);
    let configuration_digest = blake3::hash(configuration.to_string().as_bytes())
        .to_hex()
        .to_string();
    let semantic_identity = json!({
        "unit_cargo_package_identifier":package_identifier,
        "unit_cargo_target_name":"comparison",
        "unit_cargo_target_kind":"lib",
        "unit_compilation_role_kind":"target",
        "unit_configuration_digest_full":configuration_digest,
    });
    let mut unit = json!({
        "unit_cargo_package_identifier":package_identifier,
        "unit_cargo_package_source":null,
        "unit_cargo_target_name":"comparison",
        "unit_cargo_target_kind":"lib",
        "unit_compilation_role_kind":"target",
        "unit_mapped_target_flag":true,
        "unit_source_root_path":source_root,
        "unit_package_root_path":package_root,
        "unit_rust_edition_value":"2024",
        "unit_enabled_feature_values":[],
        "unit_target_cfg_values":[],
        "unit_normal_dependency_rows":[],
        "unit_build_script_present":false,
    });
    let unit_key = blake3::hash(unit.to_string().as_bytes())
        .to_hex()
        .to_string();
    unit["unit_compilation_identity_key"] = json!(unit_key);
    let site_path = directory
        .path()
        .join("src/lib.rs")
        .to_str()
        .unwrap()
        .to_owned();
    let compilation_record = json!({
        "unit_compilation_identity_key":unit_key,
        "unit_source_root_path":source_root,
        "unit_package_root_path":package_root,
        "unit_cargo_package_identifier":package_identifier,
        "unit_cargo_target_name":"comparison",
        "unit_cargo_target_kind":"lib",
        "unit_compilation_role_kind":"target",
        "unit_rust_edition_value":"2024",
        "unit_target_cfg_values":[],
        "unit_enabled_feature_values":[],
        "unit_semantic_identity_record":semantic_identity,
    });
    let target = json!({
        "target_compilation_scope_record":compilation_record,
        "target_source_file_path":site_path,
        "target_source_range_start":11,
        "target_source_range_end":15,
        "target_symbol_kind_value":"Struct",
        "target_qualified_item_path":"comparison::Leaf",
        "target_namespace_kind_values":["type"],
    });
    let site = json!({
        "site_compilation_scope_record":compilation_record,
        "site_source_absolute_path":site_path,
        "site_source_range_start":34,
        "site_source_range_end":38,
        "site_path_context_kind":"signature",
        "site_expansion_provenance_chain":[],
        "path_site_source_text":"Leaf",
        "path_resolution_state_kind":"resolved",
        "path_target_symbol_rows":[target],
        "path_unresolved_reason_kind":null,
    });
    let mut oracle_site = site.clone();
    oracle_site["path_lookup_scope_kind"] = json!("module-scope");
    oracle_site["site_proc_macro_dependent_flag"] = json!(false);
    let cancel = RunCancelFlagValue::default();
    let source_store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(source.to_str().unwrap()).unwrap();
    source_store.initialize_workspace_stage_schema().unwrap();
    let source_scope = StageRunScopeValue {
        run_generation_digest_value: "source-generation".into(),
        stage_attempt_sequence_value: "source-attempt".into(),
    };
    write_fixture_stage_unit(
        &source_store,
        &source_scope,
        "crate-graph",
        "workspace",
        &[],
        &[StageTableRowRecord {
            stage_owned_table_name: "crate-dependency-graph-table".into(),
            stage_row_record_key: unit_key,
            stage_row_payload_text: unit.to_string(),
        }],
        &cancel,
    );
    for (owner, inputs) in [
        ("crate-graph", &[][..]),
        ("source-ingest", &["crate-graph"][..]),
        ("syntax-spans", &["source-ingest"][..]),
    ] {
        write_fixture_stage_unit(
            &source_store,
            &source_scope,
            owner,
            "__complete__",
            inputs,
            &[],
            &cancel,
        );
    }
    write_fixture_stage_unit(
        &source_store,
        &source_scope,
        "name-resolution",
        "crate/fixture",
        &["crate-graph", "source-ingest", "syntax-spans"],
        &[StageTableRowRecord {
            stage_owned_table_name: "path-resolution-site-records".into(),
            stage_row_record_key: "fixture-site".into(),
            stage_row_payload_text: site.to_string(),
        }],
        &cancel,
    );
    let source_scope = source_store
        .publish_workspace_generation_manifest(&source_scope, "source-generation", &cancel)
        .unwrap();
    drop(source_store);
    let primary_value = json!({
        "oracle_configuration_kind_value":"PRIMARY",
        "oracle_build_scripts_flag":false,
        "oracle_proc_macros_flag":true,
        "oracle_manifest_scope_record":{
            "run_generation_digest_value":source_scope.run_generation_digest_value,
            "stage_attempt_sequence_value":source_scope.stage_attempt_sequence_value,
        },
        "oracle_mapped_unit_count":1,
        "oracle_path_export_rows":[oracle_site],
        "oracle_proc_macro_rows":[],
        "oracle_source_omission_rows":[],
        "oracle_cached_script_rows":[],
        "oracle_build_script_deltas":[],
    });
    OracleComparisonTestFixture {
        source,
        primary: directory.path().join("primary.json"),
        secondary: directory.path().join("secondary.json"),
        agreement: directory.path().join("agreement.db"),
        _directory: directory,
        primary_value,
    }
}

fn write_fixture_stage_unit(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    owner: &str,
    work_key: &str,
    inputs: &[&str],
    rows: &[StageTableRowRecord],
    cancel: &RunCancelFlagValue,
) {
    store
        .write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: scope,
                stage_owner_name_value: owner,
                stage_unit_work_key: work_key,
                stage_input_stage_names: inputs,
                stage_unit_output_rows: rows,
            },
            cancel,
        )
        .unwrap();
}

fn write_oracle_comparison_inputs(
    fixture: &OracleComparisonTestFixture,
    primary: &Value,
    secondary: Option<&Value>,
) {
    fs::write(&fixture.primary, primary.to_string()).unwrap();
    let mut value = secondary.unwrap_or(primary).clone();
    value["oracle_configuration_kind_value"] = json!("SECONDARY");
    value["oracle_build_scripts_flag"] = json!(true);
    fs::write(&fixture.secondary, value.to_string()).unwrap();
}

fn run_oracle_comparison_fixture(fixture: &OracleComparisonTestFixture) -> Output {
    run_comparison_cli_command(
        &fixture.source,
        "oracle-edge-differ-run",
        &[
            "--primary-semantic-input-path",
            fixture.primary.to_str().unwrap(),
            "--secondary-semantic-input-path",
            fixture.secondary.to_str().unwrap(),
            "--agreement-store-file-path",
            fixture.agreement.to_str().unwrap(),
        ],
    )
}

fn decode_oracle_comparison_result(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

/// REQ-RUST-025.1, REQ-RUST-025.4
#[test]
fn test_exact_comparison_publishes_turso() {
    let fixture = build_oracle_comparison_fixture();
    let before = run_comparison_cli_command(
        &fixture.source,
        "export-published-stage-rows",
        &["--stage-owned-table-name", "path-resolution-site-records"],
    )
    .stdout;
    let mut primary = fixture.primary_value.clone();
    let duplicate = primary["oracle_path_export_rows"][0].clone();
    primary["oracle_path_export_rows"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    write_oracle_comparison_inputs(&fixture, &primary, None);
    let result = decode_oracle_comparison_result(&run_oracle_comparison_fixture(&fixture));
    assert_eq!(result["agreement_eligible_path_count"], 1);
    assert_eq!(result["agreement_match_count_value"], 1);
    assert_eq!(result["agreement_exact_match_rate"], 1.0);
    let rows = run_comparison_cli_command(
        &fixture.agreement,
        "export-published-stage-rows",
        &["--stage-owned-table-name", "oracle-edge-agreement-table"],
    );
    assert!(
        rows.status.success(),
        "{}",
        String::from_utf8_lossy(&rows.stderr)
    );
    assert!(
        String::from_utf8(rows.stdout)
            .unwrap()
            .contains("\"agreement_status_kind\":\"match\"")
    );
    assert_eq!(
        before,
        run_comparison_cli_command(
            &fixture.source,
            "export-published-stage-rows",
            &["--stage-owned-table-name", "path-resolution-site-records"],
        )
        .stdout
    );
}

/// REQ-RUST-025.4
#[test]
fn test_changed_targets_persist_semantically() {
    let fixture = build_oracle_comparison_fixture();
    let mut primary = fixture.primary_value.clone();
    primary["oracle_path_export_rows"][0]["path_target_symbol_rows"][0]["target_source_range_start"] =
        json!(10);
    let mut secondary = fixture.primary_value.clone();
    secondary["oracle_path_export_rows"][0]["path_target_symbol_rows"][0]["target_source_range_end"] =
        json!(16);
    write_oracle_comparison_inputs(&fixture, &primary, Some(&secondary));
    let result = decode_oracle_comparison_result(&run_oracle_comparison_fixture(&fixture));
    assert_eq!(result["agreement_mismatched_path_count"], 1);
    let rows = run_comparison_cli_command(
        &fixture.agreement,
        "export-published-stage-rows",
        &["--stage-owned-table-name", "oracle-edge-agreement-table"],
    );
    assert!(rows.status.success());
    let evidence = String::from_utf8(rows.stdout).unwrap();
    assert!(evidence.contains("agreement_primary_targets_json"));
    assert!(evidence.contains("agreement_secondary_targets_json"));
    assert!(evidence.contains(",10,15,"));
    assert!(evidence.contains(",11,16,"));
}

/// REQ-RUST-025.2, REQ-RUST-025.3
#[test]
fn test_empty_denominator_is_na() {
    let fixture = build_oracle_comparison_fixture();
    let mut primary = fixture.primary_value.clone();
    primary["oracle_path_export_rows"] = json!([]);
    write_oracle_comparison_inputs(&fixture, &primary, Some(&fixture.primary_value));
    let result = decode_oracle_comparison_result(&run_oracle_comparison_fixture(&fixture));
    assert_eq!(result["agreement_eligible_path_count"], 0);
    assert_eq!(result["agreement_exact_match_rate"], Value::Null);
    assert_eq!(result["agreement_denominator_status_kind"], "N/A");
    assert_eq!(result["agreement_secondary_only_count"], 1);
}

/// REQ-RUST-025.2
#[test]
fn test_unresolved_targets_remain_mismatch() {
    let fixture = build_oracle_comparison_fixture();
    let mut primary = fixture.primary_value.clone();
    primary["oracle_path_export_rows"][0]["path_target_symbol_rows"] = json!([]);
    write_oracle_comparison_inputs(&fixture, &primary, None);
    let result = decode_oracle_comparison_result(&run_oracle_comparison_fixture(&fixture));
    assert_eq!(result["agreement_eligible_path_count"], 1);
    assert_eq!(result["agreement_match_count_value"], 0);
    let mut proc = fixture.primary_value.clone();
    proc["oracle_path_export_rows"][0]["site_proc_macro_dependent_flag"] = json!(true);
    write_oracle_comparison_inputs(&fixture, &proc, None);
    let result = decode_oracle_comparison_result(&run_oracle_comparison_fixture(&fixture));
    assert_eq!(result["agreement_proc_excluded_count"], 1);
    proc["oracle_path_export_rows"][0]["path_target_symbol_rows"][0]["target_source_range_start"] =
        json!(0);
    write_oracle_comparison_inputs(&fixture, &proc, None);
    let result = decode_oracle_comparison_result(&run_oracle_comparison_fixture(&fixture));
    assert_eq!(result["agreement_eligible_path_count"], 1);
    assert_eq!(result["agreement_match_count_value"], 0);
    assert_eq!(result["agreement_proc_excluded_count"], 0);
}

/// REQ-RUST-025.0, REQ-RUST-025.4
#[test]
fn test_scope_oracle_errors_block() {
    let fixture = build_oracle_comparison_fixture();
    for key in [
        "oracle_manifest_scope_record",
        "oracle_configuration_kind_value",
        "oracle_path_export_rows",
    ] {
        let mut invalid = fixture.primary_value.clone();
        invalid[key] = Value::Null;
        write_oracle_comparison_inputs(&fixture, &invalid, None);
        assert!(!run_oracle_comparison_fixture(&fixture).status.success());
    }
    for key in [
        "run_generation_digest_value",
        "stage_attempt_sequence_value",
    ] {
        let mut mismatch = fixture.primary_value.clone();
        mismatch["oracle_manifest_scope_record"][key] = json!("different-source-scope");
        write_oracle_comparison_inputs(&fixture, &mismatch, None);
        assert!(!run_oracle_comparison_fixture(&fixture).status.success());
    }
    let mut missing = fixture.primary_value.clone();
    missing["oracle_source_omission_rows"] =
        json!([{"file_source_absolute_path":"/missing/source.rs"}]);
    write_oracle_comparison_inputs(&fixture, &missing, None);
    assert!(!run_oracle_comparison_fixture(&fixture).status.success());
    write_oracle_comparison_inputs(&fixture, &fixture.primary_value, None);
    let same = run_comparison_cli_command(
        &fixture.source,
        "oracle-edge-differ-run",
        &[
            "--primary-semantic-input-path",
            fixture.primary.to_str().unwrap(),
            "--secondary-semantic-input-path",
            fixture.secondary.to_str().unwrap(),
            "--agreement-store-file-path",
            fixture.source.to_str().unwrap(),
        ],
    );
    assert!(!same.status.success());
    assert!(String::from_utf8_lossy(&same.stderr).contains("cannot mutate source"));
}

/// REQ-RUST-025.0
#[test]
fn test_macro_failure_blocks_publish() {
    let fixture = build_oracle_comparison_fixture();
    let mut failed = fixture.primary_value.clone();
    failed["oracle_proc_macro_rows"] = json!([{
        "macro_source_root_path":"/fixture/macro/src/lib.rs",
        "macro_loading_status_value":"failed",
        "macro_loading_origin_kind":"workspace",
        "macro_loading_error_text":"fixture load failure",
    }]);
    write_oracle_comparison_inputs(&fixture, &failed, None);
    assert!(!run_oracle_comparison_fixture(&fixture).status.success());
}

/// REQ-RUST-025.0
#[test]
fn test_malformed_macro_record_blocks_publish() {
    let fixture = build_oracle_comparison_fixture();
    let mut malformed = fixture.primary_value.clone();
    malformed["oracle_proc_macro_rows"] = json!([{
        "macro_source_root_path":"/fixture/macro/src/lib.rs",
        "macro_loading_status_value":"loaded",
        "macro_loading_origin_kind":"workspace",
    }]);
    write_oracle_comparison_inputs(&fixture, &malformed, None);
    assert!(!run_oracle_comparison_fixture(&fixture).status.success());
}

/// REQ-RUST-025.4
#[test]
fn test_oracle_agreement_registry_roundtrip() {
    let directory = tempfile::tempdir().unwrap();
    let store_path = directory.path().join("agreement.db");
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(store_path.to_str().unwrap())
            .unwrap();
    store.initialize_workspace_stage_schema().unwrap();
    let scope = StageRunScopeValue {
        run_generation_digest_value: "registry-generation".into(),
        stage_attempt_sequence_value: "registry-attempt".into(),
    };
    let cancel = RunCancelFlagValue::default();
    for (owner, inputs) in [
        ("crate-graph", &[][..]),
        ("source-ingest", &["crate-graph"][..]),
        ("syntax-spans", &["source-ingest"][..]),
    ] {
        write_fixture_stage_unit(&store, &scope, owner, "__complete__", inputs, &[], &cancel);
    }
    let expected = StageTableRowRecord {
        stage_owned_table_name: "oracle-edge-agreement-table".into(),
        stage_row_record_key: "site/fixture-site".into(),
        stage_row_payload_text: json!({"agreement_status_kind":"match"}).to_string(),
    };
    write_fixture_stage_unit(
        &store,
        &scope,
        "oracle-agreement",
        "comparison",
        &["crate-graph", "source-ingest", "syntax-spans"],
        std::slice::from_ref(&expected),
        &cancel,
    );
    write_fixture_stage_unit(
        &store,
        &scope,
        "oracle-agreement",
        "__complete__",
        &[],
        &[],
        &cancel,
    );
    let published = store
        .publish_workspace_generation_manifest(&scope, "registry-generation", &cancel)
        .unwrap();
    assert_eq!(
        store
            .read_stage_table_rows(&published, "oracle-edge-agreement-table")
            .unwrap(),
        [expected]
    );
    drop(store);
    let exported = run_comparison_cli_command(
        &store_path,
        "export-published-stage-rows",
        &["--stage-owned-table-name", "oracle-edge-agreement-table"],
    );
    assert!(
        exported.status.success(),
        "{}",
        String::from_utf8_lossy(&exported.stderr)
    );
    assert!(
        String::from_utf8_lossy(&exported.stdout).contains("\"agreement_status_kind\":\"match\"")
    );
}

/// REQ-RUST-025.4
#[test]
fn test_cancel_prevents_agreement_publish() {
    let fixture = build_oracle_comparison_fixture();
    let source =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(fixture.source.to_str().unwrap())
            .unwrap();
    let agreement =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(fixture.agreement.to_str().unwrap())
            .unwrap();
    agreement.initialize_workspace_stage_schema().unwrap();
    let cancel = RunCancelFlagValue::default();
    cancel.request_stage_work_cancellation();
    assert!(matches!(
        compare_oracle_path_sites(
            &source,
            &fixture.primary,
            &fixture.secondary,
            &agreement,
            &cancel,
        ),
        Err(OracleComparisonFailureError::Cancelled)
    ));
    assert_eq!(agreement.read_workspace_publication_scope().unwrap(), None);
}

/// REQ-RUST-025.0
#[test]
fn test_source_unpublished_generation_fails() {
    let directory = tempfile::tempdir().unwrap();
    let source_path = directory.path().join("unpublished-source.db");
    let agreement_path = directory.path().join("unpublished-agreement.db");
    let source =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(source_path.to_str().unwrap())
            .unwrap();
    let agreement =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(agreement_path.to_str().unwrap())
            .unwrap();
    source.initialize_workspace_stage_schema().unwrap();
    agreement.initialize_workspace_stage_schema().unwrap();
    let cancel = RunCancelFlagValue::default();
    let missing = directory.path().join("missing-oracle.json");
    assert!(matches!(
        compare_oracle_path_sites(&source, &missing, &missing, &agreement, &cancel,),
        Err(OracleComparisonFailureError::SourceNotPublished)
    ));
    assert_eq!(agreement.read_workspace_publication_scope().unwrap(), None);
}

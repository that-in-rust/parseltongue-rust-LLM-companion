#![allow(clippy::unwrap_used)]

use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StageTableRowRecord, StageUnitWriteRequest,
    StoreBackendFailureError,
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
    sync::atomic::{AtomicUsize, Ordering},
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
    build_oracle_comparison_fixture_with_matrix(false, None, None, None)
}

fn build_oracle_comparison_matrix_fixture() -> OracleComparisonTestFixture {
    build_oracle_comparison_fixture_with_matrix(true, None, None, None)
}

fn build_oracle_comparison_fixture_with_matrix(
    include_matrix: bool,
    source_reason: Option<&str>,
    source_targets: Option<Value>,
    source_target_range_start: Option<u64>,
) -> OracleComparisonTestFixture {
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
    let configuration = json!([[], [], "2024", "target"]);
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
    let (mut source_sites, oracle_sites) = if include_matrix {
        let mut wrong_source = site.clone();
        wrong_source["site_source_range_start"] = json!(40);
        wrong_source["site_source_range_end"] = json!(44);
        let mut wrong_oracle = wrong_source.clone();
        wrong_oracle["path_lookup_scope_kind"] = json!("module-scope");
        wrong_oracle["site_proc_macro_dependent_flag"] = json!(false);
        wrong_oracle["path_target_symbol_rows"][0]["target_source_range_start"] = json!(10);
        let mut missing_oracle = oracle_site.clone();
        missing_oracle["site_source_range_start"] = json!(45);
        missing_oracle["site_source_range_end"] = json!(49);
        let mut deferred_oracle = oracle_site.clone();
        deferred_oracle["site_source_range_start"] = json!(55);
        deferred_oracle["site_source_range_end"] = json!(59);
        deferred_oracle["path_lookup_scope_kind"] = json!("type-relative");
        let mut extra_source = site.clone();
        extra_source["site_source_range_start"] = json!(50);
        extra_source["site_source_range_end"] = json!(54);
        (
            vec![site.clone(), wrong_source, extra_source],
            vec![oracle_site, wrong_oracle, missing_oracle, deferred_oracle],
        )
    } else {
        (vec![site.clone()], vec![oracle_site])
    };
    for source_site in &mut source_sites {
        if let Some(reason) = source_reason {
            source_site["path_unresolved_reason_kind"] = json!(reason);
        }
        if let Some(targets) = &source_targets {
            source_site["path_target_symbol_rows"] = targets.clone();
        }
        if let Some(range_start) = source_target_range_start
            && let Some(target) = source_site
                .get_mut("path_target_symbol_rows")
                .and_then(Value::as_array_mut)
                .and_then(|targets| targets.first_mut())
        {
            target["target_source_range_start"] = json!(range_start);
        }
    }
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
        &source_sites
            .iter()
            .enumerate()
            .map(|(index, site)| StageTableRowRecord {
                stage_owned_table_name: "path-resolution-site-records".into(),
                stage_row_record_key: format!("fixture-site-{index}"),
                stage_row_payload_text: site.to_string(),
            })
            .collect::<Vec<_>>(),
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
        "oracle_path_export_rows":oracle_sites,
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

struct DelegatingStageStorePort<'a> {
    inner: &'a TursoWorkspaceStoreAdapter,
    changed_scope_read: Option<usize>,
    scope_read_count: AtomicUsize,
    cancel_after_write: Option<&'a RunCancelFlagValue>,
}

impl StageTableStorePort for DelegatingStageStorePort<'_> {
    fn initialize_workspace_stage_schema(&self) -> Result<(), StoreBackendFailureError> {
        self.inner.initialize_workspace_stage_schema()
    }

    fn write_stage_table_unit(
        &self,
        request: &StageUnitWriteRequest<'_>,
        cancel: &RunCancelFlagValue,
    ) -> Result<usize, StoreBackendFailureError> {
        let written = self.inner.write_stage_table_unit(request, cancel)?;
        if let Some(cancel) = self.cancel_after_write {
            cancel.request_stage_work_cancellation();
        }
        Ok(written)
    }

    fn read_stage_table_rows(
        &self,
        scope: &StageRunScopeValue,
        table: &str,
    ) -> Result<Vec<StageTableRowRecord>, StoreBackendFailureError> {
        self.inner.read_stage_table_rows(scope, table)
    }

    fn prepare_workspace_generation_scope(
        &self,
        scope: &StageRunScopeValue,
        generation: &str,
        cancel: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, StoreBackendFailureError> {
        self.inner
            .prepare_workspace_generation_scope(scope, generation, cancel)
    }

    fn write_file_content_bytes(
        &self,
        digest: &str,
        bytes: &[u8],
    ) -> Result<bool, StoreBackendFailureError> {
        self.inner.write_file_content_bytes(digest, bytes)
    }

    fn read_file_content_bytes(&self, digest: &str) -> Result<Vec<u8>, StoreBackendFailureError> {
        self.inner.read_file_content_bytes(digest)
    }

    fn publish_workspace_generation_manifest(
        &self,
        scope: &StageRunScopeValue,
        generation: &str,
        cancel: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, StoreBackendFailureError> {
        self.inner
            .publish_workspace_generation_manifest(scope, generation, cancel)
    }

    fn read_workspace_publication_scope(
        &self,
    ) -> Result<Option<StageRunScopeValue>, StoreBackendFailureError> {
        let read_number = self.scope_read_count.fetch_add(1, Ordering::SeqCst) + 1;
        if self.changed_scope_read == Some(read_number) {
            return Ok(Some(StageRunScopeValue {
                run_generation_digest_value: "changed-source-generation".into(),
                stage_attempt_sequence_value: "changed-source-attempt".into(),
            }));
        }
        self.inner.read_workspace_publication_scope()
    }
}

fn seed_existing_agreement_generation(
    agreement: &TursoWorkspaceStoreAdapter,
) -> StageRunScopeValue {
    agreement.initialize_workspace_stage_schema().unwrap();
    let scope = StageRunScopeValue {
        run_generation_digest_value: "preexisting-agreement-generation".into(),
        stage_attempt_sequence_value: "preexisting-agreement-attempt".into(),
    };
    let cancel = RunCancelFlagValue::default();
    for (owner, inputs) in [
        ("crate-graph", &[][..]),
        ("source-ingest", &["crate-graph"][..]),
        ("syntax-spans", &["source-ingest"][..]),
    ] {
        write_fixture_stage_unit(
            agreement,
            &scope,
            owner,
            "__complete__",
            inputs,
            &[],
            &cancel,
        );
    }
    write_fixture_stage_unit(
        agreement,
        &scope,
        "oracle-agreement",
        "comparison",
        &["crate-graph", "source-ingest", "syntax-spans"],
        &[StageTableRowRecord {
            stage_owned_table_name: "oracle-edge-agreement-table".into(),
            stage_row_record_key: "site/preexisting".into(),
            stage_row_payload_text: json!({"agreement_status_kind":"match"}).to_string(),
        }],
        &cancel,
    );
    write_fixture_stage_unit(
        agreement,
        &scope,
        "oracle-agreement",
        "__complete__",
        &[],
        &[],
        &cancel,
    );
    agreement
        .publish_workspace_generation_manifest(&scope, &scope.run_generation_digest_value, &cancel)
        .unwrap()
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

fn canonicalize_persisted_export_rows(export: &[u8]) -> Vec<Value> {
    let mut rows = String::from_utf8_lossy(export)
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    for row in &mut rows {
        if row["export_record_kind_value"] == "header"
            && let Some(scope) = row["export_scope_identity_record"].as_object_mut()
        {
            scope.remove("stage_attempt_sequence_value");
        }
        if row["export_row_identity_value"] == "summary"
            && let Some(summary) = row["export_row_payload_value"].as_object_mut()
        {
            summary.remove("agreement_operational_identity");
        }
    }
    rows.sort_by_key(Value::to_string);
    rows
}

/// REQ-RUST-026.1, REQ-RUST-026.4
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

/// REQ-RUST-026.4
#[test]
fn test_comparison_telemetry_survives_publication() {
    let fixture = build_oracle_comparison_fixture();
    write_oracle_comparison_inputs(&fixture, &fixture.primary_value, None);
    let result = decode_oracle_comparison_result(&run_oracle_comparison_fixture(&fixture));
    let operational = &result["agreement_operational_identity"];
    let settings = &operational["duckdb_settings"];
    for setting in [
        "memory_limit",
        "temp_directory",
        "enable_external_access",
        "autoload_known_extensions",
        "autoinstall_known_extensions",
        "allow_community_extensions",
        "threads",
    ] {
        assert!(
            settings[setting].is_string()
                || settings[setting].is_boolean()
                || settings[setting].is_number(),
            "{setting}"
        );
    }
    assert_eq!(settings["enable_external_access"], false);
    assert_eq!(settings["autoload_known_extensions"], false);
    assert_eq!(settings["autoinstall_known_extensions"], false);
    assert_eq!(settings["allow_community_extensions"], false);
    assert_eq!(settings["threads"], 1);
    let database_backing = &operational["duckdb_database_backing"];
    assert_eq!(database_backing["file_backed_flag"], true);
    let database_file_path = database_backing["database_file_path"].as_str().unwrap();
    assert!(Path::new(database_file_path).is_absolute());
    assert!(database_file_path.ends_with("oracle-path-comparison-database.duckdb"));
    assert!(!Path::new(database_file_path).exists());
    for snapshot in [
        "duckdb_memory_before_analysis",
        "duckdb_memory_after_analysis",
    ] {
        assert!(operational[snapshot].is_array(), "{snapshot}");
        assert!(
            operational[snapshot]
                .as_array()
                .unwrap()
                .iter()
                .all(|record| {
                    record["tag"].is_string()
                        && record["memory_usage_bytes"].is_number()
                        && record["temporary_storage_bytes"].is_number()
                })
        );
    }
    let spill_path = settings["temp_directory"].as_str().unwrap();
    assert!(!Path::new(spill_path).exists());
    let table_evidence = result["agreement_source_copy_table_evidence"]
        .as_array()
        .unwrap();
    assert_eq!(table_evidence.len(), 2);
    for table in table_evidence {
        assert_eq!(table["expected_row_count"], table["observed_row_count"]);
        assert_eq!(
            table["expected_framed_digest_full"],
            table["observed_framed_digest_full"]
        );
        assert_eq!(
            table["source_scope_generation_value"],
            table["observed_source_scope_generation_value"]
        );
        assert_eq!(
            table["source_scope_attempt_sequence_value"],
            table["observed_source_scope_attempt_sequence_value"]
        );
    }

    let agreement =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(fixture.agreement.to_str().unwrap())
            .unwrap();
    let scope = agreement
        .read_workspace_publication_scope()
        .unwrap()
        .unwrap();
    let summary = agreement
        .read_stage_table_rows(&scope, "oracle-edge-agreement-table")
        .unwrap()
        .into_iter()
        .find(|row| row.stage_row_record_key == "summary")
        .unwrap();
    let persisted: Value = serde_json::from_str(&summary.stage_row_payload_text).unwrap();
    assert_eq!(
        persisted["agreement_operational_identity"],
        result["agreement_operational_identity"]
    );
    assert_eq!(
        persisted["agreement_source_copy_table_evidence"],
        result["agreement_source_copy_table_evidence"]
    );
}

/// REQ-RUST-026.2, REQ-RUST-026.3, REQ-RUST-026.4
#[test]
fn test_persisted_known_answer_matrix() {
    let fixture = build_oracle_comparison_matrix_fixture();
    write_oracle_comparison_inputs(&fixture, &fixture.primary_value, None);
    let result = decode_oracle_comparison_result(&run_oracle_comparison_fixture(&fixture));
    assert_eq!(result["agreement_eligible_path_count"], 3);
    assert_eq!(result["agreement_match_count_value"], 1);
    assert_eq!(result["agreement_exact_match_rate"], 1.0 / 3.0);
    assert_eq!(result["agreement_deferred_path_count"], 1);
    assert_eq!(result["agreement_production_only_path_count"], 1);
    assert_eq!(result["agreement_missing_production_path_count"], 1);

    let rows = run_comparison_cli_command(
        &fixture.agreement,
        "export-published-stage-rows",
        &["--stage-owned-table-name", "oracle-edge-agreement-table"],
    );
    assert!(rows.status.success());
    let first_export = String::from_utf8(rows.stdout).unwrap();
    assert!(first_export.contains("associated-deferred"));
    assert!(first_export.contains("missing-production"));
    assert!(first_export.contains("production-only"));
    assert_eq!(first_export.matches("agreement_site_key").count(), 5);

    write_oracle_comparison_inputs(&fixture, &fixture.primary_value, None);
    let repeated = decode_oracle_comparison_result(&run_oracle_comparison_fixture(&fixture));
    assert_eq!(
        result["agreement_semantic_export_digest_full"],
        repeated["agreement_semantic_export_digest_full"]
    );
    assert_ne!(
        result["agreement_operational_identity"],
        repeated["agreement_operational_identity"]
    );
    let rows = run_comparison_cli_command(
        &fixture.agreement,
        "export-published-stage-rows",
        &["--stage-owned-table-name", "oracle-edge-agreement-table"],
    );
    assert!(rows.status.success());
    assert_eq!(
        canonicalize_persisted_export_rows(first_export.as_bytes()),
        canonicalize_persisted_export_rows(&rows.stdout)
    );

    let mut changed_primary = fixture.primary_value.clone();
    changed_primary["oracle_path_export_rows"][0]["path_target_symbol_rows"][0]["target_source_range_start"] =
        json!(9);
    write_oracle_comparison_inputs(&fixture, &changed_primary, None);
    let changed = decode_oracle_comparison_result(&run_oracle_comparison_fixture(&fixture));
    assert_ne!(
        result["agreement_generation_digest_value"],
        changed["agreement_generation_digest_value"]
    );
    let rows = run_comparison_cli_command(
        &fixture.agreement,
        "export-published-stage-rows",
        &["--stage-owned-table-name", "oracle-edge-agreement-table"],
    );
    assert!(rows.status.success());
    let changed_export = String::from_utf8(rows.stdout).unwrap();
    assert_eq!(changed_export.matches("agreement_site_key").count(), 5);
    assert!(changed_export.contains(",9,15,"));
}

/// REQ-RUST-026.3
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

/// REQ-RUST-026.2
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

/// REQ-RUST-026.2
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

/// REQ-RUST-026.0, REQ-RUST-026.4
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

/// REQ-RUST-026.0
#[test]
fn test_oracle_envelope_flags_strict() {
    let fixture = build_oracle_comparison_fixture();
    write_oracle_comparison_inputs(&fixture, &fixture.primary_value, None);
    let source =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(fixture.source.to_str().unwrap())
            .unwrap();
    let agreement =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(fixture.agreement.to_str().unwrap())
            .unwrap();
    let previous_scope = seed_existing_agreement_generation(&agreement);
    let original_primary: Value =
        serde_json::from_slice(&fs::read(&fixture.primary).unwrap()).unwrap();
    let original_secondary: Value =
        serde_json::from_slice(&fs::read(&fixture.secondary).unwrap()).unwrap();
    let mut invalid_case_count = 0;
    for configuration in ["PRIMARY", "SECONDARY"] {
        let expected_build_scripts = configuration == "SECONDARY";
        for (field, expected) in [
            ("oracle_build_scripts_flag", expected_build_scripts),
            ("oracle_proc_macros_flag", true),
        ] {
            for invalid_value_kind in ["missing", "null", "wrong-boolean", "wrong-type"] {
                let mut primary = original_primary.clone();
                let mut secondary = original_secondary.clone();
                let envelope = if configuration == "PRIMARY" {
                    &mut primary
                } else {
                    &mut secondary
                };
                match invalid_value_kind {
                    "missing" => {
                        envelope.as_object_mut().unwrap().remove(field);
                    }
                    "null" => envelope[field] = Value::Null,
                    "wrong-boolean" => envelope[field] = json!(!expected),
                    "wrong-type" => envelope[field] = json!("true"),
                    _ => unreachable!(),
                }
                fs::write(&fixture.primary, primary.to_string()).unwrap();
                fs::write(&fixture.secondary, secondary.to_string()).unwrap();
                let comparison = compare_oracle_path_sites(
                    &source,
                    &fixture.primary,
                    &fixture.secondary,
                    &agreement,
                    &RunCancelFlagValue::default(),
                );
                match comparison {
                    Err(OracleComparisonFailureError::InvalidEnvelope {
                        configuration: actual_configuration,
                        field: actual_field,
                    }) => {
                        assert_eq!(actual_configuration, configuration);
                        assert_eq!(actual_field, field);
                    }
                    other => panic!(
                        "{configuration} {field} {invalid_value_kind} must be rejected before publication: {other:?}"
                    ),
                }
                assert_eq!(
                    agreement.read_workspace_publication_scope().unwrap(),
                    Some(previous_scope.clone())
                );
                let rows = agreement
                    .read_stage_table_rows(&previous_scope, "oracle-edge-agreement-table")
                    .unwrap();
                assert!(
                    rows.iter()
                        .any(|row| row.stage_row_record_key == "site/preexisting")
                );
                invalid_case_count += 1;
            }
        }
    }
    assert_eq!(invalid_case_count, 16);
}

/// REQ-RUST-026.0
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

/// REQ-RUST-026.0
#[test]
fn test_malformed_macro_record_rejected() {
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

/// REQ-RUST-026.4
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

/// REQ-RUST-026.4
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

/// REQ-RUST-026.4
#[test]
fn test_source_changes_after_reads() {
    let fixture = build_oracle_comparison_fixture();
    let source =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(fixture.source.to_str().unwrap())
            .unwrap();
    let agreement =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(fixture.agreement.to_str().unwrap())
            .unwrap();
    let previous_scope = seed_existing_agreement_generation(&agreement);
    let source_port = DelegatingStageStorePort {
        inner: &source,
        changed_scope_read: Some(2),
        scope_read_count: AtomicUsize::new(0),
        cancel_after_write: None,
    };
    let cancel = RunCancelFlagValue::default();
    let comparison = compare_oracle_path_sites(
        &source_port,
        &fixture.primary,
        &fixture.secondary,
        &agreement,
        &cancel,
    );
    assert!(
        matches!(comparison, Err(OracleComparisonFailureError::SourceChanged)),
        "{comparison:?}"
    );
    assert_eq!(
        agreement.read_workspace_publication_scope().unwrap(),
        Some(previous_scope)
    );
}

/// REQ-RUST-026.4
#[test]
fn test_source_changes_before_publish() {
    let fixture = build_oracle_comparison_fixture();
    write_oracle_comparison_inputs(&fixture, &fixture.primary_value, None);
    let source =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(fixture.source.to_str().unwrap())
            .unwrap();
    let agreement =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(fixture.agreement.to_str().unwrap())
            .unwrap();
    let previous_scope = seed_existing_agreement_generation(&agreement);
    let source_port = DelegatingStageStorePort {
        inner: &source,
        changed_scope_read: Some(4),
        scope_read_count: AtomicUsize::new(0),
        cancel_after_write: None,
    };
    let cancel = RunCancelFlagValue::default();
    let comparison = compare_oracle_path_sites(
        &source_port,
        &fixture.primary,
        &fixture.secondary,
        &agreement,
        &cancel,
    );
    assert!(
        matches!(comparison, Err(OracleComparisonFailureError::SourceChanged)),
        "{comparison:?}"
    );
    assert_eq!(
        agreement.read_workspace_publication_scope().unwrap(),
        Some(previous_scope)
    );
}

/// REQ-RUST-026.4
#[test]
fn test_write_cancellation_preserves_generation() {
    let fixture = build_oracle_comparison_fixture();
    write_oracle_comparison_inputs(&fixture, &fixture.primary_value, None);
    let source =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(fixture.source.to_str().unwrap())
            .unwrap();
    let agreement =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(fixture.agreement.to_str().unwrap())
            .unwrap();
    let previous_scope = seed_existing_agreement_generation(&agreement);
    let cancel = RunCancelFlagValue::default();
    let agreement_port = DelegatingStageStorePort {
        inner: &agreement,
        changed_scope_read: None,
        scope_read_count: AtomicUsize::new(0),
        cancel_after_write: Some(&cancel),
    };
    let comparison = compare_oracle_path_sites(
        &source,
        &fixture.primary,
        &fixture.secondary,
        &agreement_port,
        &cancel,
    );
    assert!(
        matches!(comparison, Err(OracleComparisonFailureError::Cancelled)),
        "{comparison:?}"
    );
    assert_eq!(
        agreement.read_workspace_publication_scope().unwrap(),
        Some(previous_scope)
    );
}

/// REQ-RUST-026.4
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

/// REQ-RUST-026.2
#[test]
fn test_production_cfg_keeps_mismatch() {
    let fixture = build_oracle_comparison_fixture_with_matrix(
        false,
        Some("cfg_inactive"),
        Some(json!([])),
        None,
    );
    write_oracle_comparison_inputs(&fixture, &fixture.primary_value, None);
    let result = decode_oracle_comparison_result(&run_oracle_comparison_fixture(&fixture));
    assert_eq!(result["agreement_eligible_path_count"], 1);
    assert_eq!(result["agreement_match_count_value"], 0);
    assert_eq!(result["agreement_mismatched_path_count"], 1);
    assert_eq!(result["agreement_cfg_excluded_path_count"], 0);
    assert_eq!(
        result["agreement_production_reason_counts"]["cfg_inactive"],
        1
    );
}

/// REQ-RUST-026.2
#[test]
fn test_secondary_cfg_preserves_match() {
    let fixture = build_oracle_comparison_fixture();
    let mut secondary = fixture.primary_value.clone();
    secondary["oracle_path_export_rows"][0]["path_unresolved_reason_kind"] = json!("cfg_inactive");
    write_oracle_comparison_inputs(&fixture, &fixture.primary_value, Some(&secondary));
    let result = decode_oracle_comparison_result(&run_oracle_comparison_fixture(&fixture));
    assert_eq!(result["agreement_eligible_path_count"], 1);
    assert_eq!(result["agreement_match_count_value"], 1);
    assert_eq!(result["agreement_cfg_excluded_path_count"], 0);
    let rows = run_comparison_cli_command(
        &fixture.agreement,
        "export-published-stage-rows",
        &["--stage-owned-table-name", "oracle-edge-agreement-table"],
    );
    assert!(rows.status.success());
    let evidence = String::from_utf8_lossy(&rows.stdout);
    assert!(evidence.contains("agreement_secondary_unresolved_reason_kind"));
    assert!(evidence.contains("cfg_inactive"));
}

/// REQ-RUST-026.2
#[test]
fn test_cfg_wrong_target_mismatch() {
    let fixture =
        build_oracle_comparison_fixture_with_matrix(false, Some("cfg_inactive"), None, Some(10));
    write_oracle_comparison_inputs(&fixture, &fixture.primary_value, None);
    let result = decode_oracle_comparison_result(&run_oracle_comparison_fixture(&fixture));
    assert_eq!(result["agreement_eligible_path_count"], 1);
    assert_eq!(result["agreement_match_count_value"], 0);
    assert_eq!(result["agreement_mismatched_path_count"], 1);
    assert_eq!(result["agreement_cfg_excluded_path_count"], 0);
}

/// REQ-RUST-026.2
#[test]
fn test_unknown_cfg_remains_blocked() {
    let fixture = build_oracle_comparison_fixture();
    let mut primary = fixture.primary_value.clone();
    primary["oracle_path_export_rows"][0]["path_lookup_scope_kind"] = json!("unknown");
    primary["oracle_path_export_rows"][0]["path_unresolved_reason_kind"] = json!("cfg_inactive");
    write_oracle_comparison_inputs(&fixture, &primary, None);
    let result = decode_oracle_comparison_result(&run_oracle_comparison_fixture(&fixture));
    assert_eq!(result["agreement_denominator_status_kind"], "BLOCKED");
    assert_eq!(result["agreement_blocked_path_count"], 1);
    assert_eq!(result["agreement_cfg_excluded_path_count"], 0);
}

/// REQ-RUST-026.2
#[test]
fn test_primary_cfg_excludes_scored() {
    let fixture = build_oracle_comparison_fixture_with_matrix(false, None, Some(json!([])), None);
    let mut primary = fixture.primary_value.clone();
    primary["oracle_path_export_rows"][0]["path_target_symbol_rows"] = json!([]);
    primary["oracle_path_export_rows"][0]["path_unresolved_reason_kind"] = json!("cfg_inactive");
    write_oracle_comparison_inputs(&fixture, &primary, None);
    let result = decode_oracle_comparison_result(&run_oracle_comparison_fixture(&fixture));
    assert_eq!(result["agreement_eligible_path_count"], 0);
    assert_eq!(result["agreement_cfg_excluded_path_count"], 1);
    assert_eq!(result["agreement_primary_reason_counts"]["cfg_inactive"], 1);
}

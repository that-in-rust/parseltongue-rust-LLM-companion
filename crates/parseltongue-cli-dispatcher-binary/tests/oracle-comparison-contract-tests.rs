#![allow(clippy::unwrap_used)]

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
        directory.path().join("Cargo.toml"),
        "[package]\nname=\"comparison\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
    )
    .unwrap();
    fs::write(
        directory.path().join("src/lib.rs"),
        "#![no_std]\n#![no_implicit_prelude]\npub struct Leaf; pub struct Uses(pub Leaf);",
    )
    .unwrap();
    let source = directory.path().join("source.db");
    let lock = Command::new(env!("CARGO"))
        .current_dir(directory.path())
        .args(["generate-lockfile", "--offline"])
        .output()
        .unwrap();
    assert!(
        lock.status.success(),
        "{}",
        String::from_utf8_lossy(&lock.stderr)
    );
    let mapped = run_comparison_cli_command(
        &source,
        "map-workspace-full-run",
        &[
            "--workspace-root-path-value",
            directory.path().to_str().unwrap(),
        ],
    );
    assert!(
        mapped.status.success(),
        "{}",
        String::from_utf8_lossy(&mapped.stderr)
    );
    let manifest = run_comparison_cli_command(&source, "export-mapped-unit-manifest", &[]);
    assert!(
        manifest.status.success(),
        "{}",
        String::from_utf8_lossy(&manifest.stderr)
    );
    let manifest: Value = serde_json::from_slice(&manifest.stdout).unwrap();
    let exported = run_comparison_cli_command(
        &source,
        "export-published-stage-rows",
        &["--stage-owned-table-name", "path-resolution-site-records"],
    );
    assert!(
        exported.status.success(),
        "{}",
        String::from_utf8_lossy(&exported.stderr)
    );
    let rows = String::from_utf8(exported.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .filter(|row| row["export_record_kind_value"] == "row")
        .map(|row| row["export_row_payload_value"].clone())
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 1);
    let primary_value = json!({
        "oracle_configuration_kind_value":"PRIMARY",
        "oracle_manifest_scope_record":manifest["manifest_scope_identity_record"],
        "oracle_mapped_unit_count":manifest["manifest_unit_context_rows"].as_array().unwrap().iter()
            .filter(|unit|unit["unit_mapped_target_flag"]==true).count(),
        "oracle_path_export_rows":rows,"oracle_proc_macro_rows":[],
        "oracle_source_omission_rows":[],"oracle_build_script_deltas":[],
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

fn write_oracle_comparison_inputs(
    fixture: &OracleComparisonTestFixture,
    primary: &Value,
    secondary: Option<&Value>,
) {
    fs::write(&fixture.primary, primary.to_string()).unwrap();
    let mut value = secondary.unwrap_or(primary).clone();
    value["oracle_configuration_kind_value"] = json!("SECONDARY");
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

#[test]
fn test_exact_comparison_publishes_turso() {
    let fixture = build_oracle_comparison_fixture();
    let before =
        run_comparison_cli_command(&fixture.source, "export-mapped-unit-manifest", &[]).stdout;
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
        run_comparison_cli_command(&fixture.source, "export-mapped-unit-manifest", &[]).stdout
    );
}

#[test]
fn test_empty_primary_is_not_success() {
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

#[test]
fn test_unresolved_and_wrong_targets_fail() {
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

#[test]
fn test_scope_and_oracle_errors_block() {
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

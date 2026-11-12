#![allow(clippy::unwrap_used)]

use crate::{
    OracleConfigurationKindValue, OracleWorkspaceExportConfig, export_mapped_semantic_paths,
};
use serde_json::{Value, json};
use std::{fs, process::Command};

fn build_mapped_oracle_fixture() -> (tempfile::TempDir, OracleWorkspaceExportConfig, Value) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("defs/src")).unwrap();
    fs::write(root.join("Cargo.toml"), "[package]\nname=\"mapped\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[dependencies]\ndefs={path=\"defs\",default-features=false,features=[\"wanted\"]}\n[dev-dependencies]\ndefs={path=\"defs\"}\n").unwrap();
    fs::write(root.join("defs/Cargo.toml"), "[package]\nname=\"defs\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[features]\ndefault=[\"wrong\"]\nwrong=[]\nwanted=[]\n").unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub struct Wrapper(pub defs::Chosen);",
    )
    .unwrap();
    fs::write(root.join("defs/src/lib.rs"), "pub struct Local; #[cfg(feature=\"wanted\")] pub struct Chosen(pub Local); pub struct Spare; #[cfg(feature=\"wrong\")] pub struct Omitted(pub Spare);").unwrap();
    let output = Command::new(env!("CARGO"))
        .current_dir(root)
        .args(["metadata", "--offline", "--format-version=1"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: Value = serde_json::from_slice(&output.stdout).unwrap();
    let units = metadata["packages"].as_array().unwrap().iter().map(|package| {
        let target = &package["targets"][0];
        let features = if package["name"]=="defs" {vec!["wanted"]} else {vec![]};
        let cfg = features.iter().map(|feature| format!("feature={feature:?}")).collect::<Vec<_>>();
        let semantic = json!({
            "unit_cargo_package_identifier":package["id"],"unit_cargo_target_name":target["name"],
            "unit_cargo_target_kind":"lib","unit_compilation_role_kind":"target",
            "unit_configuration_digest_full":blake3::hash(json!([cfg,features,"2024","target"]).to_string().as_bytes()).to_hex().to_string(),
        });
        let source = target["src_path"].as_str().unwrap();
        let mut unit = json!({
            "unit_cargo_package_identifier":package["id"],"unit_cargo_target_name":target["name"],
            "unit_cargo_target_kind":"lib","unit_compilation_role_kind":"target",
            "unit_mapped_target_flag":true,"unit_sysroot_context_flag":false,
            "unit_rust_edition_value":"2024","unit_enabled_feature_values":features,
            "unit_target_cfg_values":cfg,"unit_source_root_path":source,
            "unit_package_root_path":std::path::Path::new(package["manifest_path"].as_str().unwrap()).parent(),
            "unit_source_root_paths":[std::path::Path::new(source).parent()],
            "unit_semantic_identity_record":semantic,
            "unit_source_closure_rows":[{
                "file_source_absolute_path":source,"file_parse_context_kind":"items",
                "file_content_digest_full":blake3::hash(&fs::read(source).unwrap()).to_hex().to_string(),
                "file_expansion_provenance_chain":[]
            }],
        });
        unit["unit_compilation_identity_key"] = json!(blake3::hash(unit.to_string().as_bytes()).to_hex().to_string());
        unit
    }).collect::<Vec<_>>();
    let manifest = json!({
        "manifest_format_version_value":1,"manifest_target_context_kind":"compiler_default",
        "manifest_compiler_version_text":env!("ORACLE_RUSTC_VERSION_TEXT"),
        "manifest_scope_identity_record":{"run_generation_digest_value":"fixture","stage_attempt_sequence_value":"one"},
        "manifest_configuration_state_record":{"workspace_source_root_path":root,"workspace_target_cfg_values":[]},
        "manifest_unit_context_rows":units,
    });
    let config = OracleWorkspaceExportConfig {
        workspace_source_root_path: root.into(),
        oracle_macro_cache_manifest: None,
        oracle_target_triple_value: None,
        oracle_enabled_feature_values: vec![],
        oracle_default_features_flag: true,
    };
    (directory, config, manifest)
}

#[test]
fn test_mapped_units_override_cfg() {
    let (_directory, config, manifest) = build_mapped_oracle_fixture();
    let output =
        export_mapped_semantic_paths(&config, &manifest, OracleConfigurationKindValue::Primary)
            .unwrap();
    assert_eq!(output["oracle_configuration_kind_value"], "PRIMARY");
    assert_eq!(output["oracle_mapped_unit_count"], 2);
    assert_eq!(
        output["oracle_manifest_scope_record"],
        manifest["manifest_scope_identity_record"]
    );
    let rows = output["oracle_path_export_rows"].as_array().unwrap();
    assert!(
        rows.iter()
            .any(|row| row["path_site_source_text"] == "Local"
                && row["site_compilation_scope_record"]["unit_cargo_target_name"] == "defs")
    );
    assert!(
        rows.iter()
            .all(|row| row["path_site_source_text"] != "Spare")
    );
    let chosen = rows
        .iter()
        .find(|row| row["path_site_source_text"] == "defs::Chosen")
        .unwrap();
    assert!(
        chosen["path_target_symbol_rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|target| target["target_symbol_kind_value"] == "Struct")
    );
}

#[test]
fn test_secondary_reuses_manifest_units() {
    let (_directory, config, manifest) = build_mapped_oracle_fixture();
    let output =
        export_mapped_semantic_paths(&config, &manifest, OracleConfigurationKindValue::Secondary)
            .unwrap();
    assert_eq!(output["oracle_configuration_kind_value"], "SECONDARY");
    assert_eq!(output["oracle_mapped_unit_count"], 2);
    assert_eq!(
        output["oracle_manifest_scope_record"],
        manifest["manifest_scope_identity_record"]
    );
    assert!(
        output["oracle_path_export_rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["path_site_source_text"] == "Local")
    );
}

#[test]
fn test_empty_mapped_units_export() {
    let (_directory, config, mut manifest) = build_mapped_oracle_fixture();
    manifest["manifest_unit_context_rows"] = json!([]);
    let output =
        export_mapped_semantic_paths(&config, &manifest, OracleConfigurationKindValue::Primary)
            .unwrap();
    assert_eq!(output["oracle_mapped_unit_count"], 0);
    assert_eq!(output["oracle_path_export_rows"], json!([]));
}

#[test]
fn test_mapped_manifest_errors_explicit() {
    let (_directory, config, manifest) = build_mapped_oracle_fixture();
    let mut malformed = manifest.clone();
    malformed["manifest_format_version_value"] = json!(999);
    let error =
        export_mapped_semantic_paths(&config, &malformed, OracleConfigurationKindValue::Primary)
            .unwrap_err();
    assert!(error.to_string().contains("manifest version"), "{error}");
    let mut duplicate = manifest.clone();
    let unit = duplicate["manifest_unit_context_rows"][0].clone();
    duplicate["manifest_unit_context_rows"]
        .as_array_mut()
        .unwrap()
        .push(unit);
    let error =
        export_mapped_semantic_paths(&config, &duplicate, OracleConfigurationKindValue::Primary)
            .unwrap_err();
    assert!(
        error.to_string().contains("duplicate compilation unit"),
        "{error}"
    );
    let mut missing = manifest.clone();
    missing["manifest_unit_context_rows"][0]["unit_source_closure_rows"][0]["file_source_absolute_path"] =
        json!(config.workspace_source_root_path.join("absent.rs"));
    let error =
        export_mapped_semantic_paths(&config, &missing, OracleConfigurationKindValue::Primary)
            .unwrap_err();
    assert!(error.to_string().contains("source closure"), "{error}");
}

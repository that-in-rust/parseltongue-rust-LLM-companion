#![allow(clippy::unwrap_used)]

use crate::{
    OracleConfigurationKindValue, OracleWorkspaceExportConfig, build_oracle_artifact_cache,
    export_mapped_semantic_paths,
};
use serde_json::{Value, json};
use std::{fs, process::Command};

fn build_artifact_test_fixture(
    macros: bool,
) -> (tempfile::TempDir, OracleWorkspaceExportConfig, Value) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    fs::create_dir_all(root.join("src")).unwrap();
    let dependency = if macros {
        "\n[dependencies]\nmacros={path=\"macros\"}\n"
    } else {
        ""
    };
    fs::write(
        root.join("Cargo.toml"),
        format!("[package]\nname=\"artifact\"\nversion=\"0.1.0\"\nedition=\"2024\"\n{dependency}"),
    )
    .unwrap();
    fs::write(
        root.join("src/lib.rs"),
        if macros {
            "#[macros::make] pub struct Original; pub struct Uses(pub Generated);"
        } else {
            "pub struct Plain(pub u8);"
        },
    )
    .unwrap();
    if macros {
        fs::create_dir_all(root.join("macros/src")).unwrap();
        fs::write(root.join("macros/Cargo.toml"),"[package]\nname=\"macros\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[lib]\nproc-macro=true\n").unwrap();
        fs::write(
            root.join("macros/src/lib.rs"),
            r#"
extern crate proc_macro;
#[proc_macro_attribute]
pub fn make(_: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    format!("pub struct Generated(pub u8); {input}").parse().unwrap()
}
"#,
        )
        .unwrap();
    }
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
    let units=metadata["packages"].as_array().unwrap().iter().map(|package| {
        let target=&package["targets"][0];
        let mapped=target["kind"][0]!="proc-macro";
        let role=if mapped {"target"} else {"host"};
        let source=target["src_path"].as_str().unwrap();
        let semantic=json!({
            "unit_cargo_package_identifier":package["id"],"unit_cargo_target_name":target["name"],
            "unit_cargo_target_kind":target["kind"][0],"unit_compilation_role_kind":role,
            "unit_configuration_digest_full":blake3::hash(json!([[],[],"2024",role]).to_string().as_bytes()).to_hex().to_string(),
        });
        let mut unit=json!({
            "unit_cargo_package_identifier":package["id"],"unit_cargo_target_name":target["name"],
            "unit_cargo_target_kind":target["kind"][0],"unit_compilation_role_kind":role,
            "unit_mapped_target_flag":mapped,"unit_sysroot_context_flag":false,
            "unit_rust_edition_value":"2024","unit_enabled_feature_values":[],"unit_target_cfg_values":[],
            "unit_source_root_path":source,"unit_package_root_path":std::path::Path::new(package["manifest_path"].as_str().unwrap()).parent(),
            "unit_semantic_identity_record":semantic,"unit_source_root_paths":[],
            "unit_source_closure_rows":if mapped {json!([{
                "file_source_absolute_path":source,
                "file_content_digest_full":blake3::hash(&fs::read(source).unwrap()).to_hex().to_string(),
                "file_parse_context_kind":"items","file_expansion_provenance_chain":[],
            }])} else {json!([])},
        });
        unit["unit_compilation_identity_key"]=json!(blake3::hash(unit.to_string().as_bytes()).to_hex().to_string());
        unit
    }).collect::<Vec<_>>();
    let manifest = json!({
        "manifest_format_version_value":1,"manifest_target_context_kind":"compiler_default",
        "manifest_mapped_target_policy":{
            "policy_target_kind_values":["lib","bin"],"policy_test_targets_flag":false,
            "policy_dependency_kind_values":["normal"],
            "policy_excluded_target_kinds":["test","bench","example","custom-build","proc-macro"],
            "policy_support_mapping_flag":false,
        },
        "manifest_compiler_version_text":env!("ORACLE_RUSTC_VERSION_TEXT"),
        "manifest_parser_source_record":{"parser_source_commit_hash":env!("ORACLE_SOURCE_COMMIT_HASH")},
        "manifest_scope_identity_record":{"run_generation_digest_value":"artifact-fixture","stage_attempt_sequence_value":"one"},
        "manifest_configuration_state_record":{"workspace_source_root_path":root,"workspace_target_cfg_values":[]},
        "manifest_unit_context_rows":units,
    });
    let request = OracleWorkspaceExportConfig {
        workspace_source_root_path: root.into(),
        oracle_macro_cache_manifest: None,
        oracle_target_triple_value: None,
        oracle_enabled_feature_values: vec![],
        oracle_default_features_flag: true,
    };
    (directory, request, manifest)
}

#[test]
fn test_cached_macro_loads_primary() {
    let (directory, mut request, manifest) = build_artifact_test_fixture(true);
    let cache = build_oracle_artifact_cache(&request, &manifest).unwrap();
    assert_eq!(
        cache["cache_scope_identity_record"],
        manifest["manifest_scope_identity_record"]
    );
    assert_eq!(
        cache["cache_macro_artifact_rows"].as_array().unwrap().len(),
        1
    );
    let path = directory.path().join("oracle-macro-cache-manifest.json");
    fs::write(&path, cache.to_string()).unwrap();
    request.oracle_macro_cache_manifest = Some(path);
    let output =
        export_mapped_semantic_paths(&request, &manifest, OracleConfigurationKindValue::Primary)
            .unwrap();
    assert_eq!(output["oracle_build_scripts_flag"], false);
    assert!(
        output["oracle_proc_macro_rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["macro_loading_status_value"] == "loaded"
                && row["macro_loading_origin_kind"] == "cached")
    );
    assert!(
        output["oracle_path_export_rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["path_site_source_text"] == "Generated"
                && !row["path_target_symbol_rows"]
                    .as_array()
                    .unwrap()
                    .is_empty())
    );
}

#[test]
fn test_empty_artifact_cache_preserved() {
    let (directory, mut request, manifest) = build_artifact_test_fixture(false);
    let cache = build_oracle_artifact_cache(&request, &manifest).unwrap();
    assert_eq!(cache["cache_macro_artifact_rows"], json!([]));
    let path = directory.path().join("oracle-macro-cache-manifest.json");
    fs::write(&path, cache.to_string()).unwrap();
    request.oracle_macro_cache_manifest = Some(path);
    let output =
        export_mapped_semantic_paths(&request, &manifest, OracleConfigurationKindValue::Primary)
            .unwrap();
    assert_eq!(
        output["oracle_manifest_scope_record"],
        manifest["manifest_scope_identity_record"]
    );
    assert!(
        output["oracle_proc_macro_rows"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn test_cached_artifact_errors_explicit() {
    let (directory, mut request, manifest) = build_artifact_test_fixture(true);
    let cache = build_oracle_artifact_cache(&request, &manifest).unwrap();
    let path = directory.path().join("oracle-macro-cache-manifest.json");
    request.oracle_macro_cache_manifest = Some(path.clone());
    assert!(
        export_mapped_semantic_paths(&request, &manifest, OracleConfigurationKindValue::Primary)
            .is_err()
    );
    for key in [
        "cache_scope_identity_record",
        "cache_compiler_version_text",
        "cache_source_revision_hash",
        "cache_macro_artifact_rows",
    ] {
        let mut invalid = cache.clone();
        invalid[key] = json!(null);
        fs::write(&path, invalid.to_string()).unwrap();
        assert!(
            export_mapped_semantic_paths(
                &request,
                &manifest,
                OracleConfigurationKindValue::Primary
            )
            .is_err()
        );
    }
    let mut invalid = cache.clone();
    invalid["cache_macro_artifact_rows"][0]["artifact_binary_digest_full"] = json!("wrong");
    fs::write(&path, invalid.to_string()).unwrap();
    assert!(
        export_mapped_semantic_paths(&request, &manifest, OracleConfigurationKindValue::Primary)
            .is_err()
    );
    let mut invalid = cache;
    invalid["cache_macro_artifact_rows"] = json!([]);
    fs::write(&path, invalid.to_string()).unwrap();
    assert!(
        export_mapped_semantic_paths(&request, &manifest, OracleConfigurationKindValue::Primary)
            .is_err()
    );
}

#[test]
fn test_artifact_compile_failure_explicit() {
    let (directory, request, mut manifest) = build_artifact_test_fixture(false);
    let source = "compile_error!(\"artifact preparation must fail\");";
    fs::write(directory.path().join("src/lib.rs"), source).unwrap();
    manifest["manifest_unit_context_rows"][0]["unit_source_closure_rows"][0]["file_content_digest_full"] =
        json!(blake3::hash(source.as_bytes()).to_hex().to_string());
    let error = build_oracle_artifact_cache(&request, &manifest).unwrap_err();
    assert!(
        error
            .error_context_detail_text
            .contains("offline artifact preparation failed"),
        "{error:?}"
    );
}

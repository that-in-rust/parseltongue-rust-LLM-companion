#![allow(clippy::unwrap_used)]

use crate::{
    OracleConfigurationKindValue, OracleWorkspaceExportConfig, build_oracle_artifact_cache,
    export_mapped_semantic_paths, oracle_artifact_contract_tests::build_artifact_test_fixture,
};
use serde_json::{Value, json};
use std::fs;

fn build_script_test_fixture() -> (tempfile::TempDir, OracleWorkspaceExportConfig, Value, Value) {
    let (directory, mut request, mut manifest) = build_artifact_test_fixture(false);
    fs::write(
        directory.path().join("build.rs"),
        r#"
fn main() {
    println!("cargo:rustc-check-cfg=cfg(generated)");
    println!("cargo:rustc-cfg=generated");
    println!("cargo:rustc-env=VISIBLE_VALUE=build-value");
    let output=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    std::fs::write(output.join("generated.rs"),"pub struct BuildGenerated(pub u8);").unwrap();
}
"#,
    )
    .unwrap();
    let source = r#"
#[cfg(generated)] pub struct Generated;
pub struct Uses(pub Generated);
#[cfg(generated)] include!(concat!(env!("OUT_DIR"),"/generated.rs"));
#[cfg(generated)] pub struct UsesOutput(pub BuildGenerated);
"#;
    fs::write(directory.path().join("src/lib.rs"), source).unwrap();
    manifest["manifest_unit_context_rows"][0]["unit_source_closure_rows"][0]["file_content_digest_full"] =
        json!(blake3::hash(source.as_bytes()).to_hex().to_string());
    let cache = build_oracle_artifact_cache(&request, &manifest).unwrap();
    let path = directory.path().join("oracle-macro-cache-manifest.json");
    fs::write(&path, cache.to_string()).unwrap();
    request.oracle_macro_cache_manifest = Some(path);
    (directory, request, manifest, cache)
}

#[test]
fn test_secondary_preserves_script_deltas() {
    let (_directory, request, manifest, cache) = build_script_test_fixture();
    let primary =
        export_mapped_semantic_paths(&request, &manifest, OracleConfigurationKindValue::Primary)
            .unwrap();
    assert_eq!(primary["oracle_build_script_deltas"], json!([]));
    let unresolved = primary["oracle_path_export_rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["path_site_source_text"] == "Generated")
        .unwrap();
    assert!(
        unresolved["path_target_symbol_rows"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let secondary =
        export_mapped_semantic_paths(&request, &manifest, OracleConfigurationKindValue::Secondary)
            .unwrap();
    assert_eq!(
        secondary["oracle_manifest_scope_record"],
        primary["oracle_manifest_scope_record"]
    );
    assert_eq!(
        secondary["oracle_mapped_unit_count"],
        primary["oracle_mapped_unit_count"]
    );
    assert_eq!(secondary["oracle_configuration_kind_value"], "SECONDARY");
    let resolved = secondary["oracle_path_export_rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["path_site_source_text"] == "Generated")
        .unwrap();
    assert!(
        !resolved["path_target_symbol_rows"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let deltas = secondary["oracle_build_script_deltas"].as_array().unwrap();
    assert_eq!(deltas.len(), 1);
    assert_eq!(deltas[0]["delta_build_script_kind"], "KL1");
    assert_eq!(deltas[0]["delta_script_cfg_values"], json!(["generated"]));
    assert_eq!(
        deltas[0]["delta_script_env_digest"],
        cache["cache_build_script_rows"][0]["script_emitted_env_digest"]
    );
    let generated = deltas[0]["delta_generated_source_rows"].as_array().unwrap();
    assert!(generated.iter().any(|row| {
        row["file_content_digest_full"]
            == blake3::hash(b"pub struct BuildGenerated(pub u8);")
                .to_hex()
                .to_string()
    }));
    assert!(!secondary.to_string().contains("build-value"));
}

#[test]
fn test_secondary_empty_script_deltas() {
    let (_directory, request, manifest) = build_artifact_test_fixture(false);
    let output =
        export_mapped_semantic_paths(&request, &manifest, OracleConfigurationKindValue::Secondary)
            .unwrap();
    assert_eq!(output["oracle_build_script_deltas"], json!([]));
}

#[test]
fn test_secondary_stale_script_errors() {
    let (_directory, mut request, manifest, cache) = build_script_test_fixture();
    let path = request.oracle_macro_cache_manifest.take().unwrap();
    let error =
        export_mapped_semantic_paths(&request, &manifest, OracleConfigurationKindValue::Secondary)
            .unwrap_err();
    assert!(
        error.error_context_detail_text.contains("recorded"),
        "{error:?}"
    );
    request.oracle_macro_cache_manifest = Some(path.clone());
    for (key, value) in [
        ("script_emitted_cfg_values", json!(["not_emitted"])),
        ("script_emitted_env_digest", json!("stale")),
        (
            "script_generated_output_path",
            json!("/missing/generated/output"),
        ),
    ] {
        let mut stale = cache.clone();
        stale["cache_build_script_rows"][0][key] = value;
        fs::write(&path, stale.to_string()).unwrap();
        let error = export_mapped_semantic_paths(
            &request,
            &manifest,
            OracleConfigurationKindValue::Secondary,
        )
        .unwrap_err();
        assert!(
            error
                .error_context_detail_text
                .contains("SECONDARY recorded"),
            "{error:?}"
        );
    }
}

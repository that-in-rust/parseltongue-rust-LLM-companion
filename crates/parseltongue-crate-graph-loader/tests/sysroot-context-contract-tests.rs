#![allow(clippy::unwrap_used)]
use parseltongue_core_row_types::*;
use parseltongue_crate_graph_loader::extend_sysroot_crate_graph;
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

fn build_sysroot_fixture_rows(root: &Path) -> Vec<StageTableRowRecord> {
    vec![
        StageTableRowRecord{stage_owned_table_name:"workspace-config-state-records".into(),stage_row_record_key:"workspace".into(),stage_row_payload_text:json!({"workspace_source_root_path":root,"workspace_sysroot_directory_path":root.join("sysroot"),"workspace_target_cfg_values":["unix","test"]}).to_string()},
        StageTableRowRecord{stage_owned_table_name:"crate-dependency-graph-table".into(),stage_row_record_key:"client".into(),stage_row_payload_text:json!({
            "unit_cargo_package_identifier":"fixture-client","unit_cargo_target_name":"client","unit_cargo_target_kind":"lib",
            "unit_compilation_role_kind":"target","unit_mapped_target_flag":true,"unit_compilation_identity_key":"client",
            "unit_rust_edition_value":"2021","unit_target_cfg_values":["unix","feature=\"client_only\""],"unit_normal_dependency_rows":[],
            "unit_source_root_path":root.join("src/lib.rs"),"unit_package_root_path":root,"unit_metadata_environment_values":{}
        }).to_string()},
    ]
}

fn build_sysroot_fixture_sources(root: &Path) {
    for name in ["core", "alloc", "std"] {
        let package = root.join("sysroot/lib/rustlib/src/rust/library").join(name);
        std::fs::create_dir_all(package.join("src")).unwrap();
        std::fs::write(
            package.join("Cargo.toml"),
            format!("[package]\nname=\"{name}\"\nversion=\"0.0.0\"\nedition=\"2024\"\n"),
        )
        .unwrap();
        std::fs::write(package.join("src/lib.rs"), "pub struct System;").unwrap();
    }
}

#[test]
fn test_sysroot_target_cfg_contexts() {
    let directory = tempfile::tempdir().unwrap();
    build_sysroot_fixture_sources(directory.path());
    let contexts = BTreeMap::from([(
        "client".into(),
        UnitPreludeConfigurationInput {
            unit_no_std_flag: false,
            unit_no_core_flag: false,
            unit_implicit_prelude_flag: true,
        },
    )]);
    let rows = extend_sysroot_crate_graph(&build_sysroot_fixture_rows(directory.path()), &contexts)
        .unwrap();
    let units = rows
        .iter()
        .filter(|row| row.stage_owned_table_name == "crate-dependency-graph-table")
        .map(|row| serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap())
        .collect::<Vec<_>>();
    let system = units
        .iter()
        .filter(|unit| unit["unit_sysroot_context_flag"] == true)
        .collect::<Vec<_>>();
    assert_eq!(system.len(), 3);
    for unit in system {
        assert_eq!(unit["unit_mapped_target_flag"], false);
        assert_eq!(unit["unit_compilation_role_kind"], "target");
        assert_eq!(unit["unit_rust_edition_value"], "2024");
        assert_eq!(unit["unit_target_cfg_values"], json!(["unix"]));
        assert_eq!(unit["unit_enabled_feature_values"], json!([]));
        assert!(
            unit["unit_sysroot_manifest_digest"]
                .as_str()
                .is_some_and(|value| value.len() == 64)
        );
    }
    let client = units
        .iter()
        .find(|unit| unit["unit_compilation_identity_key"] == "client")
        .unwrap();
    assert_eq!(client["unit_rust_edition_value"], "2021");
    let aliases = client["unit_normal_dependency_rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| row["dependency_alias_name_value"].as_str())
        .collect::<Vec<_>>();
    assert!(aliases.contains(&"std") && aliases.contains(&"core"));
}

#[test]
fn test_sysroot_missing_source_failure() {
    let directory = tempfile::tempdir().unwrap();
    let contexts = BTreeMap::from([(
        "client".into(),
        UnitPreludeConfigurationInput {
            unit_no_std_flag: true,
            unit_no_core_flag: false,
            unit_implicit_prelude_flag: false,
        },
    )]);
    let error =
        extend_sysroot_crate_graph(&build_sysroot_fixture_rows(directory.path()), &contexts)
            .unwrap_err();
    assert!(error.to_string().contains("matching sysroot sources"));
}

#[test]
fn test_sysroot_empty_context_rows() {
    assert!(
        extend_sysroot_crate_graph(&[], &BTreeMap::new())
            .unwrap()
            .is_empty()
    );
}

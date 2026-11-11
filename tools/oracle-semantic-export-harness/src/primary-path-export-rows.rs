use std::{fmt, path::PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleWorkspaceExportConfig {
    pub workspace_source_root_path: PathBuf,
    pub oracle_macro_cache_manifest: Option<PathBuf>,
    pub oracle_target_triple_value: Option<String>,
    pub oracle_enabled_feature_values: Vec<String>,
    pub oracle_default_features_flag: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleSemanticExportError {
    pub error_context_detail_text: String,
}

impl fmt::Display for OracleSemanticExportError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.write_str(&self.error_context_detail_text)
    }
}

impl std::error::Error for OracleSemanticExportError {}

pub fn export_primary_semantic_paths(
    _oracle_workspace_export_config: &OracleWorkspaceExportConfig,
) -> Result<Vec<serde_json::Value>, OracleSemanticExportError> {
    todo!()
}

#[cfg(test)]
mod oracle_path_contract_tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use std::{collections::BTreeSet, fs, path::Path};

    fn create_oracle_workspace_fixture(root: &Path, files: &[(&str, &str)]) {
        for (path, text) in files {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
    }

    fn build_oracle_fixture_config(root: &Path) -> OracleWorkspaceExportConfig {
        OracleWorkspaceExportConfig {
            workspace_source_root_path: root.to_owned(),
            oracle_macro_cache_manifest: None,
            oracle_target_triple_value: None,
            oracle_enabled_feature_values: Vec::new(),
            oracle_default_features_flag: true,
        }
    }

    #[test]
    fn test_nameres_matches_oracle_paths() {
        let directory = tempfile::tempdir().unwrap();
        let source = "pub mod defs { pub struct Present; }\nuse crate::defs::Present as Alias;\npub struct Wrap(pub Alias);\n#[cfg(test)] mod hidden;\n";
        create_oracle_workspace_fixture(
            directory.path(),
            &[
                (
                    "Cargo.toml",
                    "[package]\nname=\"paths\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
                ),
                ("src/lib.rs", source),
                ("src/hidden.rs", "pub struct Forbidden(pub Missing);"),
                ("tests/integration.rs", "pub struct Forbidden(pub Missing);"),
                ("examples/ignored.rs", "pub struct Forbidden(pub Missing);"),
            ],
        );
        let rows =
            export_primary_semantic_paths(&build_oracle_fixture_config(directory.path())).unwrap();
        let alias = rows
            .iter()
            .find(|row| row["path_site_source_text"] == "Alias")
            .unwrap();
        let start = source.rfind("Alias").unwrap();
        assert_eq!(alias["site_source_range_start"], start);
        assert_eq!(alias["site_source_range_end"], start + "Alias".len());
        let targets = alias["path_target_symbol_rows"].as_array().unwrap();
        assert!(
            targets
                .iter()
                .any(|target| target["target_source_file_path"]
                    == directory.path().join("src/lib.rs").to_str().unwrap()
                    && target["target_source_range_start"]
                        == source.find("pub struct Present;").unwrap()
                    && target["target_symbol_kind_value"] == "Struct")
        );
        assert!(
            rows.iter()
                .any(|row| row["path_site_source_text"] == "crate::defs::Present")
        );
        assert!(
            rows.iter()
                .all(|row| row["path_site_source_text"] != "Missing")
        );
    }

    #[test]
    fn test_shared_modules_preserve_context() {
        let directory = tempfile::tempdir().unwrap();
        create_oracle_workspace_fixture(
            directory.path(),
            &[
                (
                    "Cargo.toml",
                    "[workspace]\nmembers=[\"front\",\"back\",\"left\",\"right\"]\nresolver=\"3\"\n",
                ),
                (
                    "front/Cargo.toml",
                    "[package]\nname=\"front\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[dependencies]\nchosen={package=\"left\",path=\"../left\"}\n",
                ),
                (
                    "back/Cargo.toml",
                    "[package]\nname=\"back\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[dependencies]\nchosen={package=\"right\",path=\"../right\"}\n",
                ),
                (
                    "left/Cargo.toml",
                    "[package]\nname=\"left\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
                ),
                (
                    "right/Cargo.toml",
                    "[package]\nname=\"right\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
                ),
                (
                    "front/src/lib.rs",
                    "#[path=\"../../shared.rs\"] pub mod shared;",
                ),
                (
                    "back/src/lib.rs",
                    "#[path=\"../../shared.rs\"] pub mod shared;",
                ),
                ("left/src/lib.rs", "pub struct Choice;"),
                ("right/src/lib.rs", "pub struct Choice;"),
                ("shared.rs", "pub struct Wrap(pub chosen::Choice);"),
            ],
        );
        let rows =
            export_primary_semantic_paths(&build_oracle_fixture_config(directory.path())).unwrap();
        for (owner, dependency) in [("front", "left"), ("back", "right")] {
            let owner_path = directory.path().join(owner).join("src/lib.rs");
            let row = rows
                .iter()
                .find(|row| {
                    row["unit_source_root_path"] == owner_path.to_str().unwrap()
                        && row["path_site_source_text"] == "chosen::Choice"
                })
                .unwrap();
            let targets: BTreeSet<_> = row["path_target_symbol_rows"]
                .as_array()
                .unwrap()
                .iter()
                .map(|target| target["target_source_file_path"].as_str().unwrap())
                .collect();
            assert_eq!(
                targets,
                BTreeSet::from([directory
                    .path()
                    .join(dependency)
                    .join("src/lib.rs")
                    .to_str()
                    .unwrap()])
            );
        }
    }

    #[test]
    fn test_empty_oracle_paths_succeed() {
        let directory = tempfile::tempdir().unwrap();
        create_oracle_workspace_fixture(
            directory.path(),
            &[
                (
                    "Cargo.toml",
                    "[package]\nname=\"empty\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
                ),
                ("src/lib.rs", ""),
            ],
        );
        assert!(
            export_primary_semantic_paths(&build_oracle_fixture_config(directory.path()))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_missing_oracle_workspace_errors() {
        let directory = tempfile::tempdir().unwrap();
        let error = export_primary_semantic_paths(&build_oracle_fixture_config(directory.path()))
            .unwrap_err();
        assert!(!error.error_context_detail_text.is_empty());
    }
}

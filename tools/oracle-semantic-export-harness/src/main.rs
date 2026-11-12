fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oracle_cli_argument_values = std::env::args().skip(1).collect::<Vec<_>>();
    if let [
        root_flag,
        root,
        manifest_flag,
        manifest,
        output_flag,
        output,
        kind_flag,
        kind,
        remaining @ ..,
    ] = oracle_cli_argument_values.as_slice()
        && root_flag == "--workspace-source-root-path"
        && manifest_flag == "--mapped-unit-manifest-path"
        && output_flag == "--oracle-path-export-output"
        && kind_flag == "--oracle-configuration-kind"
    {
        let configuration = match kind.as_str() {
            "PRIMARY" => oracle_semantic_export_harness::OracleConfigurationKindValue::Primary,
            "SECONDARY" => oracle_semantic_export_harness::OracleConfigurationKindValue::Secondary,
            _ => return Err("expected PRIMARY or SECONDARY".into()),
        };
        let cache = match remaining {
            [] => None,
            [flag, path] if flag == "--oracle-macro-cache-manifest" => Some(path.into()),
            _ => return Err("invalid mapped-export arguments".into()),
        };
        let config = oracle_semantic_export_harness::OracleWorkspaceExportConfig {
            workspace_source_root_path: root.into(),
            oracle_macro_cache_manifest: cache,
            oracle_target_triple_value: None,
            oracle_enabled_feature_values: Vec::new(),
            oracle_default_features_flag: true,
        };
        let manifest = serde_json::from_slice(&std::fs::read(manifest)?)?;
        let rows = oracle_semantic_export_harness::export_mapped_semantic_paths(
            &config,
            &manifest,
            configuration,
        )?;
        std::fs::write(output, serde_json::to_vec_pretty(&rows)?)?;
        return Ok(());
    }
    if let [root_flag, root, output_flag, output] = oracle_cli_argument_values.as_slice()
        && root_flag == "--workspace-source-root-path"
        && output_flag == "--oracle-path-export-output"
    {
        let config = oracle_semantic_export_harness::OracleWorkspaceExportConfig {
            workspace_source_root_path: root.into(),
            oracle_macro_cache_manifest: None,
            oracle_target_triple_value: None,
            oracle_enabled_feature_values: Vec::new(),
            oracle_default_features_flag: true,
        };
        let rows = oracle_semantic_export_harness::export_primary_semantic_paths(&config)?;
        std::fs::write(output, serde_json::to_vec_pretty(&rows)?)?;
        return Ok(());
    }
    if oracle_cli_argument_values != ["--oracle-build-manifest-output"] {
        return Err(
            "expected --oracle-build-manifest-output or --workspace-source-root-path PATH --oracle-path-export-output PATH"
                .into(),
        );
    }
    let mut oracle_source_build_manifest =
        oracle_semantic_export_harness::build_oracle_source_manifest();
    let oracle_executable_content_bytes = std::fs::read(std::env::current_exe()?)?;
    oracle_source_build_manifest["oracle_executable_digest_full"] =
        blake3::hash(&oracle_executable_content_bytes)
            .to_hex()
            .to_string()
            .into();
    println!(
        "{}",
        serde_json::to_string_pretty(&oracle_source_build_manifest)?
    );
    Ok(())
}

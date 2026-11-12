//! Offline semantic oracle; never linked into the production mapper.

#[path = "mapped-oracle-export-facade.rs"]
mod mapped_oracle_export_facade;
pub use mapped_oracle_export_facade::{OracleConfigurationKindValue, export_mapped_semantic_paths};

#[path = "primary-path-export-rows.rs"]
mod primary_path_export_rows;
#[path = "primary-workspace-loader-adapter.rs"]
mod primary_workspace_loader_adapter;
pub use primary_path_export_rows::{
    OracleSemanticExportError, OracleWorkspaceExportConfig, export_primary_semantic_paths,
};

pub fn build_oracle_source_manifest() -> serde_json::Value {
    let oracle_analysis_host_value = ide::AnalysisHost::default();
    let _ = oracle_analysis_host_value.analysis();
    serde_json::json!({
        "oracle_source_commit_hash": env!("ORACLE_SOURCE_COMMIT_HASH"),
        "oracle_toolchain_version_text": env!("ORACLE_RUSTC_VERSION_TEXT"),
        "oracle_export_support_kind": "provenance_only",
        "oracle_hir_crate_bytes": std::mem::size_of::<hir::Crate>(),
    })
}

#[cfg(test)]
mod oracle_manifest_contract_tests {
    #[test]
    fn test_oracle_build_manifest_pinned() {
        let oracle_source_build_manifest = super::build_oracle_source_manifest();
        assert_eq!(
            oracle_source_build_manifest["oracle_source_commit_hash"],
            "bf6b2099d6351c37039b01447a9bb64f9a54a586"
        );
        assert_eq!(
            oracle_source_build_manifest["oracle_export_support_kind"],
            "provenance_only"
        );
        assert!(
            oracle_source_build_manifest["oracle_toolchain_version_text"]
                .as_str()
                .is_some_and(|version| version.starts_with("rustc "))
        );
    }
}

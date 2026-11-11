use std::{error::Error, path::Path, process::Command};
use parseltongue_turso_store_adapter::TursoWorkspaceStoreAdapter;

#[test]
fn test_no_analysis_crates_linked() -> Result<(), Box<dyn Error>> {
    let workspace_root_directory_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let workspace_metadata_command_output = Command::new(env!("CARGO"))
        .current_dir(workspace_root_directory_path)
        .args(["metadata", "--offline", "--locked", "--format-version", "1"]).output()?;
    assert!(workspace_metadata_command_output.status.success(), "{}", String::from_utf8_lossy(&workspace_metadata_command_output.stderr));
    let workspace_metadata_json_value: serde_json::Value = serde_json::from_slice(&workspace_metadata_command_output.stdout)?;
    let workspace_metadata_package_rows = workspace_metadata_json_value["packages"].as_array().ok_or("missing metadata packages")?;
    let core_row_package_record = workspace_metadata_package_rows.iter().find(|package| package["name"] == "parseltongue-core-row-types").ok_or("missing L1")?;
    assert_eq!(core_row_package_record["dependencies"].as_array().ok_or("missing dependencies")?.len(), 0);
    for workspace_dependency_package_row in workspace_metadata_package_rows {
        let workspace_dependency_package_name = workspace_dependency_package_row["name"].as_str().ok_or("missing package name")?.replace('-', "_");
        assert!(!workspace_dependency_package_name.starts_with("salsa")
            && !workspace_dependency_package_name.starts_with("ra_ap_hir")
            && !workspace_dependency_package_name.starts_with("ra_ap_ide")
            && workspace_dependency_package_name != "ra_ap_base_db", "forbidden production dependency: {workspace_dependency_package_name}");
    }
    assert!(workspace_metadata_package_rows.iter().any(|package| package["name"] == "turso" && package["version"] == "0.9.0-pre.5"));
    assert!(!workspace_metadata_package_rows.iter().any(|package| package["name"] == "oracle-semantic-export-harness"));
    Ok(())
}

#[test]
fn test_turso_concurrent_api_available() -> Result<(), Box<dyn Error>> {
    let workspace_database_store_adapter = TursoWorkspaceStoreAdapter::open_turso_workspace_store(":memory:")?;
    workspace_database_store_adapter.check_turso_concurrent_api()?;
    assert!(TursoWorkspaceStoreAdapter::open_turso_workspace_store("").is_err());
    assert!(TursoWorkspaceStoreAdapter::open_turso_workspace_store("invalid\0path").is_err());
    Ok(())
}

#[test]
fn test_nested_runtime_returns_error() -> Result<(), Box<dyn Error>> {
    let caller_async_runtime_handle = tokio::runtime::Builder::new_current_thread().build()?;
    caller_async_runtime_handle.block_on(async {
        assert!(TursoWorkspaceStoreAdapter::open_turso_workspace_store(":memory:").is_err());
    });
    Ok(())
}

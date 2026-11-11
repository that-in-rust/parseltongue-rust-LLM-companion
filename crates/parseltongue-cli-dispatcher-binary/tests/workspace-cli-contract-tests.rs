use std::{fs, path::Path, process::Command};
use serde_json::Value;

fn create_workspace_source_fixture(root: &Path) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("Cargo.toml"), "[package]\nname=\"fixture\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[features]\ndefault=[\"enabled\"]\nenabled=[]\n").unwrap();
    fs::write(root.join("Cargo.lock"), "version = 4\n[[package]]\nname = \"fixture\"\nversion = \"0.1.0\"\n").unwrap();
    fs::write(root.join("src/lib.rs"), "mod child; #[cfg(test)] mod absent; pub struct A { pub field: u8 } pub fn f(x: u8) -> u8 { child::g(x) }").unwrap();
    fs::write(root.join("src/child.rs"), "pub fn g(x: u8) -> u8 { x }").unwrap();
    fs::write(root.join("unrelated.rs"), [255, 0]).unwrap();
    fs::write(root.join("build.rs"), "compile_error!(\"never execute me\");").unwrap();
    let lock = Command::new(env!("CARGO")).current_dir(root).args(["generate-lockfile", "--offline"]).output().unwrap();
    assert!(lock.status.success(), "{}", String::from_utf8_lossy(&lock.stderr));
}
fn run_workspace_mapping_fixture(root: &Path, database: &Path, workers: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_parseltongue")).args(["map-workspace-full-run",
        "--workspace-root-path-value", root.to_str().unwrap(), "--workspace-store-file-path", database.to_str().unwrap(),
        "--worker-thread-count-value", workers]).output().unwrap()
}
fn read_workspace_banner_fixture(database: &Path) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_parseltongue")).args(["status-coverage-banner-show", "--json",
        "--workspace-store-file-path", database.to_str().unwrap()]).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    serde_json::from_slice(&output.stdout).unwrap()
}
#[test]
fn test_stage_rerun_is_identical() {
    let directory=tempfile::tempdir().unwrap();let root=directory.path().join("workspace");
    create_workspace_source_fixture(&root);let database=directory.path().join("store.db");
    let output=run_workspace_mapping_fixture(&root,&database,"1");
    assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
    let first=read_workspace_banner_fixture(&database);
    assert_eq!(first["source_file_record_count"],4);
    assert!(first["syntax_span_record_count"].as_u64().unwrap()>0);
    assert!(first["signature_field_record_count"].as_u64().unwrap()>0);
    assert!(first["body_site_record_count"].as_u64().unwrap()>0);
    assert_eq!(first["source_tiling_identity_holds"],true);
    assert_eq!(first["stage_implementation_status_values"]["name-resolution"],"not yet implemented");
    let output=run_workspace_mapping_fixture(&root,&database,"4");
    assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
    let second=read_workspace_banner_fixture(&database);
    assert_eq!(first["run_generation_digest_value"],second["run_generation_digest_value"]);
    assert_eq!(first["syntax_span_record_count"],second["syntax_span_record_count"]);
    fs::write(root.join("src/child.rs"),"fn broken(").unwrap();
    assert!(!run_workspace_mapping_fixture(&root,&database,"4").status.success());
    let retained=read_workspace_banner_fixture(&database);
    assert_eq!(second["run_generation_digest_value"],retained["run_generation_digest_value"]);
}

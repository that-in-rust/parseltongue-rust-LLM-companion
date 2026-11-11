#![allow(clippy::unwrap_used)]

use parseltongue_core_row_types::StoreBackendFailureError;
use parseltongue_crate_graph_loader::*;
use serde_json::json;
use std::{cell::RefCell, path::Path};

struct FixtureWorkspaceCommandRunner {
    command_argument_capture_rows: RefCell<Vec<Vec<String>>>,
    metadata_failure_message_text: Option<String>,
    metadata_mutates_lock_bytes: bool,
}

impl WorkspaceCommandRunPort for FixtureWorkspaceCommandRunner {
    fn run_workspace_command_output(
        &self,
        root: &Path,
        args: &[&str],
    ) -> Result<Vec<u8>, StoreBackendFailureError> {
        self.command_argument_capture_rows
            .borrow_mut()
            .push(args.iter().map(|s| s.to_string()).collect());
        match args {
            ["cargo", "metadata", "--offline"] => {
                if self.metadata_mutates_lock_bytes {
                    std::fs::write(root.join("Cargo.lock"), b"changed").unwrap();
                }
                if let Some(message) = &self.metadata_failure_message_text {
                    return Err(StoreBackendFailureError {
                        error_context_detail_text: message.clone(),
                    });
                }
                let id = format!("path+file://{}#sample@0.1.0", root.display());
                Ok(serde_json::to_vec(&json!({
                    "packages": [{"name":"sample","version":"0.1.0","id":id,"source":null,
                        "dependencies":[],"targets":[
                            {"kind":["lib"],"crate_types":["lib"],"name":"sample","src_path":root.join("src/lib.rs"),"edition":"2024","doc":true,"doctest":true,"test":true},
                            {"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":root.join("build.rs"),"edition":"2024","doc":false,"doctest":false,"test":false}
                        ],"features":{"default":["one"],"one":[]},"manifest_path":root.join("Cargo.toml"),
                        "metadata":{},"authors":[],"categories":[],"keywords":[],"edition":"2024"}],
                    "workspace_members":[id],"workspace_default_members":[id],
                    "resolve":{"nodes":[{"id":id,"dependencies":[],"deps":[],"features":["default","one"]}],"root":id},
                    "target_directory":root.join("target"),"workspace_root":root,"version":1,"metadata":{}
                })).unwrap())
            }
            ["rustc", "--print", "cfg"] => Ok(b"unix\ntarget_os=\"linux\"\n".to_vec()),
            ["rustc", "--print", "sysroot"] => Ok(b"/fixture/sysroot\n".to_vec()),
            _ => panic!("unexpected command {args:?}"),
        }
    }
}

fn make_fixture_command_runner() -> FixtureWorkspaceCommandRunner {
    FixtureWorkspaceCommandRunner {
        command_argument_capture_rows: RefCell::new(vec![]),
        metadata_failure_message_text: None,
        metadata_mutates_lock_bytes: false,
    }
}

struct UnmappedWorkspaceCommandRunner;
impl WorkspaceCommandRunPort for UnmappedWorkspaceCommandRunner {
    fn run_workspace_command_output(
        &self,
        root: &Path,
        args: &[&str],
    ) -> Result<Vec<u8>, StoreBackendFailureError> {
        let bytes = make_fixture_command_runner().run_workspace_command_output(root, args)?;
        if args != ["cargo", "metadata", "--offline"] {
            return Ok(bytes);
        }
        let mut document: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let mut helper = document["packages"][0].clone();
        let helper_id = "path+file:///helper#helper@0.1.0";
        helper["id"] = json!(helper_id);
        helper["name"] = json!("helper");
        helper["targets"] = json!([helper["targets"][0].clone()]);
        let mut test_target = document["packages"][0]["targets"][0].clone();
        test_target["kind"] = json!(["test"]);
        test_target["crate_types"] = json!(["bin"]);
        test_target["src_path"] = json!(root.join("tests/check.rs"));
        document["packages"][0]["targets"] = json!([test_target]);
        document["packages"][0]["dependencies"] = json!([{"name":"helper","req":"*","source":null,"rename":null,"kind":null,"target":null,"features":[],"uses_default_features":true,"optional":false}]);
        document["packages"].as_array_mut().unwrap().push(helper);
        document["resolve"]["nodes"][0]["deps"] =
            json!([{"name":"helper","pkg":helper_id,"dep_kinds":[{"kind":null,"target":null}]}]);
        document["resolve"]["nodes"][0]["dependencies"] = json!([helper_id]);
        let mut helper_node = document["resolve"]["nodes"][0].clone();
        helper_node["id"] = json!(helper_id);
        helper_node["deps"] = json!([]);
        helper_node["dependencies"] = json!([]);
        document["resolve"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(helper_node);
        Ok(serde_json::to_vec(&document).unwrap())
    }
}

#[test]
fn test_testonly_workspace_stays_unmapped() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("Cargo.lock"), b"lock").unwrap();
    let rows =
        load_workspace_crate_graph(directory.path(), &UnmappedWorkspaceCommandRunner).unwrap();
    assert!(
        !rows
            .iter()
            .any(|r| r.stage_owned_table_name == "crate-dependency-graph-table")
    );
}

#[test]
fn test_source_targets_stay_excluded() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("source.rs");
    std::fs::write(&source, b"fn source() {}").unwrap();
    assert_eq!(
        read_safe_source_bytes(&source).unwrap().1,
        b"fn source() {}"
    );
    std::fs::write(&source, b"").unwrap();
    assert!(read_safe_source_bytes(&source).unwrap().1.is_empty());
    let target = directory.path().join("target/nested");
    std::fs::create_dir_all(&target).unwrap();
    let generated = target.join("generated.rs");
    std::fs::write(&generated, b"fn generated() {}").unwrap();
    assert!(
        read_safe_source_bytes(&generated)
            .unwrap_err()
            .to_string()
            .contains("forbidden target source")
    );
    #[cfg(unix)]
    {
        let linked = directory.path().join("linked.rs");
        std::os::unix::fs::symlink(&generated, &linked).unwrap();
        assert!(
            read_safe_source_bytes(&linked)
                .unwrap_err()
                .to_string()
                .contains("forbidden target source")
        );
    }
}

#[test]
fn test_crate_graph_offline_only() {
    let directory = tempfile::tempdir().unwrap();
    let runner = make_fixture_command_runner();
    assert!(load_workspace_crate_graph(directory.path(), &runner).is_err());
    assert!(runner.command_argument_capture_rows.borrow().is_empty());
    std::fs::write(directory.path().join("Cargo.lock"), b"lock").unwrap();
    let rows = load_workspace_crate_graph(directory.path(), &runner).unwrap();
    let units: Vec<serde_json::Value> = rows
        .iter()
        .filter(|r| r.stage_owned_table_name == "crate-dependency-graph-table")
        .map(|r| serde_json::from_str(&r.stage_row_payload_text).unwrap())
        .collect();
    assert!(units.iter().any(|u| u["unit_cargo_target_kind"] == "lib" && u["unit_rust_edition_value"] == "2024"));
    assert!(units.iter().any(
        |u| u["unit_compilation_role_kind"] == "host" && u["unit_mapped_target_flag"] == false
    ));
    assert!(
        runner
            .command_argument_capture_rows
            .borrow()
            .iter()
            .all(|a| matches!(
                a.iter().map(String::as_str).collect::<Vec<_>>().as_slice(),
                ["cargo", "metadata", "--offline"]
                    | ["rustc", "--print", "cfg"]
                    | ["rustc", "--print", "sysroot"]
            ))
    );
}

#[test]
fn test_lockfile_guard_preserves_bytes() {
    let directory = tempfile::tempdir().unwrap();
    let original = b"original lock bytes\n";
    std::fs::write(directory.path().join("Cargo.lock"), original).unwrap();
    let mut runner = make_fixture_command_runner();
    runner.metadata_mutates_lock_bytes = true;
    let error = load_workspace_crate_graph(directory.path(), &runner).unwrap_err();
    assert!(error.to_string().contains("CrateGraphLoadError"));
    assert_eq!(
        std::fs::read(directory.path().join("Cargo.lock")).unwrap(),
        original
    );
}

#[test]
fn test_metadata_failure_writes_nothing() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("Cargo.lock"), b"lock").unwrap();
    let mut runner = make_fixture_command_runner();
    runner.metadata_failure_message_text = Some("offline stderr: uncached dependency".into());
    let error = load_workspace_crate_graph(directory.path(), &runner).unwrap_err();
    assert!(error.to_string().contains("uncached dependency"));
}

#[test]
fn test_cfg_from_rustc_print() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("Cargo.lock"), b"lock").unwrap();
    let runner = make_fixture_command_runner();
    let rows = load_workspace_crate_graph(directory.path(), &runner).unwrap();
    let row = rows
        .iter()
        .find(|r| r.stage_owned_table_name == "crate-dependency-graph-table")
        .unwrap();
    let unit: serde_json::Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
    assert!(
        unit["unit_target_cfg_values"]
            .as_array()
            .unwrap()
            .contains(&json!("unix"))
    );
    assert!(
        unit["unit_target_cfg_values"]
            .as_array()
            .unwrap()
            .contains(&json!("feature=\"one\""))
    );
    assert!(
        !unit["unit_target_cfg_values"]
            .as_array()
            .unwrap()
            .contains(&json!("test"))
    );
}

#[test]
fn test_build_script_never_executed() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("Cargo.lock"), b"lock").unwrap();
    std::fs::write(
        directory.path().join("build.rs"),
        b"compile_error!(\"must not run\");",
    )
    .unwrap();
    let runner = make_fixture_command_runner();
    let rows = load_workspace_crate_graph(directory.path(), &runner).unwrap();
    assert!(rows.iter().any(|r| {
        r.stage_row_payload_text
            .contains("unit_build_script_present")
    }));
    assert!(
        runner
            .command_argument_capture_rows
            .borrow()
            .iter()
            .all(|r| r[0] != "build-script-build")
    );
}

#[test]
fn test_cargo_env_from_metadata() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("Cargo.lock"), b"lock").unwrap();
    let rows =
        load_workspace_crate_graph(directory.path(), &make_fixture_command_runner()).unwrap();
    let unit: serde_json::Value = rows
        .iter()
        .find_map(|r| {
            (r.stage_owned_table_name == "crate-dependency-graph-table")
                .then(|| serde_json::from_str(&r.stage_row_payload_text).unwrap())
        })
        .unwrap();
    assert_eq!(
        unit["unit_metadata_environment_values"]["CARGO_PKG_NAME"],
        "sample"
    );
    assert_eq!(
        unit["unit_metadata_environment_values"]["CARGO_PKG_VERSION"],
        "0.1.0"
    );
    assert!(
        unit["unit_metadata_environment_values"]
            .get("OUT_DIR")
            .is_none()
    );
}

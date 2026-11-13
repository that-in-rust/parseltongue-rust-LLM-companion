#![allow(clippy::unwrap_used)]

use parseltongue_core_row_types::{RunCancelFlagValue, StageRunScopeValue, StageUnitWriteRequest};
use parseltongue_table_store_traits::StageTableStorePort;
use parseltongue_turso_store_adapter::TursoWorkspaceStoreAdapter;
use serde_json::Value;
use std::collections::BTreeSet;
use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

fn create_workspace_source_fixture(root: &Path) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("Cargo.toml"), "[package]\nname=\"fixture\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[features]\ndefault=[\"enabled\"]\nenabled=[]\n").unwrap();
    fs::write(
        root.join("Cargo.lock"),
        "version = 4\n[[package]]\nname = \"fixture\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(root.join("src/lib.rs"), "mod child; #[cfg(test)] mod absent; pub struct A { pub field: u8 } pub fn f(x: u8) -> u8 { child::g(x) }").unwrap();
    fs::write(root.join("src/child.rs"), "pub fn g(x: u8) -> u8 { x }").unwrap();
    fs::write(root.join("unrelated.rs"), [255, 0]).unwrap();
    fs::write(
        root.join("build.rs"),
        "compile_error!(\"never execute me\");",
    )
    .unwrap();
    let lock = Command::new(env!("CARGO"))
        .current_dir(root)
        .args(["generate-lockfile", "--offline"])
        .output()
        .unwrap();
    assert!(
        lock.status.success(),
        "{}",
        String::from_utf8_lossy(&lock.stderr)
    );
}
fn run_workspace_mapping_fixture(root: &Path, database: &Path, workers: &str) -> Output {
    let file_paths = allocate_child_output_paths(database).unwrap();
    eprintln!(
        "native_child_stderr_log={}",
        file_paths.child_stderr_output_path.display()
    );
    let mut command = Command::new(env!("CARGO_BIN_EXE_parseltongue"));
    command
        .args([
            "map-workspace-full-run",
            "--workspace-root-path-value",
            root.to_str().unwrap(),
            "--workspace-store-file-path",
            database.to_str().unwrap(),
            "--worker-thread-count-value",
            workers,
        ])
        .stdin(Stdio::null());
    let capture = spawn_child_output_capture(command, file_paths).unwrap();
    finish_child_output_capture(capture).unwrap()
}

struct ChildOutputFilePaths {
    child_stdout_output_path: PathBuf,
    child_stderr_output_path: PathBuf,
}

struct WorkspaceChildOutputCapture {
    workspace_child_process_handle: Child,
    child_output_file_paths: ChildOutputFilePaths,
}

fn allocate_child_output_paths(database: &Path) -> io::Result<ChildOutputFilePaths> {
    let parent = database
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let (_, child_stdout_output_path) = tempfile::Builder::new()
        .prefix("native-child-stdout-log-")
        .tempfile_in(parent)?
        .keep()
        .map_err(|error| error.error)?;
    let (_, child_stderr_output_path) = tempfile::Builder::new()
        .prefix("native-child-stderr-log-")
        .tempfile_in(parent)?
        .keep()
        .map_err(|error| error.error)?;
    Ok(ChildOutputFilePaths {
        child_stdout_output_path,
        child_stderr_output_path,
    })
}

fn spawn_child_output_capture(
    mut command: Command,
    file_paths: ChildOutputFilePaths,
) -> io::Result<WorkspaceChildOutputCapture> {
    let stdout_file = fs::File::create(&file_paths.child_stdout_output_path)?;
    let stderr_file = fs::File::create(&file_paths.child_stderr_output_path)?;
    let child = command
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .spawn()?;
    Ok(WorkspaceChildOutputCapture {
        workspace_child_process_handle: child,
        child_output_file_paths: file_paths,
    })
}

fn finish_child_output_capture(mut capture: WorkspaceChildOutputCapture) -> io::Result<Output> {
    let status = capture.workspace_child_process_handle.wait()?;
    let stdout = fs::read(&capture.child_output_file_paths.child_stdout_output_path)?;
    let stderr = fs::read(&capture.child_output_file_paths.child_stderr_output_path)?;
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

impl Drop for WorkspaceChildOutputCapture {
    fn drop(&mut self) {
        if !matches!(self.workspace_child_process_handle.try_wait(), Ok(Some(_))) {
            let _ = self.workspace_child_process_handle.kill();
            let _ = self.workspace_child_process_handle.wait();
        }
    }
}

fn build_child_fixture_command(mode: &str, ready_marker_path: &Path) -> io::Result<Command> {
    let mut command = Command::new(std::env::current_exe()?);
    command
        .args([
            "--exact",
            "test_child_fixture_waits_release",
            "--ignored",
            "--nocapture",
        ])
        .env("PARSELTONGUE_CHILD_CAPTURE_MODE", mode)
        .env("PARSELTONGUE_CHILD_READY_FILE", ready_marker_path)
        .stdin(Stdio::piped());
    Ok(command)
}

fn wait_for_ready_marker(
    capture: &mut WorkspaceChildOutputCapture,
    ready_marker_path: &Path,
    timeout: Duration,
) -> io::Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        if ready_marker_path.is_file() {
            return Ok(());
        }
        if capture.workspace_child_process_handle.try_wait()?.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "child exited before readiness",
            ));
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "child readiness deadline elapsed",
            ));
        }
        thread::yield_now();
    }
}

fn wait_child_until_exit(child: &mut Child, timeout: Duration) -> io::Result<ExitStatus> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "child exit deadline elapsed",
            ));
        }
        thread::yield_now();
    }
}

fn write_child_release_signal(capture: &mut WorkspaceChildOutputCapture) -> io::Result<()> {
    let mut child_input = capture
        .workspace_child_process_handle
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("child stdin was not piped"))?;
    child_input.write_all(&[1])
}

/// REQ-RUST-028.0
#[test]
fn test_child_stderr_live_visibility() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("store.db");
    let file_paths = allocate_child_output_paths(&database).unwrap();
    let stderr_path = file_paths.child_stderr_output_path.clone();
    let ready_marker_path = directory.path().join("native-child-ready-marker");
    let command = build_child_fixture_command("live", &ready_marker_path).unwrap();
    let mut capture = spawn_child_output_capture(command, file_paths).unwrap();

    let readiness = wait_for_ready_marker(&mut capture, &ready_marker_path, Duration::from_secs(5));
    let child_running = capture
        .workspace_child_process_handle
        .try_wait()
        .map(|status| status.is_none());
    let live_stderr = fs::read(stderr_path);
    let release = write_child_release_signal(&mut capture);
    let exit = wait_child_until_exit(
        &mut capture.workspace_child_process_handle,
        Duration::from_secs(5),
    );
    assert!(exit.is_ok(), "controlled child did not exit: {exit:?}");
    let output = finish_child_output_capture(capture).unwrap();

    assert!(
        readiness.is_ok(),
        "controlled child did not signal readiness"
    );
    assert!(child_running.unwrap(), "child exited before log inspection");
    assert!(release.is_ok(), "controlled child could not be released");
    let live_stderr = live_stderr.unwrap();
    let marker = b"native-child-diagnostic-live\n";
    assert!(
        live_stderr
            .windows(marker.len())
            .any(|bytes| bytes == marker),
        "diagnostic was unavailable before child completion: {live_stderr:?}"
    );
    assert!(output.status.success());
    assert!(
        output
            .stderr
            .windows(marker.len())
            .any(|bytes| bytes == marker)
    );
}

/// REQ-RUST-028.0
#[test]
fn test_child_output_exit_preservation() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("store.db");
    let file_paths = allocate_child_output_paths(&database).unwrap();
    let ready_marker_path = directory.path().join("native-child-ready-marker");
    let command = build_child_fixture_command("output", &ready_marker_path).unwrap();
    let mut capture = spawn_child_output_capture(command, file_paths).unwrap();
    let exit = wait_child_until_exit(
        &mut capture.workspace_child_process_handle,
        Duration::from_secs(5),
    );
    assert!(exit.is_ok(), "output fixture did not exit: {exit:?}");
    let output = finish_child_output_capture(capture).unwrap();

    assert_eq!(output.status.code(), Some(23));
    assert!(output.stdout.ends_with(b"fixture-child-stdout\n"));
    assert!(output.stderr.ends_with(b"fixture-child-stderr\n"));
}

/// REQ-RUST-028.0
#[test]
fn test_child_spawn_error_propagation() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("store.db");
    let file_paths = allocate_child_output_paths(&database).unwrap();
    let command = Command::new(directory.path().join("missing-child-executable"));
    let error = spawn_child_output_capture(command, file_paths)
        .err()
        .unwrap();

    assert_eq!(error.kind(), io::ErrorKind::NotFound);
}

/// REQ-RUST-028.0
#[test]
fn test_child_output_read_error() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("store.db");
    let file_paths = allocate_child_output_paths(&database).unwrap();
    let stdout_path = file_paths.child_stdout_output_path.clone();
    let ready_marker_path = directory.path().join("native-child-ready-marker");
    let command = build_child_fixture_command("output", &ready_marker_path).unwrap();
    let mut capture = spawn_child_output_capture(command, file_paths).unwrap();
    let exit = wait_child_until_exit(
        &mut capture.workspace_child_process_handle,
        Duration::from_secs(5),
    );
    assert!(exit.is_ok(), "output fixture did not exit: {exit:?}");
    fs::remove_file(stdout_path).unwrap();
    let error = finish_child_output_capture(capture).err().unwrap();

    assert_eq!(error.kind(), io::ErrorKind::NotFound);
}

/// REQ-RUST-028.0
#[test]
#[ignore = "subprocess fixture only"]
fn test_child_fixture_waits_release() {
    match std::env::var("PARSELTONGUE_CHILD_CAPTURE_MODE").as_deref() {
        Ok("live") => {
            let ready_marker_path = std::env::var_os("PARSELTONGUE_CHILD_READY_FILE")
                .map(PathBuf::from)
                .unwrap();
            let mut diagnostic = std::io::stderr().lock();
            diagnostic
                .write_all(b"native-child-diagnostic-live\n")
                .unwrap();
            diagnostic.flush().unwrap();
            drop(diagnostic);
            fs::write(ready_marker_path, b"ready").unwrap();
            let mut release = [0];
            std::io::stdin().read_exact(&mut release).unwrap();
        }
        Ok("output") => {
            std::io::stdout()
                .write_all(b"fixture-child-stdout\n")
                .unwrap();
            std::io::stderr()
                .write_all(b"fixture-child-stderr\n")
                .unwrap();
            std::process::exit(23);
        }
        _ => {}
    }
}

fn run_workspace_stage_fixture(root: &Path, database: &Path, stage: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_parseltongue"))
        .args([
            "map-workspace-full-run",
            "--workspace-root-path-value",
            root.to_str().unwrap(),
            "--workspace-store-file-path",
            database.to_str().unwrap(),
            "--worker-thread-count-value",
            "2",
            "--mapping-final-stage-kind",
            stage,
        ])
        .output()
        .unwrap()
}

fn write_stale_completion_marker(
    store: &TursoWorkspaceStoreAdapter,
    scope: &StageRunScopeValue,
    stage: &str,
    inputs: &[&str],
) {
    store
        .write_stage_table_unit(
            &StageUnitWriteRequest {
                stage_run_scope_value: scope,
                stage_owner_name_value: stage,
                stage_unit_work_key: "__complete__",
                stage_input_stage_names: inputs,
                stage_unit_output_rows: &[],
            },
            &RunCancelFlagValue::default(),
        )
        .unwrap();
}
fn read_workspace_banner_fixture(database: &Path) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_parseltongue"))
        .args([
            "status-coverage-banner-show",
            "--json",
            "--workspace-store-file-path",
            database.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn read_published_table_fixture(database: &Path, table: &str) -> Vec<Value> {
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let scope = store.read_workspace_publication_scope().unwrap().unwrap();
    store
        .read_stage_table_rows(&scope, table)
        .unwrap()
        .iter()
        .map(|row| serde_json::from_str(&row.stage_row_payload_text).unwrap())
        .collect()
}

fn assert_receipt_product_rows(
    store: &TursoWorkspaceStoreAdapter,
    scope: &StageRunScopeValue,
    receipt: &Value,
) -> u64 {
    let expected_tables = [
        "name-resolution-scope-records",
        "module-scope-binding-table",
        "path-resolution-site-records",
    ];
    let proofs = receipt["analysis_product_content_proofs"]
        .as_object()
        .unwrap();
    let actual_tables = proofs.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected_table_set = expected_tables.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(actual_tables, expected_table_set);
    let mut total = 0;
    for table in expected_tables {
        let mut rows = store.read_stage_table_rows(scope, table).unwrap();
        rows.sort_unstable_by(|left, right| {
            left.stage_row_record_key.cmp(&right.stage_row_record_key)
        });
        let row_count = rows.len() as u64;
        assert_eq!(
            proofs[table]["row_count"].as_u64(),
            Some(row_count),
            "{table} row count"
        );
        assert!(
            rows.windows(2)
                .all(|pair| { pair[0].stage_row_record_key != pair[1].stage_row_record_key }),
            "{table} product keys must be unique"
        );
        let mut hasher = blake3::Hasher::new();
        rows.iter().for_each(|row| {
            hasher.update(&hash_published_row_leaf(
                table,
                &row.stage_row_record_key,
                &row.stage_row_payload_text,
            ));
        });
        let digest = hasher.finalize().to_hex().to_string();
        assert_eq!(
            proofs[table]["content_digest"].as_str(),
            Some(digest.as_str()),
            "{table} ordered digest"
        );
        total += row_count;
    }
    assert_eq!(
        receipt["analysis_product_record_count"].as_u64(),
        Some(total)
    );
    total
}

fn hash_published_row_leaf(table: &str, key: &str, payload: &str) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    [table, key, payload].into_iter().for_each(|value| {
        hasher.update(&(value.len() as u64).to_le_bytes());
        hasher.update(value.as_bytes());
    });
    *hasher.finalize().as_bytes()
}

#[test]
fn test_macro_items_publish_atomically() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(
        root.join("src/lib.rs"),
        r#"
macro_rules! outer { ($name:ident) => {
    macro_rules! inner { () => { pub struct $name { pub field:u8 } }; }
    inner!();
}; }
outer!(Generated);
pub fn take(_:Generated){}
"#,
    )
    .unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let macros = read_published_table_fixture(&database, "macro-expanded-item-table");
    assert_eq!(macros.len(), 2);
    let nested = macros
        .iter()
        .find(|row| {
            row["macro_invocation_provenance_chain"]
                .as_array()
                .unwrap()
                .len()
                == 2
        })
        .unwrap();
    assert!(
        nested["macro_expanded_source_text"]
            .as_str()
            .unwrap()
            .contains("Generated")
    );
    let path = read_published_table_fixture(&database, "path-resolution-site-records")
        .into_iter()
        .find(|row| row["path_site_source_text"] == "Generated")
        .unwrap();
    assert_eq!(path["path_resolution_state_kind"], "resolved");
    let targets = path["path_target_symbol_rows"].as_array().unwrap();
    assert_eq!(targets.len(), 1);
    assert!(
        targets[0]["target_source_file_path"]
            .as_str()
            .unwrap()
            .contains("macro-source")
    );
    assert!(
        read_published_table_fixture(&database, "span-reference-text-table")
            .iter()
            .any(|row| !row["span_expansion_provenance_chain"]
                .as_array()
                .is_none_or(|chain| chain.is_empty()))
    );
    assert!(
        read_published_table_fixture(&database, "field-signature-detail-table")
            .iter()
            .any(|row| !row["span_expansion_provenance_chain"]
                .as_array()
                .is_none_or(|chain| chain.is_empty()))
    );

    let before = read_workspace_banner_fixture(&database);
    fs::write(
        root.join("src/lib.rs"),
        "macro_rules! make {($name:ident)=>{pub struct $name;};} make!(Completed); make!(1);",
    )
    .unwrap();
    let failed = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(!failed.status.success());
    assert_eq!(read_workspace_banner_fixture(&database), before);
    assert_eq!(
        read_published_table_fixture(&database, "macro-expanded-item-table"),
        macros
    );
}

#[test]
fn test_macro_defining_crate_hygiene() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::create_dir_all(root.join("defs/src")).unwrap();
    fs::write(
        root.join("defs/Cargo.toml"),
        "[package]\nname=\"defs\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
    )
    .unwrap();
    fs::write(root.join("defs/src/lib.rs"),"pub struct Original; #[macro_export] macro_rules! build {($name:ident)=>{pub struct $name {pub value:$crate::Original}};}").unwrap();
    let manifest = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        format!("{manifest}\n[dependencies]\ndefs={{path=\"defs\"}}\n"),
    )
    .unwrap();
    fs::write(root.join("src/lib.rs"),"pub struct Original; defs::build!(First); defs::build!(Second); pub fn take(_:First,_:Second){}").unwrap();
    assert!(
        Command::new(env!("CARGO"))
            .current_dir(&root)
            .args(["generate-lockfile", "--offline"])
            .output()
            .unwrap()
            .status
            .success()
    );
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let macros = read_published_table_fixture(&database, "macro-expanded-item-table");
    assert_eq!(macros.len(), 2);
    assert_ne!(
        macros[0]["macro_virtual_source_path"],
        macros[1]["macro_virtual_source_path"]
    );
    let paths = read_published_table_fixture(&database, "path-resolution-site-records");
    let defining = paths
        .iter()
        .filter(|row| {
            row["path_site_source_text"].as_str().is_some_and(|text| {
                text.chars()
                    .filter(|ch| !ch.is_whitespace())
                    .collect::<String>()
                    == "$crate::Original"
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(defining.len(), 2);
    for path in defining {
        assert_eq!(path["path_resolution_state_kind"], "resolved");
        assert!(
            path["path_target_symbol_rows"][0]["target_source_file_path"]
                .as_str()
                .unwrap()
                .ends_with("/defs/src/lib.rs")
        );
    }
}

#[test]
fn test_macro_empty_virtual_file() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(
        root.join("src/lib.rs"),
        "macro_rules! empty {()=>{};} empty!();",
    )
    .unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows = read_published_table_fixture(&database, "macro-expanded-item-table");
    assert_eq!(rows.len(), 1);
    assert!(
        rows[0]["macro_expanded_source_text"]
            .as_str()
            .unwrap()
            .trim()
            .is_empty()
    );
    let files = read_published_table_fixture(&database, "file-source-text-table");
    assert!(files.iter().any(|row| row["file_source_absolute_path"]
        == rows[0]["macro_virtual_source_path"]
        && row["file_source_byte_count"] == 0));
}

#[test]
fn test_macro_declarations_publish_rows() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(root.join("src/lib.rs"),
        "#![feature(decl_macro)] macro make { (first)=>{pub struct First;}, (second)=>{pub struct Second;}, } make!(second); pub fn take(_:Second){}").unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        read_published_table_fixture(&database, "path-resolution-site-records")
            .iter()
            .any(|row| row["path_site_source_text"] == "Second"
                && row["path_resolution_state_kind"] == "resolved")
    );
}

#[test]
fn test_flags_without_sysroot_support() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::create_dir_all(root.join("defs/src")).unwrap();
    fs::write(
        root.join("defs/Cargo.toml"),
        "[package]\nname=\"defs\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
    )
    .unwrap();
    fs::write(root.join("defs/src/lib.rs"), "#![no_std] pub struct Item;").unwrap();
    let manifest = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        format!("{manifest}\n[dependencies]\ndep={{path=\"defs\",package=\"defs\"}}\n"),
    )
    .unwrap();
    fs::write(root.join("src/lib.rs"), "#![no_std] #![no_implicit_prelude] extern crate dep as local; pub fn take(_:local::Item,_:u8){}").unwrap();
    assert!(
        Command::new(env!("CARGO"))
            .current_dir(&root)
            .args(["generate-lockfile", "--offline"])
            .output()
            .unwrap()
            .status
            .success()
    );
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let units = read_published_table_fixture(&database, "crate-dependency-graph-table");
    assert!(
        units
            .iter()
            .all(|unit| unit["unit_sysroot_context_flag"] != true)
    );
    let unit = units
        .iter()
        .find(|unit| {
            unit["unit_package_root_path"] == root.to_str().unwrap()
                && unit["unit_cargo_target_kind"] == "lib"
                && unit["unit_compilation_role_kind"] == "target"
        })
        .unwrap();
    assert!(units.iter().any(
        |unit| unit["unit_package_root_path"] == root.to_str().unwrap()
            && unit["unit_cargo_target_kind"] == "custom-build"
            && unit["unit_mapped_target_flag"] == false
    ));
    assert_eq!(unit["unit_no_std_flag"], true);
    assert_eq!(unit["unit_implicit_prelude_flag"], false);
    let paths = read_published_table_fixture(&database, "path-resolution-site-records");
    for name in ["local::Item", "u8"] {
        assert!(
            paths.iter().any(|row| row["path_site_source_text"] == name
                && row["path_resolution_state_kind"] == "resolved"),
            "{name}"
        );
    }
}

#[test]
fn test_grouped_self_import_rules() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(root.join("src/lib.rs"), "pub mod defs {pub enum Choice {First}} use defs::Choice::{self as Alias, First}; pub fn take(_:Alias,_:defs::Choice){}").unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let paths = read_published_table_fixture(&database, "path-resolution-site-records");
    for name in ["defs::Choice", "Alias"] {
        assert!(
            paths.iter().any(|row| row["path_site_source_text"] == name
                && row["path_resolution_state_kind"] == "resolved"),
            "{name}"
        );
    }
}

#[test]
fn test_nested_macro_crate_provenance() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::create_dir_all(root.join("defs/src")).unwrap();
    fs::write(
        root.join("defs/Cargo.toml"),
        "[package]\nname=\"defs\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
    )
    .unwrap();
    fs::write(root.join("defs/src/lib.rs"),"pub struct Original; #[macro_export] macro_rules! outer {($name:ident)=>{macro_rules! inner {()=>{pub struct $name {pub field:$crate::Original}};} inner!();};}").unwrap();
    let manifest = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        format!("{manifest}\n[dependencies]\ndefs={{path=\"defs\"}}\n"),
    )
    .unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub struct Original; defs::outer!(Éclair); pub fn take(_:Éclair){}",
    )
    .unwrap();
    assert!(
        Command::new(env!("CARGO"))
            .current_dir(&root)
            .args(["generate-lockfile", "--offline"])
            .output()
            .unwrap()
            .status
            .success()
    );
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let macros = read_published_table_fixture(&database, "macro-expanded-item-table");
    assert_eq!(macros.len(), 2);
    assert!(macros.iter().any(|row| {
        row["macro_invocation_provenance_chain"]
            .as_array()
            .unwrap()
            .len()
            == 2
    }));
    let paths = read_published_table_fixture(&database, "path-resolution-site-records");
    let path = paths
        .iter()
        .find(|row| {
            row["path_site_source_text"].as_str().is_some_and(|text| {
                text.chars()
                    .filter(|ch| !ch.is_whitespace())
                    .collect::<String>()
                    == "$crate::Original"
            })
        })
        .unwrap();
    assert_eq!(path["path_resolution_state_kind"], "resolved");
    assert!(
        path["path_target_symbol_rows"][0]["target_source_file_path"]
            .as_str()
            .unwrap()
            .ends_with("/defs/src/lib.rs")
    );
    assert_eq!(
        path["site_expansion_provenance_chain"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let generated = paths
        .iter()
        .find(|row| row["path_site_source_text"] == "Éclair")
        .unwrap();
    assert_eq!(generated["path_resolution_state_kind"], "resolved");
}

/// REQ-RUST-028.1
#[test]
fn test_generated_import_refresh_trace() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::create_dir_all(root.join("defs/src")).unwrap();
    fs::write(
        root.join("defs/Cargo.toml"),
        "[package]\nname=\"defs\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
    )
    .unwrap();
    fs::write(
        root.join("defs/src/lib.rs"),
        "#![no_std] #[macro_export] macro_rules! emit_generated { () => { pub struct Generated; }; } emit_generated!();",
    )
    .unwrap();
    let manifest = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        format!("{manifest}\n[dependencies]\ndefs={{path=\"defs\"}}\n"),
    )
    .unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "#![no_std] #![no_implicit_prelude] extern crate defs; use defs::Generated; pub fn consume(_:Generated){}",
    )
    .unwrap();
    assert!(
        Command::new(env!("CARGO"))
            .current_dir(&root)
            .args(["generate-lockfile", "--offline"])
            .output()
            .unwrap()
            .status
            .success()
    );
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let refreshes = stderr
        .lines()
        .filter_map(|line| {
            let (prefix, value) = line.split_once(" trace=")?;
            prefix
                .starts_with("stage=namespace-frozen-input snapshot=")
                .then(|| serde_json::from_str::<Value>(value).unwrap())
        })
        .collect::<Vec<_>>();
    assert_eq!(refreshes.len(), 2, "{stderr}");
    assert_eq!(refreshes[0]["trigger_kind"], "initial");
    assert_eq!(refreshes[0]["outer_expansion_ordinal"], 0);
    assert!(refreshes[0]["trigger_unit_compilation_identity_key"].is_null());
    assert_eq!(
        refreshes[0]["completed_invocation_occurrence_identities"],
        serde_json::json!([])
    );
    assert_eq!(refreshes[0]["emitted_semantic_row_count"], 0);
    assert_eq!(refreshes[1]["trigger_kind"], "macro_expansion");
    assert_eq!(refreshes[1]["outer_expansion_ordinal"], 1);
    assert_eq!(
        refreshes[1]["completed_invocation_occurrence_identities"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(refreshes.windows(2).all(|pair| {
        pair[0]["outer_expansion_ordinal"].as_u64().unwrap()
            < pair[1]["outer_expansion_ordinal"].as_u64().unwrap()
    }));
    assert!(refreshes[1]["emitted_semantic_row_count"].as_u64().unwrap() > 0);
    let units = read_published_table_fixture(&database, "crate-dependency-graph-table");
    let dependency = units
        .iter()
        .find(|unit| {
            unit["unit_package_root_path"] == root.join("defs").to_str().unwrap()
                && unit["unit_cargo_target_kind"] == "lib"
                && unit["unit_compilation_role_kind"] == "target"
        })
        .unwrap();
    assert_eq!(
        refreshes[1]["trigger_unit_compilation_identity_key"],
        dependency["unit_compilation_identity_key"]
    );
    assert_eq!(
        refreshes[1]["completed_invocation_occurrence_identities"][0]["unit_compilation_identity_key"]
            ["package_identifier"],
        dependency["unit_cargo_package_identifier"]
    );
    assert!(
        units
            .iter()
            .all(|unit| unit["unit_sysroot_context_flag"] != true)
    );
    let copied = stderr
        .lines()
        .filter_map(|line| {
            let (prefix, value) = line.split_once(" trace=")?;
            (prefix == "stage=namespace-frozen-input event=copy-verified")
                .then(|| serde_json::from_str::<Value>(value).unwrap())
        })
        .collect::<Vec<_>>();
    assert_eq!(copied.len(), 2, "{stderr}");
    assert_eq!(copied[0]["generation_key"], refreshes[0]["generation_key"]);
    assert_eq!(copied[1]["generation_key"], refreshes[1]["generation_key"]);
    assert_eq!(
        copied[0]["frozen_attempt_key"],
        refreshes[0]["frozen_attempt_key"]
    );
    assert_eq!(
        copied[1]["frozen_attempt_key"],
        refreshes[1]["frozen_attempt_key"]
    );
    let first_digest = copied[0]["source_input_digest"].as_str().unwrap();
    let second_digest = copied[1]["source_input_digest"].as_str().unwrap();
    assert_eq!(first_digest.len(), 64);
    assert_eq!(second_digest.len(), 64);
    assert_ne!(first_digest, second_digest);
    let fixed_points = stderr
        .lines()
        .filter_map(|line| {
            let (prefix, value) = line.split_once(" trace=")?;
            prefix
                .starts_with("stage=namespace-sql-fixed-point ")
                .then(|| serde_json::from_str::<Value>(value).unwrap())
        })
        .collect::<Vec<_>>();
    assert_eq!(fixed_points.len(), 2, "{stderr}");
    assert_eq!(fixed_points[0]["source_input_digest"], first_digest);
    assert_eq!(fixed_points[1]["source_input_digest"], second_digest);
    assert!(fixed_points.iter().all(|report| {
        report["binding_digest"]
            .as_str()
            .is_some_and(|digest| !digest.is_empty())
            && report["inner_sql_round_count"].as_u64().unwrap_or_default() > 0
    }));
    let exhausted = stderr
        .lines()
        .filter_map(|line| {
            let (prefix, value) = line.split_once(" trace=")?;
            (prefix == "stage=namespace-macro-port event=exhausted")
                .then(|| serde_json::from_str::<Value>(value).unwrap())
        })
        .collect::<Vec<_>>();
    assert_eq!(exhausted.len(), 2, "{stderr}");
    assert_eq!(
        exhausted[0]["unit_compilation_identity_key"],
        dependency["unit_compilation_identity_key"]
    );
    assert_eq!(exhausted[0]["outer_expansion_ordinal"], 1);
    let consumer = units
        .iter()
        .find(|unit| {
            unit["unit_package_root_path"] == root.to_str().unwrap()
                && unit["unit_cargo_target_kind"] == "lib"
                && unit["unit_compilation_role_kind"] == "target"
        })
        .unwrap();
    assert_eq!(
        exhausted[1]["unit_compilation_identity_key"],
        consumer["unit_compilation_identity_key"]
    );
    assert_eq!(exhausted[1]["outer_expansion_ordinal"], 1);
    let resolved_consumer_path =
        read_published_table_fixture(&database, "path-resolution-site-records")
            .into_iter()
            .find(|path| {
                path["path_site_source_text"] == "Generated"
                    && path["site_compilation_scope_record"]["unit_cargo_target_name"] == "fixture"
            })
            .unwrap();
    assert_eq!(
        resolved_consumer_path["path_resolution_state_kind"],
        "resolved"
    );
    assert_eq!(
        resolved_consumer_path["path_target_symbol_rows"][0]["target_symbol_kind_value"],
        "Struct"
    );
    assert!(
        resolved_consumer_path["path_target_symbol_rows"][0]["target_source_file_path"]
            .as_str()
            .unwrap()
            .contains("macro-source"),
        "{resolved_consumer_path:#?}"
    );
}

/// REQ-RUST-024.10
/// REQ-RUST-024.11
#[test]
fn test_persisted_resolution_path_answers() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    let source = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    fs::write(root.join("src/lib.rs"),format!("{source}\npub use child::T as PublicT; mod alias {{ pub use crate::PublicT as Again; pub fn make(x:Again)->Again{{x}} }}")).unwrap();
    fs::write(
        root.join("src/child.rs"),
        "pub struct T; pub fn g(x:u8)->u8{x}",
    )
    .unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let banner: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        banner["stage_publication_status_values"],
        serde_json::json!({
            "crate-graph": "complete",
            "source-ingest": "complete",
            "syntax-spans": "complete",
            "name-resolution": "complete"
        })
    );
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let scope = store.read_workspace_publication_scope().unwrap().unwrap();
    let rows: Vec<Value> = store
        .read_stage_table_rows(&scope, "path-resolution-site-records")
        .unwrap()
        .iter()
        .map(|row| serde_json::from_str(&row.stage_row_payload_text).unwrap())
        .collect();
    let answers: Vec<_> = rows
        .iter()
        .filter(|row| row["path_site_source_text"] == "Again")
        .collect();
    assert_eq!(answers.len(), 2);
    for answer in answers {
        assert_eq!(
            answer["site_compilation_scope_record"]["unit_module_path_values"],
            serde_json::json!(["alias"])
        );
        let targets = answer["path_target_symbol_rows"].as_array().unwrap();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0]["target_symbol_kind_value"], "Struct");
        assert!(
            targets[0]["target_source_file_path"]
                .as_str()
                .unwrap()
                .ends_with("/src/child.rs")
        );
    }
    assert!(
        !store
            .read_stage_table_rows(&scope, "module-scope-binding-table")
            .unwrap()
            .is_empty()
    );
    let receipt_rows = store
        .read_stage_table_rows(&scope, "analysis-resolution-receipt-records")
        .unwrap();
    assert_eq!(receipt_rows.len(), 1);
    let receipt: Value = serde_json::from_str(&receipt_rows[0].stage_row_payload_text).unwrap();
    let product_count = assert_receipt_product_rows(&store, &scope, &receipt);
    assert!(product_count > 0);
    drop(store);
    let banner = read_workspace_banner_fixture(&database);
    assert_eq!(
        banner["stage_implementation_status_values"]["name-resolution"],
        "complete"
    );
}

#[test]
fn test_builtin_include_name_resolution() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(root.join("src/lib.rs"),r#"include!(concat!(env!("CARGO_MANIFEST_DIR"),"/src/fixture.rs")); pub fn take(_:Included){}"#).unwrap();
    fs::write(root.join("src/fixture.rs"),"#[cfg_attr(not(test),cfg(feature=\"enabled\"))] pub struct Included {pub field:u8} #[cfg(test)] pub struct Excluded;").unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows = read_published_table_fixture(&database, "path-resolution-site-records");
    let included = rows
        .iter()
        .find(|row| row["path_site_source_text"] == "Included")
        .unwrap();
    assert_eq!(included["path_resolution_state_kind"], "resolved");
    assert!(
        !read_published_table_fixture(&database, "module-scope-binding-table")
            .iter()
            .any(|row| row["binding_resolved_name_text"] == "Excluded")
    );
}

#[test]
fn test_builtin_missing_source_reasons() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(
        root.join("src/lib.rs"),
        "include!(concat!(env!(\"OUT_DIR\"),\"/generated.rs\"));",
    )
    .unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows = read_published_table_fixture(&database, "macro-expanded-item-table");
    assert!(
        rows.iter()
            .any(|row| row["macro_unresolved_reason_kind"] == "build_script"
                && row["macro_resolution_state_kind"] == "unresolved")
    );
}

#[test]
fn test_builtin_include_callsite_base() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(root.join("src/lib.rs"), "mod defs; pub mod child;").unwrap();
    fs::write(
        root.join("src/defs.rs"),
        "#[macro_export] macro_rules! emit {()=>{include!(\"chosen.rs\");};}",
    )
    .unwrap();
    fs::write(
        root.join("src/child.rs"),
        "crate::emit!(); pub fn take(_:Chosen){}",
    )
    .unwrap();
    fs::write(root.join("src/chosen.rs"), "pub struct Chosen;").unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let paths = read_published_table_fixture(&database, "path-resolution-site-records");
    let chosen = paths
        .iter()
        .find(|row| row["path_site_source_text"] == "Chosen")
        .unwrap();
    assert_eq!(chosen["path_resolution_state_kind"], "resolved");
    assert!(
        chosen["path_target_symbol_rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["target_source_file_path"]
                .as_str()
                .is_some_and(|path| path.ends_with("/src/chosen.rs")))
    );
    assert!(
        read_published_table_fixture(&database, "macro-expanded-item-table")
            .iter()
            .any(|row| row["macro_invocation_provenance_chain"]
                .as_array()
                .is_some_and(|chain| chain.len() == 2))
    );
}

#[test]
fn test_builtin_nested_macro_shadowing() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(root.join("src/lib.rs"),r#"macro_rules! concat {($($tt:tt)*)=>{"other.rs"}} include!(concat!("right.rs")); pub fn take(_:Other){}"#).unwrap();
    fs::write(root.join("src/right.rs"), "pub struct Right;").unwrap();
    fs::write(root.join("src/other.rs"), "pub struct Other;").unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let macros = read_published_table_fixture(&database, "macro-expanded-item-table");
    assert!(macros.iter().any(|row| {
        row["macro_unresolved_reason_kind"] == "external_source_missing"
            && row["macro_unresolved_detail_text"]
                .as_str()
                .is_some_and(|detail| detail.contains("nested macro expansion unavailable"))
    }));
    assert!(
        !read_published_table_fixture(&database, "module-scope-binding-table")
            .iter()
            .any(|row| matches!(
                row["binding_resolved_name_text"].as_str(),
                Some("Right" | "Other")
            ))
    );
}

#[test]
fn test_lang_items_in_sysroot() {
    let retained = std::env::var_os("PARSELTONGUE_NATIVE_FIXTURE_DIRECTORY");
    let temporary = retained
        .as_ref()
        .map_or_else(tempfile::tempdir, tempfile::tempdir_in)
        .unwrap();
    let directory = temporary.path().to_path_buf();
    let _guard = if retained.is_some() {
        let _ = temporary.keep();
        None
    } else {
        Some(temporary)
    };
    eprintln!("native_fixture_directory={}", directory.display());
    let root = directory.join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(root.join("src/lib.rs"),"pub fn build(_:Option<Vec<u8>>,_:core::ops::Range<u8>)->std::string::String{String::new()}").unwrap();
    let database = directory.join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let paths = read_published_table_fixture(&database, "path-resolution-site-records");
    for name in [
        "Option<Vec<u8>>",
        "Vec<u8>",
        "core::ops::Range<u8>",
        "std::string::String",
    ] {
        let row = paths.iter().find(|row| {
            row["path_site_source_text"] == name
                && row["site_compilation_scope_record"]["unit_cargo_target_name"] == "fixture"
        });
        assert!(
            row.is_some_and(|row| row["path_resolution_state_kind"] == "resolved"),
            "{name}: {row:?}"
        );
    }
    let langs = read_published_table_fixture(&database, "lang-item-definition-table");
    for name in [
        "sized",
        "copy",
        "owned_box",
        "panic_in_cleanup",
        "coerce_pointee_validated",
    ] {
        assert!(
            langs.iter().any(|row| row["lang_item_source_name"] == name),
            "{name}"
        );
    }
    assert!(
        langs
            .iter()
            .all(|row| row["lang_definition_provenance_chain"].is_array())
    );
}

#[test]
fn test_path_attributes_follow_source() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(
        root.join("src/lib.rs"),
        "mod parent; pub fn take(_:parent::Selected){}",
    )
    .unwrap();
    fs::write(
        root.join("src/parent.rs"),
        "#[path=\"selected.rs\"] mod selected; pub use selected::Selected;",
    )
    .unwrap();
    fs::write(root.join("src/selected.rs"), "pub struct Selected;").unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let paths = read_published_table_fixture(&database, "path-resolution-site-records");
    assert!(
        paths
            .iter()
            .any(|row| row["path_site_source_text"] == "parent::Selected"
                && row["path_resolution_state_kind"] == "resolved")
    );
}

#[test]
fn test_mapping_stage_selection_contract() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    let database = directory.path().join("store.db");
    let run = |stage: &str| {
        Command::new(env!("CARGO_BIN_EXE_parseltongue"))
            .args([
                "map-workspace-full-run",
                "--workspace-root-path-value",
                root.to_str().unwrap(),
                "--workspace-store-file-path",
                database.to_str().unwrap(),
                "--mapping-final-stage-kind",
                stage,
            ])
            .output()
            .unwrap()
    };
    let output = run("syntax-spans");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let banner: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        banner["stage_implementation_status_values"]["syntax-spans"],
        "complete"
    );
    assert_eq!(
        banner["stage_implementation_status_values"]["name-resolution"],
        "not yet implemented"
    );
    assert!(read_published_table_fixture(&database, "module-scope-binding-table").is_empty());
    let output = run("");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown mapping final stage"));
}

/// REQ-RUST-024.11
#[test]
fn test_syntax_publication_status_complete() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    let database = directory.path().join("store.db");
    let output = run_workspace_stage_fixture(&root, &database, "syntax-spans");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let banner: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        banner["stage_implementation_status_values"]["syntax-spans"],
        "complete"
    );
    assert_eq!(
        banner["stage_publication_status_values"],
        serde_json::json!({
            "crate-graph": "complete",
            "source-ingest": "complete",
            "syntax-spans": "complete",
            "name-resolution": "pending"
        })
    );
}

/// REQ-RUST-024.11
#[test]
fn test_stale_publication_marker_pending() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    let database = directory.path().join("store.db");
    let output = run_workspace_stage_fixture(&root, &database, "syntax-spans");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let published_scope = store.read_workspace_publication_scope().unwrap().unwrap();
    let stale_scope = StageRunScopeValue {
        run_generation_digest_value: format!(
            "stale-{}",
            published_scope.run_generation_digest_value
        ),
        stage_attempt_sequence_value: "stale-attempt".into(),
    };
    write_stale_completion_marker(&store, &stale_scope, "name-resolution", &[]);
    assert!(
        store
            .read_stage_table_rows(&stale_scope, "stage-run-generation-records")
            .unwrap()
            .iter()
            .any(|row| row.stage_row_record_key == "__marker__/name-resolution/__complete__")
    );
    assert_eq!(
        store.read_workspace_publication_scope().unwrap(),
        Some(published_scope)
    );
    drop(store);

    let banner = read_workspace_banner_fixture(&database);
    assert_eq!(
        banner["stage_publication_status_values"]["name-resolution"],
        "pending"
    );
}

/// REQ-RUST-024.11
#[test]
fn test_publication_status_without_generation() {
    let directory = tempfile::tempdir().unwrap();
    let database = directory.path().join("store.db");
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    store.initialize_workspace_stage_schema().unwrap();
    drop(store);

    let banner = read_workspace_banner_fixture(&database);
    assert_eq!(
        banner["stage_publication_status_values"],
        serde_json::json!({
            "crate-graph": "pending",
            "source-ingest": "pending",
            "syntax-spans": "pending",
            "name-resolution": "pending"
        })
    );
}

#[test]
fn test_type_macros_remain_deferred() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(
        root.join("src/lib.rs"),
        "macro_rules! ty {()=>{u8};} pub fn take(_:ty!()){}",
    )
    .unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(read_published_table_fixture(&database, "macro-expanded-item-table").is_empty());
}

/// REQ-RUST-024.10
#[test]
fn test_empty_resolution_stage_rows() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(root.join("src/lib.rs"), "").unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "2");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let scope = store.read_workspace_publication_scope().unwrap().unwrap();
    assert_eq!(
        store
            .count_stage_table_rows(&scope, "path-resolution-site-records")
            .unwrap(),
        0
    );
    assert_eq!(
        store
            .count_stage_table_rows(&scope, "module-scope-binding-table")
            .unwrap(),
        19
    );
    let receipt_rows = store
        .read_stage_table_rows(&scope, "analysis-resolution-receipt-records")
        .unwrap();
    assert_eq!(receipt_rows.len(), 1);
    let receipt: Value = serde_json::from_str(&receipt_rows[0].stage_row_payload_text).unwrap();
    assert_eq!(
        receipt["analysis_product_digest_kind"],
        "blake3-ordered-row-leaves"
    );
    let proofs = receipt["analysis_product_content_proofs"]
        .as_object()
        .unwrap();
    let proved_count = proofs
        .values()
        .map(|proof| proof["row_count"].as_u64().unwrap())
        .sum::<u64>();
    assert_eq!(
        receipt["analysis_product_record_count"].as_u64(),
        Some(proved_count)
    );
    assert!(proofs.values().all(|proof| {
        proof["content_digest"]
            .as_str()
            .is_some_and(|digest| digest.len() == 64)
    }));
    assert_eq!(
        assert_receipt_product_rows(&store, &scope, &receipt),
        proved_count
    );
    assert_eq!(
        proofs["path-resolution-site-records"]["row_count"].as_u64(),
        Some(0)
    );
    assert!(Path::new(receipt["analysis_database_absolute_path"].as_str().unwrap()).is_file());
    assert!(
        store
            .read_stage_table_rows(&scope, "stage-run-generation-records")
            .unwrap()
            .iter()
            .any(|row| row.stage_row_record_key == "__marker__/name-resolution/__complete__")
    );
}

#[test]
fn test_fixture_projects_remain_unmapped() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    let manifest = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    fs::write(root.join("Cargo.toml"), format!("{manifest}\n[[example]]\nname=\"unused_lib\"\npath=\"examples/unused_lib.rs\"\ncrate-type=[\"rlib\"]\n")).unwrap();
    let library = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        format!("{library}\nmod empty; #[cfg(any())] mod dormant;"),
    )
    .unwrap();
    fs::write(root.join("src/empty.rs"), []).unwrap();
    fs::write(root.join("src/dormant.rs"), [255, 0]).unwrap();
    for path in ["tests/fixtures/project/src", "examples", "benches"] {
        fs::create_dir_all(root.join(path)).unwrap();
    }
    fs::write(
        root.join("tests/fixtures/project/Cargo.toml"),
        "[package]\nname=\"nested-fixture\"\nversion=\"0.1.0\"\n[workspace]\n",
    )
    .unwrap();
    for path in [
        "tests/fixtures/project/src/lib.rs",
        "tests/integration.rs",
        "examples/unused_lib.rs",
        "benches/unused.rs",
    ] {
        fs::write(root.join(path), [255, 0]).unwrap();
    }
    let database = directory.path().join("store.db");
    let mapped = run_workspace_mapping_fixture(&root, &database, "4");
    assert!(
        mapped.status.success(),
        "{}",
        String::from_utf8_lossy(&mapped.stderr)
    );
    let store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let scope = store.read_workspace_publication_scope().unwrap().unwrap();
    let expected: BTreeSet<_> = ["src/lib.rs", "src/child.rs", "src/empty.rs"]
        .into_iter()
        .map(|path| root.join(path).to_str().unwrap().to_owned())
        .collect();
    let requests = store
        .read_stage_table_rows(&scope, "syntax-file-work-requests")
        .unwrap();
    let actual: BTreeSet<_> = requests
        .iter()
        .map(|row| {
            let value: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
            value["file_source_absolute_path"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(requests.len(), 3);
    assert_eq!(
        store
            .count_stage_table_rows(&scope, "file-source-text-table")
            .unwrap(),
        10
    );
    store
        .visit_stage_table_rows(&scope, "span-reference-text-table", &mut |row| {
            let value: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
            assert!(expected.contains(value["file_source_absolute_path"].as_str().unwrap()));
            Ok(())
        })
        .unwrap();
    drop(store);
    assert_eq!(
        read_workspace_banner_fixture(&database)["source_tiling_identity_holds"],
        true
    );
}
/// REQ-RUST-024.10
#[test]
fn test_stage_rerun_is_identical() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "1");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let first = read_workspace_banner_fixture(&database);
    assert_eq!(first["source_file_record_count"], 4);
    assert!(first["syntax_span_record_count"].as_u64().unwrap() > 0);
    assert!(first["signature_field_record_count"].as_u64().unwrap() > 0);
    assert!(first["body_site_record_count"].as_u64().unwrap() > 0);
    assert_eq!(first["source_tiling_identity_holds"], true);
    assert_eq!(
        first["stage_implementation_status_values"]["name-resolution"],
        "complete"
    );
    let first_receipts =
        read_published_table_fixture(&database, "analysis-resolution-receipt-records");
    assert_eq!(first_receipts.len(), 1);
    let first_receipt = first_receipts[0].clone();
    let first_store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let first_scope = first_store
        .read_workspace_publication_scope()
        .unwrap()
        .unwrap();
    let first_product_count =
        assert_receipt_product_rows(&first_store, &first_scope, &first_receipt);
    assert!(first_product_count > 0);
    drop(first_store);
    let output = run_workspace_mapping_fixture(&root, &database, "4");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let second = read_workspace_banner_fixture(&database);
    assert_eq!(
        first["run_generation_digest_value"],
        second["run_generation_digest_value"]
    );
    assert_eq!(
        first["syntax_span_record_count"],
        second["syntax_span_record_count"]
    );
    let second_receipts =
        read_published_table_fixture(&database, "analysis-resolution-receipt-records");
    assert_eq!(second_receipts.len(), 1);
    let second_receipt = second_receipts[0].clone();
    let second_store =
        TursoWorkspaceStoreAdapter::open_turso_workspace_store(database.to_str().unwrap()).unwrap();
    let second_scope = second_store
        .read_workspace_publication_scope()
        .unwrap()
        .unwrap();
    let second_product_count =
        assert_receipt_product_rows(&second_store, &second_scope, &second_receipt);
    assert_eq!(first_product_count, second_product_count);
    assert_eq!(
        first_receipt["analysis_product_content_proofs"],
        second_receipt["analysis_product_content_proofs"]
    );
    drop(second_store);
    fs::write(root.join("src/child.rs"), "fn broken(").unwrap();
    assert!(
        !run_workspace_mapping_fixture(&root, &database, "4")
            .status
            .success()
    );
    let retained = read_workspace_banner_fixture(&database);
    assert_eq!(
        second["run_generation_digest_value"],
        retained["run_generation_digest_value"]
    );
}

#[test]
fn test_reason_counts_use_metadata() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(
        root.join("src/child.rs"),
        "pub fn g(x:u8)->u8 { let build_script=x; build_script }",
    )
    .unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "4");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let banner = read_workspace_banner_fixture(&database);
    assert_eq!(banner["build_script_unresolved_count"], 0);
    fs::write(root.join("src/child.rs"),"pub fn g(x:u8)->u8 { let build_script=x; let _ = include!(concat!(env!(\"OUT_DIR\"),\"/generated.rs\")); build_script }").unwrap();
    let output = run_workspace_mapping_fixture(&root, &database, "4");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let banner = read_workspace_banner_fixture(&database);
    assert_eq!(banner["build_script_unresolved_count"], 1);
    assert_eq!(
        banner["crate_unresolved_reason_counts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["unit_unresolved_reason_counts"]["build_script"]
                .as_u64()
                .unwrap())
            .sum::<u64>(),
        1
    );
}

#[test]
fn test_include_closure_digest_complete() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("workspace");
    create_workspace_source_fixture(&root);
    fs::write(
        root.join("src/child.rs"),
        "pub fn g(x:u8)->u8 { x + include!(\"value.inc\") }",
    )
    .unwrap();
    fs::write(root.join("src/value.inc"), "7").unwrap();
    let database = directory.path().join("store.db");
    let output = run_workspace_mapping_fixture(&root, &database, "4");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let first = read_workspace_banner_fixture(&database);
    assert_eq!(first["source_file_record_count"], 5);
    assert_eq!(first["source_tiling_identity_holds"], true);
    fs::write(root.join("src/value.inc"), "9").unwrap();
    let output = run_workspace_mapping_fixture(&root, &database, "4");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let second = read_workspace_banner_fixture(&database);
    assert_ne!(
        first["run_generation_digest_value"],
        second["run_generation_digest_value"]
    );
}

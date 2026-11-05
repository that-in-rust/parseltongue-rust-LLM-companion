use serde_json::{json, Value};
use std::io::Write;
use std::process::{Command, Stdio};

fn execute_source_with_engine(engine: &str, src: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tiny-reexport"))
        .args([engine, "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(src.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn compare_expected_graph_records(src: &str, spelling: &str, target: Option<&str>) {
    for engine in ["native", "model"] {
        let out = execute_source_with_engine(engine, src);
        assert!(
            out.status.success(),
            "{engine}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let actual: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(
            actual,
            json!([{"from":"make", "role":"return", "spelling":spelling, "target":target}]),
            "{engine}"
        );
    }
}

#[test]
fn resolve_real_reexport_fixture() {
    compare_expected_graph_records(
        include_str!("../fixtures/round.rs"),
        "Round",
        Some("Circle"),
    );
}

#[test]
fn normalize_raw_identifier_spelling() {
    compare_expected_graph_records(
        "pub struct Circle; pub use r#Circle as Round; pub fn make()->Round {Circle}",
        "Round",
        Some("Circle"),
    );
}

#[test]
fn follow_longest_alias_chain() {
    compare_expected_graph_records(include_str!("../fixtures/eight.rs"), "A", Some("H"));
}

#[test]
fn distinguish_invalid_from_resolved() {
    for (decls, target) in [
        ("pub struct A;", Some("A")),
        ("pub struct X; pub use X as A;", Some("X")),
        ("pub struct Y; pub use Y as A;", Some("Y")),
        ("pub use Missing as A;", None),
        ("pub use B as A; pub use A as B;", None),
        (
            "pub struct X; pub struct Y; pub use X as A; pub use Y as A;",
            None,
        ),
        ("pub struct A; pub use A as A;", None),
    ] {
        compare_expected_graph_records(
            &format!("{decls} pub fn make()->A {{loop{{}}}}"),
            "A",
            target,
        );
    }
}

#[test]
fn reject_unsupported_graph_inputs() {
    for src in [
        "pub struct A; pub mod nested {}",
        "#[cfg(unix)] pub struct A;",
        "pub fn make()->Vec<A> {loop{}}",
        "pub struct A; pub struct B; pub struct C; pub struct D; pub struct E;
         pub struct F; pub struct G; pub struct H; pub struct I;",
    ] {
        for engine in ["native", "model"] {
            let out = execute_source_with_engine(engine, src);
            assert!(!out.status.success());
            assert!(out.stdout.is_empty());
        }
    }
}

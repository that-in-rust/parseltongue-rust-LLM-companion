#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use std::{error::Error, path::Path, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest = root.join("docs/phase-20261111/parser-source-pin-manifest.json");
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(&manifest)?)?;
    let checkout = root.join(
        value["parser_source_checkout_path"]
            .as_str()
            .ok_or("source checkout missing")?,
    );
    let expected = value["parser_source_commit_hash"]
        .as_str()
        .ok_or("source pin missing")?;
    let commit = Command::new("git")
        .arg("-C")
        .arg(&checkout)
        .args(["rev-parse", "HEAD"])
        .output()?;
    let actual = std::str::from_utf8(&commit.stdout)?.trim();
    if !commit.status.success() || actual != expected {
        return Err("parser/oracle checkout does not match source manifest".into());
    }
    let status = Command::new("git")
        .arg("-C")
        .arg(&checkout)
        .args(["status", "--porcelain", "--untracked-files=no"])
        .output()?;
    if !status.status.success() || !status.stdout.is_empty() {
        return Err("parser/oracle checkout has tracked modifications".into());
    }
    let compiler = Command::new("rustc").arg("--version").output()?;
    if !compiler.status.success() {
        return Err("cannot identify parser/oracle build compiler".into());
    }
    println!("cargo:rustc-env=ORACLE_SOURCE_COMMIT_HASH={actual}");
    println!(
        "cargo:rustc-env=ORACLE_RUSTC_VERSION_TEXT={}",
        std::str::from_utf8(&compiler.stdout)?.trim()
    );
    println!("cargo:rerun-if-changed={}", manifest.display());
    println!(
        "cargo:rerun-if-changed={}",
        checkout.join(".git/HEAD").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        checkout.join("crates").display()
    );
    Ok(())
}

use std::{error::Error, path::Path, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    let oracle_source_checkout_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../rust-tools/rust-analyzer");
    let oracle_source_commit_output = Command::new("git")
        .arg("-C")
        .arg(&oracle_source_checkout_path)
        .args(["rev-parse", "HEAD"])
        .output()?;
    let oracle_source_commit_text =
        std::str::from_utf8(&oracle_source_commit_output.stdout)?.trim();
    if !oracle_source_commit_output.status.success()
        || oracle_source_commit_text != "6e37b2d62127d445d77069031b416c48c9b0b4f2"
    {
        return Err("oracle source checkout does not match SPEC pin".into());
    }
    let oracle_source_status_output = Command::new("git")
        .arg("-C")
        .arg(&oracle_source_checkout_path)
        .args(["status", "--porcelain", "--untracked-files=no"])
        .output()?;
    if !oracle_source_status_output.status.success()
        || !oracle_source_status_output.stdout.is_empty()
    {
        return Err("oracle source checkout has tracked modifications".into());
    }
    let oracle_rustc_version_output = Command::new("rustc").arg("--version").output()?;
    if !oracle_rustc_version_output.status.success() {
        return Err("cannot record oracle build toolchain".into());
    }
    println!("cargo:rustc-env=ORACLE_SOURCE_COMMIT_HASH={oracle_source_commit_text}");
    println!(
        "cargo:rustc-env=ORACLE_RUSTC_VERSION_TEXT={}",
        std::str::from_utf8(&oracle_rustc_version_output.stdout)?.trim()
    );
    println!(
        "cargo:rerun-if-changed={}",
        oracle_source_checkout_path.join(".git/HEAD").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        oracle_source_checkout_path.join("crates").display()
    );
    Ok(())
}

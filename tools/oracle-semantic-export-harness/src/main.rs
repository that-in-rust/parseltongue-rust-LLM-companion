fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oracle_cli_argument_values = std::env::args().skip(1).collect::<Vec<_>>();
    if oracle_cli_argument_values != ["--oracle-build-manifest-output"] {
        return Err("M0 supports --oracle-build-manifest-output only; semantic export is not implemented".into());
    }
    let mut oracle_source_build_manifest = oracle_semantic_export_harness::build_oracle_source_manifest();
    let oracle_executable_content_bytes = std::fs::read(std::env::current_exe()?)?;
    oracle_source_build_manifest["oracle_executable_digest_full"] =
        blake3::hash(&oracle_executable_content_bytes).to_hex().to_string().into();
    println!("{}", serde_json::to_string_pretty(&oracle_source_build_manifest)?);
    Ok(())
}

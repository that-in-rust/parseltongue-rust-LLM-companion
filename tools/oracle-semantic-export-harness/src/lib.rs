//! M0 pinned-build provenance only; semantic exports start with the resolution milestones.

pub fn build_oracle_source_manifest() -> serde_json::Value {
    let _ = (std::mem::size_of::<ide::AnalysisHost>(), std::mem::size_of::<hir::Crate>());
    todo!()
}

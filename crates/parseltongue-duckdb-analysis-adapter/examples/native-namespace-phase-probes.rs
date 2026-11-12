#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use parseltongue_core_row_types::RunCancelFlagValue;
use parseltongue_duckdb_analysis_adapter::{
    DuckdbAnalysisStoreAdapter, NAMESPACE_SOURCE_TABLE_NAMES, NamespaceAnalysisPhaseKind,
};
use parseltongue_table_store_traits::{AnalysisTableStorePort, StageTableStorePort};
use parseltongue_turso_store_adapter::TursoWorkspaceStoreAdapter;
use std::{error::Error, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let [input, database, directory, phase, rest @ ..] = arguments.as_slice() else {
        return Err(
            "expected TURSO_FILE DUCKDB_FILE RUN_DIRECTORY copy|normalization|resolution|explain|plan [SQL_FILE]".into(),
        );
    };
    let source = TursoWorkspaceStoreAdapter::open_turso_workspace_store(input)?;
    let scope = source
        .read_analysis_input_publication_scope()?
        .ok_or("no frozen native input")?;
    eprintln!("frozen_native_scope={scope:?}");
    let analysis = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
        Path::new(database),
        Path::new(directory),
        1_288_490_188,
    )?;
    let cancel = RunCancelFlagValue::default();
    let count = match phase.as_str() {
        "copy" => {
            analysis.measure_namespace_phase_work(NamespaceAnalysisPhaseKind::Copy, || {
                analysis.copy_published_stage_rows(
                    &source,
                    &scope,
                    NAMESPACE_SOURCE_TABLE_NAMES,
                    &cancel,
                )
            })?
        }
        "normalization" => analysis
            .measure_namespace_phase_work(NamespaceAnalysisPhaseKind::Normalization, || {
                analysis.normalize_frozen_namespace_rows(&scope, &cancel)
            })?,
        "resolution" => {
            analysis
                .measure_namespace_phase_work(NamespaceAnalysisPhaseKind::Resolution, || {
                    analysis.resolve_frozen_namespace_rows(&scope, &cancel)
                })?
                .fixed_point_round_count
        }
        "explain" | "plan" => {
            let query = std::fs::read_to_string(rest.first().ok_or("missing SQL file")?)?;
            let rows = analysis
                .measure_namespace_phase_work(NamespaceAnalysisPhaseKind::Resolution, || {
                    analysis.explain_namespace_query_plan(&query, &scope, phase == "explain")
                })?;
            println!("{}", rows.join("\n"));
            return Ok(());
        }
        _ => return Err("unknown namespace phase".into()),
    };
    println!("phase={phase} rows_or_rounds={count}");
    Ok(())
}

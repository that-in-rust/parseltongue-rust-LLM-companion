use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StoreBackendFailureError,
};
use parseltongue_duckdb_analysis_adapter::{
    DuckdbAnalysisStoreAdapter, OriginalSourceProjectionError,
};
use parseltongue_table_store_traits::{
    HldAnalysisPortError, HldGraphAnalysisPort, HldGraphInputStorePort, HldGraphPublicationPort,
    StageTableStorePort,
};
use std::path::Path;

pub(crate) fn publish_original_source_hld(
    scope: &StageRunScopeValue,
    workspace_root: &Path,
    store_path: &Path,
    store: &dyn StageTableStorePort,
    graph_input_store: &dyn HldGraphInputStorePort,
    graph_publication_store: &dyn HldGraphPublicationPort,
    cancel: &RunCancelFlagValue,
) -> Result<(), StoreBackendFailureError> {
    let source_snapshot = store.publish_analysis_input_snapshot(scope, cancel)?;
    let analysis_directory = store_path
        .with_extension("hld-source-analysis")
        .join(&scope.run_generation_digest_value)
        .join(&scope.stage_attempt_sequence_value);
    std::fs::create_dir_all(&analysis_directory).map_err(|error| StoreBackendFailureError {
        error_context_detail_text: error.to_string(),
    })?;
    let database = analysis_directory.join("original-source-analysis.duckdb");
    let analysis =
        DuckdbAnalysisStoreAdapter::open_source_projection_store(&database, &analysis_directory)?;
    let input = analysis
        .derive_original_top_level_containment_input(
            store,
            &source_snapshot,
            scope,
            workspace_root,
            cancel,
        )
        .map_err(map_source_projection_error)?;
    eprintln!(
        "stage=original-top-level-source-graph units={} nodes={} edges={}",
        input.input_unit_provenance_records.len(),
        input.input_node_records.len(),
        input.input_edge_records.len()
    );
    let graph_snapshot = graph_input_store
        .write_hld_graph_input(&source_snapshot, scope, &input, cancel)
        .map_err(|error| StoreBackendFailureError {
            error_context_detail_text: error.to_string(),
        })?;
    drop(input);
    let assembled = graph_input_store
        .assemble_hld_graph_input(&graph_snapshot, cancel)
        .map_err(|error| StoreBackendFailureError {
            error_context_detail_text: error.to_string(),
        })?;
    let output = analysis
        .materialize_hld_graph_output(&assembled, cancel)
        .map_err(map_analysis_error)?;
    eprintln!(
        "stage=original-top-level-hld interface_nodes={} dependency_edges={} evidence_paths={}",
        output.selected_node_records.len(),
        output.materialized_edge_records.len(),
        output.evidence_path_records.len()
    );
    graph_publication_store
        .publish_hld_graph_materialization(&graph_snapshot, scope, &output, cancel)
        .map_err(|error| StoreBackendFailureError {
            error_context_detail_text: error.to_string(),
        })?;
    Ok(())
}

fn map_source_projection_error(error: OriginalSourceProjectionError) -> StoreBackendFailureError {
    StoreBackendFailureError {
        error_context_detail_text: error.to_string(),
    }
}

fn map_analysis_error(error: HldAnalysisPortError) -> StoreBackendFailureError {
    StoreBackendFailureError {
        error_context_detail_text: error.to_string(),
    }
}

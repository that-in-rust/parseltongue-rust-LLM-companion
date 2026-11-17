use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StoreBackendFailureError,
};
use parseltongue_duckdb_analysis_adapter::DuckdbAnalysisStoreAdapter;
use parseltongue_table_store_traits::{FullGraphInputStorePort, StageTableStorePort};
use std::path::Path;

pub(crate) fn publish_full_graph_checkpoint(
    scope: &StageRunScopeValue,
    workspace_root: &Path,
    store_path: &Path,
    store: &dyn StageTableStorePort,
    full_graph_input_store: &dyn FullGraphInputStorePort,
    cancel: &RunCancelFlagValue,
) -> Result<(), StoreBackendFailureError> {
    let source_snapshot = store.publish_analysis_input_snapshot(scope, cancel)?;
    let directory = store_path
        .with_extension("full-map-source-analysis")
        .join(&scope.run_generation_digest_value)
        .join(&scope.stage_attempt_sequence_value);
    std::fs::create_dir_all(&directory).map_err(map_full_graph_error)?;
    let analysis = DuckdbAnalysisStoreAdapter::open_source_projection_store(
        &directory.join("original-item-graph-projection.duckdb"),
        &directory,
    )?;
    let mut input = analysis
        .derive_original_graph_input(store, &source_snapshot, scope, workspace_root, cancel)
        .map_err(map_full_graph_error)?;
    let calls = parseltongue_syntax_span_extractor::resolve_original_call_edges(
        store,
        &source_snapshot,
        &input,
        cancel,
    )
    .map_err(map_full_graph_error)?;
    let resolved = calls
        .iter()
        .filter(|edge| {
            matches!(
                edge.edge_resolution_value,
                parseltongue_core_row_types::HldEdgeResolutionValue::Extracted { .. }
            )
        })
        .count();
    let type_uses = parseltongue_syntax_span_extractor::resolve_signature_type_edges(
        store,
        &source_snapshot,
        &input,
        cancel,
    )
    .map_err(map_full_graph_error)?;
    let implements = parseltongue_syntax_span_extractor::resolve_impl_header_edges(
        store,
        &source_snapshot,
        &input,
        cancel,
    )
    .map_err(map_full_graph_error)?;
    let reads = parseltongue_syntax_span_extractor::resolve_body_read_edges(
        store,
        &source_snapshot,
        &input,
        cancel,
    )
    .map_err(map_full_graph_error)?;
    let imports = parseltongue_syntax_span_extractor::resolve_use_import_edges(
        store,
        &source_snapshot,
        &input,
        cancel,
    )
    .map_err(map_full_graph_error)?;
    let call_count = calls.len();
    let (type_use_count, implements_count) =
        (type_uses.len(), implements.implements_edge_records.len());
    let read_count = reads.read_edge_input_records.len();
    let import_count = imports.import_edge_input_records.len();
    let mut semantic = type_uses;
    semantic.extend(implements.implements_edge_records);
    semantic.extend(reads.read_edge_input_records);
    semantic.extend(imports.import_edge_input_records);
    input
        .input_node_records
        .extend(parseltongue_syntax_span_extractor::collect_builtin_endpoint_nodes(&semantic));
    // REQ-RUST-053.0 N1: one node per named external target, shared by every relation.
    semantic.extend(calls);
    input
        .input_node_records
        .extend(parseltongue_syntax_span_extractor::collect_external_endpoint_nodes(&semantic));
    input.input_node_records.sort();
    let count_structural_relation_rows = |relation| {
        input
            .input_edge_records
            .iter()
            .filter(|edge| edge.edge_relation_kind_value == relation)
            .count()
    };
    eprintln!(
        "stage=full-item-dependency-graph nodes={} containment={} generates={} expands={} calls={} resolved_calls={} type_uses={} implements={} implements_without_source={} reads={} reads_in_expanded_bodies={} imports={} imports_glob_skipped={} imports_in_expanded_items={} imports_without_source={} completeness=incomplete",
        input.input_node_records.len(),
        count_structural_relation_rows(parseltongue_core_row_types::HldRelationKindValue::Contains),
        count_structural_relation_rows(
            parseltongue_core_row_types::HldRelationKindValue::Generates
        ),
        count_structural_relation_rows(parseltongue_core_row_types::HldRelationKindValue::Expands),
        call_count,
        resolved,
        type_use_count,
        implements_count,
        implements.headers_without_source,
        read_count,
        reads.read_expanded_site_count,
        import_count,
        imports.imports_glob_skipped,
        imports.imports_in_expanded_items,
        imports.imports_without_source
    );
    input.input_edge_records.extend(semantic);
    input.input_edge_records.sort();
    input.input_edge_records.dedup();
    // REQ-RUST-069.0: effective visibility and reachability need the finished `contains` rows.
    input.input_node_records = parseltongue_syntax_span_extractor::resolve_node_visibility_fields(
        store,
        &source_snapshot,
        &input,
        cancel,
    )
    .map_err(map_full_graph_error)?;
    input.input_node_records.sort();
    let graph_snapshot = full_graph_input_store
        .write_full_graph_input(&source_snapshot, scope, &input, cancel)
        .map_err(map_full_graph_error)?;
    let mut assembled = full_graph_input_store
        .assemble_full_graph_input(&graph_snapshot, cancel)
        .map_err(map_full_graph_error)?;
    assembled.input_node_records.sort();
    assembled.input_edge_records.sort();
    if assembled != input {
        return Err(StoreBackendFailureError {
            error_context_detail_text:
                "frozen full-map input differs from its source-derived bundle".into(),
        });
    }
    eprintln!(
        "stage=frozen-full-map-readback attempt={} nodes={} edges={} completeness=incomplete",
        graph_snapshot.stage_attempt_sequence_value,
        assembled.input_node_records.len(),
        assembled.input_edge_records.len()
    );
    Ok(())
}

fn map_full_graph_error(error: impl std::fmt::Display) -> StoreBackendFailureError {
    StoreBackendFailureError {
        error_context_detail_text: error.to_string(),
    }
}

use super::{DuckdbAnalysisStoreAdapter, build_analysis_error};
use parseltongue_core_row_types::{
    NameResolutionStageError, NameResolutionStuckError, RunCancelFlagValue, StageRunScopeValue,
    StoreBackendFailureError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceResolutionRoundReport {
    pub binding_set_digest_value: String,
    pub fixed_point_round_count: usize,
}

impl DuckdbAnalysisStoreAdapter {
    pub fn resolve_frozen_namespace_rows(
        &self,
        scope: &StageRunScopeValue,
        cancel: &RunCancelFlagValue,
    ) -> Result<NamespaceResolutionRoundReport, StoreBackendFailureError> {
        if cancel.check_stage_work_cancelled() {
            return Err(build_analysis_error("namespace resolution cancelled"));
        }
        let connection = self._connection.lock().map_err(build_analysis_error)?;
        let count:usize=connection.query_row("SELECT count(*) FROM \"namespace-normalized-source-manifests\" WHERE generation_key=? AND attempt_key=?",duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value],|row|row.get(0)).map_err(build_analysis_error)?;
        if count != 1 {
            return Err(build_analysis_error(
                "normalized namespace source is incomplete",
            ));
        }
        eprintln!("stage=namespace-resolution-step step=unit-links event=started");
        super::namespace_unit_link_inputs::link_namespace_unit_rows(&connection, scope, cancel)?;
        eprintln!("stage=namespace-resolution-step step=unit-links event=complete");
        let report = resolve_namespace_sql_rounds(&connection, scope, cancel)
            .map_err(build_analysis_error)?;
        connection
            .execute(
                "INSERT OR REPLACE INTO \"namespace-fixed-point-manifests\" VALUES (?,?,?,?)",
                duckdb::params![
                    scope.run_generation_digest_value,
                    scope.stage_attempt_sequence_value,
                    report.binding_set_digest_value,
                    report.fixed_point_round_count as u64
                ],
            )
            .map_err(build_analysis_error)?;
        Ok(report)
    }
}

pub(super) fn resolve_namespace_sql_rounds(
    connection: &duckdb::Connection,
    scope: &StageRunScopeValue,
    cancel: &RunCancelFlagValue,
) -> Result<NamespaceResolutionRoundReport, NameResolutionStageError> {
    use super::namespace_binding_round_strata::recompute_binding_rows;
    use super::namespace_contract_input_codec::error;
    eprintln!("stage=namespace-resolution-step step=scope-relations event=started");
    super::materialize_scope_relations(connection, scope).map_err(error)?;
    eprintln!("stage=namespace-resolution-step step=scope-relations event=complete");
    [("namespace-active-import-requests","namespace-import-request-records"),("namespace-active-declaration-records","namespace-binding-declaration-records"),("namespace-active-declaration-visibility","namespace-declaration-visibility-records")].into_iter().try_for_each(|(target,source)|{
        connection.execute(&format!("CREATE OR REPLACE TEMP TABLE \"{target}\" AS SELECT * FROM \"{source}\" WHERE generation_key=? AND attempt_key=?"),duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value]).map(|_|()).map_err(error)
    })?;
    connection.execute_batch("CREATE OR REPLACE TEMP TABLE \"namespace-binding-round-inputs\" AS SELECT * FROM \"namespace-active-declaration-records\"; CREATE OR REPLACE TEMP TABLE \"namespace-binding-visibility-records\" AS SELECT * FROM \"namespace-active-declaration-visibility\";").map_err(error)?;
    let mut previous = recompute_binding_rows(connection, scope).map_err(error)?;
    for round in 1..=super::FIXED_POINT_LIMIT + 1 {
        if cancel.check_stage_work_cancelled() {
            return Err(NameResolutionStageError::Cancelled);
        }
        eprintln!(
            "stage=namespace-resolution-step round={round} step=path-candidates event=started"
        );
        super::recompute_path_candidates(connection, scope).map_err(error)?;
        eprintln!(
            "stage=namespace-resolution-step round={round} step=import-bindings event=started"
        );
        connection
            .execute_batch(include_str!("namespace-import-binding-rounds.sql"))
            .map_err(error)?;
        let digest = recompute_binding_rows(connection, scope).map_err(error)?;
        eprintln!(
            "stage=namespace-resolution-step round={round} step=binding-digest event=complete"
        );
        if previous == digest {
            return Ok(NamespaceResolutionRoundReport {
                binding_set_digest_value: digest,
                fixed_point_round_count: round,
            });
        }
        previous = digest;
    }
    Err(NameResolutionStageError::Stuck(NameResolutionStuckError {
        error_context_detail_text: "import fixed point exceeded 8192 rounds".into(),
    }))
}

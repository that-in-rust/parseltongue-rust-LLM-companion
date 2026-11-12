use super::{
    FIXED_POINT_LIMIT, namespace_binding_round_strata::recompute_binding_rows,
    namespace_contract_input_codec::*,
};
use parseltongue_core_row_types::*;

pub(super) fn resolve_contract_input(
    input: &NameResolutionInputRows,
    cancel: &RunCancelFlagValue,
) -> Result<NameResolutionOutputRows, NameResolutionStageError> {
    if cancel.check_stage_work_cancelled() {
        return Err(NameResolutionStageError::Cancelled);
    }
    let config = duckdb::Config::default()
        .enable_external_access(false)
        .and_then(|config| config.with("autoinstall_known_extensions", "false"))
        .and_then(|config| config.with("autoload_known_extensions", "false"))
        .and_then(|config| config.max_memory("1288490188B"))
        .map_err(error)?;
    let connection = duckdb::Connection::open_in_memory_with_flags(config).map_err(error)?;
    let codec = ContractInputCodec::load(&connection, input)?;
    let scope = StageRunScopeValue {
        run_generation_digest_value: "contract".into(),
        stage_attempt_sequence_value: "0".into(),
    };
    super::materialize_scope_relations(&connection, &scope).map_err(error)?;
    connection.execute_batch("CREATE TABLE \"namespace-binding-round-inputs\" AS SELECT * FROM \"namespace-binding-declaration-records\"").map_err(error)?;
    let mut previous = recompute_binding_rows(&connection, &scope).map_err(error)?;
    let mut settled = false;
    for _ in 0..=FIXED_POINT_LIMIT {
        if cancel.check_stage_work_cancelled() {
            return Err(NameResolutionStageError::Cancelled);
        }
        super::recompute_path_candidates(&connection, &scope).map_err(error)?;
        connection.execute_batch("DELETE FROM \"namespace-binding-round-inputs\";
            INSERT INTO \"namespace-binding-round-inputs\"
            SELECT * FROM \"namespace-binding-declaration-records\"
            UNION ALL SELECT i.generation_key,i.attempt_key,i.importer_scope,coalesce(i.alias_name,c.binding_name),c.namespace,c.target_key,3,i.request_key
            FROM \"namespace-import-request-records\" i JOIN \"namespace-path-candidate-records\" c
                ON c.generation_key=i.generation_key AND c.attempt_key=i.attempt_key AND c.request_key=i.request_key
            WHERE NOT i.is_glob AND c.state='resolved' AND coalesce(i.alias_name,'')<>'_'
            UNION ALL SELECT i.generation_key,i.attempt_key,i.importer_scope,b.name,b.namespace,b.target_key,2,i.request_key
            FROM \"namespace-import-request-records\" i JOIN \"namespace-path-candidate-records\" c
                ON c.generation_key=i.generation_key AND c.attempt_key=i.attempt_key AND c.request_key=i.request_key AND c.state='resolved'
            JOIN \"namespace-target-context-records\" t ON t.generation_key=c.generation_key AND t.attempt_key=c.attempt_key AND t.target_key=c.target_key
            JOIN \"namespace-binding-round-results\" b ON b.generation_key=t.generation_key AND b.attempt_key=t.attempt_key AND b.scope_key=t.child_scope
            JOIN \"namespace-target-context-records\" v ON v.generation_key=b.generation_key AND v.attempt_key=b.attempt_key AND v.target_key=b.target_key
            WHERE i.is_glob AND (v.visibility_scope IS NULL OR EXISTS (SELECT 1 FROM \"namespace-scope-ancestor-records\" a
                WHERE a.generation_key=i.generation_key AND a.attempt_key=i.attempt_key AND a.scope_key=i.importer_scope AND a.ancestor_scope=v.visibility_scope));")
            .map_err(error)?;
        let digest = recompute_binding_rows(&connection, &scope).map_err(error)?;
        if previous == digest {
            settled = true;
            break;
        }
        previous = digest;
    }
    if !settled {
        return Err(NameResolutionStageError::Stuck(NameResolutionStuckError {
            error_context_detail_text: "import fixed point exceeded 8192 rounds".into(),
        }));
    }
    let mut statement=connection.prepare("SELECT scope_key,name,namespace,target_key,priority,evidence_key FROM \"namespace-binding-round-results\"
        WHERE state='resolved' ORDER BY generation_key,attempt_key,scope_key,name,namespace,target_key,evidence_key").map_err(error)?;
    let mut rows = statement.query([]).map_err(error)?;
    let mut bindings = Vec::new();
    while let Some(row) = rows.next().map_err(error)? {
        let binding = codec.binding(
            row.get(0).map_err(error)?,
            row.get(1).map_err(error)?,
            row.get(2).map_err(error)?,
            row.get(3).map_err(error)?,
            row.get(4).map_err(error)?,
            row.get(5).map_err(error)?,
        )?;
        if binding.binding_lexical_scope_key.scope_compilation_unit_key
            == input.unit_compilation_identity_key
        {
            bindings.push(binding);
        }
    }
    bindings.sort();
    bindings.dedup();
    let mut paths=input.name_signature_path_rows.iter().map(|path| {
        let mut statement=connection.prepare("SELECT c.binding_scope,c.binding_name,c.namespace,c.target_key,b.priority,b.evidence_key
            FROM \"namespace-path-candidate-records\" c JOIN \"namespace-binding-round-results\" b ON b.generation_key=c.generation_key AND b.attempt_key=c.attempt_key
                AND b.scope_key=c.binding_scope AND b.name=c.binding_name AND b.namespace=c.namespace AND b.target_key=c.target_key
            WHERE c.request_key=? ORDER BY c.generation_key,c.attempt_key,c.request_key,c.target_key,c.namespace,c.binding_scope,c.binding_name,b.evidence_key").map_err(error)?;
        let mut rows=statement.query([key(&path.path_site_occurrence_key)]).map_err(error)?;
        let mut candidates=Vec::new();
        while let Some(row)=rows.next().map_err(error)? { candidates.push(codec.binding(row.get(0).map_err(error)?,row.get(1).map_err(error)?,row.get(2).map_err(error)?,row.get(3).map_err(error)?,row.get(4).map_err(error)?,row.get(5).map_err(error)?)?); }
        candidates.sort();candidates.dedup();
        let ambiguous=candidates.windows(2).any(|pair|pair[0].binding_namespace_kind_value==pair[1].binding_namespace_kind_value && pair[0].binding_target_identity_key!=pair[1].binding_target_identity_key);
        let state=if candidates.is_empty() {NamePathAnswerKind::Unresolved} else if ambiguous {NamePathAnswerKind::Ambiguous} else {NamePathAnswerKind::Resolved};
        Ok(NamePathAnswerRow {path_site_occurrence_key:path.path_site_occurrence_key.clone(),path_resolution_state_kind:state,
            path_resolved_target_rows:if state==NamePathAnswerKind::Resolved {candidates.clone()} else {vec![]},path_candidate_target_rows:candidates,
            path_unresolved_reason_kind:if state==NamePathAnswerKind::Resolved {None} else {Some(path.path_unavailable_reason_kind.unwrap_or(SiteUnresolvedReasonKind::ExternalSourceMissing))}})
    }).collect::<Result<Vec<_>,NameResolutionStageError>>()?;
    paths.sort_by(|left, right| {
        left.path_site_occurrence_key
            .cmp(&right.path_site_occurrence_key)
    });
    let mut scopes = input
        .name_module_scope_rows
        .iter()
        .filter(|scope| {
            scope.scope_module_identity_key.scope_compilation_unit_key
                == input.unit_compilation_identity_key
        })
        .cloned()
        .collect::<Vec<_>>();
    scopes.sort();
    scopes.dedup();
    Ok(NameResolutionOutputRows {
        resolved_module_scope_rows: scopes,
        resolved_symbol_binding_rows: bindings,
        resolved_signature_path_rows: paths,
        resolved_completed_invocation_keys: vec![],
        resolved_unavailable_reason_rows: vec![],
        resolved_semantic_output_rows: vec![],
        resolved_changing_macro_rounds: 0,
    })
}

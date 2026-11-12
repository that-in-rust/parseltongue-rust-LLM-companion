use super::namespace_frozen_identity_codec::{
    occurrence_key, path_parts, scope_key, target_key, unit_key,
};
use super::{DuckdbAnalysisStoreAdapter, build_analysis_error};
use parseltongue_core_row_types::*;
use parseltongue_table_store_traits::ResolvedMacroLookupPort;
use serde_json::{Value, json};
use std::sync::Mutex;

pub struct FrozenNamespaceMacroLookup<'a> {
    store: &'a DuckdbAnalysisStoreAdapter,
    scope: Mutex<StageRunScopeValue>,
}

impl DuckdbAnalysisStoreAdapter {
    pub fn frozen_namespace_macro_lookup(
        &self,
        scope: StageRunScopeValue,
    ) -> FrozenNamespaceMacroLookup<'_> {
        FrozenNamespaceMacroLookup {
            store: self,
            scope: Mutex::new(scope),
        }
    }
}

impl FrozenNamespaceMacroLookup<'_> {
    pub fn replace_frozen_lookup_scope(
        &self,
        scope: StageRunScopeValue,
    ) -> Result<(), StoreBackendFailureError> {
        *self.scope.lock().map_err(build_analysis_error)? = scope;
        Ok(())
    }
}

fn failure(message: impl std::fmt::Display) -> NameResolutionStageError {
    NameResolutionStageError::Input(StageInputMissingError {
        error_context_detail_text: message.to_string(),
    })
}

impl ResolvedMacroLookupPort for FrozenNamespaceMacroLookup<'_> {
    fn read_macro_invocation_binding_record(
        &self,
        request: &str,
    ) -> Result<Option<StageTableRowRecord>, NameResolutionStageError> {
        let scope = self.scope.lock().map_err(failure)?.clone();
        let connection = self.store._connection.lock().map_err(failure)?;
        let mut statement = connection
            .prepare(include_str!("namespace-macro-candidate-lookup.sql"))
            .map_err(failure)?;
        let mut rows = statement
            .query(duckdb::params![
                scope.run_generation_digest_value,
                scope.stage_attempt_sequence_value,
                request
            ])
            .map_err(failure)?;
        rows.next().map_err(failure)?.map(|row|{
            let lexical:String=row.get(0).map_err(failure)?;
            let name:String=row.get(1).map_err(failure)?;
            let target:String=row.get(2).map_err(failure)?;
            let child:Option<String>=row.get(3).map_err(failure)?;
            let priority:i32=row.get(4).map_err(failure)?;
            let visibility:Option<String>=row.get(5).map_err(failure)?;
            let parse=|text:&str|serde_json::from_str::<Value>(text).map_err(failure);
            let priority=match priority{0=>"builtin",1=>"prelude",2=>"glob",3=>"explicit",4=>"declaration",_=>return Err(failure("invalid macro priority"))};
            let visibility=match visibility.as_deref(){None=>json!({"visibility_scope_kind_value":"public"}),Some("invisible")=>json!({"visibility_scope_kind_value":"invisible"}),Some(value)=>json!({"visibility_scope_kind_value":"restricted","visibility_restricted_scope_key":parse(value)?})};
            Ok(StageTableRowRecord{stage_owned_table_name:"namespace-macro-binding-records".into(),stage_row_record_key:request.into(),stage_row_payload_text:json!({"binding_lexical_scope_key":parse(&lexical)?,"binding_resolved_name_text":name,"binding_namespace_kind_value":"macro","binding_target_identity_key":parse(&target)?,"binding_target_child_scope":child.as_deref().map(parse).transpose()?,"binding_visibility_scope_kind":visibility,"binding_selection_priority_kind":priority,"binding_import_evidence_keys":[]}).to_string()})
        }).transpose()
    }
    fn read_resolved_builtin_macro(
        &self,
        lexical: &NameResolutionScopeKey,
        path: &[NamePathSegmentKind],
    ) -> Result<Option<String>, NameResolutionStageError> {
        self.read_builtin_probe(None, lexical, path)
    }
    fn read_resolved_builtin_macro_at(
        &self,
        invocation: &SemanticOccurrenceKeyValue,
        lexical: &NameResolutionScopeKey,
        path: &[NamePathSegmentKind],
    ) -> Result<Option<String>, NameResolutionStageError> {
        self.read_builtin_probe(Some(&occurrence_key(invocation).to_string()), lexical, path)
    }
    fn read_macro_target_metadata(
        &self,
        target: &NameSymbolTargetKey,
    ) -> Result<Option<StageTableRowRecord>, NameResolutionStageError> {
        let scope = self.scope.lock().map_err(failure)?.clone();
        let connection = self.store._connection.lock().map_err(failure)?;
        read_metadata(&connection, &scope, &target_key(target).to_string())
    }
}

impl FrozenNamespaceMacroLookup<'_> {
    fn read_builtin_probe(
        &self,
        invocation: Option<&str>,
        lexical: &NameResolutionScopeKey,
        path: &[NamePathSegmentKind],
    ) -> Result<Option<String>, NameResolutionStageError> {
        if path.is_empty() {
            return Ok(None);
        }
        let scope = self.scope.lock().map_err(failure)?.clone();
        let connection = self.store._connection.lock().map_err(failure)?;
        let transaction = connection.unchecked_transaction().map_err(failure)?;
        transaction.execute_batch("CREATE OR REPLACE TEMP TABLE \"namespace-builtin-probe-requests\" AS SELECT * FROM \"namespace-path-source-requests\" LIMIT 0;").map_err(failure)?;
        let request = "namespace-builtin-probe-request";
        if let Some(invocation) = invocation {
            transaction.execute("INSERT INTO \"namespace-latest-legacy-lookups\" SELECT generation_key,attempt_key,lookup_key || '/builtin-probe',?,name,target_key,revision,source_order FROM \"namespace-latest-legacy-lookups\" WHERE generation_key=? AND attempt_key=? AND request_key=?",duckdb::params![request,scope.run_generation_digest_value,scope.stage_attempt_sequence_value,invocation]).map_err(failure)?;
        }
        transaction.execute("INSERT INTO \"namespace-builtin-probe-requests\" SELECT generation_key,attempt_key,?, ?,root_scope,extern_scope,edition,'signature','macro',?,NULL FROM \"namespace-unit-source-records\" WHERE generation_key=? AND attempt_key=? AND unit_key=?",duckdb::params![request,scope_key(lexical).to_string(),path.len() as u64,scope.run_generation_digest_value,scope.stage_attempt_sequence_value,unit_key(&lexical.scope_compilation_unit_key).to_string()]).map_err(failure)?;
        path_parts(path)
            .into_iter()
            .enumerate()
            .try_for_each(|(position, (kind, text))| {
                transaction
                    .execute(
                        "INSERT INTO \"namespace-path-segment-records\" VALUES (?,?,?,?,?,?)",
                        duckdb::params![
                            scope.run_generation_digest_value,
                            scope.stage_attempt_sequence_value,
                            request,
                            position as u64,
                            kind,
                            text
                        ],
                    )
                    .map(|_| ())
                    .map_err(failure)
            })?;
        let sql = include_str!("namespace-path-candidate-derivation.sql").replace(
            "\"namespace-active-path-requests\"",
            "\"namespace-builtin-probe-requests\"",
        );
        let target = {
            let mut statement=transaction.prepare(&format!("SELECT DISTINCT target_key FROM ({sql}) WHERE state='resolved' AND namespace='macro' ORDER BY target_key LIMIT 2")).map_err(failure)?;
            let values = statement
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(failure)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(failure)?;
            if values.len() == 1 {
                values.into_iter().next()
            } else {
                None
            }
        };
        let name = target
            .map(|target| read_metadata(&transaction, &scope, &target))
            .transpose()?
            .flatten()
            .map(|record| {
                serde_json::from_str::<Value>(&record.stage_row_payload_text)
                    .map(|value| {
                        value["target_builtin_name_value"]
                            .as_str()
                            .map(str::to_owned)
                    })
                    .map_err(failure)
            })
            .transpose()?
            .flatten();
        transaction.execute("DELETE FROM \"namespace-path-segment-records\" WHERE generation_key=? AND attempt_key=? AND request_key=?",duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value,request]).map_err(failure)?;
        transaction.execute("DELETE FROM \"namespace-latest-legacy-lookups\" WHERE generation_key=? AND attempt_key=? AND request_key=?",duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value,request]).map_err(failure)?;
        transaction.commit().map_err(failure)?;
        Ok(name)
    }
}

fn read_metadata(
    connection: &duckdb::Connection,
    scope: &StageRunScopeValue,
    target: &str,
) -> Result<Option<StageTableRowRecord>, NameResolutionStageError> {
    let mut statement=connection.prepare("SELECT DISTINCT generation_key,attempt_key,target_key,metadata FROM \"namespace-target-source-records\" WHERE generation_key=? AND attempt_key=? AND target_key=? ORDER BY generation_key,attempt_key,target_key,metadata LIMIT 2").map_err(failure)?;
    let mut rows = statement
        .query(duckdb::params![
            scope.run_generation_digest_value,
            scope.stage_attempt_sequence_value,
            target
        ])
        .map_err(failure)?;
    let record = rows
        .next()
        .map_err(failure)?
        .map(|row| row.get::<_, String>(3))
        .transpose()
        .map_err(failure)?;
    if rows.next().map_err(failure)?.is_some() {
        return Err(failure("conflicting macro target metadata"));
    }
    Ok(record.map(|payload| StageTableRowRecord {
        stage_owned_table_name: "namespace-macro-metadata-records".into(),
        stage_row_record_key: target.into(),
        stage_row_payload_text: payload,
    }))
}

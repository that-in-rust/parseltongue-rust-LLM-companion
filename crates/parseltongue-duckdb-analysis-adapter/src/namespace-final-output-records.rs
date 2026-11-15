use super::{DuckdbAnalysisStoreAdapter, build_analysis_error};
use parseltongue_core_row_types::{
    StageRunScopeValue, StageTableRowRecord, StoreBackendFailureError,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn parse(value: &str) -> Result<Value, StoreBackendFailureError> {
    serde_json::from_str(value).map_err(build_analysis_error)
}

fn visibility(value: Option<String>) -> Result<Value, StoreBackendFailureError> {
    match value.as_deref() {
        None => Ok(json!({"visibility_scope_kind_value":"public"})),
        Some("invisible") => Ok(json!({"visibility_scope_kind_value":"invisible"})),
        Some(value) => Ok(
            json!({"visibility_scope_kind_value":"restricted","visibility_restricted_scope_key":parse(value)?}),
        ),
    }
}

fn priority(value: i32) -> Result<&'static str, StoreBackendFailureError> {
    match value {
        0 => Ok("builtin"),
        1 => Ok("prelude"),
        2 => Ok("glob"),
        3 => Ok("explicit"),
        4 => Ok("declaration"),
        _ => Err(build_analysis_error("invalid output priority")),
    }
}

type TargetNamespaceGroups = BTreeMap<String, (Value, BTreeSet<String>)>;

#[derive(Default)]
struct PathNamespaceProduct {
    key: String,
    /// The path metadata of the request; absent for a request made only by an import.
    value: Option<Value>,
    resolved: TargetNamespaceGroups,
    candidates: TargetNamespaceGroups,
    ambiguous: bool,
    reason: Option<String>,
    /// REQ-RUST-038.0: set when the path starts at a sysroot crate with no unit in this generation.
    absent_sysroot_reason: Option<String>,
    /// REQ-RUST-049.0 R1: the request key is an import request; its outcome is also an import row.
    import: Option<Value>,
}

fn grouped_targets(groups: TargetNamespaceGroups) -> Vec<Value> {
    groups
        .into_values()
        .map(|(mut value, namespaces)| {
            value["target_namespace_kind_values"] = json!(namespaces);
            value
        })
        .collect()
}

/// One outcome row: the base value plus the four outcome fields under one field prefix.
fn build_outcome_row_value(
    mut value: Value,
    prefix: &str,
    outcome: &(&str, Vec<Value>, Vec<Value>, Option<String>),
) -> Value {
    value[format!("{prefix}_resolution_state_kind")] = json!(outcome.0);
    value[format!("{prefix}_target_symbol_rows")] = json!(outcome.1);
    value[format!("{prefix}_candidate_target_rows")] = json!(outcome.2);
    value[format!("{prefix}_unresolved_reason_kind")] = json!(outcome.3);
    value
}

impl PathNamespaceProduct {
    /// The rows of this request: its path row when it has path metadata, and its import row
    /// when the request key is an import request (REQ-RUST-049.0 R1).
    fn finish(self) -> Vec<(&'static str, Value)> {
        let reason = self.reason.or_else(|| {
            self.absent_sysroot_reason
                .filter(|_| !self.ambiguous && self.resolved.is_empty())
        });
        let state = if self.ambiguous {
            "ambiguous"
        } else if self.resolved.is_empty() {
            "unresolved"
        } else {
            "resolved"
        };
        let outcome = (
            state,
            grouped_targets(self.resolved),
            grouped_targets(self.candidates),
            reason,
        );
        let path = self.value.map(|value| {
            (
                "path-resolution-site-records",
                build_outcome_row_value(value, "path", &outcome),
            )
        });
        let import = self.import.map(|site| {
            (
                "import-resolution-site-records",
                build_outcome_row_value(
                    json!({ "import_site_occurrence_key": site }),
                    "import",
                    &outcome,
                ),
            )
        });
        path.into_iter().chain(import).collect()
    }
}

impl DuckdbAnalysisStoreAdapter {
    pub fn visit_frozen_namespace_products(
        &self,
        scope: &StageRunScopeValue,
        visit: &mut dyn FnMut(&StageTableRowRecord) -> Result<(), StoreBackendFailureError>,
    ) -> Result<usize, StoreBackendFailureError> {
        let connection = self._connection.lock().map_err(build_analysis_error)?;
        let complete:usize=connection.query_row("SELECT count(*) FROM \"namespace-fixed-point-manifests\" WHERE generation_key=? AND attempt_key=?",duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value],|row|row.get(0)).map_err(build_analysis_error)?;
        if complete != 1 {
            return Err(build_analysis_error("namespace source is incomplete"));
        }
        connection.execute_batch("CREATE OR REPLACE TEMP TABLE \"namespace-mapped-output-contexts\" (generation_key VARCHAR,attempt_key VARCHAR,unit_key VARCHAR);").map_err(build_analysis_error)?;
        {
            let mut statement=connection.prepare("SELECT unit_key,source_record FROM \"namespace-unit-source-records\" WHERE generation_key=? AND attempt_key=? ORDER BY generation_key,attempt_key,unit_key,source_record").map_err(build_analysis_error)?;
            let mut rows = statement
                .query(duckdb::params![
                    scope.run_generation_digest_value,
                    scope.stage_attempt_sequence_value
                ])
                .map_err(build_analysis_error)?;
            while let Some(row) = rows.next().map_err(build_analysis_error)? {
                let source: String = row.get(1).map_err(build_analysis_error)?;
                if parse(&source)?[parseltongue_core_row_types::MAPPED_TARGET_FLAG_FIELD] == true {
                    connection
                        .execute(
                            "INSERT INTO \"namespace-mapped-output-contexts\" VALUES (?,?,?)",
                            duckdb::params![
                                scope.run_generation_digest_value,
                                scope.stage_attempt_sequence_value,
                                row.get::<_, String>(0).map_err(build_analysis_error)?
                            ],
                        )
                        .map_err(build_analysis_error)?;
                }
            }
        }
        connection.execute("CREATE OR REPLACE TEMP TABLE \"namespace-output-target-metadata\" AS SELECT DISTINCT generation_key,attempt_key,target_key,metadata FROM \"namespace-target-source-records\" WHERE generation_key=? AND attempt_key=?",duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value]).map_err(build_analysis_error)?;
        let conflicts:usize=connection.query_row("SELECT count(*) FROM (SELECT target_key FROM \"namespace-output-target-metadata\" GROUP BY target_key HAVING count(*)>1)",[],|row|row.get(0)).map_err(build_analysis_error)?;
        if conflicts > 0 {
            return Err(build_analysis_error("conflicting final target metadata"));
        }
        let mut count = 0;
        let mut emit = |table: &str, value: Value| {
            let payload = value.to_string();
            visit(&StageTableRowRecord {
                stage_owned_table_name: table.into(),
                stage_row_record_key: blake3::hash(payload.as_bytes()).to_hex().to_string(),
                stage_row_payload_text: payload,
            })?;
            count += 1;
            Ok::<_, StoreBackendFailureError>(())
        };
        {
            let mut statement=connection.prepare("SELECT DISTINCT s.scope_key,s.parent_scope,s.parent_lookup,s.module_boundary,s.prelude_scope FROM \"namespace-scope-context-records\" s JOIN \"namespace-mapped-output-contexts\" u USING(generation_key,attempt_key,unit_key) ORDER BY s.scope_key,s.parent_scope NULLS FIRST,s.parent_lookup,s.module_boundary,s.prelude_scope NULLS FIRST").map_err(build_analysis_error)?;
            let mut rows = statement.query([]).map_err(build_analysis_error)?;
            while let Some(row) = rows.next().map_err(build_analysis_error)? {
                emit(
                    "name-resolution-scope-records",
                    json!({"scope_module_identity_key":parse(&row.get::<_,String>(0).map_err(build_analysis_error)?)?,"scope_parent_identity_key":row.get::<_,Option<String>>(1).map_err(build_analysis_error)?.as_deref().map(parse).transpose()?,"scope_parent_lookup_flag":row.get::<_,bool>(2).map_err(build_analysis_error)?,"scope_module_boundary_flag":row.get::<_,bool>(3).map_err(build_analysis_error)?,"scope_implicit_prelude_key":row.get::<_,Option<String>>(4).map_err(build_analysis_error)?.as_deref().map(parse).transpose()?}),
                )?;
            }
        }
        {
            let mut statement = connection
                .prepare(include_str!("namespace-final-binding-products.sql"))
                .map_err(build_analysis_error)?;
            let mut rows = statement.query([]).map_err(build_analysis_error)?;
            while let Some(row) = rows.next().map_err(build_analysis_error)? {
                let evidence = match row
                    .get::<_, duckdb::types::Value>(7)
                    .map_err(build_analysis_error)?
                {
                    duckdb::types::Value::List(values) => values
                        .into_iter()
                        .map(|value| match value {
                            duckdb::types::Value::Text(text) => Ok(text),
                            _ => Err(build_analysis_error("invalid output evidence")),
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    _ => return Err(build_analysis_error("output evidence list absent")),
                };
                let typed = evidence
                    .iter()
                    .filter_map(|text| serde_json::from_str::<Value>(text).ok())
                    .filter(|value| value["span_source_file_path"].is_string())
                    .collect::<Vec<_>>();
                emit(
                    "module-scope-binding-table",
                    json!({"binding_lexical_scope_key":parse(&row.get::<_,String>(0).map_err(build_analysis_error)?)?,"binding_resolved_name_text":row.get::<_,String>(1).map_err(build_analysis_error)?,"binding_namespace_kind_value":row.get::<_,String>(2).map_err(build_analysis_error)?,"binding_target_identity_key":parse(&row.get::<_,String>(3).map_err(build_analysis_error)?)?,"binding_target_child_scope":row.get::<_,Option<String>>(4).map_err(build_analysis_error)?.as_deref().map(parse).transpose()?,"binding_selection_priority_kind":priority(row.get(5).map_err(build_analysis_error)?)?,"binding_visibility_scope_kind":visibility(row.get(6).map_err(build_analysis_error)?)?,"binding_import_evidence_keys":typed,"binding_resolution_evidence_values":evidence,"binding_target_metadata_value":parse(&row.get::<_,String>(8).map_err(build_analysis_error)?)?}),
                )?;
            }
        }
        {
            let mut statement = connection
                .prepare(include_str!("namespace-final-path-products.sql"))
                .map_err(build_analysis_error)?;
            let mut rows = statement.query([]).map_err(build_analysis_error)?;
            let mut current = PathNamespaceProduct::default();
            while let Some(row) = rows.next().map_err(build_analysis_error)? {
                let key: String = row.get(0).map_err(build_analysis_error)?;
                if key != current.key {
                    if !current.key.is_empty() {
                        for (table, value) in std::mem::take(&mut current).finish() {
                            emit(table, value)?;
                        }
                    }
                    let import = row
                        .get::<_, bool>(7)
                        .map_err(build_analysis_error)?
                        .then(|| parse(&key))
                        .transpose()?;
                    let value = row
                        .get::<_, Option<String>>(1)
                        .map_err(build_analysis_error)?
                        .as_deref()
                        .map(parse)
                        .transpose()?;
                    if value.is_none() && import.is_none() {
                        return Err(build_analysis_error("path metadata absent"));
                    }
                    current = PathNamespaceProduct {
                        key,
                        value,
                        reason: row.get(2).map_err(build_analysis_error)?,
                        absent_sysroot_reason: row.get(6).map_err(build_analysis_error)?,
                        import,
                        ..Default::default()
                    };
                }
                let state: Option<String> = row.get(3).map_err(build_analysis_error)?;
                if let Some(state) = state {
                    current.ambiguous |= state == "ambiguous";
                    let metadata: String = row.get(4).map_err(build_analysis_error)?;
                    let value = parse(&metadata)?;
                    let namespace: String = row.get(5).map_err(build_analysis_error)?;
                    current
                        .candidates
                        .entry(metadata.clone())
                        .or_insert_with(|| (value.clone(), BTreeSet::new()))
                        .1
                        .insert(namespace.clone());
                    if state == "resolved" {
                        current
                            .resolved
                            .entry(metadata)
                            .or_insert_with(|| (value, BTreeSet::new()))
                            .1
                            .insert(namespace);
                    }
                }
            }
            if !current.key.is_empty() {
                for (table, value) in current.finish() {
                    emit(table, value)?;
                }
            }
        }
        Ok(count)
    }
}

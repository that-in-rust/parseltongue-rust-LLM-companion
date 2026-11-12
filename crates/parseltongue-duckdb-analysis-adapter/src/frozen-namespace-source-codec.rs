use super::{DuckdbAnalysisStoreAdapter, build_analysis_error};
use duckdb::types::Value as SqlValue;
use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StageTableRowRecord, StoreBackendFailureError,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const NAMESPACE_SOURCE_TABLE_NAMES: &[&str] = &[
    "crate-dependency-graph-table",
    "syntax-namespace-input-scopes",
    "syntax-namespace-input-items",
    "syntax-namespace-input-imports",
    "syntax-namespace-input-paths",
    "syntax-namespace-input-macros",
    "syntax-namespace-input-externs",
    "legacy-macro-scope-records",
];

impl DuckdbAnalysisStoreAdapter {
    pub fn normalize_frozen_namespace_rows(
        &self,
        scope: &StageRunScopeValue,
        cancel: &RunCancelFlagValue,
    ) -> Result<usize, StoreBackendFailureError> {
        check_cancel(cancel)?;
        let mut connection = self._connection.lock().map_err(build_analysis_error)?;
        NAMESPACE_SOURCE_TABLE_NAMES.iter().try_for_each(|table|{
            let count:usize=connection.query_row("SELECT count(*) FROM \"frozen-generation-table-manifests\" WHERE generation_key=? AND attempt_key=? AND table_name=?",duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value,table],|row|row.get(0)).map_err(build_analysis_error)?;
            if count!=1 {return Err(build_analysis_error(format!("missing frozen source manifest: {table}")));}
            Ok(())
        })?;
        connection
            .execute_batch(
                &include_str!("namespace-contract-input-schema.sql")
                    .replace("CREATE TABLE ", "CREATE TABLE IF NOT EXISTS "),
            )
            .map_err(build_analysis_error)?;
        connection
            .execute_batch(include_str!("namespace-frozen-input-schema.sql"))
            .map_err(build_analysis_error)?;
        let reader = connection.try_clone().map_err(build_analysis_error)?;
        let transaction = connection.transaction().map_err(build_analysis_error)?;
        let tables = [
            "namespace-scope-context-records",
            "namespace-binding-declaration-records",
            "namespace-target-context-records",
            "namespace-path-source-requests",
            "namespace-path-segment-records",
            "namespace-import-request-records",
            "namespace-declaration-visibility-records",
            "namespace-import-visibility-records",
            "namespace-unit-source-records",
            "namespace-target-source-records",
            "namespace-path-metadata-records",
            "namespace-macro-source-records",
            "namespace-extern-source-records",
            "namespace-legacy-scope-records",
        ];
        tables.iter().try_for_each(|table| {
            transaction
                .execute(
                    &format!("DELETE FROM \"{table}\" WHERE generation_key=? AND attempt_key=?"),
                    duckdb::params![
                        scope.run_generation_digest_value,
                        scope.stage_attempt_sequence_value
                    ],
                )
                .map(|_| ())
                .map_err(build_analysis_error)
        })?;
        let count = {
            let mut sink = NamespaceSourceAppender {
                appenders: tables
                    .iter()
                    .map(|table| {
                        Ok((
                            *table,
                            transaction.appender(table).map_err(build_analysis_error)?,
                        ))
                    })
                    .collect::<Result<_, StoreBackendFailureError>>()?,
                scope,
                graph: BTreeMap::new(),
                editions: BTreeMap::new(),
                budget:
                    super::namespace_appender_batch_budget::NamespaceAppenderBatchBudget::default(),
            };
            let count=NAMESPACE_SOURCE_TABLE_NAMES.iter().try_fold(0,|total,table|{
                let (expected,digest):(usize,String)=reader.query_row("SELECT row_count,content_digest FROM \"frozen-generation-table-manifests\" WHERE generation_key=? AND attempt_key=? AND table_name=? ORDER BY generation_key,attempt_key,table_name",duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value,table],|row|Ok((row.get(0)?,row.get(1)?))).map_err(build_analysis_error)?;
                let mut previous:Option<String>=None;
                let mut count=0;
                let mut hasher=blake3::Hasher::new();
                loop {
                    check_cancel(cancel)?;
                    let mut statement=reader.prepare("SELECT row_key,payload FROM \"frozen-generation-source-records\" WHERE generation_key=? AND attempt_key=? AND table_name=? AND (? IS NULL OR row_key>?) ORDER BY generation_key,attempt_key,table_name,row_key LIMIT 128").map_err(build_analysis_error)?;
                    let mut rows=statement.query(duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value,table,previous,previous]).map_err(build_analysis_error)?;
                    let mut page=0;
                    while let Some(row)=rows.next().map_err(build_analysis_error)? {
                        check_cancel(cancel)?;
                        let key:String=row.get(0).map_err(build_analysis_error)?;
                        let payload:String=row.get(1).map_err(build_analysis_error)?;
                        let value:Value=serde_json::from_str(&payload).map_err(build_analysis_error)?;
                        if !value.is_object(){return Err(build_analysis_error("namespace source row must be an object"));}
                        sink.append_source(table,&value)?;
                        super::frozen_generation_appender_copy::hash_record(&mut hasher,&StageTableRowRecord {stage_owned_table_name:(*table).into(),stage_row_record_key:key.clone(),stage_row_payload_text:payload});
                        previous=Some(key);
                        count+=1;
                        page+=1;
                    }
                    if page<128 {break;}
                }
                if count!=expected || hasher.finalize().to_hex().as_str()!=digest {return Err(build_analysis_error("frozen source count or digest changed before normalization"));}
                Ok::<_,StoreBackendFailureError>(total+count)
            })?;
            sink.appenders
                .values_mut()
                .try_for_each(|appender| appender.flush().map_err(build_analysis_error))?;
            count
        };
        transaction.execute("INSERT INTO \"namespace-target-context-records\"
            SELECT generation_key,attempt_key,target_key,min(child_scope),NULL,bool_or(scope_identity)
            FROM \"namespace-target-source-records\" WHERE generation_key=? AND attempt_key=? GROUP BY generation_key,attempt_key,target_key;",duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value]) .map_err(build_analysis_error)?;
        // Identical AST requests can be represented by both an import and a path.
        ["namespace-path-source-requests","namespace-path-segment-records","namespace-scope-context-records"].iter().try_for_each(|table|{
            transaction.execute(&format!("CREATE OR REPLACE TEMP TABLE \"namespace-distinct-input-records\" AS SELECT DISTINCT * FROM \"{table}\" WHERE generation_key=? AND attempt_key=?"),duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value]).map_err(build_analysis_error)?;
            transaction.execute(&format!("DELETE FROM \"{table}\" WHERE generation_key=? AND attempt_key=?"),duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value]).map_err(build_analysis_error)?;
            transaction.execute_batch(&format!("INSERT INTO \"{table}\" SELECT * FROM \"namespace-distinct-input-records\";")).map_err(build_analysis_error)
        })?;
        check_cancel(cancel)?;
        transaction
            .execute(
                "INSERT OR REPLACE INTO \"namespace-normalized-source-manifests\" VALUES (?,?,?)",
                duckdb::params![
                    scope.run_generation_digest_value,
                    scope.stage_attempt_sequence_value,
                    count as u64
                ],
            )
            .map_err(build_analysis_error)?;
        transaction.commit().map_err(build_analysis_error)?;
        drop(reader);
        connection
            .execute_batch("CHECKPOINT")
            .map_err(build_analysis_error)?;
        Ok(count)
    }
}

fn check_cancel(cancel: &RunCancelFlagValue) -> Result<(), StoreBackendFailureError> {
    if cancel.check_stage_work_cancelled() {
        Err(build_analysis_error("namespace normalization cancelled"))
    } else {
        Ok(())
    }
}

fn text(value: &Value, key: &str) -> Result<String, StoreBackendFailureError> {
    value[key]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| build_analysis_error(format!("namespace source string absent: {key}")))
}

fn object_key(value: &Value) -> Result<String, StoreBackendFailureError> {
    if value.is_object() {
        Ok(value.to_string())
    } else {
        Err(build_analysis_error(
            "namespace source identity must be an object",
        ))
    }
}

fn optional_key(value: &Value) -> Result<SqlValue, StoreBackendFailureError> {
    if value.is_null() {
        Ok(SqlValue::Null)
    } else {
        object_key(value).map(SqlValue::Text)
    }
}

fn scope_parts(value: &Value) -> Result<&Vec<Value>, StoreBackendFailureError> {
    value["scope_semantic_path_parts"]
        .as_array()
        .ok_or_else(|| build_analysis_error("namespace scope path absent"))
}

fn related_scope(scope: &Value, parts: Value) -> Value {
    json!({"scope_compilation_unit_key":scope["scope_compilation_unit_key"],"scope_semantic_path_parts":parts})
}

fn visibility(value: &Value) -> Result<SqlValue, StoreBackendFailureError> {
    match value["visibility_scope_kind_value"].as_str() {
        Some("public") => Ok(SqlValue::Null),
        Some("restricted") => optional_key(&value["visibility_restricted_scope_key"]),
        Some("invisible") => Ok(SqlValue::Text("invisible".into())),
        _ => Err(build_analysis_error("namespace visibility absent")),
    }
}

struct NamespaceSourceAppender<'a> {
    appenders: BTreeMap<&'static str, duckdb::Appender<'a>>,
    scope: &'a StageRunScopeValue,
    graph: BTreeMap<String, Value>,
    editions: BTreeMap<String, i32>,
    budget: super::namespace_appender_batch_budget::NamespaceAppenderBatchBudget,
}

impl NamespaceSourceAppender<'_> {
    fn append(
        &mut self,
        table: &str,
        values: Vec<SqlValue>,
    ) -> Result<(), StoreBackendFailureError> {
        let bytes = self.scope.run_generation_digest_value.len()
            + self.scope.stage_attempt_sequence_value.len()
            + values
                .iter()
                .map(|value| match value {
                    SqlValue::Text(text) => text.len(),
                    SqlValue::Blob(bytes) => bytes.len(),
                    SqlValue::List(values) => values.len() * 8,
                    _ => 8,
                })
                .sum::<usize>();
        self.appenders
            .get_mut(table)
            .ok_or_else(|| build_analysis_error("namespace appender absent"))?
            .append_row(duckdb::appender_params_from_iter(
                [
                    SqlValue::Text(self.scope.run_generation_digest_value.clone()),
                    SqlValue::Text(self.scope.stage_attempt_sequence_value.clone()),
                ]
                .into_iter()
                .chain(values),
            ))
            .map_err(build_analysis_error)?;
        if self.budget.record_appender_row_budget(bytes) {
            self.appenders
                .values_mut()
                .try_for_each(|appender| appender.flush().map_err(build_analysis_error))?;
        }
        Ok(())
    }

    fn edition(&self, scope: &Value) -> Result<i32, StoreBackendFailureError> {
        self.editions
            .get(&object_key(&scope["scope_compilation_unit_key"])?)
            .copied()
            .ok_or_else(|| build_analysis_error("path compilation unit absent"))
    }

    fn path(
        &mut self,
        site: &Value,
        scope: &Value,
        segments: &Value,
        kind: &str,
        filter: &str,
        reason: &Value,
    ) -> Result<(), StoreBackendFailureError> {
        let request = object_key(site)?;
        let segments = segments
            .as_array()
            .ok_or_else(|| build_analysis_error("namespace path segments absent"))?;
        let filter = match filter {
            "Any" => "any",
            "Type" => "type",
            "Value" => "value",
            "Macro" => "macro",
            "TypeOrValue" => "type-or-value",
            _ => return Err(build_analysis_error("namespace path filter invalid")),
        };
        self.append(
            "namespace-path-source-requests",
            vec![
                request.clone().into(),
                object_key(scope)?.into(),
                related_scope(scope, json!([])).to_string().into(),
                related_scope(scope, json!(["@extern"])).to_string().into(),
                SqlValue::Int(self.edition(scope)?),
                SqlValue::Text(kind.into()),
                SqlValue::Text(filter.into()),
                SqlValue::BigInt(segments.len() as i64),
                reason
                    .as_str()
                    .map(|value| SqlValue::Text(value.into()))
                    .unwrap_or(SqlValue::Null),
            ],
        )?;
        segments.iter().enumerate().try_for_each(|(position,segment)|{
            let kind=text(segment,"kind")?;
            let (kind,name)=match kind.as_str(){
                "name"=>("name",text(segment,"name")?.trim_start_matches("r#").to_owned()),
                "crate"=>("crate",String::new()),"self"=>("self",String::new()),"super"=>("super",String::new()),"global"=>("global",String::new()),
                "defining_crate"=>("defining",json!({"scope_compilation_unit_key":segment["unit"],"scope_semantic_path_parts":[]}).to_string()),
                _=>return Err(build_analysis_error("namespace path segment invalid")),
            };
            self.append("namespace-path-segment-records",vec![request.clone().into(),SqlValue::BigInt(position as i64),SqlValue::Text(kind.into()),name.into()])
        })
    }

    fn append_source(
        &mut self,
        table: &str,
        value: &Value,
    ) -> Result<(), StoreBackendFailureError> {
        match table {
            "crate-dependency-graph-table" => {
                self.graph
                    .insert(text(value, "unit_compilation_identity_key")?, value.clone());
                Ok(())
            }
            "syntax-namespace-input-scopes" => {
                let scope = &value["scope_module_identity_key"];
                let parts = scope_parts(scope)?;
                let unit = object_key(&scope["scope_compilation_unit_key"])?;
                self.append(
                    "namespace-scope-context-records",
                    vec![
                        unit.clone().into(),
                        object_key(scope)?.into(),
                        optional_key(&value["scope_parent_identity_key"])?,
                        SqlValue::Boolean(
                            value["scope_parent_lookup_flag"]
                                .as_bool()
                                .ok_or_else(|| build_analysis_error("parent lookup flag absent"))?,
                        ),
                        SqlValue::Boolean(
                            value["scope_module_boundary_flag"]
                                .as_bool()
                                .ok_or_else(|| {
                                    build_analysis_error("module boundary flag absent")
                                })?,
                        ),
                        optional_key(&value["scope_implicit_prelude_key"])?,
                    ],
                )?;
                if parts.is_empty() {
                    let cargo = text(value, "scope_cargo_unit_key")?;
                    let graph = self
                        .graph
                        .get(&cargo)
                        .ok_or_else(|| build_analysis_error("scope cargo unit absent"))?
                        .clone();
                    let edition = text(&graph, "unit_rust_edition_value")?
                        .parse::<i32>()
                        .map_err(build_analysis_error)?;
                    if ![2015, 2018, 2021, 2024].contains(&edition) {
                        return Err(build_analysis_error("unsupported Rust edition"));
                    }
                    self.editions.insert(unit.clone(), edition);
                    self.append(
                        "namespace-unit-source-records",
                        vec![
                            unit.into(),
                            cargo.into(),
                            object_key(scope)?.into(),
                            related_scope(scope, json!(["@extern"])).to_string().into(),
                            SqlValue::Int(edition),
                            graph.to_string().into(),
                        ],
                    )?;
                    self.append("namespace-target-source-records",vec![json!({"target_identity_variant_kind":"crate","target_compilation_unit_key":scope["scope_compilation_unit_key"]}).to_string().into(),object_key(scope)?.into(),json!({"target_symbol_kind_value":"Crate"}).to_string().into(),SqlValue::Boolean(true)])?;
                }
                Ok(())
            }
            "syntax-namespace-input-items" => {
                let scope = &value["binding_lexical_scope_key"];
                let target = object_key(&value["binding_target_identity_key"])?;
                let metadata = &value["binding_target_metadata_value"];
                if !metadata.is_object() {
                    return Err(build_analysis_error("target metadata absent"));
                }
                let child = optional_key(&value["binding_target_child_scope"])?;
                self.append(
                    "namespace-target-source-records",
                    vec![
                        target.clone().into(),
                        child,
                        metadata.to_string().into(),
                        SqlValue::Boolean(!value["binding_target_child_scope"].is_null()),
                    ],
                )?;
                let namespace = text(value, "binding_namespace_kind_value")?;
                if !["type", "value", "macro"].contains(&namespace.as_str()) {
                    return Err(build_analysis_error("binding namespace invalid"));
                }
                let legacy = metadata["target_macro_definition_record"].is_object()
                    && metadata["target_macro_definition_record"]["macro_declaration_style_flag"]
                        == false;
                let public = value["binding_visibility_scope_kind"]["visibility_scope_kind_value"]
                    == "public";
                if namespace == "macro" && legacy && !(public && scope_parts(scope)?.is_empty()) {
                    return Ok(());
                }
                let priority = match text(value, "binding_selection_priority_kind")?.as_str() {
                    "builtin" => 0,
                    "prelude" => 1,
                    "glob" => 2,
                    "explicit" => 3,
                    "declaration" => 4,
                    _ => return Err(build_analysis_error("binding priority invalid")),
                };
                let fields = vec![
                    object_key(scope)?.into(),
                    text(value, "binding_resolved_name_text")?
                        .trim_start_matches("r#")
                        .to_owned()
                        .into(),
                    namespace.into(),
                    target.into(),
                    SqlValue::Int(priority),
                    SqlValue::Text(String::new()),
                ];
                self.append("namespace-binding-declaration-records", fields.clone())?;
                self.append(
                    "namespace-declaration-visibility-records",
                    fields
                        .into_iter()
                        .chain([visibility(&value["binding_visibility_scope_kind"])?])
                        .collect(),
                )
            }
            "syntax-namespace-input-paths" => {
                let kind = text(value, "path_resolution_source_kind")?.to_lowercase();
                self.path(
                    &value["path_site_occurrence_key"],
                    &value["path_lexical_scope_key"],
                    &value["path_source_segment_rows"],
                    &kind,
                    &text(value, "path_requested_namespace_kind")?,
                    &value["path_unavailable_reason_kind"],
                )?;
                self.append(
                    "namespace-path-metadata-records",
                    vec![
                        object_key(&value["path_site_occurrence_key"])?.into(),
                        value["path_source_metadata_value"].to_string().into(),
                    ],
                )
            }
            "syntax-namespace-input-imports" => {
                let request = object_key(&value["import_site_occurrence_key"])?;
                self.path(
                    &value["import_site_occurrence_key"],
                    &value["import_lexical_scope_key"],
                    &value["import_path_segment_rows"],
                    "import",
                    "Any",
                    &Value::Null,
                )?;
                self.append(
                    "namespace-import-request-records",
                    vec![
                        request.clone().into(),
                        object_key(&value["import_lexical_scope_key"])?.into(),
                        value["import_binding_alias_text"]
                            .as_str()
                            .map(|alias| SqlValue::Text(alias.trim_start_matches("r#").into()))
                            .unwrap_or(SqlValue::Null),
                        SqlValue::Boolean(
                            value["import_glob_expansion_flag"]
                                .as_bool()
                                .ok_or_else(|| build_analysis_error("import glob flag absent"))?,
                        ),
                    ],
                )?;
                self.append(
                    "namespace-import-visibility-records",
                    vec![
                        request.into(),
                        visibility(&value["import_visibility_scope_kind"])?,
                    ],
                )
            }
            "syntax-namespace-input-macros" => {
                self.path(
                    &value["invocation_site_occurrence_key"],
                    &value["invocation_lexical_scope_key"],
                    &value["invocation_macro_path_segments"],
                    "macro",
                    "Macro",
                    &Value::Null,
                )?;
                self.append(
                    "namespace-macro-source-records",
                    vec![
                        object_key(&value["invocation_site_occurrence_key"])?.into(),
                        object_key(&value["invocation_lexical_scope_key"])?.into(),
                        value.to_string().into(),
                    ],
                )
            }
            "syntax-namespace-input-externs" => self.append(
                "namespace-extern-source-records",
                vec![value.to_string().into()],
            ),
            "legacy-macro-scope-records" => {
                let order = value["legacy_source_order_parts"]
                    .as_array()
                    .ok_or_else(|| build_analysis_error("legacy source order absent"))?
                    .iter()
                    .map(|part| {
                        part.as_u64()
                            .map(SqlValue::UBigInt)
                            .ok_or_else(|| build_analysis_error("legacy source order invalid"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.append(
                    "namespace-legacy-scope-records",
                    vec![
                        text(value, "legacy_lookup_identity_key")?.into(),
                        object_key(&value["legacy_source_site_key"])?.into(),
                        text(value, "legacy_macro_name_text")?
                            .trim_start_matches("r#")
                            .to_owned()
                            .into(),
                        optional_key(&value["legacy_target_identity_key"])?,
                        SqlValue::UBigInt(
                            value["legacy_scope_revision_value"]
                                .as_u64()
                                .ok_or_else(|| build_analysis_error("legacy revision absent"))?,
                        ),
                        SqlValue::List(order),
                    ],
                )
            }
            "legacy-macro-source-events" | "lang-item-definition-table" => Ok(()),
            _ => Err(build_analysis_error("unsupported namespace source table")),
        }
    }
}

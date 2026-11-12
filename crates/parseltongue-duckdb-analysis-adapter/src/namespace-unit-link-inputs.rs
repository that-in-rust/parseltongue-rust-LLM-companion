use super::{DuckdbAnalysisStoreAdapter, build_analysis_error};
use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StoreBackendFailureError,
};
use serde_json::{Value, json};

impl DuckdbAnalysisStoreAdapter {
    pub fn link_frozen_namespace_units(
        &self,
        scope: &StageRunScopeValue,
        cancel: &RunCancelFlagValue,
    ) -> Result<usize, StoreBackendFailureError> {
        let connection = self._connection.lock().map_err(build_analysis_error)?;
        link_namespace_unit_rows(&connection, scope, cancel)
    }
}

pub(super) fn link_namespace_unit_rows(
    connection: &duckdb::Connection,
    scope: &StageRunScopeValue,
    cancel: &RunCancelFlagValue,
) -> Result<usize, StoreBackendFailureError> {
    check_cancel(cancel)?;
    connection
        .execute_batch(include_str!("namespace-frozen-input-schema.sql"))
        .map_err(build_analysis_error)?;
    let complete:usize=connection.query_row("SELECT count(*) FROM \"namespace-normalized-source-manifests\" WHERE generation_key=? AND attempt_key=?",duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value],|row|row.get(0)).map_err(build_analysis_error)?;
    if complete != 1 {
        return Err(build_analysis_error("normalized unit source is incomplete"));
    }
    let transaction = connection
        .unchecked_transaction()
        .map_err(build_analysis_error)?;
    [
        "namespace-unit-link-records",
        "namespace-unit-dependency-records",
        "namespace-extern-link-records",
    ]
    .iter()
    .try_for_each(|table| {
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
    {
        let mut units = transaction
            .appender("namespace-unit-link-records")
            .map_err(build_analysis_error)?;
        let mut dependencies = transaction
            .appender("namespace-unit-dependency-records")
            .map_err(build_analysis_error)?;
        let mut statement=connection.prepare("SELECT unit_key,root_scope,extern_scope,edition,source_record FROM \"namespace-unit-source-records\" WHERE generation_key=? AND attempt_key=? ORDER BY generation_key,attempt_key,unit_key,cargo_key,root_scope,extern_scope,edition,source_record").map_err(build_analysis_error)?;
        let mut rows = statement
            .query(duckdb::params![
                scope.run_generation_digest_value,
                scope.stage_attempt_sequence_value
            ])
            .map_err(build_analysis_error)?;
        while let Some(row) = rows.next().map_err(build_analysis_error)? {
            check_cancel(cancel)?;
            let key: String = row.get(0).map_err(build_analysis_error)?;
            let root: String = row.get(1).map_err(build_analysis_error)?;
            let external: String = row.get(2).map_err(build_analysis_error)?;
            let edition: i32 = row.get(3).map_err(build_analysis_error)?;
            let source: String = row.get(4).map_err(build_analysis_error)?;
            let unit: Value = serde_json::from_str(&key).map_err(build_analysis_error)?;
            let graph: Value = serde_json::from_str(&source).map_err(build_analysis_error)?;
            let prelude =
                json!({"scope_compilation_unit_key":unit,"scope_semantic_path_parts":["@prelude"]})
                    .to_string();
            let target =
                json!({"target_identity_variant_kind":"crate","target_compilation_unit_key":unit})
                    .to_string();
            units
                .append_row(duckdb::params![
                    scope.run_generation_digest_value,
                    scope.stage_attempt_sequence_value,
                    key,
                    read_text(&unit, "unit_cargo_package_identifier")?,
                    read_text(&unit, "unit_compilation_role_kind")?,
                    read_text(&unit, "unit_cargo_target_kind")?,
                    read_text(&unit, "unit_cargo_target_name")?,
                    root,
                    external,
                    prelude,
                    target,
                    edition,
                    graph["unit_implicit_prelude_flag"]
                        .as_bool()
                        .unwrap_or(true),
                    graph["unit_no_std_flag"].as_bool().unwrap_or(false),
                    graph["unit_no_core_flag"].as_bool().unwrap_or(false),
                    graph["unit_sysroot_context_flag"]
                        .as_bool()
                        .unwrap_or(false)
                ])
                .map_err(build_analysis_error)?;
            graph["unit_normal_dependency_rows"]
                .as_array()
                .ok_or_else(|| build_analysis_error("unit dependencies absent"))?
                .iter()
                .try_for_each(|dependency| {
                    dependencies
                        .append_row(duckdb::params![
                            scope.run_generation_digest_value,
                            scope.stage_attempt_sequence_value,
                            key,
                            read_text(dependency, "dependency_alias_name_value")?
                                .trim_start_matches("r#"),
                            read_text(dependency, "dependency_cargo_package_identifier")?,
                            read_text(dependency, "dependency_compilation_role_kind")?,
                            dependency["dependency_sysroot_context_flag"]
                                .as_bool()
                                .unwrap_or(false)
                        ])
                        .map_err(build_analysis_error)
                })?;
        }
        units.flush().map_err(build_analysis_error)?;
        dependencies.flush().map_err(build_analysis_error)?;
    }
    {
        let mut appender = transaction
            .appender("namespace-extern-link-records")
            .map_err(build_analysis_error)?;
        let mut statement=connection.prepare("SELECT source_record FROM \"namespace-extern-source-records\" WHERE generation_key=? AND attempt_key=? ORDER BY generation_key,attempt_key,source_record").map_err(build_analysis_error)?;
        let mut rows = statement
            .query(duckdb::params![
                scope.run_generation_digest_value,
                scope.stage_attempt_sequence_value
            ])
            .map_err(build_analysis_error)?;
        while let Some(row) = rows.next().map_err(build_analysis_error)? {
            check_cancel(cancel)?;
            let source: String = row.get(0).map_err(build_analysis_error)?;
            let value: Value = serde_json::from_str(&source).map_err(build_analysis_error)?;
            let lexical = &value["extern_declaration_scope_key"];
            if !lexical.is_object() {
                return Err(build_analysis_error("extern scope absent"));
            }
            let visibility = &value["extern_declaration_visibility"];
            let restriction = match read_text(visibility, "visibility_scope_kind_value")? {
                "public" => None,
                "restricted" => Some(visibility["visibility_restricted_scope_key"].to_string()),
                "invisible" => Some("invisible".into()),
                _ => return Err(build_analysis_error("extern visibility invalid")),
            };
            appender
                .append_row(duckdb::params![
                    scope.run_generation_digest_value,
                    scope.stage_attempt_sequence_value,
                    lexical["scope_compilation_unit_key"].to_string(),
                    lexical.to_string(),
                    read_text(&value, "extern_declaration_source_name")?,
                    read_text(&value, "extern_declaration_alias_name")?,
                    restriction
                ])
                .map_err(build_analysis_error)?;
        }
        appender.flush().map_err(build_analysis_error)?;
    }
    transaction.execute("CREATE OR REPLACE TEMP TABLE \"namespace-active-unit-contexts\" AS SELECT * FROM \"namespace-unit-link-records\" WHERE generation_key=? AND attempt_key=?",duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value]).map_err(build_analysis_error)?;
    transaction.execute_batch("CREATE OR REPLACE TEMP TABLE \"namespace-dependency-target-records\" AS
        SELECT d.*,t.target_key,t.root_scope,t.target_kind FROM \"namespace-unit-dependency-records\" d
        JOIN \"namespace-active-unit-contexts\" u ON u.generation_key=d.generation_key AND u.attempt_key=d.attempt_key AND u.unit_key=d.unit_key
        LEFT JOIN \"namespace-active-unit-contexts\" t ON t.package_key=d.package_key AND t.compilation_role=d.compilation_role AND t.target_kind IN ('lib','rlib','dylib','staticlib','cdylib','proc-macro');
        CREATE OR REPLACE TEMP TABLE \"namespace-derived-extern-bindings\" AS
        SELECT d.generation_key,d.attempt_key,u.extern_scope AS scope_key,d.name,'type' AS namespace,d.target_key,CASE WHEN d.sysroot_context THEN 2 ELSE 3 END AS priority,'namespace-unit-link-evidence/auto/'||d.name AS evidence_key,NULL::VARCHAR AS visibility_scope
        FROM \"namespace-dependency-target-records\" d JOIN \"namespace-active-unit-contexts\" u ON u.unit_key=d.unit_key
        WHERE u.implicit_prelude AND d.target_key IS NOT NULL AND (NOT d.sysroot_context OR (NOT u.no_core AND (d.name='core' OR (d.name='std' AND NOT u.no_std))))
        UNION ALL
        SELECT e.generation_key,e.attempt_key,e.scope_key,e.alias_name,'type',coalesce(d.target_key,CASE WHEN e.source_name='self' THEN u.target_key END),4,'namespace-unit-link-evidence/explicit/'||e.alias_name,e.visibility_scope
        FROM \"namespace-extern-link-records\" e JOIN \"namespace-active-unit-contexts\" u ON u.generation_key=e.generation_key AND u.attempt_key=e.attempt_key AND u.unit_key=e.unit_key
        LEFT JOIN \"namespace-dependency-target-records\" d ON d.unit_key=e.unit_key AND d.name=e.source_name
        WHERE e.alias_name<>'_' AND (d.target_key IS NOT NULL OR e.source_name='self');").map_err(build_analysis_error)?;
    ["namespace-binding-declaration-records","namespace-declaration-visibility-records"].iter().try_for_each(|table|transaction.execute(&format!("DELETE FROM \"{table}\" WHERE generation_key=? AND attempt_key=? AND starts_with(evidence_key,'namespace-unit-link-evidence/')"),duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value]).map(|_|()).map_err(build_analysis_error))?;
    transaction.execute_batch("INSERT INTO \"namespace-binding-declaration-records\" SELECT DISTINCT * EXCLUDE(visibility_scope) FROM \"namespace-derived-extern-bindings\"; INSERT INTO \"namespace-declaration-visibility-records\" SELECT DISTINCT * FROM \"namespace-derived-extern-bindings\";").map_err(build_analysis_error)?;
    let count = transaction
        .query_row(
            "SELECT count(*) FROM \"namespace-derived-extern-bindings\"",
            [],
            |row| row.get(0),
        )
        .map_err(build_analysis_error)?;
    check_cancel(cancel)?;
    transaction.commit().map_err(build_analysis_error)?;
    Ok(count)
}

fn read_text<'a>(value: &'a Value, key: &str) -> Result<&'a str, StoreBackendFailureError> {
    value[key]
        .as_str()
        .ok_or_else(|| build_analysis_error(format!("unit link string absent: {key}")))
}

fn check_cancel(cancel: &RunCancelFlagValue) -> Result<(), StoreBackendFailureError> {
    if cancel.check_stage_work_cancelled() {
        Err(build_analysis_error("unit linking cancelled"))
    } else {
        Ok(())
    }
}

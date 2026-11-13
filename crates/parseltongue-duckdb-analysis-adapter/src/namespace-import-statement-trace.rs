use parseltongue_core_row_types::{NameResolutionStageError, StageRunScopeValue};
use std::time::{Duration, Instant};

pub(super) const IMPORT_BINDING_SQL_SCRIPT: &str =
    include_str!("namespace-import-binding-rounds.sql");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ImportBindingStatementSlice<'a> {
    ordinal: usize,
    name: &'static str,
    sql: &'a str,
}

const IMPORT_BINDING_STATEMENT_SPECS: [(&str, &str); 9] = [
    (
        "clear_binding_round_inputs",
        "DELETE FROM \"namespace-binding-round-inputs\";",
    ),
    (
        "seed_declaration_binding_inputs",
        "INSERT INTO \"namespace-binding-round-inputs\"\nSELECT * FROM \"namespace-active-declaration-records\";",
    ),
    (
        "apply_direct_import_bindings",
        "INSERT INTO \"namespace-binding-round-inputs\"\nSELECT i.generation_key,i.attempt_key,i.importer_scope,regexp_replace",
    ),
    (
        "apply_glob_import_bindings",
        "INSERT INTO \"namespace-binding-round-inputs\"\nSELECT i.generation_key,i.attempt_key,i.importer_scope,b.name,b.namespace",
    ),
    (
        "apply_macro_prelude_bindings",
        "INSERT INTO \"namespace-binding-round-inputs\"\nSELECT p.generation_key,p.attempt_key,p.importer_scope,b.name,b.namespace",
    ),
    (
        "clear_binding_visibility_inputs",
        "DELETE FROM \"namespace-binding-visibility-records\";",
    ),
    (
        "seed_declaration_visibility_inputs",
        "INSERT INTO \"namespace-binding-visibility-records\"\nSELECT * FROM \"namespace-active-declaration-visibility\";",
    ),
    (
        "apply_import_visibility_evidence",
        "INSERT INTO \"namespace-binding-visibility-records\"\nSELECT b.*,v.visibility_scope FROM \"namespace-binding-round-inputs\" b",
    ),
    (
        "apply_prelude_visibility_evidence",
        "INSERT INTO \"namespace-binding-visibility-records\"\nSELECT b.*,NULL FROM \"namespace-binding-round-inputs\" b",
    ),
];

fn build_import_boundary_error() -> NameResolutionStageError {
    super::namespace_contract_input_codec::error(
        "unsupported fixed namespace import binding script boundary contract",
    )
}

fn split_import_binding_script(
    script: &str,
) -> Result<Vec<ImportBindingStatementSlice<'_>>, NameResolutionStageError> {
    if script.contains("--")
        || script.contains("/*")
        || script.contains("*/")
        || script.bytes().filter(|byte| *byte == b'\'').count() % 2 != 0
        || script.bytes().filter(|byte| *byte == b'"').count() % 2 != 0
    {
        return Err(build_import_boundary_error());
    }

    let positions = IMPORT_BINDING_STATEMENT_SPECS
        .iter()
        .map(|(_, marker)| {
            let mut matches = script.match_indices(marker);
            match (matches.next(), matches.next()) {
                (Some((position, _)), None) => Ok(position),
                _ => Err(build_import_boundary_error()),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    if positions.first() != Some(&0) || positions.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(build_import_boundary_error());
    }

    let slices = IMPORT_BINDING_STATEMENT_SPECS
        .iter()
        .zip(positions.iter())
        .enumerate()
        .map(|(index, ((name, _), start))| {
            let end = positions.get(index + 1).copied().unwrap_or(script.len());
            let sql = &script[*start..end];
            ImportBindingStatementSlice {
                ordinal: index + 1,
                name,
                sql,
            }
        })
        .collect::<Vec<_>>();

    if slices
        .iter()
        .any(|slice| slice.sql.matches(';').count() != 1 || !slice.sql.trim_end().ends_with(';'))
    {
        return Err(build_import_boundary_error());
    }

    Ok(slices)
}

pub(super) fn execute_import_binding_statements<F>(
    connection: &duckdb::Connection,
    script: &str,
    scope: &StageRunScopeValue,
    inner_round: usize,
    mut emit: F,
) -> Result<(), NameResolutionStageError>
where
    F: FnMut(String),
{
    split_import_binding_script(script)?
        .into_iter()
        .try_for_each(|statement| {
            emit(build_statement_trace_event(
                scope,
                inner_round,
                &statement,
                "started",
                Duration::ZERO,
                None,
            ));
            let started = Instant::now();
            match connection.execute_batch(statement.sql) {
                Ok(()) => {
                    emit(build_statement_trace_event(
                        scope,
                        inner_round,
                        &statement,
                        "complete",
                        started.elapsed(),
                        None,
                    ));
                    Ok(())
                }
                Err(failure) => {
                    let failure_text = failure.to_string();
                    emit(build_statement_trace_event(
                        scope,
                        inner_round,
                        &statement,
                        "failed",
                        started.elapsed(),
                        Some(&failure_text),
                    ));
                    Err(super::namespace_contract_input_codec::error(failure))
                }
            }
        })
}

fn build_statement_trace_event(
    scope: &StageRunScopeValue,
    inner_round: usize,
    statement: &ImportBindingStatementSlice<'_>,
    event_name: &str,
    elapsed: Duration,
    failure: Option<&str>,
) -> String {
    serde_json::json!({
        "schema": "namespace_import_statement_trace_v1",
        "stage": "namespace-resolution-step",
        "step": "import-binding-statement",
        "event": event_name,
        "generation": &scope.run_generation_digest_value,
        "attempt": &scope.stage_attempt_sequence_value,
        "inner_round": inner_round,
        "statement_ordinal": statement.ordinal,
        "statement_name": statement.name,
        "elapsed_ms": elapsed.as_millis(),
        "error": failure,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use parseltongue_core_row_types::StageRunScopeValue;
    use serde_json::Value;

    type ImportBindingInputRow = (String, String, String, String, String, String, i32, String);
    type ImportBindingVisibilityRow = (
        String,
        String,
        String,
        String,
        String,
        String,
        i32,
        String,
        Option<String>,
    );
    type ImportBindingResultRow = (
        String,
        String,
        String,
        String,
        String,
        String,
        i32,
        String,
        String,
    );

    const IMPORT_BINDING_FIXTURE_SCHEMA: &str = r#"
CREATE TABLE "namespace-binding-round-inputs" (
    generation_key VARCHAR, attempt_key VARCHAR, scope_key VARCHAR, name VARCHAR,
    namespace VARCHAR, target_key VARCHAR, priority INTEGER, evidence_key VARCHAR
);
CREATE TABLE "namespace-active-declaration-records" (
    generation_key VARCHAR, attempt_key VARCHAR, scope_key VARCHAR, name VARCHAR,
    namespace VARCHAR, target_key VARCHAR, priority INTEGER, evidence_key VARCHAR
);
CREATE TABLE "namespace-active-import-requests" (
    generation_key VARCHAR, attempt_key VARCHAR, importer_scope VARCHAR,
    alias_name VARCHAR, request_key VARCHAR, is_glob BOOLEAN
);
CREATE TABLE "namespace-path-candidate-records" (
    generation_key VARCHAR, attempt_key VARCHAR, request_key VARCHAR, target_key VARCHAR,
    namespace VARCHAR, binding_name VARCHAR, state VARCHAR
);
CREATE TABLE "namespace-binding-round-results" (
    generation_key VARCHAR NOT NULL, attempt_key VARCHAR NOT NULL, scope_key VARCHAR NOT NULL,
    name VARCHAR NOT NULL, namespace VARCHAR NOT NULL, target_key VARCHAR NOT NULL,
    priority INTEGER NOT NULL, evidence_key VARCHAR NOT NULL, state VARCHAR NOT NULL,
    PRIMARY KEY(generation_key, attempt_key, scope_key, name, namespace, target_key, evidence_key)
);
CREATE TABLE "namespace-binding-visibility-records" (
    generation_key VARCHAR, attempt_key VARCHAR, scope_key VARCHAR, name VARCHAR,
    namespace VARCHAR, target_key VARCHAR, priority INTEGER, evidence_key VARCHAR,
    visibility_scope VARCHAR
);
CREATE TABLE "namespace-active-declaration-visibility" (
    generation_key VARCHAR, attempt_key VARCHAR, scope_key VARCHAR, name VARCHAR,
    namespace VARCHAR, target_key VARCHAR, priority INTEGER, evidence_key VARCHAR,
    visibility_scope VARCHAR
);
CREATE TABLE "namespace-import-visibility-records" (
    generation_key VARCHAR, attempt_key VARCHAR, request_key VARCHAR, visibility_scope VARCHAR
);
CREATE TABLE "namespace-macro-prelude-records" (
    generation_key VARCHAR, attempt_key VARCHAR, importer_scope VARCHAR,
    source_scope VARCHAR, evidence_key VARCHAR
);
CREATE TABLE "namespace-scope-ancestor-records" (
    generation_key VARCHAR, attempt_key VARCHAR, scope_key VARCHAR, ancestor_scope VARCHAR
);
"#;

    const IMPORT_TARGET_CONTEXT_SCHEMA: &str = r#"
CREATE TABLE "namespace-target-context-records" (
    generation_key VARCHAR, attempt_key VARCHAR, target_key VARCHAR,
    child_scope VARCHAR, visibility_scope VARCHAR
);
"#;

    fn create_import_scope_value() -> StageRunScopeValue {
        StageRunScopeValue {
            run_generation_digest_value: "generation".into(),
            stage_attempt_sequence_value: "attempt".into(),
        }
    }

    fn create_import_fixture_connection(with_target_context: bool) -> duckdb::Connection {
        let connection = duckdb::Connection::open_in_memory().unwrap();
        connection
            .execute_batch(IMPORT_BINDING_FIXTURE_SCHEMA)
            .unwrap();
        if with_target_context {
            connection
                .execute_batch(IMPORT_TARGET_CONTEXT_SCHEMA)
                .unwrap();
        }
        connection
            .execute_batch(
                r#"
INSERT INTO "namespace-binding-round-inputs" VALUES
    ('generation','attempt','stale_scope','Stale','type','stale_target',0,'stale_evidence');
INSERT INTO "namespace-active-declaration-records" VALUES
    ('generation','attempt','root','Declared','type','declared_target',1,'declared_evidence');
INSERT INTO "namespace-active-declaration-visibility" VALUES
    ('generation','attempt','root','Declared','type','declared_target',1,'declared_evidence','public_scope');
INSERT INTO "namespace-active-import-requests" VALUES
    ('generation','attempt','allowed_consumer','DirectAlias','direct_request',false),
    ('generation','attempt','allowed_consumer',NULL,'glob_allowed_request',true),
    ('generation','attempt','blocked_consumer',NULL,'glob_blocked_request',true);
INSERT INTO "namespace-path-candidate-records" VALUES
    ('generation','attempt','direct_request','direct_target','type','DirectName','resolved'),
    ('generation','attempt','glob_allowed_request','glob_module','type','GlobModule','resolved'),
    ('generation','attempt','glob_blocked_request','glob_module','type','GlobModule','resolved');
INSERT INTO "namespace-binding-round-results" VALUES
    ('generation','attempt','glob_source','GlobItem','type','glob_item_target',2,'glob_declaration','resolved'),
    ('generation','attempt','macro_source','PreludeMacro','macro','prelude_target',2,'macro_declaration','resolved');
INSERT INTO "namespace-binding-visibility-records" VALUES
    ('generation','attempt','glob_source','GlobItem','type','glob_item_target',2,'glob_declaration','allowed_visibility');
INSERT INTO "namespace-import-visibility-records" VALUES
    ('generation','attempt','direct_request','direct_visibility'),
    ('generation','attempt','glob_allowed_request','glob_import_visibility'),
    ('generation','attempt','glob_blocked_request','blocked_import_visibility');
INSERT INTO "namespace-macro-prelude-records" VALUES
    ('generation','attempt','allowed_consumer','macro_source','prelude_evidence');
INSERT INTO "namespace-scope-ancestor-records" VALUES
    ('generation','attempt','allowed_consumer','allowed_visibility');
"#,
            )
            .unwrap();
        if with_target_context {
            connection
                .execute_batch(
                    r#"
INSERT INTO "namespace-target-context-records" VALUES
    ('generation','attempt','glob_module','glob_source','fallback_visibility'),
    ('generation','attempt','glob_item_target',NULL,'fallback_item_visibility'),
    ('generation','attempt','prelude_target',NULL,NULL);
"#,
                )
                .unwrap();
        }
        connection
    }

    fn collect_binding_input_rows(connection: &duckdb::Connection) -> Vec<ImportBindingInputRow> {
        let mut statement = connection
            .prepare(
                "SELECT generation_key,attempt_key,scope_key,name,namespace,target_key,priority,evidence_key FROM \"namespace-binding-round-inputs\" ORDER BY generation_key,attempt_key,scope_key,name,namespace,target_key,evidence_key",
            )
            .unwrap();
        statement
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                ))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn collect_binding_visibility_rows(
        connection: &duckdb::Connection,
    ) -> Vec<ImportBindingVisibilityRow> {
        let mut statement = connection
            .prepare(
                "SELECT generation_key,attempt_key,scope_key,name,namespace,target_key,priority,evidence_key,visibility_scope FROM \"namespace-binding-visibility-records\" ORDER BY generation_key,attempt_key,scope_key,name,namespace,target_key,evidence_key,visibility_scope",
            )
            .unwrap();
        statement
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                ))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn collect_binding_result_rows(connection: &duckdb::Connection) -> Vec<ImportBindingResultRow> {
        let mut statement = connection
            .prepare(
                "SELECT generation_key,attempt_key,scope_key,name,namespace,target_key,priority,evidence_key,state FROM \"namespace-binding-round-results\" WHERE generation_key='generation' AND attempt_key='attempt' ORDER BY generation_key,attempt_key,scope_key,name,namespace,target_key,evidence_key",
            )
            .unwrap();
        statement
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                ))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn execute_old_import_batch(
        connection: &duckdb::Connection,
    ) -> Result<(), NameResolutionStageError> {
        connection
            .execute_batch(IMPORT_BINDING_SQL_SCRIPT)
            .map_err(super::super::namespace_contract_input_codec::error)
    }

    fn collect_import_binding_digest(connection: &duckdb::Connection) -> String {
        crate::namespace_binding_round_strata::recompute_binding_rows(
            connection,
            &create_import_scope_value(),
        )
        .unwrap()
    }

    /// REQ-RUST-028.2
    #[test]
    fn test_import_sql_statement_boundaries() {
        let slices = split_import_binding_script(IMPORT_BINDING_SQL_SCRIPT).unwrap();
        assert_eq!(slices.len(), 9);
        assert_eq!(
            slices.iter().map(|slice| slice.sql).collect::<String>(),
            IMPORT_BINDING_SQL_SCRIPT
        );
        for (index, slice) in slices.iter().enumerate() {
            assert_eq!(slice.ordinal, index + 1);
            assert_eq!(slice.name, IMPORT_BINDING_STATEMENT_SPECS[index].0);
            assert!(slice.sql.trim_end().ends_with(';'));
        }
    }

    /// REQ-RUST-028.2
    #[test]
    fn test_sql_quoted_comment_boundaries() {
        let quoted = IMPORT_BINDING_SQL_SCRIPT.replacen(
            "SELECT * FROM \"namespace-active-declaration-records\";",
            "SELECT ';' FROM \"namespace-active-declaration-records\";",
            1,
        );
        let commented = format!("{IMPORT_BINDING_SQL_SCRIPT}-- unsupported ; comment\n");
        assert!(split_import_binding_script(&quoted).is_err());
        assert!(split_import_binding_script(&commented).is_err());
    }

    /// REQ-RUST-028.2
    #[test]
    fn test_import_sql_rejects_unsupported() {
        let connection = duckdb::Connection::open_in_memory().unwrap();
        connection
            .execute_batch("CREATE TABLE \"trace_sentinel\" (value INTEGER); INSERT INTO \"trace_sentinel\" VALUES (7);")
            .unwrap();
        let mut empty_events = Vec::new();
        let empty_result = execute_import_binding_statements(
            &connection,
            "",
            &create_import_scope_value(),
            1,
            |event| empty_events.push(event),
        );
        assert!(empty_result.is_err());
        assert!(empty_events.is_empty());
        let unsupported = format!("DELETE FROM \"trace_sentinel\";{IMPORT_BINDING_SQL_SCRIPT}");
        let mut events = Vec::new();
        let result = execute_import_binding_statements(
            &connection,
            &unsupported,
            &create_import_scope_value(),
            1,
            |event| events.push(event),
        );
        assert!(result.is_err());
        assert!(events.is_empty());
        let sentinel: i32 = connection
            .query_row("SELECT value FROM \"trace_sentinel\"", [], |row| row.get(0))
            .unwrap();
        assert_eq!(sentinel, 7);
        assert!(split_import_binding_script("").is_err());
    }

    /// REQ-RUST-028.2
    #[test]
    fn test_import_batch_output_equivalence() {
        let batch = create_import_fixture_connection(true);
        let traced = create_import_fixture_connection(true);
        execute_old_import_batch(&batch).unwrap();
        execute_import_binding_statements(
            &traced,
            IMPORT_BINDING_SQL_SCRIPT,
            &create_import_scope_value(),
            3,
            |_| {},
        )
        .unwrap();

        let batch_inputs = collect_binding_input_rows(&batch);
        let traced_inputs = collect_binding_input_rows(&traced);
        let batch_visibility = collect_binding_visibility_rows(&batch);
        let traced_visibility = collect_binding_visibility_rows(&traced);
        assert_eq!(batch_inputs, traced_inputs);
        assert_eq!(batch_visibility, traced_visibility);
        assert!(batch_inputs.iter().any(|row| {
            row.2 == "allowed_consumer" && row.3 == "DirectAlias" && row.5 == "direct_target"
        }));
        assert!(batch_inputs.iter().any(|row| {
            row.2 == "allowed_consumer" && row.3 == "GlobItem" && row.5 == "glob_item_target"
        }));
        assert!(
            !batch_inputs
                .iter()
                .any(|row| { row.2 == "blocked_consumer" && row.3 == "GlobItem" })
        );
        assert!(batch_inputs.iter().any(|row| {
            row.2 == "allowed_consumer" && row.3 == "PreludeMacro" && row.5 == "prelude_target"
        }));
        assert!(batch_visibility.iter().any(|row| {
            row.3 == "DirectAlias" && row.8.as_deref() == Some("direct_visibility")
        }));
        assert!(batch_visibility.iter().any(|row| {
            row.3 == "GlobItem" && row.8.as_deref() == Some("glob_import_visibility")
        }));
        assert!(
            batch_visibility
                .iter()
                .any(|row| { row.3 == "PreludeMacro" && row.8.is_none() })
        );

        let batch_digest = collect_import_binding_digest(&batch);
        let traced_digest = collect_import_binding_digest(&traced);
        assert_eq!(batch_digest, traced_digest);
        assert_eq!(
            collect_binding_result_rows(&batch),
            collect_binding_result_rows(&traced)
        );
    }

    /// REQ-RUST-028.2
    #[test]
    fn test_import_batch_trace_markers() {
        let connection = create_import_fixture_connection(true);
        let scope = StageRunScopeValue {
            run_generation_digest_value: "generation\"quoted".into(),
            stage_attempt_sequence_value: "attempt\nline".into(),
        };
        let mut events = Vec::new();
        execute_import_binding_statements(
            &connection,
            IMPORT_BINDING_SQL_SCRIPT,
            &scope,
            7,
            |event| events.push(event),
        )
        .unwrap();
        assert_eq!(events.len(), 18);
        assert!(events[0].contains("generation\\\"quoted"));
        assert!(events[0].contains("attempt\\nline"));
        let (event_pairs, trailing_events) = events.as_chunks::<2>();
        assert!(trailing_events.is_empty());
        for (index, pair) in event_pairs.iter().enumerate() {
            let started: Value = serde_json::from_str(&pair[0]).unwrap();
            let completed: Value = serde_json::from_str(&pair[1]).unwrap();
            for event in [&started, &completed] {
                assert_eq!(event["generation"], scope.run_generation_digest_value);
                assert_eq!(event["attempt"], scope.stage_attempt_sequence_value);
                assert_eq!(event["inner_round"], 7);
                assert_eq!(event["statement_ordinal"], index + 1);
                assert_eq!(
                    event["statement_name"],
                    IMPORT_BINDING_STATEMENT_SPECS[index].0
                );
                assert!(event["elapsed_ms"].is_number());
            }
            assert_eq!(started["event"], "started");
            assert_eq!(started["elapsed_ms"], 0);
            assert_eq!(completed["event"], "complete");
        }
    }

    /// REQ-RUST-028.2
    #[test]
    fn test_import_batch_failure_equivalence() {
        for explicit_transaction in [false, true] {
            let batch = create_import_fixture_connection(false);
            let traced = create_import_fixture_connection(false);
            if explicit_transaction {
                batch.execute_batch("BEGIN TRANSACTION").unwrap();
                traced.execute_batch("BEGIN TRANSACTION").unwrap();
            }
            let batch_error = execute_old_import_batch(&batch).unwrap_err();
            let mut events = Vec::new();
            let traced_error = execute_import_binding_statements(
                &traced,
                IMPORT_BINDING_SQL_SCRIPT,
                &create_import_scope_value(),
                4,
                |event| events.push(event),
            )
            .unwrap_err();
            let batch_text = match &batch_error {
                NameResolutionStageError::Input(error) => error.error_context_detail_text.as_str(),
                _ => "",
            };
            let traced_text = match &traced_error {
                NameResolutionStageError::Input(error) => error.error_context_detail_text.as_str(),
                _ => "",
            };
            let missing_table_error =
                "Catalog Error: Table with name namespace-target-context-records does not exist!";
            assert!(batch_text.contains(missing_table_error));
            assert!(traced_text.contains(missing_table_error));
            let raw_failure_connection = create_import_fixture_connection(false);
            if explicit_transaction {
                raw_failure_connection
                    .execute_batch("BEGIN TRANSACTION")
                    .unwrap();
            }
            let slices = split_import_binding_script(IMPORT_BINDING_SQL_SCRIPT).unwrap();
            slices.iter().take(3).for_each(|statement| {
                raw_failure_connection.execute_batch(statement.sql).unwrap();
            });
            let raw_failure = raw_failure_connection
                .execute_batch(slices[3].sql)
                .unwrap_err()
                .to_string();
            assert_eq!(traced_text, raw_failure);
            if explicit_transaction {
                batch.execute_batch("ROLLBACK").unwrap();
                traced.execute_batch("ROLLBACK").unwrap();
                assert_eq!(collect_binding_input_rows(&batch).len(), 1);
                assert_eq!(
                    collect_binding_input_rows(&traced),
                    collect_binding_input_rows(&batch)
                );
                assert!(
                    collect_binding_input_rows(&batch)
                        .iter()
                        .all(|row| row.3 == "Stale")
                );
                assert_eq!(
                    collect_binding_visibility_rows(&traced),
                    collect_binding_visibility_rows(&batch)
                );
            } else {
                assert_eq!(collect_binding_input_rows(&batch).len(), 2);
                assert_eq!(
                    collect_binding_input_rows(&traced),
                    collect_binding_input_rows(&batch)
                );
                assert_eq!(
                    collect_binding_visibility_rows(&traced),
                    collect_binding_visibility_rows(&batch)
                );
                assert!(
                    collect_binding_input_rows(&batch)
                        .iter()
                        .all(|row| row.3 != "Stale")
                );
            }
            let visibility_rows = collect_binding_visibility_rows(&traced);
            assert!(
                !visibility_rows
                    .iter()
                    .any(|row| { row.7 == "direct_request" || row.7 == "prelude_evidence" })
            );
            let parsed_events = events
                .iter()
                .map(|event| serde_json::from_str::<Value>(event).unwrap())
                .collect::<Vec<_>>();
            assert_eq!(parsed_events.len(), 8);
            assert_eq!(parsed_events[7]["event"], "failed");
            assert_eq!(parsed_events[7]["statement_ordinal"], 4);
            assert_eq!(parsed_events[7]["error"], traced_text);
        }
    }
}

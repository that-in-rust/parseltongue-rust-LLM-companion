use super::build_analysis_error;
use parseltongue_core_row_types::{StageRunScopeValue, StoreBackendFailureError};

pub fn recompute_binding_round(
    connection: &duckdb::Connection,
    scope: &StageRunScopeValue,
) -> Result<String, StoreBackendFailureError> {
    let transaction = connection
        .unchecked_transaction()
        .map_err(build_analysis_error)?;
    transaction
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS \"namespace-binding-round-results\" (
        generation_key VARCHAR NOT NULL,attempt_key VARCHAR NOT NULL,scope_key VARCHAR NOT NULL,
        name VARCHAR NOT NULL,namespace VARCHAR NOT NULL,target_key VARCHAR NOT NULL,
        priority INTEGER NOT NULL,evidence_key VARCHAR NOT NULL,state VARCHAR NOT NULL,
        PRIMARY KEY(generation_key,attempt_key,scope_key,name,namespace,target_key,evidence_key));",
        )
        .map_err(build_analysis_error)?;
    transaction.execute("DELETE FROM \"namespace-binding-round-results\" WHERE generation_key=? AND attempt_key=?",
        duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value]).map_err(build_analysis_error)?;
    transaction.execute("INSERT INTO \"namespace-binding-round-results\"
        WITH ranked AS (
            SELECT DISTINCT *,max(priority) OVER (PARTITION BY generation_key,attempt_key,scope_key,name,namespace) AS selected_priority
            FROM \"namespace-binding-round-inputs\" WHERE generation_key=? AND attempt_key=?
        ), selected AS (
            SELECT generation_key,attempt_key,scope_key,name,namespace,target_key,priority,evidence_key
            FROM ranked WHERE priority=selected_priority
        ) SELECT *,CASE WHEN count(DISTINCT target_key) OVER (
            PARTITION BY generation_key,attempt_key,scope_key,name,namespace)>1
            THEN 'ambiguous' ELSE 'resolved' END FROM selected",
        duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value]).map_err(build_analysis_error)?;
    let mut hasher = blake3::Hasher::new();
    {
        let mut statement = transaction.prepare("SELECT generation_key,attempt_key,scope_key,name,namespace,target_key,priority,evidence_key,state
            FROM \"namespace-binding-round-results\" WHERE generation_key=? AND attempt_key=?
            ORDER BY generation_key,attempt_key,scope_key,name,namespace,target_key,evidence_key")
            .map_err(build_analysis_error)?;
        let mut rows = statement
            .query(duckdb::params![
                scope.run_generation_digest_value,
                scope.stage_attempt_sequence_value
            ])
            .map_err(build_analysis_error)?;
        while let Some(row) = rows.next().map_err(build_analysis_error)? {
            (0..9).try_for_each(|column| {
                let text = if column == 6 {
                    row.get::<_, i32>(column).map(|value| value.to_string())
                } else {
                    row.get::<_, String>(column)
                }
                .map_err(build_analysis_error)?;
                hasher.update(&(text.len() as u64).to_le_bytes());
                hasher.update(text.as_bytes());
                Ok::<_, StoreBackendFailureError>(())
            })?;
        }
    }
    transaction.commit().map_err(build_analysis_error)?;
    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    fn scope() -> StageRunScopeValue {
        StageRunScopeValue {
            run_generation_digest_value: "generation".into(),
            stage_attempt_sequence_value: "attempt".into(),
        }
    }

    fn connection() -> duckdb::Connection {
        let connection = duckdb::Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE \"namespace-binding-round-inputs\" (
            generation_key VARCHAR,attempt_key VARCHAR,scope_key VARCHAR,name VARCHAR,
            namespace VARCHAR,target_key VARCHAR,priority INTEGER,evidence_key VARCHAR);",
            )
            .unwrap();
        connection
    }

    fn insert(connection: &duckdb::Connection, target: &str, priority: i32, evidence: &str) {
        connection.execute("INSERT INTO \"namespace-binding-round-inputs\" VALUES ('generation','attempt','root','Chosen','type',?,?,?)",duckdb::params![target,priority,evidence]).unwrap();
    }

    fn rows(connection: &duckdb::Connection) -> Vec<(String, String)> {
        connection.prepare("SELECT target_key,state FROM \"namespace-binding-round-results\" WHERE generation_key='generation' AND attempt_key='attempt' ORDER BY generation_key,attempt_key,scope_key,name,namespace,target_key,evidence_key").unwrap()
            .query_map([],|row|Ok((row.get(0)?,row.get(1)?))).unwrap()
            .collect::<Result<Vec<_>,_>>().unwrap()
    }

    #[test]
    fn late_explicit_overrides_glob() {
        let connection = connection();
        insert(&connection, "earlier", 2, "glob");
        let first = recompute_binding_round(&connection, &scope()).unwrap();
        assert_eq!(
            rows(&connection),
            vec![("earlier".into(), "resolved".into())]
        );
        insert(&connection, "later", 3, "named");
        let second = recompute_binding_round(&connection, &scope()).unwrap();
        assert_ne!(first, second);
        assert_eq!(rows(&connection), vec![("later".into(), "resolved".into())]);
        assert_eq!(
            second,
            recompute_binding_round(&connection, &scope()).unwrap()
        );
        connection
            .execute(
                "DELETE FROM \"namespace-binding-round-inputs\" WHERE priority=3",
                [],
            )
            .unwrap();
        assert_eq!(
            first,
            recompute_binding_round(&connection, &scope()).unwrap()
        );
    }

    #[test]
    fn conflicting_glob_targets_ambiguous() {
        let connection = connection();
        insert(&connection, "left", 2, "left-glob");
        insert(&connection, "right", 2, "right-glob");
        recompute_binding_round(&connection, &scope()).unwrap();
        assert_eq!(
            rows(&connection),
            vec![
                ("left".into(), "ambiguous".into()),
                ("right".into(), "ambiguous".into())
            ]
        );
    }

    #[test]
    fn equal_targets_are_not_ambiguous() {
        let connection = connection();
        insert(&connection, "same", 2, "left-glob");
        insert(&connection, "same", 2, "right-glob");
        recompute_binding_round(&connection, &scope()).unwrap();
        assert!(
            rows(&connection)
                .iter()
                .all(|(_, state)| state == "resolved")
        );
    }

    #[test]
    fn empty_round_and_source_failure() {
        let connection = connection();
        let first = recompute_binding_round(&connection, &scope()).unwrap();
        assert!(rows(&connection).is_empty());
        assert_eq!(
            first,
            recompute_binding_round(&connection, &scope()).unwrap()
        );
        connection
            .execute_batch("DROP TABLE \"namespace-binding-round-inputs\"")
            .unwrap();
        assert!(recompute_binding_round(&connection, &scope()).is_err());
        assert!(rows(&connection).is_empty());
    }
}

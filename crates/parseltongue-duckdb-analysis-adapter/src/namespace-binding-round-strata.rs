use super::build_analysis_error;
use parseltongue_core_row_types::{StageRunScopeValue, StoreBackendFailureError};

pub(super) fn recompute_binding_round(
    _connection: &duckdb::Connection,
    _scope: &StageRunScopeValue,
) -> Result<String, StoreBackendFailureError> {
    Err(build_analysis_error(
        "full-set binding round not implemented",
    ))
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

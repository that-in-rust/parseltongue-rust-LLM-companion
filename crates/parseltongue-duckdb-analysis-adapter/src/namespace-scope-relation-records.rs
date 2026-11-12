use super::build_analysis_error;
use parseltongue_core_row_types::{StageRunScopeValue, StoreBackendFailureError};

pub fn materialize_scope_relations(
    _connection: &duckdb::Connection,
    _scope: &StageRunScopeValue,
) -> Result<(), StoreBackendFailureError> {
    Err(build_analysis_error("scope relation SQL not implemented"))
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
        connection.execute_batch("CREATE TABLE \"namespace-scope-context-records\" (
            generation_key VARCHAR,attempt_key VARCHAR,unit_key VARCHAR,scope_key VARCHAR,
            parent_scope VARCHAR,parent_lookup BOOLEAN,module_boundary BOOLEAN,prelude_scope VARCHAR);
            INSERT INTO \"namespace-scope-context-records\" VALUES
            ('generation','attempt','unit','root',NULL,false,true,NULL),
            ('generation','attempt','unit','module','root',false,true,NULL),
            ('generation','attempt','unit','body','module',true,false,NULL),
            ('generation','attempt','unit','block','body',true,false,NULL);")
            .unwrap();
        connection
    }

    fn ancestors(connection: &duckdb::Connection, source: &str, table: &str) -> Vec<String> {
        connection.prepare(&format!("SELECT ancestor_scope FROM \"{table}\" WHERE generation_key='generation' AND attempt_key='attempt' AND scope_key=? ORDER BY depth,ancestor_scope")).unwrap()
            .query_map([source],|row|row.get(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap()
    }

    #[test]
    fn body_scopes_chain_to_module_not_parent_module() {
        let connection = connection();
        materialize_scope_relations(&connection, &scope()).unwrap();
        assert_eq!(
            ancestors(&connection, "block", "namespace-lexical-parent-records"),
            vec!["block", "body", "module"]
        );
        assert_eq!(
            ancestors(&connection, "module", "namespace-lexical-parent-records"),
            vec!["module"]
        );
        assert_eq!(
            ancestors(&connection, "block", "namespace-scope-ancestor-records"),
            vec!["block", "body", "module", "root"]
        );
    }

    #[test]
    fn super_skips_body_scopes() {
        let connection = connection();
        materialize_scope_relations(&connection, &scope()).unwrap();
        let module: String = connection.query_row("SELECT module_scope FROM \"namespace-scope-module-records\" WHERE scope_key='block' ORDER BY generation_key,attempt_key,scope_key",[],|row|row.get(0)).unwrap();
        assert_eq!(module, "module");
        let parent: String = connection.query_row("SELECT parent_module FROM \"namespace-module-parent-records\" WHERE scope_key='module' ORDER BY generation_key,attempt_key,scope_key",[],|row|row.get(0)).unwrap();
        assert_eq!(parent, "root");
    }

    #[test]
    fn missing_and_cyclic_parents_rejected() {
        let connection = connection();
        connection.execute("UPDATE \"namespace-scope-context-records\" SET parent_scope='absent' WHERE scope_key='body'",[]).unwrap();
        assert!(materialize_scope_relations(&connection, &scope()).is_err());
        connection.execute("UPDATE \"namespace-scope-context-records\" SET parent_scope='block' WHERE scope_key='body'",[]).unwrap();
        assert!(materialize_scope_relations(&connection, &scope()).is_err());
    }

    #[test]
    fn empty_scope_set_is_valid() {
        let connection = connection();
        connection
            .execute("DELETE FROM \"namespace-scope-context-records\"", [])
            .unwrap();
        materialize_scope_relations(&connection, &scope()).unwrap();
        assert!(ancestors(&connection, "block", "namespace-scope-ancestor-records").is_empty());
    }
}

use super::build_analysis_error;
use parseltongue_core_row_types::{StageRunScopeValue, StoreBackendFailureError};

pub fn materialize_scope_relations(
    connection: &duckdb::Connection,
    scope: &StageRunScopeValue,
) -> Result<(), StoreBackendFailureError> {
    let transaction = connection
        .unchecked_transaction()
        .map_err(build_analysis_error)?;
    transaction
        .execute(
            "CREATE OR REPLACE TEMP TABLE \"namespace-active-scope-records\" AS
        SELECT * FROM \"namespace-scope-context-records\" WHERE generation_key=? AND attempt_key=?",
            duckdb::params![
                scope.run_generation_digest_value,
                scope.stage_attempt_sequence_value
            ],
        )
        .map_err(build_analysis_error)?;
    let missing: usize = transaction.query_row("SELECT count(*) FROM \"namespace-active-scope-records\" c
        LEFT JOIN \"namespace-active-scope-records\" p ON c.parent_scope=p.scope_key AND c.unit_key=p.unit_key
        WHERE c.parent_scope IS NOT NULL AND p.scope_key IS NULL",[],|row|row.get(0)).map_err(build_analysis_error)?;
    if missing != 0 {
        return Err(build_analysis_error(
            "StageInputMissingError: missing or cross-unit parent scope",
        ));
    }
    eprintln!("stage=namespace-resolution-query query=scope-cycle-check event=started");
    let cyclic: usize = transaction.query_row("WITH RECURSIVE walk AS (
        SELECT scope_key,scope_key AS current_scope,[scope_key] AS trail,false AS cyclic
        FROM \"namespace-active-scope-records\"
        UNION ALL
        SELECT w.scope_key,p.parent_scope,list_append(w.trail,p.parent_scope),list_contains(w.trail,p.parent_scope)
        FROM walk w JOIN \"namespace-active-scope-records\" p ON p.scope_key=w.current_scope
        WHERE NOT w.cyclic AND p.parent_scope IS NOT NULL
    ) SELECT count(*) FROM walk WHERE cyclic",[],|row|row.get(0)).map_err(build_analysis_error)?;
    eprintln!("stage=namespace-resolution-query query=scope-cycle-check event=complete");
    if cyclic != 0 {
        return Err(build_analysis_error(
            "StageInputMissingError: cyclic scope parents",
        ));
    }
    transaction.execute_batch("CREATE TABLE IF NOT EXISTS \"namespace-scope-ancestor-records\" (
        generation_key VARCHAR NOT NULL,attempt_key VARCHAR NOT NULL,scope_key VARCHAR NOT NULL,
        ancestor_scope VARCHAR NOT NULL,depth INTEGER NOT NULL,
        PRIMARY KEY(generation_key,attempt_key,scope_key,ancestor_scope));
        CREATE TABLE IF NOT EXISTS \"namespace-lexical-parent-records\" (
        generation_key VARCHAR NOT NULL,attempt_key VARCHAR NOT NULL,scope_key VARCHAR NOT NULL,
        ancestor_scope VARCHAR NOT NULL,depth INTEGER NOT NULL,
        PRIMARY KEY(generation_key,attempt_key,scope_key,ancestor_scope));
        CREATE TABLE IF NOT EXISTS \"namespace-scope-module-records\" (
        generation_key VARCHAR NOT NULL,attempt_key VARCHAR NOT NULL,scope_key VARCHAR NOT NULL,module_scope VARCHAR NOT NULL,
        PRIMARY KEY(generation_key,attempt_key,scope_key));
        CREATE TABLE IF NOT EXISTS \"namespace-module-parent-records\" (
        generation_key VARCHAR NOT NULL,attempt_key VARCHAR NOT NULL,scope_key VARCHAR NOT NULL,parent_module VARCHAR NOT NULL,
        PRIMARY KEY(generation_key,attempt_key,scope_key));").map_err(build_analysis_error)?;
    [
        "namespace-scope-ancestor-records",
        "namespace-lexical-parent-records",
        "namespace-scope-module-records",
        "namespace-module-parent-records",
    ]
    .into_iter()
    .try_for_each(|table| {
        transaction
            .execute(
                &format!("DELETE FROM \"{table}\" WHERE generation_key=? AND attempt_key=?"),
                duckdb::params![
                    scope.run_generation_digest_value,
                    scope.stage_attempt_sequence_value
                ],
            )
            .map_err(build_analysis_error)?;
        Ok::<_, StoreBackendFailureError>(())
    })?;
    transaction.execute_batch("INSERT INTO \"namespace-scope-ancestor-records\"
        WITH RECURSIVE ancestry AS (
            SELECT generation_key,attempt_key,scope_key,scope_key AS ancestor_scope,0 AS depth FROM \"namespace-active-scope-records\"
            UNION ALL
            SELECT a.generation_key,a.attempt_key,a.scope_key,p.parent_scope,a.depth+1
            FROM ancestry a JOIN \"namespace-active-scope-records\" p ON p.scope_key=a.ancestor_scope
            WHERE p.parent_scope IS NOT NULL
        ) SELECT * FROM ancestry;
        INSERT INTO \"namespace-lexical-parent-records\"
        WITH RECURSIVE lexical AS (
            SELECT generation_key,attempt_key,scope_key,scope_key AS ancestor_scope,0 AS depth FROM \"namespace-active-scope-records\"
            UNION ALL
            SELECT a.generation_key,a.attempt_key,a.scope_key,p.parent_scope,a.depth+1
            FROM lexical a JOIN \"namespace-active-scope-records\" p ON p.scope_key=a.ancestor_scope
            WHERE p.parent_scope IS NOT NULL AND p.parent_lookup
        ) SELECT * FROM lexical;
        INSERT INTO \"namespace-scope-module-records\"
            SELECT a.generation_key,a.attempt_key,a.scope_key,a.ancestor_scope FROM \"namespace-scope-ancestor-records\" a
            JOIN \"namespace-active-scope-records\" s ON s.scope_key=a.ancestor_scope
                AND s.generation_key=a.generation_key AND s.attempt_key=a.attempt_key
            WHERE s.module_boundary
            QUALIFY row_number() OVER (PARTITION BY a.generation_key,a.attempt_key,a.scope_key ORDER BY a.depth,a.ancestor_scope)=1;
        INSERT INTO \"namespace-module-parent-records\"
            SELECT a.generation_key,a.attempt_key,a.scope_key,a.ancestor_scope FROM \"namespace-scope-ancestor-records\" a
            JOIN \"namespace-active-scope-records\" s ON s.scope_key=a.ancestor_scope
                AND s.generation_key=a.generation_key AND s.attempt_key=a.attempt_key
            JOIN \"namespace-active-scope-records\" c ON c.scope_key=a.scope_key
                AND c.generation_key=a.generation_key AND c.attempt_key=a.attempt_key
            WHERE s.module_boundary AND c.module_boundary AND a.depth>0
            QUALIFY row_number() OVER (PARTITION BY a.generation_key,a.attempt_key,a.scope_key ORDER BY a.depth,a.ancestor_scope)=1;")
        .map_err(build_analysis_error)?;
    transaction.commit().map_err(build_analysis_error)?;
    Ok(())
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

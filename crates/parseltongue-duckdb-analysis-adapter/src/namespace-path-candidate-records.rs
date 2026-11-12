use super::build_analysis_error;
use parseltongue_core_row_types::{StageRunScopeValue, StoreBackendFailureError};

pub fn recompute_path_candidates(
    connection: &duckdb::Connection,
    scope: &StageRunScopeValue,
) -> Result<(), StoreBackendFailureError> {
    let transaction = connection
        .unchecked_transaction()
        .map_err(build_analysis_error)?;
    transaction
        .execute_batch(include_str!("namespace-frozen-input-schema.sql"))
        .map_err(build_analysis_error)?;
    let conflicts:usize=transaction.query_row("SELECT count(*) FROM (SELECT lookup_key,revision FROM \"namespace-legacy-scope-records\" WHERE generation_key=? AND attempt_key=? GROUP BY lookup_key,revision HAVING count(DISTINCT coalesce(target_key,'unresolved'))>1)",duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value],|row|row.get(0)).map_err(build_analysis_error)?;
    if conflicts > 0 {
        return Err(build_analysis_error("conflicting textual lookup revision"));
    }
    transaction.execute("CREATE OR REPLACE TEMP TABLE \"namespace-latest-legacy-lookups\" AS SELECT * FROM \"namespace-legacy-scope-records\" WHERE generation_key=? AND attempt_key=? QUALIFY row_number() OVER (PARTITION BY generation_key,attempt_key,lookup_key ORDER BY revision DESC,request_key,name,target_key NULLS FIRST,source_order)=1",duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value]).map_err(build_analysis_error)?;
    transaction.execute_batch("CREATE TABLE IF NOT EXISTS \"namespace-binding-visibility-records\" (
        generation_key VARCHAR,attempt_key VARCHAR,scope_key VARCHAR,name VARCHAR,
        namespace VARCHAR,target_key VARCHAR,priority INTEGER,evidence_key VARCHAR,visibility_scope VARCHAR);")
        .map_err(build_analysis_error)?;
    transaction
        .execute(
            "CREATE OR REPLACE TEMP TABLE \"namespace-active-path-requests\" AS
        SELECT * FROM \"namespace-path-source-requests\" WHERE generation_key=? AND attempt_key=?",
            duckdb::params![
                scope.run_generation_digest_value,
                scope.stage_attempt_sequence_value
            ],
        )
        .map_err(build_analysis_error)?;
    let missing: usize = transaction.query_row("SELECT count(*) FROM \"namespace-active-path-requests\" r
        LEFT JOIN \"namespace-scope-context-records\" s ON s.generation_key=r.generation_key AND s.attempt_key=r.attempt_key AND s.scope_key=r.scope_key
        WHERE s.scope_key IS NULL",[],|row|row.get(0)).map_err(build_analysis_error)?;
    if missing != 0 {
        return Err(build_analysis_error(
            "StageInputMissingError: path lexical scope absent",
        ));
    }
    transaction.execute_batch("CREATE TABLE IF NOT EXISTS \"namespace-path-candidate-records\" (
        generation_key VARCHAR NOT NULL,attempt_key VARCHAR NOT NULL,request_key VARCHAR NOT NULL,
        target_key VARCHAR NOT NULL,namespace VARCHAR NOT NULL,binding_scope VARCHAR NOT NULL,binding_name VARCHAR NOT NULL,state VARCHAR NOT NULL,
        PRIMARY KEY(generation_key,attempt_key,request_key,target_key,namespace,binding_scope,binding_name));")
        .map_err(build_analysis_error)?;
    transaction.execute("DELETE FROM \"namespace-path-candidate-records\" WHERE generation_key=? AND attempt_key=?",
        duckdb::params![scope.run_generation_digest_value,scope.stage_attempt_sequence_value]).map_err(build_analysis_error)?;
    eprintln!("stage=namespace-path-derivation event=started total_and_art_index_bytes={:?}",
        transaction.query_row("SELECT coalesce(sum(memory_usage_bytes),0)::UBIGINT,coalesce(sum(memory_usage_bytes) FILTER (WHERE tag='ART_INDEX'),0)::UBIGINT FROM duckdb_memory()",[],|row| Ok((row.get::<_,u64>(0)?,row.get::<_,u64>(1)?))));
    transaction
        .execute_batch(&format!(
            "INSERT INTO \"namespace-path-candidate-records\" {}",
            include_str!("namespace-path-candidate-derivation.sql")
        ))
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
            ('generation','attempt','unit','root',NULL,false,true,'prelude'),
            ('generation','attempt','unit','module','root',false,true,'prelude'),
            ('generation','attempt','unit','body','module',true,false,'prelude'),
            ('generation','attempt','unit','prelude',NULL,false,false,NULL),
            ('generation','attempt','unit','extern',NULL,false,false,NULL),
            ('generation','attempt','dependency','dep',NULL,false,true,NULL);
            CREATE TABLE \"namespace-binding-round-results\" (
            generation_key VARCHAR,attempt_key VARCHAR,scope_key VARCHAR,name VARCHAR,
            namespace VARCHAR,target_key VARCHAR,priority INTEGER,evidence_key VARCHAR,state VARCHAR);
            INSERT INTO \"namespace-binding-round-results\" VALUES
            ('generation','attempt','root','RootItem','type','root-item',4,'','resolved'),
            ('generation','attempt','root','module','type','module-target',4,'','resolved'),
            ('generation','attempt','module','Local','type','local-item',4,'','resolved'),
            ('generation','attempt','module','Hidden','type','hidden-item',4,'','resolved'),
            ('generation','attempt','module','Local','value','local-value',4,'','resolved'),
            ('generation','attempt','prelude','u8','type','primitive',0,'','resolved'),
            ('generation','attempt','extern','alias','type','dep-root',4,'','resolved'),
            ('generation','attempt','dep','Foreign','type','foreign-item',4,'','resolved');
            CREATE TABLE \"namespace-target-context-records\" (
            generation_key VARCHAR,attempt_key VARCHAR,target_key VARCHAR,child_scope VARCHAR,
            visibility_scope VARCHAR,scope_identity BOOLEAN);
            INSERT INTO \"namespace-target-context-records\" VALUES
            ('generation','attempt','root-target','root',NULL,true),
            ('generation','attempt','module-target','module',NULL,true),
            ('generation','attempt','root-item',NULL,NULL,false),
            ('generation','attempt','local-item',NULL,NULL,false),
            ('generation','attempt','hidden-item',NULL,'module',false),
            ('generation','attempt','local-value',NULL,NULL,false),
            ('generation','attempt','primitive',NULL,NULL,false),
            ('generation','attempt','dep-root','dep',NULL,true),
            ('generation','attempt','foreign-item',NULL,NULL,false);
            CREATE TABLE \"namespace-path-source-requests\" (
            generation_key VARCHAR,attempt_key VARCHAR,request_key VARCHAR,scope_key VARCHAR,
            unit_root VARCHAR,extern_scope VARCHAR,edition INTEGER,source_kind VARCHAR,namespace_filter VARCHAR,
            segment_count INTEGER,unavailable_reason VARCHAR);
            CREATE TABLE \"namespace-path-segment-records\" (
            generation_key VARCHAR,attempt_key VARCHAR,request_key VARCHAR,position INTEGER,
            segment_kind VARCHAR,segment_text VARCHAR);")
            .unwrap();
        crate::materialize_scope_relations(&connection, &scope()).unwrap();
        connection
    }

    fn request(
        connection: &duckdb::Connection,
        key: &str,
        lexical_scope: &str,
        parts: &[&str],
        edition: i32,
        source_kind: &str,
        filter: &str,
    ) {
        connection.execute("INSERT INTO \"namespace-path-source-requests\" VALUES ('generation','attempt',?,?,'root','extern',?,?,?, ?,NULL)",duckdb::params![key,lexical_scope,edition,source_kind,filter,parts.len() as i32]).unwrap();
        parts.iter().enumerate().for_each(|(position,text)| {
            let kind = match *text { "crate"=>"crate", "self"=>"self", "super"=>"super", "::"=>"global", _=>"name" };
            connection.execute("INSERT INTO \"namespace-path-segment-records\" VALUES ('generation','attempt',?,?,?,?)",duckdb::params![key,position as i32,kind,*text]).unwrap();
        });
    }

    fn targets(connection: &duckdb::Connection, key: &str) -> Vec<String> {
        connection.prepare("SELECT DISTINCT target_key FROM \"namespace-path-candidate-records\" WHERE request_key=? ORDER BY target_key").unwrap()
            .query_map([key],|row|row.get(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap()
    }

    #[test]
    fn crate_self_super_and_body_scope_roots() {
        let connection = connection();
        request(
            &connection,
            "crate",
            "body",
            &["crate", "RootItem"],
            2024,
            "signature",
            "type",
        );
        request(
            &connection,
            "self",
            "body",
            &["self", "Local"],
            2024,
            "signature",
            "type",
        );
        request(
            &connection,
            "super",
            "body",
            &["super", "RootItem"],
            2024,
            "signature",
            "type",
        );
        request(
            &connection,
            "body",
            "body",
            &["Local"],
            2024,
            "signature",
            "type",
        );
        recompute_path_candidates(&connection, &scope()).unwrap();
        assert_eq!(targets(&connection, "crate"), vec!["root-item"]);
        assert_eq!(targets(&connection, "self"), vec!["local-item"]);
        assert_eq!(targets(&connection, "super"), vec!["root-item"]);
        assert_eq!(targets(&connection, "body"), vec!["local-item"]);
    }

    #[test]
    fn edition_and_extern_alias_paths() {
        let connection = connection();
        request(
            &connection,
            "modern",
            "module",
            &["alias", "Foreign"],
            2018,
            "signature",
            "type",
        );
        request(
            &connection,
            "global",
            "module",
            &["::", "alias", "Foreign"],
            2018,
            "signature",
            "type",
        );
        request(
            &connection,
            "old",
            "module",
            &["alias", "Foreign"],
            2015,
            "signature",
            "type",
        );
        request(
            &connection,
            "old-import",
            "module",
            &["RootItem"],
            2015,
            "import",
            "type",
        );
        recompute_path_candidates(&connection, &scope()).unwrap();
        assert_eq!(targets(&connection, "modern"), vec!["foreign-item"]);
        assert_eq!(targets(&connection, "global"), vec!["foreign-item"]);
        assert!(targets(&connection, "old").is_empty());
        assert_eq!(targets(&connection, "old-import"), vec!["root-item"]);
    }

    #[test]
    fn prelude_raw_names_and_grouped_self() {
        let connection = connection();
        request(
            &connection,
            "prelude",
            "module",
            &["u8"],
            2024,
            "signature",
            "type",
        );
        request(
            &connection,
            "qualified",
            "module",
            &["self", "u8"],
            2024,
            "signature",
            "type",
        );
        request(
            &connection,
            "raw",
            "module",
            &["r#Local"],
            2024,
            "signature",
            "type",
        );
        request(
            &connection,
            "grouped",
            "module",
            &["Local", "self"],
            2024,
            "import",
            "any",
        );
        recompute_path_candidates(&connection, &scope()).unwrap();
        assert_eq!(targets(&connection, "prelude"), vec!["primitive"]);
        assert!(targets(&connection, "qualified").is_empty());
        assert_eq!(targets(&connection, "raw"), vec!["local-item"]);
        assert_eq!(targets(&connection, "grouped"), vec!["local-item"]);
    }

    #[test]
    fn restricted_visibility_and_disabled_prelude() {
        let connection = connection();
        connection.execute("UPDATE \"namespace-scope-context-records\" SET prelude_scope=NULL WHERE scope_key='body'",[]).unwrap();
        request(
            &connection,
            "hidden",
            "root",
            &["module", "Hidden"],
            2024,
            "signature",
            "type",
        );
        request(
            &connection,
            "visible",
            "body",
            &["Hidden"],
            2024,
            "signature",
            "type",
        );
        request(
            &connection,
            "disabled",
            "body",
            &["u8"],
            2024,
            "signature",
            "type",
        );
        recompute_path_candidates(&connection, &scope()).unwrap();
        assert!(targets(&connection, "hidden").is_empty());
        assert_eq!(targets(&connection, "visible"), vec!["hidden-item"]);
        assert!(targets(&connection, "disabled").is_empty());
    }

    #[test]
    fn empty_and_invalid_path_scopes() {
        let connection = connection();
        recompute_path_candidates(&connection, &scope()).unwrap();
        assert!(targets(&connection, "absent").is_empty());
        request(
            &connection,
            "invalid",
            "absent",
            &["Local"],
            2024,
            "signature",
            "type",
        );
        assert!(recompute_path_candidates(&connection, &scope()).is_err());
    }
}

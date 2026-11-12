use super::build_analysis_error;
use parseltongue_core_row_types::{StageRunScopeValue, StoreBackendFailureError};

pub fn recompute_path_candidates(
    _connection: &duckdb::Connection,
    _scope: &StageRunScopeValue,
) -> Result<(), StoreBackendFailureError> {
    Err(build_analysis_error("path candidate SQL not implemented"))
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

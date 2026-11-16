WITH RECURSIVE "absent-sysroot-unit-scopes" AS (
    -- REQ-RUST-038.0: extern scopes with no `core` and no `std` unit in this generation.
    SELECT DISTINCT r.generation_key,r.attempt_key,r.extern_scope
    FROM "namespace-path-source-requests" r
    WHERE NOT EXISTS (
        SELECT 1 FROM "namespace-binding-declaration-records" d
        WHERE d.generation_key=r.generation_key AND d.attempt_key=r.attempt_key
            AND d.scope_key=r.extern_scope AND d.name IN ('core','std') AND d.namespace='type')
), "absent-sysroot-import-names" AS (
    -- REQ-RUST-038.1: names bound by a non-glob import whose own path starts at an absent sysroot crate.
    SELECT DISTINCT i.generation_key,i.attempt_key,i.importer_scope,
        coalesce(i.alias_name,(
            SELECT arg_max(l.segment_text,l.position) FROM "namespace-path-segment-records" l
            WHERE l.generation_key=i.generation_key AND l.attempt_key=i.attempt_key
                AND l.request_key=i.request_key AND l.segment_kind='name')) AS bound_name
    FROM "namespace-import-request-records" i
    JOIN "namespace-path-source-requests" q USING(generation_key,attempt_key,request_key)
    JOIN "absent-sysroot-unit-scopes" a ON a.generation_key=q.generation_key AND a.attempt_key=q.attempt_key
        AND a.extern_scope=q.extern_scope
    JOIN "namespace-path-segment-records" f ON f.generation_key=q.generation_key AND f.attempt_key=q.attempt_key
        AND f.request_key=q.request_key AND f.segment_kind='name'
        AND f.segment_text IN ('core','alloc','std','proc_macro','test')
        AND f.position=(SELECT count(*) FROM "namespace-path-segment-records" g
            WHERE g.generation_key=q.generation_key AND g.attempt_key=q.attempt_key
                AND g.request_key=q.request_key AND g.position=0 AND g.segment_kind='global')
    WHERE NOT i.is_glob
), "first-name-lookup-scopes" AS (
    -- Lexical scope chain of the first name of every non-import path in a unit whose sysroot is absent.
    -- REQ-RUST-053.0 N5: import paths are on the chain too, and each scope carries its distance from the site.
    SELECT r.generation_key,r.attempt_key,r.request_key,r.scope_key,n.segment_text AS name,r.segment_count,
        r.source_kind='import' AS import_flag,0 AS depth
    FROM "namespace-path-source-requests" r
    JOIN "absent-sysroot-unit-scopes" a ON a.generation_key=r.generation_key AND a.attempt_key=r.attempt_key
        AND a.extern_scope=r.extern_scope
    JOIN "namespace-path-segment-records" n ON n.generation_key=r.generation_key AND n.attempt_key=r.attempt_key
        AND n.request_key=r.request_key AND n.position=0 AND n.segment_kind='name'
    UNION
    SELECT c.generation_key,c.attempt_key,c.request_key,s.parent_scope,c.name,c.segment_count,c.import_flag,c.depth+1
    FROM "first-name-lookup-scopes" c
    JOIN "namespace-scope-context-records" s ON s.generation_key=c.generation_key AND s.attempt_key=c.attempt_key
        AND s.scope_key=c.scope_key
    WHERE s.parent_lookup AND s.parent_scope IS NOT NULL AND s.parent_scope<>s.scope_key AND c.depth<256
), "absent-sysroot-name-requests" AS (
    -- Case 1: an enclosing scope imports the name from an absent sysroot crate.
    -- Case 2: a one-segment name that no scope on the chain declares, ending at the standard prelude.
    SELECT c.generation_key,c.attempt_key,c.request_key
    FROM "first-name-lookup-scopes" c
    JOIN "namespace-scope-context-records" s ON s.generation_key=c.generation_key AND s.attempt_key=c.attempt_key
        AND s.scope_key=c.scope_key
    WHERE NOT c.import_flag
    GROUP BY c.generation_key,c.attempt_key,c.request_key
    HAVING bool_or(EXISTS (
            SELECT 1 FROM "absent-sysroot-import-names" m
            WHERE m.generation_key=c.generation_key AND m.attempt_key=c.attempt_key
                AND m.importer_scope=c.scope_key AND m.bound_name=c.name))
        OR (max(c.segment_count)=1 AND bool_or(s.prelude_scope IS NOT NULL) AND NOT bool_or(EXISTS (
            SELECT 1 FROM "namespace-binding-declaration-records" d
            WHERE d.generation_key=c.generation_key AND d.attempt_key=c.attempt_key
                AND d.scope_key=c.scope_key AND d.name=c.name)))
), "written-request-path-texts" AS (
    -- REQ-RUST-053.0 N3: the names a request writes, joined. A leading global and the trailing
    -- `self` of an import are not names of the item; any other segment that is not a name gives no text.
    SELECT l.generation_key,l.attempt_key,l.request_key,
        string_agg(l.segment_text,'::' ORDER BY l.position) FILTER (WHERE l.segment_kind='name') AS path_text,
        string_agg(l.segment_text,'::' ORDER BY l.position)
            FILTER (WHERE l.segment_kind='name' AND l.position>0) AS tail_text,
        min(l.position) FILTER (WHERE l.segment_kind='name') AS first_name_position,
        bool_and(l.segment_kind='name' OR (l.segment_kind='global' AND l.position=0)
            OR (l.segment_kind='self' AND l.position>0 AND l.position=t.last_position)) AS plain_flag
    FROM "namespace-path-segment-records" l
    JOIN (SELECT generation_key,attempt_key,request_key,max(position) AS last_position
        FROM "namespace-path-segment-records" GROUP BY generation_key,attempt_key,request_key) t
        USING(generation_key,attempt_key,request_key)
    GROUP BY l.generation_key,l.attempt_key,l.request_key
), "scope-name-binder-records" AS (
    -- REQ-RUST-053.0 N5: what can bind a name in a scope ahead of a prelude: an item declared
    -- there, or a non-glob import, by its alias or its last name.
    SELECT d.generation_key,d.attempt_key,d.scope_key,d.name,d.target_key AS binder_key
    FROM "namespace-binding-declaration-records" d WHERE d.priority>=2
    UNION
    SELECT i.generation_key,i.attempt_key,i.importer_scope,
        coalesce(i.alias_name,(
            SELECT arg_max(l.segment_text,l.position) FROM "namespace-path-segment-records" l
            WHERE l.generation_key=i.generation_key AND l.attempt_key=i.attempt_key
                AND l.request_key=i.request_key AND l.segment_kind='name')),i.request_key
    FROM "namespace-import-request-records" i WHERE NOT i.is_glob AND i.request_kind='import'
), "glob-import-scope-records" AS (
    -- A bound's request for its trait is stored as a glob too; it puts no name in lexical scope.
    SELECT DISTINCT generation_key,attempt_key,importer_scope AS scope_key
    FROM "namespace-import-request-records" WHERE is_glob AND request_kind='import'
), "crate-rooted-request-names" AS (
    -- Proof 1: the path starts at the name of a sysroot crate with no unit in this generation.
    -- Without a leading `::`, no scope on the chain may bind that name or hold a glob import.
    SELECT r.generation_key,r.attempt_key,r.request_key,w.path_text,h.segment_text AS crate_name
    FROM "namespace-path-source-requests" r
    JOIN "written-request-path-texts" w USING(generation_key,attempt_key,request_key)
    JOIN "namespace-path-segment-records" h ON h.generation_key=r.generation_key AND h.attempt_key=r.attempt_key
        AND h.request_key=r.request_key AND h.position=w.first_name_position AND h.segment_kind='name'
        AND h.segment_text IN ('core','alloc','std','proc_macro','test')
    WHERE w.plain_flag AND NOT EXISTS (
            SELECT 1 FROM "namespace-binding-declaration-records" d
            WHERE d.generation_key=r.generation_key AND d.attempt_key=r.attempt_key
                AND d.scope_key=r.extern_scope AND d.name=h.segment_text AND d.namespace='type')
        AND (w.first_name_position=1 OR (
            EXISTS (SELECT 1 FROM "first-name-lookup-scopes" c
                WHERE c.generation_key=r.generation_key AND c.attempt_key=r.attempt_key
                    AND c.request_key=r.request_key AND c.depth=0)
            AND NOT EXISTS (SELECT 1 FROM "first-name-lookup-scopes" c
                WHERE c.generation_key=r.generation_key AND c.attempt_key=r.attempt_key
                    AND c.request_key=r.request_key AND (
                    EXISTS (SELECT 1 FROM "scope-name-binder-records" b
                        WHERE b.generation_key=c.generation_key AND b.attempt_key=c.attempt_key
                            AND b.scope_key=c.scope_key AND b.name=c.name AND b.binder_key<>r.request_key)
                    OR EXISTS (SELECT 1 FROM "glob-import-scope-records" g
                        WHERE g.generation_key=c.generation_key AND g.attempt_key=c.attempt_key
                            AND g.scope_key=c.scope_key)))))
), "import-composed-request-names" AS (
    -- Proof 2: the first name is bound by one `use` item whose own path has proof 1. That import is
    -- the only binder of the name in its scope, and no nearer scope binds the name or holds a glob import.
    SELECT c.generation_key,c.attempt_key,c.request_key,
        m.path_text||coalesce('::'||w.tail_text,'') AS path_text,m.crate_name,m.request_key AS import_request_key
    FROM "first-name-lookup-scopes" c
    JOIN "written-request-path-texts" w USING(generation_key,attempt_key,request_key)
    JOIN "scope-name-binder-records" b ON b.generation_key=c.generation_key AND b.attempt_key=c.attempt_key
        AND b.scope_key=c.scope_key AND b.name=c.name
    JOIN "crate-rooted-request-names" m ON m.generation_key=b.generation_key AND m.attempt_key=b.attempt_key
        AND m.request_key=b.binder_key
    WHERE NOT c.import_flag AND w.plain_flag AND w.first_name_position=0
        AND 1=(SELECT count(DISTINCT o.binder_key) FROM "scope-name-binder-records" o
            WHERE o.generation_key=c.generation_key AND o.attempt_key=c.attempt_key
                AND o.scope_key=c.scope_key AND o.name=c.name)
        AND NOT EXISTS (SELECT 1 FROM "first-name-lookup-scopes" n
            WHERE n.generation_key=c.generation_key AND n.attempt_key=c.attempt_key
                AND n.request_key=c.request_key AND n.depth<c.depth AND (
                EXISTS (SELECT 1 FROM "scope-name-binder-records" o
                    WHERE o.generation_key=n.generation_key AND o.attempt_key=n.attempt_key
                        AND o.scope_key=n.scope_key AND o.name=n.name)
                OR EXISTS (SELECT 1 FROM "glob-import-scope-records" g
                    WHERE g.generation_key=n.generation_key AND g.attempt_key=n.attempt_key
                        AND g.scope_key=n.scope_key)))
), "prelude-only-request-names" AS (
    -- Proof 3: one name that no scope on the chain declares or imports, with no glob import on
    -- the chain, in a unit with an implicit prelude.
    SELECT c.generation_key,c.attempt_key,c.request_key,any_value(c.name) AS path_text
    FROM "first-name-lookup-scopes" c
    JOIN "written-request-path-texts" w USING(generation_key,attempt_key,request_key)
    JOIN "namespace-scope-context-records" s ON s.generation_key=c.generation_key AND s.attempt_key=c.attempt_key
        AND s.scope_key=c.scope_key
    WHERE NOT c.import_flag AND c.segment_count=1 AND w.plain_flag
    GROUP BY c.generation_key,c.attempt_key,c.request_key
    HAVING bool_or(s.prelude_scope IS NOT NULL)
        AND NOT bool_or(EXISTS (
            SELECT 1 FROM "namespace-binding-declaration-records" d
            WHERE d.generation_key=c.generation_key AND d.attempt_key=c.attempt_key
                AND d.scope_key=c.scope_key AND d.name=c.name))
        AND NOT bool_or(EXISTS (
            SELECT 1 FROM "scope-name-binder-records" b
            WHERE b.generation_key=c.generation_key AND b.attempt_key=c.attempt_key
                AND b.scope_key=c.scope_key AND b.name=c.name))
        AND NOT bool_or(EXISTS (
            SELECT 1 FROM "glob-import-scope-records" g
            WHERE g.generation_key=c.generation_key AND g.attempt_key=c.attempt_key
                AND g.scope_key=c.scope_key))
)
SELECT DISTINCT r.request_key,p.metadata,r.unavailable_reason,c.state,m.metadata,c.namespace,
    CASE WHEN (h.segment_text IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM "namespace-binding-declaration-records" d
        WHERE d.generation_key=r.generation_key AND d.attempt_key=r.attempt_key
            AND d.scope_key=r.extern_scope AND d.name=h.segment_text AND d.namespace='type'))
        OR x.request_key IS NOT NULL
        THEN 'external_source_missing' END AS absent_sysroot_reason,
    -- REQ-RUST-049.0 R1: the request is a `use` import (not a bound, not the prelude); its outcome is also one import row.
    i.request_key IS NOT NULL AS import_request_flag,
    -- REQ-RUST-053.0 N3: the external name the read source proves, by the first proof that holds.
    CASE WHEN e1.request_key IS NOT NULL OR e2.request_key IS NOT NULL THEN 'crate_rooted_path'
        WHEN e3.request_key IS NOT NULL THEN 'prelude_name' END AS external_name_kind,
    coalesce(e1.crate_name,e2.crate_name) AS external_crate_name,
    coalesce(e1.path_text,e2.path_text,e3.path_text) AS external_item_path_text,
    CASE WHEN e1.request_key IS NOT NULL THEN 'path_root_names_crate'
        WHEN e2.request_key IS NOT NULL THEN 'use_import_names_path'
        WHEN e3.request_key IS NOT NULL THEN 'implicit_prelude_name' END AS external_site_proof_kind,
    CASE WHEN e1.request_key IS NULL THEN e2.import_request_key END AS external_import_request_key
FROM "namespace-path-source-requests" r
LEFT JOIN "namespace-path-metadata-records" p USING(generation_key,attempt_key,request_key)
JOIN "namespace-scope-context-records" s ON s.generation_key=r.generation_key AND s.attempt_key=r.attempt_key AND s.scope_key=r.scope_key
JOIN "namespace-mapped-output-contexts" u ON u.generation_key=s.generation_key AND u.attempt_key=s.attempt_key AND u.unit_key=s.unit_key
LEFT JOIN "namespace-path-candidate-records" c ON c.generation_key=r.generation_key AND c.attempt_key=r.attempt_key AND c.request_key=r.request_key
LEFT JOIN "namespace-output-target-metadata" m ON m.generation_key=c.generation_key AND m.attempt_key=c.attempt_key AND m.target_key=c.target_key
LEFT JOIN "namespace-path-segment-records" g ON g.generation_key=r.generation_key AND g.attempt_key=r.attempt_key AND g.request_key=r.request_key
    AND g.position=0 AND g.segment_kind='global'
LEFT JOIN "namespace-path-segment-records" h ON h.generation_key=r.generation_key AND h.attempt_key=r.attempt_key AND h.request_key=r.request_key
    AND h.position=CASE WHEN g.request_key IS NULL THEN 0 ELSE 1 END AND h.segment_kind='name'
    AND h.segment_text IN ('core','alloc','std','proc_macro','test')
LEFT JOIN "absent-sysroot-name-requests" x ON x.generation_key=r.generation_key AND x.attempt_key=r.attempt_key AND x.request_key=r.request_key
LEFT JOIN "crate-rooted-request-names" e1 ON e1.generation_key=r.generation_key AND e1.attempt_key=r.attempt_key AND e1.request_key=r.request_key
LEFT JOIN "import-composed-request-names" e2 ON e2.generation_key=r.generation_key AND e2.attempt_key=r.attempt_key AND e2.request_key=r.request_key
LEFT JOIN "prelude-only-request-names" e3 ON e3.generation_key=r.generation_key AND e3.attempt_key=r.attempt_key AND e3.request_key=r.request_key
LEFT JOIN (SELECT DISTINCT generation_key,attempt_key,request_key FROM "namespace-import-request-records" WHERE request_kind='import') i
    ON i.generation_key=r.generation_key AND i.attempt_key=r.attempt_key AND i.request_key=r.request_key
WHERE p.metadata IS NOT NULL OR i.request_key IS NOT NULL
ORDER BY r.request_key,p.metadata,r.unavailable_reason NULLS FIRST,c.state NULLS FIRST,m.metadata NULLS FIRST,c.namespace NULLS FIRST

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
    SELECT r.generation_key,r.attempt_key,r.request_key,r.scope_key,n.segment_text AS name,r.segment_count
    FROM "namespace-path-source-requests" r
    JOIN "absent-sysroot-unit-scopes" a ON a.generation_key=r.generation_key AND a.attempt_key=r.attempt_key
        AND a.extern_scope=r.extern_scope
    JOIN "namespace-path-segment-records" n ON n.generation_key=r.generation_key AND n.attempt_key=r.attempt_key
        AND n.request_key=r.request_key AND n.position=0 AND n.segment_kind='name'
    WHERE r.source_kind<>'import'
    UNION
    SELECT c.generation_key,c.attempt_key,c.request_key,s.parent_scope,c.name,c.segment_count
    FROM "first-name-lookup-scopes" c
    JOIN "namespace-scope-context-records" s ON s.generation_key=c.generation_key AND s.attempt_key=c.attempt_key
        AND s.scope_key=c.scope_key
    WHERE s.parent_lookup AND s.parent_scope IS NOT NULL AND s.parent_scope<>s.scope_key
), "absent-sysroot-name-requests" AS (
    -- Case 1: an enclosing scope imports the name from an absent sysroot crate.
    -- Case 2: a one-segment name that no scope on the chain declares, ending at the standard prelude.
    SELECT c.generation_key,c.attempt_key,c.request_key
    FROM "first-name-lookup-scopes" c
    JOIN "namespace-scope-context-records" s ON s.generation_key=c.generation_key AND s.attempt_key=c.attempt_key
        AND s.scope_key=c.scope_key
    GROUP BY c.generation_key,c.attempt_key,c.request_key
    HAVING bool_or(EXISTS (
            SELECT 1 FROM "absent-sysroot-import-names" m
            WHERE m.generation_key=c.generation_key AND m.attempt_key=c.attempt_key
                AND m.importer_scope=c.scope_key AND m.bound_name=c.name))
        OR (max(c.segment_count)=1 AND bool_or(s.prelude_scope IS NOT NULL) AND NOT bool_or(EXISTS (
            SELECT 1 FROM "namespace-binding-declaration-records" d
            WHERE d.generation_key=c.generation_key AND d.attempt_key=c.attempt_key
                AND d.scope_key=c.scope_key AND d.name=c.name)))
)
SELECT DISTINCT r.request_key,p.metadata,r.unavailable_reason,c.state,m.metadata,c.namespace,
    CASE WHEN (h.segment_text IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM "namespace-binding-declaration-records" d
        WHERE d.generation_key=r.generation_key AND d.attempt_key=r.attempt_key
            AND d.scope_key=r.extern_scope AND d.name=h.segment_text AND d.namespace='type'))
        OR x.request_key IS NOT NULL
        THEN 'external_source_missing' END AS absent_sysroot_reason
FROM "namespace-path-source-requests" r
JOIN "namespace-path-metadata-records" p USING(generation_key,attempt_key,request_key)
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
ORDER BY r.request_key,p.metadata,r.unavailable_reason NULLS FIRST,c.state NULLS FIRST,m.metadata NULLS FIRST,c.namespace NULLS FIRST

WITH RECURSIVE seeds AS (
    SELECT r.request_key,l.ancestor_scope AS current_scope,l.depth AS lookup_rank
    FROM "namespace-active-path-requests" r
    JOIN "namespace-path-segment-records" s ON s.generation_key=r.generation_key
        AND s.attempt_key=r.attempt_key AND s.request_key=r.request_key AND s.position=0
    JOIN "namespace-lexical-parent-records" l ON l.generation_key=r.generation_key
        AND l.attempt_key=r.attempt_key AND l.scope_key=r.scope_key
    WHERE s.segment_kind='name' AND NOT (r.edition=2015 AND r.source_kind='import')
    UNION ALL
    SELECT r.request_key,r.unit_root,0 FROM "namespace-active-path-requests" r
    JOIN "namespace-path-segment-records" s ON s.generation_key=r.generation_key
        AND s.attempt_key=r.attempt_key AND s.request_key=r.request_key AND s.position=0
    WHERE s.segment_kind='name' AND r.edition=2015 AND r.source_kind='import'
    UNION ALL
    SELECT r.request_key,c.prelude_scope,10000 FROM "namespace-active-path-requests" r
    JOIN "namespace-path-segment-records" s ON s.generation_key=r.generation_key
        AND s.attempt_key=r.attempt_key AND s.request_key=r.request_key AND s.position=0
    JOIN "namespace-scope-context-records" c ON c.generation_key=r.generation_key
        AND c.attempt_key=r.attempt_key AND c.scope_key=r.scope_key
    WHERE s.segment_kind='name' AND c.prelude_scope IS NOT NULL
        AND NOT (r.edition=2015 AND r.source_kind='import')
    UNION ALL
    SELECT r.request_key,r.extern_scope,20000 FROM "namespace-active-path-requests" r
    JOIN "namespace-path-segment-records" s ON s.generation_key=r.generation_key
        AND s.attempt_key=r.attempt_key AND s.request_key=r.request_key AND s.position=0
    WHERE s.segment_kind='name' AND r.edition<>2015 AND r.extern_scope IS NOT NULL
    UNION ALL
    SELECT r.request_key,r.scope_key,0 FROM "namespace-active-path-requests" r
    JOIN "namespace-path-segment-records" s ON s.generation_key=r.generation_key
        AND s.attempt_key=r.attempt_key AND s.request_key=r.request_key AND s.position=0
    WHERE s.segment_kind<>'name'
), walk AS (
    SELECT request_key,0 AS position,current_scope,lookup_rank,
        NULL::VARCHAR AS target_key,NULL::VARCHAR AS namespace,
        NULL::VARCHAR AS binding_scope,NULL::VARCHAR AS binding_name FROM seeds
    UNION ALL (
        SELECT w.request_key,w.position+1,t.child_scope,w.lookup_rank,
            b.target_key,b.namespace,b.scope_key,b.name
        FROM walk w JOIN "namespace-active-path-requests" r ON r.request_key=w.request_key
        JOIN "namespace-path-segment-records" s ON s.generation_key=r.generation_key
            AND s.attempt_key=r.attempt_key AND s.request_key=r.request_key AND s.position=w.position
        LEFT JOIN "namespace-path-segment-records" next ON next.generation_key=r.generation_key
            AND next.attempt_key=r.attempt_key AND next.request_key=r.request_key AND next.position=w.position+1
        JOIN "namespace-binding-round-results" b ON b.generation_key=r.generation_key
            AND b.attempt_key=r.attempt_key AND b.scope_key=w.current_scope
            AND b.name=regexp_replace(s.segment_text,'^r#','')
        JOIN "namespace-target-context-records" t ON t.generation_key=r.generation_key
            AND t.attempt_key=r.attempt_key AND t.target_key=b.target_key
        LEFT JOIN "namespace-binding-visibility-records" v ON v.generation_key=b.generation_key
            AND v.attempt_key=b.attempt_key AND v.scope_key=b.scope_key AND v.name=b.name
            AND v.namespace=b.namespace AND v.target_key=b.target_key AND v.priority=b.priority AND v.evidence_key=b.evidence_key
        CROSS JOIN LATERAL (SELECT CASE WHEN v.target_key IS NULL THEN t.visibility_scope ELSE v.visibility_scope END AS restriction_scope) visibility
        WHERE s.segment_kind='name' AND r.unavailable_reason IS NULL
            AND (visibility.restriction_scope IS NULL OR EXISTS (
                SELECT 1 FROM "namespace-scope-ancestor-records" a
                WHERE a.generation_key=r.generation_key AND a.attempt_key=r.attempt_key
                    AND a.scope_key=r.scope_key AND a.ancestor_scope=visibility.restriction_scope))
            AND CASE WHEN w.position+1<r.segment_count THEN b.namespace='type'
                ELSE r.namespace_filter='any' OR r.namespace_filter=b.namespace
                    OR (r.namespace_filter='type-or-value' AND b.namespace IN ('type','value')) END
        QUALIFY w.lookup_rank=min(w.lookup_rank) OVER (PARTITION BY w.request_key,w.position,b.namespace)
        UNION ALL
        SELECT w.request_key,w.position+1,chosen.current_scope,w.lookup_rank,
            CASE WHEN s.segment_kind='self' AND w.position>0 THEN w.target_key ELSE t.target_key END,
            'type',CASE WHEN s.segment_kind='self' AND w.position>0 THEN w.binding_scope ELSE chosen.current_scope END,
            CASE WHEN s.segment_kind='self' AND w.position>0 THEN w.binding_name ELSE s.segment_text END
        FROM walk w JOIN "namespace-active-path-requests" r ON r.request_key=w.request_key
        JOIN "namespace-path-segment-records" s ON s.generation_key=r.generation_key
            AND s.attempt_key=r.attempt_key AND s.request_key=r.request_key AND s.position=w.position
        LEFT JOIN "namespace-scope-module-records" m ON m.generation_key=r.generation_key
            AND m.attempt_key=r.attempt_key AND m.scope_key=w.current_scope
        LEFT JOIN "namespace-module-parent-records" p ON p.generation_key=r.generation_key
            AND p.attempt_key=r.attempt_key AND p.scope_key=m.module_scope
        CROSS JOIN LATERAL (SELECT CASE s.segment_kind
            WHEN 'crate' THEN r.unit_root WHEN 'global' THEN CASE WHEN r.edition=2015 THEN r.unit_root ELSE r.extern_scope END
            WHEN 'defining' THEN s.segment_text WHEN 'super' THEN p.parent_module
            WHEN 'self' THEN CASE WHEN w.position=0 THEN m.module_scope ELSE w.current_scope END
            ELSE NULL END AS current_scope) chosen
        LEFT JOIN "namespace-target-context-records" t ON t.generation_key=r.generation_key
            AND t.attempt_key=r.attempt_key AND t.child_scope=chosen.current_scope AND t.scope_identity
        WHERE s.segment_kind<>'name' AND r.unavailable_reason IS NULL
            AND ((s.segment_kind='self' AND w.position>0 AND w.namespace='type') OR t.target_key IS NOT NULL
                OR (w.position+1<r.segment_count AND chosen.current_scope IS NOT NULL))
    )
), finished AS (
    SELECT DISTINCT r.generation_key,r.attempt_key,w.request_key,w.target_key,w.namespace,w.binding_scope,w.binding_name
    FROM walk w JOIN "namespace-active-path-requests" r ON r.request_key=w.request_key
    WHERE w.position=r.segment_count AND w.target_key IS NOT NULL
        AND (r.namespace_filter='any' OR r.namespace_filter=w.namespace
            OR (r.namespace_filter='type-or-value' AND w.namespace IN ('type','value')))
)
SELECT *,CASE WHEN count(DISTINCT target_key) OVER (
    PARTITION BY generation_key,attempt_key,request_key,namespace)>1 THEN 'ambiguous' ELSE 'resolved' END AS state
FROM finished

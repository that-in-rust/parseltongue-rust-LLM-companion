SELECT b.scope_key,b.name,b.namespace,b.target_key,t.child_scope,b.priority,
    CASE WHEN v.target_key IS NULL THEN t.visibility_scope ELSE v.visibility_scope END AS visibility,
    list(DISTINCT b.evidence_key ORDER BY b.evidence_key),m.metadata
FROM "namespace-binding-round-results" b
JOIN "namespace-scope-context-records" s ON s.generation_key=b.generation_key AND s.attempt_key=b.attempt_key AND s.scope_key=b.scope_key
JOIN "namespace-mapped-output-contexts" u ON u.generation_key=s.generation_key AND u.attempt_key=s.attempt_key AND u.unit_key=s.unit_key
JOIN "namespace-target-context-records" t ON t.generation_key=b.generation_key AND t.attempt_key=b.attempt_key AND t.target_key=b.target_key
JOIN "namespace-output-target-metadata" m ON m.generation_key=b.generation_key AND m.attempt_key=b.attempt_key AND m.target_key=b.target_key
LEFT JOIN "namespace-binding-visibility-records" v ON v.generation_key=b.generation_key AND v.attempt_key=b.attempt_key AND v.scope_key=b.scope_key
    AND v.name=b.name AND v.namespace=b.namespace AND v.target_key=b.target_key AND v.priority=b.priority AND v.evidence_key=b.evidence_key
GROUP BY b.scope_key,b.name,b.namespace,b.target_key,t.child_scope,b.priority,visibility,m.metadata
ORDER BY b.scope_key,b.name,b.namespace,b.target_key,b.priority,t.child_scope NULLS FIRST,visibility NULLS FIRST,m.metadata

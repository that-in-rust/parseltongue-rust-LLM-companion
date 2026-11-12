SELECT c.binding_scope,c.binding_name,c.target_key,t.child_scope,coalesce(b.priority,4),
    CASE WHEN v.target_key IS NULL THEN t.visibility_scope ELSE v.visibility_scope END
FROM "namespace-path-candidate-records" c
JOIN "namespace-target-context-records" t
    ON t.generation_key=c.generation_key AND t.attempt_key=c.attempt_key AND t.target_key=c.target_key
LEFT JOIN "namespace-binding-round-results" b
    ON b.generation_key=c.generation_key AND b.attempt_key=c.attempt_key AND b.scope_key=c.binding_scope
    AND b.name=c.binding_name AND b.namespace=c.namespace AND b.target_key=c.target_key
LEFT JOIN "namespace-binding-visibility-records" v
    ON v.generation_key=b.generation_key AND v.attempt_key=b.attempt_key AND v.scope_key=b.scope_key
    AND v.name=b.name AND v.namespace=b.namespace AND v.target_key=b.target_key
    AND v.priority=b.priority AND v.evidence_key=b.evidence_key
WHERE c.generation_key=? AND c.attempt_key=? AND c.request_key=? AND c.namespace='macro' AND c.state='resolved'
ORDER BY c.generation_key,c.attempt_key,c.request_key,c.target_key,c.namespace,c.binding_scope,c.binding_name,
    t.child_scope NULLS FIRST,b.priority NULLS FIRST,b.evidence_key NULLS FIRST,v.visibility_scope NULLS FIRST
LIMIT 1

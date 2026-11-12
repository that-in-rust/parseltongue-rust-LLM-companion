SELECT DISTINCT r.request_key,p.metadata,r.unavailable_reason,c.state,m.metadata,c.namespace
FROM "namespace-path-source-requests" r
JOIN "namespace-path-metadata-records" p USING(generation_key,attempt_key,request_key)
JOIN "namespace-scope-context-records" s ON s.generation_key=r.generation_key AND s.attempt_key=r.attempt_key AND s.scope_key=r.scope_key
JOIN "namespace-mapped-output-contexts" u ON u.generation_key=s.generation_key AND u.attempt_key=s.attempt_key AND u.unit_key=s.unit_key
LEFT JOIN "namespace-path-candidate-records" c ON c.generation_key=r.generation_key AND c.attempt_key=r.attempt_key AND c.request_key=r.request_key
LEFT JOIN "namespace-output-target-metadata" m ON m.generation_key=c.generation_key AND m.attempt_key=c.attempt_key AND m.target_key=c.target_key
ORDER BY r.request_key,p.metadata,r.unavailable_reason NULLS FIRST,c.state NULLS FIRST,m.metadata NULLS FIRST,c.namespace NULLS FIRST

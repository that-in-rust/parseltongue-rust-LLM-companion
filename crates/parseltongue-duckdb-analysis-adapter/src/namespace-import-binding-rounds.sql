DELETE FROM "namespace-binding-round-inputs";
INSERT INTO "namespace-binding-round-inputs"
SELECT * FROM "namespace-active-declaration-records";
INSERT INTO "namespace-binding-round-inputs"
SELECT i.generation_key,i.attempt_key,i.importer_scope,regexp_replace(coalesce(i.alias_name,c.binding_name),'^r#',''),c.namespace,c.target_key,3,i.request_key
FROM "namespace-active-import-requests" i JOIN "namespace-path-candidate-records" c
    ON c.generation_key=i.generation_key AND c.attempt_key=i.attempt_key AND c.request_key=i.request_key
WHERE NOT i.is_glob AND c.state='resolved' AND coalesce(i.alias_name,'')<>'_';
INSERT INTO "namespace-binding-round-inputs"
SELECT i.generation_key,i.attempt_key,i.importer_scope,b.name,b.namespace,b.target_key,2,i.request_key
FROM "namespace-active-import-requests" i JOIN "namespace-path-candidate-records" c
    ON c.generation_key=i.generation_key AND c.attempt_key=i.attempt_key AND c.request_key=i.request_key AND c.state='resolved'
JOIN "namespace-target-context-records" t ON t.generation_key=c.generation_key AND t.attempt_key=c.attempt_key AND t.target_key=c.target_key
JOIN "namespace-binding-round-results" b ON b.generation_key=t.generation_key AND b.attempt_key=t.attempt_key AND b.scope_key=t.child_scope
JOIN "namespace-target-context-records" v ON v.generation_key=b.generation_key AND v.attempt_key=b.attempt_key AND v.target_key=b.target_key
LEFT JOIN "namespace-binding-visibility-records" bv ON bv.generation_key=b.generation_key AND bv.attempt_key=b.attempt_key
    AND bv.scope_key=b.scope_key AND bv.name=b.name AND bv.namespace=b.namespace AND bv.target_key=b.target_key AND bv.priority=b.priority AND bv.evidence_key=b.evidence_key
CROSS JOIN LATERAL (SELECT CASE WHEN bv.target_key IS NULL THEN v.visibility_scope ELSE bv.visibility_scope END AS restriction_scope) visibility
WHERE i.is_glob AND (visibility.restriction_scope IS NULL OR EXISTS (
    SELECT 1 FROM "namespace-scope-ancestor-records" a WHERE a.generation_key=i.generation_key
        AND a.attempt_key=i.attempt_key AND a.scope_key=i.importer_scope AND a.ancestor_scope=visibility.restriction_scope));
INSERT INTO "namespace-binding-round-inputs"
SELECT p.generation_key,p.attempt_key,p.importer_scope,b.name,b.namespace,b.target_key,2,p.evidence_key
FROM "namespace-macro-prelude-records" p JOIN "namespace-binding-round-results" b
    ON b.generation_key=p.generation_key AND b.attempt_key=p.attempt_key AND b.scope_key=p.source_scope AND b.namespace='macro'
JOIN "namespace-target-context-records" t ON t.generation_key=b.generation_key AND t.attempt_key=b.attempt_key AND t.target_key=b.target_key
LEFT JOIN "namespace-binding-visibility-records" v ON v.generation_key=b.generation_key AND v.attempt_key=b.attempt_key
    AND v.scope_key=b.scope_key AND v.name=b.name AND v.namespace=b.namespace AND v.target_key=b.target_key AND v.priority=b.priority AND v.evidence_key=b.evidence_key
WHERE (CASE WHEN v.target_key IS NULL THEN t.visibility_scope ELSE v.visibility_scope END) IS NULL;
DELETE FROM "namespace-binding-visibility-records";
INSERT INTO "namespace-binding-visibility-records"
SELECT * FROM "namespace-active-declaration-visibility";
INSERT INTO "namespace-binding-visibility-records"
SELECT b.*,v.visibility_scope FROM "namespace-binding-round-inputs" b
JOIN "namespace-import-visibility-records" v ON b.generation_key=v.generation_key
    AND b.attempt_key=v.attempt_key AND b.evidence_key=v.request_key;
INSERT INTO "namespace-binding-visibility-records"
SELECT b.*,NULL FROM "namespace-binding-round-inputs" b JOIN "namespace-macro-prelude-records" p
    ON b.generation_key=p.generation_key AND b.attempt_key=p.attempt_key AND b.evidence_key=p.evidence_key;

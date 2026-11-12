CREATE TABLE IF NOT EXISTS "namespace-unit-source-records" (
 generation_key VARCHAR,attempt_key VARCHAR,unit_key VARCHAR,cargo_key VARCHAR,
 root_scope VARCHAR,extern_scope VARCHAR,edition INTEGER,source_record VARCHAR);
CREATE TABLE IF NOT EXISTS "namespace-target-source-records" (
 generation_key VARCHAR,attempt_key VARCHAR,target_key VARCHAR,child_scope VARCHAR,
 metadata VARCHAR,scope_identity BOOLEAN);
CREATE TABLE IF NOT EXISTS "namespace-path-metadata-records" (
 generation_key VARCHAR,attempt_key VARCHAR,request_key VARCHAR,metadata VARCHAR);
CREATE TABLE IF NOT EXISTS "namespace-macro-source-records" (
 generation_key VARCHAR,attempt_key VARCHAR,request_key VARCHAR,scope_key VARCHAR,source_record VARCHAR);
CREATE TABLE IF NOT EXISTS "namespace-extern-source-records" (
 generation_key VARCHAR,attempt_key VARCHAR,source_record VARCHAR);
CREATE TABLE IF NOT EXISTS "namespace-legacy-scope-records" (
 generation_key VARCHAR,attempt_key VARCHAR,lookup_key VARCHAR,request_key VARCHAR,name VARCHAR,
 target_key VARCHAR,revision UBIGINT,source_order UBIGINT[]);
CREATE TABLE IF NOT EXISTS "namespace-normalized-source-manifests" (
 generation_key VARCHAR,attempt_key VARCHAR,source_rows UBIGINT,
 PRIMARY KEY(generation_key,attempt_key));

CREATE TABLE "namespace-scope-context-records" (
 generation_key VARCHAR,attempt_key VARCHAR,unit_key VARCHAR,scope_key VARCHAR,
 parent_scope VARCHAR,parent_lookup BOOLEAN,module_boundary BOOLEAN,prelude_scope VARCHAR);
CREATE TABLE "namespace-binding-declaration-records" (
 generation_key VARCHAR,attempt_key VARCHAR,scope_key VARCHAR,name VARCHAR,
 namespace VARCHAR,target_key VARCHAR,priority INTEGER,evidence_key VARCHAR);
CREATE TABLE "namespace-target-context-records" (
 generation_key VARCHAR,attempt_key VARCHAR,target_key VARCHAR,child_scope VARCHAR,
 visibility_scope VARCHAR,scope_identity BOOLEAN);
CREATE TABLE "namespace-path-source-requests" (
 generation_key VARCHAR,attempt_key VARCHAR,request_key VARCHAR,scope_key VARCHAR,
 unit_root VARCHAR,extern_scope VARCHAR,edition INTEGER,source_kind VARCHAR,namespace_filter VARCHAR,
 segment_count INTEGER,unavailable_reason VARCHAR);
CREATE TABLE "namespace-path-segment-records" (
 generation_key VARCHAR,attempt_key VARCHAR,request_key VARCHAR,position INTEGER,
 segment_kind VARCHAR,segment_text VARCHAR);
CREATE TABLE "namespace-import-request-records" (
 generation_key VARCHAR,attempt_key VARCHAR,request_key VARCHAR,importer_scope VARCHAR,
 alias_name VARCHAR,is_glob BOOLEAN);

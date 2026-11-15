-- REQ-RUST-051.0: each source-written macro call node, joined to the textual-scope lookup of
-- its macro path, to the definition node that lookup names, and to its expansion row.
CREATE OR REPLACE TEMP MACRO "macro-call-unit-text"(unit_value) AS concat_ws('|',
    json_extract_string(unit_value, '$.unit_cargo_package_identifier'),
    json_extract_string(unit_value, '$.unit_cargo_target_name'),
    json_extract_string(unit_value, '$.unit_cargo_target_kind'),
    json_extract_string(unit_value, '$.unit_compilation_role_kind'),
    json_extract_string(unit_value, '$.unit_configuration_digest_full'));

CREATE OR REPLACE TEMP VIEW "macro-call-node-spans" AS
SELECT candidate.*,
    "macro-call-unit-text"(json_extract(root_scope_payload, '$.scope_module_identity_key.scope_compilation_unit_key')) AS unit_text,
    json_extract_string(request_payload, '$.file_source_absolute_path') AS file_path,
    json_extract_string(unit_payload, '$.unit_package_root_path') AS package_root,
    json_extract_string(span_payload, '$.span_syntax_kind_value') AS syntax_kind,
    CAST(json_extract_string(span_payload, '$.span_byte_start_offset') AS UBIGINT) AS start_byte,
    CAST(json_extract_string(span_payload, '$.span_byte_end_offset') AS UBIGINT) AS end_byte
FROM "original-item-graph-candidates" candidate
WHERE json_extract_string(span_payload, '$.span_syntax_kind_value') IN ('MACRO_CALL', 'MACRO_RULES', 'MACRO_DEF');

-- Order kind 3 is the lookup of the path of a macro call; its site is that path. The latest
-- scope revision of a site is the one in force, as in the name resolver.
CREATE OR REPLACE TEMP VIEW "macro-call-scope-lookups" AS
SELECT DISTINCT generation_key, attempt_key,
    "macro-call-unit-text"(json_extract(payload, '$.legacy_source_site_key.unit_compilation_identity_key')) AS unit_text,
    json_extract_string(payload, '$.legacy_source_site_key.span_source_file_path') AS site_path,
    CAST(json_extract_string(payload, '$.legacy_source_site_key.span_byte_range_value[0]') AS UBIGINT) AS site_start,
    CAST(json_extract_string(payload, '$.legacy_source_site_key.span_byte_range_value[1]') AS UBIGINT) AS site_end,
    json_extract_string(payload, '$.legacy_macro_name_text') AS macro_name,
    json_extract_string(payload, '$.legacy_target_identity_key.target_identity_variant_kind') AS target_kind,
    "macro-call-unit-text"(json_extract(payload, '$.legacy_target_identity_key.target_semantic_occurrence_key.unit_compilation_identity_key')) AS target_unit_text,
    json_extract_string(payload, '$.legacy_target_identity_key.target_semantic_occurrence_key.span_source_file_path') AS target_path,
    CAST(json_extract_string(payload, '$.legacy_target_identity_key.target_semantic_occurrence_key.span_byte_range_value[0]') AS UBIGINT) AS target_start,
    CAST(json_extract_string(payload, '$.legacy_target_identity_key.target_semantic_occurrence_key.span_byte_range_value[1]') AS UBIGINT) AS target_end
FROM "frozen-generation-source-records"
WHERE table_name = 'legacy-macro-scope-records'
    AND json_extract_string(payload, '$.legacy_source_order_parts[1]') = '3'
    AND json_array_length(json_extract(payload, '$.legacy_source_site_key.span_expansion_provenance_chain')) = 0
QUALIFY rank() OVER (
    PARTITION BY generation_key, attempt_key, unit_text, site_path, site_start, site_end
    ORDER BY CAST(json_extract_string(payload, '$.legacy_scope_revision_value') AS UBIGINT) DESC) = 1;

CREATE OR REPLACE TEMP VIEW "macro-call-expands-candidates" AS
SELECT call.generation_key, call.attempt_key,
    call.unit_payload, call.root_scope_payload, call.request_payload, call.file_payload, call.span_payload,
    coalesce(lookup.macro_name, '') AS macro_name,
    coalesce(lookup.target_kind, '') AS target_kind,
    coalesce(definition.unit_payload, 'null') AS definition_unit_payload,
    coalesce(definition.root_scope_payload, 'null') AS definition_root_scope_payload,
    coalesce(definition.request_payload, 'null') AS definition_request_payload,
    coalesce(definition.file_payload, 'null') AS definition_file_payload,
    coalesce(definition.span_payload, 'null') AS definition_span_payload,
    coalesce(expansion.payload, 'null') AS expansion_payload
FROM "macro-call-node-spans" call
LEFT JOIN "macro-call-scope-lookups" lookup
    ON lookup.generation_key = call.generation_key AND lookup.attempt_key = call.attempt_key
    AND lookup.unit_text = call.unit_text
    AND call.file_path = call.package_root || '/' || lookup.site_path
    AND lookup.site_start >= call.start_byte AND lookup.site_end <= call.end_byte
LEFT JOIN "macro-call-node-spans" definition
    ON definition.generation_key = call.generation_key AND definition.attempt_key = call.attempt_key
    AND definition.syntax_kind IN ('MACRO_RULES', 'MACRO_DEF')
    AND definition.unit_text = lookup.target_unit_text
    AND definition.file_path = definition.package_root || '/' || lookup.target_path
    AND definition.start_byte = lookup.target_start AND definition.end_byte = lookup.target_end
LEFT JOIN "frozen-generation-source-records" expansion
    ON expansion.generation_key = call.generation_key AND expansion.attempt_key = call.attempt_key
    AND expansion.table_name = 'macro-expanded-item-table'
    AND json_array_length(json_extract(expansion.payload, '$.macro_invocation_identity_record.span_expansion_provenance_chain')) = 0
    AND "macro-call-unit-text"(json_extract(expansion.payload, '$.macro_invocation_identity_record.unit_compilation_identity_key')) = call.unit_text
    AND call.file_path = call.package_root || '/' ||
        json_extract_string(expansion.payload, '$.macro_invocation_identity_record.span_source_file_path')
    AND CAST(json_extract_string(expansion.payload, '$.macro_invocation_identity_record.span_byte_range_value[0]') AS UBIGINT) = call.start_byte
    AND CAST(json_extract_string(expansion.payload, '$.macro_invocation_identity_record.span_byte_range_value[1]') AS UBIGINT) = call.end_byte
WHERE call.syntax_kind = 'MACRO_CALL';

WITH frozen AS (
    SELECT table_name, payload
    FROM "frozen-generation-source-records"
    WHERE generation_key = ? AND attempt_key = ?
), candidates AS (
    SELECT
        json_extract_string(payload, '$.binding_lexical_scope_key') AS owner_scope,
        json_extract_string(payload, '$.binding_visibility_scope_kind') AS visibility
    FROM frozen
    WHERE table_name = 'syntax-namespace-input-items'
        AND json_extract_string(payload, '$.binding_target_identity_key.target_identity_variant_kind') = 'definition'
        AND json_extract_string(payload, '$.binding_target_identity_key.target_semantic_occurrence_key') = ?
    UNION ALL
    SELECT
        json_extract_string(payload, '$.import_lexical_scope_key') AS owner_scope,
        json_extract_string(payload, '$.import_visibility_scope_kind') AS visibility
    FROM frozen
    WHERE table_name = 'syntax-namespace-input-imports' AND ?
        AND json_extract_string(payload, '$.import_resolution_source_kind') = 'Import'
        AND json_extract_string(payload, '$.import_site_occurrence_key.unit_compilation_identity_key') = ?
        AND json_extract_string(payload, '$.import_site_occurrence_key.span_source_file_path') = ?
        AND json_array_length(json_extract(payload, '$.import_site_occurrence_key.span_expansion_provenance_chain')) = 0
        AND CAST(json_extract_string(payload, '$.import_site_occurrence_key.span_byte_range_value[0]') AS UBIGINT) >= ?
        AND CAST(json_extract_string(payload, '$.import_site_occurrence_key.span_byte_range_value[1]') AS UBIGINT) <= ?
        AND CAST(json_extract_string(payload, '$.import_site_occurrence_key.span_byte_range_value[0]') AS UBIGINT)
            < CAST(json_extract_string(payload, '$.import_site_occurrence_key.span_byte_range_value[1]') AS UBIGINT)
)
SELECT DISTINCT owner_scope, visibility
FROM candidates
JOIN frozen AS module ON module.table_name = 'syntax-namespace-input-scopes'
    AND json_extract_string(module.payload, '$.scope_module_identity_key') = owner_scope
    AND json_extract_string(module.payload, '$.scope_module_boundary_flag') = 'true'
ORDER BY owner_scope, visibility;

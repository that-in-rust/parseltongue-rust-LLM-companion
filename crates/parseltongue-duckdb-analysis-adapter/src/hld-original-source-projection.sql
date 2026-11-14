CREATE OR REPLACE TEMP VIEW "original-source-area-candidates" AS
SELECT
    unit.generation_key,
    unit.attempt_key,
    unit.payload AS unit_payload,
    root_scope.payload AS root_scope_payload,
    request.payload AS request_payload,
    file.payload AS file_payload
FROM "frozen-generation-source-records" AS unit
JOIN "frozen-generation-source-records" AS root_scope
    ON root_scope.generation_key = unit.generation_key
    AND root_scope.attempt_key = unit.attempt_key
    AND root_scope.table_name = 'syntax-namespace-input-scopes'
    AND json_extract_string(root_scope.payload, '$.scope_cargo_unit_key') =
        json_extract_string(unit.payload, '$.unit_compilation_identity_key')
    AND json_array_length(json_extract(root_scope.payload, '$.scope_module_identity_key.scope_semantic_path_parts')) = 0
JOIN "frozen-generation-source-records" AS request
    ON request.generation_key = unit.generation_key
    AND request.attempt_key = unit.attempt_key
    AND request.table_name = 'syntax-file-work-requests'
    AND json_extract_string(request.payload, '$.unit_compilation_identity_key') =
        json_extract_string(unit.payload, '$.unit_compilation_identity_key')
    AND json_extract_string(request.payload, '$.file_parse_context_kind') = 'items'
    AND json_array_length(json_extract(request.payload, '$.file_expansion_provenance_chain')) = 0
JOIN "frozen-generation-source-records" AS file
    ON file.generation_key = request.generation_key
    AND file.attempt_key = request.attempt_key
    AND file.table_name = 'file-source-text-table'
    AND json_extract_string(file.payload, '$.file_source_absolute_path') =
        json_extract_string(request.payload, '$.file_source_absolute_path')
WHERE unit.table_name = 'crate-dependency-graph-table'
    AND json_extract_string(unit.payload, '$.unit_mapped_target_flag') = 'true';

CREATE OR REPLACE TEMP VIEW "original-source-missing-files" AS
SELECT
    unit.generation_key,
    unit.attempt_key,
    request.payload AS request_payload
FROM "frozen-generation-source-records" AS unit
JOIN "frozen-generation-source-records" AS root_scope
    ON root_scope.generation_key = unit.generation_key
    AND root_scope.attempt_key = unit.attempt_key
    AND root_scope.table_name = 'syntax-namespace-input-scopes'
    AND json_extract_string(root_scope.payload, '$.scope_cargo_unit_key') =
        json_extract_string(unit.payload, '$.unit_compilation_identity_key')
    AND json_array_length(json_extract(root_scope.payload, '$.scope_module_identity_key.scope_semantic_path_parts')) = 0
JOIN "frozen-generation-source-records" AS request
    ON request.generation_key = unit.generation_key
    AND request.attempt_key = unit.attempt_key
    AND request.table_name = 'syntax-file-work-requests'
    AND json_extract_string(request.payload, '$.unit_compilation_identity_key') =
        json_extract_string(unit.payload, '$.unit_compilation_identity_key')
    AND json_extract_string(request.payload, '$.file_parse_context_kind') = 'items'
    AND json_array_length(json_extract(request.payload, '$.file_expansion_provenance_chain')) = 0
LEFT JOIN "frozen-generation-source-records" AS file
    ON file.generation_key = request.generation_key
    AND file.attempt_key = request.attempt_key
    AND file.table_name = 'file-source-text-table'
    AND json_extract_string(file.payload, '$.file_source_absolute_path') =
        json_extract_string(request.payload, '$.file_source_absolute_path')
WHERE unit.table_name = 'crate-dependency-graph-table'
    AND json_extract_string(unit.payload, '$.unit_mapped_target_flag') = 'true'
    AND file.payload IS NULL;

CREATE OR REPLACE TEMP VIEW "original-source-unowned-items" AS
SELECT
    area.generation_key,
    area.attempt_key,
    span.payload AS span_payload
FROM "original-source-area-candidates" AS area
JOIN "frozen-generation-source-records" AS span
    ON span.generation_key = area.generation_key
    AND span.attempt_key = area.attempt_key
    AND span.table_name = 'span-reference-text-table'
    AND json_extract_string(span.payload, '$.unit_compilation_identity_key') =
        json_extract_string(area.unit_payload, '$.unit_compilation_identity_key')
    AND json_extract_string(span.payload, '$.file_source_absolute_path') =
        json_extract_string(area.request_payload, '$.file_source_absolute_path')
    AND json_extract_string(span.payload, '$.span_configuration_active_flag') = 'true'
    AND json_array_length(json_extract(span.payload, '$.span_expansion_provenance_chain')) = 0
LEFT JOIN "frozen-generation-source-records" AS ownership
    ON ownership.generation_key = span.generation_key
    AND ownership.attempt_key = span.attempt_key
    AND ownership.table_name = 'span-byte-ownership-ranges'
    AND json_extract_string(ownership.payload, '$.unit_compilation_identity_key') =
        json_extract_string(span.payload, '$.unit_compilation_identity_key')
    AND json_extract_string(ownership.payload, '$.file_source_absolute_path') =
        json_extract_string(span.payload, '$.file_source_absolute_path')
    AND json_extract_string(ownership.payload, '$.span_byte_start_offset') =
        json_extract_string(span.payload, '$.span_byte_start_offset')
    AND json_extract_string(ownership.payload, '$.span_byte_end_offset') =
        json_extract_string(span.payload, '$.span_byte_end_offset')
    AND json_extract_string(ownership.payload, '$.span_gap_classification_kind') = 'item'
    AND json_array_length(json_extract(ownership.payload, '$.span_expansion_provenance_chain')) = 0
WHERE ownership.payload IS NULL
    AND NOT EXISTS (
        SELECT 1
        FROM "frozen-generation-source-records" AS parent
        WHERE parent.generation_key = span.generation_key
            AND parent.attempt_key = span.attempt_key
            AND parent.table_name = 'span-reference-text-table'
            AND json_extract_string(parent.payload, '$.unit_compilation_identity_key') =
                json_extract_string(span.payload, '$.unit_compilation_identity_key')
            AND json_extract_string(parent.payload, '$.file_source_absolute_path') =
                json_extract_string(span.payload, '$.file_source_absolute_path')
            AND json_array_length(json_extract(parent.payload, '$.span_expansion_provenance_chain')) = 0
            AND CAST(json_extract_string(parent.payload, '$.span_byte_start_offset') AS UBIGINT) <
                CAST(json_extract_string(span.payload, '$.span_byte_start_offset') AS UBIGINT)
            AND CAST(json_extract_string(parent.payload, '$.span_byte_end_offset') AS UBIGINT) >
                CAST(json_extract_string(span.payload, '$.span_byte_end_offset') AS UBIGINT)
    );

CREATE OR REPLACE TEMP VIEW "original-source-item-candidates" AS
SELECT
    area.generation_key,
    area.attempt_key,
    area.unit_payload,
    area.root_scope_payload,
    area.request_payload,
    area.file_payload,
    span.payload AS span_payload
FROM "original-source-area-candidates" AS area
JOIN "frozen-generation-source-records" AS span
    ON span.generation_key = area.generation_key
    AND span.attempt_key = area.attempt_key
    AND span.table_name = 'span-reference-text-table'
    AND json_extract_string(span.payload, '$.unit_compilation_identity_key') =
        json_extract_string(area.unit_payload, '$.unit_compilation_identity_key')
    AND json_extract_string(span.payload, '$.file_source_absolute_path') =
        json_extract_string(area.request_payload, '$.file_source_absolute_path')
    AND json_extract_string(span.payload, '$.span_configuration_active_flag') = 'true'
    AND json_array_length(json_extract(span.payload, '$.span_expansion_provenance_chain')) = 0
JOIN "frozen-generation-source-records" AS ownership
    ON ownership.generation_key = span.generation_key
    AND ownership.attempt_key = span.attempt_key
    AND ownership.table_name = 'span-byte-ownership-ranges'
    AND json_extract_string(ownership.payload, '$.unit_compilation_identity_key') =
        json_extract_string(span.payload, '$.unit_compilation_identity_key')
    AND json_extract_string(ownership.payload, '$.file_source_absolute_path') =
        json_extract_string(span.payload, '$.file_source_absolute_path')
    AND json_extract_string(ownership.payload, '$.span_byte_start_offset') =
        json_extract_string(span.payload, '$.span_byte_start_offset')
    AND json_extract_string(ownership.payload, '$.span_byte_end_offset') =
        json_extract_string(span.payload, '$.span_byte_end_offset')
    AND json_extract_string(ownership.payload, '$.span_gap_classification_kind') = 'item'
    AND json_array_length(json_extract(ownership.payload, '$.span_expansion_provenance_chain')) = 0
WHERE NOT EXISTS (
    SELECT 1
    FROM "frozen-generation-source-records" AS parent
    WHERE parent.generation_key = span.generation_key
        AND parent.attempt_key = span.attempt_key
        AND parent.table_name = 'span-reference-text-table'
        AND json_extract_string(parent.payload, '$.unit_compilation_identity_key') =
            json_extract_string(span.payload, '$.unit_compilation_identity_key')
        AND json_extract_string(parent.payload, '$.file_source_absolute_path') =
            json_extract_string(span.payload, '$.file_source_absolute_path')
        AND json_array_length(json_extract(parent.payload, '$.span_expansion_provenance_chain')) = 0
        AND CAST(json_extract_string(parent.payload, '$.span_byte_start_offset') AS UBIGINT) <
            CAST(json_extract_string(span.payload, '$.span_byte_start_offset') AS UBIGINT)
        AND CAST(json_extract_string(parent.payload, '$.span_byte_end_offset') AS UBIGINT) >
            CAST(json_extract_string(span.payload, '$.span_byte_end_offset') AS UBIGINT)
);

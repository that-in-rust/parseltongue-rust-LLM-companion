CREATE OR REPLACE TEMP VIEW "original-module-file-links" AS
SELECT declaring.generation_key, declaring.attempt_key,
    declaring.unit_payload, declaring.root_scope_payload, declaring.request_payload,
    declaring.file_payload, span.payload AS span_payload,
    target.unit_payload AS target_unit_payload,
    target.root_scope_payload AS target_root_scope_payload,
    target.request_payload AS target_request_payload,
    target.file_payload AS target_file_payload
FROM "frozen-generation-source-records" link
JOIN "original-item-source-inventory" span
    ON span.generation_key = link.generation_key AND span.attempt_key = link.attempt_key
    AND json_extract_string(span.payload, '$.span_semantic_occurrence_key') =
        json_extract_string(link.payload, '$.source_request_origin_digest')
    AND span.projection_state = 'included' AND span.syntax_kind = 'MODULE'
JOIN "original-source-area-candidates" declaring
    ON declaring.generation_key = span.generation_key AND declaring.attempt_key = span.attempt_key
    AND json_extract_string(declaring.unit_payload, '$.unit_compilation_identity_key') = span.unit_key
    AND json_extract_string(declaring.request_payload, '$.file_source_absolute_path') = span.file_path
JOIN "original-source-area-candidates" target
    ON target.generation_key = span.generation_key AND target.attempt_key = span.attempt_key
    AND json_extract_string(target.unit_payload, '$.unit_compilation_identity_key') = span.unit_key
    AND EXISTS (
        SELECT 1 FROM json_each(link.payload, '$.source_request_path_candidates') candidate
        WHERE json_extract_string(candidate.value, '$') =
            json_extract_string(target.request_payload, '$.file_source_absolute_path'))
WHERE link.table_name = 'module-source-closure-requests'
    AND json_extract_string(link.payload, '$.source_request_origin_kind') = 'module_declaration';

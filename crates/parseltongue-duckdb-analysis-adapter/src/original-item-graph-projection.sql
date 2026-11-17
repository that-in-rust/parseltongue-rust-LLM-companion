CREATE OR REPLACE TEMP VIEW "original-item-source-spans" AS
SELECT generation_key, attempt_key, payload,
    json_extract_string(payload, '$.unit_compilation_identity_key') AS unit_key,
    json_extract_string(payload, '$.file_source_absolute_path') AS file_path,
    CAST(json_extract_string(payload, '$.span_byte_start_offset') AS UBIGINT) AS start_byte,
    CAST(json_extract_string(payload, '$.span_byte_end_offset') AS UBIGINT) AS end_byte,
    json_extract_string(payload, '$.span_syntax_kind_value') AS syntax_kind,
    json_extract_string(payload, '$.span_configuration_active_flag') = 'true' AS active
FROM "frozen-generation-source-records"
WHERE table_name = 'span-reference-text-table'
    AND json_array_length(json_extract(payload, '$.span_expansion_provenance_chain')) = 0;

CREATE OR REPLACE TEMP VIEW "original-item-source-ancestors" AS
SELECT child.*, parent.payload AS parent_payload, parent.syntax_kind AS parent_kind,
    parent.start_byte AS parent_start, parent.end_byte AS parent_end, parent.active AS parent_active
FROM "original-item-source-spans" child
JOIN "original-item-source-spans" parent
    ON parent.generation_key = child.generation_key AND parent.attempt_key = child.attempt_key
    AND parent.unit_key = child.unit_key AND parent.file_path = child.file_path
    AND parent.start_byte < child.start_byte AND parent.end_byte > child.end_byte;

CREATE OR REPLACE TEMP VIEW "original-item-source-inventory" AS
SELECT span.*, CASE
    WHEN NOT active OR EXISTS (
        SELECT 1 FROM "original-item-source-ancestors" a
        WHERE a.generation_key = span.generation_key AND a.attempt_key = span.attempt_key
            AND a.payload = span.payload AND NOT a.parent_active
    ) THEN 'inactive'
    WHEN EXISTS (
        SELECT 1 FROM "original-item-source-ancestors" a
        WHERE a.generation_key = span.generation_key AND a.attempt_key = span.attempt_key
            AND a.payload = span.payload AND a.parent_kind IN ('FN', 'CONST', 'STATIC', 'TYPE_ALIAS')
    ) THEN 'body-evidence'
    WHEN syntax_kind = 'IMPL' THEN 'impl-record'
    ELSE 'included' END AS projection_state
FROM "original-item-source-spans" span;

CREATE OR REPLACE TEMP VIEW "original-item-graph-candidates" AS
SELECT area.*, span.payload AS span_payload,
    coalesce(owner.parent_payload, 'null') AS owner_payload,
    coalesce(container.parent_payload, 'null') AS container_payload,
    coalesce((SELECT to_json(list(json_extract_string(a.parent_payload, '$.span_declared_name_value') ORDER BY a.parent_start))
        FROM "original-item-source-ancestors" a
        WHERE a.generation_key = span.generation_key AND a.attempt_key = span.attempt_key
            AND a.payload = span.payload AND a.parent_kind IN ('MODULE', 'TRAIT', 'STRUCT', 'UNION', 'ENUM', 'VARIANT')), '[]') AS lexical_names,
    coalesce((SELECT to_json(list(json_object('header', header.payload::JSON, 'member', member.value)))
        FROM "frozen-generation-source-records" header, json_each(header.payload, '$.impl_member_occurrence_rows') member
        WHERE header.generation_key = span.generation_key AND header.attempt_key = span.attempt_key
            AND header.table_name = 'impl-header-index-table'
            AND json_extract_string(header.payload, '$.impl_source_occurrence_key') = json_extract_string(owner.parent_payload, '$.span_semantic_occurrence_key')
            AND json_extract_string(member.value, '$.member_source_occurrence_key') = json_extract_string(span.payload, '$.span_semantic_occurrence_key')), '[]') AS impl_bridges
FROM "original-source-area-candidates" area
JOIN "original-item-source-inventory" span
    ON span.generation_key = area.generation_key AND span.attempt_key = area.attempt_key
    AND span.unit_key = json_extract_string(area.unit_payload, '$.unit_compilation_identity_key')
    AND span.file_path = json_extract_string(area.request_payload, '$.file_source_absolute_path')
LEFT JOIN LATERAL (
    SELECT a.parent_payload FROM "original-item-source-ancestors" a
    WHERE a.generation_key = span.generation_key AND a.attempt_key = span.attempt_key
        AND a.payload = span.payload
    ORDER BY a.parent_end - a.parent_start LIMIT 1
) owner ON true
LEFT JOIN LATERAL (
    SELECT a.parent_payload FROM "original-item-source-ancestors" a
    WHERE a.generation_key = span.generation_key AND a.attempt_key = span.attempt_key
        AND a.payload = span.payload AND a.parent_kind IN ('MODULE', 'TRAIT', 'STRUCT', 'UNION', 'ENUM', 'VARIANT')
    ORDER BY a.parent_end - a.parent_start LIMIT 1
) container ON true
WHERE span.projection_state = 'included';

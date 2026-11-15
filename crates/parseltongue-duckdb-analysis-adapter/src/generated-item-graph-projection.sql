-- REQ-RUST-046.0: items written by a macro expansion, joined to the macro call that wrote them
-- and to the file area of the outermost source-written call.
CREATE OR REPLACE TEMP VIEW "generated-item-source-spans" AS
SELECT generation_key, attempt_key, payload,
    json_extract_string(payload, '$.unit_compilation_identity_key') AS unit_key,
    json_extract_string(payload, '$.file_source_absolute_path') AS file_path,
    CAST(json_extract_string(payload, '$.span_byte_start_offset') AS UBIGINT) AS start_byte,
    CAST(json_extract_string(payload, '$.span_byte_end_offset') AS UBIGINT) AS end_byte,
    json_extract_string(payload, '$.span_syntax_kind_value') AS syntax_kind,
    json_extract_string(payload, '$.span_configuration_active_flag') = 'true' AS active,
    CAST(json_extract(payload, '$.span_expansion_provenance_chain') AS VARCHAR[]) AS chain
FROM "frozen-generation-source-records"
WHERE table_name = 'span-reference-text-table'
    AND json_array_length(json_extract(payload, '$.span_expansion_provenance_chain')) > 0;

CREATE OR REPLACE TEMP VIEW "generated-item-source-ancestors" AS
SELECT child.*, parent.payload AS parent_payload, parent.syntax_kind AS parent_kind,
    parent.start_byte AS parent_start, parent.end_byte AS parent_end, parent.active AS parent_active
FROM "generated-item-source-spans" child
JOIN "generated-item-source-spans" parent
    ON parent.generation_key = child.generation_key AND parent.attempt_key = child.attempt_key
    AND parent.unit_key = child.unit_key AND parent.file_path = child.file_path
    AND parent.start_byte < child.start_byte AND parent.end_byte > child.end_byte;

CREATE OR REPLACE TEMP VIEW "generated-item-source-inventory" AS
SELECT span.*, CASE
    WHEN NOT active OR EXISTS (
        SELECT 1 FROM "generated-item-source-ancestors" a
        WHERE a.generation_key = span.generation_key AND a.attempt_key = span.attempt_key
            AND a.payload = span.payload AND NOT a.parent_active
    ) THEN 'inactive'
    WHEN EXISTS (
        SELECT 1 FROM "generated-item-source-ancestors" a
        WHERE a.generation_key = span.generation_key AND a.attempt_key = span.attempt_key
            AND a.payload = span.payload AND a.parent_kind IN ('FN', 'CONST', 'STATIC', 'TYPE_ALIAS')
    ) THEN 'body-evidence'
    WHEN syntax_kind = 'IMPL' THEN 'impl-record'
    WHEN syntax_kind IN ('VARIANT', 'RECORD_FIELD', 'TUPLE_FIELD') THEN 'deferred-member'
    ELSE 'included' END AS projection_state
FROM "generated-item-source-spans" span;

CREATE OR REPLACE TEMP VIEW "generated-item-macro-calls" AS
SELECT generation_key, attempt_key, payload,
    json_extract_string(payload, '$.macro_virtual_source_path') AS virtual_path,
    CAST(json_extract(payload, '$.macro_invocation_provenance_chain') AS VARCHAR[]) AS chain,
    json_extract_string(payload, '$.macro_invocation_identity_record.span_source_file_path') AS call_path,
    CAST(json_extract_string(payload, '$.macro_invocation_identity_record.span_byte_range_value[0]') AS UBIGINT) AS call_start,
    CAST(json_extract_string(payload, '$.macro_invocation_identity_record.span_byte_range_value[1]') AS UBIGINT) AS call_end
FROM "frozen-generation-source-records"
WHERE table_name = 'macro-expanded-item-table'
    AND json_extract_string(payload, '$.macro_resolution_state_kind') = 'expanded';

CREATE OR REPLACE TEMP VIEW "generated-item-graph-candidates" AS
SELECT area.*, span.payload AS span_payload,
    coalesce(owner.parent_payload, 'null') AS owner_payload,
    coalesce(container.parent_payload, 'null') AS container_payload,
    coalesce((SELECT to_json(list(json_extract_string(a.parent_payload, '$.span_declared_name_value') ORDER BY a.parent_start))
        FROM "generated-item-source-ancestors" a
        WHERE a.generation_key = span.generation_key AND a.attempt_key = span.attempt_key
            AND a.payload = span.payload AND a.parent_kind IN ('MODULE', 'TRAIT')), '[]') AS lexical_names,
    coalesce((SELECT to_json(list(json_object('header', header.payload::JSON, 'member', member.value)))
        FROM "frozen-generation-source-records" header, json_each(header.payload, '$.impl_member_occurrence_rows') member
        WHERE header.generation_key = span.generation_key AND header.attempt_key = span.attempt_key
            AND header.table_name = 'impl-header-index-table'
            AND json_extract_string(header.payload, '$.impl_source_occurrence_key') = json_extract_string(owner.parent_payload, '$.span_semantic_occurrence_key')
            AND json_extract_string(member.value, '$.member_source_occurrence_key') = json_extract_string(span.payload, '$.span_semantic_occurrence_key')), '[]') AS impl_bridges,
    expanded.payload AS expanded_file_payload,
    last_call.payload AS call_payload,
    outer_call.payload AS outer_call_payload,
    coalesce(call_container.payload, 'null') AS call_container_payload,
    coalesce((SELECT to_json(list(json_extract_string(o.payload, '$.span_declared_name_value') ORDER BY o.start_byte))
        FROM "original-item-source-spans" o
        WHERE o.generation_key = span.generation_key AND o.attempt_key = span.attempt_key
            AND o.unit_key = span.unit_key
            AND o.file_path = json_extract_string(area.request_payload, '$.file_source_absolute_path')
            AND o.syntax_kind IN ('MODULE', 'TRAIT')
            AND o.start_byte < outer_call.call_start AND o.end_byte > outer_call.call_end), '[]') AS call_lexical_names,
    coalesce(call_span.payload, 'null') AS call_span_payload
FROM "generated-item-source-inventory" span
JOIN "generated-item-macro-calls" last_call
    ON last_call.generation_key = span.generation_key AND last_call.attempt_key = span.attempt_key
    AND last_call.virtual_path = span.file_path
JOIN "generated-item-macro-calls" outer_call
    ON outer_call.generation_key = span.generation_key AND outer_call.attempt_key = span.attempt_key
    AND len(outer_call.chain) = 1 AND list_contains(span.chain, outer_call.chain[1])
JOIN "original-source-area-candidates" area
    ON area.generation_key = span.generation_key AND area.attempt_key = span.attempt_key
    AND json_extract_string(area.unit_payload, '$.unit_compilation_identity_key') = span.unit_key
    AND json_extract_string(area.request_payload, '$.file_source_absolute_path') =
        json_extract_string(area.unit_payload, '$.unit_package_root_path') || '/' || outer_call.call_path
JOIN "frozen-generation-source-records" expanded
    ON expanded.generation_key = span.generation_key AND expanded.attempt_key = span.attempt_key
    AND expanded.table_name = 'file-source-text-table'
    AND json_extract_string(expanded.payload, '$.file_source_absolute_path') = span.file_path
LEFT JOIN LATERAL (
    SELECT a.parent_payload FROM "generated-item-source-ancestors" a
    WHERE a.generation_key = span.generation_key AND a.attempt_key = span.attempt_key
        AND a.payload = span.payload
    ORDER BY a.parent_end - a.parent_start LIMIT 1
) owner ON true
LEFT JOIN LATERAL (
    SELECT a.parent_payload FROM "generated-item-source-ancestors" a
    WHERE a.generation_key = span.generation_key AND a.attempt_key = span.attempt_key
        AND a.payload = span.payload AND a.parent_kind IN ('MODULE', 'TRAIT')
    ORDER BY a.parent_end - a.parent_start LIMIT 1
) container ON true
LEFT JOIN LATERAL (
    SELECT o.payload FROM "original-item-source-spans" o
    WHERE o.generation_key = span.generation_key AND o.attempt_key = span.attempt_key
        AND o.unit_key = span.unit_key
        AND o.file_path = json_extract_string(area.request_payload, '$.file_source_absolute_path')
        AND o.syntax_kind IN ('MODULE', 'TRAIT')
        AND o.start_byte < outer_call.call_start AND o.end_byte > outer_call.call_end
    ORDER BY o.end_byte - o.start_byte LIMIT 1
) call_container ON true
-- The node of the call that wrote the item; only a source-written call is a node today.
LEFT JOIN LATERAL (
    SELECT o.payload FROM "original-item-source-spans" o
    WHERE o.generation_key = span.generation_key AND o.attempt_key = span.attempt_key
        AND o.unit_key = span.unit_key
        AND o.file_path = json_extract_string(area.request_payload, '$.file_source_absolute_path')
        AND o.syntax_kind = 'MACRO_CALL' AND len(last_call.chain) = 1
        AND o.start_byte = last_call.call_start AND o.end_byte = last_call.call_end
    LIMIT 1
) call_span ON true
WHERE span.projection_state = 'included';

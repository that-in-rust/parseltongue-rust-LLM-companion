DROP TABLE IF EXISTS "hld-analysis-path-records";
DROP TABLE IF EXISTS "hld-analysis-edge-records";
DROP TABLE IF EXISTS "hld-analysis-node-records";
DROP TABLE IF EXISTS "hld-analysis-node-inputs";
DROP TABLE IF EXISTS "hld-analysis-unit-origins";

CREATE TEMP TABLE "hld-analysis-unit-origins" (
    unit_identity_key VARCHAR NOT NULL,
    unit_origin_kind VARCHAR NOT NULL
);

CREATE TEMP TABLE "hld-analysis-node-inputs" (
    run_generation_key VARCHAR NOT NULL,
    endpoint_identity_key VARCHAR NOT NULL,
    endpoint_category_kind VARCHAR NOT NULL,
    unit_identity_key VARCHAR,
    item_kind_value VARCHAR,
    owner_kind_value VARCHAR,
    function_body_ancestry_flag BOOLEAN NOT NULL,
    effective_visibility_kind VARCHAR,
    reachable_outside_crate_flag BOOLEAN NOT NULL,
    source_relative_path VARCHAR,
    qualified_path_text VARCHAR NOT NULL,
    signature_text VARCHAR
);

CREATE TEMP TABLE "hld-analysis-edge-records" (
    run_generation_key VARCHAR NOT NULL,
    evidence_identity_key VARCHAR NOT NULL,
    evidence_site_key VARCHAR NOT NULL,
    source_identity_key VARCHAR NOT NULL,
    target_identity_key VARCHAR,
    relation_kind_value VARCHAR NOT NULL,
    resolution_status_kind VARCHAR NOT NULL,
    dispatch_metadata_text VARCHAR,
    source_unit_identity_key VARCHAR,
    target_unit_identity_key VARCHAR
);

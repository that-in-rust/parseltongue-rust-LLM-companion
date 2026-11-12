use parseltongue_core_row_types::{StageTableRowRecord, StoreBackendFailureError};

pub(super) fn project_namespace_source_record(
    mut row: StageTableRowRecord,
) -> Result<StageTableRowRecord, StoreBackendFailureError> {
    let Some(columns) = declared_namespace_source_columns(&row.stage_owned_table_name) else {
        return Ok(row);
    };
    let value: serde_json::Value =
        serde_json::from_str(&row.stage_row_payload_text).map_err(super::build_analysis_error)?;
    let serde_json::Value::Object(fields) = value else {
        return Err(super::build_analysis_error(
            "namespace source row must be an object",
        ));
    };
    let fields = fields
        .into_iter()
        .filter(|(name, _)| columns.contains(&name.as_str()))
        .collect();
    row.stage_row_payload_text = serde_json::Value::Object(fields).to_string();
    Ok(row)
}

fn declared_namespace_source_columns(table: &str) -> Option<&'static [&'static str]> {
    match table {
        "crate-dependency-graph-table" => Some(&[
            "unit_compilation_identity_key",
            "unit_rust_edition_value",
            "unit_normal_dependency_rows",
            "unit_implicit_prelude_flag",
            "unit_no_std_flag",
            "unit_no_core_flag",
            "unit_sysroot_context_flag",
            parseltongue_core_row_types::MAPPED_TARGET_FLAG_FIELD,
        ]),
        "syntax-namespace-input-scopes" => Some(&[
            "scope_module_identity_key",
            "scope_parent_identity_key",
            "scope_parent_lookup_flag",
            "scope_module_boundary_flag",
            "scope_implicit_prelude_key",
            "scope_cargo_unit_key",
        ]),
        "syntax-namespace-input-items" => Some(&[
            "binding_lexical_scope_key",
            "binding_resolved_name_text",
            "binding_namespace_kind_value",
            "binding_target_identity_key",
            "binding_target_child_scope",
            "binding_visibility_scope_kind",
            "binding_selection_priority_kind",
            "binding_target_metadata_value",
        ]),
        "syntax-namespace-input-paths" => Some(&[
            "path_site_occurrence_key",
            "path_lexical_scope_key",
            "path_source_segment_rows",
            "path_requested_namespace_kind",
            "path_resolution_source_kind",
            "path_unavailable_reason_kind",
            "path_source_metadata_value",
        ]),
        "syntax-namespace-input-imports" => Some(&[
            "import_site_occurrence_key",
            "import_lexical_scope_key",
            "import_path_segment_rows",
            "import_binding_alias_text",
            "import_glob_expansion_flag",
            "import_visibility_scope_kind",
        ]),
        "syntax-namespace-input-macros" => Some(&[
            "invocation_site_occurrence_key",
            "invocation_lexical_scope_key",
            "invocation_macro_path_segments",
        ]),
        "syntax-namespace-input-externs" => Some(&[
            "extern_declaration_scope_key",
            "extern_declaration_source_name",
            "extern_declaration_alias_name",
            "extern_declaration_visibility",
        ]),
        "legacy-macro-scope-records" => Some(&[
            "legacy_lookup_identity_key",
            "legacy_source_site_key",
            "legacy_macro_name_text",
            "legacy_target_identity_key",
            "legacy_scope_revision_value",
            "legacy_source_order_parts",
        ]),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;

    /// REQ-RUST-024.8
    #[test]
    fn test_namespace_source_projection_identity() {
        let row=StageTableRowRecord{stage_owned_table_name:"syntax-namespace-input-items".into(),stage_row_record_key:"unchanged-source-identity".into(),stage_row_payload_text:json!({"binding_resolved_name_text":"kept","binding_target_metadata_value":{"raw_macro_definition":"required"},"unused_raw_source":"x".repeat(100000)}).to_string()};
        let projected = project_namespace_source_record(row).unwrap();
        assert_eq!(projected.stage_row_record_key, "unchanged-source-identity");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&projected.stage_row_payload_text).unwrap(),
            json!({"binding_resolved_name_text":"kept","binding_target_metadata_value":{"raw_macro_definition":"required"}})
        );
        let unknown = StageTableRowRecord {
            stage_owned_table_name: "span-reference-text-table".into(),
            stage_row_record_key: "raw".into(),
            stage_row_payload_text: "not JSON; generic copies are byte-exact".into(),
        };
        assert_eq!(
            project_namespace_source_record(unknown.clone()).unwrap(),
            unknown
        );
        assert!(
            project_namespace_source_record(StageTableRowRecord {
                stage_owned_table_name: "syntax-namespace-input-items".into(),
                stage_row_record_key: "bad".into(),
                stage_row_payload_text: "[]".into()
            })
            .is_err()
        );
    }
}

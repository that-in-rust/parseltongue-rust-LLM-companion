use parseltongue_core_row_types::*;
use serde_json::{Value, json};

pub(super) fn build_name_projection_error(
    message: impl std::fmt::Display,
) -> StoreBackendFailureError {
    StoreBackendFailureError {
        error_context_detail_text: message.to_string(),
    }
}

pub(super) fn encode_unresolved_reason_value(reason: SiteUnresolvedReasonKind) -> &'static str {
    match reason {
        SiteUnresolvedReasonKind::CfgInactive => "cfg_inactive",
        SiteUnresolvedReasonKind::BuildScript => "build_script",
        SiteUnresolvedReasonKind::ProcMacro => "proc_macro",
        SiteUnresolvedReasonKind::ExternalSourceMissing => "external_source_missing",
        SiteUnresolvedReasonKind::UnknownReceiver => "unknown_receiver",
        SiteUnresolvedReasonKind::TraitSolverNeeded => "trait_solver_needed",
    }
}

pub(super) fn read_record_string_value<'a>(
    value: &'a Value,
    key: &str,
) -> Result<&'a str, StoreBackendFailureError> {
    value[key]
        .as_str()
        .ok_or_else(|| build_name_projection_error(format!("missing string {key}")))
}

pub(super) fn encode_digest_hex_value(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn decode_digest_hex_value(value: &str) -> Result<[u8; 32], StoreBackendFailureError> {
    if value.len() != 64 || !value.is_ascii() {
        return Err(build_name_projection_error("invalid full digest"));
    }
    (0..32)
        .map(|index| {
            u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
                .map_err(build_name_projection_error)
        })
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| build_name_projection_error("invalid digest length"))
}

pub(super) fn encode_compilation_unit_value(unit: &CrateCompilationUnitKey) -> Value {
    json!({
        "unit_cargo_package_identifier":unit.unit_cargo_package_identifier,
        "unit_cargo_target_name":unit.unit_cargo_target_name,
        "unit_cargo_target_kind":unit.unit_cargo_target_kind,
        "unit_compilation_role_kind":match unit.unit_compilation_role_kind {CompilationUnitRoleKind::Host=>"host",CompilationUnitRoleKind::Target=>"target"},
        "unit_configuration_digest_full":encode_digest_hex_value(&unit.unit_configuration_digest_full),
    })
}

pub(super) fn decode_compilation_unit_value(
    value: &Value,
) -> Result<CrateCompilationUnitKey, StoreBackendFailureError> {
    Ok(CrateCompilationUnitKey {
        unit_cargo_package_identifier: read_record_string_value(
            value,
            "unit_cargo_package_identifier",
        )?
        .into(),
        unit_cargo_target_name: read_record_string_value(value, "unit_cargo_target_name")?.into(),
        unit_cargo_target_kind: read_record_string_value(value, "unit_cargo_target_kind")?.into(),
        unit_compilation_role_kind: match read_record_string_value(
            value,
            "unit_compilation_role_kind",
        )? {
            "host" => CompilationUnitRoleKind::Host,
            "target" => CompilationUnitRoleKind::Target,
            _ => return Err(build_name_projection_error("invalid compilation role")),
        },
        unit_configuration_digest_full: decode_digest_hex_value(read_record_string_value(
            value,
            "unit_configuration_digest_full",
        )?)?,
    })
}

pub(super) fn encode_resolution_scope_value(scope: &NameResolutionScopeKey) -> Value {
    json!({"scope_compilation_unit_key":encode_compilation_unit_value(&scope.scope_compilation_unit_key),"scope_semantic_path_parts":scope.scope_semantic_path_parts})
}

pub(super) fn encode_occurrence_identity_value(occurrence: &SemanticOccurrenceKeyValue) -> Value {
    let parts = occurrence.read_semantic_occurrence_parts();
    let range = parts.span_byte_range_value.read_source_range_pair();
    json!({
        "unit_compilation_identity_key":encode_compilation_unit_value(parts.unit_compilation_identity_key),
        "span_source_file_path":parts.span_source_file_path,
        "span_byte_range_value":[range.0,range.1],
        "span_content_digest_full":encode_digest_hex_value(parts.span_content_digest_full.read_full_digest_bytes()),
        "span_expansion_provenance_chain":parts.span_expansion_provenance_chain.iter().map(encode_digest_hex_value).collect::<Vec<_>>(),
    })
}

pub(super) fn encode_symbol_target_value(target: &NameSymbolTargetKey) -> Value {
    match target {
        NameSymbolTargetKey::Definition(key) => {
            json!({"target_identity_variant_kind":"definition","target_semantic_occurrence_key":encode_occurrence_identity_value(key)})
        }
        NameSymbolTargetKey::GenericParameter(key, name) => {
            json!({"target_identity_variant_kind":"generic","target_semantic_occurrence_key":encode_occurrence_identity_value(key),"target_parameter_name_value":name})
        }
        NameSymbolTargetKey::Builtin(name) => {
            json!({"target_identity_variant_kind":"builtin","target_builtin_name_value":name})
        }
        NameSymbolTargetKey::CrateRoot(unit) => {
            json!({"target_identity_variant_kind":"crate","target_compilation_unit_key":encode_compilation_unit_value(unit)})
        }
    }
}

pub(super) fn encode_scope_record_value(row: &NameScopeRecordRow) -> Value {
    json!({
        "scope_module_identity_key":encode_resolution_scope_value(&row.scope_module_identity_key),
        "scope_parent_identity_key":row.scope_parent_identity_key.as_ref().map(encode_resolution_scope_value),
        "scope_parent_lookup_flag":row.scope_parent_lookup_flag,
        "scope_module_boundary_flag":row.scope_module_boundary_flag,
        "scope_implicit_prelude_key":row.scope_implicit_prelude_key.as_ref().map(encode_resolution_scope_value),
    })
}

pub(super) fn encode_binding_record_value(row: &NameResolvedBindingRow, metadata: Value) -> Value {
    json!({
        "binding_lexical_scope_key":encode_resolution_scope_value(&row.binding_lexical_scope_key),
        "binding_resolved_name_text":row.binding_resolved_name_text,
        "binding_namespace_kind_value":encode_namespace_kind_value(row.binding_namespace_kind_value),
        "binding_target_identity_key":encode_symbol_target_value(&row.binding_target_identity_key),
        "binding_target_child_scope":row.binding_target_child_scope.as_ref().map(encode_resolution_scope_value),
        "binding_visibility_scope_kind":match &row.binding_visibility_scope_kind {
            NameBindingVisibilityKind::Public=>json!({"visibility_scope_kind_value":"public"}),
            NameBindingVisibilityKind::Invisible=>json!({"visibility_scope_kind_value":"invisible"}),
            NameBindingVisibilityKind::Restricted(key)=>json!({"visibility_scope_kind_value":"restricted","visibility_restricted_scope_key":encode_resolution_scope_value(key)}),
        },
        "binding_selection_priority_kind":match row.binding_selection_priority_kind{
            NameBindingPriorityKind::Builtin=>"builtin",NameBindingPriorityKind::Prelude=>"prelude",NameBindingPriorityKind::Glob=>"glob",NameBindingPriorityKind::Explicit=>"explicit",NameBindingPriorityKind::Declaration=>"declaration",
        },
        "binding_import_evidence_keys":row.binding_import_evidence_keys.iter().map(encode_occurrence_identity_value).collect::<Vec<_>>(),
        "binding_target_metadata_value":metadata,
    })
}

pub(super) fn encode_visibility_scope_value(visibility: &NameBindingVisibilityKind) -> Value {
    match visibility {
        NameBindingVisibilityKind::Public => json!({"visibility_scope_kind_value":"public"}),
        NameBindingVisibilityKind::Invisible => json!({"visibility_scope_kind_value":"invisible"}),
        NameBindingVisibilityKind::Restricted(scope) => json!({
            "visibility_scope_kind_value":"restricted",
            "visibility_restricted_scope_key":encode_resolution_scope_value(scope),
        }),
    }
}

pub(super) fn encode_path_segment_values(segments: &[NamePathSegmentKind]) -> Vec<Value> {
    segments
        .iter()
        .map(|segment| match segment {
            NamePathSegmentKind::CrateRoot => json!({"kind":"crate"}),
            NamePathSegmentKind::SelfModule => json!({"kind":"self"}),
            NamePathSegmentKind::ParentModule => json!({"kind":"super"}),
            NamePathSegmentKind::GlobalRoot => json!({"kind":"global"}),
            NamePathSegmentKind::DefiningCrate(unit) => {
                json!({"kind":"defining_crate","unit":encode_compilation_unit_value(unit)})
            }
            NamePathSegmentKind::Name(name) => json!({"kind":"name","name":name}),
        })
        .collect()
}

pub(super) fn encode_namespace_kind_value(namespace: NamespaceBindingKindValue) -> &'static str {
    match namespace {
        NamespaceBindingKindValue::Type => "type",
        NamespaceBindingKindValue::Value => "value",
        NamespaceBindingKindValue::Macro => "macro",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_path_root_kinds() {
        let unit = CrateCompilationUnitKey {
            unit_cargo_package_identifier: "fixture".into(),
            unit_cargo_target_name: "fixture".into(),
            unit_cargo_target_kind: "lib".into(),
            unit_compilation_role_kind: CompilationUnitRoleKind::Host,
            unit_configuration_digest_full: [7; 32],
        };
        let segments = encode_path_segment_values(&[
            NamePathSegmentKind::CrateRoot,
            NamePathSegmentKind::SelfModule,
            NamePathSegmentKind::ParentModule,
            NamePathSegmentKind::GlobalRoot,
            NamePathSegmentKind::DefiningCrate(unit.clone()),
            NamePathSegmentKind::Name("ordinary".into()),
        ]);
        assert_eq!(
            segments
                .iter()
                .map(|value| value["kind"].as_str())
                .collect::<Vec<_>>(),
            vec![
                Some("crate"),
                Some("self"),
                Some("super"),
                Some("global"),
                Some("defining_crate"),
                Some("name")
            ]
        );
        assert_eq!(
            decode_compilation_unit_value(&segments[4]["unit"]),
            Ok(unit)
        );
        assert_eq!(segments[5]["name"], "ordinary");
        assert_eq!(
            encode_visibility_scope_value(&NameBindingVisibilityKind::Invisible)["visibility_scope_kind_value"],
            "invisible"
        );
    }

    #[test]
    fn preserves_empty_paths() {
        assert!(encode_path_segment_values(&[]).is_empty());
    }

    #[test]
    fn rejects_invalid_compilation_identities() {
        assert!(decode_compilation_unit_value(&json!({})).is_err());
        assert!(
            decode_compilation_unit_value(&json!({
                "unit_cargo_package_identifier":"fixture",
                "unit_cargo_target_name":"fixture","unit_cargo_target_kind":"lib",
                "unit_compilation_role_kind":"host","unit_configuration_digest_full":"short",
            }))
            .is_err()
        );
    }
}

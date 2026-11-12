use parseltongue_core_row_types::*;
use serde_json::{Value, json};

pub(super) fn build_name_projection_error(
    message: impl std::fmt::Display,
) -> StoreBackendFailureError {
    StoreBackendFailureError {
        error_context_detail_text: message.to_string(),
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

pub(super) fn decode_resolution_scope_value(
    value: &Value,
) -> Result<NameResolutionScopeKey, StoreBackendFailureError> {
    Ok(NameResolutionScopeKey {
        scope_compilation_unit_key: decode_compilation_unit_value(
            &value["scope_compilation_unit_key"],
        )?,
        scope_semantic_path_parts: serde_json::from_value(
            value["scope_semantic_path_parts"].clone(),
        )
        .map_err(build_name_projection_error)?,
    })
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

pub(super) fn decode_occurrence_identity_value(
    value: &Value,
) -> Result<SemanticOccurrenceKeyValue, StoreBackendFailureError> {
    let range: [u64; 2] = serde_json::from_value(value["span_byte_range_value"].clone())
        .map_err(build_name_projection_error)?;
    let provenance: Vec<String> =
        serde_json::from_value(value["span_expansion_provenance_chain"].clone())
            .map_err(build_name_projection_error)?;
    SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
        &decode_compilation_unit_value(&value["unit_compilation_identity_key"])?,
        read_record_string_value(value, "span_source_file_path")?,
        SpanByteRangeValue::try_from((range[0], range[1])).map_err(build_name_projection_error)?,
        SpanDigestIdValue::from(decode_digest_hex_value(read_record_string_value(
            value,
            "span_content_digest_full",
        )?)?),
        &provenance
            .iter()
            .map(|value| decode_digest_hex_value(value))
            .collect::<Result<Vec<_>, _>>()?,
    )
    .map_err(build_name_projection_error)
}

fn encode_symbol_target_value(target: &NameSymbolTargetKey) -> Value {
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

fn decode_symbol_target_value(
    value: &Value,
) -> Result<NameSymbolTargetKey, StoreBackendFailureError> {
    match read_record_string_value(value, "target_identity_variant_kind")? {
        "definition" => Ok(NameSymbolTargetKey::Definition(
            decode_occurrence_identity_value(&value["target_semantic_occurrence_key"])?,
        )),
        "generic" => Ok(NameSymbolTargetKey::GenericParameter(
            decode_occurrence_identity_value(&value["target_semantic_occurrence_key"])?,
            read_record_string_value(value, "target_parameter_name_value")?.into(),
        )),
        "builtin" => Ok(NameSymbolTargetKey::Builtin(
            read_record_string_value(value, "target_builtin_name_value")?.into(),
        )),
        "crate" => Ok(NameSymbolTargetKey::CrateRoot(
            decode_compilation_unit_value(&value["target_compilation_unit_key"])?,
        )),
        other => Err(build_name_projection_error(format!(
            "unknown target identity {other}"
        ))),
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

pub(super) fn decode_scope_record_value(
    value: &Value,
) -> Result<NameScopeRecordRow, StoreBackendFailureError> {
    Ok(NameScopeRecordRow {
        scope_module_identity_key: decode_resolution_scope_value(
            &value["scope_module_identity_key"],
        )?,
        scope_parent_identity_key: (!value["scope_parent_identity_key"].is_null())
            .then(|| decode_resolution_scope_value(&value["scope_parent_identity_key"]))
            .transpose()?,
        scope_parent_lookup_flag: value["scope_parent_lookup_flag"]
            .as_bool()
            .ok_or_else(|| build_name_projection_error("missing parent lookup flag"))?,
        scope_module_boundary_flag: value["scope_module_boundary_flag"]
            .as_bool()
            .ok_or_else(|| build_name_projection_error("missing module boundary flag"))?,
        scope_implicit_prelude_key: (!value["scope_implicit_prelude_key"].is_null())
            .then(|| decode_resolution_scope_value(&value["scope_implicit_prelude_key"]))
            .transpose()?,
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

pub(super) fn encode_namespace_kind_value(namespace: NamespaceBindingKindValue) -> &'static str {
    match namespace {
        NamespaceBindingKindValue::Type => "type",
        NamespaceBindingKindValue::Value => "value",
        NamespaceBindingKindValue::Macro => "macro",
    }
}

pub(super) fn decode_binding_record_value(
    value: &Value,
) -> Result<NameResolvedBindingRow, StoreBackendFailureError> {
    let visibility = &value["binding_visibility_scope_kind"];
    Ok(NameResolvedBindingRow {
        binding_lexical_scope_key: decode_resolution_scope_value(
            &value["binding_lexical_scope_key"],
        )?,
        binding_resolved_name_text: read_record_string_value(value, "binding_resolved_name_text")?
            .into(),
        binding_namespace_kind_value: match read_record_string_value(
            value,
            "binding_namespace_kind_value",
        )? {
            "type" => NamespaceBindingKindValue::Type,
            "value" => NamespaceBindingKindValue::Value,
            "macro" => NamespaceBindingKindValue::Macro,
            _ => return Err(build_name_projection_error("invalid namespace")),
        },
        binding_target_identity_key: decode_symbol_target_value(
            &value["binding_target_identity_key"],
        )?,
        binding_target_child_scope: (!value["binding_target_child_scope"].is_null())
            .then(|| decode_resolution_scope_value(&value["binding_target_child_scope"]))
            .transpose()?,
        binding_visibility_scope_kind: match read_record_string_value(
            visibility,
            "visibility_scope_kind_value",
        )? {
            "public" => NameBindingVisibilityKind::Public,
            "invisible" => NameBindingVisibilityKind::Invisible,
            "restricted" => NameBindingVisibilityKind::Restricted(decode_resolution_scope_value(
                &visibility["visibility_restricted_scope_key"],
            )?),
            _ => return Err(build_name_projection_error("invalid visibility")),
        },
        binding_selection_priority_kind: match read_record_string_value(
            value,
            "binding_selection_priority_kind",
        )? {
            "builtin" => NameBindingPriorityKind::Builtin,
            "prelude" => NameBindingPriorityKind::Prelude,
            "glob" => NameBindingPriorityKind::Glob,
            "explicit" => NameBindingPriorityKind::Explicit,
            "declaration" => NameBindingPriorityKind::Declaration,
            _ => return Err(build_name_projection_error("invalid binding priority")),
        },
        binding_import_evidence_keys: value["binding_import_evidence_keys"]
            .as_array()
            .ok_or_else(|| build_name_projection_error("missing import evidence"))?
            .iter()
            .map(decode_occurrence_identity_value)
            .collect::<Result<_, _>>()?,
    })
}

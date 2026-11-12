use parseltongue_core_row_types::*;
use serde_json::{Value, json};

fn hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(super) fn unit_key(unit: &CrateCompilationUnitKey) -> Value {
    json!({"unit_cargo_package_identifier":unit.unit_cargo_package_identifier,"unit_cargo_target_name":unit.unit_cargo_target_name,"unit_cargo_target_kind":unit.unit_cargo_target_kind,"unit_compilation_role_kind":match unit.unit_compilation_role_kind{CompilationUnitRoleKind::Host=>"host",CompilationUnitRoleKind::Target=>"target"},"unit_configuration_digest_full":hex(&unit.unit_configuration_digest_full)})
}

pub(super) fn scope_key(scope: &NameResolutionScopeKey) -> Value {
    json!({"scope_compilation_unit_key":unit_key(&scope.scope_compilation_unit_key),"scope_semantic_path_parts":scope.scope_semantic_path_parts})
}

pub(super) fn occurrence_key(key: &SemanticOccurrenceKeyValue) -> Value {
    let parts = key.read_semantic_occurrence_parts();
    let range = parts.span_byte_range_value.read_source_range_pair();
    json!({"unit_compilation_identity_key":unit_key(parts.unit_compilation_identity_key),"span_source_file_path":parts.span_source_file_path,"span_byte_range_value":[range.0,range.1],"span_content_digest_full":hex(parts.span_content_digest_full.read_full_digest_bytes()),"span_expansion_provenance_chain":parts.span_expansion_provenance_chain.iter().map(hex).collect::<Vec<_>>()})
}

pub(super) fn target_key(target: &NameSymbolTargetKey) -> Value {
    match target {
        NameSymbolTargetKey::Definition(key) => {
            json!({"target_identity_variant_kind":"definition","target_semantic_occurrence_key":occurrence_key(key)})
        }
        NameSymbolTargetKey::GenericParameter(key, name) => {
            json!({"target_identity_variant_kind":"generic","target_semantic_occurrence_key":occurrence_key(key),"target_parameter_name_value":name})
        }
        NameSymbolTargetKey::Builtin(name) => {
            json!({"target_identity_variant_kind":"builtin","target_builtin_name_value":name})
        }
        NameSymbolTargetKey::CrateRoot(unit) => {
            json!({"target_identity_variant_kind":"crate","target_compilation_unit_key":unit_key(unit)})
        }
    }
}

pub(super) fn path_parts(path: &[NamePathSegmentKind]) -> Vec<(&'static str, String)> {
    path.iter()
        .map(|part| match part {
            NamePathSegmentKind::Name(name) => ("name", name.trim_start_matches("r#").to_owned()),
            NamePathSegmentKind::CrateRoot => ("crate", String::new()),
            NamePathSegmentKind::SelfModule => ("self", String::new()),
            NamePathSegmentKind::ParentModule => ("super", String::new()),
            NamePathSegmentKind::GlobalRoot => ("global", String::new()),
            NamePathSegmentKind::DefiningCrate(unit) => (
                "defining",
                json!({"scope_compilation_unit_key":unit_key(unit),"scope_semantic_path_parts":[]})
                    .to_string(),
            ),
        })
        .collect()
}

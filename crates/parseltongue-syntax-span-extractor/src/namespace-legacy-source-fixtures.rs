#![allow(clippy::unwrap_used)]
use super::*;

struct LegacyMacroFixtureStore {
    source: String,
}

impl StageTableStorePort for LegacyMacroFixtureStore {
    fn initialize_workspace_stage_schema(&self) -> Result<(), StoreBackendFailureError> {
        Ok(())
    }
    fn write_stage_table_unit(
        &self,
        _: &StageUnitWriteRequest<'_>,
        _: &RunCancelFlagValue,
    ) -> Result<usize, StoreBackendFailureError> {
        Err(build_name_projection_error("unexpected stage write"))
    }
    fn read_stage_table_rows(
        &self,
        _: &StageRunScopeValue,
        table: &str,
    ) -> Result<Vec<StageTableRowRecord>, StoreBackendFailureError> {
        Ok(if table == "file-source-text-table" {
            vec![StageTableRowRecord {
            stage_owned_table_name:table.into(),stage_row_record_key:"file".into(),stage_row_payload_text:json!({
                "file_source_absolute_path":"/fixture/src/lib.rs","file_content_digest_full":blake3::hash(self.source.as_bytes()).to_hex().to_string()}).to_string()}]
        } else {
            vec![]
        })
    }
    fn write_file_content_bytes(
        &self,
        _: &str,
        _: &[u8],
    ) -> Result<bool, StoreBackendFailureError> {
        Ok(true)
    }
    fn read_file_content_bytes(&self, digest: &str) -> Result<Vec<u8>, StoreBackendFailureError> {
        if digest == blake3::hash(self.source.as_bytes()).to_hex().as_str() {
            Ok(self.source.as_bytes().to_vec())
        } else {
            Err(build_name_projection_error("unknown source"))
        }
    }
    fn publish_workspace_generation_manifest(
        &self,
        _: &StageRunScopeValue,
        _: &str,
        _: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, StoreBackendFailureError> {
        Err(build_name_projection_error("unexpected publication"))
    }
    fn read_workspace_publication_scope(
        &self,
    ) -> Result<Option<StageRunScopeValue>, StoreBackendFailureError> {
        Ok(None)
    }
}

fn unit() -> Value {
    json!({"unit_cargo_package_identifier":"fixture","unit_cargo_target_name":"fixture","unit_cargo_target_kind":"lib",
        "unit_compilation_role_kind":"target","unit_target_cfg_values":[],"unit_enabled_feature_values":[],
        "unit_rust_edition_value":"2024","unit_source_root_path":"/fixture/src/lib.rs","unit_package_root_path":"/fixture",
        "unit_compilation_identity_key":"fixture-unit"})
}

fn project(source: &str) -> Vec<Value> {
    let store = LegacyMacroFixtureStore {
        source: source.into(),
    };
    let cancel = RunCancelFlagValue::default();
    let mut state = CrateSyntaxProjectionState::build_crate_projection_state(
        unit(),
        &json!({"workspace_source_root_path":"/fixture"}),
        &store,
        &StageRunScopeValue {
            run_generation_digest_value: "generation".into(),
            stage_attempt_sequence_value: "attempt".into(),
        },
        &cancel,
    )
    .unwrap();
    state.project_unit_source_root().unwrap();
    state.projection_legacy_source_event_rows
}

#[test]
fn persisted_projection_emits_textual_scopes() {
    let events = project("m!(); #[macro_use] mod imported {macro_rules! m {()=>{}}} m!();");
    assert!(!events.is_empty());
    assert!(
        events
            .iter()
            .any(|event| event["legacy_module_export_flag"] == true)
    );
    let mut rows = Vec::new();
    crate::visit_legacy_macro_scope_records(&events, &mut |row| {
        rows.push(serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap());
        Ok(())
    })
    .unwrap();
    assert!(
        rows.iter()
            .any(|row| !row["legacy_target_identity_key"].is_null())
    );
    assert!(
        rows.iter()
            .all(|row| row["legacy_source_unit_key"].is_object())
    );
}

#[test]
fn projection_empty_and_inactive_scopes() {
    assert!(project("").is_empty());
    assert!(project("#[cfg(any())] mod inactive {macro_rules! m {()=>{}}m!();}").is_empty());
}

#[test]
fn projected_virtual_macro_scope() {
    let store = LegacyMacroFixtureStore {
        source: "make!(); generated!();".into(),
    };
    let cancel = RunCancelFlagValue::default();
    let mut state = CrateSyntaxProjectionState::build_crate_projection_state(
        unit(),
        &json!({"workspace_source_root_path":"/fixture"}),
        &store,
        &StageRunScopeValue {
            run_generation_digest_value: "generation".into(),
            stage_attempt_sequence_value: "attempt".into(),
        },
        &cancel,
    )
    .unwrap();
    state.project_unit_source_root().unwrap();
    let invocation = state
        .projection_resolution_input_rows
        .name_macro_invocation_rows[0]
        .clone();
    let definition = json!({"macro_definition_unit_key":encode_compilation_unit_value(&state.projection_resolution_input_rows.unit_compilation_identity_key)});
    let expanded = DeclarativeMacroExpansionOutput {
        macro_expanded_source_text: "macro_rules! make {()=>{}} macro_rules! generated {()=>{}}"
            .into(),
        macro_token_origin_rows: vec![],
        macro_matched_arm_index: 0,
    };
    state
        .project_macro_expansion_rows(&invocation, &definition, &expanded)
        .unwrap();
    let mut rows = Vec::new();
    crate::visit_legacy_macro_scope_records(
        &state.projection_legacy_source_event_rows,
        &mut |row| {
            rows.push(serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap());
            Ok(())
        },
    )
    .unwrap();
    assert!(
        rows.iter()
            .any(|row| row["legacy_macro_name_text"] == "make"
                && row["legacy_target_identity_key"].is_null())
    );
    assert!(
        rows.iter()
            .any(|row| row["legacy_macro_name_text"] == "generated"
                && !row["legacy_target_identity_key"].is_null())
    );
    assert!(state.projection_legacy_source_event_rows.iter().any(
        |event| event["legacy_event_kind_value"] == "definition"
            && event["legacy_source_order_parts"][0] == 0
            && event["legacy_source_order_parts"][1] == 5
    ));
}

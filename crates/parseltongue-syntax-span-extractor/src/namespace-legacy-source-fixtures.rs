#![allow(clippy::unwrap_used)]
use super::*;
use parseltongue_table_store_traits::{ItemMacroExpansionPort, ResolvedMacroLookupPort};

#[derive(Default)]
struct OrderedMacroFixtureLookup {
    bindings: std::sync::Mutex<BTreeMap<String, Value>>,
    requests: std::sync::Mutex<BTreeMap<String, String>>,
    fail: std::sync::atomic::AtomicBool,
}

impl OrderedMacroFixtureLookup {
    fn accept_namespace_fixture_rows(&self, rows: &[StageTableRowRecord]) {
        rows.iter().for_each(|row| {
            let value: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
            if row.stage_owned_table_name == "syntax-namespace-input-items"
                && value["binding_namespace_kind_value"] == "macro"
            {
                self.bindings.lock().unwrap().insert(
                    value["binding_resolved_name_text"].as_str().unwrap().into(),
                    value,
                );
            } else if row.stage_owned_table_name == "syntax-namespace-input-macros" {
                self.requests.lock().unwrap().insert(
                    value["invocation_site_occurrence_key"].to_string(),
                    value["invocation_macro_path_segments"][0]["name"]
                        .as_str()
                        .unwrap()
                        .into(),
                );
            }
        });
    }
}

impl ResolvedMacroLookupPort for OrderedMacroFixtureLookup {
    fn read_macro_invocation_binding_record(
        &self,
        key: &str,
    ) -> Result<Option<StageTableRowRecord>, NameResolutionStageError> {
        if self.fail.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(NameResolutionStageError::Cancelled);
        }
        Ok(self
            .requests
            .lock()
            .unwrap()
            .get(key)
            .and_then(|name| self.bindings.lock().unwrap().get(name).cloned())
            .map(|value| StageTableRowRecord {
                stage_owned_table_name: "namespace-macro-binding-records".into(),
                stage_row_record_key: key.into(),
                stage_row_payload_text: value.to_string(),
            }))
    }
    fn read_resolved_builtin_macro(
        &self,
        _: &NameResolutionScopeKey,
        _: &[NamePathSegmentKind],
    ) -> Result<Option<String>, NameResolutionStageError> {
        Ok(None)
    }
    fn read_macro_target_metadata(
        &self,
        target: &NameSymbolTargetKey,
    ) -> Result<Option<StageTableRowRecord>, NameResolutionStageError> {
        let key = encode_symbol_target_value(target);
        Ok(self
            .bindings
            .lock()
            .unwrap()
            .values()
            .find(|value| value["binding_target_identity_key"] == key)
            .map(|value| StageTableRowRecord {
                stage_owned_table_name: "namespace-macro-metadata-records".into(),
                stage_row_record_key: key.to_string(),
                stage_row_payload_text: value["binding_target_metadata_value"].to_string(),
            }))
    }
}

#[test]
fn ordered_macro_expansion_barriers() {
    let store=LegacyMacroFixtureStore{source:"macro_rules! make {()=>{macro_rules! nested {()=>{pub struct Made;}} nested!();}} make!(); macro_rules! later {()=>{pub struct Later;}} later!();".into()};
    let scope = StageRunScopeValue {
        run_generation_digest_value: "generation".into(),
        stage_attempt_sequence_value: "attempt".into(),
    };
    let cancel = RunCancelFlagValue::default();
    let workspace = json!({"workspace_source_root_path":"/fixture"});
    let mut projection = CrateSyntaxProjectionState::build_crate_projection_state(
        unit(),
        &workspace,
        &store,
        &scope,
        &cancel,
    )
    .unwrap();
    projection.project_unit_source_root().unwrap();
    let initial = crate::syntax_namespace_input_persistence::namespace_input_row_records(
        &projection,
        &MacroProjectionRowOffsets::default(),
    )
    .collect::<Result<Vec<_>, _>>()
    .unwrap();
    let lookup = OrderedMacroFixtureLookup::default();
    lookup.accept_namespace_fixture_rows(&initial);
    drop(projection);
    let port =
        crate::prepare_macro_expansion_port(unit(), &workspace, &store, &scope, &cancel, &lookup)
            .unwrap();
    lookup
        .fail
        .store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(port.expand_next_resolved_macro_items().is_err());
    lookup
        .fail
        .store(false, std::sync::atomic::Ordering::Relaxed);
    let first = port.expand_next_resolved_macro_items().unwrap().unwrap();
    assert_eq!(first.expansion_completed_invocation_keys.len(), 1);
    assert_eq!(first.expansion_macro_invocation_rows.len(), 1);
    lookup.accept_namespace_fixture_rows(&first.expansion_semantic_output_rows);
    let second = port.expand_next_resolved_macro_items().unwrap().unwrap();
    assert!(second.expansion_semantic_output_rows.iter().any(|row|row.stage_owned_table_name=="syntax-namespace-input-items" && serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap()["binding_resolved_name_text"]=="Made"));
    lookup.accept_namespace_fixture_rows(&second.expansion_semantic_output_rows);
    let third = port.expand_next_resolved_macro_items().unwrap().unwrap();
    assert!(third.expansion_semantic_output_rows.iter().any(|row|row.stage_owned_table_name=="syntax-namespace-input-items" && serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap()["binding_resolved_name_text"]=="Later"));
    assert!(port.expand_next_resolved_macro_items().unwrap().is_none());
    cancel.request_stage_work_cancellation();
    assert!(port.expand_next_resolved_macro_items().is_err());
}

#[test]
fn ordered_macro_expansion_empty() {
    let store = LegacyMacroFixtureStore {
        source: String::new(),
    };
    let lookup = OrderedMacroFixtureLookup::default();
    let cancel = RunCancelFlagValue::default();
    let port = crate::prepare_macro_expansion_port(
        unit(),
        &json!({"workspace_source_root_path":"/fixture"}),
        &store,
        &StageRunScopeValue {
            run_generation_digest_value: "generation".into(),
            stage_attempt_sequence_value: "attempt".into(),
        },
        &cancel,
        &lookup,
    )
    .unwrap();
    assert!(port.expand_next_resolved_macro_items().unwrap().is_none());
}

#[test]
fn unresolved_macro_completion_records() {
    let store = LegacyMacroFixtureStore {
        source: "missing!();".into(),
    };
    let lookup = OrderedMacroFixtureLookup::default();
    let cancel = RunCancelFlagValue::default();
    let port = crate::prepare_macro_expansion_port(
        unit(),
        &json!({"workspace_source_root_path":"/fixture"}),
        &store,
        &StageRunScopeValue {
            run_generation_digest_value: "generation".into(),
            stage_attempt_sequence_value: "attempt".into(),
        },
        &cancel,
        &lookup,
    )
    .unwrap();
    assert!(port.expand_next_resolved_macro_items().unwrap().is_none());
    let output = port.finish_unresolved_macro_items().unwrap();
    assert_eq!(output.expansion_unavailable_reason_rows.len(), 1);
    assert_eq!(output.expansion_semantic_output_rows.len(), 1);
    assert_eq!(
        serde_json::from_str::<Value>(
            &output.expansion_semantic_output_rows[0].stage_row_payload_text
        )
        .unwrap()["macro_resolution_state_kind"],
        "unresolved"
    );
    assert!(
        port.finish_unresolved_macro_items()
            .unwrap()
            .expansion_semantic_output_rows
            .is_empty()
    );
}

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

struct NamespacePersistenceFixture {
    source: LegacyMacroFixtureStore,
    units: Vec<Value>,
    written: std::sync::Mutex<Vec<StageTableRowRecord>>,
}

impl StageTableStorePort for NamespacePersistenceFixture {
    fn initialize_workspace_stage_schema(&self) -> Result<(), StoreBackendFailureError> {
        Ok(())
    }
    fn write_stage_table_unit(
        &self,
        request: &StageUnitWriteRequest<'_>,
        cancel: &RunCancelFlagValue,
    ) -> Result<usize, StoreBackendFailureError> {
        if cancel.check_stage_work_cancelled() {
            return Err(build_name_projection_error("cancelled"));
        }
        self.written
            .lock()
            .unwrap()
            .extend_from_slice(request.stage_unit_output_rows);
        Ok(request.stage_unit_output_rows.len())
    }
    fn read_stage_table_rows(
        &self,
        scope: &StageRunScopeValue,
        table: &str,
    ) -> Result<Vec<StageTableRowRecord>, StoreBackendFailureError> {
        let values = match table {
            "workspace-config-state-records" => {
                vec![json!({"workspace_source_root_path":"/fixture"})]
            }
            "crate-dependency-graph-table" => self.units.clone(),
            _ => return self.source.read_stage_table_rows(scope, table),
        };
        Ok(values
            .into_iter()
            .enumerate()
            .map(|(index, value)| StageTableRowRecord {
                stage_owned_table_name: table.into(),
                stage_row_record_key: index.to_string(),
                stage_row_payload_text: value.to_string(),
            })
            .collect())
    }
    fn write_file_content_bytes(
        &self,
        digest: &str,
        bytes: &[u8],
    ) -> Result<bool, StoreBackendFailureError> {
        self.source.write_file_content_bytes(digest, bytes)
    }
    fn read_file_content_bytes(&self, digest: &str) -> Result<Vec<u8>, StoreBackendFailureError> {
        self.source.read_file_content_bytes(digest)
    }
    fn publish_workspace_generation_manifest(
        &self,
        scope: &StageRunScopeValue,
        generation: &str,
        cancel: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, StoreBackendFailureError> {
        self.source
            .publish_workspace_generation_manifest(scope, generation, cancel)
    }
    fn read_workspace_publication_scope(
        &self,
    ) -> Result<Option<StageRunScopeValue>, StoreBackendFailureError> {
        Ok(None)
    }
}

#[test]
fn mapped_unit_namespace_persistence() {
    [(true,false),(false,true),(false,false)].into_iter().for_each(|(mapped,sysroot)|{
        let mut metadata=unit();
        metadata["unit_mapped_target_flag"]=json!(mapped);
        metadata["unit_sysroot_context_flag"]=json!(sysroot);
        let fixture=NamespacePersistenceFixture{source:LegacyMacroFixtureStore{source:"pub struct Persisted;".into()},units:vec![metadata],written:Default::default()};
        let count=crate::persist_namespace_syntax_inputs(&StageRunScopeValue{run_generation_digest_value:"generation".into(),stage_attempt_sequence_value:"attempt".into()},&fixture,&RunCancelFlagValue::default()).unwrap();
        let written=fixture.written.lock().unwrap();
        assert_eq!(count>0,mapped||sysroot);
        assert_eq!(written.iter().filter(|row|row.stage_owned_table_name=="syntax-namespace-input-items").any(|row|serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap()["binding_resolved_name_text"]=="Persisted"),mapped||sysroot);
    });
    let fixture = NamespacePersistenceFixture {
        source: LegacyMacroFixtureStore {
            source: "fn {".into(),
        },
        units: vec![{
            let mut value = unit();
            value["unit_mapped_target_flag"] = json!(true);
            value
        }],
        written: Default::default(),
    };
    assert!(
        crate::persist_namespace_syntax_inputs(
            &StageRunScopeValue {
                run_generation_digest_value: "generation".into(),
                stage_attempt_sequence_value: "attempt".into()
            },
            &fixture,
            &RunCancelFlagValue::default()
        )
        .is_err()
    );
    assert!(fixture.written.lock().unwrap().is_empty());
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
fn namespace_delta_empty_and_invalid_metadata() {
    let store = LegacyMacroFixtureStore {
        source: String::new(),
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
    let offsets = read_projection_row_offsets(&state);
    assert_eq!(
        crate::syntax_namespace_input_persistence::namespace_input_row_records(&state, &offsets)
            .count(),
        0
    );
    let rows = crate::syntax_namespace_input_persistence::namespace_input_row_records(
        &state,
        &MacroProjectionRowOffsets::default(),
    )
    .collect::<Result<Vec<_>, _>>()
    .unwrap();
    assert!(rows.iter().filter(|row|row.stage_owned_table_name=="syntax-namespace-input-scopes").all(|row|serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap()["scope_cargo_unit_key"]=="fixture-unit"));
    state.projection_target_metadata_rows.clear();
    let invalid = crate::syntax_namespace_input_persistence::namespace_input_row_records(
        &state,
        &MacroProjectionRowOffsets::default(),
    )
    .collect::<Result<Vec<_>, _>>();
    assert!(invalid.is_err());
    assert!(state.read_legacy_lookup_updates().unwrap().is_empty());
    state.projection_legacy_source_event_rows.push(Value::Null);
    assert!(state.read_legacy_lookup_updates().is_err());
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
    let mut invalid = state.projection_legacy_source_event_rows[0].clone();
    invalid["legacy_event_kind_value"] = json!("exit");
    invalid["legacy_source_order_parts"] = json!([9999, 0, 0]);
    state.projection_legacy_source_event_rows.push(invalid);
    assert!(state.read_legacy_lookup_updates().is_err());
    state.projection_legacy_source_event_rows.pop();
    let initial = state.read_legacy_lookup_updates().unwrap();
    assert!(!initial.is_empty());
    assert!(initial.iter().all(|row| {
        serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap()["legacy_scope_revision_value"]
            == 0
    }));
    assert!(state.read_legacy_lookup_updates().unwrap().is_empty());
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
    let delta = state
        .project_macro_expansion_rows(&invocation, &definition, &expanded)
        .unwrap();
    let items = delta
        .expansion_semantic_output_rows
        .iter()
        .filter(|row| row.stage_owned_table_name == "syntax-namespace-input-items")
        .collect::<Vec<_>>();
    assert!(!items.is_empty());
    assert!(items.iter().all(|row|serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap()["binding_target_metadata_value"].is_object()));
    assert!(
        delta
            .expansion_semantic_output_rows
            .iter()
            .any(|row| row.stage_owned_table_name == "legacy-macro-source-events")
    );
    assert!(
        !delta
            .expansion_semantic_output_rows
            .iter()
            .any(|row| row.stage_owned_table_name == "syntax-namespace-input-macros")
    );
    let updates = delta
        .expansion_semantic_output_rows
        .iter()
        .filter(|row| row.stage_owned_table_name == "legacy-macro-scope-records")
        .map(|row| serde_json::from_str::<Value>(&row.stage_row_payload_text).unwrap())
        .collect::<Vec<_>>();
    assert!(!updates.is_empty());
    assert!(
        updates
            .iter()
            .all(|row| row["legacy_macro_name_text"] == "generated"
                && row["legacy_scope_revision_value"] == 1
                && !row["legacy_target_identity_key"].is_null())
    );
    assert!(state.read_legacy_lookup_updates().unwrap().is_empty());
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

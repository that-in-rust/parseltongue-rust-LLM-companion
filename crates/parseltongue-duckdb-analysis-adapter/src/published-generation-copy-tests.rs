#![allow(clippy::unwrap_used)]

use super::*;
use parseltongue_core_row_types::{StageTableRowRecord, StageUnitWriteRequest};
use serde_json::json;
use std::collections::BTreeMap;

fn namespace_source_fixture() -> Source {
    let unit = json!({"unit_cargo_package_identifier":"fixture","unit_cargo_target_name":"fixture","unit_cargo_target_kind":"lib","unit_compilation_role_kind":"target","unit_configuration_digest_full":"0".repeat(64)});
    let root = json!({"scope_compilation_unit_key":unit,"scope_semantic_path_parts":[]});
    let item = json!({"target_identity_variant_kind":"builtin","target_builtin_name_value":"u8"});
    let scope_row = json!({"scope_module_identity_key":root,"scope_parent_identity_key":null,"scope_parent_lookup_flag":false,"scope_module_boundary_flag":true,"scope_implicit_prelude_key":null,"scope_cargo_unit_key":"fixture-unit"});
    let binding = json!({"binding_lexical_scope_key":root,"binding_resolved_name_text":"r#Item","binding_namespace_kind_value":"type","binding_target_identity_key":item,"binding_target_child_scope":null,"binding_visibility_scope_kind":{"visibility_scope_kind_value":"public"},"binding_selection_priority_kind":"declaration","binding_import_evidence_keys":[],"binding_target_metadata_value":{"target_symbol_kind_value":"Struct"}});
    let path = json!({"path_site_occurrence_key":{"site":"path"},"path_lexical_scope_key":root,"path_source_segment_rows":[{"kind":"name","name":"r#Item"}],"path_requested_namespace_kind":"Type","path_resolution_source_kind":"Signature","path_unavailable_reason_kind":null,"path_source_metadata_value":{"path_source_absolute_path":"/fixture/lib.rs"}});
    let mut records = NAMESPACE_SOURCE_TABLE_NAMES
        .iter()
        .map(|table| (table.to_string(), vec![]))
        .collect::<BTreeMap<_, _>>();
    for (table, value) in [
        ("syntax-namespace-input-scopes", scope_row),
        ("syntax-namespace-input-items", binding),
        ("syntax-namespace-input-paths", path),
        (
            "crate-dependency-graph-table",
            json!({"unit_compilation_identity_key":"fixture-unit","unit_rust_edition_value":"2024","unit_normal_dependency_rows":[]}),
        ),
    ] {
        let text = value.to_string();
        records
            .entry(table.into())
            .or_default()
            .push(StageTableRowRecord {
                stage_owned_table_name: table.into(),
                stage_row_record_key: blake3::hash(text.as_bytes()).to_hex().to_string(),
                stage_row_payload_text: text,
            });
    }
    Source {
        published: Some(scope()),
        records,
        fail: false,
        wrong_count: false,
    }
}

#[test]
fn frozen_namespace_normalization_contract() {
    let directory = tempfile::tempdir().unwrap();
    let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
        &directory.path().join("analysis.duckdb"),
        directory.path(),
        64 * 1024 * 1024,
    )
    .unwrap();
    store
        .copy_published_stage_rows(
            &namespace_source_fixture(),
            &scope(),
            NAMESPACE_SOURCE_TABLE_NAMES,
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    assert_eq!(
        store
            .normalize_frozen_namespace_rows(&scope(), &RunCancelFlagValue::default())
            .unwrap(),
        4
    );
    let connection = store._connection.lock().unwrap();
    let name:String=connection.query_row("SELECT name FROM \"namespace-binding-declaration-records\" ORDER BY generation_key,attempt_key,scope_key,name,namespace,target_key,priority,evidence_key",[],|row|row.get(0)).unwrap();
    assert_eq!(name, "Item");
    let segments: usize = connection
        .query_row(
            "SELECT count(*) FROM \"namespace-path-segment-records\"",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(segments, 1);
    let generation:String=connection.query_row("SELECT generation_key FROM \"namespace-normalized-source-manifests\" ORDER BY generation_key,attempt_key",[],|row|row.get(0)).unwrap();
    assert_eq!(generation, scope().run_generation_digest_value);
}

#[test]
fn frozen_namespace_generation_integrity() {
    let directory = tempfile::tempdir().unwrap();
    let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
        &directory.path().join("analysis.duckdb"),
        directory.path(),
        64 * 1024 * 1024,
    )
    .unwrap();
    let first = scope();
    let second = StageRunScopeValue {
        run_generation_digest_value: "second".into(),
        ..scope()
    };
    let mut source = namespace_source_fixture();
    [&first, &second].into_iter().for_each(|scope| {
        source.published = Some(scope.clone());
        store
            .copy_published_stage_rows(
                &source,
                scope,
                NAMESPACE_SOURCE_TABLE_NAMES,
                &RunCancelFlagValue::default(),
            )
            .unwrap();
        assert_eq!(
            store
                .normalize_frozen_namespace_rows(scope, &RunCancelFlagValue::default())
                .unwrap(),
            4
        );
    });
    let connection = store._connection.lock().unwrap();
    let count:usize=connection.query_row("SELECT count(*) FROM \"namespace-binding-declaration-records\" WHERE generation_key='generation'",[],|row|row.get(0)).unwrap();
    assert_eq!(count, 1);
    drop(connection);
    let invalid = StageRunScopeValue {
        run_generation_digest_value: "corrupt".into(),
        ..scope()
    };
    source.published = Some(invalid.clone());
    store
        .copy_published_stage_rows(
            &source,
            &invalid,
            NAMESPACE_SOURCE_TABLE_NAMES,
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    store._connection.lock().unwrap().execute("UPDATE \"frozen-generation-source-records\" SET payload=replace(payload,'\"scope_parent_lookup_flag\":false','\"scope_parent_lookup_flag\":true') WHERE generation_key='corrupt' AND table_name='syntax-namespace-input-scopes'",[]).unwrap();
    let error = store
        .normalize_frozen_namespace_rows(&invalid, &RunCancelFlagValue::default())
        .unwrap_err();
    assert!(error.to_string().contains("digest changed"), "{error}");
    let count:usize=store._connection.lock().unwrap().query_row("SELECT count(*) FROM \"namespace-normalized-source-manifests\" WHERE generation_key='corrupt'",[],|row|row.get(0)).unwrap();
    assert_eq!(count, 0);
}

fn append_namespace_fixture_value(source: &mut Source, table: &str, value: serde_json::Value) {
    let payload = value.to_string();
    let rows = source.records.get_mut(table).unwrap();
    rows.push(StageTableRowRecord {
        stage_owned_table_name: table.into(),
        stage_row_record_key: blake3::hash(payload.as_bytes()).to_hex().to_string(),
        stage_row_payload_text: payload,
    });
    rows.sort_by(|left, right| left.stage_row_record_key.cmp(&right.stage_row_record_key));
}

fn append_compilation_fixture(
    source: &mut Source,
    unit: &serde_json::Value,
    graph: serde_json::Value,
) -> serde_json::Value {
    let root = json!({"scope_compilation_unit_key":unit,"scope_semantic_path_parts":[]});
    let prelude =
        json!({"scope_compilation_unit_key":unit,"scope_semantic_path_parts":["@prelude"]});
    [vec![],vec!["@prelude"],vec!["@extern"]].into_iter().for_each(|parts|{
        append_namespace_fixture_value(source,"syntax-namespace-input-scopes",json!({"scope_module_identity_key":{"scope_compilation_unit_key":unit,"scope_semantic_path_parts":parts},"scope_parent_identity_key":null,"scope_parent_lookup_flag":false,"scope_module_boundary_flag":parts.is_empty(),"scope_implicit_prelude_key":if parts.is_empty(){prelude.clone()}else{serde_json::Value::Null},"scope_cargo_unit_key":graph["unit_compilation_identity_key"]}));
    });
    append_namespace_fixture_value(source, "crate-dependency-graph-table", graph);
    root
}

#[test]
fn frozen_compilation_dependency_alias_rules() {
    ["2015","2018"].into_iter().for_each(|edition|[false,true].into_iter().for_each(|declared|[false,true].into_iter().for_each(|implicit|{
        let directory=tempfile::tempdir().unwrap();
        let store=DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(&directory.path().join("analysis.duckdb"),directory.path(),64*1024*1024).unwrap();
        let mut source=namespace_source_fixture();
        let original:serde_json::Value=serde_json::from_str(&source.records["syntax-namespace-input-scopes"][0].stage_row_payload_text).unwrap();
        let unit=&original["scope_module_identity_key"]["scope_compilation_unit_key"];
        let dependency=json!({"unit_cargo_package_identifier":"dep-package","unit_cargo_target_name":"dep","unit_cargo_target_kind":"lib","unit_compilation_role_kind":"target","unit_configuration_digest_full":"1".repeat(64)});
        source.records.get_mut("crate-dependency-graph-table").unwrap().clear();
        source.records.get_mut("syntax-namespace-input-scopes").unwrap().clear();
        let root=append_compilation_fixture(&mut source,unit,json!({"unit_compilation_identity_key":"fixture-unit","unit_rust_edition_value":edition,"unit_no_core_flag":true,"unit_no_std_flag":true,"unit_implicit_prelude_flag":implicit,"unit_normal_dependency_rows":[{"dependency_alias_name_value":"dep_alias","dependency_cargo_package_identifier":"dep-package","dependency_compilation_role_kind":"target"}]}));
        let dependency_root=append_compilation_fixture(&mut source,&dependency,json!({"unit_compilation_identity_key":"dep-unit","unit_rust_edition_value":"2018","unit_no_core_flag":true,"unit_no_std_flag":true,"unit_implicit_prelude_flag":false,"unit_normal_dependency_rows":[]}));
        let target=json!({"target_identity_variant_kind":"builtin","target_builtin_name_value":"DependencyThing"});
        append_namespace_fixture_value(&mut source,"syntax-namespace-input-items",json!({"binding_lexical_scope_key":dependency_root,"binding_resolved_name_text":"Thing","binding_namespace_kind_value":"type","binding_target_identity_key":target,"binding_target_child_scope":null,"binding_visibility_scope_kind":{"visibility_scope_kind_value":"public"},"binding_selection_priority_kind":"declaration","binding_import_evidence_keys":[],"binding_target_metadata_value":{"target_symbol_kind_value":"Struct"}}));
        if declared {
            append_namespace_fixture_value(&mut source,"syntax-namespace-input-externs",json!({"extern_declaration_scope_key":root,"extern_declaration_source_name":"dep_alias","extern_declaration_alias_name":"renamed","extern_declaration_visibility":{"visibility_scope_kind_value":"public"},"extern_declaration_attributes":[],"extern_declaration_source_path":"/fixture/lib.rs","extern_declaration_source_start":0}));
        }
        append_namespace_fixture_value(&mut source,"syntax-namespace-input-paths",json!({"path_site_occurrence_key":{"site":"dependency-path"},"path_lexical_scope_key":root,"path_source_segment_rows":[{"kind":"name","name":if declared {"renamed"}else{"dep_alias"}},{"kind":"name","name":"Thing"}],"path_requested_namespace_kind":"Type","path_resolution_source_kind":"Signature","path_unavailable_reason_kind":null,"path_source_metadata_value":{}}));
        store.copy_published_stage_rows(&source,&scope(),NAMESPACE_SOURCE_TABLE_NAMES,&RunCancelFlagValue::default()).unwrap();
        store.normalize_frozen_namespace_rows(&scope(),&RunCancelFlagValue::default()).unwrap();
        store.link_frozen_namespace_units(&scope(),&RunCancelFlagValue::default()).unwrap();
        store.resolve_frozen_namespace_rows(&scope(),&RunCancelFlagValue::default()).unwrap();
        let count:usize=store._connection.lock().unwrap().query_row("SELECT count(*) FROM \"namespace-path-candidate-records\" WHERE request_key=? AND target_key=? AND state='resolved'",duckdb::params![json!({"site":"dependency-path"}).to_string(),target.to_string()],|row|row.get(0)).unwrap();
        assert_eq!(count,usize::from(declared || (edition!="2015" && implicit)),"edition={edition} declared={declared} implicit={implicit}");
    })));
}

fn append_standard_library_fixture(source: &mut Source, name: &str) {
    let unit = json!({"unit_cargo_package_identifier":format!("sysroot-{name}"),"unit_cargo_target_name":name,"unit_cargo_target_kind":"lib","unit_compilation_role_kind":"target","unit_configuration_digest_full":"2".repeat(64)});
    let cargo = format!("sysroot-{name}");
    let root = append_compilation_fixture(
        source,
        &unit,
        json!({"unit_compilation_identity_key":cargo,"unit_rust_edition_value":"2021","unit_no_core_flag":true,"unit_no_std_flag":true,"unit_implicit_prelude_flag":true,"unit_sysroot_context_flag":true,"unit_normal_dependency_rows":[]}),
    );
    let prelude =
        json!({"scope_compilation_unit_key":unit,"scope_semantic_path_parts":["prelude"]});
    let edition = json!({"scope_compilation_unit_key":unit,"scope_semantic_path_parts":["prelude","rust_2024"]});
    [(prelude.clone(),root.clone()),(edition.clone(),prelude.clone())].into_iter().for_each(|(scope,parent)|{
        append_namespace_fixture_value(source,"syntax-namespace-input-scopes",json!({"scope_module_identity_key":scope,"scope_parent_identity_key":parent,"scope_parent_lookup_flag":false,"scope_module_boundary_flag":true,"scope_implicit_prelude_key":null,"scope_cargo_unit_key":cargo}));
    });
    let target = |suffix: &str| json!({"target_identity_variant_kind":"builtin","target_builtin_name_value":format!("{name}-{suffix}")});
    [(root.clone(),"prelude",target("prelude"),Some(prelude.clone()),"type",json!({"target_symbol_kind_value":"Module"})),
        (prelude,"rust_2024",target("edition"),Some(edition.clone()),"type",json!({"target_symbol_kind_value":"Module"})),
        (edition,"Selected",target("Selected"),None,"type",json!({"target_symbol_kind_value":"Struct"})),
        (root,"sdk_macro",target("macro"),None,"macro",json!({"target_symbol_kind_value":"Macro","target_macro_definition_record":{"macro_declaration_style_flag":false}}))
    ].into_iter().for_each(|(scope,name,target,child,namespace,metadata)|{
        append_namespace_fixture_value(source,"syntax-namespace-input-items",json!({"binding_lexical_scope_key":scope,"binding_resolved_name_text":name,"binding_namespace_kind_value":namespace,"binding_target_identity_key":target,"binding_target_child_scope":child,"binding_visibility_scope_kind":{"visibility_scope_kind_value":"public"},"binding_selection_priority_kind":"declaration","binding_import_evidence_keys":[],"binding_target_metadata_value":metadata}));
    });
}

#[test]
fn frozen_standard_type_macro_preludes() {
    [(false,true),(true,true),(false,false)].into_iter().for_each(|(no_std,implicit)|{
        let directory=tempfile::tempdir().unwrap();
        let store=DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(&directory.path().join("analysis.duckdb"),directory.path(),64*1024*1024).unwrap();
        let mut source=namespace_source_fixture();
        let original:serde_json::Value=serde_json::from_str(&source.records["syntax-namespace-input-scopes"][0].stage_row_payload_text).unwrap();
        let unit=&original["scope_module_identity_key"]["scope_compilation_unit_key"];
        source.records.get_mut("crate-dependency-graph-table").unwrap().clear();
        source.records.get_mut("syntax-namespace-input-scopes").unwrap().clear();
        let dependencies=["core","std"].map(|name|json!({"dependency_alias_name_value":name,"dependency_cargo_package_identifier":format!("sysroot-{name}"),"dependency_compilation_role_kind":"target","dependency_sysroot_context_flag":true}));
        let root=append_compilation_fixture(&mut source,unit,json!({"unit_compilation_identity_key":"fixture-unit","unit_rust_edition_value":"2024","unit_no_core_flag":false,"unit_no_std_flag":no_std,"unit_implicit_prelude_flag":implicit,"unit_normal_dependency_rows":dependencies}));
        ["core","std"].into_iter().for_each(|name|append_standard_library_fixture(&mut source,name));
        append_namespace_fixture_value(&mut source,"syntax-namespace-input-paths",json!({"path_site_occurrence_key":{"site":"standard-type"},"path_lexical_scope_key":root,"path_source_segment_rows":[{"kind":"name","name":"Selected"}],"path_requested_namespace_kind":"Type","path_resolution_source_kind":"Signature","path_unavailable_reason_kind":null,"path_source_metadata_value":{}}));
        append_namespace_fixture_value(&mut source,"syntax-namespace-input-macros",json!({"invocation_site_occurrence_key":{"site":"standard-macro"},"invocation_lexical_scope_key":root,"invocation_macro_path_segments":[{"kind":"name","name":"sdk_macro"}],"invocation_macro_input_text":""}));
        store.copy_published_stage_rows(&source,&scope(),NAMESPACE_SOURCE_TABLE_NAMES,&RunCancelFlagValue::default()).unwrap();
        store.normalize_frozen_namespace_rows(&scope(),&RunCancelFlagValue::default()).unwrap();
        store.resolve_frozen_namespace_rows(&scope(),&RunCancelFlagValue::default()).unwrap();
        ["standard-type","standard-macro"].into_iter().for_each(|site|{
            let connection=store._connection.lock().unwrap();
            let mut statement=connection.prepare("SELECT target_key FROM \"namespace-path-candidate-records\" WHERE request_key=? ORDER BY generation_key,attempt_key,request_key,target_key,namespace,binding_scope,binding_name").unwrap();
            let targets=statement.query_map([json!({"site":site}).to_string()],|row|row.get::<_,String>(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
            let library=if no_std {"core"}else{"std"};
            let suffix=if site=="standard-type" {"Selected"}else{"macro"};
            let expected=if implicit {vec![json!({"target_identity_variant_kind":"builtin","target_builtin_name_value":format!("{library}-{suffix}")}).to_string()]}else{vec![]};
            assert_eq!(targets,expected,"no_std={no_std} implicit={implicit} site={site}");
        });
    });
}

#[test]
fn frozen_namespace_sql_resolution_contract() {
    let directory = tempfile::tempdir().unwrap();
    let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
        &directory.path().join("analysis.duckdb"),
        directory.path(),
        64 * 1024 * 1024,
    )
    .unwrap();
    assert!(
        store
            .link_frozen_namespace_units(&scope(), &RunCancelFlagValue::default())
            .is_err()
    );
    assert!(
        store
            .resolve_frozen_namespace_rows(&scope(), &RunCancelFlagValue::default())
            .is_err()
    );
    let mut source = namespace_source_fixture();
    store
        .copy_published_stage_rows(
            &source,
            &scope(),
            NAMESPACE_SOURCE_TABLE_NAMES,
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    store
        .normalize_frozen_namespace_rows(&scope(), &RunCancelFlagValue::default())
        .unwrap();
    let report = store
        .resolve_frozen_namespace_rows(&scope(), &RunCancelFlagValue::default())
        .unwrap();
    assert_eq!(report.binding_set_digest_value.len(), 64);
    assert!(report.fixed_point_round_count > 0);
    let count:usize=store._connection.lock().unwrap().query_row("SELECT count(*) FROM \"namespace-path-candidate-records\" WHERE namespace='type' AND state='resolved'",[],|row|row.get(0)).unwrap();
    assert_eq!(count, 1);
    let cancelled = RunCancelFlagValue::default();
    cancelled.request_stage_work_cancellation();
    assert!(
        store
            .link_frozen_namespace_units(&scope(), &cancelled)
            .is_err()
    );
    assert!(
        store
            .resolve_frozen_namespace_rows(&scope(), &cancelled)
            .is_err()
    );
    let empty = StageRunScopeValue {
        run_generation_digest_value: "empty".into(),
        ..scope()
    };
    source.published = Some(empty.clone());
    source.records.values_mut().for_each(Vec::clear);
    store
        .copy_published_stage_rows(
            &source,
            &empty,
            NAMESPACE_SOURCE_TABLE_NAMES,
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    store
        .normalize_frozen_namespace_rows(&empty, &RunCancelFlagValue::default())
        .unwrap();
    assert_eq!(
        store
            .resolve_frozen_namespace_rows(&empty, &RunCancelFlagValue::default())
            .unwrap()
            .fixed_point_round_count,
        1
    );
    let invalid = StageRunScopeValue {
        run_generation_digest_value: "invalid-extern".into(),
        ..scope()
    };
    let mut source = namespace_source_fixture();
    source.published = Some(invalid.clone());
    let scope_row: serde_json::Value = serde_json::from_str(
        &source.records["syntax-namespace-input-scopes"][0].stage_row_payload_text,
    )
    .unwrap();
    append_namespace_fixture_value(
        &mut source,
        "syntax-namespace-input-externs",
        json!({"extern_declaration_scope_key":scope_row["scope_module_identity_key"]}),
    );
    store
        .copy_published_stage_rows(
            &source,
            &invalid,
            NAMESPACE_SOURCE_TABLE_NAMES,
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    store
        .normalize_frozen_namespace_rows(&invalid, &RunCancelFlagValue::default())
        .unwrap();
    assert!(
        store
            .link_frozen_namespace_units(&invalid, &RunCancelFlagValue::default())
            .is_err()
    );
    let count:usize=store._connection.lock().unwrap().query_row("SELECT count(*) FROM \"namespace-unit-link-records\" WHERE generation_key='invalid-extern'",[],|row|row.get(0)).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn frozen_namespace_latest_textual_lookup() {
    [false,true].into_iter().for_each(|null_latest|{
        let directory=tempfile::tempdir().unwrap();
        let store=DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(&directory.path().join("analysis.duckdb"),directory.path(),64*1024*1024).unwrap();
        let mut source=namespace_source_fixture();
        let scope_row:serde_json::Value=serde_json::from_str(&source.records["syntax-namespace-input-scopes"][0].stage_row_payload_text).unwrap();
        let root=&scope_row["scope_module_identity_key"];
        let target=|name:&str|json!({"target_identity_variant_kind":"builtin","target_builtin_name_value":name});
        ["old","new","module"].into_iter().for_each(|name|{
            append_namespace_fixture_value(&mut source,"syntax-namespace-input-items",json!({"binding_lexical_scope_key":root,"binding_resolved_name_text":"m","binding_namespace_kind_value":"macro","binding_target_identity_key":target(name),"binding_target_child_scope":null,"binding_visibility_scope_kind":{"visibility_scope_kind_value":"restricted","visibility_restricted_scope_key":root},"binding_selection_priority_kind":"declaration","binding_import_evidence_keys":[],"binding_target_metadata_value":{"target_symbol_kind_value":"Macro","target_macro_definition_record":{"macro_declaration_style_flag":name=="module"}}}));
        });
        append_namespace_fixture_value(&mut source,"syntax-namespace-input-macros",json!({"invocation_site_occurrence_key":{"site":"macro"},"invocation_lexical_scope_key":root,"invocation_macro_path_segments":[{"kind":"name","name":"m"}],"invocation_macro_input_text":""}));
        append_namespace_fixture_value(&mut source,"syntax-namespace-input-imports",json!({"import_site_occurrence_key":{"site":"import"},"import_lexical_scope_key":root,"import_path_segment_rows":[{"kind":"name","name":"m"}],"import_binding_alias_text":"Alias","import_glob_expansion_flag":false,"import_visibility_scope_kind":{"visibility_scope_kind_value":"public"}}));
        ["macro","import"].into_iter().for_each(|site|{
            [(1,target("old")),(2,if null_latest {serde_json::Value::Null}else{target("new")})].into_iter().for_each(|(revision,target)|{
                append_namespace_fixture_value(&mut source,"legacy-macro-scope-records",json!({"legacy_lookup_identity_key":format!("lookup-{site}"),"legacy_source_site_key":{"site":site},"legacy_macro_name_text":"m","legacy_target_identity_key":target,"legacy_scope_revision_value":revision,"legacy_source_order_parts":[10,4]}));
            });
        });
        store.copy_published_stage_rows(&source,&scope(),NAMESPACE_SOURCE_TABLE_NAMES,&RunCancelFlagValue::default()).unwrap();
        store.normalize_frozen_namespace_rows(&scope(),&RunCancelFlagValue::default()).unwrap();
        store.resolve_frozen_namespace_rows(&scope(),&RunCancelFlagValue::default()).unwrap();
        let connection=store._connection.lock().unwrap();
        let resolved:String=connection.query_row("SELECT target_key FROM \"namespace-path-candidate-records\" WHERE request_key=? AND namespace='macro' ORDER BY generation_key,attempt_key,request_key,target_key,namespace,binding_scope,binding_name",[json!({"site":"macro"}).to_string()],|row|row.get(0)).unwrap();
        assert_eq!(resolved,target(if null_latest {"module"}else{"new"}).to_string());
        let imported:String=connection.query_row("SELECT target_key FROM \"namespace-binding-round-results\" WHERE name='Alias' ORDER BY generation_key,attempt_key,scope_key,name,namespace,target_key,evidence_key",[],|row|row.get(0)).unwrap();
        assert_eq!(imported,resolved);
        let private:usize=connection.query_row("SELECT count(*) FROM \"namespace-binding-declaration-records\" WHERE target_key IN (?,?)",duckdb::params![target("old").to_string(),target("new").to_string()],|row|row.get(0)).unwrap();
        assert_eq!(private,0);
        drop(connection);
        let invalid=StageRunScopeValue {stage_attempt_sequence_value:"conflicting-legacy".into(),..scope()};
        source.published=Some(invalid.clone());
        append_namespace_fixture_value(&mut source,"legacy-macro-scope-records",json!({"legacy_lookup_identity_key":"lookup-macro","legacy_source_site_key":{"site":"macro"},"legacy_macro_name_text":"m","legacy_target_identity_key":target("old"),"legacy_scope_revision_value":2,"legacy_source_order_parts":[10,4]}));
        store.copy_published_stage_rows(&source,&invalid,NAMESPACE_SOURCE_TABLE_NAMES,&RunCancelFlagValue::default()).unwrap();
        store.normalize_frozen_namespace_rows(&invalid,&RunCancelFlagValue::default()).unwrap();
        let error=store.resolve_frozen_namespace_rows(&invalid,&RunCancelFlagValue::default()).unwrap_err();
        assert!(error.to_string().contains("conflicting textual lookup revision"),"{error}");
    });
}

#[test]
fn frozen_namespace_normalization_empty_and_errors() {
    let directory = tempfile::tempdir().unwrap();
    let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
        &directory.path().join("analysis.duckdb"),
        directory.path(),
        64 * 1024 * 1024,
    )
    .unwrap();
    assert!(
        store
            .normalize_frozen_namespace_rows(&scope(), &RunCancelFlagValue::default())
            .is_err()
    );
    let mut source = namespace_source_fixture();
    source.records.values_mut().for_each(Vec::clear);
    store
        .copy_published_stage_rows(
            &source,
            &scope(),
            NAMESPACE_SOURCE_TABLE_NAMES,
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    assert_eq!(
        store
            .normalize_frozen_namespace_rows(&scope(), &RunCancelFlagValue::default())
            .unwrap(),
        0
    );
    let cancelled = RunCancelFlagValue::default();
    cancelled.request_stage_work_cancellation();
    assert!(
        store
            .normalize_frozen_namespace_rows(&scope(), &cancelled)
            .is_err()
    );
    let invalid = StageRunScopeValue {
        stage_attempt_sequence_value: "invalid".into(),
        ..scope()
    };
    source.published = Some(invalid.clone());
    source
        .records
        .get_mut("syntax-namespace-input-scopes")
        .unwrap()
        .push(StageTableRowRecord {
            stage_owned_table_name: "syntax-namespace-input-scopes".into(),
            stage_row_record_key: "bad".into(),
            stage_row_payload_text: "null".into(),
        });
    store
        .copy_published_stage_rows(
            &source,
            &invalid,
            NAMESPACE_SOURCE_TABLE_NAMES,
            &RunCancelFlagValue::default(),
        )
        .unwrap();
    assert!(
        store
            .normalize_frozen_namespace_rows(&invalid, &RunCancelFlagValue::default())
            .is_err()
    );
    let connection = store._connection.lock().unwrap();
    let count:usize=connection.query_row("SELECT count(*) FROM \"namespace-normalized-source-manifests\" WHERE attempt_key='invalid'",[],|row|row.get(0)).unwrap();
    assert_eq!(count, 0);
}

struct Source {
    published: Option<StageRunScopeValue>,
    records: BTreeMap<String, Vec<StageTableRowRecord>>,
    fail: bool,
    wrong_count: bool,
}

impl StageTableStorePort for Source {
    fn count_stage_table_rows(
        &self,
        scope: &StageRunScopeValue,
        table: &str,
    ) -> Result<usize, StoreBackendFailureError> {
        self.read_stage_table_rows(scope, table)
            .map(|rows| rows.len() + usize::from(self.wrong_count))
    }
    fn initialize_workspace_stage_schema(&self) -> Result<(), StoreBackendFailureError> {
        Ok(())
    }
    fn write_stage_table_unit(
        &self,
        _: &StageUnitWriteRequest<'_>,
        _: &RunCancelFlagValue,
    ) -> Result<usize, StoreBackendFailureError> {
        Err(build_analysis_error("read only"))
    }
    fn read_stage_table_rows(
        &self,
        _: &StageRunScopeValue,
        table: &str,
    ) -> Result<Vec<StageTableRowRecord>, StoreBackendFailureError> {
        if self.fail {
            return Err(build_analysis_error("source failure"));
        }
        Ok(self.records.get(table).cloned().unwrap_or_default())
    }
    fn write_file_content_bytes(
        &self,
        _: &str,
        _: &[u8],
    ) -> Result<bool, StoreBackendFailureError> {
        Err(build_analysis_error("read only"))
    }
    fn read_file_content_bytes(&self, _: &str) -> Result<Vec<u8>, StoreBackendFailureError> {
        Err(build_analysis_error("read only"))
    }
    fn publish_workspace_generation_manifest(
        &self,
        _: &StageRunScopeValue,
        _: &str,
        _: &RunCancelFlagValue,
    ) -> Result<StageRunScopeValue, StoreBackendFailureError> {
        Err(build_analysis_error("read only"))
    }
    fn read_workspace_publication_scope(
        &self,
    ) -> Result<Option<StageRunScopeValue>, StoreBackendFailureError> {
        Ok(self.published.clone())
    }
}

fn scope() -> StageRunScopeValue {
    StageRunScopeValue {
        run_generation_digest_value: "generation".into(),
        stage_attempt_sequence_value: "attempt".into(),
    }
}

fn source() -> Source {
    Source {
        published: Some(scope()),
        fail: false,
        wrong_count: false,
        records: BTreeMap::from([(
            "fixture-source-input-records".into(),
            vec![
                StageTableRowRecord {
                    stage_owned_table_name: "fixture-source-input-records".into(),
                    stage_row_record_key: "a".into(),
                    stage_row_payload_text: "unicode λ\0payload".into(),
                },
                StageTableRowRecord {
                    stage_owned_table_name: "fixture-source-input-records".into(),
                    stage_row_record_key: "b".into(),
                    stage_row_payload_text: "{}".into(),
                },
            ],
        )]),
    }
}

#[test]
fn test_published_generation_copy_equality() {
    let directory = tempfile::tempdir().unwrap();
    let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
        &directory.path().join("analysis.db"),
        directory.path(),
        1_288_490_188,
    )
    .unwrap();
    let source = source();
    let cancel = RunCancelFlagValue::default();
    assert_eq!(
        store
            .copy_published_stage_rows(
                &source,
                &scope(),
                &[
                    "fixture-source-input-records",
                    "fixture-empty-input-records"
                ],
                &cancel
            )
            .unwrap(),
        2
    );
    assert_eq!(
        store
            .count_analysis_stage_rows(&scope(), "fixture-source-input-records")
            .unwrap(),
        2
    );
    assert_eq!(
        store
            .count_analysis_stage_rows(&scope(), "fixture-empty-input-records")
            .unwrap(),
        0
    );
    assert!(
        store
            .count_analysis_stage_rows(&scope(), "not-copied-input-records")
            .is_err()
    );
    assert!(
        store
            .copy_published_stage_rows(
                &source,
                &scope(),
                &["fixture-source-input-records"],
                &cancel
            )
            .is_err()
    );
}

#[test]
fn test_copy_rejects_partial_cancelled_and_failed_sources() {
    let directory = tempfile::tempdir().unwrap();
    let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
        &directory.path().join("analysis.db"),
        directory.path(),
        1_288_490_188,
    )
    .unwrap();
    let mut source = source();
    let cancel = RunCancelFlagValue::default();
    source.published = None;
    assert!(
        store
            .copy_published_stage_rows(
                &source,
                &scope(),
                &["fixture-source-input-records"],
                &cancel
            )
            .is_err()
    );
    source.published = Some(scope());
    source.fail = true;
    assert!(
        store
            .copy_published_stage_rows(
                &source,
                &scope(),
                &["fixture-source-input-records"],
                &cancel
            )
            .is_err()
    );
    assert!(
        store
            .count_analysis_stage_rows(&scope(), "fixture-source-input-records")
            .is_err()
    );
    source.fail = false;
    cancel.request_stage_work_cancellation();
    assert!(
        store
            .copy_published_stage_rows(
                &source,
                &scope(),
                &["fixture-source-input-records"],
                &cancel
            )
            .is_err()
    );
    assert!(
        store
            .count_analysis_stage_rows(&scope(), "fixture-source-input-records")
            .is_err()
    );
}

#[test]
fn test_count_mismatch_and_order_errors_rollback() {
    let directory = tempfile::tempdir().unwrap();
    let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
        &directory.path().join("analysis.db"),
        directory.path(),
        1_288_490_188,
    )
    .unwrap();
    let mut source = source();
    source.wrong_count = true;
    assert!(
        store
            .copy_published_stage_rows(
                &source,
                &scope(),
                &["fixture-source-input-records"],
                &RunCancelFlagValue::default()
            )
            .is_err()
    );
    assert!(
        store
            .count_analysis_stage_rows(&scope(), "fixture-source-input-records")
            .is_err()
    );
    source.wrong_count = false;
    source
        .records
        .get_mut("fixture-source-input-records")
        .unwrap()
        .reverse();
    assert!(
        store
            .copy_published_stage_rows(
                &source,
                &scope(),
                &["fixture-source-input-records"],
                &RunCancelFlagValue::default()
            )
            .is_err()
    );
    assert!(
        store
            .count_analysis_stage_rows(&scope(), "fixture-source-input-records")
            .is_err()
    );
    source
        .records
        .get_mut("fixture-source-input-records")
        .unwrap()
        .reverse();
    assert_eq!(
        store
            .copy_published_stage_rows(
                &source,
                &scope(),
                &["fixture-source-input-records"],
                &RunCancelFlagValue::default()
            )
            .unwrap(),
        2
    );
}

#[test]
fn test_duckdb_thread_export_determinism() {
    let exports = [1,4].map(|threads| {
        let directory = tempfile::tempdir().unwrap();
        let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(&directory.path().join("analysis.db"),directory.path(),1_288_490_188).unwrap();
        store._connection.lock().unwrap().execute("SET threads = ?", [threads]).unwrap();
        store.copy_published_stage_rows(&source(),&scope(),&["fixture-source-input-records"],&RunCancelFlagValue::default()).unwrap();
        let connection = store._connection.lock().unwrap();
        let mut statement = connection.prepare("SELECT generation_key,attempt_key,table_name,row_key,payload FROM \"frozen-generation-source-records\" ORDER BY generation_key,attempt_key,table_name,row_key").unwrap();
        statement.query_map([],|row| (0..5).map(|column|row.get::<_,String>(column)).collect::<Result<Vec<_>,_>>()).unwrap().collect::<Result<Vec<_>,_>>().unwrap()
    });
    assert_eq!(exports[0], exports[1]);
}

#[test]
fn test_turso_published_appender_handoff() {
    let directory = tempfile::tempdir().unwrap();
    let source =
        parseltongue_turso_store_adapter::TursoWorkspaceStoreAdapter::open_turso_workspace_store(
            ":memory:",
        )
        .unwrap();
    source.initialize_workspace_stage_schema().unwrap();
    let scope = scope();
    let cancel = RunCancelFlagValue::default();
    let records = [StageTableRowRecord {
        stage_owned_table_name: "span-reference-text-table".into(),
        stage_row_record_key: "source/λ".into(),
        stage_row_payload_text: "{\"source\":\"identity\"}".into(),
    }];
    for stage in ["crate-graph", "source-ingest", "syntax-spans"] {
        source
            .write_stage_table_unit(
                &StageUnitWriteRequest {
                    stage_run_scope_value: &scope,
                    stage_owner_name_value: stage,
                    stage_unit_work_key: "__complete__",
                    stage_input_stage_names: &[],
                    stage_unit_output_rows: if stage == "syntax-spans" {
                        &records
                    } else {
                        &[]
                    },
                },
                &cancel,
            )
            .unwrap();
    }
    let published = source
        .publish_workspace_generation_manifest(&scope, &scope.run_generation_digest_value, &cancel)
        .unwrap();
    let store = DuckdbAnalysisStoreAdapter::open_bounded_analysis_store(
        &directory.path().join("analysis.db"),
        directory.path(),
        1_288_490_188,
    )
    .unwrap();
    assert_eq!(
        store
            .copy_published_stage_rows(&source, &published, &["span-reference-text-table"], &cancel)
            .unwrap(),
        1
    );
    assert_eq!(
        store
            .count_analysis_stage_rows(&published, "span-reference-text-table")
            .unwrap(),
        source
            .count_stage_table_rows(&published, "span-reference-text-table")
            .unwrap()
    );
    let connection = store._connection.lock().unwrap();
    let payload: String = connection.query_row(
        "SELECT payload FROM \"frozen-generation-source-records\" ORDER BY generation_key,attempt_key,table_name,row_key",[],|row|row.get(0),
    ).unwrap();
    assert_eq!(payload, records[0].stage_row_payload_text);
}

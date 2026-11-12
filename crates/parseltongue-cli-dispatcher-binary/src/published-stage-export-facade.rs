use super::build_pipeline_stage_error;
use parseltongue_core_row_types::{StageRunScopeValue, StoreBackendFailureError};
use parseltongue_table_store_traits::StageTableStorePort;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    path::Path,
};

fn read_published_export_scope(
    store: &dyn StageTableStorePort,
) -> Result<StageRunScopeValue, StoreBackendFailureError> {
    store.read_workspace_publication_scope()?.ok_or_else(|| {
        build_pipeline_stage_error("StagePublicationMissingError: no published generation")
    })
}

fn build_scope_identity_record(scope: &StageRunScopeValue) -> Value {
    json!({
        "run_generation_digest_value":scope.run_generation_digest_value,
        "stage_attempt_sequence_value":scope.stage_attempt_sequence_value,
    })
}

fn read_export_table_values(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    table: &str,
) -> Result<Vec<Value>, StoreBackendFailureError> {
    store
        .read_stage_table_rows(scope, table)?
        .into_iter()
        .map(|row| {
            serde_json::from_str(&row.stage_row_payload_text).map_err(build_pipeline_stage_error)
        })
        .collect()
}

fn read_export_string_value<'a>(
    record: &'a Value,
    key: &str,
) -> Result<&'a str, StoreBackendFailureError> {
    record[key].as_str().ok_or_else(|| {
        build_pipeline_stage_error(format!("ManifestInputInvalidError: missing {key}"))
    })
}

fn read_source_digest_values(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
) -> Result<BTreeMap<String, Value>, StoreBackendFailureError> {
    let mut files = BTreeMap::new();
    store.visit_stage_table_rows(scope, "file-source-text-table", &mut |row| {
        let file: Value = serde_json::from_str(&row.stage_row_payload_text)
            .map_err(build_pipeline_stage_error)?;
        files.insert(
            read_export_string_value(&file, "file_source_absolute_path")?.to_owned(),
            file["file_content_digest_full"].clone(),
        );
        Ok(())
    })?;
    Ok(files)
}

pub(crate) fn write_published_stage_rows(
    store: &dyn StageTableStorePort,
    table: &str,
    filter: Option<&str>,
    output: &mut dyn Write,
) -> Result<usize, StoreBackendFailureError> {
    let scope = read_published_export_scope(store)?;
    let header = json!({
        "export_record_kind_value":"header",
        "export_scope_identity_record":build_scope_identity_record(&scope),
        "export_owned_table_name":table,
    });
    writeln!(output, "{header}").map_err(build_pipeline_stage_error)?;
    let mut count = 0;
    store.visit_stage_table_rows(&scope, table, &mut |row| {
        let payload: Value = serde_json::from_str(&row.stage_row_payload_text)
            .map_err(build_pipeline_stage_error)?;
        if filter.is_some_and(|filter| !row.stage_row_payload_text.contains(filter)) {
            return Ok(());
        }
        let value = json!({
            "export_record_kind_value":"row",
            "export_row_identity_value":row.stage_row_record_key,
            "export_row_payload_value":payload,
        });
        writeln!(output, "{value}").map_err(build_pipeline_stage_error)?;
        count += 1;
        Ok(())
    })?;
    Ok(count)
}

pub(crate) fn read_published_unit_manifest(
    store: &dyn StageTableStorePort,
) -> Result<Value, StoreBackendFailureError> {
    let scope = read_published_export_scope(store)?;
    let configurations = read_export_table_values(store, &scope, "workspace-config-state-records")?;
    let [configuration] = configurations.as_slice() else {
        return Err(build_pipeline_stage_error(
            "ManifestInputInvalidError: expected one workspace configuration",
        ));
    };
    let compiler = read_export_string_value(configuration, "workspace_compiler_version_text")?;
    if !configuration["workspace_parser_source_record"].is_object() {
        return Err(build_pipeline_stage_error(
            "ManifestInputInvalidError: snapshot parser identity missing",
        ));
    }
    let files = read_source_digest_values(store, &scope)?;
    let requests = read_export_table_values(store, &scope, "syntax-file-work-requests")?
        .into_iter()
        .try_fold(
            BTreeMap::<String, Vec<Value>>::new(),
            |mut requests, request| {
                let unit =
                    read_export_string_value(&request, "unit_compilation_identity_key")?.to_owned();
                let path = read_export_string_value(&request, "file_source_absolute_path")?;
                let file = files.get(path).ok_or_else(|| {
                    build_pipeline_stage_error(format!("ManifestSourceMissingError: {path}"))
                })?;
                let digest = file.as_str().ok_or_else(|| {
                    build_pipeline_stage_error(
                        "ManifestInputInvalidError: missing file_content_digest_full",
                    )
                })?;
                let mut request = request;
                request["file_content_digest_full"] = json!(digest);
                requests.entry(unit).or_default().push(request);
                Ok::<_, StoreBackendFailureError>(requests)
            },
        )?;
    let mut requests = requests;
    let units = read_export_table_values(store, &scope, "crate-dependency-graph-table")?
        .into_iter()
        .map(|mut unit| {
            let identity = read_export_string_value(&unit, "unit_compilation_identity_key")?;
            let closure = requests.remove(identity).unwrap_or_default();
            let roots = closure
                .iter()
                .map(|request| read_export_string_value(request, "file_source_absolute_path"))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .filter_map(|path| Path::new(path).parent())
                .collect::<BTreeSet<_>>();
            let semantic = json!({
                "unit_cargo_package_identifier":unit["unit_cargo_package_identifier"],
                "unit_cargo_target_name":unit["unit_cargo_target_name"],
                "unit_cargo_target_kind":unit["unit_cargo_target_kind"],
                "unit_compilation_role_kind":unit["unit_compilation_role_kind"],
                "unit_configuration_digest_full":blake3::hash(json!([
                    unit["unit_target_cfg_values"],unit["unit_enabled_feature_values"],
                    unit["unit_rust_edition_value"],unit["unit_compilation_role_kind"],
                ]).to_string().as_bytes()).to_hex().to_string(),
            });
            unit["unit_semantic_identity_record"] = semantic;
            unit["unit_source_root_paths"] = json!(roots);
            unit["unit_source_closure_rows"] = json!(closure);
            Ok(unit)
        })
        .collect::<Result<Vec<_>, StoreBackendFailureError>>()?;
    if !requests.is_empty() {
        return Err(build_pipeline_stage_error(
            "ManifestInputInvalidError: source closure has unknown compilation units",
        ));
    }
    Ok(json!({
        "manifest_format_version_value":1,
        "manifest_target_context_kind":"compiler_default",
        "manifest_mapped_target_policy":{
            "policy_target_kind_values":["lib","bin"],
            "policy_test_targets_flag":false,
            "policy_dependency_kind_values":["normal"],
            "policy_excluded_target_kinds":["test","bench","example","custom-build","proc-macro"],
            "policy_support_mapping_flag":false,
        },
        "manifest_compiler_version_text":compiler,
        "manifest_parser_source_record":configuration["workspace_parser_source_record"],
        "manifest_scope_identity_record":build_scope_identity_record(&scope),
        "manifest_configuration_state_record":configuration,
        "manifest_unit_context_rows":units,
    }))
}

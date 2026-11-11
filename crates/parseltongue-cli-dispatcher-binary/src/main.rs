use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StageTableRowRecord, StageUnitWriteRequest,
    StoreBackendFailureError,
};
use parseltongue_crate_graph_loader::{
    OfflineWorkspaceCommandRunner, ingest_requested_source_file, ingest_workspace_source_files,
    load_workspace_crate_graph,
};
use parseltongue_stage_worker_scheduler::map_bounded_stage_workers;
use parseltongue_syntax_span_extractor::{SyntaxFileInputData, extract_file_syntax_rows};
use parseltongue_table_store_traits::StageTableStorePort;
use parseltongue_turso_store_adapter::TursoWorkspaceStoreAdapter;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

fn build_pipeline_stage_error(message: impl std::fmt::Display) -> StoreBackendFailureError {
    StoreBackendFailureError {
        error_context_detail_text: message.to_string(),
    }
}
fn read_required_json_string<'a>(
    value: &'a Value,
    key: &str,
) -> Result<&'a str, StoreBackendFailureError> {
    value[key]
        .as_str()
        .ok_or_else(|| build_pipeline_stage_error(format!("missing field {key}")))
}
fn read_stage_json_records(
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
fn write_pipeline_stage_unit(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    stage: &str,
    unit: &str,
    rows: &[StageTableRowRecord],
    cancel: &RunCancelFlagValue,
) -> Result<(), StoreBackendFailureError> {
    let inputs: &[&str] = match stage {
        "source-ingest" => &["crate-graph"],
        "syntax-spans" => &["source-ingest"],
        _ => &[],
    };
    store.write_stage_table_unit(
        &StageUnitWriteRequest {
            stage_run_scope_value: scope,
            stage_owner_name_value: stage,
            stage_unit_work_key: unit,
            stage_input_stage_names: inputs,
            stage_unit_output_rows: rows,
        },
        cancel,
    )?;
    Ok(())
}
fn build_syntax_file_request(
    unit: &str,
    path: &str,
    base: &str,
    context: &str,
    provenance: &[String],
) -> StageTableRowRecord {
    let value = json!({"unit_compilation_identity_key":unit,"file_source_absolute_path":path,"file_module_directory_path":base,
        "file_parse_context_kind":context,"file_expansion_provenance_chain":provenance});
    StageTableRowRecord {
        stage_owned_table_name: "syntax-file-work-requests".into(),
        stage_row_record_key: blake3::hash(value.to_string().as_bytes())
            .to_hex()
            .to_string(),
        stage_row_payload_text: value.to_string(),
    }
}
fn read_file_module_directory(path: &Path) -> Result<PathBuf, StoreBackendFailureError> {
    let parent = path
        .parent()
        .ok_or_else(|| build_pipeline_stage_error("source parent absent"))?;
    if path
        .file_name()
        .is_some_and(|name| matches!(name.to_str(), Some("lib.rs" | "main.rs" | "mod.rs")))
    {
        Ok(parent.to_owned())
    } else {
        Ok(parent.join(
            path.file_stem()
                .ok_or_else(|| build_pipeline_stage_error("module stem absent"))?,
        ))
    }
}
fn build_syntax_file_input(
    request: &Value,
    unit: &Value,
    bytes: Vec<u8>,
) -> Result<SyntaxFileInputData, StoreBackendFailureError> {
    Ok(SyntaxFileInputData {
        file_source_absolute_path: read_required_json_string(request, "file_source_absolute_path")?
            .into(),
        file_source_content_bytes: bytes,
        unit_compilation_identity_key: read_required_json_string(
            request,
            "unit_compilation_identity_key",
        )?
        .into(),
        unit_rust_edition_value: read_required_json_string(unit, "unit_rust_edition_value")?.into(),
        unit_target_cfg_values: serde_json::from_value(unit["unit_target_cfg_values"].clone())
            .map_err(build_pipeline_stage_error)?,
        file_module_directory_path: read_required_json_string(
            request,
            "file_module_directory_path",
        )?
        .into(),
        unit_metadata_environment_values: serde_json::from_value(
            unit["unit_metadata_environment_values"].clone(),
        )
        .map_err(build_pipeline_stage_error)?,
        file_parse_context_kind: read_required_json_string(request, "file_parse_context_kind")?
            .into(),
        file_expansion_provenance_chain: serde_json::from_value(
            request["file_expansion_provenance_chain"].clone(),
        )
        .map_err(build_pipeline_stage_error)?,
    })
}

fn run_persisted_syntax_stage(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    cancel: &RunCancelFlagValue,
    workers: usize,
) -> Result<(), StoreBackendFailureError> {
    let units: BTreeMap<String, Value> = store
        .read_stage_table_rows(scope, "crate-dependency-graph-table")?
        .into_iter()
        .map(|row| {
            Ok((
                row.stage_row_record_key,
                serde_json::from_str(&row.stage_row_payload_text)
                    .map_err(build_pipeline_stage_error)?,
            ))
        })
        .collect::<Result<_, StoreBackendFailureError>>()?;
    for (key, unit) in &units {
        if unit["unit_mapped_target_flag"] != true {
            continue;
        }
        let path = Path::new(read_required_json_string(unit, "unit_source_root_path")?)
            .canonicalize()
            .map_err(build_pipeline_stage_error)?;
        let parent = path
            .parent()
            .ok_or_else(|| build_pipeline_stage_error("root source parent absent"))?;
        let request = build_syntax_file_request(
            key,
            &path.to_string_lossy(),
            &parent.to_string_lossy(),
            "items",
            &[],
        );
        write_pipeline_stage_unit(
            store,
            scope,
            "syntax-spans",
            &format!("enqueue/{}", request.stage_row_record_key),
            &[request],
            cancel,
        )?;
    }
    loop {
        let requests = store.read_stage_table_rows(scope, "syntax-file-work-requests")?;
        let markers: BTreeSet<_> = store
            .read_stage_table_rows(scope, "stage-run-generation-records")?
            .into_iter()
            .map(|r| r.stage_row_record_key)
            .collect();
        let pending: BTreeMap<_, _> = requests
            .into_iter()
            .filter(|request| {
                !markers.contains(&format!(
                    "__marker__/syntax-spans/{}",
                    request.stage_row_record_key
                ))
            })
            .map(|r| (r.stage_row_record_key, r.stage_row_payload_text))
            .collect();
        let files: BTreeMap<_, _> =
            read_stage_json_records(store, scope, "file-source-text-table")?
                .into_iter()
                .map(|v| {
                    Ok((
                        read_required_json_string(&v, "file_source_absolute_path")?.to_owned(),
                        v,
                    ))
                })
                .collect::<Result<_, StoreBackendFailureError>>()?;
        let keys: Vec<_> = pending.keys().cloned().collect();
        for batch in keys.chunks(workers) {
            let inputs: BTreeMap<_, _> = batch
                .iter()
                .map(|key| {
                    let request: Value =
                        serde_json::from_str(&pending[key]).map_err(build_pipeline_stage_error)?;
                    let unit = &units
                        [read_required_json_string(&request, "unit_compilation_identity_key")?];
                    let path = read_required_json_string(&request, "file_source_absolute_path")?;
                    let file = files.get(path).ok_or_else(|| {
                        build_pipeline_stage_error(format!(
                            "StageInputMissingError: source file {path}"
                        ))
                    })?;
                    let bytes = store.read_file_content_bytes(read_required_json_string(
                        file,
                        "file_content_digest_full",
                    )?)?;
                    let input = build_syntax_file_input(&request, unit, bytes)?;
                    Ok((key.clone(), input))
                })
                .collect::<Result<_, StoreBackendFailureError>>()?;
            let output = map_bounded_stage_workers(batch, workers, cancel, |key| {
                extract_file_syntax_rows(&inputs[key])
                    .map(|result| (key.to_owned(), result.stage_unit_output_rows))
            })?;
            output.into_iter().try_for_each(|(key, rows)| {
                write_pipeline_stage_unit(store, scope, "syntax-spans", &key, &rows, cancel)
            })?;
        }
        let requests = read_stage_json_records(store, scope, "module-source-closure-requests")?;
        let known: BTreeSet<_> = store
            .read_stage_table_rows(scope, "syntax-file-work-requests")?
            .into_iter()
            .map(|r| r.stage_row_record_key)
            .collect();
        let mut added = 0;
        for request in requests {
            let candidates: Vec<String> =
                serde_json::from_value(request["source_request_path_candidates"].clone())
                    .map_err(build_pipeline_stage_error)?;
            let existing: Vec<_> = candidates
                .iter()
                .filter(|p| Path::new(p).is_file())
                .collect();
            if existing.len() != 1 {
                return Err(build_pipeline_stage_error(format!(
                    "WorkspaceNotCompilingError: source closure expected one file, found {}: {candidates:?}",
                    existing.len()
                )));
            }
            let path = Path::new(existing[0])
                .canonicalize()
                .map_err(build_pipeline_stage_error)?;
            let context = read_required_json_string(&request, "source_request_context_kind")?;
            let base = if context == "items" {
                read_file_module_directory(&path)?
            } else {
                path.parent()
                    .ok_or_else(|| build_pipeline_stage_error("include parent absent"))?
                    .to_owned()
            };
            let mut provenance: Vec<String> =
                serde_json::from_value(request["file_expansion_provenance_chain"].clone())
                    .map_err(build_pipeline_stage_error)?;
            provenance
                .push(read_required_json_string(&request, "source_request_origin_digest")?.into());
            let job = build_syntax_file_request(
                read_required_json_string(&request, "unit_compilation_identity_key")?,
                &path.to_string_lossy(),
                &base.to_string_lossy(),
                context,
                &provenance,
            );
            if known.contains(&job.stage_row_record_key) {
                continue;
            }
            if !files.contains_key(path.to_string_lossy().as_ref()) {
                ingest_requested_source_file(scope, store, cancel, &path)?;
            }
            write_pipeline_stage_unit(
                store,
                scope,
                "syntax-spans",
                &format!("enqueue/{}", job.stage_row_record_key),
                &[job],
                cancel,
            )?;
            added += 1;
        }
        if added == 0 {
            break;
        }
    }
    write_pipeline_stage_unit(store, scope, "syntax-spans", "__complete__", &[], cancel)
}
fn build_workspace_coverage_banner(
    store: &dyn StageTableStorePort,
) -> Result<Value, StoreBackendFailureError> {
    let scope = store
        .read_workspace_publication_scope()?
        .ok_or_else(|| build_pipeline_stage_error("no completed published generation"))?;
    let files = read_stage_json_records(store, &scope, "file-source-text-table")?;
    let lengths: BTreeMap<_, _> = files
        .iter()
        .map(|v| {
            Ok((
                read_required_json_string(v, "file_source_absolute_path")?.to_owned(),
                v["file_source_byte_count"]
                    .as_u64()
                    .ok_or_else(|| build_pipeline_stage_error("missing byte length"))?,
            ))
        })
        .collect::<Result<_, StoreBackendFailureError>>()?;
    let mut groups: BTreeMap<String, (String, Vec<(u64, u64)>)> = BTreeMap::new();
    for range in read_stage_json_records(store, &scope, "span-byte-ownership-ranges")? {
        let path = read_required_json_string(&range, "file_source_absolute_path")?;
        let key = json!([
            range["unit_compilation_identity_key"],
            path,
            range["span_expansion_provenance_chain"]
        ])
        .to_string();
        groups
            .entry(key)
            .or_insert_with(|| (path.into(), Vec::new()))
            .1
            .push((
                range["span_byte_start_offset"]
                    .as_u64()
                    .ok_or_else(|| build_pipeline_stage_error("missing range start"))?,
                range["span_byte_end_offset"]
                    .as_u64()
                    .ok_or_else(|| build_pipeline_stage_error("missing range end"))?,
            ));
    }
    let tiling = groups.values_mut().all(|(path, ranges)| {
        ranges.sort_unstable();
        let end = ranges.iter().try_fold(0, |end, (start, next)| {
            if *start == end && *next >= *start {
                Some(*next)
            } else {
                None
            }
        });
        end == lengths.get(path).copied()
    });
    let spans = read_stage_json_records(store, &scope, "span-reference-text-table")?;
    let inactive = spans
        .iter()
        .filter(|s| s["site_unresolved_reason_kind"] == "cfg_inactive")
        .count();
    let span_count = spans.len();
    drop(spans);
    let fields = store
        .read_stage_table_rows(&scope, "field-signature-detail-table")?
        .len();
    let sites = store.read_stage_table_rows(&scope, "body-site-reference-table")?;
    let build_script = sites
        .iter()
        .filter(|s| s.stage_row_payload_text.contains("\"build_script\""))
        .count();
    Ok(
        json!({"run_generation_digest_value":scope.run_generation_digest_value,
        "source_file_record_count":files.len(),"syntax_span_record_count":span_count,
        "signature_field_record_count":fields,"body_site_record_count":sites.len(),
        "source_tiling_identity_holds":tiling,"inactive_item_record_count":inactive,
        "build_script_unresolved_count":build_script,
        "stage_implementation_status_values":{"crate-graph":"complete","source-ingest":"complete","syntax-spans":"complete",
            "name-resolution":"not yet implemented","macro-expansion":"not yet implemented","proc-macro-shapes":"not yet implemented",
            "signature-lowering":"not yet implemented","call-resolution":"not yet implemented","visibility-hld":"not yet implemented"}}),
    )
}
fn run_workspace_mapping_pipeline(
    root: &Path,
    store: &dyn StageTableStorePort,
    cancel: &RunCancelFlagValue,
    workers: usize,
) -> Result<Value, StoreBackendFailureError> {
    let started = std::time::Instant::now();
    let graph = load_workspace_crate_graph(root, &OfflineWorkspaceCommandRunner)?;
    eprintln!(
        "stage=crate-graph elapsed_seconds={:.3}",
        started.elapsed().as_secs_f64()
    );
    let nonce = format!(
        "{}-{}-{:?}",
        std::process::id(),
        env!("CARGO_PKG_VERSION"),
        std::time::SystemTime::now()
    );
    let scope = StageRunScopeValue {
        run_generation_digest_value: "staging".into(),
        stage_attempt_sequence_value: blake3::hash(nonce.as_bytes()).to_hex().to_string(),
    };
    write_pipeline_stage_unit(store, &scope, "crate-graph", "workspace", &graph, cancel)?;
    write_pipeline_stage_unit(store, &scope, "crate-graph", "__complete__", &[], cancel)?;
    drop(graph);
    let started = std::time::Instant::now();
    ingest_workspace_source_files(&scope, store, cancel)?;
    eprintln!(
        "stage=source-ingest elapsed_seconds={:.3}",
        started.elapsed().as_secs_f64()
    );
    write_pipeline_stage_unit(store, &scope, "source-ingest", "__complete__", &[], cancel)?;
    let started = std::time::Instant::now();
    run_persisted_syntax_stage(store, &scope, cancel, workers)?;
    eprintln!(
        "stage=syntax-spans elapsed_seconds={:.3}",
        started.elapsed().as_secs_f64()
    );
    let mut digest = blake3::Hasher::new();
    digest.update(b"parseltongue/0.1.0/m1-schema-1/ra_ap_syntax-0.0.362");
    for table in [
        "workspace-config-state-records",
        "crate-dependency-graph-table",
        "file-source-text-table",
    ] {
        for row in store.read_stage_table_rows(&scope, table)? {
            digest.update(&(row.stage_row_payload_text.len() as u64).to_le_bytes());
            digest.update(row.stage_row_payload_text.as_bytes());
        }
    }
    let started = std::time::Instant::now();
    store.publish_workspace_generation_manifest(
        &scope,
        digest.finalize().to_hex().as_ref(),
        cancel,
    )?;
    eprintln!(
        "stage=generation-publication elapsed_seconds={:.3}",
        started.elapsed().as_secs_f64()
    );
    build_workspace_coverage_banner(store)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let command = args
        .first()
        .ok_or("expected map-workspace-full-run or status-coverage-banner-show")?;
    let mut flags = BTreeMap::new();
    let mut index = 1;
    while index < args.len() {
        if args[index] == "--json" {
            index += 1;
            continue;
        }
        let value = args.get(index + 1).ok_or("missing flag value")?;
        if !matches!(
            args[index].as_str(),
            "--workspace-root-path-value"
                | "--workspace-store-file-path"
                | "--worker-thread-count-value"
        ) {
            return Err(format!("unknown flag {}", args[index]).into());
        }
        flags.insert(args[index].clone(), value.clone());
        index += 2;
    }
    let path = flags
        .get("--workspace-store-file-path")
        .ok_or("missing --workspace-store-file-path")?;
    if command == "status-coverage-banner-show" && !Path::new(path).is_file() {
        return Err("workspace store does not exist".into());
    }
    let store = TursoWorkspaceStoreAdapter::open_turso_workspace_store(path)?;
    store.initialize_workspace_stage_schema()?;
    let result = match command.as_str() {
        "status-coverage-banner-show" => build_workspace_coverage_banner(&store)?,
        "map-workspace-full-run" => {
            let root = Path::new(
                flags
                    .get("--workspace-root-path-value")
                    .ok_or("missing --workspace-root-path-value")?,
            )
            .canonicalize()?;
            let workers: usize = flags
                .get("--worker-thread-count-value")
                .map(String::as_str)
                .unwrap_or("4")
                .parse()?;
            if workers == 0 {
                return Err("worker count must be positive".into());
            }
            let cancel = RunCancelFlagValue::default();
            let signal = cancel.clone();
            ctrlc::set_handler(move || signal.request_stage_work_cancellation())?;
            run_workspace_mapping_pipeline(&root, &store, &cancel, workers)?
        }
        _ => return Err(format!("unknown command {command}").into()),
    };
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StageTableRowRecord, StageUnitWriteRequest,
    StoreBackendFailureError,
};
use parseltongue_crate_graph_loader::{
    OfflineWorkspaceCommandRunner, extend_sysroot_crate_graph, ingest_requested_source_file,
    ingest_workspace_source_files, load_workspace_crate_graph,
};
use parseltongue_stage_worker_scheduler::map_bounded_stage_workers;
use parseltongue_syntax_span_extractor::{
    SyntaxFileInputData, check_sysroot_context_needed, extract_file_syntax_rows,
    read_parser_compiler_identity, read_unit_prelude_configuration, run_persisted_name_resolution,
};
use parseltongue_table_store_traits::StageTableStorePort;
use parseltongue_turso_store_adapter::TursoWorkspaceStoreAdapter;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

#[path = "oracle-path-comparison-facade.rs"]
mod oracle_path_comparison_facade;
#[path = "published-stage-export-facade.rs"]
mod published_stage_export_facade;

#[cfg(all(
    feature = "system-heap-allocation-profile",
    not(feature = "native-production-heap-allocator")
))]
#[global_allocator]
static NATIVE_HEAP_PROFILE_ALLOCATOR: dhat::Alloc = dhat::Alloc;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SyntaxStageOperationKind {
    Discovery,
    Extraction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MappingFinalStageKind {
    SyntaxSpans,
    NameResolution,
}
fn run_persisted_syntax_stage(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    cancel: &RunCancelFlagValue,
    workers: usize,
    mode: SyntaxStageOperationKind,
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
        if unit["unit_mapped_target_flag"] != true && unit["unit_sysroot_context_flag"] != true {
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
                mode == SyntaxStageOperationKind::Extraction
                    || !markers.contains(&format!(
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
        let timing = keys
            .chunks(workers)
            .try_fold((0.0, 0.0, 0.0, 0usize), |timing, batch| {
                let started = std::time::Instant::now();
                let inputs: BTreeMap<_, _> = batch
                    .iter()
                    .map(|key| {
                        let request: Value = serde_json::from_str(&pending[key])
                            .map_err(build_pipeline_stage_error)?;
                        let unit = &units
                            [read_required_json_string(&request, "unit_compilation_identity_key")?];
                        let path =
                            read_required_json_string(&request, "file_source_absolute_path")?;
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
                let load = started.elapsed().as_secs_f64();
                let started = std::time::Instant::now();
                let output = map_bounded_stage_workers(batch, workers, cancel, |key| {
                    extract_file_syntax_rows(&inputs[key]).map(|result| {
                        (
                            key.to_owned(),
                            result
                                .stage_unit_output_rows
                                .into_iter()
                                .filter(|row| {
                                    mode == SyntaxStageOperationKind::Extraction
                                        || row.stage_owned_table_name
                                            == "module-source-closure-requests"
                                })
                                .collect::<Vec<_>>(),
                        )
                    })
                })?;
                let parse = started.elapsed().as_secs_f64();
                let records = output.iter().map(|(_, rows)| rows.len()).sum::<usize>();
                let started = std::time::Instant::now();
                output.into_iter().try_for_each(|(key, rows)| {
                    write_pipeline_stage_unit(store, scope, "syntax-spans", &key, &rows, cancel)
                })?;
                Ok::<_, StoreBackendFailureError>((
                    timing.0 + load,
                    timing.1 + parse,
                    timing.2 + started.elapsed().as_secs_f64(),
                    timing.3 + records,
                ))
            })?;
        eprintln!(
            "stage=syntax-batches mode={mode:?} units={} records={} load_seconds={:.3} parse_seconds={:.3} write_seconds={:.3}",
            keys.len(),
            timing.3,
            timing.0,
            timing.1,
            timing.2
        );
        if mode == SyntaxStageOperationKind::Extraction {
            break;
        }
        let started = std::time::Instant::now();
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
        eprintln!(
            "stage=source-closure-enqueue units={added} elapsed_seconds={:.3}",
            started.elapsed().as_secs_f64()
        );
        if added == 0 {
            break;
        }
    }
    match mode {
        SyntaxStageOperationKind::Discovery => Ok(()),
        SyntaxStageOperationKind::Extraction => {
            write_pipeline_stage_unit(store, scope, "syntax-spans", "__complete__", &[], cancel)
        }
    }
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
    store.visit_stage_table_rows(&scope, "span-byte-ownership-ranges", &mut |row| {
        let range: Value = serde_json::from_str(&row.stage_row_payload_text)
            .map_err(build_pipeline_stage_error)?;
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
        Ok(())
    })?;
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
    let mut reasons: BTreeMap<String, (usize, usize)> = store
        .read_stage_table_rows(&scope, "crate-dependency-graph-table")?
        .into_iter()
        .map(|row| (row.stage_row_record_key, (0, 0)))
        .collect();
    let mut inactive = 0;
    let span_count =
        store.visit_stage_table_rows(&scope, "span-reference-text-table", &mut |row| {
            let value: Value = serde_json::from_str(&row.stage_row_payload_text)
                .map_err(build_pipeline_stage_error)?;
            if value["site_unresolved_reason_kind"] == "cfg_inactive" {
                inactive += 1;
                reasons
                    .entry(
                        read_required_json_string(&value, "unit_compilation_identity_key")?
                            .to_owned(),
                    )
                    .or_default()
                    .0 += 1;
            }
            Ok(())
        })?;
    let fields = store.count_stage_table_rows(&scope, "field-signature-detail-table")?;
    let mut build_script = 0;
    let sites = store.visit_stage_table_rows(&scope, "body-site-reference-table", &mut |row| {
        let value: Value = serde_json::from_str(&row.stage_row_payload_text)
            .map_err(build_pipeline_stage_error)?;
        if value["site_unresolved_reason_kind"] == "build_script" {
            build_script += 1;
            reasons
                .entry(
                    read_required_json_string(&value, "unit_compilation_identity_key")?.to_owned(),
                )
                .or_default()
                .1 += 1;
        }
        Ok(())
    })?;
    let reasons:Vec<_>=reasons.into_iter().map(|(unit,(inactive,build_script))|
        json!({"unit_compilation_identity_key":unit,"unit_unresolved_reason_counts":{"cfg_inactive":inactive,"build_script":build_script}})).collect();
    let name_resolution_complete = store
        .read_stage_table_rows(&scope, "stage-run-generation-records")?
        .iter()
        .any(|row| row.stage_row_record_key == "__marker__/name-resolution/__complete__");
    Ok(
        json!({"run_generation_digest_value":scope.run_generation_digest_value,
        "source_file_record_count":files.len(),"syntax_span_record_count":span_count,
        "signature_field_record_count":fields,"body_site_record_count":sites,
        "source_tiling_identity_holds":tiling,"inactive_item_record_count":inactive,
        "build_script_unresolved_count":build_script,
        "crate_unresolved_reason_counts":reasons,
        "stage_implementation_status_values":{"crate-graph":"complete","source-ingest":"complete","syntax-spans":"complete",
            "name-resolution":if name_resolution_complete {"complete"} else {"not yet implemented"},"macro-expansion":"not yet implemented","proc-macro-shapes":"not yet implemented",
            "signature-lowering":"not yet implemented","call-resolution":"not yet implemented","visibility-hld":"not yet implemented"}}),
    )
}
fn compute_workspace_generation_digest(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
) -> Result<String, StoreBackendFailureError> {
    let mut digest = blake3::Hasher::new();
    digest.update(b"parseltongue/0.1.0/m1-schema-1");
    digest.update(include_bytes!(
        "../../../docs/phase-20261111/parser-source-pin-manifest.json"
    ));
    [
        "workspace-config-state-records",
        "crate-dependency-graph-table",
        "file-source-text-table",
    ]
    .into_iter()
    .try_for_each(|table| {
        store
            .visit_stage_table_rows(scope, table, &mut |row| {
                digest.update(&(row.stage_row_payload_text.len() as u64).to_le_bytes());
                digest.update(row.stage_row_payload_text.as_bytes());
                Ok(())
            })
            .map(|_| ())
    })?;
    Ok(digest.finalize().to_hex().to_string())
}
fn prepare_sysroot_staging_scope(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    cancel: &RunCancelFlagValue,
    workers: usize,
) -> Result<StageRunScopeValue, StoreBackendFailureError> {
    let files = read_stage_json_records(store, scope, "file-source-text-table")?
        .into_iter()
        .map(|value| {
            Ok((
                read_required_json_string(&value, "file_source_absolute_path")?.to_owned(),
                value,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, StoreBackendFailureError>>()?;
    let requests = read_stage_json_records(store, scope, "syntax-file-work-requests")?;
    let needed = requests.iter().try_fold(false, |needed, request| {
        if needed {
            return Ok(true);
        }
        let path = read_required_json_string(request, "file_source_absolute_path")?;
        let file = files
            .get(path)
            .ok_or_else(|| build_pipeline_stage_error("reached source missing"))?;
        let bytes = store.read_file_content_bytes(read_required_json_string(
            file,
            "file_content_digest_full",
        )?)?;
        Ok::<_, StoreBackendFailureError>(check_sysroot_context_needed(
            std::str::from_utf8(&bytes).map_err(build_pipeline_stage_error)?,
        ))
    })?;
    let graph = store.read_stage_table_rows(scope, "crate-dependency-graph-table")?;
    let contexts=graph.iter().filter_map(|row|{
        let unit=serde_json::from_str::<Value>(&row.stage_row_payload_text);
        match unit{
            Ok(unit) if unit["unit_mapped_target_flag"]==true=>Some(Ok((row,unit))),
            Ok(_)=>None,Err(error)=>Some(Err(build_pipeline_stage_error(error)))
        }
    }).map(|entry|{
        let (row,unit)=entry?;
        let path=read_required_json_string(&unit,"unit_source_root_path")?;
        let file=files.get(path).ok_or_else(||build_pipeline_stage_error(format!("crate root missing: {path}")))?;
        let bytes=store.read_file_content_bytes(read_required_json_string(file,"file_content_digest_full")?)?;
        let request=serde_json::json!({"file_source_absolute_path":path,"unit_compilation_identity_key":row.stage_row_record_key,
            "file_module_directory_path":read_file_module_directory(Path::new(path))?,"file_parse_context_kind":"items","file_expansion_provenance_chain":[]});
        let flags=read_unit_prelude_configuration(&build_syntax_file_input(&request,&unit,bytes)?)?;
        Ok((row.stage_row_record_key.clone(),flags))
    }).collect::<Result<BTreeMap<_,_>,StoreBackendFailureError>>()?;
    if !needed {
        let graph = graph
            .into_iter()
            .map(|mut row| {
                if let Some(flags) = contexts.get(&row.stage_row_record_key) {
                    let mut value: Value = serde_json::from_str(&row.stage_row_payload_text)
                        .map_err(build_pipeline_stage_error)?;
                    value["unit_no_std_flag"] = json!(flags.unit_no_std_flag);
                    value["unit_no_core_flag"] = json!(flags.unit_no_core_flag);
                    value["unit_implicit_prelude_flag"] = json!(flags.unit_implicit_prelude_flag);
                    row.stage_row_payload_text = value.to_string();
                }
                Ok(row)
            })
            .collect::<Result<Vec<_>, StoreBackendFailureError>>()?;
        let graph = graph
            .into_iter()
            .chain(store.read_stage_table_rows(scope, "workspace-config-state-records")?)
            .collect::<Vec<_>>();
        write_pipeline_stage_unit(store, scope, "crate-graph", "workspace", &graph, cancel)?;
        return Ok(scope.clone());
    }
    let graph = graph
        .into_iter()
        .chain(store.read_stage_table_rows(scope, "workspace-config-state-records")?)
        .collect::<Vec<_>>();
    let graph = extend_sysroot_crate_graph(&graph, &contexts)?;
    let next = StageRunScopeValue {
        run_generation_digest_value: format!(
            "staging-sysroot/{}",
            scope.stage_attempt_sequence_value
        ),
        stage_attempt_sequence_value: scope.stage_attempt_sequence_value.clone(),
    };
    write_pipeline_stage_unit(store, &next, "crate-graph", "workspace", &graph, cancel)?;
    write_pipeline_stage_unit(store, &next, "crate-graph", "__complete__", &[], cancel)?;
    ingest_workspace_source_files(&next, store, cancel)?;
    write_pipeline_stage_unit(store, &next, "source-ingest", "__complete__", &[], cancel)?;
    run_persisted_syntax_stage(
        store,
        &next,
        cancel,
        workers,
        SyntaxStageOperationKind::Discovery,
    )?;
    Ok(next)
}
fn run_workspace_mapping_pipeline(
    root: &Path,
    store: &dyn StageTableStorePort,
    cancel: &RunCancelFlagValue,
    workers: usize,
    final_stage: MappingFinalStageKind,
) -> Result<Value, StoreBackendFailureError> {
    let started = std::time::Instant::now();
    let graph = load_workspace_crate_graph(root, &OfflineWorkspaceCommandRunner)?
        .into_iter()
        .map(|mut row| {
            if row.stage_owned_table_name == "workspace-config-state-records" {
                let mut value: Value = serde_json::from_str(&row.stage_row_payload_text)
                    .map_err(build_pipeline_stage_error)?;
                value["workspace_compiler_version_text"] = json!(read_parser_compiler_identity());
                value["workspace_parser_source_record"] = serde_json::from_str(include_str!(
                    "../../../docs/phase-20261111/parser-source-pin-manifest.json"
                ))
                .map_err(build_pipeline_stage_error)?;
                value["workspace_required_stage_names"] =
                    if final_stage == MappingFinalStageKind::NameResolution {
                        json!([
                            "crate-graph",
                            "source-ingest",
                            "syntax-spans",
                            "name-resolution"
                        ])
                    } else {
                        json!(["crate-graph", "source-ingest", "syntax-spans"])
                    };
                row.stage_row_payload_text = value.to_string();
            }
            Ok(row)
        })
        .collect::<Result<Vec<_>, StoreBackendFailureError>>()?;
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
    run_persisted_syntax_stage(
        store,
        &scope,
        cancel,
        workers,
        SyntaxStageOperationKind::Discovery,
    )?;
    eprintln!(
        "stage=source-closure elapsed_seconds={:.3}",
        started.elapsed().as_secs_f64()
    );
    let scope = if final_stage == MappingFinalStageKind::NameResolution {
        prepare_sysroot_staging_scope(store, &scope, cancel, workers)?
    } else {
        scope
    };
    let generation = compute_workspace_generation_digest(store, &scope)?;
    let started = std::time::Instant::now();
    let scope = store.prepare_workspace_generation_scope(&scope, &generation, cancel)?;
    eprintln!(
        "stage=generation-preparation elapsed_seconds={:.3}",
        started.elapsed().as_secs_f64()
    );
    let started = std::time::Instant::now();
    run_persisted_syntax_stage(
        store,
        &scope,
        cancel,
        workers,
        SyntaxStageOperationKind::Extraction,
    )?;
    eprintln!(
        "stage=syntax-spans elapsed_seconds={:.3}",
        started.elapsed().as_secs_f64()
    );
    let started = std::time::Instant::now();
    if final_stage == MappingFinalStageKind::NameResolution {
        run_persisted_name_resolution(&scope, store, cancel)?;
    }
    eprintln!(
        "stage=name-resolution elapsed_seconds={:.3}",
        started.elapsed().as_secs_f64()
    );
    let started = std::time::Instant::now();
    store.publish_workspace_generation_manifest(
        &scope,
        &scope.run_generation_digest_value,
        cancel,
    )?;
    eprintln!(
        "stage=generation-publication elapsed_seconds={:.3}",
        started.elapsed().as_secs_f64()
    );
    build_workspace_coverage_banner(store)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(
        feature = "system-heap-allocation-profile",
        not(feature = "native-production-heap-allocator")
    ))]
    let _heap_profile = dhat::Profiler::builder()
        .file_name(
            std::env::var_os("PARSELTONGUE_HEAP_PROFILE_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("heap-allocation-profile-records.json")),
        )
        .trim_backtraces(Some(16))
        .build();
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
                | "--mapping-final-stage-kind"
                | "--stage-owned-table-name"
                | "--stage-row-filter-value"
                | "--primary-semantic-input-path"
                | "--secondary-semantic-input-path"
                | "--agreement-store-file-path"
        ) {
            return Err(format!("unknown flag {}", args[index]).into());
        }
        flags.insert(args[index].clone(), value.clone());
        index += 2;
    }
    let path = flags
        .get("--workspace-store-file-path")
        .ok_or("missing --workspace-store-file-path")?;
    let exporting = matches!(
        command.as_str(),
        "export-published-stage-rows" | "export-mapped-unit-manifest" | "oracle-edge-differ-run"
    );
    if (command == "status-coverage-banner-show" || exporting) && !Path::new(path).is_file() {
        return Err("workspace store does not exist".into());
    }
    let store = TursoWorkspaceStoreAdapter::open_turso_workspace_store(path)?;
    if !exporting {
        store.initialize_workspace_stage_schema()?;
    }
    let result = match command.as_str() {
        "oracle-edge-differ-run" => {
            let output = flags
                .get("--agreement-store-file-path")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    Path::new(path).with_file_name("oracle-edge-agreement-store.db")
                });
            let parent = output.parent().unwrap_or_else(|| Path::new("."));
            let normalized = parent.canonicalize()?.join(
                output
                    .file_name()
                    .ok_or("agreement store filename missing")?,
            );
            if normalized == Path::new(path).canonicalize()? {
                return Err("comparison cannot mutate source store".into());
            }
            let agreement = TursoWorkspaceStoreAdapter::open_turso_workspace_store(
                output.to_str().ok_or("agreement store path is not UTF-8")?,
            )?;
            agreement.initialize_workspace_stage_schema()?;
            oracle_path_comparison_facade::run_oracle_agreement_comparison(
                &store,
                Path::new(
                    flags
                        .get("--primary-semantic-input-path")
                        .ok_or("missing PRIMARY input")?,
                ),
                Path::new(
                    flags
                        .get("--secondary-semantic-input-path")
                        .ok_or("missing SECONDARY input")?,
                ),
                &agreement,
                &RunCancelFlagValue::default(),
            )?
        }
        "status-coverage-banner-show" => build_workspace_coverage_banner(&store)?,
        "export-mapped-unit-manifest" => {
            published_stage_export_facade::read_published_unit_manifest(&store)?
        }
        "export-published-stage-rows" => {
            let table = flags
                .get("--stage-owned-table-name")
                .ok_or("missing --stage-owned-table-name")?;
            let stdout = std::io::stdout();
            let mut output = BufWriter::new(stdout.lock());
            published_stage_export_facade::write_published_stage_rows(
                &store,
                table,
                flags.get("--stage-row-filter-value").map(String::as_str),
                &mut output,
            )?;
            output.flush()?;
            return Ok(());
        }
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
            let final_stage = match flags.get("--mapping-final-stage-kind").map(String::as_str) {
                None | Some("name-resolution") => MappingFinalStageKind::NameResolution,
                Some("syntax-spans") => MappingFinalStageKind::SyntaxSpans,
                Some(_) => return Err("unknown mapping final stage".into()),
            };
            run_workspace_mapping_pipeline(&root, &store, &cancel, workers, final_stage)?
        }
        _ => return Err(format!("unknown command {command}").into()),
    };
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

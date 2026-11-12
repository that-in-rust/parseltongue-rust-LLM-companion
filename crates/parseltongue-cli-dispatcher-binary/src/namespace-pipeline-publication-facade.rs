use super::{build_pipeline_stage_error, read_required_json_string, write_pipeline_stage_unit};
use parseltongue_core_row_types::{
    MAPPED_TARGET_FLAG_FIELD, RunCancelFlagValue, StageRunScopeValue, StageTableRowRecord,
    StageUnitWriteRequest, StoreBackendFailureError,
};
use parseltongue_duckdb_analysis_adapter::{
    DuckdbAnalysisStoreAdapter, NAMESPACE_SOURCE_TABLE_NAMES, NamespaceAnalysisPhaseError,
    NamespaceAnalysisPhaseKind, NamespaceResolutionRoundReport,
};
use parseltongue_syntax_span_extractor::prepare_macro_expansion_port;
use parseltongue_table_store_traits::{
    AnalysisTableStorePort, ItemMacroExpansionPort, StageTableStorePort,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

const NAMESPACE_OUTPUT_BATCH_BYTES: usize = 8 * 1024 * 1024;
const NAMESPACE_RECEIPT_TABLE_NAME: &str = "analysis-resolution-receipt-records";
const NAMESPACE_PRODUCT_TABLE_NAMES: [&str; 3] = [
    "name-resolution-scope-records",
    "module-scope-binding-table",
    "path-resolution-site-records",
];

fn read_stage_table_values(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    table: &str,
) -> Result<Vec<Value>, StoreBackendFailureError> {
    let mut values = Vec::new();
    store.visit_stage_table_rows(scope, table, &mut |row| {
        values.push(
            serde_json::from_str(&row.stage_row_payload_text)
                .map_err(build_pipeline_stage_error)?,
        );
        Ok(())
    })?;
    Ok(values)
}

fn order_units_dependency_first(units: Vec<Value>) -> Result<Vec<Value>, StoreBackendFailureError> {
    let keys = units
        .iter()
        .map(|unit| {
            read_required_json_string(unit, "unit_compilation_identity_key").map(str::to_owned)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let dependencies = units
        .iter()
        .map(|unit| {
            let package = &unit["unit_cargo_package_identifier"];
            let role = &unit["unit_compilation_role_kind"];
            let deps = unit["unit_normal_dependency_rows"]
                .as_array()
                .ok_or_else(|| build_pipeline_stage_error("unit dependency rows absent"))?;
            Ok(units
                .iter()
                .zip(&keys)
                .filter(|(candidate, _)| {
                    candidate["unit_cargo_target_kind"] == "lib"
                        && candidate["unit_compilation_identity_key"]
                            != unit["unit_compilation_identity_key"]
                        && ((unit["unit_cargo_target_kind"] == "bin"
                            && &candidate["unit_cargo_package_identifier"] == package
                            && &candidate["unit_compilation_role_kind"] == role)
                            || deps.iter().any(|dep| {
                                candidate["unit_cargo_package_identifier"]
                                    == dep["dependency_cargo_package_identifier"]
                                    && candidate["unit_compilation_role_kind"]
                                        == dep["dependency_compilation_role_kind"]
                            }))
                })
                .map(|(_, key)| key.clone())
                .collect::<BTreeSet<_>>())
        })
        .collect::<Result<Vec<_>, StoreBackendFailureError>>()?;
    let pending = keys
        .into_iter()
        .zip(units.into_iter().zip(dependencies))
        .collect::<BTreeMap<_, _>>();
    let count = pending.len();
    (0..count)
        .try_fold(
            (pending, BTreeSet::new(), Vec::new()),
            |(mut pending, mut done, mut ordered), _| {
                let key = pending
                    .iter()
                    .find(|(_, (_, deps))| deps.is_subset(&done))
                    .map(|(key, _)| key.clone())
                    .ok_or_else(|| {
                        build_pipeline_stage_error("compilation unit dependency cycle")
                    })?;
                let (unit, _) = pending.remove(&key).ok_or_else(|| {
                    build_pipeline_stage_error("dependency order lost a compilation unit")
                })?;
                done.insert(key);
                ordered.push(unit);
                Ok((pending, done, ordered))
            },
        )
        .map(|(_, _, ordered)| ordered)
}

struct PendingNamespaceOutputPublication<'a> {
    store: &'a dyn StageTableStorePort,
    scope: &'a StageRunScopeValue,
    cancel: &'a RunCancelFlagValue,
    base: String,
    records: Vec<StageTableRowRecord>,
    bytes: usize,
    chunk: usize,
}

impl<'a> PendingNamespaceOutputPublication<'a> {
    fn new_pending_output_publication(
        store: &'a dyn StageTableStorePort,
        scope: &'a StageRunScopeValue,
        cancel: &'a RunCancelFlagValue,
        base: String,
    ) -> Self {
        Self {
            store,
            scope,
            cancel,
            base,
            records: Vec::new(),
            bytes: 0,
            chunk: 0,
        }
    }
    fn write_pending_output_rows(
        &self,
        key: &str,
        rows: &[StageTableRowRecord],
    ) -> Result<(), StoreBackendFailureError> {
        self.store
            .write_stage_table_unit(
                &StageUnitWriteRequest {
                    stage_run_scope_value: self.scope,
                    stage_owner_name_value: "name-resolution",
                    stage_unit_work_key: key,
                    stage_input_stage_names: &["syntax-namespace-input-records"],
                    stage_unit_output_rows: rows,
                },
                self.cancel,
            )
            .map(|_| ())
    }
    fn flush_pending_output_rows(&mut self) -> Result<(), StoreBackendFailureError> {
        if !self.records.is_empty() {
            self.write_pending_output_rows(
                &format!("{}/pending/{:08}", self.base, self.chunk),
                &self.records,
            )?;
            self.records.clear();
            self.bytes = 0;
            self.chunk += 1;
        }
        Ok(())
    }
    fn push_pending_output_row(
        &mut self,
        row: StageTableRowRecord,
    ) -> Result<(), StoreBackendFailureError> {
        if self.cancel.check_stage_work_cancelled() {
            return Err(build_pipeline_stage_error(
                "namespace publication cancelled",
            ));
        }
        if self.bytes + row.stage_row_payload_text.len() > NAMESPACE_OUTPUT_BATCH_BYTES {
            self.flush_pending_output_rows()?;
        }
        self.bytes += row.stage_row_payload_text.len();
        self.records.push(row);
        Ok(())
    }
    fn finish_pending_output_publication(mut self) -> Result<(), StoreBackendFailureError> {
        self.flush_pending_output_rows()?;
        self.write_pending_output_rows(&self.base, &[])
    }
}

fn publish_namespace_output_rows(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    cancel: &RunCancelFlagValue,
    key: String,
    rows: Vec<StageTableRowRecord>,
) -> Result<(), StoreBackendFailureError> {
    let mut publication = PendingNamespaceOutputPublication::new_pending_output_publication(
        store, scope, cancel, key,
    );
    rows.into_iter()
        .try_for_each(|row| publication.push_pending_output_row(row))?;
    publication.finish_pending_output_publication()
}

fn hash_product_row_leaf(row: &StageTableRowRecord) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    [
        &row.stage_owned_table_name,
        &row.stage_row_record_key,
        &row.stage_row_payload_text,
    ]
    .into_iter()
    .for_each(|value| {
        hasher.update(&(value.len() as u64).to_le_bytes());
        hasher.update(value.as_bytes());
    });
    *hasher.finalize().as_bytes()
}

fn refresh_frozen_namespace_inputs(
    analysis: &DuckdbAnalysisStoreAdapter,
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    cancel: &RunCancelFlagValue,
) -> Result<(StageRunScopeValue, NamespaceResolutionRoundReport), NamespaceAnalysisPhaseError> {
    let frozen = store.publish_analysis_input_snapshot(scope, cancel)?;
    analysis.discard_unpublished_analysis_inputs()?;
    let started = std::time::Instant::now();
    eprintln!(
        "stage=namespace-frozen-input snapshot={}",
        json!({"generation_key":frozen.run_generation_digest_value,"attempt_key":frozen.stage_attempt_sequence_value})
    );
    let copied = analysis.measure_namespace_phase_work(NamespaceAnalysisPhaseKind::Copy, || {
        analysis.copy_published_stage_rows(store, &frozen, NAMESPACE_SOURCE_TABLE_NAMES, cancel)
    })?;
    analysis.measure_namespace_phase_work(NamespaceAnalysisPhaseKind::Normalization, || {
        analysis.normalize_frozen_namespace_rows(&frozen, cancel)
    })?;
    let report = analysis
        .measure_namespace_phase_work(NamespaceAnalysisPhaseKind::Resolution, || {
            analysis.resolve_frozen_namespace_rows(&frozen, cancel)
        })?;
    eprintln!(
        "stage=namespace-sql-fixed-point source_rows={copied} rounds={} elapsed_seconds={:.3}",
        report.fixed_point_round_count,
        started.elapsed().as_secs_f64()
    );
    Ok((frozen, report))
}

pub(crate) fn run_namespace_analysis_pipeline(
    scope: &StageRunScopeValue,
    store: &dyn StageTableStorePort,
    cancel: &RunCancelFlagValue,
    directory: &Path,
) -> Result<(), NamespaceAnalysisPhaseError> {
    let started = std::time::Instant::now();
    let directory = directory
        .join(&scope.run_generation_digest_value)
        .join(&scope.stage_attempt_sequence_value);
    std::fs::create_dir_all(&directory).map_err(build_pipeline_stage_error)?;
    let database = directory.join("namespace-resolution-analysis-data.duckdb");
    let analysis = DuckdbAnalysisStoreAdapter::open_measured_analysis_store(&database, &directory)?;
    let configurations = read_stage_table_values(store, scope, "workspace-config-state-records")?;
    let [workspace] = configurations.as_slice() else {
        return Err(build_pipeline_stage_error("expected one workspace configuration").into());
    };
    let units = order_units_dependency_first(
        read_stage_table_values(store, scope, "crate-dependency-graph-table")?
            .into_iter()
            .filter(|unit| {
                unit[MAPPED_TARGET_FLAG_FIELD] == true || unit["unit_sysroot_context_flag"] == true
            })
            .collect(),
    )?;
    let (mut frozen, mut report) =
        refresh_frozen_namespace_inputs(&analysis, store, scope, cancel)?;
    let lookup = analysis.frozen_namespace_macro_lookup(frozen.clone());
    let mut rounds = report.fixed_point_round_count;
    let mut expansions = 0;
    units.into_iter().try_for_each(|unit| {
        let unit_key =
            read_required_json_string(&unit, "unit_compilation_identity_key")?.to_owned();
        let port = prepare_macro_expansion_port(unit, workspace, store, scope, cancel, &lookup)
            .map_err(build_pipeline_stage_error)?;
        while let Some(round) = port
            .expand_next_resolved_macro_items()
            .map_err(|error| build_pipeline_stage_error(format!("{error:?}")))?
        {
            expansions += 1;
            publish_namespace_output_rows(
                store,
                scope,
                cancel,
                format!("crate/macros/{expansions:08}"),
                round.expansion_semantic_output_rows,
            )?;
            (frozen, report) = refresh_frozen_namespace_inputs(&analysis, store, scope, cancel)?;
            rounds += report.fixed_point_round_count;
            lookup.replace_frozen_lookup_scope(frozen.clone())?;
        }
        let unresolved = port
            .finish_unresolved_macro_items()
            .map_err(|error| build_pipeline_stage_error(format!("{error:?}")))?;
        publish_namespace_output_rows(
            store,
            scope,
            cancel,
            format!(
                "crate/unresolved/{}",
                blake3::hash(unit_key.as_bytes()).to_hex()
            ),
            unresolved.expansion_semantic_output_rows,
        )?;
        Ok::<_, NamespaceAnalysisPhaseError>(())
    })?;
    let mut expected = NAMESPACE_PRODUCT_TABLE_NAMES
        .into_iter()
        .map(|table| (table.to_owned(), Vec::new()))
        .collect::<BTreeMap<String, Vec<(String, [u8; 32])>>>();
    let mut publication = PendingNamespaceOutputPublication::new_pending_output_publication(
        store,
        scope,
        cancel,
        "crate/products".into(),
    );
    let products = analysis.visit_frozen_namespace_products(&frozen, &mut |row| {
        expected
            .entry(row.stage_owned_table_name.clone())
            .or_default()
            .push((row.stage_row_record_key.clone(), hash_product_row_leaf(row)));
        publication.push_pending_output_row(row.clone())
    })?;
    publication.finish_pending_output_publication()?;
    let product_snapshot = store.publish_analysis_input_snapshot(scope, cancel)?;
    let proofs = expected
        .into_iter()
        .map(|(table, mut rows)| {
            rows.sort_unstable_by(|left, right| left.0.cmp(&right.0));
            if rows.windows(2).any(|pair| pair[0].0 == pair[1].0) {
                return Err(build_pipeline_stage_error(
                    "duplicate final namespace product key",
                ));
            }
            let mut expected = blake3::Hasher::new();
            rows.iter().for_each(|(_, leaf)| {
                expected.update(leaf);
            });
            let expected_digest = expected.finalize();
            let expected_digest_text = expected_digest.to_hex().to_string();
            let mut actual_rows = Vec::new();
            store.visit_stage_table_rows(&product_snapshot, &table, &mut |row| {
                actual_rows.push(row);
                Ok(())
            })?;
            actual_rows.sort_unstable_by(|left, right| {
                left.stage_row_record_key.cmp(&right.stage_row_record_key)
            });
            if actual_rows
                .windows(2)
                .any(|pair| pair[0].stage_row_record_key == pair[1].stage_row_record_key)
            {
                return Err(build_pipeline_stage_error(
                    "duplicate published namespace product key",
                ));
            }
            let count = actual_rows.len();
            let mut actual = blake3::Hasher::new();
            actual_rows.iter().for_each(|row| {
                actual.update(&hash_product_row_leaf(row));
            });
            if count != rows.len() || actual.finalize() != expected_digest {
                return Err(build_pipeline_stage_error(format!(
                    "namespace product digest mismatch: {table}"
                )));
            }
            Ok((
                table,
                json!({"row_count":count,"content_digest":expected_digest_text}),
            ))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let receipt = json!({"analysis_database_absolute_path":database.canonicalize().map_err(build_pipeline_stage_error)?,"analysis_generation_key":frozen.run_generation_digest_value,"analysis_attempt_key":frozen.stage_attempt_sequence_value,"analysis_binding_set_digest":report.binding_set_digest_value,"analysis_fixed_point_round_count":rounds,"analysis_macro_expansion_count":expansions,"analysis_product_record_count":products,"analysis_product_content_proofs":proofs,"analysis_product_digest_kind":"blake3-ordered-row-leaves","analysis_elapsed_seconds":started.elapsed().as_secs_f64()});
    publish_namespace_output_rows(
        store,
        scope,
        cancel,
        "crate/receipt".into(),
        vec![StageTableRowRecord {
            stage_owned_table_name: NAMESPACE_RECEIPT_TABLE_NAME.into(),
            stage_row_record_key: "namespace-analysis-receipt-record".into(),
            stage_row_payload_text: receipt.to_string(),
        }],
    )?;
    write_pipeline_stage_unit(store, scope, "name-resolution", "__complete__", &[], cancel)
        .map_err(Into::into)
}

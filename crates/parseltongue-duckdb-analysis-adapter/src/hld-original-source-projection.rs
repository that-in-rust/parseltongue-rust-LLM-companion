use super::DuckdbAnalysisStoreAdapter;
#[path = "original-item-graph-projection.rs"]
mod original_item_graph_projection;
use parseltongue_core_row_types::{
    CompilationUnitRoleKind, CrateCompilationUnitKey, FileAreaIdentityKey, HldEdgeInputRecord,
    HldEdgeResolutionValue, HldEffectiveVisibilityKind, HldEvidenceRowKey,
    HldInputCompletenessKind, HldItemKindValue, HldMaterializationInputRecord,
    HldNodeInputCategory, HldNodeInputRecord, HldOwnerKindValue, HldRelationKindValue,
    HldSourceCoverageKind, HldTargetIdentityValue, HldUnitOriginKind, HldUnitProvenanceRecord,
    NameSymbolTargetKey, RunCancelFlagValue, RunGenerationDigestValue, SemanticOccurrenceKeyValue,
    SpanByteRangeValue, SpanDigestIdValue, StageRunScopeValue, StoreBackendFailureError,
};
use parseltongue_table_store_traits::{AnalysisTableStorePort, StageTableStorePort};
use serde_json::Value;
use std::{collections::BTreeSet, path::Path};

const ORIGINAL_SOURCE_PROJECTION_TABLE_NAMES: &[&str] = &[
    "crate-dependency-graph-table",
    "syntax-namespace-input-scopes",
    "syntax-namespace-input-items",
    "syntax-namespace-input-imports",
    "syntax-file-work-requests",
    "file-source-text-table",
    "span-reference-text-table",
    "span-byte-ownership-ranges",
];

#[derive(Debug, thiserror::Error)]
pub enum OriginalSourceEvidenceError {
    #[error("original source file digest does not match its content")]
    FileDigestMismatch,
    #[error("original source span range is invalid")]
    InvalidSpanRange,
    #[error("original source span is not valid UTF-8")]
    InvalidSpanUtf8,
    #[error("original source span text or digest does not match persisted evidence")]
    SpanEvidenceMismatch,
}

#[derive(Debug, thiserror::Error)]
pub enum OriginalSourceProjectionError {
    #[error("source projection storage failed: {0}")]
    Backend(#[from] StoreBackendFailureError),
    #[error("mapped source unit has no canonical root namespace mapping: {raw_unit_key}")]
    MissingRootMapping { raw_unit_key: String },
    #[error("reached original source request has no frozen file row: {file_path}")]
    MissingFileEvidence { file_path: String },
    #[error("cfg-active original top-level syntax span has no item ownership row")]
    MissingSpanOwnership,
    #[error("original item has no unique namespace context: {occurrence:?} ({count} contexts)")]
    InvalidNamespaceContext {
        occurrence: Box<SemanticOccurrenceKeyValue>,
        count: usize,
    },
    #[error("original source row is malformed in {table}: {reason}")]
    InvalidSourceRow { table: String, reason: String },
    #[error("active top-level syntax kind is unsupported: {syntax_kind}")]
    UnsupportedActiveSyntax { syntax_kind: String },
    #[error("original source evidence is invalid")]
    InvalidEvidence(#[from] OriginalSourceEvidenceError),
}

impl DuckdbAnalysisStoreAdapter {
    pub fn open_source_projection_store(
        database: &Path,
        run_directory: &Path,
    ) -> Result<Self, StoreBackendFailureError> {
        if !run_directory.is_dir() {
            return Err(StoreBackendFailureError {
                error_context_detail_text: "source projection run directory must exist".into(),
            });
        }
        let spill = run_directory
            .canonicalize()
            .map_err(|error| StoreBackendFailureError {
                error_context_detail_text: error.to_string(),
            })?
            .join("duckdb-source-projection-spill-files");
        std::fs::create_dir_all(&spill).map_err(|error| StoreBackendFailureError {
            error_context_detail_text: error.to_string(),
        })?;
        let spill = spill.to_str().ok_or_else(|| StoreBackendFailureError {
            error_context_detail_text: "source projection spill path is not UTF-8".into(),
        })?;
        let config = duckdb::Config::default()
            .with("temp_directory", spill)
            .and_then(|config| config.enable_external_access(true))
            .and_then(|config| config.with("autoinstall_known_extensions", "false"))
            .and_then(|config| config.with("autoload_known_extensions", "false"))
            .and_then(|config| config.with("allow_community_extensions", "false"))
            .and_then(|config| config.with("threads", "2"))
            .and_then(|config| config.with("preserve_insertion_order", "false"))
            .map_err(|error| StoreBackendFailureError {
                error_context_detail_text: error.to_string(),
            })?;
        let connection =
            duckdb::Connection::open_with_flags(database, config).map_err(|error| {
                StoreBackendFailureError {
                    error_context_detail_text: error.to_string(),
                }
            })?;
        connection
            .execute_batch("LOAD json; SET enable_external_access = false;")
            .map_err(|error| StoreBackendFailureError {
                error_context_detail_text: error.to_string(),
            })?;
        let external_access_enabled = connection
            .query_row(
                "SELECT current_setting('enable_external_access')",
                [],
                |row| row.get::<_, bool>(0),
            )
            .map_err(|error| StoreBackendFailureError {
                error_context_detail_text: error.to_string(),
            })?;
        if external_access_enabled {
            return Err(StoreBackendFailureError {
                error_context_detail_text:
                    "source projection requires external access disabled after extension load"
                        .into(),
            });
        }
        let memory_limit = connection
            .query_row("SELECT current_setting('memory_limit')", [], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|error| StoreBackendFailureError {
                error_context_detail_text: error.to_string(),
            })?;
        eprintln!("duckdb-source-projection-memory-limit={memory_limit}");
        Ok(Self {
            _connection: std::sync::Mutex::new(connection),
        })
    }

    pub fn derive_original_top_level_containment_input(
        &self,
        source: &dyn StageTableStorePort,
        source_scope: &StageRunScopeValue,
        destination_base_scope: &StageRunScopeValue,
        workspace_root: &Path,
        cancel: &RunCancelFlagValue,
    ) -> Result<HldMaterializationInputRecord, OriginalSourceProjectionError> {
        self.copy_published_stage_rows(
            source,
            source_scope,
            ORIGINAL_SOURCE_PROJECTION_TABLE_NAMES,
            cancel,
        )?;
        let connection = self._connection.lock().map_err(|error| {
            OriginalSourceProjectionError::Backend(StoreBackendFailureError {
                error_context_detail_text: error.to_string(),
            })
        })?;
        connection
            .execute_batch(include_str!("hld-original-source-projection.sql"))
            .map_err(|error| projection_backend_error(error.to_string()))?;
        let missing = read_projection_rows::<1>(
            &connection,
            "SELECT unit.payload FROM \"frozen-generation-source-records\" AS unit WHERE unit.generation_key=? AND unit.attempt_key=? AND unit.table_name='crate-dependency-graph-table' AND json_extract_string(unit.payload, '$.unit_mapped_target_flag')='true' AND NOT EXISTS (SELECT 1 FROM \"frozen-generation-source-records\" AS root_scope WHERE root_scope.generation_key=unit.generation_key AND root_scope.attempt_key=unit.attempt_key AND root_scope.table_name='syntax-namespace-input-scopes' AND json_extract_string(root_scope.payload, '$.scope_cargo_unit_key')=json_extract_string(unit.payload, '$.unit_compilation_identity_key') AND json_array_length(json_extract(root_scope.payload, '$.scope_module_identity_key.scope_semantic_path_parts'))=0)",
            source_scope,
        )?;
        if let Some(row) = missing.first() {
            let unit: Value = decode_json(&row[0], "crate-dependency-graph-table")?;
            return Err(OriginalSourceProjectionError::MissingRootMapping {
                raw_unit_key: required_string(
                    &unit,
                    "unit_compilation_identity_key",
                    "crate-dependency-graph-table",
                )?
                .to_owned(),
            });
        }
        let missing_files = read_projection_rows::<1>(
            &connection,
            "SELECT request_payload FROM \"original-source-missing-files\" WHERE generation_key=? AND attempt_key=?",
            source_scope,
        )?;
        if let Some(row) = missing_files.first() {
            let request: Value = decode_json(&row[0], "syntax-file-work-requests")?;
            return Err(OriginalSourceProjectionError::MissingFileEvidence {
                file_path: required_string(
                    &request,
                    "file_source_absolute_path",
                    "syntax-file-work-requests",
                )?
                .into(),
            });
        }
        let unowned_items = read_projection_rows::<1>(
            &connection,
            "SELECT span_payload FROM \"original-source-unowned-items\" WHERE generation_key=? AND attempt_key=?",
            source_scope,
        )?;
        if !unowned_items.is_empty() {
            return Err(OriginalSourceProjectionError::MissingSpanOwnership);
        }
        let root_rows = read_projection_rows::<2>(
            &connection,
            "SELECT unit.payload,root_scope.payload FROM \"frozen-generation-source-records\" AS unit JOIN \"frozen-generation-source-records\" AS root_scope ON root_scope.generation_key=unit.generation_key AND root_scope.attempt_key=unit.attempt_key AND root_scope.table_name='syntax-namespace-input-scopes' AND json_extract_string(root_scope.payload, '$.scope_cargo_unit_key')=json_extract_string(unit.payload, '$.unit_compilation_identity_key') AND json_array_length(json_extract(root_scope.payload, '$.scope_module_identity_key.scope_semantic_path_parts'))=0 WHERE unit.generation_key=? AND unit.attempt_key=? AND unit.table_name='crate-dependency-graph-table' AND json_extract_string(unit.payload, '$.unit_mapped_target_flag')='true'",
            source_scope,
        )?;
        let area_rows = read_projection_rows::<4>(
            &connection,
            "SELECT unit_payload,root_scope_payload,request_payload,file_payload FROM \"original-source-area-candidates\" WHERE generation_key=? AND attempt_key=? ORDER BY unit_payload,request_payload",
            source_scope,
        )?;
        let item_rows = read_projection_rows::<5>(
            &connection,
            "SELECT unit_payload,root_scope_payload,request_payload,file_payload,span_payload FROM \"original-source-item-candidates\" WHERE generation_key=? AND attempt_key=? ORDER BY unit_payload,request_payload,span_payload",
            source_scope,
        )?;
        let generation = decode_generation(&source_scope.run_generation_digest_value)?;
        let units = root_rows
            .iter()
            .map(|row| {
                let raw: Value = decode_json(&row[0], "crate-dependency-graph-table")?;
                let root: Value = decode_json(&row[1], "syntax-namespace-input-scopes")?;
                canonical_unit(&raw, &root).map(|unit| HldUnitProvenanceRecord {
                    unit_compilation_identity_key: unit,
                    unit_origin_kind: HldUnitOriginKind::Workspace,
                })
            })
            .collect::<Result<BTreeSet<_>, OriginalSourceProjectionError>>()?;
        let mut nodes = BTreeSet::new();
        let mut edges = BTreeSet::new();
        for row in &area_rows {
            check_projection_cancel(cancel)?;
            let unit_value: Value = decode_json(&row[0], "crate-dependency-graph-table")?;
            let root_value: Value = decode_json(&row[1], "syntax-namespace-input-scopes")?;
            let request: Value = decode_json(&row[2], "syntax-file-work-requests")?;
            let file: Value = decode_json(&row[3], "file-source-text-table")?;
            let unit = canonical_unit(&unit_value, &root_value)?;
            let path = source_relative_path(&unit_value, &request, workspace_root)?;
            let absolute_path = required_string(
                &request,
                "file_source_absolute_path",
                "syntax-file-work-requests",
            )?;
            let file_digest =
                required_string(&file, "file_content_digest_full", "file-source-text-table")?;
            let bytes = source.read_file_content_bytes(file_digest)?;
            validate_original_file_digest(&bytes, file_digest)?;
            let area = HldTargetIdentityValue::FilePublicArea(
                FileAreaIdentityKey::build_file_area_identity(&unit, &path).map_err(|error| {
                    invalid_source_row("syntax-file-work-requests", error.to_string())
                })?,
            );
            nodes.insert(HldNodeInputRecord {
                run_generation_digest_value: generation,
                node_target_reference: area,
                node_input_category: HldNodeInputCategory::FilePublicArea,
                node_item_kind_value: Some(HldItemKindValue::FilePublicArea),
                node_owner_kind_value: None,
                node_function_body_ancestry_flag: false,
                node_effective_visibility_kind: None,
                node_reachable_outside_crate_flag: None,
                node_source_relative_path: Some(path),
                node_qualified_path_text: format!(
                    "{}::<file-area>",
                    unit.unit_cargo_package_identifier
                ),
                node_signature_text: None,
            });
            let _ = absolute_path;
        }
        for row in &item_rows {
            check_projection_cancel(cancel)?;
            let unit_value: Value = decode_json(&row[0], "crate-dependency-graph-table")?;
            let root_value: Value = decode_json(&row[1], "syntax-namespace-input-scopes")?;
            let request: Value = decode_json(&row[2], "syntax-file-work-requests")?;
            let file: Value = decode_json(&row[3], "file-source-text-table")?;
            let span: Value = decode_json(&row[4], "span-reference-text-table")?;
            let unit = canonical_unit(&unit_value, &root_value)?;
            let path = source_relative_path(&unit_value, &request, workspace_root)?;
            let absolute_path = required_string(
                &request,
                "file_source_absolute_path",
                "syntax-file-work-requests",
            )?;
            let file_digest =
                required_string(&file, "file_content_digest_full", "file-source-text-table")?;
            let bytes = source.read_file_content_bytes(file_digest)?;
            let start = required_u64(&span, "span_byte_start_offset", "span-reference-text-table")?;
            let end = required_u64(&span, "span_byte_end_offset", "span-reference-text-table")?;
            let text =
                required_string(&span, "span_source_code_text", "span-reference-text-table")?;
            let span_digest = required_string(
                &span,
                "span_content_digest_full",
                "span-reference-text-table",
            )?;
            validate_original_source_evidence(&bytes, file_digest, start, end, text, span_digest)?;
            let syntax_kind =
                required_string(&span, "span_syntax_kind_value", "span-reference-text-table")?;
            let Some(item_kind) = decode_item_kind(syntax_kind)? else {
                continue;
            };
            let digest = decode_digest(span_digest)?;
            let occurrence = SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
                &unit,
                &path,
                SpanByteRangeValue::try_from((start, end)).map_err(|error| {
                    invalid_source_row("span-reference-text-table", error.to_string())
                })?,
                SpanDigestIdValue::from(digest),
                &[],
            )
            .map_err(|error| invalid_source_row("span-reference-text-table", error.to_string()))?;
            let target =
                HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(occurrence.clone()));
            let area = HldTargetIdentityValue::FilePublicArea(
                FileAreaIdentityKey::build_file_area_identity(&unit, &path).map_err(|error| {
                    invalid_source_row("syntax-file-work-requests", error.to_string())
                })?,
            );
            let context = read_original_namespace_context(
                &connection,
                source_scope,
                &root_value["scope_module_identity_key"]["scope_compilation_unit_key"],
                &occurrence,
                item_kind == HldItemKindValue::UseStatement,
            )?;
            let visibility = context.visibility;
            let is_root = context.module_path.is_empty();
            let reachability = if visibility == HldEffectiveVisibilityKind::RestrictedNonPrivate
                || visibility == HldEffectiveVisibilityKind::PrivateEquivalent
            {
                Some(false)
            } else if is_root {
                Some(true)
            } else {
                None
            };
            let item_name = span
                .get("span_declared_name_value")
                .and_then(Value::as_str)
                .unwrap_or(syntax_kind)
                .trim_start_matches("r#");
            let label = std::iter::once(unit.unit_cargo_package_identifier.as_str())
                .chain(context.module_path.iter().map(String::as_str))
                .chain(std::iter::once(item_name))
                .collect::<Vec<_>>()
                .join("::");
            nodes.insert(HldNodeInputRecord {
                run_generation_digest_value: generation,
                node_target_reference: target.clone(),
                node_input_category: HldNodeInputCategory::LocalItem,
                node_item_kind_value: Some(item_kind),
                node_owner_kind_value: Some(if is_root {
                    HldOwnerKindValue::CrateRoot
                } else {
                    HldOwnerKindValue::Module
                }),
                node_function_body_ancestry_flag: false,
                node_effective_visibility_kind: Some(visibility),
                node_reachable_outside_crate_flag: reachability,
                node_source_relative_path: Some(path),
                node_qualified_path_text: label,
                node_signature_text: None,
            });
            edges.insert(HldEdgeInputRecord {
                run_generation_digest_value: generation,
                edge_source_reference: area.clone(),
                edge_relation_kind_value: HldRelationKindValue::Contains,
                edge_resolution_value: HldEdgeResolutionValue::Extracted {
                    target: target.clone(),
                },
                edge_evidence_row_key: HldEvidenceRowKey {
                    evidence_site_occurrence_key: occurrence.clone(),
                    evidence_source_occurrence_key: area,
                    evidence_relation_kind_value: HldRelationKindValue::Contains,
                    evidence_target_reference_value: Some(target),
                },
                edge_dispatch_metadata_text: None,
            });
            let _ = absolute_path;
        }
        let input = HldMaterializationInputRecord {
            run_generation_digest_value: generation,
            stage_attempt_sequence_value: destination_base_scope
                .stage_attempt_sequence_value
                .clone(),
            semantic_input_completeness_kind: HldInputCompletenessKind::CompleteTypedInput,
            source_coverage_kind: HldSourceCoverageKind::OriginalTopLevelContainment,
            input_unit_provenance_records: units.into_iter().collect(),
            input_node_records: nodes.into_iter().collect(),
            input_edge_records: edges.into_iter().collect(),
        };
        parseltongue_core_row_types::validate_hld_materialization_input(&input)
            .map_err(|error| invalid_source_row("original-source-graph", error.to_string()))?;
        Ok(input)
    }
}

fn projection_backend_error(message: String) -> OriginalSourceProjectionError {
    OriginalSourceProjectionError::Backend(StoreBackendFailureError {
        error_context_detail_text: message,
    })
}

fn invalid_source_row(table: &str, reason: String) -> OriginalSourceProjectionError {
    OriginalSourceProjectionError::InvalidSourceRow {
        table: table.into(),
        reason,
    }
}

fn check_projection_cancel(
    cancel: &RunCancelFlagValue,
) -> Result<(), OriginalSourceProjectionError> {
    if cancel.check_stage_work_cancelled() {
        Err(projection_backend_error(
            "original source projection cancelled".into(),
        ))
    } else {
        Ok(())
    }
}

fn read_projection_rows<const N: usize>(
    connection: &duckdb::Connection,
    query: &str,
    scope: &StageRunScopeValue,
) -> Result<Vec<[String; N]>, OriginalSourceProjectionError> {
    let mut statement = connection
        .prepare(query)
        .map_err(|error| projection_backend_error(error.to_string()))?;
    statement
        .query_map(
            duckdb::params![
                scope.run_generation_digest_value,
                scope.stage_attempt_sequence_value
            ],
            |row| {
                let columns = (0..N)
                    .map(|index| row.get::<_, String>(index))
                    .collect::<Result<Vec<_>, _>>()?;
                columns
                    .try_into()
                    .map_err(|_| duckdb::Error::InvalidColumnIndex(N))
            },
        )
        .map_err(|error| projection_backend_error(error.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| projection_backend_error(error.to_string()))
}

fn decode_json(text: &str, table: &str) -> Result<Value, OriginalSourceProjectionError> {
    serde_json::from_str(text).map_err(|error| invalid_source_row(table, error.to_string()))
}

fn required_string<'a>(
    value: &'a Value,
    key: &str,
    table: &str,
) -> Result<&'a str, OriginalSourceProjectionError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| invalid_source_row(table, format!("missing string field {key}")))
}

fn required_u64(
    value: &Value,
    key: &str,
    table: &str,
) -> Result<u64, OriginalSourceProjectionError> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid_source_row(table, format!("missing integer field {key}")))
}

fn decode_digest(text: &str) -> Result<[u8; 32], OriginalSourceProjectionError> {
    blake3::Hash::from_hex(text)
        .map(|value| *value.as_bytes())
        .map_err(|error| invalid_source_row("span-reference-text-table", error.to_string()))
}

fn decode_generation(
    text: &str,
) -> Result<RunGenerationDigestValue, OriginalSourceProjectionError> {
    decode_digest(text).map(RunGenerationDigestValue)
}

fn canonical_unit(
    raw: &Value,
    root: &Value,
) -> Result<CrateCompilationUnitKey, OriginalSourceProjectionError> {
    let raw_key = required_string(
        raw,
        "unit_compilation_identity_key",
        "crate-dependency-graph-table",
    )?;
    if required_string(
        root,
        "scope_cargo_unit_key",
        "syntax-namespace-input-scopes",
    )? != raw_key
    {
        return Err(invalid_source_row(
            "syntax-namespace-input-scopes",
            "root mapping does not match raw unit key".into(),
        ));
    }
    let identity = root
        .get("scope_module_identity_key")
        .and_then(Value::as_object)
        .and_then(|scope| scope.get("scope_compilation_unit_key"))
        .and_then(Value::as_object)
        .ok_or_else(|| {
            invalid_source_row(
                "syntax-namespace-input-scopes",
                "canonical root unit is missing".into(),
            )
        })?;
    let role = match identity
        .get("unit_compilation_role_kind")
        .and_then(Value::as_str)
    {
        Some("target") => CompilationUnitRoleKind::Target,
        Some("host") => CompilationUnitRoleKind::Host,
        _ => {
            return Err(invalid_source_row(
                "syntax-namespace-input-scopes",
                "canonical unit role is invalid".into(),
            ));
        }
    };
    let digest = required_object_string(
        identity,
        "unit_configuration_digest_full",
        "syntax-namespace-input-scopes",
    )?;
    Ok(CrateCompilationUnitKey {
        unit_cargo_package_identifier: required_object_string(
            identity,
            "unit_cargo_package_identifier",
            "syntax-namespace-input-scopes",
        )?
        .into(),
        unit_cargo_target_name: required_object_string(
            identity,
            "unit_cargo_target_name",
            "syntax-namespace-input-scopes",
        )?
        .into(),
        unit_cargo_target_kind: required_object_string(
            identity,
            "unit_cargo_target_kind",
            "syntax-namespace-input-scopes",
        )?
        .into(),
        unit_compilation_role_kind: role,
        unit_configuration_digest_full: decode_digest(digest)?,
    })
}

fn required_object_string<'a>(
    value: &'a serde_json::Map<String, Value>,
    key: &str,
    table: &str,
) -> Result<&'a str, OriginalSourceProjectionError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| invalid_source_row(table, format!("missing string field {key}")))
}

fn source_relative_path(
    raw_unit: &Value,
    request: &Value,
    workspace_root: &Path,
) -> Result<String, OriginalSourceProjectionError> {
    let source = Path::new(required_string(
        request,
        "file_source_absolute_path",
        "syntax-file-work-requests",
    )?);
    let package = Path::new(required_string(
        raw_unit,
        "unit_package_root_path",
        "crate-dependency-graph-table",
    )?);
    let relative = source
        .strip_prefix(package)
        .or_else(|_| source.strip_prefix(workspace_root));
    let path = relative
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| {
            format!(
                "external-source/{}/source.rs",
                blake3::hash(source.to_string_lossy().as_bytes()).to_hex()
            )
        });
    Ok(path)
}

fn validate_original_file_digest(
    bytes: &[u8],
    expected: &str,
) -> Result<(), OriginalSourceProjectionError> {
    if blake3::hash(bytes).to_hex().as_str() == expected {
        Ok(())
    } else {
        Err(OriginalSourceEvidenceError::FileDigestMismatch.into())
    }
}

fn decode_item_kind(kind: &str) -> Result<Option<HldItemKindValue>, OriginalSourceProjectionError> {
    match kind {
        "FN" => Ok(Some(HldItemKindValue::Function)),
        "STRUCT" => Ok(Some(HldItemKindValue::Struct)),
        "ENUM" => Ok(Some(HldItemKindValue::Enum)),
        "TRAIT" => Ok(Some(HldItemKindValue::Trait)),
        "TYPE_ALIAS" => Ok(Some(HldItemKindValue::TypeAlias)),
        "CONST" => Ok(Some(HldItemKindValue::Constant)),
        "STATIC" => Ok(Some(HldItemKindValue::Static)),
        "MODULE" => Ok(Some(HldItemKindValue::Module)),
        "USE" => Ok(Some(HldItemKindValue::UseStatement)),
        "MACRO_CALL" | "MACRO_RULES" | "IMPL" | "EXTERN_BLOCK" => Ok(None),
        other => Err(OriginalSourceProjectionError::UnsupportedActiveSyntax {
            syntax_kind: other.into(),
        }),
    }
}

struct OriginalItemNamespaceContext {
    module_path: Vec<String>,
    visibility: HldEffectiveVisibilityKind,
}

fn read_original_namespace_context(
    connection: &duckdb::Connection,
    scope: &StageRunScopeValue,
    unit: &Value,
    occurrence: &SemanticOccurrenceKeyValue,
    is_import: bool,
) -> Result<OriginalItemNamespaceContext, OriginalSourceProjectionError> {
    let parts = occurrence.read_semantic_occurrence_parts();
    let (start, end) = parts.span_byte_range_value.read_source_range_pair();
    let identity = serde_json::json!({
        "unit_compilation_identity_key": unit,
        "span_source_file_path": parts.span_source_file_path,
        "span_byte_range_value": [start, end],
        "span_content_digest_full": blake3::Hash::from(*parts.span_content_digest_full.read_full_digest_bytes()).to_hex().to_string(),
        "span_expansion_provenance_chain": [],
    });
    let mut statement = connection
        .prepare(include_str!("hld-original-namespace-context.sql"))
        .map_err(|error| projection_backend_error(error.to_string()))?;
    let records = statement
        .query_map(
            duckdb::params![
                scope.run_generation_digest_value,
                scope.stage_attempt_sequence_value,
                identity.to_string(),
                is_import,
                unit.to_string(),
                parts.span_source_file_path,
                start,
                end,
            ],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(|error| projection_backend_error(error.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| projection_backend_error(error.to_string()))?;
    let [(owner, visibility)] = records.as_slice() else {
        return Err(OriginalSourceProjectionError::InvalidNamespaceContext {
            occurrence: Box::new(occurrence.clone()),
            count: records.len(),
        });
    };
    let owner: Value = decode_json(owner, "syntax-namespace-input-scopes")?;
    let module_path = decode_original_scope_path(&owner, unit)?;
    let visibility: Value = decode_json(visibility, "syntax-namespace-input-items")?;
    let visibility = match required_string(
        &visibility,
        "visibility_scope_kind_value",
        "syntax-namespace-input-items",
    )? {
        "public" => HldEffectiveVisibilityKind::Public,
        "restricted" => {
            let restricted =
                decode_original_scope_path(&visibility["visibility_restricted_scope_key"], unit)?;
            if restricted == module_path {
                HldEffectiveVisibilityKind::PrivateEquivalent
            } else if module_path.starts_with(&restricted) {
                HldEffectiveVisibilityKind::RestrictedNonPrivate
            } else {
                return Err(invalid_source_row(
                    "syntax-namespace-input-items",
                    "visibility scope is not an ancestor of the owner".into(),
                ));
            }
        }
        _ => {
            return Err(invalid_source_row(
                "syntax-namespace-input-items",
                "unsupported original item visibility".into(),
            ));
        }
    };
    Ok(OriginalItemNamespaceContext {
        module_path,
        visibility,
    })
}

fn decode_original_scope_path(
    scope: &Value,
    unit: &Value,
) -> Result<Vec<String>, OriginalSourceProjectionError> {
    if &scope["scope_compilation_unit_key"] != unit {
        return Err(invalid_source_row(
            "syntax-namespace-input-scopes",
            "namespace scope has a different compilation unit".into(),
        ));
    }
    let parts: Vec<String> = serde_json::from_value(scope["scope_semantic_path_parts"].clone())
        .map_err(|error| invalid_source_row("syntax-namespace-input-scopes", error.to_string()))?;
    if parts
        .iter()
        .any(|part| part.is_empty() || part.starts_with('@'))
    {
        return Err(invalid_source_row(
            "syntax-namespace-input-scopes",
            "original item owner is not a module path".into(),
        ));
    }
    Ok(parts)
}

pub(super) fn validate_original_source_evidence(
    file_bytes: &[u8],
    expected_file_digest: &str,
    start: u64,
    end: u64,
    expected_span_text: &str,
    expected_span_digest: &str,
) -> Result<(), OriginalSourceEvidenceError> {
    if blake3::hash(file_bytes).to_hex().as_str() != expected_file_digest {
        return Err(OriginalSourceEvidenceError::FileDigestMismatch);
    }
    let (Ok(start), Ok(end)) = (usize::try_from(start), usize::try_from(end)) else {
        return Err(OriginalSourceEvidenceError::InvalidSpanRange);
    };
    let Some(span_bytes) = file_bytes.get(start..end) else {
        return Err(OriginalSourceEvidenceError::InvalidSpanRange);
    };
    let span_text = std::str::from_utf8(span_bytes)
        .map_err(|_| OriginalSourceEvidenceError::InvalidSpanUtf8)?;
    if span_text != expected_span_text
        || blake3::hash(span_bytes).to_hex().as_str() != expected_span_digest
    {
        return Err(OriginalSourceEvidenceError::SpanEvidenceMismatch);
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{
        OriginalSourceEvidenceError, OriginalSourceProjectionError, decode_item_kind,
        read_original_namespace_context, validate_original_source_evidence,
    };
    use duckdb::Connection;
    use parseltongue_core_row_types::{
        CompilationUnitRoleKind, CrateCompilationUnitKey, HldEffectiveVisibilityKind,
        SemanticOccurrenceKeyValue, SpanByteRangeValue, SpanDigestIdValue, StageRunScopeValue,
    };
    use serde_json::{Value, json};

    /// REQ-RUST-040.0
    #[test]
    fn test_source_namespace_input_validation() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch("CREATE TABLE \"frozen-generation-source-records\" (generation_key VARCHAR, attempt_key VARCHAR, table_name VARCHAR, payload JSON)").unwrap();
        let unit = CrateCompilationUnitKey {
            unit_cargo_package_identifier: "fixture".into(),
            unit_cargo_target_name: "fixture".into(),
            unit_cargo_target_kind: "lib".into(),
            unit_compilation_role_kind: CompilationUnitRoleKind::Target,
            unit_configuration_digest_full: [1; 32],
        };
        let unit_json = json!({
            "unit_cargo_package_identifier": "fixture",
            "unit_cargo_target_name": "fixture",
            "unit_cargo_target_kind": "lib",
            "unit_compilation_role_kind": "target",
            "unit_configuration_digest_full": "01".repeat(32),
        });
        let occurrence = SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
            &unit,
            "src/child.rs",
            SpanByteRangeValue::try_from((0, 10)).unwrap(),
            SpanDigestIdValue::from([2; 32]),
            &[],
        )
        .unwrap();
        let scope = StageRunScopeValue {
            run_generation_digest_value: "g".into(),
            stage_attempt_sequence_value: "a".into(),
        };
        let owner =
            json!({"scope_compilation_unit_key":unit_json, "scope_semantic_path_parts":["child"]});
        connection.execute("INSERT INTO \"frozen-generation-source-records\" VALUES ('g', 'a', 'syntax-namespace-input-scopes', ?)",
            [json!({"scope_module_identity_key":owner, "scope_module_boundary_flag":true}).to_string()]).unwrap();
        let public = json!({"visibility_scope_kind_value":"public"});
        let restricted = |parts: Value| json!({"visibility_scope_kind_value":"restricted", "visibility_restricted_scope_key":{"scope_compilation_unit_key":unit_json,"scope_semantic_path_parts":parts}});
        for (label, visibilities, range, generation, expected) in [
            (
                "public",
                vec![public.clone()],
                [1, 9],
                "g",
                Some(HldEffectiveVisibilityKind::Public),
            ),
            (
                "private",
                vec![restricted(json!(["child"]))],
                [1, 9],
                "g",
                Some(HldEffectiveVisibilityKind::PrivateEquivalent),
            ),
            (
                "ancestor",
                vec![restricted(json!([]))],
                [1, 9],
                "g",
                Some(HldEffectiveVisibilityKind::RestrictedNonPrivate),
            ),
            ("missing", vec![], [1, 9], "g", None),
            (
                "conflicting",
                vec![public.clone(), restricted(json!(["child"]))],
                [1, 9],
                "g",
                None,
            ),
            (
                "unknown kind",
                vec![json!({"visibility_scope_kind_value":"future"})],
                [1, 9],
                "g",
                None,
            ),
            (
                "unknown scope",
                vec![restricted(json!(["missing"]))],
                [1, 9],
                "g",
                None,
            ),
            (
                "malformed scope",
                vec![restricted(json!([42]))],
                [1, 9],
                "g",
                None,
            ),
            (
                "stale generation",
                vec![public.clone()],
                [1, 9],
                "old",
                None,
            ),
            ("outside item", vec![public.clone()], [1, 11], "g", None),
            ("inverted range", vec![public.clone()], [9, 1], "g", None),
            ("empty range", vec![public], [1, 1], "g", None),
        ] {
            connection.execute("DELETE FROM \"frozen-generation-source-records\" WHERE table_name='syntax-namespace-input-imports'", []).unwrap();
            for visibility in visibilities {
                let payload = json!({
                    "import_lexical_scope_key":owner,
                    "import_visibility_scope_kind":visibility,
                    "import_resolution_source_kind":"Import",
                    "import_site_occurrence_key":{
                        "unit_compilation_identity_key":unit_json,
                        "span_source_file_path":"src/child.rs",
                        "span_byte_range_value":range,
                        "span_expansion_provenance_chain":[],
                    },
                });
                connection.execute("INSERT INTO \"frozen-generation-source-records\" VALUES (?, 'a', 'syntax-namespace-input-imports', ?)", [generation, &payload.to_string()]).unwrap();
            }
            let actual =
                read_original_namespace_context(&connection, &scope, &unit_json, &occurrence, true);
            match expected {
                Some(visibility) => {
                    let context = actual.unwrap();
                    assert_eq!(context.visibility, visibility, "{label}");
                    assert_eq!(context.module_path, ["child"]);
                }
                None => assert!(actual.is_err(), "{label} must fail closed"),
            }
        }
    }

    /// REQ-RUST-040.0
    #[test]
    fn test_source_missing_evidence_detected() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE \"frozen-generation-source-records\" (generation_key VARCHAR, attempt_key VARCHAR, table_name VARCHAR, payload JSON);",
            )
            .unwrap();
        let insert = |table: &str, payload: Value| {
            connection
                .execute(
                    "INSERT INTO \"frozen-generation-source-records\" VALUES ('g', 'a', ?, ?)",
                    duckdb::params![table, payload.to_string()],
                )
                .unwrap();
        };
        let absolute_path = "/workspace/src/lib.rs";
        insert(
            "crate-dependency-graph-table",
            json!({"unit_compilation_identity_key":"raw","unit_mapped_target_flag":true}),
        );
        insert(
            "syntax-namespace-input-scopes",
            json!({"scope_cargo_unit_key":"raw","scope_module_identity_key":{"scope_semantic_path_parts":[]}}),
        );
        for path in [absolute_path, "/workspace/src/missing.rs"] {
            insert(
                "syntax-file-work-requests",
                json!({"unit_compilation_identity_key":"raw","file_parse_context_kind":"items","file_expansion_provenance_chain":[],"file_source_absolute_path":path}),
            );
        }
        insert(
            "file-source-text-table",
            json!({"file_source_absolute_path":absolute_path,"file_content_digest_full":"digest"}),
        );
        insert(
            "span-reference-text-table",
            json!({"unit_compilation_identity_key":"raw","file_source_absolute_path":absolute_path,"span_configuration_active_flag":true,"span_expansion_provenance_chain":[],"span_byte_start_offset":0,"span_byte_end_offset":1}),
        );
        connection
            .execute_batch(include_str!("hld-original-source-projection.sql"))
            .unwrap();
        let missing_files: i64 = connection
            .query_row(
                "SELECT count(*) FROM \"original-source-missing-files\"",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let missing_ownership: i64 = connection
            .query_row(
                "SELECT count(*) FROM \"original-source-unowned-items\"",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(missing_files, 1);
        assert_eq!(missing_ownership, 1);
    }

    /// REQ-RUST-040.0
    #[test]
    fn test_source_file_digest_mismatch() {
        assert!(matches!(
            validate_original_source_evidence(
                b"pub struct A;",
                "wrong",
                0,
                13,
                "pub struct A;",
                "wrong"
            ),
            Err(OriginalSourceEvidenceError::FileDigestMismatch)
        ));
    }

    /// REQ-RUST-040.0
    #[test]
    fn test_source_span_range_rejected() {
        let bytes = b"pub struct A;";
        assert!(matches!(
            validate_original_source_evidence(
                bytes,
                blake3::hash(bytes).to_hex().as_str(),
                0,
                99,
                "",
                "",
            ),
            Err(OriginalSourceEvidenceError::InvalidSpanRange)
        ));
    }

    /// REQ-RUST-040.0
    #[test]
    fn test_source_span_utf8_rejected() {
        let bytes = [0xff];
        assert!(matches!(
            validate_original_source_evidence(
                &bytes,
                blake3::hash(&bytes).to_hex().as_str(),
                0,
                1,
                "",
                "",
            ),
            Err(OriginalSourceEvidenceError::InvalidSpanUtf8)
        ));
    }

    /// REQ-RUST-040.0
    #[test]
    fn test_source_span_text_mismatch() {
        let bytes = b"pub struct A;";
        assert!(matches!(
            validate_original_source_evidence(
                bytes,
                blake3::hash(bytes).to_hex().as_str(),
                0,
                bytes.len() as u64,
                "different",
                blake3::hash(bytes).to_hex().as_str(),
            ),
            Err(OriginalSourceEvidenceError::SpanEvidenceMismatch)
        ));
    }

    /// REQ-RUST-040.0
    #[test]
    fn test_source_unknown_syntax_rejected() {
        assert!(matches!(
            decode_item_kind("ACTIVE_UNKNOWN_SYNTAX"),
            Err(OriginalSourceProjectionError::UnsupportedActiveSyntax { syntax_kind })
                if syntax_kind == "ACTIVE_UNKNOWN_SYNTAX"
        ));
    }
}

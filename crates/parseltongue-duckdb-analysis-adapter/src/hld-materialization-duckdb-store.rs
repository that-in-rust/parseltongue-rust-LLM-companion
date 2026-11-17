use crate::DuckdbAnalysisStoreAdapter;
use parseltongue_core_row_types::{
    CrateCompilationUnitKey, HldDependencyOriginKind, HldEdgeResolutionValue,
    HldEndpointRollupBucket, HldEvidencePathRecord, HldEvidenceRowKey, HldExternalNameKind,
    HldFileRollupRecord, HldFolderEndpointBucket, HldFolderRollupRecord, HldGraphEdgeKey,
    HldMaterializationInputRecord, HldMaterializationOutputRecord,
    HldMaterializationValidationError, HldMaterializedEdgeRecord, HldNodeInputCategory,
    HldNodeInputRecord, HldOutputCompletenessKind, HldOutputEdgeStatus, HldRelationKindValue,
    HldSelectedNodeRecord, HldTargetIdentityValue, HldUnitOriginKind, HldViaKindValue,
    NameSymbolTargetKey, RunCancelFlagValue, SemanticOccurrenceKeyValue,
    validate_hld_materialization_input,
};
use parseltongue_table_store_traits::{
    HldAnalysisOperationKind, HldAnalysisPortError, HldGraphAnalysisPort,
};
use serde_json::{Value as JsonValue, json};
use std::collections::BTreeMap;

const HLD_NORMALIZE_NODE_SQL: &str = r#"
CREATE OR REPLACE TEMP TABLE "hld-analysis-node-records" AS
SELECT
    input.*,
    CASE
        WHEN endpoint_category_kind IN ('crate_root', 'file_public_area') THEN TRUE
        WHEN endpoint_category_kind = 'local_item'
            AND item_kind_value = 'use_statement'
            AND owner_kind_value IN ('crate_root', 'module', 'trait', 'impl')
            AND function_body_ancestry_flag = FALSE
            THEN TRUE
        WHEN endpoint_category_kind = 'local_item'
            AND item_kind_value IN (
                'module', 'function', 'struct', 'enum', 'trait', 'type_alias',
                'constant', 'static', 'associated_function', 'associated_constant',
                'associated_type'
            )
            AND owner_kind_value IN ('crate_root', 'module', 'trait', 'impl')
            AND function_body_ancestry_flag = FALSE
            AND effective_visibility_kind IN ('public', 'restricted_non_private')
            THEN TRUE
        ELSE FALSE
    END AS node_selected_flag,
    endpoint_category_kind IN ('external_definition', 'builtin') AS terminal_target_flag
FROM "hld-analysis-node-inputs" AS input;
"#;

const HLD_RECURSIVE_PATH_SQL: &str = r#"
CREATE OR REPLACE TEMP TABLE "hld-analysis-path-records" AS
WITH RECURSIVE path_walks (
    run_generation_key,
    source_identity_key,
    current_endpoint_identity_key,
    relation_kind_value,
    ordered_endpoint_identity_path,
    ordered_evidence_identity_path
) AS (
    SELECT
        edge.run_generation_key,
        source.endpoint_identity_key AS source_identity_key,
        target.endpoint_identity_key AS current_endpoint_identity_key,
        edge.relation_kind_value,
        [source.endpoint_identity_key, target.endpoint_identity_key]
            AS ordered_endpoint_identity_path,
        [edge.evidence_identity_key] AS ordered_evidence_identity_path
    FROM "hld-analysis-edge-records" AS edge
    JOIN "hld-analysis-node-records" AS source
        ON source.run_generation_key = edge.run_generation_key
        AND source.endpoint_identity_key = edge.source_identity_key
    JOIN "hld-analysis-node-records" AS target
        ON target.run_generation_key = edge.run_generation_key
        AND target.endpoint_identity_key = edge.target_identity_key
    WHERE edge.resolution_status_kind = 'extracted'
        AND source.node_selected_flag
    UNION ALL
    SELECT
        path.run_generation_key,
        path.source_identity_key,
        target.endpoint_identity_key AS current_endpoint_identity_key,
        edge.relation_kind_value,
        list_append(path.ordered_endpoint_identity_path, target.endpoint_identity_key)
            AS ordered_endpoint_identity_path,
        list_append(path.ordered_evidence_identity_path, edge.evidence_identity_key)
            AS ordered_evidence_identity_path
    FROM path_walks AS path
    JOIN "hld-analysis-node-records" AS current
        ON current.run_generation_key = path.run_generation_key
        AND current.endpoint_identity_key = path.current_endpoint_identity_key
    JOIN "hld-analysis-edge-records" AS edge
        ON edge.run_generation_key = path.run_generation_key
        AND edge.source_identity_key = path.current_endpoint_identity_key
    JOIN "hld-analysis-node-records" AS target
        ON target.run_generation_key = edge.run_generation_key
        AND target.endpoint_identity_key = edge.target_identity_key
    WHERE edge.resolution_status_kind = 'extracted'
        AND path.relation_kind_value NOT IN ('contains', 'imports')
        AND edge.relation_kind_value NOT IN ('contains', 'imports')
        AND current.endpoint_category_kind = 'local_item'
        AND NOT current.node_selected_flag
        AND (
            target.node_selected_flag
            OR target.terminal_target_flag
            OR NOT list_contains(
                path.ordered_endpoint_identity_path,
                target.endpoint_identity_key
            )
        )
)
SELECT DISTINCT
    path.run_generation_key,
    path.source_identity_key,
    target.endpoint_identity_key AS target_identity_key,
    path.relation_kind_value,
    CASE
        WHEN array_length(path.ordered_endpoint_identity_path) = 2 THEN 'direct'
        ELSE 'collapsed'
    END AS collapse_via_kind,
    path.ordered_endpoint_identity_path,
    path.ordered_evidence_identity_path
FROM path_walks AS path
JOIN "hld-analysis-node-records" AS target
    ON target.run_generation_key = path.run_generation_key
    AND target.endpoint_identity_key = path.current_endpoint_identity_key
WHERE target.node_selected_flag OR target.terminal_target_flag;
"#;

impl HldGraphAnalysisPort for DuckdbAnalysisStoreAdapter {
    fn materialize_hld_graph_output(
        &self,
        input_record: &HldMaterializationInputRecord,
        cancel_flag: &RunCancelFlagValue,
    ) -> Result<HldMaterializationOutputRecord, HldAnalysisPortError> {
        check_hld_analysis_cancel(cancel_flag)?;
        validate_hld_materialization_input(input_record)?;
        let mut connection = self
            ._connection
            .lock()
            .map_err(|_| HldAnalysisPortError::LockPoisoned)?;
        check_hld_analysis_cancel(cancel_flag)?;
        let transaction = connection.transaction().map_err(|source| {
            build_hld_backend_error(HldAnalysisOperationKind::PrepareTransaction, source)
        })?;
        transaction
            .execute_batch(include_str!("hld-materialization-direct-projection.sql"))
            .map_err(|source| {
                build_hld_backend_error(HldAnalysisOperationKind::CreateWorkingRelations, source)
            })?;
        insert_hld_unit_origins(&transaction, input_record)?;
        insert_hld_node_inputs(&transaction, input_record)?;
        transaction
            .execute_batch(HLD_NORMALIZE_NODE_SQL)
            .map_err(|source| {
                build_hld_backend_error(HldAnalysisOperationKind::NormalizeNodeInputs, source)
            })?;
        let rollup_bucket_indexes = insert_hld_rollup_buckets(&transaction, input_record)?;
        insert_hld_edge_inputs(&transaction, input_record)?;
        create_hld_recursive_paths(&transaction)?;
        let mut output_record =
            read_hld_materialization_output(&transaction, input_record, &rollup_bucket_indexes)?;
        check_hld_analysis_cancel(cancel_flag)?;
        transaction.commit().map_err(|source| {
            build_hld_backend_error(HldAnalysisOperationKind::CommitTransaction, source)
        })?;
        check_hld_analysis_cancel(cancel_flag)?;
        output_record.selected_node_records.sort();
        output_record.materialized_edge_records.sort();
        output_record.evidence_path_records.sort();
        output_record
            .file_rollup_records
            .as_mut()
            .into_iter()
            .for_each(|records| records.sort());
        output_record
            .folder_rollup_records
            .as_mut()
            .into_iter()
            .for_each(|records| records.sort());
        Ok(output_record)
    }
}

fn create_hld_recursive_paths(
    transaction: &duckdb::Transaction<'_>,
) -> Result<(), HldAnalysisPortError> {
    transaction
        .execute_batch(HLD_RECURSIVE_PATH_SQL)
        .map_err(|source| {
            build_hld_backend_error(HldAnalysisOperationKind::ProjectRecursivePaths, source)
        })
}

fn check_hld_analysis_cancel(cancel_flag: &RunCancelFlagValue) -> Result<(), HldAnalysisPortError> {
    if cancel_flag.check_stage_work_cancelled() {
        Err(HldAnalysisPortError::Cancelled)
    } else {
        Ok(())
    }
}

struct HldRollupBucketIndexes {
    file_buckets_by_identity: BTreeMap<String, HldEndpointRollupBucket>,
    folder_buckets_by_identity: BTreeMap<String, HldFolderEndpointBucket>,
}

fn build_hld_rollup_buckets(
    node: &HldNodeInputRecord,
) -> Result<(HldEndpointRollupBucket, HldFolderEndpointBucket), HldAnalysisPortError> {
    match node.node_input_category {
        HldNodeInputCategory::FilePublicArea => {
            let unit = read_hld_target_unit(&node.node_target_reference).ok_or_else(|| {
                invalid_hld_rollup_input(HldMaterializationValidationError::InvalidNodeCategory)
            })?;
            let path = node.node_source_relative_path.as_deref().ok_or_else(|| {
                invalid_hld_rollup_input(
                    HldMaterializationValidationError::InvalidSourceRelativePath,
                )
            })?;
            source_hld_rollup_buckets(unit, path)
        }
        HldNodeInputCategory::LocalItem => {
            let unit = read_hld_target_unit(&node.node_target_reference).ok_or_else(|| {
                invalid_hld_rollup_input(HldMaterializationValidationError::InvalidNodeCategory)
            })?;
            let path = node.node_source_relative_path.as_deref().ok_or_else(|| {
                invalid_hld_rollup_input(
                    HldMaterializationValidationError::InvalidSourceRelativePath,
                )
            })?;
            source_hld_rollup_buckets(unit, path)
        }
        HldNodeInputCategory::ExternalDefinition => match node.node_source_relative_path.as_deref()
        {
            Some(path) => {
                let unit = read_hld_target_unit(&node.node_target_reference).ok_or_else(|| {
                    invalid_hld_rollup_input(HldMaterializationValidationError::InvalidNodeCategory)
                })?;
                source_hld_rollup_buckets(unit, path)
            }
            None => Ok(canonical_hld_rollup_buckets(
                node.node_target_reference.clone(),
            )),
        },
        HldNodeInputCategory::CrateRoot => match &node.node_target_reference {
            HldTargetIdentityValue::Symbol(NameSymbolTargetKey::CrateRoot(unit)) => Ok((
                HldEndpointRollupBucket::CrateRoot {
                    unit_compilation_identity_key: unit.clone(),
                },
                HldFolderEndpointBucket::CrateRoot {
                    unit_compilation_identity_key: unit.clone(),
                },
            )),
            _ => Err(invalid_hld_rollup_input(
                HldMaterializationValidationError::InvalidNodeCategory,
            )),
        },
        HldNodeInputCategory::Builtin => match &node.node_target_reference {
            HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Builtin(_)) => Ok(
                canonical_hld_rollup_buckets(node.node_target_reference.clone()),
            ),
            _ => Err(invalid_hld_rollup_input(
                HldMaterializationValidationError::InvalidNodeCategory,
            )),
        },
    }
}

fn source_hld_rollup_buckets(
    unit_identity: CrateCompilationUnitKey,
    source_relative_path: &str,
) -> Result<(HldEndpointRollupBucket, HldFolderEndpointBucket), HldAnalysisPortError> {
    let normalized_path = normalize_hld_source_path(source_relative_path)?;
    let folder_path = parent_hld_source_folder(&normalized_path);
    Ok((
        HldEndpointRollupBucket::SourceFile {
            unit_compilation_identity_key: unit_identity.clone(),
            file_source_relative_path: normalized_path,
        },
        HldFolderEndpointBucket::SourceFolder {
            unit_compilation_identity_key: unit_identity,
            folder_source_relative_path: folder_path,
        },
    ))
}

fn canonical_hld_rollup_buckets(
    target_reference: HldTargetIdentityValue,
) -> (HldEndpointRollupBucket, HldFolderEndpointBucket) {
    (
        HldEndpointRollupBucket::CanonicalExternalIdentity {
            endpoint_target_reference: target_reference.clone(),
        },
        HldFolderEndpointBucket::CanonicalExternalIdentity {
            endpoint_target_reference: target_reference,
        },
    )
}

fn normalize_hld_source_path(path: &str) -> Result<String, HldAnalysisPortError> {
    let components = path
        .split('/')
        .filter(|component| !component.is_empty() && *component != ".")
        .collect::<Vec<_>>();
    if path.starts_with('/')
        || path.contains('\\')
        || components.is_empty()
        || components.contains(&"..")
    {
        return Err(invalid_hld_rollup_input(
            HldMaterializationValidationError::InvalidSourceRelativePath,
        ));
    }
    Ok(components.join("/"))
}

fn parent_hld_source_folder(path: &str) -> String {
    path.rsplit_once('/')
        .map(|(parent, _)| parent)
        .filter(|parent| !parent.is_empty())
        .unwrap_or(".")
        .to_owned()
}

fn encode_hld_file_bucket(bucket: &HldEndpointRollupBucket) -> Result<String, serde_json::Error> {
    let encoded = match bucket {
        HldEndpointRollupBucket::SourceFile {
            unit_compilation_identity_key,
            file_source_relative_path,
        } => json!([
            "source_file",
            encode_hld_unit_value(unit_compilation_identity_key),
            file_source_relative_path
        ]),
        HldEndpointRollupBucket::CrateRoot {
            unit_compilation_identity_key,
        } => json!([
            "crate_root",
            encode_hld_unit_value(unit_compilation_identity_key)
        ]),
        HldEndpointRollupBucket::CanonicalExternalIdentity {
            endpoint_target_reference,
        } => json!([
            "canonical_external",
            encode_hld_target_value(endpoint_target_reference)
        ]),
    };
    serde_json::to_string(&encoded)
}

fn encode_hld_folder_bucket(bucket: &HldFolderEndpointBucket) -> Result<String, serde_json::Error> {
    let encoded = match bucket {
        HldFolderEndpointBucket::SourceFolder {
            unit_compilation_identity_key,
            folder_source_relative_path,
        } => json!([
            "source_folder",
            encode_hld_unit_value(unit_compilation_identity_key),
            folder_source_relative_path
        ]),
        HldFolderEndpointBucket::CrateRoot {
            unit_compilation_identity_key,
        } => json!([
            "crate_root",
            encode_hld_unit_value(unit_compilation_identity_key)
        ]),
        HldFolderEndpointBucket::CanonicalExternalIdentity {
            endpoint_target_reference,
        } => json!([
            "canonical_external",
            encode_hld_target_value(endpoint_target_reference)
        ]),
    };
    serde_json::to_string(&encoded)
}

fn insert_hld_rollup_buckets(
    transaction: &duckdb::Transaction<'_>,
    input_record: &HldMaterializationInputRecord,
) -> Result<HldRollupBucketIndexes, HldAnalysisPortError> {
    let unique_nodes = input_record
        .input_node_records
        .iter()
        .map(|node| {
            encode_hld_target_identity(&node.node_target_reference)
                .map(|identity| (identity, node))
                .map_err(|source| {
                    build_hld_backend_error(HldAnalysisOperationKind::NormalizeNodeInputs, source)
                })
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let mut indexes = HldRollupBucketIndexes {
        file_buckets_by_identity: BTreeMap::new(),
        folder_buckets_by_identity: BTreeMap::new(),
    };
    unique_nodes
        .iter()
        .try_for_each(|(endpoint_identity, node)| {
            let (file_bucket, folder_bucket) = build_hld_rollup_buckets(node)?;
            let file_identity = encode_hld_file_bucket(&file_bucket).map_err(|source| {
                build_hld_backend_error(HldAnalysisOperationKind::NormalizeNodeInputs, source)
            })?;
            let folder_identity = encode_hld_folder_bucket(&folder_bucket).map_err(|source| {
                build_hld_backend_error(HldAnalysisOperationKind::NormalizeNodeInputs, source)
            })?;
            transaction
                .execute(
                    "INSERT INTO \"hld-analysis-rollup-buckets\" VALUES (?, ?, ?)",
                    duckdb::params![endpoint_identity, file_identity, folder_identity],
                )
                .map(|_| ())
                .map_err(|source| {
                    build_hld_backend_error(HldAnalysisOperationKind::NormalizeNodeInputs, source)
                })?;
            indexes
                .file_buckets_by_identity
                .insert(file_identity, file_bucket);
            indexes
                .folder_buckets_by_identity
                .insert(folder_identity, folder_bucket);
            Ok::<(), HldAnalysisPortError>(())
        })?;
    Ok(indexes)
}

fn invalid_hld_rollup_input(
    validation_error: HldMaterializationValidationError,
) -> HldAnalysisPortError {
    HldAnalysisPortError::InvalidInput(validation_error)
}

fn build_hld_backend_error(
    operation: HldAnalysisOperationKind,
    source: impl std::error::Error + Send + Sync + 'static,
) -> HldAnalysisPortError {
    HldAnalysisPortError::Backend {
        operation,
        source: Box::new(source),
    }
}

fn insert_hld_unit_origins(
    transaction: &duckdb::Transaction<'_>,
    input_record: &HldMaterializationInputRecord,
) -> Result<(), HldAnalysisPortError> {
    let unique_origins = input_record
        .input_unit_provenance_records
        .iter()
        .map(|record| {
            encode_hld_unit_identity(&record.unit_compilation_identity_key)
                .map(|identity| (identity, record.unit_origin_kind))
                .map_err(|source| {
                    build_hld_backend_error(HldAnalysisOperationKind::NormalizeNodeInputs, source)
                })
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    unique_origins.iter().try_for_each(|(identity, origin)| {
        transaction
            .execute(
                "INSERT INTO \"hld-analysis-unit-origins\" VALUES (?, ?)",
                duckdb::params![identity, encode_hld_unit_origin(*origin)],
            )
            .map(|_| ())
            .map_err(|source| {
                build_hld_backend_error(HldAnalysisOperationKind::NormalizeNodeInputs, source)
            })
    })
}

fn insert_hld_node_inputs(
    transaction: &duckdb::Transaction<'_>,
    input_record: &HldMaterializationInputRecord,
) -> Result<(), HldAnalysisPortError> {
    let unique_nodes = input_record
        .input_node_records
        .iter()
        .map(|node| {
            encode_hld_target_identity(&node.node_target_reference)
                .map(|identity| (identity, node))
                .map_err(|source| {
                    build_hld_backend_error(HldAnalysisOperationKind::NormalizeNodeInputs, source)
                })
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    unique_nodes.iter().try_for_each(|(identity, node)| {
        let unit_identity = read_hld_target_unit(&node.node_target_reference)
            .map(|unit_identity| encode_hld_unit_identity(&unit_identity))
            .transpose()
            .map_err(|source| {
                build_hld_backend_error(
                    HldAnalysisOperationKind::NormalizeNodeInputs,
                    source,
                )
            })?;
        transaction
            .execute(
                "INSERT INTO \"hld-analysis-node-inputs\" VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                duckdb::params![
                    encode_hld_hex_digest(&input_record.run_generation_digest_value.0),
                    identity,
                    encode_hld_node_category(node.node_input_category),
                    unit_identity,
                    node.node_item_kind_value.map(encode_hld_item_kind),
                    node.node_owner_kind_value.map(encode_hld_owner_kind),
                    node.node_function_body_ancestry_flag,
                    node.node_effective_visibility_kind.map(encode_hld_visibility_kind),
                    node.node_reachable_outside_crate_flag,
                    node.node_source_relative_path,
                    node.node_qualified_path_text,
                    node.node_signature_text
                ],
            )
            .map(|_| ())
            .map_err(|source| {
                build_hld_backend_error(
                    HldAnalysisOperationKind::NormalizeNodeInputs,
                    source,
                )
            })
    })
}

fn insert_hld_edge_inputs(
    transaction: &duckdb::Transaction<'_>,
    input_record: &HldMaterializationInputRecord,
) -> Result<(), HldAnalysisPortError> {
    let unique_edges = input_record
        .input_edge_records
        .iter()
        .map(|edge| {
            encode_hld_evidence_identity(&edge.edge_evidence_row_key)
                .map(|identity| (identity, edge))
                .map_err(|source| {
                    build_hld_backend_error(HldAnalysisOperationKind::NormalizeEdgeInputs, source)
                })
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    unique_edges
        .iter()
        .try_for_each(|(evidence_identity, edge)| {
            let source_identity =
                encode_hld_target_identity(&edge.edge_source_reference).map_err(|source| {
                    build_hld_backend_error(HldAnalysisOperationKind::NormalizeEdgeInputs, source)
                })?;
            let source_unit_identity = read_hld_target_unit(&edge.edge_source_reference)
                .map(|unit_identity| encode_hld_unit_identity(&unit_identity))
                .transpose()
                .map_err(|source| {
                    build_hld_backend_error(HldAnalysisOperationKind::NormalizeEdgeInputs, source)
                })?;
            let (resolution_status, target_reference) = match &edge.edge_resolution_value {
                HldEdgeResolutionValue::Extracted { target } => ("extracted", Some(target)),
                HldEdgeResolutionValue::Unresolved { .. } => ("unresolved", None),
                HldEdgeResolutionValue::Inferred { candidate_target } => {
                    ("inferred", candidate_target.as_ref())
                }
                HldEdgeResolutionValue::Proposed { candidate_target } => {
                    ("proposed", candidate_target.as_ref())
                }
            };
            let target_identity = target_reference
                .map(encode_hld_target_identity)
                .transpose()
                .map_err(|source| {
                    build_hld_backend_error(HldAnalysisOperationKind::NormalizeEdgeInputs, source)
                })?;
            let target_unit_identity = target_reference
                .and_then(read_hld_target_unit)
                .map(|unit_identity| encode_hld_unit_identity(&unit_identity))
                .transpose()
                .map_err(|source| {
                    build_hld_backend_error(HldAnalysisOperationKind::NormalizeEdgeInputs, source)
                })?;
            let site_identity = serde_json::to_string(&encode_hld_occurrence_value(
                &edge.edge_evidence_row_key.evidence_site_occurrence_key,
            ))
            .map_err(|source| {
                build_hld_backend_error(HldAnalysisOperationKind::NormalizeEdgeInputs, source)
            })?;
            transaction
            .execute(
                "INSERT INTO \"hld-analysis-edge-records\" VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                duckdb::params![
                    encode_hld_hex_digest(&input_record.run_generation_digest_value.0),
                    evidence_identity,
                    site_identity,
                    source_identity,
                    target_identity,
                    encode_hld_relation_kind(edge.edge_relation_kind_value),
                    resolution_status,
                    edge.edge_dispatch_metadata_text,
                    source_unit_identity,
                    target_unit_identity
                ],
            )
            .map(|_| ())
            .map_err(|source| {
                build_hld_backend_error(
                    HldAnalysisOperationKind::NormalizeEdgeInputs,
                    source,
                )
            })
        })
}

fn read_hld_materialization_output(
    transaction: &duckdb::Transaction<'_>,
    input_record: &HldMaterializationInputRecord,
    rollup_bucket_indexes: &HldRollupBucketIndexes,
) -> Result<HldMaterializationOutputRecord, HldAnalysisPortError> {
    let node_by_identity = input_record
        .input_node_records
        .iter()
        .map(|node| {
            encode_hld_target_identity(&node.node_target_reference)
                .map(|identity| (identity, node))
                .map_err(|source| {
                    build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
                })
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let mut selected_statement = transaction
        .prepare("SELECT endpoint_identity_key FROM \"hld-analysis-node-records\" WHERE node_selected_flag ORDER BY endpoint_identity_key")
        .map_err(|source| {
            build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
        })?;
    let mut selected_node_records = selected_statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|source| {
            build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
        })?
        .map(|row_result| {
            row_result
                .map_err(|source| {
                    build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
                })
                .and_then(|identity| {
                    node_by_identity
                        .get(&identity)
                        .map(|node| HldSelectedNodeRecord {
                            run_generation_digest_value: input_record.run_generation_digest_value,
                            node_target_reference: node.node_target_reference.clone(),
                            node_qualified_path_text: node.node_qualified_path_text.clone(),
                            node_source_relative_path: node.node_source_relative_path.clone(),
                            node_effective_visibility_kind: node.node_effective_visibility_kind,
                            node_reachable_outside_crate_flag: node
                                .node_reachable_outside_crate_flag,
                            node_signature_text: node.node_signature_text.clone(),
                        })
                        .ok_or_else(|| {
                            build_hld_backend_error(
                                HldAnalysisOperationKind::ReadAnalysisOutputs,
                                std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    "selected endpoint identity is absent from validated input",
                                ),
                            )
                        })
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let edge_query = r#"
WITH path_evidence AS (
    SELECT
        path.run_generation_key,
        path.source_identity_key,
        path.target_identity_key,
        path.relation_kind_value,
        path.collapse_via_kind,
        evidence.evidence_identity_key
    FROM "hld-analysis-path-records" AS path
    CROSS JOIN UNNEST(path.ordered_evidence_identity_path)
        AS evidence(evidence_identity_key)
),
edge_evidence_counts AS (
    SELECT
        run_generation_key,
        source_identity_key,
        target_identity_key,
        relation_kind_value,
        collapse_via_kind,
        COUNT(DISTINCT evidence_identity_key) AS evidence_row_count
    FROM path_evidence
    GROUP BY
        run_generation_key,
        source_identity_key,
        target_identity_key,
        relation_kind_value,
        collapse_via_kind
)
SELECT
    edge.run_generation_key,
    edge.source_identity_key,
    edge.target_identity_key,
    edge.relation_kind_value,
    edge.collapse_via_kind,
    source.qualified_path_text AS source_qualified_path_text,
    target.qualified_path_text AS target_qualified_path_text,
    source.signature_text AS source_signature_text,
    target.signature_text AS target_signature_text,
    CASE
        WHEN source.unit_identity_key IS NOT NULL
            AND source.unit_identity_key = target.unit_identity_key
            THEN 'same_crate'
        WHEN target.endpoint_category_kind = 'builtin' THEN 'std'
        WHEN target_origin.unit_origin_kind = 'workspace' THEN 'workspace_crate'
        WHEN target_origin.unit_origin_kind = 'standard_library' THEN 'std'
        ELSE 'external_crate'
    END AS dependency_origin_kind,
    edge.evidence_row_count
FROM edge_evidence_counts AS edge
JOIN "hld-analysis-node-records" AS source
    ON source.run_generation_key = edge.run_generation_key
    AND source.endpoint_identity_key = edge.source_identity_key
JOIN "hld-analysis-node-records" AS target
    ON target.run_generation_key = edge.run_generation_key
    AND target.endpoint_identity_key = edge.target_identity_key
LEFT JOIN "hld-analysis-unit-origins" AS target_origin
    ON target_origin.unit_identity_key = target.unit_identity_key
ORDER BY
    edge.run_generation_key,
    edge.source_identity_key,
    edge.target_identity_key,
    edge.relation_kind_value,
    edge.collapse_via_kind
"#;
    let materialize_edge_sql =
        format!("CREATE TEMP TABLE \"hld-analysis-materialized-edges\" AS {edge_query}");
    transaction
        .execute_batch(&materialize_edge_sql)
        .map_err(|source| {
            build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
        })?;
    let mut statement = transaction
        .prepare("SELECT run_generation_key, source_identity_key, target_identity_key, relation_kind_value, collapse_via_kind, source_qualified_path_text, target_qualified_path_text, source_signature_text, target_signature_text, dependency_origin_kind, evidence_row_count FROM \"hld-analysis-materialized-edges\" ORDER BY run_generation_key, source_identity_key, target_identity_key, relation_kind_value, collapse_via_kind")
        .map_err(|source| {
        build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
    })?;
    let mut materialized_edge_records = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, usize>(10)?,
            ))
        })
        .map_err(|source| {
            build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
        })?
        .map(|row_result| {
            let (
                _generation,
                source_identity,
                target_identity,
                relation_text,
                via_text,
                source_path,
                target_path,
                source_signature,
                target_signature,
                origin_text,
                evidence_count,
            ) = row_result.map_err(|source| {
                build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
            })?;
            let source_target = read_hld_identity_target(&node_by_identity, &source_identity)?;
            let target_reference = read_hld_identity_target(&node_by_identity, &target_identity)?;
            Ok(HldMaterializedEdgeRecord {
                edge_key: HldGraphEdgeKey {
                    run_generation_digest_value: input_record.run_generation_digest_value,
                    edge_source_occurrence_key: source_target,
                    edge_target_reference_value: target_reference,
                    edge_relation_kind_value: read_hld_relation_kind(&relation_text)?,
                    edge_collapse_via_kind: read_hld_via_kind(&via_text)?,
                },
                edge_source_qualified_path: source_path,
                edge_target_qualified_path: target_path,
                edge_source_signature_text: source_signature,
                edge_target_signature_text: target_signature,
                edge_resolution_status_kind: HldOutputEdgeStatus::Extracted,
                edge_evidence_row_count: evidence_count,
                edge_dependency_origin_kind: read_hld_dependency_origin(&origin_text)?,
            })
        })
        .collect::<Result<Vec<_>, HldAnalysisPortError>>()?;

    let evidence_by_identity = input_record
        .input_edge_records
        .iter()
        .map(|edge| {
            encode_hld_evidence_identity(&edge.edge_evidence_row_key)
                .map(|identity| (identity, edge.edge_evidence_row_key.clone()))
                .map_err(|source| {
                    build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
                })
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let mut path_statement = transaction
        .prepare("SELECT source_identity_key, target_identity_key, relation_kind_value, collapse_via_kind, to_json(ordered_endpoint_identity_path), to_json(ordered_evidence_identity_path) FROM \"hld-analysis-path-records\"")
        .map_err(|source| {
            build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
        })?;
    let mut evidence_path_records = path_statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .map_err(|source| {
            build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
        })?
        .map(|row_result| {
            let (
                source_identity,
                target_identity,
                relation_text,
                via_text,
                endpoint_path_json,
                evidence_path_json,
            ) = row_result.map_err(|source| {
                build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
            })?;
            let source_target = read_hld_identity_target(&node_by_identity, &source_identity)?;
            let target_reference = read_hld_identity_target(&node_by_identity, &target_identity)?;
            let endpoint_identity_path = serde_json::from_str::<Vec<String>>(&endpoint_path_json)
                .map_err(|source| {
                build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
            })?;
            let ordered_endpoint_path = endpoint_identity_path
                .iter()
                .map(|identity| read_hld_identity_target(&node_by_identity, identity))
                .collect::<Result<Vec<_>, _>>()?;
            let evidence_identity_path = serde_json::from_str::<Vec<String>>(&evidence_path_json)
                .map_err(|source| {
                build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
            })?;
            let ordered_evidence_row_keys = evidence_identity_path
                .iter()
                .map(|identity| {
                    evidence_by_identity.get(identity).cloned().ok_or_else(|| {
                        HldAnalysisPortError::InvalidInput(
                            HldMaterializationValidationError::InvalidEvidenceIdentity,
                        )
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(HldEvidencePathRecord {
                edge_key: HldGraphEdgeKey {
                    run_generation_digest_value: input_record.run_generation_digest_value,
                    edge_source_occurrence_key: source_target.clone(),
                    edge_target_reference_value: target_reference.clone(),
                    edge_relation_kind_value: read_hld_relation_kind(&relation_text)?,
                    edge_collapse_via_kind: read_hld_via_kind(&via_text)?,
                },
                ordered_endpoint_path,
                ordered_site_occurrence_keys: ordered_evidence_row_keys
                    .iter()
                    .map(|evidence| evidence.evidence_site_occurrence_key.clone())
                    .collect(),
                ordered_evidence_row_keys,
            })
        })
        .collect::<Result<Vec<_>, HldAnalysisPortError>>()?;

    let file_rollup_records = read_hld_file_rollups(transaction, rollup_bucket_indexes)?;
    let folder_rollup_records = read_hld_folder_rollups(transaction, rollup_bucket_indexes)?;
    selected_node_records.sort();
    materialized_edge_records.sort();
    evidence_path_records.sort();
    Ok(HldMaterializationOutputRecord {
        run_generation_digest_value: input_record.run_generation_digest_value,
        output_completeness_kind: HldOutputCompletenessKind::CompleteGraph,
        source_coverage_kind: input_record.source_coverage_kind,
        selected_node_records,
        materialized_edge_records,
        evidence_path_records,
        file_rollup_records: Some(file_rollup_records),
        folder_rollup_records: Some(folder_rollup_records),
    })
}

fn read_hld_file_rollups(
    transaction: &duckdb::Transaction<'_>,
    indexes: &HldRollupBucketIndexes,
) -> Result<Vec<HldFileRollupRecord>, HldAnalysisPortError> {
    let query = r#"
SELECT
    source_bucket.file_bucket_identity,
    target_bucket.file_bucket_identity,
    edge.relation_kind_value,
    edge.collapse_via_kind,
    COUNT(*) AS edge_row_count
FROM "hld-analysis-materialized-edges" AS edge
JOIN "hld-analysis-rollup-buckets" AS source_bucket
    ON source_bucket.endpoint_identity_key = edge.source_identity_key
JOIN "hld-analysis-rollup-buckets" AS target_bucket
    ON target_bucket.endpoint_identity_key = edge.target_identity_key
GROUP BY
    edge.run_generation_key,
    source_bucket.file_bucket_identity,
    target_bucket.file_bucket_identity,
    edge.relation_kind_value,
    edge.collapse_via_kind
ORDER BY
    source_bucket.file_bucket_identity,
    target_bucket.file_bucket_identity,
    edge.relation_kind_value,
    edge.collapse_via_kind
"#;
    let mut statement = transaction.prepare(query).map_err(|source| {
        build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
    })?;
    statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, usize>(4)?,
            ))
        })
        .map_err(|source| {
            build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
        })?
        .map(|row_result| {
            let (source_identity, target_identity, relation, via, edge_count) = row_result
                .map_err(|source| {
                    build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
                })?;
            Ok(HldFileRollupRecord {
                source_endpoint_bucket: read_hld_file_bucket(indexes, &source_identity)?,
                target_endpoint_bucket: read_hld_file_bucket(indexes, &target_identity)?,
                edge_relation_kind_value: read_hld_relation_kind(&relation)?,
                edge_collapse_via_kind: read_hld_via_kind(&via)?,
                edge_row_count: edge_count,
            })
        })
        .collect()
}

fn read_hld_folder_rollups(
    transaction: &duckdb::Transaction<'_>,
    indexes: &HldRollupBucketIndexes,
) -> Result<Vec<HldFolderRollupRecord>, HldAnalysisPortError> {
    let query = r#"
SELECT
    source_bucket.folder_bucket_identity,
    target_bucket.folder_bucket_identity,
    edge.relation_kind_value,
    edge.collapse_via_kind,
    COUNT(*) AS edge_row_count
FROM "hld-analysis-materialized-edges" AS edge
JOIN "hld-analysis-rollup-buckets" AS source_bucket
    ON source_bucket.endpoint_identity_key = edge.source_identity_key
JOIN "hld-analysis-rollup-buckets" AS target_bucket
    ON target_bucket.endpoint_identity_key = edge.target_identity_key
GROUP BY
    edge.run_generation_key,
    source_bucket.folder_bucket_identity,
    target_bucket.folder_bucket_identity,
    edge.relation_kind_value,
    edge.collapse_via_kind
ORDER BY
    source_bucket.folder_bucket_identity,
    target_bucket.folder_bucket_identity,
    edge.relation_kind_value,
    edge.collapse_via_kind
"#;
    let mut statement = transaction.prepare(query).map_err(|source| {
        build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
    })?;
    statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, usize>(4)?,
            ))
        })
        .map_err(|source| {
            build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
        })?
        .map(|row_result| {
            let (source_identity, target_identity, relation, via, edge_count) = row_result
                .map_err(|source| {
                    build_hld_backend_error(HldAnalysisOperationKind::ReadAnalysisOutputs, source)
                })?;
            Ok(HldFolderRollupRecord {
                source_folder_bucket: read_hld_folder_bucket(indexes, &source_identity)?,
                target_folder_bucket: read_hld_folder_bucket(indexes, &target_identity)?,
                edge_relation_kind_value: read_hld_relation_kind(&relation)?,
                edge_collapse_via_kind: read_hld_via_kind(&via)?,
                edge_row_count: edge_count,
            })
        })
        .collect()
}

fn read_hld_file_bucket(
    indexes: &HldRollupBucketIndexes,
    identity: &str,
) -> Result<HldEndpointRollupBucket, HldAnalysisPortError> {
    indexes
        .file_buckets_by_identity
        .get(identity)
        .cloned()
        .ok_or_else(|| {
            invalid_hld_rollup_input(HldMaterializationValidationError::InvalidNodeCategory)
        })
}

fn read_hld_folder_bucket(
    indexes: &HldRollupBucketIndexes,
    identity: &str,
) -> Result<HldFolderEndpointBucket, HldAnalysisPortError> {
    indexes
        .folder_buckets_by_identity
        .get(identity)
        .cloned()
        .ok_or_else(|| {
            invalid_hld_rollup_input(HldMaterializationValidationError::InvalidNodeCategory)
        })
}

fn read_hld_identity_target(
    node_by_identity: &BTreeMap<String, &HldNodeInputRecord>,
    identity: &str,
) -> Result<HldTargetIdentityValue, HldAnalysisPortError> {
    node_by_identity
        .get(identity)
        .map(|node| node.node_target_reference.clone())
        .ok_or_else(|| {
            HldAnalysisPortError::InvalidInput(
                HldMaterializationValidationError::MissingLocalEndpoint,
            )
        })
}

fn read_hld_relation_kind(
    relation_text: &str,
) -> Result<HldRelationKindValue, HldAnalysisPortError> {
    match relation_text {
        "calls" => Ok(HldRelationKindValue::Calls),
        "uses_type" => Ok(HldRelationKindValue::UsesType),
        "imports" => Ok(HldRelationKindValue::Imports),
        "reads" => Ok(HldRelationKindValue::Reads),
        "implements" => Ok(HldRelationKindValue::Implements),
        "reexports" => Ok(HldRelationKindValue::Reexports),
        "contains" => Ok(HldRelationKindValue::Contains),
        "derives" => Ok(HldRelationKindValue::Derives),
        "generates" => Ok(HldRelationKindValue::Generates),
        "expands" => Ok(HldRelationKindValue::Expands),
        _ => Err(read_hld_output_error("unknown relation kind")),
    }
}

fn read_hld_via_kind(via_text: &str) -> Result<HldViaKindValue, HldAnalysisPortError> {
    match via_text {
        "direct" => Ok(HldViaKindValue::Direct),
        "collapsed" => Ok(HldViaKindValue::Collapsed),
        _ => Err(read_hld_output_error("unknown collapse-via kind")),
    }
}

fn read_hld_dependency_origin(
    origin_text: &str,
) -> Result<HldDependencyOriginKind, HldAnalysisPortError> {
    match origin_text {
        "same_crate" => Ok(HldDependencyOriginKind::SameCrate),
        "workspace_crate" => Ok(HldDependencyOriginKind::WorkspaceCrate),
        "std" => Ok(HldDependencyOriginKind::StandardLibrary),
        "external_crate" => Ok(HldDependencyOriginKind::ExternalCrate),
        _ => Err(read_hld_output_error("unknown dependency origin")),
    }
}

fn read_hld_output_error(message: &'static str) -> HldAnalysisPortError {
    build_hld_backend_error(
        HldAnalysisOperationKind::ReadAnalysisOutputs,
        std::io::Error::new(std::io::ErrorKind::InvalidData, message),
    )
}

fn encode_hld_unit_identity(
    unit_identity: &CrateCompilationUnitKey,
) -> Result<String, serde_json::Error> {
    serde_json::to_string(&encode_hld_unit_value(unit_identity))
}

fn encode_hld_unit_value(unit_identity: &CrateCompilationUnitKey) -> JsonValue {
    json!([
        unit_identity.unit_cargo_package_identifier,
        unit_identity.unit_cargo_target_name,
        unit_identity.unit_cargo_target_kind,
        match unit_identity.unit_compilation_role_kind {
            parseltongue_core_row_types::CompilationUnitRoleKind::Host => "host",
            parseltongue_core_row_types::CompilationUnitRoleKind::Target => "target",
        },
        encode_hld_hex_digest(&unit_identity.unit_configuration_digest_full)
    ])
}

fn encode_hld_occurrence_value(occurrence: &SemanticOccurrenceKeyValue) -> JsonValue {
    let parts = occurrence.read_semantic_occurrence_parts();
    json!([
        encode_hld_unit_value(parts.unit_compilation_identity_key),
        parts.span_source_file_path,
        parts.span_byte_range_value.read_source_range_pair(),
        encode_hld_hex_digest(parts.span_content_digest_full.read_full_digest_bytes()),
        parts
            .span_expansion_provenance_chain
            .iter()
            .map(|digest| encode_hld_hex_digest(digest))
            .collect::<Vec<_>>()
    ])
}

fn encode_hld_target_value(target: &HldTargetIdentityValue) -> JsonValue {
    match target {
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(occurrence)) => {
            json!(["definition", encode_hld_occurrence_value(occurrence)])
        }
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::GenericParameter(
            occurrence,
            parameter_name,
        )) => {
            json!([
                "generic_parameter",
                encode_hld_occurrence_value(occurrence),
                parameter_name
            ])
        }
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Builtin(name)) => {
            json!(["builtin", name])
        }
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::CrateRoot(unit_identity)) => {
            json!(["crate_root", encode_hld_unit_value(unit_identity)])
        }
        HldTargetIdentityValue::ExternalNamed(external) => json!([
            "external_named",
            match external.external_name_kind_value {
                HldExternalNameKind::CrateRootedPath => "crate_rooted_path",
                HldExternalNameKind::PreludeName => "prelude_name",
                HldExternalNameKind::BuiltinMember => "builtin_member",
                HldExternalNameKind::CoreDeclaredMember => "core_declared_member",
            },
            external.external_crate_name_value,
            external.external_item_path_text
        ]),
        HldTargetIdentityValue::FilePublicArea(file_area) => json!([
            "file_public_area",
            encode_hld_unit_value(file_area.read_unit_identity_value()),
            file_area.read_source_relative_path()
        ]),
    }
}

pub(crate) fn encode_hld_target_identity(
    target: &HldTargetIdentityValue,
) -> Result<String, serde_json::Error> {
    serde_json::to_string(&encode_hld_target_value(target))
}

fn encode_hld_evidence_identity(
    evidence_key: &HldEvidenceRowKey,
) -> Result<String, serde_json::Error> {
    serde_json::to_string(&json!([
        encode_hld_occurrence_value(&evidence_key.evidence_site_occurrence_key),
        encode_hld_target_value(&evidence_key.evidence_source_occurrence_key),
        encode_hld_relation_kind(evidence_key.evidence_relation_kind_value),
        evidence_key
            .evidence_target_reference_value
            .as_ref()
            .map(encode_hld_target_value)
    ]))
}

fn encode_hld_hex_digest(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn read_hld_target_unit(target: &HldTargetIdentityValue) -> Option<CrateCompilationUnitKey> {
    match target {
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Definition(occurrence))
        | HldTargetIdentityValue::Symbol(NameSymbolTargetKey::GenericParameter(occurrence, _)) => {
            Some(
                occurrence
                    .read_semantic_occurrence_parts()
                    .unit_compilation_identity_key
                    .clone(),
            )
        }
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::CrateRoot(unit_identity)) => {
            Some(unit_identity.clone())
        }
        HldTargetIdentityValue::Symbol(NameSymbolTargetKey::Builtin(_))
        | HldTargetIdentityValue::ExternalNamed(_) => None,
        HldTargetIdentityValue::FilePublicArea(file_area) => {
            Some(file_area.read_unit_identity_value().clone())
        }
    }
}

fn encode_hld_node_category(category: HldNodeInputCategory) -> &'static str {
    match category {
        HldNodeInputCategory::LocalItem => "local_item",
        HldNodeInputCategory::FilePublicArea => "file_public_area",
        HldNodeInputCategory::CrateRoot => "crate_root",
        HldNodeInputCategory::ExternalDefinition => "external_definition",
        HldNodeInputCategory::Builtin => "builtin",
    }
}

fn encode_hld_item_kind(kind: parseltongue_core_row_types::HldItemKindValue) -> &'static str {
    use parseltongue_core_row_types::HldItemKindValue as Kind;
    match kind {
        Kind::Module => "module",
        Kind::FilePublicArea => "file_public_area",
        Kind::Function => "function",
        Kind::Struct => "struct",
        Kind::Enum => "enum",
        Kind::Trait => "trait",
        Kind::TypeAlias => "type_alias",
        Kind::Constant => "constant",
        Kind::Static => "static",
        Kind::AssociatedFunction => "associated_function",
        Kind::AssociatedConstant => "associated_constant",
        Kind::AssociatedType => "associated_type",
        Kind::EnumVariant => "enum_variant",
        Kind::ImplContainer => "impl_container",
        Kind::UseStatement => "use_statement",
        Kind::Field => "field",
        Kind::FunctionBodyItem => "function_body_item",
        Kind::GenericParameter => "generic_parameter",
        Kind::Unknown => "unknown",
        Kind::Union => "union",
        Kind::MacroDefinition => "macro_definition",
        Kind::MacroInvocation => "macro_invocation",
        Kind::ExternBlock => "extern_block",
        Kind::ExternCrate => "extern_crate",
        Kind::InlineAssembly => "inline_assembly",
    }
}

fn encode_hld_owner_kind(kind: parseltongue_core_row_types::HldOwnerKindValue) -> &'static str {
    use parseltongue_core_row_types::HldOwnerKindValue as Kind;
    match kind {
        Kind::CrateRoot => "crate_root",
        Kind::Module => "module",
        Kind::Trait => "trait",
        Kind::Impl => "impl",
        Kind::FunctionBody => "function_body",
        Kind::Other => "other",
        Kind::ExternBlock => "extern_block",
    }
}

fn encode_hld_visibility_kind(
    kind: parseltongue_core_row_types::HldEffectiveVisibilityKind,
) -> &'static str {
    use parseltongue_core_row_types::HldEffectiveVisibilityKind as Kind;
    match kind {
        Kind::Public => "public",
        Kind::RestrictedNonPrivate => "restricted_non_private",
        Kind::PrivateEquivalent => "private_equivalent",
        Kind::NotApplicable => "not_applicable",
        Kind::Unknown => "unknown",
    }
}

fn encode_hld_unit_origin(origin: HldUnitOriginKind) -> &'static str {
    match origin {
        HldUnitOriginKind::Workspace => "workspace",
        HldUnitOriginKind::StandardLibrary => "standard_library",
        HldUnitOriginKind::ExternalCrate => "external_crate",
    }
}

fn encode_hld_relation_kind(kind: HldRelationKindValue) -> &'static str {
    match kind {
        HldRelationKindValue::Calls => "calls",
        HldRelationKindValue::UsesType => "uses_type",
        HldRelationKindValue::Imports => "imports",
        HldRelationKindValue::Reads => "reads",
        HldRelationKindValue::Implements => "implements",
        HldRelationKindValue::Reexports => "reexports",
        HldRelationKindValue::Contains => "contains",
        HldRelationKindValue::Derives => "derives",
        HldRelationKindValue::Generates => "generates",
        HldRelationKindValue::Expands => "expands",
    }
}

#[cfg(test)]
mod original_kind_codec_tests {
    use super::*;

    /// REQ-RUST-032.0, REQ-RUST-045.0
    #[test]
    fn test_encode_original_syntax_kinds() {
        use parseltongue_core_row_types::HldItemKindValue as Kind;
        for (kind, text) in [
            (Kind::Union, "union"),
            (Kind::MacroDefinition, "macro_definition"),
            (Kind::MacroInvocation, "macro_invocation"),
            (Kind::ExternBlock, "extern_block"),
            (Kind::ExternCrate, "extern_crate"),
            (Kind::InlineAssembly, "inline_assembly"),
        ] {
            assert_eq!(encode_hld_item_kind(kind), text);
        }
        assert_eq!(
            encode_hld_owner_kind(parseltongue_core_row_types::HldOwnerKindValue::ExternBlock),
            "extern_block"
        );
    }
}

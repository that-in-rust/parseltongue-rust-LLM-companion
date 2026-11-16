use super::syntax_namespace_record_codec::{
    decode_compilation_unit_value, encode_occurrence_identity_value,
};
use super::{SyntaxFileInputData, build_syntax_occurrence_key, restore_source_byte_offset};
use parseltongue_core_row_types::*;
use parseltongue_table_store_traits::StageTableStorePort;
use ra_ap_syntax::{AstNode, SyntaxNode, ast};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::Path,
};

#[derive(Debug, thiserror::Error)]
pub enum ImplHeaderSourceError {
    #[error("impl header extraction cancelled")]
    Cancelled,
    #[error("impl header store: {0}")]
    Store(#[from] StoreBackendFailureError),
    #[error("impl header JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid impl header source field {field} in {table}")]
    InvalidField {
        table: &'static str,
        field: &'static str,
    },
    #[error("impl header source reference is missing or conflicting: {key}")]
    InvalidReference { key: String },
}

pub(super) fn read_impl_source_context<'a>(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    workspace: &'a Path,
) -> Result<ImplHeaderSourceContext<'a>, ImplHeaderSourceError> {
    let read = |table| read_impl_stage_values(store, scope, table);
    let units = read("crate-dependency-graph-table")?
        .into_iter()
        .map(|value| {
            Ok((
                required_impl_source_string(&value, "unit_compilation_identity_key")?.to_owned(),
                value,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, ImplHeaderSourceError>>()?;
    let mut roots = BTreeMap::new();
    for row in read("syntax-namespace-input-scopes")? {
        let key = &row["scope_module_identity_key"];
        if key["scope_semantic_path_parts"]
            .as_array()
            .is_some_and(Vec::is_empty)
        {
            let raw = required_impl_source_string(&row, "scope_cargo_unit_key")?.to_owned();
            let unit = decode_compilation_unit_value(&key["scope_compilation_unit_key"])?;
            if roots
                .insert(raw.clone(), unit.clone())
                .is_some_and(|previous| previous != unit)
            {
                return Err(ImplHeaderSourceError::InvalidReference { key: raw });
            }
        }
    }
    let spans = read("span-reference-text-table")?
        .into_iter()
        .map(|value| {
            Ok((
                required_impl_source_string(&value, "span_semantic_occurrence_key")?.to_owned(),
                value,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, ImplHeaderSourceError>>()?;
    let mut targets = BTreeMap::<String, BTreeSet<String>>::new();
    for row in read("syntax-namespace-input-items")? {
        targets
            .entry(impl_target_lookup_key(
                &row["binding_target_metadata_value"],
            ))
            .or_default()
            .insert(row["binding_target_identity_key"].to_string());
    }
    let mut paths = BTreeMap::<String, Vec<Value>>::new();
    for row in read("path-resolution-site-records")? {
        let key = json!([
            row["site_compilation_scope_record"]["unit_semantic_identity_record"],
            row["site_source_absolute_path"],
            row["site_source_range_start"],
            row["site_source_range_end"],
            row["site_expansion_provenance_chain"]
        ])
        .to_string();
        paths.entry(key).or_default().push(row);
    }
    Ok(ImplHeaderSourceContext {
        units,
        roots,
        spans,
        targets,
        paths,
        workspace,
    })
}

pub fn persist_impl_header_rows(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    workspace: &Path,
    cancel: &RunCancelFlagValue,
) -> Result<usize, ImplHeaderSourceError> {
    if cancel.check_stage_work_cancelled() {
        return Err(ImplHeaderSourceError::Cancelled);
    }
    let snapshot = store.publish_analysis_input_snapshot(scope, cancel)?;
    let context = read_impl_source_context(store, &snapshot, workspace)?;
    let read = |table| read_impl_stage_values(store, &snapshot, table);
    let mut records = Vec::new();
    for field in read("field-signature-detail-table")?
        .into_iter()
        .filter(|row| row["site_syntax_kind_value"] == "IMPL")
    {
        if cancel.check_stage_work_cancelled() {
            return Err(ImplHeaderSourceError::Cancelled);
        }
        let syntax = &field["impl_header_syntax_record"];
        let key = required_impl_source_string(syntax, "impl_source_occurrence_key")?;
        let span = context.span(key)?;
        if span["span_configuration_active_flag"] != true {
            return Err(ImplHeaderSourceError::InvalidReference { key: key.into() });
        }
        let occurrence = context.occurrence(span)?;
        let mut container = syntax["impl_source_container"].clone();
        container["source_file_relative_path"] = occurrence["span_source_file_path"].clone();
        if let Some(owner) = container["container_source_occurrence_key"].as_str() {
            container["container_semantic_occurrence_key"] =
                context.occurrence(context.span(owner)?)?;
        }
        let members = syntax["impl_member_occurrence_rows"]
            .as_array()
            .ok_or(ImplHeaderSourceError::InvalidField {
                table: "field-signature-detail-table",
                field: "impl_member_occurrence_rows",
            })?
            .iter()
            .map(|member| {
                let mut row = member.clone();
                let source = context.span(required_impl_source_string(
                    member,
                    "member_source_occurrence_key",
                )?)?;
                row["member_semantic_occurrence_key"] = context.occurrence(source)?;
                Ok(row)
            })
            .collect::<Result<Vec<_>, ImplHeaderSourceError>>()?;
        let value = json!({
            "impl_source_occurrence_key":key,
            "impl_semantic_occurrence_key":occurrence,
            "impl_self_type_resolution":context.resolve_type(span, &syntax["impl_self_type_source"], false)?,
            "impl_trait_type_resolution":if syntax["impl_trait_type_source"].is_null() {Value::Null} else {
                context.resolve_type(span, &syntax["impl_trait_type_source"], true)?
            },
            "impl_member_occurrence_rows":members,
            "impl_source_container":container,
            "impl_function_body_ancestry_flag":syntax["impl_function_body_ancestry_flag"],
            "impl_negative_implementation_flag":syntax["impl_negative_implementation_flag"],
            "impl_generic_parameters_flag":syntax["impl_generic_parameters_flag"],
            "impl_header_coverage_kind":"header-paths-only"
        });
        records.push(StageTableRowRecord {
            stage_owned_table_name: "impl-header-index-table".into(),
            stage_row_record_key: key.into(),
            stage_row_payload_text: value.to_string(),
        });
    }
    store.write_stage_table_unit(
        &StageUnitWriteRequest {
            stage_run_scope_value: scope,
            stage_owner_name_value: "signature-type-lowering-stage",
            stage_unit_work_key: "impl-header-source-records",
            stage_input_stage_names: &[
                "syntax-spans",
                "syntax-namespace-input-records",
                "name-resolution",
            ],
            stage_unit_output_rows: &records,
        },
        cancel,
    )?;
    Ok(records.len())
}

pub(super) struct ImplHeaderSourceContext<'a> {
    units: BTreeMap<String, Value>,
    roots: BTreeMap<String, CrateCompilationUnitKey>,
    spans: BTreeMap<String, Value>,
    targets: BTreeMap<String, BTreeSet<String>>,
    paths: BTreeMap<String, Vec<Value>>,
    workspace: &'a Path,
}

impl ImplHeaderSourceContext<'_> {
    /// The path of every file that holds a stored span of the run.
    pub(super) fn list_span_file_paths(&self) -> BTreeSet<&str> {
        self.spans
            .values()
            .filter_map(|span| span["file_source_absolute_path"].as_str())
            .collect()
    }

    pub(super) fn span(&self, key: &str) -> Result<&Value, ImplHeaderSourceError> {
        self.spans
            .get(key)
            .ok_or_else(|| ImplHeaderSourceError::InvalidReference { key: key.into() })
    }

    /// The path of a source file as occurrence identities name it: relative to
    /// the package root of its unit, else to the workspace.
    pub(super) fn relate_unit_source_path(
        &self,
        unit: &str,
        absolute: &str,
    ) -> Result<String, ImplHeaderSourceError> {
        let package = self
            .units
            .get(unit)
            .ok_or_else(|| ImplHeaderSourceError::InvalidReference { key: unit.into() })?;
        Ok(Path::new(absolute)
            .strip_prefix(required_impl_source_string(
                package,
                "unit_package_root_path",
            )?)
            .or_else(|_| Path::new(absolute).strip_prefix(self.workspace))
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|_| {
                format!(
                    "external-source/{}/source.rs",
                    blake3::hash(absolute.as_bytes()).to_hex()
                )
            }))
    }

    pub(super) fn occurrence(&self, span: &Value) -> Result<Value, ImplHeaderSourceError> {
        let raw = required_impl_source_string(span, "unit_compilation_identity_key")?;
        let unit = self
            .roots
            .get(raw)
            .ok_or_else(|| ImplHeaderSourceError::InvalidReference { key: raw.into() })?;
        let absolute = required_impl_source_string(span, "file_source_absolute_path")?;
        let relative = self.relate_unit_source_path(raw, absolute)?;
        let start = required_impl_source_offset(span, "span_byte_start_offset")?;
        let end = required_impl_source_offset(span, "span_byte_end_offset")?;
        let text = required_impl_source_string(span, "span_source_code_text")?;
        if end.checked_sub(start) != Some(text.len() as u64)
            || span["span_content_digest_full"]
                != blake3::hash(text.as_bytes()).to_hex().to_string()
        {
            return Err(ImplHeaderSourceError::InvalidReference {
                key: absolute.into(),
            });
        }
        let provenance = span["span_expansion_provenance_chain"]
            .as_array()
            .ok_or(ImplHeaderSourceError::InvalidField {
                table: "span-reference-text-table",
                field: "span_expansion_provenance_chain",
            })?
            .iter()
            .map(|value| {
                decode_impl_digest_value(value.as_str().ok_or(
                    ImplHeaderSourceError::InvalidField {
                        table: "span-reference-text-table",
                        field: "span_expansion_provenance_chain",
                    },
                )?)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let key = SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
            unit,
            &relative,
            SpanByteRangeValue::try_from((start, end)).map_err(|_| {
                ImplHeaderSourceError::InvalidReference {
                    key: absolute.into(),
                }
            })?,
            SpanDigestIdValue::from(*blake3::hash(text.as_bytes()).as_bytes()),
            &provenance,
        )
        .map_err(|_| ImplHeaderSourceError::InvalidReference {
            key: absolute.into(),
        })?;
        Ok(encode_occurrence_identity_value(&key))
    }

    pub(super) fn read_type_path_records(
        &self,
        span: &Value,
        source: &Value,
    ) -> Result<&[Value], ImplHeaderSourceError> {
        let occurrence = self.occurrence(span)?;
        let key = json!([
            occurrence["unit_compilation_identity_key"],
            span["file_source_absolute_path"],
            source["span_byte_start_offset"],
            source["span_byte_end_offset"],
            span["span_expansion_provenance_chain"]
        ])
        .to_string();
        Ok(self.paths.get(&key).map(Vec::as_slice).unwrap_or(&[]))
    }

    /// The one graph identity a resolved path target row names, when the
    /// namespace items hold exactly one for it.
    pub(super) fn read_single_target_identity(
        &self,
        target: &Value,
    ) -> Result<Option<Value>, ImplHeaderSourceError> {
        match self
            .targets
            .get(&impl_target_lookup_key(target))
            .map(|identities| identities.iter().collect::<Vec<_>>())
            .as_deref()
        {
            Some([identity]) => Ok(Some(serde_json::from_str(identity)?)),
            _ => Ok(None),
        }
    }

    pub(super) fn resolve_type(
        &self,
        span: &Value,
        source: &Value,
        trait_target: bool,
    ) -> Result<Value, ImplHeaderSourceError> {
        let unresolved = |reason: &str| {
            json!({"resolution_state_kind":"unresolved", "unresolved_reason_kind":reason,
            "target_identity":Value::Null, "type_source_evidence":source})
        };
        if source["type_simple_path_flag"] != true {
            return Ok(unresolved("trait_solver_needed"));
        }
        let rows = self.read_type_path_records(span, source)?;
        if rows.is_empty() {
            return Ok(unresolved("unknown_receiver"));
        }
        let [path] = rows else {
            return Ok(unresolved("trait_solver_needed"));
        };
        if path["path_resolution_state_kind"] != "resolved" {
            let reason = path["path_unresolved_reason_kind"]
                .as_str()
                .unwrap_or("unknown_receiver");
            if !matches!(
                reason,
                "cfg_inactive"
                    | "build_script"
                    | "proc_macro"
                    | "external_source_missing"
                    | "unknown_receiver"
                    | "trait_solver_needed"
            ) {
                return Err(ImplHeaderSourceError::InvalidField {
                    table: "path-resolution-site-records",
                    field: "path_unresolved_reason_kind",
                });
            }
            // REQ-RUST-053.0 N4: the external name the lookup proved stays beside its reason.
            let mut value = unresolved(reason);
            if !path["path_external_name_record"].is_null() {
                value["external_name_record"] = path["path_external_name_record"].clone();
            }
            return Ok(value);
        }
        let targets = path["path_target_symbol_rows"].as_array().ok_or(
            ImplHeaderSourceError::InvalidField {
                table: "path-resolution-site-records",
                field: "path_target_symbol_rows",
            },
        )?;
        let [target] = targets.as_slice() else {
            return Ok(unresolved("trait_solver_needed"));
        };
        let kind = target["target_symbol_kind_value"].as_str().unwrap_or("");
        if (trait_target && kind != "Trait")
            || (!trait_target && !matches!(kind, "Struct" | "Enum" | "Union" | "Builtin"))
        {
            return Ok(unresolved("trait_solver_needed"));
        }
        let identity = if kind == "Builtin" {
            json!({"target_identity_variant_kind":"builtin", "target_builtin_name_value":required_impl_source_string(target, "target_builtin_name_value")?})
        } else {
            let Some(identities) = self.targets.get(&impl_target_lookup_key(target)) else {
                return Ok(unresolved("unknown_receiver"));
            };
            if identities.len() != 1 {
                return Ok(unresolved("trait_solver_needed"));
            }
            let Some(identity) = identities.first() else {
                return Ok(unresolved("unknown_receiver"));
            };
            serde_json::from_str(identity)?
        };
        Ok(
            json!({"resolution_state_kind":"resolved", "unresolved_reason_kind":Value::Null,
            "target_identity":identity, "type_source_evidence":source}),
        )
    }
}

pub(super) fn read_impl_stage_values(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    table: &'static str,
) -> Result<Vec<Value>, ImplHeaderSourceError> {
    store
        .read_stage_table_rows(scope, table)?
        .iter()
        .map(|row| serde_json::from_str(&row.stage_row_payload_text).map_err(Into::into))
        .collect()
}

fn required_impl_source_string<'a>(
    value: &'a Value,
    field: &'static str,
) -> Result<&'a str, ImplHeaderSourceError> {
    value[field]
        .as_str()
        .ok_or(ImplHeaderSourceError::InvalidField {
            table: "impl-header-source-records",
            field,
        })
}

fn required_impl_source_offset(
    value: &Value,
    field: &'static str,
) -> Result<u64, ImplHeaderSourceError> {
    value[field]
        .as_u64()
        .ok_or(ImplHeaderSourceError::InvalidField {
            table: "impl-header-source-records",
            field,
        })
}

fn decode_impl_digest_value(value: &str) -> Result<[u8; 32], ImplHeaderSourceError> {
    let error = || ImplHeaderSourceError::InvalidField {
        table: "span-reference-text-table",
        field: "span_expansion_provenance_chain",
    };
    if value.len() != 64 || !value.is_ascii() {
        return Err(error());
    }
    let mut bytes = [0; 32];
    for (index, slot) in bytes.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).map_err(|_| error())?;
    }
    Ok(bytes)
}

fn impl_target_lookup_key(value: &Value) -> String {
    let mut identity = value.clone();
    if let Some(fields) = identity.as_object_mut() {
        fields.remove("target_namespace_kind_values");
    }
    identity.to_string()
}

pub(super) fn extract_impl_header_syntax(
    input: &SyntaxFileInputData,
    item: &ast::Impl,
    removed: &[usize],
    active: &HashMap<SyntaxNode, bool>,
) -> Value {
    let node = item.syntax();
    let body = node
        .ancestors()
        .skip(1)
        .any(|owner| ast::Fn::can_cast(owner.kind()));
    let container = node
        .ancestors()
        .skip(1)
        .find(|owner| ast::Module::can_cast(owner.kind()) || ast::Trait::can_cast(owner.kind()));
    let members = item.assoc_item_list().into_iter().flat_map(|list| list.syntax().children().collect::<Vec<_>>())
        .filter(|member| super::check_syntax_item_node(member) && active.get(member).copied().unwrap_or(true))
        .map(|member| json!({
            "member_source_occurrence_key":build_syntax_occurrence_key(input, &member, removed).0,
            "member_declared_name_value":member.children().find(|child| child.kind() == ra_ap_syntax::SyntaxKind::NAME).map(|name| name.text().to_string()),
            "member_syntax_kind_value":format!("{:?}",member.kind()),
            "member_generic_parameters_flag":member.children().any(|child| child.kind() == ra_ap_syntax::SyntaxKind::GENERIC_PARAM_LIST || child.kind() == ra_ap_syntax::SyntaxKind::WHERE_CLAUSE),
            "member_self_parameter_flag":ast::Fn::cast(member.clone()).and_then(|function| function.param_list()).is_some_and(|parameters| parameters.self_param().is_some()),
            "member_self_parameter_kind":ast::Fn::cast(member.clone()).and_then(|function| function.param_list()).and_then(|parameters| parameters.self_param()).map(|parameter| if parameter.ty().is_some() { "typed" } else if parameter.amp_token().is_some() { "reference" } else { "value" }),
        })).collect::<Vec<_>>();
    let type_source = |ty: ast::Type| {
        let simple = ast::PathType::cast(ty.syntax().clone())
            .and_then(|ty| ty.path())
            .filter(|path| {
                !path.syntax().descendants().any(|child| {
                    matches!(
                        child.kind(),
                        ra_ap_syntax::SyntaxKind::GENERIC_ARG_LIST
                            | ra_ap_syntax::SyntaxKind::TYPE_ANCHOR
                    )
                })
            });
        let range = ty.syntax().text_range();
        let start = restore_source_byte_offset(range.start(), removed);
        let end = restore_source_byte_offset(range.end(), removed);
        json!({"span_byte_start_offset":start, "span_byte_end_offset":end,
            "type_source_text":std::str::from_utf8(&input.file_source_content_bytes[start..end]).ok(),
            "type_simple_path_flag":simple.is_some()})
    };
    json!({
        "impl_source_occurrence_key":build_syntax_occurrence_key(input, node, removed).0,
        "impl_self_type_source":item.self_ty().map(type_source),
        "impl_trait_type_source":item.trait_().map(type_source),
        "impl_member_occurrence_rows":members,
        "impl_function_body_ancestry_flag":body,
        "impl_negative_implementation_flag":item.excl_token().is_some(),
        "impl_generic_parameters_flag":node.children().any(|child| matches!(child.kind(), ra_ap_syntax::SyntaxKind::GENERIC_PARAM_LIST | ra_ap_syntax::SyntaxKind::WHERE_CLAUSE)),
        "impl_source_container":{
            "container_kind_value":if body {"body_evidence"} else if !input.file_expansion_provenance_chain.is_empty() {"expanded_source"}
                else if container.as_ref().is_some_and(|node| ast::Trait::can_cast(node.kind())) {"trait"}
                else if container.is_some() {"module"} else {"file_area"},
            "container_source_occurrence_key":container.as_ref().map(|node| build_syntax_occurrence_key(input, node, removed).0),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// REQ-RUST-035.0
    #[test]
    fn test_impl_target_namespace_metadata() {
        let source = json!({"target_source_file_path":"/workspace/src/lib.rs", "target_source_range_start":0,
            "target_source_range_end":19, "target_symbol_kind_value":"Struct", "target_compilation_scope_record":{"unit":"u"}});
        let mut resolved = source.clone();
        resolved["target_namespace_kind_values"] = json!(["type"]);
        assert_eq!(
            impl_target_lookup_key(&source),
            impl_target_lookup_key(&resolved)
        );
        resolved["target_source_range_end"] = json!(20);
        assert_ne!(
            impl_target_lookup_key(&source),
            impl_target_lookup_key(&resolved)
        );
    }
}

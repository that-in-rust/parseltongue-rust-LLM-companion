use super::name_resolution_record_codec::*;
use super::{
    check_node_configuration_active, parse_compatible_rust_syntax, read_effective_module_path,
    restore_source_byte_offset,
};
use parseltongue_core_row_types::*;
use parseltongue_table_store_traits::StageTableStorePort;
use ra_ap_syntax::{AstNode, Edition, SyntaxKind, SyntaxNode, ast};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

pub(super) struct CrateSyntaxProjectionState<'a> {
    pub(super) projection_resolution_input_rows: NameResolutionInputRows,
    pub(super) projection_target_metadata_rows: BTreeMap<NameSymbolTargetKey, Value>,
    pub(super) projection_path_metadata_rows: BTreeMap<SemanticOccurrenceKeyValue, Value>,
    projection_macro_source_rows:
        BTreeMap<SemanticOccurrenceKeyValue, MacroInvocationSourceContext>,
    projection_unit_record_value: Value,
    projection_workspace_root_path: PathBuf,
    projection_source_file_rows: BTreeMap<String, String>,
    projection_active_file_paths: BTreeSet<String>,
    projection_target_cfg_values: BTreeSet<String>,
    projection_source_store_port: &'a dyn StageTableStorePort,
    pub(super) projection_cancel_flag_value: &'a RunCancelFlagValue,
}

struct NamespaceSourceFileContext {
    source_file_absolute_path: String,
    source_file_original_text: String,
    source_crlf_removed_offsets: Vec<usize>,
    source_macro_crate_bindings: BTreeMap<usize, CrateCompilationUnitKey>,
    source_expansion_provenance_rows: Vec<[u8; 32]>,
}

struct MacroInvocationSourceContext {
    macro_source_absolute_path: String,
    macro_argument_range_start: usize,
    macro_invocation_directory_path: PathBuf,
    macro_argument_crate_bindings: BTreeMap<usize, CrateCompilationUnitKey>,
    macro_source_provenance_rows: Vec<[u8; 32]>,
}

struct MacroProjectionRowOffsets {
    module_scope_row_count: usize,
    symbol_binding_row_count: usize,
    import_request_row_count: usize,
    signature_path_row_count: usize,
    macro_invocation_row_count: usize,
}

fn read_projection_row_offsets(input: &NameResolutionInputRows) -> MacroProjectionRowOffsets {
    MacroProjectionRowOffsets {
        module_scope_row_count: input.name_module_scope_rows.len(),
        symbol_binding_row_count: input.name_symbol_binding_rows.len(),
        import_request_row_count: input.name_import_request_rows.len(),
        signature_path_row_count: input.name_signature_path_rows.len(),
        macro_invocation_row_count: input.name_macro_invocation_rows.len(),
    }
}

trait SourcePathCanonicalPort {
    fn read_canonical_source_path(&self, path: &Path) -> Option<PathBuf>;
}

struct NativeCanonicalSourcePort;

impl SourcePathCanonicalPort for NativeCanonicalSourcePort {
    fn read_canonical_source_path(&self, path: &Path) -> Option<PathBuf> {
        path.canonicalize().ok()
    }
}

pub(super) fn build_unit_compilation_key(
    unit: &Value,
) -> Result<CrateCompilationUnitKey, StoreBackendFailureError> {
    Ok(CrateCompilationUnitKey {
        unit_cargo_package_identifier: read_record_string_value(
            unit,
            "unit_cargo_package_identifier",
        )?
        .into(),
        unit_cargo_target_name: read_record_string_value(unit, "unit_cargo_target_name")?.into(),
        unit_cargo_target_kind: read_record_string_value(unit, "unit_cargo_target_kind")?.into(),
        unit_compilation_role_kind: match read_record_string_value(
            unit,
            "unit_compilation_role_kind",
        )? {
            "host" => CompilationUnitRoleKind::Host,
            "target" => CompilationUnitRoleKind::Target,
            _ => return Err(build_name_projection_error("invalid compilation role")),
        },
        unit_configuration_digest_full: *blake3::hash(
            json!([
                unit["unit_target_cfg_values"],
                unit["unit_enabled_feature_values"],
                unit["unit_rust_edition_value"],
                unit["unit_compilation_role_kind"]
            ])
            .to_string()
            .as_bytes(),
        )
        .as_bytes(),
    })
}

impl<'a> CrateSyntaxProjectionState<'a> {
    pub(super) fn build_crate_projection_state(
        unit: Value,
        workspace: &Value,
        store: &'a dyn StageTableStorePort,
        run: &StageRunScopeValue,
        cancel: &'a RunCancelFlagValue,
    ) -> Result<Self, StoreBackendFailureError> {
        let key = build_unit_compilation_key(&unit)?;
        let root = NameResolutionScopeKey {
            scope_compilation_unit_key: key.clone(),
            scope_semantic_path_parts: vec![],
        };
        let prelude = NameResolutionScopeKey {
            scope_semantic_path_parts: vec!["@prelude".into()],
            ..root.clone()
        };
        let external = NameResolutionScopeKey {
            scope_semantic_path_parts: vec!["@extern".into()],
            ..root.clone()
        };
        let edition = match read_record_string_value(&unit, "unit_rust_edition_value")? {
            "2015" => RustSourceEditionKind::Edition2015,
            "2018" => RustSourceEditionKind::Edition2018,
            "2021" => RustSourceEditionKind::Edition2021,
            "2024" => RustSourceEditionKind::Edition2024,
            _ => return Err(build_name_projection_error("unsupported edition")),
        };
        let files = store
            .read_stage_table_rows(run, "file-source-text-table")?
            .into_iter()
            .map(|row| {
                let value: Value = serde_json::from_str(&row.stage_row_payload_text)
                    .map_err(build_name_projection_error)?;
                Ok((
                    read_record_string_value(&value, "file_source_absolute_path")?.into(),
                    read_record_string_value(&value, "file_content_digest_full")?.into(),
                ))
            })
            .collect::<Result<_, StoreBackendFailureError>>()?;
        let mut result = Self {
            projection_resolution_input_rows: NameResolutionInputRows {
                unit_compilation_identity_key: key,
                unit_rust_edition_kind: edition,
                unit_extern_prelude_key: Some(external.clone()),
                name_module_scope_rows: vec![
                    NameScopeRecordRow {
                        scope_module_identity_key: root,
                        scope_parent_identity_key: None,
                        scope_parent_lookup_flag: false,
                        scope_module_boundary_flag: true,
                        scope_implicit_prelude_key: Some(prelude.clone()),
                    },
                    NameScopeRecordRow {
                        scope_module_identity_key: prelude.clone(),
                        scope_parent_identity_key: None,
                        scope_parent_lookup_flag: false,
                        scope_module_boundary_flag: false,
                        scope_implicit_prelude_key: None,
                    },
                    NameScopeRecordRow {
                        scope_module_identity_key: external,
                        scope_parent_identity_key: None,
                        scope_parent_lookup_flag: false,
                        scope_module_boundary_flag: false,
                        scope_implicit_prelude_key: None,
                    },
                ],
                name_symbol_binding_rows: vec![],
                name_import_request_rows: vec![],
                name_signature_path_rows: vec![],
                name_macro_invocation_rows: vec![],
            },
            projection_target_cfg_values: serde_json::from_value(
                unit["unit_target_cfg_values"].clone(),
            )
            .map_err(build_name_projection_error)?,
            projection_workspace_root_path: PathBuf::from(read_record_string_value(
                workspace,
                "workspace_source_root_path",
            )?),
            projection_unit_record_value: unit,
            projection_source_file_rows: files,
            projection_active_file_paths: BTreeSet::new(),
            projection_target_metadata_rows: BTreeMap::new(),
            projection_path_metadata_rows: BTreeMap::new(),
            projection_macro_source_rows: BTreeMap::new(),
            projection_source_store_port: store,
            projection_cancel_flag_value: cancel,
        };
        for name in [
            "bool", "char", "str", "usize", "isize", "u8", "u16", "u32", "u64", "u128", "i8",
            "i16", "i32", "i64", "i128", "f16", "f32", "f64", "f128",
        ] {
            let target = NameSymbolTargetKey::Builtin(name.into());
            result.projection_target_metadata_rows.insert(
                target.clone(),
                json!({"target_builtin_name_value":name,"target_symbol_kind_value":"Builtin"}),
            );
            result
                .projection_resolution_input_rows
                .name_symbol_binding_rows
                .push(NameResolvedBindingRow {
                    binding_lexical_scope_key: prelude.clone(),
                    binding_resolved_name_text: name.into(),
                    binding_namespace_kind_value: NamespaceBindingKindValue::Type,
                    binding_target_identity_key: target,
                    binding_target_child_scope: None,
                    binding_visibility_scope_kind: NameBindingVisibilityKind::Public,
                    binding_selection_priority_kind: NameBindingPriorityKind::Builtin,
                    binding_import_evidence_keys: vec![],
                });
        }
        Ok(result)
    }

    pub(super) fn build_compilation_scope_record(&self, scope: &NameResolutionScopeKey) -> Value {
        json!({
            "unit_source_root_path":self.projection_unit_record_value["unit_source_root_path"],
            "unit_cargo_package_identifier":self.projection_unit_record_value["unit_cargo_package_identifier"],
            "unit_cargo_target_name":self.projection_unit_record_value["unit_cargo_target_name"],
            "unit_cargo_target_kind":self.projection_unit_record_value["unit_cargo_target_kind"],
            "unit_rust_edition_value":self.projection_unit_record_value["unit_rust_edition_value"],
            "unit_module_path_values":scope.scope_semantic_path_parts.iter().filter(|part|!part.starts_with('@')).collect::<Vec<_>>(),
        })
    }

    fn build_source_occurrence_key(
        &self,
        file: &NamespaceSourceFileContext,
        node: &SyntaxNode,
    ) -> Result<SemanticOccurrenceKeyValue, StoreBackendFailureError> {
        let range = node.text_range();
        let start = restore_source_byte_offset(range.start(), &file.source_crlf_removed_offsets);
        let end = restore_source_byte_offset(range.end(), &file.source_crlf_removed_offsets);
        let absolute = Path::new(&file.source_file_absolute_path);
        let package = Path::new(read_record_string_value(
            &self.projection_unit_record_value,
            "unit_package_root_path",
        )?);
        let relative = absolute
            .strip_prefix(package)
            .or_else(|_| absolute.strip_prefix(&self.projection_workspace_root_path))
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|_| {
                format!(
                    "external-source/{}/source.rs",
                    blake3::hash(file.source_file_absolute_path.as_bytes()).to_hex()
                )
            });
        SemanticOccurrenceKeyValue::build_semantic_occurrence_key(
            &self
                .projection_resolution_input_rows
                .unit_compilation_identity_key,
            &relative,
            SpanByteRangeValue::try_from((start as u64, end as u64))
                .map_err(build_name_projection_error)?,
            SpanDigestIdValue::from(
                *blake3::hash(&file.source_file_original_text.as_bytes()[start..end]).as_bytes(),
            ),
            &file.source_expansion_provenance_rows,
        )
        .map_err(build_name_projection_error)
    }

    fn build_child_projection_scope(
        &mut self,
        parent: &NameResolutionScopeKey,
        label: &str,
        node: &SyntaxNode,
        file: &NamespaceSourceFileContext,
        module: bool,
    ) -> NameResolutionScopeKey {
        let part = if module {
            label.to_owned()
        } else {
            format!(
                "@{label}:{}",
                blake3::hash(
                    json!([
                        file.source_file_absolute_path,
                        u32::from(node.text_range().start()),
                        u32::from(node.text_range().end())
                    ])
                    .to_string()
                    .as_bytes()
                )
                .to_hex()
            )
        };
        let mut child = parent.clone();
        child.scope_semantic_path_parts.push(part);
        let prelude = self
            .projection_resolution_input_rows
            .name_module_scope_rows
            .iter()
            .find(|row| &row.scope_module_identity_key == parent)
            .and_then(|row| row.scope_implicit_prelude_key.clone());
        self.projection_resolution_input_rows
            .name_module_scope_rows
            .push(NameScopeRecordRow {
                scope_module_identity_key: child.clone(),
                scope_parent_identity_key: Some(parent.clone()),
                scope_parent_lookup_flag: !module,
                scope_module_boundary_flag: module,
                scope_implicit_prelude_key: prelude,
            });
        child
    }

    fn build_node_visibility_kind(
        &self,
        node: &SyntaxNode,
        scope: &NameResolutionScopeKey,
    ) -> NameBindingVisibilityKind {
        let visibility = node
            .children()
            .find_map(ast::Visibility::cast)
            .map(|visibility| visibility.syntax().text().to_string());
        let mut module = scope.clone();
        while !self
            .projection_resolution_input_rows
            .name_module_scope_rows
            .iter()
            .any(|row| row.scope_module_identity_key == module && row.scope_module_boundary_flag)
        {
            let Some(parent) = self
                .projection_resolution_input_rows
                .name_module_scope_rows
                .iter()
                .find(|row| row.scope_module_identity_key == module)
                .and_then(|row| row.scope_parent_identity_key.clone())
            else {
                break;
            };
            module = parent;
        }
        match visibility.as_deref().map(|text| text.replace(' ', "")) {
            Some(text) if text == "pub" => NameBindingVisibilityKind::Public,
            Some(text) if text == "pub(crate)" => {
                NameBindingVisibilityKind::Restricted(NameResolutionScopeKey {
                    scope_semantic_path_parts: vec![],
                    ..module
                })
            }
            Some(text) if text == "pub(super)" => {
                module.scope_semantic_path_parts.pop();
                NameBindingVisibilityKind::Restricted(module)
            }
            Some(text) if text.starts_with("pub(in") => {
                let parts = text
                    .trim_start_matches("pub(in")
                    .trim_end_matches(')')
                    .split("::")
                    .filter(|part| *part != "crate" && *part != "self")
                    .map(str::to_owned)
                    .collect();
                NameBindingVisibilityKind::Restricted(NameResolutionScopeKey {
                    scope_semantic_path_parts: parts,
                    ..module
                })
            }
            _ => NameBindingVisibilityKind::Restricted(module),
        }
    }

    fn register_source_symbol_binding(
        &mut self,
        node: &SyntaxNode,
        file: &NamespaceSourceFileContext,
        scope: &NameResolutionScopeKey,
        symbol_kind_name_pair: (&str, &str),
        namespace: NamespaceBindingKindValue,
        child: Option<NameResolutionScopeKey>,
    ) -> Result<NameSymbolTargetKey, StoreBackendFailureError> {
        let (kind, name) = symbol_kind_name_pair;
        let occurrence = self.build_source_occurrence_key(file, node)?;
        let target = if matches!(kind, "TypeParam" | "ConstParam" | "SelfType") {
            NameSymbolTargetKey::GenericParameter(occurrence, name.into())
        } else {
            NameSymbolTargetKey::Definition(occurrence)
        };
        let range = node.text_range();
        self.projection_target_metadata_rows.insert(target.clone(),json!({
            "target_source_file_path":file.source_file_absolute_path,
            "target_source_range_start":restore_source_byte_offset(range.start(),&file.source_crlf_removed_offsets),
            "target_source_range_end":restore_source_byte_offset(range.end(),&file.source_crlf_removed_offsets),
            "target_symbol_kind_value":kind,"target_compilation_scope_record":self.build_compilation_scope_record(scope),
        }));
        if kind == "Macro" {
            let trees = node
                .children()
                .filter_map(ast::TokenTree::cast)
                .collect::<Vec<_>>();
            let body = trees
                .last()
                .ok_or_else(|| build_name_projection_error("macro definition body absent"))?;
            let range = body.syntax().text_range();
            let start =
                restore_source_byte_offset(range.start(), &file.source_crlf_removed_offsets);
            let end = restore_source_byte_offset(range.end(), &file.source_crlf_removed_offsets);
            let source = &file.source_file_original_text[start..end];
            let crates=source.match_indices("$crate").map(|(offset,_)|{
                let unit=file.source_macro_crate_bindings.get(&(start+offset)).unwrap_or(&self.projection_resolution_input_rows.unit_compilation_identity_key);
                json!({"macro_source_byte_offset":offset,"macro_definition_unit_key":encode_compilation_unit_value(unit)})
            }).collect::<Vec<_>>();
            let pattern = trees
                .first()
                .filter(|_| node.kind() == SyntaxKind::MACRO_DEF && trees.len() > 1)
                .map(|tree| read_original_node_source(tree.syntax(), file));
            self.projection_target_metadata_rows
                .get_mut(&target)
                .ok_or_else(|| build_name_projection_error("macro metadata missing"))?["target_macro_definition_record"] = json!({
                "macro_definition_source_text":source,"macro_pattern_source_text":pattern,
                "macro_source_absolute_path":file.source_file_absolute_path,"macro_source_range_start":start,
                "macro_definition_unit_key":encode_compilation_unit_value(&self.projection_resolution_input_rows.unit_compilation_identity_key),
                "macro_definition_edition_kind":self.projection_unit_record_value["unit_rust_edition_value"],
                "macro_definition_crate_rows":crates,
            });
        }
        let inherited = node.parent().is_some_and(|parent| {
            parent.kind() == SyntaxKind::ASSOC_ITEM_LIST
                && parent
                    .parent()
                    .is_some_and(|owner| ast::Trait::can_cast(owner.kind()))
        });
        let exported_macro = kind == "Macro"
            && super::collect_effective_node_attributes(node, &self.projection_target_cfg_values)?
                .iter()
                .any(|attribute| {
                    attribute == "macro_export" || attribute.starts_with("macro_export(")
                });
        let visibility = if exported_macro {
            NameBindingVisibilityKind::Public
        } else if inherited {
            self.projection_resolution_input_rows
                .name_symbol_binding_rows
                .iter()
                .find(|row| row.binding_target_child_scope.as_ref() == Some(scope))
                .map(|row| row.binding_visibility_scope_kind.clone())
                .unwrap_or(NameBindingVisibilityKind::Public)
        } else {
            self.build_node_visibility_kind(node, scope)
        };
        self.projection_resolution_input_rows
            .name_symbol_binding_rows
            .push(NameResolvedBindingRow {
                binding_lexical_scope_key: scope.clone(),
                binding_resolved_name_text: name.trim_start_matches("r#").into(),
                binding_namespace_kind_value: namespace,
                binding_target_identity_key: target.clone(),
                binding_target_child_scope: child,
                binding_visibility_scope_kind: visibility,
                binding_selection_priority_kind: NameBindingPriorityKind::Declaration,
                binding_import_evidence_keys: vec![],
            });
        if exported_macro && !scope.scope_semantic_path_parts.is_empty() {
            let mut exported = self
                .projection_resolution_input_rows
                .name_symbol_binding_rows
                .last()
                .cloned()
                .ok_or_else(|| build_name_projection_error("macro binding missing"))?;
            exported
                .binding_lexical_scope_key
                .scope_semantic_path_parts
                .clear();
            self.projection_resolution_input_rows
                .name_symbol_binding_rows
                .push(exported);
        }
        Ok(target)
    }

    pub(super) fn project_unit_source_root(&mut self) -> Result<(), StoreBackendFailureError> {
        let root =
            read_record_string_value(&self.projection_unit_record_value, "unit_source_root_path")?
                .to_owned();
        let scope = NameResolutionScopeKey {
            scope_compilation_unit_key: self
                .projection_resolution_input_rows
                .unit_compilation_identity_key
                .clone(),
            scope_semantic_path_parts: vec![],
        };
        self.projection_target_metadata_rows.insert(NameSymbolTargetKey::CrateRoot(scope.scope_compilation_unit_key.clone()),json!({
            "target_source_file_path":root,"target_source_range_start":0,"target_source_range_end":0,
            "target_symbol_kind_value":"Crate","target_compilation_scope_record":self.build_compilation_scope_record(&scope),
        }));
        self.visit_namespace_source_file(&root, &scope)
    }

    pub(super) fn project_macro_expansion_rows(
        &mut self,
        invocation: &NameMacroInvocationRow,
        definition: &Value,
        expanded: &DeclarativeMacroExpansionOutput,
    ) -> Result<NameExpansionRoundRows, StoreBackendFailureError> {
        let context = self
            .projection_macro_source_rows
            .get(&invocation.invocation_site_occurrence_key)
            .ok_or_else(|| build_name_projection_error("macro invocation source context absent"))?;
        let invocation_value =
            encode_occurrence_identity_value(&invocation.invocation_site_occurrence_key);
        let digest = *blake3::hash(invocation_value.to_string().as_bytes()).as_bytes();
        let mut provenance = context.macro_source_provenance_rows.clone();
        provenance.push(digest);
        let path = Path::new(read_record_string_value(
            &self.projection_unit_record_value,
            "unit_package_root_path",
        )?)
        .join(".parseltongue/macro-source")
        .join(encode_digest_hex_value(&digest))
        .join("expanded.rs")
        .to_string_lossy()
        .into_owned();
        let directory = context.macro_invocation_directory_path.clone();
        let source = &expanded.macro_expanded_source_text;
        let default_unit = decode_compilation_unit_value(&definition["macro_definition_unit_key"])?;
        let crates = source
            .match_indices("$crate")
            .filter_map(|(offset, _)| {
                expanded
                    .macro_token_origin_rows
                    .iter()
                    .find(|origin| {
                        let range = origin.token_output_byte_range.read_source_range_pair();
                        range.0 == offset as u64 || range.0 == offset as u64 + 1
                    })
                    .map(|origin| {
                        let original =
                            origin.token_original_byte_range.read_source_range_pair().0 as usize;
                        let unit = match origin.token_source_origin_kind {
                            MacroSourceOriginKind::Invocation => context
                                .macro_argument_crate_bindings
                                .get(&(context.macro_argument_range_start + original))
                                .cloned()
                                .unwrap_or_else(|| {
                                    invocation
                                        .invocation_lexical_scope_key
                                        .scope_compilation_unit_key
                                        .clone()
                                }),
                            MacroSourceOriginKind::Definition => {
                                definition["macro_definition_crate_rows"]
                                    .as_array()
                                    .into_iter()
                                    .flatten()
                                    .find(|row| {
                                        row["macro_source_byte_offset"].as_u64().is_some_and(
                                            |start| {
                                                start <= original as u64
                                                    && (original as u64) < start + 6
                                            },
                                        )
                                    })
                                    .map(|row| {
                                        decode_compilation_unit_value(
                                            &row["macro_definition_unit_key"],
                                        )
                                    })
                                    .transpose()?
                                    .unwrap_or_else(|| default_unit.clone())
                            }
                        };
                        Ok((offset, unit))
                    })
            })
            .collect::<Result<BTreeMap<_, _>, StoreBackendFailureError>>()?;
        let normalized = crates
            .keys()
            .rev()
            .fold(source.clone(), |mut text, offset| {
                text.replace_range(*offset..*offset + 1, "_");
                text
            })
            .replace("\r\n", "\n");
        let edition = match self.projection_resolution_input_rows.unit_rust_edition_kind {
            RustSourceEditionKind::Edition2015 => Edition::Edition2015,
            RustSourceEditionKind::Edition2018 => Edition::Edition2018,
            RustSourceEditionKind::Edition2021 => Edition::Edition2021,
            RustSourceEditionKind::Edition2024 => Edition::Edition2024,
        };
        let parsed = parse_compatible_rust_syntax(&normalized, "items", edition)?;
        if let Some(error) = parsed.errors().first() {
            return Err(build_name_projection_error(format!(
                "macro expansion {path}: {error}"
            )));
        }
        let file = NamespaceSourceFileContext {
            source_file_absolute_path: path.clone(),
            source_file_original_text: source.clone(),
            source_crlf_removed_offsets: source
                .match_indices("\r\n")
                .enumerate()
                .map(|(count, (offset, _))| offset - count)
                .collect(),
            source_macro_crate_bindings: crates,
            source_expansion_provenance_rows: provenance.clone(),
        };
        let chain = provenance
            .iter()
            .map(encode_digest_hex_value)
            .collect::<Vec<_>>();
        let syntax = super::extract_file_syntax_core(
            &super::SyntaxFileInputData {
                file_source_absolute_path: path.clone(),
                file_source_content_bytes: source.as_bytes().to_vec(),
                unit_compilation_identity_key: read_record_string_value(
                    &self.projection_unit_record_value,
                    "unit_compilation_identity_key",
                )?
                .into(),
                unit_rust_edition_value: read_record_string_value(
                    &self.projection_unit_record_value,
                    "unit_rust_edition_value",
                )?
                .into(),
                unit_target_cfg_values: self.projection_target_cfg_values.clone(),
                file_module_directory_path: directory.to_string_lossy().into_owned(),
                unit_metadata_environment_values: BTreeMap::new(),
                file_parse_context_kind: "items".into(),
                file_expansion_provenance_chain: chain.clone(),
            },
            Some(&normalized),
        )?;
        let origins=expanded.macro_token_origin_rows.iter().map(|origin|{
            let output=origin.token_output_byte_range.read_source_range_pair();
            let original=origin.token_original_byte_range.read_source_range_pair();
            let (kind,path,start)=match origin.token_source_origin_kind{
                MacroSourceOriginKind::Definition=>("definition",definition["macro_source_absolute_path"].clone(),definition["macro_source_range_start"].as_u64().unwrap_or(0)),
                MacroSourceOriginKind::Invocation=>("invocation",json!(context.macro_source_absolute_path),context.macro_argument_range_start as u64),
            };
            json!({"token_output_byte_range":[output.0,output.1],"token_original_byte_range":[start+original.0,start+original.1],"token_source_origin_kind":kind,"token_source_absolute_path":path})
        }).collect::<Vec<_>>();
        let input = &self.projection_resolution_input_rows;
        let before = read_projection_row_offsets(input);
        self.visit_namespace_syntax_node(
            &parsed.syntax_node(),
            &invocation.invocation_lexical_scope_key,
            &directory,
            &file,
        )?;
        let input = &self.projection_resolution_input_rows;
        let content = blake3::hash(source.as_bytes()).to_hex().to_string();
        self.projection_source_store_port
            .write_file_content_bytes(&content, source.as_bytes())?;
        let mut rows = syntax.stage_unit_output_rows;
        rows.push(super::build_syntax_output_record("file-source-text-table",path.clone(),json!({
            "file_source_absolute_path":path,"file_content_digest_full":content,"file_source_byte_count":source.len(),"file_expansion_provenance_chain":chain,
        })));
        rows.push(super::build_syntax_output_record("macro-expanded-item-table",encode_digest_hex_value(&digest),json!({
            "macro_invocation_identity_record":invocation_value,"macro_invocation_provenance_chain":chain,
            "macro_virtual_source_path":path,"macro_expanded_source_text":source,"macro_token_origin_rows":origins,
            "macro_matched_arm_index":expanded.macro_matched_arm_index,"macro_resolution_state_kind":"expanded",
        })));
        Ok(NameExpansionRoundRows {
            expansion_module_scope_rows: input.name_module_scope_rows
                [before.module_scope_row_count..]
                .to_vec(),
            expansion_symbol_binding_rows: input.name_symbol_binding_rows
                [before.symbol_binding_row_count..]
                .to_vec(),
            expansion_import_request_rows: input.name_import_request_rows
                [before.import_request_row_count..]
                .to_vec(),
            expansion_signature_path_rows: input.name_signature_path_rows
                [before.signature_path_row_count..]
                .to_vec(),
            expansion_macro_invocation_rows: input.name_macro_invocation_rows
                [before.macro_invocation_row_count..]
                .to_vec(),
            expansion_completed_invocation_keys: vec![
                invocation.invocation_site_occurrence_key.clone(),
            ],
            expansion_semantic_output_rows: rows,
            ..Default::default()
        })
    }

    fn visit_namespace_source_file(
        &mut self,
        path: &str,
        scope: &NameResolutionScopeKey,
    ) -> Result<(), StoreBackendFailureError> {
        if self
            .projection_cancel_flag_value
            .check_stage_work_cancelled()
        {
            return Err(build_name_projection_error("NameResolutionCancelledError"));
        }
        if !self.projection_active_file_paths.insert(path.into()) {
            return Err(build_name_projection_error("recursive module source file"));
        }
        let digest = self.projection_source_file_rows.get(path).ok_or_else(|| {
            build_name_projection_error(format!(
                "StageInputMissingError: source closure did not ingest {path}"
            ))
        })?;
        let bytes = self
            .projection_source_store_port
            .read_file_content_bytes(digest)?;
        let source = String::from_utf8(bytes).map_err(build_name_projection_error)?;
        let removed = source
            .match_indices("\r\n")
            .enumerate()
            .map(|(count, (offset, _))| offset - count)
            .collect::<Vec<_>>();
        let normalized = source.replace("\r\n", "\n");
        let edition = match self.projection_resolution_input_rows.unit_rust_edition_kind {
            RustSourceEditionKind::Edition2015 => Edition::Edition2015,
            RustSourceEditionKind::Edition2018 => Edition::Edition2018,
            RustSourceEditionKind::Edition2021 => Edition::Edition2021,
            RustSourceEditionKind::Edition2024 => Edition::Edition2024,
        };
        let parsed = parse_compatible_rust_syntax(&normalized, "items", edition)?;
        if let Some(error) = parsed.errors().first() {
            return Err(build_name_projection_error(format!("{path}: {error}")));
        }
        let file = NamespaceSourceFileContext {
            source_file_absolute_path: path.into(),
            source_file_original_text: source,
            source_crlf_removed_offsets: removed,
            source_macro_crate_bindings: BTreeMap::new(),
            source_expansion_provenance_rows: vec![],
        };
        let parent = Path::new(path)
            .parent()
            .ok_or_else(|| build_name_projection_error("source parent absent"))?;
        let directory = if Path::new(path)
            .file_name()
            .is_some_and(|name| matches!(name.to_str(), Some("lib.rs" | "main.rs" | "mod.rs")))
        {
            parent.to_owned()
        } else {
            parent.join(
                Path::new(path)
                    .file_stem()
                    .ok_or_else(|| build_name_projection_error("source stem absent"))?,
            )
        };
        self.visit_namespace_syntax_node(&parsed.syntax_node(), scope, &directory, &file)?;
        self.projection_active_file_paths.remove(path);
        Ok(())
    }

    fn visit_namespace_syntax_node(
        &mut self,
        node: &SyntaxNode,
        scope: &NameResolutionScopeKey,
        directory: &Path,
        file: &NamespaceSourceFileContext,
    ) -> Result<(), StoreBackendFailureError> {
        if self
            .projection_cancel_flag_value
            .check_stage_work_cancelled()
        {
            return Err(build_name_projection_error("NameResolutionCancelledError"));
        }
        if !check_node_configuration_active(node, &self.projection_target_cfg_values)? {
            return Ok(());
        }
        let kind = node.kind();
        let name = node
            .children()
            .find_map(ast::Name::cast)
            .map(|name| name.text().trim_start_matches("r#").to_owned());
        if let Some(module) = ast::Module::cast(node.clone()) {
            let name = name.ok_or_else(|| build_name_projection_error("unnamed module"))?;
            let child = self.build_child_projection_scope(scope, &name, node, file, true);
            self.register_source_symbol_binding(
                node,
                file,
                scope,
                ("Module", &name),
                NamespaceBindingKindValue::Type,
                Some(child.clone()),
            )?;
            if let Some(items) = module.item_list() {
                let child_directory = directory.join(
                    read_effective_module_path(node, &self.projection_target_cfg_values)?
                        .unwrap_or(name),
                );
                return self.visit_namespace_syntax_node(
                    items.syntax(),
                    &child,
                    &child_directory,
                    file,
                );
            }
            let candidates = if let Some(path) =
                read_effective_module_path(node, &self.projection_target_cfg_values)?
            {
                vec![directory.join(path)]
            } else {
                vec![
                    directory.join(format!("{name}.rs")),
                    directory.join(&name).join("mod.rs"),
                ]
            };
            let selected = candidates
                .into_iter()
                .filter(|path| !path.components().any(|part| part.as_os_str() == "target"))
                .filter_map(|path| NativeCanonicalSourcePort.read_canonical_source_path(&path))
                .map(|path| path.to_string_lossy().into_owned())
                .find(|path| self.projection_source_file_rows.contains_key(path));
            return self.visit_namespace_source_file(
                &selected.ok_or_else(|| {
                    build_name_projection_error(format!("StageInputMissingError: module {name}"))
                })?,
                &child,
            );
        }
        let item_kind = read_syntax_symbol_kind(kind);
        let needs_scope = matches!(
            kind,
            SyntaxKind::FN
                | SyntaxKind::STRUCT
                | SyntaxKind::ENUM
                | SyntaxKind::UNION
                | SyntaxKind::TRAIT
                | SyntaxKind::IMPL
                | SyntaxKind::BLOCK_EXPR
                | SyntaxKind::TYPE_ALIAS
                | SyntaxKind::TYPE_PARAM
        );
        let nested = if needs_scope {
            Some(self.build_child_projection_scope(scope, &format!("{kind:?}"), node, file, false))
        } else {
            None
        };
        if let (Some((label, namespace)), Some(name)) = (item_kind, name.as_deref()) {
            let associated = (!matches!(
                label,
                "Function" | "Const" | "Static" | "ConstParam" | "Macro" | "Variant"
            ))
            .then(|| nested.clone())
            .flatten();
            self.register_source_symbol_binding(
                node,
                file,
                scope,
                (label, name),
                namespace,
                associated,
            )?;
            if (label == "Struct"
                && !node
                    .children()
                    .any(|child| child.kind() == SyntaxKind::RECORD_FIELD_LIST))
                || label == "Variant"
            {
                self.register_source_symbol_binding(
                    node,
                    file,
                    scope,
                    (label, name),
                    NamespaceBindingKindValue::Value,
                    None,
                )?;
            }
        }
        let active = nested.as_ref().unwrap_or(scope);
        if matches!(kind, SyntaxKind::TRAIT | SyntaxKind::IMPL) {
            self.register_source_symbol_binding(
                node,
                file,
                active,
                (
                    if kind == SyntaxKind::TRAIT {
                        "TypeParam"
                    } else {
                        "SelfType"
                    },
                    "Self",
                ),
                NamespaceBindingKindValue::Type,
                Some(active.clone()),
            )?;
        }
        if matches!(kind, SyntaxKind::TYPE_PARAM | SyntaxKind::TRAIT) {
            let bounds = node
                .children()
                .filter(|child| child.kind() == SyntaxKind::TYPE_BOUND_LIST)
                .flat_map(|list| list.children())
                .filter_map(|bound| bound.descendants().find_map(ast::Path::cast))
                .collect::<Vec<_>>();
            for bound in bounds {
                self.projection_resolution_input_rows
                    .name_import_request_rows
                    .push(NameImportRequestRow {
                        import_site_occurrence_key: self
                            .build_source_occurrence_key(file, bound.syntax())?,
                        import_lexical_scope_key: active.clone(),
                        import_path_segment_rows: read_contextual_path_segments(
                            &bound, file, false,
                        ),
                        import_binding_alias_text: None,
                        import_glob_expansion_flag: true,
                        import_visibility_scope_kind: NameBindingVisibilityKind::Public,
                        import_resolution_source_kind: NameResolutionPathKind::Signature,
                    });
            }
        }
        if let Some(use_tree) = ast::UseTree::cast(node.clone())
            && use_tree.use_tree_list().is_none()
            && let Some(path) = use_tree.path()
        {
            let alias = use_tree.rename().map(|rename| {
                rename
                    .syntax()
                    .text()
                    .to_string()
                    .trim_start_matches("as")
                    .trim()
                    .trim_start_matches("r#")
                    .to_owned()
            });
            self.projection_resolution_input_rows
                .name_import_request_rows
                .push(NameImportRequestRow {
                    import_site_occurrence_key: self.build_source_occurrence_key(file, node)?,
                    import_lexical_scope_key: scope.clone(),
                    import_path_segment_rows: read_contextual_path_segments(&path, file, true),
                    import_binding_alias_text: alias,
                    import_glob_expansion_flag: use_tree.star_token().is_some(),
                    import_visibility_scope_kind: node
                        .ancestors()
                        .find_map(ast::Use::cast)
                        .map(|item| self.build_node_visibility_kind(item.syntax(), scope))
                        .unwrap_or(NameBindingVisibilityKind::Invisible),
                    import_resolution_source_kind: NameResolutionPathKind::Import,
                });
        }
        if let Some(path) = ast::Path::cast(node.clone())
            && check_exported_signature_path(&path)
        {
            let occurrence = self.build_source_occurrence_key(file, node)?;
            let in_use = node.ancestors().any(|node| ast::Use::can_cast(node.kind()));
            let range = node.text_range();
            self.projection_path_metadata_rows.insert(occurrence.clone(),json!({
                    "unit_source_root_path":self.projection_unit_record_value["unit_source_root_path"],
                    "site_source_absolute_path":file.source_file_absolute_path,
                    "site_source_range_start":restore_source_byte_offset(range.start(),&file.source_crlf_removed_offsets),
                    "site_source_range_end":restore_source_byte_offset(range.end(),&file.source_crlf_removed_offsets),
                    "path_site_source_text":read_original_node_source(node,file),
                    "site_expansion_provenance_chain":file.source_expansion_provenance_rows.iter().map(encode_digest_hex_value).collect::<Vec<_>>(),
                    "site_compilation_scope_record":self.build_compilation_scope_record(scope),
                }));
            self.projection_resolution_input_rows
                .name_signature_path_rows
                .push(NamePathRequestRow {
                    path_site_occurrence_key: occurrence,
                    path_lexical_scope_key: scope.clone(),
                    path_source_segment_rows: if in_use {
                        read_contextual_path_segments(&path, file, true)
                    } else {
                        read_contextual_path_segments(&path, file, false)
                    },
                    path_requested_namespace_kind: if in_use {
                        NameNamespaceFilterKind::Any
                    } else {
                        NameNamespaceFilterKind::TypeOrValue
                    },
                    path_resolution_source_kind: if in_use {
                        NameResolutionPathKind::Import
                    } else {
                        NameResolutionPathKind::Signature
                    },
                    path_unavailable_reason_kind: None,
                });
        }
        if let Some(call) = ast::MacroCall::cast(node.clone())
            && !node
                .parent()
                .is_some_and(|parent| ast::MacroExpr::can_cast(parent.kind()))
        {
            let occurrence = self.build_source_occurrence_key(file, node)?;
            let argument = call.token_tree();
            let start = argument
                .as_ref()
                .map(|tree| {
                    restore_source_byte_offset(
                        tree.syntax().text_range().start(),
                        &file.source_crlf_removed_offsets,
                    )
                })
                .unwrap_or(0);
            self.projection_macro_source_rows.insert(
                occurrence.clone(),
                MacroInvocationSourceContext {
                    macro_source_absolute_path: file.source_file_absolute_path.clone(),
                    macro_argument_range_start: start,
                    macro_invocation_directory_path: directory.into(),
                    macro_argument_crate_bindings: file.source_macro_crate_bindings.clone(),
                    macro_source_provenance_rows: file.source_expansion_provenance_rows.clone(),
                },
            );
            self.projection_resolution_input_rows
                .name_macro_invocation_rows
                .push(NameMacroInvocationRow {
                    invocation_site_occurrence_key: occurrence,
                    invocation_lexical_scope_key: scope.clone(),
                    invocation_macro_path_segments: call
                        .path()
                        .as_ref()
                        .map(|path| read_contextual_path_segments(path, file, false))
                        .unwrap_or_default(),
                    invocation_argument_source_text: argument
                        .map(|tree| read_original_node_source(tree.syntax(), file))
                        .unwrap_or_default(),
                });
        }
        for child in node.children() {
            self.visit_namespace_syntax_node(&child, active, directory, file)?;
        }
        Ok(())
    }
}

fn read_original_node_source(node: &SyntaxNode, file: &NamespaceSourceFileContext) -> String {
    let range = node.text_range();
    file.source_file_original_text[restore_source_byte_offset(
        range.start(),
        &file.source_crlf_removed_offsets,
    )
        ..restore_source_byte_offset(range.end(), &file.source_crlf_removed_offsets)]
        .into()
}

fn read_contextual_path_segments(
    path: &ast::Path,
    file: &NamespaceSourceFileContext,
    grouped: bool,
) -> Vec<NamePathSegmentKind> {
    let mut parts = if grouped {
        read_grouped_import_segments(path)
    } else {
        read_ast_path_segments(path)
    };
    if parts.first() == Some(&NamePathSegmentKind::Name("_crate".into())) {
        let start = restore_source_byte_offset(
            path.syntax().text_range().start(),
            &file.source_crlf_removed_offsets,
        );
        let unit = file.source_macro_crate_bindings.get(&start).or_else(|| {
            path.syntax()
                .ancestors()
                .filter_map(ast::UseTree::cast)
                .filter_map(|tree| tree.path())
                .find_map(|prefix| {
                    let start = restore_source_byte_offset(
                        prefix.syntax().text_range().start(),
                        &file.source_crlf_removed_offsets,
                    );
                    file.source_macro_crate_bindings.get(&start)
                })
        });
        if let Some(unit) = unit {
            parts[0] = NamePathSegmentKind::DefiningCrate(unit.clone());
        }
    }
    parts
}

fn read_syntax_symbol_kind(kind: SyntaxKind) -> Option<(&'static str, NamespaceBindingKindValue)> {
    match kind {
        SyntaxKind::FN => Some(("Function", NamespaceBindingKindValue::Value)),
        SyntaxKind::STRUCT => Some(("Struct", NamespaceBindingKindValue::Type)),
        SyntaxKind::ENUM => Some(("Enum", NamespaceBindingKindValue::Type)),
        SyntaxKind::UNION => Some(("Union", NamespaceBindingKindValue::Type)),
        SyntaxKind::TRAIT => Some(("Trait", NamespaceBindingKindValue::Type)),
        SyntaxKind::TYPE_ALIAS => Some(("TypeAlias", NamespaceBindingKindValue::Type)),
        SyntaxKind::CONST => Some(("Const", NamespaceBindingKindValue::Value)),
        SyntaxKind::STATIC => Some(("Static", NamespaceBindingKindValue::Value)),
        SyntaxKind::VARIANT => Some(("Variant", NamespaceBindingKindValue::Type)),
        SyntaxKind::MACRO_RULES | SyntaxKind::MACRO_DEF => {
            Some(("Macro", NamespaceBindingKindValue::Macro))
        }
        SyntaxKind::TYPE_PARAM => Some(("TypeParam", NamespaceBindingKindValue::Type)),
        SyntaxKind::CONST_PARAM => Some(("ConstParam", NamespaceBindingKindValue::Value)),
        _ => None,
    }
}

fn read_ast_path_segments(path: &ast::Path) -> Vec<NamePathSegmentKind> {
    let mut parts = path
        .qualifier()
        .as_ref()
        .map(read_ast_path_segments)
        .unwrap_or_else(|| {
            if path.coloncolon_token().is_some() {
                vec![NamePathSegmentKind::GlobalRoot]
            } else {
                vec![]
            }
        });
    if let Some(segment) = path.segment() {
        match segment.kind() {
            Some(ast::PathSegmentKind::CrateKw) => parts.push(NamePathSegmentKind::CrateRoot),
            Some(ast::PathSegmentKind::SelfKw) => {
                if parts.is_empty() {
                    parts.push(NamePathSegmentKind::SelfModule);
                }
            }
            Some(ast::PathSegmentKind::SuperKw) => parts.push(NamePathSegmentKind::ParentModule),
            Some(ast::PathSegmentKind::SelfTypeKw) => {
                parts.push(NamePathSegmentKind::Name("Self".into()))
            }
            Some(ast::PathSegmentKind::Name(name)) => parts.push(NamePathSegmentKind::Name(
                name.text().trim_start_matches("r#").into(),
            )),
            Some(ast::PathSegmentKind::Type {
                type_ref,
                trait_ref,
            }) => {
                parts = trait_ref
                    .and_then(|ty| ty.path())
                    .as_ref()
                    .map(read_ast_path_segments)
                    .or_else(|| {
                        type_ref
                            .and_then(|ty| ast::PathType::cast(ty.syntax().clone()))
                            .and_then(|ty| ty.path())
                            .as_ref()
                            .map(read_ast_path_segments)
                    })
                    .unwrap_or_default();
            }
            _ => {}
        }
    }
    parts
}

fn read_grouped_import_segments(path: &ast::Path) -> Vec<NamePathSegmentKind> {
    let parents = path
        .syntax()
        .ancestors()
        .filter_map(ast::UseTree::cast)
        .skip(1)
        .filter_map(|tree| tree.path())
        .collect::<Vec<_>>();
    parents
        .iter()
        .rev()
        .flat_map(read_ast_path_segments)
        .chain(read_ast_path_segments(path))
        .collect()
}

fn check_exported_signature_path(path: &ast::Path) -> bool {
    if path
        .syntax()
        .parent()
        .is_some_and(|parent| ast::Path::can_cast(parent.kind()))
    {
        return false;
    }
    let ancestors = path.syntax().ancestors().collect::<Vec<_>>();
    if ancestors.iter().any(|node| ast::Use::can_cast(node.kind())) {
        return true;
    }
    if ancestors
        .iter()
        .find_map(|node| ast::Fn::cast(node.clone()))
        .and_then(|function| function.body())
        .is_some_and(|body| {
            body.syntax()
                .text_range()
                .contains_range(path.syntax().text_range())
        })
    {
        return false;
    }
    ancestors
        .iter()
        .any(|node| ast::Type::can_cast(node.kind()) || ast::TypeBound::can_cast(node.kind()))
}

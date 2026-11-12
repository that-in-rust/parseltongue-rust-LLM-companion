use parseltongue_core_row_types::*;
use std::collections::BTreeMap;

pub(super) fn key(value: &impl std::fmt::Debug) -> String {
    format!("{value:?}")
}

pub(super) fn error(message: impl std::fmt::Display) -> NameResolutionStageError {
    NameResolutionStageError::Input(StageInputMissingError {
        error_context_detail_text: message.to_string(),
    })
}

pub(super) fn namespace(value: NamespaceBindingKindValue) -> &'static str {
    match value {
        NamespaceBindingKindValue::Type => "type",
        NamespaceBindingKindValue::Value => "value",
        NamespaceBindingKindValue::Macro => "macro",
    }
}

pub(super) fn filter(value: NameNamespaceFilterKind) -> &'static str {
    match value {
        NameNamespaceFilterKind::Any => "any",
        NameNamespaceFilterKind::Type => "type",
        NameNamespaceFilterKind::Value => "value",
        NameNamespaceFilterKind::Macro => "macro",
        NameNamespaceFilterKind::TypeOrValue => "type-or-value",
    }
}

pub(super) fn root(scope: &NameResolutionScopeKey) -> NameResolutionScopeKey {
    NameResolutionScopeKey {
        scope_compilation_unit_key: scope.scope_compilation_unit_key.clone(),
        scope_semantic_path_parts: vec![],
    }
}

pub(super) struct ContractInputCodec {
    pub targets: BTreeMap<String, NameSymbolTargetKey>,
    pub scopes: BTreeMap<String, NameResolutionScopeKey>,
    pub originals: BTreeMap<(String, String, String, String), NameResolvedBindingRow>,
    pub imports: BTreeMap<String, NameImportRequestRow>,
}

impl ContractInputCodec {
    pub fn load(
        connection: &duckdb::Connection,
        input: &NameResolutionInputRows,
    ) -> Result<Self, NameResolutionStageError> {
        connection
            .execute_batch(include_str!("namespace-contract-input-schema.sql"))
            .map_err(error)?;
        let mut result = Self {
            targets: BTreeMap::new(),
            scopes: BTreeMap::new(),
            originals: BTreeMap::new(),
            imports: BTreeMap::new(),
        };
        input.all_name_scope_rows().try_for_each(|row| {
            let scope = &row.scope_module_identity_key;
            result.scopes.insert(key(scope),scope.clone());
            connection.execute("INSERT INTO \"namespace-scope-context-records\" VALUES ('contract','0',?,?,?,?,?,?)",
                duckdb::params![key(&scope.scope_compilation_unit_key),key(scope),row.scope_parent_identity_key.as_ref().map(key),row.scope_parent_lookup_flag,row.scope_module_boundary_flag,row.scope_implicit_prelude_key.as_ref().map(key)]).map_err(error)?;
            Ok::<_,NameResolutionStageError>(())
        })?;
        let roots = input
            .all_name_scope_rows()
            .filter(|row| {
                row.scope_module_identity_key
                    .scope_semantic_path_parts
                    .is_empty()
            })
            .map(|row| &row.scope_module_identity_key);
        let mut target_contexts = BTreeMap::new();
        roots.for_each(|scope| {
            let target = NameSymbolTargetKey::CrateRoot(scope.scope_compilation_unit_key.clone());
            result.targets.insert(key(&target), target.clone());
            target_contexts.insert(key(&target), (Some(key(scope)), None::<String>, true));
        });
        input.all_name_binding_rows().try_for_each(|binding| {
            let target = key(&binding.binding_target_identity_key);
            result.targets.insert(target.clone(),binding.binding_target_identity_key.clone());
            let visibility = match &binding.binding_visibility_scope_kind {
                NameBindingVisibilityKind::Public=>None,
                NameBindingVisibilityKind::Restricted(scope)=>Some(key(scope)),
                NameBindingVisibilityKind::Invisible=>Some("invisible".into()),
            };
            target_contexts.entry(target.clone()).or_insert((binding.binding_target_child_scope.as_ref().map(key),visibility,binding.binding_target_child_scope.is_some()));
            let name = binding.binding_resolved_name_text.trim_start_matches("r#");
            connection.execute("INSERT INTO \"namespace-declaration-visibility-records\" VALUES ('contract','0',?,?,?,?,?,'',?)",
                duckdb::params![key(&binding.binding_lexical_scope_key),name,namespace(binding.binding_namespace_kind_value),target,binding.binding_selection_priority_kind as i32,
                    match &binding.binding_visibility_scope_kind {NameBindingVisibilityKind::Public=>None,NameBindingVisibilityKind::Restricted(scope)=>Some(key(scope)),NameBindingVisibilityKind::Invisible=>Some("invisible".into())}]).map_err(error)?;
            result.originals.insert((key(&binding.binding_lexical_scope_key),name.into(),namespace(binding.binding_namespace_kind_value).into(),target.clone()),binding.clone());
            connection.execute("INSERT INTO \"namespace-binding-declaration-records\" VALUES ('contract','0',?,?,?,?,?, '')",
                duckdb::params![key(&binding.binding_lexical_scope_key),name,namespace(binding.binding_namespace_kind_value),target,binding.binding_selection_priority_kind as i32]).map_err(error)?;
            Ok::<_,NameResolutionStageError>(())
        })?;
        target_contexts.into_iter().try_for_each(|(target,(child,visibility,identity))| {
            connection.execute("INSERT INTO \"namespace-target-context-records\" VALUES ('contract','0',?,?,?,?)",duckdb::params![target,child,visibility,identity]).map_err(error)?;
            Ok::<_,NameResolutionStageError>(())
        })?;
        input
            .name_signature_path_rows
            .iter()
            .try_for_each(|path| result.insert_path(connection, input, path))?;
        input.name_import_request_rows.iter().try_for_each(|import| {
            let request = key(&import.import_site_occurrence_key);
            result.imports.insert(request.clone(),import.clone());
            connection.execute("INSERT INTO \"namespace-import-visibility-records\" VALUES ('contract','0',?,?)",
                duckdb::params![request,match &import.import_visibility_scope_kind {NameBindingVisibilityKind::Public=>None,NameBindingVisibilityKind::Restricted(scope)=>Some(key(scope)),NameBindingVisibilityKind::Invisible=>Some("invisible".into())}]).map_err(error)?;
            result.insert_path(connection,input,&NamePathRequestRow {
                path_site_occurrence_key:import.import_site_occurrence_key.clone(),path_lexical_scope_key:import.import_lexical_scope_key.clone(),
                path_source_segment_rows:import.import_path_segment_rows.clone(),path_requested_namespace_kind:NameNamespaceFilterKind::Any,
                path_resolution_source_kind:import.import_resolution_source_kind,path_unavailable_reason_kind:None,
            })?;
            connection.execute("INSERT INTO \"namespace-import-request-records\" VALUES ('contract','0',?,?,?,?)",
                duckdb::params![request,key(&import.import_lexical_scope_key),import.import_binding_alias_text,import.import_glob_expansion_flag]).map_err(error)?;
            Ok::<_,NameResolutionStageError>(())
        })?;
        Ok(result)
    }

    fn insert_path(
        &self,
        connection: &duckdb::Connection,
        input: &NameResolutionInputRows,
        path: &NamePathRequestRow,
    ) -> Result<(), NameResolutionStageError> {
        let edition = match input.unit_rust_edition_kind {
            RustSourceEditionKind::Edition2015 => 2015,
            RustSourceEditionKind::Edition2018 => 2018,
            RustSourceEditionKind::Edition2021 => 2021,
            RustSourceEditionKind::Edition2024 => 2024,
        };
        let request = key(&path.path_site_occurrence_key);
        connection.execute("INSERT INTO \"namespace-path-source-requests\" VALUES ('contract','0',?,?,?,?,?,?,?,?,?)",
            duckdb::params![request,key(&path.path_lexical_scope_key),key(&root(&path.path_lexical_scope_key)),input.unit_extern_prelude_key.as_ref().map(key),edition,
                match path.path_resolution_source_kind {NameResolutionPathKind::Import=>"import",NameResolutionPathKind::Signature=>"signature"},
                filter(path.path_requested_namespace_kind),path.path_source_segment_rows.len() as i32,path.path_unavailable_reason_kind.map(|reason|format!("{reason:?}"))]).map_err(error)?;
        path.path_source_segment_rows.iter().enumerate().try_for_each(|(position,segment)| {
            let (kind,text) = match segment {
                NamePathSegmentKind::CrateRoot=>("crate",String::new()),NamePathSegmentKind::SelfModule=>("self",String::new()),NamePathSegmentKind::ParentModule=>("super",String::new()),
                NamePathSegmentKind::GlobalRoot=>("global",String::new()),NamePathSegmentKind::DefiningCrate(unit)=>("defining",key(&NameResolutionScopeKey {scope_compilation_unit_key:unit.clone(),scope_semantic_path_parts:vec![]})),
                NamePathSegmentKind::Name(name)=>("name",name.trim_start_matches("r#").into()),
            };
            connection.execute("INSERT INTO \"namespace-path-segment-records\" VALUES ('contract','0',?,?,?,?)",duckdb::params![request,position as i32,kind,text]).map_err(error)?;
            Ok::<_,NameResolutionStageError>(())
        })
    }

    pub fn binding(
        &self,
        scope: String,
        name: String,
        namespace: String,
        target: String,
        priority: i32,
        evidence: String,
    ) -> Result<NameResolvedBindingRow, NameResolutionStageError> {
        if let Some(original) = self.originals.get(&(
            scope.clone(),
            name.clone(),
            namespace.clone(),
            target.clone(),
        )) && original.binding_selection_priority_kind as i32 == priority
        {
            return Ok(original.clone());
        }
        let import = self
            .imports
            .get(&evidence)
            .ok_or_else(|| error("binding import evidence missing"))?;
        let original = self
            .originals
            .values()
            .find(|binding| key(&binding.binding_target_identity_key) == target)
            .ok_or_else(|| error("binding target metadata missing"))?;
        Ok(NameResolvedBindingRow {
            binding_lexical_scope_key: self
                .scopes
                .get(&scope)
                .cloned()
                .ok_or_else(|| error("binding lexical scope missing"))?,
            binding_resolved_name_text: name,
            binding_namespace_kind_value: match namespace.as_str() {
                "type" => NamespaceBindingKindValue::Type,
                "value" => NamespaceBindingKindValue::Value,
                "macro" => NamespaceBindingKindValue::Macro,
                _ => return Err(error("invalid binding namespace")),
            },
            binding_target_identity_key: self
                .targets
                .get(&target)
                .cloned()
                .ok_or_else(|| error("binding target missing"))?,
            binding_target_child_scope: original.binding_target_child_scope.clone(),
            binding_visibility_scope_kind: import.import_visibility_scope_kind.clone(),
            binding_selection_priority_kind: if priority == 2 {
                NameBindingPriorityKind::Glob
            } else {
                NameBindingPriorityKind::Explicit
            },
            binding_import_evidence_keys: vec![import.import_site_occurrence_key.clone()],
        })
    }
}

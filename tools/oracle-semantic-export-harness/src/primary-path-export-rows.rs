use hir::{Adt, HasSource, ItemInNs, Module, ModuleDef, Name, PathResolution, Semantics};
use ide_db::RootDatabase;
use parseltongue_core_row_types::SiteUnresolvedReasonKind;
use project_model::{
    CargoConfig, CargoFeatures, ProjectManifest, ProjectWorkspace, ProjectWorkspaceKind,
    RustLibSource,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    path::PathBuf,
};
use syntax::{AstNode, SyntaxNode, ast};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleWorkspaceExportConfig {
    pub workspace_source_root_path: PathBuf,
    pub oracle_macro_cache_manifest: Option<PathBuf>,
    pub oracle_target_triple_value: Option<String>,
    pub oracle_enabled_feature_values: Vec<String>,
    pub oracle_default_features_flag: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleSemanticExportError {
    pub error_context_detail_text: String,
}

impl fmt::Display for OracleSemanticExportError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.write_str(&self.error_context_detail_text)
    }
}

impl std::error::Error for OracleSemanticExportError {}

type OraclePackageRootIndex = BTreeMap<(String, String), String>;

pub fn export_primary_semantic_paths(
    request: &OracleWorkspaceExportConfig,
) -> Result<Vec<serde_json::Value>, OracleSemanticExportError> {
    if request.oracle_macro_cache_manifest.is_some() {
        return Err(build_oracle_export_error(
            "cached proc-macro injection is not implemented",
        ));
    }
    let root = request
        .workspace_source_root_path
        .canonicalize()
        .map_err(build_oracle_export_error)?;
    let root = vfs::AbsPathBuf::try_from(
        root.to_str()
            .ok_or_else(|| build_oracle_export_error("workspace path is not UTF-8"))?,
    )
    .map_err(|_| build_oracle_export_error("workspace root is not an absolute UTF-8 path"))?;
    let manifest = ProjectManifest::discover_single(&root).map_err(build_oracle_export_error)?;
    let configuration = build_primary_cargo_configuration(request, &root);
    let workspace = ProjectWorkspace::load(manifest, &configuration, &|_| {})
        .map_err(build_oracle_export_error)?;
    let package_roots: OraclePackageRootIndex = match &workspace.kind {
        ProjectWorkspaceKind::Cargo { cargo, .. } => cargo
            .packages()
            .flat_map(|package| {
                let data = &cargo[package];
                data.targets.iter().map(move |target| {
                    (
                        (cargo[*target].root.to_string(), data.name.clone()),
                        data.id.to_string(),
                    )
                })
            })
            .collect(),
        _ => return Err(build_oracle_export_error("a Cargo workspace is required")),
    };
    let selected: BTreeSet<String> = match &workspace.kind {
        ProjectWorkspaceKind::Cargo { cargo, .. } => cargo
            .packages()
            .filter(|package| cargo[*package].is_member)
            .flat_map(|package| cargo[package].targets.iter())
            .map(|target| &cargo[*target])
            .filter(|target| matches!(target.kind.as_cargo_target(), Some("lib" | "bin")))
            .map(|target| target.root.to_string())
            .collect(),
        _ => return Err(build_oracle_export_error("a Cargo workspace is required")),
    };
    let exclusions = match &workspace.kind {
        ProjectWorkspaceKind::Cargo { cargo, .. } => vec![
            root.join("target"),
            root.join(".git"),
            cargo.target_directory().to_path_buf(),
        ],
        _ => vec![root.join("target"), root.join(".git")],
    };
    let loaded = crate::primary_workspace_loader_adapter::load_primary_workspace_database(
        workspace,
        &configuration,
        &exclusions,
    )?;
    let database = loaded.oracle_database_owned_value;
    let vfs = loaded.oracle_source_virtual_files;
    let _server = loaded.oracle_macro_server_handle;
    let semantics = Semantics::new(&database);
    let modules: Vec<_> = hir::Crate::all(&database)
        .into_iter()
        .filter(|krate| selected.contains(&vfs.file_path(krate.root_file(&database)).to_string()))
        .flat_map(|krate| collect_oracle_module_contexts(krate.root_module(&database), &database))
        .collect();
    let contexts = modules.iter().fold(
        BTreeMap::<String, BTreeSet<String>>::new(),
        |mut contexts, module| {
            let source = module.definition_source(&database);
            semantics.parse_or_expand(source.file_id);
            contexts
                .entry(read_oracle_context_key(*module, &semantics))
                .or_default()
                .insert(format!("{module:?}"));
            contexts
        },
    );
    let mut rows = Vec::new();
    for module in modules {
        let owner = build_oracle_compilation_record(module, &database, &vfs, &package_roots)?;
        let source = module.definition_source(&database);
        let tree = source.value.node();
        semantics.parse_or_expand(source.file_id);
        let shared = contexts
            .get(&read_oracle_context_key(module, &semantics))
            .is_some_and(|contexts| contexts.len() > 1);
        for path in tree
            .descendants()
            .filter_map(ast::Path::cast)
            .filter(|path| check_oracle_signature_path(path, module, &database))
        {
            let range = semantics.original_range(path.syntax());
            let file = vfs.file_path(range.file_id.file_id(&database)).to_string();
            let targets =
                build_oracle_path_targets(module, &path, &semantics, &vfs, &package_roots, shared)?;
            let targets: BTreeMap<_, _> = targets
                .into_iter()
                .map(|target| (target.to_string(), target))
                .collect();
            let reason = targets
                .is_empty()
                .then_some(SiteUnresolvedReasonKind::ExternalSourceMissing);
            rows.push(json!({
                "unit_source_root_path":vfs.file_path(module.krate(&database).root_file(&database)).to_string(),
                "site_compilation_scope_record":owner,
                "site_source_absolute_path":file,
                "site_source_range_start":u32::from(range.range.start()),
                "site_source_range_end":u32::from(range.range.end()),
                "path_site_source_text":path.syntax().text().to_string(),
                "path_target_symbol_rows":targets.into_values().collect::<Vec<_>>(),
                "path_unresolved_reason_kind":reason.map(read_oracle_reason_label),
            }));
        }
    }
    rows.sort_by_cached_key(Value::to_string);
    rows.dedup();
    Ok(rows)
}

fn build_oracle_compilation_record(
    module: Module,
    database: &RootDatabase,
    files: &vfs::Vfs,
    packages: &OraclePackageRootIndex,
) -> Result<Value, OracleSemanticExportError> {
    let krate = module.krate(database);
    let root = files.file_path(krate.root_file(database)).to_string();
    let name = krate
        .display_name(database)
        .map(|name| name.canonical_name().to_string())
        .ok_or_else(|| build_oracle_export_error("a compilation unit has no target name"))?;
    let package_name = match krate.origin(database) {
        ide_db::base_db::CrateOrigin::Local {
            name: Some(name), ..
        }
        | ide_db::base_db::CrateOrigin::Library { name, .. }
        | ide_db::base_db::CrateOrigin::Rustc { name } => name.to_string(),
        _ => name.clone(),
    };
    let package = packages
        .get(&(root.clone(), package_name))
        .cloned()
        .or_else(|| {
            krate
                .is_builtin(database)
                .then(|| format!("sysroot+{root}#{name}"))
        })
        .ok_or_else(|| {
            build_oracle_export_error(format!("no Cargo package identity for {root}"))
        })?;
    Ok(json!({
        "unit_source_root_path":root,
        "unit_cargo_package_identifier":package,
        "unit_cargo_target_name":name,
        "unit_cargo_target_kind":krate.target_kind(database).as_cargo_target().unwrap_or("lib"),
        "unit_rust_edition_value":krate.edition(database).to_string(),
        "unit_module_path_values":module.path_segments(database)
            .map(|name|name.display(database,krate.edition(database)).to_string()).collect::<Vec<_>>(),
    }))
}

fn read_oracle_context_key(module: Module, semantics: &Semantics<'_, RootDatabase>) -> String {
    let source = module.definition_source(semantics.db);
    let range = semantics.original_range(&source.value.node());
    format!(
        "{:?}:{:?}",
        range.file_id.file_id(semantics.db),
        source.value.node().text_range()
    )
}

fn build_oracle_path_targets(
    module: Module,
    path: &ast::Path,
    semantics: &Semantics<'_, RootDatabase>,
    files: &vfs::Vfs,
    packages: &OraclePackageRootIndex,
    shared: bool,
) -> Result<Vec<Value>, OracleSemanticExportError> {
    if !shared {
        return read_oracle_namespace_paths(path, semantics)
            .into_iter()
            .map(|(namespace, resolution)| {
                build_oracle_resolution_target(resolution, semantics, files, packages).map(
                    |target| {
                        target.map(|mut target| {
                            target["target_namespace_kind_value"] = json!(namespace);
                            target
                        })
                    },
                )
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|targets| targets.into_iter().flatten().collect());
    }
    if let Some(target) = resolve_oracle_generic_target(module, path, semantics, files, packages)? {
        return Ok(vec![target]);
    }
    if let Some(definitions) = resolve_oracle_associated_paths(module, path, semantics)? {
        return definitions
            .into_iter()
            .map(|definition| build_oracle_symbol_target(definition, semantics, files, packages))
            .collect::<Result<Vec<_>, _>>()
            .map(|targets| targets.into_iter().flatten().collect());
    }
    if let Some(definitions) = resolve_oracle_module_paths(module, path, semantics.db) {
        return definitions
            .into_iter()
            .map(|definition| build_oracle_symbol_target(definition, semantics, files, packages))
            .collect::<Result<Vec<_>, _>>()
            .map(|targets| targets.into_iter().flatten().collect());
    }
    let range = semantics.original_range(path.syntax());
    Err(build_oracle_export_error(format!(
        "module-scoped API cannot resolve shared-context path {} in {}",
        path.syntax().text(),
        files.file_path(range.file_id.file_id(semantics.db))
    )))
}

fn read_oracle_namespace_paths<'db>(
    _path: &ast::Path,
    _semantics: &Semantics<'db, RootDatabase>,
) -> Vec<(&'static str, PathResolution<'db>)> {
    todo!()
}

fn resolve_oracle_associated_paths(
    _module: Module,
    _path: &ast::Path,
    _semantics: &Semantics<'_, RootDatabase>,
) -> Result<Option<Vec<ModuleDef>>, OracleSemanticExportError> {
    todo!()
}

fn read_oracle_source_syntax<Definition: HasSource>(
    definition: Definition,
    semantics: &Semantics<'_, RootDatabase>,
) -> Option<SyntaxNode> {
    semantics
        .source(definition)
        .map(|source| source.value.syntax().clone())
}

fn read_oracle_generic_source(
    definition: hir::GenericDef,
    semantics: &Semantics<'_, RootDatabase>,
) -> Option<SyntaxNode> {
    match definition {
        hir::GenericDef::Function(definition) => read_oracle_source_syntax(definition, semantics),
        hir::GenericDef::Adt(Adt::Struct(definition)) => {
            read_oracle_source_syntax(definition, semantics)
        }
        hir::GenericDef::Adt(Adt::Enum(definition)) => {
            read_oracle_source_syntax(definition, semantics)
        }
        hir::GenericDef::Adt(Adt::Union(definition)) => {
            read_oracle_source_syntax(definition, semantics)
        }
        hir::GenericDef::Trait(definition) => read_oracle_source_syntax(definition, semantics),
        hir::GenericDef::TypeAlias(definition) => read_oracle_source_syntax(definition, semantics),
        hir::GenericDef::Impl(definition) => read_oracle_source_syntax(definition, semantics),
        hir::GenericDef::Const(definition) => read_oracle_source_syntax(definition, semantics),
        hir::GenericDef::Static(definition) => read_oracle_source_syntax(definition, semantics),
    }
}

fn resolve_oracle_generic_target(
    module: Module,
    path: &ast::Path,
    semantics: &Semantics<'_, RootDatabase>,
    files: &vfs::Vfs,
    packages: &OraclePackageRootIndex,
) -> Result<Option<Value>, OracleSemanticExportError> {
    let Some(ast::PathSegmentKind::Name(name)) = path.segment().and_then(|segment| segment.kind())
    else {
        return Ok(None);
    };
    if path.qualifier().is_some() {
        return Ok(None);
    }
    let site = semantics.original_range(path.syntax());
    let mut owners: Vec<_> = module
        .declarations(semantics.db)
        .into_iter()
        .filter_map(|definition| match definition {
            ModuleDef::Function(definition) => Some(hir::GenericDef::Function(definition)),
            ModuleDef::Adt(definition) => Some(hir::GenericDef::Adt(definition)),
            ModuleDef::Trait(definition) => Some(hir::GenericDef::Trait(definition)),
            ModuleDef::TypeAlias(definition) => Some(hir::GenericDef::TypeAlias(definition)),
            ModuleDef::Const(definition) => Some(hir::GenericDef::Const(definition)),
            ModuleDef::Static(definition) => Some(hir::GenericDef::Static(definition)),
            _ => None,
        })
        .chain(
            hir::Impl::all_in_module(semantics.db, module)
                .into_iter()
                .map(hir::GenericDef::Impl),
        )
        .filter_map(|definition| {
            let node = read_oracle_generic_source(definition, semantics)?;
            let range = semantics.original_range(&node);
            (range.file_id == site.file_id && range.range.contains_range(site.range))
                .then_some((node.text_range().len(), definition))
        })
        .collect();
    owners.sort_by_key(|(length, _)| *length);
    let parameter = owners
        .into_iter()
        .flat_map(|(_, definition)| definition.params(semantics.db))
        .find(|parameter| parameter.name(semantics.db) == Name::new_root(name.text()));
    match parameter {
        Some(hir::GenericParam::TypeParam(parameter)) => build_oracle_resolution_target(
            PathResolution::TypeParam(parameter),
            semantics,
            files,
            packages,
        ),
        Some(hir::GenericParam::ConstParam(parameter)) => build_oracle_resolution_target(
            PathResolution::ConstParam(parameter),
            semantics,
            files,
            packages,
        ),
        _ => Ok(None),
    }
}

fn build_oracle_resolution_target(
    resolution: PathResolution,
    semantics: &Semantics<'_, RootDatabase>,
    files: &vfs::Vfs,
    packages: &OraclePackageRootIndex,
) -> Result<Option<Value>, OracleSemanticExportError> {
    let (target, module) = match resolution {
        PathResolution::Def(definition) => {
            return build_oracle_symbol_target(definition, semantics, files, packages);
        }
        PathResolution::TypeParam(parameter) => (
            build_oracle_target_source(parameter.merge(), "TypeParam", semantics, files),
            parameter.module(semantics.db),
        ),
        PathResolution::ConstParam(parameter) => (
            build_oracle_target_source(parameter.merge(), "ConstParam", semantics, files),
            parameter.module(semantics.db),
        ),
        PathResolution::SelfType(definition) => (
            build_oracle_target_source(definition, "SelfType", semantics, files),
            definition.module(semantics.db),
        ),
        _ => return Ok(None),
    };
    target
        .map(|mut target| {
            target["target_compilation_scope_record"] =
                build_oracle_compilation_record(module, semantics.db, files, packages)?;
            Ok(target)
        })
        .transpose()
}

fn build_primary_cargo_configuration(
    request: &OracleWorkspaceExportConfig,
    root: &vfs::AbsPathBuf,
) -> CargoConfig {
    CargoConfig {
        all_targets: false,
        set_test: false,
        target: request.oracle_target_triple_value.clone(),
        features: CargoFeatures::Selected {
            features: request.oracle_enabled_feature_values.clone(),
            no_default_features: !request.oracle_default_features_flag,
        },
        sysroot: Some(RustLibSource::Discover),
        metadata_extra_args: vec!["--offline".into()],
        extra_includes: vec![root.clone()],
        extra_env: [("CARGO_NET_OFFLINE".into(), Some("true".into()))]
            .into_iter()
            .collect(),
        cfg_overrides: project_model::CfgOverrides {
            global: cfg::CfgDiff::new(
                vec![],
                vec![cfg::CfgAtom::Flag(hir::Symbol::intern("rust_analyzer"))],
            ),
            ..project_model::CfgOverrides::default()
        },
        ..CargoConfig::default()
    }
}

pub(super) fn build_oracle_export_error(error: impl fmt::Display) -> OracleSemanticExportError {
    OracleSemanticExportError {
        error_context_detail_text: error.to_string(),
    }
}

fn read_oracle_reason_label(reason: SiteUnresolvedReasonKind) -> &'static str {
    match reason {
        SiteUnresolvedReasonKind::CfgInactive => "cfg_inactive",
        SiteUnresolvedReasonKind::BuildScript => "build_script",
        SiteUnresolvedReasonKind::ProcMacro => "proc_macro",
        SiteUnresolvedReasonKind::ExternalSourceMissing => "external_source_missing",
        SiteUnresolvedReasonKind::UnknownReceiver => "unknown_receiver",
        SiteUnresolvedReasonKind::TraitSolverNeeded => "trait_solver_needed",
    }
}

fn collect_oracle_module_contexts(module: Module, database: &RootDatabase) -> Vec<Module> {
    std::iter::once(module)
        .chain(
            module
                .children(database)
                .flat_map(|child| collect_oracle_module_contexts(child, database)),
        )
        .collect()
}

fn check_oracle_meta_enabled(meta: ast::Meta, options: &cfg::CfgOptions) -> bool {
    match meta {
        ast::Meta::CfgMeta(meta) => {
            meta.cfg_predicate()
                .and_then(|predicate| options.check(&cfg::CfgExpr::parse_from_ast(predicate)))
                == Some(true)
        }
        ast::Meta::CfgAttrMeta(meta) => {
            meta.cfg_predicate()
                .and_then(|predicate| options.check(&cfg::CfgExpr::parse_from_ast(predicate)))
                != Some(true)
                || meta
                    .metas()
                    .all(|meta| check_oracle_meta_enabled(meta, options))
        }
        _ => true,
    }
}

fn check_oracle_signature_path(path: &ast::Path, module: Module, database: &RootDatabase) -> bool {
    if path
        .syntax()
        .parent()
        .is_some_and(|parent| ast::Path::can_cast(parent.kind()))
    {
        return false;
    }
    let ancestors: Vec<_> = path.syntax().ancestors().collect();
    let inline = ancestors
        .iter()
        .filter_map(|node| ast::Module::cast(node.clone()))
        .find(|module| module.item_list().is_some());
    if inline.is_some_and(|inline| {
        !module.is_inline(database)
            || module.definition_source(database).value.node().text_range()
                != inline.syntax().text_range()
    }) {
        return false;
    }
    if !ancestors.iter().all(|node| {
        node.children()
            .filter_map(ast::Attr::cast)
            .filter_map(|attribute| attribute.meta())
            .all(|meta| check_oracle_meta_enabled(meta, module.krate(database).cfg(database)))
    }) {
        return false;
    }
    if ancestors.iter().any(|node| ast::Use::can_cast(node.kind())) {
        return true;
    }
    if let Some(function) = ancestors
        .iter()
        .find_map(|node| ast::Fn::cast(node.clone()))
        && function.body().is_some_and(|body| {
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

fn resolve_oracle_module_paths(
    module: Module,
    path: &ast::Path,
    database: &RootDatabase,
) -> Option<Vec<ModuleDef>> {
    let parents: Vec<_> = path
        .syntax()
        .ancestors()
        .filter_map(ast::UseTree::cast)
        .skip(1)
        .collect();
    let segments = parents
        .into_iter()
        .rev()
        .filter_map(|tree| tree.path())
        .chain(std::iter::once(path.clone()))
        .flat_map(|path| path.segments().collect::<Vec<_>>())
        .map(|segment| segment.kind())
        .collect::<Option<Vec<_>>>()?;
    let (module, names) =
        segments
            .into_iter()
            .try_fold(
                (module, Vec::new()),
                |(module, names), segment| match segment {
                    ast::PathSegmentKind::CrateKw => {
                        Some((module.krate(database).root_module(database), names))
                    }
                    ast::PathSegmentKind::SelfKw => Some((module, names)),
                    ast::PathSegmentKind::SuperKw => Some((module.parent(database)?, names)),
                    ast::PathSegmentKind::Name(name) => Some((
                        module,
                        names
                            .into_iter()
                            .chain(std::iter::once(Name::new_root(name.text())))
                            .collect(),
                    )),
                    _ => None,
                },
            )?;
    if names.is_empty() {
        return Some(vec![ModuleDef::Module(module)]);
    }
    for end in 1..names.len() {
        if !module
            .resolve_mod_path(database, names[..end].iter().cloned())?
            .any(|item| matches!(item, ItemInNs::Types(ModuleDef::Module(_))))
        {
            return None;
        }
    }
    let definitions: Vec<_> = module
        .resolve_mod_path(database, names)?
        .filter(|item| {
            let imported = path
                .syntax()
                .ancestors()
                .any(|node| ast::UseTree::can_cast(node.kind()));
            let value = path
                .syntax()
                .parent()
                .is_some_and(|node| ast::PathExpr::can_cast(node.kind()));
            imported
                || matches!(
                    (item, value),
                    (ItemInNs::Types(_), false) | (ItemInNs::Values(_), true)
                )
        })
        .map(ItemInNs::into_module_def)
        .collect();
    (!definitions.is_empty()).then_some(definitions)
}

fn build_oracle_source_record(
    node: &SyntaxNode,
    kind: &str,
    semantics: &Semantics<'_, RootDatabase>,
    vfs: &vfs::Vfs,
) -> Value {
    let range = semantics.original_range(node);
    json!({
        "target_source_file_path":vfs.file_path(range.file_id.file_id(semantics.db)).to_string(),
        "target_source_range_start":u32::from(range.range.start()),
        "target_source_range_end":u32::from(range.range.end()),
        "target_symbol_kind_value":kind,
    })
}

fn build_oracle_target_source<Definition: HasSource>(
    definition: Definition,
    kind: &str,
    semantics: &Semantics<'_, RootDatabase>,
    vfs: &vfs::Vfs,
) -> Option<Value> {
    let source = semantics.source(definition)?;
    Some(build_oracle_source_record(
        source.value.syntax(),
        kind,
        semantics,
        vfs,
    ))
}

fn build_oracle_symbol_target(
    definition: ModuleDef,
    semantics: &Semantics<'_, RootDatabase>,
    vfs: &vfs::Vfs,
    packages: &OraclePackageRootIndex,
) -> Result<Option<Value>, OracleSemanticExportError> {
    let target = match definition {
        ModuleDef::Module(module) => {
            if let Some(source) = module.declaration_source(semantics.db) {
                semantics.parse_or_expand(source.file_id);
                Some(build_oracle_source_record(
                    source.value.syntax(),
                    "Module",
                    semantics,
                    vfs,
                ))
            } else {
                Some(
                    json!({"target_source_file_path":vfs.file_path(module.krate(semantics.db).root_file(semantics.db)).to_string(),"target_source_range_start":0,"target_source_range_end":0,"target_symbol_kind_value":"Crate"}),
                )
            }
        }
        ModuleDef::Function(definition) => {
            build_oracle_target_source(definition, "Function", semantics, vfs)
        }
        ModuleDef::Adt(Adt::Struct(definition)) => {
            build_oracle_target_source(definition, "Struct", semantics, vfs)
        }
        ModuleDef::Adt(Adt::Enum(definition)) => {
            build_oracle_target_source(definition, "Enum", semantics, vfs)
        }
        ModuleDef::Adt(Adt::Union(definition)) => {
            build_oracle_target_source(definition, "Union", semantics, vfs)
        }
        ModuleDef::EnumVariant(definition) => {
            build_oracle_target_source(definition, "Variant", semantics, vfs)
        }
        ModuleDef::Const(definition) => {
            build_oracle_target_source(definition, "Const", semantics, vfs)
        }
        ModuleDef::Static(definition) => {
            build_oracle_target_source(definition, "Static", semantics, vfs)
        }
        ModuleDef::Trait(definition) => {
            build_oracle_target_source(definition, "Trait", semantics, vfs)
        }
        ModuleDef::TypeAlias(definition) => {
            build_oracle_target_source(definition, "TypeAlias", semantics, vfs)
        }
        ModuleDef::Macro(definition) => {
            build_oracle_target_source(definition, "Macro", semantics, vfs)
        }
        ModuleDef::BuiltinType(definition) => Some(
            json!({"target_builtin_name_value":definition.name().as_str(),"target_symbol_kind_value":"Builtin"}),
        ),
    };
    target
        .map(|mut target| {
            let module = definition.module(semantics.db).or(match definition {
                ModuleDef::Module(module) => Some(module),
                _ => None,
            });
            target["target_compilation_scope_record"] = module
                .map(|module| build_oracle_compilation_record(module, semantics.db, vfs, packages))
                .transpose()?
                .unwrap_or(Value::Null);
            Ok(target)
        })
        .transpose()
}

#[cfg(test)]
mod oracle_path_contract_tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use std::{collections::BTreeSet, fs, path::Path};

    fn create_oracle_workspace_fixture(root: &Path, files: &[(&str, &str)]) {
        for (path, text) in files {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
    }

    fn build_oracle_fixture_config(root: &Path) -> OracleWorkspaceExportConfig {
        OracleWorkspaceExportConfig {
            workspace_source_root_path: root.to_owned(),
            oracle_macro_cache_manifest: None,
            oracle_target_triple_value: None,
            oracle_enabled_feature_values: Vec::new(),
            oracle_default_features_flag: true,
        }
    }

    #[test]
    fn test_nameres_matches_oracle_paths() {
        let directory = tempfile::tempdir().unwrap();
        let source = "pub mod defs { pub struct Present; }\nuse crate::defs::Present as Alias;\npub struct Wrap(pub Alias);\n#[cfg(test)] mod hidden;\n";
        create_oracle_workspace_fixture(
            directory.path(),
            &[
                (
                    "Cargo.toml",
                    "[package]\nname=\"paths\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
                ),
                ("src/lib.rs", source),
                ("src/hidden.rs", "pub struct Forbidden(pub Missing);"),
                ("tests/integration.rs", "pub struct Forbidden(pub Missing);"),
                ("examples/ignored.rs", "pub struct Forbidden(pub Missing);"),
            ],
        );
        let rows =
            export_primary_semantic_paths(&build_oracle_fixture_config(directory.path())).unwrap();
        let alias = rows
            .iter()
            .find(|row| row["path_site_source_text"] == "Alias")
            .unwrap();
        let start = source.rfind("Alias").unwrap();
        assert_eq!(alias["site_source_range_start"], start);
        assert_eq!(alias["site_source_range_end"], start + "Alias".len());
        let targets = alias["path_target_symbol_rows"].as_array().unwrap();
        assert!(
            targets
                .iter()
                .any(|target| target["target_source_file_path"]
                    == directory.path().join("src/lib.rs").to_str().unwrap()
                    && target["target_source_range_start"]
                        == source.find("pub struct Present;").unwrap()
                    && target["target_symbol_kind_value"] == "Struct")
        );
        assert!(
            rows.iter()
                .any(|row| row["path_site_source_text"] == "crate::defs::Present")
        );
        assert!(
            rows.iter()
                .all(|row| row["path_site_source_text"] != "Missing")
        );
    }

    #[test]
    fn test_shared_modules_preserve_context() {
        let directory = tempfile::tempdir().unwrap();
        create_oracle_workspace_fixture(
            directory.path(),
            &[
                (
                    "Cargo.toml",
                    "[workspace]\nmembers=[\"front\",\"back\",\"left\",\"right\"]\nresolver=\"3\"\n",
                ),
                (
                    "front/Cargo.toml",
                    "[package]\nname=\"front\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[dependencies]\nchosen={package=\"left\",path=\"../left\"}\n",
                ),
                (
                    "back/Cargo.toml",
                    "[package]\nname=\"back\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[dependencies]\nchosen={package=\"right\",path=\"../right\"}\n",
                ),
                (
                    "left/Cargo.toml",
                    "[package]\nname=\"left\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
                ),
                (
                    "right/Cargo.toml",
                    "[package]\nname=\"right\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
                ),
                (
                    "front/src/lib.rs",
                    "#[path=\"../../shared.rs\"] pub mod shared;",
                ),
                (
                    "back/src/lib.rs",
                    "#[path=\"../../shared.rs\"] pub mod shared;",
                ),
                ("left/src/lib.rs", "pub struct Choice;"),
                ("right/src/lib.rs", "pub struct Choice;"),
                ("shared.rs", "pub struct Wrap(pub chosen::Choice);"),
            ],
        );
        let rows =
            export_primary_semantic_paths(&build_oracle_fixture_config(directory.path())).unwrap();
        for (owner, dependency) in [("front", "left"), ("back", "right")] {
            let owner_path = directory.path().join(owner).join("src/lib.rs");
            let row = rows
                .iter()
                .find(|row| {
                    row["unit_source_root_path"] == owner_path.to_str().unwrap()
                        && row["path_site_source_text"] == "chosen::Choice"
                })
                .unwrap();
            let targets: BTreeSet<_> = row["path_target_symbol_rows"]
                .as_array()
                .unwrap()
                .iter()
                .map(|target| target["target_source_file_path"].as_str().unwrap())
                .collect();
            assert_eq!(
                targets,
                BTreeSet::from([directory
                    .path()
                    .join(dependency)
                    .join("src/lib.rs")
                    .to_str()
                    .unwrap()])
            );
        }
    }

    fn create_shared_scope_fixture(root: &Path, source: &str) {
        create_oracle_workspace_fixture(
            root,
            &[
                (
                    "Cargo.toml",
                    "[workspace]\nmembers=[\"front\",\"back\",\"left\",\"right\"]\nresolver=\"3\"\n",
                ),
                (
                    "front/Cargo.toml",
                    "[package]\nname=\"front\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[dependencies]\nchosen={package=\"left\",path=\"../left\"}\n",
                ),
                (
                    "back/Cargo.toml",
                    "[package]\nname=\"back\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[dependencies]\nchosen={package=\"right\",path=\"../right\"}\n",
                ),
                (
                    "left/Cargo.toml",
                    "[package]\nname=\"left\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
                ),
                (
                    "right/Cargo.toml",
                    "[package]\nname=\"right\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
                ),
                (
                    "front/src/lib.rs",
                    "#[path=\"../../shared.rs\"] pub mod shared;",
                ),
                (
                    "back/src/lib.rs",
                    "#[path=\"../../shared.rs\"] pub mod shared;",
                ),
                ("left/src/lib.rs", "pub mod nested { pub struct Choice; }"),
                ("right/src/lib.rs", "pub mod nested { pub struct Choice; }"),
                ("shared.rs", source),
            ],
        );
    }

    #[test]
    fn test_scope_unit_targets_distinct() {
        let directory = tempfile::tempdir().unwrap();
        create_oracle_workspace_fixture(
            directory.path(),
            &[
                (
                    "Cargo.toml",
                    "[package]\nname=\"twins\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[lib]\npath=\"src/lib.rs\"\n[[bin]]\nname=\"twins\"\npath=\"src/lib.rs\"\n",
                ),
                (
                    "src/lib.rs",
                    "pub struct Thing; pub struct Wrap(pub Thing); pub fn main() {}",
                ),
            ],
        );
        let rows =
            export_primary_semantic_paths(&build_oracle_fixture_config(directory.path())).unwrap();
        let things: Vec<_> = rows
            .iter()
            .filter(|row| row["path_site_source_text"] == "Thing")
            .collect();
        assert_eq!(things.len(), 2);
        let kinds: BTreeSet<_> = things
            .iter()
            .map(|row| {
                let scope = &row["site_compilation_scope_record"];
                assert_eq!(scope["unit_cargo_target_name"], "twins");
                assert!(
                    scope["unit_cargo_package_identifier"]
                        .as_str()
                        .unwrap()
                        .starts_with("path+file://")
                );
                let targets = row["path_target_symbol_rows"].as_array().unwrap();
                assert_eq!(targets.len(), 1);
                assert_eq!(targets[0]["target_compilation_scope_record"], *scope);
                scope["unit_cargo_target_kind"].as_str().unwrap()
            })
            .collect();
        assert_eq!(kinds, BTreeSet::from(["bin", "lib"]));
    }

    #[test]
    fn test_scope_namespaces_remain_separate() {
        let directory = tempfile::tempdir().unwrap();
        create_oracle_workspace_fixture(
            directory.path(),
            &[
                (
                    "Cargo.toml",
                    "[package]\nname=\"namespaces\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
                ),
                (
                    "src/lib.rs",
                    "pub struct Thing {} #[allow(non_snake_case)] pub fn Thing() -> Thing { Thing {} } pub struct Wrap(pub Thing);",
                ),
            ],
        );
        let rows =
            export_primary_semantic_paths(&build_oracle_fixture_config(directory.path())).unwrap();
        let things: Vec<_> = rows
            .iter()
            .filter(|row| row["path_site_source_text"] == "Thing")
            .collect();
        assert_eq!(things.len(), 2);
        for row in things {
            let targets = row["path_target_symbol_rows"].as_array().unwrap();
            assert_eq!(targets.len(), 1);
            assert_eq!(targets[0]["target_symbol_kind_value"], "Struct");
        }
    }

    #[test]
    fn test_scope_modules_retain_occurrences() {
        let directory = tempfile::tempdir().unwrap();
        create_oracle_workspace_fixture(
            directory.path(),
            &[
                (
                    "Cargo.toml",
                    "[package]\nname=\"modules\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
                ),
                (
                    "src/lib.rs",
                    "#[path=\"shared.rs\"] pub mod one; #[path=\"shared.rs\"] pub mod two;",
                ),
                (
                    "src/shared.rs",
                    "pub struct Thing; pub struct Wrap(pub Thing);",
                ),
            ],
        );
        let rows =
            export_primary_semantic_paths(&build_oracle_fixture_config(directory.path())).unwrap();
        let things: Vec<_> = rows
            .iter()
            .filter(|row| row["path_site_source_text"] == "Thing")
            .collect();
        assert_eq!(things.len(), 2);
        let scopes: BTreeSet<_> = things
            .iter()
            .map(|row| {
                let scope = &row["site_compilation_scope_record"];
                let targets = row["path_target_symbol_rows"].as_array().unwrap();
                assert_eq!(targets.len(), 1);
                assert_eq!(targets[0]["target_compilation_scope_record"], *scope);
                scope["unit_module_path_values"].to_string()
            })
            .collect();
        assert_eq!(
            scopes,
            BTreeSet::from([json!(["one"]).to_string(), json!(["two"]).to_string()])
        );
    }

    #[test]
    fn test_scope_generics_retain_context() {
        let directory = tempfile::tempdir().unwrap();
        let source = "pub struct Wrap<T>(pub T);";
        create_shared_scope_fixture(directory.path(), source);
        let rows =
            export_primary_semantic_paths(&build_oracle_fixture_config(directory.path())).unwrap();
        let paths: Vec<_> = rows
            .iter()
            .filter(|row| row["path_site_source_text"] == "T")
            .collect();
        assert_eq!(paths.len(), 2);
        for row in paths {
            let targets = row["path_target_symbol_rows"].as_array().unwrap();
            assert_eq!(targets.len(), 1);
            assert_eq!(targets[0]["target_symbol_kind_value"], "TypeParam");
            assert_eq!(
                targets[0]["target_source_range_start"],
                source.find("<T>").unwrap() + 1
            );
            assert_eq!(
                targets[0]["target_compilation_scope_record"],
                row["site_compilation_scope_record"]
            );
        }
    }

    #[test]
    fn test_scope_imports_retain_context() {
        let directory = tempfile::tempdir().unwrap();
        create_shared_scope_fixture(
            directory.path(),
            "use chosen::{nested::{Choice as Alias}}; pub struct Wrap(pub Alias);",
        );
        let rows =
            export_primary_semantic_paths(&build_oracle_fixture_config(directory.path())).unwrap();
        let paths: Vec<_> = rows
            .iter()
            .filter(|row| row["path_site_source_text"] == "Choice")
            .collect();
        assert_eq!(paths.len(), 2);
        for (owner, dependency) in [("front", "left"), ("back", "right")] {
            let root = directory.path().join(owner).join("src/lib.rs");
            let row = paths
                .iter()
                .find(|row| row["unit_source_root_path"] == root.to_str().unwrap())
                .unwrap();
            let targets = row["path_target_symbol_rows"].as_array().unwrap();
            assert_eq!(targets.len(), 1);
            assert_eq!(
                targets[0]["target_source_file_path"],
                directory
                    .path()
                    .join(dependency)
                    .join("src/lib.rs")
                    .to_str()
                    .unwrap()
            );
            assert_eq!(
                targets[0]["target_compilation_scope_record"]["unit_module_path_values"],
                json!(["nested"])
            );
        }
    }

    #[test]
    fn test_empty_oracle_paths_succeed() {
        let directory = tempfile::tempdir().unwrap();
        create_oracle_workspace_fixture(
            directory.path(),
            &[
                (
                    "Cargo.toml",
                    "[package]\nname=\"empty\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
                ),
                ("src/lib.rs", ""),
            ],
        );
        assert!(
            export_primary_semantic_paths(&build_oracle_fixture_config(directory.path()))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_missing_oracle_workspace_errors() {
        let directory = tempfile::tempdir().unwrap();
        let error = export_primary_semantic_paths(&build_oracle_fixture_config(directory.path()))
            .unwrap_err();
        assert!(!error.error_context_detail_text.is_empty());
    }
}

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
            let range = semantics.original_range(&source.value.node());
            contexts
                .entry(vfs.file_path(range.file_id.file_id(&database)).to_string())
                .or_default()
                .insert(format!("{:?}", module.krate(&database)));
            contexts
        },
    );
    let mut rows = Vec::new();
    for module in modules {
        let owner = build_oracle_compilation_record(module, &database, &vfs, &package_roots)?;
        let source = module.definition_source(&database);
        let tree = source.value.node();
        semantics.parse_or_expand(source.file_id);
        for path in tree
            .descendants()
            .filter_map(ast::Path::cast)
            .filter(|path| check_oracle_signature_path(path, module, &database))
        {
            let range = semantics.original_range(path.syntax());
            let file = vfs.file_path(range.file_id.file_id(&database)).to_string();
            let scoped = resolve_oracle_module_paths(module, &path, &database);
            let targets = if let Some(definitions) = scoped {
                definitions
                    .into_iter()
                    .filter_map(|definition| {
                        build_oracle_symbol_target(definition, &semantics, &vfs)
                    })
                    .collect::<Vec<_>>()
            } else if contexts
                .get(&file)
                .is_some_and(|contexts| contexts.len() > 1)
            {
                return Err(build_oracle_export_error(format!(
                    "module-scoped API cannot resolve multi-crate path {} in {file}",
                    path.syntax().text()
                )));
            } else {
                semantics
                    .resolve_path(&path)
                    .and_then(|resolution| match resolution {
                        PathResolution::Def(definition) => {
                            build_oracle_symbol_target(definition, &semantics, &vfs)
                        }
                        _ => None,
                    })
                    .into_iter()
                    .collect()
            };
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
    _module: Module,
    _database: &RootDatabase,
    _files: &vfs::Vfs,
    _packages: &OraclePackageRootIndex,
) -> Result<Value, OracleSemanticExportError> {
    todo!()
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
    let segments = path
        .segments()
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
) -> Option<Value> {
    match definition {
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
    }
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

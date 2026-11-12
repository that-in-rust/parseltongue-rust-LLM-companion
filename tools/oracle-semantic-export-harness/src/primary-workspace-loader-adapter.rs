use crate::primary_path_export_rows::{OracleSemanticExportError, build_oracle_export_error};
use hir_expand::proc_macro::ProcMacrosBuilder;
use ide_db::{ChangeWithProcMacros, RootDatabase, base_db::ProcMacroLoadingError};
use project_model::{CargoConfig, ProjectWorkspace};
use vfs::loader::{Handle, LoadingProgress};

pub(super) struct OracleLoadedWorkspaceState {
    pub oracle_database_owned_value: RootDatabase,
    pub oracle_source_virtual_files: vfs::Vfs,
    pub oracle_macro_server_handle: Option<proc_macro_api::ProcMacroClient>,
    pub oracle_macro_loading_rows: Vec<serde_json::Value>,
    pub oracle_source_omission_rows: Vec<serde_json::Value>,
}

pub(super) fn load_primary_workspace_database(
    workspace: ProjectWorkspace,
    configuration: &CargoConfig,
    exclusions: &[vfs::AbsPathBuf],
    cached_macros: &std::collections::BTreeMap<String, vfs::AbsPathBuf>,
) -> Result<OracleLoadedWorkspaceState, OracleSemanticExportError> {
    let (sender, receiver) = crossbeam_channel::unbounded();
    let mut loader = vfs_notify::NotifyHandle::spawn(sender);
    let mut files = vfs::Vfs::default();
    let server = workspace
        .find_sysroot_proc_macro_srv()
        .transpose()
        .map_err(build_oracle_export_error)?
        .map(|path| {
            proc_macro_api::ProcMacroClient::spawn(
                &path,
                &configuration.extra_env,
                workspace.toolchain.as_ref(),
                1,
            )
        })
        .transpose()
        .map_err(build_oracle_export_error)?;
    let (graph, macros) = workspace.to_crate_graph(
        &mut |path| {
            let contents = loader.load_sync(path);
            let path = vfs::VfsPath::from(path.to_path_buf());
            files.set_file_contents(path.clone(), contents);
            files
                .file_id(&path)
                .and_then(|(file, excluded)| (excluded == vfs::FileExcluded::No).then_some(file))
        },
        &configuration.extra_env,
    );
    let mut macro_loading = Vec::new();
    let macros: ProcMacrosBuilder = macros
        .into_iter()
        .map(|(krate, path)| {
            let source=files.file_path(graph[krate].basic.root_file_id).to_string();
            let cached=cached_macros.get(&source);
            let path=match cached {Some(path)=>Ok(path.clone()),None=>path.map(|(_,path)|path)};
            let loaded = path.and_then(|path| match &server {
                Some(server) => load_cargo::load_proc_macro(server, &path, &[]),
                None => Err(ProcMacroLoadingError::ProcMacroSrvError(
                    "no sysroot proc-macro server".into(),
                )),
            });
            macro_loading.push(serde_json::json!({
                "macro_source_root_path":files.file_path(graph[krate].basic.root_file_id).to_string(),
                "macro_loading_status_value":if loaded.is_ok() {"loaded"} else {"failed"},
                "macro_loading_origin_kind":if cached.is_some() {"cached"} else {"workspace"},
                "macro_loading_error_text":loaded.as_ref().err().map(ToString::to_string),
            }));
            (krate, loaded)
        })
        .collect();
    let folders = load_cargo::ProjectFolders::new(&[workspace], exclusions, None);
    loader.set_config(vfs::loader::Config {
        load: folders.load,
        watch: vec![],
        version: 0,
    });
    for message in receiver {
        match message {
            vfs::loader::Message::Progress {
                n_done: LoadingProgress::Finished,
                ..
            } => {
                break;
            }
            vfs::loader::Message::Loaded { files: loaded }
            | vfs::loader::Message::Changed { files: loaded } => {
                for (path, content) in loaded {
                    files.set_file_contents(path.into(), content);
                }
            }
            _ => {}
        }
    }
    let mut change = ChangeWithProcMacros::default();
    let mut source_omissions = Vec::new();
    for (_, file) in files.take_changes() {
        if let vfs::Change::Create(content, _) | vfs::Change::Modify(content, _) = file.change {
            match String::from_utf8(content) {
                Ok(content) => change.change_file(file.file_id, Some(content)),
                Err(error) => source_omissions.push(serde_json::json!({
                    "omitted_source_absolute_path":files.file_path(file.file_id).to_string(),
                    "omitted_source_reason_text":error.to_string(),
                })),
            }
        }
    }
    change.set_roots(folders.source_root_config.partition(&files));
    change.set_crate_graph(graph);
    change.set_proc_macros(macros);
    let mut database = RootDatabase::new(None);
    database.enable_proc_attr_macros();
    database.apply_change(change);
    Ok(OracleLoadedWorkspaceState {
        oracle_database_owned_value: database,
        oracle_source_virtual_files: files,
        oracle_macro_server_handle: server,
        oracle_macro_loading_rows: macro_loading,
        oracle_source_omission_rows: source_omissions,
    })
}

use crate::{OracleSemanticExportError, primary_path_export_rows::build_oracle_export_error};
use cfg::{CfgAtom, CfgOptions};
use ide_db::{
    RootDatabase,
    base_db::{Env, SourceDatabase},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn apply_recorded_script_options(
    unit: &Value,
    rows: &[Value],
    actual: &CfgOptions,
    environment: &Env,
    mut options: CfgOptions,
) -> Result<(CfgOptions, Option<Value>), OracleSemanticExportError> {
    if rows
        .iter()
        .all(|row| row["script_package_identifier_text"] != unit["unit_cargo_package_identifier"])
    {
        return Ok((options, None));
    }
    let mut matching = rows
        .iter()
        .filter(|row| {
            row["script_package_identifier_text"] == unit["unit_cargo_package_identifier"]
                && row["script_generated_output_path"].as_str()
                    == environment.get("OUT_DIR").as_deref()
        })
        .collect::<Vec<_>>();
    matching.sort_by_key(|row| row.to_string());
    matching.dedup();
    let [script] = matching.as_slice() else {
        return Err(build_oracle_export_error(
            "SECONDARY recorded output path missing or ambiguous",
        ));
    };
    let names =
        serde_json::from_value::<BTreeSet<String>>(script["script_emitted_env_names"].clone())
            .map_err(build_oracle_export_error)?;
    let emitted = names
        .into_iter()
        .map(|name| {
            let value = environment.get(&name).ok_or_else(|| {
                build_oracle_export_error(format!("SECONDARY recorded env missing: {name}"))
            })?;
            Ok((name, value))
        })
        .collect::<Result<BTreeMap<_, _>, OracleSemanticExportError>>()?;
    if script["script_emitted_env_digest"]
        != crate::oracle_artifact_cache_facade::build_recorded_env_digest(&emitted)
    {
        return Err(build_oracle_export_error(
            "SECONDARY recorded env digest differs",
        ));
    }
    let flags =
        serde_json::from_value::<BTreeSet<String>>(script["script_emitted_cfg_values"].clone())
            .map_err(build_oracle_export_error)?;
    let atoms = flags
        .iter()
        .map(|flag| {
            if let Some((key, value)) = flag.split_once('=') {
                let value = serde_json::from_str::<String>(value.trim())
                    .map_err(build_oracle_export_error)?;
                Ok(CfgAtom::KeyValue {
                    key: hir::Symbol::intern(key.trim()),
                    value: hir::Symbol::intern(&value),
                })
            } else {
                Ok(CfgAtom::Flag(hir::Symbol::intern(flag)))
            }
        })
        .collect::<Result<Vec<_>, OracleSemanticExportError>>()?;
    if atoms.iter().any(|atom| !actual.check_atom(atom)) {
        return Err(build_oracle_export_error("SECONDARY recorded cfg differs"));
    }
    atoms.into_iter().for_each(|atom| match atom {
        CfgAtom::Flag(key) => options.insert_atom(key),
        CfgAtom::KeyValue { key, value } => options.insert_key_value(key, value),
    });
    Ok((
        options,
        Some(json!({
            "delta_build_script_kind":"KL1",
            "delta_compilation_unit_key":unit["unit_compilation_identity_key"],
            "delta_semantic_identity_record":unit["unit_semantic_identity_record"],
            "delta_script_cfg_values":flags,
            "delta_script_env_names":script["script_emitted_env_names"],
            "delta_script_env_digest":script["script_emitted_env_digest"],
            "delta_script_output_path":script["script_generated_output_path"],
            "delta_generated_source_rows":[],
        })),
    ))
}

pub(super) fn append_generated_source_records(
    deltas: &mut [Value],
    database: &RootDatabase,
    files: &vfs::Vfs,
) -> Result<(), OracleSemanticExportError> {
    deltas.iter_mut().try_for_each(|delta| {
        let output = delta["delta_script_output_path"]
            .as_str()
            .ok_or_else(|| build_oracle_export_error("SECONDARY recorded output path missing"))?;
        let output = vfs::AbsPathBuf::try_from(output).map_err(|_| {
            build_oracle_export_error("SECONDARY recorded output path is not absolute UTF-8")
        })?;
        let sources = files
            .iter()
            .filter(|(_, path)| path.as_path().is_some_and(|path| path.starts_with(&output)))
            .map(|(file, path)| {
                let text = database.file_text(file);
                (
                    path.to_string(),
                    blake3::hash(text.text(database).as_bytes())
                        .to_hex()
                        .to_string(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        delta["delta_generated_source_rows"] = json!(
            sources
                .into_iter()
                .map(|(path, digest)| json!({
                    "file_source_absolute_path":path,"file_content_digest_full":digest,
                }))
                .collect::<Vec<_>>()
        );
        Ok(())
    })
}

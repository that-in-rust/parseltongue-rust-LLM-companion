//! REQ-RUST-062.0 C10, C11: which declaration a type name written at file level of a
//! `core` source file stands for. Pure lookups over the stored declaration rows; a name
//! that is not proven gives `None`.
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[cfg(test)]
#[path = "core-file-scope-name-tests.rs"]
mod core_file_scope_name_tests;

const CORE_CRATE_ROOT_PATH: &str = "core/src/lib.rs";

fn check_scope_binding_row(row: &Value) -> bool {
    row["declaration_owner_kind_value"] == "file_scope_name_binding"
}

/// The file scope rows of `file` that bind `name`.
fn list_named_scope_rows<'a>(rows: &'a [Value], file: &str, name: &str) -> Vec<&'a Value> {
    rows.iter()
        .filter(|row| {
            check_scope_binding_row(row)
                && row["declaration_source_relative_path"] == file
                && row["member_declared_name_value"] == name
        })
        .collect()
}

fn list_row_attribute_texts(row: &Value) -> impl Iterator<Item = &str> {
    row["member_attribute_text_values"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
}

/// C10: a row used as a proof step has no attribute text that contains `cfg`.
fn check_row_always_active(row: &Value) -> bool {
    row["member_attribute_text_values"].is_array()
        && list_row_attribute_texts(row).all(|text| !text.contains("cfg"))
}

/// C10 (a): the one struct, enum or union that `file` declares as `name` at file level.
fn find_declared_scope_type<'a>(rows: &'a [Value], file: &str, name: &str) -> Option<&'a Value> {
    match list_named_scope_rows(rows, file, name).as_slice() {
        [row]
            if row["scope_binding_form_kind"] == "item"
                && matches!(
                    row["type_declaration_kind_value"].as_str(),
                    Some("struct" | "enum" | "union")
                )
                && check_row_always_active(row) =>
        {
            Some(row)
        }
        _ => None,
    }
}

/// C11: the one row of the crate root that binds `name`, when it is a file module with
/// no `#[path` attribute.
fn find_root_file_module<'a>(rows: &'a [Value], name: &str) -> Option<&'a Value> {
    match list_named_scope_rows(rows, CORE_CRATE_ROOT_PATH, name).as_slice() {
        [row]
            if row["type_declaration_kind_value"] == "module_file"
                && check_row_always_active(row)
                && list_row_attribute_texts(row).all(|text| !text.starts_with("#[path")) =>
        {
            Some(row)
        }
        _ => None,
    }
}

fn build_row_site_value(row: &Value) -> Value {
    json!({
        "declaration_source_relative_path":row["declaration_source_relative_path"],
        "member_name_start_offset":row["member_name_start_offset"],
        "member_name_end_offset":row["member_name_end_offset"]
    })
}

/// C11: the declaration the prelude of `core` gives `name`, with the rows that prove it.
fn find_prelude_scope_type(rows: &[Value], name: &str) -> Option<Value> {
    let imports = rows
        .iter()
        .filter(|row| {
            check_scope_binding_row(row)
                && list_row_attribute_texts(row).any(|text| text.contains("prelude_import"))
        })
        .collect::<Vec<_>>();
    let [import] = imports.as_slice() else {
        return None;
    };
    let globbed = import["declaration_source_relative_path"] == CORE_CRATE_ROOT_PATH
        && import["scope_binding_form_kind"] == "use_glob"
        && check_row_always_active(import)
        && import["use_leaf_path_text"]
            .as_str()
            .is_some_and(|path| path.starts_with("prelude::"));
    if !globbed {
        return None;
    }
    find_root_file_module(rows, "prelude")?;
    let leaves = rows
        .iter()
        .filter(|row| {
            row["declaration_owner_kind_value"] == "prelude_use_leaf"
                && row["use_leaf_bound_name_value"] == name
        })
        .collect::<Vec<_>>();
    let paths = leaves
        .iter()
        .map(|row| row["use_leaf_path_text"].as_str())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let (Some(leaf), [Some(path)]) = (leaves.first(), paths.as_slice()) else {
        return None;
    };
    let module = match path.split("::").collect::<Vec<_>>().as_slice() {
        ["crate", module, last] if *last == name => *module,
        _ => return None,
    };
    find_root_file_module(rows, module)?;
    let files = [
        format!("core/src/{module}.rs"),
        format!("core/src/{module}/mod.rs"),
    ];
    let stored = files
        .iter()
        .filter(|file| {
            rows.iter()
                .any(|row| row["declaration_source_relative_path"] == file.as_str())
        })
        .collect::<Vec<_>>();
    let [file] = stored.as_slice() else {
        return None;
    };
    let declared = find_declared_scope_type(rows, file, name)?;
    Some(json!({
        "type_name_proof_kind":"prelude_import",
        "type_declaration_site_value":build_row_site_value(declared),
        "prelude_import_site_value":build_row_site_value(import),
        "prelude_leaf_site_value":build_row_site_value(leaf),
        "prelude_leaf_path_text":path
    }))
}

/// C10: the declaration that the type name `name`, written at file level of `file`,
/// stands for: a record with `type_declaration_site_value` and how it was proven.
pub(crate) fn find_file_scope_type(rows: &[Value], file: &str, name: &str) -> Option<Value> {
    let proof = match find_declared_scope_type(rows, file, name) {
        Some(declared) => Some(json!({
            "type_name_proof_kind":"file_scope_declaration",
            "type_declaration_site_value":build_row_site_value(declared)
        })),
        None => {
            let unread = rows.iter().any(|row| {
                check_scope_binding_row(row)
                    && row["declaration_source_relative_path"] == file
                    && (row["member_declared_name_value"] == name
                        || matches!(
                            row["scope_binding_form_kind"].as_str(),
                            Some("use_glob" | "unread_form")
                        ))
            });
            let crate_named = rows.iter().any(|row| {
                row["member_declared_name_value"] == name
                    && (row["scope_binding_form_kind"] == "extern_crate"
                        || row["type_declaration_kind_value"] == "extern_crate_rename")
            });
            (!unread && !crate_named)
                .then(|| find_prelude_scope_type(rows, name))
                .flatten()
        }
    };
    proof.map(|mut proof| {
        proof["type_name_value"] = json!(name);
        proof["name_written_source_path"] = json!(file);
        proof
    })
}

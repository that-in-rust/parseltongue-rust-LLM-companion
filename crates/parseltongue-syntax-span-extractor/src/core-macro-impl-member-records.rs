//! REQ-RUST-063.0 M4 to M7: the member that a `macro_rules!` of `core` writes into an
//! inherent impl of a builtin type, and a method called on a const of such a type. Pure
//! lookups over the stored rows; a member that is not proven gives `None`.
use crate::core_file_scope_name_records::{
    build_row_site_value, check_row_always_active, check_scope_binding_row,
    list_row_attribute_texts,
};
use serde_json::{Value, json};

#[cfg(test)]
#[path = "core-macro-impl-member-tests.rs"]
mod core_macro_impl_member_tests;

fn check_row_source_file(row: &Value, file: &str) -> bool {
    row["declaration_source_relative_path"] == file
}

/// M4: whether the file scope row may define the macro `name` for the code after it: a
/// definition of that name, a module or extern crate under `macro_use`, or a form whose
/// names are not read.
fn check_macro_scope_row(row: &Value, name: &str) -> bool {
    check_scope_binding_row(row)
        && match row["scope_binding_form_kind"].as_str() {
            Some("macro_definition") => row["member_declared_name_value"] == name,
            Some("module" | "extern_crate") => {
                list_row_attribute_texts(row).any(|text| text.contains("macro_use"))
            }
            Some("unread_form") => true,
            _ => false,
        }
}

/// The rows of `file` that may define the macro `name`, in source order.
fn list_macro_scope_rows<'a>(rows: &'a [Value], file: &str, name: &str) -> Vec<&'a Value> {
    let mut found = rows
        .iter()
        .filter(|row| check_row_source_file(row, file) && check_macro_scope_row(row, name))
        .collect::<Vec<_>>();
    found.sort_by_key(|row| row["member_name_start_offset"].as_u64());
    found
}

fn check_macro_definition_row(row: &Value) -> bool {
    row["scope_binding_form_kind"] == "macro_definition" && check_row_always_active(row)
}

/// The one file with stored rows that the file module `module` of `file` names.
fn find_module_source_file(rows: &[Value], file: &str, module: &str) -> Option<String> {
    let folder = match file.rsplit_once('/')? {
        (folder, "mod.rs" | "lib.rs") => folder.to_owned(),
        _ => file.strip_suffix(".rs")?.to_owned(),
    };
    let stored = |path: &String| rows.iter().any(|row| check_row_source_file(row, path));
    let mut found = [
        format!("{folder}/{module}.rs"),
        format!("{folder}/{module}/mod.rs"),
    ]
    .into_iter()
    .filter(stored);
    found.next().filter(|_| found.next().is_none())
}

/// M4 (b): what the `#[macro_use]` module row `module` of `file` says about the macro
/// `name`. `None`: its file defines nothing that may be `name`, so the row is passed
/// over. `Some(None)`: not proven.
fn read_module_macro_definition<'a>(
    rows: &'a [Value],
    file: &str,
    module: &Value,
    name: &str,
) -> Option<Option<&'a Value>> {
    let source = Some(module)
        .filter(|row| {
            row["type_declaration_kind_value"] == "module_file"
                && check_row_always_active(row)
                && list_row_attribute_texts(row).all(|text| !text.starts_with("#[path"))
        })
        .and_then(|row| row["member_declared_name_value"].as_str())
        .and_then(|declared| find_module_source_file(rows, file, declared));
    let Some(source) = source else {
        return Some(None);
    };
    match list_macro_scope_rows(rows, &source, name).as_slice() {
        [] => None,
        [earlier @ .., last] => Some(
            Some(*last)
                .filter(|row| check_macro_definition_row(row))
                .filter(|_| {
                    earlier
                        .iter()
                        .all(|row| row["scope_binding_form_kind"] != "macro_definition")
                }),
        ),
    }
}

/// M4: the definition that the macro `name`, called in `file` at `offset`, stands for:
/// the last one written before the call, in the file or in a `#[macro_use]` module of it.
pub(crate) fn find_macro_scope_definition(
    rows: &[Value],
    file: &str,
    offset: u64,
    name: &str,
) -> Option<Value> {
    let proof = |kind: &str, definition: &Value, module: Option<&Value>| {
        json!({
            "macro_scope_proof_kind":kind,
            "macro_definition_site_value":build_row_site_value(definition),
            "macro_arm_count_value":definition["macro_arm_count_value"],
            "macro_use_module_site_value":module.map(build_row_site_value),
        })
    };
    list_macro_scope_rows(rows, file, name)
        .into_iter()
        .rev()
        .filter(|row| row["member_name_start_offset"].as_u64() < Some(offset))
        .find_map(|row| match row["scope_binding_form_kind"].as_str() {
            Some("macro_definition") => {
                Some(check_macro_definition_row(row).then(|| proof("same_file", row, None)))
            }
            Some("module") => read_module_macro_definition(rows, file, row, name).map(|found| {
                found.map(|definition| proof("macro_use_module", definition, Some(row)))
            }),
            _ => Some(None),
        })
        .flatten()
}

fn check_owner_always_active(row: &Value) -> bool {
    row["owner_attribute_text_values"]
        .as_array()
        .is_some_and(|texts| {
            texts
                .iter()
                .all(|text| text.as_str().is_some_and(|text| !text.contains("cfg")))
        })
}

/// M5: the one row of the one-armed macro at `definition` that declares `name` as a
/// `kind` item directly in the arm's body, in force in every configuration.
fn find_macro_body_member<'a>(
    rows: &'a [Value],
    definition: &Value,
    name: &str,
    kind: &str,
) -> Option<&'a Value> {
    let site = &definition["macro_definition_site_value"];
    let mut named = rows.iter().filter(|row| {
        row["declaration_owner_kind_value"] == "macro_definition_body"
            && row["declaration_source_relative_path"] == site["declaration_source_relative_path"]
            && row["owner_macro_name_offset"] == site["member_name_start_offset"]
            && row["member_declared_name_value"] == name
            && row["member_item_kind_value"] == kind
    });
    named
        .next()
        .filter(|_| definition["macro_arm_count_value"] == 1 && named.next().is_none())
        .filter(|row| {
            row["member_name_parameter_flag"] == false
                && row["macro_body_nesting_depth"] == 0
                && row["macro_body_repetition_flag"] == false
                && row["macro_item_boundary_flag"] == true
                && check_row_always_active(row)
                && (kind != "fn" || row["macro_self_parameter_flag"] == true)
        })
}

/// M5: the inherent member `name` of kind `kind` (`const` or `fn`) of the builtin type
/// `builtin`, with the rows that prove it.
pub(crate) fn find_macro_impl_member(
    rows: &[Value],
    builtin: &str,
    name: &str,
    kind: &str,
) -> Option<Value> {
    let written = rows.iter().any(|row| {
        row["declaration_owner_kind_value"] == "inherent_impl"
            && row["impl_self_type_text"] == builtin
            && row["member_declared_name_value"] == name
    });
    let mut pairs = rows
        .iter()
        .filter(|row| {
            row["declaration_owner_kind_value"] == "impl_macro_call"
                && row["impl_self_type_text"] == builtin
                && row["owner_trait_path_text"].is_null()
                && row["impl_generic_parameter_text"].is_null()
                && check_row_always_active(row)
                && check_owner_always_active(row)
        })
        .filter_map(|call| {
            let macro_name = call["owner_macro_name_value"].as_str()?;
            let definition = find_macro_scope_definition(
                rows,
                call["declaration_source_relative_path"].as_str()?,
                call["member_name_start_offset"].as_u64()?,
                macro_name,
            )?;
            let member = find_macro_body_member(rows, &definition, name, kind)?;
            Some(json!({
                "builtin_type_name_value":builtin,
                "member_declared_name_value":name,
                "member_item_kind_value":kind,
                "member_written_type_text":member["member_written_type_text"],
                "impl_macro_call_site_value":build_row_site_value(call),
                "macro_scope_proof_value":definition,
                "macro_body_item_site_value":build_row_site_value(member),
                "inherent_lookup_rule_kind":"inherent_item_declared_once",
            }))
        });
    pairs.next().filter(|_| !written && pairs.next().is_none())
}

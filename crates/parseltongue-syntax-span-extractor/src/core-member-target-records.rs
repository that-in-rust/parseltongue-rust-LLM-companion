//! REQ-RUST-056.0: the target of a method call whose receiver's type is written in the
//! workspace and whose method `core` declares. The receiver's type is read from stored
//! evidence, every name in it is proven against the stored path lookups, and the probe of
//! `receiver-type-probe-records.rs` picks the declaration. Anything not proven withholds
//! the name: a missed target is preferable to a wrong one.
use crate::external_named_target_records::{ExternalNameProofRecord, build_external_proof_value};
use crate::impl_header_source_records::{
    ImplHeaderSourceContext, ImplHeaderSourceError, read_impl_stage_values,
};
use crate::original_call_resolution_records::OriginalCallResolutionError;
use crate::receiver_type_probe_records::{
    CoreMemberProbeOutcome, ScopeTraitMethodValue, WrittenTypeShapeValue, parse_written_type_shape,
    probe_core_member_declaration,
};
use crate::type_relative_member_records::{BuiltinMemberNameOutcome, ImplMemberLookupIndex};
use parseltongue_core_row_types::*;
use parseltongue_table_store_traits::StageTableStorePort;
use ra_ap_syntax::{AstNode, Edition, SourceFile, SyntaxKind};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[cfg(test)]
#[path = "core-member-target-tests.rs"]
mod core_member_target_tests;

/// What one run stores for the lookup: the declarations of `core`, and what decides
/// which traits are in scope of a call in the workspace.
#[derive(Default)]
pub(crate) struct CoreMemberLookupIndex {
    core_declaration_value_rows: Vec<Value>,
    /// Why no call of this run can be given a `core` target, when one reason covers all.
    scope_proof_withheld_reason: Option<&'static str>,
    /// `(renamed name, new name)` for each `use .. as NAME` of `core`; no renamed name
    /// when the rename is of `self`.
    core_rename_name_pairs: Vec<(Option<String>, String)>,
    /// The names the prelude files of `core` re-export.
    scope_prelude_leaf_names: BTreeSet<String>,
    /// Every identifier written in the workspace's read source.
    workspace_token_name_values: BTreeSet<String>,
    /// `(trait name, member name)` for each member a workspace trait declares.
    workspace_trait_member_names: BTreeSet<(String, String)>,
}

/// The identifiers written in `text`, and whether a `use` in it may import by glob: a
/// `*` between `::`, `{` or `,` and `;`, `}` or `,`. Tokens inside macro definitions count.
/// Last, whether it holds an `extern crate` item.
fn scan_source_scope_tokens(text: &str) -> (BTreeSet<String>, bool, bool) {
    let tree = SourceFile::parse(text, Edition::Edition2021).tree();
    let outside = tree
        .syntax()
        .descendants()
        .any(|node| node.kind() == SyntaxKind::EXTERN_CRATE);
    let tokens = tree
        .syntax()
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| !token.kind().is_trivia())
        .collect::<Vec<_>>();
    let names = tokens
        .iter()
        .filter(|token| token.kind() == SyntaxKind::IDENT)
        .map(|token| token.text().trim_start_matches("r#").to_owned())
        .collect();
    let glob = tokens.windows(3).any(|window| {
        matches!(
            window[0].kind(),
            SyntaxKind::COLON2 | SyntaxKind::L_CURLY | SyntaxKind::COMMA
        ) && window[1].kind() == SyntaxKind::STAR
            && matches!(
                window[2].kind(),
                SyntaxKind::SEMICOLON | SyntaxKind::R_CURLY | SyntaxKind::COMMA
            )
    });
    (names, glob, outside)
}

/// The last name of a written path, without its generic arguments or spaces.
fn read_path_last_name(text: &str) -> &str {
    let plain = text.split('<').next().unwrap_or(text);
    plain.rsplit("::").next().unwrap_or(plain).trim()
}

/// The lookup index of `scope`. A run that stored no declarations of `core` gives an
/// index that names nothing.
pub(crate) fn read_core_member_index(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
    context: &ImplHeaderSourceContext<'_>,
    nodes: &[HldNodeInputRecord],
) -> Result<CoreMemberLookupIndex, ImplHeaderSourceError> {
    let declarations = read_impl_stage_values(store, scope, "sysroot-member-declaration-records")?;
    if declarations.is_empty() {
        return Ok(CoreMemberLookupIndex::default());
    }
    let units = read_impl_stage_values(store, scope, "crate-dependency-graph-table")?;
    // A file with a stored span is a file of a mapped unit and is read whole: an
    // `extern crate` or an attribute is no stored span. A file with no span is another
    // target's, unless an `extern crate` in it could put names in a unit's scope.
    let unit_files = context.list_span_file_paths();
    let texts = read_impl_stage_values(store, scope, "file-source-text-table")?
        .iter()
        .map(|file| {
            let digest = file["file_content_digest_full"].as_str().unwrap_or("");
            let member = file["file_source_absolute_path"]
                .as_str()
                .is_some_and(|path| unit_files.contains(path));
            let text = String::from_utf8(store.read_file_content_bytes(digest)?).ok();
            Ok(text.map(|text| (member, scan_source_scope_tokens(&text))))
        })
        .collect::<Result<Option<Vec<_>>, ImplHeaderSourceError>>()?
        .filter(|texts| texts.iter().any(|(member, _)| *member));
    let stray = texts
        .iter()
        .flatten()
        .any(|(member, (.., outside))| !member && *outside);
    let (tokens, glob) = texts.iter().flatten().filter(|(member, _)| *member).fold(
        (BTreeSet::new(), false),
        |(mut names, glob), (_, (found, star, _))| {
            names.extend(found.iter().cloned());
            (names, glob || *star)
        },
    );
    let of_kind = |kind: &'static str| {
        declarations
            .iter()
            .filter(move |row| row["declaration_owner_kind_value"] == kind)
    };
    let prelude = of_kind("prelude_use_leaf")
        .filter_map(|row| row["member_declared_name_value"].as_str())
        .filter(|name| *name != "*")
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    // Each reason says what the run did not read; the first that holds covers every call.
    let reason = if texts.is_none() {
        Some("workspace_source_text_unread")
    } else if stray {
        Some("unit_file_membership_unproven")
    } else if units.iter().any(|unit| unit["unit_no_std_flag"] != true) {
        Some("std_prelude_not_read")
    } else if units.iter().any(|unit| {
        unit["unit_normal_dependency_rows"]
            .as_array()
            .is_none_or(|rows| !rows.is_empty())
    }) {
        Some("dependency_crates_not_read")
    } else if tokens.contains("alloc") || tokens.contains("std") {
        Some("alloc_or_std_named")
    } else if glob {
        Some("scope_glob_import_unread")
    } else if prelude.is_empty() {
        Some("prelude_rows_missing")
    } else if of_kind("prelude_use_leaf").any(|row| {
        row["member_declared_name_value"] == "*" && row["impl_self_type_text"] != "super::v1"
    }) {
        Some("prelude_glob_unread")
    } else {
        None
    };
    let renames = of_kind("type_name_declaration")
        .filter(|row| row["type_declaration_kind_value"] == "use_rename")
        .filter_map(|row| {
            Some((
                row["rename_source_name_value"].as_str().map(str::to_owned),
                row["member_declared_name_value"].as_str()?.to_owned(),
            ))
        })
        .collect();
    Ok(CoreMemberLookupIndex {
        core_rename_name_pairs: renames,
        core_declaration_value_rows: declarations,
        scope_proof_withheld_reason: reason,
        scope_prelude_leaf_names: prelude,
        workspace_token_name_values: tokens,
        workspace_trait_member_names: nodes
            .iter()
            .filter(|node| node.node_owner_kind_value == Some(HldOwnerKindValue::Trait))
            .filter_map(|node| {
                let mut names = node.node_qualified_path_text.rsplit("::");
                let member = names.next()?.trim_start_matches("r#").to_owned();
                Some((names.next()?.to_owned(), member))
            })
            .collect(),
    })
}

fn render_written_type_shape(shape: &WrittenTypeShapeValue) -> String {
    match shape {
        WrittenTypeShapeValue::Reference { mutable, inner } => {
            format!(
                "&{}{}",
                if *mutable { "mut " } else { "" },
                render_written_type_shape(inner)
            )
        }
        WrittenTypeShapeValue::Pointer { mutable, inner } => format!(
            "*{} {}",
            if *mutable { "mut" } else { "const" },
            render_written_type_shape(inner)
        ),
        WrittenTypeShapeValue::Array(inner) => format!("[{}; _]", render_written_type_shape(inner)),
        WrittenTypeShapeValue::Slice(inner) => format!("[{}]", render_written_type_shape(inner)),
        WrittenTypeShapeValue::Named { name, arguments } if arguments.is_empty() => name.clone(),
        WrittenTypeShapeValue::Named { name, arguments } => format!(
            "{name}<{}>",
            arguments
                .iter()
                .map(render_written_type_shape)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        WrittenTypeShapeValue::Wildcard => "_".to_owned(),
        WrittenTypeShapeValue::Opaque => "?".to_owned(),
    }
}

fn strip_shape_reference_layers(shape: &WrittenTypeShapeValue) -> &WrittenTypeShapeValue {
    match shape {
        WrittenTypeShapeValue::Reference { inner, .. } => strip_shape_reference_layers(inner),
        other => other,
    }
}

/// Whether the stored lookup of one path of a written type proves its name: a builtin
/// type, or a name of `core` whose last segment is the name as written.
fn check_written_path_proven(
    span: &Value,
    path: &Value,
    context: &ImplHeaderSourceContext<'_>,
) -> Result<bool, OriginalCallResolutionError> {
    let Some(text) = path["type_source_text"].as_str() else {
        return Ok(false);
    };
    let name = read_path_last_name(text);
    let [record] = context.read_type_path_records(span, path)? else {
        return Ok(false);
    };
    Ok(match record["path_resolution_state_kind"].as_str() {
        Some("resolved") => matches!(
            record["path_target_symbol_rows"].as_array().map(Vec::as_slice),
            Some([target]) if target["target_symbol_kind_value"] == "Builtin"
                && target["target_builtin_name_value"] == name
                && text == name
        ),
        Some("unresolved") => {
            let named = &record["path_external_name_record"];
            record["path_unresolved_reason_kind"] == "external_source_missing"
                && named["external_crate_name"] == "core"
                && named["external_name_kind"] == "crate_rooted_path"
                && named["external_item_path_text"]
                    .as_str()
                    .is_some_and(|found| {
                        found.starts_with("core::") && read_path_last_name(found) == name
                    })
        }
        _ => false,
    })
}

fn check_header_definition_target(resolution: &Value) -> bool {
    resolution["resolution_state_kind"] == "resolved"
        && resolution["target_identity"]["target_identity_variant_kind"] == "definition"
}

impl CoreMemberLookupIndex {
    /// Whether a trait of `core` declared as `name` may be in scope of a call in the
    /// workspace: `name`, or a name `core` renames it to, is re-exported by the prelude or
    /// written in the workspace. A rename of `self` may rename any trait.
    fn check_core_trait_scope(&self, name: &str) -> bool {
        let written = |name: &str| {
            self.scope_prelude_leaf_names.contains(name)
                || self.workspace_token_name_values.contains(name)
        };
        let mut names = BTreeSet::from([name]);
        loop {
            let found = self
                .core_rename_name_pairs
                .iter()
                .filter(|(renamed, _)| {
                    renamed
                        .as_deref()
                        .is_none_or(|renamed| names.contains(renamed))
                })
                .map(|(_, alias)| alias.as_str())
                .filter(|alias| !names.contains(alias))
                .collect::<Vec<_>>();
            if found.is_empty() {
                return names.into_iter().any(written);
            }
            names.extend(found);
        }
    }

    /// P4, P5: the traits that may be in scope of a call and declare `method`. A trait of
    /// `core` is in scope only through the prelude or by its name written in the
    /// workspace; a workspace trait counts wherever it is declared.
    fn list_scope_trait_methods(
        &self,
        method: &str,
        members: &ImplMemberLookupIndex,
    ) -> Result<Vec<ScopeTraitMethodValue>, &'static str> {
        let named = |row: &&Value| {
            row["member_declared_name_value"] == method || row["member_name_parameter_flag"] == true
        };
        if self
            .core_declaration_value_rows
            .iter()
            .filter(named)
            .any(|row| {
                matches!(
                    row["declaration_owner_kind_value"].as_str(),
                    Some("macro_definition_body" | "macro_call_argument")
                ) && row["macro_owner_shape_kind"] == "trait_declaration"
            })
        {
            return Err("macro_trait_owner_unknown");
        }
        let mut found = self
            .core_declaration_value_rows
            .iter()
            .filter(|row| {
                row["declaration_owner_kind_value"] == "trait_declaration"
                    && row["member_declared_name_value"] == method
                    && row["member_self_parameter_kind"] != "none"
            })
            .filter(|row| {
                let name = read_path_last_name(row["owner_trait_path_text"].as_str().unwrap_or(""));
                name.is_empty() || self.check_core_trait_scope(name)
            })
            .map(|row| ScopeTraitMethodValue {
                member_self_parameter_kind: row["member_self_parameter_kind"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_owned(),
                trait_impl_self_shapes: None,
            })
            .collect::<Vec<_>>();
        for header in &members.impl_header_value_rows {
            let implemented = &header["impl_trait_type_resolution"];
            if implemented.is_null() {
                continue;
            }
            if !check_header_definition_target(implemented) {
                // A trait outside the workspace is named in the workspace: handled above.
                if implemented["unresolved_reason_kind"] == "external_source_missing" {
                    continue;
                }
                return Err("workspace_trait_impl_unresolved");
            }
            let written = implemented["type_source_evidence"]["type_source_text"]
                .as_str()
                .unwrap_or("")
                .replace(char::is_whitespace, "");
            let key = (read_path_last_name(&written).to_owned(), method.to_owned());
            if !self.workspace_trait_member_names.contains(&key) {
                continue;
            }
            let stored = &header["impl_self_type_resolution"];
            let shapes = (header["impl_generic_parameters_flag"] == false).then(|| {
                vec![
                    stored["target_identity"]["target_builtin_name_value"]
                        .as_str()
                        .filter(|_| {
                            stored["resolution_state_kind"] == "resolved"
                                && stored["type_source_evidence"]["type_simple_path_flag"] == true
                        })
                        .map_or(WrittenTypeShapeValue::Opaque, |name| {
                            WrittenTypeShapeValue::Named {
                                name: name.to_owned(),
                                arguments: Vec::new(),
                            }
                        }),
                ]
            });
            let declared = header["impl_member_occurrence_rows"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|member| {
                    member["member_declared_name_value"] == method
                        && member["member_syntax_kind_value"] == "FN"
                });
            let kinds = match declared {
                // The impl takes the trait's default body: its `self` kind is not stored.
                None => vec!["value", "reference", "mutable_reference"],
                Some(member) => match member["member_self_parameter_kind"].as_str() {
                    Some(kind) => vec![kind],
                    None => Vec::new(),
                },
            };
            found.extend(kinds.into_iter().map(|kind| ScopeTraitMethodValue {
                member_self_parameter_kind: kind.to_owned(),
                trait_impl_self_shapes: shapes.clone(),
            }));
        }
        Ok(found)
    }
}

/// The types an indexed place may have: the slice for a range written as the index, else
/// the element or the slice. `None` when `shape` is not an array or slice behind references.
fn list_index_result_shapes(
    shape: &WrittenTypeShapeValue,
    range: bool,
) -> Option<Vec<WrittenTypeShapeValue>> {
    let (WrittenTypeShapeValue::Array(element) | WrittenTypeShapeValue::Slice(element)) =
        strip_shape_reference_layers(shape)
    else {
        return None;
    };
    let slice = WrittenTypeShapeValue::Slice(element.clone());
    Some(if range {
        vec![slice]
    } else {
        vec![(**element).clone(), slice]
    })
}

/// The `core` member that the method call at `span` names, with the record of its
/// lookup, or the reason the name is withheld. `None` when the lookup does not apply:
/// no declarations stored, or a receiver that is not a parameter with a written type.
pub(crate) fn find_core_member_target(
    span: &Value,
    receiver: &Value,
    context: &ImplHeaderSourceContext<'_>,
    members: &ImplMemberLookupIndex,
    index: &CoreMemberLookupIndex,
) -> Result<Option<(BuiltinMemberNameOutcome, Value)>, OriginalCallResolutionError> {
    let declarations = index.core_declaration_value_rows.as_slice();
    let (Some(method), Some([first, steps @ ..])) = (
        receiver["method_declared_name_value"].as_str(),
        receiver["receiver_place_step_rows"]
            .as_array()
            .map(Vec::as_slice),
    ) else {
        return Ok(None);
    };
    let written = &first["receiver_written_type_evidence"];
    let Some(text) = written["type_source_text"].as_str().filter(|_| {
        !declarations.is_empty()
            && first["place_step_kind"] == "parameter_identifier"
            && steps
                .iter()
                .all(|step| step["place_step_kind"] == "index_access")
    }) else {
        return Ok(None);
    };
    let mut evidence = json!({
        "receiver_type_proof_kind":"parameter_written_type",
        "receiver_written_type_text":text,
        "method_probe_unread_source":["alloc", "std"],
        "lookup_assumption_values":["core_macro_expansions_import_no_trait"]
    });
    let withheld = |reason: &'static str, evidence: Value| Ok(Some((Err(reason), evidence)));
    if let Some(reason) = index.scope_proof_withheld_reason {
        return withheld(reason, evidence);
    }
    let Some(paths) = written["type_path_range_rows"].as_array() else {
        return withheld("receiver_type_name_unproven", evidence);
    };
    for path in paths {
        if !check_written_path_proven(span, path, context)? {
            return withheld("receiver_type_name_unproven", evidence);
        }
    }
    // A workspace impl of a trait from outside for a type from outside may give an index
    // expression another type: `impl Index<Local> for [u8]`.
    if !steps.is_empty()
        && members.impl_header_value_rows.iter().any(|header| {
            !header["impl_trait_type_resolution"].is_null()
                && !check_header_definition_target(&header["impl_trait_type_resolution"])
                && !check_header_definition_target(&header["impl_self_type_resolution"])
        })
    {
        return withheld("workspace_foreign_impl_present", evidence);
    }
    let mut shapes = vec![parse_written_type_shape(text, &[])];
    for step in steps {
        let range = step["index_range_expression_flag"] == true;
        let Some(next) = shapes
            .iter()
            .map(|shape| list_index_result_shapes(shape, range))
            .collect::<Option<Vec<_>>>()
        else {
            return withheld("receiver_type_not_modelled", evidence);
        };
        shapes = next.into_iter().flatten().collect();
    }
    let scope_traits = match index.list_scope_trait_methods(method, members) {
        Ok(found) => found,
        Err(reason) => return withheld(reason, evidence),
    };
    let outcomes = shapes
        .iter()
        .map(|shape| {
            (
                shape,
                probe_core_member_declaration(shape, method, declarations, &scope_traits),
            )
        })
        .collect::<Vec<_>>();
    // The call compiles, so its receiver has the method. A type that cannot have it, and
    // that no `Deref` leads away from, is not the receiver's type.
    let refuted = |(shape, outcome): &&(&WrittenTypeShapeValue, CoreMemberProbeOutcome)| {
        *outcome == CoreMemberProbeOutcome::Withheld("no_member_declared")
            && match strip_shape_reference_layers(shape) {
                WrittenTypeShapeValue::Array(_) | WrittenTypeShapeValue::Slice(_) => true,
                WrittenTypeShapeValue::Named { name, arguments } => {
                    arguments.is_empty() && is_hld_builtin_type(name)
                }
                _ => false,
            }
    };
    let (shape, outcome) = match outcomes.as_slice() {
        [only] => only,
        many => {
            let kept = many
                .iter()
                .filter(|found| !refuted(found))
                .collect::<Vec<_>>();
            match kept.as_slice() {
                [only] => *only,
                _ => return withheld("index_result_type_unproven", evidence),
            }
        }
    };
    evidence["receiver_type_text"] = json!(render_written_type_shape(shape));
    let (row, step) = match outcome {
        CoreMemberProbeOutcome::Named {
            declaration_row_index,
            probe_step_text,
        } => (&declarations[*declaration_row_index], probe_step_text),
        CoreMemberProbeOutcome::Withheld(reason) => return withheld(reason, evidence),
    };
    let Some(owner) = row["impl_self_type_text"].as_str() else {
        return withheld("inherent_member_unproven", evidence);
    };
    evidence["method_probe_step_text"] = json!(step);
    for key in [
        "declaration_source_relative_path",
        "member_name_start_offset",
        "member_name_end_offset",
    ] {
        evidence[key] = row[key].clone();
    }
    Ok(Some((
        Ok(ExternalNameProofRecord {
            external_target_identity_value: HldTargetIdentityValue::ExternalNamed(
                HldExternalNamedKey {
                    external_name_kind_value: HldExternalNameKind::CoreDeclaredMember,
                    external_crate_name_value: Some("core".to_owned()),
                    external_item_path_text: format!("{owner}::{method}"),
                },
            ),
            external_proof_metadata_value: build_external_proof_value(
                "core_declared_member",
                "core_declaration_probe",
                Value::Null,
            ),
        }),
        evidence,
    )))
}

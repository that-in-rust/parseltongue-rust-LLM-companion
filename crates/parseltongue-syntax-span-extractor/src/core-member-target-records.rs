//! REQ-RUST-056.0: the target of a method call whose receiver's type is written in the
//! workspace and whose method `core` declares. The receiver's type is read from stored
//! evidence, every name in it is proven against the stored path lookups, and the probe of
//! `receiver-type-probe-records.rs` picks the declaration. Anything not proven withholds
//! the name: a missed target is preferable to a wrong one.
use crate::core_file_scope_name_records::find_file_scope_type;
use crate::core_macro_impl_member_records::find_builtin_constant_member;
use crate::external_named_target_records::{ExternalNameProofRecord, build_external_proof_value};
use crate::impl_header_source_records::{
    ImplHeaderSourceContext, ImplHeaderSourceError, read_impl_stage_values,
};
use crate::original_call_resolution_records::{
    BoundMethodLookupIndex, OriginalCallResolutionError, list_bound_method_candidates,
};
use crate::receiver_type_probe_records::{
    CoreMemberProbeOutcome, ScopeTraitMethodValue, WrittenTypeShapeValue,
    check_macro_member_candidate, list_shape_type_names, list_unproven_type_names,
    parse_return_type_shape, parse_written_type_shape, probe_core_member_declaration,
};
use crate::syntax_namespace_record_codec::{
    decode_occurrence_identity_value, decode_symbol_target_value,
};
use crate::type_relative_member_records::{BuiltinMemberNameOutcome, ImplMemberLookupIndex};
use parseltongue_core_row_types::*;
use parseltongue_table_store_traits::StageTableStorePort;
use ra_ap_syntax::ast::{self, HasAttrs, HasName};
use ra_ap_syntax::{AstNode, Edition, SourceFile, SyntaxKind};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

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
    /// Every identifier written in the source files of the mapped units, with the
    /// number of times it is written.
    workspace_token_name_counts: BTreeMap<String, usize>,
    /// `(trait name, member name)` for each member a workspace trait declares.
    workspace_trait_member_names: BTreeSet<(String, String)>,
    /// REQ-RUST-059.0 S7: the text of the `self` parameter of each function, by the
    /// stored occurrence key of the function.
    self_parameter_source_texts: BTreeMap<String, String>,
    /// REQ-RUST-061.0: the last name of the path of every graph node of the workspace.
    workspace_item_leaf_names: BTreeSet<String>,
    /// REQ-RUST-059.0: `(owner, row)` for the stored declared type of each `static` and
    /// each stored field; the owner is the graph identity of the `static`, or of the item
    /// that declares the field.
    owner_declared_type_rows: Vec<(HldTargetIdentityValue, Value)>,
    /// REQ-RUST-064.0 L4: `(owner, row)` for the stored return type of each function.
    owner_return_type_rows: Vec<(HldTargetIdentityValue, Value)>,
}

/// The graph identity of the active stored span `key`; `None` when it has none.
fn read_span_graph_identity(
    key: &str,
    context: &ImplHeaderSourceContext<'_>,
) -> Option<HldTargetIdentityValue> {
    let span = context
        .span(key)
        .ok()
        .filter(|span| span["span_configuration_active_flag"] == true)?;
    let occurrence = decode_occurrence_identity_value(&context.occurrence(span).ok()?).ok()?;
    Some(NameSymbolTargetKey::Definition(occurrence).into())
}

fn check_declared_type_row(row: &Value) -> bool {
    row["declared_type_owner_kind"] == "STATIC"
        || matches!(
            row["site_syntax_kind_value"].as_str(),
            Some("TUPLE_FIELD" | "RECORD_FIELD")
        )
}

/// The identifiers written in `text`, and whether a `use` in it may import by glob: a
/// `*` between `::`, `{` or `,` and `;`, `}` or `,`. Tokens inside macro definitions count.
/// Last, whether it holds an `extern crate` item.
fn scan_source_scope_tokens(text: &str) -> (BTreeMap<String, usize>, bool, bool) {
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
        .fold(BTreeMap::new(), |mut counts, name| {
            *counts.entry(name).or_insert(0usize) += 1;
            counts
        });
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
        (BTreeMap::new(), false),
        |(mut names, glob), (_, (found, star, _))| {
            for (name, count) in found {
                *names.entry(name.clone()).or_insert(0usize) += count;
            }
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
    } else if tokens.contains_key("alloc") || tokens.contains_key("std") {
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
    let signatures = read_impl_stage_values(store, scope, "field-signature-detail-table")?;
    let receivers = signatures
        .iter()
        .filter(|row| row["site_syntax_kind_value"] == "SELF_PARAM")
        .filter_map(|row| {
            Some((
                row["site_owner_occurrence_key"].as_str()?.to_owned(),
                row["site_syntax_source_text"].as_str()?.to_owned(),
            ))
        })
        .collect();
    let (returned, owned) = signatures
        .into_iter()
        .filter(|row| check_declared_type_row(row) || row["site_syntax_kind_value"] == "RET_TYPE")
        .filter_map(|row| {
            // REQ-RUST-071.0 E1: a field row is owned by the field; its struct is beside it.
            let owner_key = row["site_field_owner_occurrence_key"]
                .as_str()
                .or(row["site_owner_occurrence_key"].as_str())?;
            Some((read_span_graph_identity(owner_key, context)?, row))
        })
        .partition(|(_, row)| row["site_syntax_kind_value"] == "RET_TYPE");
    Ok(CoreMemberLookupIndex {
        self_parameter_source_texts: receivers,
        owner_declared_type_rows: owned,
        owner_return_type_rows: returned,
        core_rename_name_pairs: renames,
        core_declaration_value_rows: declarations,
        scope_proof_withheld_reason: reason,
        scope_prelude_leaf_names: prelude,
        workspace_token_name_counts: tokens,
        workspace_item_leaf_names: nodes
            .iter()
            .filter_map(|node| node.node_qualified_path_text.rsplit("::").next())
            .map(|name| name.trim_start_matches("r#").to_owned())
            .collect(),
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
        WrittenTypeShapeValue::SizedUnknown => "_".to_owned(),
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
        self.list_core_trait_names(name).into_iter().any(|name| {
            self.scope_prelude_leaf_names.contains(name)
                || self.workspace_token_name_counts.contains_key(name)
        })
    }

    /// REQ-RUST-060.0 U2: whether the trait `name` of `core` is in scope through the
    /// prelude only: a prelude file exports one of its names, and the workspace writes
    /// none of them.
    fn check_prelude_only_scope(&self, name: &str) -> bool {
        let names = self.list_core_trait_names(name);
        names
            .iter()
            .any(|name| self.scope_prelude_leaf_names.contains(*name))
            && !names
                .iter()
                .any(|name| self.workspace_token_name_counts.contains_key(*name))
    }

    /// `name` and every name that `core` renames it to, directly or through another.
    fn list_core_trait_names<'index>(&'index self, name: &'index str) -> BTreeSet<&'index str> {
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
                return names;
            }
            names.extend(found);
        }
    }

    /// REQ-RUST-061.0 M1 to M3: the one trait declaration of `core` that declares the
    /// method `method`, when nothing else in `core` or the workspace has that name.
    fn find_sole_trait_member(&self, method: &str) -> Option<(BuiltinMemberNameOutcome, Value)> {
        let rows = self.core_declaration_value_rows.as_slice();
        let named = |row: &Value| row["member_declared_name_value"] == method;
        let of_kind = |row: &Value, kind: &str| row["declaration_owner_kind_value"] == kind;
        let mut declared = rows
            .iter()
            .filter(|row| of_kind(row, "trait_declaration") && named(row));
        let row = declared.next().filter(|_| declared.next().is_none())?;
        let name = row["owner_trait_path_text"]
            .as_str()
            .map(read_path_last_name)
            .filter(|name| !name.is_empty())?;
        let method_form = row["member_self_parameter_kind"]
            .as_str()
            .is_some_and(|kind| kind != "none");
        let other = rows.iter().any(|found| {
            let tokens =
                of_kind(found, "macro_definition_body") || of_kind(found, "macro_call_argument");
            (of_kind(found, "inherent_impl") && named(found))
                || check_macro_member_candidate(found, method, rows)
                || (tokens
                    && (named(found) || found["member_name_parameter_flag"] == true)
                    && found["macro_owner_shape_kind"] == "trait_declaration")
                || (of_kind(found, "trait_impl")
                    && named(found)
                    && found["owner_trait_path_text"]
                        .as_str()
                        .map(read_path_last_name)
                        != Some(name))
        });
        if self.scope_proof_withheld_reason.is_some()
            || self.workspace_item_leaf_names.contains(method)
            || !method_form
            || other
            || !self.check_core_trait_scope(name)
        {
            return None;
        }
        let evidence = json!({
            "receiver_type_proof_kind":"method_name_sole_declaration",
            "target_owner_kind_value":"trait_declaration",
            "owner_trait_name_value":name,
            "method_probe_unread_source":["alloc", "std"],
            "lookup_assumption_values":["source_compiles_method_resolves"],
            "declaration_source_relative_path":row["declaration_source_relative_path"],
            "member_name_start_offset":row["member_name_start_offset"],
            "member_name_end_offset":row["member_name_end_offset"]
        });
        Some((
            Ok(build_core_member_proof(format!("{name}::{method}"))),
            evidence,
        ))
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
                prelude_trait_name_value: row["owner_trait_path_text"]
                    .as_str()
                    .map(read_path_last_name)
                    .filter(|name| !name.is_empty() && self.check_prelude_only_scope(name))
                    .map(str::to_owned),
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
                prelude_trait_name_value: None,
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

/// REQ-RUST-057.0 A2: the type a stored impl writes after `type NAME =`, for a parameter
/// typed `Self::NAME`, with the record of how the impl was found.
#[derive(Debug)]
pub(crate) struct AliasReceiverTypeValue {
    pub(crate) alias_type_source_text: String,
    pub(crate) alias_lookup_evidence_value: Value,
}

/// REQ-RUST-057.0 A3 to A8: the type behind `Self::NAME` in the function `owner`, or the
/// reason it is not proven.
pub(crate) fn find_alias_receiver_type(
    owner: &str,
    name: &str,
    context: &ImplHeaderSourceContext<'_>,
    members: &ImplMemberLookupIndex,
    bounds: &BoundMethodLookupIndex,
) -> Result<Result<AliasReceiverTypeValue, &'static str>, OriginalCallResolutionError> {
    let unproven = Ok(Err("associated_type_impl_unproven"));
    // A3: the function's impl, with a stored trait and self type.
    let Some(enclosing) = members
        .impl_header_value_rows
        .iter()
        .find(|header| {
            header["impl_member_occurrence_rows"]
                .as_array()
                .is_some_and(|rows| {
                    rows.iter()
                        .any(|member| member["member_source_occurrence_key"] == owner)
                })
        })
        .filter(|header| {
            header["impl_function_body_ancestry_flag"] == false
                && check_header_definition_target(&header["impl_trait_type_resolution"])
        })
    else {
        return unproven;
    };
    // A4: the one stored `type NAME` of the self type.
    let self_type = &enclosing["impl_self_type_resolution"];
    let Some(found) = members
        .find_type_member_target("Self", self_type, name, "TYPE_ALIAS")
        .filter(|found| found.member_target_occurrence.is_some())
    else {
        return unproven;
    };
    let Some(declaring) = members.impl_header_value_rows.iter().find(|header| {
        header["impl_semantic_occurrence_key"]
            == found.member_lookup_evidence["impl_header_occurrence"]
    }) else {
        return unproven;
    };
    // A6: the same self type, as stored and as written.
    let written_self = |header: &Value| {
        header["impl_self_type_resolution"]["type_source_evidence"]["type_source_text"].clone()
    };
    if !check_header_definition_target(&declaring["impl_trait_type_resolution"])
        || written_self(declaring).is_null()
        || written_self(declaring) != written_self(enclosing)
    {
        return unproven;
    }
    // A5: the function's own impl, or an impl of a supertrait of its trait.
    if declaring["impl_semantic_occurrence_key"] != enclosing["impl_semantic_occurrence_key"] {
        let identity =
            |header: &Value| -> Result<HldTargetIdentityValue, OriginalCallResolutionError> {
                Ok(decode_symbol_target_value(
                    &header["impl_trait_type_resolution"]["target_identity"],
                )?
                .into())
            };
        if !bounds.check_trait_supertrait_reach(
            &identity(enclosing)?,
            &identity(declaring)?,
            context,
        )? {
            return Ok(Err("associated_type_trait_unproven"));
        }
    }
    // A7: every type path of the written type is proven.
    let member = declaring["impl_member_occurrence_rows"]
        .as_array()
        .and_then(|rows| {
            rows.iter()
                .find(|member| member["member_declared_name_value"] == name)
        });
    let written = member.map(|member| &member["member_alias_type_evidence"]);
    let (Some(member), Some(alias), Some(paths)) = (
        member,
        written.and_then(|written| written["type_source_text"].as_str()),
        written.and_then(|written| written["type_path_range_rows"].as_array()),
    ) else {
        return Ok(Err("receiver_type_name_unproven"));
    };
    let Some(key) = member["member_source_occurrence_key"].as_str() else {
        return Ok(Err("receiver_type_name_unproven"));
    };
    let span = context.span(key)?;
    for path in paths {
        if !check_written_path_proven(span, path, context)? {
            return Ok(Err("receiver_type_name_unproven"));
        }
    }
    Ok(Ok(AliasReceiverTypeValue {
        alias_type_source_text: alias.to_owned(),
        alias_lookup_evidence_value: json!({
            "associated_type_alias_text":alias,
            "associated_type_member_occurrence":member["member_semantic_occurrence_key"],
            "associated_type_impl_occurrence":declaring["impl_semantic_occurrence_key"],
            "enclosing_impl_header_occurrence":enclosing["impl_semantic_occurrence_key"]
        }),
    }))
}

/// One `&` or `&mut` taken off the front of a written type; `None` when there is none.
fn strip_reference_layer_prefix<'text>(written: &&'text str) -> Option<&'text str> {
    let inner = written.strip_prefix('&')?.trim_start();
    Some(inner.strip_prefix("mut ").map_or(inner, str::trim_start))
}

/// REQ-RUST-057.0 A2: the reference layers and `NAME` of a written type that is
/// `Self::NAME` behind `&` and `&mut` only. White space between the tokens changes
/// nothing: a macro expansion writes `&mut Self ::NAME`.
fn read_self_alias_parts(text: &str) -> Option<(&str, &str)> {
    let text = text.trim();
    let rest = std::iter::successors(Some(text), strip_reference_layer_prefix).last()?;
    let name = rest
        .strip_prefix("Self")?
        .trim_start()
        .strip_prefix("::")?
        .trim_start();
    (!name.is_empty()
        && name
            .chars()
            .all(|found| found.is_alphanumeric() || found == '_'))
    .then(|| (&text[..text.len() - rest.len()], name))
}

/// What the base of a receiver place proves: the written type its index steps start
/// from, or the reason that type is withheld; with the record of the lookup so far.
type ReceiverPlaceTypeOutcome = (Result<String, &'static str>, Value);

/// REQ-RUST-055.0, REQ-RUST-057.0: the type a parameter writes, each name in it proven.
/// `None` when the parameter's evidence holds no written type.
fn find_parameter_place_type(
    span: &Value,
    first: &Value,
    context: &ImplHeaderSourceContext<'_>,
    members: &ImplMemberLookupIndex,
    index: &CoreMemberLookupIndex,
    bounds: &BoundMethodLookupIndex,
) -> Result<Option<ReceiverPlaceTypeOutcome>, OriginalCallResolutionError> {
    let written = &first["receiver_written_type_evidence"];
    let Some(text) = written["type_source_text"].as_str() else {
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
    let Some((layers, name)) = read_self_alias_parts(text) else {
        let Some(paths) = written["type_path_range_rows"].as_array() else {
            return withheld("receiver_type_name_unproven", evidence);
        };
        for path in paths {
            if !check_written_path_proven(span, path, context)? {
                return withheld("receiver_type_name_unproven", evidence);
            }
        }
        return Ok(Some((Ok(text.to_owned()), evidence)));
    };
    // REQ-RUST-057.0: the type is what a stored impl writes for `NAME`.
    evidence["receiver_type_proof_kind"] = json!("parameter_associated_type_alias");
    let Some(owner) = span["site_owner_occurrence_key"].as_str() else {
        return withheld("associated_type_impl_unproven", evidence);
    };
    match find_alias_receiver_type(owner, name, context, members, bounds)? {
        Ok(found) => {
            for (key, value) in found
                .alias_lookup_evidence_value
                .as_object()
                .into_iter()
                .flatten()
            {
                evidence[key.as_str()] = value.clone();
            }
            if let Some(assumed) = evidence["lookup_assumption_values"].as_array_mut() {
                assumed.push(json!("source_compiles_one_associated_type"));
            }
            Ok(Some((
                Ok(format!("{layers}{}", found.alias_type_source_text)),
                evidence,
            )))
        }
        Err(reason) => withheld(reason, evidence),
    }
}

/// Whether `text` is a function that returns its written return type: not `async`.
fn check_plain_function_text(text: &str) -> bool {
    SourceFile::parse(text, Edition::Edition2021)
        .tree()
        .syntax()
        .descendants()
        .find_map(ast::Fn::cast)
        .is_some_and(|function| function.async_token().is_none())
}

/// REQ-RUST-064.0 L2 to L7: the type of a receiver that is a local variable which one
/// plain `let` binds to a method call: the return type that the one workspace function
/// named by that call declares. `None` when the receiver stores no such `let`.
fn find_local_binding_type(
    span: &Value,
    receiver: &Value,
    context: &ImplHeaderSourceContext<'_>,
    index: &CoreMemberLookupIndex,
    bounds: &BoundMethodLookupIndex,
) -> Result<Option<ReceiverPlaceTypeOutcome>, OriginalCallResolutionError> {
    let local = &receiver["receiver_local_binding_evidence"];
    let Some(name) = receiver["receiver_declared_name_value"]
        .as_str()
        .filter(|_| !local.is_null())
    else {
        return Ok(None);
    };
    let mut evidence = json!({
        "receiver_type_proof_kind":"local_binding_call_result",
        "receiver_binding_name_value":name,
        "binding_statement_start_offset":local["binding_statement_start_offset"],
        "binding_statement_end_offset":local["binding_statement_end_offset"],
        "binding_name_token_count":local["binding_name_token_count"],
        "method_probe_unread_source":["alloc", "std"],
        "lookup_assumption_values":["core_macro_expansions_import_no_trait"]
    });
    let withheld = |reason: &'static str, evidence: Value| Ok(Some((Err(reason), evidence)));
    if let Some(reason) = index.scope_proof_withheld_reason {
        return withheld(reason, evidence);
    }
    // L2: the function is written in a source file, and no other token of the unit
    // writes the name, so no item of that name exists for the pattern to match.
    let original = span["span_expansion_provenance_chain"]
        .as_array()
        .is_some_and(Vec::is_empty);
    let written = index
        .workspace_token_name_counts
        .get(name)
        .and_then(|count| u64::try_from(*count).ok());
    if !original
        || index.scope_prelude_leaf_names.contains(name)
        || written.is_none()
        || written != local["binding_name_token_count"].as_u64()
    {
        return withheld("local_binding_name_unproven", evidence);
    }
    // L3: the initializer names one workspace function through a bound.
    let Some(owner) = span["site_owner_occurrence_key"].as_str() else {
        return withheld("local_initializer_call_unproven", evidence);
    };
    let call = &local["binding_initializer_call_evidence"];
    let walked = list_bound_method_candidates(owner, context.span(owner)?, call, context, bounds)?;
    let Some((target, walk)) = walked.and_then(|(found, walk)| {
        let mut found = found.into_iter();
        found
            .next()
            .filter(|_| found.next().is_none())
            .zip(Some(walk))
    }) else {
        return withheld("local_initializer_call_unproven", evidence);
    };
    evidence["binding_initializer_bound_walk"] = walk.build_walk_evidence_value();
    // L4: that function declares one return type, and returns it.
    let rows = index
        .owner_return_type_rows
        .iter()
        .filter(|(owner, _)| *owner == target)
        .map(|(_, row)| row)
        .collect::<Vec<_>>();
    let ([row], Some(key)) = (
        rows.as_slice(),
        rows.first()
            .and_then(|row| row["site_owner_occurrence_key"].as_str()),
    ) else {
        return withheld("local_initializer_return_unproven", evidence);
    };
    let declared = context.span(key)?;
    evidence["binding_initializer_target_occurrence"] = context.occurrence(declared)?;
    // L3: a bound trait with no source may declare the name too. A method that takes
    // `self` by value is found at the first step, where a second one is an error.
    if !walk.missing_source_traits.is_empty() {
        let by_value = index
            .self_parameter_source_texts
            .get(key)
            .is_some_and(|text| matches!(text.trim(), "self" | "mut self"));
        if !by_value {
            return withheld("local_initializer_call_unproven", evidence);
        }
        if let Some(assumed) = evidence["lookup_assumption_values"].as_array_mut() {
            assumed.push(json!("source_compiles_method_resolves"));
        }
    }
    let text = row["site_syntax_source_text"]
        .as_str()
        .and_then(|text| text.strip_prefix("->"))
        .map(str::trim);
    let (Some(text), true) = (
        text,
        declared["span_syntax_kind_value"] == "FN"
            && declared["span_source_code_text"]
                .as_str()
                .is_some_and(check_plain_function_text),
    ) else {
        return withheld("local_initializer_return_unproven", evidence);
    };
    evidence["receiver_written_type_text"] = json!(text);
    evidence["binding_return_type_file_path"] = row["file_source_absolute_path"].clone();
    evidence["binding_return_type_start_offset"] = row["span_byte_start_offset"].clone();
    evidence["binding_return_type_end_offset"] = row["span_byte_end_offset"].clone();
    // L5: every name of the type is a builtin type or a name of `core`.
    let Some(paths) = row["type_path_source_rows"].as_array() else {
        return withheld("receiver_type_name_unproven", evidence);
    };
    for path in paths {
        let proven = path["type_path_self_flag"] != true
            && match build_row_path_evidence(row, path) {
                Some(site) => check_written_path_proven(&site, path, context)?,
                None => false,
            };
        if !proven {
            return withheld("receiver_type_name_unproven", evidence);
        }
    }
    Ok(Some((Ok(text.to_owned()), evidence)))
}

/// The stored lookup evidence of one type path that the stored row `row` writes.
fn build_row_path_evidence(row: &Value, path: &Value) -> Option<Value> {
    let text = path["type_source_text"].as_str()?;
    Some(json!({
        "unit_compilation_identity_key":row["unit_compilation_identity_key"],
        "file_source_absolute_path":row["file_source_absolute_path"],
        "span_byte_start_offset":path["span_byte_start_offset"],
        "span_byte_end_offset":path["span_byte_end_offset"],
        "span_source_code_text":text,
        "span_content_digest_full":blake3::hash(text.as_bytes()).to_hex().to_string(),
        "span_expansion_provenance_chain":row["span_expansion_provenance_chain"]
    }))
}

/// The one definition of symbol kind `kind` that a stored path lookup resolved to.
fn read_record_definition_identity(
    record: &Value,
    kind: &str,
    context: &ImplHeaderSourceContext<'_>,
) -> Result<Option<HldTargetIdentityValue>, OriginalCallResolutionError> {
    let Some([symbol]) = record["path_target_symbol_rows"]
        .as_array()
        .map(Vec::as_slice)
        .filter(|_| record["path_resolution_state_kind"] == "resolved")
    else {
        return Ok(None);
    };
    if symbol["target_symbol_kind_value"] != kind {
        return Ok(None);
    }
    Ok(context
        .read_single_target_identity(symbol)?
        .map(|identity| decode_symbol_target_value(&identity))
        .transpose()?
        .map(HldTargetIdentityValue::from))
}

/// REQ-RUST-059.0 S3: the declared name of a record field, and the type the field
/// writes. `None` when the stored text is not one field, or the field has an attribute.
/// A field that a `cfg` removed has no stored row; an attribute on a stored field is not
/// read here and may remove it, and then no position among the stored fields is proven.
fn parse_stored_field_text(row: &Value) -> Option<(Option<String>, String)> {
    let text = row["site_syntax_source_text"].as_str()?;
    let tuple = row["site_syntax_kind_value"] == "TUPLE_FIELD";
    let wrapped = if tuple {
        format!("struct X({text});")
    } else {
        format!("struct X {{ {text} }}")
    };
    let parse = SourceFile::parse(&wrapped, Edition::Edition2021);
    if !parse.errors().is_empty() {
        return None;
    }
    let tree = parse.tree();
    let written = |ty: Option<ast::Type>| ty.map(|ty| ty.syntax().text().to_string());
    if tuple {
        let mut fields = tree
            .syntax()
            .descendants()
            .filter_map(ast::TupleField::cast);
        let field = fields.next().filter(|_| fields.next().is_none())?;
        (field.attrs().next().is_none())
            .then(|| written(field.ty()))
            .flatten()
            .map(|ty| (None, ty))
    } else {
        let mut fields = tree
            .syntax()
            .descendants()
            .filter_map(ast::RecordField::cast);
        let field = fields.next().filter(|_| fields.next().is_none())?;
        let name = field.name()?.text().trim_start_matches("r#").to_owned();
        (field.attrs().next().is_none())
            .then(|| written(field.ty()))
            .flatten()
            .map(|ty| (Some(name), ty))
    }
}

impl CoreMemberLookupIndex {
    /// REQ-RUST-059.0 S2: the one stored declared type of the `static` `owner`.
    fn find_static_declared_type(&self, owner: &HldTargetIdentityValue) -> Option<&Value> {
        let mut rows = self
            .owner_declared_type_rows
            .iter()
            .filter(|(found, row)| found == owner && row["declared_type_owner_kind"] == "STATIC")
            .map(|(_, row)| row);
        rows.next().filter(|_| rows.next().is_none())
    }

    /// REQ-RUST-059.0 S3: the stored field `name` of the struct `owner`, with the type
    /// it writes: a tuple field by its position among the stored fields, a record field
    /// by its name. `None` when any field of the struct is not read.
    fn find_struct_field_type(
        &self,
        owner: &HldTargetIdentityValue,
        name: &str,
    ) -> Option<(&Value, String)> {
        let mut rows = self
            .owner_declared_type_rows
            .iter()
            .filter(|(found, row)| found == owner && row["declared_type_owner_kind"].is_null())
            .map(|(_, row)| row)
            .collect::<Vec<_>>();
        rows.sort_by_key(|row| row["span_byte_start_offset"].as_u64());
        let fields = rows
            .iter()
            .map(|row| parse_stored_field_text(row))
            .collect::<Option<Vec<_>>>()?;
        rows.into_iter()
            .zip(fields)
            .enumerate()
            .find(|(position, (_, (declared, _)))| match declared {
                Some(declared) => declared == name,
                None => position.to_string() == name,
            })
            .map(|(_, (row, (_, written)))| (row, written))
    }
}

/// REQ-RUST-059.0 S3: the field `name` of the workspace struct that the type `written`
/// of the stored row `row` names: one path without generic arguments, behind `&` and
/// `&mut` only, that the stored lookup resolved to one struct.
fn find_field_step_type<'index>(
    row: &Value,
    written: &str,
    name: &str,
    context: &ImplHeaderSourceContext<'_>,
    index: &'index CoreMemberLookupIndex,
) -> Result<Option<(&'index Value, String)>, OriginalCallResolutionError> {
    let shape = parse_written_type_shape(written, &[]);
    let (WrittenTypeShapeValue::Named { arguments, .. }, Some([path])) = (
        strip_shape_reference_layers(&shape),
        row["type_path_source_rows"].as_array().map(Vec::as_slice),
    ) else {
        return Ok(None);
    };
    if !arguments.is_empty() || path["type_path_self_flag"] == true {
        return Ok(None);
    }
    let Some(evidence) = build_row_path_evidence(row, path) else {
        return Ok(None);
    };
    let [record] = context.read_type_path_records(&evidence, path)? else {
        return Ok(None);
    };
    Ok(read_record_definition_identity(record, "Struct", context)?
        .and_then(|owner| index.find_struct_field_type(&owner, name)))
}

/// The stored field row a place has reached, the type it writes, and the record of each
/// field step walked.
type FieldPlaceWalkValue<'index> = (Vec<Value>, &'index Value, String);

fn build_field_step_record(name: &str, field: &Value, written: &str) -> Value {
    json!({
        "field_declared_name_value":name,
        "field_source_text":field["site_syntax_source_text"],
        "field_type_source_text":written
    })
}

/// REQ-RUST-059.0 S3: each field step of `fields`, from the struct the type so far
/// names. `None` when a step is not proven.
fn walk_field_step_types<'index>(
    start: Option<FieldPlaceWalkValue<'index>>,
    fields: &[Value],
    context: &ImplHeaderSourceContext<'_>,
    index: &'index CoreMemberLookupIndex,
) -> Result<Option<FieldPlaceWalkValue<'index>>, OriginalCallResolutionError> {
    fields.iter().try_fold(start, |state, step| {
        let (Some((rows, row, written)), Some(name)) =
            (state, step["field_declared_name_value"].as_str())
        else {
            return Ok(None);
        };
        Ok(
            find_field_step_type(row, &written, name, context, index)?.map(|(field, written)| {
                let walked = build_field_step_record(name, field, &written);
                (
                    rows.into_iter().chain([walked]).collect::<Vec<_>>(),
                    field,
                    written,
                )
            }),
        )
    })
}

/// REQ-RUST-059.0 S3, S5: the outcome of a walk over field steps: the type the last
/// stored row writes, when every step is proven and every type path of that type is.
fn decide_field_place_type(
    walked: Option<FieldPlaceWalkValue<'_>>,
    mut evidence: Value,
    context: &ImplHeaderSourceContext<'_>,
) -> Result<ReceiverPlaceTypeOutcome, OriginalCallResolutionError> {
    let Some((rows, row, written)) = walked else {
        return Ok((Err("receiver_field_type_unproven"), evidence));
    };
    if !rows.is_empty() {
        evidence["receiver_field_step_rows"] = json!(rows);
    }
    let Some(paths) = row["type_path_source_rows"].as_array() else {
        return Ok((Err("receiver_type_name_unproven"), evidence));
    };
    for path in paths {
        let proven = match build_row_path_evidence(row, path) {
            Some(site) => check_written_path_proven(&site, path, context)?,
            None => false,
        };
        if !proven {
            return Ok((Err("receiver_type_name_unproven"), evidence));
        }
    }
    Ok((Ok(written), evidence))
}

/// REQ-RUST-059.0 S1 to S5: the type written for the place `first` then `fields`, when
/// `first` is a value path that the stored lookup resolved to one `static` of the
/// workspace. `None` when it names no such `static`.
fn find_static_place_type(
    span: &Value,
    first: &Value,
    fields: &[Value],
    context: &ImplHeaderSourceContext<'_>,
    index: &CoreMemberLookupIndex,
) -> Result<Option<ReceiverPlaceTypeOutcome>, OriginalCallResolutionError> {
    // S1: the value path names one `static`.
    let Some(record) = context
        .read_type_path_records(span, first)?
        .iter()
        .find(|record| record["site_path_context_kind"] == "body_value_path")
    else {
        return Ok(None);
    };
    let Some(owner) = read_record_definition_identity(record, "Static", context)? else {
        return Ok(None);
    };
    // S2: the type the `static` declares, as written.
    let Some((declared, text)) = index
        .find_static_declared_type(&owner)
        .and_then(|row| Some((row, row["site_syntax_source_text"].as_str()?)))
    else {
        return Ok(None);
    };
    let evidence = json!({
        "receiver_type_proof_kind":if fields.is_empty() {
            "static_declared_type"
        } else {
            "static_declared_type_field"
        },
        "receiver_written_type_text":text,
        "receiver_static_path_text":first["path_source_text"],
        "method_probe_unread_source":["alloc", "std"],
        "lookup_assumption_values":["core_macro_expansions_import_no_trait"]
    });
    if let Some(reason) = index.scope_proof_withheld_reason {
        return Ok(Some((Err(reason), evidence)));
    }
    let start = Some((Vec::new(), declared, text.to_owned()));
    let walked = walk_field_step_types(start, fields, context, index)?;
    decide_field_place_type(walked, evidence, context).map(Some)
}

/// REQ-RUST-059.0 S7: the type written for `self` then `fields` in the function `span`
/// is in. `None` when the function has no plain `self`, `&self` or `&mut self`, when it
/// is in no stored impl whose self type is one plain name resolved to a definition, or
/// when no field step follows.
fn find_self_place_type(
    span: &Value,
    fields: &[Value],
    context: &ImplHeaderSourceContext<'_>,
    members: &ImplMemberLookupIndex,
    index: &CoreMemberLookupIndex,
) -> Result<Option<ReceiverPlaceTypeOutcome>, OriginalCallResolutionError> {
    let (Some(owner), Some((step, rest))) = (
        span["site_owner_occurrence_key"].as_str(),
        fields.split_first(),
    ) else {
        return Ok(None);
    };
    let Some(parameter) = index.self_parameter_source_texts.get(owner) else {
        return Ok(None);
    };
    let layers = match parameter.split_whitespace().collect::<String>().as_str() {
        "self" | "mutself" => "",
        "&self" => "&",
        "&mutself" => "&mut ",
        _ => return Ok(None),
    };
    let Some((header, written)) = members
        .impl_header_value_rows
        .iter()
        .find(|header| {
            header["impl_member_occurrence_rows"]
                .as_array()
                .is_some_and(|rows| {
                    rows.iter()
                        .any(|member| member["member_source_occurrence_key"] == owner)
                })
        })
        .filter(|header| {
            header["impl_function_body_ancestry_flag"] == false
                && check_header_definition_target(&header["impl_self_type_resolution"])
        })
        .and_then(|header| {
            let written =
                header["impl_self_type_resolution"]["type_source_evidence"]["type_source_text"]
                    .as_str()?;
            matches!(
                parse_written_type_shape(written, &[]),
                WrittenTypeShapeValue::Named { arguments, .. } if arguments.is_empty()
            )
            .then_some((header, written))
        })
    else {
        return Ok(None);
    };
    let identity: HldTargetIdentityValue =
        decode_symbol_target_value(&header["impl_self_type_resolution"]["target_identity"])?.into();
    let evidence = json!({
        "receiver_type_proof_kind":"self_declared_type_field",
        "receiver_written_type_text":format!("{layers}{written}"),
        "receiver_self_parameter_text":parameter,
        "enclosing_impl_header_occurrence":header["impl_semantic_occurrence_key"],
        "method_probe_unread_source":["alloc", "std"],
        "lookup_assumption_values":["core_macro_expansions_import_no_trait"]
    });
    if let Some(reason) = index.scope_proof_withheld_reason {
        return Ok(Some((Err(reason), evidence)));
    }
    // The first field is of the impl's self type; the rest are walked as for a `static`.
    let start = step["field_declared_name_value"].as_str().and_then(|name| {
        let (field, written) = index.find_struct_field_type(&identity, name)?;
        Some((
            vec![build_field_step_record(name, field, &written)],
            field,
            written,
        ))
    });
    let walked = walk_field_step_types(start, rest, context, index)?;
    decide_field_place_type(walked, evidence, context).map(Some)
}

/// The `core` member that the method call at `span` names, with the record of its
/// lookup, or the reason the name is withheld. `None` when the lookup does not apply:
/// no declarations stored, or a name that more than one declaration may answer
/// (REQ-RUST-061.0) on a receiver that is neither a parameter with a written type nor a
/// `static` of the workspace or `self`, followed by fields (REQ-RUST-059.0).
pub(crate) fn find_core_member_target(
    span: &Value,
    receiver: &Value,
    context: &ImplHeaderSourceContext<'_>,
    members: &ImplMemberLookupIndex,
    index: &CoreMemberLookupIndex,
    bounds: &BoundMethodLookupIndex,
) -> Result<Option<(BuiltinMemberNameOutcome, Value)>, OriginalCallResolutionError> {
    let declarations = index.core_declaration_value_rows.as_slice();
    let Some(method) = receiver["method_declared_name_value"]
        .as_str()
        .filter(|_| !declarations.is_empty())
    else {
        return Ok(None);
    };
    // REQ-RUST-061.0: a name that one trait of `core` declares needs no receiver type.
    if let Some(found) = index.find_sole_trait_member(method) {
        return Ok(Some(found));
    }
    // REQ-RUST-062.0 C2: a receiver that is a method call.
    let inner = &receiver["receiver_inner_call_evidence"];
    if !inner.is_null() {
        return find_call_result_member(span, inner, method, context, members, index, bounds);
    }
    // REQ-RUST-064.0: a receiver that is a local variable bound to a method call.
    if let Some((written, evidence)) =
        find_local_binding_type(span, receiver, context, index, bounds)?
    {
        return match written {
            Ok(text) => name_probed_core_member(
                &[parse_written_type_shape(&text, &[])],
                method,
                evidence,
                members,
                index,
                &[],
            ),
            Err(reason) => Ok(Some((Err(reason), evidence))),
        };
    }
    let Some([first, places @ ..]) = receiver["receiver_place_step_rows"]
        .as_array()
        .map(Vec::as_slice)
    else {
        return Ok(None);
    };
    // REQ-RUST-059.0 S4: field steps first, then index steps only.
    let (fields, steps) = places.split_at(
        places
            .iter()
            .take_while(|step| step["place_step_kind"] == "field_access")
            .count(),
    );
    if !steps
        .iter()
        .all(|step| step["place_step_kind"] == "index_access")
    {
        return Ok(None);
    }
    let place = match first["place_step_kind"].as_str() {
        Some("parameter_identifier") if fields.is_empty() => {
            find_parameter_place_type(span, first, context, members, index, bounds)?
        }
        Some("value_path") => find_static_place_type(span, first, fields, context, index)?,
        Some("self_value") => find_self_place_type(span, fields, context, members, index)?,
        _ => None,
    };
    let withheld = |reason: &'static str, evidence: Value| Ok(Some((Err(reason), evidence)));
    let Some((written, evidence)) = place else {
        // REQ-RUST-063.0 M6: a const of a builtin type, and nothing after it.
        let constant = (first["place_step_kind"] == "value_path" && places.is_empty())
            .then(|| {
                find_builtin_constant_member(span, first, method, context, members, declarations)
            })
            .transpose()?
            .flatten();
        return Ok(constant.map(|(outcome, record)| {
            let outcome = match index.scope_proof_withheld_reason {
                Some(reason) => Err(reason),
                None => outcome.map(build_core_member_proof),
            };
            (outcome, record)
        }));
    };
    let shape_text = match written {
        Ok(text) => text,
        Err(reason) => return withheld(reason, evidence),
    };
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
    let mut shapes = vec![parse_written_type_shape(&shape_text, &[])];
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
    name_probed_core_member(&shapes, method, evidence, members, index, &[])
}

/// The strings of the array `row[key]`; none when the row stores no such array.
fn list_row_name_values(row: &Value, key: &str) -> Vec<String> {
    row[key]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

/// REQ-RUST-062.0 C2 to C6: the member `method` names on a receiver that is the method
/// call `inner`. The receiver's type is the return type that the declaration naming
/// `inner` writes. `None` when `inner` has no lookup, or its declaration stores no return
/// type; the withheld reason of `inner` is kept.
fn find_call_result_member(
    span: &Value,
    inner: &Value,
    method: &str,
    context: &ImplHeaderSourceContext<'_>,
    members: &ImplMemberLookupIndex,
    index: &CoreMemberLookupIndex,
    bounds: &BoundMethodLookupIndex,
) -> Result<Option<(BuiltinMemberNameOutcome, Value)>, OriginalCallResolutionError> {
    let Some((outcome, lookup)) =
        find_core_member_target(span, inner, context, members, index, bounds)?
    else {
        return Ok(None);
    };
    let cited = [
        "declaration_source_relative_path",
        "member_name_start_offset",
        "member_name_end_offset",
    ];
    let declared = index
        .core_declaration_value_rows
        .iter()
        .find(|row| cited.iter().all(|key| row[*key] == lookup[*key]))
        .and_then(|row| Some((row, row["member_return_type_text"].as_str()?)));
    let assumed = lookup["lookup_assumption_values"].clone();
    let mut evidence = json!({
        "receiver_type_proof_kind":"call_result_declared_type",
        "receiver_inner_call_lookup":lookup,
        "method_probe_unread_source":["alloc", "std"],
        "lookup_assumption_values":assumed
    });
    let label = match outcome {
        Ok(ExternalNameProofRecord {
            external_target_identity_value: HldTargetIdentityValue::ExternalNamed(key),
            ..
        }) => key.external_item_path_text,
        Ok(_) => return Ok(None),
        Err(reason) => return Ok(Some((Err(reason), evidence))),
    };
    let Some((row, returned)) = declared else {
        return Ok(None);
    };
    evidence["receiver_inner_call_target_label"] = json!(label);
    evidence["receiver_return_type_text"] = json!(returned);
    // C4: the names the declaration's owner declares, less those not proven `Sized`.
    let relaxed = list_row_name_values(row, "owner_unsized_relaxed_names");
    let owner_generics = [
        "trait_generic_parameter_names",
        "impl_generic_parameter_names",
    ]
    .iter()
    .flat_map(|key| list_row_name_values(row, key))
    .collect::<Vec<_>>();
    let sized = owner_generics
        .iter()
        .cloned()
        .chain(list_row_name_values(row, "trait_associated_type_names"))
        .filter(|name| !relaxed.contains(name))
        .collect::<Vec<_>>();
    let generics = owner_generics
        .into_iter()
        .chain(list_row_name_values(row, "member_generic_parameter_names"))
        .collect::<Vec<_>>();
    let shape = parse_return_type_shape(returned, &sized, &generics);
    // C8, C13: a name `core` declares more than once is proven by its file's scope.
    let declarations = index.core_declaration_value_rows.as_slice();
    let Some(proofs) = prove_return_type_names(&shape, row, method, declarations) else {
        evidence["receiver_type_text"] = json!(render_written_type_shape(&shape));
        return Ok(Some((Err("receiver_type_name_unproven"), evidence)));
    };
    let proven = proofs
        .iter()
        .filter_map(|proof| proof["type_name_value"].as_str().map(str::to_owned))
        .collect::<Vec<_>>();
    if !proofs.is_empty() {
        evidence["receiver_type_name_proofs"] = json!(proofs);
    }
    name_probed_core_member(&[shape], method, evidence, members, index, &proven)
}

/// REQ-RUST-062.0 C12: whether the impl that holds the member `row` writes the type name
/// of `proof` in its self type, and its own file does not prove it the same declaration.
fn check_impl_name_differs(row: &Value, proof: &Value, declarations: &[Value]) -> bool {
    let name = proof["type_name_value"].as_str().unwrap_or("");
    let generics = list_row_name_values(row, "impl_generic_parameter_names");
    let written =
        parse_written_type_shape(row["impl_self_type_text"].as_str().unwrap_or(""), &generics);
    list_shape_type_names(&written).contains(&name)
        && !(row["owner_file_scope_flag"] == true
            && row["declaration_source_relative_path"]
                .as_str()
                .and_then(|file| find_file_scope_type(declarations, file, name))
                .is_some_and(|found| {
                    found["type_declaration_site_value"] == proof["type_declaration_site_value"]
                }))
}

/// REQ-RUST-062.0 C8, C10, C12: one proof per type name of `shape` that the name alone
/// does not prove, where `shape` is the return type the member `row` writes. `None` when
/// a name is not proven, or an impl that declares `method` may mean another type by it.
fn prove_return_type_names(
    shape: &WrittenTypeShapeValue,
    row: &Value,
    method: &str,
    declarations: &[Value],
) -> Option<Vec<Value>> {
    let names = list_unproven_type_names(shape, declarations);
    let file = row["declaration_source_relative_path"]
        .as_str()
        .filter(|_| names.is_empty() || row["owner_file_scope_flag"] == true)?;
    names
        .iter()
        .map(|name| {
            find_file_scope_type(declarations, file, name).filter(|proof| {
                !declarations
                    .iter()
                    .filter(|found| {
                        found["member_declared_name_value"] == method
                            && matches!(
                                found["declaration_owner_kind_value"].as_str(),
                                Some("inherent_impl" | "trait_impl")
                            )
                    })
                    .any(|found| check_impl_name_differs(found, proof, declarations))
            })
        })
        .collect()
}

/// REQ-RUST-056.0 P2 to P5: the declaration the probe finds for `method` on a receiver
/// whose type is one of `shapes`, written into the lookup record `evidence`.
/// `proven_type_names` are the type names REQ-RUST-062.0 C10 proved for the shapes.
fn name_probed_core_member(
    shapes: &[WrittenTypeShapeValue],
    method: &str,
    mut evidence: Value,
    members: &ImplMemberLookupIndex,
    index: &CoreMemberLookupIndex,
    proven_type_names: &[String],
) -> Result<Option<(BuiltinMemberNameOutcome, Value)>, OriginalCallResolutionError> {
    let declarations = index.core_declaration_value_rows.as_slice();
    let withheld = |reason: &'static str, evidence: Value| Ok(Some((Err(reason), evidence)));
    let scope_traits = match index.list_scope_trait_methods(method, members) {
        Ok(found) => found,
        Err(reason) => return withheld(reason, evidence),
    };
    let outcomes = shapes
        .iter()
        .map(|shape| {
            (
                shape,
                probe_core_member_declaration(
                    shape,
                    method,
                    declarations,
                    &scope_traits,
                    proven_type_names,
                ),
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
            prelude_trait_assumed_names,
        } => {
            // REQ-RUST-060.0 U3: the row says which prelude traits it assumes away.
            if !prelude_trait_assumed_names.is_empty() {
                evidence["prelude_trait_assumed_names"] = json!(prelude_trait_assumed_names);
                if let Some(assumed) = evidence["lookup_assumption_values"].as_array_mut() {
                    assumed.push(json!("prelude_trait_impl_unrefuted"));
                }
            }
            (&declarations[*declaration_row_index], probe_step_text)
        }
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
        Ok(build_core_member_proof(format!("{owner}::{method}"))),
        evidence,
    )))
}

/// The stored name of a member that a declaration of `core` proves.
fn build_core_member_proof(path: String) -> ExternalNameProofRecord {
    ExternalNameProofRecord {
        external_target_identity_value: HldTargetIdentityValue::ExternalNamed(
            HldExternalNamedKey {
                external_name_kind_value: HldExternalNameKind::CoreDeclaredMember,
                external_crate_name_value: Some("core".to_owned()),
                external_item_path_text: path,
            },
        ),
        external_proof_metadata_value: build_external_proof_value(
            "core_declared_member",
            "core_declaration_probe",
            Value::Null,
        ),
    }
}

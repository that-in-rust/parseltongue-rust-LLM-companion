use crate::impl_header_source_records::{
    ImplHeaderSourceContext, ImplHeaderSourceError, read_impl_stage_values,
};
use parseltongue_core_row_types::*;
use parseltongue_table_store_traits::StageTableStorePort;
use serde_json::{Value, json};

#[cfg(test)]
#[path = "type-relative-member-tests.rs"]
mod type_relative_member_tests;

/// The stored impl headers of one run, for finding the member a path such as
/// `Self::NAME` or `u8::NAME` names (REQ-RUST-052.0 G5).
#[derive(Default)]
pub(super) struct ImplMemberLookupIndex {
    pub(super) impl_header_value_rows: Vec<Value>,
}

pub(super) fn read_impl_member_index(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
) -> Result<ImplMemberLookupIndex, ImplHeaderSourceError> {
    Ok(ImplMemberLookupIndex {
        impl_header_value_rows: read_impl_stage_values(store, scope, "impl-header-index-table")?,
    })
}

/// The first segment of a two-segment path such as `T::NAME`, when it is one
/// plain identifier, and the member's name.
pub(super) fn read_type_relative_parts(text: &str) -> Option<(&str, &str)> {
    let (prefix, member) = text.split_once("::")?;
    let (prefix, member) = (prefix.trim(), member.trim());
    let plain = |name: &str| {
        !name.is_empty()
            && name
                .chars()
                .all(|letter| letter.is_alphanumeric() || letter == '_')
    };
    (plain(prefix) && plain(member)).then_some((prefix, member))
}

/// The parts of a qualified path `<A as T>::NAME`: the anchor type `A` and the trait path
/// `T`, each with its byte offset in the path's text, and the member's name
/// (REQ-RUST-052.0 G14).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct QualifiedPathPartsValue<'a> {
    pub(super) anchor_type_source_text: &'a str,
    pub(super) anchor_type_text_offset: usize,
    pub(super) trait_path_source_text: &'a str,
    pub(super) trait_path_text_offset: usize,
    pub(super) member_declared_name_value: &'a str,
}

/// `None` when `text` is not `<A as T>::NAME` with `A` and `T` plain paths without
/// generic arguments and `NAME` one identifier.
pub(super) fn read_qualified_path_parts(text: &str) -> Option<QualifiedPathPartsValue<'_>> {
    let inner = text.strip_prefix('<')?;
    let (inner, tail) = inner.split_once('>')?;
    let (anchor, named) = inner.split_once(" as ")?;
    let member = tail.trim_start().strip_prefix("::")?.trim();
    // A path of identifiers: no generic arguments, reference, slice or second `as`.
    let plain_path = |path: &str| {
        !path.is_empty()
            && path.split("::").all(|name| {
                let name = name.trim();
                !name.is_empty()
                    && name
                        .chars()
                        .all(|letter| letter.is_alphanumeric() || letter == '_' || letter == '#')
            })
    };
    let offset = |part: &str| part.as_ptr() as usize - text.as_ptr() as usize;
    let (anchor, named) = (anchor.trim(), named.trim());
    (plain_path(anchor) && plain_path(named) && plain_path(member) && !member.contains("::")).then(
        || QualifiedPathPartsValue {
            anchor_type_source_text: anchor,
            anchor_type_text_offset: offset(anchor),
            trait_path_source_text: named,
            trait_path_text_offset: offset(named),
            member_declared_name_value: member,
        },
    )
}

/// The impl member a qualified path `<A as T>::NAME` names, read from the stored lookups
/// of `A` and `T` (REQ-RUST-052.0 G14). `None` when `text` is not such a path.
pub(super) fn find_qualified_path_member<'a>(
    evidence: &Value,
    text: &str,
    context: &ImplHeaderSourceContext<'_>,
    members: &'a ImplMemberLookupIndex,
    kind: &str,
) -> Result<Option<TypeMemberLookupValue<'a>>, ImplHeaderSourceError> {
    let Some(parts) = read_qualified_path_parts(text) else {
        return Ok(None);
    };
    let start =
        evidence["span_byte_start_offset"]
            .as_u64()
            .ok_or(ImplHeaderSourceError::InvalidField {
                table: "path-resolution-site-records",
                field: "span_byte_start_offset",
            })?;
    let part = |offset: usize, text: &str| {
        json!({
            "span_byte_start_offset":start + offset as u64,
            "span_byte_end_offset":start + (offset + text.len()) as u64,
            "type_simple_path_flag":true,
            "type_source_text":text
        })
    };
    let anchor = context.resolve_type(
        evidence,
        &part(parts.anchor_type_text_offset, parts.anchor_type_source_text),
        false,
    )?;
    let named = context.resolve_type(
        evidence,
        &part(parts.trait_path_text_offset, parts.trait_path_source_text),
        true,
    )?;
    Ok(Some(members.find_qualified_member_target(
        &parts, &anchor, &named, kind,
    )))
}

/// The outcome of looking a member up through a type: the stored occurrence
/// of the member when the impl headers prove one, and the record of what was
/// looked at.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TypeMemberLookupValue<'a> {
    pub(super) member_target_occurrence: Option<&'a Value>,
    pub(super) member_lookup_evidence: Value,
}

fn find_named_impl_member<'a>(header: &'a Value, name: &str) -> Option<&'a Value> {
    header["impl_member_occurrence_rows"]
        .as_array()?
        .iter()
        .find(|member| {
            member["member_declared_name_value"]
                .as_str()
                .map(|declared| declared.trim_start_matches("r#"))
                == Some(name.trim_start_matches("r#"))
        })
}

impl ImplMemberLookupIndex {
    /// The stored self type of the impl whose member list holds `owner_key`.
    pub(super) fn read_owner_self_type(&self, owner_key: &str) -> Option<&Value> {
        self.impl_header_value_rows
            .iter()
            .find(|header| {
                header["impl_member_occurrence_rows"]
                    .as_array()
                    .is_some_and(|members| {
                        members
                            .iter()
                            .any(|member| member["member_source_occurrence_key"] == owner_key)
                    })
            })
            .map(|header| &header["impl_self_type_resolution"])
    }

    /// The member `name` of the impl of the trait `trait_type` for the type `anchor_type`
    /// (both stored type resolutions) (REQ-RUST-052.0 G14).
    ///
    /// The candidates are those of `find_type_member_target`, less the impls the path
    /// cannot name: an inherent impl, and an impl of another stored trait. An impl whose
    /// trait has no stored target may be an impl of this trait and stays a candidate. No
    /// member is given when the trait the path names has no stored target.
    pub(super) fn find_qualified_member_target(
        &self,
        parts: &QualifiedPathPartsValue<'_>,
        anchor_type: &Value,
        trait_type: &Value,
        kind: &str,
    ) -> TypeMemberLookupValue<'_> {
        let stored_trait = trait_type["resolution_state_kind"] == "resolved";
        let found = self.find_admitted_member_target(
            parts.anchor_type_source_text,
            anchor_type,
            parts.member_declared_name_value,
            kind,
            |header| {
                let stored = &header["impl_trait_type_resolution"];
                !stored.is_null()
                    && (!stored_trait
                        || stored["resolution_state_kind"] != "resolved"
                        || stored["target_identity"] == trait_type["target_identity"])
            },
        );
        let anchor_stored = found.is_some();
        let (member, mut evidence) = match found {
            Some(found) => (
                found.member_target_occurrence.filter(|_| stored_trait),
                found.member_lookup_evidence,
            ),
            None => (
                None,
                json!({
                    "self_type_source_text":parts.anchor_type_source_text,
                    "anchor_type_resolution":anchor_type
                }),
            ),
        };
        evidence["qualified_trait_source_text"] = json!(parts.trait_path_source_text);
        evidence["qualified_trait_resolution"] = trait_type.clone();
        evidence["qualified_path_outcome_kind"] =
            json!(match (anchor_stored, stored_trait, member.is_some()) {
                (false, _, _) => "anchor_type_not_stored",
                (true, false, _) => "qualified_trait_not_stored",
                (true, true, true) => "impl_member_of_named_trait",
                (true, true, false) => "impl_member_not_proven",
            });
        TypeMemberLookupValue {
            member_target_occurrence: member,
            member_lookup_evidence: evidence,
        }
    }

    /// What the stored impl headers say about the member `name` of syntax kind
    /// `kind` reached through the type `self_type` (a stored self type
    /// resolution). `None` when the type itself has no stored target.
    ///
    /// The candidates are the members named `name` in the stored impls of that
    /// type and in the stored impls whose self type has no stored target (a
    /// generic or blanket impl may be an impl of this type). The member is the
    /// target only when there is exactly one candidate and it is in a trait
    /// impl of the type with a stored trait, no generic parameter and no
    /// negation, and has the kind asked for. The record lists every candidate
    /// either way, and what was not read.
    pub(super) fn find_type_member_target(
        &self,
        prefix_text: &str,
        self_type: &Value,
        name: &str,
        kind: &str,
    ) -> Option<TypeMemberLookupValue<'_>> {
        self.find_admitted_member_target(prefix_text, self_type, name, kind, |_| true)
    }

    /// `find_type_member_target` over the stored impls `admit` lets through.
    fn find_admitted_member_target(
        &self,
        prefix_text: &str,
        self_type: &Value,
        name: &str,
        kind: &str,
        admit: impl Fn(&Value) -> bool,
    ) -> Option<TypeMemberLookupValue<'_>> {
        if self_type["resolution_state_kind"] != "resolved" {
            return None;
        }
        let identity = &self_type["target_identity"];
        let same = |header: &Value| {
            let stored = &header["impl_self_type_resolution"];
            stored["resolution_state_kind"] == "resolved" && stored["target_identity"] == *identity
        };
        let trait_text = |header: &Value| {
            header["impl_trait_type_resolution"]["type_source_evidence"]["type_source_text"].clone()
        };
        let candidates = self
            .impl_header_value_rows
            .iter()
            .filter(|header| {
                same(header)
                    || header["impl_self_type_resolution"]["resolution_state_kind"] != "resolved"
            })
            .filter(|header| admit(header))
            .filter_map(|header| {
                find_named_impl_member(header, name).map(|member| (header, member))
            })
            .collect::<Vec<_>>();
        let proven = match candidates.as_slice() {
            [(header, member)] => (same(header)
                && header["impl_trait_type_resolution"]["resolution_state_kind"] == "resolved"
                && header["impl_generic_parameters_flag"] == false
                && header["impl_negative_implementation_flag"] == false
                && member["member_syntax_kind_value"] == kind)
                .then_some((*header, *member)),
            _ => None,
        };
        let builtin = &identity["target_builtin_name_value"];
        let unread = match builtin.as_str() {
            Some(name) => vec![
                format!("inherent items of builtin type {name}"),
                format!("trait impls for {name} in crates whose source was not read"),
            ],
            None => vec!["blanket trait impls in crates whose source was not read".to_owned()],
        };
        let mut evidence = json!({
            "self_type_source_text":prefix_text,
            "self_type_builtin_name":builtin,
            "self_type_identity":identity,
            "member_lookup_outcome_kind":match (candidates.len(), proven.is_some()) {
                (0, _) => "no_stored_impl_member",
                (1, true) => "single_trait_impl_member",
                (1, false) => "single_member_unproven",
                _ => "several_stored_impl_members",
            },
            "member_candidate_impl_rows":candidates.iter().map(|(header, member)| json!({
                "impl_header_occurrence":header["impl_semantic_occurrence_key"],
                "impl_trait_source_text":trait_text(header),
                "impl_self_type_stored_flag":same(header),
                "impl_trait_stored_flag":header["impl_trait_type_resolution"]["resolution_state_kind"] == "resolved",
                "impl_generic_parameters_flag":header["impl_generic_parameters_flag"],
                "impl_negative_implementation_flag":header["impl_negative_implementation_flag"],
                "member_syntax_kind_value":member["member_syntax_kind_value"]
            })).collect::<Vec<_>>(),
            "sources_not_read":unread
        });
        if let Some((header, _)) = proven {
            evidence["impl_trait_source_text"] = trait_text(header);
            evidence["impl_header_occurrence"] = header["impl_semantic_occurrence_key"].clone();
        }
        Some(TypeMemberLookupValue {
            member_target_occurrence: proven
                .map(|(_, member)| &member["member_semantic_occurrence_key"]),
            member_lookup_evidence: evidence,
        })
    }
}

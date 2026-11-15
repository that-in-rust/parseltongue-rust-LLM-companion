use crate::impl_header_source_records::{ImplHeaderSourceError, read_impl_stage_values};
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

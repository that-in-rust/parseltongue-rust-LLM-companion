use crate::namespace_binding_normalization_rules::check_binding_scope_visible;
use parseltongue_core_row_types::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub(crate) enum ResolvedPathLookupKind {
    Missing,
    Ambiguous(Vec<NameResolvedBindingRow>),
    Resolved(Vec<NameResolvedBindingRow>),
}

pub(crate) struct ScopeBindingLookupState<'a> {
    pub(crate) lookup_input_record_rows: &'a NameResolutionInputRows,
    lookup_scope_record_rows: BTreeMap<&'a NameResolutionScopeKey, &'a NameScopeRecordRow>,
    lookup_symbol_binding_rows:
        BTreeMap<&'a NameResolutionScopeKey, BTreeMap<&'a str, Vec<&'a NameResolvedBindingRow>>>,
}

impl<'a> ScopeBindingLookupState<'a> {
    pub(crate) fn build_namespace_lookup_rows(
        input: &'a NameResolutionInputRows,
        bindings: &'a [NameResolvedBindingRow],
    ) -> Self {
        Self {
            lookup_input_record_rows: input,
            lookup_scope_record_rows: input
                .name_module_scope_rows
                .iter()
                .map(|row| (&row.scope_module_identity_key, row))
                .collect(),
            lookup_symbol_binding_rows: bindings.iter().fold(BTreeMap::new(), |mut scopes, row| {
                scopes
                    .entry(&row.binding_lexical_scope_key)
                    .or_insert_with(BTreeMap::new)
                    .entry(row.binding_resolved_name_text.as_str())
                    .or_insert_with(Vec::new)
                    .push(row);
                scopes
            }),
        }
    }

    fn read_scope_parent_chain<'b>(
        &'b self,
        scope: &'b NameResolutionScopeKey,
    ) -> impl Iterator<Item = &'b NameResolutionScopeKey> {
        std::iter::successors(Some(scope), |key| {
            self.lookup_scope_record_rows
                .get(key)?
                .scope_parent_identity_key
                .as_ref()
        })
    }

    fn read_nearest_module_scope(
        &self,
        scope: &NameResolutionScopeKey,
    ) -> Option<NameResolutionScopeKey> {
        self.read_scope_parent_chain(scope)
            .find(|key| {
                self.lookup_scope_record_rows
                    .get(key)
                    .is_some_and(|row| row.scope_module_boundary_flag)
            })
            .cloned()
    }

    fn read_unit_root_scope(
        &self,
        unit: &CrateCompilationUnitKey,
    ) -> Option<NameResolutionScopeKey> {
        self.lookup_scope_record_rows
            .values()
            .find(|row| {
                row.scope_module_boundary_flag
                    && row.scope_parent_identity_key.is_none()
                    && &row.scope_module_identity_key.scope_compilation_unit_key == unit
            })
            .map(|row| row.scope_module_identity_key.clone())
    }

    fn read_direct_namespace_bindings(
        &self,
        scope: &NameResolutionScopeKey,
        name: &str,
        namespace: NamespaceBindingKindValue,
        requester: &NameResolutionScopeKey,
    ) -> Vec<NameResolvedBindingRow> {
        self.lookup_symbol_binding_rows
            .get(scope)
            .and_then(|names| names.get(name))
            .into_iter()
            .flatten()
            .filter(|row| {
                row.binding_namespace_kind_value == namespace
                    && check_binding_scope_visible(&row.binding_visibility_scope_kind, requester)
            })
            .map(|row| (*row).clone())
            .collect()
    }

    fn read_unqualified_namespace_bindings(
        &self,
        scope: &NameResolutionScopeKey,
        name: &str,
        namespace: NamespaceBindingKindValue,
        requester: &NameResolutionScopeKey,
    ) -> Vec<NameResolvedBindingRow> {
        let chain: Vec<_> = std::iter::successors(Some(scope), |key| {
            let row = self.lookup_scope_record_rows.get(key)?;
            row.scope_parent_lookup_flag
                .then_some(row.scope_parent_identity_key.as_ref())
                .flatten()
        })
        .collect();
        let local = chain
            .iter()
            .map(|key| self.read_direct_namespace_bindings(key, name, namespace, requester))
            .find(|rows| !rows.is_empty());
        let external = (self.lookup_input_record_rows.unit_rust_edition_kind
            != RustSourceEditionKind::Edition2015)
            .then_some(
                self.lookup_input_record_rows
                    .unit_extern_prelude_key
                    .as_ref(),
            )
            .flatten()
            .map(|key| self.read_direct_namespace_bindings(key, name, namespace, requester))
            .filter(|rows| !rows.is_empty());
        local
            .or(external)
            .or_else(|| {
                chain
                    .iter()
                    .filter_map(|key| {
                        self.lookup_scope_record_rows
                            .get(key)?
                            .scope_implicit_prelude_key
                            .as_ref()
                    })
                    .map(|key| self.read_direct_namespace_bindings(key, name, namespace, requester))
                    .find(|rows| !rows.is_empty())
            })
            .unwrap_or_default()
    }

    fn read_filtered_scope_bindings(
        &self,
        scope: &NameResolutionScopeKey,
        name: &str,
        filter: NameNamespaceFilterKind,
        requester: &NameResolutionScopeKey,
        unqualified: bool,
    ) -> ResolvedPathLookupKind {
        let rows: Vec<_> = [
            NamespaceBindingKindValue::Type,
            NamespaceBindingKindValue::Value,
            NamespaceBindingKindValue::Macro,
        ]
        .into_iter()
        .flat_map(|namespace| {
            if unqualified {
                self.read_unqualified_namespace_bindings(scope, name, namespace, requester)
            } else {
                self.read_direct_namespace_bindings(scope, name, namespace, requester)
            }
        })
        .collect();
        classify_path_binding_rows(filter_namespace_binding_rows(rows, filter))
    }

    pub(crate) fn resolve_scope_name_path(
        &self,
        requester: &NameResolutionScopeKey,
        segments: &[NamePathSegmentKind],
        filter: NameNamespaceFilterKind,
        kind: NameResolutionPathKind,
    ) -> ResolvedPathLookupKind {
        if segments.is_empty() {
            return ResolvedPathLookupKind::Missing;
        }
        let prefix_len = segments
            .iter()
            .take_while(|segment| !matches!(segment, NamePathSegmentKind::Name(_)))
            .count();
        let initial = if kind == NameResolutionPathKind::Import
            && self.lookup_input_record_rows.unit_rust_edition_kind
                == RustSourceEditionKind::Edition2015
        {
            self.read_unit_root_scope(&requester.scope_compilation_unit_key)
        } else {
            Some(requester.clone())
        };
        let start = segments[..prefix_len]
            .iter()
            .try_fold(initial, |scope, segment| {
                Some(match segment {
                    NamePathSegmentKind::CrateRoot => {
                        self.read_unit_root_scope(&requester.scope_compilation_unit_key)
                    }
                    NamePathSegmentKind::DefiningCrate(unit) => self.read_unit_root_scope(unit),
                    NamePathSegmentKind::SelfModule => self.read_nearest_module_scope(requester),
                    NamePathSegmentKind::ParentModule => {
                        let module = self.read_nearest_module_scope(scope.as_ref()?)?;
                        self.lookup_scope_record_rows
                            .get(&module)?
                            .scope_parent_identity_key
                            .as_ref()
                            .and_then(|parent| self.read_nearest_module_scope(parent))
                    }
                    NamePathSegmentKind::GlobalRoot => {
                        if self.lookup_input_record_rows.unit_rust_edition_kind
                            == RustSourceEditionKind::Edition2015
                        {
                            self.read_unit_root_scope(&requester.scope_compilation_unit_key)
                        } else {
                            self.lookup_input_record_rows
                                .unit_extern_prelude_key
                                .clone()
                        }
                    }
                    NamePathSegmentKind::Name(_) => None,
                })
            })
            .flatten();
        let Some(start) = start else {
            return ResolvedPathLookupKind::Missing;
        };
        let names = &segments[prefix_len..];
        let Some(NamePathSegmentKind::Name(first)) = names.first() else {
            return classify_path_binding_rows(filter_namespace_binding_rows(
                vec![NameResolvedBindingRow {
                    binding_lexical_scope_key: start.clone(),
                    binding_resolved_name_text: "crate".into(),
                    binding_namespace_kind_value: NamespaceBindingKindValue::Type,
                    binding_target_identity_key: NameSymbolTargetKey::CrateRoot(
                        start.scope_compilation_unit_key.clone(),
                    ),
                    binding_target_child_scope: Some(start),
                    binding_visibility_scope_kind: NameBindingVisibilityKind::Public,
                    binding_selection_priority_kind: NameBindingPriorityKind::Declaration,
                    binding_import_evidence_keys: Vec::new(),
                }],
                filter,
            ));
        };
        let initial = self.read_filtered_scope_bindings(
            &start,
            first,
            if names.len() == 1 {
                filter
            } else {
                NameNamespaceFilterKind::Type
            },
            requester,
            prefix_len == 0,
        );
        names[1..]
            .iter()
            .enumerate()
            .fold(initial, |state, (index, segment)| {
                let ResolvedPathLookupKind::Resolved(rows) = state else {
                    return state;
                };
                let Some(child) = rows
                    .first()
                    .and_then(|row| row.binding_target_child_scope.as_ref())
                else {
                    return ResolvedPathLookupKind::Missing;
                };
                let NamePathSegmentKind::Name(name) = segment else {
                    return ResolvedPathLookupKind::Missing;
                };
                self.read_filtered_scope_bindings(
                    child,
                    name,
                    if index + 2 == names.len() {
                        filter
                    } else {
                        NameNamespaceFilterKind::Type
                    },
                    requester,
                    false,
                )
            })
    }
}

fn filter_namespace_binding_rows(
    rows: Vec<NameResolvedBindingRow>,
    filter: NameNamespaceFilterKind,
) -> Vec<NameResolvedBindingRow> {
    let has_type = rows
        .iter()
        .any(|row| row.binding_namespace_kind_value == NamespaceBindingKindValue::Type);
    rows.into_iter()
        .filter(|row| match filter {
            NameNamespaceFilterKind::Any => true,
            NameNamespaceFilterKind::Type => {
                row.binding_namespace_kind_value == NamespaceBindingKindValue::Type
            }
            NameNamespaceFilterKind::Value => {
                row.binding_namespace_kind_value == NamespaceBindingKindValue::Value
            }
            NameNamespaceFilterKind::Macro => {
                row.binding_namespace_kind_value == NamespaceBindingKindValue::Macro
            }
            NameNamespaceFilterKind::TypeOrValue => {
                row.binding_namespace_kind_value
                    == if has_type {
                        NamespaceBindingKindValue::Type
                    } else {
                        NamespaceBindingKindValue::Value
                    }
            }
        })
        .collect()
}

fn classify_path_binding_rows(rows: Vec<NameResolvedBindingRow>) -> ResolvedPathLookupKind {
    if rows.is_empty() {
        return ResolvedPathLookupKind::Missing;
    }
    let targets = rows
        .iter()
        .fold(BTreeMap::<_, BTreeSet<_>>::new(), |mut groups, row| {
            groups
                .entry(row.binding_namespace_kind_value)
                .or_default()
                .insert(&row.binding_target_identity_key);
            groups
        });
    if targets.values().any(|targets| targets.len() > 1) {
        ResolvedPathLookupKind::Ambiguous(rows)
    } else {
        ResolvedPathLookupKind::Resolved(rows)
    }
}

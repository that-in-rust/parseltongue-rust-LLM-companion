use parseltongue_core_row_types::{
    NameBindingVisibilityKind, NameResolutionScopeKey, NameResolvedBindingRow,
    NamespaceBindingKindValue,
};
use std::collections::{BTreeMap, BTreeSet};

type NameBindingCandidateGroups = BTreeMap<
    (NameResolutionScopeKey, String, NamespaceBindingKindValue),
    Vec<NameResolvedBindingRow>,
>;

pub(crate) fn check_scope_contains_scope(
    outer: &NameResolutionScopeKey,
    inner: &NameResolutionScopeKey,
) -> bool {
    outer.scope_compilation_unit_key == inner.scope_compilation_unit_key
        && inner
            .scope_semantic_path_parts
            .starts_with(&outer.scope_semantic_path_parts)
}

pub(crate) fn check_binding_scope_visible(
    visibility: &NameBindingVisibilityKind,
    scope: &NameResolutionScopeKey,
) -> bool {
    match visibility {
        NameBindingVisibilityKind::Public => true,
        NameBindingVisibilityKind::Restricted(outer) => check_scope_contains_scope(outer, scope),
        NameBindingVisibilityKind::Invisible => false,
    }
}

pub(crate) fn intersect_binding_visibility_scopes(
    left: &NameBindingVisibilityKind,
    right: &NameBindingVisibilityKind,
) -> NameBindingVisibilityKind {
    use NameBindingVisibilityKind::{Invisible, Public, Restricted};
    match (left, right) {
        (Invisible, _) | (_, Invisible) => Invisible,
        (Public, other) | (other, Public) => other.clone(),
        (Restricted(left), Restricted(right)) if check_scope_contains_scope(left, right) => {
            Restricted(right.clone())
        }
        (Restricted(left), Restricted(right)) if check_scope_contains_scope(right, left) => {
            Restricted(left.clone())
        }
        _ => Invisible,
    }
}

fn union_binding_visibility_scopes(
    left: &NameBindingVisibilityKind,
    right: &NameBindingVisibilityKind,
) -> NameBindingVisibilityKind {
    use NameBindingVisibilityKind::{Invisible, Public, Restricted};
    match (left, right) {
        (Public, _) | (_, Public) => Public,
        (Invisible, other) | (other, Invisible) => other.clone(),
        (Restricted(left), Restricted(right)) if check_scope_contains_scope(left, right) => {
            Restricted(left.clone())
        }
        (Restricted(left), Restricted(right)) if check_scope_contains_scope(right, left) => {
            Restricted(right.clone())
        }
        _ => Invisible,
    }
}

pub(crate) fn normalize_scope_binding_rows(
    bindings: impl IntoIterator<Item = NameResolvedBindingRow>,
) -> Vec<NameResolvedBindingRow> {
    bindings
        .into_iter()
        .fold(NameBindingCandidateGroups::new(), |mut groups, binding| {
            groups
                .entry((
                    binding.binding_lexical_scope_key.clone(),
                    binding.binding_resolved_name_text.clone(),
                    binding.binding_namespace_kind_value,
                ))
                .or_default()
                .push(binding);
            groups
        })
        .into_values()
        .flat_map(|bindings| {
            let highest = bindings
                .iter()
                .map(|binding| binding.binding_selection_priority_kind)
                .max();
            bindings
                .into_iter()
                .filter(|binding| Some(binding.binding_selection_priority_kind) == highest)
                .fold(BTreeMap::new(), |mut targets, binding| {
                    let key = (
                        binding.binding_target_identity_key.clone(),
                        binding.binding_target_child_scope.clone(),
                    );
                    targets
                        .entry(key)
                        .and_modify(|existing: &mut NameResolvedBindingRow| {
                            existing.binding_visibility_scope_kind =
                                union_binding_visibility_scopes(
                                    &existing.binding_visibility_scope_kind,
                                    &binding.binding_visibility_scope_kind,
                                );
                            existing.binding_import_evidence_keys = existing
                                .binding_import_evidence_keys
                                .iter()
                                .cloned()
                                .chain(binding.binding_import_evidence_keys.iter().cloned())
                                .collect::<BTreeSet<_>>()
                                .into_iter()
                                .collect();
                        })
                        .or_insert_with(|| {
                            let mut binding = binding;
                            binding.binding_import_evidence_keys = binding
                                .binding_import_evidence_keys
                                .into_iter()
                                .collect::<BTreeSet<_>>()
                                .into_iter()
                                .collect();
                            binding
                        });
                    targets
                })
                .into_values()
        })
        .collect()
}

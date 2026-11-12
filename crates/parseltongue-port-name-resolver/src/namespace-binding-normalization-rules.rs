use parseltongue_core_row_types::{
    NameBindingVisibilityKind, NameResolutionScopeKey, NameResolvedBindingRow,
    NamespaceBindingKindValue,
};
use std::collections::BTreeSet;

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
    let mut bindings = bindings.into_iter().collect::<Vec<_>>();
    bindings.sort_by(|left, right| {
        read_binding_scope_identity(left).cmp(&read_binding_scope_identity(right))
    });
    let (mut output, group) = bindings.into_iter().fold(
        (Vec::new(), Vec::new()),
        |(mut output, mut group), binding| {
            if group.last().is_some_and(|last| {
                read_binding_scope_identity(last) != read_binding_scope_identity(&binding)
            }) {
                output.extend(select_scope_binding_rows(std::mem::take(&mut group)));
            }
            group.push(binding);
            (output, group)
        },
    );
    output.extend(select_scope_binding_rows(group));
    output
}

fn read_binding_scope_identity(
    binding: &NameResolvedBindingRow,
) -> (&NameResolutionScopeKey, &str, NamespaceBindingKindValue) {
    (
        &binding.binding_lexical_scope_key,
        &binding.binding_resolved_name_text,
        binding.binding_namespace_kind_value,
    )
}

fn select_scope_binding_rows(
    mut bindings: Vec<NameResolvedBindingRow>,
) -> Vec<NameResolvedBindingRow> {
    let highest = bindings
        .iter()
        .map(|binding| binding.binding_selection_priority_kind)
        .max();
    bindings.retain(|binding| Some(binding.binding_selection_priority_kind) == highest);
    bindings.sort_by(|left, right| {
        (
            &left.binding_target_identity_key,
            &left.binding_target_child_scope,
        )
            .cmp(&(
                &right.binding_target_identity_key,
                &right.binding_target_child_scope,
            ))
    });
    bindings
        .into_iter()
        .fold(Vec::new(), |mut targets, mut binding| {
            if let Some(existing) =
                targets
                    .last_mut()
                    .filter(|existing: &&mut NameResolvedBindingRow| {
                        existing.binding_target_identity_key == binding.binding_target_identity_key
                            && existing.binding_target_child_scope
                                == binding.binding_target_child_scope
                    })
            {
                existing.binding_visibility_scope_kind = union_binding_visibility_scopes(
                    &existing.binding_visibility_scope_kind,
                    &binding.binding_visibility_scope_kind,
                );
                existing.binding_import_evidence_keys = existing
                    .binding_import_evidence_keys
                    .iter()
                    .cloned()
                    .chain(binding.binding_import_evidence_keys)
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
            } else {
                binding.binding_import_evidence_keys = binding
                    .binding_import_evidence_keys
                    .into_iter()
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
                targets.push(binding);
            }
            targets
        })
}

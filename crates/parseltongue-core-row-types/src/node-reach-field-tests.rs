#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use HldEffectiveVisibilityKind::{
    NotApplicable, PrivateEquivalent, Public, RestrictedNonPrivate, Unknown,
};
use NodeReachOwnerValue::{
    Container, FileArea, InherentMember, MacroCall, TraitImplMember, TraitMember,
};
use NodeReachParentValue::{CrateRoot, Node, Outside, Unresolved};

type Row = (
    &'static str,
    HldEffectiveVisibilityKind,
    NodeReachOwnerValue<&'static str>,
);

fn derive_fixture_reach_fields(
    rows: &[Row],
    reexport: bool,
) -> BTreeMap<&'static str, (HldEffectiveVisibilityKind, Option<bool>)> {
    let nodes = rows
        .iter()
        .cloned()
        .map(|(key, declared, owner)| {
            (
                key,
                NodeReachInputRecord {
                    node_declared_visibility_kind: declared,
                    node_reach_owner_value: owner,
                    node_reexport_possible_flag: reexport,
                },
            )
        })
        .collect();
    derive_node_reach_fields(&nodes)
        .into_iter()
        .map(|(key, fields)| {
            (
                key,
                (
                    fields.node_effective_visibility_kind,
                    fields.node_reachable_outside_crate_flag,
                ),
            )
        })
        .collect()
}

/// The shapes of pinned itoa, and one row per rule that itoa does not reach.
fn make_reach_fixture_rows() -> Vec<Row> {
    let impl_of = |trait_type, self_type| TraitImplMember {
        trait_type,
        self_type,
    };
    vec![
        ("area", PrivateEquivalent, FileArea { module: CrateRoot }),
        ("call", PrivateEquivalent, MacroCall),
        ("Buffer", Public, Container(Node("area"))),
        (
            "Buffer::new",
            Public,
            InherentMember {
                self_type: Node("Buffer"),
            },
        ),
        (
            "Buffer::hidden",
            PrivateEquivalent,
            InherentMember {
                self_type: Node("Buffer"),
            },
        ),
        (
            "Default::default",
            PrivateEquivalent,
            impl_of(Outside, Node("Buffer")),
        ),
        ("Integer", Public, Container(Node("area"))),
        (
            "Integer::MAX",
            PrivateEquivalent,
            TraitMember(Node("Integer")),
        ),
        (
            "u8::MAX",
            PrivateEquivalent,
            impl_of(Node("Integer"), Outside),
        ),
        ("private", PrivateEquivalent, Container(Node("area"))),
        ("Sealed", Public, Container(Node("private"))),
        (
            "Sealed::write",
            PrivateEquivalent,
            TraitMember(Node("Sealed")),
        ),
        (
            "u8::write",
            PrivateEquivalent,
            impl_of(Node("Sealed"), Outside),
        ),
        ("Unsigned", PrivateEquivalent, Container(Node("area"))),
        (
            "u8::fmt",
            PrivateEquivalent,
            impl_of(Node("Unsigned"), Outside),
        ),
        ("ext", PrivateEquivalent, Container(Node("area"))),
        (
            "ext_area",
            PrivateEquivalent,
            FileArea {
                module: Node("ext"),
            },
        ),
        ("mulhi", RestrictedNonPrivate, Container(Node("ext_area"))),
        ("ext_open", Public, Container(Node("ext_area"))),
        ("open", Public, Container(Node("area"))),
        (
            "open_area",
            PrivateEquivalent,
            FileArea {
                module: Node("open"),
            },
        ),
        ("open_item", Public, Container(Node("open_area"))),
        (
            "lost::member",
            PrivateEquivalent,
            impl_of(Unresolved, Node("Buffer")),
        ),
        (
            "lost::self",
            Public,
            InherentMember {
                self_type: Unresolved,
            },
        ),
        (
            "half",
            PrivateEquivalent,
            impl_of(Node("Integer"), Unresolved),
        ),
        (
            "half_hidden",
            PrivateEquivalent,
            impl_of(Node("Sealed"), Unresolved),
        ),
        ("exported", Unknown, Container(Node("area"))),
        ("absent", Public, Container(Node("no such node"))),
    ]
}

/// REQ-RUST-069.0 V2-V7, R1-R4: one row per rule. The three file areas get no fields (V6).
#[test]
fn test_node_reach_field_rules() {
    let fields = derive_fixture_reach_fields(&make_reach_fixture_rows(), false);
    let expected = [
        ("call", NotApplicable, Some(false)),               // V5, R1
        ("Buffer", Public, Some(true)),                     // V4, R2, R3: the crate root
        ("Buffer::new", Public, Some(true)),                // V4, R2: the self type
        ("Buffer::hidden", PrivateEquivalent, Some(false)), // V4, R1
        ("Default::default", Public, Some(true)),           // V3, R3: a trait outside
        ("Integer", Public, Some(true)),
        ("Integer::MAX", Public, Some(true)), // V2
        ("u8::MAX", Public, Some(true)),      // V3, R3: a builtin self type
        ("private", PrivateEquivalent, Some(false)),
        ("Sealed", Public, Some(false)), // R2: a private inline module
        ("Sealed::write", Public, Some(false)), // V2, R2
        ("u8::write", Public, Some(false)), // V3, R2
        ("Unsigned", PrivateEquivalent, Some(false)),
        ("u8::fmt", PrivateEquivalent, Some(false)), // V3: a private trait
        ("ext", PrivateEquivalent, Some(false)),
        ("mulhi", RestrictedNonPrivate, Some(false)), // V1, R1
        ("ext_open", Public, Some(false)),            // R2: the module of the file
        ("open", Public, Some(true)),
        ("open_item", Public, Some(true)), // R2: through a file area
        ("lost::member", Unknown, None),   // V3: the trait is not resolved
        ("lost::self", Public, None),      // R4
        ("half", Public, None),            // R4: one parent of two
        ("half_hidden", Public, Some(false)), // one parent is not reachable
        ("exported", Unknown, None),       // V7
        ("absent", Public, None),          // R4: the parent is no node
    ];
    assert_eq!(expected.len(), fields.len());
    for (key, visibility, reachable) in expected {
        assert_eq!((visibility, reachable), fields[key], "{key}");
    }
}

/// REQ-RUST-069.0 R5: with a `use` that is not private, a public node under a parent that
/// is not reachable has no value; every other node is unchanged.
#[test]
fn test_reexport_reach_flag_unknown() {
    let rows = make_reach_fixture_rows();
    let (closed, open) = (
        derive_fixture_reach_fields(&rows, false),
        derive_fixture_reach_fields(&rows, true),
    );
    let changed = closed
        .iter()
        .filter(|(key, fields)| open[*key] != **fields)
        .map(|(key, _)| *key)
        .collect::<Vec<_>>();
    assert_eq!(
        vec![
            "Sealed",
            "Sealed::write",
            "ext_open",
            "half_hidden",
            "u8::write"
        ],
        changed
    );
    assert!(changed.iter().all(|key| open[key] == (Public, None)));
}

/// A parent cycle cannot be written in source; the function still ends, with no value.
#[test]
fn test_parent_cycle_reach_ends() {
    let rows = [
        ("first", Public, Container(Node("second"))),
        ("second", Public, Container(Node("first"))),
        ("looped", Public, TraitMember(Node("looped"))),
    ];
    let fields = derive_fixture_reach_fields(&rows, false);
    assert_eq!((Public, None), fields["first"]);
    assert_eq!((Public, None), fields["second"]);
    assert_eq!((Unknown, None), fields["looped"]);
}

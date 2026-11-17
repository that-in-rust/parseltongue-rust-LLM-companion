//! REQ-RUST-056.0: the shape of a written type, and Rust's method probe over the member
//! declarations of `core`. Pure functions over text and rows: no store, no file system.
//! A target is named only when exactly one declaration is proven to fit; anything the
//! rows cannot decide withholds the name. A missed target is preferable to a wrong one.

#![allow(dead_code)]
use parseltongue_core_row_types::is_hld_builtin_type;
use ra_ap_syntax::{
    AstNode, Edition, SourceFile, SyntaxKind,
    ast::{self, HasGenericArgs},
};
use serde_json::Value;
use std::collections::BTreeSet;
use std::ops::ControlFlow;

#[cfg(test)]
#[path = "receiver-type-probe-tests.rs"]
mod receiver_type_probe_tests;

/// A type as its source writes it. Paths keep their last segment only: whether that name
/// is the `core` type of the same name is the caller's proof, not this module's.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WrittenTypeShapeValue {
    Reference {
        mutable: bool,
        inner: Box<WrittenTypeShapeValue>,
    },
    Pointer {
        mutable: bool,
        inner: Box<WrittenTypeShapeValue>,
    },
    Array(Box<WrittenTypeShapeValue>),
    Slice(Box<WrittenTypeShapeValue>),
    Named {
        name: String,
        arguments: Vec<WrittenTypeShapeValue>,
    },
    /// A generic parameter of the impl that declares a member: it fits any type.
    Wildcard,
    /// A type this model does not read: it is never proven to fit or to differ.
    Opaque,
    /// REQ-RUST-062.0 C4, C5: a type that is not named and is proven `Sized`: it fits a
    /// generic parameter of an impl and nothing else.
    SizedUnknown,
}

/// A trait in scope at the call that declares the method name (P4, P5).
#[derive(Clone, Debug)]
pub(crate) struct ScopeTraitMethodValue {
    /// `value`, `reference` or `mutable_reference`: the method's `self` parameter.
    pub(crate) member_self_parameter_kind: String,
    /// The self type of every impl of the trait, or `None` when not all can be read.
    pub(crate) trait_impl_self_shapes: Option<Vec<WrittenTypeShapeValue>>,
    /// REQ-RUST-060.0 U2: the trait's name, when it is a trait of `core` that is in
    /// scope through the prelude only.
    pub(crate) prelude_trait_name_value: Option<String>,
}

/// What the probe decided for one call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CoreMemberProbeOutcome {
    /// Exactly one declaration fits at the first fitting step.
    Named {
        declaration_row_index: usize,
        probe_step_text: String,
        /// REQ-RUST-060.0 U3: the prelude traits assumed not to be implemented for the
        /// types of the earlier forms.
        prelude_trait_assumed_names: Vec<String>,
    },
    /// No name: the detail says what could not be proven.
    Withheld(&'static str),
}

/// What one receiver form decides for the probe.
enum ProbeFormDecisionValue {
    /// The probe ends with this outcome.
    Decided(CoreMemberProbeOutcome),
    /// REQ-RUST-060.0 U1: no inherent member fits, and only these prelude traits may.
    Assumed(Vec<String>),
    /// Nothing fits.
    Passed,
}

/// P1: the shape of the type `text` writes. `parameters` are the generic parameter names
/// of the impl that writes it; each is a wildcard.
pub(crate) fn parse_written_type_shape(text: &str, parameters: &[String]) -> WrittenTypeShapeValue {
    let parse = SourceFile::parse(&format!("type X = {text};"), Edition::Edition2021);
    if !parse.errors().is_empty() {
        return WrittenTypeShapeValue::Opaque;
    }
    parse
        .tree()
        .syntax()
        .descendants()
        .find_map(ast::TypeAlias::cast)
        .and_then(|alias| alias.ty())
        .map_or(WrittenTypeShapeValue::Opaque, |ty| {
            build_written_type_shape(&ty, parameters, &mut BTreeSet::new())
        })
}

/// REQ-RUST-062.0 C4: the shape of the return type `text` of a member of `core`.
/// `generics` are the generic parameter names of the member and of its trait or impl:
/// none of them names a type. A direct generic argument of the returned type that is one
/// of `sized`, or `Self::NAME` with `NAME` one of them, is a type proven `Sized`.
pub(crate) fn parse_return_type_shape(
    text: &str,
    sized: &[String],
    generics: &[String],
) -> WrittenTypeShapeValue {
    let parse = SourceFile::parse(&format!("type X = {text};"), Edition::Edition2021);
    let returned = parse
        .errors()
        .is_empty()
        .then(|| parse.tree())
        .and_then(|tree| tree.syntax().descendants().find_map(ast::TypeAlias::cast))
        .and_then(|alias| alias.ty());
    let Some(returned) = returned else {
        return WrittenTypeShapeValue::Opaque;
    };
    let written = match &returned {
        ast::Type::PathType(found) => found
            .path()
            .and_then(|path| path.segment())
            .and_then(|segment| segment.generic_arg_list())
            .into_iter()
            .flat_map(|list| list.generic_args())
            .filter(|argument| !matches!(argument, ast::GenericArg::LifetimeArg(_)))
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    let proven = |argument: &ast::GenericArg| {
        let ast::GenericArg::TypeArg(found) = argument else {
            return false;
        };
        let text = found
            .syntax()
            .text()
            .to_string()
            .split_whitespace()
            .collect::<String>();
        sized
            .iter()
            .any(|name| text == *name || text == format!("Self::{name}"))
    };
    match build_written_type_shape(&returned, generics, &mut BTreeSet::new()) {
        WrittenTypeShapeValue::Named { name, arguments } if arguments.len() == written.len() => {
            WrittenTypeShapeValue::Named {
                name,
                arguments: written
                    .iter()
                    .zip(arguments)
                    .map(|(argument, shape)| {
                        if proven(argument) {
                            WrittenTypeShapeValue::SizedUnknown
                        } else {
                            shape
                        }
                    })
                    .collect(),
            }
        }
        shape => shape,
    }
}

fn build_inner_type_shape(
    ty: Option<ast::Type>,
    parameters: &[String],
    seen: &mut BTreeSet<String>,
) -> Box<WrittenTypeShapeValue> {
    Box::new(ty.map_or(WrittenTypeShapeValue::Opaque, |ty| {
        build_written_type_shape(&ty, parameters, seen)
    }))
}

/// `seen` holds the parameters already met: a parameter written twice must name the
/// same type both times, which a wildcard cannot say, so its later uses are opaque.
fn build_written_type_shape(
    ty: &ast::Type,
    parameters: &[String],
    seen: &mut BTreeSet<String>,
) -> WrittenTypeShapeValue {
    match ty {
        ast::Type::ParenType(found) => *build_inner_type_shape(found.ty(), parameters, seen),
        ast::Type::RefType(found) => WrittenTypeShapeValue::Reference {
            mutable: found.mut_token().is_some(),
            inner: build_inner_type_shape(found.ty(), parameters, seen),
        },
        ast::Type::PtrType(found) => WrittenTypeShapeValue::Pointer {
            mutable: found.mut_token().is_some(),
            inner: build_inner_type_shape(found.ty(), parameters, seen),
        },
        ast::Type::ArrayType(found) => {
            WrittenTypeShapeValue::Array(build_inner_type_shape(found.ty(), parameters, seen))
        }
        ast::Type::SliceType(found) => {
            WrittenTypeShapeValue::Slice(build_inner_type_shape(found.ty(), parameters, seen))
        }
        ast::Type::PathType(found) => found
            .path()
            .and_then(|path| build_named_path_shape(&path, parameters, seen))
            .unwrap_or(WrittenTypeShapeValue::Opaque),
        _ => WrittenTypeShapeValue::Opaque,
    }
}

/// `None` for a path this model does not read: `Self`, a type anchor, a path through a
/// generic parameter, generic arguments before the last segment, `Fn(..)` sugar.
fn build_named_path_shape(
    path: &ast::Path,
    parameters: &[String],
    seen: &mut BTreeSet<String>,
) -> Option<WrittenTypeShapeValue> {
    let read_plain_name = |segment: &ast::PathSegment| {
        segment
            .name_ref()
            .filter(|name| name.text() != "Self")
            .map(|name| name.text().trim_start_matches("r#").to_owned())
    };
    let segment = path.segment()?;
    let name = read_plain_name(&segment)
        .filter(|name| !matches!(name.as_str(), "self" | "super" | "crate"))?;
    if segment.syntax().children().any(|child| {
        !matches!(
            child.kind(),
            SyntaxKind::NAME_REF | SyntaxKind::GENERIC_ARG_LIST
        )
    }) {
        return None;
    }
    let qualifiers = std::iter::successors(path.qualifier(), |found| found.qualifier())
        .map(|found| {
            found
                .segment()
                .filter(|segment| segment.generic_arg_list().is_none())
                .and_then(|segment| read_plain_name(&segment))
                .filter(|name| !parameters.contains(name))
        })
        .collect::<Option<Vec<_>>>()?;
    if qualifiers.is_empty() && segment.generic_arg_list().is_none() && parameters.contains(&name) {
        return Some(if seen.insert(name) {
            WrittenTypeShapeValue::Wildcard
        } else {
            WrittenTypeShapeValue::Opaque
        });
    }
    let arguments = segment
        .generic_arg_list()
        .into_iter()
        .flat_map(|list| list.generic_args())
        .filter_map(|argument| match argument {
            ast::GenericArg::TypeArg(found) => {
                Some(*build_inner_type_shape(found.ty(), parameters, seen))
            }
            ast::GenericArg::LifetimeArg(_) => None,
            _ => Some(WrittenTypeShapeValue::Opaque),
        })
        .collect();
    Some(WrittenTypeShapeValue::Named { name, arguments })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ShapeFitKind {
    Fits,
    Differs,
    Unknown,
}

fn merge_shape_fit_kinds(left: ShapeFitKind, right: ShapeFitKind) -> ShapeFitKind {
    match (left, right) {
        (ShapeFitKind::Differs, _) | (_, ShapeFitKind::Differs) => ShapeFitKind::Differs,
        (ShapeFitKind::Unknown, _) | (_, ShapeFitKind::Unknown) => ShapeFitKind::Unknown,
        _ => ShapeFitKind::Fits,
    }
}

/// P3: whether the type an impl writes can be the type `probed`. A wildcard stands for
/// a sized type, so against a slice, `str` or an unread type it is not decided.
fn unify_written_type_shapes(
    declared: &WrittenTypeShapeValue,
    probed: &WrittenTypeShapeValue,
) -> ShapeFitKind {
    use WrittenTypeShapeValue::{
        Array, Named, Opaque, Pointer, Reference, SizedUnknown, Slice, Wildcard,
    };
    match (declared, probed) {
        (Wildcard, Opaque | Slice(_) | Wildcard) => ShapeFitKind::Unknown,
        (Wildcard, Named { name, .. }) if name == "str" => ShapeFitKind::Unknown,
        (Wildcard, _) => ShapeFitKind::Fits,
        (Opaque | SizedUnknown, _) | (_, Opaque | Wildcard | SizedUnknown) => ShapeFitKind::Unknown,
        (
            Reference {
                mutable: left,
                inner: declared,
            },
            Reference {
                mutable: right,
                inner: probed,
            },
        )
        | (
            Pointer {
                mutable: left,
                inner: declared,
            },
            Pointer {
                mutable: right,
                inner: probed,
            },
        ) => {
            if left == right {
                unify_written_type_shapes(declared, probed)
            } else {
                ShapeFitKind::Differs
            }
        }
        (Array(declared), Array(probed)) | (Slice(declared), Slice(probed)) => {
            unify_written_type_shapes(declared, probed)
        }
        (
            Named {
                name: left,
                arguments: declared,
            },
            Named {
                name: right,
                arguments: probed,
            },
        ) => {
            if left != right {
                ShapeFitKind::Differs
            } else if declared.len() != probed.len() {
                ShapeFitKind::Unknown
            } else {
                declared
                    .iter()
                    .zip(probed)
                    .map(|(declared, probed)| unify_written_type_shapes(declared, probed))
                    .fold(ShapeFitKind::Fits, merge_shape_fit_kinds)
            }
        }
        _ => ShapeFitKind::Differs,
    }
}

/// The receiver type a method with this `self` parameter takes, for the self type `owner`.
fn unify_member_receiver_shape(
    kind: &str,
    owner: &WrittenTypeShapeValue,
    form: &WrittenTypeShapeValue,
) -> ShapeFitKind {
    let reference = |mutable| WrittenTypeShapeValue::Reference {
        mutable,
        inner: Box::new(owner.clone()),
    };
    match kind {
        "value" => unify_written_type_shapes(owner, form),
        "reference" => unify_written_type_shapes(&reference(false), form),
        "mutable_reference" => unify_written_type_shapes(&reference(true), form),
        _ => ShapeFitKind::Unknown,
    }
}

fn list_shape_type_names(shape: &WrittenTypeShapeValue) -> Vec<&str> {
    match shape {
        WrittenTypeShapeValue::Reference { inner, .. }
        | WrittenTypeShapeValue::Pointer { inner, .. }
        | WrittenTypeShapeValue::Array(inner)
        | WrittenTypeShapeValue::Slice(inner) => list_shape_type_names(inner),
        WrittenTypeShapeValue::Named { name, arguments } => std::iter::once(name.as_str())
            .chain(arguments.iter().flat_map(list_shape_type_names))
            .collect(),
        WrittenTypeShapeValue::Wildcard
        | WrittenTypeShapeValue::Opaque
        | WrittenTypeShapeValue::SizedUnknown => Vec::new(),
    }
}

/// A type name outside the builtin types stands for one type only when `core` gives
/// that name to exactly one struct, enum or union and to no alias or rename.
fn check_type_name_declared(name: &str, declarations: &[Value]) -> bool {
    is_hld_builtin_type(name)
        || matches!(
            declarations
                .iter()
                .filter(|row| {
                    row["declaration_owner_kind_value"] == "type_name_declaration"
                        && row["member_declared_name_value"] == name
                })
                .map(|row| row["type_declaration_kind_value"].as_str())
                .collect::<Vec<_>>()
                .as_slice(),
            [Some("struct" | "enum" | "union")]
        )
}

/// REQ-RUST-054.0 D12: whether a macro that takes its member's name from its caller may
/// be given `method_name`. It may not when the macro has one name, no `use` renames
/// it, it is called in `core`, and every call that may be of it writes other names and
/// passes no macro variable on.
fn check_macro_name_reach(row: &Value, method_name: &str, declarations: &[Value]) -> bool {
    let Some(name) = row["owner_macro_name_value"]
        .as_str()
        .filter(|_| row["declaration_owner_kind_value"] == "macro_definition_body")
    else {
        return true;
    };
    let renamed = declarations.iter().any(|found| {
        found["type_declaration_kind_value"] == "use_rename"
            && found["rename_source_name_value"]
                .as_str()
                .is_none_or(|source| source == name)
    });
    let call_rows = || {
        declarations
            .iter()
            .filter(|found| found["declaration_owner_kind_value"] == "macro_call_name_tokens")
    };
    // `$m!(..)` calls this macro only when a call somewhere passes this macro's name.
    let passed = call_rows().any(|call| {
        call["macro_call_identifier_values"]
            .as_array()
            .is_none_or(|names| names.iter().any(|found| found == name))
    });
    let calls = call_rows()
        .filter(|call| match call["owner_macro_name_value"].as_str() {
            Some(called) => called == name,
            None => passed,
        })
        .collect::<Vec<_>>();
    renamed
        || !calls
            .iter()
            .any(|call| call["owner_macro_name_value"] == name)
        || calls.iter().any(|call| {
            call["owner_macro_name_value"].is_null()
                || call["macro_call_metavariable_flag"] == true
                || call["macro_call_identifier_values"]
                    .as_array()
                    .is_none_or(|names| names.iter().any(|found| found == method_name))
        })
}

/// A `fn` written as tokens that may be an inherent method named `method_name`.
pub(crate) fn check_macro_member_candidate(
    row: &Value,
    method_name: &str,
    declarations: &[Value],
) -> bool {
    matches!(
        row["declaration_owner_kind_value"].as_str(),
        Some("macro_definition_body" | "macro_call_argument")
    ) && (row["member_declared_name_value"] == method_name
        || (row["member_name_parameter_flag"] == true
            && check_macro_name_reach(row, method_name, declarations)))
        && row["macro_self_parameter_flag"] == true
        && !matches!(
            row["macro_owner_shape_kind"].as_str(),
            Some("trait_impl" | "trait_declaration")
        )
}

/// An inherent method of the probed name: its row, `self` kind and impl self type.
struct InherentMemberCandidateValue<'a> {
    declaration_row_index: usize,
    member_self_parameter_kind: &'a str,
    impl_self_type_shape: WrittenTypeShapeValue,
    /// A bound on the impl or a `cfg` attribute: the member may not exist for the type.
    member_presence_unproven_flag: bool,
}

fn build_inherent_member_candidate(
    index: usize,
    row: &Value,
) -> Option<InherentMemberCandidateValue<'_>> {
    let kind = row["member_self_parameter_kind"]
        .as_str()
        .filter(|kind| *kind != "none")?;
    let parameters = row["impl_generic_parameter_names"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|name| name.as_str().map(str::to_owned))
        .collect::<Vec<_>>();
    let conditional = [
        "member_attribute_text_values",
        "owner_attribute_text_values",
    ]
    .iter()
    .flat_map(|key| row[*key].as_array().into_iter().flatten())
    .any(|text| text.as_str().is_some_and(|text| text.starts_with("#[cfg")));
    Some(InherentMemberCandidateValue {
        declaration_row_index: index,
        member_self_parameter_kind: kind,
        impl_self_type_shape: parse_written_type_shape(
            row["impl_self_type_text"].as_str().unwrap_or(""),
            &parameters,
        ),
        member_presence_unproven_flag: conditional || row["impl_bound_restriction_flag"] != false,
    })
}

/// P2: the receiver type, each type a reference in it points to, then the slice of a
/// final array.
fn list_receiver_probe_steps(receiver: &WrittenTypeShapeValue) -> Vec<WrittenTypeShapeValue> {
    let derefs = std::iter::successors(Some(receiver.clone()), |step| match step {
        WrittenTypeShapeValue::Reference { inner, .. } => Some((**inner).clone()),
        _ => None,
    })
    .collect::<Vec<_>>();
    let unsized_step = match derefs.last() {
        Some(WrittenTypeShapeValue::Array(element)) => {
            Some(WrittenTypeShapeValue::Slice(element.clone()))
        }
        _ => None,
    };
    derefs.into_iter().chain(unsized_step).collect()
}

/// P3 to P5 for one receiver form: inherent members first, then the traits in scope.
fn decide_probe_form_outcome(
    form: &WrittenTypeShapeValue,
    probe_step_text: String,
    candidates: &[InherentMemberCandidateValue<'_>],
    scope_traits: &[ScopeTraitMethodValue],
) -> ProbeFormDecisionValue {
    use ProbeFormDecisionValue::{Assumed, Decided, Passed};
    let fits = candidates
        .iter()
        .map(|candidate| {
            let fit = unify_member_receiver_shape(
                candidate.member_self_parameter_kind,
                &candidate.impl_self_type_shape,
                form,
            );
            match (fit, candidate.member_presence_unproven_flag) {
                (ShapeFitKind::Fits, true) => (candidate, ShapeFitKind::Unknown),
                _ => (candidate, fit),
            }
        })
        .filter(|(_, fit)| *fit != ShapeFitKind::Differs)
        .collect::<Vec<_>>();
    if fits.iter().any(|(_, fit)| *fit == ShapeFitKind::Unknown) {
        return Decided(CoreMemberProbeOutcome::Withheld("inherent_member_unproven"));
    }
    let may_fit = |found: &&ScopeTraitMethodValue| {
        found
            .trait_impl_self_shapes
            .clone()
            .unwrap_or_else(|| vec![WrittenTypeShapeValue::Wildcard])
            .iter()
            .any(|owner| {
                unify_member_receiver_shape(&found.member_self_parameter_kind, owner, form)
                    != ShapeFitKind::Differs
            })
    };
    match fits.as_slice() {
        [(candidate, _)] => Decided(CoreMemberProbeOutcome::Named {
            declaration_row_index: candidate.declaration_row_index,
            probe_step_text,
            prelude_trait_assumed_names: Vec::new(),
        }),
        // REQ-RUST-060.0 U1, U5: only prelude traits of `core` let the probe go on.
        [] => match scope_traits
            .iter()
            .filter(may_fit)
            .map(|found| found.prelude_trait_name_value.clone())
            .collect::<Option<Vec<_>>>()
        {
            Some(names) if names.is_empty() => Passed,
            Some(names) => Assumed(names),
            None => Decided(CoreMemberProbeOutcome::Withheld("scope_trait_may_fit")),
        },
        _ => Decided(CoreMemberProbeOutcome::Withheld(
            "inherent_member_ambiguous",
        )),
    }
}

/// P2 to P5: Rust's method probe for `receiver.method_name(..)` over `declarations`.
pub(crate) fn probe_core_member_declaration(
    receiver: &WrittenTypeShapeValue,
    method_name: &str,
    declarations: &[Value],
    scope_traits: &[ScopeTraitMethodValue],
) -> CoreMemberProbeOutcome {
    if *receiver == WrittenTypeShapeValue::Opaque {
        return CoreMemberProbeOutcome::Withheld("receiver_type_not_modelled");
    }
    if !list_shape_type_names(receiver)
        .iter()
        .all(|name| check_type_name_declared(name, declarations))
    {
        return CoreMemberProbeOutcome::Withheld("receiver_type_name_unproven");
    }
    if declarations
        .iter()
        .any(|row| check_macro_member_candidate(row, method_name, declarations))
    {
        return CoreMemberProbeOutcome::Withheld("macro_member_owner_unknown");
    }
    let candidates = declarations
        .iter()
        .enumerate()
        .filter(|(_, row)| {
            row["declaration_owner_kind_value"] == "inherent_impl"
                && row["member_declared_name_value"] == method_name
        })
        .filter_map(|(index, row)| build_inherent_member_candidate(index, row))
        .collect::<Vec<_>>();
    let walked = list_receiver_probe_steps(receiver)
        .iter()
        .enumerate()
        .flat_map(|(step, shape)| {
            let reference = |mutable| WrittenTypeShapeValue::Reference {
                mutable,
                inner: Box::new(shape.clone()),
            };
            [
                (format!("step {step} by value"), shape.clone()),
                (format!("step {step} by reference"), reference(false)),
                (format!("step {step} by mutable reference"), reference(true)),
            ]
        })
        .try_fold(
            BTreeSet::new(),
            |assumed: BTreeSet<String>, (text, form)| match decide_probe_form_outcome(
                &form,
                text,
                &candidates,
                scope_traits,
            ) {
                ProbeFormDecisionValue::Decided(outcome) => ControlFlow::Break((outcome, assumed)),
                ProbeFormDecisionValue::Assumed(names) => {
                    ControlFlow::Continue(assumed.into_iter().chain(names).collect())
                }
                ProbeFormDecisionValue::Passed => ControlFlow::Continue(assumed),
            },
        );
    // REQ-RUST-060.0 U3, U4: a name found after a prelude trait was passed carries the
    // trait's name; anything else found after it keeps the trait's reason.
    match walked {
        ControlFlow::Break((
            CoreMemberProbeOutcome::Named {
                declaration_row_index,
                probe_step_text,
                ..
            },
            assumed,
        )) => CoreMemberProbeOutcome::Named {
            declaration_row_index,
            probe_step_text,
            prelude_trait_assumed_names: assumed.into_iter().collect(),
        },
        ControlFlow::Break((outcome, assumed)) if assumed.is_empty() => outcome,
        ControlFlow::Continue(assumed) if assumed.is_empty() => {
            CoreMemberProbeOutcome::Withheld("no_member_declared")
        }
        _ => CoreMemberProbeOutcome::Withheld("scope_trait_may_fit"),
    }
}

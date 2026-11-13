#![allow(clippy::unwrap_used)]

use hir::Semantics;
use hir_expand::{
    EditionedFileId, ExpandError, ExpandErrorKind, HirFileId, Lookup, MacroCallId, MacroCallKind,
    MacroDefKind,
    tt::{self, DelimiterKind, Leaf, Spacing, TopSubtree, TopSubtreeBuilder, TtElement, TtIter},
};
use ide_db::RootDatabase;
use project_model::{ProjectManifest, ProjectWorkspace};
use std::{fs, ops::Range};
use syntax::{AstNode, ast};
use vfs::VfsPath;

const RAW_MACRO_TOKEN_LIMIT: usize = 2_097_152;
const RAW_MACRO_FIXTURE_SOURCE: &str = r#"
macro_rules! make {
    ($value:expr) => {
        fn generated() {
            let _ = $value * 2;
            let _ = $value * 2;
            let _ = $crate::helper();
            let _ = definition_only();
        }
    };
}

macro_rules! reject {
    ($value:literal) => { $value };
}

macro_rules! passthrough {
    ($value:expr) => { $value };
}

fn helper() {}
fn definition_only() {}

make!(café() + 1);
reject!();
fn precedence() {
    let _ = passthrough!(café() + 1) * 2;
}
"#;

#[derive(Debug, Eq, PartialEq)]
struct RawMacroExpansionOutput {
    canonical_source_text: String,
    token_positions: Vec<MacroTokenPositionRecord>,
    subtree_topology: Vec<MacroSubtreeTopologyRecord>,
    matched_arm: Option<u32>,
    call_site_span: tt::Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MacroTokenPositionRecord {
    tt_path: Vec<usize>,
    output_byte_range: Range<usize>,
    token_text: String,
    emission_kind: MacroTokenEmissionKind,
    spacing: Option<Spacing>,
    span: tt::Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MacroSubtreeTopologyRecord {
    tt_path: Vec<usize>,
    delimiter_kind: DelimiterKind,
    subtree_token_count: usize,
    output_byte_range: Range<usize>,
    delimiter: tt::Delimiter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MacroTokenEmissionKind {
    Leaf,
    OpenDelimiter,
    CloseDelimiter,
}

#[derive(Debug)]
enum RawMacroExpansionError {
    NotResolved,
    WrongCallKind,
    WrongDefinitionKind,
    Expansion(ExpandError),
    TokenLimit { observed: usize, limit: usize },
}

struct MacroFixtureDatabaseState {
    _temporary_directory: tempfile::TempDir,
    database: RootDatabase,
    _source_virtual_files: vfs::Vfs,
    _macro_server_handle: Option<proc_macro_api::ProcMacroClient>,
    source_file_id: vfs::FileId,
    source_text: String,
}

fn check_macro_expansion_limit(observed: usize) -> Result<(), RawMacroExpansionError> {
    if observed > RAW_MACRO_TOKEN_LIMIT {
        Err(RawMacroExpansionError::TokenLimit {
            observed,
            limit: RAW_MACRO_TOKEN_LIMIT,
        })
    } else {
        Ok(())
    }
}

fn macro_token_source_range(fixture: &MacroFixtureDatabaseState, span: tt::Span) -> Range<usize> {
    assert_eq!(span.anchor.file_id.file_id(), fixture.source_file_id);
    let source_file = EditionedFileId::from_span_file_id(&fixture.database, span.anchor.file_id);
    let anchor_start = HirFileId::from(source_file)
        .ast_id_map(&fixture.database)
        .get_erased(span.anchor.ast_id)
        .text_range()
        .start();
    let source_start = anchor_start + span.range.start();
    let source_end = anchor_start + span.range.end();
    u32::from(source_start) as usize..u32::from(source_end) as usize
}

fn assert_delimiter_metadata_preserved(
    expansion: &RawMacroExpansionOutput,
    topology: &MacroSubtreeTopologyRecord,
) {
    let open_event = expansion
        .token_positions
        .iter()
        .find(|position| {
            position.tt_path == topology.tt_path
                && position.emission_kind == MacroTokenEmissionKind::OpenDelimiter
        })
        .unwrap();
    let close_event = expansion
        .token_positions
        .iter()
        .find(|position| {
            position.tt_path == topology.tt_path
                && position.emission_kind == MacroTokenEmissionKind::CloseDelimiter
        })
        .unwrap();
    assert_eq!(open_event.span, topology.delimiter.open);
    assert_eq!(close_event.span, topology.delimiter.close);
}

fn append_macro_token_event(
    output: &mut RawMacroExpansionOutput,
    tt_path: &[usize],
    token_text: &str,
    emission_kind: MacroTokenEmissionKind,
    spacing: Option<Spacing>,
    span: tt::Span,
) {
    let start = output.canonical_source_text.len();
    output.canonical_source_text.push_str(token_text);
    let end = output.canonical_source_text.len();
    output.token_positions.push(MacroTokenPositionRecord {
        tt_path: tt_path.to_vec(),
        output_byte_range: start..end,
        token_text: token_text.to_owned(),
        emission_kind,
        spacing,
        span,
    });
}

fn render_macro_tree_events(
    tokens: TtIter<'_>,
    tt_path: &mut Vec<usize>,
    output: &mut RawMacroExpansionOutput,
) {
    for (index, token) in tokens.enumerate() {
        tt_path.push(index);
        match token {
            TtElement::Leaf(leaf) => {
                let spacing = match &leaf {
                    Leaf::Punct(punct) => Some(punct.spacing),
                    _ => None,
                };
                append_macro_token_event(
                    output,
                    tt_path,
                    &leaf.to_string(),
                    MacroTokenEmissionKind::Leaf,
                    spacing,
                    *leaf.span(),
                );
                if spacing != Some(Spacing::Joint) {
                    output.canonical_source_text.push(' ');
                }
            }
            TtElement::Subtree(subtree, children) => {
                let start = output.canonical_source_text.len();
                let delimiter = subtree.delimiter;
                let pair = match delimiter.kind {
                    DelimiterKind::Parenthesis => Some(("(", ")")),
                    DelimiterKind::Brace => Some(("{", "}")),
                    DelimiterKind::Bracket => Some(("[", "]")),
                    DelimiterKind::Invisible => None,
                };
                if let Some((open, _)) = pair {
                    append_macro_token_event(
                        output,
                        tt_path,
                        open,
                        MacroTokenEmissionKind::OpenDelimiter,
                        None,
                        delimiter.open,
                    );
                }
                render_macro_tree_events(children, tt_path, output);
                if let Some((_, close)) = pair {
                    append_macro_token_event(
                        output,
                        tt_path,
                        close,
                        MacroTokenEmissionKind::CloseDelimiter,
                        None,
                        delimiter.close,
                    );
                    output.canonical_source_text.push(' ');
                }
                output.subtree_topology.push(MacroSubtreeTopologyRecord {
                    tt_path: tt_path.clone(),
                    delimiter_kind: delimiter.kind,
                    subtree_token_count: subtree.usize_len(),
                    output_byte_range: start..output.canonical_source_text.len(),
                    delimiter,
                });
            }
        }
        tt_path.pop();
    }
}

fn render_raw_macro_expansion(
    expanded_tokens: &TopSubtree,
    matched_arm: Option<u32>,
    call_site_span: tt::Span,
) -> RawMacroExpansionOutput {
    let mut output = RawMacroExpansionOutput {
        canonical_source_text: String::new(),
        token_positions: Vec::new(),
        subtree_topology: Vec::new(),
        matched_arm,
        call_site_span,
    };
    render_macro_tree_events(expanded_tokens.iter(), &mut Vec::new(), &mut output);
    let root = expanded_tokens.top_subtree();
    output.subtree_topology.push(MacroSubtreeTopologyRecord {
        tt_path: Vec::new(),
        delimiter_kind: root.delimiter.kind,
        subtree_token_count: root.usize_len(),
        output_byte_range: 0..output.canonical_source_text.len(),
        delimiter: root.delimiter,
    });
    output
        .subtree_topology
        .sort_by(|left, right| left.tt_path.cmp(&right.tt_path));
    output
}

fn load_macro_fixture_database() -> MacroFixtureDatabaseState {
    let (temporary_directory, request, _) =
        crate::oracle_artifact_contract_tests::build_artifact_test_fixture(false);
    let source_path = temporary_directory.path().join("src/lib.rs");
    fs::write(&source_path, RAW_MACRO_FIXTURE_SOURCE).unwrap();
    let canonical_root = temporary_directory.path().canonicalize().unwrap();
    let root = vfs::AbsPathBuf::try_from(canonical_root.to_str().unwrap()).unwrap();
    let manifest = ProjectManifest::discover_single(&root).unwrap();
    let configuration = super::build_primary_cargo_configuration(&request, &root);
    let workspace = ProjectWorkspace::load(manifest, &configuration, &|_| {}).unwrap();
    let loaded = crate::primary_workspace_loader_adapter::load_primary_workspace_database(
        workspace,
        &configuration,
        &[],
        &Default::default(),
    )
    .unwrap();
    let source_absolute_path = vfs::AbsPathBuf::try_from(source_path.to_str().unwrap()).unwrap();
    let source_file_id = loaded
        .oracle_source_virtual_files
        .file_id(&VfsPath::from(source_absolute_path))
        .map(|(file_id, _)| file_id)
        .unwrap();
    MacroFixtureDatabaseState {
        _temporary_directory: temporary_directory,
        database: loaded.oracle_database_owned_value,
        _source_virtual_files: loaded.oracle_source_virtual_files,
        _macro_server_handle: loaded.oracle_macro_server_handle,
        source_file_id,
        source_text: RAW_MACRO_FIXTURE_SOURCE.to_owned(),
    }
}

fn expand_resolved_macro_call(
    fixture: &MacroFixtureDatabaseState,
    macro_name: &str,
) -> Result<(MacroCallId, RawMacroExpansionOutput), RawMacroExpansionError> {
    let semantics = Semantics::new(&fixture.database);
    let macro_call = semantics
        .parse_guess_edition(fixture.source_file_id)
        .syntax()
        .descendants()
        .filter_map(ast::MacroCall::cast)
        .find(|macro_call| {
            macro_call
                .path()
                .is_some_and(|path| path.syntax().text() == macro_name)
        })
        .ok_or(RawMacroExpansionError::NotResolved)?;
    let expanded = semantics
        .expand_macro_call(&macro_call)
        .ok_or(RawMacroExpansionError::NotResolved)?;
    let call_id = expanded
        .file_id
        .macro_file()
        .ok_or(RawMacroExpansionError::NotResolved)?;
    let loc = call_id.lookup(&fixture.database);
    if !matches!(loc.kind, MacroCallKind::FnLike { .. }) {
        return Err(RawMacroExpansionError::WrongCallKind);
    }
    let definition_ast_id = match &loc.def.kind {
        MacroDefKind::Declarative(ast_id, _) => *ast_id,
        _ => return Err(RawMacroExpansionError::WrongDefinitionKind),
    };
    let macro_argument = call_id.macro_arg_considering_derives(&fixture.database, &loc.kind);
    let expander = definition_ast_id.decl_macro_expander(&fixture.database, loc.def.krate);
    let call_site_span = macro_argument.2;
    let expansion = expander.expand(
        &fixture.database,
        &macro_argument.0,
        call_id,
        macro_argument.2,
    );
    let token_count = expansion.value.0.top_subtree().count();
    check_macro_expansion_limit(token_count)?;
    let matched_arm = expansion.value.1;
    if let Some(error) = expansion.err {
        return Err(RawMacroExpansionError::Expansion(error));
    }
    Ok((
        call_id,
        render_raw_macro_expansion(&expansion.value.0, matched_arm, call_site_span),
    ))
}

/// REQ-RUST-029.0
#[test]
fn test_raw_macro_expansion_bytes() {
    let fixture = load_macro_fixture_database();
    let (_, expansion) = expand_resolved_macro_call(&fixture, "make").unwrap();
    assert_eq!(
        expansion.canonical_source_text,
        "fn generated () {let _ = (café () + 1 ) * 2 ; let _ = (café () + 1 ) * 2 ; let _ = $crate :: helper () ; let _ = definition_only () ; } "
    );
    assert_eq!(expansion.matched_arm, Some(0));
    let actual_events = expansion
        .token_positions
        .iter()
        .map(|position| {
            (
                position.emission_kind,
                position.token_text.as_str(),
                position.output_byte_range.clone(),
                position.spacing,
            )
        })
        .collect::<Vec<_>>();
    let expected_events = vec![
        (MacroTokenEmissionKind::Leaf, "fn", 0..2, None),
        (MacroTokenEmissionKind::Leaf, "generated", 3..12, None),
        (MacroTokenEmissionKind::OpenDelimiter, "(", 13..14, None),
        (MacroTokenEmissionKind::CloseDelimiter, ")", 14..15, None),
        (MacroTokenEmissionKind::OpenDelimiter, "{", 16..17, None),
        (MacroTokenEmissionKind::Leaf, "let", 17..20, None),
        (MacroTokenEmissionKind::Leaf, "_", 21..22, None),
        (
            MacroTokenEmissionKind::Leaf,
            "=",
            23..24,
            Some(Spacing::Alone),
        ),
        (MacroTokenEmissionKind::OpenDelimiter, "(", 25..26, None),
        (MacroTokenEmissionKind::Leaf, "café", 26..31, None),
        (MacroTokenEmissionKind::OpenDelimiter, "(", 32..33, None),
        (MacroTokenEmissionKind::CloseDelimiter, ")", 33..34, None),
        (
            MacroTokenEmissionKind::Leaf,
            "+",
            35..36,
            Some(Spacing::Alone),
        ),
        (MacroTokenEmissionKind::Leaf, "1", 37..38, None),
        (MacroTokenEmissionKind::CloseDelimiter, ")", 39..40, None),
        (
            MacroTokenEmissionKind::Leaf,
            "*",
            41..42,
            Some(Spacing::Alone),
        ),
        (MacroTokenEmissionKind::Leaf, "2", 43..44, None),
        (
            MacroTokenEmissionKind::Leaf,
            ";",
            45..46,
            Some(Spacing::Alone),
        ),
        (MacroTokenEmissionKind::Leaf, "let", 47..50, None),
        (MacroTokenEmissionKind::Leaf, "_", 51..52, None),
        (
            MacroTokenEmissionKind::Leaf,
            "=",
            53..54,
            Some(Spacing::Alone),
        ),
        (MacroTokenEmissionKind::OpenDelimiter, "(", 55..56, None),
        (MacroTokenEmissionKind::Leaf, "café", 56..61, None),
        (MacroTokenEmissionKind::OpenDelimiter, "(", 62..63, None),
        (MacroTokenEmissionKind::CloseDelimiter, ")", 63..64, None),
        (
            MacroTokenEmissionKind::Leaf,
            "+",
            65..66,
            Some(Spacing::Alone),
        ),
        (MacroTokenEmissionKind::Leaf, "1", 67..68, None),
        (MacroTokenEmissionKind::CloseDelimiter, ")", 69..70, None),
        (
            MacroTokenEmissionKind::Leaf,
            "*",
            71..72,
            Some(Spacing::Alone),
        ),
        (MacroTokenEmissionKind::Leaf, "2", 73..74, None),
        (
            MacroTokenEmissionKind::Leaf,
            ";",
            75..76,
            Some(Spacing::Alone),
        ),
        (MacroTokenEmissionKind::Leaf, "let", 77..80, None),
        (MacroTokenEmissionKind::Leaf, "_", 81..82, None),
        (
            MacroTokenEmissionKind::Leaf,
            "=",
            83..84,
            Some(Spacing::Alone),
        ),
        (MacroTokenEmissionKind::Leaf, "$crate", 85..91, None),
        (
            MacroTokenEmissionKind::Leaf,
            ":",
            92..93,
            Some(Spacing::Joint),
        ),
        (
            MacroTokenEmissionKind::Leaf,
            ":",
            93..94,
            Some(Spacing::Alone),
        ),
        (MacroTokenEmissionKind::Leaf, "helper", 95..101, None),
        (MacroTokenEmissionKind::OpenDelimiter, "(", 102..103, None),
        (MacroTokenEmissionKind::CloseDelimiter, ")", 103..104, None),
        (
            MacroTokenEmissionKind::Leaf,
            ";",
            105..106,
            Some(Spacing::Alone),
        ),
        (MacroTokenEmissionKind::Leaf, "let", 107..110, None),
        (MacroTokenEmissionKind::Leaf, "_", 111..112, None),
        (
            MacroTokenEmissionKind::Leaf,
            "=",
            113..114,
            Some(Spacing::Alone),
        ),
        (
            MacroTokenEmissionKind::Leaf,
            "definition_only",
            115..130,
            None,
        ),
        (MacroTokenEmissionKind::OpenDelimiter, "(", 131..132, None),
        (MacroTokenEmissionKind::CloseDelimiter, ")", 132..133, None),
        (
            MacroTokenEmissionKind::Leaf,
            ";",
            134..135,
            Some(Spacing::Alone),
        ),
        (MacroTokenEmissionKind::CloseDelimiter, "}", 136..137, None),
    ];
    assert_eq!(actual_events, expected_events);
    let actual_paths = expansion
        .token_positions
        .iter()
        .map(|position| position.tt_path.clone())
        .collect::<Vec<_>>();
    let expected_paths = vec![
        vec![0],
        vec![1],
        vec![2],
        vec![2],
        vec![3],
        vec![3, 0],
        vec![3, 1],
        vec![3, 2],
        vec![3, 3],
        vec![3, 3, 0],
        vec![3, 3, 1],
        vec![3, 3, 1],
        vec![3, 3, 2],
        vec![3, 3, 3],
        vec![3, 3],
        vec![3, 4],
        vec![3, 5],
        vec![3, 6],
        vec![3, 7],
        vec![3, 8],
        vec![3, 9],
        vec![3, 10],
        vec![3, 10, 0],
        vec![3, 10, 1],
        vec![3, 10, 1],
        vec![3, 10, 2],
        vec![3, 10, 3],
        vec![3, 10],
        vec![3, 11],
        vec![3, 12],
        vec![3, 13],
        vec![3, 14],
        vec![3, 15],
        vec![3, 16],
        vec![3, 17],
        vec![3, 18],
        vec![3, 19],
        vec![3, 20],
        vec![3, 21],
        vec![3, 21],
        vec![3, 22],
        vec![3, 23],
        vec![3, 24],
        vec![3, 25],
        vec![3, 26],
        vec![3, 27],
        vec![3, 27],
        vec![3, 28],
        vec![3],
    ];
    assert_eq!(actual_paths, expected_paths);
}

/// REQ-RUST-029.0
#[test]
fn test_macro_token_origin_positions() {
    let fixture = load_macro_fixture_database();
    let (_, expansion) = expand_resolved_macro_call(&fixture, "make").unwrap();
    let captured = expansion
        .token_positions
        .iter()
        .filter(|position| position.token_text == "café")
        .collect::<Vec<_>>();
    assert_eq!(captured.len(), 2);
    assert_ne!(captured[0].tt_path, captured[1].tt_path);
    assert_ne!(captured[0].output_byte_range, captured[1].output_byte_range);
    assert_eq!(captured[0].span, captured[1].span);
    assert_eq!(expansion.call_site_span.ctx, captured[0].span.ctx);
    assert_eq!(expansion.call_site_span.anchor, captured[0].span.anchor);
    assert_eq!(
        captured
            .iter()
            .map(|position| position.output_byte_range.clone())
            .collect::<Vec<_>>(),
        vec![26..31, 56..61]
    );
    let captured_source_range = macro_token_source_range(&fixture, captured[0].span);
    assert_eq!(
        fixture.source_text.get(captured_source_range.clone()),
        Some("café")
    );
    assert_eq!(
        macro_token_source_range(&fixture, captured[1].span),
        captured_source_range
    );

    let dollar_crate = expansion
        .token_positions
        .iter()
        .find(|position| position.token_text == "$crate")
        .unwrap();
    let definition_call = expansion
        .token_positions
        .iter()
        .find(|position| position.token_text == "definition_only")
        .unwrap();
    assert_eq!(dollar_crate.output_byte_range, 85..91);
    assert_eq!(definition_call.output_byte_range, 115..130);
    let dollar_crate_span_range = macro_token_source_range(&fixture, dollar_crate.span);
    assert_eq!(
        fixture.source_text.get(dollar_crate_span_range.clone()),
        Some("crate")
    );
    let dollar_crate_source_range =
        dollar_crate_span_range.start.checked_sub(1).unwrap()..dollar_crate_span_range.end;
    assert_eq!(
        fixture.source_text.as_bytes()[dollar_crate_source_range.start],
        b'$'
    );
    let definition_source_range = macro_token_source_range(&fixture, definition_call.span);
    assert_eq!(
        fixture.source_text.get(dollar_crate_source_range.clone()),
        Some("$crate")
    );
    assert_eq!(
        fixture.source_text.get(definition_source_range.clone()),
        Some("definition_only")
    );
    assert!(dollar_crate_source_range.start < definition_source_range.start);
    assert_eq!(dollar_crate.span.anchor, definition_call.span.anchor);
    assert_ne!(dollar_crate.span.anchor, captured[0].span.anchor);
    assert_eq!(dollar_crate.span.ctx, definition_call.span.ctx);
    assert_ne!(dollar_crate.span.ctx, captured[0].span.ctx);
}

/// REQ-RUST-029.0
#[test]
fn test_expr_invisible_group_topology() {
    let fixture = load_macro_fixture_database();
    let (_, expansion) = expand_resolved_macro_call(&fixture, "passthrough").unwrap();
    assert_eq!(expansion.canonical_source_text, "(café () + 1 ) ");
    let actual_topology = expansion
        .subtree_topology
        .iter()
        .map(|subtree| {
            (
                subtree.tt_path.clone(),
                subtree.delimiter_kind,
                subtree.output_byte_range.clone(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual_topology,
        vec![
            (vec![], DelimiterKind::Invisible, 0..16),
            (vec![0], DelimiterKind::Parenthesis, 0..16),
            (vec![0, 1], DelimiterKind::Parenthesis, 7..10),
        ]
    );
    let expression_group = expansion
        .subtree_topology
        .iter()
        .find(|subtree| subtree.tt_path == vec![0])
        .unwrap();
    let call_group = expansion
        .subtree_topology
        .iter()
        .find(|subtree| subtree.tt_path == vec![0, 1])
        .unwrap();
    assert_delimiter_metadata_preserved(&expansion, expression_group);
    assert_delimiter_metadata_preserved(&expansion, call_group);
    assert!(
        expansion
            .token_positions
            .iter()
            .all(|position| !position.tt_path.is_empty())
    );

    let input_span = expansion
        .token_positions
        .iter()
        .find(|position| position.token_text == "café")
        .unwrap()
        .span;
    let mut nested_builder = TopSubtreeBuilder::new(tt::Delimiter::invisible_spanned(input_span));
    nested_builder.open(DelimiterKind::Invisible, input_span);
    nested_builder.push(Leaf::Punct(tt::Punct {
        char: ':',
        spacing: Spacing::Alone,
        span: input_span,
    }));
    nested_builder.close(input_span);
    let isolated_nested_invisible_renderer_proof = nested_builder.build();
    let nested_rendered_proof = render_raw_macro_expansion(
        &isolated_nested_invisible_renderer_proof,
        None,
        expansion.call_site_span,
    );
    assert_eq!(nested_rendered_proof.canonical_source_text, ": ");
    assert_eq!(
        nested_rendered_proof.token_positions,
        vec![MacroTokenPositionRecord {
            tt_path: vec![0, 0],
            output_byte_range: 0..1,
            token_text: ":".to_owned(),
            emission_kind: MacroTokenEmissionKind::Leaf,
            spacing: Some(Spacing::Alone),
            span: input_span,
        }]
    );
    assert_eq!(nested_rendered_proof.subtree_topology.len(), 2);
    for (topology, expected_path) in nested_rendered_proof
        .subtree_topology
        .iter()
        .zip([Vec::new(), vec![0]])
    {
        assert_eq!(topology.tt_path, expected_path);
        assert_eq!(topology.delimiter_kind, DelimiterKind::Invisible);
        assert_eq!(topology.output_byte_range, 0..2);
        assert_eq!(topology.delimiter.open, input_span);
        assert_eq!(topology.delimiter.close, input_span);
    }
}

/// REQ-RUST-029.0
#[test]
fn test_joint_hidden_spacing_preserved() {
    let fixture = load_macro_fixture_database();
    let (_, expansion) = expand_resolved_macro_call(&fixture, "make").unwrap();
    assert!(expansion.token_positions.iter().any(|position| {
        position.token_text == ":" && position.spacing == Some(Spacing::Joint)
    }));
    assert!(expansion.token_positions.iter().any(|position| {
        position.token_text == ":" && position.spacing == Some(Spacing::Alone)
    }));
    assert!(expansion.canonical_source_text.contains("$crate :: helper"));

    let colon_span = expansion
        .token_positions
        .iter()
        .find(|position| position.token_text == ":")
        .unwrap()
        .span;
    let isolated_renderer_proof = TopSubtree::invisible_from_leaves(
        colon_span,
        [
            Leaf::Punct(tt::Punct {
                char: ':',
                spacing: Spacing::Joint,
                span: colon_span,
            }),
            Leaf::Punct(tt::Punct {
                char: ':',
                spacing: Spacing::JointHidden,
                span: colon_span,
            }),
        ],
    );
    let rendered_proof =
        render_raw_macro_expansion(&isolated_renderer_proof, None, expansion.call_site_span);
    assert_eq!(rendered_proof.canonical_source_text, ":: ");
    assert_eq!(
        rendered_proof
            .token_positions
            .iter()
            .map(|position| position.spacing)
            .collect::<Vec<_>>(),
        vec![Some(Spacing::Joint), Some(Spacing::JointHidden)]
    );
    let actual_events = rendered_proof
        .token_positions
        .iter()
        .map(|position| {
            (
                position.tt_path.clone(),
                position.output_byte_range.clone(),
                position.token_text.as_str(),
                position.spacing,
                position.span,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual_events,
        vec![
            (vec![0], 0..1, ":", Some(Spacing::Joint), colon_span),
            (vec![1], 1..2, ":", Some(Spacing::JointHidden), colon_span),
        ]
    );
    let root_topology = rendered_proof
        .subtree_topology
        .iter()
        .find(|topology| topology.tt_path.is_empty())
        .unwrap();
    assert_eq!(root_topology.delimiter_kind, DelimiterKind::Invisible);
    assert_eq!(root_topology.output_byte_range, 0..3);
    assert_eq!(root_topology.delimiter.open, colon_span);
    assert_eq!(root_topology.delimiter.close, colon_span);
}

/// REQ-RUST-029.0
#[test]
fn test_unicode_output_byte_ranges() {
    let fixture = load_macro_fixture_database();
    let (_, expansion) = expand_resolved_macro_call(&fixture, "make").unwrap();
    let unicode_ranges = expansion
        .token_positions
        .iter()
        .filter(|position| position.token_text == "café")
        .map(|position| position.output_byte_range.clone())
        .collect::<Vec<_>>();
    assert_eq!(unicode_ranges, vec![26..31, 56..61]);
    for range in unicode_ranges {
        assert_eq!(
            &expansion.canonical_source_text.as_bytes()[range],
            "café".as_bytes()
        );
    }
}

/// REQ-RUST-029.0
#[test]
fn test_macro_token_limit_boundary() {
    assert!(check_macro_expansion_limit(RAW_MACRO_TOKEN_LIMIT).is_ok());
    assert!(matches!(
        check_macro_expansion_limit(RAW_MACRO_TOKEN_LIMIT + 1),
        Err(RawMacroExpansionError::TokenLimit {
            observed: 2_097_153,
            limit: 2_097_152
        })
    ));
}

/// REQ-RUST-029.0
#[test]
fn test_macro_expansion_error_typed() {
    let fixture = load_macro_fixture_database();
    match expand_resolved_macro_call(&fixture, "reject") {
        Err(RawMacroExpansionError::Expansion(error)) => {
            assert!(matches!(error.kind(), ExpandErrorKind::Mbe(_)));
        }
        other => panic!("expected typed matcher expansion error, got {other:?}"),
    }
}

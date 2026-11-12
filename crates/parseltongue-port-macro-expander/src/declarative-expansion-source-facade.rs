use crate::macro_expansion_rule_engine::{DeclarativeMacro, MacroCallStyle};
use parseltongue_core_row_types::{
    DeclarativeMacroExpansionInput, DeclarativeMacroExpansionOutput, MacroExpansionFailureKind,
    MacroSourceOriginKind, MacroTokenOriginRecord, RunCancelFlagValue, RustSourceEditionKind,
};
use parser::LexedStr;
use span::{
    Edition, EditionedFileId, FileId, ROOT_ERASED_FILE_AST_ID, Span, SpanAnchor, SyntaxContext,
    TextRange, TextSize,
};
use tt::{
    Delimiter, DelimiterKind, Leaf, Spacing, TopSubtree, TopSubtreeBuilder,
    iter::{TtElement, TtIter},
};

pub fn expand_declarative_macro_items(
    input: &DeclarativeMacroExpansionInput,
    cancel: &RunCancelFlagValue,
) -> Result<DeclarativeMacroExpansionOutput, MacroExpansionFailureKind> {
    check_macro_expansion_cancelled(cancel)?;
    let definition_edition = read_macro_source_edition(input.macro_definition_edition_kind);
    let invocation_edition = read_macro_source_edition(input.macro_invocation_edition_kind);
    let definition =
        build_macro_source_tokens(&input.macro_definition_source_text, definition_edition, 0)
            .map_err(MacroExpansionFailureKind::InvalidDefinition)?;
    let arguments =
        build_macro_source_tokens(&input.macro_argument_source_text, invocation_edition, 1)
            .map_err(MacroExpansionFailureKind::InvalidInvocation)?;
    let pattern = input
        .macro_pattern_source_text
        .as_deref()
        .map(|pattern| build_macro_source_tokens(pattern, definition_edition, 0))
        .transpose()
        .map_err(MacroExpansionFailureKind::InvalidDefinition)?;
    let rules = if let Some(pattern) = &pattern {
        DeclarativeMacro::parse_macro2(Some(pattern), &definition, SyntaxContext::edition)
    } else {
        DeclarativeMacro::parse_macro_rules(&definition, SyntaxContext::edition)
    };
    if let Some(error) = rules.err() {
        return Err(MacroExpansionFailureKind::InvalidDefinition(
            error.to_string(),
        ));
    }
    let call_site = arguments.top_subtree().delimiter.open;
    let expanded = rules.expand(
        SyntaxContext::edition,
        &arguments,
        |_| {},
        MacroCallStyle::FnLike,
        call_site,
    );
    check_macro_expansion_cancelled(cancel)?;
    if let Some(error) = expanded.err {
        return Err(match &error.inner.1 {
            crate::macro_expansion_rule_engine::ExpandErrorKind::LimitExceeded
            | crate::macro_expansion_rule_engine::ExpandErrorKind::UnresolvedBinding(_) => {
                MacroExpansionFailureKind::Expansion(error.to_string())
            }
            _ => MacroExpansionFailureKind::NoMatchingRule,
        });
    }
    let (tokens, arm) = expanded.value;
    let arm = arm.ok_or(MacroExpansionFailureKind::NoMatchingRule)?;
    let mut output = DeclarativeMacroExpansionOutput {
        macro_expanded_source_text: String::new(),
        macro_token_origin_rows: Vec::new(),
        macro_matched_arm_index: arm,
    };
    render_macro_output_tokens(tokens.iter(), &mut output)?;
    check_macro_expansion_cancelled(cancel)?;
    Ok(output)
}

fn check_macro_expansion_cancelled(
    cancel: &RunCancelFlagValue,
) -> Result<(), MacroExpansionFailureKind> {
    if cancel.check_stage_work_cancelled() {
        Err(MacroExpansionFailureKind::Cancelled)
    } else {
        Ok(())
    }
}

fn read_macro_source_edition(edition: RustSourceEditionKind) -> Edition {
    match edition {
        RustSourceEditionKind::Edition2015 => Edition::Edition2015,
        RustSourceEditionKind::Edition2018 => Edition::Edition2018,
        RustSourceEditionKind::Edition2021 => Edition::Edition2021,
        RustSourceEditionKind::Edition2024 => Edition::Edition2024,
    }
}

fn build_macro_source_span(start: usize, end: usize, edition: Edition, file: u32) -> Span {
    Span {
        range: TextRange::new(TextSize::new(start as u32), TextSize::new(end as u32)),
        anchor: SpanAnchor {
            file_id: EditionedFileId::new(FileId::from_raw(file), edition),
            ast_id: ROOT_ERASED_FILE_AST_ID,
        },
        ctx: SyntaxContext::root(edition),
    }
}

fn read_macro_delimiter_kind(text: &str) -> Option<(DelimiterKind, bool)> {
    match text {
        "(" => Some((DelimiterKind::Parenthesis, true)),
        ")" => Some((DelimiterKind::Parenthesis, false)),
        "{" => Some((DelimiterKind::Brace, true)),
        "}" => Some((DelimiterKind::Brace, false)),
        "[" => Some((DelimiterKind::Bracket, true)),
        "]" => Some((DelimiterKind::Bracket, false)),
        _ => None,
    }
}

fn build_macro_source_tokens(
    source: &str,
    edition: Edition,
    file: u32,
) -> Result<TopSubtree, String> {
    u32::try_from(source.len()).map_err(|_| "macro source exceeds parser range".to_owned())?;
    let lexed = LexedStr::new(edition, source);
    if let Some((_, error)) = lexed.errors().next() {
        return Err(error.into());
    }
    let indexes = (0..lexed.len())
        .filter(|index| !lexed.kind(*index).is_trivia())
        .collect::<Vec<_>>();
    let first = *indexes.first().ok_or("missing outer macro delimiter")?;
    let last = *indexes.last().ok_or("missing outer macro delimiter")?;
    let (kind, true) =
        read_macro_delimiter_kind(lexed.text(first)).ok_or("missing opening macro delimiter")?
    else {
        return Err("expected opening macro delimiter".into());
    };
    if first == last || read_macro_delimiter_kind(lexed.text(last)) != Some((kind, false)) {
        return Err("unbalanced outer macro delimiter".into());
    }
    let source_span = |index: usize| {
        build_macro_source_span(
            lexed.text_start(index),
            lexed.text_start(index) + lexed.text(index).len(),
            edition,
            file,
        )
    };
    let mut builder = TopSubtreeBuilder::new(Delimiter {
        kind,
        open: source_span(first),
        close: source_span(last),
    });
    let mut nesting = Vec::new();
    for &index in indexes.iter().skip(1).take(indexes.len() - 2) {
        let text = lexed.text(index);
        let span = source_span(index);
        if let Some((kind, open)) = read_macro_delimiter_kind(text) {
            if open {
                nesting.push(kind);
                builder.open(kind, span);
            } else if nesting.pop() == Some(kind) {
                builder.close(span);
            } else {
                return Err("unbalanced nested macro delimiter".into());
            }
        } else if lexed.kind(index).is_literal() {
            builder.push(Leaf::Literal(tt::token_to_literal(text, span)));
        } else if lexed.kind(index) == ra_ap_syntax::SyntaxKind::LIFETIME_IDENT {
            let mut quote = span;
            quote.range = TextRange::at(span.range.start(), TextSize::new(1));
            builder.push(Leaf::Punct(tt::Punct {
                char: '\'',
                spacing: Spacing::Joint,
                span: quote,
            }));
            let mut identifier = span;
            identifier.range =
                TextRange::new(span.range.start() + TextSize::new(1), span.range.end());
            builder.push(Leaf::Ident(tt::Ident::new(&text[1..], identifier)));
        } else if text
            .chars()
            .next()
            .is_some_and(|character| character.is_alphabetic() || character == '_')
        {
            builder.push(Leaf::Ident(tt::Ident::new(text, span)));
        } else {
            let mut characters = text.chars();
            let character = characters.next().ok_or("empty macro token")?;
            if characters.next().is_some() {
                return Err(format!("unexpected macro token {text}"));
            }
            let joint = index + 1 < lexed.len() && !lexed.kind(index + 1).is_trivia();
            builder.push(Leaf::Punct(tt::Punct {
                char: character,
                spacing: if joint {
                    Spacing::Joint
                } else {
                    Spacing::Alone
                },
                span,
            }));
        }
    }
    if !nesting.is_empty() {
        return Err("unclosed nested macro delimiter".into());
    }
    Ok(builder.build())
}

fn append_macro_output_token(
    text: &str,
    span: Span,
    output: &mut DeclarativeMacroExpansionOutput,
) -> Result<(), MacroExpansionFailureKind> {
    let start = output.macro_expanded_source_text.len() as u64;
    output.macro_expanded_source_text.push_str(text);
    let end = output.macro_expanded_source_text.len() as u64;
    let token_output_byte_range = (start, end).try_into().map_err(
        |error: parseltongue_core_row_types::CoreRowValidationError| {
            MacroExpansionFailureKind::Expansion(error.to_string())
        },
    )?;
    let token_original_byte_range = (
        u32::from(span.range.start()) as u64,
        u32::from(span.range.end()) as u64,
    )
        .try_into()
        .map_err(
            |error: parseltongue_core_row_types::CoreRowValidationError| {
                MacroExpansionFailureKind::Expansion(error.to_string())
            },
        )?;
    output.macro_token_origin_rows.push(MacroTokenOriginRecord {
        token_output_byte_range,
        token_original_byte_range,
        token_source_origin_kind: if span.anchor.file_id.file_id().index() == 1 {
            MacroSourceOriginKind::Invocation
        } else {
            MacroSourceOriginKind::Definition
        },
    });
    Ok(())
}

fn render_macro_output_tokens(
    tokens: TtIter<'_>,
    output: &mut DeclarativeMacroExpansionOutput,
) -> Result<(), MacroExpansionFailureKind> {
    for token in tokens {
        match token {
            TtElement::Leaf(leaf) => {
                append_macro_output_token(&leaf.to_string(), *leaf.span(), output)?;
                if !matches!(
                    leaf,
                    Leaf::Punct(tt::Punct {
                        spacing: Spacing::Joint,
                        ..
                    })
                ) {
                    output.macro_expanded_source_text.push(' ');
                }
            }
            TtElement::Subtree(subtree, children) => {
                let pair = match subtree.delimiter.kind {
                    DelimiterKind::Parenthesis => Some(("(", ")")),
                    DelimiterKind::Brace => Some(("{", "}")),
                    DelimiterKind::Bracket => Some(("[", "]")),
                    DelimiterKind::Invisible => None,
                };
                if let Some((open, _)) = pair {
                    append_macro_output_token(open, subtree.delimiter.open, output)?;
                }
                render_macro_output_tokens(children, output)?;
                if let Some((_, close)) = pair {
                    append_macro_output_token(close, subtree.delimiter.close, output)?;
                    output.macro_expanded_source_text.push(' ');
                }
            }
        }
    }
    Ok(())
}

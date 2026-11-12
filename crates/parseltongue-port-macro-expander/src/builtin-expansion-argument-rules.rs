use parseltongue_core_row_types::*;
use parser::LexedStr;
use ra_ap_syntax::{AstNode, Edition, ast};

pub trait BuiltinMacroLookupPort {
    fn read_builtin_macro_name(&self, path: &[NamePathSegmentKind]) -> Option<String>;
}

pub fn expand_builtin_macro_items(
    input: &BuiltinMacroExpansionInput,
    lookup: &dyn BuiltinMacroLookupPort,
    cancel: &RunCancelFlagValue,
) -> Result<BuiltinMacroExpansionOutput, MacroExpansionFailureKind> {
    check_builtin_work_cancelled(cancel)?;
    let value = evaluate_builtin_macro_value(
        &input.macro_builtin_identifier_text,
        &input.macro_argument_source_text,
        input,
        lookup,
        cancel,
        0,
    )?;
    check_builtin_work_cancelled(cancel)?;
    Ok(match value {
        BuiltinArgumentValueKind::Text(value)
            if input.macro_builtin_identifier_text == "include" =>
        {
            BuiltinMacroExpansionOutput::Include(value)
        }
        BuiltinArgumentValueKind::Text(value) => {
            BuiltinMacroExpansionOutput::Expression(format!("{value:?}"))
        }
        BuiltinArgumentValueKind::Number(value) => BuiltinMacroExpansionOutput::Expression(value),
        BuiltinArgumentValueKind::Boolean(value) => {
            BuiltinMacroExpansionOutput::Expression(value.to_string())
        }
        BuiltinArgumentValueKind::Optional(Some(value)) => {
            BuiltinMacroExpansionOutput::Expression(format!("Some({value:?})"))
        }
        BuiltinArgumentValueKind::Optional(None) => {
            BuiltinMacroExpansionOutput::Expression("None".into())
        }
        BuiltinArgumentValueKind::Unavailable(reason, detail) => {
            BuiltinMacroExpansionOutput::Unavailable(reason, detail)
        }
    })
}

enum BuiltinArgumentValueKind {
    Text(String),
    Number(String),
    Boolean(bool),
    Optional(Option<String>),
    Unavailable(SiteUnresolvedReasonKind, String),
}

fn build_builtin_argument_error(error: impl std::fmt::Display) -> MacroExpansionFailureKind {
    MacroExpansionFailureKind::InvalidInvocation(error.to_string())
}

fn check_builtin_work_cancelled(
    cancel: &RunCancelFlagValue,
) -> Result<(), MacroExpansionFailureKind> {
    if cancel.check_stage_work_cancelled() {
        Err(MacroExpansionFailureKind::Cancelled)
    } else {
        Ok(())
    }
}

fn read_builtin_source_edition(input: &BuiltinMacroExpansionInput) -> Edition {
    match input.macro_invocation_edition_kind {
        RustSourceEditionKind::Edition2015 => Edition::Edition2015,
        RustSourceEditionKind::Edition2018 => Edition::Edition2018,
        RustSourceEditionKind::Edition2021 => Edition::Edition2021,
        RustSourceEditionKind::Edition2024 => Edition::Edition2024,
    }
}

fn read_builtin_delimiter_close(token: &str) -> Option<&'static str> {
    match token {
        "(" => Some(")"),
        "[" => Some("]"),
        "{" => Some("}"),
        _ => None,
    }
}

fn read_builtin_argument_body(
    source: &str,
    edition: Edition,
) -> Result<&str, MacroExpansionFailureKind> {
    let lexed = LexedStr::new(edition, source);
    if let Some((_, error)) = lexed.errors().next() {
        return Err(build_builtin_argument_error(error));
    }
    let indexes = (0..lexed.len())
        .filter(|index| !lexed.kind(*index).is_trivia())
        .collect::<Vec<_>>();
    let first = *indexes
        .first()
        .ok_or_else(|| build_builtin_argument_error("macro delimiter missing"))?;
    let last = *indexes
        .last()
        .ok_or_else(|| build_builtin_argument_error("macro delimiter missing"))?;
    if read_builtin_delimiter_close(lexed.text(first)) != Some(lexed.text(last)) {
        return Err(build_builtin_argument_error("unbalanced macro delimiters"));
    }
    let stack = indexes.iter().try_fold(Vec::new(), |mut stack, index| {
        let token = lexed.text(*index);
        if let Some(close) = read_builtin_delimiter_close(token) {
            stack.push(close);
        } else if matches!(token, ")" | "]" | "}") && stack.pop() != Some(token) {
            return Err(build_builtin_argument_error("unbalanced nested delimiter"));
        }
        if stack.is_empty() && *index != last {
            return Err(build_builtin_argument_error(
                "tokens outside macro delimiters",
            ));
        }
        Ok(stack)
    })?;
    if !stack.is_empty() {
        return Err(build_builtin_argument_error("unclosed macro delimiter"));
    }
    Ok(&source[lexed.text_start(first) + 1..lexed.text_start(last)])
}

fn split_builtin_argument_values(
    source: &str,
    edition: Edition,
) -> Result<Vec<String>, MacroExpansionFailureKind> {
    let lexed = LexedStr::new(edition, source);
    if let Some((_, error)) = lexed.errors().next() {
        return Err(build_builtin_argument_error(error));
    }
    let (stack, start, mut values) = (0..lexed.len())
        .filter(|index| !lexed.kind(*index).is_trivia())
        .try_fold(
            (Vec::new(), 0, Vec::new()),
            |(mut stack, mut start, mut values), index| {
                let token = lexed.text(index);
                if let Some(close) = read_builtin_delimiter_close(token) {
                    stack.push(close);
                } else if matches!(token, ")" | "]" | "}") && stack.pop() != Some(token) {
                    return Err(build_builtin_argument_error(
                        "unbalanced argument delimiter",
                    ));
                } else if token == "," && stack.is_empty() {
                    let value = source[start..lexed.text_start(index)].trim();
                    if value.is_empty() {
                        return Err(build_builtin_argument_error("empty builtin argument"));
                    }
                    values.push(value.into());
                    start = lexed.text_start(index) + 1;
                }
                Ok((stack, start, values))
            },
        )?;
    if !stack.is_empty() {
        return Err(build_builtin_argument_error("unclosed argument delimiter"));
    }
    if !source[start..].trim().is_empty() {
        values.push(source[start..].trim().into());
    }
    Ok(values)
}

fn read_builtin_path_segments(path: &ast::Path) -> Vec<NamePathSegmentKind> {
    let mut parts = path
        .qualifier()
        .as_ref()
        .map(read_builtin_path_segments)
        .unwrap_or_else(|| {
            if path.coloncolon_token().is_some() {
                vec![NamePathSegmentKind::GlobalRoot]
            } else {
                vec![]
            }
        });
    if let Some(segment) = path.segment() {
        match segment.kind() {
            Some(ast::PathSegmentKind::CrateKw) => parts.push(NamePathSegmentKind::CrateRoot),
            Some(ast::PathSegmentKind::SelfKw) => parts.push(NamePathSegmentKind::SelfModule),
            Some(ast::PathSegmentKind::SuperKw) => parts.push(NamePathSegmentKind::ParentModule),
            Some(ast::PathSegmentKind::Name(name)) => parts.push(NamePathSegmentKind::Name(
                name.text().trim_start_matches("r#").into(),
            )),
            _ => {}
        }
    }
    parts
}

fn evaluate_builtin_argument_value(
    source: &str,
    input: &BuiltinMacroExpansionInput,
    lookup: &dyn BuiltinMacroLookupPort,
    cancel: &RunCancelFlagValue,
    depth: usize,
) -> Result<BuiltinArgumentValueKind, MacroExpansionFailureKind> {
    check_builtin_work_cancelled(cancel)?;
    if depth > 128 {
        return Ok(BuiltinArgumentValueKind::Unavailable(
            SiteUnresolvedReasonKind::ExternalSourceMissing,
            "builtin argument expansion nesting exceeds 128".into(),
        ));
    }
    let parsed = ast::Expr::parse(source, read_builtin_source_edition(input));
    if let Some(error) = parsed.errors().first() {
        return Err(build_builtin_argument_error(error));
    }
    let node = parsed.syntax_node();
    if let Some(literal) = ast::Literal::cast(node.clone()) {
        return Ok(match literal.kind() {
            ast::LiteralKind::String(value) => BuiltinArgumentValueKind::Text(
                value
                    .value()
                    .map_err(|error| build_builtin_argument_error(format!("{error:?}")))?
                    .into_owned(),
            ),
            ast::LiteralKind::Char(value) => BuiltinArgumentValueKind::Text(
                value
                    .value()
                    .map_err(|error| build_builtin_argument_error(format!("{error:?}")))?
                    .to_string(),
            ),
            ast::LiteralKind::IntNumber(value) => BuiltinArgumentValueKind::Number(
                value
                    .value()
                    .map_err(build_builtin_argument_error)?
                    .to_string(),
            ),
            ast::LiteralKind::FloatNumber(value) => {
                BuiltinArgumentValueKind::Number(value.value_string())
            }
            ast::LiteralKind::Bool(value) => BuiltinArgumentValueKind::Boolean(value),
            _ => return Err(build_builtin_argument_error("unsupported concat literal")),
        });
    }
    if let Some(prefix) = ast::PrefixExpr::cast(node.clone())
        && source.trim_start().starts_with('-')
        && let Some(expression) = prefix.expr()
    {
        return match evaluate_builtin_argument_value(
            &expression.syntax().text().to_string(),
            input,
            lookup,
            cancel,
            depth + 1,
        )? {
            BuiltinArgumentValueKind::Number(value) => {
                Ok(BuiltinArgumentValueKind::Number(format!("-{value}")))
            }
            _ => Err(build_builtin_argument_error(
                "expected numeric literal after minus",
            )),
        };
    }
    if let Some(expression) = ast::MacroExpr::cast(node)
        && let Some(call) = expression.macro_call()
        && let Some(path) = call.path()
        && let Some(tokens) = call.token_tree()
    {
        return match lookup.read_builtin_macro_name(&read_builtin_path_segments(&path)) {
            Some(name) => evaluate_builtin_macro_value(
                &name,
                &tokens.syntax().text().to_string(),
                input,
                lookup,
                cancel,
                depth + 1,
            ),
            None => Ok(BuiltinArgumentValueKind::Unavailable(
                SiteUnresolvedReasonKind::ExternalSourceMissing,
                format!(
                    "nested macro expansion unavailable: {}",
                    path.syntax().text()
                ),
            )),
        };
    }
    Err(build_builtin_argument_error(
        "expected literal or supported eager builtin",
    ))
}

fn evaluate_builtin_macro_value(
    name: &str,
    arguments: &str,
    input: &BuiltinMacroExpansionInput,
    lookup: &dyn BuiltinMacroLookupPort,
    cancel: &RunCancelFlagValue,
    depth: usize,
) -> Result<BuiltinArgumentValueKind, MacroExpansionFailureKind> {
    check_builtin_work_cancelled(cancel)?;
    let edition = read_builtin_source_edition(input);
    let body = read_builtin_argument_body(arguments, edition)?;
    if name == "stringify" {
        let lexed = LexedStr::new(edition, body);
        let (_, text) = (0..lexed.len())
            .filter(|index| !lexed.kind(*index).is_trivia())
            .fold((0, String::new()), |(end, mut text), index| {
                let start = lexed.text_start(index);
                if !text.is_empty() && start > end {
                    text.push(' ');
                }
                text.push_str(lexed.text(index));
                (start + lexed.text(index).len(), text)
            });
        return Ok(BuiltinArgumentValueKind::Text(text));
    }
    if name == "cfg" {
        let lexed = LexedStr::new(edition, body);
        let normalized = (0..lexed.len())
            .filter(|index| !lexed.kind(*index).is_trivia())
            .map(|index| lexed.text(index))
            .collect::<Vec<_>>()
            .join(" ");
        return evaluate_cfg_expression_truth(&normalized, &input.macro_compilation_cfg_values)
            .map(BuiltinArgumentValueKind::Boolean)
            .map_err(build_builtin_argument_error);
    }
    let values = split_builtin_argument_values(body, edition)?;
    if name == "concat" {
        return values.iter().try_fold(
            BuiltinArgumentValueKind::Text(String::new()),
            |state, source| {
                let BuiltinArgumentValueKind::Text(mut text) = state else {
                    return Ok(state);
                };
                match evaluate_builtin_argument_value(source, input, lookup, cancel, depth + 1)? {
                    BuiltinArgumentValueKind::Text(value)
                    | BuiltinArgumentValueKind::Number(value) => text.push_str(&value),
                    BuiltinArgumentValueKind::Boolean(value) => text.push_str(&value.to_string()),
                    value @ BuiltinArgumentValueKind::Unavailable(..) => return Ok(value),
                    _ => return Err(build_builtin_argument_error("concat expects literals")),
                }
                Ok(BuiltinArgumentValueKind::Text(text))
            },
        );
    }
    if matches!(name, "env" | "option_env" | "include") {
        if values.is_empty() || values.len() > if name == "env" { 2 } else { 1 } {
            return Err(build_builtin_argument_error("wrong builtin argument count"));
        }
        let argument =
            evaluate_builtin_argument_value(&values[0], input, lookup, cancel, depth + 1)?;
        let key = match argument {
            BuiltinArgumentValueKind::Text(value) => value,
            value @ BuiltinArgumentValueKind::Unavailable(..) => return Ok(value),
            _ => return Err(build_builtin_argument_error("expected string literal")),
        };
        if name == "include" {
            return Ok(BuiltinArgumentValueKind::Text(key));
        }
        return match input.macro_metadata_environment_rows.get(&key) {
            Some(value) if name == "option_env" => {
                Ok(BuiltinArgumentValueKind::Optional(value.clone()))
            }
            Some(Some(value)) => Ok(BuiltinArgumentValueKind::Text(value.clone())),
            Some(None) => Err(build_builtin_argument_error(format!(
                "environment variable explicitly absent: {key}"
            ))),
            None => Ok(BuiltinArgumentValueKind::Unavailable(
                if key == "OUT_DIR" {
                    SiteUnresolvedReasonKind::BuildScript
                } else {
                    SiteUnresolvedReasonKind::ExternalSourceMissing
                },
                format!("metadata environment value unavailable: {key}"),
            )),
        };
    }
    Ok(BuiltinArgumentValueKind::Unavailable(
        SiteUnresolvedReasonKind::ExternalSourceMissing,
        format!("compiler builtin expansion unavailable: {name}"),
    ))
}

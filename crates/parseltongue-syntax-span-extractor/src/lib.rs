use parseltongue_core_row_types::{StageTableRowRecord, StoreBackendFailureError};
use ra_ap_syntax::{AstNode, AstToken, Edition, SyntaxNode, ast};
use serde_json::{Value, json};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap},
    path::{Path, PathBuf},
};

#[path = "crate-syntax-namespace-projection.rs"]
mod crate_syntax_namespace_projection;
#[path = "name-resolution-record-codec.rs"]
mod name_resolution_record_codec;
#[path = "name-resolution-stage-adapter.rs"]
mod name_resolution_stage_adapter;
#[path = "persisted-macro-expansion-adapter.rs"]
mod persisted_macro_expansion_adapter;
pub use name_resolution_stage_adapter::run_persisted_name_resolution;

fn parse_compatible_rust_syntax(
    source: &str,
    context: &str,
    edition: Edition,
) -> Result<ra_ap_syntax::Parse<SyntaxNode>, StoreBackendFailureError> {
    use ra_ap_parser::{Input, LexedStr, StrStep, TopEntryPoint};
    use ra_ap_syntax::{SyntaxKind, SyntaxTreeBuilder, TextSize};
    u32::try_from(source.len()).map_err(build_syntax_stage_failure)?;
    let lexed = LexedStr::new(edition, source);
    let (input, _) = (0..lexed.len()).fold(
        (Input::with_capacity(lexed.len()), false),
        |(mut input, was_joint), index| {
            let kind = lexed.kind(index);
            if kind.is_trivia() {
                (input, false)
            } else if kind == SyntaxKind::IDENT {
                let contextual = SyntaxKind::from_contextual_keyword(lexed.text(index), edition)
                    .unwrap_or(SyntaxKind::IDENT);
                let macro_path = (index + 1..lexed.len())
                    .find(|next| !lexed.kind(*next).is_trivia())
                    .is_some_and(|next| lexed.kind(next) == ra_ap_syntax::T![!]);
                input.push_ident(
                    if macro_path && contextual != ra_ap_syntax::T![macro_rules] {
                        SyntaxKind::IDENT
                    } else {
                        contextual
                    },
                    edition,
                );
                (input, was_joint)
            } else {
                if was_joint {
                    input.was_joint();
                }
                input.push(kind, edition);
                if kind == SyntaxKind::FLOAT_NUMBER {
                    if !lexed.text(index).ends_with('.') {
                        input.was_joint();
                        (input, was_joint)
                    } else {
                        (input, false)
                    }
                } else {
                    (input, true)
                }
            }
        },
    );
    let entry = if context == "expression" {
        TopEntryPoint::Expr
    } else {
        TopEntryPoint::SourceFile
    };
    let mut builder = SyntaxTreeBuilder::default();
    let exhausted = lexed.intersperse_trivia(&entry.parse(&input), &mut |step| match step {
        StrStep::Token { kind, text } => builder.token(kind, text),
        StrStep::Enter { kind } => builder.start_node(kind),
        StrStep::Exit => builder.finish_node(),
        StrStep::Error { msg, pos } => builder.error(msg.into(), TextSize::new(pos as u32)),
    });
    if !exhausted {
        return Err(build_syntax_stage_failure(
            "source tokens remain unconsumed",
        ));
    }
    for (index, error) in lexed.errors() {
        builder.error(error.into(), TextSize::new(lexed.text_start(index) as u32));
    }
    Ok(builder.finish())
}

fn restore_source_byte_offset(offset: ra_ap_syntax::TextSize, removed: &[usize]) -> usize {
    let normalized = u32::from(offset) as usize;
    normalized + removed.partition_point(|position| *position < normalized)
}

fn normalize_cfg_source_tokens(source: &str) -> Result<String, StoreBackendFailureError> {
    let parsed = ast::Expr::parse(&format!("cfg!({source}\n)"), Edition::Edition2024);
    if !parsed.errors().is_empty() {
        return Err(build_syntax_stage_failure(format!(
            "invalid cfg token source: {source}"
        )));
    }
    let tree = parsed
        .syntax_node()
        .descendants()
        .find_map(ast::MacroCall::cast)
        .and_then(|call| call.token_tree())
        .ok_or_else(|| build_syntax_stage_failure("cfg token tree absent"))?;
    let tokens: Vec<_> = tree
        .syntax()
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| !token.kind().is_trivia())
        .collect();
    tokens
        .get(1..tokens.len().saturating_sub(1))
        .ok_or_else(|| build_syntax_stage_failure("cfg delimiters absent"))?
        .iter()
        .try_fold(String::new(), |mut normalized, token| {
            if let Some(literal) = ast::String::cast(token.clone()) {
                let value = literal
                    .value()
                    .map_err(|error| build_syntax_stage_failure(format!("{error:?}")))?;
                normalized.push_str(&format!("{value:?}"));
            } else {
                normalized.push_str(token.text().trim_start_matches("r#"));
            }
            Ok(normalized)
        })
}

pub fn evaluate_cfg_expression_truth(
    source: &str,
    values: &BTreeSet<String>,
) -> Result<bool, StoreBackendFailureError> {
    parseltongue_core_row_types::evaluate_cfg_expression_truth(
        &normalize_cfg_source_tokens(source)?,
        values,
    )
}

pub struct SyntaxFileInputData {
    pub file_source_absolute_path: String,
    pub file_source_content_bytes: Vec<u8>,
    pub unit_compilation_identity_key: String,
    pub unit_rust_edition_value: String,
    pub unit_target_cfg_values: BTreeSet<String>,
    pub file_module_directory_path: String,
    pub unit_metadata_environment_values: BTreeMap<String, String>,
    pub file_parse_context_kind: String,
    pub file_expansion_provenance_chain: Vec<String>,
}
#[derive(Debug, Eq, PartialEq)]
pub struct SourceClosureFileRequest {
    pub source_request_path_candidates: Vec<String>,
    pub source_request_context_kind: String,
    pub source_request_origin_digest: String,
}
#[derive(Debug)]
pub struct SyntaxFileExtractionResult {
    pub stage_unit_output_rows: Vec<StageTableRowRecord>,
    pub source_closure_file_requests: Vec<SourceClosureFileRequest>,
}
fn build_syntax_stage_failure(cause: impl std::fmt::Display) -> StoreBackendFailureError {
    StoreBackendFailureError {
        error_context_detail_text: format!("WorkspaceNotCompilingError: {cause}"),
    }
}
fn split_attribute_argument_values(source: &str) -> Vec<String> {
    let mut depth = 0;
    let mut quoted = false;
    let mut escaped = false;
    let mut start = 0;
    let mut values = Vec::new();
    for (offset, ch) in source.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && quoted {
            escaped = true;
            continue;
        }
        if ch == '"' {
            quoted = !quoted;
            continue;
        }
        if quoted {
            continue;
        }
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                values.push(source[start..offset].trim().into());
                start = offset + 1;
            }
            _ => {}
        }
    }
    if !source[start..].trim().is_empty() {
        values.push(source[start..].trim().into());
    }
    values
}
fn expand_effective_attribute_values(
    source: &str,
    cfg: &BTreeSet<String>,
) -> Result<Vec<String>, StoreBackendFailureError> {
    let normalized = ["cfg", "path"]
        .iter()
        .any(|prefix| source.trim_start_matches("r#").starts_with(prefix))
        .then(|| normalize_cfg_source_tokens(source))
        .transpose()?;
    let source = normalized.as_deref().unwrap_or(source);
    if let Some(arguments) = source
        .strip_prefix("cfg_attr")
        .and_then(|s| s.trim().strip_prefix('('))
        .and_then(|s| s.strip_suffix(')'))
    {
        let arguments = split_attribute_argument_values(arguments);
        let (condition, attributes) = arguments
            .split_first()
            .ok_or_else(|| build_syntax_stage_failure("empty cfg_attr"))?;
        if evaluate_cfg_expression_truth(condition, cfg)? {
            attributes
                .iter()
                .try_fold(Vec::new(), |mut output, attribute| {
                    output.extend(expand_effective_attribute_values(attribute, cfg)?);
                    Ok(output)
                })
        } else {
            Ok(Vec::new())
        }
    } else {
        Ok(vec![source.to_owned()])
    }
}
fn collect_effective_node_attributes(
    node: &SyntaxNode,
    cfg: &BTreeSet<String>,
) -> Result<Vec<String>, StoreBackendFailureError> {
    node.children()
        .filter_map(ast::Attr::cast)
        .try_fold(Vec::new(), |mut attributes, attribute| {
            let text = attribute.syntax().text().to_string();
            let text = text
                .trim()
                .strip_prefix("#![")
                .or_else(|| text.trim().strip_prefix("#["))
                .and_then(|s| s.strip_suffix(']'))
                .ok_or_else(|| build_syntax_stage_failure("invalid attribute source"))?;
            attributes.extend(expand_effective_attribute_values(text.trim(), cfg)?);
            Ok(attributes)
        })
}
fn check_node_configuration_active(
    node: &SyntaxNode,
    cfg: &BTreeSet<String>,
) -> Result<bool, StoreBackendFailureError> {
    for attribute in collect_effective_node_attributes(node, cfg)? {
        if let Some(condition) = attribute
            .strip_prefix("cfg")
            .and_then(|s| s.trim().strip_prefix('('))
            .and_then(|s| s.strip_suffix(')'))
            && !evaluate_cfg_expression_truth(condition, cfg)?
        {
            return Ok(false);
        }
    }
    Ok(true)
}
fn decode_rust_string_literal(source: &str) -> Option<String> {
    let parse = ast::Expr::parse(source, Edition::Edition2024);
    if !parse.errors().is_empty() {
        return None;
    }
    parse
        .syntax_node()
        .descendants_with_tokens()
        .filter_map(|e| e.into_token())
        .find_map(|token| {
            ast::String::cast(token).and_then(|s| s.value().ok().map(|v| v.into_owned()))
        })
}
fn evaluate_include_argument_value(
    source: &str,
    environment: &BTreeMap<String, String>,
) -> Option<String> {
    let source = source.trim();
    if let Some(value) = decode_rust_string_literal(source)
        && (source.starts_with('"') || source.starts_with("r#") || source.starts_with("r\""))
    {
        return Some(value);
    }
    if let Some(arguments) = source
        .strip_prefix("concat!")
        .and_then(|s| s.trim().strip_prefix('('))
        .and_then(|s| s.strip_suffix(')'))
    {
        return split_attribute_argument_values(arguments)
            .iter()
            .map(|argument| evaluate_include_argument_value(argument, environment))
            .collect::<Option<Vec<_>>>()
            .map(|values| values.concat());
    }
    if let Some(arguments) = source
        .strip_prefix("env!")
        .and_then(|s| s.trim().strip_prefix('('))
        .and_then(|s| s.strip_suffix(')'))
    {
        return environment
            .get(&decode_rust_string_literal(arguments)?)
            .cloned();
    }
    None
}
fn read_effective_module_path(
    node: &SyntaxNode,
    cfg: &BTreeSet<String>,
) -> Result<Option<String>, StoreBackendFailureError> {
    Ok(collect_effective_node_attributes(node, cfg)?
        .iter()
        .find_map(|attribute| {
            attribute
                .strip_prefix("path")
                .and_then(|s| s.trim().strip_prefix('='))
                .and_then(|s| decode_rust_string_literal(s.trim()))
        }))
}
fn build_syntax_occurrence_key(
    input: &SyntaxFileInputData,
    node: &SyntaxNode,
    removed: &[usize],
) -> (String, Value) {
    let range = node.text_range();
    let canonical = json!([
        input.unit_compilation_identity_key,
        input.file_source_absolute_path,
        restore_source_byte_offset(range.start(), removed),
        restore_source_byte_offset(range.end(), removed),
        format!("{:?}", node.kind()),
        input.file_expansion_provenance_chain
    ]);
    (
        blake3::hash(canonical.to_string().as_bytes())
            .to_hex()
            .to_string(),
        canonical,
    )
}
fn check_syntax_item_node(node: &SyntaxNode) -> bool {
    if ast::Variant::can_cast(node.kind()) {
        return true;
    }
    if !ast::Item::can_cast(node.kind()) {
        return false;
    }
    if ast::MacroCall::can_cast(node.kind()) {
        return node.parent().is_some_and(|p| {
            matches!(
                format!("{:?}", p.kind()).as_str(),
                "SOURCE_FILE" | "ITEM_LIST" | "ASSOC_ITEM_LIST" | "EXTERN_ITEM_LIST" | "STMT_LIST"
            )
        });
    }
    true
}
fn build_syntax_output_record(table: &str, key: String, payload: Value) -> StageTableRowRecord {
    StageTableRowRecord {
        stage_owned_table_name: table.into(),
        stage_row_record_key: key,
        stage_row_payload_text: payload.to_string(),
    }
}
pub fn extract_file_syntax_rows(
    input: &SyntaxFileInputData,
) -> Result<SyntaxFileExtractionResult, StoreBackendFailureError> {
    extract_file_syntax_core(input, None)
}

fn extract_file_syntax_core(
    input: &SyntaxFileInputData,
    syntax_override: Option<&str>,
) -> Result<SyntaxFileExtractionResult, StoreBackendFailureError> {
    let source = std::str::from_utf8(&input.file_source_content_bytes).map_err(|error| {
        build_syntax_stage_failure(format!(
            "{} non-UTF-8 at offset {}",
            input.file_source_absolute_path,
            error.valid_up_to()
        ))
    })?;
    let edition = match input.unit_rust_edition_value.as_str() {
        "2015" => Edition::Edition2015,
        "2018" => Edition::Edition2018,
        "2021" => Edition::Edition2021,
        "2024" => Edition::Edition2024,
        _ => return Err(build_syntax_stage_failure("unknown Rust edition")),
    };
    let removed: Vec<_> = source
        .match_indices("\r\n")
        .enumerate()
        .map(|(count, (offset, _))| offset - count)
        .collect();
    let normalized = if removed.is_empty() {
        Cow::Borrowed(source)
    } else {
        Cow::Owned(source.replace("\r\n", "\n"))
    };
    let parsed = parse_compatible_rust_syntax(
        syntax_override.unwrap_or(&normalized),
        &input.file_parse_context_kind,
        edition,
    )?;
    let (root, errors) = (parsed.syntax_node(), parsed.errors());
    if let Some(error) = errors.first() {
        return Err(build_syntax_stage_failure(format!(
            "{} offset {}: {error}",
            input.file_source_absolute_path,
            restore_source_byte_offset(error.range().start(), &removed)
        )));
    }
    let mut active_flags = HashMap::new();
    for node in root.descendants() {
        let inherited = node
            .parent()
            .and_then(|p| active_flags.get(&p))
            .copied()
            .unwrap_or(true);
        let active =
            inherited && check_node_configuration_active(&node, &input.unit_target_cfg_values)?;
        active_flags.insert(node, active);
    }
    let mut rows = Vec::new();
    let mut requests = Vec::new();
    let items: Vec<_> = root.descendants().filter(check_syntax_item_node).collect();
    for node in &items {
        let (key, canonical) = build_syntax_occurrence_key(input, node, &removed);
        let active = active_flags.get(node).copied().unwrap_or(true);
        let name = node
            .children()
            .find(|child| format!("{:?}", child.kind()) == "NAME")
            .map(|n| n.text().to_string());
        let visibility = node
            .children()
            .find(|child| format!("{:?}", child.kind()) == "VISIBILITY")
            .map(|n| n.text().to_string())
            .unwrap_or_else(|| "private".into());
        let range = node.text_range();
        let start = restore_source_byte_offset(range.start(), &removed);
        let end = restore_source_byte_offset(range.end(), &removed);
        rows.push(build_syntax_output_record("span-reference-text-table",key.clone(),json!({
            "span_semantic_occurrence_key":key,"span_occurrence_canonical_key":canonical,
            "unit_compilation_identity_key":input.unit_compilation_identity_key,"file_source_absolute_path":input.file_source_absolute_path,
            "span_byte_start_offset":start,"span_byte_end_offset":end,
            "span_content_digest_full":blake3::hash(&input.file_source_content_bytes[start..end]).to_hex().to_string(),
            "span_syntax_kind_value":format!("{:?}",node.kind()),"span_declared_name_value":name,"span_visibility_source_text":visibility,
            "span_source_code_text":&source[start..end],"span_configuration_active_flag":active,
            "site_unresolved_reason_kind":if active{Value::Null}else{json!("cfg_inactive")},
            "span_expansion_provenance_chain":input.file_expansion_provenance_chain
        })));
        if !active {
            continue;
        }
        if let Some(module) = ast::Module::cast(node.clone())
            && module.semicolon_token().is_some()
        {
            use ra_ap_syntax::ast::HasName;
            let name = module
                .name()
                .ok_or_else(|| build_syntax_stage_failure("module name missing"))?
                .text()
                .trim_start_matches("r#")
                .to_owned();
            let mut base = PathBuf::from(&input.file_module_directory_path);
            let ancestors: Vec<_> = node
                .ancestors()
                .skip(1)
                .filter_map(ast::Module::cast)
                .collect();
            for ancestor in ancestors.iter().rev() {
                if let Some(path) =
                    read_effective_module_path(ancestor.syntax(), &input.unit_target_cfg_values)?
                {
                    base.push(path);
                } else if let Some(name) = ancestor.name() {
                    base.push(name.text().trim_start_matches("r#"));
                }
            }
            let candidates = if let Some(path) =
                read_effective_module_path(node, &input.unit_target_cfg_values)?
            {
                if ancestors.is_empty() {
                    base = Path::new(&input.file_source_absolute_path)
                        .parent()
                        .ok_or_else(|| build_syntax_stage_failure("module file parent absent"))?
                        .to_path_buf();
                }
                vec![base.join(path).to_string_lossy().into_owned()]
            } else {
                vec![
                    base.join(format!("{name}.rs"))
                        .to_string_lossy()
                        .into_owned(),
                    base.join(&name)
                        .join("mod.rs")
                        .to_string_lossy()
                        .into_owned(),
                ]
            };
            requests.push(SourceClosureFileRequest {
                source_request_path_candidates: candidates,
                source_request_context_kind: "items".into(),
                source_request_origin_digest: key,
            });
        }
    }
    let top_items: Vec<_> = items
        .iter()
        .filter(|node| !node.ancestors().skip(1).any(|n| check_syntax_item_node(&n)))
        .collect();
    let mut cursor = 0_usize;
    for node in top_items {
        let range = node.text_range();
        let start = restore_source_byte_offset(range.start(), &removed);
        let end = restore_source_byte_offset(range.end(), &removed);
        if start > cursor {
            rows.push(build_ownership_range_record(
                input,
                cursor,
                start,
                if source[cursor..start].trim().is_empty() {
                    "whitespace"
                } else {
                    "comment_attribute_trivia"
                },
            ));
        }
        rows.push(build_ownership_range_record(input, start, end, "item"));
        cursor = end;
    }
    if cursor < source.len() {
        rows.push(build_ownership_range_record(
            input,
            cursor,
            source.len(),
            if input.file_parse_context_kind == "expression" {
                "expression_root"
            } else if source[cursor..].trim().is_empty() {
                "whitespace"
            } else {
                "comment_attribute_trivia"
            },
        ));
    }
    for node in root.descendants() {
        if !active_flags.get(&node).copied().unwrap_or(true) {
            continue;
        }
        let kind = format!("{:?}", node.kind());
        let field = matches!(
            kind.as_str(),
            "RECORD_FIELD"
                | "TUPLE_FIELD"
                | "PARAM"
                | "SELF_PARAM"
                | "RET_TYPE"
                | "TYPE_PARAM"
                | "CONST_PARAM"
                | "LIFETIME_PARAM"
                | "WHERE_PRED"
                | "TYPE_BOUND"
                | "ATTR"
        );
        let body = matches!(
            kind.as_str(),
            "CALL_EXPR"
                | "METHOD_CALL_EXPR"
                | "PATH_EXPR"
                | "MACRO_CALL"
                | "FIELD_EXPR"
                | "RECORD_EXPR"
        );
        if !field && !body && kind != "IMPL" {
            continue;
        }
        let (key, _) = build_syntax_occurrence_key(input, &node, &removed);
        let owner = node
            .ancestors()
            .skip(1)
            .find(check_syntax_item_node)
            .map(|n| build_syntax_occurrence_key(input, &n, &removed).0);
        let range = node.text_range();
        let start = restore_source_byte_offset(range.start(), &removed);
        let end = restore_source_byte_offset(range.end(), &removed);
        let mut payload = json!({"unit_compilation_identity_key":input.unit_compilation_identity_key,
            "file_source_absolute_path":input.file_source_absolute_path,
            "span_byte_start_offset":start,"span_byte_end_offset":end,
            "site_syntax_kind_value":kind,"site_syntax_source_text":&source[start..end],
            "site_owner_occurrence_key":owner,"site_resolution_status_kind":"pending",
            "site_unresolved_reason_kind":Value::Null,"span_expansion_provenance_chain":input.file_expansion_provenance_chain});
        if let Some(call) = ast::MacroCall::cast(node.clone()) {
            let name = call
                .path()
                .map(|p| p.syntax().text().to_string())
                .unwrap_or_default();
            if name.split("::").last() == Some("include") {
                let tokens = call
                    .token_tree()
                    .map(|t| t.syntax().text().to_string())
                    .unwrap_or_default();
                let arguments = tokens.get(1..tokens.len().saturating_sub(1)).unwrap_or("");
                if let Some(path) = evaluate_include_argument_value(
                    arguments,
                    &input.unit_metadata_environment_values,
                ) {
                    let parent = Path::new(&input.file_source_absolute_path)
                        .parent()
                        .ok_or_else(|| build_syntax_stage_failure("include parent absent"))?;
                    let context = if node
                        .parent()
                        .is_some_and(|p| format!("{:?}", p.kind()) == "MACRO_EXPR")
                    {
                        "expression"
                    } else {
                        "items"
                    };
                    requests.push(SourceClosureFileRequest {
                        source_request_path_candidates: vec![
                            parent.join(path).to_string_lossy().into_owned(),
                        ],
                        source_request_context_kind: context.into(),
                        source_request_origin_digest: key.clone(),
                    });
                } else {
                    payload["site_unresolved_reason_kind"] = json!("build_script");
                    payload["site_resolution_status_kind"] = json!("unresolved");
                }
            }
        }
        rows.push(build_syntax_output_record(
            if body {
                "body-site-reference-table"
            } else {
                "field-signature-detail-table"
            },
            key,
            payload,
        ));
    }
    for request in &requests {
        rows.push(build_syntax_output_record(
            "module-source-closure-requests",
            request.source_request_origin_digest.clone(),
            json!({
                "source_request_path_candidates":request.source_request_path_candidates,
                "source_request_context_kind":request.source_request_context_kind,
                "source_request_origin_digest":request.source_request_origin_digest,
                "unit_compilation_identity_key":input.unit_compilation_identity_key,
                "file_expansion_provenance_chain":input.file_expansion_provenance_chain
            }),
        ));
    }
    Ok(SyntaxFileExtractionResult {
        stage_unit_output_rows: rows,
        source_closure_file_requests: requests,
    })
}
fn build_ownership_range_record(
    input: &SyntaxFileInputData,
    start: usize,
    end: usize,
    kind: &str,
) -> StageTableRowRecord {
    let key = blake3::hash(
        json!([
            input.unit_compilation_identity_key,
            input.file_source_absolute_path,
            input.file_expansion_provenance_chain,
            start,
            end,
            "ownership"
        ])
        .to_string()
        .as_bytes(),
    )
    .to_hex()
    .to_string();
    build_syntax_output_record(
        "span-byte-ownership-ranges",
        key,
        json!({"unit_compilation_identity_key":input.unit_compilation_identity_key,
        "file_source_absolute_path":input.file_source_absolute_path,"span_byte_start_offset":start,"span_byte_end_offset":end,
        "span_gap_classification_kind":kind,"span_expansion_provenance_chain":input.file_expansion_provenance_chain}),
    )
}

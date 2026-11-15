//! REQ-RUST-050.0: the display label of a full graph node, copied from stored source text.
use super::*;

#[cfg(test)]
#[path = "item-node-label-tests.rs"]
mod item_node_label_tests;

const LABEL_TEXT_CHARACTER_LIMIT: usize = 160;

/// L6: one line of source text, bounded.
pub(super) fn normalize_label_source_text(text: &str) -> String {
    let joined = text
        .lines()
        .skip_while(|line| line.trim_start().starts_with("//"))
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" ::", "::")
        .replace(":: ", "::");
    let line = joined.strip_suffix(';').unwrap_or(&joined).trim_end();
    if line.chars().count() > LABEL_TEXT_CHARACTER_LIMIT {
        let kept = line
            .chars()
            .take(LABEL_TEXT_CHARACTER_LIMIT)
            .collect::<String>();
        format!("{kept} …")
    } else {
        line.to_owned()
    }
}

/// L2: `<SelfText as TraitText>` or `SelfText`, from the impl header's own source text.
pub(super) fn build_impl_header_segment(header: &Value) -> String {
    let read_header_type_text = |key: &str| {
        header[key]["type_source_evidence"]["type_source_text"]
            .as_str()
            .map(normalize_label_source_text)
            .filter(|text| !text.is_empty())
    };
    let trait_written = !header["impl_trait_type_resolution"].is_null();
    match (
        read_header_type_text("impl_self_type_resolution"),
        read_header_type_text("impl_trait_type_resolution"),
    ) {
        (Some(self_text), Some(trait_text)) => format!("<{self_text} as {trait_text}>"),
        (Some(self_text), None) if !trait_written => self_text,
        _ => "IMPL".into(),
    }
}

/// L3, L4, L5, L7: the last segment of an item's label.
pub(super) fn build_item_name_segment(span: &Value, call_span: Option<&Value>) -> String {
    let kind = span["span_syntax_kind_value"].as_str().unwrap_or_default();
    let read_span_label_text = |span: &Value| {
        Some(normalize_label_source_text(
            span["span_source_code_text"].as_str().unwrap_or_default(),
        ))
        .filter(|text| !text.is_empty())
    };
    let name = span["span_declared_name_value"]
        .as_str()
        .map(|name| name.trim_start_matches("r#"))
        .filter(|name| !name.is_empty() && *name != "_");
    match (kind, name) {
        ("MACRO_CALL" | "USE", _) => read_span_label_text(span).unwrap_or_else(|| kind.into()),
        (_, Some(name)) => name.into(),
        ("CONST", None) => call_span
            .and_then(read_span_label_text)
            .map_or_else(|| "const _".into(), |call| format!("const _ #{call}")),
        _ => kind.into(),
    }
}

/// L1: crate target name, file path, enclosing names, impl header, last segment.
pub(super) fn join_node_label_segments(
    unit: &CrateCompilationUnitKey,
    path: &str,
    names: &[String],
    impl_header: Option<&Value>,
    last: &str,
) -> String {
    [unit.unit_cargo_target_name.as_str(), path]
        .into_iter()
        .map(str::to_owned)
        .chain(names.iter().cloned())
        .chain(impl_header.map(build_impl_header_segment))
        .chain(std::iter::once(last.to_owned()))
        .collect::<Vec<_>>()
        .join("::")
}

use parseltongue_core_row_types::StoreBackendFailureError;
use ra_ap_syntax::{AstNode, SyntaxKind, SyntaxNode, T, ast};
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub struct LegacyMacroNodeEvents {
    pub before_node_rows: Vec<Value>,
    pub after_node_rows: Vec<Value>,
}

pub fn read_legacy_node_events(
    node: &SyntaxNode,
    prefix: &[u64],
    unit: &Value,
    cfg: &BTreeSet<String>,
    read_key: &dyn Fn(&SyntaxNode, bool) -> Result<Value, StoreBackendFailureError>,
) -> Result<LegacyMacroNodeEvents, StoreBackendFailureError> {
    let mut events = LegacyMacroNodeEvents {
        before_node_rows: Vec::new(),
        after_node_rows: Vec::new(),
    };
    if !super::check_node_configuration_active(node, cfg)? {
        return Ok(events);
    }
    let start = u32::from(node.text_range().start()) as u64;
    let event = |position,
                 phase,
                 ordinal,
                 kind: &str,
                 name: &str,
                 site: Value,
                 target: Value,
                 export| {
        let order = prefix
            .iter()
            .copied()
            .chain([position, phase, ordinal])
            .collect::<Vec<_>>();
        json!({"legacy_source_unit_key":unit,"legacy_source_order_parts":order,"legacy_event_kind_value":kind,
            "legacy_macro_name_text":name,"legacy_source_site_key":site,"legacy_target_identity_key":target,
            "legacy_module_export_flag":export})
    };
    if matches!(node.kind(), SyntaxKind::MODULE | SyntaxKind::BLOCK_EXPR) {
        let export = node.kind() == SyntaxKind::MODULE
            && super::collect_effective_node_attributes(node, cfg)?
                .iter()
                .any(|attribute| attribute == "macro_use" || attribute.starts_with("macro_use("));
        events.before_node_rows.push(event(
            start,
            1,
            0,
            "enter",
            "",
            Value::Null,
            Value::Null,
            export,
        ));
        events.after_node_rows.push(event(
            u32::from(node.text_range().end()) as u64,
            0,
            0,
            "exit",
            "",
            Value::Null,
            Value::Null,
            false,
        ));
    }
    if node.kind() == SyntaxKind::MACRO_RULES
        && let Some(name) = node.children().find_map(ast::Name::cast)
    {
        events.before_node_rows.push(event(
            start,
            2,
            0,
            "definition",
            name.text(),
            Value::Null,
            read_key(node, true)?,
            false,
        ));
    }
    if ast::MacroCall::can_cast(node.kind()) {
        let names = node
            .descendants_with_tokens()
            .filter_map(|element| element.into_token())
            .filter(|token| !token.kind().is_trivia())
            .scan(None::<String>, |previous, token| {
                let name = if token.kind() == T![!] {
                    previous.take()
                } else {
                    None
                };
                *previous = (token.kind() == SyntaxKind::IDENT)
                    .then(|| token.text().trim_start_matches("r#").to_owned());
                Some(name)
            })
            .flatten()
            .collect::<BTreeSet<_>>();
        if !names.is_empty() {
            let site = read_key(node, false)?;
            events
                .before_node_rows
                .extend(names.into_iter().enumerate().map(|(index, name)| {
                    event(
                        start,
                        4,
                        index as u64,
                        "request",
                        &name,
                        site.clone(),
                        Value::Null,
                        false,
                    )
                }));
        }
    }
    let import = ast::UseTree::cast(node.clone())
        .filter(|tree| {
            tree.use_tree_list().is_none()
                && tree.star_token().is_none()
                && !tree
                    .syntax()
                    .ancestors()
                    .skip(1)
                    .filter_map(ast::UseTree::cast)
                    .any(|parent| parent.path().is_some())
        })
        .and_then(|tree| tree.path());
    let macro_path = ast::Path::cast(node.clone()).filter(|path| {
        path.syntax()
            .parent()
            .is_some_and(|parent| ast::MacroCall::can_cast(parent.kind()))
    });
    if let Some(path) = import
        .or(macro_path)
        .filter(|path| path.qualifier().is_none())
        && let Some(name) = path.segment().and_then(|segment| segment.name_ref())
    {
        events.before_node_rows.push(event(
            start,
            3,
            0,
            "request",
            name.text(),
            read_key(node, false)?,
            Value::Null,
            false,
        ));
    }
    Ok(events)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use ra_ap_syntax::{Edition, SyntaxKind};
    use serde_json::json;

    fn collect(source: &str, prefix: &[u64]) -> Vec<Value> {
        let parsed = ra_ap_syntax::SourceFile::parse(source, Edition::Edition2024);
        assert!(parsed.errors().is_empty());
        fn walk(node: &SyntaxNode, prefix: &[u64], rows: &mut Vec<Value>) {
            if !crate::check_node_configuration_active(node, &BTreeSet::new()).unwrap() {
                return;
            }
            let events=read_legacy_node_events(node,prefix,&json!("unit"),&BTreeSet::new(),&|node,definition|Ok(json!({
                "start":u32::from(node.text_range().start()),"kind":format!("{:?}",node.kind()),"prefix":prefix,"definition":definition}))).unwrap();
            rows.extend(events.before_node_rows);
            node.children().for_each(|child| walk(&child, prefix, rows));
            rows.extend(events.after_node_rows);
        }
        let mut rows = Vec::new();
        walk(&parsed.syntax_node(), prefix, &mut rows);
        rows
    }
    fn requests(events: &[Value]) -> Vec<Value> {
        let mut rows = Vec::new();
        crate::visit_legacy_macro_scope_records(events, &mut |row| {
            let value: Value = serde_json::from_str(&row.stage_row_payload_text).unwrap();
            if value["legacy_source_site_key"]["kind"] == format!("{:?}", SyntaxKind::MACRO_CALL) {
                rows.push(value);
            }
            Ok(())
        })
        .unwrap();
        rows
    }
    #[test]
    fn syntax_legacy_macro_order() {
        let rows = requests(&collect(
            "m!(); macro_rules! m {()=>{}} m!(); fn f(){macro_rules! m {()=>{}} m!();} m!();",
            &[],
        ));
        assert_eq!(rows.len(), 4);
        assert!(rows[0]["legacy_target_identity_key"].is_null());
        assert!(!rows[1]["legacy_target_identity_key"].is_null());
        assert_ne!(
            rows[1]["legacy_target_identity_key"],
            rows[2]["legacy_target_identity_key"]
        );
        assert_eq!(
            rows[1]["legacy_target_identity_key"],
            rows[3]["legacy_target_identity_key"]
        );
    }
    #[test]
    fn syntax_macro_use_modules() {
        let rows = requests(&collect(
            "m!(); # [macro_use] mod exported { macro_rules! m {()=>{}} } m!(); mod private {macro_rules! m {()=>{}}} m!();",
            &[],
        ));
        assert_eq!(rows.len(), 3);
        assert!(rows[0]["legacy_target_identity_key"].is_null());
        assert!(!rows[1]["legacy_target_identity_key"].is_null());
        assert_eq!(
            rows[1]["legacy_target_identity_key"],
            rows[2]["legacy_target_identity_key"]
        );
    }
    #[test]
    fn syntax_nested_builtin_shadow() {
        let rows = requests(&collect(
            "macro_rules! r#concat {()=>{}} include!(r#concat!(\"stringify!()\",stringify!(x)));",
            &[],
        ));
        let names = rows
            .iter()
            .map(|row| row["legacy_macro_name_text"].as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(names, BTreeSet::from(["concat", "include", "stringify"]));
        assert!(
            !rows
                .iter()
                .find(|row| row["legacy_macro_name_text"] == "concat")
                .unwrap()["legacy_target_identity_key"]
                .is_null()
        );
    }
    #[test]
    fn syntax_virtual_expansion_order() {
        let mut events = collect("make!(); generated!();", &[]);
        events.extend(collect("macro_rules! generated {()=>{}}", &[0, 5]));
        let rows = requests(&events);
        assert_eq!(rows.len(), 2);
        assert!(!rows[1]["legacy_target_identity_key"].is_null());
        assert_eq!(
            rows[1]["legacy_target_identity_key"]["prefix"],
            json!([0, 5])
        );
    }
    #[test]
    fn syntax_legacy_empty_errors() {
        assert!(collect("", &[]).is_empty());
        assert!(
            requests(&collect(
                "#[cfg(any())] mod skipped {macro_rules! m {()=>{}}m!();}",
                &[]
            ))
            .is_empty()
        );
        let parsed = ra_ap_syntax::SourceFile::parse("m!();", Edition::Edition2024);
        let call = parsed
            .syntax_node()
            .descendants()
            .find(|node| node.kind() == SyntaxKind::MACRO_CALL)
            .unwrap();
        assert!(
            read_legacy_node_events(&call, &[], &json!("unit"), &BTreeSet::new(), &|_, _| Err(
                StoreBackendFailureError {
                    error_context_detail_text: "source key failed".into()
                }
            ))
            .is_err()
        );
    }
}

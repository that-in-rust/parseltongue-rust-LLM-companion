use parseltongue_core_row_types::StoreBackendFailureError;
use ra_ap_syntax::SyntaxNode;
use serde_json::Value;
use std::collections::BTreeSet;

pub struct LegacyMacroNodeEvents {
    pub before_node_rows: Vec<Value>,
    pub after_node_rows: Vec<Value>,
}

pub fn read_legacy_node_events(
    _node: &SyntaxNode,
    _prefix: &[u64],
    _unit: &Value,
    _cfg: &BTreeSet<String>,
    _read_key: &dyn Fn(&SyntaxNode, bool) -> Result<Value, StoreBackendFailureError>,
) -> Result<LegacyMacroNodeEvents, StoreBackendFailureError> {
    todo!("ordered legacy macro source events")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use ra_ap_syntax::{AstNode, Edition, SyntaxKind};
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
        walk(&parsed.syntax(), prefix, &mut rows);
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
            .syntax()
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

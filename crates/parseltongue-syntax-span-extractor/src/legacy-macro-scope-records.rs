use parseltongue_core_row_types::{StageTableRowRecord, StoreBackendFailureError};
use serde_json::Value;

pub fn visit_legacy_macro_scope_records(
    _events: &[Value],
    _visitor: &mut dyn FnMut(StageTableRowRecord) -> Result<(), StoreBackendFailureError>,
) -> Result<usize, StoreBackendFailureError> {
    todo!("ordered textual macro scope records")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use serde_json::json;

    fn event(order: &[u64], kind: &str, name: &str, key: &str) -> Value {
        json!({"legacy_source_unit_key":"unit","legacy_source_order_parts":order,
            "legacy_event_kind_value":kind,"legacy_macro_name_text":name,
            "legacy_source_site_key":key,"legacy_target_identity_key":key,
            "legacy_module_export_flag":false})
    }
    fn read(events: &[Value]) -> Result<Vec<Value>, StoreBackendFailureError> {
        let mut rows = Vec::new();
        visit_legacy_macro_scope_records(events, &mut |row| {
            assert_eq!(row.stage_owned_table_name, "legacy-macro-scope-records");
            rows.push(serde_json::from_str(&row.stage_row_payload_text).unwrap());
            Ok(())
        })?;
        Ok(rows)
    }
    fn targets(rows: &[Value]) -> Vec<Value> {
        rows.iter()
            .map(|row| row["legacy_target_identity_key"].clone())
            .collect()
    }

    #[test]
    fn legacy_macro_textual_order() {
        let events = vec![
            event(&[5], "request", "m", "before"),
            event(&[10], "definition", "m", "first"),
            event(&[15], "request", "m", "after"),
            event(&[20], "enter", "", ""),
            event(&[25], "definition", "m", "inner"),
            event(&[26], "request", "m", "nested"),
            event(&[27], "enter", "", ""),
            event(&[28], "definition", "m", "deep"),
            event(&[29], "exit", "", ""),
            event(&[30], "request", "m", "restored"),
            event(&[31], "exit", "", ""),
            event(&[32], "request", "m", "outer"),
        ];
        let expected = read(&events).unwrap();
        assert_eq!(
            targets(&expected),
            vec![
                Value::Null,
                json!("first"),
                json!("inner"),
                json!("inner"),
                json!("first")
            ]
        );
        assert_eq!(
            read(&events.into_iter().rev().collect::<Vec<_>>()).unwrap(),
            expected
        );
    }

    #[test]
    fn macro_use_module_order() {
        let mut enter = event(&[10], "enter", "", "");
        enter["legacy_module_export_flag"] = json!(true);
        let events = vec![
            event(&[9], "request", "m", "before"),
            enter,
            event(&[11], "definition", "m", "exported"),
            event(&[12], "enter", "", ""),
            event(&[13], "definition", "hidden", "hidden"),
            event(&[14], "exit", "", ""),
            event(&[16], "exit", "", ""),
            event(&[17], "request", "m", "after"),
            event(&[18], "request", "hidden", "hidden-after"),
            event(&[20], "enter", "", ""),
            event(&[21], "definition", "m", "private"),
            event(&[23], "exit", "", ""),
            event(&[24], "request", "m", "outer"),
        ];
        assert_eq!(
            targets(&read(&events).unwrap()),
            vec![
                Value::Null,
                json!("exported"),
                Value::Null,
                json!("exported")
            ]
        );
    }

    #[test]
    fn spliced_expansion_scope_order() {
        let events = vec![
            event(&[10, 4], "request", "generated", "initial"),
            event(&[10, 4, 1, 0, 2], "definition", "generated", "expanded"),
            event(&[10, 4, 1, 1, 4], "request", "generated", "chain"),
            event(&[20, 4], "request", "generated", "following"),
            event(&[30, 2], "definition", "generated", "later"),
            event(&[40, 4], "request", "generated", "last"),
        ];
        assert_eq!(
            targets(&read(&events).unwrap()),
            vec![
                Value::Null,
                json!("expanded"),
                json!("expanded"),
                json!("later")
            ]
        );
    }

    #[test]
    fn legacy_macro_scope_empty() {
        assert!(read(&[]).unwrap().is_empty());
        assert!(
            read(&[event(&[0], "enter", "", ""), event(&[1], "exit", "", "")])
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn legacy_macro_scope_errors() {
        for events in [
            vec![event(&[0], "exit", "", "")],
            vec![event(&[0], "enter", "", "")],
            vec![event(&[0], "unknown", "", "")],
            vec![json!({})],
            vec![
                event(&[0], "definition", "m", "a"),
                event(&[0], "definition", "m", "b"),
            ],
        ] {
            assert!(read(&events).is_err());
        }
        let error =
            visit_legacy_macro_scope_records(&[event(&[0], "request", "m", "site")], &mut |_| {
                Err(StoreBackendFailureError {
                    error_context_detail_text: "visitor failed".into(),
                })
            })
            .unwrap_err();
        assert!(error.to_string().contains("visitor failed"));
    }
}

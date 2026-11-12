use parseltongue_core_row_types::{StageTableRowRecord, StoreBackendFailureError};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn legacy_macro_scope_error(message: &str) -> StoreBackendFailureError {
    StoreBackendFailureError {
        error_context_detail_text: format!("legacy macro scope: {message}"),
    }
}

struct LegacyMacroScopeState {
    source_unit_key: Option<String>,
    scope_frames: Vec<(BTreeMap<String, Value>, bool)>,
}

impl LegacyMacroScopeState {
    fn finish_unit(&self) -> Result<(), StoreBackendFailureError> {
        if self.scope_frames.len() == 1 {
            Ok(())
        } else {
            Err(legacy_macro_scope_error("unclosed source scope"))
        }
    }

    fn read_event(
        &mut self,
        unit: &str,
        event: &Value,
    ) -> Result<Option<StageTableRowRecord>, StoreBackendFailureError> {
        if self.source_unit_key.as_deref() != Some(unit) {
            self.finish_unit()?;
            self.source_unit_key = Some(unit.into());
            self.scope_frames = vec![(BTreeMap::new(), false)];
        }
        match event["legacy_event_kind_value"].as_str() {
            Some("enter") => {
                let export = event["legacy_module_export_flag"]
                    .as_bool()
                    .ok_or_else(|| legacy_macro_scope_error("module export flag absent"))?;
                self.scope_frames.push((BTreeMap::new(), export));
            }
            Some("exit") => {
                if self.scope_frames.len() == 1 {
                    return Err(legacy_macro_scope_error("unmatched source scope exit"));
                }
                let (definitions, export) = self
                    .scope_frames
                    .pop()
                    .ok_or_else(|| legacy_macro_scope_error("source scope absent"))?;
                if export {
                    self.scope_frames
                        .last_mut()
                        .ok_or_else(|| legacy_macro_scope_error("parent source scope absent"))?
                        .0
                        .extend(definitions);
                }
            }
            Some("definition" | "request") => {
                let name = event["legacy_macro_name_text"]
                    .as_str()
                    .filter(|name| !name.is_empty())
                    .ok_or_else(|| legacy_macro_scope_error("macro name absent"))?
                    .trim_start_matches("r#");
                if event["legacy_event_kind_value"] == "definition" {
                    let target = &event["legacy_target_identity_key"];
                    if target.is_null() {
                        return Err(legacy_macro_scope_error("macro definition target absent"));
                    }
                    self.scope_frames
                        .last_mut()
                        .ok_or_else(|| legacy_macro_scope_error("source scope absent"))?
                        .0
                        .insert(name.into(), target.clone());
                } else {
                    let site = &event["legacy_source_site_key"];
                    if site.is_null() {
                        return Err(legacy_macro_scope_error("source request key absent"));
                    }
                    let target = self
                        .scope_frames
                        .iter()
                        .rev()
                        .find_map(|(definitions, _)| definitions.get(name))
                        .cloned()
                        .unwrap_or(Value::Null);
                    let value = json!({"legacy_source_unit_key":event["legacy_source_unit_key"],
                        "legacy_source_order_parts":event["legacy_source_order_parts"],"legacy_source_site_key":site,
                        "legacy_macro_name_text":name,"legacy_target_identity_key":target});
                    let payload = value.to_string();
                    return Ok(Some(StageTableRowRecord {
                        stage_owned_table_name: "legacy-macro-scope-records".into(),
                        stage_row_record_key: blake3::hash(payload.as_bytes()).to_hex().to_string(),
                        stage_row_payload_text: payload,
                    }));
                }
            }
            _ => return Err(legacy_macro_scope_error("unknown source event kind")),
        }
        Ok(None)
    }
}

pub fn visit_legacy_macro_scope_records(
    events: &[Value],
    visitor: &mut dyn FnMut(StageTableRowRecord) -> Result<(), StoreBackendFailureError>,
) -> Result<usize, StoreBackendFailureError> {
    let ordered = events
        .iter()
        .try_fold(BTreeMap::new(), |mut ordered, event| {
            let unit = &event["legacy_source_unit_key"];
            if unit.is_null() {
                return Err(legacy_macro_scope_error("compilation unit absent"));
            }
            let order = event["legacy_source_order_parts"]
                .as_array()
                .filter(|order| !order.is_empty())
                .ok_or_else(|| legacy_macro_scope_error("source order absent"))?
                .iter()
                .map(|value| {
                    value
                        .as_u64()
                        .ok_or_else(|| legacy_macro_scope_error("invalid source order"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            if let Some(previous) = ordered.insert((unit.to_string(), order), event)
                && previous != event
            {
                return Err(legacy_macro_scope_error("conflicting source order"));
            }
            Ok(ordered)
        })?;
    let mut state = LegacyMacroScopeState {
        source_unit_key: None,
        scope_frames: vec![(BTreeMap::new(), false)],
    };
    let count = ordered
        .into_iter()
        .try_fold(0, |count, ((unit, _), event)| {
            if let Some(row) = state.read_event(&unit, event)? {
                visitor(row)?;
                Ok(count + 1)
            } else {
                Ok(count)
            }
        })?;
    state.finish_unit()?;
    Ok(count)
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

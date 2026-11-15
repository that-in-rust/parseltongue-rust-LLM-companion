use crate::impl_header_source_records::{
    ImplHeaderSourceContext, ImplHeaderSourceError, read_impl_stage_values,
};
use parseltongue_core_row_types::StageRunScopeValue;
use parseltongue_table_store_traits::StageTableStorePort;
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[cfg(test)]
#[path = "expanded-site-origin-tests.rs"]
mod expanded_site_origin_tests;

/// For each expanded text, by its path: where every output token was copied
/// from in a source file, in output order.
#[derive(Debug, Default)]
pub(super) struct ExpandedTokenOriginIndex {
    pub(super) path_token_origin_rows: BTreeMap<String, Vec<Value>>,
}

/// Reads the stored token origins of every expansion in the run.
pub(super) fn read_expanded_token_origins(
    store: &dyn StageTableStorePort,
    scope: &StageRunScopeValue,
) -> Result<ExpandedTokenOriginIndex, ImplHeaderSourceError> {
    let invalid = |field| ImplHeaderSourceError::InvalidField {
        table: "macro-expanded-item-table",
        field,
    };
    read_impl_stage_values(store, scope, "macro-expanded-item-table")?
        .into_iter()
        .filter(|row| !row["macro_virtual_source_path"].is_null())
        .map(|mut row| {
            let path = row["macro_virtual_source_path"]
                .as_str()
                .ok_or(invalid("macro_virtual_source_path"))?
                .to_owned();
            match row["macro_token_origin_rows"].take() {
                Value::Array(tokens) => Ok((path, tokens)),
                _ => Err(invalid("macro_token_origin_rows")),
            }
        })
        .collect::<Result<_, _>>()
        .map(|path_token_origin_rows| ExpandedTokenOriginIndex {
            path_token_origin_rows,
        })
}

/// The output byte range of one stored token origin row.
fn read_token_output_range(token: &Value) -> Result<(u64, u64), ImplHeaderSourceError> {
    let range = &token["token_output_byte_range"];
    range[0]
        .as_u64()
        .zip(range[1].as_u64())
        .ok_or(ImplHeaderSourceError::InvalidField {
            table: "macro-expanded-item-table",
            field: "token_output_byte_range",
        })
}

impl ExpandedTokenOriginIndex {
    /// The first and the last output token inside a site of one expanded text.
    /// A site with no stored expansion, with no token inside it, or whose edge
    /// cuts a token is rejected.
    pub(super) fn locate_site_token_pair(
        &self,
        path: &str,
        start: u64,
        end: u64,
    ) -> Result<(&Value, &Value), ImplHeaderSourceError> {
        let missing = || ImplHeaderSourceError::InvalidReference {
            key: format!("{path}:{start}..{end}"),
        };
        let inside = self
            .path_token_origin_rows
            .get(path)
            .ok_or_else(missing)?
            .iter()
            .map(|token| read_token_output_range(token).map(|range| (range, token)))
            .filter(|token| {
                token
                    .as_ref()
                    .map_or(true, |((first, last), _)| *first < end && start < *last)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if inside
            .iter()
            .any(|((first, last), _)| *first < start || end < *last)
        {
            return Err(missing());
        }
        inside
            .iter()
            .min_by_key(|(range, _)| *range)
            .zip(inside.iter().max_by_key(|(range, _)| *range))
            .map(|((_, first), (_, last))| (*first, *last))
            .ok_or_else(missing)
    }

    /// Where a site in expanded text was copied from: its first and its last
    /// token, each with its source file, byte range and origin kind, stored
    /// apart. `Null` for a site in original text.
    pub(super) fn build_site_origin_value(
        &self,
        site: &Value,
        context: &ImplHeaderSourceContext<'_>,
    ) -> Result<Value, ImplHeaderSourceError> {
        let invalid = |field| ImplHeaderSourceError::InvalidField {
            table: "body-site-reference-table",
            field,
        };
        if site["span_expansion_provenance_chain"]
            .as_array()
            .ok_or(invalid("span_expansion_provenance_chain"))?
            .is_empty()
        {
            return Ok(Value::Null);
        }
        let text = |field| site[field].as_str().ok_or(invalid(field));
        let offset = |field| site[field].as_u64().ok_or(invalid(field));
        let unit = text("unit_compilation_identity_key")?;
        let (first, last) = self.locate_site_token_pair(
            text("file_source_absolute_path")?,
            offset("span_byte_start_offset")?,
            offset("span_byte_end_offset")?,
        )?;
        let describe = |token: &Value| {
            let absolute = token["token_source_absolute_path"].as_str().ok_or(
                ImplHeaderSourceError::InvalidField {
                    table: "macro-expanded-item-table",
                    field: "token_source_absolute_path",
                },
            )?;
            Ok::<_, ImplHeaderSourceError>(json!({
                "token_source_relative_path":context.relate_unit_source_path(unit, absolute)?,
                "token_source_byte_range":token["token_original_byte_range"],
                "token_source_origin_kind":token["token_source_origin_kind"]
            }))
        };
        Ok(json!({
            "site_first_token_origin":describe(first)?,
            "site_last_token_origin":describe(last)?
        }))
    }
}

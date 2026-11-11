//! Scope filtering utilities for HTTP endpoints
//!
//! # 4-Word Naming: scope_filter_utilities_module
//!
//! Provides helper functions for parsing and validating ?scope=<L1>||<L2> parameters.

use std::sync::Arc;
use parseltongue_core::storage::CozoDbStorage;

/// Parse scope parameter and build Datalog filter clause
///
/// # 4-Word Name: parse_scope_build_filter_clause
///
/// # Arguments
/// * `scope` - Optional scope string like "src||core" or "src"
///
/// # Returns
/// * Tuple of (filter_clause, empty_string_for_error)
/// * If scope is invalid, both strings will be empty
///
/// # Examples
/// - `Some("src||core")` -> `", root_subfolder_L1 = 'src', root_subfolder_L2 = 'core'"`
/// - `Some("src")` -> `", root_subfolder_L1 = 'src'"`
/// - `None` -> `""`
pub fn parse_scope_build_filter_clause(scope: &Option<String>) -> String {
    match scope {
        None => String::new(),
        Some(s) if s.trim().is_empty() => String::new(),
        Some(s) => {
            let parts: Vec<&str> = s.split("||").collect();
            let l1 = parts.first().map(|s| s.trim()).unwrap_or("");
            let l2 = parts.get(1).map(|s| s.trim()).unwrap_or("");

            let mut filter = String::new();
            if !l1.is_empty() {
                filter.push_str(&format!(", root_subfolder_L1 = '{}'", escape_single_quotes(l1)));
            }
            if !l2.is_empty() {
                filter.push_str(&format!(", root_subfolder_L2 = '{}'", escape_single_quotes(l2)));
            }
            filter
        }
    }
}

/// Validate scope parameter exists in database
///
/// # 4-Word Name: validate_scope_exists_in_database
///
/// # Arguments
/// * `storage` - Database storage reference
/// * `scope` - Scope string to validate
///
/// # Returns
/// * `Ok(())` if scope exists or is None/empty
/// * `Err((error_message, did_you_mean_suggestions))` if scope invalid
pub async fn validate_scope_exists_in_database(
    storage: &Arc<CozoDbStorage>,
    scope: &Option<String>,
) -> Result<(), (String, Vec<String>)> {
    // If no scope provided, validation passes
    let scope_str = match scope {
        None => return Ok(()),
        Some(s) if s.trim().is_empty() => return Ok(()),
        Some(s) => s,
    };

    // Parse scope into L1 and L2
    let parts: Vec<&str> = scope_str.split("||").collect();
    let l1 = parts.first().map(|s| s.trim()).filter(|s| !s.is_empty());
    let l2 = parts.get(1).map(|s| s.trim()).filter(|s| !s.is_empty());

    // Query database to check if this scope exists
    let query = build_scope_existence_query(l1, l2);
    let result = storage.raw_query(&query).await;

    match result {
        Ok(named_rows) if !named_rows.rows.is_empty() => Ok(()),
        Ok(_) | Err(_) => {
            // Scope doesn't exist - fetch suggestions
            let suggestions = fetch_scope_suggestions_from_database(storage, l1, l2).await;
            let error_message = format!("No entities found for scope '{}'", scope_str);
            Err((error_message, suggestions))
        }
    }
}

/// Build query to check scope existence
///
/// # 4-Word Name: build_scope_existence_query
fn build_scope_existence_query(l1: Option<&str>, l2: Option<&str>) -> String {
    let mut conditions = vec!["*CodeGraph{ISGL1_key: entity, root_subfolder_L1: l1, root_subfolder_L2: l2}".to_string()];

    if let Some(l1_val) = l1 {
        conditions.push(format!("l1 = '{}'", escape_single_quotes(l1_val)));
    }
    if let Some(l2_val) = l2 {
        conditions.push(format!("l2 = '{}'", escape_single_quotes(l2_val)));
    }

    format!("?[entity] := {}", conditions.join(", "))
}

/// Fetch scope suggestions from database
///
/// # 4-Word Name: fetch_scope_suggestions_from_database
///
/// Returns suggestions filtered by same starting letter as the invalid scope.
async fn fetch_scope_suggestions_from_database(
    storage: &Arc<CozoDbStorage>,
    l1: Option<&str>,
    _l2: Option<&str>,
) -> Vec<String> {
    // Query all unique L1/L2 combinations
    let query = "?[l1, l2] := *CodeGraph{root_subfolder_L1: l1, root_subfolder_L2: l2}";
    let result = storage.raw_query(query).await;

    let all_scopes: Vec<(String, String)> = match result {
        Ok(named_rows) => named_rows
            .rows
            .iter()
            .filter_map(|row| {
                if row.len() >= 2 {
                    let l1_str = extract_string_from_datavalue(&row[0]);
                    let l2_str = extract_string_from_datavalue(&row[1]);
                    Some((l1_str, l2_str))
                } else {
                    None
                }
            })
            .collect(),
        Err(_) => return Vec::new(),
    };

    // Filter suggestions by starting letter
    let mut suggestions: Vec<String> = all_scopes
        .into_iter()
        .filter_map(|(scope_l1, scope_l2)| {
            // Match first letter of L1 if provided
            if let Some(input_l1) = l1 {
                if let (Some(input_first), Some(scope_first)) = (
                    input_l1.chars().next(),
                    scope_l1.chars().next(),
                ) {
                    if input_first.to_lowercase().to_string()
                        != scope_first.to_lowercase().to_string()
                    {
                        return None;
                    }
                }
            }

            // Build suggestion string
            if scope_l2.is_empty() {
                Some(scope_l1)
            } else {
                Some(format!("{}||{}", scope_l1, scope_l2))
            }
        })
        .collect();

    // Deduplicate and sort
    suggestions.sort();
    suggestions.dedup();

    // Limit to 10 suggestions
    suggestions.truncate(10);

    suggestions
}

/// Extract string from CozoDB DataValue
///
/// # 4-Word Name: extract_string_from_datavalue
fn extract_string_from_datavalue(value: &cozo::DataValue) -> String {
    match value {
        cozo::DataValue::Str(s) => s.to_string(),
        _ => String::new(),
    }
}

/// Escape single quotes for Datalog queries
///
/// # 4-Word Name: escape_single_quotes_for_datalog
fn escape_single_quotes(s: &str) -> String {
    s.replace('\'', "\\'")
}

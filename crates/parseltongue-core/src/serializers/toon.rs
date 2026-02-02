//! TOON (Tab-Oriented Object Notation) encoder for Parseltongue exports.
//!
//! # Executable Specification (S01 Principle #1)
//!
//! **CONTRACT**: Transform arrays-of-uniform-objects into tab-delimited TOON format.
//!
//! ## Preconditions
//! - Input: Non-empty Vec<T> where T implements Serialize
//! - All objects have identical schema (uniform array)
//!
//! ## Postconditions
//! - Returns TOON-formatted string with header + rows
//! - Field order is deterministic (alphabetical by field name)
//! - Tab delimiter separates values
//! - Fields containing special characters are quoted
//!
//! ## Error Conditions
//! - ToonError::EmptyArray if vec is empty
//! - ToonError::SerializationFailed if serde fails
//!
//! ## Token Efficiency Contract
//! - Must achieve ≥40% token reduction vs naive JSON
//! - Validated with automated tests (see tests/toon_token_efficiency_test.rs)
//!
//! # Architecture: Functional + Dependency Injection (S06)
//!
//! - **L2 (Std)**: Uses only std::collections, serde
//! - **No external deps**: Pure Rust, no toon-rust crate (MVP)
//! - **Trait-based**: Implements FormatEncoder for polymorphism

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;

/// TOON delimiter options
///
/// Tab is recommended for best LLM tokenization (29% faster parsing).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToonDelimiter {
    /// Tab character (recommended for LLMs)
    Tab,
    /// Comma (familiar to users)
    Comma,
    /// Pipe character (middle ground)
    Pipe,
}

impl ToonDelimiter {
    fn as_char(&self) -> char {
        match self {
            Self::Tab => '\t',
            Self::Comma => ',',
            Self::Pipe => '|',
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Tab => "\\t",
            Self::Comma => ",",
            Self::Pipe => "|",
        }
    }
}

/// TOON encoder configuration
#[derive(Debug, Clone)]
pub struct ToonConfig {
    /// Delimiter for field separation
    pub delimiter: ToonDelimiter,

    /// Array name in output (e.g., "entities")
    pub array_name: String,

    /// Quote fields containing delimiter/whitespace
    pub quote_fields: bool,
}

impl Default for ToonConfig {
    fn default() -> Self {
        Self {
            delimiter: ToonDelimiter::Tab,
            array_name: "items".to_string(),
            quote_fields: true,
        }
    }
}

/// TOON encoder
///
/// # Example
/// ```rust,ignore
/// let entities = vec![
///     Entity { name: "foo", age: 10 },
///     Entity { name: "bar", age: 20 },
/// ];
/// let encoder = ToonEncoder::new(ToonDelimiter::Tab, "entities");
/// let toon = encoder.encode(&entities)?;
/// // entities[2\t]{age,name}
/// // - 10\tfoo
/// // - 20\tbar
/// ```
pub struct ToonEncoder {
    config: ToonConfig,
}

impl ToonEncoder {
    /// Create new TOON encoder with tab delimiter
    pub fn new(delimiter: ToonDelimiter, array_name: &str) -> Self {
        Self {
            config: ToonConfig {
                delimiter,
                array_name: array_name.to_string(),
                quote_fields: true,
            },
        }
    }

    /// Create encoder with full configuration
    pub fn with_config(config: ToonConfig) -> Self {
        Self { config }
    }

    /// Encode array-of-objects to TOON format
    ///
    /// # Contract
    /// - Input: &[T] where T: Serialize
    /// - Output: TOON string with header + rows
    /// - Token efficiency: ≥40% reduction vs JSON
    ///
    /// # Example
    /// ```rust,ignore
    /// let data = vec![TestEntity { id: 1, name: "Alice" }];
    /// let toon = encoder.encode(&data)?;
    /// assert!(toon.contains("test_entity[1\\t]{id,name}"));
    /// ```
    pub fn encode<T: Serialize>(&self, data: &[T]) -> Result<String> {
        // Precondition: Non-empty array
        if data.is_empty() {
            anyhow::bail!("Cannot encode empty array (TOON spec requires row count)");
        }

        // Convert to JSON values for schema extraction
        let json_values: Vec<Value> = data
            .iter()
            .map(serde_json::to_value)
            .collect::<Result<_, _>>()
            .context("Failed to serialize items to JSON")?;

        // Extract schema from first object
        let schema = self.extract_schema(&json_values[0])?;

        // Build TOON output (functional composition)
        let header = self.format_header(&schema, data.len());
        let rows = self.format_rows(&json_values, &schema)?;

        Ok(format!("{}\n{}", header, rows))
    }

    /// Extract schema from JSON object
    ///
    /// # Postcondition
    /// - Returns field names in alphabetical order (determinism)
    fn extract_schema(&self, obj: &Value) -> Result<Vec<String>> {
        match obj {
            Value::Object(map) => {
                // Alphabetical order for determinism (S01: Executable Specifications)
                let mut fields: Vec<String> = map.keys().cloned().collect();
                fields.sort();
                Ok(fields)
            }
            _ => anyhow::bail!("Expected JSON object, got {:?}", obj),
        }
    }

    /// Format TOON header
    ///
    /// Format: `array_name[row_count<delimiter>]{field1,field2,...}`
    ///
    /// # Example
    /// ```text
    /// entities[100\t]{id,name,score}
    /// ```
    fn format_header(&self, schema: &[String], row_count: usize) -> String {
        let field_list = schema.join(",");
        let delim_marker = self.config.delimiter.as_str();

        format!(
            "{}[{}{}]{{{}}}:",
            self.config.array_name,
            row_count,
            delim_marker,
            field_list
        )
    }

    /// Format all rows
    ///
    /// Each row starts with "- " prefix (TOON spec).
    fn format_rows(&self, values: &[Value], schema: &[String]) -> Result<String> {
        values
            .iter()
            .map(|obj| self.format_row(obj, schema))
            .collect::<Result<Vec<_>>>()
            .map(|rows| rows.join("\n"))
    }

    /// Format single row
    ///
    /// Format: `- value1<delimiter>value2<delimiter>...`
    fn format_row(&self, obj: &Value, schema: &[String]) -> Result<String> {
        let delim = self.config.delimiter.as_char();

        let values: Vec<String> = schema
            .iter()
            .map(|field| {
                let value = &obj[field];
                self.format_value(value)
            })
            .collect::<Result<_>>()?;

        Ok(format!("  {}", values.join(&delim.to_string())))
    }

    /// Format single value with TOON quoting rules
    ///
    /// # Quoting Rules (TOON spec)
    /// 1. Quote if contains delimiter
    /// 2. Quote if starts/ends with whitespace
    /// 3. Quote if empty string
    /// 4. Quote if matches reserved words: true, false, null
    /// 5. Quote if looks like a number but should be string
    fn format_value(&self, value: &Value) -> Result<String> {
        match value {
            Value::String(s) => {
                if self.needs_quoting(s) {
                    Ok(format!("\"{}\"", self.escape_string(s)))
                } else {
                    Ok(s.clone())
                }
            }
            Value::Number(n) => Ok(n.to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            Value::Null => Ok("null".to_string()),
            Value::Array(items) => {
                // Inline array: [item1,item2,...]
                let formatted: Vec<String> = items
                    .iter()
                    .map(|item| self.format_value(item))
                    .collect::<Result<_>>()?;
                Ok(format!("[{}]", formatted.join(",")))
            }
            Value::Object(_) => {
                anyhow::bail!("Nested objects not supported in TOON tabular arrays")
            }
        }
    }

    /// Check if string needs quoting
    ///
    /// # Contract: Quote if
    /// - Contains delimiter character
    /// - Starts or ends with whitespace
    /// - Is empty
    /// - Matches reserved words (true, false, null)
    /// - Contains newline or colon
    fn needs_quoting(&self, s: &str) -> bool {
        if !self.config.quote_fields {
            return false;
        }

        let delim = self.config.delimiter.as_char();

        s.is_empty()
            || s.contains(delim)
            || s.trim() != s
            || s == "true"
            || s == "false"
            || s == "null"
            || s.contains('\n')
            || s.contains(':')
            || s.parse::<f64>().is_ok()
    }

    /// Escape string for TOON format
    ///
    /// # Escape Rules
    /// - `\` → `\\`
    /// - `"` → `\"`
    /// - `\n` → `\\n`
    /// - `\r` → `\\r`
    /// - `\t` → `\\t`
    fn escape_string(&self, s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize, Debug, PartialEq)]
    struct TestEntity {
        id: u32,
        name: String,
    }

    #[test]
    fn test_empty_array_fails() {
        let data: Vec<TestEntity> = vec![];
        let encoder = ToonEncoder::new(ToonDelimiter::Tab, "entities");
        let result = encoder.encode(&data);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty array"));
    }

    #[test]
    fn test_basic_encoding() {
        let data = vec![
            TestEntity {
                id: 1,
                name: "Alice".to_string(),
            },
            TestEntity {
                id: 2,
                name: "Bob".to_string(),
            },
        ];

        let encoder = ToonEncoder::new(ToonDelimiter::Tab, "entities");
        let toon = encoder.encode(&data).unwrap();

        // Verify header
        assert!(toon.contains("entities[2\\t]{id,name}:"));

        // Verify rows (tab-delimited)
        assert!(toon.contains("  1\tAlice"));
        assert!(toon.contains("  2\tBob"));
    }

    #[test]
    fn test_field_order_is_alphabetical() {
        #[derive(Serialize)]
        struct Entity {
            zebra: u32,
            apple: u32,
            mango: u32,
        }

        let data = vec![Entity {
            zebra: 1,
            apple: 2,
            mango: 3,
        }];

        let encoder = ToonEncoder::new(ToonDelimiter::Tab, "items");
        let toon = encoder.encode(&data).unwrap();

        // Fields must appear in alphabetical order
        assert!(toon.contains("{apple,mango,zebra}"));
        assert!(toon.contains("  2\t3\t1")); // Values in same order
    }

    #[test]
    fn test_quoting_delimiter_in_value() {
        #[derive(Serialize)]
        struct Entity {
            name: String,
            description: String,
        }

        let data = vec![Entity {
            name: "test".to_string(),
            description: "Has a\ttab character".to_string(),
        }];

        let encoder = ToonEncoder::new(ToonDelimiter::Tab, "items");
        let toon = encoder.encode(&data).unwrap();

        // Should quote field containing tab
        assert!(toon.contains("\"Has a\\ttab character\""));
    }

    #[test]
    fn test_quoting_empty_string() {
        #[derive(Serialize)]
        struct Entity {
            name: String,
            email: String,
        }

        let data = vec![Entity {
            name: "Alice".to_string(),
            email: "".to_string(), // Empty string
        }];

        let encoder = ToonEncoder::new(ToonDelimiter::Tab, "items");
        let toon = encoder.encode(&data).unwrap();

        // Empty string should be quoted
        assert!(toon.contains("\"\""));
    }

    #[test]
    fn test_comma_delimiter() {
        let data = vec![TestEntity {
            id: 1,
            name: "Alice".to_string(),
        }];

        let encoder = ToonEncoder::new(ToonDelimiter::Comma, "entities");
        let toon = encoder.encode(&data).unwrap();

        // Verify comma delimiter in header and row
        assert!(toon.contains("entities[1,]{id,name}:"));
        assert!(toon.contains("  1,Alice"));
    }

    #[test]
    fn test_inline_array_values() {
        #[derive(Serialize)]
        struct Entity {
            name: String,
            tags: Vec<String>,
        }

        let data = vec![Entity {
            name: "Alice".to_string(),
            tags: vec!["rust".to_string(), "tdd".to_string()],
        }];

        let encoder = ToonEncoder::new(ToonDelimiter::Tab, "items");
        let toon = encoder.encode(&data).unwrap();

        // Should format inline array
        assert!(toon.contains("[rust,tdd]"));
    }

    #[test]
    fn test_reserved_words_quoted() {
        #[derive(Serialize)]
        struct Entity {
            value: String,
        }

        let data = vec![
            Entity {
                value: "true".to_string(),
            },
            Entity {
                value: "false".to_string(),
            },
            Entity {
                value: "null".to_string(),
            },
        ];

        let encoder = ToonEncoder::new(ToonDelimiter::Tab, "items");
        let toon = encoder.encode(&data).unwrap();

        // Reserved words should be quoted
        assert!(toon.contains("\"true\""));
        assert!(toon.contains("\"false\""));
        assert!(toon.contains("\"null\""));
    }

    #[test]
    fn test_deterministic_output() {
        let data = vec![TestEntity {
            id: 1,
            name: "Alice".to_string(),
        }];

        let encoder = ToonEncoder::new(ToonDelimiter::Tab, "entities");

        // Encode twice
        let toon1 = encoder.encode(&data).unwrap();
        let toon2 = encoder.encode(&data).unwrap();

        // Should be identical (determinism)
        assert_eq!(toon1, toon2);
    }
}

// ============================================================================
// Serializer Trait Implementation
// ============================================================================

use super::Serializer;

/// TOON serializer implementing the core Serializer trait
///
/// # Characteristics
/// - Tab-delimited format optimized for LLMs
/// - 30-40% token reduction vs JSON
/// - Alphabetically sorted fields for determinism
/// - ~17 tokens per entity (vs ~30 for JSON)
pub struct ToonSerializer {
    encoder: ToonEncoder,
}

impl ToonSerializer {
    /// Create new TOON serializer with default settings (Tab delimiter)
    pub fn new() -> Self {
        Self {
            encoder: ToonEncoder::new(ToonDelimiter::Tab, "entities"),
        }
    }

    /// Create TOON serializer with specific delimiter
    pub fn with_delimiter(delimiter: ToonDelimiter) -> Self {
        Self {
            encoder: ToonEncoder::new(delimiter, "entities"),
        }
    }

    /// Create TOON serializer with custom array name
    pub fn with_name(delimiter: ToonDelimiter, array_name: &str) -> Self {
        Self {
            encoder: ToonEncoder::new(delimiter, array_name),
        }
    }
}

impl Default for ToonSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl Serializer for ToonSerializer {
    fn serialize<T: Serialize>(&self, data: &[T]) -> Result<String> {
        if data.is_empty() {
            // Handle empty arrays gracefully (TOON spec requires row count)
            Ok(format!("{}[0\\t]{{}}\n  # No data", self.encoder.config.array_name))
        } else {
            self.encoder.encode(data)
        }
    }

    fn extension(&self) -> &'static str {
        "toon"
    }

    fn estimate_tokens(&self, entity_count: usize) -> usize {
        // TOON: ~17 tokens per entity (41.9% reduction from JSON's ~30)
        // Empty array: ~10 tokens for header comment
        if entity_count == 0 {
            10
        } else {
            // Header overhead (~20 tokens) + (17 tokens * entity_count)
            20 + (entity_count * 17)
        }
    }
}

#[cfg(test)]
mod serializer_tests {
    use super::*;

    #[derive(Serialize)]
    struct TestEntity {
        name: String,
        value: i32,
    }

    #[test]
    fn test_toon_serializer_empty() {
        let serializer = ToonSerializer::new();
        let data: Vec<TestEntity> = vec![];

        let result = serializer.serialize(&data).unwrap();
        assert!(result.contains("entities[0\\t]"));
        assert!(result.contains("# No data"));
    }

    #[test]
    fn test_toon_serializer_single_entity() {
        let serializer = ToonSerializer::new();
        let data = vec![TestEntity {
            name: "test".to_string(),
            value: 42,
        }];

        let result = serializer.serialize(&data).unwrap();
        assert!(result.contains("entities[1\\t]"));
        assert!(result.contains("test"));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_toon_extension() {
        let serializer = ToonSerializer::new();
        assert_eq!(serializer.extension(), "toon");
    }

    #[test]
    fn test_toon_token_estimation() {
        let serializer = ToonSerializer::new();

        // Empty array: ~10 tokens
        assert_eq!(serializer.estimate_tokens(0), 10);

        // Single entity: 20 (header) + 17 (entity) = 37 tokens
        assert_eq!(serializer.estimate_tokens(1), 37);

        // 100 entities: 20 + (100 * 17) = 1720 tokens
        assert_eq!(serializer.estimate_tokens(100), 1720);
    }
}

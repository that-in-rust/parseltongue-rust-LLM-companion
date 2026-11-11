//! JSON serializer implementation
//!
//! Standard JSON format for maximum tool compatibility.

use super::Serializer;
use anyhow::Result;
use serde::Serialize;

/// JSON serializer using serde_json
///
/// # Characteristics
/// - Human-readable format
/// - Universal tool compatibility
/// - ~30 tokens per entity (baseline)
/// - Pretty-printed for readability
pub struct JsonSerializer;

impl JsonSerializer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl Serializer for JsonSerializer {
    fn serialize<T: Serialize>(&self, data: &[T]) -> Result<String> {
        // serde_json handles empty arrays gracefully: "[]"
        Ok(serde_json::to_string_pretty(data)?)
    }

    fn extension(&self) -> &'static str {
        "json"
    }

    fn estimate_tokens(&self, entity_count: usize) -> usize {
        // JSON: ~30 tokens per entity (measured empirically)
        // Empty array: 2 tokens for "[]"
        if entity_count == 0 {
            2
        } else {
            entity_count * 30
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestEntity {
        name: String,
        value: i32,
    }

    #[test]
    fn test_json_empty_array() {
        let serializer = JsonSerializer::new();
        let data: Vec<TestEntity> = vec![];

        let result = serializer.serialize(&data).unwrap();
        assert_eq!(result, "[]");
    }

    #[test]
    fn test_json_single_entity() {
        let serializer = JsonSerializer::new();
        let data = vec![TestEntity {
            name: "test".to_string(),
            value: 42,
        }];

        let result = serializer.serialize(&data).unwrap();
        assert!(result.contains("test"));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_json_extension() {
        let serializer = JsonSerializer::new();
        assert_eq!(serializer.extension(), "json");
    }

    #[test]
    fn test_json_token_estimation() {
        let serializer = JsonSerializer::new();

        // Empty array
        assert_eq!(serializer.estimate_tokens(0), 2);

        // Single entity: ~30 tokens
        assert_eq!(serializer.estimate_tokens(1), 30);

        // 100 entities: ~3000 tokens
        assert_eq!(serializer.estimate_tokens(100), 3000);
    }
}

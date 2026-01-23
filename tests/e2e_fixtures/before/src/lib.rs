// Minimal test codebase - BEFORE state
// Production code only (no #[test] functions to avoid TEST exclusion)

/// Configuration struct that will be unchanged
pub struct Config {
    pub name: String,
    pub value: i32,
}

impl Config {
    /// Creates new config - this function will be UNCHANGED
    pub fn new(name: &str, value: i32) -> Self {
        Config {
            name: name.to_string(),
            value,
        }
    }

    /// Method that will be UNCHANGED
    pub fn display(&self) -> String {
        format!("{}: {}", self.name, self.value)
    }
}

/// Helper function that will be REMOVED in AFTER state
pub fn helper_to_remove(x: i32) -> i32 {
    x * 2
}

/// Function that will be MODIFIED (content change) in AFTER state
pub fn function_to_modify(input: &str) -> String {
    format!("Original: {}", input)
}

/// Function that calls helper - will show in blast radius when helper is removed
pub fn caller_function(x: i32) -> i32 {
    helper_to_remove(x) + 10
}

/// Another function that uses Config - unchanged
pub fn use_config(config: &Config) -> String {
    config.display()
}

/// Public API function - unchanged
pub fn main_api_function(name: &str, value: i32) -> String {
    let config = Config::new(name, value);
    use_config(&config)
}

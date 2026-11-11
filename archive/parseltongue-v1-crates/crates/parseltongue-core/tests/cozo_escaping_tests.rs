/// Tests for CozoDB query string escaping
///
/// Verifies that special characters (backslash, single quote) are properly escaped
/// when constructing CozoDB queries to prevent parse errors.

#[cfg(test)]
mod cozo_escaping_tests {
    use parseltongue_core::storage::cozo_client::escape_for_cozo_string;

    #[test]
    fn test_escape_backslash_windows_path() {
        // Windows paths contain backslashes that must be escaped
        let input = r"C:\Users\Developer\MyApp\Service.cs";
        let expected = r"C:\\Users\\Developer\\MyApp\\Service.cs";
        assert_eq!(escape_for_cozo_string(input), expected);
    }

    #[test]
    fn test_escape_php_namespace() {
        // PHP namespaces use backslash as separator
        let input = r"MyApp\Controllers\UserController";
        let expected = r"MyApp\\Controllers\\UserController";
        assert_eq!(escape_for_cozo_string(input), expected);
    }

    #[test]
    fn test_escape_single_quote() {
        // Single quotes in strings must be escaped
        let input = "User's Profile";
        let expected = r"User\'s Profile";
        assert_eq!(escape_for_cozo_string(input), expected);
    }

    #[test]
    fn test_escape_backslash_and_quote() {
        // Combined test: both backslash and quote
        let input = r"C:\Path\User's\File.cs";
        let expected = r"C:\\Path\\User\'s\\File.cs";
        assert_eq!(escape_for_cozo_string(input), expected);
    }

    #[test]
    fn test_escape_empty_string() {
        let input = "";
        let expected = "";
        assert_eq!(escape_for_cozo_string(input), expected);
    }

    #[test]
    fn test_escape_no_special_chars() {
        let input = "normal/unix/path.rs";
        let expected = "normal/unix/path.rs";
        assert_eq!(escape_for_cozo_string(input), expected);
    }
}

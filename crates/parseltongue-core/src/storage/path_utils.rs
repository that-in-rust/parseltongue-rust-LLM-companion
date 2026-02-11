// Path normalization utilities for cross-platform consistency (v1.6.5)
//
// All paths stored in CozoDB are normalized to forward slashes (/) regardless
// of the host operating system. This ensures queries work identically on
// macOS, Linux, and Windows.
//
// Reference: Git, SCIP, SonarQube, Semgrep all normalize to forward slashes.

use path_slash::PathBufExt;
use std::path::Path;

/// Normalize file path and split into folder + filename components.
///
/// # 4-Word Name: normalize_split_file_path
///
/// # Arguments
/// * `abs_path` - Absolute path to the file
/// * `workspace_root` - Workspace root directory (will be stripped)
///
/// # Returns
/// * `(folder_path, filename)` tuple where:
///   - `folder_path`: Forward-slash separated folder path (empty string for root-level files)
///   - `filename`: Filename only (no directory components)
///
/// # Examples
/// ```
/// use std::path::Path;
/// use parseltongue_core::storage::path_utils::normalize_split_file_path;
///
/// let workspace = Path::new("/workspace");
/// let file = Path::new("/workspace/src/main.rs");
/// let (folder, name) = normalize_split_file_path(file, workspace);
/// assert_eq!(folder, "src");
/// assert_eq!(name, "main.rs");
///
/// // Root-level file
/// let root_file = Path::new("/workspace/Cargo.toml");
/// let (folder, name) = normalize_split_file_path(root_file, workspace);
/// assert_eq!(folder, "");
/// assert_eq!(name, "Cargo.toml");
/// ```
pub fn normalize_split_file_path(abs_path: &Path, workspace_root: &Path) -> (String, String) {
    // Strip workspace root to get relative path
    let relative = abs_path.strip_prefix(workspace_root).unwrap_or(abs_path);

    // Normalize to forward slashes (cross-platform)
    let normalized = relative.to_path_buf().to_slash_lossy().to_string();

    // Split at last '/' separator
    match normalized.rsplit_once('/') {
        Some((folder, file)) => (folder.to_string(), file.to_string()),
        None => (String::new(), normalized), // Root-level file
    }
}

/// Extract L1 and L2 subfolder levels from normalized path.
///
/// # 4-Word Name: extract_subfolder_levels_from_path
///
/// # Arguments
/// * `normalized_path` - Forward-slash normalized relative path
///
/// # Returns
/// * `(L1, L2)` tuple where:
///   - `L1`: First-level folder (root-level folder name, or "." for root files)
///   - `L2`: Second-level folder (subfolder within L1, or empty string)
///
/// # Examples
/// ```
/// use parseltongue_core::storage::path_utils::extract_subfolder_levels_from_path;
///
/// let (l1, l2) = extract_subfolder_levels_from_path("src/main.rs");
/// assert_eq!(l1, "src");
/// assert_eq!(l2, "");
///
/// let (l1, l2) = extract_subfolder_levels_from_path("src/core/parser.rs");
/// assert_eq!(l1, "src");
/// assert_eq!(l2, "core");
///
/// let (l1, l2) = extract_subfolder_levels_from_path("crates/parseltongue-core/src/lib.rs");
/// assert_eq!(l1, "crates");
/// assert_eq!(l2, "parseltongue-core");
///
/// // Root-level file
/// let (l1, l2) = extract_subfolder_levels_from_path("Cargo.toml");
/// assert_eq!(l1, ".");
/// assert_eq!(l2, "");
/// ```
pub fn extract_subfolder_levels_from_path(normalized_path: &str) -> (String, String) {
    // Strip leading "./" if present (common in relative paths)
    let cleaned = normalized_path.strip_prefix("./").unwrap_or(normalized_path);
    let parts: Vec<&str> = cleaned.split('/').collect();

    match parts.len() {
        0 => (".".to_string(), String::new()),       // Empty path (shouldn't happen)
        1 => (".".to_string(), String::new()),       // Root-level file: Cargo.toml
        2 => (parts[0].to_string(), String::new()),  // src/main.rs -> ("src", "")
        _ => (parts[0].to_string(), parts[1].to_string()), // src/core/lib.rs -> ("src", "core")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_normalize_split_file_path_root_level() {
        let workspace = PathBuf::from("/workspace");
        let file = PathBuf::from("/workspace/Cargo.toml");
        let (folder, name) = normalize_split_file_path(&file, &workspace);

        assert_eq!(folder, "");
        assert_eq!(name, "Cargo.toml");
    }

    #[test]
    fn test_normalize_split_file_path_one_level() {
        let workspace = PathBuf::from("/workspace");
        let file = PathBuf::from("/workspace/src/main.rs");
        let (folder, name) = normalize_split_file_path(&file, &workspace);

        assert_eq!(folder, "src");
        assert_eq!(name, "main.rs");
    }

    #[test]
    fn test_normalize_split_file_path_nested() {
        let workspace = PathBuf::from("/workspace");
        let file = PathBuf::from("/workspace/crates/parseltongue-core/src/lib.rs");
        let (folder, name) = normalize_split_file_path(&file, &workspace);

        assert_eq!(folder, "crates/parseltongue-core/src");
        assert_eq!(name, "lib.rs");
    }

    #[test]
    fn test_normalize_split_file_path_spaces() {
        let workspace = PathBuf::from("/workspace");
        let file = PathBuf::from("/workspace/My Documents/code/app.rs");
        let (folder, name) = normalize_split_file_path(&file, &workspace);

        assert_eq!(folder, "My Documents/code");
        assert_eq!(name, "app.rs");
    }

    #[test]
    fn test_extract_subfolder_levels_root() {
        let (l1, l2) = extract_subfolder_levels_from_path("Cargo.toml");
        assert_eq!(l1, ".");
        assert_eq!(l2, "");
    }

    #[test]
    fn test_extract_subfolder_levels_one_level() {
        let (l1, l2) = extract_subfolder_levels_from_path("src/main.rs");
        assert_eq!(l1, "src");
        assert_eq!(l2, "");
    }

    #[test]
    fn test_extract_subfolder_levels_two_levels() {
        let (l1, l2) = extract_subfolder_levels_from_path("src/core/parser.rs");
        assert_eq!(l1, "src");
        assert_eq!(l2, "core");
    }

    #[test]
    fn test_extract_subfolder_levels_three_plus_levels() {
        let (l1, l2) = extract_subfolder_levels_from_path("crates/parseltongue-core/src/storage/cozo_client.rs");
        assert_eq!(l1, "crates");
        assert_eq!(l2, "parseltongue-core");
    }

    #[test]
    fn test_extract_subfolder_levels_spaces() {
        let (l1, l2) = extract_subfolder_levels_from_path("My Documents/code/app.rs");
        assert_eq!(l1, "My Documents");
        assert_eq!(l2, "code");
    }

    #[test]
    fn test_extract_subfolder_levels_dot_slash_prefix() {
        let (l1, l2) = extract_subfolder_levels_from_path("./crates/parseltongue-core/src/lib.rs");
        assert_eq!(l1, "crates");
        assert_eq!(l2, "parseltongue-core");
    }

    #[test]
    fn test_extract_subfolder_levels_dot_slash_root() {
        let (l1, l2) = extract_subfolder_levels_from_path("./Cargo.toml");
        assert_eq!(l1, ".");
        assert_eq!(l2, "");
    }
}

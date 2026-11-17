//! REQ-RUST-072.0 D3-D5, D7: the stored form of a path under the workspace root, and the
//! disk path of a stored form. Pure: no file is opened here.

#[cfg(test)]
#[path = "workspace-root-path-tests.rs"]
mod workspace_root_path_tests;

/// The root every stored path of a workspace file starts with, in place of the real one.
pub const WORKSPACE_FIXED_ROOT_PATH: &str = "/parseltongue-workspace-root";

/// The marker a `cargo` package id of a local package writes before its directory.
const PACKAGE_FILE_URL_MARKER: &str = "file://";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceRootRelocationError {
    /// The real root is not an absolute path below `/`.
    RealRootNotAbsolute,
    /// A real path starts with the fixed root, so its stored form would be ambiguous.
    FixedRootNameClash,
}

impl std::fmt::Display for WorkspaceRootRelocationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::RealRootNotAbsolute => "workspace root is not an absolute directory path",
            Self::FixedRootNameClash => "a real path starts with the fixed workspace root",
        })
    }
}

impl std::error::Error for WorkspaceRootRelocationError {}

/// The canonical real root of the mapped workspace, without a final `/`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceRootRelocationValue {
    workspace_real_root_path: String,
}

/// The rest of `text` after `root`, when `root` ends at a path component boundary.
fn strip_root_path_prefix<'text>(text: &'text str, root: &str) -> Option<&'text str> {
    text.strip_prefix(root)
        .filter(|rest| rest.is_empty() || rest.starts_with(['/', '#']))
}

/// `text` with `from` replaced by `to` where a path starts: at the start of the text, or
/// behind the `file://` of a package id.
fn replace_leading_root_path(text: &str, from: &str, to: &str) -> Option<String> {
    let start = text
        .find(PACKAGE_FILE_URL_MARKER)
        .map_or(0, |marker| marker + PACKAGE_FILE_URL_MARKER.len());
    strip_root_path_prefix(&text[start..], from).map(|rest| format!("{}{to}{rest}", &text[..start]))
}

impl WorkspaceRootRelocationValue {
    /// D6 is the caller's: `real_root` is the canonical root. A final `/` is dropped.
    pub fn build_workspace_root_relocation(
        real_root: &str,
    ) -> Result<Self, WorkspaceRootRelocationError> {
        let root = real_root.trim_end_matches('/');
        if !root.starts_with('/') {
            return Err(WorkspaceRootRelocationError::RealRootNotAbsolute);
        }
        if strip_root_path_prefix(root, WORKSPACE_FIXED_ROOT_PATH).is_some() {
            return Err(WorkspaceRootRelocationError::FixedRootNameClash);
        }
        Ok(Self {
            workspace_real_root_path: root.to_owned(),
        })
    }

    /// D3, D4, D7: the stored form of a path or of a package id.
    pub fn relocate_workspace_path_text(
        &self,
        text: &str,
    ) -> Result<String, WorkspaceRootRelocationError> {
        if replace_leading_root_path(text, WORKSPACE_FIXED_ROOT_PATH, "").is_some() {
            return Err(WorkspaceRootRelocationError::FixedRootNameClash);
        }
        Ok(replace_leading_root_path(
            text,
            &self.workspace_real_root_path,
            WORKSPACE_FIXED_ROOT_PATH,
        )
        .unwrap_or_else(|| text.to_owned()))
    }

    /// D5: the disk form of a stored path or package id.
    pub fn restore_workspace_disk_path(&self, stored: &str) -> String {
        replace_leading_root_path(
            stored,
            WORKSPACE_FIXED_ROOT_PATH,
            &self.workspace_real_root_path,
        )
        .unwrap_or_else(|| stored.to_owned())
    }
}

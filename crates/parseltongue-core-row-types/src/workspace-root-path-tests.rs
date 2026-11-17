#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;

/// REQ-RUST-072.0 D3, D4, D5, D7
#[test]
fn test_workspace_root_path_relocation() {
    let build = WorkspaceRootRelocationValue::build_workspace_root_relocation;
    let relocation = build("/a/itoa").unwrap();
    let cases = [
        (
            "/a/itoa/src/lib.rs",
            "/parseltongue-workspace-root/src/lib.rs",
        ),
        ("/a/itoa", "/parseltongue-workspace-root"),
        (
            "path+file:///a/itoa#1.0.18",
            "path+file:///parseltongue-workspace-root#1.0.18",
        ),
        (
            "path+file:///a/itoa/member#0.1.0",
            "path+file:///parseltongue-workspace-root/member#0.1.0",
        ),
        // D3: a sibling that shares a name prefix, and D4: paths outside the root.
        ("/a/itoa2/src/lib.rs", "/a/itoa2/src/lib.rs"),
        ("/a/ito", "/a/ito"),
        ("/b/a/itoa/x.rs", "/b/a/itoa/x.rs"),
        ("src/lib.rs", "src/lib.rs"),
        (
            "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.0",
            "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.0",
        ),
    ];
    let stored = cases.map(|(real, _)| relocation.relocate_workspace_path_text(real));
    assert_eq!(stored, cases.map(|(_, stored)| Ok(stored.to_owned())));
    // D5: restoring a stored form gives the path back.
    let restored = cases.map(|(_, stored)| relocation.restore_workspace_disk_path(stored));
    assert_eq!(restored, cases.map(|(real, _)| real.to_owned()));
    assert_eq!(build("/a/itoa/"), Ok(relocation.clone()));
    // D7 and the unusable roots.
    let clash = Err(WorkspaceRootRelocationError::FixedRootNameClash);
    assert_eq!(
        [
            build("a/itoa"),
            build("/"),
            build(""),
            build("/parseltongue-workspace-root"),
            build("/parseltongue-workspace-root/itoa"),
        ],
        [
            Err(WorkspaceRootRelocationError::RealRootNotAbsolute),
            Err(WorkspaceRootRelocationError::RealRootNotAbsolute),
            Err(WorkspaceRootRelocationError::RealRootNotAbsolute),
            clash.clone(),
            clash.clone(),
        ]
    );
    assert_eq!(
        [
            "/parseltongue-workspace-root/src/lib.rs",
            "path+file:///parseltongue-workspace-root#1.0.18",
        ]
        .map(|real| relocation.relocate_workspace_path_text(real)),
        [
            Err(WorkspaceRootRelocationError::FixedRootNameClash),
            Err(WorkspaceRootRelocationError::FixedRootNameClash),
        ]
    );
    assert_eq!(build("/parseltongue-workspace-root2").map(|_| ()), Ok(()));
}

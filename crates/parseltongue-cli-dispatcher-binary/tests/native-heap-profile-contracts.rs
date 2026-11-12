#![cfg(all(
    feature = "system-heap-allocation-profile",
    not(feature = "native-production-heap-allocator")
))]
#![allow(clippy::unwrap_used)]

use std::{fs, process::Command};

#[test]
fn test_native_heap_profile_flushes_errors() {
    [None, Some("unknown-command")]
        .into_iter()
        .for_each(|command| {
            let directory = tempfile::tempdir().unwrap();
            let profile = directory
                .path()
                .join("heap-allocation-profile-records.json");
            let mut child = Command::new(env!("CARGO_BIN_EXE_parseltongue"));
            child.env("PARSELTONGUE_HEAP_PROFILE_PATH", &profile);
            if let Some(command) = command {
                child.arg(command);
            }
            let output = child.output().unwrap();
            assert!(!output.status.success());
            let data: serde_json::Value =
                serde_json::from_slice(&fs::read(profile).unwrap()).unwrap();
            assert_eq!(data["dhatFileVersion"], 2);
            assert_eq!(data["mode"], "rust-heap");
            assert!(data["pps"].as_array().is_some());
        });
}

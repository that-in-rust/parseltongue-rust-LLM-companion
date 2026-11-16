#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::{SYSROOT_MEMBER_TABLE_NAME, collect_sysroot_member_rows};
use serde_json::Value;
use std::fs;

/// REQ-RUST-054.0 D6, D9
#[test]
fn test_core_directory_member_rows() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("core/src");
    fs::create_dir_all(source.join("mem")).unwrap();
    fs::create_dir_all(directory.path().join("alloc/src")).unwrap();
    fs::write(source.join("z.rs"), "impl Z { fn last_one(self) {} }").unwrap();
    fs::write(
        source.join("mem/maybe_uninit.rs"),
        "impl<T> MaybeUninit<T> { pub fn write(&mut self, val: T) -> &mut T { loop {} } }",
    )
    .unwrap();
    fs::write(source.join("notes.txt"), "impl N { fn not_read(self) {} }").unwrap();
    fs::write(
        directory.path().join("alloc/src/lib.rs"),
        "impl A { fn other_crate(self) {} }",
    )
    .unwrap();
    let rows = collect_sysroot_member_rows(directory.path()).unwrap();
    let keys = rows
        .iter()
        .map(|row| row.stage_row_record_key.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        vec!["core/src/mem/maybe_uninit.rs#32", "core/src/z.rs#12"],
        keys
    );
    assert!(
        rows.iter()
            .all(|row| row.stage_owned_table_name == SYSROOT_MEMBER_TABLE_NAME)
    );
    let write: Value = serde_json::from_str(&rows[0].stage_row_payload_text).unwrap();
    assert_eq!(
        "core/src/mem/maybe_uninit.rs",
        write["declaration_source_relative_path"]
    );
    assert_eq!("write", write["member_declared_name_value"]);
    assert_eq!("MaybeUninit<T>", write["impl_self_type_text"]);

    fs::write(source.join("bad.rs"), [0xff_u8, 0xfe, 0x00]).unwrap();
    let error = collect_sysroot_member_rows(directory.path()).unwrap_err();
    assert!(
        error.error_context_detail_text.contains("core/src/bad.rs"),
        "{error:?}"
    );
    let missing = collect_sysroot_member_rows(&directory.path().join("absent")).unwrap_err();
    assert!(
        missing.error_context_detail_text.contains("core/src"),
        "{missing:?}"
    );
}

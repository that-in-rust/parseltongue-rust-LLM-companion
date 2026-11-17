#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::{
    SYSROOT_MEMBER_TABLE_NAME, check_sysroot_source_record, collect_sysroot_member_rows,
    read_sysroot_source_record,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;

fn hash_bytes_sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// REQ-RUST-065.0 D1, D3, D4, D5
#[test]
fn test_core_source_listing_record() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("core/src");
    fs::create_dir_all(source.join("mem")).unwrap();
    let (last, first) = ("impl Z { fn z(self) {} }", "impl M { fn m(self) {} }");
    fs::write(source.join("z.rs"), last).unwrap();
    fs::write(source.join("mem/a.rs"), first).unwrap();
    fs::write(source.join("notes.txt"), "not read").unwrap();
    // D1: one `sha256sum` line per file, in byte order of the relative path.
    let listing = format!(
        "{}  core/src/mem/a.rs\n{}  core/src/z.rs\n",
        hash_bytes_sha256_hex(first.as_bytes()),
        hash_bytes_sha256_hex(last.as_bytes())
    );
    let record = read_sysroot_source_record(directory.path()).unwrap();
    assert_eq!(
        json!({
            "sysroot_source_directory_path": "core/src",
            "sysroot_source_file_count": 2,
            "sysroot_source_byte_count": first.len() + last.len(),
            "sysroot_listing_digest_sha256": hash_bytes_sha256_hex(listing.as_bytes()),
        }),
        record
    );
    // D4: the rows come with the record of the bytes they were parsed from.
    let (parsed, rows) = collect_sysroot_member_rows(directory.path()).unwrap();
    assert_eq!((&record, 2), (&parsed, rows.len()));
    assert!(
        check_sysroot_source_record(&json!({"workspace_sysroot_source_record": record}), &parsed)
            .is_ok()
    );
    // D3: one byte more gives another digest. D4: the stored record no longer holds.
    fs::write(source.join("z.rs"), format!("{last} ")).unwrap();
    let (changed, _) = collect_sysroot_member_rows(directory.path()).unwrap();
    assert_ne!(
        record["sysroot_listing_digest_sha256"],
        changed["sysroot_listing_digest_sha256"]
    );
    let stored = json!({"workspace_sysroot_source_record": record});
    let error = check_sysroot_source_record(&stored, &changed).unwrap_err();
    assert!(
        error.error_context_detail_text.contains("core/src"),
        "{error:?}"
    );
    assert!(check_sysroot_source_record(&json!({}), &changed).is_err());
    // D5: a directory that cannot be read fails; no file is skipped.
    assert!(read_sysroot_source_record(&directory.path().join("absent")).is_err());
}

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
    let (_, rows) = collect_sysroot_member_rows(directory.path()).unwrap();
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

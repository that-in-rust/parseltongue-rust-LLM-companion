#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::find_file_scope_type;
use crate::sysroot_member_declaration_records::extract_member_declaration_rows;
use serde_json::{Value, json};

const ROOT: &str = "#[prelude_import] use prelude::rust_2024::*;\npub mod prelude; pub mod result; pub mod fmt; pub mod convert;\n";
const PRELUDE: &str = "pub use crate::result::Result::{self, Err, Ok};\n";
const RESULT: &str = "pub enum Result<T, E> { Ok(T), Err(E) }\n";
const FMT: &str = "pub type Result = crate::result::Result<(), Error>;\n";
const CONVERT: &str =
    "use crate::fmt;\npub trait TryInto<T> { fn try_into(self) -> Result<T, ()>; }\n";

/// The declaration rows of a small `core`: five files, each replaced or joined by `swap`.
fn list_scope_fixture_rows(swap: &[(&str, &str)]) -> Vec<Value> {
    [
        ("core/src/lib.rs", ROOT),
        ("core/src/prelude/v1.rs", PRELUDE),
        ("core/src/result.rs", RESULT),
        ("core/src/fmt/mod.rs", FMT),
        ("core/src/convert/mod.rs", CONVERT),
    ]
    .into_iter()
    .chain(swap.iter().copied())
    .collect::<std::collections::BTreeMap<_, _>>()
    .into_iter()
    .flat_map(|(path, text)| extract_member_declaration_rows(path, text))
    .collect()
}

/// REQ-RUST-062.0 C10, C11: a name the file declares, and a name the prelude gives.
#[test]
fn test_file_scope_type_proven() {
    let rows = list_scope_fixture_rows(&[]);
    let declared = find_file_scope_type(&rows, "core/src/result.rs", "Result").unwrap();
    let site = json!({
        "declaration_source_relative_path":"core/src/result.rs",
        "member_name_start_offset":RESULT.find("Result").unwrap(),
        "member_name_end_offset":RESULT.find("Result").unwrap() + 6
    });
    assert_eq!(
        json!(["file_scope_declaration", site]),
        json!([
            declared["type_name_proof_kind"],
            declared["type_declaration_site_value"]
        ])
    );
    let given = find_file_scope_type(&rows, "core/src/convert/mod.rs", "Result").unwrap();
    assert_eq!(
        json!({
            "type_name_value":"Result",
            "name_written_source_path":"core/src/convert/mod.rs",
            "type_name_proof_kind":"prelude_import",
            "type_declaration_site_value":site,
            "prelude_import_site_value":{
                "declaration_source_relative_path":"core/src/lib.rs",
                "member_name_start_offset":ROOT.find('*').unwrap(),
                "member_name_end_offset":ROOT.find('*').unwrap() + 1
            },
            "prelude_leaf_site_value":{
                "declaration_source_relative_path":"core/src/prelude/v1.rs",
                "member_name_start_offset":PRELUDE.find("self").unwrap(),
                "member_name_end_offset":PRELUDE.find("self").unwrap() + 4
            },
            "prelude_leaf_path_text":"crate::result::Result"
        }),
        given
    );
    // The alias file binds the name to something that is not a struct, enum or union.
    assert_eq!(
        None,
        find_file_scope_type(&rows, "core/src/fmt/mod.rs", "Result")
    );
}

/// REQ-RUST-062.0 C10, C11: each missing proof step withholds the name.
#[test]
fn test_file_scope_type_unproven() {
    let unproven = |swap: &[(&str, &str)]| {
        let rows = list_scope_fixture_rows(swap);
        assert_eq!(
            None,
            find_file_scope_type(&rows, "core/src/convert/mod.rs", "Result"),
            "{swap:?}"
        );
    };
    let convert = "core/src/convert/mod.rs";
    // The file may bind the name some other way.
    unproven(&[(convert, "use crate::fmt::*;\n")]);
    unproven(&[(convert, "declare_names!();\n")]);
    unproven(&[(convert, "extern \"C\" { type Result; }\n")]);
    unproven(&[(convert, "use crate::fmt::Result;\n")]);
    unproven(&[(convert, "mod Result {}\n")]);
    unproven(&[("core/src/num.rs", "extern crate alloc as Result;\n")]);
    // The prelude does not give the name one declaration.
    let root = "core/src/lib.rs";
    unproven(&[(root, "pub mod prelude; pub mod result;\n")]);
    unproven(&[(
        root,
        "#[prelude_import] use other::*;\npub mod prelude; pub mod result;\n",
    )]);
    unproven(&[(
        root,
        "#[prelude_import] use prelude::rust_2024::*;\npub mod prelude; #[path = \"x.rs\"] pub mod result;\n",
    )]);
    unproven(&[(
        root,
        "#[prelude_import] use prelude::rust_2024::*;\npub mod prelude; pub mod result { }\n",
    )]);
    unproven(&[(
        root,
        "#[prelude_import] use prelude::rust_2024::*;\npub mod prelude; #[cfg(a)] pub mod result;\n",
    )]);
    unproven(&[(
        "core/src/prelude/v1.rs",
        "pub use crate::fmt::Result;\npub use crate::result::Result::{self};\n",
    )]);
    unproven(&[("core/src/prelude/v1.rs", "pub use crate::a::b::Result;\n")]);
    unproven(&[("core/src/prelude/v1.rs", "pub use crate::result::Other;\n")]);
    unproven(&[(
        "core/src/result.rs",
        "#[cfg(a)] pub enum Result<T, E> { Ok(T), Err(E) }\n",
    )]);
    unproven(&[("core/src/result.rs", "pub type Result = ();\n")]);
    unproven(&[("core/src/result/mod.rs", "pub struct Other;\n")]);
}

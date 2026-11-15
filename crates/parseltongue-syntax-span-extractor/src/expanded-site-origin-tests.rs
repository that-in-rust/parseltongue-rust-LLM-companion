#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use serde_json::json;

fn make_token_origin_index() -> ExpandedTokenOriginIndex {
    let token = |output: [u64; 2], original: [u64; 2], kind: &str| {
        json!({
            "token_output_byte_range": output,
            "token_original_byte_range": original,
            "token_source_absolute_path": "/workspace/src/lib.rs",
            "token_source_origin_kind": kind
        })
    };
    ExpandedTokenOriginIndex {
        path_token_origin_rows: BTreeMap::from([(
            "/workspace/.parseltongue/macro-source/ab/expanded.rs".to_owned(),
            vec![
                token([0, 4], [100, 104], "definition"),
                token([5, 7], [900, 902], "invocation"),
                token([9, 12], [110, 113], "definition"),
            ],
        )]),
    }
}

/// REQ-RUST-052.0 G3: the first and the last token inside a site are found apart; a site
/// that is one token names it twice.
#[test]
fn test_site_token_origin_pair() {
    let index = make_token_origin_index();
    let path = "/workspace/.parseltongue/macro-source/ab/expanded.rs";
    let ranges = |start, end| {
        let (first, last) = index.locate_site_token_pair(path, start, end).unwrap();
        (
            first["token_original_byte_range"].clone(),
            first["token_source_origin_kind"].clone(),
            last["token_original_byte_range"].clone(),
            last["token_source_origin_kind"].clone(),
        )
    };
    assert_eq!(
        ranges(5, 12),
        (
            json!([900, 902]),
            json!("invocation"),
            json!([110, 113]),
            json!("definition")
        )
    );
    assert_eq!(
        ranges(0, 4),
        (
            json!([100, 104]),
            json!("definition"),
            json!([100, 104]),
            json!("definition")
        )
    );
}

/// REQ-RUST-052.0 G3: a site with no stored expansion, or with no token inside it, is a
/// typed error, never an origin.
#[test]
fn test_site_origin_missing_rejected() {
    let index = make_token_origin_index();
    let path = "/workspace/.parseltongue/macro-source/ab/expanded.rs";
    assert!(matches!(
        index.locate_site_token_pair("/workspace/other/expanded.rs", 0, 4),
        Err(ImplHeaderSourceError::InvalidReference { .. })
    ));
    assert!(matches!(
        index.locate_site_token_pair(path, 4, 5),
        Err(ImplHeaderSourceError::InvalidReference { .. })
    ));
    assert!(
        matches!(
            index.locate_site_token_pair(path, 1, 12),
            Err(ImplHeaderSourceError::InvalidReference { .. })
        ),
        "a site that starts inside a token is not a site of this text"
    );
}

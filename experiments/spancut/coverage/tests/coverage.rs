use coverage::*;

fn gap_kinds(c: &Coverage) -> Vec<GapKind> {
    c.gaps.iter().map(|g| g.kind).collect()
}

#[test]
fn test_sum_equals_file_length() {
    let src = "pub fn a() {}\n\npub struct B;\n";
    let c = cover_file(src);
    assert_eq!(c.file_len, src.len());
    assert_eq!(covered_bytes(&c), src.len());
    assert!(is_identity(&c));
    assert_eq!(c.items.len(), 2);
}

#[test]
fn test_empty_file_identity_holds() {
    let c = cover_file("");
    assert!(is_identity(&c));
    assert_eq!(covered_bytes(&c), 0);
}

#[test]
fn test_gap_whitespace_comment_classified() {
    let src = "pub fn a() {}\n// loose comment\n\npub fn b() {}\n";
    let c = cover_file(src);
    assert!(is_identity(&c));
    let kinds = gap_kinds(&c);
    assert!(kinds.contains(&GapKind::Whitespace), "{kinds:?}");
    assert!(kinds.contains(&GapKind::Comment), "{kinds:?}");
    assert!(!kinds.contains(&GapKind::Other), "{kinds:?}");
}

#[test]
fn test_inner_attr_gap_classified() {
    let src = "//! crate docs\n#![allow(dead_code)]\n\npub fn a() {}\n";
    let c = cover_file(src);
    assert!(is_identity(&c));
    let kinds = gap_kinds(&c);
    assert!(kinds.contains(&GapKind::InnerAttr), "{kinds:?}");
    assert!(kinds.contains(&GapKind::Doc), "{kinds:?}");
    assert!(!kinds.contains(&GapKind::Other), "{kinds:?}");
}

#[test]
fn test_module_docs_not_other() {
    let src = "//! line one\n//! line two\n\npub struct S;\n";
    let c = cover_file(src);
    assert!(is_identity(&c));
    let docs: usize = c
        .gaps
        .iter()
        .filter(|g| g.kind == GapKind::Doc)
        .map(|g| g.end - g.start)
        .sum();
    assert_eq!(docs, "//! line one".len() + "//! line two".len());
    assert!(!gap_kinds(&c).contains(&GapKind::Other));
}

#[test]
fn test_error_region_gap_flagged() {
    let src = "pub fn a() {}\n}}}\npub fn b() {}\n";
    let c = cover_file(src);
    assert!(is_identity(&c));
    assert!(gap_kinds(&c).contains(&GapKind::Error), "{:?}", c.gaps);
}

#[test]
fn test_nested_items_not_module_level() {
    let src = "pub fn outer() {\n    fn inner() {}\n    struct S;\n}\n";
    let c = cover_file(src);
    assert!(is_identity(&c));
    assert_eq!(c.items.len(), 1);
    assert_eq!(c.items[0].kind, "fn");
}

#[test]
fn test_items_sorted_non_overlapping() {
    let src = "use a::b;\nconst X: u8 = 1;\nstatic Y: u8 = 2;\ntype Z = u8;\nmacro_rules! m { () => {} }\nextern crate core;\n";
    let c = cover_file(src);
    assert!(is_identity(&c));
    assert_eq!(c.items.len(), 6);
    for w in c.items.windows(2) {
        assert!(w[0].end <= w[1].start);
    }
}

#[test]
fn test_gap_bytes_sum_matches() {
    let src = "pub fn a() {}\n\n// c\npub fn b() {}\n";
    let c = cover_file(src);
    let by_kind: usize = gap_bytes_by_kind(&c).iter().map(|(_, n)| n).sum();
    let gaps: usize = c.gaps.iter().map(|g| g.end - g.start).sum();
    assert_eq!(by_kind, gaps);
    let items: usize = c.items.iter().map(|i| i.end - i.start).sum();
    assert_eq!(items + gaps, src.len());
}

#[test]
fn test_frontmatter_shebang_classified() {
    let src = "#!/usr/bin/env cargo\n---\n[dependencies]\nclap = \"4\"\n---\n\nfn main() {}\n";
    let c = cover_file(src);
    assert!(is_identity(&c));
    let kinds = gap_kinds(&c);
    assert!(kinds.contains(&GapKind::Frontmatter), "{kinds:?}");
    assert!(!kinds.contains(&GapKind::Other), "{kinds:?}");
}

#[test]
fn test_by_kind_covers_every_gap() {
    let src = "#!/usr/bin/env cargo\n---\n[dependencies]\n---\n//! docs\n#![allow(unused)]\n// c\n}}\nfn main() {}\n";
    let c = cover_file(src);
    let by_kind: usize = gap_bytes_by_kind(&c).iter().map(|(_, n)| n).sum();
    let gaps: usize = c.gaps.iter().map(|g| g.end - g.start).sum();
    assert_eq!(
        by_kind, gaps,
        "every gap kind must be reported: {:?}",
        c.gaps
    );
}

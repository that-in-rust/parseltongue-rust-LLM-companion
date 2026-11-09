//! usage: coverage <file-list> [base-dir]  → one TSV line per file + a summary on stderr.
use coverage::{cover_file, gap_bytes_by_kind, is_identity, GapKind};
use std::collections::BTreeMap;

fn main() {
    let list = std::env::args().nth(1).expect("file list");
    let base = std::env::args().nth(2).unwrap_or_default();
    let mut files = 0usize;
    let mut fails = 0usize;
    let mut total_len = 0usize;
    let mut item_bytes = 0usize;
    let mut by_kind: BTreeMap<String, usize> = BTreeMap::new();
    let mut files_with_error = 0usize;
    let mut files_with_other = 0usize;
    println!("file\tlen\titems\titem_bytes\tws\tcomment\tdoc\tfrontmatter\tinner_attr\terror\tother\tidentity");
    for line in std::fs::read_to_string(&list).expect("read list").lines() {
        let path = if base.is_empty() || line.starts_with('/') {
            line.to_string()
        } else {
            format!("{base}/{line}")
        };
        let Ok(src) = std::fs::read_to_string(&path) else {
            continue;
        };
        let c = cover_file(&src);
        let ok = is_identity(&c);
        let gk = gap_bytes_by_kind(&c);
        let get = |k: GapKind| {
            gk.iter()
                .find(|(kk, _)| *kk == k)
                .map(|(_, n)| *n)
                .unwrap_or(0)
        };
        let ib: usize = c.items.iter().map(|i| i.end - i.start).sum();
        files += 1;
        total_len += c.file_len;
        item_bytes += ib;
        if !ok {
            fails += 1;
        }
        if get(GapKind::Error) > 0 {
            files_with_error += 1;
        }
        if get(GapKind::Other) > 0 {
            files_with_other += 1;
        }
        for (k, n) in &gk {
            *by_kind.entry(format!("{k:?}")).or_default() += n;
        }
        println!(
            "{line}\t{}\t{}\t{ib}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            c.file_len,
            c.items.len(),
            get(GapKind::Whitespace),
            get(GapKind::Comment),
            get(GapKind::Doc),
            get(GapKind::Frontmatter),
            get(GapKind::InnerAttr),
            get(GapKind::Error),
            get(GapKind::Other),
            if ok { "ok" } else { "FAIL" }
        );
    }
    eprintln!("files={files} identity_failures={fails} total_bytes={total_len} item_bytes={item_bytes} ({:.2} %)", 100.0 * item_bytes as f64 / total_len.max(1) as f64);
    eprintln!("gap bytes by kind: {by_kind:?}");
    eprintln!("files_with_error_gap={files_with_error} files_with_other_gap={files_with_other}");
}

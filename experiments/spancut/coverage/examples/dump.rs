use ra_ap_syntax::{AstNode, Edition, NodeOrToken, SourceFile};
fn main() {
    let path = std::env::args().nth(1).unwrap();
    let src = if path == "-" {
        "//! crate docs\n#![allow(dead_code)]\n\npub fn a() {}\n".to_string()
    } else {
        std::fs::read_to_string(&path).unwrap()
    };
    let file = SourceFile::parse(&src, Edition::Edition2021).tree();
    for ch in file.syntax().children_with_tokens() {
        let r = ch.text_range();
        let txt: String = src[usize::from(r.start())..usize::from(r.end())]
            .chars()
            .take(60)
            .collect();
        match ch {
            NodeOrToken::Node(n) => println!(
                "NODE  {:?}\t{}..{}\t{:?}",
                n.kind(),
                usize::from(r.start()),
                usize::from(r.end()),
                txt.replace('\n', "⏎")
            ),
            NodeOrToken::Token(t) => println!(
                "TOKEN {:?}\t{}..{}\t{:?}",
                t.kind(),
                usize::from(r.start()),
                usize::from(r.end()),
                txt.replace('\n', "⏎")
            ),
        }
    }
}

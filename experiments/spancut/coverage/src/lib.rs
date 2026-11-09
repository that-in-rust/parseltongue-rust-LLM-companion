//! Byte-coverage invariant: for one file, module-level item spans + classified gaps == file length,
//! with no overlaps and no unexplained bytes.

use ra_ap_syntax::{ast, AstNode, Edition, NodeOrToken, SourceFile, SyntaxKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub start: usize,
    pub end: usize,
    pub kind: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GapKind {
    Whitespace,
    Comment,
    Doc,
    Frontmatter,
    InnerAttr,
    Error,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gap {
    pub start: usize,
    pub end: usize,
    pub kind: GapKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coverage {
    pub file_len: usize,
    pub items: Vec<Item>,
    pub gaps: Vec<Gap>,
}

fn kind_name(item: &ast::Item) -> &'static str {
    match item {
        ast::Item::Fn(_) => "fn",
        ast::Item::Struct(_) => "struct",
        ast::Item::Enum(_) => "enum",
        ast::Item::Union(_) => "union",
        ast::Item::Trait(_) => "trait",
        ast::Item::Impl(_) => "impl",
        ast::Item::Module(_) => "mod",
        ast::Item::Use(_) => "use",
        ast::Item::Const(_) => "const",
        ast::Item::Static(_) => "static",
        ast::Item::TypeAlias(_) => "type",
        ast::Item::MacroRules(_) => "macro_rules",
        ast::Item::MacroDef(_) => "macro_def",
        ast::Item::MacroCall(_) => "macro_call",
        ast::Item::ExternBlock(_) => "extern_block",
        ast::Item::ExternCrate(_) => "extern_crate",
        _ => "other",
    }
}

/// Parse one file and return every module-level item range plus every byte between them, classified.
pub fn cover_file(src: &str) -> Coverage {
    let parse = SourceFile::parse(src, Edition::Edition2021);
    let file = parse.tree();
    let mut items = Vec::new();
    let mut gaps: Vec<Gap> = Vec::new();
    for child in file.syntax().children_with_tokens() {
        let range = child.text_range();
        let (start, end) = (usize::from(range.start()), usize::from(range.end()));
        match child {
            NodeOrToken::Node(node) => {
                if let Some(item) = ast::Item::cast(node.clone()) {
                    items.push(Item {
                        start,
                        end,
                        kind: kind_name(&item).to_string(),
                    });
                    continue;
                }
                let kind = match node.kind() {
                    SyntaxKind::ATTR => GapKind::InnerAttr,
                    SyntaxKind::DOC_COMMENT => GapKind::Doc,
                    SyntaxKind::ERROR => GapKind::Error,
                    _ => GapKind::Other,
                };
                push_gap(&mut gaps, start, end, kind);
            }
            NodeOrToken::Token(token) => {
                let kind = match token.kind() {
                    SyntaxKind::WHITESPACE => GapKind::Whitespace,
                    SyntaxKind::COMMENT => GapKind::Comment,
                    SyntaxKind::SHEBANG => GapKind::Frontmatter,
                    SyntaxKind::FRONTMATTER => GapKind::Frontmatter,
                    _ => GapKind::Other,
                };
                push_gap(&mut gaps, start, end, kind);
            }
        }
    }
    Coverage {
        file_len: src.len(),
        items,
        gaps,
    }
}

fn push_gap(gaps: &mut Vec<Gap>, start: usize, end: usize, kind: GapKind) {
    if start == end {
        return;
    }
    if let Some(last) = gaps.last_mut() {
        if last.end == start && last.kind == kind {
            last.end = end;
            return;
        }
    }
    gaps.push(Gap { start, end, kind });
}

/// Bytes accounted for by items plus gaps.
pub fn covered_bytes(c: &Coverage) -> usize {
    c.items.iter().map(|i| i.end - i.start).sum::<usize>()
        + c.gaps.iter().map(|g| g.end - g.start).sum::<usize>()
}

/// True when items and gaps tile the file exactly: sorted, non-overlapping, contiguous, summing to file_len.
pub fn is_identity(c: &Coverage) -> bool {
    let mut ranges: Vec<(usize, usize)> = c
        .items
        .iter()
        .map(|i| (i.start, i.end))
        .chain(c.gaps.iter().map(|g| (g.start, g.end)))
        .collect();
    ranges.sort_unstable();
    let mut cursor = 0;
    for (s, e) in ranges {
        if s != cursor || e < s {
            return false;
        }
        cursor = e;
    }
    cursor == c.file_len && covered_bytes(c) == c.file_len
}

/// Bytes per gap kind.
pub fn gap_bytes_by_kind(c: &Coverage) -> Vec<(GapKind, usize)> {
    let kinds = [
        GapKind::Whitespace,
        GapKind::Comment,
        GapKind::Doc,
        GapKind::Frontmatter,
        GapKind::InnerAttr,
        GapKind::Error,
        GapKind::Other,
    ];
    kinds
        .iter()
        .map(|k| {
            (
                *k,
                c.gaps
                    .iter()
                    .filter(|g| g.kind == *k)
                    .map(|g| g.end - g.start)
                    .sum(),
            )
        })
        .collect()
}

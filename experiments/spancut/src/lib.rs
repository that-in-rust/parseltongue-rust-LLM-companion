//! spancut — lexer + ITEM_RECOVERY_SET span cutter (GREEN phase: minimum implementation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Fn,
    Struct,
    Enum,
    Trait,
    Impl,
    Mod,
    Use,
    Const,
    Static,
    Type,
    Macro,
    MacroCall,
    Union,
    ExternBlock,
    ExternCrate,
    Error,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Vis {
    Pub,
    PubRestricted,
    Priv,
    NotApplicable,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub kind: Kind,
    pub name: String,
    pub vis: Vis,
    pub start: usize,
    pub end: usize,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
enum Tk {
    Id(String),
    Punct(char),
    Str,
    Char,
    Life,
    Doc,
    InnerDoc,
    Num,
}
#[derive(Debug, Clone)]
struct Tok {
    tk: Tk,
    start: usize,
    end: usize,
}

fn lex(src: &str) -> Vec<Tok> {
    let b = src.as_bytes();
    let n = b.len();
    let mut i = 0;
    let mut out = Vec::new();
    let push = |out: &mut Vec<Tok>, tk, s, e| {
        out.push(Tok {
            tk,
            start: s,
            end: e,
        })
    };
    while i < n {
        let c = b[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if b[i..].starts_with(b"//") {
            let j = src[i..].find('\n').map(|k| i + k).unwrap_or(n);
            if b[i..].starts_with(b"//!") {
                push(&mut out, Tk::InnerDoc, i, j);
            } else if b[i..].starts_with(b"///") {
                push(&mut out, Tk::Doc, i, j);
            }
            i = j;
            continue;
        }
        if b[i..].starts_with(b"/*") {
            let mut d = 0;
            let mut j = i;
            while j < n {
                if b[j..].starts_with(b"/*") {
                    d += 1;
                    j += 2;
                } else if b[j..].starts_with(b"*/") {
                    d -= 1;
                    j += 2;
                    if d == 0 {
                        break;
                    }
                } else {
                    j += 1;
                }
            }
            i = j;
            continue;
        }
        // raw strings r"…", r#"…"#, br"…"
        {
            let mut k = i;
            if b[k] == b'b' {
                k += 1;
            }
            if k < n && b[k] == b'r' {
                let mut h = k + 1;
                while h < n && b[h] == b'#' {
                    h += 1;
                }
                if h < n && b[h] == b'"' {
                    let hashes = h - (k + 1);
                    let close = format!("\"{}", "#".repeat(hashes));
                    let j = src[h + 1..]
                        .find(&close)
                        .map(|x| h + 1 + x + close.len())
                        .unwrap_or(n);
                    push(&mut out, Tk::Str, i, j);
                    i = j;
                    continue;
                }
            }
        }
        if c == b'"' || (c == b'b' && i + 1 < n && b[i + 1] == b'"') {
            let mut j = i + if c == b'b' { 2 } else { 1 };
            while j < n && b[j] != b'"' {
                j += if b[j] == b'\\' { 2 } else { 1 };
            }
            let j = (j + 1).min(n);
            push(&mut out, Tk::Str, i, j);
            i = j;
            continue;
        }
        if c == b'\'' || (c == b'b' && i + 1 < n && b[i + 1] == b'\'') {
            let k = i + if c == b'b' { 2 } else { 1 };
            if k < n && b[k] == b'\\' {
                let j = src[k + 1..].find('\'').map(|x| k + 2 + x).unwrap_or(n);
                push(&mut out, Tk::Char, i, j);
                i = j;
                continue;
            }
            // char literal: one UTF-8 char followed by '
            let ch_len = src[k..].chars().next().map(|ch| ch.len_utf8()).unwrap_or(1);
            if k + ch_len < n && b[k + ch_len] == b'\'' {
                push(&mut out, Tk::Char, i, k + ch_len + 1);
                i = k + ch_len + 1;
                continue;
            }
            let mut j = k;
            while j < n && (b[j].is_ascii_alphanumeric() || b[j] == b'_') {
                j += 1;
            }
            push(&mut out, Tk::Life, i, j);
            i = j.max(i + 1);
            continue;
        }
        if c.is_ascii_alphabetic() || c == b'_' || c >= 0x80 {
            let mut j = i;
            while j < n && (b[j].is_ascii_alphanumeric() || b[j] == b'_' || b[j] >= 0x80) {
                j += 1;
            }
            push(&mut out, Tk::Id(src[i..j].to_string()), i, j);
            i = j;
            continue;
        }
        if c.is_ascii_digit() {
            let mut j = i;
            while j < n && (b[j].is_ascii_alphanumeric() || b[j] == b'_' || b[j] == b'.') {
                j += 1;
            }
            push(&mut out, Tk::Num, i, j);
            i = j;
            continue;
        }
        push(&mut out, Tk::Punct(c as char), i, i + 1);
        i += 1;
    }
    out
}

const ITEM_KW: &[&str] = &[
    "fn",
    "struct",
    "enum",
    "impl",
    "trait",
    "const",
    "static",
    "mod",
    "use",
    "macro_rules",
    "union",
    "type",
    "extern",
    "macro",
];
const MODS: &[&str] = &["pub", "async", "unsafe", "const", "extern", "default"];
const RECOVERY: &[&str] = &[
    "fn",
    "struct",
    "enum",
    "impl",
    "trait",
    "const",
    "static",
    "mod",
    "use",
    "macro_rules",
    "union",
    "type",
    "extern",
    "macro",
    "pub",
    "crate",
    "let",
    "async",
    "unsafe",
];
fn is_id(t: &Tok, s: &str) -> bool {
    matches!(&t.tk, Tk::Id(x) if x == s)
}
fn is_p(t: &Tok, c: char) -> bool {
    matches!(&t.tk, Tk::Punct(x) if *x == c)
}
fn kind_of(s: &str) -> Kind {
    match s {
        "fn" => Kind::Fn,
        "struct" => Kind::Struct,
        "enum" => Kind::Enum,
        "impl" => Kind::Impl,
        "trait" => Kind::Trait,
        "const" => Kind::Const,
        "static" => Kind::Static,
        "mod" => Kind::Mod,
        "use" => Kind::Use,
        "macro_rules" | "macro" => Kind::Macro,
        "union" => Kind::Union,
        "type" => Kind::Type,
        "extern" => Kind::ExternBlock,
        _ => Kind::Error,
    }
}

pub fn cut_item_spans(src: &str) -> Vec<Span> {
    let t = lex(src);
    let n = t.len();
    let mut spans = Vec::new();
    let mut i = 0;
    while i < n {
        // inner docs / inner attributes (`//!`, `#![..]`) are module-level: skip, they are not part of the next item
        if t[i].tk == Tk::InnerDoc {
            i += 1;
            continue;
        }
        if is_p(&t[i], '#') && i + 1 < n && is_p(&t[i + 1], '!') {
            let mut q = i + 2;
            let mut d = 0;
            while q < n {
                if is_p(&t[q], '[') {
                    d += 1;
                } else if is_p(&t[q], ']') {
                    d -= 1;
                }
                q += 1;
                if d == 0 {
                    break;
                }
            }
            i = q;
            continue;
        }
        // attributes and docs: find first non-attr token j
        let start = t[i].start;
        let mut j = i;
        loop {
            if j >= n {
                break;
            }
            if t[j].tk == Tk::Doc {
                j += 1;
                continue;
            }
            if is_p(&t[j], '#') {
                j += 1;
                if j < n && is_p(&t[j], '!') {
                    j += 1;
                }
                if j < n && is_p(&t[j], '[') {
                    let mut d = 0;
                    while j < n {
                        if is_p(&t[j], '[') {
                            d += 1;
                        } else if is_p(&t[j], ']') {
                            d -= 1;
                        }
                        j += 1;
                        if d == 0 {
                            break;
                        }
                    }
                }
                continue;
            }
            break;
        }
        if j >= n {
            break;
        }
        // visibility and modifiers
        let mut vis = Vis::Priv;
        let mut m = j;
        while m < n {
            if let Tk::Id(x) = &t[m].tk {
                if MODS.contains(&x.as_str()) {
                    if x == "pub" {
                        vis = Vis::Pub;
                        if m + 1 < n && is_p(&t[m + 1], '(') {
                            let mut d = 0;
                            let mut q = m + 1;
                            while q < n {
                                if is_p(&t[q], '(') {
                                    d += 1;
                                } else if is_p(&t[q], ')') {
                                    d -= 1;
                                }
                                q += 1;
                                if d == 0 {
                                    break;
                                }
                            }
                            vis = Vis::PubRestricted;
                            m = q;
                            continue;
                        }
                    }
                    // `const fn` / `unsafe fn` / `extern "C" fn`: only a modifier if followed by an item keyword eventually
                    if x == "extern"
                        && m + 1 < n
                        && (is_p(&t[m + 1], '{')
                            || (t[m + 1].tk == Tk::Str && m + 2 < n && is_p(&t[m + 2], '{')))
                    {
                        break;
                    }
                    if x == "extern" && m + 1 < n && t[m + 1].tk == Tk::Str {
                        m += 2;
                        continue;
                    }
                    if (x == "const" || x == "extern")
                        && !(m + 1 < n
                            && (matches!(&t[m+1].tk, Tk::Id(y) if ITEM_KW.contains(&y.as_str()) || MODS.contains(&y.as_str()))
                                || t[m + 1].tk == Tk::Str))
                    {
                        break;
                    }
                    m += 1;
                    continue;
                }
            }
            break;
        }
        if m < n {
            if let Tk::Id(kw) = &t[m].tk {
                if ITEM_KW.contains(&kw.as_str()) {
                    let mut kind = kind_of(kw);
                    if kind == Kind::ExternBlock && m + 1 < n && is_id(&t[m + 1], "crate") {
                        kind = Kind::ExternCrate;
                    }
                    let semicolon_only = matches!(
                        kind,
                        Kind::Use | Kind::Const | Kind::Static | Kind::Type | Kind::ExternCrate
                    );
                    let name_at = if (kw == "macro_rules" && m + 1 < n && is_p(&t[m + 1], '!'))
                        || kind == Kind::ExternCrate
                    {
                        m + 2
                    } else {
                        m + 1
                    };
                    let name = if kind == Kind::Impl {
                        String::new()
                    } else if name_at < n {
                        if let Tk::Id(nm) = &t[name_at].tk {
                            nm.clone()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };
                    let item_col = line_col(src, start).1;
                    let mut d: i32 = 0;
                    let mut q = m + 1;
                    let mut end = None;
                    let mut conf = 1.0f32;
                    let mut seen_brace = false;
                    let mut cur_vis = vis.clone();
                    if kind == Kind::Impl || kind == Kind::ExternBlock || kind == Kind::Macro {
                        cur_vis = Vis::NotApplicable;
                    }
                    while q < n {
                        match &t[q].tk {
                            Tk::Punct(c) if "([{".contains(*c) => {
                                d += 1;
                                if *c == '{' {
                                    seen_brace = true;
                                }
                            }
                            Tk::Punct(c) if ")]}".contains(*c) => {
                                d -= 1;
                                if d < 0 {
                                    conf = 0.3;
                                    end = Some(t[q].start);
                                    break;
                                }
                                if d == 0 && *c == '}' && seen_brace && !semicolon_only {
                                    end = Some(t[q].end);
                                    q += 1;
                                    break;
                                }
                            }
                            Tk::Punct(';') if d == 0 => {
                                end = Some(t[q].end);
                                q += 1;
                                break;
                            }
                            Tk::Id(x)
                                if d == 0
                                    && q > m + 1
                                    && [
                                        "fn",
                                        "struct",
                                        "enum",
                                        "impl",
                                        "trait",
                                        "mod",
                                        "use",
                                        "macro_rules",
                                    ]
                                    .contains(&x.as_str())
                                    && !matches!(&t[q-1].tk, Tk::Id(p) if MODS.contains(&p.as_str()) || p == "dyn" || p == "where")
                                    && !matches!(&t[q-1].tk, Tk::Punct(c) if "><+:=&(,".contains(*c)) =>
                            {
                                conf = 0.3;
                                end = Some(t[q].start);
                                break;
                            }
                            // indentation resync: an item keyword at the SAME column as this item's start while we are still inside
                            // its braces means a `}` is missing above — close grey here (heuristic; rust-analyzer would nest it)
                            Tk::Id(x)
                                if d > 0
                                    && RECOVERY.contains(&x.as_str())
                                    && line_col(src, t[q].start).1 == item_col
                                    && src[..t[q].start]
                                        .rfind('\n')
                                        .map(|p| src[p + 1..t[q].start].trim().is_empty())
                                        .unwrap_or(false) =>
                            {
                                conf = 0.3;
                                end = Some(t[q].start);
                                break;
                            }
                            Tk::Punct('#')
                                if d == 0
                                    && q > m + 1
                                    && !seen_brace
                                    && q + 1 < n
                                    && is_p(&t[q + 1], '[') =>
                            {
                                conf = 0.3;
                                end = Some(t[q].start);
                                break;
                            }
                            // an outer attribute or doc comment first on its line at the item's column while still open: next item begins
                            Tk::Punct('#') | Tk::Doc
                                if d > 0
                                    && line_col(src, t[q].start).1 == item_col
                                    && src[..t[q].start]
                                        .rfind('\n')
                                        .map(|p| src[p + 1..t[q].start].trim().is_empty())
                                        .unwrap_or(false)
                                    && (t[q].tk == Tk::Doc
                                        || (q + 1 < n && is_p(&t[q + 1], '['))) =>
                            {
                                conf = 0.3;
                                end = Some(t[q].start);
                                break;
                            }
                            _ => {}
                        }
                        q += 1;
                    }
                    let end = end.unwrap_or_else(|| {
                        conf = 0.3;
                        t[n - 1].end
                    });
                    spans.push(Span {
                        kind,
                        name,
                        vis: cur_vis,
                        start,
                        end,
                        confidence: conf,
                    });
                    i = q.max(j + 1);
                    continue;
                }
            }
        }
        // macro invocation at item level: name! ( … ) ;  |  name! { … }
        // macro path: a (:: b)* !
        let mut pe = m;
        while pe + 2 < n
            && matches!(&t[pe].tk, Tk::Id(_))
            && is_p(&t[pe + 1], ':')
            && is_p(&t[pe + 2], ':')
        {
            pe += 3;
        }
        if pe + 1 < n && matches!(&t[pe].tk, Tk::Id(_)) && is_p(&t[pe + 1], '!') {
            let name = if let Tk::Id(x) = &t[pe].tk {
                x.clone()
            } else {
                String::new()
            };
            let mut d = 0;
            let mut q = pe + 2;
            let mut end = None;
            while q < n {
                if matches!(&t[q].tk, Tk::Punct(c) if "([{".contains(*c)) {
                    d += 1;
                } else if matches!(&t[q].tk, Tk::Punct(c) if ")]}".contains(*c)) {
                    d -= 1;
                    if d == 0 {
                        end = Some(t[q].end);
                        q += 1;
                        break;
                    }
                }
                q += 1;
            }
            let mut conf = 0.9;
            let mut end = end.unwrap_or_else(|| {
                conf = 0.3;
                t[n - 1].end
            });
            if q < n && is_p(&t[q], ';') {
                end = t[q].end;
                q += 1;
            }
            spans.push(Span {
                kind: Kind::MacroCall,
                name,
                vis: Vis::NotApplicable,
                start,
                end,
                confidence: conf,
            });
            i = q;
            continue;
        }
        // garbage: resync to the next recovery token
        let mut q = j + 1;
        while q < n
            && !matches!(&t[q].tk, Tk::Id(x) if RECOVERY.contains(&x.as_str()))
            && !is_p(&t[q], '#')
        {
            q += 1;
        }
        let end = if q > 0 { t[q - 1].end } else { t[j].end };
        spans.push(Span {
            kind: Kind::Error,
            name: String::new(),
            vis: Vis::NotApplicable,
            start: t[j].start,
            end,
            confidence: 0.0,
        });
        i = q;
    }
    spans
}
pub fn line_col(src: &str, off: usize) -> (usize, usize) {
    let line = src[..off].matches('\n').count() + 1;
    let col = off - src[..off].rfind('\n').map(|p| p + 1).unwrap_or(0);
    (line, col)
}

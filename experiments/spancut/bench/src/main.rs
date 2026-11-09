use ra_ap_syntax::{ast::HasModuleItem, AstNode, Edition, SourceFile};
use std::time::Instant;
fn main() {
    let list = std::env::args().nth(1).unwrap();
    let threads: usize = std::env::args().nth(2).map(|s| s.parse().unwrap()).unwrap_or(1);
    let files: Vec<String> = std::fs::read_to_string(list).unwrap().lines().map(|s| s.to_string()).collect();
    let t0 = Instant::now();
    let srcs: Vec<String> = files.iter().filter_map(|f| std::fs::read_to_string(f).ok()).collect();
    let bytes: usize = srcs.iter().map(|s| s.len()).sum();
    let lines: usize = srcs.iter().map(|s| s.matches('\n').count()).sum();
    eprintln!("read {} files {} MB {} lines in {:.2}s", srcs.len(), bytes / 1_000_000, lines, t0.elapsed().as_secs_f64());
    let run = |name: &str, f: &(dyn Fn(&str) -> usize + Sync)| {
        let t = Instant::now();
        let n: usize = std::thread::scope(|sc| {
            let chunk = srcs.len().div_ceil(threads);
            let hs: Vec<_> = srcs.chunks(chunk).map(|c| sc.spawn(move || c.iter().map(|s| f(s)).sum::<usize>())).collect();
            hs.into_iter().map(|h| h.join().unwrap()).sum()
        });
        let s = t.elapsed().as_secs_f64();
        println!("{name:14} threads {threads}  items {n:8}  {:.2} s  {:.1} MB/s  {:.0} klines/s", s, bytes as f64 / 1e6 / s, lines as f64 / 1e3 / s);
    };
    run("spancut", &|s| spancut::cut_item_spans(s).len());
    run("ra_ap_syntax", &|s| SourceFile::parse(s, Edition::Edition2021).tree().items().count());
    run("ra_ap+walk", &|s| SourceFile::parse(s, Edition::Edition2021).tree().syntax().descendants().count());
    run("syn", &|s| syn::parse_file(s).map(|f| f.items.len()).unwrap_or(0));
}

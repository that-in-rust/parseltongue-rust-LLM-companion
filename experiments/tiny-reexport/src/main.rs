use anyhow::{ensure, Result};
use serde_json::json;
use std::io::Read;
use tiny_reexport::{facts::Facts, model};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 2 && matches!(args[0].as_str(), "model" | "native"),
        "usage: tiny-reexport native|model FILE|-"
    );
    let input: Box<dyn Read> = if args[1] == "-" {
        Box::new(std::io::stdin().lock())
    } else {
        Box::new(std::fs::File::open(&args[1])?)
    };
    let mut source = String::new();
    input.take(65537).read_to_string(&mut source)?;
    ensure!(source.len() <= 65536, "source exceeds 64 KiB limit");
    let facts = Facts::parse_source_into_facts(&source)?;
    let mut session = (args[0] == "model")
        .then(model::load_cpu_model_session)
        .transpose()?;
    let mut graph = Vec::new();
    for (function, spelling, address) in &facts.queries {
        let target = match &mut session {
            Some(s) => model::resolve_model_return_target(s, &facts, *address)?,
            None => facts.resolve_native_return_target(*address),
        };
        graph.push(json!({"from":function,"role":"return","spelling":spelling,
            "target":target.map(|i| &facts.names[i])}));
    }
    println!("{}", serde_json::to_string(&graph)?);
    Ok(())
}

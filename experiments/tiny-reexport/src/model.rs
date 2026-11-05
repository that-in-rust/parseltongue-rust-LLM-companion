use crate::facts::{Facts, SLOTS};
use anyhow::{bail, ensure, Context, Result};
use ort::{session::Session, value::Tensor};

pub fn load_cpu_model_session() -> Result<Session> {
    let library = std::env::var_os("ORT_DYLIB_PATH")
        .context("set ORT_DYLIB_PATH to the ONNX Runtime 1.24 shared library")?;
    ort::init_from(library)?.commit();
    Ok(Session::builder()?
        .with_intra_threads(1)
        .map_err(ort::Error::<()>::from)?
        .with_inter_threads(1)
        .map_err(ort::Error::<()>::from)?
        .commit_from_memory(include_bytes!("../resolver.onnx"))?)
}

pub fn resolve_model_return_target(
    session: &mut Session,
    facts: &Facts,
    query: usize,
) -> Result<Option<usize>> {
    let defs = Tensor::from_array(([1, SLOTS], facts.defs.map(f32::from).to_vec()))?;
    let aliases = Tensor::from_array((
        [SLOTS, SLOTS],
        facts
            .aliases
            .iter()
            .flatten()
            .copied()
            .map(f32::from)
            .collect::<Vec<_>>(),
    ))?;
    let mut address = vec![0f32; SLOTS];
    address[query] = 1.;
    let outputs = session.run(ort::inputs![
        "defs" => defs, "aliases" => aliases, "query" => Tensor::from_array(([1, SLOTS], address))?
    ])?;
    let (_, answer) = outputs["answer"].try_extract_tensor::<f32>()?;
    ensure!(answer.len() == SLOTS, "wrong model output size");
    let mut target = None;
    for (i, &bit) in answer.iter().enumerate() {
        match bit {
            0. => (),
            1. if target.is_none() && i < facts.names.len() => target = Some(i),
            _ => bail!("invalid model output, expected one-hot or zero"),
        }
    }
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_three_address_tables() -> Result<()> {
        let mut session = load_cpu_model_session()?;
        // Each row: missing, terminal, link to 0/1/2, or conflicting links.
        for table_id in 0..6usize.pow(3) {
            let mut table = table_id;
            let mut facts = Facts {
                names: vec!["X".into(), "Y".into(), "Z".into()],
                ..Facts::default()
            };
            for row in 0..3 {
                match table % 6 {
                    0 => (),
                    1 => facts.defs[row] = 1,
                    i @ 2..=4 => facts.aliases[row][i - 2] = 1,
                    _ => {
                        facts.aliases[row][0] = 1;
                        facts.aliases[row][1] = 1;
                    }
                }
                table /= 6;
            }
            for query in 0..3 {
                assert_eq!(
                    resolve_model_return_target(&mut session, &facts, query)?,
                    facts.resolve_native_return_target(query),
                    "table {table_id}, query {query}"
                );
            }
        }
        Ok(())
    }
}

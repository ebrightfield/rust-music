use anyhow::{Context, Result};
use music::note::pitch_class::Pc;
use music::note_collections::pc_set::{PcContent, PcShape};
use music::svg::IntervalBuilder;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::input::{parse_input_to_pcs, pc_label, resolve_theme};

pub struct LinearIntervalArgs {
    pub input: Vec<String>,
    pub output: Option<String>,
    pub theme: Option<String>,
    pub title: Option<String>,
    pub verbose: bool,
}

pub fn run_linear(args: LinearIntervalArgs) -> Result<()> {
    let pcs = PcContent::new(parse_input_to_pcs(&args.input)?);
    anyhow::ensure!(
        pcs.len() >= 2,
        "linear interval diagrams need at least two pitches"
    );
    if let Some(path) = &args.output {
        let ext = Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("svg");
        anyhow::ensure!(
            ext == "svg",
            "interval-linear only supports .svg output (got .{ext})"
        );
    }

    let shape: PcShape = pcs.to_shape();
    let mut builder = IntervalBuilder::new()
        .from_pc_shape(&shape)
        .theme(resolve_theme(args.theme.as_deref())?);
    if let Some(title) = &args.title {
        builder = builder.title(title);
    }
    let svg = builder.build_linear();
    match &args.output {
        Some(path) => {
            fs::write(path, &svg).with_context(|| format!("failed to write {path}"))?;
            if args.verbose {
                eprintln!("wrote {path} ({} bytes)", svg.len());
            }
        }
        None => io::stdout().write_all(svg.as_bytes())?,
    }
    Ok(())
}

pub struct IntervalPairsArgs {
    pub input: Vec<String>,
    pub pairs: Vec<String>,
    pub output: Option<String>,
    pub format: Option<String>,
    pub verbose: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PairFormat {
    Text,
    Json,
}

impl PairFormat {
    fn resolve(explicit: Option<&str>, output: Option<&str>) -> Result<Self> {
        let value = explicit.or_else(|| {
            output.and_then(|path| Path::new(path).extension().and_then(|ext| ext.to_str()))
        });
        match value.unwrap_or("text").to_ascii_lowercase().as_str() {
            "text" | "txt" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            other => {
                anyhow::bail!("unsupported interval-pairs format '{other}' (options: text, json)")
            }
        }
    }
}

struct PairResult {
    from: String,
    from_pc: u8,
    to: String,
    to_pc: u8,
    ascending_semitones: u8,
    interval_class: u8,
}

pub fn run_pairs(args: IntervalPairsArgs) -> Result<()> {
    let pcs = unique_sorted(parse_input_to_pcs(&args.input)?);
    anyhow::ensure!(
        pcs.len() >= 2,
        "interval-pairs needs at least two distinct pitches"
    );
    let selected = if args.pairs.is_empty() {
        all_pairs(&pcs)
    } else {
        parse_queries(&args.pairs, &pcs)?
    };
    let results: Vec<PairResult> = selected
        .into_iter()
        .map(|(from, to)| {
            let ascending = (u8::from(to) + 12 - u8::from(from)) % 12;
            PairResult {
                from: pc_label(from).to_string(),
                from_pc: u8::from(from),
                to: pc_label(to).to_string(),
                to_pc: u8::from(to),
                ascending_semitones: ascending,
                interval_class: ascending.min(12 - ascending),
            }
        })
        .collect();

    let format = PairFormat::resolve(args.format.as_deref(), args.output.as_deref())?;
    let rendered = match format {
        PairFormat::Text => {
            results
                .iter()
                .map(|pair| {
                    format!(
                        "{} ({}) -> {} ({}): {} semitones, ic{}",
                        pair.from,
                        pair.from_pc,
                        pair.to,
                        pair.to_pc,
                        pair.ascending_semitones,
                        pair.interval_class
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
                + "\n"
        }
        PairFormat::Json => {
            serde_json::to_string_pretty(
                &results
                    .iter()
                    .map(|pair| {
                        serde_json::json!({
                            "from": pair.from,
                            "from_pc": pair.from_pc,
                            "to": pair.to,
                            "to_pc": pair.to_pc,
                            "ascending_semitones": pair.ascending_semitones,
                            "interval_class": pair.interval_class,
                        })
                    })
                    .collect::<Vec<_>>(),
            )? + "\n"
        }
    };
    match &args.output {
        Some(path) => {
            fs::write(path, &rendered).with_context(|| format!("failed to write {path}"))?
        }
        None => io::stdout().write_all(rendered.as_bytes())?,
    }
    if args.verbose {
        eprintln!("interval-pairs: {} result(s)", results.len());
    }
    Ok(())
}

fn unique_sorted(mut pcs: Vec<Pc>) -> Vec<Pc> {
    pcs.sort_by_key(|pc| u8::from(*pc));
    pcs.dedup();
    pcs
}

fn all_pairs(pcs: &[Pc]) -> Vec<(Pc, Pc)> {
    let mut pairs = Vec::new();
    for (index, from) in pcs.iter().enumerate() {
        for to in &pcs[index + 1..] {
            pairs.push((*from, *to));
        }
    }
    pairs
}

fn parse_queries(values: &[String], members: &[Pc]) -> Result<Vec<(Pc, Pc)>> {
    values
        .iter()
        .map(|value| {
            let pair = parse_input_to_pcs(&[value.clone()])
                .with_context(|| format!("invalid interval pair '{value}'"))?;
            anyhow::ensure!(
                pair.len() == 2,
                "interval pair '{value}' must contain exactly two pitches"
            );
            anyhow::ensure!(
                members.contains(&pair[0]) && members.contains(&pair[1]),
                "interval pair '{value}' contains a pitch outside the input set"
            );
            anyhow::ensure!(
                pair[0] != pair[1],
                "interval pair '{value}' repeats one pitch"
            );
            Ok((pair[0], pair[1]))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_pairs_has_n_choose_two_entries() {
        let pcs = vec![Pc::Pc0, Pc::Pc4, Pc::Pc7];
        assert_eq!(all_pairs(&pcs).len(), 3);
    }

    #[test]
    fn directed_query_preserves_order() {
        let pairs = parse_queries(&["G,C".into()], &[Pc::Pc0, Pc::Pc4, Pc::Pc7]).unwrap();
        assert_eq!(pairs, vec![(Pc::Pc7, Pc::Pc0)]);
    }
}

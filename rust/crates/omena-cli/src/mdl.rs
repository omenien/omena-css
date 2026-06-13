use crate::{io::read_source, output::print_json, paths::path_string};
use omena_query::summarize_omena_query_design_system_minimum_description;
use std::path::PathBuf;

pub(crate) fn compress_file(
    path: PathBuf,
    budget_bits: Option<f64>,
    json: bool,
) -> Result<(), String> {
    let source = read_source(&path)?;
    let rule_count = source.matches('{').count();
    let observation_count = source
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    let source_hash = format!(
        "len{}-sum{}",
        source.len(),
        source.bytes().map(u64::from).sum::<u64>()
    );
    let value_frequencies = css_declaration_value_histogram(&source);
    let summary = summarize_omena_query_design_system_minimum_description(
        path_string(&path),
        source_hash,
        rule_count,
        observation_count,
        &value_frequencies,
    );
    if let Some(budget_bits) = budget_bits
        && summary.total_bits > budget_bits
    {
        return Err(format!(
            "MDL budget exceeded: total_bits={} budget_bits={budget_bits}",
            summary.total_bits
        ));
    }

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("total bits: {}", summary.total_bits);
    println!("model bits: {}", summary.model_bits);
    println!("residual bits: {}", summary.residual_bits);
    println!("unit: {}", summary.unit);
    Ok(())
}

/// Build an empirical value-symbol frequency histogram from CSS declarations.
///
/// Each `prop: value;` declaration contributes its trimmed value string as a
/// symbol; the returned vector is the per-symbol occurrence count. Recurring
/// design-token values peak the histogram (low entropy / compressible); scattered
/// one-off values flatten it. Deterministic and dependency-light.
fn css_declaration_value_histogram(source: &str) -> Vec<usize> {
    let mut counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for line in source.lines() {
        let line = line.trim();
        let Some((_, rhs)) = line.split_once(':') else {
            continue;
        };
        let value = rhs.trim().trim_end_matches([';', '}']).trim();
        if value.is_empty() || value.contains('{') {
            continue;
        }
        *counts.entry(value.to_string()).or_insert(0) += 1;
    }
    counts.into_values().collect()
}

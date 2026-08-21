use std::hint::black_box;
use std::time::{Duration, Instant};

use omena_benchmarks::style_corpus;
use omena_parser::{
    StyleDialect, collect_emission_selector_facts_from_cst, collect_style_fact_collection,
    facts_from_cst, parse, with_omena_parser_fact_collection_instrumentation,
};
use serde::Serialize;

const SAMPLE_BATCH_COUNT: usize = 7;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParserFactCollectionProfileV0 {
    schema_version: &'static str,
    product: &'static str,
    measurement_pin: String,
    release_build: bool,
    operating_system: &'static str,
    architecture: &'static str,
    sample_batch_count: usize,
    samples: Vec<ParserFactCollectionSampleV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParserFactCollectionSampleV0 {
    name: String,
    dialect: &'static str,
    byte_length: usize,
    iterations_per_batch: usize,
    parse_median_nanoseconds: u128,
    preparsed_fact_collection_median_nanoseconds: u128,
    full_collection_median_nanoseconds: u128,
    full_collection_traversal_entry_count: u64,
    full_collection_families: Vec<&'static str>,
    full_collection_registered_family_count: u64,
    full_collection_registered_families: Vec<&'static str>,
    preparsed_fact_to_parse_ratio_milli: u128,
    full_collection_to_parse_ratio_milli: u128,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut inputs = vec![
        profile_input("synthetic-rich-css-12k", StyleDialect::Css, 12 * 1024),
        profile_input("synthetic-rich-css-48k", StyleDialect::Css, 48 * 1024),
        profile_input("synthetic-rich-css-148k", StyleDialect::Css, 148 * 1024),
    ];
    inputs.extend(style_corpus().into_iter().map(|sample| ProfileInput {
        name: format!("workspace-{}", sample.name),
        dialect: sample.dialect,
        source: sample.source,
    }));

    let samples = inputs.into_iter().map(measure_sample).collect::<Vec<_>>();
    let profile = ParserFactCollectionProfileV0 {
        schema_version: "0",
        product: "omena-benchmarks.parser-fact-collection-profile",
        measurement_pin: std::env::var("OMENA_MEASUREMENT_PIN")
            .unwrap_or_else(|_| "unrecorded".to_string()),
        release_build: !cfg!(debug_assertions),
        operating_system: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        sample_batch_count: SAMPLE_BATCH_COUNT,
        samples,
    };
    println!("{}", serde_json::to_string_pretty(&profile)?);
    Ok(())
}

struct ProfileInput {
    name: String,
    dialect: StyleDialect,
    source: String,
}

fn profile_input(name: &str, dialect: StyleDialect, target_bytes: usize) -> ProfileInput {
    let mut source = String::with_capacity(target_bytes + 512);
    let mut index = 0usize;
    while source.len() < target_bytes {
        source.push_str(&format!(
            ".card{index}, main > .grid{index}:hover {{\n  --tone-{index}: rgb(10 20 30 / 80%);\n  color: var(--tone-{index}, black);\n  animation: pulse{index} 120ms ease-out;\n}}\n@keyframes pulse{index} {{ from {{ opacity: 0; }} to {{ opacity: 1; }} }}\n"
        ));
        index += 1;
    }
    ProfileInput {
        name: name.to_string(),
        dialect,
        source,
    }
}

fn measure_sample(input: ProfileInput) -> ParserFactCollectionSampleV0 {
    let iterations_per_batch = iterations_for_size(input.source.len());
    let parsed = parse(input.source.as_str(), input.dialect);

    warm_up(|| {
        black_box(parse(black_box(input.source.as_str()), input.dialect));
    });
    warm_up(|| {
        let facts = facts_from_cst(black_box(input.source.as_str()), black_box(&parsed));
        let emission = collect_emission_selector_facts_from_cst(
            black_box(input.source.as_str()),
            black_box(&parsed),
        );
        black_box((facts, emission));
    });
    warm_up(|| {
        black_box(collect_style_fact_collection(
            black_box(input.source.as_str()),
            input.dialect,
        ));
    });

    let parse = median_batch_duration(iterations_per_batch, || {
        black_box(parse(black_box(input.source.as_str()), input.dialect));
    });
    let preparsed_facts = median_batch_duration(iterations_per_batch, || {
        let facts = facts_from_cst(black_box(input.source.as_str()), black_box(&parsed));
        let emission = collect_emission_selector_facts_from_cst(
            black_box(input.source.as_str()),
            black_box(&parsed),
        );
        black_box((facts, emission));
    });
    let full = median_batch_duration(iterations_per_batch, || {
        black_box(collect_style_fact_collection(
            black_box(input.source.as_str()),
            input.dialect,
        ));
    });
    let (_, traversal) = with_omena_parser_fact_collection_instrumentation(|| {
        black_box(collect_style_fact_collection(
            black_box(input.source.as_str()),
            input.dialect,
        ));
    });

    let parse_ns = per_iteration_nanoseconds(parse, iterations_per_batch);
    let preparsed_ns = per_iteration_nanoseconds(preparsed_facts, iterations_per_batch);
    let full_ns = per_iteration_nanoseconds(full, iterations_per_batch);
    ParserFactCollectionSampleV0 {
        name: input.name,
        dialect: dialect_label(input.dialect),
        byte_length: input.source.len(),
        iterations_per_batch,
        parse_median_nanoseconds: parse_ns,
        preparsed_fact_collection_median_nanoseconds: preparsed_ns,
        full_collection_median_nanoseconds: full_ns,
        full_collection_traversal_entry_count: traversal.traversal_entry_count,
        full_collection_families: traversal.families,
        full_collection_registered_family_count: traversal.registered_family_count,
        full_collection_registered_families: traversal.registered_families,
        preparsed_fact_to_parse_ratio_milli: ratio_milli(preparsed_ns, parse_ns),
        full_collection_to_parse_ratio_milli: ratio_milli(full_ns, parse_ns),
    }
}

fn warm_up(mut operation: impl FnMut()) {
    for _ in 0..2 {
        operation();
    }
}

fn median_batch_duration(iterations: usize, mut operation: impl FnMut()) -> Duration {
    let mut samples = Vec::with_capacity(SAMPLE_BATCH_COUNT);
    for _ in 0..SAMPLE_BATCH_COUNT {
        let started = Instant::now();
        for _ in 0..iterations {
            operation();
        }
        samples.push(started.elapsed());
    }
    samples.sort_unstable();
    samples[SAMPLE_BATCH_COUNT / 2]
}

const fn iterations_for_size(byte_length: usize) -> usize {
    if byte_length <= 20 * 1024 {
        20
    } else if byte_length <= 64 * 1024 {
        8
    } else {
        3
    }
}

fn per_iteration_nanoseconds(duration: Duration, iterations: usize) -> u128 {
    duration.as_nanos() / iterations as u128
}

fn ratio_milli(numerator: u128, denominator: u128) -> u128 {
    numerator.saturating_mul(1_000) / denominator.max(1)
}

const fn dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}

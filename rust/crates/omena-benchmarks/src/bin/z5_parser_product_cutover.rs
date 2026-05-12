use std::{hint::black_box, time::Instant};

use engine_style_parser::summarize_css_modules_intermediate as summarize_legacy_intermediate;
use omena_benchmarks::{parse_legacy_style_sample, style_corpus, validate_legacy_style_sample};
use omena_parser::summarize_css_modules_intermediate as summarize_omena_intermediate;

const DEFAULT_MAX_RATIO: f64 = 1.10;
const ITERATIONS: usize = 40;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let max_ratio = std::env::var("CME_Z5_PARSER_PRODUCT_MAX_RATIO")
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(DEFAULT_MAX_RATIO);

    let mut worst_ratio = 0.0_f64;
    for sample in style_corpus() {
        validate_legacy_style_sample(sample.path, sample.source.as_str())?;
        warm_up(sample.path, sample.source.as_str(), sample.dialect);

        let legacy = measure_iterations(ITERATIONS, || {
            if let Some(sheet) =
                parse_legacy_style_sample(black_box(sample.path), black_box(sample.source.as_str()))
            {
                black_box(summarize_legacy_intermediate(black_box(&sheet)));
            }
        });
        let omena = measure_iterations(ITERATIONS, || {
            black_box(summarize_omena_intermediate(
                black_box(&sample.source),
                black_box(sample.dialect),
            ));
        });
        let ratio = omena.as_secs_f64() / legacy.as_secs_f64();
        worst_ratio = worst_ratio.max(ratio);

        if ratio > max_ratio {
            return Err(format!(
                "parser product cutover ratio exceeded for {}: omena={:.3}ms legacy={:.3}ms ratio={:.3} max={:.3}",
                sample.name,
                omena.as_secs_f64() * 1000.0 / ITERATIONS as f64,
                legacy.as_secs_f64() * 1000.0 / ITERATIONS as f64,
                ratio,
                max_ratio,
            )
            .into());
        }

        println!(
            "validated parser-product cutover sample: {} omena={:.3}ms legacy={:.3}ms ratio={:.3}",
            sample.name,
            omena.as_secs_f64() * 1000.0 / ITERATIONS as f64,
            legacy.as_secs_f64() * 1000.0 / ITERATIONS as f64,
            ratio,
        );
    }

    println!(
        "validated parser-product cutover readiness: samples={} iterations={} maxRatio={:.3} worstRatio={:.3}",
        style_corpus().len(),
        ITERATIONS,
        max_ratio,
        worst_ratio,
    );
    Ok(())
}

fn warm_up(path: &str, source: &str, dialect: omena_parser::StyleDialect) {
    for _ in 0..4 {
        if let Some(sheet) = parse_legacy_style_sample(black_box(path), black_box(source)) {
            black_box(summarize_legacy_intermediate(black_box(&sheet)));
        }
        black_box(summarize_omena_intermediate(
            black_box(source),
            black_box(dialect),
        ));
    }
}

fn measure_iterations(iterations: usize, mut f: impl FnMut()) -> std::time::Duration {
    let started = Instant::now();
    for _ in 0..iterations {
        f();
    }
    started.elapsed()
}

use std::hint::black_box;

use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use omena_benchmarks::{measure_omena_parser_product_sample, style_corpus};
use omena_transform_print::{default_print_options, print_transform_cst_source_with_dialect};

#[library_benchmark]
fn omena_parser_product_summary_corpus() -> usize {
    let mut total_bytes = 0;
    for sample in style_corpus() {
        let measurement =
            measure_omena_parser_product_sample(sample.source.as_str(), sample.dialect);
        black_box(measurement);
        total_bytes += sample.source.len();
    }
    total_bytes
}

#[library_benchmark]
fn omena_transform_print_identity_corpus() -> usize {
    let mut total_bytes = 0;
    for sample in style_corpus().into_iter().take(3) {
        let artifact = print_transform_cst_source_with_dialect(
            sample.path,
            sample.source.as_str(),
            sample.dialect,
            "omena-benchmarks.instruction-count-advisory",
            &[],
            default_print_options(),
        );
        total_bytes += artifact.css.len();
        black_box(artifact);
    }
    total_bytes
}

library_benchmark_group!(
    name = z5_instruction_count_advisory;
    benchmarks =
        omena_parser_product_summary_corpus,
        omena_transform_print_identity_corpus
);

main!(library_benchmark_groups = z5_instruction_count_advisory);

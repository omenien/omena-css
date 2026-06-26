use std::hint::black_box;

use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use omena_benchmarks::style_corpus;
use omena_query::{
    OmenaQueryStyleMemoHostV0, OmenaQueryStyleResolutionInputsV0, OmenaQueryStyleSourceInputV0,
};

#[library_benchmark]
fn cold_open_query_corpus_n() -> usize {
    measure_cold_open_query_corpus(1)
}

#[library_benchmark]
fn cold_open_query_corpus_2n() -> usize {
    measure_cold_open_query_corpus(2)
}

#[library_benchmark(setup = setup_memoized_recheck_query_corpus_n)]
fn memoized_recheck_query_corpus_n(fixture: RecheckFixture) -> usize {
    measure_memoized_recheck_query_corpus(fixture)
}

#[library_benchmark(setup = setup_memoized_recheck_query_corpus_2n)]
fn memoized_recheck_query_corpus_2n(fixture: RecheckFixture) -> usize {
    measure_memoized_recheck_query_corpus(fixture)
}

fn measure_cold_open_query_corpus(repetitions: usize) -> usize {
    let corpus = query_corpus(repetitions);
    let target_path = corpus[0].style_path.as_str();
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
    let mut host = OmenaQueryStyleMemoHostV0::new();
    let diagnostics = host.workspace_style_diagnostics(
        target_path,
        corpus.as_slice(),
        &[],
        &[],
        &[],
        &resolution_inputs,
    );
    black_box(diagnostics);
    corpus.iter().map(|source| source.style_source.len()).sum()
}

struct RecheckFixture {
    corpus: Vec<OmenaQueryStyleSourceInputV0>,
    host: OmenaQueryStyleMemoHostV0,
    resolution_inputs: OmenaQueryStyleResolutionInputsV0,
    target_path: String,
    extra_work_iterations: usize,
}

fn setup_memoized_recheck_query_corpus_n() -> RecheckFixture {
    setup_memoized_recheck_query_corpus(1, 0)
}

fn setup_memoized_recheck_query_corpus_2n() -> RecheckFixture {
    setup_memoized_recheck_query_corpus(2, 25_000_000)
}

fn setup_memoized_recheck_query_corpus(
    repetitions: usize,
    extra_work_iterations: usize,
) -> RecheckFixture {
    let corpus = query_corpus(repetitions);
    let target_path = corpus[0].style_path.clone();
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
    let mut host = OmenaQueryStyleMemoHostV0::new();
    let initial = host.workspace_style_diagnostics(
        target_path.as_str(),
        corpus.as_slice(),
        &[],
        &[],
        &[],
        &resolution_inputs,
    );
    black_box(initial);

    RecheckFixture {
        corpus,
        host,
        resolution_inputs,
        target_path,
        extra_work_iterations,
    }
}

fn measure_memoized_recheck_query_corpus(mut fixture: RecheckFixture) -> usize {
    fixture.corpus[0]
        .style_source
        .push_str("\n.perfGateProbe { color: currentColor; }\n");
    let diagnostics = fixture.host.workspace_style_diagnostics(
        fixture.target_path.as_str(),
        fixture.corpus.as_slice(),
        &[],
        &[],
        &[],
        &fixture.resolution_inputs,
    );
    black_box(diagnostics);
    let mut extra = 0usize;
    for iteration in 0..fixture.extra_work_iterations {
        extra = extra.wrapping_add(iteration ^ fixture.corpus.len());
    }
    black_box(extra);
    fixture
        .corpus
        .iter()
        .map(|source| source.style_source.len())
        .sum()
}

fn query_corpus(repetitions: usize) -> Vec<OmenaQueryStyleSourceInputV0> {
    let samples = style_corpus();
    let mut corpus = Vec::with_capacity(samples.len() * repetitions);
    for repetition in 0..repetitions {
        for sample in &samples {
            corpus.push(OmenaQueryStyleSourceInputV0 {
                style_path: format!("/workspace/perf/{repetition}/{}", sample.path),
                style_source: sample.source.clone(),
            });
        }
    }
    corpus
}

library_benchmark_group!(
    name = z5_perf_gate_spine;
    benchmarks =
        cold_open_query_corpus_n,
        cold_open_query_corpus_2n,
        memoized_recheck_query_corpus_n,
        memoized_recheck_query_corpus_2n
);

main!(library_benchmark_groups = z5_perf_gate_spine);

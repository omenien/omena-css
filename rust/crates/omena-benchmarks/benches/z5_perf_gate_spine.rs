use std::hint::black_box;

use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use omena_abstract_value::AbstractClassValueV0;
use omena_benchmarks::style_corpus;
use omena_cross_file_summary::{UnifiedHypergraphEdgeKindV0, UnifiedHypergraphHyperedgeV0};
use omena_query::{
    OmenaQueryStyleMemoHostV0, OmenaQueryStyleResolutionInputsV0, OmenaQueryStyleSourceInputV0,
};
use omena_streaming_ifds::{
    StreamingIFDSDemandIndexV0, StreamingIfdsEventInputV0, run_streaming_ifds_demand_with_index_v0,
    streaming_ifds_demand_index_v0, streaming_ifds_event_input_v0,
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

#[library_benchmark(setup = setup_committed_graph_edit_query_corpus_n)]
fn committed_graph_edit_query_corpus_n(fixture: RecheckFixture) -> usize {
    measure_committed_graph_edit_query_corpus(fixture)
}

#[library_benchmark(setup = setup_committed_graph_edit_query_corpus_2n)]
fn committed_graph_edit_query_corpus_2n(fixture: RecheckFixture) -> usize {
    measure_committed_graph_edit_query_corpus(fixture)
}

#[library_benchmark(setup = setup_demand_ifds_fixed_query_corpus_n)]
fn demand_ifds_fixed_query_corpus_n(fixture: DemandFixture) -> usize {
    measure_demand_ifds_fixed_query_corpus(fixture)
}

#[library_benchmark(setup = setup_demand_ifds_fixed_query_corpus_2n)]
fn demand_ifds_fixed_query_corpus_2n(fixture: DemandFixture) -> usize {
    measure_demand_ifds_fixed_query_corpus(fixture)
}

#[library_benchmark(setup = setup_demand_ifds_fixed_query_corpus_4n)]
fn demand_ifds_fixed_query_corpus_4n(fixture: DemandFixture) -> usize {
    measure_demand_ifds_fixed_query_corpus(fixture)
}

#[library_benchmark(setup = setup_demand_ifds_fixed_query_corpus_8n)]
fn demand_ifds_fixed_query_corpus_8n(fixture: DemandFixture) -> usize {
    measure_demand_ifds_fixed_query_corpus(fixture)
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

struct DemandFixture {
    start_node_ids: Vec<String>,
    target_node_ids: Vec<String>,
    index: StreamingIFDSDemandIndexV0,
    events: Vec<StreamingIfdsEventInputV0>,
    corpus_edge_count: usize,
}

struct RecheckFixture {
    corpus: Vec<OmenaQueryStyleSourceInputV0>,
    host: OmenaQueryStyleMemoHostV0,
    resolution_inputs: OmenaQueryStyleResolutionInputsV0,
    target_path: String,
}

fn setup_memoized_recheck_query_corpus_n() -> RecheckFixture {
    setup_memoized_recheck_query_corpus(1)
}

fn setup_memoized_recheck_query_corpus_2n() -> RecheckFixture {
    setup_memoized_recheck_query_corpus(2)
}

fn setup_committed_graph_edit_query_corpus_n() -> RecheckFixture {
    setup_memoized_recheck_query_corpus(1)
}

fn setup_committed_graph_edit_query_corpus_2n() -> RecheckFixture {
    setup_memoized_recheck_query_corpus(2)
}

fn setup_demand_ifds_fixed_query_corpus_n() -> DemandFixture {
    setup_demand_ifds_fixed_query_corpus(1)
}

fn setup_demand_ifds_fixed_query_corpus_2n() -> DemandFixture {
    setup_demand_ifds_fixed_query_corpus(2)
}

fn setup_demand_ifds_fixed_query_corpus_4n() -> DemandFixture {
    setup_demand_ifds_fixed_query_corpus(4)
}

fn setup_demand_ifds_fixed_query_corpus_8n() -> DemandFixture {
    setup_demand_ifds_fixed_query_corpus(8)
}

fn setup_memoized_recheck_query_corpus(repetitions: usize) -> RecheckFixture {
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
    }
}

fn setup_demand_ifds_fixed_query_corpus(scale: usize) -> DemandFixture {
    let branch_count = 128 * scale;
    let chain_depth = 8;
    let mut hyperedges = Vec::<UnifiedHypergraphHyperedgeV0>::new();
    for branch in 0..branch_count {
        let mut tail = "root".to_string();
        for depth in 0..chain_depth {
            let head = format!("branch-{branch}-node-{depth}");
            hyperedges.push(demand_hyperedge(
                format!("edge-{branch}-{depth}"),
                tail.as_str(),
                head.as_str(),
            ));
            tail = head;
        }
    }
    let target_node_ids = vec![format!("branch-0-node-{}", chain_depth - 1)];
    let index = streaming_ifds_demand_index_v0(hyperedges.as_slice());
    DemandFixture {
        start_node_ids: vec!["root".to_string()],
        target_node_ids,
        index,
        events: vec![streaming_ifds_event_input_v0(
            "event-root",
            1,
            "root",
            AbstractClassValueV0::Exact {
                value: "seed".to_string(),
            },
            None,
        )],
        corpus_edge_count: hyperedges.len(),
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
    fixture
        .corpus
        .iter()
        .map(|source| source.style_source.len())
        .sum()
}

fn measure_demand_ifds_fixed_query_corpus(fixture: DemandFixture) -> usize {
    let report = run_streaming_ifds_demand_with_index_v0(
        fixture.start_node_ids.as_slice(),
        fixture.target_node_ids.as_slice(),
        &fixture.index,
        fixture.events.as_slice(),
    );
    let request_work = report
        .transfer_visit_count
        .saturating_add(report.fact_keys.len())
        .saturating_add(report.slice_scc_count);
    black_box(report);
    black_box(fixture.corpus_edge_count);
    request_work
}

fn measure_committed_graph_edit_query_corpus(mut fixture: RecheckFixture) -> usize {
    fixture.corpus[0]
        .style_source
        .push_str("\n.committedGraphProbe { color: currentColor; }\n");
    let selector = fixture.host.workspace_revision_selector(
        fixture.corpus.as_slice(),
        &[],
        &[],
        &[],
        &fixture.resolution_inputs,
    );
    black_box(selector);
    fixture
        .corpus
        .iter()
        .map(|source| source.style_source.len())
        .sum()
}

fn demand_hyperedge(id: String, from: &str, to: &str) -> UnifiedHypergraphHyperedgeV0 {
    let edge_kind = UnifiedHypergraphEdgeKindV0::ComposesLocal;
    let source_edge_kind = edge_kind.as_wire_label();
    UnifiedHypergraphHyperedgeV0 {
        schema_version: "0",
        product: "omena-benchmarks.demand-ifds-fixed-query",
        layer_marker: "hypergraph-ifds",
        feature_gate: "hypergraph-ifds",
        hyperedge_id: id.clone(),
        edge_kind,
        source_summary_edge_id: id,
        source_edge_kind,
        source_status: "known",
        tail_node_ids: vec![from.to_string()],
        head_node_id: to.to_string(),
        order_significant_tail: false,
    }
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
        memoized_recheck_query_corpus_2n,
        committed_graph_edit_query_corpus_n,
        committed_graph_edit_query_corpus_2n,
        demand_ifds_fixed_query_corpus_n,
        demand_ifds_fixed_query_corpus_2n,
        demand_ifds_fixed_query_corpus_4n,
        demand_ifds_fixed_query_corpus_8n
);

main!(library_benchmark_groups = z5_perf_gate_spine);

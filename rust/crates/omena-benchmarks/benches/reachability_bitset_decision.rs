use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;

use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use omena_cross_file_summary::{collect_reachable_node_ids, collect_reachable_node_ids_bitset};

#[library_benchmark]
fn reachability_btreeset_closure_on_shared_corpus() -> usize {
    let adjacency = benchmark_adjacency();
    let reachable = collect_reachable_node_ids("node-000", &adjacency);
    black_box(reachable.len())
}

#[library_benchmark]
fn reachability_bitset_closure_on_shared_corpus() -> usize {
    let adjacency = benchmark_adjacency();
    let reachable = collect_reachable_node_ids_bitset("node-000", &adjacency);
    black_box(reachable.len())
}

fn benchmark_adjacency() -> BTreeMap<String, BTreeSet<String>> {
    let mut adjacency = BTreeMap::<String, BTreeSet<String>>::new();
    for index in 0..96 {
        let current = format!("node-{index:03}");
        if index + 1 < 96 {
            adjacency
                .entry(current.clone())
                .or_default()
                .insert(format!("node-{:03}", index + 1));
        }
        if index + 8 < 96 {
            adjacency
                .entry(current.clone())
                .or_default()
                .insert(format!("node-{:03}", index + 8));
        }
        if index % 9 == 0 && index + 17 < 96 {
            adjacency
                .entry(current)
                .or_default()
                .insert(format!("node-{:03}", index + 17));
        }
    }
    adjacency
}

library_benchmark_group!(
    name = reachability_bitset_decision;
    benchmarks =
        reachability_btreeset_closure_on_shared_corpus,
        reachability_bitset_closure_on_shared_corpus
);

main!(library_benchmark_groups = reachability_bitset_decision);

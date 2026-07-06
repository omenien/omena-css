use std::collections::{BTreeMap, BTreeSet, VecDeque};

use omena_abstract_value::AbstractClassValueV0;
use omena_benchmarks::{bundler_productization_corpus, style_corpus};
use omena_cross_file_summary::{
    BatchHypergraphConnectivityOracle, OmenaUnifiedHypergraphConnectivityOracle,
    UnifiedHypergraphEdgeKindV0, UnifiedHypergraphHyperedgeV0, collect_reachable_node_ids_bitset,
};
use omena_parser::{
    ClosedWorldLinkedModuleV0, ModuleIdV0, ModuleInstanceKeyV0,
    summarize_closed_world_reachability_bitset_parity_v0,
};
use omena_reachability_datalog_lab::{
    datalog_fact_keys_v0, datalog_reachable_node_ids, selector_equality_witness_v0,
};
use omena_streaming_ifds::{
    ExactStreamingConnectivityOracleV0, omena_streaming_ifds_batch_fact_keys_v0,
    run_streaming_ifds_demand_v0, run_streaming_ifds_exact_v0, streaming_ifds_event_input_v0,
    streaming_ifds_structural_projection_node_ids_v0,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffReachabilityEquivalenceFileReportV0 {
    pub fixture_id: String,
    pub fixture_family: &'static str,
    pub start_node_id: String,
    pub baseline_reachable_node_ids: Vec<String>,
    pub candidate_reachable_node_ids: Vec<String>,
    pub streaming_reachable_node_ids: Vec<String>,
    pub bitset_reachable_node_ids: Vec<String>,
    pub sets_equal: bool,
    pub streaming_matches_batch: bool,
    pub bitset_matches_batch: bool,
    pub product_reachability_parity_with_batch: bool,
    pub product_reachability_delta_used: bool,
    pub batch_fact_keys: Vec<String>,
    pub incremental_fact_keys: Vec<String>,
    pub ascent_fact_keys: Vec<String>,
    pub demand_fact_keys: Vec<String>,
    pub projected_batch_fact_keys: Vec<String>,
    pub fact_keys_batch_incremental_equal: bool,
    pub fact_keys_three_way_equal: bool,
    pub fact_keys_demand_matches_projected_batch: bool,
    pub has_multi_edge_closure_fact_key: bool,
    pub has_value_carrying_fact_key: bool,
    pub has_strict_demand_projection: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffClosureHashBitsetParityFileReportV0 {
    pub fixture_id: String,
    pub module_instance_count: usize,
    pub symbol_name_count: usize,
    pub reachability_equal: bool,
    pub closure_hash_equal: bool,
    pub btreeset_closure_hash: String,
    pub bitset_closure_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffSelectorEqualityRelationReportV0 {
    pub relation_id: &'static str,
    pub left: String,
    pub right: String,
    pub equal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffReachabilityEquivalenceReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub baseline_evaluator: &'static str,
    pub candidate_evaluator: &'static str,
    pub fixture_count: usize,
    pub equal_fixture_count: usize,
    pub streaming_equal_fixture_count: usize,
    pub bitset_equal_fixture_count: usize,
    pub product_parity_fixture_count: usize,
    pub fact_key_batch_incremental_equal_fixture_count: usize,
    pub fact_key_three_way_equal_fixture_count: usize,
    pub fact_key_demand_projected_equal_fixture_count: usize,
    pub strict_demand_projection_fixture_count: usize,
    pub multi_edge_closure_fixture_count: usize,
    pub value_carrying_fact_key_fixture_count: usize,
    pub closure_hash_bitset_parity_fixture_count: usize,
    pub closure_hash_bitset_parity_equal_fixture_count: usize,
    pub multi_module_closure_hash_fixture_count: usize,
    pub all_sets_equal: bool,
    pub streaming_matches_batch: bool,
    pub all_reachability_bitset_parity_equal: bool,
    pub all_closure_hash_bitset_parity_equal: bool,
    pub product_reachability_parity_with_batch: bool,
    pub all_fact_keys_batch_incremental_equal: bool,
    pub all_fact_keys_three_way_equal: bool,
    pub all_fact_keys_four_way_equal: bool,
    pub selector_relation_count: usize,
    pub selector_relations_equal: bool,
    pub files: Vec<OmenaDiffReachabilityEquivalenceFileReportV0>,
    pub closure_hash_files: Vec<OmenaDiffClosureHashBitsetParityFileReportV0>,
    pub selector_relations: Vec<OmenaDiffSelectorEqualityRelationReportV0>,
}

#[derive(Debug, Clone)]
struct ReachabilityEquivalenceFixtureV0 {
    id: String,
    family: &'static str,
    start_node_id: String,
    demand_target_node_ids: Vec<String>,
    seed_value: AbstractClassValueV0,
    hyperedges: Vec<UnifiedHypergraphHyperedgeV0>,
}

#[derive(Debug, Clone)]
struct ClosureHashBitsetParityFixtureV0 {
    id: String,
    entrypoints: Vec<ModuleInstanceKeyV0>,
    linked_modules: Vec<ClosedWorldLinkedModuleV0>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProductReachabilityParityV0 {
    parity_with_batch: bool,
    delta_used: bool,
    batch_fact_keys: Vec<String>,
    incremental_fact_keys: Vec<String>,
}

pub fn summarize_reachability_second_oracle_equivalence_v0()
-> OmenaDiffReachabilityEquivalenceReportV0 {
    let baseline = BatchHypergraphConnectivityOracle;
    let streaming = ExactStreamingConnectivityOracleV0::default();
    let files = reachability_equivalence_fixtures_v0()
        .into_iter()
        .map(|fixture| {
            let baseline_reachable_node_ids =
                baseline.reachable_node_ids(fixture.start_node_id.as_str(), &fixture.hyperedges);
            let candidate_reachable_node_ids =
                datalog_reachable_node_ids(fixture.start_node_id.as_str(), &fixture.hyperedges);
            let streaming_reachable_node_ids =
                streaming.reachable_node_ids(fixture.start_node_id.as_str(), &fixture.hyperedges);
            let bitset_reachable_node_ids = collect_reachable_node_ids_bitset(
                fixture.start_node_id.as_str(),
                &fixture_adjacency(&fixture.hyperedges),
            );
            let sets_equal = baseline_reachable_node_ids == candidate_reachable_node_ids;
            let streaming_matches_batch =
                baseline_reachable_node_ids == streaming_reachable_node_ids;
            let bitset_matches_batch = baseline_reachable_node_ids == bitset_reachable_node_ids;
            let product_report = product_reachability_parity_for_fixture_v0(&fixture);
            let fact_keys_batch_incremental_equal =
                product_report.batch_fact_keys == product_report.incremental_fact_keys;
            let ascent_fact_keys = datalog_fact_keys_v0(
                fixture.start_node_id.as_str(),
                seed_value_key(&fixture.seed_value),
                &fixture.hyperedges,
            );
            let demand_report = demand_reachability_parity_for_fixture_v0(&fixture);
            let structural_projection_node_ids =
                structural_projection_node_ids_for_fixture_v0(&fixture);
            let projected_batch_fact_keys = project_fact_keys_to_nodes(
                &product_report.batch_fact_keys,
                &structural_projection_node_ids,
            );
            let fact_keys_demand_matches_projected_batch =
                demand_report.fact_keys == projected_batch_fact_keys;
            let fact_keys_three_way_equal = fact_keys_batch_incremental_equal
                && product_report.batch_fact_keys == ascent_fact_keys;
            let has_multi_edge_closure_fact_key =
                has_multi_edge_closure_fact_key(&fixture, &product_report.batch_fact_keys);
            let has_value_carrying_fact_key = product_report
                .batch_fact_keys
                .iter()
                .any(|key| fact_key_value(key).is_some_and(|value| value != "top"));
            OmenaDiffReachabilityEquivalenceFileReportV0 {
                fixture_id: fixture.id,
                fixture_family: fixture.family,
                start_node_id: fixture.start_node_id,
                baseline_reachable_node_ids,
                candidate_reachable_node_ids,
                streaming_reachable_node_ids,
                bitset_reachable_node_ids,
                sets_equal,
                streaming_matches_batch,
                bitset_matches_batch,
                product_reachability_parity_with_batch: product_report.parity_with_batch,
                product_reachability_delta_used: product_report.delta_used,
                batch_fact_keys: product_report.batch_fact_keys,
                incremental_fact_keys: product_report.incremental_fact_keys,
                ascent_fact_keys,
                demand_fact_keys: demand_report.fact_keys,
                projected_batch_fact_keys,
                fact_keys_batch_incremental_equal,
                fact_keys_three_way_equal,
                fact_keys_demand_matches_projected_batch,
                has_multi_edge_closure_fact_key,
                has_value_carrying_fact_key,
                has_strict_demand_projection: demand_report
                    .strict_subset_of_forward_reachable_nodes,
            }
        })
        .collect::<Vec<_>>();
    let selector_relations = selector_equality_relation_fixtures_v0();
    let closure_hash_files = closure_hash_bitset_parity_fixtures_v0()
        .into_iter()
        .map(|fixture| {
            match summarize_closed_world_reachability_bitset_parity_v0(
                fixture.entrypoints,
                fixture.linked_modules,
            ) {
                Ok(report) => OmenaDiffClosureHashBitsetParityFileReportV0 {
                    fixture_id: fixture.id,
                    module_instance_count: report.module_instance_count,
                    symbol_name_count: report.symbol_name_count,
                    reachability_equal: report.reachability_equal,
                    closure_hash_equal: report.closure_hash_equal,
                    btreeset_closure_hash: report.btreeset_closure_hash,
                    bitset_closure_hash: report.bitset_closure_hash,
                },
                Err(error) => OmenaDiffClosureHashBitsetParityFileReportV0 {
                    fixture_id: fixture.id,
                    module_instance_count: 0,
                    symbol_name_count: 0,
                    reachability_equal: false,
                    closure_hash_equal: false,
                    btreeset_closure_hash: format!("error:{error:?}"),
                    bitset_closure_hash: String::new(),
                },
            }
        })
        .collect::<Vec<_>>();
    let equal_fixture_count = files.iter().filter(|file| file.sets_equal).count();
    let streaming_equal_fixture_count = files
        .iter()
        .filter(|file| file.streaming_matches_batch)
        .count();
    let bitset_equal_fixture_count = files
        .iter()
        .filter(|file| file.bitset_matches_batch)
        .count();
    let product_parity_fixture_count = files
        .iter()
        .filter(|file| file.product_reachability_parity_with_batch)
        .count();
    let fact_key_batch_incremental_equal_fixture_count = files
        .iter()
        .filter(|file| file.fact_keys_batch_incremental_equal)
        .count();
    let fact_key_three_way_equal_fixture_count = files
        .iter()
        .filter(|file| file.fact_keys_three_way_equal)
        .count();
    let fact_key_demand_projected_equal_fixture_count = files
        .iter()
        .filter(|file| file.fact_keys_demand_matches_projected_batch)
        .count();
    let strict_demand_projection_fixture_count = files
        .iter()
        .filter(|file| file.has_strict_demand_projection)
        .count();
    let multi_edge_closure_fixture_count = files
        .iter()
        .filter(|file| file.has_multi_edge_closure_fact_key)
        .count();
    let value_carrying_fact_key_fixture_count = files
        .iter()
        .filter(|file| file.has_value_carrying_fact_key)
        .count();
    let closure_hash_bitset_parity_equal_fixture_count = closure_hash_files
        .iter()
        .filter(|file| file.reachability_equal && file.closure_hash_equal)
        .count();
    let multi_module_closure_hash_fixture_count = closure_hash_files
        .iter()
        .filter(|file| file.module_instance_count >= 2 && file.symbol_name_count > 0)
        .count();
    let selector_relations_equal = selector_relations.iter().all(|relation| relation.equal);

    let fixture_count = files.len();
    let closure_hash_bitset_parity_fixture_count = closure_hash_files.len();
    OmenaDiffReachabilityEquivalenceReportV0 {
        schema_version: "0",
        product: "omena-diff-test.reachability-equivalence",
        baseline_evaluator: "batch-hypergraph-connectivity-oracle",
        candidate_evaluator: "datalog-reachability-witness",
        fixture_count,
        equal_fixture_count,
        streaming_equal_fixture_count,
        bitset_equal_fixture_count,
        product_parity_fixture_count,
        fact_key_batch_incremental_equal_fixture_count,
        fact_key_three_way_equal_fixture_count,
        fact_key_demand_projected_equal_fixture_count,
        strict_demand_projection_fixture_count,
        multi_edge_closure_fixture_count,
        value_carrying_fact_key_fixture_count,
        closure_hash_bitset_parity_fixture_count,
        closure_hash_bitset_parity_equal_fixture_count,
        multi_module_closure_hash_fixture_count,
        all_sets_equal: equal_fixture_count == fixture_count,
        streaming_matches_batch: streaming_equal_fixture_count == fixture_count,
        all_reachability_bitset_parity_equal: bitset_equal_fixture_count == fixture_count,
        all_closure_hash_bitset_parity_equal: closure_hash_bitset_parity_equal_fixture_count
            == closure_hash_bitset_parity_fixture_count
            && multi_module_closure_hash_fixture_count > 0,
        product_reachability_parity_with_batch: product_parity_fixture_count == fixture_count,
        all_fact_keys_batch_incremental_equal: fact_key_batch_incremental_equal_fixture_count
            == fixture_count,
        all_fact_keys_three_way_equal: fact_key_three_way_equal_fixture_count == fixture_count,
        all_fact_keys_four_way_equal: fact_key_three_way_equal_fixture_count == fixture_count
            && fact_key_demand_projected_equal_fixture_count == fixture_count
            && strict_demand_projection_fixture_count > 0,
        selector_relation_count: selector_relations.len(),
        selector_relations_equal,
        files,
        closure_hash_files,
        selector_relations,
    }
}

fn fixture_adjacency(
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut adjacency = BTreeMap::<String, BTreeSet<String>>::new();
    for edge in hyperedges {
        for tail in &edge.tail_node_ids {
            adjacency
                .entry(tail.clone())
                .or_default()
                .insert(edge.head_node_id.clone());
        }
    }
    adjacency
}

fn product_reachability_parity_for_fixture_v0(
    fixture: &ReachabilityEquivalenceFixtureV0,
) -> ProductReachabilityParityV0 {
    let seed = vec![streaming_ifds_event_input_v0(
        format!("{}:seed", fixture.id),
        1,
        fixture.start_node_id.clone(),
        fixture.seed_value.clone(),
        None,
    )];
    let first = run_streaming_ifds_exact_v0(
        format!("{}:initial", fixture.id),
        fixture.start_node_id.as_str(),
        &fixture.hyperedges,
        &seed,
        &ExactStreamingConnectivityOracleV0::default(),
        None,
    );
    let warm_event = vec![streaming_ifds_event_input_v0(
        format!("{}:warm", fixture.id),
        2,
        fixture.start_node_id.clone(),
        fixture.seed_value.clone(),
        None,
    )];
    let warm = run_streaming_ifds_exact_v0(
        format!("{}:warm-run", fixture.id),
        fixture.start_node_id.as_str(),
        &fixture.hyperedges,
        &warm_event,
        &ExactStreamingConnectivityOracleV0::default(),
        Some(&first.summary_cache),
    );
    let batch_fact_keys = omena_streaming_ifds_batch_fact_keys_v0(&fixture.hyperedges, &warm_event);
    let incremental_fact_keys = warm
        .summary_cache
        .iter()
        .flat_map(|entry| entry.fact_keys.iter().cloned())
        .collect::<Vec<_>>();

    ProductReachabilityParityV0 {
        parity_with_batch: warm.reachability_parity_with_batch && warm.precision_parity_with_batch,
        delta_used: warm.reachability_delta_used,
        batch_fact_keys,
        incremental_fact_keys,
    }
}

fn demand_reachability_parity_for_fixture_v0(
    fixture: &ReachabilityEquivalenceFixtureV0,
) -> omena_streaming_ifds::StreamingIFDSDemandReportV0 {
    let event = vec![streaming_ifds_event_input_v0(
        format!("{}:demand", fixture.id),
        3,
        fixture.start_node_id.clone(),
        fixture.seed_value.clone(),
        None,
    )];
    run_streaming_ifds_demand_v0(
        std::slice::from_ref(&fixture.start_node_id),
        &fixture.demand_target_node_ids,
        &fixture.hyperedges,
        &event,
    )
}

fn structural_projection_node_ids_for_fixture_v0(
    fixture: &ReachabilityEquivalenceFixtureV0,
) -> Vec<String> {
    streaming_ifds_structural_projection_node_ids_v0(
        std::slice::from_ref(&fixture.start_node_id),
        &fixture.demand_target_node_ids,
        &fixture.hyperedges,
    )
}

fn project_fact_keys_to_nodes(fact_keys: &[String], node_ids: &[String]) -> Vec<String> {
    let nodes = node_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    fact_keys
        .iter()
        .filter(|key| {
            key.rsplit_once('|')
                .is_some_and(|(node_id, _)| nodes.contains(node_id))
        })
        .cloned()
        .collect()
}

fn reachability_equivalence_fixtures_v0() -> Vec<ReachabilityEquivalenceFixtureV0> {
    let mut fixtures = vec![
        multi_hop_cross_file_fixture_v0(),
        value_carrying_compose_fixture_v0(),
        sparse_css_seed_fixture_v0(),
        sass_module_seed_fixture_v0(),
    ];
    fixtures.extend(
        style_corpus()
            .into_iter()
            .map(|sample| sample_reachability_fixture_v0("style-corpus", sample.name, sample.path)),
    );
    fixtures.extend(
        bundler_productization_corpus().into_iter().map(|sample| {
            sample_reachability_fixture_v0("bundler-corpus", sample.name, sample.path)
        }),
    );
    fixtures
}

fn multi_hop_cross_file_fixture_v0() -> ReachabilityEquivalenceFixtureV0 {
    let start = "styleModule|/workspace/App.module.scss|root";
    let base = "styleSymbol|/workspace/base.module.scss|base";
    let theme = "styleSymbol|/workspace/theme.module.scss|theme";
    let terminal = "styleSymbol|/workspace/terminal.module.scss|terminal";
    ReachabilityEquivalenceFixtureV0 {
        id: "multi-hop-composes-sass-chain".to_string(),
        family: "cross-file-reachability",
        start_node_id: start.to_string(),
        demand_target_node_ids: vec![theme.to_string()],
        seed_value: AbstractClassValueV0::Top,
        hyperedges: vec![
            hyperedge(
                "edge-app-base",
                start,
                base,
                UnifiedHypergraphEdgeKindV0::ComposesExternal,
            ),
            hyperedge(
                "edge-base-theme",
                base,
                theme,
                UnifiedHypergraphEdgeKindV0::SassUse,
            ),
            hyperedge(
                "edge-theme-terminal",
                theme,
                terminal,
                UnifiedHypergraphEdgeKindV0::SassForward,
            ),
        ],
    }
}

fn value_carrying_compose_fixture_v0() -> ReachabilityEquivalenceFixtureV0 {
    let start = "styleModule|/workspace/Button.module.scss|button";
    let base = "styleSymbol|/workspace/Button.module.scss|base";
    let primary = "styleSymbol|/workspace/theme.module.scss|primary";
    ReachabilityEquivalenceFixtureV0 {
        id: "value-carrying-composes-chain".to_string(),
        family: "cross-file-reachability",
        start_node_id: start.to_string(),
        demand_target_node_ids: vec![base.to_string()],
        seed_value: AbstractClassValueV0::Exact {
            value: "btn".to_string(),
        },
        hyperedges: vec![
            hyperedge(
                "edge-button-base",
                start,
                base,
                UnifiedHypergraphEdgeKindV0::ComposesLocal,
            ),
            hyperedge(
                "edge-base-primary",
                base,
                primary,
                UnifiedHypergraphEdgeKindV0::ComposesExternal,
            ),
        ],
    }
}

fn sparse_css_seed_fixture_v0() -> ReachabilityEquivalenceFixtureV0 {
    sample_reachability_fixture_v0(
        "wpt-style-seed",
        "css-selector-reachability",
        "wpt/selectors.css",
    )
}

fn sass_module_seed_fixture_v0() -> ReachabilityEquivalenceFixtureV0 {
    sample_reachability_fixture_v0(
        "sass-spec-seed",
        "sass-module-forwarding",
        "sass/module-forwarding.scss",
    )
}

fn closure_hash_bitset_parity_fixtures_v0() -> Vec<ClosureHashBitsetParityFixtureV0> {
    let app = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("/workspace/App.module.css"));
    let tokens = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("/workspace/tokens.module.css"));
    let theme = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("/workspace/theme.module.css"));
    vec![ClosureHashBitsetParityFixtureV0 {
        id: "closed-world-multi-module-symbols".to_string(),
        entrypoints: vec![app.clone()],
        linked_modules: vec![
            ClosedWorldLinkedModuleV0::new(app)
                .with_dependency(tokens.clone())
                .with_class_name("button"),
            ClosedWorldLinkedModuleV0::new(tokens.clone())
                .with_dependency(theme.clone())
                .with_value_name("spacing"),
            ClosedWorldLinkedModuleV0::new(theme)
                .with_keyframe_name("fade")
                .with_custom_property_name("--brand"),
        ],
    }]
}

fn sample_reachability_fixture_v0(
    family: &'static str,
    sample_name: &str,
    sample_path: &str,
) -> ReachabilityEquivalenceFixtureV0 {
    let start = format!("styleModule|/workspace/{sample_path}|root");
    let local = format!("styleSymbol|/workspace/{sample_path}|local");
    ReachabilityEquivalenceFixtureV0 {
        id: format!("{family}:{sample_name}"),
        family,
        start_node_id: start.clone(),
        demand_target_node_ids: vec![local.clone()],
        seed_value: AbstractClassValueV0::Top,
        hyperedges: vec![hyperedge(
            &format!("edge-{family}-{sample_name}"),
            start.as_str(),
            local.as_str(),
            UnifiedHypergraphEdgeKindV0::Value,
        )],
    }
}

fn fact_key_value(key: &str) -> Option<&str> {
    key.rsplit_once('|').map(|(_, value)| value)
}

fn seed_value_key(value: &AbstractClassValueV0) -> String {
    match value {
        AbstractClassValueV0::Bottom => "bottom".to_string(),
        AbstractClassValueV0::Exact { value } => format!("exact:{value}"),
        AbstractClassValueV0::FiniteSet { values } => {
            let mut values = values.clone();
            values.sort();
            values.dedup();
            format!("finiteSet:{}", values.join(","))
        }
        AbstractClassValueV0::Top => "top".to_string(),
        _ => "unsupportedSeedValue".to_string(),
    }
}

fn fact_key_node_id(key: &str) -> &str {
    key.rsplit_once('|').map(|(node, _)| node).unwrap_or(key)
}

fn has_multi_edge_closure_fact_key(
    fixture: &ReachabilityEquivalenceFixtureV0,
    fact_keys: &[String],
) -> bool {
    let distances = node_distances_from_start(&fixture.start_node_id, &fixture.hyperedges);
    fact_keys.iter().any(|key| {
        distances
            .get(fact_key_node_id(key))
            .is_some_and(|distance| *distance >= 2)
    })
}

fn node_distances_from_start(
    start_node_id: &str,
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> BTreeMap<String, usize> {
    let mut adjacency = BTreeMap::<String, Vec<String>>::new();
    for edge in hyperedges {
        for tail in &edge.tail_node_ids {
            adjacency
                .entry(tail.clone())
                .or_default()
                .push(edge.head_node_id.clone());
        }
    }

    let mut distances = BTreeMap::<String, usize>::from([(start_node_id.to_string(), 0)]);
    let mut pending = VecDeque::from([start_node_id.to_string()]);
    while let Some(node_id) = pending.pop_front() {
        let next_distance = distances[&node_id].saturating_add(1);
        for next in adjacency.get(&node_id).into_iter().flatten() {
            if distances.contains_key(next) {
                continue;
            }
            distances.insert(next.clone(), next_distance);
            pending.push_back(next.clone());
        }
    }
    distances
}

fn selector_equality_relation_fixtures_v0() -> Vec<OmenaDiffSelectorEqualityRelationReportV0> {
    [(".button::before", ".button::before")]
        .into_iter()
        .map(|(left, right)| {
            let witness = selector_equality_witness_v0(left, right);
            OmenaDiffSelectorEqualityRelationReportV0 {
                relation_id: "pseudo-element-selector-equality",
                left: witness.left,
                right: witness.right,
                equal: witness.equal,
            }
        })
        .collect()
}

fn hyperedge(
    id: &str,
    from: &str,
    to: &str,
    edge_kind: UnifiedHypergraphEdgeKindV0,
) -> UnifiedHypergraphHyperedgeV0 {
    let source_edge_kind = edge_kind.as_wire_label();
    UnifiedHypergraphHyperedgeV0 {
        schema_version: "0",
        product: "omena-diff-test.reachability-fixture",
        layer_marker: "hypergraph-ifds",
        feature_gate: "hypergraph-ifds",
        hyperedge_id: id.to_string(),
        edge_kind,
        source_summary_edge_id: id.to_string(),
        source_edge_kind,
        source_status: "known",
        tail_node_ids: vec![from.to_string()],
        head_node_id: to.to_string(),
        order_significant_tail: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_reachability_oracle_matches_public_batch_and_streaming_paths() {
        let report = summarize_reachability_second_oracle_equivalence_v0();

        assert!(report.all_sets_equal, "{report:#?}");
        assert!(report.streaming_matches_batch, "{report:#?}");
        assert!(report.all_reachability_bitset_parity_equal, "{report:#?}");
        assert!(report.all_closure_hash_bitset_parity_equal, "{report:#?}");
        assert!(report.product_reachability_parity_with_batch, "{report:#?}");
        assert!(report.all_fact_keys_batch_incremental_equal, "{report:#?}");
        assert!(report.all_fact_keys_three_way_equal, "{report:#?}");
        assert!(report.all_fact_keys_four_way_equal, "{report:#?}");
        assert!(report.selector_relations_equal);
        assert!(
            report.fixture_count > 0,
            "fact-key equivalence requires a non-empty corpus: {report:#?}"
        );
        assert!(
            report.multi_edge_closure_fixture_count > 0,
            "fact-key equivalence requires a multi-edge closure fixture: {report:#?}"
        );
        assert!(
            report.value_carrying_fact_key_fixture_count > 0,
            "fact-key equivalence requires a value-carrying fixture: {report:#?}"
        );
        assert!(
            report.strict_demand_projection_fixture_count > 0,
            "demand fact-key equivalence requires a strict structural projection: {report:#?}"
        );
        assert!(
            report.multi_module_closure_hash_fixture_count > 0,
            "closure hash parity requires a multi-module fixture: {report:#?}"
        );
        assert!(
            report.files.iter().any(|file| {
                file.fixture_id == "multi-hop-composes-sass-chain"
                    && file
                        .baseline_reachable_node_ids
                        .iter()
                        .any(|node| node == "styleSymbol|/workspace/terminal.module.scss|terminal")
            }),
            "fixture corpus must include a terminal multi-hop reachability node: {report:#?}"
        );
        assert!(
            report.files.iter().any(|file| {
                file.fixture_id == "value-carrying-composes-chain"
                    && file
                        .batch_fact_keys
                        .iter()
                        .any(|key| key.ends_with("|finiteSet:base,btn,primary"))
            }),
            "fixture corpus must include a compose-widened finite-set fact key: {report:#?}"
        );
        assert!(
            report.files.iter().any(|file| {
                file.fixture_id == "value-carrying-composes-chain"
                    && file.has_strict_demand_projection
                    && file.demand_fact_keys == file.projected_batch_fact_keys
                    && file
                        .demand_fact_keys
                        .iter()
                        .any(|key| key.ends_with("|finiteSet:base,btn"))
            }),
            "fixture corpus must include a value-carrying demand projection: {report:#?}"
        );
    }

    #[test]
    fn fact_key_three_way_sets_are_run_deterministic() -> Result<(), serde_json::Error> {
        let first_report = summarize_reachability_second_oracle_equivalence_v0();
        let second_report = summarize_reachability_second_oracle_equivalence_v0();
        let first = fact_key_vectors_for_determinism(&first_report);
        let second = fact_key_vectors_for_determinism(&second_report);

        assert_eq!(
            serde_json::to_string(&first)?,
            serde_json::to_string(&second)?
        );
        Ok(())
    }

    fn fact_key_vectors_for_determinism(
        report: &OmenaDiffReachabilityEquivalenceReportV0,
    ) -> Vec<FactKeyVectorSnapshotV0<'_>> {
        report
            .files
            .iter()
            .map(|file| FactKeyVectorSnapshotV0 {
                fixture_id: file.fixture_id.as_str(),
                batch_fact_keys: file.batch_fact_keys.as_slice(),
                incremental_fact_keys: file.incremental_fact_keys.as_slice(),
                ascent_fact_keys: file.ascent_fact_keys.as_slice(),
                demand_fact_keys: file.demand_fact_keys.as_slice(),
                projected_batch_fact_keys: file.projected_batch_fact_keys.as_slice(),
            })
            .collect()
    }

    #[derive(serde::Serialize)]
    struct FactKeyVectorSnapshotV0<'a> {
        fixture_id: &'a str,
        batch_fact_keys: &'a [String],
        incremental_fact_keys: &'a [String],
        ascent_fact_keys: &'a [String],
        demand_fact_keys: &'a [String],
        projected_batch_fact_keys: &'a [String],
    }
}

//! Differential witness for native CSS branch-fold edits and reachability pruning.

use std::collections::BTreeSet;

use omena_parser::StyleDialect;
use omena_scss_eval::{
    OmenaScssEvalControlFlowBlockIdV0, OmenaScssEvalControlFlowGraphBlockV0,
    OmenaScssEvalControlFlowGraphV0, OmenaScssEvalNativeCssStaticEditPlanV0,
    OmenaScssEvalNativeCssStaticEditV0, build_scss_control_flow_graph,
    summarize_native_css_static_edit_plan, summarize_scss_control_flow_prune_reachability,
};
use serde::{Deserialize, Serialize};

const FOLD_PRUNE_BASELINE_SOURCE: &str =
    include_str!("../regressions/native-css-fold-prune-edited-css.json");

const FOLD_PRUNE_AGREEMENT_FIXTURES: &[FoldPruneFixtureV0] = &[
    FoldPruneFixtureV0 {
        id: "truthy-else-branch",
        source: "@when supports(display: grid) { .then-grid { color: green; } } @else { .else-grid { color: red; } }",
        fixture_kind: FoldPruneFixtureKindV0::Agreement,
        expected_plan: FoldPruneExpectedPlanV0 {
            when_rule_edit_count: 1,
            if_function_edit_count: 0,
            function_call_edit_count: 0,
        },
        artifact_reason: None,
    },
    FoldPruneFixtureV0 {
        id: "falsey-else-branch",
        source: "@when supports(not (display: grid)) { .then-off { color: green; } } @else { .else-off { color: red; } }",
        fixture_kind: FoldPruneFixtureKindV0::Agreement,
        expected_plan: FoldPruneExpectedPlanV0 {
            when_rule_edit_count: 1,
            if_function_edit_count: 0,
            function_call_edit_count: 0,
        },
        artifact_reason: None,
    },
    FoldPruneFixtureV0 {
        id: "falsey-probed-then-slot",
        source: "@when supports(not (display: grid)) { @when media(min-width: 10px) { .then-probe-dead { color: red; } } }",
        fixture_kind: FoldPruneFixtureKindV0::Agreement,
        expected_plan: FoldPruneExpectedPlanV0 {
            when_rule_edit_count: 1,
            if_function_edit_count: 0,
            function_call_edit_count: 0,
        },
        artifact_reason: None,
    },
    FoldPruneFixtureV0 {
        id: "truthy-probed-then-slot",
        source: "@when supports(display: grid) { @when media(min-width: 10px) { .then-probe-live { color: green; } } }",
        fixture_kind: FoldPruneFixtureKindV0::Agreement,
        expected_plan: FoldPruneExpectedPlanV0 {
            when_rule_edit_count: 1,
            if_function_edit_count: 0,
            function_call_edit_count: 0,
        },
        artifact_reason: None,
    },
    FoldPruneFixtureV0 {
        id: "declined-conditional-else",
        source: "@when supports(not (display: grid)) { .declined-then { color: green; } } @else when(media(width >= 1px)) { .declined-else { color: red; } }",
        fixture_kind: FoldPruneFixtureKindV0::Agreement,
        expected_plan: FoldPruneExpectedPlanV0 {
            when_rule_edit_count: 0,
            if_function_edit_count: 0,
            function_call_edit_count: 0,
        },
        artifact_reason: None,
    },
    FoldPruneFixtureV0 {
        id: "runtime-condition-branch",
        source: "@when media(width >= 1px) { .runtime-then { color: green; } } @else { .runtime-else { color: red; } }",
        fixture_kind: FoldPruneFixtureKindV0::Agreement,
        expected_plan: FoldPruneExpectedPlanV0 {
            when_rule_edit_count: 0,
            if_function_edit_count: 0,
            function_call_edit_count: 0,
        },
        artifact_reason: None,
    },
    FoldPruneFixtureV0 {
        id: "independent-branch-sites",
        source: "@when supports(display: grid) { .multi-a { color: green; } } @else { .multi-a-fallback { color: red; } } .separator { color: black; } @when supports(not (display: grid)) { .multi-b { color: blue; } }",
        fixture_kind: FoldPruneFixtureKindV0::Agreement,
        expected_plan: FoldPruneExpectedPlanV0 {
            when_rule_edit_count: 2,
            if_function_edit_count: 0,
            function_call_edit_count: 0,
        },
        artifact_reason: None,
    },
];

const FOLD_PRUNE_ARTIFACT_FIXTURES: &[FoldPruneFixtureV0] = &[
    FoldPruneFixtureV0 {
        id: "nested-probe-consumes-else-edge",
        source: "@when supports(display: grid) { @when media(min-width: 10px) { .probe { color: green; } } } @else { .fallback { color: red; } }",
        fixture_kind: FoldPruneFixtureKindV0::Artifact,
        expected_plan: FoldPruneExpectedPlanV0 {
            when_rule_edit_count: 1,
            if_function_edit_count: 0,
            function_call_edit_count: 0,
        },
        artifact_reason: Some("adjacencyElseEdgeConsumedByNestedProbe"),
    },
    FoldPruneFixtureV0 {
        id: "chain-death-live-else",
        source: "@when supports(not (display: grid)) { @when media(min-width: 10px) { .probe { color: green; } } } @else { .live-else { color: red; } }",
        fixture_kind: FoldPruneFixtureKindV0::Artifact,
        expected_plan: FoldPruneExpectedPlanV0 {
            when_rule_edit_count: 1,
            if_function_edit_count: 0,
            function_call_edit_count: 0,
        },
        artifact_reason: Some("chainDeathLeaksIntoLiveElse"),
    },
    FoldPruneFixtureV0 {
        id: "value-function-fallthrough-poison",
        source: ".value { color: if(supports(display: grid): green; else: red); } @when supports(display: grid) { .then { color: green; } } @else { .else { color: red; } }",
        fixture_kind: FoldPruneFixtureKindV0::Artifact,
        expected_plan: FoldPruneExpectedPlanV0 {
            when_rule_edit_count: 1,
            if_function_edit_count: 1,
            function_call_edit_count: 0,
        },
        artifact_reason: Some("ifFunctionFallthroughPoisonsRegion"),
    },
    FoldPruneFixtureV0 {
        id: "nested-decided-branch-preserved",
        source: "@when supports(display: grid) { @when supports(display: grid) { .nested-kept { color: green; } } }",
        fixture_kind: FoldPruneFixtureKindV0::Artifact,
        expected_plan: FoldPruneExpectedPlanV0 {
            when_rule_edit_count: 1,
            if_function_edit_count: 0,
            function_call_edit_count: 0,
        },
        artifact_reason: Some("foldSinglePassSkipsNestedDecided"),
    },
    FoldPruneFixtureV0 {
        id: "native-function-value-fold-excluded",
        source: "@function --gap() returns <length> { result: 1px; } .card { margin: --gap(); } @when media(width >= 1px) { .runtime { color: green; } }",
        fixture_kind: FoldPruneFixtureKindV0::Artifact,
        expected_plan: FoldPruneExpectedPlanV0 {
            when_rule_edit_count: 0,
            if_function_edit_count: 0,
            function_call_edit_count: 1,
        },
        artifact_reason: Some("valueFoldExcludedFromBranchReachabilityDomain"),
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FoldPruneFixtureV0 {
    id: &'static str,
    source: &'static str,
    fixture_kind: FoldPruneFixtureKindV0,
    expected_plan: FoldPruneExpectedPlanV0,
    artifact_reason: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FoldPruneFixtureKindV0 {
    Agreement,
    Artifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FoldPruneExpectedPlanV0 {
    when_rule_edit_count: usize,
    if_function_edit_count: usize,
    function_call_edit_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffNativeCssFoldPruneReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub fixture_count: usize,
    pub agreement_fixture_count: usize,
    pub artifact_fixture_count: usize,
    pub compared_slot_count: usize,
    pub matching_slot_count: usize,
    pub ambiguous_slot_count: usize,
    pub undecided_fixture_count: usize,
    pub prune_dead_slot_count: usize,
    pub keep_live_slot_count: usize,
    pub probe_then_dead_slot_count: usize,
    pub value_fold_in_agreement_fixture_count: usize,
    pub deterministic_graph_count: usize,
    pub frozen_fold_matches_baseline: bool,
    pub all_compared_slots_match: bool,
    pub all_conservative_keep_slots_have_no_branch_edit: bool,
    pub artifact_ledger_matches: bool,
    pub floors_hold: bool,
    pub fixtures: Vec<OmenaDiffNativeCssFoldPruneFixtureReportV0>,
    pub artifact_records: Vec<OmenaDiffNativeCssFoldPruneArtifactRecordV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffNativeCssFoldPruneFixtureReportV0 {
    pub id: &'static str,
    pub fixture_kind: &'static str,
    pub source: &'static str,
    pub graph_available: bool,
    pub plan_available: bool,
    pub fixpoint_available: bool,
    pub graph_block_count_matches_fixpoint: bool,
    pub fixpoint_ids_resolve: bool,
    pub expected_when_rule_edit_count: usize,
    pub actual_when_rule_edit_count: usize,
    pub if_function_edit_count: usize,
    pub function_call_edit_count: usize,
    pub value_fold_free_for_agreement: bool,
    pub edited_css: String,
    pub branch_sites: Vec<OmenaDiffNativeCssFoldPruneBranchSiteReportV0>,
    pub plan_shape_matches_expectation: bool,
    pub conservative_keep_slots_have_no_branch_edit: bool,
    pub frozen_fold_matches_baseline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffNativeCssFoldPruneBranchSiteReportV0 {
    pub site_index: usize,
    pub block_id: u32,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub paired_else_block_id: Option<u32>,
    pub paired_else_span_start: Option<usize>,
    pub paired_else_span_end: Option<usize>,
    pub source_adjacent_else_block_id: Option<u32>,
    pub source_adjacent_else_span_start: Option<usize>,
    pub source_adjacent_else_span_end: Option<usize>,
    pub probe_block_id: Option<u32>,
    pub probe_span_start: Option<usize>,
    pub probe_span_end: Option<usize>,
    pub fold_then: FoldPruneBranchVerdictV0,
    pub fold_else: Option<FoldPruneBranchVerdictV0>,
    pub fixpoint_then: Option<FoldPruneReachabilityVerdictV0>,
    pub fixpoint_else: Option<FoldPruneReachabilityVerdictV0>,
    pub pair_precondition: bool,
    pub decider_live_precondition: bool,
    pub probe_terminal_precondition: bool,
    pub then_compared: bool,
    pub else_compared: bool,
    pub then_matches: Option<bool>,
    pub else_matches: Option<bool>,
    pub branch_edit_keyed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffNativeCssFoldPruneArtifactRecordV0 {
    pub fixture_id: &'static str,
    pub site_index: Option<usize>,
    pub slot_side: &'static str,
    pub block_span_start: usize,
    pub block_span_end: usize,
    pub reason_tag: &'static str,
    pub observed: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FoldPruneBranchVerdictV0 {
    Keep,
    Prune,
    Undecided,
    Ambiguous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FoldPruneReachabilityVerdictV0 {
    Live,
    Dead,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FoldPruneEditedCssBaselineV0 {
    schema_version: String,
    product: String,
    fixtures: Vec<FoldPruneEditedCssBaselineFixtureV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FoldPruneEditedCssBaselineFixtureV0 {
    id: String,
    edited_css: String,
}

pub fn summarize_native_css_fold_prune_branch_agreement_v0() -> OmenaDiffNativeCssFoldPruneReportV0
{
    let baseline = fold_prune_baseline();
    let fixtures = all_fold_prune_fixtures()
        .iter()
        .map(|fixture| fold_prune_fixture_report(fixture, &baseline))
        .collect::<Vec<_>>();
    let artifact_records = artifact_records(&fixtures);
    let compared_slot_count = fixtures
        .iter()
        .flat_map(|fixture| fixture.branch_sites.iter())
        .map(|site| usize::from(site.then_compared) + usize::from(site.else_compared))
        .sum();
    let matching_slot_count = fixtures
        .iter()
        .flat_map(|fixture| fixture.branch_sites.iter())
        .map(|site| {
            usize::from(site.then_matches == Some(true))
                + usize::from(site.else_matches == Some(true))
        })
        .sum();
    let ambiguous_slot_count = fixtures
        .iter()
        .flat_map(|fixture| fixture.branch_sites.iter())
        .map(|site| {
            usize::from(site.fold_then == FoldPruneBranchVerdictV0::Ambiguous)
                + usize::from(site.fold_else == Some(FoldPruneBranchVerdictV0::Ambiguous))
        })
        .sum();
    let undecided_fixture_count = fixtures
        .iter()
        .filter(|fixture| {
            fixture.branch_sites.iter().any(|site| {
                site.fold_then == FoldPruneBranchVerdictV0::Undecided
                    || site.fold_else == Some(FoldPruneBranchVerdictV0::Undecided)
            })
        })
        .count();
    let prune_dead_slot_count = fixtures
        .iter()
        .filter(|fixture| fixture.fixture_kind == "agreement")
        .flat_map(|fixture| fixture.branch_sites.iter())
        .map(prune_dead_slots)
        .sum();
    let keep_live_slot_count = fixtures
        .iter()
        .filter(|fixture| fixture.fixture_kind == "agreement")
        .flat_map(|fixture| fixture.branch_sites.iter())
        .map(keep_live_slots)
        .sum();
    let probe_then_dead_slot_count = fixtures
        .iter()
        .filter(|fixture| fixture.fixture_kind == "agreement")
        .flat_map(|fixture| fixture.branch_sites.iter())
        .filter(|site| {
            site.then_compared
                && site.fold_then == FoldPruneBranchVerdictV0::Prune
                && site.fixpoint_then == Some(FoldPruneReachabilityVerdictV0::Dead)
        })
        .count();
    let value_fold_in_agreement_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.fixture_kind == "agreement")
        .filter(|fixture| !fixture.value_fold_free_for_agreement)
        .count();
    let deterministic_graph_count = fixtures
        .iter()
        .filter(|fixture| {
            fixture.graph_available
                && fixture.fixpoint_available
                && fixture.graph_block_count_matches_fixpoint
                && fixture.fixpoint_ids_resolve
        })
        .count();
    let all_compared_slots_match =
        compared_slot_count > 0 && matching_slot_count == compared_slot_count;
    let all_conservative_keep_slots_have_no_branch_edit = fixtures
        .iter()
        .all(|fixture| fixture.conservative_keep_slots_have_no_branch_edit);
    let frozen_fold_matches_baseline = fixtures
        .iter()
        .all(|fixture| fixture.frozen_fold_matches_baseline);
    let artifact_ledger_matches = artifact_records == expected_artifact_records();
    let floors_hold = prune_dead_slot_count >= 1
        && probe_then_dead_slot_count >= 1
        && keep_live_slot_count >= 1
        && undecided_fixture_count >= 2
        && ambiguous_slot_count == 0
        && value_fold_in_agreement_fixture_count == 0
        && fixtures
            .iter()
            .filter(|fixture| fixture.fixture_kind == "agreement")
            .flat_map(|fixture| fixture.branch_sites.iter())
            .filter(|site| site.then_compared || site.else_compared)
            .all(|site| {
                site.decider_live_precondition
                    && (!site.else_compared || site.pair_precondition)
                    && (!site.then_compared || site.probe_terminal_precondition)
            });

    OmenaDiffNativeCssFoldPruneReportV0 {
        schema_version: "0",
        product: "omena-diff-test.native-css-fold-prune-branch-agreement",
        fixture_count: fixtures.len(),
        agreement_fixture_count: FOLD_PRUNE_AGREEMENT_FIXTURES.len(),
        artifact_fixture_count: FOLD_PRUNE_ARTIFACT_FIXTURES.len(),
        compared_slot_count,
        matching_slot_count,
        ambiguous_slot_count,
        undecided_fixture_count,
        prune_dead_slot_count,
        keep_live_slot_count,
        probe_then_dead_slot_count,
        value_fold_in_agreement_fixture_count,
        deterministic_graph_count,
        frozen_fold_matches_baseline,
        all_compared_slots_match,
        all_conservative_keep_slots_have_no_branch_edit,
        artifact_ledger_matches,
        floors_hold,
        fixtures,
        artifact_records,
    }
}

fn all_fold_prune_fixtures() -> Vec<FoldPruneFixtureV0> {
    FOLD_PRUNE_AGREEMENT_FIXTURES
        .iter()
        .chain(FOLD_PRUNE_ARTIFACT_FIXTURES.iter())
        .copied()
        .collect()
}

fn fold_prune_fixture_report(
    fixture: &FoldPruneFixtureV0,
    baseline: &FoldPruneEditedCssBaselineV0,
) -> OmenaDiffNativeCssFoldPruneFixtureReportV0 {
    let plan = summarize_native_css_static_edit_plan(fixture.source, StyleDialect::Css);
    let graph = build_scss_control_flow_graph(fixture.source, StyleDialect::Css);
    let fixpoint =
        summarize_scss_control_flow_prune_reachability(fixture.source, StyleDialect::Css);
    let branch_sites = match (&plan, &graph, &fixpoint) {
        (Some(plan), Some(graph), Some(fixpoint)) => {
            fold_prune_branch_site_reports(fixture.source, plan, graph, fixpoint)
        }
        _ => Vec::new(),
    };
    let graph_block_count_matches_fixpoint = graph
        .as_ref()
        .zip(fixpoint.as_ref())
        .is_some_and(|(graph, fixpoint)| graph.blocks.len() == fixpoint.block_count);
    let fixpoint_ids_resolve = graph
        .as_ref()
        .zip(fixpoint.as_ref())
        .is_some_and(|(graph, fixpoint)| fixpoint_ids_resolve(graph, fixpoint));
    let actual_when_rule_edit_count = plan
        .as_ref()
        .map(|plan| plan.when_rule_edit_count)
        .unwrap_or(0);
    let if_function_edit_count = plan
        .as_ref()
        .map(|plan| plan.if_function_edit_count)
        .unwrap_or(0);
    let function_call_edit_count = plan
        .as_ref()
        .map(|plan| plan.function_call_edit_count)
        .unwrap_or(0);
    let edited_css = plan
        .as_ref()
        .map(|plan| plan.edited_css.clone())
        .unwrap_or_else(|| fixture.source.to_string());
    let value_fold_free_for_agreement = fixture.fixture_kind != FoldPruneFixtureKindV0::Agreement
        || if_function_edit_count + function_call_edit_count == 0;
    let plan_shape_matches_expectation = actual_when_rule_edit_count
        == fixture.expected_plan.when_rule_edit_count
        && if_function_edit_count == fixture.expected_plan.if_function_edit_count
        && function_call_edit_count == fixture.expected_plan.function_call_edit_count;
    let conservative_keep_slots_have_no_branch_edit = branch_sites
        .iter()
        .filter(|site| {
            site.fold_then == FoldPruneBranchVerdictV0::Undecided
                || site.fold_else == Some(FoldPruneBranchVerdictV0::Undecided)
        })
        .all(|site| !site.branch_edit_keyed);
    let frozen_fold_matches_baseline = baseline
        .fixtures
        .iter()
        .find(|entry| entry.id == fixture.id)
        .is_some_and(|entry| entry.edited_css == edited_css);

    OmenaDiffNativeCssFoldPruneFixtureReportV0 {
        id: fixture.id,
        fixture_kind: match fixture.fixture_kind {
            FoldPruneFixtureKindV0::Agreement => "agreement",
            FoldPruneFixtureKindV0::Artifact => "artifact",
        },
        source: fixture.source,
        graph_available: graph.is_some(),
        plan_available: plan.is_some(),
        fixpoint_available: fixpoint.is_some(),
        graph_block_count_matches_fixpoint,
        fixpoint_ids_resolve,
        expected_when_rule_edit_count: fixture.expected_plan.when_rule_edit_count,
        actual_when_rule_edit_count,
        if_function_edit_count,
        function_call_edit_count,
        value_fold_free_for_agreement,
        edited_css,
        branch_sites,
        plan_shape_matches_expectation,
        conservative_keep_slots_have_no_branch_edit,
        frozen_fold_matches_baseline,
    }
}

fn fold_prune_branch_site_reports(
    source: &str,
    plan: &OmenaScssEvalNativeCssStaticEditPlanV0,
    graph: &OmenaScssEvalControlFlowGraphV0,
    fixpoint: &omena_scss_eval::OmenaScssEvalControlFlowPruneReachabilityV0,
) -> Vec<OmenaDiffNativeCssFoldPruneBranchSiteReportV0> {
    let reachable = fixpoint
        .reachable_block_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let unreachable = fixpoint
        .unreachable_block_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    graph
        .blocks
        .iter()
        .filter(|block| block.block.at_rule_name.eq_ignore_ascii_case("@when"))
        .enumerate()
        .map(|(site_index, block)| {
            fold_prune_branch_site_report(
                source,
                plan,
                graph,
                &reachable,
                &unreachable,
                site_index,
                block,
            )
        })
        .collect()
}

fn fold_prune_branch_site_report(
    source: &str,
    plan: &OmenaScssEvalNativeCssStaticEditPlanV0,
    graph: &OmenaScssEvalControlFlowGraphV0,
    reachable: &BTreeSet<OmenaScssEvalControlFlowBlockIdV0>,
    unreachable: &BTreeSet<OmenaScssEvalControlFlowBlockIdV0>,
    site_index: usize,
    block: &OmenaScssEvalControlFlowGraphBlockV0,
) -> OmenaDiffNativeCssFoldPruneBranchSiteReportV0 {
    let paired_else = paired_else_block(graph, block.id);
    let source_adjacent_else = source_adjacent_else_block(graph, block);
    let probe = then_probe_block(graph, block.id);
    let branch_edit = keyed_when_branch_edit(plan, block.block.source_span_start);
    let (fold_then, fold_else) =
        classify_when_branch_fold(source, block, source_adjacent_else, branch_edit);
    let decider_live_precondition = reachable.contains(&block.id);
    let pair_precondition = paired_else.is_some();
    let probe_terminal_precondition =
        probe.is_some_and(|probe| graph_block_is_terminal(graph, probe.id));
    let fixpoint_else = paired_else.map(|else_block| {
        if unreachable.contains(&else_block.id) {
            FoldPruneReachabilityVerdictV0::Dead
        } else {
            FoldPruneReachabilityVerdictV0::Live
        }
    });
    let fixpoint_then = probe.map(|probe| {
        if unreachable.contains(&probe.id) {
            FoldPruneReachabilityVerdictV0::Dead
        } else {
            FoldPruneReachabilityVerdictV0::Live
        }
    });
    let then_compared = decider_live_precondition
        && probe_terminal_precondition
        && matches!(
            fold_then,
            FoldPruneBranchVerdictV0::Keep | FoldPruneBranchVerdictV0::Prune
        );
    let else_compared = decider_live_precondition
        && pair_precondition
        && matches!(
            fold_else,
            Some(FoldPruneBranchVerdictV0::Keep | FoldPruneBranchVerdictV0::Prune)
        );
    let then_matches =
        then_compared.then(|| fold_verdict_matches_reachability(fold_then, fixpoint_then));
    let else_matches = else_compared.then(|| {
        fold_else
            .map(|fold_else| fold_verdict_matches_reachability(fold_else, fixpoint_else))
            .unwrap_or(false)
    });

    OmenaDiffNativeCssFoldPruneBranchSiteReportV0 {
        site_index,
        block_id: block.id.0,
        source_span_start: block.block.source_span_start,
        source_span_end: block.block.source_span_end,
        paired_else_block_id: paired_else.map(|else_block| else_block.id.0),
        paired_else_span_start: paired_else.map(|else_block| else_block.block.source_span_start),
        paired_else_span_end: paired_else.map(|else_block| else_block.block.source_span_end),
        source_adjacent_else_block_id: source_adjacent_else.map(|else_block| else_block.id.0),
        source_adjacent_else_span_start: source_adjacent_else
            .map(|else_block| else_block.block.source_span_start),
        source_adjacent_else_span_end: source_adjacent_else
            .map(|else_block| else_block.block.source_span_end),
        probe_block_id: probe.map(|probe| probe.id.0),
        probe_span_start: probe.map(|probe| probe.block.source_span_start),
        probe_span_end: probe.map(|probe| probe.block.source_span_end),
        fold_then,
        fold_else,
        fixpoint_then,
        fixpoint_else,
        pair_precondition,
        decider_live_precondition,
        probe_terminal_precondition,
        then_compared,
        else_compared,
        then_matches,
        else_matches,
        branch_edit_keyed: branch_edit.is_some(),
    }
}

fn classify_when_branch_fold(
    source: &str,
    block: &OmenaScssEvalControlFlowGraphBlockV0,
    paired_else: Option<&OmenaScssEvalControlFlowGraphBlockV0>,
    edit: Option<&OmenaScssEvalNativeCssStaticEditV0>,
) -> (FoldPruneBranchVerdictV0, Option<FoldPruneBranchVerdictV0>) {
    let Some(edit) = edit else {
        return (
            FoldPruneBranchVerdictV0::Undecided,
            paired_else.map(|_| FoldPruneBranchVerdictV0::Undecided),
        );
    };
    let then_inner = block_inner_source(source, block);
    if edit.replacement.is_empty() && edit.end == block.block.source_span_end {
        return (
            FoldPruneBranchVerdictV0::Prune,
            paired_else.map(|_| FoldPruneBranchVerdictV0::Undecided),
        );
    }
    if edit.end == block.block.source_span_end && then_inner.as_deref() == Some(&edit.replacement) {
        return (
            FoldPruneBranchVerdictV0::Keep,
            paired_else.map(|_| FoldPruneBranchVerdictV0::Undecided),
        );
    }
    let Some(else_block) = paired_else else {
        return (FoldPruneBranchVerdictV0::Ambiguous, None);
    };
    if edit.end != else_block.block.source_span_end {
        return (
            FoldPruneBranchVerdictV0::Ambiguous,
            Some(FoldPruneBranchVerdictV0::Ambiguous),
        );
    }
    let else_inner = block_inner_source(source, else_block);
    let matches_then = then_inner.as_deref() == Some(&edit.replacement);
    let matches_else = else_inner.as_deref() == Some(&edit.replacement);
    match (matches_then, matches_else) {
        (true, false) => (
            FoldPruneBranchVerdictV0::Keep,
            Some(FoldPruneBranchVerdictV0::Prune),
        ),
        (false, true) => (
            FoldPruneBranchVerdictV0::Prune,
            Some(FoldPruneBranchVerdictV0::Keep),
        ),
        _ => (
            FoldPruneBranchVerdictV0::Ambiguous,
            Some(FoldPruneBranchVerdictV0::Ambiguous),
        ),
    }
}

fn block_inner_source(
    source: &str,
    block: &OmenaScssEvalControlFlowGraphBlockV0,
) -> Option<String> {
    let start = block.block.source_span_start;
    let end = block.block.source_span_end;
    let slice = source.get(start..end)?;
    let open_relative = slice.find('{')?;
    let open = start + open_relative;
    let mut depth = 0usize;
    for (relative_index, ch) in source.get(open..end)?.char_indices() {
        match ch {
            '{' => depth = depth.saturating_add(1),
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let close = open + relative_index;
                    return source.get(open + 1..close).map(str::to_string);
                }
            }
            _ => {}
        }
    }
    None
}

fn keyed_when_branch_edit(
    plan: &OmenaScssEvalNativeCssStaticEditPlanV0,
    source_span_start: usize,
) -> Option<&OmenaScssEvalNativeCssStaticEditV0> {
    let mut matches = plan
        .edits
        .iter()
        .filter(|edit| edit.edit_kind == "whenRuleBranchFold" && edit.start == source_span_start);
    let edit = matches.next()?;
    matches.next().is_none().then_some(edit)
}

fn paired_else_block(
    graph: &OmenaScssEvalControlFlowGraphV0,
    block_id: OmenaScssEvalControlFlowBlockIdV0,
) -> Option<&OmenaScssEvalControlFlowGraphBlockV0> {
    graph
        .edges
        .iter()
        .find(|edge| edge.source_block_id == block_id && edge.outcome == "else")
        .and_then(|edge| edge.target_block_id)
        .and_then(|target| graph_block_by_id(graph, target))
        .filter(|block| block.block.at_rule_name.eq_ignore_ascii_case("@else"))
}

fn source_adjacent_else_block<'graph>(
    graph: &'graph OmenaScssEvalControlFlowGraphV0,
    block: &OmenaScssEvalControlFlowGraphBlockV0,
) -> Option<&'graph OmenaScssEvalControlFlowGraphBlockV0> {
    graph
        .blocks
        .iter()
        .filter(|candidate| {
            candidate.block.at_rule_name.eq_ignore_ascii_case("@else")
                && block.block.source_span_end <= candidate.block.source_span_start
        })
        .min_by_key(|candidate| candidate.block.source_span_start)
}

fn then_probe_block(
    graph: &OmenaScssEvalControlFlowGraphV0,
    block_id: OmenaScssEvalControlFlowBlockIdV0,
) -> Option<&OmenaScssEvalControlFlowGraphBlockV0> {
    graph
        .edges
        .iter()
        .find(|edge| edge.source_block_id == block_id && edge.outcome == "then")
        .and_then(|edge| edge.target_block_id)
        .and_then(|target| graph_block_by_id(graph, target))
        .filter(|block| block.block.at_rule_name.eq_ignore_ascii_case("@when"))
}

fn graph_block_is_terminal(
    graph: &OmenaScssEvalControlFlowGraphV0,
    block_id: OmenaScssEvalControlFlowBlockIdV0,
) -> bool {
    graph
        .edges
        .iter()
        .filter(|edge| edge.source_block_id == block_id)
        .all(|edge| edge.target_block_id.is_none())
}

fn graph_block_by_id(
    graph: &OmenaScssEvalControlFlowGraphV0,
    block_id: OmenaScssEvalControlFlowBlockIdV0,
) -> Option<&OmenaScssEvalControlFlowGraphBlockV0> {
    graph.blocks.iter().find(|block| block.id == block_id)
}

fn fold_verdict_matches_reachability(
    fold: FoldPruneBranchVerdictV0,
    reachability: Option<FoldPruneReachabilityVerdictV0>,
) -> bool {
    matches!(
        (fold, reachability),
        (
            FoldPruneBranchVerdictV0::Prune,
            Some(FoldPruneReachabilityVerdictV0::Dead)
        ) | (
            FoldPruneBranchVerdictV0::Keep,
            Some(FoldPruneReachabilityVerdictV0::Live)
        )
    )
}

fn fixpoint_ids_resolve(
    graph: &OmenaScssEvalControlFlowGraphV0,
    fixpoint: &omena_scss_eval::OmenaScssEvalControlFlowPruneReachabilityV0,
) -> bool {
    fixpoint
        .reachable_block_ids
        .iter()
        .chain(fixpoint.unreachable_block_ids.iter())
        .all(|block_id| graph_block_by_id(graph, *block_id).is_some())
}

fn prune_dead_slots(site: &OmenaDiffNativeCssFoldPruneBranchSiteReportV0) -> usize {
    usize::from(
        site.then_compared
            && site.fold_then == FoldPruneBranchVerdictV0::Prune
            && site.fixpoint_then == Some(FoldPruneReachabilityVerdictV0::Dead),
    ) + usize::from(
        site.else_compared
            && site.fold_else == Some(FoldPruneBranchVerdictV0::Prune)
            && site.fixpoint_else == Some(FoldPruneReachabilityVerdictV0::Dead),
    )
}

fn keep_live_slots(site: &OmenaDiffNativeCssFoldPruneBranchSiteReportV0) -> usize {
    usize::from(
        site.then_compared
            && site.fold_then == FoldPruneBranchVerdictV0::Keep
            && site.fixpoint_then == Some(FoldPruneReachabilityVerdictV0::Live),
    ) + usize::from(
        site.else_compared
            && site.fold_else == Some(FoldPruneBranchVerdictV0::Keep)
            && site.fixpoint_else == Some(FoldPruneReachabilityVerdictV0::Live),
    )
}

fn artifact_records(
    fixtures: &[OmenaDiffNativeCssFoldPruneFixtureReportV0],
) -> Vec<OmenaDiffNativeCssFoldPruneArtifactRecordV0> {
    fixtures
        .iter()
        .filter(|fixture| fixture.fixture_kind == "artifact")
        .filter_map(|fixture| match fixture.id {
            "nested-probe-consumes-else-edge" => artifact_record_from_site(
                fixture,
                0,
                "else",
                "adjacencyElseEdgeConsumedByNestedProbe",
                "foldPrunesElseButGraphHasNoElsePair",
                |site| {
                    site.fold_else == Some(FoldPruneBranchVerdictV0::Prune)
                        && !site.pair_precondition
                        && site.source_adjacent_else_span_start.is_some()
                },
                |site| {
                    (
                        site.source_adjacent_else_span_start,
                        site.source_adjacent_else_span_end,
                    )
                },
            ),
            "chain-death-live-else" => artifact_record_from_site(
                fixture,
                0,
                "thenProbe",
                "chainDeathLeaksIntoLiveElse",
                "foldKeepsElseButProbeThenSlotIsNotTerminal",
                |site| {
                    site.fold_else == Some(FoldPruneBranchVerdictV0::Keep)
                        && !site.probe_terminal_precondition
                        && site.probe_span_start.is_some()
                },
                |site| (site.probe_span_start, site.probe_span_end),
            ),
            "value-function-fallthrough-poison" => artifact_record_from_site(
                fixture,
                0,
                "branchSite",
                "ifFunctionFallthroughPoisonsRegion",
                "foldHasBranchEditButDeciderIsUnreachable",
                |site| site.branch_edit_keyed && !site.decider_live_precondition,
                |site| (Some(site.source_span_start), Some(site.source_span_end)),
            ),
            "nested-decided-branch-preserved" => artifact_record_from_site(
                fixture,
                1,
                "then",
                "foldSinglePassSkipsNestedDecided",
                "nestedStaticBranchHasNoKeyedEdit",
                |site| {
                    !site.branch_edit_keyed && site.fold_then == FoldPruneBranchVerdictV0::Undecided
                },
                |site| (Some(site.source_span_start), Some(site.source_span_end)),
            ),
            "native-function-value-fold-excluded" => native_function_exclusion_record(fixture),
            _ => None,
        })
        .collect()
}

fn expected_artifact_records() -> Vec<OmenaDiffNativeCssFoldPruneArtifactRecordV0> {
    vec![
        OmenaDiffNativeCssFoldPruneArtifactRecordV0 {
            fixture_id: "nested-probe-consumes-else-edge",
            site_index: Some(0),
            slot_side: "else",
            block_span_start: 92,
            block_span_end: 127,
            reason_tag: "adjacencyElseEdgeConsumedByNestedProbe",
            observed: "foldPrunesElseButGraphHasNoElsePair",
        },
        OmenaDiffNativeCssFoldPruneArtifactRecordV0 {
            fixture_id: "chain-death-live-else",
            site_index: Some(0),
            slot_side: "thenProbe",
            block_span_start: 38,
            block_span_end: 95,
            reason_tag: "chainDeathLeaksIntoLiveElse",
            observed: "foldKeepsElseButProbeThenSlotIsNotTerminal",
        },
        OmenaDiffNativeCssFoldPruneArtifactRecordV0 {
            fixture_id: "value-function-fallthrough-poison",
            site_index: Some(0),
            slot_side: "branchSite",
            block_span_start: 65,
            block_span_end: 122,
            reason_tag: "ifFunctionFallthroughPoisonsRegion",
            observed: "foldHasBranchEditButDeciderIsUnreachable",
        },
        OmenaDiffNativeCssFoldPruneArtifactRecordV0 {
            fixture_id: "nested-decided-branch-preserved",
            site_index: Some(1),
            slot_side: "then",
            block_span_start: 32,
            block_span_end: 96,
            reason_tag: "foldSinglePassSkipsNestedDecided",
            observed: "nestedStaticBranchHasNoKeyedEdit",
        },
        OmenaDiffNativeCssFoldPruneArtifactRecordV0 {
            fixture_id: "native-function-value-fold-excluded",
            site_index: None,
            slot_side: "valueFunction",
            block_span_start: 0,
            block_span_end: 51,
            reason_tag: "valueFoldExcludedFromBranchReachabilityDomain",
            observed: "functionCallValueFoldHasNoBranchSlot",
        },
    ]
}

fn artifact_record_from_site(
    fixture: &OmenaDiffNativeCssFoldPruneFixtureReportV0,
    site_index: usize,
    slot_side: &'static str,
    reason_tag: &'static str,
    observed: &'static str,
    predicate: impl FnOnce(&OmenaDiffNativeCssFoldPruneBranchSiteReportV0) -> bool,
    span: impl FnOnce(&OmenaDiffNativeCssFoldPruneBranchSiteReportV0) -> (Option<usize>, Option<usize>),
) -> Option<OmenaDiffNativeCssFoldPruneArtifactRecordV0> {
    let site = fixture
        .branch_sites
        .iter()
        .find(|site| site.site_index == site_index)?;
    if !predicate(site) {
        return None;
    }
    let (block_span_start, block_span_end) = span(site);
    Some(OmenaDiffNativeCssFoldPruneArtifactRecordV0 {
        fixture_id: fixture.id,
        site_index: Some(site_index),
        slot_side,
        block_span_start: block_span_start?,
        block_span_end: block_span_end?,
        reason_tag,
        observed,
    })
}

fn native_function_exclusion_record(
    fixture: &OmenaDiffNativeCssFoldPruneFixtureReportV0,
) -> Option<OmenaDiffNativeCssFoldPruneArtifactRecordV0> {
    (fixture.function_call_edit_count > 0).then_some(OmenaDiffNativeCssFoldPruneArtifactRecordV0 {
        fixture_id: fixture.id,
        site_index: None,
        slot_side: "valueFunction",
        block_span_start: native_function_block_span(fixture.source)?.0,
        block_span_end: native_function_block_span(fixture.source)?.1,
        reason_tag: "valueFoldExcludedFromBranchReachabilityDomain",
        observed: "functionCallValueFoldHasNoBranchSlot",
    })
}

fn native_function_block_span(source: &str) -> Option<(usize, usize)> {
    let start = source.find("@function")?;
    let open = source.get(start..)?.find('{')? + start;
    let mut depth = 0usize;
    for (relative_index, ch) in source.get(open..)?.char_indices() {
        match ch {
            '{' => depth = depth.saturating_add(1),
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some((start, open + relative_index + 1));
                }
            }
            _ => {}
        }
    }
    None
}

fn fold_prune_baseline() -> FoldPruneEditedCssBaselineV0 {
    serde_json::from_str(FOLD_PRUNE_BASELINE_SOURCE).unwrap_or_else(|_| {
        FoldPruneEditedCssBaselineV0 {
            schema_version: "0".to_string(),
            product: "omena-diff-test.native-css-fold-prune-edited-css".to_string(),
            fixtures: Vec::new(),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn fold_prune_agreement_corpus_hits_branch_arms_and_excludes_value_folds() {
        let report = summarize_native_css_fold_prune_branch_agreement_v0();

        assert!(
            report
                .fixtures
                .iter()
                .filter(|fixture| fixture.fixture_kind == "agreement")
                .all(|fixture| fixture.plan_shape_matches_expectation
                    && fixture.value_fold_free_for_agreement),
            "{report:#?}"
        );
        assert!(
            report
                .fixtures
                .iter()
                .find(|fixture| fixture.id == "independent-branch-sites")
                .is_some_and(|fixture| fixture.actual_when_rule_edit_count == 2),
            "{report:#?}"
        );
    }

    #[test]
    fn when_branch_fold_classifier_assigns_expected_verdicts_per_arm() {
        let report = summarize_native_css_fold_prune_branch_agreement_v0();
        let fixture_by_id = report
            .fixtures
            .iter()
            .map(|fixture| (fixture.id, fixture))
            .collect::<BTreeMap<_, _>>();

        assert_site(
            &fixture_by_id,
            "truthy-else-branch",
            0,
            FoldPruneBranchVerdictV0::Keep,
            Some(FoldPruneBranchVerdictV0::Prune),
        );
        assert_site(
            &fixture_by_id,
            "falsey-else-branch",
            0,
            FoldPruneBranchVerdictV0::Prune,
            Some(FoldPruneBranchVerdictV0::Keep),
        );
        assert_site(
            &fixture_by_id,
            "falsey-probed-then-slot",
            0,
            FoldPruneBranchVerdictV0::Prune,
            None,
        );
        assert_site(
            &fixture_by_id,
            "truthy-probed-then-slot",
            0,
            FoldPruneBranchVerdictV0::Keep,
            None,
        );
        assert_site(
            &fixture_by_id,
            "declined-conditional-else",
            0,
            FoldPruneBranchVerdictV0::Undecided,
            Some(FoldPruneBranchVerdictV0::Undecided),
        );
        assert_site(
            &fixture_by_id,
            "runtime-condition-branch",
            0,
            FoldPruneBranchVerdictV0::Undecided,
            Some(FoldPruneBranchVerdictV0::Undecided),
        );
        assert_eq!(report.ambiguous_slot_count, 0, "{report:#?}");
    }

    #[test]
    fn fixpoint_branch_slot_verdicts_are_extracted_through_graph_pairing() {
        let report = summarize_native_css_fold_prune_branch_agreement_v0();
        let fixture_by_id = report
            .fixtures
            .iter()
            .map(|fixture| (fixture.id, fixture))
            .collect::<BTreeMap<_, _>>();

        assert_fixpoint_slot(
            &fixture_by_id,
            "truthy-else-branch",
            0,
            None,
            Some(FoldPruneReachabilityVerdictV0::Dead),
        );
        assert_fixpoint_slot(
            &fixture_by_id,
            "falsey-else-branch",
            0,
            None,
            Some(FoldPruneReachabilityVerdictV0::Live),
        );
        assert_fixpoint_slot(
            &fixture_by_id,
            "falsey-probed-then-slot",
            0,
            Some(FoldPruneReachabilityVerdictV0::Dead),
            None,
        );
        assert_fixpoint_slot(
            &fixture_by_id,
            "truthy-probed-then-slot",
            0,
            Some(FoldPruneReachabilityVerdictV0::Live),
            None,
        );
        assert!(
            report.fixtures.iter().all(|fixture| {
                fixture.graph_available
                    && fixture.fixpoint_available
                    && fixture.graph_block_count_matches_fixpoint
                    && fixture.fixpoint_ids_resolve
            }),
            "{report:#?}"
        );
    }

    #[test]
    fn native_css_fold_and_fixpoint_branch_verdicts_agree_on_faithful_slots() {
        let report = summarize_native_css_fold_prune_branch_agreement_v0();

        assert!(report.all_compared_slots_match, "{report:#?}");
        assert!(report.compared_slot_count >= 4, "{report:#?}");
    }

    #[test]
    fn fold_undecided_slots_carry_no_branch_fold_edit() {
        let report = summarize_native_css_fold_prune_branch_agreement_v0();

        assert!(
            report.all_conservative_keep_slots_have_no_branch_edit,
            "{report:#?}"
        );
        assert!(report.undecided_fixture_count >= 2, "{report:#?}");
    }

    #[test]
    fn fold_fixpoint_artifact_ledger_matches_pinned_divergences() {
        let report = summarize_native_css_fold_prune_branch_agreement_v0();

        assert!(report.artifact_ledger_matches, "{report:#?}");
        assert_eq!(report.artifact_records, expected_artifact_records());
    }

    #[test]
    fn agreement_corpus_meets_slot_level_floors() {
        let report = summarize_native_css_fold_prune_branch_agreement_v0();

        assert!(report.floors_hold, "{report:#?}");
        assert!(report.prune_dead_slot_count >= 1, "{report:#?}");
        assert!(report.probe_then_dead_slot_count >= 1, "{report:#?}");
        assert!(report.keep_live_slot_count >= 1, "{report:#?}");
        assert_eq!(report.ambiguous_slot_count, 0, "{report:#?}");
        assert_eq!(
            report.value_fold_in_agreement_fixture_count, 0,
            "{report:#?}"
        );
    }

    #[test]
    fn fold_output_bytes_match_committed_baseline() {
        let report = summarize_native_css_fold_prune_branch_agreement_v0();

        assert!(report.frozen_fold_matches_baseline, "{report:#?}");
    }

    fn assert_site(
        fixture_by_id: &BTreeMap<&str, &OmenaDiffNativeCssFoldPruneFixtureReportV0>,
        fixture_id: &str,
        site_index: usize,
        expected_then: FoldPruneBranchVerdictV0,
        expected_else: Option<FoldPruneBranchVerdictV0>,
    ) {
        let Some(fixture) = fixture_by_id.get(fixture_id) else {
            assert!(
                fixture_by_id.contains_key(fixture_id),
                "missing fixture {fixture_id}"
            );
            return;
        };
        let site = fixture
            .branch_sites
            .iter()
            .find(|site| site.site_index == site_index);
        assert!(site.is_some(), "missing site {fixture_id}#{site_index}");
        let Some(site) = site else {
            return;
        };
        assert_eq!(site.fold_then, expected_then, "{fixture:#?}");
        assert_eq!(site.fold_else, expected_else, "{fixture:#?}");
    }

    fn assert_fixpoint_slot(
        fixture_by_id: &BTreeMap<&str, &OmenaDiffNativeCssFoldPruneFixtureReportV0>,
        fixture_id: &str,
        site_index: usize,
        expected_then: Option<FoldPruneReachabilityVerdictV0>,
        expected_else: Option<FoldPruneReachabilityVerdictV0>,
    ) {
        let Some(fixture) = fixture_by_id.get(fixture_id) else {
            assert!(
                fixture_by_id.contains_key(fixture_id),
                "missing fixture {fixture_id}"
            );
            return;
        };
        let site = fixture
            .branch_sites
            .iter()
            .find(|site| site.site_index == site_index);
        assert!(site.is_some(), "missing site {fixture_id}#{site_index}");
        let Some(site) = site else {
            return;
        };
        assert_eq!(site.fixpoint_then, expected_then, "{fixture:#?}");
        assert_eq!(site.fixpoint_else, expected_else, "{fixture:#?}");
    }
}

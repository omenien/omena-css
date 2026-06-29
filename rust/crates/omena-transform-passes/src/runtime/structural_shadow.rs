use std::collections::BTreeSet;

use omena_parser::{
    StyleDialect, summarize_omena_parser_parity_lite, summarize_omena_parser_style_facts,
};
use omena_transform_cst::TransformPassKind;

use super::provenance::derive_transform_mutation_spans;
use crate::{
    TransformProvenanceMutationSpanV0, TransformStructuralIrShadowEquivalenceReportV0,
    TransformStructuralIrShadowFieldReportV0, TransformStructuralIrShadowFixtureReportV0,
    domains::{
        cascade_flatten::{
            collect_layer_flatten_proof_candidates_with_lexer,
            collect_scope_flatten_proof_candidates_with_lexer,
            flatten_css_layers_with_ir_transaction, flatten_css_layers_with_lexer,
            flatten_css_scopes_with_ir_transaction, flatten_css_scopes_with_lexer,
        },
        nesting::{unwrap_css_nesting_with_ir_transaction, unwrap_css_nesting_with_lexer},
        rule_cleanup::{
            dedupe_exact_css_rules_with_ir_transaction, dedupe_exact_css_rules_with_lexer,
            remove_empty_css_rules_with_ir_transaction, remove_empty_css_rules_with_lexer,
        },
        rule_merge::{
            merge_adjacent_same_block_css_selectors_with_ir_transaction,
            merge_adjacent_same_block_css_selectors_with_lexer,
            merge_adjacent_same_selector_css_rules_with_ir_transaction,
            merge_adjacent_same_selector_css_rules_with_lexer,
        },
        static_eval::{
            StaticMediaEvaluationOptions, evaluate_static_container_rules_with_ir_transaction,
            evaluate_static_container_rules_with_lexer,
            evaluate_static_media_rules_with_ir_transaction,
            evaluate_static_media_rules_with_lexer,
            evaluate_static_supports_rules_with_ir_transaction,
            evaluate_static_supports_rules_with_lexer,
        },
    },
};

const COMPARED_FIELDS: [&str; 6] = [
    "canonicalCssBytes",
    "selectorSet",
    "declarationSet",
    "cascadeOutcome",
    "mutationSpanRanges",
    "mutationCount",
];

#[derive(Debug, Clone, Copy)]
pub struct TransformStructuralIrShadowFixtureInputV0<'source> {
    pub fixture: &'source str,
    pub pass: TransformPassKind,
    pub dialect: StyleDialect,
    pub source: &'source str,
    pub closed_bundle: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StructuralShadowPathSnapshotV0 {
    output_css: String,
    mutation_count: usize,
    selector_values: Vec<String>,
    declaration_values: Vec<String>,
    cascade_values: Vec<String>,
    mutation_span_values: Vec<String>,
}

pub fn summarize_structural_ir_shadow_equivalence_v0()
-> TransformStructuralIrShadowEquivalenceReportV0 {
    let fixtures = structural_shadow_fixtures();
    summarize_structural_ir_shadow_equivalence_for_fixtures_v0(fixtures.as_slice())
}

pub fn summarize_structural_ir_shadow_equivalence_for_fixtures_v0(
    fixtures: &[TransformStructuralIrShadowFixtureInputV0<'_>],
) -> TransformStructuralIrShadowEquivalenceReportV0 {
    let reports = fixtures
        .iter()
        .copied()
        .map(structural_shadow_report_for_fixture)
        .collect::<Vec<_>>();
    let all_fields_match = reports.iter().all(|report| report.all_fields_match);

    TransformStructuralIrShadowEquivalenceReportV0 {
        schema_version: "0",
        product: "omena-transform-passes.structural-ir-shadow-equivalence",
        fixture_count: reports.len(),
        compared_pass_ids: compared_pass_ids(),
        compared_fields: COMPARED_FIELDS.to_vec(),
        reports,
        all_fields_match,
    }
}

fn structural_shadow_report_for_fixture(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
) -> TransformStructuralIrShadowFixtureReportV0 {
    let string_snapshot = string_path_snapshot(fixture);
    let ir_snapshot = ir_path_snapshot(fixture);
    let (ir_path_mutation_count, fields) = match ir_snapshot {
        Ok(ir_snapshot) => (
            Some(ir_snapshot.mutation_count),
            vec![
                shadow_field_report(
                    "canonicalCssBytes",
                    [string_snapshot.output_css.clone()],
                    [ir_snapshot.output_css],
                ),
                shadow_field_report(
                    "selectorSet",
                    string_snapshot.selector_values,
                    ir_snapshot.selector_values,
                ),
                shadow_field_report(
                    "declarationSet",
                    string_snapshot.declaration_values,
                    ir_snapshot.declaration_values,
                ),
                shadow_field_report(
                    "cascadeOutcome",
                    string_snapshot.cascade_values,
                    ir_snapshot.cascade_values,
                ),
                shadow_field_report(
                    "mutationSpanRanges",
                    string_snapshot.mutation_span_values,
                    ir_snapshot.mutation_span_values,
                ),
                shadow_field_report(
                    "mutationCount",
                    [string_snapshot.mutation_count.to_string()],
                    [ir_snapshot.mutation_count.to_string()],
                ),
            ],
        ),
        Err(error) => {
            let error = format!("irPathError:{error}");
            (
                None,
                vec![
                    shadow_field_report(
                        "canonicalCssBytes",
                        [string_snapshot.output_css],
                        [error.clone()],
                    ),
                    shadow_field_report(
                        "selectorSet",
                        string_snapshot.selector_values,
                        [error.clone()],
                    ),
                    shadow_field_report(
                        "declarationSet",
                        string_snapshot.declaration_values,
                        [error.clone()],
                    ),
                    shadow_field_report(
                        "cascadeOutcome",
                        string_snapshot.cascade_values,
                        [error.clone()],
                    ),
                    shadow_field_report(
                        "mutationSpanRanges",
                        string_snapshot.mutation_span_values,
                        [error.clone()],
                    ),
                    shadow_field_report(
                        "mutationCount",
                        [string_snapshot.mutation_count.to_string()],
                        [error],
                    ),
                ],
            )
        }
    };
    let all_fields_match = fields.iter().all(|field| field.matches);

    TransformStructuralIrShadowFixtureReportV0 {
        schema_version: "0",
        product: "omena-transform-passes.structural-ir-shadow-fixture",
        fixture: fixture.fixture.to_string(),
        pass_id: fixture.pass.id(),
        dialect: dialect_label(fixture.dialect),
        string_path_mutation_count: Some(string_snapshot.mutation_count),
        ir_path_mutation_count,
        fields,
        all_fields_match,
    }
}

fn string_path_snapshot(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
) -> StructuralShadowPathSnapshotV0 {
    let (output_css, mutation_count) = match fixture.pass {
        TransformPassKind::NestingUnwrap => {
            unwrap_css_nesting_with_lexer(fixture.source, fixture.dialect)
        }
        TransformPassKind::ScopeFlatten => {
            flatten_css_scopes_with_lexer(fixture.source, fixture.dialect)
        }
        TransformPassKind::LayerFlatten => {
            flatten_css_layers_with_lexer(fixture.source, fixture.dialect, fixture.closed_bundle)
        }
        TransformPassKind::RuleDeduplication => {
            dedupe_exact_css_rules_with_lexer(fixture.source, fixture.dialect)
        }
        TransformPassKind::RuleMerging => {
            merge_adjacent_same_selector_css_rules_with_lexer(fixture.source, fixture.dialect)
        }
        TransformPassKind::SelectorMerging => {
            merge_adjacent_same_block_css_selectors_with_lexer(fixture.source, fixture.dialect)
        }
        TransformPassKind::EmptyRuleRemoval => {
            remove_empty_css_rules_with_lexer(fixture.source, fixture.dialect)
        }
        TransformPassKind::SupportsStaticEval | TransformPassKind::DeadSupportsBranchRemoval => {
            evaluate_static_supports_rules_with_lexer(fixture.source, fixture.dialect)
        }
        TransformPassKind::MediaStaticEval | TransformPassKind::DeadMediaBranchRemoval => {
            evaluate_static_media_rules_with_lexer(
                fixture.source,
                fixture.dialect,
                StaticMediaEvaluationOptions::default(),
            )
        }
        TransformPassKind::ContainerStaticEval => {
            evaluate_static_container_rules_with_lexer(fixture.source, fixture.dialect)
        }
        _ => (fixture.source.to_string(), 0),
    };
    path_snapshot_from_output(fixture, output_css, mutation_count)
}

fn ir_path_snapshot(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
) -> Result<StructuralShadowPathSnapshotV0, String> {
    let (output_css, mutation_count) = match fixture.pass {
        TransformPassKind::NestingUnwrap => {
            unwrap_css_nesting_with_ir_transaction(fixture.source, fixture.dialect)
                .map_err(|error| format!("{error:?}"))?
        }
        TransformPassKind::ScopeFlatten => {
            flatten_css_scopes_with_ir_transaction(fixture.source, fixture.dialect)
                .map_err(|error| format!("{error:?}"))?
        }
        TransformPassKind::LayerFlatten => flatten_css_layers_with_ir_transaction(
            fixture.source,
            fixture.dialect,
            fixture.closed_bundle,
        )
        .map_err(|error| format!("{error:?}"))?,
        TransformPassKind::RuleDeduplication => {
            dedupe_exact_css_rules_with_ir_transaction(fixture.source, fixture.dialect)
                .map_err(|error| format!("{error:?}"))?
        }
        TransformPassKind::RuleMerging => {
            merge_adjacent_same_selector_css_rules_with_ir_transaction(
                fixture.source,
                fixture.dialect,
            )
            .map_err(|error| format!("{error:?}"))?
        }
        TransformPassKind::SelectorMerging => {
            merge_adjacent_same_block_css_selectors_with_ir_transaction(
                fixture.source,
                fixture.dialect,
            )
            .map_err(|error| format!("{error:?}"))?
        }
        TransformPassKind::EmptyRuleRemoval => {
            remove_empty_css_rules_with_ir_transaction(fixture.source, fixture.dialect)
                .map_err(|error| format!("{error:?}"))?
        }
        TransformPassKind::SupportsStaticEval | TransformPassKind::DeadSupportsBranchRemoval => {
            evaluate_static_supports_rules_with_ir_transaction(fixture.source, fixture.dialect)
                .map_err(|error| format!("{error:?}"))?
        }
        TransformPassKind::MediaStaticEval | TransformPassKind::DeadMediaBranchRemoval => {
            evaluate_static_media_rules_with_ir_transaction(
                fixture.source,
                fixture.dialect,
                StaticMediaEvaluationOptions::default(),
            )
            .map_err(|error| format!("{error:?}"))?
        }
        TransformPassKind::ContainerStaticEval => {
            evaluate_static_container_rules_with_ir_transaction(fixture.source, fixture.dialect)
                .map_err(|error| format!("{error:?}"))?
        }
        _ => (fixture.source.to_string(), 0),
    };
    Ok(path_snapshot_from_output(
        fixture,
        output_css,
        mutation_count,
    ))
}

fn path_snapshot_from_output(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
    output_css: String,
    mutation_count: usize,
) -> StructuralShadowPathSnapshotV0 {
    StructuralShadowPathSnapshotV0 {
        selector_values: selector_values_for_source(&output_css, fixture.dialect),
        declaration_values: declaration_values_for_source(&output_css, fixture.dialect),
        cascade_values: cascade_values_for_fixture(fixture),
        mutation_span_values: mutation_span_values(derive_transform_mutation_spans(
            fixture.source,
            output_css.as_str(),
        )),
        output_css,
        mutation_count,
    }
}

fn structural_shadow_fixtures() -> Vec<TransformStructuralIrShadowFixtureInputV0<'static>> {
    vec![
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "nesting-descendant-and-pseudo",
            pass: TransformPassKind::NestingUnwrap,
            dialect: StyleDialect::Css,
            source: ".card { color: red; & .title { color: blue; } &:hover { color: green; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "nesting-conditional-group",
            pass: TransformPassKind::NestingUnwrap,
            dialect: StyleDialect::Css,
            source: "@media (min-width: 40rem) { .card { color: red; & .title { color: blue; } } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "scope-root-flatten",
            pass: TransformPassKind::ScopeFlatten,
            dialect: StyleDialect::Css,
            source: "@scope (:root) { .card { color: red; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "scope-limit-blocked",
            pass: TransformPassKind::ScopeFlatten,
            dialect: StyleDialect::Css,
            source: "@scope (.theme) to (.stop) { .card { color: red; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "layer-closed-bundle-flatten",
            pass: TransformPassKind::LayerFlatten,
            dialect: StyleDialect::Css,
            source: "@layer theme { .card { color: red; } }",
            closed_bundle: true,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "layer-open-bundle-blocked",
            pass: TransformPassKind::LayerFlatten,
            dialect: StyleDialect::Css,
            source: "@layer theme { .card { color: red; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "rule-dedup-overridden-declarations",
            pass: TransformPassKind::RuleDeduplication,
            dialect: StyleDialect::Css,
            source: ".a { color: red; color: blue; --tone: red; --tone: blue; color: green !important; color: black !important; } :export { token: red; token: blue; }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "rule-dedup-duplicate-rules",
            pass: TransformPassKind::RuleDeduplication,
            dialect: StyleDialect::Css,
            source: ".a { color: red; } .b { color: red; } .a { color: blue; } .a { color: red; }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "rule-merge-adjacent-ordinary",
            pass: TransformPassKind::RuleMerging,
            dialect: StyleDialect::Css,
            source: ".a { color: red; } .a { background: blue; } .a { outline: 0; } .b { color: red; }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "rule-merge-adjacent-conditional-wrappers",
            pass: TransformPassKind::RuleMerging,
            dialect: StyleDialect::Css,
            source: "@media (prefers-color-scheme: dark) { .card { color: white; } } @media (prefers-color-scheme: dark) { .card .title { color: #ddd; } } @supports (display: grid) { .grid { display: grid; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "selector-merge-adjacent-same-block",
            pass: TransformPassKind::SelectorMerging,
            dialect: StyleDialect::Css,
            source: ".a { color: red; } .b { color: red; } .c { color: red; } .d { color: blue; }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "selector-merge-nested-same-block",
            pass: TransformPassKind::SelectorMerging,
            dialect: StyleDialect::Css,
            source: "@media (min-width: 1px) { .m { color: black; } .n { color: black; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "empty-rule-ordinary-and-group",
            pass: TransformPassKind::EmptyRuleRemoval,
            dialect: StyleDialect::Css,
            source: ".a {} @media (min-width: 1px) { .b {} } @keyframes spin { from {} to { opacity: 1; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "empty-rule-preserves-comment-block",
            pass: TransformPassKind::EmptyRuleRemoval,
            dialect: StyleDialect::Css,
            source: ".a { /* keep */ } .b { color: red; }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "supports-static-true-unwrap",
            pass: TransformPassKind::SupportsStaticEval,
            dialect: StyleDialect::Css,
            source: "@supports (display: grid) { .a { display: grid; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "supports-static-false-remove",
            pass: TransformPassKind::DeadSupportsBranchRemoval,
            dialect: StyleDialect::Css,
            source: "@supports not (display: grid) { .a { display: grid; } } .b { color: red; }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "media-static-true-unwrap",
            pass: TransformPassKind::MediaStaticEval,
            dialect: StyleDialect::Css,
            source: "@media all { .a { color: red; } } @media (min-width: 40PX) { .b { color: blue; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "media-static-false-remove",
            pass: TransformPassKind::DeadMediaBranchRemoval,
            dialect: StyleDialect::Css,
            source: "@media not all { .a { color: red; } } .b { color: blue; }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "container-static-false-remove",
            pass: TransformPassKind::ContainerStaticEval,
            dialect: StyleDialect::Css,
            source: "@container (max-width: -1px) { .a { color: red; } } .b { color: blue; }",
            closed_bundle: false,
        },
    ]
}

fn compared_pass_ids() -> Vec<&'static str> {
    vec![
        "container-static-eval",
        "dead-media-branch-removal",
        "dead-supports-branch-removal",
        "empty-rule-removal",
        "layer-flatten",
        "media-static-eval",
        "nesting-unwrap",
        "rule-deduplication",
        "rule-merging",
        "scope-flatten",
        "selector-merging",
        "supports-static-eval",
    ]
}

fn selector_values_for_source(source: &str, dialect: StyleDialect) -> Vec<String> {
    let summary = summarize_omena_parser_style_facts(source, dialect);
    sorted_unique(
        summary
            .class_selector_names
            .into_iter()
            .map(|name| format!("class:{name}"))
            .chain(
                summary
                    .id_selector_names
                    .into_iter()
                    .map(|name| format!("id:{name}")),
            )
            .chain(
                summary
                    .placeholder_selector_names
                    .into_iter()
                    .map(|name| format!("placeholder:{name}")),
            )
            .collect::<Vec<_>>(),
    )
}

fn declaration_values_for_source(source: &str, dialect: StyleDialect) -> Vec<String> {
    let summary = summarize_omena_parser_parity_lite(source, dialect);
    sorted_unique(vec![
        format!("ruleCount:{}", summary.rule_count),
        format!("declarationCount:{}", summary.declaration_count),
        format!(
            "declarationKindCounts:{}",
            serde_json::to_string(&summary.declaration_kind_counts).unwrap_or_default()
        ),
        format!(
            "atRuleKindCounts:{}",
            serde_json::to_string(&summary.at_rule_kind_counts).unwrap_or_default()
        ),
    ])
}

fn cascade_values_for_fixture(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
) -> Vec<String> {
    match fixture.pass {
        TransformPassKind::ScopeFlatten => sorted_unique(
            collect_scope_flatten_proof_candidates_with_lexer(fixture.source, fixture.dialect)
                .into_iter()
                .map(|candidate| {
                    format!(
                        "scope:{}..{}:accepted={}:blocked={:?}:root={}:witness={}",
                        candidate.source_span_start,
                        candidate.source_span_end,
                        candidate.proof.accepted,
                        candidate.proof.blocked_reason,
                        candidate.proof.root_selector,
                        candidate.proof.cascade_safe_witness
                    )
                })
                .collect::<Vec<_>>(),
        ),
        TransformPassKind::LayerFlatten => sorted_unique(
            collect_layer_flatten_proof_candidates_with_lexer(
                fixture.source,
                fixture.dialect,
                fixture.closed_bundle,
            )
            .into_iter()
            .map(|candidate| {
                format!(
                    "layer:{}..{}:accepted={}:blocked={:?}:name={:?}:witness={}",
                    candidate.source_span_start,
                    candidate.source_span_end,
                    candidate.proof.accepted,
                    candidate.proof.blocked_reason,
                    candidate.proof.layer_name,
                    candidate.proof.cascade_safe_witness
                )
            })
            .collect::<Vec<_>>(),
        ),
        _ => Vec::new(),
    }
}

fn mutation_span_values(spans: Vec<TransformProvenanceMutationSpanV0>) -> Vec<String> {
    spans
        .into_iter()
        .map(|span| {
            format!(
                "{}..{}=>{}..{}",
                span.source_span_start,
                span.source_span_end,
                span.generated_span_start,
                span.generated_span_end
            )
        })
        .collect()
}

fn shadow_field_report(
    field: &'static str,
    string_path_values: impl IntoIterator<Item = String>,
    ir_path_values: impl IntoIterator<Item = String>,
) -> TransformStructuralIrShadowFieldReportV0 {
    let string_path_values = sorted_unique(string_path_values);
    let ir_path_values = sorted_unique(ir_path_values);
    let matches = string_path_values == ir_path_values;
    TransformStructuralIrShadowFieldReportV0 {
        field,
        string_path_values,
        ir_path_values,
        matches,
    }
}

fn sorted_unique(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}

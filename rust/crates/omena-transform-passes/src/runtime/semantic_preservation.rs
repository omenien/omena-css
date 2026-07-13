#[cfg(test)]
use omena_cascade::{run_cascade_conformance_seed_corpus, run_wpt_cascade_seed_corpus};
use omena_parser::{ClosedWorldBundleV0, StyleDialect};
use omena_transform_cst::lower_transform_ir_from_source;
use omena_transform_cst::{IrNodeKindV0, IrNodeV0, TransformIrV0, TransformPassKind};
#[cfg(test)]
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::model::TransformSemanticPreservationTelemetryV0;
use crate::{
    domains::{
        css_modules_values::{
            collect_css_modules_value_semantic_facts_from_ir,
            collect_tree_shake_css_modules_value_removals_from_ir,
        },
        custom_property::{
            collect_css_custom_property_semantic_facts_from_ir,
            collect_tree_shake_css_custom_property_removals_from_ir,
        },
        keyframes::{
            collect_referenced_keyframe_names_from_ir,
            collect_tree_shake_css_keyframe_removals_from_ir, keyframe_name_is_reachable,
        },
        nesting::expand_nested_selector,
        reachability::class_name_is_reachable,
    },
    helpers::selectors::selector_branch_owner_class_names,
};

impl TransformSemanticPreservationTelemetryV0 {
    pub(crate) fn record(&mut self, decision: &TransformSemanticPreservationDecisionV0) {
        self.observed_pass_count += 1;
        if decision.preserved {
            self.preserved_pass_count += 1;
        } else {
            self.blocked_pass_count += 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformSemanticPreservationDecisionV0 {
    pub pass_id: &'static str,
    pub preserved: bool,
    pub input_entry_count: usize,
    pub output_entry_count: usize,
    pub mismatch_count: usize,
}

pub fn compare_transform_css_semantics_v0(
    input_css: &str,
    output_css: &str,
    dialect: StyleDialect,
) -> TransformSemanticPreservationDecisionV0 {
    let input_ir = lower_transform_ir_from_source(
        input_css,
        dialect,
        "omena-transform-passes.semantic-comparison.input",
    );
    let output_ir = lower_transform_ir_from_source(
        output_css,
        dialect,
        "omena-transform-passes.semantic-comparison.output",
    );
    let scope = SemanticObservationScopeV0::from_parts(None, None, &[], dialect);
    compare_semantic_observation_for_pass_with_scopes(
        "external-css-lowering",
        &input_ir,
        &output_ir,
        scope,
        scope,
    )
}

pub(crate) fn semantic_preservation_applies(pass: TransformPassKind) -> bool {
    matches!(
        pass,
        TransformPassKind::EmptyRuleRemoval
            | TransformPassKind::RuleDeduplication
            | TransformPassKind::RuleMerging
            | TransformPassKind::SelectorMerging
            | TransformPassKind::NestingUnwrap
            | TransformPassKind::ScopeFlatten
            | TransformPassKind::LayerFlatten
            | TransformPassKind::TreeShakeClass
            | TransformPassKind::TreeShakeKeyframes
            | TransformPassKind::TreeShakeValue
            | TransformPassKind::TreeShakeCustomProperty
    )
}

#[cfg(test)]
pub(crate) fn compare_semantic_observation_for_pass(
    pass_id: &'static str,
    input_ir: &TransformIrV0,
    output_ir: &TransformIrV0,
) -> TransformSemanticPreservationDecisionV0 {
    compare_semantic_observation_for_pass_with_scope(
        pass_id,
        input_ir,
        output_ir,
        SemanticObservationScopeV0::default(),
    )
}

#[cfg(test)]
pub(crate) fn compare_semantic_observation_for_pass_with_scope<'a>(
    pass_id: &'static str,
    input_ir: &TransformIrV0,
    output_ir: &TransformIrV0,
    scope: SemanticObservationScopeV0<'a>,
) -> TransformSemanticPreservationDecisionV0 {
    compare_semantic_observation_for_pass_with_scopes(pass_id, input_ir, output_ir, scope, scope)
}

pub(crate) fn compare_semantic_observation_for_pass_with_scopes<'a>(
    pass_id: &'static str,
    input_ir: &TransformIrV0,
    output_ir: &TransformIrV0,
    input_scope: SemanticObservationScopeV0<'a>,
    output_scope: SemanticObservationScopeV0<'a>,
) -> TransformSemanticPreservationDecisionV0 {
    let input = semantic_observation(input_ir, input_scope);
    let output = semantic_observation(output_ir, output_scope);
    let mismatch_count = semantic_observation_mismatch_count(&input, &output);
    TransformSemanticPreservationDecisionV0 {
        pass_id,
        preserved: mismatch_count == 0,
        input_entry_count: input.len(),
        output_entry_count: output.len(),
        mismatch_count,
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SemanticObservationScopeV0<'a> {
    reachable_class_names: Option<&'a [String]>,
    reachable_keyframe_names: Option<&'a [String]>,
    ignored_source_ranges: &'a [(usize, usize)],
    dialect: StyleDialect,
    force_ir_declarations: bool,
}

impl Default for SemanticObservationScopeV0<'_> {
    fn default() -> Self {
        Self {
            reachable_class_names: None,
            reachable_keyframe_names: None,
            ignored_source_ranges: &[],
            dialect: StyleDialect::Css,
            force_ir_declarations: false,
        }
    }
}

impl<'a> SemanticObservationScopeV0<'a> {
    fn from_parts(
        reachable_class_names: Option<&'a [String]>,
        reachable_keyframe_names: Option<&'a [String]>,
        ignored_source_ranges: &'a [(usize, usize)],
        dialect: StyleDialect,
    ) -> Self {
        Self {
            reachable_class_names,
            reachable_keyframe_names,
            ignored_source_ranges,
            dialect,
            force_ir_declarations: !ignored_source_ranges.is_empty(),
        }
    }

    pub(crate) fn for_pass(
        pass: TransformPassKind,
        dialect: StyleDialect,
        closed_world_bundle: Option<&'a ClosedWorldBundleV0>,
        projection: &'a SemanticObservationProjectionV0,
    ) -> Self {
        match pass {
            TransformPassKind::TreeShakeClass
            | TransformPassKind::TreeShakeKeyframes
            | TransformPassKind::TreeShakeValue
            | TransformPassKind::TreeShakeCustomProperty => Self::from_parts(
                closed_world_bundle.map(|bundle| bundle.reachability().class_names()),
                projection.reachable_keyframe_names(),
                projection.ignored_source_ranges(),
                dialect,
            ),
            _ => Self::from_parts(
                None,
                projection.reachable_keyframe_names(),
                projection.ignored_source_ranges(),
                dialect,
            ),
        }
    }

    #[cfg(test)]
    fn for_reachable_class_names(reachable_class_names: &'a [String]) -> Self {
        Self::from_parts(Some(reachable_class_names), None, &[], StyleDialect::Css)
    }

    #[cfg(test)]
    fn for_ignored_source_ranges(ignored_source_ranges: &'a [(usize, usize)]) -> Self {
        Self::from_parts(None, None, ignored_source_ranges, StyleDialect::Css)
    }

    #[cfg(test)]
    fn for_reachable_class_names_and_ignored_source_ranges(
        reachable_class_names: &'a [String],
        ignored_source_ranges: &'a [(usize, usize)],
    ) -> Self {
        Self::from_parts(
            Some(reachable_class_names),
            None,
            ignored_source_ranges,
            StyleDialect::Css,
        )
    }

    pub(crate) fn without_ignored_source_ranges(self) -> Self {
        Self {
            reachable_class_names: self.reachable_class_names,
            reachable_keyframe_names: self.reachable_keyframe_names,
            ignored_source_ranges: &[],
            dialect: self.dialect,
            force_ir_declarations: self.force_ir_declarations,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SemanticObservationProjectionV0 {
    ignored_source_ranges: Vec<(usize, usize)>,
    reachable_keyframe_names: Option<Vec<String>>,
}

impl SemanticObservationProjectionV0 {
    pub(crate) fn for_pass_input(
        pass: TransformPassKind,
        input_ir: &TransformIrV0,
        dialect: StyleDialect,
        closed_world_bundle: Option<&ClosedWorldBundleV0>,
    ) -> Self {
        let Some(bundle) = closed_world_bundle else {
            return Self::default();
        };
        match pass {
            TransformPassKind::TreeShakeKeyframes => Self {
                ignored_source_ranges: collect_tree_shake_css_keyframe_removals_from_ir(
                    input_ir,
                    bundle.reachability().keyframe_names(),
                    bundle.reachability().class_names(),
                )
                .into_iter()
                .map(|removal| (removal.source_span_start, removal.source_span_end))
                .collect(),
                reachable_keyframe_names: None,
            },
            TransformPassKind::TreeShakeValue => Self {
                ignored_source_ranges: collect_tree_shake_css_modules_value_removals_from_ir(
                    input_ir,
                    dialect,
                    bundle.reachability().value_names(),
                    bundle.reachability().keyframe_names(),
                    bundle.reachability().class_names(),
                )
                .into_iter()
                .map(|removal| (removal.source_span_start, removal.source_span_end))
                .collect(),
                reachable_keyframe_names: None,
            },
            TransformPassKind::TreeShakeCustomProperty => Self {
                ignored_source_ranges: collect_tree_shake_css_custom_property_removals_from_ir(
                    input_ir,
                    dialect,
                    bundle.reachability().custom_property_names(),
                    bundle.reachability().keyframe_names(),
                    bundle.reachability().class_names(),
                )
                .into_iter()
                .map(|removal| (removal.source_span_start, removal.source_span_end))
                .collect(),
                reachable_keyframe_names: reachable_keyframe_names_for_closed_class_scope(
                    input_ir,
                    bundle.reachability().keyframe_names(),
                    bundle.reachability().class_names(),
                ),
            },
            _ => {
                let _ = dialect;
                Self::default()
            }
        }
    }

    pub(crate) fn ignored_source_ranges(&self) -> &[(usize, usize)] {
        self.ignored_source_ranges.as_slice()
    }

    fn reachable_keyframe_names(&self) -> Option<&[String]> {
        self.reachable_keyframe_names.as_deref()
    }

    #[cfg(test)]
    fn for_keyframe_reachability(
        input_ir: &TransformIrV0,
        reachable_keyframe_names: &[String],
        reachable_class_names: &[String],
    ) -> Self {
        Self {
            ignored_source_ranges: collect_tree_shake_css_keyframe_removals_from_ir(
                input_ir,
                reachable_keyframe_names,
                reachable_class_names,
            )
            .into_iter()
            .map(|removal| (removal.source_span_start, removal.source_span_end))
            .collect(),
            reachable_keyframe_names: None,
        }
    }

    #[cfg(test)]
    fn for_value_reachability(
        input_ir: &TransformIrV0,
        dialect: StyleDialect,
        reachable_value_names: &[String],
        reachable_keyframe_names: &[String],
        reachable_class_names: &[String],
    ) -> Self {
        Self {
            ignored_source_ranges: collect_tree_shake_css_modules_value_removals_from_ir(
                input_ir,
                dialect,
                reachable_value_names,
                reachable_keyframe_names,
                reachable_class_names,
            )
            .into_iter()
            .map(|removal| (removal.source_span_start, removal.source_span_end))
            .collect(),
            reachable_keyframe_names: None,
        }
    }

    #[cfg(test)]
    fn for_custom_property_reachability(
        input_ir: &TransformIrV0,
        dialect: StyleDialect,
        reachable_custom_property_names: &[String],
        reachable_keyframe_names: &[String],
        reachable_class_names: &[String],
    ) -> Self {
        Self {
            ignored_source_ranges: collect_tree_shake_css_custom_property_removals_from_ir(
                input_ir,
                dialect,
                reachable_custom_property_names,
                reachable_keyframe_names,
                reachable_class_names,
            )
            .into_iter()
            .map(|removal| (removal.source_span_start, removal.source_span_end))
            .collect(),
            reachable_keyframe_names: reachable_keyframe_names_for_closed_class_scope(
                input_ir,
                reachable_keyframe_names,
                reachable_class_names,
            ),
        }
    }
}

fn reachable_keyframe_names_for_closed_class_scope(
    input_ir: &TransformIrV0,
    explicit_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> Option<Vec<String>> {
    let mut names = collect_referenced_keyframe_names_from_ir(input_ir, reachable_class_names)?;
    for name in explicit_keyframe_names {
        if !names.iter().any(|candidate| candidate == name) {
            names.push(name.clone());
        }
    }
    Some(names)
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransformSemanticPreservationKillRateReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub fixture_count: usize,
    pub rejected_count: usize,
    pub required_rejected_count: usize,
    pub non_empty_corpus: bool,
    pub kill_rate_passed: bool,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransformSemanticModelConformanceReportV0 {
    pub schema_version: String,
    pub product: String,
    pub cascade_seed_product: String,
    pub cascade_seed_case_count: usize,
    pub cascade_seed_failed_count: usize,
    pub cascade_seed_digest: String,
    pub wpt_seed_product: String,
    pub wpt_seed_case_count: usize,
    pub wpt_seed_failed_count: usize,
    pub wpt_seed_digest: String,
    pub semantic_observation_case_count: usize,
    pub semantic_observation_failed_count: usize,
    pub model_conformance_passed: bool,
}

#[cfg(test)]
pub(crate) fn summarize_semantic_preservation_model_conformance()
-> Result<TransformSemanticModelConformanceReportV0, serde_json::Error> {
    let cascade_seed = run_cascade_conformance_seed_corpus();
    let wpt_seed = run_wpt_cascade_seed_corpus();
    let cascade_seed_source = serde_json::to_string(&cascade_seed)?;
    let wpt_seed_source = serde_json::to_string(&wpt_seed)?;
    let semantic_observation_results = semantic_model_conformance_case_results();
    let semantic_observation_failed_count = semantic_observation_results
        .iter()
        .filter(|result| !**result)
        .count();

    Ok(TransformSemanticModelConformanceReportV0 {
        schema_version: "0".to_string(),
        product: "omena-transform-passes.semantic-preservation-model-conformance".to_string(),
        cascade_seed_product: cascade_seed.product.to_string(),
        cascade_seed_case_count: cascade_seed.case_count,
        cascade_seed_failed_count: cascade_seed.failed_count,
        cascade_seed_digest: stable_semantic_report_digest(&[
            "cascade-seed",
            cascade_seed_source.as_str(),
        ]),
        wpt_seed_product: wpt_seed.product.to_string(),
        wpt_seed_case_count: wpt_seed.case_count,
        wpt_seed_failed_count: wpt_seed.failed_count,
        wpt_seed_digest: stable_semantic_report_digest(&["wpt-seed", wpt_seed_source.as_str()]),
        semantic_observation_case_count: semantic_observation_results.len(),
        semantic_observation_failed_count,
        model_conformance_passed: cascade_seed.failed_count == 0
            && wpt_seed.failed_count == 0
            && semantic_observation_failed_count == 0,
    })
}

#[cfg(test)]
pub(crate) fn summarize_semantic_preservation_kill_rate_for_fixture_source(
    source: &str,
    dialect: StyleDialect,
) -> Result<TransformSemanticPreservationKillRateReportV0, serde_json::Error> {
    let fixtures = serde_json::from_str::<Vec<TransformSemanticPreservationFixtureV0>>(source)?;
    let mut rejected_count = 0usize;

    for fixture in &fixtures {
        let Some(pass) = transform_pass_kind_from_fixture_id(fixture.pass_id.as_str()) else {
            continue;
        };
        if !semantic_preservation_applies(pass) {
            continue;
        }
        let input_ir = lower_transform_ir_from_source(
            fixture.input.as_str(),
            dialect,
            "omena-transform-passes.semantic-preservation.input",
        );
        let output_ir = lower_transform_ir_from_source(
            fixture.output.as_str(),
            dialect,
            "omena-transform-passes.semantic-preservation.output",
        );
        let projection = if !fixture.reachable_custom_property_names.is_empty()
            || pass == TransformPassKind::TreeShakeCustomProperty
        {
            SemanticObservationProjectionV0::for_custom_property_reachability(
                &input_ir,
                dialect,
                &fixture.reachable_custom_property_names,
                &fixture.reachable_keyframe_names,
                &fixture.reachable_class_names,
            )
        } else if !fixture.reachable_value_names.is_empty()
            || pass == TransformPassKind::TreeShakeValue
        {
            SemanticObservationProjectionV0::for_value_reachability(
                &input_ir,
                dialect,
                &fixture.reachable_value_names,
                &fixture.reachable_keyframe_names,
                &fixture.reachable_class_names,
            )
        } else if !fixture.reachable_keyframe_names.is_empty()
            || pass == TransformPassKind::TreeShakeKeyframes
        {
            SemanticObservationProjectionV0::for_keyframe_reachability(
                &input_ir,
                &fixture.reachable_keyframe_names,
                &fixture.reachable_class_names,
            )
        } else {
            SemanticObservationProjectionV0::default()
        };
        let scope = if !fixture.reachable_class_names.is_empty()
            && !projection.ignored_source_ranges().is_empty()
        {
            SemanticObservationScopeV0::for_reachable_class_names_and_ignored_source_ranges(
                &fixture.reachable_class_names,
                projection.ignored_source_ranges(),
            )
        } else if !fixture.reachable_class_names.is_empty() {
            SemanticObservationScopeV0::for_reachable_class_names(&fixture.reachable_class_names)
        } else if !projection.ignored_source_ranges().is_empty() {
            SemanticObservationScopeV0::for_ignored_source_ranges(
                projection.ignored_source_ranges(),
            )
        } else {
            SemanticObservationScopeV0::default()
        };
        let decision = compare_semantic_observation_for_pass_with_scopes(
            pass.id(),
            &input_ir,
            &output_ir,
            scope,
            scope.without_ignored_source_ranges(),
        );
        if !decision.preserved {
            rejected_count += 1;
        }
    }

    let required_rejected_count = fixtures
        .iter()
        .filter(|fixture| fixture.expected_rejected)
        .count();
    Ok(TransformSemanticPreservationKillRateReportV0 {
        schema_version: "0",
        product: "omena-transform-passes.semantic-preservation-kill-rate",
        fixture_count: fixtures.len(),
        rejected_count,
        required_rejected_count,
        non_empty_corpus: !fixtures.is_empty(),
        kill_rate_passed: !fixtures.is_empty() && rejected_count >= required_rejected_count,
    })
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransformSemanticPreservationFixtureV0 {
    pass_id: String,
    input: String,
    output: String,
    expected_rejected: bool,
    #[serde(default)]
    reachable_class_names: Vec<String>,
    #[serde(default)]
    reachable_keyframe_names: Vec<String>,
    #[serde(default)]
    reachable_value_names: Vec<String>,
    #[serde(default)]
    reachable_custom_property_names: Vec<String>,
}

#[cfg(test)]
fn transform_pass_kind_from_fixture_id(pass_id: &str) -> Option<TransformPassKind> {
    match pass_id {
        "empty-rule-removal" => Some(TransformPassKind::EmptyRuleRemoval),
        "rule-deduplication" => Some(TransformPassKind::RuleDeduplication),
        "rule-merging" => Some(TransformPassKind::RuleMerging),
        "selector-merging" => Some(TransformPassKind::SelectorMerging),
        "nesting-unwrap" => Some(TransformPassKind::NestingUnwrap),
        "scope-flatten" => Some(TransformPassKind::ScopeFlatten),
        "layer-flatten" => Some(TransformPassKind::LayerFlatten),
        "tree-shake-class" => Some(TransformPassKind::TreeShakeClass),
        "tree-shake-keyframes" => Some(TransformPassKind::TreeShakeKeyframes),
        "tree-shake-value" => Some(TransformPassKind::TreeShakeValue),
        "tree-shake-custom-property" => Some(TransformPassKind::TreeShakeCustomProperty),
        _ => None,
    }
}

type SemanticObservationV0 = BTreeMap<SemanticObservationKeyV0, SemanticObservationValueV0>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SemanticObservationKeyV0 {
    selector_key: String,
    property: String,
    context_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticObservationValueV0 {
    value: String,
    important: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticDeclarationCandidateV0 {
    key: SemanticObservationKeyV0,
    value: SemanticObservationValueV0,
    source_order: usize,
}

fn semantic_observation(
    ir: &TransformIrV0,
    scope: SemanticObservationScopeV0<'_>,
) -> SemanticObservationV0 {
    let mut observation = SemanticObservationV0::new();
    let mut candidates = ir
        .nodes
        .iter()
        .filter(|node| !node.deleted)
        .filter(|node| {
            !source_range_is_fully_ignored(node.source_span_start, node.source_span_end, scope)
        })
        .filter_map(|node| match node.kind {
            IrNodeKindV0::StyleRule => semantic_style_rule_candidates(ir, node, scope),
            IrNodeKindV0::AtRule => semantic_at_rule_style_rule_candidates(ir, node, scope),
            _ => None,
        })
        .flatten()
        .collect::<Vec<_>>();
    candidates.extend(semantic_css_modules_value_candidates(ir, scope));
    candidates.extend(semantic_custom_property_candidates(ir, scope));
    candidates.sort_by_key(|candidate| candidate.source_order);

    for candidate in candidates {
        match observation.get(&candidate.key) {
            Some(current) if current.important && !candidate.value.important => {
                continue;
            }
            _ => {
                observation.insert(candidate.key, candidate.value);
            }
        }
    }

    observation
}

fn semantic_custom_property_candidates(
    ir: &TransformIrV0,
    scope: SemanticObservationScopeV0<'_>,
) -> Vec<SemanticDeclarationCandidateV0> {
    collect_css_custom_property_semantic_facts_from_ir(ir)
        .into_iter()
        .filter(|fact| {
            !source_range_is_ignored(fact.source_span_start, fact.source_span_end, scope)
        })
        .map(|fact| SemanticDeclarationCandidateV0 {
            source_order: fact.source_span_start,
            key: SemanticObservationKeyV0 {
                selector_key: fact.fact_kind.to_string(),
                property: fact.name,
                context_key: "css-custom-properties".to_string(),
            },
            value: SemanticObservationValueV0 {
                value: fact.value,
                important: false,
            },
        })
        .collect()
}

fn semantic_css_modules_value_candidates(
    ir: &TransformIrV0,
    scope: SemanticObservationScopeV0<'_>,
) -> Vec<SemanticDeclarationCandidateV0> {
    collect_css_modules_value_semantic_facts_from_ir(ir, scope.dialect)
        .into_iter()
        .filter(|fact| {
            !source_range_is_ignored(fact.source_span_start, fact.source_span_end, scope)
        })
        .map(|fact| SemanticDeclarationCandidateV0 {
            source_order: fact.source_span_start,
            key: SemanticObservationKeyV0 {
                selector_key: fact.fact_kind.to_string(),
                property: fact.name,
                context_key: "css-modules".to_string(),
            },
            value: SemanticObservationValueV0 {
                value: fact.value,
                important: false,
            },
        })
        .collect()
}

fn semantic_style_rule_candidates(
    ir: &TransformIrV0,
    node: &IrNodeV0,
    scope: SemanticObservationScopeV0<'_>,
) -> Option<Vec<SemanticDeclarationCandidateV0>> {
    if has_deleted_ancestor(ir, node) {
        return None;
    }
    let selector_keys =
        observation_selector_keys(expanded_style_rule_selector_keys(ir, node)?, scope)
            .into_iter()
            .filter(|selector_key| {
                !selector_key.eq_ignore_ascii_case(":export")
                    && !selector_key.starts_with(":import")
            })
            .collect::<Vec<_>>();
    if selector_keys.is_empty() {
        return None;
    }
    let context_key = ancestor_at_rule_context_key(ir, node);
    let mut declarations = semantic_declarations_from_style_rule_text(ir, node, scope)
        .unwrap_or_else(|| semantic_declarations_from_direct_ir_children(ir, node, scope));
    declarations.sort_by_key(|declaration| declaration.source_order);

    Some(candidates_from_selector_declarations(
        selector_keys.as_slice(),
        context_key.as_str(),
        declarations,
    ))
}

fn semantic_at_rule_style_rule_candidates(
    ir: &TransformIrV0,
    node: &IrNodeV0,
    scope: SemanticObservationScopeV0<'_>,
) -> Option<Vec<SemanticDeclarationCandidateV0>> {
    if has_deleted_ancestor(ir, node) {
        return None;
    }
    let source = node_text(ir, node)?;
    let open = source.find('{')?;
    let close = source.rfind('}')?;
    if close <= open {
        return None;
    }
    let prelude = source.get(..open)?.trim();
    if !at_rule_prelude_is_reachable_in_scope(prelude, scope) {
        return None;
    }
    if has_style_rule_ancestor(ir, node) {
        return nested_at_rule_declaration_candidates(ir, node, prelude, scope);
    }
    let body = source.get(open + 1..close)?;
    let context_key = join_context_components(
        ancestor_at_rule_context_key(ir, node),
        at_rule_context_component_from_prelude(prelude),
    );
    let mut candidates = Vec::new();

    for (index, rule_source) in top_level_style_rule_sources(body).into_iter().enumerate() {
        let Some(rule_open) = rule_source.find('{') else {
            continue;
        };
        let Some(selector) = rule_source.get(..rule_open) else {
            continue;
        };
        let selector_keys =
            observation_selector_keys(selector_keys_from_selector_text(selector), scope)
                .into_iter()
                .filter(|selector_key| {
                    !selector_key.eq_ignore_ascii_case(":export")
                        && !selector_key.starts_with(":import")
                })
                .collect::<Vec<_>>();
        if selector_keys.is_empty() {
            continue;
        }
        let declarations = semantic_declarations_from_rule_source(
            rule_source.as_str(),
            node.global_order
                .saturating_mul(4096)
                .saturating_add(index.saturating_mul(1024)),
        )
        .unwrap_or_default();
        candidates.extend(candidates_from_selector_declarations(
            selector_keys.as_slice(),
            context_key.as_str(),
            declarations,
        ));
    }

    if candidates.is_empty() {
        None
    } else {
        Some(candidates)
    }
}

fn nested_at_rule_declaration_candidates(
    ir: &TransformIrV0,
    node: &IrNodeV0,
    prelude: &str,
    scope: SemanticObservationScopeV0<'_>,
) -> Option<Vec<SemanticDeclarationCandidateV0>> {
    let (selector_keys, context_key) =
        if let Some(nest_selector) = nest_at_rule_selector_from_prelude(prelude) {
            let parent_selector = nearest_style_ancestor_expanded_selector(ir, node)?;
            let selector = expand_nested_selector(parent_selector.as_str(), nest_selector)?;
            (
                selector_keys_from_selector_text(selector.as_str()),
                ancestor_at_rule_context_key(ir, node),
            )
        } else {
            (
                nearest_style_ancestor_selector_keys(ir, node)?,
                join_context_components(
                    ancestor_at_rule_context_key(ir, node),
                    at_rule_context_component_from_prelude(prelude),
                ),
            )
        };
    let selector_keys = observation_selector_keys(selector_keys, scope)
        .into_iter()
        .filter(|selector_key| {
            !selector_key.eq_ignore_ascii_case(":export") && !selector_key.starts_with(":import")
        })
        .collect::<Vec<_>>();
    if selector_keys.is_empty() {
        return None;
    }
    let mut declarations = semantic_declarations_from_direct_ir_children(ir, node, scope);
    if declarations.is_empty() {
        return None;
    }
    declarations.sort_by_key(|declaration| declaration.source_order);
    Some(candidates_from_selector_declarations(
        selector_keys.as_slice(),
        context_key.as_str(),
        declarations,
    ))
}

fn at_rule_prelude_is_reachable_in_scope(
    prelude: &str,
    scope: SemanticObservationScopeV0<'_>,
) -> bool {
    let Some(reachable_keyframe_names) = scope.reachable_keyframe_names else {
        return true;
    };
    let Some(keyframe_name) = keyframe_name_from_at_rule_prelude(prelude) else {
        return true;
    };
    keyframe_name_is_reachable(keyframe_name, reachable_keyframe_names)
}

fn keyframe_name_from_at_rule_prelude(prelude: &str) -> Option<&str> {
    let trimmed = prelude.trim();
    let after_keyword = trimmed
        .strip_prefix("@keyframes")
        .or_else(|| trimmed.strip_prefix("@-webkit-keyframes"))?;
    after_keyword.split_whitespace().next()
}

fn observation_selector_keys(
    selector_keys: Vec<String>,
    scope: SemanticObservationScopeV0<'_>,
) -> Vec<String> {
    match scope.reachable_class_names {
        Some(reachable_class_names) => selector_keys
            .into_iter()
            .filter(|selector_key| {
                selector_is_reachable_in_closed_class_scope(selector_key, reachable_class_names)
            })
            .collect(),
        None => selector_keys,
    }
}

fn selector_is_reachable_in_closed_class_scope(
    selector_key: &str,
    reachable_class_names: &[String],
) -> bool {
    let Some(owner_class_names) = selector_branch_owner_class_names(selector_key) else {
        return true;
    };
    owner_class_names
        .iter()
        .any(|owner| class_name_is_reachable(owner, reachable_class_names))
}

fn source_range_is_ignored(
    source_span_start: usize,
    source_span_end: usize,
    scope: SemanticObservationScopeV0<'_>,
) -> bool {
    scope
        .ignored_source_ranges
        .iter()
        .any(|(start, end)| source_span_start < *end && source_span_end > *start)
}

fn source_range_is_fully_ignored(
    source_span_start: usize,
    source_span_end: usize,
    scope: SemanticObservationScopeV0<'_>,
) -> bool {
    scope
        .ignored_source_ranges
        .iter()
        .any(|(start, end)| source_span_start >= *start && source_span_end <= *end)
}

fn candidates_from_selector_declarations(
    selector_keys: &[String],
    context_key: &str,
    declarations: Vec<SemanticDeclarationV0>,
) -> Vec<SemanticDeclarationCandidateV0> {
    declarations
        .into_iter()
        .flat_map(|declaration| {
            let property = declaration.property;
            let value = declaration.value;
            let context_key = context_key.to_string();
            selector_keys
                .iter()
                .map(move |selector_key| SemanticDeclarationCandidateV0 {
                    key: SemanticObservationKeyV0 {
                        selector_key: selector_key.clone(),
                        property: property.clone(),
                        context_key: context_key.clone(),
                    },
                    value: SemanticObservationValueV0 {
                        value: value.clone(),
                        important: declaration.important,
                    },
                    source_order: declaration.source_order,
                })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticDeclarationV0 {
    property: String,
    value: String,
    important: bool,
    source_order: usize,
}

fn semantic_declaration_from_ir(
    ir: &TransformIrV0,
    node: &IrNodeV0,
) -> Option<SemanticDeclarationV0> {
    if has_deleted_ancestor(ir, node) {
        return None;
    }
    let source = node_text(ir, node)?.trim().trim_end_matches(';').trim();
    semantic_declaration_from_source(source, node.global_order)
}

fn semantic_declarations_from_style_rule_text(
    ir: &TransformIrV0,
    node: &IrNodeV0,
    scope: SemanticObservationScopeV0<'_>,
) -> Option<Vec<SemanticDeclarationV0>> {
    if scope.force_ir_declarations {
        return None;
    }
    let source = node_text(ir, node)?;
    semantic_declarations_from_rule_source(source, node.global_order.saturating_mul(1024))
}

fn semantic_declarations_from_rule_source(
    source: &str,
    base_source_order: usize,
) -> Option<Vec<SemanticDeclarationV0>> {
    let open = source.find('{')?;
    let close = source.rfind('}')?;
    if close <= open {
        return None;
    }
    let body = source.get(open + 1..close)?;
    if contains_nested_block_or_comment(body) {
        return None;
    }
    let declarations = split_declaration_list(body)
        .into_iter()
        .enumerate()
        .filter_map(|(index, declaration)| {
            semantic_declaration_from_source(
                declaration.as_str(),
                base_source_order.saturating_add(index),
            )
        })
        .collect::<Vec<_>>();
    if declarations.is_empty() {
        None
    } else {
        Some(declarations)
    }
}

fn semantic_declaration_from_source(
    source: &str,
    source_order: usize,
) -> Option<SemanticDeclarationV0> {
    if source.is_empty() || contains_nested_block_or_comment(source) {
        return None;
    }
    let colon = declaration_colon_index(source)?;
    let property = source.get(..colon)?.trim();
    let value = source.get(colon + 1..)?.trim();
    if property.is_empty() || value.is_empty() {
        return None;
    }
    let property = if property.starts_with("--") {
        property.to_string()
    } else {
        property.to_ascii_lowercase()
    };
    Some(SemanticDeclarationV0 {
        property,
        value: normalize_declaration_value(value),
        important: declaration_value_is_important(value),
        source_order,
    })
}

fn expanded_style_rule_selector_keys(ir: &TransformIrV0, node: &IrNodeV0) -> Option<Vec<String>> {
    let selector = expanded_style_rule_selector_text(ir, node)?;
    let selector_keys = selector_keys_from_selector_text(selector.as_str());
    if selector_keys.is_empty() {
        None
    } else {
        Some(selector_keys)
    }
}

fn expanded_style_rule_selector_text(ir: &TransformIrV0, node: &IrNodeV0) -> Option<String> {
    let source = node_text(ir, node)?;
    let open = source.find('{')?;
    let mut selector = source.get(..open)?.trim().to_string();
    let mut expanded_parent_selector: Option<String> = None;
    for parent_selector in style_rule_ancestor_selectors(ir, node)?.into_iter().rev() {
        expanded_parent_selector = Some(match expanded_parent_selector {
            Some(expanded) => expand_nested_selector(expanded.as_str(), parent_selector.as_str())?,
            None => parent_selector,
        });
    }
    if let Some(parent_selector) = expanded_parent_selector {
        selector = expand_nested_selector(parent_selector.as_str(), selector.as_str())?;
    }
    Some(selector)
}

fn nearest_style_ancestor_selector_keys(
    ir: &TransformIrV0,
    node: &IrNodeV0,
) -> Option<Vec<String>> {
    let mut parent = node.parent;
    while let Some(parent_id) = parent {
        let parent_node = ir.nodes.get(parent_id.index())?;
        if parent_node.deleted {
            return None;
        }
        if parent_node.kind == IrNodeKindV0::StyleRule {
            return expanded_style_rule_selector_keys(ir, parent_node);
        }
        parent = parent_node.parent;
    }
    None
}

fn nearest_style_ancestor_expanded_selector(ir: &TransformIrV0, node: &IrNodeV0) -> Option<String> {
    let mut parent = node.parent;
    while let Some(parent_id) = parent {
        let parent_node = ir.nodes.get(parent_id.index())?;
        if parent_node.deleted {
            return None;
        }
        if parent_node.kind == IrNodeKindV0::StyleRule {
            return expanded_style_rule_selector_text(ir, parent_node);
        }
        parent = parent_node.parent;
    }
    None
}

fn style_rule_ancestor_selectors(ir: &TransformIrV0, node: &IrNodeV0) -> Option<Vec<String>> {
    let mut selectors = Vec::new();
    let mut parent = node.parent;
    while let Some(parent_id) = parent {
        let parent_node = ir.nodes.get(parent_id.index())?;
        if parent_node.deleted {
            return None;
        }
        match parent_node.kind {
            IrNodeKindV0::StyleRule => {
                let source = node_text(ir, parent_node)?;
                let open = source.find('{')?;
                selectors.push(source.get(..open)?.trim().to_string());
            }
            IrNodeKindV0::AtRule => {
                if let Some(selector) = node_text(ir, parent_node)
                    .and_then(|source| source.get(..source.find('{').unwrap_or(source.len())))
                    .and_then(nest_at_rule_selector_from_prelude)
                {
                    selectors.push(selector.to_string());
                }
            }
            _ => {}
        }
        parent = parent_node.parent;
    }
    Some(selectors)
}

fn nest_at_rule_selector_from_prelude(prelude: &str) -> Option<&str> {
    let prelude = prelude.trim();
    let selector = prelude.strip_prefix("@nest")?.trim();
    (!selector.is_empty()).then_some(selector)
}

fn selector_keys_from_selector_text(selector: &str) -> Vec<String> {
    split_selector_list(selector)
        .into_iter()
        .map(|selector| normalize_selector_key(selector.as_str()))
        .filter(|selector| !selector.is_empty())
        .collect::<Vec<_>>()
}

fn ancestor_at_rule_context_key(ir: &TransformIrV0, node: &IrNodeV0) -> String {
    let mut ancestors = Vec::new();
    let mut parent = node.parent;
    while let Some(parent_id) = parent {
        let Some(parent_node) = ir.nodes.get(parent_id.index()) else {
            break;
        };
        if parent_node.deleted {
            break;
        }
        if parent_node.kind == IrNodeKindV0::AtRule
            && let Some(context) = at_rule_context_component(ir, parent_node)
        {
            ancestors.push(context);
        }
        parent = parent_node.parent;
    }
    ancestors.reverse();
    ancestors.join("|")
}

fn at_rule_context_component(ir: &TransformIrV0, node: &IrNodeV0) -> Option<String> {
    let source = node_text(ir, node)?;
    let open = source.find('{').unwrap_or(source.len());
    at_rule_context_component_from_prelude(source.get(..open)?.trim())
}

fn at_rule_context_component_from_prelude(prelude: &str) -> Option<String> {
    if prelude.is_empty() {
        return None;
    }
    let normalized = normalize_space(prelude);
    if at_rule_prelude_is_semantically_transparent(normalized.as_str()) {
        None
    } else {
        Some(normalized)
    }
}

fn at_rule_prelude_is_semantically_transparent(prelude: &str) -> bool {
    let lower = prelude.to_ascii_lowercase();
    let compact = lower
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .collect::<String>();
    compact == "@scope(:root)"
        || lower.starts_with("@nest ")
        || lower
            .strip_prefix("@layer")
            .is_some_and(|name| !name.trim().is_empty() && !name.contains(','))
}

fn join_context_components(base: String, current: Option<String>) -> String {
    match (base.is_empty(), current) {
        (true, Some(current)) => current,
        (false, Some(current)) => format!("{base}|{current}"),
        _ => base,
    }
}

fn semantic_declarations_from_direct_ir_children(
    ir: &TransformIrV0,
    node: &IrNodeV0,
    scope: SemanticObservationScopeV0<'_>,
) -> Vec<SemanticDeclarationV0> {
    let mut declarations = semantic_declarations_from_direct_source_segments(ir, node, scope);
    declarations.extend(
        node.children
            .iter()
            .filter_map(|child_id| ir.nodes.get(child_id.index()))
            .filter(|child| !child.deleted && child.kind == IrNodeKindV0::Declaration)
            .filter(|child| {
                !source_range_is_ignored(child.source_span_start, child.source_span_end, scope)
            })
            .filter_map(|child| semantic_declaration_from_ir(ir, child)),
    );
    declarations.sort_by_key(|declaration| declaration.source_order);
    declarations.dedup_by(|left, right| {
        left.source_order == right.source_order
            && left.property == right.property
            && left.value == right.value
            && left.important == right.important
    });
    declarations
}

fn semantic_declarations_from_direct_source_segments(
    ir: &TransformIrV0,
    node: &IrNodeV0,
    scope: SemanticObservationScopeV0<'_>,
) -> Vec<SemanticDeclarationV0> {
    let Some((body_start, body_end)) = node_body_bounds(ir, node) else {
        return Vec::new();
    };
    let mut children = node
        .children
        .iter()
        .filter_map(|child_id| ir.nodes.get(child_id.index()))
        .filter(|child| {
            !child.deleted
                && matches!(child.kind, IrNodeKindV0::StyleRule | IrNodeKindV0::AtRule)
                && child.source_span_start >= body_start
                && child.source_span_end <= body_end
        })
        .collect::<Vec<_>>();
    children.sort_by_key(|child| (child.source_span_start, child.global_order));

    let mut declarations = Vec::new();
    let mut cursor = body_start;
    for child in children {
        if cursor < child.source_span_start {
            declarations.extend(semantic_declarations_from_source_segment(
                ir,
                cursor,
                child.source_span_start,
                scope,
            ));
        }
        cursor = cursor.max(child.source_span_end);
    }
    if cursor < body_end {
        declarations.extend(semantic_declarations_from_source_segment(
            ir, cursor, body_end, scope,
        ));
    }
    declarations
}

fn semantic_declarations_from_source_segment(
    ir: &TransformIrV0,
    start: usize,
    end: usize,
    scope: SemanticObservationScopeV0<'_>,
) -> Vec<SemanticDeclarationV0> {
    if source_range_is_ignored(start, end, scope) {
        return Vec::new();
    }
    let Some(segment) = ir.source_text().get(start..end) else {
        return Vec::new();
    };
    split_declaration_list(segment)
        .into_iter()
        .enumerate()
        .filter_map(|(index, declaration)| {
            semantic_declaration_from_source(declaration.as_str(), start.saturating_add(index))
        })
        .collect()
}

fn node_body_bounds(ir: &TransformIrV0, node: &IrNodeV0) -> Option<(usize, usize)> {
    let source = node_text(ir, node)?;
    let open = source.find('{')?;
    let close = source.rfind('}')?;
    if close <= open {
        return None;
    }
    Some((
        node.source_span_start.checked_add(open + 1)?,
        node.source_span_start.checked_add(close)?,
    ))
}

fn semantic_observation_mismatch_count(
    input: &SemanticObservationV0,
    output: &SemanticObservationV0,
) -> usize {
    let missing_or_changed = input
        .iter()
        .filter(|(key, value)| output.get(*key) != Some(*value))
        .count();
    let added = output
        .keys()
        .filter(|key| !input.contains_key(*key))
        .count();
    missing_or_changed + added
}

fn has_deleted_ancestor(ir: &TransformIrV0, node: &IrNodeV0) -> bool {
    let mut parent = node.parent;
    while let Some(parent_id) = parent {
        let Some(parent_node) = ir.nodes.get(parent_id.index()) else {
            return true;
        };
        if parent_node.deleted {
            return true;
        }
        parent = parent_node.parent;
    }
    false
}

fn has_style_rule_ancestor(ir: &TransformIrV0, node: &IrNodeV0) -> bool {
    let mut parent = node.parent;
    while let Some(parent_id) = parent {
        let Some(parent_node) = ir.nodes.get(parent_id.index()) else {
            return false;
        };
        if parent_node.kind == IrNodeKindV0::StyleRule {
            return true;
        }
        parent = parent_node.parent;
    }
    false
}

fn node_text<'a>(ir: &'a TransformIrV0, node: &'a IrNodeV0) -> Option<&'a str> {
    node.canonical_text.as_deref().or_else(|| {
        ir.source_text()
            .get(node.source_span_start..node.source_span_end)
    })
}

fn normalize_selector_key(selector: &str) -> String {
    normalize_space(selector)
}

fn normalize_declaration_value(value: &str) -> String {
    normalize_space(value)
}

fn normalize_space(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn split_selector_list(selector: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut quote = None;
    let mut escaped = false;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    for (index, byte) in selector.bytes().enumerate() {
        if let Some(quote_byte) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == quote_byte {
                quote = None;
            }
            continue;
        }

        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => paren_depth = paren_depth.saturating_add(1),
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'[' => bracket_depth = bracket_depth.saturating_add(1),
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b',' if paren_depth == 0 && bracket_depth == 0 => {
                if let Some(part) = selector.get(start..index) {
                    parts.push(part.trim().to_string());
                }
                start = index.saturating_add(1);
            }
            _ => {}
        }
    }

    if let Some(part) = selector.get(start..) {
        parts.push(part.trim().to_string());
    }
    parts
}

fn split_declaration_list(body: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut quote = None;
    let mut escaped = false;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    for (index, byte) in body.bytes().enumerate() {
        if let Some(quote_byte) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == quote_byte {
                quote = None;
            }
            continue;
        }

        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => paren_depth = paren_depth.saturating_add(1),
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'[' => bracket_depth = bracket_depth.saturating_add(1),
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b';' if paren_depth == 0 && bracket_depth == 0 => {
                if let Some(part) = body.get(start..index) {
                    let trimmed = part.trim();
                    if !trimmed.is_empty() {
                        parts.push(trimmed.to_string());
                    }
                }
                start = index.saturating_add(1);
            }
            _ => {}
        }
    }

    if let Some(part) = body.get(start..) {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            parts.push(trimmed.to_string());
        }
    }
    parts
}

fn top_level_style_rule_sources(body: &str) -> Vec<String> {
    let bytes = body.as_bytes();
    let mut rules = Vec::new();
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        let Some(relative_open) = body.get(cursor..).and_then(|tail| tail.find('{')) else {
            break;
        };
        let open = cursor.saturating_add(relative_open);
        let selector_start = body
            .get(..open)
            .and_then(|prefix| prefix.rfind('}').map(|index| index.saturating_add(1)))
            .unwrap_or(0);
        let Some(close) = matching_brace_index(body, open) else {
            break;
        };
        if let Some(rule_source) = body.get(selector_start..=close) {
            let trimmed = rule_source.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('@') {
                rules.push(trimmed.to_string());
            }
        }
        cursor = close.saturating_add(1);
    }

    rules
}

fn matching_brace_index(source: &str, open: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    if bytes.get(open) != Some(&b'{') {
        return None;
    }
    let mut index = open;
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(quote_byte) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == quote_byte {
                quote = None;
            }
            index += 1;
            continue;
        }

        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'{' => depth = depth.saturating_add(1),
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn contains_nested_block_or_comment(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut index = 0usize;
    let mut quote = None;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(quote_byte) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == quote_byte {
                quote = None;
            }
            index += 1;
            continue;
        }
        if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
            index += 1;
            continue;
        }
        if matches!(byte, b'{' | b'}') || (byte == b'/' && bytes.get(index + 1) == Some(&b'*')) {
            return true;
        }
        index += 1;
    }
    false
}

fn declaration_colon_index(source: &str) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut index = 0usize;
    let mut quote = None;
    let mut escaped = false;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(quote_byte) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == quote_byte {
                quote = None;
            }
            index += 1;
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => paren_depth = paren_depth.saturating_add(1),
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'[' => bracket_depth = bracket_depth.saturating_add(1),
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b':' if paren_depth == 0 && bracket_depth == 0 => return Some(index),
            _ => {}
        }
        index += 1;
    }
    None
}

fn declaration_value_is_important(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'!' {
            let rest = value.get(index + 1..).unwrap_or_default().trim_start();
            return rest
                .get(.."important".len())
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case("important"));
        }
        index += 1;
    }
    false
}

#[cfg(test)]
fn semantic_model_conformance_case_results() -> Vec<bool> {
    let cases = [
        (
            "empty-rule-removal",
            ".a { color: red; }\n.a { color: blue; }\n.empty {}\n",
            ".a { color: red; }\n.a { color: blue; }\n",
            true,
        ),
        (
            "rule-deduplication",
            ".a { color: red !important; }\n.a { color: blue; }\n",
            ".a { color: red !important; }\n.a { color: blue; }\n",
            true,
        ),
        (
            "rule-deduplication",
            "@media (min-width: 1px) { .a { color: red; } }\n.a { color: blue; }\n",
            "@media (min-width: 1px) { .a { color: red; } }\n.a { color: blue; }\n",
            true,
        ),
        (
            "rule-deduplication",
            ".a { color: red !important; }\n.a { color: blue; }\n",
            ".a { color: blue; }\n",
            false,
        ),
        (
            "tree-shake-class",
            ".used { color: red; }\n.dead { color: blue; }\n",
            ".used { color: red; }\n",
            true,
        ),
        (
            "tree-shake-keyframes",
            "@keyframes used { to { opacity: 1; } }\n@keyframes dead { to { opacity: 0; } }\n.btn { animation: used 1s; }\n",
            "@keyframes used { to { opacity: 1; } }\n.btn { animation: used 1s; }\n",
            true,
        ),
        (
            "tree-shake-value",
            "@value used: red;\n@value dead: blue;\n.btn { color: used; }\n",
            "@value used: red;\n.btn { color: used; }\n",
            true,
        ),
        (
            "tree-shake-custom-property",
            "@property --used { syntax: \"<color>\"; inherits: false; initial-value: red; }\n@property --dead { syntax: \"<color>\"; inherits: false; initial-value: blue; }\n:root { --used: red; --dead: blue; }\n.btn { color: var(--used); }\n",
            "@property --used { syntax: \"<color>\"; inherits: false; initial-value: red; }\n:root { --used: red; }\n.btn { color: var(--used); }\n",
            true,
        ),
        (
            "nesting-unwrap",
            ".card { color: red; & .title { color: blue; } }\n",
            ".card { color: red; }\n.card .title { color: blue; }\n",
            true,
        ),
        (
            "scope-flatten",
            "@scope (:root) { .card { color: red; } }\n",
            ".card { color: red; }\n",
            true,
        ),
        (
            "layer-flatten",
            "@layer theme { .card { color: red; } }\n",
            ".card { color: red; }\n",
            true,
        ),
    ];

    cases
        .into_iter()
        .map(|(pass_id, input, output, expected_preserved)| {
            let input_ir = lower_transform_ir_from_source(input, StyleDialect::Css, "input");
            let output_ir = lower_transform_ir_from_source(output, StyleDialect::Css, "output");
            let reachable_class_names = vec!["used".to_string()];
            let keyframe_class_names = vec!["btn".to_string()];
            let value_class_names = vec!["btn".to_string()];
            let custom_property_class_names = vec!["btn".to_string()];
            let projection = if pass_id == "tree-shake-keyframes" {
                SemanticObservationProjectionV0::for_keyframe_reachability(
                    &input_ir,
                    &[],
                    &keyframe_class_names,
                )
            } else if pass_id == "tree-shake-value" {
                SemanticObservationProjectionV0::for_value_reachability(
                    &input_ir,
                    StyleDialect::Css,
                    &[],
                    &[],
                    &value_class_names,
                )
            } else if pass_id == "tree-shake-custom-property" {
                SemanticObservationProjectionV0::for_custom_property_reachability(
                    &input_ir,
                    StyleDialect::Css,
                    &[],
                    &[],
                    &custom_property_class_names,
                )
            } else {
                SemanticObservationProjectionV0::default()
            };
            let scope = if pass_id == "tree-shake-class" {
                SemanticObservationScopeV0::for_reachable_class_names(&reachable_class_names)
            } else if pass_id == "tree-shake-keyframes" {
                SemanticObservationScopeV0::for_ignored_source_ranges(
                    projection.ignored_source_ranges(),
                )
            } else if pass_id == "tree-shake-value" {
                SemanticObservationScopeV0::for_reachable_class_names_and_ignored_source_ranges(
                    &value_class_names,
                    projection.ignored_source_ranges(),
                )
            } else if pass_id == "tree-shake-custom-property" {
                SemanticObservationScopeV0::for_reachable_class_names_and_ignored_source_ranges(
                    &custom_property_class_names,
                    projection.ignored_source_ranges(),
                )
            } else {
                SemanticObservationScopeV0::default()
            };
            let decision = compare_semantic_observation_for_pass_with_scopes(
                pass_id,
                &input_ir,
                &output_ir,
                scope,
                scope.without_ignored_source_ranges(),
            );
            decision.preserved == expected_preserved
        })
        .collect()
}

#[cfg(test)]
fn stable_semantic_report_digest(parts: &[&str]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for part in parts {
        for byte in part.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observation_ignores_removed_empty_rules() {
        let input = lower_transform_ir_from_source(
            ".a { color: red; }\n.empty {}\n",
            StyleDialect::Css,
            "test",
        );
        let output =
            lower_transform_ir_from_source(".a { color: red; }\n", StyleDialect::Css, "test");
        let decision = compare_semantic_observation_for_pass("empty-rule-removal", &input, &output);

        assert!(decision.preserved);
        assert_eq!(decision.mismatch_count, 0);
        assert_eq!(decision.input_entry_count, 1);
        assert_eq!(decision.output_entry_count, 1);
    }

    #[test]
    fn observation_catches_declared_value_changes() {
        let input = lower_transform_ir_from_source(".a { color: red; }", StyleDialect::Css, "test");
        let output =
            lower_transform_ir_from_source(".a { color: blue; }", StyleDialect::Css, "test");
        let decision = compare_semantic_observation_for_pass("rule-deduplication", &input, &output);

        assert!(!decision.preserved);
        assert_eq!(decision.mismatch_count, 1);
    }

    #[test]
    fn observation_projects_class_tree_shake_to_reachable_selectors() {
        let reachable_class_names = vec!["used".to_string()];
        let input = lower_transform_ir_from_source(
            ".used { color: red; }\n.dead { color: blue; }\n.used, .dead-mixed { background: blue; }\n",
            StyleDialect::Css,
            "test",
        );
        let output = lower_transform_ir_from_source(
            ".used { color: red; }\n.used { background: blue; }\n",
            StyleDialect::Css,
            "test",
        );
        let decision = compare_semantic_observation_for_pass_with_scope(
            "tree-shake-class",
            &input,
            &output,
            SemanticObservationScopeV0::for_reachable_class_names(&reachable_class_names),
        );

        assert!(decision.preserved);
        assert_eq!(decision.mismatch_count, 0);
    }

    #[test]
    fn observation_rejects_reachable_class_tree_shake_changes() {
        let reachable_class_names = vec!["used".to_string()];
        let input = lower_transform_ir_from_source(
            ".used { color: red; }\n.dead { color: blue; }\n",
            StyleDialect::Css,
            "test",
        );
        let output =
            lower_transform_ir_from_source(".used { color: green; }\n", StyleDialect::Css, "test");
        let decision = compare_semantic_observation_for_pass_with_scope(
            "tree-shake-class",
            &input,
            &output,
            SemanticObservationScopeV0::for_reachable_class_names(&reachable_class_names),
        );

        assert!(!decision.preserved);
    }

    #[test]
    fn observation_projects_keyframe_tree_shake_to_reachable_rules() {
        let reachable_class_names = vec!["btn".to_string()];
        let input = lower_transform_ir_from_source(
            "@keyframes used { to { opacity: 1; } }\n@keyframes dead { to { opacity: 0; } }\n.btn { animation: used 1s; }\n",
            StyleDialect::Css,
            "test",
        );
        let output = lower_transform_ir_from_source(
            "@keyframes used { to { opacity: 1; } }\n.btn { animation: used 1s; }\n",
            StyleDialect::Css,
            "test",
        );
        let projection = SemanticObservationProjectionV0::for_keyframe_reachability(
            &input,
            &[],
            &reachable_class_names,
        );
        let decision = compare_semantic_observation_for_pass_with_scopes(
            "tree-shake-keyframes",
            &input,
            &output,
            SemanticObservationScopeV0::for_ignored_source_ranges(
                projection.ignored_source_ranges(),
            ),
            SemanticObservationScopeV0::default(),
        );

        assert!(decision.preserved);
        assert_eq!(decision.mismatch_count, 0);
    }

    #[test]
    fn observation_rejects_reachable_keyframe_tree_shake_changes() {
        let reachable_class_names = vec!["btn".to_string()];
        let input = lower_transform_ir_from_source(
            "@keyframes used { to { opacity: 1; } }\n@keyframes dead { to { opacity: 0; } }\n.btn { animation: used 1s; }\n",
            StyleDialect::Css,
            "test",
        );
        let output = lower_transform_ir_from_source(
            "@keyframes used { to { opacity: 0; } }\n.btn { animation: used 1s; }\n",
            StyleDialect::Css,
            "test",
        );
        let projection = SemanticObservationProjectionV0::for_keyframe_reachability(
            &input,
            &[],
            &reachable_class_names,
        );
        let decision = compare_semantic_observation_for_pass_with_scopes(
            "tree-shake-keyframes",
            &input,
            &output,
            SemanticObservationScopeV0::for_ignored_source_ranges(
                projection.ignored_source_ranges(),
            ),
            SemanticObservationScopeV0::default(),
        );

        assert!(!decision.preserved);
    }

    #[test]
    fn observation_projects_value_tree_shake_to_reachable_values() {
        let reachable_class_names = vec!["btn".to_string()];
        let input = lower_transform_ir_from_source(
            "@value used: red;\n@value dead: blue;\n.btn { color: used; }\n",
            StyleDialect::Css,
            "test",
        );
        let output = lower_transform_ir_from_source(
            "@value used: red;\n.btn { color: used; }\n",
            StyleDialect::Css,
            "test",
        );
        let projection = SemanticObservationProjectionV0::for_value_reachability(
            &input,
            StyleDialect::Css,
            &[],
            &[],
            &reachable_class_names,
        );
        let decision = compare_semantic_observation_for_pass_with_scopes(
            "tree-shake-value",
            &input,
            &output,
            SemanticObservationScopeV0::for_reachable_class_names_and_ignored_source_ranges(
                &reachable_class_names,
                projection.ignored_source_ranges(),
            ),
            SemanticObservationScopeV0::for_reachable_class_names(&reachable_class_names),
        );

        assert!(decision.preserved);
        assert_eq!(decision.mismatch_count, 0);
    }

    #[test]
    fn observation_rejects_reachable_value_tree_shake_changes() {
        let reachable_class_names = vec!["btn".to_string()];
        let input = lower_transform_ir_from_source(
            "@value used: red;\n@value dead: blue;\n.btn { color: used; }\n",
            StyleDialect::Css,
            "test",
        );
        let output = lower_transform_ir_from_source(
            "@value used: blue;\n.btn { color: used; }\n",
            StyleDialect::Css,
            "test",
        );
        let projection = SemanticObservationProjectionV0::for_value_reachability(
            &input,
            StyleDialect::Css,
            &[],
            &[],
            &reachable_class_names,
        );
        let decision = compare_semantic_observation_for_pass_with_scopes(
            "tree-shake-value",
            &input,
            &output,
            SemanticObservationScopeV0::for_reachable_class_names_and_ignored_source_ranges(
                &reachable_class_names,
                projection.ignored_source_ranges(),
            ),
            SemanticObservationScopeV0::for_reachable_class_names(&reachable_class_names),
        );

        assert!(!decision.preserved);
    }

    #[test]
    fn observation_projects_custom_property_tree_shake_to_reachable_roots() {
        let reachable_class_names = vec!["btn".to_string()];
        let input = lower_transform_ir_from_source(
            "@property --used { syntax: \"<color>\"; inherits: false; initial-value: red; }\n@property --dead { syntax: \"<color>\"; inherits: false; initial-value: blue; }\n:root { --used: red; --dead: blue; }\n.btn { color: var(--used); }\n",
            StyleDialect::Css,
            "test",
        );
        let output = lower_transform_ir_from_source(
            "@property --used { syntax: \"<color>\"; inherits: false; initial-value: red; }\n:root { --used: red; }\n.btn { color: var(--used); }\n",
            StyleDialect::Css,
            "test",
        );
        let projection = SemanticObservationProjectionV0::for_custom_property_reachability(
            &input,
            StyleDialect::Css,
            &[],
            &[],
            &reachable_class_names,
        );
        let decision = compare_semantic_observation_for_pass_with_scopes(
            "tree-shake-custom-property",
            &input,
            &output,
            SemanticObservationScopeV0::for_reachable_class_names_and_ignored_source_ranges(
                &reachable_class_names,
                projection.ignored_source_ranges(),
            ),
            SemanticObservationScopeV0::for_reachable_class_names(&reachable_class_names),
        );

        assert!(decision.preserved);
        assert_eq!(decision.mismatch_count, 0);
    }

    #[test]
    fn observation_rejects_reachable_custom_property_registration_changes() {
        let reachable_class_names = vec!["btn".to_string()];
        let input = lower_transform_ir_from_source(
            "@property --used { syntax: \"<color>\"; inherits: false; initial-value: red; }\n@property --dead { syntax: \"<color>\"; inherits: false; initial-value: blue; }\n:root { --used: red; --dead: blue; }\n.btn { color: var(--used); }\n",
            StyleDialect::Css,
            "test",
        );
        let output = lower_transform_ir_from_source(
            "@property --used { syntax: \"<color>\"; inherits: false; initial-value: blue; }\n:root { --used: red; }\n.btn { color: var(--used); }\n",
            StyleDialect::Css,
            "test",
        );
        let projection = SemanticObservationProjectionV0::for_custom_property_reachability(
            &input,
            StyleDialect::Css,
            &[],
            &[],
            &reachable_class_names,
        );
        let decision = compare_semantic_observation_for_pass_with_scopes(
            "tree-shake-custom-property",
            &input,
            &output,
            SemanticObservationScopeV0::for_reachable_class_names_and_ignored_source_ranges(
                &reachable_class_names,
                projection.ignored_source_ranges(),
            ),
            SemanticObservationScopeV0::for_reachable_class_names(&reachable_class_names),
        );

        assert!(!decision.preserved);
    }

    #[test]
    fn observation_expands_selector_lists_for_selector_merging() {
        let input = lower_transform_ir_from_source(
            ".a { color: red; }\n.b { color: red; }\n:is(.c, .d) { color: blue; }\n",
            StyleDialect::Css,
            "test",
        );
        let output = lower_transform_ir_from_source(
            ".a, .b { color: red; }\n:is(.c, .d) { color: blue; }\n",
            StyleDialect::Css,
            "test",
        );
        let decision = compare_semantic_observation_for_pass("selector-merging", &input, &output);

        assert!(decision.preserved);
        assert_eq!(decision.mismatch_count, 0);
        assert_eq!(decision.input_entry_count, 3);
        assert_eq!(decision.output_entry_count, 3);
    }

    #[test]
    fn observation_preserves_rule_merging_declaration_union() {
        let input = lower_transform_ir_from_source(
            ".a { color: red; }\n.a { background: blue; }\n",
            StyleDialect::Css,
            "test",
        );
        let output = lower_transform_ir_from_source(
            ".a { color: red; background: blue; }\n",
            StyleDialect::Css,
            "test",
        );
        let decision = compare_semantic_observation_for_pass("rule-merging", &input, &output);

        assert!(decision.preserved);
        assert_eq!(decision.mismatch_count, 0);
        assert_eq!(decision.input_entry_count, 2);
        assert_eq!(decision.output_entry_count, 2);
    }

    #[test]
    fn semantic_preservation_broken_translation_corpus_rejects_known_bad_outputs()
    -> Result<(), serde_json::Error> {
        let report = summarize_semantic_preservation_kill_rate_for_fixture_source(
            include_str!("../../fixtures/semantic-preservation/broken-simple.json"),
            StyleDialect::Css,
        )?;

        assert!(report.non_empty_corpus);
        assert_eq!(report.fixture_count, 2);
        assert_eq!(report.required_rejected_count, 2);
        assert_eq!(report.rejected_count, 2);
        assert!(report.kill_rate_passed);
        Ok(())
    }

    #[test]
    fn semantic_preservation_broken_merge_corpus_rejects_known_bad_outputs()
    -> Result<(), serde_json::Error> {
        let report = summarize_semantic_preservation_kill_rate_for_fixture_source(
            include_str!("../../fixtures/semantic-preservation/broken-merge.json"),
            StyleDialect::Css,
        )?;

        assert!(report.non_empty_corpus);
        assert_eq!(report.fixture_count, 2);
        assert_eq!(report.required_rejected_count, 2);
        assert_eq!(report.rejected_count, 2);
        assert!(report.kill_rate_passed);
        Ok(())
    }

    #[test]
    fn semantic_preservation_broken_shake_corpus_rejects_known_bad_outputs()
    -> Result<(), serde_json::Error> {
        let report = summarize_semantic_preservation_kill_rate_for_fixture_source(
            include_str!("../../fixtures/semantic-preservation/broken-shake.json"),
            StyleDialect::Css,
        )?;

        assert!(report.non_empty_corpus);
        assert_eq!(report.fixture_count, 8);
        assert_eq!(report.required_rejected_count, 8);
        assert_eq!(report.rejected_count, 8);
        assert!(report.kill_rate_passed);
        Ok(())
    }

    #[test]
    fn semantic_preservation_broken_flatten_corpus_rejects_known_bad_outputs()
    -> Result<(), serde_json::Error> {
        let report = summarize_semantic_preservation_kill_rate_for_fixture_source(
            include_str!("../../fixtures/semantic-preservation/broken-flatten.json"),
            StyleDialect::Css,
        )?;

        assert!(report.non_empty_corpus);
        assert_eq!(report.fixture_count, 3);
        assert_eq!(report.required_rejected_count, 3);
        assert_eq!(report.rejected_count, 3);
        assert!(report.kill_rate_passed);
        Ok(())
    }

    #[test]
    fn semantic_preservation_model_conformance_report_matches_committed_artifact()
    -> Result<(), serde_json::Error> {
        let actual = summarize_semantic_preservation_model_conformance()?;
        let expected = serde_json::from_str::<TransformSemanticModelConformanceReportV0>(
            include_str!("../../fixtures/semantic-preservation/model-conformance.json"),
        )?;

        assert_eq!(actual, expected);
        assert!(actual.model_conformance_passed);
        assert_eq!(actual.cascade_seed_failed_count, 0);
        assert_eq!(actual.wpt_seed_failed_count, 0);
        assert_eq!(actual.semantic_observation_failed_count, 0);
        Ok(())
    }
}

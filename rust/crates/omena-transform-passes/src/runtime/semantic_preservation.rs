//! Semantic preservation comparator for transform outputs (adoption gate for external lowering).

#[cfg(test)]
use omena_cascade::{run_cascade_conformance_seed_corpus, run_wpt_cascade_seed_corpus};
use omena_parser::{ClosedWorldBundleV0, StyleDialect};
use omena_transform_cst::{
    IrBlockSpanV0, IrNodeKindV0, IrNodeV0, TransformIrV0, TransformPassKind,
    lower_transform_ir_from_source, structural_block_spans_for_source,
};
#[cfg(test)]
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;

use crate::model::{
    TransformSemanticObservationKeyAxisV0, TransformSemanticObservationOrderingRuleV0,
    TransformSemanticObservationSurfaceV0, TransformSemanticObservationValueAxisV0,
    TransformSemanticPreservationClaimScopeV0, TransformSemanticPreservationTelemetryV0,
    TransformSemanticPreservationVocabularyReviewV0, TransformSemanticUnobservedAxisV0,
};
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

impl Default for TransformSemanticPreservationTelemetryV0 {
    fn default() -> Self {
        Self {
            observed_pass_count: 0,
            preserved_pass_count: 0,
            blocked_pass_count: 0,
            observed_surface: semantic_observation_surface_descriptor(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ExternalCssSemanticChangeKindV0 {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ExternalCssSemanticChangeClassificationV0 {
    Understood,
    Passthrough,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCssSemanticEntryV0 {
    pub selector: String,
    pub property: String,
    pub context: String,
    pub value: String,
    pub important: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCssSemanticChangeV0 {
    pub kind: ExternalCssSemanticChangeKindV0,
    pub classification: ExternalCssSemanticChangeClassificationV0,
    pub explanation: &'static str,
    pub before: Option<ExternalCssSemanticEntryV0>,
    pub after: Option<ExternalCssSemanticEntryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCssSemanticDiffV0 {
    pub input_entry_count: usize,
    pub output_entry_count: usize,
    pub total_change_count: usize,
    pub understood_change_count: usize,
    pub passthrough_change_count: usize,
    pub all_changes_classified: bool,
    pub changes: Vec<ExternalCssSemanticChangeV0>,
}

pub fn compare_external_css_semantic_changes_v0(
    input_css: &str,
    output_css: &str,
    dialect: StyleDialect,
) -> ExternalCssSemanticDiffV0 {
    let input_ir = lower_transform_ir_from_source(
        input_css,
        dialect,
        "omena-transform-passes.external-css-comparison.input",
    );
    let output_ir = lower_transform_ir_from_source(
        output_css,
        dialect,
        "omena-transform-passes.external-css-comparison.output",
    );
    let scope = SemanticObservationScopeV0::from_parts(None, None, &[], dialect);
    let input = semantic_observation(&input_ir, scope);
    let output = semantic_observation(&output_ir, scope);
    let mut changes = Vec::new();

    for (key, input_value) in &input {
        match output.get(key) {
            None => changes.push(classify_external_semantic_change(
                ExternalCssSemanticChangeKindV0::Removed,
                Some(external_semantic_entry(key, input_value)),
                None,
                &input,
                &output,
            )),
            Some(output_value) if output_value != input_value => {
                changes.push(classify_external_semantic_change(
                    ExternalCssSemanticChangeKindV0::Modified,
                    Some(external_semantic_entry(key, input_value)),
                    Some(external_semantic_entry(key, output_value)),
                    &input,
                    &output,
                ))
            }
            Some(_) => {}
        }
    }
    for (key, output_value) in &output {
        if !input.contains_key(key) {
            changes.push(classify_external_semantic_change(
                ExternalCssSemanticChangeKindV0::Added,
                None,
                Some(external_semantic_entry(key, output_value)),
                &input,
                &output,
            ));
        }
    }
    changes.sort();
    external_css_semantic_diff_from_changes(input.len(), output.len(), changes)
}

pub fn external_css_semantic_diff_is_total_v0(report: &ExternalCssSemanticDiffV0) -> bool {
    report.total_change_count == report.changes.len()
        && report.understood_change_count
            == report
                .changes
                .iter()
                .filter(|change| {
                    change.classification == ExternalCssSemanticChangeClassificationV0::Understood
                })
                .count()
        && report.passthrough_change_count
            == report
                .changes
                .iter()
                .filter(|change| {
                    change.classification == ExternalCssSemanticChangeClassificationV0::Passthrough
                })
                .count()
        && report.understood_change_count + report.passthrough_change_count
            == report.total_change_count
}

fn external_css_semantic_diff_from_changes(
    input_entry_count: usize,
    output_entry_count: usize,
    changes: Vec<ExternalCssSemanticChangeV0>,
) -> ExternalCssSemanticDiffV0 {
    let understood_change_count = changes
        .iter()
        .filter(|change| {
            change.classification == ExternalCssSemanticChangeClassificationV0::Understood
        })
        .count();
    let passthrough_change_count = changes.len().saturating_sub(understood_change_count);
    let mut report = ExternalCssSemanticDiffV0 {
        input_entry_count,
        output_entry_count,
        total_change_count: changes.len(),
        understood_change_count,
        passthrough_change_count,
        all_changes_classified: false,
        changes,
    };
    report.all_changes_classified = external_css_semantic_diff_is_total_v0(&report);
    report
}

fn classify_external_semantic_change(
    kind: ExternalCssSemanticChangeKindV0,
    before: Option<ExternalCssSemanticEntryV0>,
    after: Option<ExternalCssSemanticEntryV0>,
    input: &SemanticObservationV0,
    output: &SemanticObservationV0,
) -> ExternalCssSemanticChangeV0 {
    let understood_prefix_addition = kind == ExternalCssSemanticChangeKindV0::Added
        && after.as_ref().is_some_and(|entry| {
            vendor_unprefixed_property(entry.property.as_str()).is_some_and(|unprefixed| {
                let peer = SemanticObservationKeyV0 {
                    selector_key: entry.selector.clone(),
                    property: unprefixed.to_string(),
                    context_key: entry.context.clone(),
                };
                [input.get(&peer), output.get(&peer)]
                    .into_iter()
                    .flatten()
                    .any(|peer_value| {
                        peer_value.value == entry.value && peer_value.important == entry.important
                    })
            })
        });
    let (classification, explanation) = if understood_prefix_addition {
        (
            ExternalCssSemanticChangeClassificationV0::Understood,
            "targetVendorPrefixAddition",
        )
    } else {
        (
            ExternalCssSemanticChangeClassificationV0::Passthrough,
            "externalSemanticChange",
        )
    };
    ExternalCssSemanticChangeV0 {
        kind,
        classification,
        explanation,
        before,
        after,
    }
}

fn vendor_unprefixed_property(property: &str) -> Option<&str> {
    ["-webkit-", "-moz-", "-ms-", "-o-"]
        .into_iter()
        .find_map(|prefix| property.strip_prefix(prefix))
        .filter(|property| !property.is_empty())
}

fn external_semantic_entry(
    key: &SemanticObservationKeyV0,
    value: &SemanticObservationValueV0,
) -> ExternalCssSemanticEntryV0 {
    ExternalCssSemanticEntryV0 {
        selector: key.selector_key.clone(),
        property: key.property.clone(),
        context: key.context_key.clone(),
        value: value.value.clone(),
        important: value.important,
    }
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

impl SemanticObservationKeyV0 {
    const FIELD_BINDINGS: [(&'static str, TransformSemanticObservationKeyAxisV0); 3] = [
        (
            "selector_key",
            TransformSemanticObservationKeyAxisV0::Selector,
        ),
        ("property", TransformSemanticObservationKeyAxisV0::Property),
        (
            "context_key",
            TransformSemanticObservationKeyAxisV0::Context,
        ),
    ];
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticObservationValueV0 {
    value: String,
    important: bool,
}

impl SemanticObservationValueV0 {
    const FIELD_BINDINGS: [(&'static str, TransformSemanticObservationValueAxisV0); 2] = [
        ("value", TransformSemanticObservationValueAxisV0::Value),
        (
            "important",
            TransformSemanticObservationValueAxisV0::Important,
        ),
    ];
}

fn semantic_observation_surface_descriptor() -> TransformSemanticObservationSurfaceV0 {
    TransformSemanticObservationSurfaceV0 {
        key_axes: SemanticObservationKeyV0::FIELD_BINDINGS
            .iter()
            .map(|(_, axis)| *axis)
            .collect(),
        value_axes: SemanticObservationValueV0::FIELD_BINDINGS
            .iter()
            .map(|(_, axis)| *axis)
            .collect(),
        ordering_rules: vec![
            TransformSemanticObservationOrderingRuleV0::SourceOrder,
            TransformSemanticObservationOrderingRuleV0::ImportantPrecedence,
        ],
        unobserved_axes: vec![
            TransformSemanticUnobservedAxisV0::InterSelectorSpecificityCompetition,
            TransformSemanticUnobservedAxisV0::CascadeLayerOrder,
            TransformSemanticUnobservedAxisV0::Origin,
            TransformSemanticUnobservedAxisV0::ScopeProximity,
            TransformSemanticUnobservedAxisV0::DomDependentMatching,
            TransformSemanticUnobservedAxisV0::Inheritance,
            TransformSemanticUnobservedAxisV0::CustomPropertyEnvironment,
            TransformSemanticUnobservedAxisV0::AnimationAndTransition,
        ],
        claim_scope: TransformSemanticPreservationClaimScopeV0::ObservedSurfaceOnly,
        vocabulary_review:
            TransformSemanticPreservationVocabularyReviewV0::DeferredUntilFullCascadeObservation,
    }
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
    let block_view = node_text_block_view(ir, node)?;
    let prelude = block_view.prelude(block_view.primary)?.trim();
    if !at_rule_prelude_is_reachable_in_scope(prelude, scope) {
        return None;
    }
    if has_style_rule_ancestor(ir, node) {
        return nested_at_rule_declaration_candidates(ir, node, prelude, scope);
    }
    let context_key = join_context_components(
        ancestor_at_rule_context_key(ir, node),
        at_rule_context_component_from_prelude(prelude),
    );
    let mut candidates = Vec::new();

    for (index, rule_span) in block_view.direct_child_spans().into_iter().enumerate() {
        let selector = block_view.prelude(rule_span)?;
        if selector.trim_start().starts_with('@') {
            continue;
        }
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
        let declarations = semantic_declarations_from_block(
            block_view.source,
            rule_span,
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
    let block_view = node_text_block_view(ir, node)?;
    semantic_declarations_from_block(
        block_view.source,
        block_view.primary,
        node.global_order.saturating_mul(1024),
    )
}

fn semantic_declarations_from_block(
    source: &str,
    span: IrBlockSpanV0,
    base_source_order: usize,
) -> Option<Vec<SemanticDeclarationV0>> {
    let body = source.get(span.body_start..span.body_end)?;
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
    let mut selector = node_block_prelude(ir, node)?.trim().to_string();
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
                selectors.push(node_block_prelude(ir, parent_node)?.trim().to_string());
            }
            IrNodeKindV0::AtRule => {
                if let Some(selector) =
                    node_block_prelude(ir, parent_node).and_then(nest_at_rule_selector_from_prelude)
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
    let prelude = node_block_prelude(ir, node).or_else(|| node_text(ir, node))?;
    at_rule_context_component_from_prelude(prelude.trim())
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
    let Some(block_span) = node.block_span else {
        return Vec::new();
    };
    let body_start = block_span.body_start;
    let body_end = block_span.body_end;
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

struct NodeTextBlockViewV0<'source> {
    source: &'source str,
    primary: IrBlockSpanV0,
    spans: Vec<IrBlockSpanV0>,
}

impl NodeTextBlockViewV0<'_> {
    fn prelude(&self, span: IrBlockSpanV0) -> Option<&str> {
        self.source.get(span.prelude_start..span.open_brace_start)
    }

    fn direct_child_spans(&self) -> Vec<IrBlockSpanV0> {
        let nested = self
            .spans
            .iter()
            .copied()
            .filter(|span| {
                *span != self.primary
                    && self.primary.body_start <= span.prelude_start
                    && span.rule_end <= self.primary.body_end
            })
            .collect::<Vec<_>>();
        nested
            .iter()
            .copied()
            .filter(|candidate| {
                !nested.iter().any(|owner| {
                    owner != candidate
                        && owner.body_start <= candidate.prelude_start
                        && candidate.rule_end <= owner.body_end
                })
            })
            .collect()
    }
}

fn node_text_block_view<'source>(
    ir: &'source TransformIrV0,
    node: &'source IrNodeV0,
) -> Option<NodeTextBlockViewV0<'source>> {
    let source = node_text(ir, node)?;
    if node.canonical_text.is_some() {
        let spans = structural_block_spans_for_source(source, ir_style_dialect(ir)?);
        let primary = spans
            .iter()
            .copied()
            .filter(|span| span.prelude_start == 0)
            .max_by_key(|span| span.rule_end)?;
        return Some(NodeTextBlockViewV0 {
            source,
            primary,
            spans,
        });
    }

    let primary = shift_block_span(node.block_span?, node.source_span_start)?;
    let spans = ir
        .structural_block_spans()
        .iter()
        .copied()
        .filter(|span| {
            node.source_span_start <= span.prelude_start && span.rule_end <= node.source_span_end
        })
        .filter_map(|span| shift_block_span(span, node.source_span_start))
        .collect();
    Some(NodeTextBlockViewV0 {
        source,
        primary,
        spans,
    })
}

fn shift_block_span(span: IrBlockSpanV0, offset: usize) -> Option<IrBlockSpanV0> {
    Some(IrBlockSpanV0 {
        prelude_start: span.prelude_start.checked_sub(offset)?,
        open_brace_start: span.open_brace_start.checked_sub(offset)?,
        body_start: span.body_start.checked_sub(offset)?,
        body_end: span.body_end.checked_sub(offset)?,
        rule_end: span.rule_end.checked_sub(offset)?,
    })
}

fn ir_style_dialect(ir: &TransformIrV0) -> Option<StyleDialect> {
    match ir.dialect {
        "css" => Some(StyleDialect::Css),
        "scss" => Some(StyleDialect::Scss),
        "sass" => Some(StyleDialect::Sass),
        "less" => Some(StyleDialect::Less),
        _ => None,
    }
}

fn node_block_prelude<'source>(
    ir: &'source TransformIrV0,
    node: &'source IrNodeV0,
) -> Option<&'source str> {
    let view = node_text_block_view(ir, node)?;
    view.source
        .get(view.primary.prelude_start..view.primary.open_brace_start)
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SemanticObservationContractV0 {
    schema_version: String,
    product: String,
    cases: Vec<SemanticObservationContractCaseV0>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SemanticObservationContractCaseV0 {
    case_id: String,
    entries: Vec<SemanticObservationContractEntryV0>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SemanticObservationContractEntryV0 {
    selector: String,
    property: String,
    context: String,
    value: String,
    important: bool,
}

#[cfg(test)]
fn semantic_observation_contract_snapshot() -> SemanticObservationContractV0 {
    let cases = [
        (
            "cascade-and-media",
            StyleDialect::Css,
            ".card { color: red; } .card { color: blue !important; } @media (width > 20rem) { .card { display: grid; } }",
        ),
        (
            "nested-scss",
            StyleDialect::Scss,
            ".card { color: red; &:hover { color: blue; } @media (width > 20rem) { &__title { display: block; } } }",
        ),
        (
            "keyframes",
            StyleDialect::Css,
            "@keyframes fade { from { opacity: 0; } 50% { opacity: .5; } to { opacity: 1; } } .card { animation: fade 1s; }",
        ),
        (
            "css-modules",
            StyleDialect::Css,
            "@value tone: red; :export { exported: tone; } .button { composes: base from \"./base.css\"; color: tone; }",
        ),
        (
            "delimiter-and-comment",
            StyleDialect::Css,
            ".card { content: \"{; }\"; /* observer falls back to typed declarations */ color: red; }",
        ),
    ]
    .into_iter()
    .map(|(case_id, dialect, source)| {
        let ir = lower_transform_ir_from_source(source, dialect, case_id);
        let scope = SemanticObservationScopeV0::from_parts(None, None, &[], dialect);
        let entries = semantic_observation(&ir, scope)
            .into_iter()
            .map(|(key, value)| SemanticObservationContractEntryV0 {
                selector: key.selector_key,
                property: key.property,
                context: key.context_key,
                value: value.value,
                important: value.important,
            })
            .collect();
        SemanticObservationContractCaseV0 {
            case_id: case_id.to_string(),
            entries,
        }
    })
    .collect();

    SemanticObservationContractV0 {
        schema_version: "0".to_string(),
        product: "omena-transform-passes.semantic-observation-contract".to_string(),
        cases,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn struct_field_names(source: &str, struct_name: &str) -> Vec<String> {
        let marker = format!("struct {struct_name} {{");
        let body = source.split_once(&marker).map(|(_, body)| body);
        assert!(body.is_some(), "missing struct {struct_name}");
        let Some(body) = body else {
            return Vec::new();
        };
        body.lines()
            .take_while(|line| line.trim() != "}")
            .filter_map(|line| {
                let (field, _) = line.trim().trim_end_matches(',').split_once(':')?;
                let field = field.trim();
                field
                    .chars()
                    .all(|character| character == '_' || character.is_ascii_alphanumeric())
                    .then(|| field.to_string())
            })
            .collect()
    }

    fn observer_shape_matches_bindings(source: &str) -> bool {
        let key_fields = struct_field_names(source, "SemanticObservationKeyV0");
        let value_fields = struct_field_names(source, "SemanticObservationValueV0");
        let bound_key_fields = SemanticObservationKeyV0::FIELD_BINDINGS
            .iter()
            .map(|(field, _)| field.to_string())
            .collect::<Vec<_>>();
        let bound_value_fields = SemanticObservationValueV0::FIELD_BINDINGS
            .iter()
            .map(|(field, _)| field.to_string())
            .collect::<Vec<_>>();
        key_fields == bound_key_fields && value_fields == bound_value_fields
    }

    #[test]
    fn semantic_observation_surface_descriptor_matches_observer_types() {
        let source = include_str!("semantic_preservation.rs");
        assert!(observer_shape_matches_bindings(source));

        let descriptor = semantic_observation_surface_descriptor();
        assert_eq!(
            descriptor.key_axes,
            SemanticObservationKeyV0::FIELD_BINDINGS
                .iter()
                .map(|(_, axis)| *axis)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            descriptor.value_axes,
            SemanticObservationValueV0::FIELD_BINDINGS
                .iter()
                .map(|(_, axis)| *axis)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            descriptor.ordering_rules,
            vec![
                TransformSemanticObservationOrderingRuleV0::SourceOrder,
                TransformSemanticObservationOrderingRuleV0::ImportantPrecedence,
            ]
        );
        assert_eq!(descriptor.unobserved_axes.len(), 8);
        assert_eq!(
            descriptor.claim_scope,
            TransformSemanticPreservationClaimScopeV0::ObservedSurfaceOnly
        );
        assert_eq!(
            descriptor.vocabulary_review,
            TransformSemanticPreservationVocabularyReviewV0::DeferredUntilFullCascadeObservation
        );
    }

    #[test]
    fn semantic_observation_surface_descriptor_detects_shape_drift() {
        let source = include_str!("semantic_preservation.rs");
        let widened = source.replacen(
            "context_key: String,",
            "context_key: String,\n    specificity: String,",
            1,
        );
        assert!(!observer_shape_matches_bindings(&widened));
    }

    #[test]
    fn semantic_observation_ordering_matches_the_disclosed_rules() {
        let ir = lower_transform_ir_from_source(
            ".a { color: red; } .a { color: blue !important; } .a { color: green; } .b { color: red; } .b { color: blue; }",
            StyleDialect::Css,
            "semantic-observation-ordering",
        );
        let observation = semantic_observation(&ir, SemanticObservationScopeV0::default());

        let important = observation.get(&SemanticObservationKeyV0 {
            selector_key: ".a".to_string(),
            property: "color".to_string(),
            context_key: String::new(),
        });
        assert!(important.is_some(), "important declaration observation");
        if let Some(important) = important {
            assert_eq!(important.value, "blue !important");
            assert!(important.important);
        }

        let source_order = observation.get(&SemanticObservationKeyV0 {
            selector_key: ".b".to_string(),
            property: "color".to_string(),
            context_key: String::new(),
        });
        assert!(
            source_order.is_some(),
            "source-order declaration observation"
        );
        if let Some(source_order) = source_order {
            assert_eq!(source_order.value, "blue");
            assert!(!source_order.important);
        }
    }

    #[test]
    fn semantic_observation_matches_committed_contract() -> Result<(), serde_json::Error> {
        let actual = semantic_observation_contract_snapshot();
        let expected = serde_json::from_str::<SemanticObservationContractV0>(include_str!(
            "../../fixtures/semantic-preservation/observer-contract.json"
        ))?;

        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn semantic_observation_contract_detects_a_dropped_declaration() -> Result<(), serde_json::Error>
    {
        let expected = serde_json::from_str::<SemanticObservationContractV0>(include_str!(
            "../../fixtures/semantic-preservation/observer-contract.json"
        ))?;
        let mut lossy = semantic_observation_contract_snapshot();
        assert_eq!(lossy.cases[0].case_id, "cascade-and-media");
        let case = &mut lossy.cases[0];
        let entry_count = case.entries.len();
        case.entries
            .retain(|entry| entry.property != "display" || entry.context.is_empty());
        assert_eq!(case.entries.len() + 1, entry_count);
        assert_ne!(lossy, expected);

        let input = lower_transform_ir_from_source(
            ".card { color: red; display: grid; }",
            StyleDialect::Css,
            "semantic-drop-input",
        );
        let output = lower_transform_ir_from_source(
            ".card { color: red; }",
            StyleDialect::Css,
            "semantic-drop-output",
        );
        let decision = compare_semantic_observation_for_pass("rule-deduplication", &input, &output);
        assert!(!decision.preserved);
        assert_eq!(decision.mismatch_count, 1);
        Ok(())
    }

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
    fn external_css_diff_classifies_known_prefixes_and_preserves_unknown_changes() {
        let input = ".input { appearance: none; } ::placeholder { color: gray; }";
        let output = ".input { -webkit-appearance: none; appearance: none; } ::-moz-placeholder { color: gray; } ::placeholder { color: gray; }";
        let report = compare_external_css_semantic_changes_v0(input, output, StyleDialect::Css);

        assert!(report.all_changes_classified);
        assert!(report.understood_change_count >= 1);
        assert!(report.passthrough_change_count >= 1);
        assert_eq!(
            report.understood_change_count + report.passthrough_change_count,
            report.total_change_count
        );
        assert!(report.changes.iter().any(|change| {
            change.classification == ExternalCssSemanticChangeClassificationV0::Understood
                && change
                    .after
                    .as_ref()
                    .is_some_and(|entry| entry.property == "-webkit-appearance")
        }));
        assert!(report.changes.iter().any(|change| {
            change.classification == ExternalCssSemanticChangeClassificationV0::Passthrough
                && change
                    .after
                    .as_ref()
                    .is_some_and(|entry| entry.selector.contains("::-moz-placeholder"))
        }));
    }

    #[test]
    fn external_css_diff_totality_rejects_an_unreported_change() {
        let mut report = compare_external_css_semantic_changes_v0(
            ".input { appearance: none; }",
            ".input { -webkit-appearance: none; appearance: none; }",
            StyleDialect::Css,
        );
        assert!(report.all_changes_classified);
        assert!(external_css_semantic_diff_is_total_v0(&report));

        report.changes.clear();
        assert!(!external_css_semantic_diff_is_total_v0(&report));
    }

    #[test]
    fn external_css_diff_does_not_understand_a_prefix_with_different_semantics() {
        let report = compare_external_css_semantic_changes_v0(
            ".input { appearance: none; }",
            ".input { -webkit-appearance: auto; appearance: none; }",
            StyleDialect::Css,
        );

        assert_eq!(report.understood_change_count, 0);
        assert_eq!(report.passthrough_change_count, 1);
        assert!(report.all_changes_classified);
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

use std::collections::{BTreeMap, BTreeSet};

use omena_cascade::{
    CascadeDeclaration, CascadeKey, CascadeLevel, CascadeOutcome, CascadeValue, LayerRank,
    SelectorMatchVerdict, Specificity, StaticSupportsAssumptionV0, StaticSupportsEvalVerdictV0,
    cascade_margin_for_outcome, cascade_property, evaluate_static_supports_condition,
    parse_simple_selector_signature, selector_co_match_verdict,
};
use omena_parser::{LexedToken, lex};
use omena_query_checker_orchestrator::{
    CanonicalSelector, OmenaCheckerCascadeDeclarationInputV0, OmenaCheckerCascadeEvaluationV0,
    OmenaCheckerCascadeInputV0, OmenaCheckerCategoricalInputV0,
    OmenaCheckerCategoricalPrimitiveRolePairInputV0, OmenaCheckerCategoricalRoleMappingInputV0,
    OmenaCheckerCustomPropertyInputV0, OmenaCheckerCustomPropertyRegistrationInputV0,
    OmenaCheckerRgFlowCouplingInputV0, OmenaCheckerRgFlowCouplingSpaceInputV0,
    OmenaCheckerRgFlowInputV0, OmenaCheckerSmtInputV0,
    OmenaCheckerSmtLayerInversionDeclarationInputV0, OmenaCheckerSmtLayerInversionInputV0,
    OmenaCheckerSmtLayerInversionObligationInputV0, OmenaCheckerSmtObligationInputV0,
    checker_cascade_primitive_role_catalog_v0, run_omena_query_checker_cascade_gate_v0,
    run_omena_query_checker_categorical_gate_v0, run_omena_query_checker_rg_flow_gate_v0,
    run_omena_query_checker_smt_gate_v0, run_omena_query_checker_smt_layer_inversion_gate_v0,
};
use omena_query_checker_orchestrator::{
    REPLICA_ENSEMBLE_FEATURE_GATE_V0, REPLICA_ENSEMBLE_LAYER_MARKER_V0,
    REPLICA_ENSEMBLE_SCHEMA_VERSION_V0, ReplicaSiteOutcomeV0, site as replica_ensemble_site,
};
use omena_query_core::{
    AbstractClassValueV0, AbstractPropertyValueCandidateV0,
    iterate_reduced_class_value_product_constraints,
    narrow_abstract_property_value_for_cascade_branch, prefix_suffix_class_value,
};
use omena_query_transform_runner::expand_css_nested_selector;
use omena_syntax::SyntaxKind;

use super::{
    OmenaQueryCascadeConfidenceV0, OmenaQueryCascadeNarrowingEvidenceV0,
    OmenaQueryInlineStyleRuntimeOverrideV0, OmenaQueryRuntimeStateDriverSummaryV0,
    OmenaQueryRuntimeStateScenarioEvidenceV0, OmenaQueryRuntimeStateScenarioV0,
    OmenaQueryRuntimeStateStaticBoundaryV0, OmenaQueryStaticConditionPruningEvidenceV0,
    OmenaQueryStyleDiagnosticV0, ParserByteSpanV0, ParserRangeV0,
    omena_parser_dialect_for_style_path, parser_range_for_byte_span,
    summarize_static_css_custom_property_fixed_point_from_source,
};

const LSP_DIAGNOSTIC_TAG_UNNECESSARY: u8 = 1;

/// Cascade checker surface with an explicit deep-analysis switch.
///
/// The default surface entry passes `deep_analysis == false`: the rg-flow +
/// categorical *theory* diagnostics are opt-in deep-analysis hints, so the
/// default LSP/CLI surface keeps only the product cascade diagnostics (e.g.
/// `circularVar`).
///
/// `deep_analysis == false` (the default) emits only the product cascade gate
/// diagnostics. `deep_analysis == true` additionally surfaces the opt-in rg-flow
/// (`rgFlowRelevantOperator`) and categorical
/// (`categoricalCascadeEvidenceInconsistency`) theory hints — but those hints are
/// *deduplicated* against the product `circularVar` warning: on a single
/// custom-property reference cycle the product chain already emits a `circularVar`
/// warning over the cyclic declarations, so the two whole-file-ranged theory hints
/// that key off the same `has_reference_cycle` predicate would be a redundant
/// triple-fire. When a theory hint's range overlaps a range where `circularVar`
/// already fired, the hint is folded into that `circularVar` diagnostic's
/// provenance instead of surfacing a second/third diagnostic, so a lone var cycle
/// yields exactly one diagnostic.
pub(super) fn summarize_query_cascade_checker_diagnostics_with_deep_analysis(
    style_uri: &str,
    source: &str,
    deep_analysis: bool,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let (checker_input, declaration_ranges, custom_property_ranges) =
        collect_query_checker_cascade_input(style_uri, source);
    let mut diagnostics = Vec::new();

    // Theory diagnostics are produced eagerly only when deep-analysis is on; the
    // default surface skips the (whole-file-ranged, non-actionable) theory hints
    // entirely so the LSP/CLI output stays clean.
    let (rg_flow_diagnostics, categorical_diagnostics, smt_diagnostics) = if deep_analysis {
        (
            summarize_query_rg_flow_coupling_diagnostics(source, &checker_input.custom_properties),
            summarize_query_categorical_cascade_evidence_diagnostics(
                source,
                &checker_input.custom_properties,
            ),
            summarize_query_smt_cascade_obligation_diagnostics(
                source,
                &checker_input.declarations,
                &declaration_ranges,
            ),
        )
    } else {
        (Vec::new(), Vec::new(), Vec::new())
    };

    let gate = run_omena_query_checker_cascade_gate_v0(checker_input.clone());
    if !gate.enforcement_passed {
        return vec![OmenaQueryStyleDiagnosticV0 {
            code: "checkerDiagnosticGateFailed",
            severity: "warning",
            provenance: vec![
                "omena-query-checker-orchestrator.cascade-gate",
                "omena-query.cascade-checker",
            ],
            range: parser_range_for_byte_span(
                source,
                ParserByteSpanV0 {
                    start: 0,
                    end: source.len(),
                },
            ),
            message: "Checker diagnostic gate rejected unregistered rule output.".to_string(),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        }];
    }

    // Build the product cascade gate diagnostics first so the `circularVar`
    // ranges are known before the theory hints are deduplicated against them.
    for evaluation in gate.evaluations {
        if evaluation.rule_code_name == "iacvt-prone"
            && evaluation
                .custom_property_names
                .iter()
                .all(|name| !custom_property_ranges.contains_key(name))
        {
            continue;
        }
        let range = evaluation
            .declaration_ids
            .iter()
            .find_map(|declaration_id| declaration_ranges.get(declaration_id).copied())
            .or_else(|| {
                evaluation
                    .custom_property_names
                    .iter()
                    .find_map(|name| custom_property_ranges.get(name).copied())
            })
            .unwrap_or_else(|| {
                parser_range_for_byte_span(
                    source,
                    ParserByteSpanV0 {
                        start: 0,
                        end: source.len(),
                    },
                )
            });
        let mut provenance = vec![
            "omena-query-checker-orchestrator.cascade-gate",
            "omena-checker.cascade-rules",
            "omena-query.cascade-checker",
        ];
        provenance.extend(evaluation.mechanism_products.iter().copied());
        let cascade_narrowing = summarize_query_cascade_narrowing_for_evaluation(
            &evaluation,
            checker_input.declarations.as_slice(),
        );
        if cascade_narrowing.is_some() {
            provenance.extend([
                "omena-query.cascade-narrowing",
                "omena-abstract-value.property-value-narrowing",
                "omena-abstract-value.reduced-product-iteration",
            ]);
        }
        let cascade_confidence = summarize_query_cascade_confidence_for_evaluation(
            &evaluation,
            checker_input.declarations.as_slice(),
        );
        if cascade_confidence.is_some() {
            provenance.extend(["omena-cascade.margin", "omena-query.cascade-confidence"]);
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: query_cascade_checker_code(evaluation.rule_code_name),
            severity: query_cascade_checker_diagnostic_severity(evaluation.rule_code_name),
            provenance,
            range,
            message: evaluation.message,
            tags: query_cascade_checker_diagnostic_tags(evaluation.rule_code_name),
            create_custom_property: None,
            cascade_narrowing,
            cascade_confidence,
            polynomial_provenance: None,
            cross_file_scc: None,
        });
    }

    if deep_analysis {
        deduplicate_query_theory_hints_against_circular_var(
            &mut diagnostics,
            rg_flow_diagnostics,
            categorical_diagnostics,
        );
        // The SMT cascade-violation diagnostics are anchored on the specific
        // longhand declaration that breaks the combination obligation (not the
        // whole-file span the rg-flow/categorical hints use), so they are a
        // distinct, actionable diagnostic and are appended directly rather than
        // deduplicated against `circularVar`.
        diagnostics.extend(smt_diagnostics);
    }

    diagnostics
}

/// Fold the opt-in rg-flow / categorical theory hints into the product
/// `circularVar` diagnostics on any range where `circularVar` already fired,
/// instead of surfacing a redundant second/third whole-file-ranged hint.
///
/// On a single custom-property reference cycle the product chain emits one
/// `circularVar` warning anchored on the cyclic declaration, while both theory
/// hints key off the same `has_reference_cycle` predicate and re-detect the same
/// cycle. Surfacing all three is a triple-fire, so each theory hint whose range
/// overlaps an already-fired `circularVar` range is suppressed and its
/// `omena-checker.*` provenance label is merged into the matching `circularVar`
/// diagnostic (preserving the audit trail that the theory mechanism ran without
/// emitting a duplicate squiggle). A theory hint that does *not* overlap any
/// `circularVar` range (e.g. a genuinely acyclic high-gain hub, if a deeper
/// producer is later wired) is kept as a distinct diagnostic.
fn deduplicate_query_theory_hints_against_circular_var(
    diagnostics: &mut Vec<OmenaQueryStyleDiagnosticV0>,
    theory_hints: impl IntoIterator<Item = OmenaQueryStyleDiagnosticV0>,
    extra_theory_hints: impl IntoIterator<Item = OmenaQueryStyleDiagnosticV0>,
) {
    let circular_var_ranges = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "circularVar")
        .map(|diagnostic| diagnostic.range)
        .collect::<BTreeSet<_>>();

    for hint in theory_hints.into_iter().chain(extra_theory_hints) {
        // A whole-file-ranged theory hint (start-of-file origin) covers the
        // cyclic declaration that `circularVar` already flagged, so treat
        // "circularVar fired anywhere" as the dedup trigger and fold the hint's
        // provenance into every `circularVar` diagnostic. A hint with an exact
        // matching range is likewise deduplicated.
        let overlaps = !circular_var_ranges.is_empty()
            && (circular_var_ranges.contains(&hint.range)
                || query_theory_hint_range_is_whole_file(&hint.range));
        if overlaps {
            for diagnostic in diagnostics
                .iter_mut()
                .filter(|diagnostic| diagnostic.code == "circularVar")
            {
                for label in hint.provenance.iter().copied() {
                    if !diagnostic.provenance.contains(&label) {
                        diagnostic.provenance.push(label);
                    }
                }
            }
        } else {
            diagnostics.push(hint);
        }
    }
}

/// A theory hint is whole-file-ranged when it starts at the document origin; the
/// rg-flow / categorical hints are always emitted over the whole-file span, so a
/// hint starting at line/char 0 is treated as covering any in-file `circularVar`.
fn query_theory_hint_range_is_whole_file(range: &ParserRangeV0) -> bool {
    range.start.line == 0 && range.start.character == 0
}

/// Surface the real RG-flow coupling-Jacobian-spectrum diagnostic in the query
/// style path.
///
/// The coupling space is extracted from the parsed custom-property dependency
/// graph: the `before` state is the raw declared structure, and the `after`
/// state adds the custom properties that participate in a reference cycle,
/// those that resolve to the guaranteed-invalid value, and an acyclic fan-out
/// pressure term for high-gain custom-property hubs. A diverging stylesheet
/// (growing cyclic / guaranteed-invalid / high-gain coupling) drives the
/// spectral radius above one through `estimate_coupling_jacobian_spectrum_v0`,
/// so the gate emits `rg-flow-relevant-operator`. A settled stylesheet keeps
/// `before == after`, the spectral radius is zero, and nothing is surfaced.
fn summarize_query_rg_flow_coupling_diagnostics(
    source: &str,
    custom_properties: &[OmenaCheckerCustomPropertyInputV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(flow) = query_rg_flow_coupling_for_custom_properties(custom_properties) else {
        return Vec::new();
    };

    let gate =
        run_omena_query_checker_rg_flow_gate_v0(OmenaCheckerRgFlowInputV0 { flows: vec![flow] });
    if !gate.enforcement_passed {
        return Vec::new();
    }

    let whole_file_range = parser_range_for_byte_span(
        source,
        ParserByteSpanV0 {
            start: 0,
            end: source.len(),
        },
    );

    gate.evaluations
        .into_iter()
        .map(|evaluation| {
            let mut provenance = vec![
                "omena-query-checker-orchestrator.rg-flow-gate",
                "omena-checker.rg-flow-rules",
                "omena-query.cascade-checker",
            ];
            provenance.extend(evaluation.mechanism_products.iter().copied());
            OmenaQueryStyleDiagnosticV0 {
                code: "rgFlowRelevantOperator",
                severity: "hint",
                provenance,
                range: whole_file_range,
                message: evaluation.message,
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            }
        })
        .collect()
}

/// Surface the real categorical cascade primitive-to-role functor diagnostic in
/// the query style path.
///
/// The role mapping models the categorical witness role each exercised cascade
/// primitive plays. The cascade-ranking primitive (`cascade_property`) is
/// supposed to be the cosheaf-colimit witness: the least-fixed-point ranking of
/// every custom property converges to a single computed value. When the parsed
/// custom-property reference graph contains a cycle the ranking colimit cannot
/// converge, so the ranking primitive is forced to play a *second*, conflicting
/// categorical role in the same stylesheet. The functor then has one object
/// (`cascade_property`) mapped to two distinct role objects, which is not a
/// well-defined function on objects: `apply_cascade_role_mapping_functor_v0`
/// rejects the mapping and the gate surfaces
/// `categoricalCascadeEvidenceInconsistency`.
///
/// A stylesheet whose custom-property graph is acyclic maps every primitive to
/// exactly one canonical role, the functor accepts the mapping, and nothing is
/// surfaced. The diagnostic therefore depends on the functor verdict over the
/// actual reference graph, not on a literal: replacing the verdict with a
/// constant would either fire on every stylesheet or none.
fn summarize_query_categorical_cascade_evidence_diagnostics(
    source: &str,
    custom_properties: &[OmenaCheckerCustomPropertyInputV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(mapping) = query_categorical_role_mapping_for_cascade(custom_properties) else {
        return Vec::new();
    };

    let gate = run_omena_query_checker_categorical_gate_v0(OmenaCheckerCategoricalInputV0 {
        mappings: vec![mapping],
    });
    if !gate.enforcement_passed {
        return Vec::new();
    }

    let whole_file_range = parser_range_for_byte_span(
        source,
        ParserByteSpanV0 {
            start: 0,
            end: source.len(),
        },
    );

    gate.evaluations
        .into_iter()
        .map(|evaluation| {
            let mut provenance = vec![
                "omena-query-checker-orchestrator.categorical-gate",
                "omena-checker.categorical-rules",
                "omena-query.cascade-checker",
            ];
            provenance.extend(evaluation.mechanism_products.iter().copied());
            OmenaQueryStyleDiagnosticV0 {
                code: "categoricalCascadeEvidenceInconsistency",
                severity: "hint",
                provenance,
                range: whole_file_range,
                message:
                    "Cascade custom-property ranking forms a reference cycle, so the categorical \
                     cosheaf-colimit witness for the cascade-ranking primitive is not functorial: \
                     the ranking primitive plays conflicting categorical roles in this stylesheet."
                        .to_string(),
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            }
        })
        .collect()
}

/// Surface real SMT cascade proof-obligation diagnostics in the query style path.
///
/// The first obligation family is the canonical *box-shorthand combination*
/// obligation, built from a real parsed signal: when a single selector declares
/// the complete canonical longhand quartet of a known box shorthand (e.g.
/// `margin-top` … `margin-left`), combining those four longhands into the
/// `margin` shorthand is a cascade-sensitive rewrite. The obligation encodes the rewrite's
/// preconditions as `require:name=bool` literals derived from the parsed
/// declarations (supported shorthand, canonical top/right/bottom/left order, no
/// `!important` longhand, no empty value, adjacent source order). The gate runs
/// the genuine `evaluate_omena_checker_smt_rules` mechanism, which discharges the
/// conjunction through the active SMT backend. The default product build remains
/// solver-free and uses the propositional `StubSmtBackendV0`; opt-in `smt-z3`
/// builds route this same product gate through z3. A malformed quartet (e.g. an
/// `!important` longhand or a non-adjacent source order) makes the conjunction
/// `Unsat`, so the backend rejects the proof obligation and the gate surfaces
/// `cascade.smt-violation`. A well-formed quartet is `Sat` and nothing is
/// surfaced.
///
/// The second obligation family is the opt-in z3 `@layer` flatten-inversion lane.
/// It groups parsed declarations by `(selector, property)` and sends layered
/// competitors to `omena-smt`'s QF_LIA layer-ordering search. Default builds do
/// not emit this z3-only diagnostic; `smt-z3` builds surface it only when the
/// solver proves flattening layer boundaries would invert the winning
/// declaration.
///
/// The diagnostic therefore depends on the solver verdict over the parsed facts:
/// replacing the backend verdict with a constant would either fire on every
/// quartet or none, breaking the satisfiable/unsatisfiable split.
fn summarize_query_smt_cascade_obligation_diagnostics(
    source: &str,
    declarations: &[OmenaCheckerCascadeDeclarationInputV0],
    declaration_ranges: &BTreeMap<String, ParserRangeV0>,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let box_obligations = query_smt_box_shorthand_obligations(declarations);
    let layer_inversion_obligations = query_smt_layer_inversion_obligations(declarations);
    if box_obligations.is_empty() && layer_inversion_obligations.is_empty() {
        return Vec::new();
    }

    // Remember which declaration anchors each obligation so an emitted violation
    // can be ranged on the offending longhand rather than the whole file.
    let anchor_ranges = box_obligations
        .iter()
        .chain(layer_inversion_obligations.iter())
        .filter_map(|(obligation, anchor_declaration_id)| {
            declaration_ranges
                .get(anchor_declaration_id)
                .copied()
                .map(|range| (smt_obligation_id(obligation), range))
        })
        .collect::<BTreeMap<_, _>>();

    let box_gate = run_omena_query_checker_smt_gate_v0(OmenaCheckerSmtInputV0 {
        obligations: box_obligations
            .into_iter()
            .filter_map(|(obligation, _)| match obligation {
                QuerySmtCascadeObligation::BoxShorthand(obligation) => Some(obligation),
                QuerySmtCascadeObligation::LayerInversion(_) => None,
            })
            .collect(),
    });
    let layer_inversion_gate =
        run_omena_query_checker_smt_layer_inversion_gate_v0(OmenaCheckerSmtLayerInversionInputV0 {
            obligations: layer_inversion_obligations
                .into_iter()
                .filter_map(|(obligation, _)| match obligation {
                    QuerySmtCascadeObligation::BoxShorthand(_) => None,
                    QuerySmtCascadeObligation::LayerInversion(obligation) => Some(obligation),
                })
                .collect(),
        });
    if !box_gate.enforcement_passed || !layer_inversion_gate.enforcement_passed {
        return Vec::new();
    }

    let whole_file_range = parser_range_for_byte_span(
        source,
        ParserByteSpanV0 {
            start: 0,
            end: source.len(),
        },
    );

    box_gate
        .evaluations
        .into_iter()
        .chain(layer_inversion_gate.evaluations)
        .map(|evaluation| {
            let range = anchor_ranges
                .get(&evaluation.obligation_id)
                .copied()
                .unwrap_or(whole_file_range);
            let mut provenance = vec![
                query_smt_gate_provenance(evaluation.obligation_id.as_str()),
                "omena-checker.smt-rules",
                "omena-query.cascade-checker",
            ];
            provenance.extend(evaluation.mechanism_products.iter().copied());
            OmenaQueryStyleDiagnosticV0 {
                code: "cascadeSmtViolation",
                severity: "warning",
                provenance,
                range,
                message: query_smt_diagnostic_message(evaluation.obligation_id.as_str()),
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            }
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum QuerySmtCascadeObligation {
    BoxShorthand(OmenaCheckerSmtObligationInputV0),
    LayerInversion(OmenaCheckerSmtLayerInversionObligationInputV0),
}

fn smt_obligation_id(obligation: &QuerySmtCascadeObligation) -> String {
    match obligation {
        QuerySmtCascadeObligation::BoxShorthand(obligation) => obligation.obligation_id.clone(),
        QuerySmtCascadeObligation::LayerInversion(obligation) => obligation.obligation_id.clone(),
    }
}

fn query_smt_diagnostic_message(obligation_id: &str) -> String {
    if query_smt_is_layer_inversion_obligation(obligation_id) {
        return "Opt-in z3 SMT layer-flatten proof obligation found an @layer ordering inversion: flattening this layered cascade would change the winning declaration.".to_string();
    }
    "Box-shorthand combination proof obligation is unsatisfiable: these longhands cannot be safely combined into the shorthand without changing the cascade outcome.".to_string()
}

fn query_smt_gate_provenance(obligation_id: &str) -> &'static str {
    if query_smt_is_layer_inversion_obligation(obligation_id) {
        "omena-query-checker-orchestrator.smt-layer-inversion-gate"
    } else {
        "omena-query-checker-orchestrator.smt-gate"
    }
}

fn query_smt_is_layer_inversion_obligation(obligation_id: &str) -> bool {
    obligation_id.contains("layer-flatten-inversion")
}

/// Build the SMT box-shorthand combination obligations the parsed stylesheet
/// exercises.
///
/// A selector that declares the full canonical longhand quartet of a known box
/// shorthand is a flatten/combination candidate. The returned obligation encodes
/// the cascade-safety preconditions as `require:name=bool` literals mirroring
/// `omena-smt`'s `canonical_box_shorthand_combination_input_v0`, derived from the
/// actual parsed longhand declarations. Each obligation is paired with the
/// declaration id of the first longhand that *breaks* a precondition (or the
/// quartet's first longhand when every precondition holds) so an emitted
/// violation can be ranged precisely.
///
/// Returns an empty vector when the stylesheet declares no complete box-shorthand
/// quartet (no combination obligation to discharge).
fn query_smt_box_shorthand_obligations(
    declarations: &[OmenaCheckerCascadeDeclarationInputV0],
) -> Vec<(QuerySmtCascadeObligation, String)> {
    let mut obligations = Vec::new();

    let mut by_selector = BTreeMap::<&str, Vec<&OmenaCheckerCascadeDeclarationInputV0>>::new();
    for declaration in declarations {
        by_selector
            .entry(declaration.selector.as_str())
            .or_default()
            .push(declaration);
    }

    for (selector, selector_declarations) in by_selector {
        for (shorthand, expected_longhands) in query_smt_box_shorthand_longhand_quartets() {
            // Pick the first declaration of each expected longhand, preserving the
            // canonical top/right/bottom/left expectation order.
            let mut quartet = Vec::with_capacity(expected_longhands.len());
            for expected in expected_longhands {
                let Some(declaration) = selector_declarations
                    .iter()
                    .copied()
                    .find(|declaration| declaration.property == *expected)
                else {
                    break;
                };
                quartet.push(declaration);
            }
            if quartet.len() != expected_longhands.len() {
                // Selector does not declare the complete canonical quartet, so it
                // is not a combination candidate for this shorthand.
                continue;
            }

            let canonical_order = quartet
                .iter()
                .zip(expected_longhands.iter())
                .all(|(declaration, expected)| declaration.property == *expected);
            let no_important = quartet.iter().all(|declaration| !declaration.important);
            let no_empty_value = quartet
                .iter()
                .all(|declaration| !declaration.value.trim().is_empty());
            let adjacent_source_order = quartet
                .windows(2)
                .all(|pair| pair[1].source_order == pair[0].source_order + 1);

            let canonical_terms = vec![
                "require:supported-shorthand-property=true".to_string(),
                format!("require:canonical-longhand-quartet={canonical_order}"),
                format!("require:no-important-longhand={no_important}"),
                format!("require:no-empty-longhand-value={no_empty_value}"),
                format!("require:adjacent-source-order={adjacent_source_order}"),
            ];

            // Anchor on the first longhand that breaks a precondition so the
            // squiggle lands on the offending declaration; fall back to the
            // quartet's first longhand when nothing is broken.
            let anchor_declaration_id = quartet
                .iter()
                .find(|declaration| declaration.important || declaration.value.trim().is_empty())
                .map(|declaration| declaration.declaration_id.clone())
                .unwrap_or_else(|| quartet[0].declaration_id.clone());

            obligations.push((
                QuerySmtCascadeObligation::BoxShorthand(OmenaCheckerSmtObligationInputV0 {
                    obligation_id: format!(
                        "stylesheet://{selector}::{shorthand}-shorthand-combination"
                    ),
                    l1_primitive: "boxShorthandCombination".to_string(),
                    canonical_terms,
                }),
                anchor_declaration_id,
            ));
        }
    }

    obligations
}

fn query_smt_layer_inversion_obligations(
    declarations: &[OmenaCheckerCascadeDeclarationInputV0],
) -> Vec<(QuerySmtCascadeObligation, String)> {
    let mut obligations = Vec::new();
    let mut by_property = BTreeMap::<&str, Vec<&OmenaCheckerCascadeDeclarationInputV0>>::new();

    for declaration in declarations {
        if declaration.layer_order.is_some() {
            by_property
                .entry(declaration.property.as_str())
                .or_default()
                .push(declaration);
        }
    }

    for (property, competing_declarations) in by_property {
        for (left_index, left) in competing_declarations.iter().enumerate() {
            for right in competing_declarations.iter().skip(left_index + 1) {
                if left.layer_order == right.layer_order
                    || selector_co_match_verdict(left.selector.as_str(), right.selector.as_str())
                        == SelectorMatchVerdict::No
                {
                    continue;
                }

                let mut pair = vec![*left, *right];
                pair.sort_by_key(|declaration| declaration.source_order);
                let anchor_declaration_id = pair
                    .iter()
                    .find_map(|higher_layer| {
                        let higher_rank = higher_layer.layer_order?;
                        pair.iter()
                            .any(|lower_layer| {
                                lower_layer.layer_order.is_some_and(|lower_rank| {
                                    higher_rank > lower_rank
                                        && lower_layer.source_order > higher_layer.source_order
                                })
                            })
                            .then(|| higher_layer.declaration_id.clone())
                    })
                    .unwrap_or_else(|| pair[0].declaration_id.clone());
                let layer_declarations = pair
                    .into_iter()
                    .filter_map(|declaration| {
                        let layer_rank = declaration.layer_order?;
                        Some(OmenaCheckerSmtLayerInversionDeclarationInputV0 {
                            declaration_id: declaration.declaration_id.clone(),
                            layer_rank: i64::from(layer_rank),
                            source_order: i64::from(declaration.source_order),
                        })
                    })
                    .collect::<Vec<_>>();

                let selector_pair = [left.selector.as_str(), right.selector.as_str()]
                    .into_iter()
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join("~");
                obligations.push((
                    QuerySmtCascadeObligation::LayerInversion(
                        OmenaCheckerSmtLayerInversionObligationInputV0 {
                            obligation_id: format!(
                                "stylesheet://{selector_pair}::{property}-layer-flatten-inversion"
                            ),
                            declarations: layer_declarations,
                        },
                    ),
                    anchor_declaration_id,
                ));
            }
        }
    }

    obligations
}

/// The canonical top/right/bottom/left longhand quartets for the box shorthands
/// `omena-smt` proves combinable, mirroring `smt_box_shorthand_longhands_v0`.
pub(super) fn query_smt_box_shorthand_longhand_quartets() -> Vec<(&'static str, [&'static str; 4])>
{
    vec![
        (
            "margin",
            ["margin-top", "margin-right", "margin-bottom", "margin-left"],
        ),
        (
            "padding",
            [
                "padding-top",
                "padding-right",
                "padding-bottom",
                "padding-left",
            ],
        ),
        (
            "border-color",
            [
                "border-top-color",
                "border-right-color",
                "border-bottom-color",
                "border-left-color",
            ],
        ),
        (
            "border-style",
            [
                "border-top-style",
                "border-right-style",
                "border-bottom-style",
                "border-left-style",
            ],
        ),
        (
            "border-width",
            [
                "border-top-width",
                "border-right-width",
                "border-bottom-width",
                "border-left-width",
            ],
        ),
        (
            "scroll-margin",
            [
                "scroll-margin-top",
                "scroll-margin-right",
                "scroll-margin-bottom",
                "scroll-margin-left",
            ],
        ),
        (
            "scroll-padding",
            [
                "scroll-padding-top",
                "scroll-padding-right",
                "scroll-padding-bottom",
                "scroll-padding-left",
            ],
        ),
    ]
}

/// Build the cascade primitive-to-role mapping for the parsed stylesheet.
///
/// The baseline is the cascade engine's canonical primitive-to-role catalog: the
/// full repertoire of cascade primitives (ranking, layer/scope flattening,
/// shorthand combination, static `@supports` evaluation) that the stylesheet's
/// ranking participates in, each in its single canonical role. This baseline is
/// functorial, so a stylesheet whose custom-property ranking converges produces
/// an accepted verdict and no diagnostic.
///
/// When any declared custom property participates in a reference cycle, the
/// least-fixed-point ranking colimit cannot converge. The cascade-ranking
/// primitive can therefore no longer serve as its canonical cosheaf-colimit
/// witness, so it is given a conflicting second role. With the ranking primitive
/// now mapped to two distinct role objects, the functor object mapping is
/// many-valued, `apply_cascade_role_mapping_functor_v0` cannot witness
/// composition, and the verdict is rejected.
///
/// Returns `None` when the stylesheet declares no custom properties (no ranking
/// colimit obligation to witness).
fn query_categorical_role_mapping_for_cascade(
    custom_properties: &[OmenaCheckerCustomPropertyInputV0],
) -> Option<OmenaCheckerCategoricalRoleMappingInputV0> {
    if custom_properties.is_empty() {
        return None;
    }

    let declared = custom_properties
        .iter()
        .map(|property| property.name.as_str())
        .collect::<BTreeSet<_>>();
    let dependencies = custom_properties
        .iter()
        .map(|property| {
            (
                property.name.as_str(),
                property
                    .dependencies
                    .iter()
                    .map(String::as_str)
                    .filter(|dependency| declared.contains(dependency))
                    .collect::<BTreeSet<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let has_reference_cycle = declared
        .iter()
        .any(|name| query_custom_property_in_reference_cycle(name, &dependencies));

    let mut primitive_role_pairs = Vec::new();

    if has_reference_cycle {
        // Conflicting second role for the ranking primitive, placed before the
        // canonical catalog: a non-convergent ranking colimit cannot be the
        // cosheaf-colimit witness, so the functor object mapping becomes
        // many-valued and the verdict is rejected.
        primitive_role_pairs.push(OmenaCheckerCategoricalPrimitiveRolePairInputV0 {
            primitive_name: "cascade_property".to_string(),
            categorical_role: "cascade-ranking non-convergent witness".to_string(),
        });
    }

    primitive_role_pairs.extend(checker_cascade_primitive_role_catalog_v0().into_iter().map(
        |(primitive_name, categorical_role)| OmenaCheckerCategoricalPrimitiveRolePairInputV0 {
            primitive_name: primitive_name.to_string(),
            categorical_role: categorical_role.to_string(),
        },
    ));

    Some(OmenaCheckerCategoricalRoleMappingInputV0 {
        mapping_id: "stylesheet://cascade-primitive-role-evidence".to_string(),
        primitive_role_pairs,
    })
}

/// Derive the `(primitive_name, categorical_role)` pairs the parsed stylesheet's
/// custom-property ranking exercises, for the cascade-at-position categorical
/// evidence attachment. Shares the reference-cycle detection with the diagnostic
/// path so both observe the same functor verdict.
pub(super) fn query_exercised_cascade_primitive_role_pairs_from_source(
    source: &str,
) -> Vec<(String, String)> {
    let (checker_input, _, _) = collect_query_checker_cascade_input("query://categorical", source);
    match query_categorical_role_mapping_for_cascade(&checker_input.custom_properties) {
        Some(mapping) => mapping
            .primitive_role_pairs
            .into_iter()
            .map(|pair| (pair.primitive_name, pair.categorical_role))
            .collect(),
        None => Vec::new(),
    }
}

fn query_rg_flow_coupling_for_custom_properties(
    custom_properties: &[OmenaCheckerCustomPropertyInputV0],
) -> Option<OmenaCheckerRgFlowCouplingInputV0> {
    if custom_properties.is_empty() {
        return None;
    }

    let declared = custom_properties
        .iter()
        .map(|property| property.name.as_str())
        .collect::<BTreeSet<_>>();
    let dependencies = custom_properties
        .iter()
        .map(|property| {
            (
                property.name.as_str(),
                property
                    .dependencies
                    .iter()
                    .map(String::as_str)
                    .filter(|dependency| declared.contains(dependency))
                    .collect::<BTreeSet<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let k_env = custom_properties.len();
    let k_decl = custom_properties
        .iter()
        .filter(|property| {
            property
                .dependencies
                .iter()
                .any(|dependency| declared.contains(dependency.as_str()))
        })
        .count();
    let k_cycle = declared
        .iter()
        .filter(|name| query_custom_property_in_reference_cycle(name, &dependencies))
        .count();
    let k_dirty = custom_properties
        .iter()
        .filter(|property| property.guaranteed_invalid)
        .count();
    let acyclic_high_gain_pressure = if k_cycle == 0 {
        query_acyclic_high_gain_coupling_pressure(&dependencies)
    } else {
        0
    };
    let after_k_decl = k_decl.saturating_add(acyclic_high_gain_pressure);

    Some(OmenaCheckerRgFlowCouplingInputV0 {
        workspace_path: "stylesheet://custom-property-coupling".to_string(),
        before: OmenaCheckerRgFlowCouplingSpaceInputV0 {
            k_env,
            k_decl,
            k_cycle: 0,
            k_dirty: 0,
        },
        after: OmenaCheckerRgFlowCouplingSpaceInputV0 {
            k_env,
            k_decl: after_k_decl,
            k_cycle,
            k_dirty,
        },
    })
}

fn query_acyclic_high_gain_coupling_pressure(
    dependencies: &BTreeMap<&str, BTreeSet<&str>>,
) -> usize {
    let mut fanout_by_dependency = BTreeMap::<&str, usize>::new();
    for edges in dependencies.values() {
        for dependency in edges {
            *fanout_by_dependency.entry(*dependency).or_default() += 1;
        }
    }

    fanout_by_dependency
        .values()
        .filter(|count| **count >= 3)
        .map(|count| count.saturating_mul(count.saturating_sub(1)) / 2)
        .sum()
}

fn query_custom_property_in_reference_cycle(
    start: &str,
    dependencies: &BTreeMap<&str, BTreeSet<&str>>,
) -> bool {
    let mut stack = dependencies
        .get(start)
        .map(|edges| edges.iter().copied().collect::<Vec<_>>())
        .unwrap_or_default();
    let mut visited = BTreeSet::new();
    while let Some(node) = stack.pop() {
        if node == start {
            return true;
        }
        if !visited.insert(node) {
            continue;
        }
        if let Some(edges) = dependencies.get(node) {
            stack.extend(edges.iter().copied());
        }
    }
    false
}

fn query_cascade_checker_code(code: &'static str) -> &'static str {
    match code {
        "unreachable-declaration" => "unreachableDeclaration",
        "dead-cascade-layer" => "deadCascadeLayer",
        "iacvt-prone" => "iacvtProne",
        "circular-var" => "circularVar",
        "registered-property-type-mismatch" => "registeredPropertyTypeMismatch",
        "unspecified-cascade-tie" => "unspecifiedCascadeTie",
        "designer-intent-inconsistency" => "designerIntentInconsistency",
        _ => "cascadeAware",
    }
}

fn query_cascade_checker_diagnostic_severity(code: &'static str) -> &'static str {
    match code {
        "unreachable-declaration" | "dead-cascade-layer" | "designer-intent-inconsistency" => {
            "hint"
        }
        _ => "warning",
    }
}

fn query_cascade_checker_diagnostic_tags(code: &'static str) -> Vec<u8> {
    match code {
        "unreachable-declaration" | "dead-cascade-layer" => {
            vec![LSP_DIAGNOSTIC_TAG_UNNECESSARY]
        }
        _ => Vec::new(),
    }
}

fn summarize_query_cascade_narrowing_for_evaluation(
    evaluation: &OmenaCheckerCascadeEvaluationV0,
    declarations: &[OmenaCheckerCascadeDeclarationInputV0],
) -> Option<OmenaQueryCascadeNarrowingEvidenceV0> {
    let anchor_id = evaluation.declaration_ids.first()?;
    let anchor = declarations
        .iter()
        .find(|declaration| declaration.declaration_id == *anchor_id)?;
    let site_declarations = declarations
        .iter()
        .filter(|declaration| {
            declaration.selector == anchor.selector
                && declaration.property == anchor.property
                && declaration.condition_context == anchor.condition_context
        })
        .collect::<Vec<_>>();
    if site_declarations.is_empty() {
        return None;
    }

    let property_candidates = site_declarations
        .iter()
        .map(|declaration| AbstractPropertyValueCandidateV0 {
            property_name: declaration.property.clone(),
            value: declaration.value.clone(),
            pseudo_state: None,
            condition_context: declaration.condition_context.clone(),
            layer_name: declaration.layer_name.clone(),
            layer_order: declaration.layer_order,
            source_order: Some(declaration.source_order),
            important: declaration.important,
            same_selector_ordering: true,
        })
        .collect::<Vec<_>>();
    let property_value_narrowing = narrow_abstract_property_value_for_cascade_branch(
        anchor.property.as_str(),
        None,
        anchor.condition_context.as_slice(),
        anchor.layer_name.as_deref(),
        anchor.layer_order,
        true,
        property_candidates.as_slice(),
    );

    let selector_class_names = query_selector_class_names(anchor.selector.as_str());
    let element_class_constraints =
        query_element_class_signature_constraints(selector_class_names.as_slice());
    let element_class_iteration =
        iterate_reduced_class_value_product_constraints(element_class_constraints.as_slice());

    Some(OmenaQueryCascadeNarrowingEvidenceV0 {
        schema_version: "0",
        product: "omena-query.cascade-narrowing-evidence",
        selector: anchor.selector.as_str().to_string(),
        selector_class_names,
        property_name: anchor.property.clone(),
        condition_context: anchor.condition_context.clone(),
        declaration_ids: site_declarations
            .into_iter()
            .map(|declaration| declaration.declaration_id.clone())
            .collect(),
        element_class_iteration,
        property_value_narrowing,
        runtime_state: summarize_query_runtime_state_for_evaluation(evaluation, declarations),
    })
}

fn summarize_query_cascade_confidence_for_evaluation(
    evaluation: &OmenaCheckerCascadeEvaluationV0,
    declarations: &[OmenaCheckerCascadeDeclarationInputV0],
) -> Option<OmenaQueryCascadeConfidenceV0> {
    if !matches!(
        evaluation.rule_code_name,
        "unreachable-declaration" | "dead-cascade-layer"
    ) {
        return None;
    }
    let margin = query_cascade_margin_for_evaluation(evaluation, declarations)?;
    let abs_distance = margin.signed_distance.unsigned_abs();
    let dominant_axis_weight_basis_points =
        query_cascade_confidence_axis_weight_basis_points(margin.dominant_axis);
    let sigmoid_temperature_basis_points = 1_200u16;
    let confidence_score_basis_points = query_cascade_confidence_score_basis_points(
        abs_distance,
        dominant_axis_weight_basis_points,
        sigmoid_temperature_basis_points,
    );

    Some(OmenaQueryCascadeConfidenceV0 {
        schema_version: "0",
        product: "omena-query.cascade-confidence",
        feature_gate: "cascade-confidence-v0",
        confidence_kind: "fixtureWitnessTierWeightedSigmoid",
        claim_level: "fixtureWitnessResearchHint",
        theorem_claimed: false,
        public_safety_claim_ready: false,
        calibration_stage: "fixtureWitnessTierWeightSigmoidV0",
        margin_product: margin.product,
        margin_kind: margin.margin_kind,
        dominant_axis: margin.dominant_axis,
        dominant_axis_weight_basis_points,
        sigmoid_temperature_basis_points,
        signed_distance: margin.signed_distance,
        abs_distance,
        confidence_score_basis_points,
        confidence_bucket: query_cascade_confidence_bucket(confidence_score_basis_points),
        winner_declaration_id: margin.winner_declaration_id,
        challenger_declaration_id: margin.challenger_declaration_id,
    })
}

fn query_cascade_margin_for_evaluation(
    evaluation: &OmenaCheckerCascadeEvaluationV0,
    declarations: &[OmenaCheckerCascadeDeclarationInputV0],
) -> Option<omena_cascade::CascadeMarginV0> {
    let anchor_id = evaluation.declaration_ids.first()?;
    let anchor = declarations
        .iter()
        .find(|declaration| declaration.declaration_id == *anchor_id)?;
    let site_declarations = declarations
        .iter()
        .filter(|declaration| {
            declaration.selector == anchor.selector
                && declaration.property == anchor.property
                && declaration.condition_context == anchor.condition_context
        })
        .map(query_diagnostic_cascade_declaration_from_input)
        .collect::<Vec<_>>();
    if site_declarations.len() < 2 {
        return None;
    }

    let outcome = cascade_property(site_declarations, anchor.property.as_str());
    cascade_margin_for_outcome(&outcome)
}

fn query_diagnostic_cascade_declaration_from_input(
    input: &OmenaCheckerCascadeDeclarationInputV0,
) -> CascadeDeclaration {
    let mut declaration = query_runtime_cascade_declaration_from_input(input);
    declaration.id = input.declaration_id.clone();
    declaration
}

fn query_cascade_confidence_axis_weight_basis_points(axis: &str) -> u16 {
    match axis {
        "level" => 7_000,
        "layerRank" => 6_000,
        "scopeProximity" => 5_000,
        "specificityIds" => 4_000,
        "specificityClasses" => 3_000,
        "specificityElements" => 2_000,
        "sourceOrder" => 1_000,
        _ => 500,
    }
}

fn query_cascade_confidence_score_basis_points(
    abs_distance: u64,
    axis_weight_basis_points: u16,
    sigmoid_temperature_basis_points: u16,
) -> u16 {
    let signed_input = (abs_distance as f64 * f64::from(axis_weight_basis_points))
        / f64::from(sigmoid_temperature_basis_points);
    let confidence = 1.0 / (1.0 + (-signed_input).exp());
    (confidence * 10_000.0).round().clamp(0.0, 10_000.0) as u16
}

fn query_cascade_confidence_bucket(score_basis_points: u16) -> &'static str {
    match score_basis_points {
        0..=5_999 => "narrow",
        6_000..=8_499 => "moderate",
        _ => "clear",
    }
}

fn summarize_query_runtime_state_for_evaluation(
    evaluation: &OmenaCheckerCascadeEvaluationV0,
    declarations: &[OmenaCheckerCascadeDeclarationInputV0],
) -> Option<OmenaQueryRuntimeStateScenarioEvidenceV0> {
    let anchor_id = evaluation.declaration_ids.first()?;
    let anchor = declarations
        .iter()
        .find(|declaration| declaration.declaration_id == *anchor_id)?;
    let selector_class_names = query_selector_class_names(anchor.selector.as_str());
    let candidate_declarations = declarations
        .iter()
        .filter(|declaration| declaration.property == anchor.property)
        .filter(|declaration| {
            query_runtime_selector_matches_anchor_classes(
                anchor.selector.as_str(),
                declaration.selector.as_str(),
            )
        })
        .collect::<Vec<_>>();
    if candidate_declarations.is_empty() {
        return None;
    }

    let pseudo_states = query_runtime_candidate_pseudo_states(candidate_declarations.as_slice());
    let (condition_contexts, static_condition_pruning) = query_runtime_candidate_condition_contexts(
        candidate_declarations.as_slice(),
        anchor.condition_context.as_slice(),
    );
    let mut scenarios = Vec::new();

    for condition_context in &condition_contexts {
        scenarios.push(query_runtime_state_scenario(
            anchor.property.as_str(),
            None,
            condition_context.as_slice(),
            candidate_declarations.as_slice(),
        ));
        for pseudo_state in &pseudo_states {
            scenarios.push(query_runtime_state_scenario(
                anchor.property.as_str(),
                Some(pseudo_state.as_str()),
                condition_context.as_slice(),
                candidate_declarations.as_slice(),
            ));
        }
    }

    let pseudo_scenario_count = scenarios
        .iter()
        .filter(|scenario| scenario.pseudo_state.is_some())
        .count();
    let media_scenario_count = condition_contexts
        .iter()
        .filter(|context| !context.is_empty())
        .count();

    Some(OmenaQueryRuntimeStateScenarioEvidenceV0 {
        schema_version: "0",
        product: "omena-query.runtime-state-scenario-evidence",
        selector: anchor.selector.as_str().to_string(),
        selector_class_names,
        property_name: anchor.property.clone(),
        scenario_join_kind: "fixtureWitnessedScenarioJoin",
        confidence_tier: query_runtime_state_confidence_tier(scenarios.as_slice(), &[]),
        static_boundary: OmenaQueryRuntimeStateStaticBoundaryV0 {
            boundary_kind: "staticValueAssumingNoRuntimeOverride",
            static_value_assuming_no_runtime_override: true,
            tracks_dom_mutation: false,
            tracks_class_list_mutation: false,
        },
        driver_summaries: vec![
            OmenaQueryRuntimeStateDriverSummaryV0 {
                driver: "pseudoStateScenarioSweep",
                status: if pseudo_scenario_count == 0 {
                    "noRuntimePseudoStates"
                } else {
                    "fixtureWitnessed"
                },
                scenario_count: pseudo_scenario_count,
                provenance: vec![
                    "omena-cascade.selector-signature",
                    "omena-query.runtime-state-driver",
                ],
            },
            OmenaQueryRuntimeStateDriverSummaryV0 {
                driver: "inlineStyleHighestSpecificityTier",
                status: "awaitingSourceFacts",
                scenario_count: 0,
                provenance: vec![
                    "omena-bridge.source-syntax-index",
                    "omena-query.runtime-state-driver",
                ],
            },
            OmenaQueryRuntimeStateDriverSummaryV0 {
                driver: "mediaEnvironmentScenarioSweep",
                status: if media_scenario_count == 0 {
                    "noConditionalEnvironment"
                } else {
                    "fixtureWitnessed"
                },
                scenario_count: media_scenario_count,
                provenance: vec![
                    "omena-query.cascade-condition-context",
                    "omena-query.runtime-state-driver",
                ],
            },
            OmenaQueryRuntimeStateDriverSummaryV0 {
                driver: "staticRuntimeOverrideBoundary",
                status: "documentedAnalyticalBoundary",
                scenario_count: scenarios.len(),
                provenance: vec![
                    "omena-query.static-runtime-boundary",
                    "omena-query.runtime-state-driver",
                ],
            },
        ],
        scenarios,
        static_condition_pruning,
        inline_style_overrides: Vec::new(),
    })
}

pub(super) fn query_runtime_state_confidence_tier(
    scenarios: &[OmenaQueryRuntimeStateScenarioV0],
    inline_style_overrides: &[OmenaQueryInlineStyleRuntimeOverrideV0],
) -> &'static str {
    if !inline_style_overrides.is_empty()
        || scenarios.iter().any(|scenario| {
            scenario.pseudo_state.is_some()
                || !scenario.condition_context.is_empty()
                || scenario.scenario_kind == "inlineStyleOverride"
        })
    {
        "conditionalDefinite"
    } else {
        "staticDefinite"
    }
}

fn query_runtime_selector_matches_anchor_classes(
    anchor_selector: &str,
    candidate_selector: &str,
) -> bool {
    selector_co_match_verdict(anchor_selector, candidate_selector) != SelectorMatchVerdict::No
}

fn query_runtime_candidate_pseudo_states(
    declarations: &[&OmenaCheckerCascadeDeclarationInputV0],
) -> Vec<String> {
    declarations
        .iter()
        .filter_map(|declaration| parse_simple_selector_signature(declaration.selector.as_str()))
        .flat_map(|signature| signature.required_pseudo_states.into_iter())
        .filter(|pseudo_state| query_runtime_pseudo_state_is_dynamic(pseudo_state.as_str()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn query_runtime_pseudo_state_is_dynamic(pseudo_state: &str) -> bool {
    matches!(
        pseudo_state,
        "active"
            | "checked"
            | "disabled"
            | "enabled"
            | "focus"
            | "focus-visible"
            | "focus-within"
            | "hover"
            | "target"
            | "visited"
    )
}

fn query_runtime_candidate_condition_contexts(
    declarations: &[&OmenaCheckerCascadeDeclarationInputV0],
    anchor_condition_context: &[String],
) -> (
    Vec<Vec<String>>,
    Vec<OmenaQueryStaticConditionPruningEvidenceV0>,
) {
    let mut contexts = declarations
        .iter()
        .map(|declaration| declaration.condition_context.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if contexts.is_empty() {
        contexts.push(Vec::new());
    }
    let mut pruning = Vec::new();
    contexts.retain(|context| {
        let Some(evidence) = query_condition_context_static_supports_pruning_evidence(
            context.as_slice(),
            Some(anchor_condition_context),
        ) else {
            return true;
        };
        let keep = !evidence.pruned;
        pruning.push(evidence);
        keep
    });
    if contexts.is_empty() {
        contexts.push(Vec::new());
    }
    (contexts, pruning)
}

pub(crate) fn query_condition_context_static_supports_pruning_evidence(
    condition_context: &[String],
    anchor_condition_context: Option<&[String]>,
) -> Option<OmenaQueryStaticConditionPruningEvidenceV0> {
    let verdict = query_condition_context_static_supports_verdict(condition_context)?;
    if verdict != StaticSupportsEvalVerdictV0::AlwaysFalse {
        return None;
    }
    let anchor_context = anchor_condition_context.is_some_and(|anchor| anchor == condition_context);
    Some(OmenaQueryStaticConditionPruningEvidenceV0 {
        schema_version: "0",
        product: "omena-query.static-condition-pruning-evidence",
        condition_context: condition_context.to_vec(),
        assumption: "modernBrowser",
        verdict: query_static_supports_verdict_label(verdict),
        pruned: !anchor_context,
        anchor_context,
    })
}

fn query_condition_context_static_supports_verdict(
    condition_context: &[String],
) -> Option<StaticSupportsEvalVerdictV0> {
    let mut saw_supports = false;
    let mut saw_unknown = false;
    for entry in condition_context {
        let Some(condition) = query_supports_condition_from_context_entry(entry.as_str()) else {
            continue;
        };
        saw_supports = true;
        let witness = evaluate_static_supports_condition(
            condition,
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        match witness.verdict {
            StaticSupportsEvalVerdictV0::AlwaysFalse => {
                return Some(StaticSupportsEvalVerdictV0::AlwaysFalse);
            }
            StaticSupportsEvalVerdictV0::Unknown => {
                saw_unknown = true;
            }
            StaticSupportsEvalVerdictV0::AlwaysTrue => {}
        }
    }
    if !saw_supports {
        None
    } else if saw_unknown {
        Some(StaticSupportsEvalVerdictV0::Unknown)
    } else {
        Some(StaticSupportsEvalVerdictV0::AlwaysTrue)
    }
}

fn query_supports_condition_from_context_entry(entry: &str) -> Option<&str> {
    let trimmed = entry.trim_start();
    let prefix = "@supports";
    if trimmed.len() < prefix.len() {
        return None;
    }
    let (candidate_prefix, rest) = trimmed.split_at(prefix.len());
    if !candidate_prefix.eq_ignore_ascii_case(prefix) {
        return None;
    }
    if rest
        .chars()
        .next()
        .is_some_and(|ch| !ch.is_whitespace() && ch != '(')
    {
        return None;
    }
    let condition = rest.trim_start();
    if condition.is_empty() {
        None
    } else {
        Some(condition)
    }
}

fn query_static_supports_verdict_label(verdict: StaticSupportsEvalVerdictV0) -> &'static str {
    match verdict {
        StaticSupportsEvalVerdictV0::AlwaysTrue => "AlwaysTrue",
        StaticSupportsEvalVerdictV0::AlwaysFalse => "AlwaysFalse",
        StaticSupportsEvalVerdictV0::Unknown => "Unknown",
    }
}

fn query_runtime_state_scenario(
    property_name: &str,
    pseudo_state: Option<&str>,
    condition_context: &[String],
    declarations: &[&OmenaCheckerCascadeDeclarationInputV0],
) -> OmenaQueryRuntimeStateScenarioV0 {
    let active_declarations = declarations
        .iter()
        .copied()
        .filter(|declaration| declaration.condition_context == condition_context)
        .filter(|declaration| {
            query_runtime_selector_active_for_pseudo_state(declaration, pseudo_state)
        })
        .collect::<Vec<_>>();
    let property_candidates = active_declarations
        .iter()
        .map(|declaration| AbstractPropertyValueCandidateV0 {
            property_name: declaration.property.clone(),
            value: declaration.value.clone(),
            pseudo_state: query_runtime_declaration_primary_pseudo_state(declaration),
            condition_context: declaration.condition_context.clone(),
            layer_name: declaration.layer_name.clone(),
            layer_order: declaration.layer_order,
            source_order: Some(declaration.source_order),
            important: declaration.important,
            same_selector_ordering: false,
        })
        .collect::<Vec<_>>();
    let property_value_narrowing = narrow_abstract_property_value_for_cascade_branch(
        property_name,
        pseudo_state,
        condition_context,
        None,
        None,
        false,
        property_candidates.as_slice(),
    );
    let outcome = if active_declarations.is_empty() {
        CascadeOutcome::Top
    } else {
        cascade_property(
            active_declarations
                .iter()
                .map(|declaration| query_runtime_cascade_declaration_from_input(declaration))
                .collect::<Vec<_>>(),
            property_name,
        )
    };
    let (winner_declaration_id, winner_value) = match outcome {
        CascadeOutcome::Definite { winner, .. } => {
            let value = match winner.value {
                CascadeValue::Literal(value) => Some(value),
                _ => None,
            };
            (Some(winner.id), value)
        }
        _ => (None, None),
    };

    OmenaQueryRuntimeStateScenarioV0 {
        scenario_kind: if condition_context.is_empty() {
            "pseudoState"
        } else {
            "mediaEnvironment"
        },
        pseudo_state: pseudo_state.map(str::to_string),
        condition_context: condition_context.to_vec(),
        declaration_ids: active_declarations
            .into_iter()
            .map(|declaration| declaration.declaration_id.clone())
            .collect(),
        winner_declaration_id,
        winner_value,
        property_value_narrowing,
    }
}

fn query_runtime_selector_active_for_pseudo_state(
    declaration: &OmenaCheckerCascadeDeclarationInputV0,
    pseudo_state: Option<&str>,
) -> bool {
    let Some(signature) = parse_simple_selector_signature(declaration.selector.as_str()) else {
        return declaration
            .selector
            .as_str()
            .split(':')
            .nth(1)
            .is_none_or(|required| Some(required) == pseudo_state);
    };
    let required = signature
        .required_pseudo_states
        .into_iter()
        .filter(|state| query_runtime_pseudo_state_is_dynamic(state.as_str()))
        .collect::<BTreeSet<_>>();
    match pseudo_state {
        Some(pseudo_state) => required.is_empty() || required.contains(pseudo_state),
        None => required.is_empty(),
    }
}

fn query_runtime_declaration_primary_pseudo_state(
    declaration: &OmenaCheckerCascadeDeclarationInputV0,
) -> Option<String> {
    parse_simple_selector_signature(declaration.selector.as_str())?
        .required_pseudo_states
        .into_iter()
        .find(|state| query_runtime_pseudo_state_is_dynamic(state.as_str()))
}

fn query_runtime_cascade_declaration_from_input(
    input: &OmenaCheckerCascadeDeclarationInputV0,
) -> CascadeDeclaration {
    let level = if input.important {
        CascadeLevel::AuthorImportant
    } else {
        CascadeLevel::AuthorNormal
    };
    let layer_rank = LayerRank(input.layer_order.unwrap_or(0));
    let specificity = parse_simple_selector_signature(input.selector.as_str())
        .map(|signature| signature.specificity)
        .unwrap_or(Specificity::ZERO);
    let value = input.value.trim().to_string();

    CascadeDeclaration {
        id: input.declaration_id.clone(),
        property: input.property.clone(),
        value: CascadeValue::Literal(value),
        key: CascadeKey::new(level, layer_rank, 0, specificity, input.source_order),
    }
}

fn query_element_class_signature_constraints(
    selector_class_names: &[String],
) -> Vec<AbstractClassValueV0> {
    if selector_class_names.is_empty() {
        return Vec::new();
    }

    let first = selector_class_names.first().cloned().unwrap_or_default();
    let last = selector_class_names.last().cloned().unwrap_or_default();
    let signature_min_length = selector_class_names
        .iter()
        .map(String::len)
        .sum::<usize>()
        .saturating_add(selector_class_names.len().saturating_sub(1));

    vec![prefix_suffix_class_value(
        first,
        last,
        Some(signature_min_length),
        None,
    )]
}

fn query_selector_class_names(selector: &str) -> Vec<String> {
    let bytes = selector.as_bytes();
    let mut index = 0usize;
    let mut names = BTreeSet::new();
    while index < bytes.len() {
        if bytes[index] != b'.' {
            index += 1;
            continue;
        }
        let start = index + 1;
        let mut end = start;
        while end < bytes.len() && query_selector_class_name_byte(bytes[end]) {
            end += 1;
        }
        if end > start {
            names.insert(selector[start..end].to_string());
            index = end;
        } else {
            index += 1;
        }
    }
    names.into_iter().collect()
}

fn query_selector_class_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

fn collect_query_checker_custom_property_registrations(
    style_uri: &str,
    source: &str,
) -> Vec<OmenaCheckerCustomPropertyRegistrationInputV0> {
    let dialect = omena_parser_dialect_for_style_path(style_uri);
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut registrations = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::AtKeyword
            && tokens[index].text.eq_ignore_ascii_case("@property")
            && let Some((registration, next_index)) =
                parse_query_checker_custom_property_registration(source, tokens, index)
        {
            registrations.push(registration);
            index = next_index;
            continue;
        }
        index += 1;
    }
    registrations
}

fn parse_query_checker_custom_property_registration(
    source: &str,
    tokens: &[LexedToken],
    at_property_index: usize,
) -> Option<(OmenaCheckerCustomPropertyRegistrationInputV0, usize)> {
    let name_index = skip_query_trivia(tokens, at_property_index + 1, tokens.len());
    let name = normalize_query_checker_custom_property_name(tokens.get(name_index)?.text.as_str())?;
    let block_start_index = find_query_registration_block_start(tokens, name_index + 1)?;
    let block_end_index = matching_query_registration_block_end(tokens, block_start_index)?;
    let declarations = collect_query_checker_registration_declarations(
        source,
        tokens,
        block_start_index,
        block_end_index,
    );

    Some((
        OmenaCheckerCustomPropertyRegistrationInputV0 {
            name,
            syntax: declarations.get("syntax").cloned(),
            inherits: declarations.get("inherits").cloned(),
            initial_value: declarations.get("initial-value").cloned(),
        },
        block_end_index + 1,
    ))
}

fn collect_query_checker_registration_declarations(
    source: &str,
    tokens: &[LexedToken],
    block_start_index: usize,
    block_end_index: usize,
) -> BTreeMap<String, String> {
    let mut declarations = BTreeMap::new();
    let mut index = block_start_index + 1;
    while index < block_end_index {
        index = skip_query_trivia(tokens, index, block_end_index);
        while index < block_end_index && query_registration_statement_ends(tokens[index].kind) {
            index = skip_query_trivia(tokens, index + 1, block_end_index);
        }
        if index >= block_end_index {
            break;
        }
        let property_index = index;
        let Some(colon_index) =
            find_query_registration_colon(tokens, property_index, block_end_index)
        else {
            break;
        };
        let property = source
            [token_start(tokens, property_index)..token_start(tokens, colon_index)]
            .trim()
            .to_ascii_lowercase();
        let value_start_index = skip_query_trivia(tokens, colon_index + 1, block_end_index);
        let value_end_index =
            find_query_registration_statement_end(tokens, value_start_index, block_end_index);
        if value_start_index < value_end_index {
            let raw_value = source
                [token_start(tokens, value_start_index)..token_end(tokens, value_end_index - 1)]
                .trim();
            let (value, important) = strip_query_registration_important(raw_value);
            if !important && !property.is_empty() {
                declarations.insert(property, value.to_string());
            }
        }
        index = value_end_index.saturating_add(1);
    }
    declarations
}

fn skip_query_trivia(tokens: &[LexedToken], mut index: usize, end: usize) -> usize {
    while index < end && tokens[index].kind.is_trivia() {
        index += 1;
    }
    index
}

fn normalize_query_checker_custom_property_name(text: &str) -> Option<String> {
    let name = text.trim();
    (name.starts_with("--") && name.len() > 2).then(|| name.to_string())
}

fn find_query_registration_block_start(tokens: &[LexedToken], index: usize) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .skip(index)
        .find_map(|(candidate_index, token)| {
            (token.kind == SyntaxKind::LeftBrace).then_some(candidate_index)
        })
}

fn matching_query_registration_block_end(
    tokens: &[LexedToken],
    block_start_index: usize,
) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(block_start_index) {
        match token.kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_query_registration_colon(tokens: &[LexedToken], start: usize, end: usize) -> Option<usize> {
    for (index, token) in tokens.iter().enumerate().take(end).skip(start) {
        match token.kind {
            SyntaxKind::Colon => return Some(index),
            SyntaxKind::RightBrace => return None,
            kind if query_registration_statement_ends(kind) => return None,
            _ => {}
        }
    }
    None
}

fn find_query_registration_statement_end(tokens: &[LexedToken], start: usize, end: usize) -> usize {
    tokens
        .iter()
        .enumerate()
        .take(end)
        .skip(start)
        .find_map(|(index, token)| {
            (query_registration_statement_ends(token.kind) || token.kind == SyntaxKind::RightBrace)
                .then_some(index)
        })
        .unwrap_or(end)
}

fn query_registration_statement_ends(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon
    )
}

fn strip_query_registration_important(value: &str) -> (&str, bool) {
    let compact = value
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    if !compact.ends_with("!important") {
        return (value, false);
    }
    let Some(bang_index) = value.rfind('!') else {
        return (value, false);
    };
    (value[..bang_index].trim_end(), true)
}

fn token_start(tokens: &[LexedToken], index: usize) -> usize {
    u32::from(tokens[index].range.start()) as usize
}

fn token_end(tokens: &[LexedToken], index: usize) -> usize {
    u32::from(tokens[index].range.end()) as usize
}

fn collect_query_checker_cascade_input(
    style_uri: &str,
    source: &str,
) -> (
    OmenaCheckerCascadeInputV0,
    BTreeMap<String, ParserRangeV0>,
    BTreeMap<String, ParserRangeV0>,
) {
    let declarations = collect_query_checker_cascade_declarations(source);
    let custom_property_registrations =
        collect_query_checker_custom_property_registrations(style_uri, source);
    let declaration_ranges = declarations
        .iter()
        .map(|declaration| {
            (
                declaration.input.declaration_id.clone(),
                parser_range_for_byte_span(source, declaration.byte_span),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let dialect = omena_parser_dialect_for_style_path(style_uri);
    let guaranteed_invalid_custom_properties =
        summarize_static_css_custom_property_fixed_point_from_source(source, dialect)
            .entries
            .into_iter()
            .filter(|entry| entry.guaranteed_invalid)
            .map(|entry| entry.name)
            .collect::<BTreeSet<_>>();
    let mut custom_properties_by_name =
        BTreeMap::<String, (BTreeSet<String>, bool, ParserByteSpanV0)>::new();

    for declaration in &declarations {
        if !declaration.input.property.starts_with("--") {
            continue;
        }
        let entry = custom_properties_by_name
            .entry(declaration.input.property.clone())
            .or_insert_with(|| {
                (
                    BTreeSet::new(),
                    guaranteed_invalid_custom_properties.contains(&declaration.input.property),
                    declaration.byte_span,
                )
            });
        entry.1 |= guaranteed_invalid_custom_properties.contains(&declaration.input.property);
        for dependency in collect_query_var_references_in_value(&declaration.input.value) {
            entry.0.insert(dependency);
        }
    }

    let custom_property_ranges = custom_properties_by_name
        .iter()
        .map(|(name, (_, _, byte_span))| {
            (name.clone(), parser_range_for_byte_span(source, *byte_span))
        })
        .collect::<BTreeMap<_, _>>();
    let custom_properties = custom_properties_by_name
        .into_iter()
        .map(
            |(name, (dependencies, guaranteed_invalid, _))| OmenaCheckerCustomPropertyInputV0 {
                name,
                dependencies: dependencies.into_iter().collect(),
                guaranteed_invalid,
            },
        )
        .collect::<Vec<_>>();

    (
        OmenaCheckerCascadeInputV0 {
            declarations: declarations
                .into_iter()
                .map(|declaration| declaration.input)
                .collect(),
            custom_properties,
            custom_property_registrations,
        },
        declaration_ranges,
        custom_property_ranges,
    )
}

/// Compute the REAL per-`(selector, property)` cascade winners a parsed
/// stylesheet produces, projected as replica-ensemble site outcomes.
///
/// This is the cross-file replica-ensemble's per-file input: each in-graph CSS
/// module is one replica, and its "site outcomes" are the winning value of every
/// `(selector, property)` it declares. The winners are not literals — they are
/// computed by the genuine `cascade_property` ranking over the parsed
/// declarations: declarations are grouped by `(selector, property)`, each is
/// assigned a real `CascadeKey` (author-important vs author-normal level, real
/// `@layer` rank, real selector specificity from `parse_simple_selector_signature`,
/// and source order), and the lexicographic cascade key picks the winner. The
/// winning declaration's value is carried as the outcome identity (via the
/// `CascadeDeclaration.id`), so two replicas *agree* on a site iff their winning
/// value for that `(selector, property)` is identical, and *disagree* iff their
/// cascades resolve to different values. No replica snapshot is fabricated.
///
/// Custom-property declarations (`--*`) are excluded: their winner is a token
/// whose meaning depends on the whole-graph fixed point, not a directly
/// comparable per-file value.
pub(super) fn collect_query_replica_ensemble_site_outcomes(
    source: &str,
) -> Vec<ReplicaSiteOutcomeV0> {
    let declarations = collect_query_checker_cascade_declarations(source);

    // Group the parsed declarations by their `(selector, property)` cascade site.
    let mut by_site: BTreeMap<(String, String), Vec<CascadeDeclaration>> = BTreeMap::new();
    for declaration in &declarations {
        let property = declaration.input.property.as_str();
        if property.starts_with("--") {
            continue;
        }
        let cascade_declaration = query_cascade_declaration_from_input(&declaration.input);
        by_site
            .entry((
                declaration.input.selector.as_str().to_string(),
                declaration.input.property.clone(),
            ))
            .or_default()
            .push(cascade_declaration);
    }

    by_site
        .into_iter()
        .filter_map(|((selector, property), site_declarations)| {
            let outcome = cascade_property(site_declarations, &property);
            // Only definite winners are comparable across replicas; an
            // `Inherit`/`Top`/`RankedSet` site carries no concrete per-file value to
            // overlap on, and `DefiniteOnly` projection would drop it anyway.
            if !matches!(outcome, omena_cascade::CascadeOutcome::Definite { .. }) {
                return None;
            }
            Some(ReplicaSiteOutcomeV0 {
                schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
                product: "omena-ensemble.replica-site-outcome",
                layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
                feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
                site: replica_ensemble_site(selector, property),
                outcome,
                provenance: None,
            })
        })
        .collect()
}

/// Lift a parsed cascade declaration onto an `omena-cascade` `CascadeDeclaration`
/// with a real cascade key. The winning value is carried as the declaration `id`
/// so the replica-overlap projection (`definite:<id>`) keys agreement on the
/// resolved value rather than a per-file synthetic identifier.
fn query_cascade_declaration_from_input(
    input: &OmenaCheckerCascadeDeclarationInputV0,
) -> CascadeDeclaration {
    let level = if input.important {
        CascadeLevel::AuthorImportant
    } else {
        CascadeLevel::AuthorNormal
    };
    let layer_rank = LayerRank(input.layer_order.unwrap_or(0));
    let specificity = parse_simple_selector_signature(input.selector.as_str())
        .map(|signature| signature.specificity)
        .unwrap_or(Specificity::ZERO);
    let value = input.value.trim().to_string();

    CascadeDeclaration {
        id: value.clone(),
        property: input.property.clone(),
        value: CascadeValue::Literal(value),
        key: CascadeKey::new(level, layer_rank, 0, specificity, input.source_order),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct QueryCheckerCascadeDeclaration {
    pub(super) input: OmenaCheckerCascadeDeclarationInputV0,
    pub(super) byte_span: ParserByteSpanV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QueryCheckerCascadeScope {
    condition_context: Vec<String>,
    layer_name: Option<String>,
    layer_order: Option<i32>,
}

/// Test-only per-thread counter of cascade-declaration collections, so the
/// substrate-backed narrowing paths can assert "zero re-collections per candidate"
/// (rfcs#63 E-ii) instead of assuming it.
#[cfg(test)]
pub(crate) mod cascade_declarations_collect_probe {
    use std::cell::Cell;

    thread_local! {
        static COLLECT_CALLS: Cell<usize> = const { Cell::new(0) };
    }

    pub(crate) fn reset() {
        COLLECT_CALLS.with(|calls| calls.set(0));
    }

    pub(crate) fn count() -> usize {
        COLLECT_CALLS.with(Cell::get)
    }

    pub(super) fn record() {
        COLLECT_CALLS.with(|calls| calls.set(calls.get() + 1));
    }
}

pub(super) fn collect_query_checker_cascade_declarations(
    source: &str,
) -> Vec<QueryCheckerCascadeDeclaration> {
    #[cfg(test)]
    cascade_declarations_collect_probe::record();
    let mut declarations = Vec::new();
    let mut layer_orders = BTreeMap::new();
    let mut next_layer_order = 0i32;
    collect_query_checker_cascade_blocks(
        source,
        0,
        source.len(),
        None,
        Vec::new(),
        None,
        None,
        &mut layer_orders,
        &mut next_layer_order,
        &mut declarations,
    );
    declarations
}

#[allow(clippy::too_many_arguments)]
fn collect_query_checker_cascade_blocks(
    source: &str,
    start: usize,
    end: usize,
    parent_selector: Option<String>,
    condition_context: Vec<String>,
    layer_name: Option<String>,
    layer_order: Option<i32>,
    layer_orders: &mut BTreeMap<String, i32>,
    next_layer_order: &mut i32,
    declarations: &mut Vec<QueryCheckerCascadeDeclaration>,
) {
    let mut index = start;
    while let Some(open_index) = find_query_top_level_byte(source, index, end, b'{') {
        let Some(close_index) = matching_query_block_end(source, open_index, end) else {
            break;
        };
        let prelude_start = query_prelude_start(source, start, open_index);
        let prelude = source[prelude_start..open_index].trim();
        let body_start = open_index + 1;

        if let Some(layer) = query_layer_name_from_prelude(prelude) {
            let order = *layer_orders.entry(layer.clone()).or_insert_with(|| {
                let order = *next_layer_order;
                *next_layer_order += 1;
                order
            });
            collect_query_checker_cascade_blocks(
                source,
                body_start,
                close_index,
                parent_selector.clone(),
                condition_context.clone(),
                Some(layer),
                Some(order),
                layer_orders,
                next_layer_order,
                declarations,
            );
        } else if let Some(at_root_selector) = query_at_root_selector_from_prelude(prelude) {
            // RFC-0007-E4 (#45): `@at-root <selector> { … }` resets the cascade context to the
            // document root and applies `<selector>` as the new context — the inner block's
            // declarations belong to `<selector>`, NOT to the enclosing parent. The walker
            // previously hit the generic `@`-rule arm below, which kept `parent_selector` and
            // recursed without ever recording the block's declarations against a selector, so a
            // nested `@at-root .b { … }` was dropped from cascade analysis (the bare
            // `@at-root { … }` block form already worked because its inner `.b` is a plain
            // selector rule). We re-root by clearing `parent_selector` (root context) and treating
            // the trailing selector list exactly like an ordinary nested rule: emit its direct
            // declarations and recurse for any further nesting.
            let mut canonical_members = Vec::new();
            for member in split_query_selector_list(&at_root_selector) {
                let canonical_selector = canonical_query_checker_selector(None, &member);
                if !canonical_members.contains(&canonical_selector) {
                    canonical_members.push(canonical_selector);
                }
            }

            for canonical_selector in canonical_members {
                collect_query_checker_direct_declarations(
                    source,
                    body_start,
                    close_index,
                    &canonical_selector,
                    QueryCheckerCascadeScope {
                        condition_context: condition_context.clone(),
                        layer_name: layer_name.clone(),
                        layer_order,
                    },
                    declarations,
                );
                collect_query_checker_cascade_blocks(
                    source,
                    body_start,
                    close_index,
                    Some(canonical_selector),
                    condition_context.clone(),
                    layer_name.clone(),
                    layer_order,
                    layer_orders,
                    next_layer_order,
                    declarations,
                );
            }
        } else if prelude.starts_with('@') {
            let mut nested_condition_context = condition_context.clone();
            nested_condition_context.push(normalize_query_condition_prelude(prelude));
            collect_query_checker_cascade_blocks(
                source,
                body_start,
                close_index,
                parent_selector.clone(),
                nested_condition_context,
                layer_name.clone(),
                layer_order,
                layer_orders,
                next_layer_order,
                declarations,
            );
        } else if !prelude.is_empty() {
            // A selector list (`.a, .b { … }`) records one declaration set per
            // member so each member can tie with a sibling rule on the same
            // selector (RFC-0007 B2). Identical canonical members within one
            // prelude are de-duplicated to avoid a spurious self-tie.
            let mut canonical_members = Vec::new();
            for member in split_query_selector_list(prelude) {
                let canonical_selector =
                    canonical_query_checker_selector(parent_selector.as_deref(), &member);
                if !canonical_members.contains(&canonical_selector) {
                    canonical_members.push(canonical_selector);
                }
            }

            for canonical_selector in canonical_members {
                collect_query_checker_direct_declarations(
                    source,
                    body_start,
                    close_index,
                    &canonical_selector,
                    QueryCheckerCascadeScope {
                        condition_context: condition_context.clone(),
                        layer_name: layer_name.clone(),
                        layer_order,
                    },
                    declarations,
                );
                collect_query_checker_cascade_blocks(
                    source,
                    body_start,
                    close_index,
                    Some(canonical_selector),
                    condition_context.clone(),
                    layer_name.clone(),
                    layer_order,
                    layer_orders,
                    next_layer_order,
                    declarations,
                );
            }
        }

        index = close_index + 1;
    }
}

/// Splits a selector-list prelude on top-level commas, ignoring commas nested
/// inside `()` (e.g. `:is(.a, .b)`), `[]`, or string literals (RFC-0007 B2).
/// Returns one entry per member; a prelude with no top-level comma returns a
/// single-element vector containing the whole (trimmed) prelude.
fn split_query_selector_list(prelude: &str) -> Vec<String> {
    let mut members = Vec::new();
    let mut segment_start = 0usize;
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    while index < prelude.len() {
        let Some(ch) = prelude[index..].chars().next() else {
            break;
        };
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = prelude[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ',' if paren_depth == 0 && bracket_depth == 0 => {
                let member = prelude[segment_start..index].trim();
                if !member.is_empty() {
                    members.push(member.to_string());
                }
                segment_start = index + ch.len_utf8();
            }
            _ => {}
        }
        index += ch.len_utf8();
    }

    let tail = prelude[segment_start..].trim();
    if !tail.is_empty() {
        members.push(tail.to_string());
    }
    if members.is_empty() {
        members.push(prelude.trim().to_string());
    }
    members
}

fn canonical_query_checker_selector(parent_selector: Option<&str>, selector: &str) -> String {
    let selector = selector.trim();
    match parent_selector {
        Some(parent_selector) => expand_css_nested_selector(parent_selector, selector)
            .unwrap_or_else(|| fallback_expand_query_nested_selector(parent_selector, selector)),
        None => selector.to_string(),
    }
}

fn fallback_expand_query_nested_selector(parent_selector: &str, selector: &str) -> String {
    if selector.contains('&') {
        selector.replace('&', parent_selector)
    } else {
        format!("{parent_selector} {selector}")
    }
}

fn collect_query_checker_direct_declarations(
    source: &str,
    body_start: usize,
    body_end: usize,
    selector: &str,
    scope: QueryCheckerCascadeScope,
    declarations: &mut Vec<QueryCheckerCascadeDeclaration>,
) {
    let mut statement_start = body_start;
    let mut index = body_start;
    while index < body_end {
        if let Some(open_index) = find_query_top_level_byte(source, index, body_end, b'{') {
            while let Some(semicolon_index) =
                find_query_top_level_byte(source, index, open_index, b';')
            {
                push_query_checker_declaration(
                    source,
                    statement_start,
                    semicolon_index,
                    selector,
                    &scope,
                    declarations,
                );
                statement_start = semicolon_index + 1;
                index = statement_start;
            }
            let Some(close_index) = matching_query_block_end(source, open_index, body_end) else {
                break;
            };
            statement_start = close_index + 1;
            index = statement_start;
            continue;
        }

        while let Some(semicolon_index) = find_query_top_level_byte(source, index, body_end, b';') {
            push_query_checker_declaration(
                source,
                statement_start,
                semicolon_index,
                selector,
                &scope,
                declarations,
            );
            statement_start = semicolon_index + 1;
            index = statement_start;
        }
        break;
    }

    push_query_checker_declaration(
        source,
        statement_start,
        body_end,
        selector,
        &scope,
        declarations,
    );
}

fn push_query_checker_declaration(
    source: &str,
    start: usize,
    end: usize,
    selector: &str,
    scope: &QueryCheckerCascadeScope,
    declarations: &mut Vec<QueryCheckerCascadeDeclaration>,
) {
    let Some((trimmed_start, trimmed_end)) = trimmed_query_span(source, start, end) else {
        return;
    };
    let raw_statement = &source[trimmed_start..trimmed_end];
    // Strip CSS/Sass comments before the property/value split. A leading
    // `/* */` block (or a `//` line comment) that precedes the property name
    // otherwise poisons the property string (e.g. `/* primary */ color`), so the
    // whitespace guard below rejects it and the declaration is silently dropped
    // from cascade analysis (RFC-0007 B1).
    let statement = strip_query_statement_comments(raw_statement);
    let statement = statement.as_str();
    let Some(colon_offset) = find_query_top_level_colon(statement) else {
        return;
    };
    let property = statement[..colon_offset].trim();
    if property.is_empty()
        || property.starts_with('@')
        // Sass `$`-variable assignments are compile-time bindings that are erased
        // before CSS emission, so they never participate in the cascade. Skip
        // them here (symmetric to the `--custom-property` cascade path, which
        // does belong in the cascade) so re-binding a `$`-var is not mistaken for
        // a duplicate CSS declaration / cascade tie.
        || property.starts_with('$')
        || property.contains(char::is_whitespace)
        || property.contains('{')
        || property.contains('}')
    {
        return;
    }
    let mut value = statement[colon_offset + 1..].trim().to_string();
    let important = query_value_has_important_suffix(&value);
    if important {
        value = value
            .trim_end()
            .trim_end_matches(|ch: char| ch.is_ascii_whitespace())
            .trim_end_matches("!important")
            .trim_end()
            .to_string();
    }
    let source_order = declarations.len();
    let declaration_id = format!("decl-{source_order}");
    declarations.push(QueryCheckerCascadeDeclaration {
        input: OmenaCheckerCascadeDeclarationInputV0 {
            declaration_id,
            selector: CanonicalSelector::from_canonical(selector),
            property: property.to_string(),
            value: value.clone(),
            source_order: source_order.min(u32::MAX as usize) as u32,
            condition_context: scope.condition_context.clone(),
            layer_name: scope.layer_name.clone(),
            layer_order: scope.layer_order,
            important,
            var_references: collect_query_var_references_in_value(&value),
        },
        byte_span: ParserByteSpanV0 {
            start: trimmed_start,
            end: trimmed_end,
        },
    });
}

fn query_value_has_important_suffix(value: &str) -> bool {
    value
        .trim_end()
        .to_ascii_lowercase()
        .ends_with("!important")
}

fn trimmed_query_span(source: &str, start: usize, end: usize) -> Option<(usize, usize)> {
    let mut trimmed_start = start;
    let mut trimmed_end = end;
    while trimmed_start < trimmed_end
        && source[trimmed_start..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
    {
        trimmed_start += source[trimmed_start..].chars().next()?.len_utf8();
    }
    while trimmed_end > trimmed_start
        && source[..trimmed_end]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace)
    {
        trimmed_end -= source[..trimmed_end].chars().next_back()?.len_utf8();
    }
    (trimmed_start < trimmed_end).then_some((trimmed_start, trimmed_end))
}

/// Removes CSS/Sass comments from a single declaration statement, quote-aware.
///
/// `/* ... */` block comments are elided entirely; `//` line comments are
/// truncated to the end of their line (Sass semantics). Comment delimiters
/// inside string literals are preserved. The result is used for the
/// property/value split so a comment positioned before a property name no
/// longer poisons it (RFC-0007 B1).
fn strip_query_statement_comments(statement: &str) -> String {
    let mut out = String::with_capacity(statement.len());
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut paren_depth = 0usize;
    while index < statement.len() {
        let Some(ch) = statement[index..].chars().next() else {
            break;
        };
        if let Some(quote_ch) = quote {
            out.push(ch);
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = statement[index..].chars().next() {
                    out.push(escaped);
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        if statement[index..].starts_with("/*") {
            match statement[index + 2..].find("*/") {
                Some(close_offset) => {
                    // Replace the comment with a single space so adjacent tokens
                    // (e.g. `color/* x */:`) do not get glued together.
                    out.push(' ');
                    index += close_offset + 4;
                }
                // Unterminated block comment: drop the remainder.
                None => break,
            }
            continue;
        }
        // A `//` outside parentheses is a Sass line comment; inside parentheses
        // (e.g. `url(http://example.com)`) it is part of a value and must be
        // preserved, otherwise the value is corrupted into an unbalanced token.
        if paren_depth == 0 && statement[index..].starts_with("//") {
            // Sass line comment: skip to the next newline (or end of statement).
            match statement[index..].find('\n') {
                Some(newline_offset) => {
                    out.push('\n');
                    index += newline_offset + 1;
                }
                None => break,
            }
            continue;
        }
        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            _ => {}
        }
        out.push(ch);
        index += ch.len_utf8();
    }
    out
}

fn find_query_top_level_colon(statement: &str) -> Option<usize> {
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut paren_depth = 0usize;

    while index < statement.len() {
        let ch = statement[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = statement[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            '(' => {
                paren_depth += 1;
                index += ch.len_utf8();
            }
            ')' => {
                paren_depth = paren_depth.saturating_sub(1);
                index += ch.len_utf8();
            }
            ':' if paren_depth == 0 => return Some(index),
            _ => index += ch.len_utf8(),
        }
    }
    None
}

fn query_prelude_start(source: &str, search_start: usize, open_index: usize) -> usize {
    source[search_start..open_index]
        .rfind(['{', '}', ';'])
        .map(|offset| search_start + offset + 1)
        .unwrap_or(search_start)
}

fn query_layer_name_from_prelude(prelude: &str) -> Option<String> {
    let rest = prelude.trim_start().strip_prefix("@layer")?.trim();
    let name = rest
        .split(|ch: char| ch.is_ascii_whitespace() || matches!(ch, ',' | '{' | ';'))
        .next()
        .unwrap_or_default()
        .trim_matches(['"', '\'']);
    if name.is_empty() {
        Some("(anonymous-layer)".to_string())
    } else {
        Some(name.to_string())
    }
}

fn normalize_query_condition_prelude(prelude: &str) -> String {
    prelude.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// RFC-0007-E4 (#45): recognize the `@at-root <selector>` form and return the trailing selector
/// list. Returns `None` for the bare block form (`@at-root { … }`, no selector) — which already
/// works via the generic selector recursion — and for the `@at-root (with: …) <selector>` /
/// `@at-root (without: …) <selector>` query forms, whose leading `(...)` clause we do not yet
/// model; those keep falling through to the generic at-rule handling rather than risk mis-rooting.
/// Any other at-rule returns `None`.
fn query_at_root_selector_from_prelude(prelude: &str) -> Option<String> {
    let rest = prelude.trim_start().strip_prefix("@at-root")?;
    // Require a boundary after the keyword so `@at-rootish` never matches.
    if let Some(next) = rest.chars().next()
        && !next.is_ascii_whitespace()
    {
        return None;
    }
    let selector = rest.trim();
    // Bare block form (no selector) or the `(with:/without:)` query form: defer to generic handling.
    if selector.is_empty() || selector.starts_with('(') {
        return None;
    }
    Some(selector.to_string())
}

fn collect_query_var_references_in_value(value: &str) -> Vec<String> {
    let mut refs = BTreeSet::new();
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = value[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if query_function_name_starts_at(value, index, "var") => {
                let open_index = index + "var".len();
                let Some(close_index) = matching_query_paren_end(value, open_index, value.len())
                else {
                    index += ch.len_utf8();
                    continue;
                };
                collect_query_var_references_from_arguments(
                    &value[open_index + 1..close_index],
                    &mut refs,
                );
                index = close_index + 1;
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }
    refs.into_iter().collect()
}

fn collect_query_var_references_from_arguments(arguments: &str, refs: &mut BTreeSet<String>) {
    let parts = split_query_top_level_arguments(arguments);
    let Some(first_argument) = parts.first().map(|part| part.trim()) else {
        return;
    };
    if first_argument.starts_with("--") {
        refs.insert(first_argument.to_string());
    }
    for fallback in parts.iter().skip(1) {
        for reference in collect_query_var_references_in_value(fallback) {
            refs.insert(reference);
        }
    }
}

fn split_query_top_level_arguments(arguments: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut paren_depth = 0usize;

    while index < arguments.len() {
        let Some(ch) = arguments[index..].chars().next() else {
            break;
        };
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = arguments[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            '(' => {
                paren_depth += 1;
                index += ch.len_utf8();
            }
            ')' => {
                paren_depth = paren_depth.saturating_sub(1);
                index += ch.len_utf8();
            }
            ',' if paren_depth == 0 => {
                parts.push(&arguments[start..index]);
                index += ch.len_utf8();
                start = index;
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }
    parts.push(&arguments[start..]);
    parts
}

fn query_function_name_starts_at(value: &str, index: usize, function_name: &str) -> bool {
    value
        .get(index..index + function_name.len())
        .is_some_and(|name| name.eq_ignore_ascii_case(function_name))
        && value[index + function_name.len()..].starts_with('(')
}

fn find_query_top_level_byte(source: &str, start: usize, end: usize, needle: u8) -> Option<usize> {
    let mut index = start;
    let mut quote: Option<char> = None;
    let mut paren_depth = 0usize;
    while index < end {
        let ch = source[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = source[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        if source[index..].starts_with("/*")
            && let Some(close_offset) = source[index + 2..end].find("*/")
        {
            index += close_offset + 4;
            continue;
        }
        // Sass `//` line comments outside parentheses are not declaration
        // boundaries, so a `;` (or `{`) buried in one must not be treated as a
        // statement delimiter (RFC-0007 B1). Inside parens (`url(http://…)`) the
        // `//` is part of a value, so it is left intact.
        if paren_depth == 0 && source[index..end].starts_with("//") {
            match source[index..end].find('\n') {
                Some(newline_offset) => {
                    index += newline_offset + 1;
                    continue;
                }
                None => return None,
            }
        }
        // Match the requested delimiter exactly as before (paren-unaware) so the
        // existing statement-boundary behavior is unchanged; `paren_depth` is
        // tracked only to gate the `//` line-comment skip above.
        if ch.len_utf8() == 1 && source.as_bytes()[index] == needle {
            return Some(index);
        }
        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            '(' => {
                paren_depth += 1;
                index += ch.len_utf8();
            }
            ')' => {
                paren_depth = paren_depth.saturating_sub(1);
                index += ch.len_utf8();
            }
            _ => index += ch.len_utf8(),
        }
    }
    None
}

fn matching_query_block_end(source: &str, open_index: usize, end: usize) -> Option<usize> {
    matching_query_delimiter_end(source, open_index, end, b'{', b'}')
}

fn matching_query_paren_end(source: &str, open_index: usize, end: usize) -> Option<usize> {
    matching_query_delimiter_end(source, open_index, end, b'(', b')')
}

fn matching_query_delimiter_end(
    source: &str,
    open_index: usize,
    end: usize,
    open: u8,
    close: u8,
) -> Option<usize> {
    if source.as_bytes().get(open_index).copied()? != open {
        return None;
    }
    let mut index = open_index + 1;
    let mut depth = 1usize;
    let mut quote: Option<char> = None;
    // Only gate `//` line-comment skipping for brace matching, where a `}` in a
    // comment would otherwise close the block early. A `//` inside a value's
    // parentheses (`url(http://…)`) is part of the value, so it must be left
    // intact — track an inner paren depth to distinguish the two.
    let track_line_comments = open == b'{';
    let mut inner_paren_depth = 0usize;

    while index < end {
        let ch = source[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = source[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        if source[index..].starts_with("/*")
            && let Some(close_offset) = source[index + 2..end].find("*/")
        {
            index += close_offset + 4;
            continue;
        }
        // Sass `//` line comment: skip to the next newline so a `}` (RFC-0007 B1)
        // buried in a comment does not close the block prematurely. Restricted to
        // brace matching, and only outside value parentheses.
        if track_line_comments && inner_paren_depth == 0 && source[index..end].starts_with("//") {
            match source[index..end].find('\n') {
                Some(newline_offset) => {
                    index += newline_offset + 1;
                    continue;
                }
                None => return None,
            }
        }
        if track_line_comments {
            match ch {
                '(' => inner_paren_depth += 1,
                ')' => inner_paren_depth = inner_paren_depth.saturating_sub(1),
                _ => {}
            }
        }
        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if ch.len_utf8() == 1 && source.as_bytes()[index] == open => {
                depth += 1;
                index += 1;
            }
            _ if ch.len_utf8() == 1 && source.as_bytes()[index] == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
                index += 1;
            }
            _ => index += ch.len_utf8(),
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recorded(source: &str) -> Vec<(String, String, String)> {
        collect_query_checker_cascade_declarations(source)
            .into_iter()
            .map(|declaration| {
                (
                    declaration.input.selector.into_string(),
                    declaration.input.property,
                    declaration.input.value,
                )
            })
            .collect()
    }

    fn diagnostic_codes(source: &str) -> Vec<&'static str> {
        summarize_query_cascade_checker_diagnostics_with_deep_analysis(
            "file:///tmp/test.scss",
            source,
            false,
        )
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
    }

    fn diagnostic_codes_with_deep_analysis(source: &str, deep_analysis: bool) -> Vec<&'static str> {
        summarize_query_cascade_checker_diagnostics_with_deep_analysis(
            "file:///tmp/test.scss",
            source,
            deep_analysis,
        )
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
    }

    fn cascade_codes(source: &str) -> Vec<&'static str> {
        diagnostic_codes(source)
            .into_iter()
            .filter(|code| matches!(*code, "unreachableDeclaration" | "unspecifiedCascadeTie"))
            .collect()
    }

    fn layered_declaration(
        declaration_id: &str,
        selector: &str,
        property: &str,
        source_order: u32,
        layer_order: i32,
    ) -> OmenaCheckerCascadeDeclarationInputV0 {
        OmenaCheckerCascadeDeclarationInputV0 {
            declaration_id: declaration_id.to_string(),
            selector: CanonicalSelector::from_canonical(selector),
            property: property.to_string(),
            value: "red".to_string(),
            source_order,
            condition_context: Vec::new(),
            layer_name: Some(format!("layer-{layer_order}")),
            layer_order: Some(layer_order),
            important: false,
            var_references: Vec::new(),
        }
    }

    // ---- B1: comment poisoning ----------------------------------------

    #[test]
    fn b1_block_comment_before_property_does_not_drop_declaration() {
        let recorded = recorded(".a { /* primary */ color: red; color: blue; }");
        let properties: Vec<_> = recorded.iter().map(|(_, property, _)| property).collect();
        assert_eq!(properties, vec!["color", "color"], "{recorded:?}");
    }

    #[test]
    fn b1_block_comment_repro_fires_tie_and_unreachable() {
        let cascade = cascade_codes(".a { /* primary */ color: red; color: blue; }");
        assert!(
            cascade.contains(&"unreachableDeclaration")
                && cascade.contains(&"unspecifiedCascadeTie"),
            "expected both cascade diagnostics, got {cascade:?}"
        );
    }

    #[test]
    fn b1_line_comment_before_declarations_does_not_drop_them() {
        let cascade = cascade_codes(".a { // primary\ncolor: red; color: blue; }");
        assert!(
            cascade.contains(&"unreachableDeclaration")
                && cascade.contains(&"unspecifiedCascadeTie"),
            "expected both cascade diagnostics, got {cascade:?}"
        );
    }

    #[test]
    fn b1_value_comment_is_stripped_but_property_survives() {
        let recorded = recorded(".a { color /* c */ : red /* d */; }");
        assert_eq!(
            recorded,
            vec![(".a".to_string(), "color".to_string(), "red".to_string())],
            "comment-laden declaration should still record cleanly"
        );
    }

    // ---- B1 over-correction: commented-out declarations stay inert -----

    #[test]
    fn b1_line_commented_out_declaration_is_not_analyzed_as_live() {
        // The override is commented out, so there is no live duplicate / tie.
        let cascade = cascade_codes(".a { color: red; // color: blue;\n}");
        assert!(
            cascade.is_empty(),
            "commented-out decl must not tie: {cascade:?}"
        );
    }

    #[test]
    fn b1_block_commented_out_declaration_is_not_analyzed_as_live() {
        let cascade = cascade_codes(".a { color: red; /* color: blue; */ }");
        assert!(
            cascade.is_empty(),
            "commented-out decl must not tie: {cascade:?}"
        );
    }

    #[test]
    fn b1_url_with_double_slash_value_is_preserved_and_later_tie_still_fires() {
        // The `//` inside `url(http://…)` must not be treated as a line comment,
        // and the genuine `color` duplicate that follows must still tie.
        let source = ".a { background: url(http://example.com/a.png); color: red; color: blue; }";
        let recorded = recorded(source);
        assert!(
            recorded
                .iter()
                .any(|(_, property, value)| property == "background"
                    && value == "url(http://example.com/a.png)"),
            "url value should survive intact: {recorded:?}"
        );
        let cascade = cascade_codes(source);
        assert!(
            cascade.contains(&"unspecifiedCascadeTie"),
            "later real tie should still fire: {cascade:?}"
        );
    }

    // ---- B2: selector-list cross-rule tie -----------------------------

    #[test]
    fn b2_selector_list_member_records_separately() {
        let recorded = recorded(".a, .b { color: red; }");
        let selectors: Vec<_> = recorded.iter().map(|(selector, ..)| selector).collect();
        assert_eq!(selectors, vec![".a", ".b"], "{recorded:?}");
    }

    #[test]
    fn b2_selector_list_member_ties_with_sibling_rule() {
        let cascade = cascade_codes(".a, .b { color: red; }\n.a { color: blue; }");
        assert!(
            cascade.contains(&"unreachableDeclaration")
                && cascade.contains(&"unspecifiedCascadeTie"),
            "list member .a should tie with .a sibling: {cascade:?}"
        );
    }

    // ---- B2 over-correction: no spurious ties -------------------------

    #[test]
    fn b2_distinct_list_member_does_not_tie_with_unrelated_rule() {
        // `.a, .b` vs `.c` share no selector, so no tie may be reported.
        let cascade = cascade_codes(".a, .b { color: red; }\n.c { color: blue; }");
        assert!(
            cascade.is_empty(),
            "unrelated rule must not tie: {cascade:?}"
        );
    }

    #[test]
    fn b2_duplicate_member_in_one_prelude_is_deduplicated() {
        // `.a, .a` is a single rule; the duplicated member must not self-tie.
        let recorded = recorded(".a, .a { color: red; }");
        assert_eq!(
            recorded.len(),
            1,
            "identical members must be de-duplicated: {recorded:?}"
        );
        let cascade = cascade_codes(".a, .a { color: red; }");
        assert!(
            cascade.is_empty(),
            "deduped member must not self-tie: {cascade:?}"
        );
    }

    #[test]
    fn b2_comma_inside_functional_pseudo_is_not_split() {
        // The comma inside `:is(.a, .b)` is paren-protected, so the rule records
        // as a single opaque-compound selector rather than two bogus members.
        let recorded = recorded(":is(.a, .b) { color: red; }");
        let selectors: Vec<_> = recorded.iter().map(|(selector, ..)| selector).collect();
        assert_eq!(selectors, vec![":is(.a, .b)"], "{recorded:?}");
    }

    #[test]
    fn runtime_selector_filter_uses_conservative_co_match_axes() {
        assert!(query_runtime_selector_matches_anchor_classes(
            ".btn",
            "button.btn"
        ));
        assert!(query_runtime_selector_matches_anchor_classes(
            ".btn",
            ".btn.active"
        ));
        assert!(query_runtime_selector_matches_anchor_classes(
            ".btn:is(.active)",
            ".btn .icon"
        ));
        assert!(!query_runtime_selector_matches_anchor_classes(
            "div.btn", "span.btn"
        ));
        assert!(!query_runtime_selector_matches_anchor_classes(
            "#save", "#cancel"
        ));
    }

    #[test]
    fn layer_inversion_obligations_group_property_equal_co_matching_selectors_pairwise() {
        let declarations = vec![
            layered_declaration("base", ".btn", "color", 20, 0),
            layered_declaration("theme", "button.btn", "color", 10, 1),
            layered_declaration("other-property", "button.btn", "background", 30, 2),
        ];

        let obligations = query_smt_layer_inversion_obligations(&declarations);

        assert_eq!(obligations.len(), 1, "{obligations:?}");
        let layer_obligations = obligations
            .iter()
            .filter_map(|(obligation, _)| match obligation {
                QuerySmtCascadeObligation::LayerInversion(obligation) => Some(obligation),
                QuerySmtCascadeObligation::BoxShorthand(_) => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            layer_obligations.len(),
            1,
            "expected layer inversion obligation: {obligations:?}"
        );
        let obligation = layer_obligations[0];
        assert_eq!(
            obligation
                .declarations
                .iter()
                .map(|declaration| declaration.declaration_id.as_str())
                .collect::<Vec<_>>(),
            vec!["theme", "base"]
        );
    }

    #[test]
    fn layer_inversion_obligations_skip_disjoint_single_valued_axes() {
        let declarations = vec![
            layered_declaration("button", "button.btn", "color", 10, 0),
            layered_declaration("anchor", "a.btn", "color", 20, 1),
        ];

        let obligations = query_smt_layer_inversion_obligations(&declarations);

        assert!(
            obligations.is_empty(),
            "conflicting required tags must not compete: {obligations:?}"
        );
    }

    #[test]
    fn layer_inversion_obligations_keep_maybe_co_matches_competing() {
        let declarations = vec![
            layered_declaration("base", ".btn .icon", "color", 20, 0),
            layered_declaration("theme", ".btn:is(.active)", "color", 10, 1),
        ];

        let obligations = query_smt_layer_inversion_obligations(&declarations);

        assert_eq!(
            obligations.len(),
            1,
            "unsupported selector structure must stay possibly competing: {obligations:?}"
        );
    }

    // ---- WP7-b: de-noise rg-flow + categorical theory hints -----------

    /// A two-property custom-property reference cycle that the product chain
    /// flags as `circularVar`.
    const VAR_CYCLE_SOURCE: &str = ":root { --a: var(--b); --b: var(--a); }";

    #[test]
    fn wp7b_var_cycle_still_fires_circular_var_warning() {
        // Over-correction guard: the product `circularVar` warning must keep
        // firing on a real custom-property reference cycle regardless of the
        // deep-analysis flag — the dedup removes only the theory hints.
        for deep_analysis in [false, true] {
            let codes = diagnostic_codes_with_deep_analysis(VAR_CYCLE_SOURCE, deep_analysis);
            assert!(
                codes.contains(&"circularVar"),
                "circularVar must still fire on a real var cycle (deep_analysis={deep_analysis}): {codes:?}"
            );
        }
    }

    #[test]
    fn wp7b_default_surface_var_cycle_emits_only_circular_var() {
        // Default surface (deep-analysis OFF): a lone var cycle yields exactly the
        // `circularVar` warning and no whole-file-ranged theory hints.
        let codes = diagnostic_codes(VAR_CYCLE_SOURCE);
        assert!(
            codes.contains(&"circularVar"),
            "circularVar must fire on the default surface: {codes:?}"
        );
        assert!(
            !codes.contains(&"rgFlowRelevantOperator"),
            "rg-flow theory hint must be OFF by default: {codes:?}"
        );
        assert!(
            !codes.contains(&"categoricalCascadeEvidenceInconsistency"),
            "categorical theory hint must be OFF by default: {codes:?}"
        );
        // No theory triple-fire: the cycle yields the product `circularVar`
        // warning (and any other product cascade diagnostics) but neither of the
        // two redundant, whole-file-ranged theory hints.
        assert!(
            codes.iter().all(|code| !matches!(
                *code,
                "rgFlowRelevantOperator" | "categoricalCascadeEvidenceInconsistency"
            )),
            "default surface must surface no theory hints for a lone var cycle: {codes:?}"
        );
    }

    #[test]
    fn wp7b_deep_analysis_dedups_theory_hints_into_circular_var() -> Result<(), &'static str> {
        // Deep-analysis ON: the rg-flow + categorical hints key off the same
        // reference-cycle predicate as `circularVar`, so they are deduplicated
        // (folded into `circularVar`'s provenance) rather than triple-firing.
        let codes = diagnostic_codes_with_deep_analysis(VAR_CYCLE_SOURCE, true);
        assert!(
            codes.contains(&"circularVar"),
            "circularVar must fire with deep analysis ON: {codes:?}"
        );
        assert!(
            !codes.contains(&"rgFlowRelevantOperator"),
            "rg-flow hint must be deduplicated against circularVar: {codes:?}"
        );
        assert!(
            !codes.contains(&"categoricalCascadeEvidenceInconsistency"),
            "categorical hint must be deduplicated against circularVar: {codes:?}"
        );

        // The suppressed theory mechanisms' provenance is merged into the
        // surviving `circularVar` diagnostic so the audit trail is preserved.
        let diagnostics = summarize_query_cascade_checker_diagnostics_with_deep_analysis(
            "file:///tmp/test.scss",
            VAR_CYCLE_SOURCE,
            true,
        );
        let circular_var = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "circularVar")
            .ok_or("circularVar diagnostic must exist")?;
        assert!(
            circular_var
                .provenance
                .iter()
                .any(|label| label.contains("rg-flow")),
            "rg-flow provenance should be folded into circularVar: {:?}",
            circular_var.provenance
        );
        assert!(
            circular_var
                .provenance
                .iter()
                .any(|label| label.contains("categorical")),
            "categorical provenance should be folded into circularVar: {:?}",
            circular_var.provenance
        );
        Ok(())
    }

    #[test]
    fn wp7b_deep_analysis_reaches_theory_gate_on_cyclic_input() {
        // With deep-analysis ON the theory producers are reachable (the gate runs)
        // even though their output is deduplicated here: the rg-flow coupling and
        // categorical mapping are both populated for a cyclic stylesheet, so the
        // underlying mechanisms still execute (proving the opt-in path is live).
        let (checker_input, _, _) =
            collect_query_checker_cascade_input("file:///tmp/test.scss", VAR_CYCLE_SOURCE);
        let rg_flow = summarize_query_rg_flow_coupling_diagnostics(
            VAR_CYCLE_SOURCE,
            &checker_input.custom_properties,
        );
        let categorical = summarize_query_categorical_cascade_evidence_diagnostics(
            VAR_CYCLE_SOURCE,
            &checker_input.custom_properties,
        );
        assert!(
            !rg_flow.is_empty(),
            "rg-flow theory gate should fire on a cyclic stylesheet when reached"
        );
        assert!(
            !categorical.is_empty(),
            "categorical theory gate should fire on a cyclic stylesheet when reached"
        );
    }

    #[test]
    fn wp7b_acyclic_stylesheet_emits_no_theory_hints_even_with_deep_analysis() {
        // Over-correction guard (the other direction): an acyclic custom-property
        // graph must not spuriously surface a theory hint under deep analysis.
        let acyclic = ":root { --a: 1px; --b: var(--a); }";
        let codes = diagnostic_codes_with_deep_analysis(acyclic, true);
        assert!(
            !codes.contains(&"rgFlowRelevantOperator")
                && !codes.contains(&"categoricalCascadeEvidenceInconsistency"),
            "acyclic stylesheet must not surface theory hints: {codes:?}"
        );
    }

    #[test]
    fn wp7b_acyclic_high_gain_hub_surfaces_standalone_rg_flow_hint() {
        let high_gain = r#"
:root {
  --seed: 1px;
  --a: var(--seed);
  --b: var(--seed);
  --c: var(--seed);
  --d: var(--seed);
}
"#;

        let default_codes = diagnostic_codes_with_deep_analysis(high_gain, false);
        assert!(
            !default_codes.contains(&"rgFlowRelevantOperator"),
            "rg-flow theory hint must stay off on the default surface: {default_codes:?}"
        );

        let deep_codes = diagnostic_codes_with_deep_analysis(high_gain, true);
        assert!(
            deep_codes.contains(&"rgFlowRelevantOperator"),
            "acyclic high-gain hub should surface a standalone rg-flow hint: {deep_codes:?}"
        );
        assert!(
            !deep_codes.contains(&"circularVar"),
            "standalone rg-flow hint must not depend on circularVar: {deep_codes:?}"
        );
    }
}

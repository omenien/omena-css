use std::collections::{BTreeMap, BTreeSet};

use omena_cascade::{SelectorMatchVerdict, selector_co_match_verdict};
use omena_query_checker_orchestrator::{
    OmenaCheckerCascadeDeclarationInputV0, OmenaCheckerSmtInputV0,
    OmenaCheckerSmtLayerInversionDeclarationInputV0, OmenaCheckerSmtLayerInversionInputV0,
    OmenaCheckerSmtLayerInversionObligationInputV0, OmenaCheckerSmtObligationInputV0,
    run_omena_query_checker_smt_gate_v0, run_omena_query_checker_smt_layer_inversion_gate_v0,
};

use super::{
    OmenaQueryStyleDiagnosticV0, ParserByteSpanV0, ParserRangeV0, parser_range_for_byte_span,
};

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
/// solver-free and uses the product-owned propositional backend; opt-in
/// `smt-z3` builds route this same product gate through z3. A malformed quartet (e.g. an
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
pub(super) fn summarize_query_smt_cascade_obligation_diagnostics(
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
pub(super) enum QuerySmtCascadeObligation {
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

pub(super) fn query_smt_layer_inversion_obligations(
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
pub(crate) fn query_smt_box_shorthand_longhand_quartets() -> Vec<(&'static str, [&'static str; 4])>
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

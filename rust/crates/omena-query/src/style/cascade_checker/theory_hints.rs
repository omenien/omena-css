use std::collections::{BTreeMap, BTreeSet};

use omena_query_checker_orchestrator::{
    OmenaCheckerCategoricalInputV0, OmenaCheckerCategoricalPrimitiveRolePairInputV0,
    OmenaCheckerCategoricalRoleMappingInputV0, OmenaCheckerCustomPropertyInputV0,
    OmenaCheckerRgFlowCouplingInputV0, OmenaCheckerRgFlowCouplingSpaceInputV0,
    OmenaCheckerRgFlowInputV0, checker_cascade_primitive_role_catalog_v0,
    run_omena_query_checker_categorical_gate_v0, run_omena_query_checker_rg_flow_gate_v0,
};

use super::super::{
    OmenaQueryStyleDiagnosticV0, ParserByteSpanV0, ParserRangeV0, parser_range_for_byte_span,
};
use super::collect_query_checker_cascade_input;

/// Default surfaces already emit a concrete product `circularVar` warning on
/// cyclic custom-property declarations. Deep-analysis theory hints that cover
/// the same cycle are folded into that diagnostic's provenance instead of
/// producing duplicate squiggles.
pub(super) fn deduplicate_query_theory_hints_against_circular_var(
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

fn query_theory_hint_range_is_whole_file(range: &ParserRangeV0) -> bool {
    range.start.line == 0 && range.start.character == 0
}

pub(super) fn summarize_query_rg_flow_coupling_diagnostics(
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

pub(super) fn summarize_query_categorical_cascade_evidence_diagnostics(
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

pub(crate) fn query_exercised_cascade_primitive_role_pairs_from_source(
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

//! Refinement entry points layered above the byte-stable cascade proof module.

use omena_refinement_trait::{
    RefinementVerdictV0, RefinementWitnessV0, refinement_provenance_v0, refinement_witness_v0,
};

use crate::{
    CascadeDeclaration, CascadeLevel, LayerFlattenInputV0, ScopeFlattenInputV0,
    StaticSupportsAssumptionV0, StaticSupportsEvalVerdictV0, evaluate_static_supports_condition,
    prove_layer_flatten_candidate, prove_scope_flatten_candidate,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CascadeRefinementContextV0 {
    pub supports_condition: Option<String>,
    pub scope_root_selector: Option<String>,
    pub layer_name: Option<String>,
    pub closed_bundle: bool,
}

impl Default for CascadeRefinementContextV0 {
    fn default() -> Self {
        Self {
            supports_condition: None,
            scope_root_selector: None,
            layer_name: None,
            closed_bundle: true,
        }
    }
}

pub fn refine_declaration_in_context(
    declaration: &CascadeDeclaration,
    context: &CascadeRefinementContextV0,
) -> RefinementWitnessV0 {
    let mut provenances = Vec::new();
    let mut verdicts = Vec::new();

    if let Some(condition) = context.supports_condition.as_deref() {
        let supports = evaluate_static_supports_condition(
            condition,
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        provenances.push(refinement_provenance_v0(
            "supports-predicate",
            Some("evaluate_static_supports_condition"),
        ));
        verdicts.push(match supports.verdict {
            StaticSupportsEvalVerdictV0::AlwaysTrue => RefinementVerdictV0::SatisfiedAll,
            StaticSupportsEvalVerdictV0::AlwaysFalse => RefinementVerdictV0::Unsatisfiable,
            StaticSupportsEvalVerdictV0::Unknown => RefinementVerdictV0::Unknown,
        });
    }

    if let Some(root_selector) = context.scope_root_selector.as_deref() {
        let scope = prove_scope_flatten_candidate(ScopeFlattenInputV0 {
            root_selector: root_selector.to_string(),
            limit_selector: None,
            scoped_rule_count: 1,
            peer_scope_count: 0,
            competing_unscoped_rule_count: 0,
            inside_layer: context.layer_name.is_some(),
        });
        provenances.push(refinement_provenance_v0(
            "scope-predicate",
            Some("prove_scope_flatten_candidate"),
        ));
        verdicts.push(if scope.accepted {
            RefinementVerdictV0::SatisfiedAll
        } else {
            RefinementVerdictV0::Unknown
        });
    }

    if context.layer_name.is_some() {
        let layer = prove_layer_flatten_candidate(LayerFlattenInputV0 {
            layer_name: context.layer_name.clone(),
            layer_rule_count: 1,
            peer_layer_count: 0,
            unlayered_rule_count: 0,
            important_declaration_count: usize::from(matches!(
                declaration.key.level,
                CascadeLevel::AuthorImportant
                    | CascadeLevel::UserImportant
                    | CascadeLevel::UserAgentImportant
            )),
            closed_bundle: context.closed_bundle,
        });
        provenances.push(refinement_provenance_v0(
            "layer-predicate",
            Some("prove_layer_flatten_candidate"),
        ));
        verdicts.push(if layer.accepted {
            RefinementVerdictV0::SatisfiedAll
        } else {
            RefinementVerdictV0::Unknown
        });
    }

    let verdict = combine_refinement_verdicts(&verdicts);
    refinement_witness_v0("cascade-refinement-conjunction", verdict, provenances)
}

fn combine_refinement_verdicts(verdicts: &[RefinementVerdictV0]) -> RefinementVerdictV0 {
    if verdicts.is_empty() {
        return RefinementVerdictV0::SatisfiedAll;
    }
    if verdicts.contains(&RefinementVerdictV0::Unsatisfiable) {
        return RefinementVerdictV0::Unsatisfiable;
    }
    if verdicts
        .iter()
        .all(|verdict| *verdict == RefinementVerdictV0::SatisfiedAll)
    {
        return RefinementVerdictV0::SatisfiedAll;
    }
    if verdicts.contains(&RefinementVerdictV0::SatisfiedAll) {
        RefinementVerdictV0::SatisfiedSome
    } else {
        RefinementVerdictV0::Unknown
    }
}

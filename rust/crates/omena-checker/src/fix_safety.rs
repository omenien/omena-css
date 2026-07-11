use omena_abstract_value::FactPrecision;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FixSafetyV0 {
    Safe,
    Conservative,
    ManualReview,
}

impl FixSafetyV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Conservative => "conservative",
            Self::ManualReview => "manualReview",
        }
    }
}

/// Evidence requirements and observations used to classify one proposed edit.
///
/// Syntax preservation, file-local semantics, and closed-world semantics are
/// independent evidence scopes. A caller declares only the scopes the edit
/// needs, then supplies observations for those scopes. Missing required
/// evidence always lowers the result to manual review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FixSafetyEvidenceInputV0 {
    pub syntax_preserving: bool,
    pub local_semantics_required: bool,
    pub local_semantics_ready: bool,
    pub closed_world_required: bool,
    pub closed_world_ready: bool,
    pub reference_precision_required: bool,
    pub reference_precision: Option<FactPrecision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FixSafetyAssessmentV0 {
    pub safety: FixSafetyV0,
    pub safety_name: &'static str,
    pub precision_backed: bool,
    pub rationale: Vec<&'static str>,
}

pub fn compute_fix_safety(input: FixSafetyEvidenceInputV0) -> FixSafetyAssessmentV0 {
    let mut rationale = Vec::new();
    let mut manual_review = false;
    let mut conservative = false;

    if input.syntax_preserving {
        rationale.push("syntaxSafe");
    } else {
        rationale.push("syntaxNotPreserved");
        manual_review = true;
    }

    if input.local_semantics_required {
        if input.local_semantics_ready {
            rationale.push("localSemanticSafe");
        } else {
            rationale.push("localSemanticEvidenceMissing");
            manual_review = true;
        }
    }

    if input.closed_world_required {
        if input.closed_world_ready {
            rationale.push("workspaceClosedWorldSafe");
        } else {
            rationale.push("workspaceClosedWorldEvidenceMissing");
            manual_review = true;
        }
    }

    match input.reference_precision {
        Some(FactPrecision::Exact) => rationale.push("factPrecisionExact"),
        Some(FactPrecision::Conservative) => {
            rationale.push("factPrecisionConservative");
            conservative = true;
        }
        Some(FactPrecision::Heuristic) => {
            rationale.push("factPrecisionHeuristic");
            manual_review = true;
        }
        Some(FactPrecision::Unknown) => {
            rationale.push("factPrecisionUnknown");
            manual_review = true;
        }
        None if input.reference_precision_required => {
            rationale.push("factPrecisionMissing");
            manual_review = true;
        }
        None => {}
    }

    let safety = if manual_review {
        FixSafetyV0::ManualReview
    } else if conservative {
        FixSafetyV0::Conservative
    } else {
        FixSafetyV0::Safe
    };

    FixSafetyAssessmentV0 {
        safety,
        safety_name: safety.as_str(),
        precision_backed: input.reference_precision.is_some(),
        rationale,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OmenaCheckerRuleFixabilityV0;

    fn exact_workspace_input() -> FixSafetyEvidenceInputV0 {
        FixSafetyEvidenceInputV0 {
            syntax_preserving: true,
            local_semantics_required: true,
            local_semantics_ready: true,
            closed_world_required: true,
            closed_world_ready: true,
            reference_precision_required: true,
            reference_precision: Some(FactPrecision::Exact),
        }
    }

    #[test]
    fn evidence_signals_derive_all_safety_classes() {
        let safe = compute_fix_safety(exact_workspace_input());
        assert_eq!(safe.safety, FixSafetyV0::Safe);
        assert!(safe.precision_backed);
        assert_eq!(
            safe.rationale,
            vec![
                "syntaxSafe",
                "localSemanticSafe",
                "workspaceClosedWorldSafe",
                "factPrecisionExact",
            ]
        );

        let conservative = compute_fix_safety(FixSafetyEvidenceInputV0 {
            reference_precision: Some(FactPrecision::Conservative),
            ..exact_workspace_input()
        });
        assert_eq!(conservative.safety, FixSafetyV0::Conservative);
        assert!(
            conservative
                .rationale
                .contains(&"factPrecisionConservative")
        );

        for precision in [FactPrecision::Heuristic, FactPrecision::Unknown] {
            let manual = compute_fix_safety(FixSafetyEvidenceInputV0 {
                reference_precision: Some(precision),
                ..exact_workspace_input()
            });
            assert_eq!(manual.safety, FixSafetyV0::ManualReview);
        }
    }

    #[test]
    fn missing_required_evidence_fails_closed() {
        let cases = [
            FixSafetyEvidenceInputV0 {
                syntax_preserving: false,
                ..exact_workspace_input()
            },
            FixSafetyEvidenceInputV0 {
                local_semantics_ready: false,
                ..exact_workspace_input()
            },
            FixSafetyEvidenceInputV0 {
                closed_world_ready: false,
                ..exact_workspace_input()
            },
            FixSafetyEvidenceInputV0 {
                reference_precision: None,
                ..exact_workspace_input()
            },
        ];

        for input in cases {
            assert_eq!(compute_fix_safety(input).safety, FixSafetyV0::ManualReview);
        }
    }

    #[test]
    fn syntax_only_edits_are_honest_about_precision() {
        let assessment = compute_fix_safety(FixSafetyEvidenceInputV0 {
            syntax_preserving: true,
            local_semantics_required: false,
            local_semantics_ready: false,
            closed_world_required: false,
            closed_world_ready: false,
            reference_precision_required: false,
            reference_precision: None,
        });
        assert_eq!(assessment.safety, FixSafetyV0::Safe);
        assert!(!assessment.precision_backed);
        assert_eq!(assessment.rationale, vec!["syntaxSafe"]);
    }

    #[test]
    fn fixability_and_safety_remain_independent_axes() {
        let fixabilities = [
            OmenaCheckerRuleFixabilityV0::None,
            OmenaCheckerRuleFixabilityV0::CodeAction,
            OmenaCheckerRuleFixabilityV0::Autofix,
        ];
        let safety_classes = [
            FixSafetyV0::Safe,
            FixSafetyV0::Conservative,
            FixSafetyV0::ManualReview,
        ];
        let combinations = fixabilities
            .into_iter()
            .flat_map(|fixability| {
                safety_classes
                    .into_iter()
                    .map(move |safety| (fixability, safety))
            })
            .collect::<Vec<_>>();

        assert_eq!(combinations.len(), 9);
        assert!(combinations.contains(&(
            OmenaCheckerRuleFixabilityV0::Autofix,
            FixSafetyV0::ManualReview,
        )));
    }
}

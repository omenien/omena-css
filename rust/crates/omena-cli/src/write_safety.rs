use omena_checker::{FixSafetyAssessmentV0, FixSafetyV0};
use omena_query::OmenaQueryTransformDecisionV0;
use serde::Serialize;
use std::{fmt, fs, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum SourceWriteModeV0 {
    SafeOnly,
    AllowConservative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum SourceWriteKindV0 {
    LintFix,
    Formatting,
    Transform,
    MigrationPlan,
}

/// Additional evidence required by each product write surface.
///
/// Lint fixes need the shared safety assessment. Formatting also needs an
/// observed idempotence result. Transforms retain their engine decisions, and
/// migrations require an explicitly reviewed plan before a classified edit can
/// reach the filesystem gate.
pub(crate) enum SourceWriteEvidenceV0<'a> {
    LintFix,
    Formatting {
        idempotent: bool,
    },
    Transform {
        decisions: &'a [OmenaQueryTransformDecisionV0],
    },
    MigrationPlan {
        reviewed: bool,
    },
}

impl SourceWriteEvidenceV0<'_> {
    const fn kind(&self) -> SourceWriteKindV0 {
        match self {
            Self::LintFix => SourceWriteKindV0::LintFix,
            Self::Formatting { .. } => SourceWriteKindV0::Formatting,
            Self::Transform { .. } => SourceWriteKindV0::Transform,
            Self::MigrationPlan { .. } => SourceWriteKindV0::MigrationPlan,
        }
    }

    fn transform_decisions(&self) -> Vec<OmenaQueryTransformDecisionV0> {
        match self {
            Self::Transform { decisions } => decisions.to_vec(),
            Self::LintFix | Self::Formatting { .. } | Self::MigrationPlan { .. } => Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum SourceWriteRejectionReasonV0 {
    ConservativeRequiresOptIn,
    ManualReviewRequired,
    FormattingNotIdempotent,
    TransformDidNotApply,
    TransformBlocked,
    TransformRejected,
    MigrationPlanNotReviewed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SourceWriteRejectionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub output_path: String,
    pub write_kind: SourceWriteKindV0,
    pub safety: FixSafetyV0,
    pub precision_backed: bool,
    pub rationale: Vec<&'static str>,
    pub reason: SourceWriteRejectionReasonV0,
    pub transform_decisions: Vec<OmenaQueryTransformDecisionV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SourceWriteReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub output_path: String,
    pub write_kind: SourceWriteKindV0,
    pub safety: FixSafetyV0,
    pub precision_backed: bool,
    pub rationale: Vec<&'static str>,
    pub wrote: bool,
    pub transform_decisions: Vec<OmenaQueryTransformDecisionV0>,
}

#[derive(Debug)]
pub(crate) enum SourceWriteErrorV0 {
    Rejected(SourceWriteRejectionV0),
    Io {
        output_path: String,
        message: String,
    },
}

impl fmt::Display for SourceWriteErrorV0 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rejected(rejection) => write!(
                formatter,
                "source write rejected for {}: {:?}",
                rejection.output_path, rejection.reason
            ),
            Self::Io {
                output_path,
                message,
            } => write!(formatter, "failed to write {output_path}: {message}"),
        }
    }
}

pub(crate) fn apply_write_with_safety(
    output_path: &Path,
    content: &[u8],
    assessment: &FixSafetyAssessmentV0,
    mode: SourceWriteModeV0,
    evidence: SourceWriteEvidenceV0<'_>,
) -> Result<SourceWriteReportV0, SourceWriteErrorV0> {
    let output_path_string = output_path.to_string_lossy().into_owned();
    let transform_decisions = evidence.transform_decisions();

    if let Some(reason) = rejection_reason(assessment.safety, mode, &evidence) {
        return Err(SourceWriteErrorV0::Rejected(SourceWriteRejectionV0 {
            schema_version: "0",
            product: "omena-cli.source-write-rejection",
            output_path: output_path_string,
            write_kind: evidence.kind(),
            safety: assessment.safety,
            precision_backed: assessment.precision_backed,
            rationale: assessment.rationale.clone(),
            reason,
            transform_decisions,
        }));
    }

    fs::write(output_path, content).map_err(|error| SourceWriteErrorV0::Io {
        output_path: output_path_string.clone(),
        message: error.to_string(),
    })?;

    Ok(SourceWriteReportV0 {
        schema_version: "0",
        product: "omena-cli.source-write-report",
        output_path: output_path_string,
        write_kind: evidence.kind(),
        safety: assessment.safety,
        precision_backed: assessment.precision_backed,
        rationale: assessment.rationale.clone(),
        wrote: true,
        transform_decisions,
    })
}

fn rejection_reason(
    safety: FixSafetyV0,
    mode: SourceWriteModeV0,
    evidence: &SourceWriteEvidenceV0<'_>,
) -> Option<SourceWriteRejectionReasonV0> {
    match safety {
        FixSafetyV0::ManualReview => {
            return Some(SourceWriteRejectionReasonV0::ManualReviewRequired);
        }
        FixSafetyV0::Conservative if mode == SourceWriteModeV0::SafeOnly => {
            return Some(SourceWriteRejectionReasonV0::ConservativeRequiresOptIn);
        }
        FixSafetyV0::Safe | FixSafetyV0::Conservative => {}
    }

    match evidence {
        SourceWriteEvidenceV0::LintFix => None,
        SourceWriteEvidenceV0::Formatting { idempotent: false } => {
            Some(SourceWriteRejectionReasonV0::FormattingNotIdempotent)
        }
        SourceWriteEvidenceV0::Formatting { idempotent: true } => None,
        SourceWriteEvidenceV0::MigrationPlan { reviewed: false } => {
            Some(SourceWriteRejectionReasonV0::MigrationPlanNotReviewed)
        }
        SourceWriteEvidenceV0::MigrationPlan { reviewed: true } => None,
        SourceWriteEvidenceV0::Transform { decisions } => {
            let mut applied = false;
            for decision in *decisions {
                match decision {
                    OmenaQueryTransformDecisionV0::Applied { .. } => applied = true,
                    OmenaQueryTransformDecisionV0::NoChange { .. } => {}
                    OmenaQueryTransformDecisionV0::Blocked { .. } => {
                        return Some(SourceWriteRejectionReasonV0::TransformBlocked);
                    }
                    OmenaQueryTransformDecisionV0::Rejected { .. } => {
                        return Some(SourceWriteRejectionReasonV0::TransformRejected);
                    }
                }
            }
            (!applied).then_some(SourceWriteRejectionReasonV0::TransformDidNotApply)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omena_checker::{FixSafetyEvidenceInputV0, compute_fix_safety};
    use omena_query::execute_omena_query_transform_passes_from_source;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    fn fixture_path(label: &str) -> PathBuf {
        let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "omena-source-write-{label}-{}-{id}.css",
            std::process::id()
        ))
    }

    fn assessment(safety: FixSafetyV0) -> FixSafetyAssessmentV0 {
        let reference_precision = match safety {
            FixSafetyV0::Safe => Some(omena_query::FactPrecision::Exact),
            FixSafetyV0::Conservative => Some(omena_query::FactPrecision::Conservative),
            FixSafetyV0::ManualReview => Some(omena_query::FactPrecision::Unknown),
        };
        compute_fix_safety(FixSafetyEvidenceInputV0 {
            syntax_preserving: true,
            local_semantics_required: true,
            local_semantics_ready: true,
            closed_world_required: false,
            closed_world_ready: false,
            reference_precision_required: true,
            reference_precision,
        })
    }

    #[test]
    fn safe_writes_and_conservative_requires_opt_in() -> Result<(), String> {
        let safe_path = fixture_path("safe");
        let report = apply_write_with_safety(
            &safe_path,
            b".safe {}\n",
            &assessment(FixSafetyV0::Safe),
            SourceWriteModeV0::SafeOnly,
            SourceWriteEvidenceV0::LintFix,
        )
        .map_err(|error| error.to_string())?;
        assert!(report.wrote);
        assert_eq!(
            fs::read_to_string(&safe_path).map_err(|error| error.to_string())?,
            ".safe {}\n"
        );
        fs::remove_file(safe_path).map_err(|error| error.to_string())?;

        let conservative_path = fixture_path("conservative");
        let denied = apply_write_with_safety(
            &conservative_path,
            b".conservative {}\n",
            &assessment(FixSafetyV0::Conservative),
            SourceWriteModeV0::SafeOnly,
            SourceWriteEvidenceV0::LintFix,
        )
        .expect_err("conservative writes need explicit opt-in");
        assert!(matches!(
            denied,
            SourceWriteErrorV0::Rejected(SourceWriteRejectionV0 {
                reason: SourceWriteRejectionReasonV0::ConservativeRequiresOptIn,
                ..
            })
        ));
        assert!(!conservative_path.exists());

        apply_write_with_safety(
            &conservative_path,
            b".conservative {}\n",
            &assessment(FixSafetyV0::Conservative),
            SourceWriteModeV0::AllowConservative,
            SourceWriteEvidenceV0::LintFix,
        )
        .map_err(|error| error.to_string())?;
        fs::remove_file(conservative_path).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn manual_review_never_reaches_the_filesystem() {
        let path = fixture_path("manual");
        let denied = apply_write_with_safety(
            &path,
            b".manual {}\n",
            &assessment(FixSafetyV0::ManualReview),
            SourceWriteModeV0::AllowConservative,
            SourceWriteEvidenceV0::LintFix,
        )
        .expect_err("manual review must not be writable");
        assert!(matches!(
            denied,
            SourceWriteErrorV0::Rejected(SourceWriteRejectionV0 {
                reason: SourceWriteRejectionReasonV0::ManualReviewRequired,
                ..
            })
        ));
        assert!(!path.exists());
    }

    #[test]
    fn formatting_requires_observed_idempotence() {
        let path = fixture_path("formatting");
        let denied = apply_write_with_safety(
            &path,
            b".formatting {}\n",
            &assessment(FixSafetyV0::Safe),
            SourceWriteModeV0::SafeOnly,
            SourceWriteEvidenceV0::Formatting { idempotent: false },
        )
        .expect_err("non-idempotent formatting must not be written");
        assert!(matches!(
            denied,
            SourceWriteErrorV0::Rejected(SourceWriteRejectionV0 {
                reason: SourceWriteRejectionReasonV0::FormattingNotIdempotent,
                ..
            })
        ));
        assert!(!path.exists());
    }

    #[test]
    fn transform_writes_retain_real_execution_decisions() -> Result<(), String> {
        let execution = execute_omena_query_transform_passes_from_source(
            "fixture.css",
            ".card { color: #ffffff; }",
            &["color-compression".to_string()],
        );
        assert!(
            execution
                .execution
                .decisions
                .iter()
                .any(|decision| matches!(decision, OmenaQueryTransformDecisionV0::Applied { .. }))
        );

        let path = fixture_path("transform");
        let report = apply_write_with_safety(
            &path,
            execution.execution.output_css.as_bytes(),
            &assessment(FixSafetyV0::Safe),
            SourceWriteModeV0::SafeOnly,
            SourceWriteEvidenceV0::Transform {
                decisions: execution.execution.decisions.as_slice(),
            },
        )
        .map_err(|error| error.to_string())?;
        assert_eq!(report.transform_decisions, execution.execution.decisions);
        fs::remove_file(path).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn migration_requires_a_reviewed_plan() {
        let path = fixture_path("migration");
        let denied = apply_write_with_safety(
            &path,
            b".migration {}\n",
            &assessment(FixSafetyV0::Safe),
            SourceWriteModeV0::SafeOnly,
            SourceWriteEvidenceV0::MigrationPlan { reviewed: false },
        )
        .expect_err("migration writes require a reviewed plan");
        assert!(matches!(
            denied,
            SourceWriteErrorV0::Rejected(SourceWriteRejectionV0 {
                reason: SourceWriteRejectionReasonV0::MigrationPlanNotReviewed,
                ..
            })
        ));
        assert!(!path.exists());
    }
}

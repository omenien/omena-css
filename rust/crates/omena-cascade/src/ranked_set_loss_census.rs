use std::{
    panic::Location,
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering as AtomicOrdering},
    },
};

use serde::Serialize;

use crate::{
    CascadeDeclaration, CascadeLevel, CascadeOutcome, SpecificityExactnessV0,
    axis_order::CascadeKeyAxisV0,
    ranking::{InexactSpecificityAdjudicationV0, adjudicate_inexact_specificity_v0},
};

static CAPTURE_ACTIVE: AtomicBool = AtomicBool::new(false);
static CAPTURED_ROWS: Mutex<Vec<CascadeRankedSetLossCensusRowV0>> = Mutex::new(Vec::new());
static CAPTURE_STATE_RECOVERY_COUNT: AtomicUsize = AtomicUsize::new(0);
static MEASUREMENT_INVOCATION_COUNT: AtomicUsize = AtomicUsize::new(0);
static RANKED_SET_OUTCOME_COUNT: AtomicUsize = AtomicUsize::new(0);
static MULTI_CANDIDATE_INEXACT_RANKED_SET_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CascadeRankedSetFunctionV0 {
    CascadeProperty,
    CascadePropertyOpenWorld,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CascadeAxisPrefixV0 {
    Level,
    LayerRank,
    /// Retained for 0.x wire compatibility. The current specification order
    /// places specificity before scope proximity, so the pre-specificity
    /// classifier cannot emit this variant.
    ScopeProximity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CascadeRankedSetLossClassV0 {
    RecoverableAxisDominant { axis: CascadeAxisPrefixV0 },
    AxisWinnerInexact,
    NoStrictAxisDominance,
    SingleInexactCandidate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeRankedSetLossCandidateV0 {
    pub declaration_id: String,
    pub level: CascadeLevel,
    pub layer_rank: i32,
    pub scope_proximity: u32,
    pub specificity_exactness: SpecificityExactnessV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeRankedSetLossCensusRowV0 {
    pub function: CascadeRankedSetFunctionV0,
    pub invocation_site: &'static str,
    pub source_path: String,
    pub property: String,
    pub declaration_ids: Vec<String>,
    pub candidate_count: usize,
    pub candidates: Vec<CascadeRankedSetLossCandidateV0>,
    pub classification: CascadeRankedSetLossClassV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeRankedSetLossCaptureV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub capture_state_recovery_count: usize,
    pub measurement_invocation_count: usize,
    pub ranked_set_outcome_count: usize,
    pub multi_candidate_inexact_ranked_set_count: usize,
    pub rows: Vec<CascadeRankedSetLossCensusRowV0>,
}

/// Captures inexactness-bail `RankedSet` outcomes produced while `operation` runs.
///
/// Capture is process-wide so worker threads participate in the same bounded
/// measurement. Nested or concurrent captures are rejected instead of merging
/// unrelated populations.
pub fn capture_cascade_ranked_set_losses<R>(
    operation: impl FnOnce() -> R,
) -> Result<(R, CascadeRankedSetLossCaptureV0), &'static str> {
    CAPTURE_ACTIVE
        .compare_exchange(false, true, AtomicOrdering::AcqRel, AtomicOrdering::Acquire)
        .map_err(|_| "cascade ranked-set loss capture is already active")?;
    CAPTURE_STATE_RECOVERY_COUNT.store(0, AtomicOrdering::Release);
    captured_rows().clear();
    MEASUREMENT_INVOCATION_COUNT.store(0, AtomicOrdering::Release);
    RANKED_SET_OUTCOME_COUNT.store(0, AtomicOrdering::Release);
    MULTI_CANDIDATE_INEXACT_RANKED_SET_COUNT.store(0, AtomicOrdering::Release);
    let guard = CaptureGuard;
    let result = operation();
    let mut rows = std::mem::take(&mut *captured_rows());
    rows.sort_by(|left, right| {
        (
            left.function,
            left.invocation_site,
            left.source_path.as_str(),
            left.property.as_str(),
            left.declaration_ids.as_slice(),
        )
            .cmp(&(
                right.function,
                right.invocation_site,
                right.source_path.as_str(),
                right.property.as_str(),
                right.declaration_ids.as_slice(),
            ))
    });
    drop(guard);
    Ok((
        result,
        CascadeRankedSetLossCaptureV0 {
            schema_version: "0",
            product: "omena-cascade.ranked-set-loss-capture",
            capture_state_recovery_count: CAPTURE_STATE_RECOVERY_COUNT
                .load(AtomicOrdering::Acquire),
            measurement_invocation_count: MEASUREMENT_INVOCATION_COUNT
                .load(AtomicOrdering::Acquire),
            ranked_set_outcome_count: RANKED_SET_OUTCOME_COUNT.load(AtomicOrdering::Acquire),
            multi_candidate_inexact_ranked_set_count: MULTI_CANDIDATE_INEXACT_RANKED_SET_COUNT
                .load(AtomicOrdering::Acquire),
            rows,
        },
    ))
}

pub fn classify_cascade_ranked_set_loss(
    declarations: &[CascadeDeclaration],
) -> CascadeRankedSetLossClassV0 {
    match adjudicate_inexact_specificity_v0(declarations) {
        InexactSpecificityAdjudicationV0::Recoverable { deciding_axis, .. } => {
            CascadeRankedSetLossClassV0::RecoverableAxisDominant {
                axis: match deciding_axis {
                    CascadeKeyAxisV0::Level => CascadeAxisPrefixV0::Level,
                    CascadeKeyAxisV0::LayerRank => CascadeAxisPrefixV0::LayerRank,
                    CascadeKeyAxisV0::ScopeProximity
                    | CascadeKeyAxisV0::SpecificityIds
                    | CascadeKeyAxisV0::SpecificityClasses
                    | CascadeKeyAxisV0::SpecificityElements
                    | CascadeKeyAxisV0::SourceOrder => unreachable!(
                        "an exact winner cannot cross inexact specificity to recover on a later axis"
                    ),
                },
            }
        }
        InexactSpecificityAdjudicationV0::AxisWinnerInexact => {
            CascadeRankedSetLossClassV0::AxisWinnerInexact
        }
        InexactSpecificityAdjudicationV0::NoStrictAxisDominance => {
            CascadeRankedSetLossClassV0::NoStrictAxisDominance
        }
        InexactSpecificityAdjudicationV0::SingleInexactCandidate => {
            CascadeRankedSetLossClassV0::SingleInexactCandidate
        }
    }
}

pub(crate) fn observe_cascade_outcome(
    function: CascadeRankedSetFunctionV0,
    caller: &'static Location<'static>,
    outcome: &CascadeOutcome,
) {
    if !CAPTURE_ACTIVE.load(AtomicOrdering::Acquire) {
        return;
    }
    MEASUREMENT_INVOCATION_COUNT.fetch_add(1, AtomicOrdering::AcqRel);
    let CascadeOutcome::RankedSet(declarations) = outcome else {
        return;
    };
    RANKED_SET_OUTCOME_COUNT.fetch_add(1, AtomicOrdering::AcqRel);
    if !declarations
        .iter()
        .any(|declaration| declaration.specificity_exactness == SpecificityExactnessV0::Inexact)
    {
        return;
    }
    if declarations.len() > 1 {
        MULTI_CANDIDATE_INEXACT_RANKED_SET_COUNT.fetch_add(1, AtomicOrdering::AcqRel);
    }
    let row = CascadeRankedSetLossCensusRowV0 {
        function,
        invocation_site: invocation_site(caller.file()),
        source_path: caller.file().to_string(),
        property: declarations
            .first()
            .map(|declaration| declaration.property.to_string())
            .unwrap_or_default(),
        declaration_ids: declarations
            .iter()
            .map(|declaration| declaration.id.clone())
            .collect(),
        candidate_count: declarations.len(),
        candidates: declarations
            .iter()
            .map(|declaration| CascadeRankedSetLossCandidateV0 {
                declaration_id: declaration.id.clone(),
                level: declaration.key.level,
                layer_rank: declaration.key.layer_rank.get(),
                scope_proximity: declaration.key.scope_proximity,
                specificity_exactness: declaration.specificity_exactness,
            })
            .collect(),
        classification: classify_cascade_ranked_set_loss(declarations),
    };
    captured_rows().push(row);
}

fn invocation_site(source_path: &str) -> &'static str {
    if source_path.ends_with("omena-query/src/style/cascade_checker/runtime_state.rs") {
        "queryRuntimeStateScenarioEvaluation"
    } else if source_path.ends_with("omena-query/src/style/cascade_checker/confidence.rs") {
        "queryCascadeMarginForEvaluation"
    } else if source_path.ends_with("omena-query/src/style/cascade_checker/replica_ensemble.rs") {
        "collectQueryReplicaEnsembleSiteOutcomes"
    } else if source_path.ends_with("omena-cascade/src/computed_value.rs") {
        "computeCascadeComputedValue"
    } else if source_path.ends_with("omena-transform-passes/src/runtime/winner_equality.rs") {
        "transformWinnerEqualityFromCascadeOutcome"
    } else {
        "unclassified"
    }
}

fn captured_rows() -> std::sync::MutexGuard<'static, Vec<CascadeRankedSetLossCensusRowV0>> {
    let (rows, recovered) = recover_captured_rows(CAPTURED_ROWS.lock(), &CAPTURED_ROWS);
    if recovered {
        CAPTURE_STATE_RECOVERY_COUNT.fetch_add(1, AtomicOrdering::AcqRel);
    }
    rows
}

fn recover_captured_rows<'a>(
    lock: std::sync::LockResult<std::sync::MutexGuard<'a, Vec<CascadeRankedSetLossCensusRowV0>>>,
    mutex: &'a Mutex<Vec<CascadeRankedSetLossCensusRowV0>>,
) -> (
    std::sync::MutexGuard<'a, Vec<CascadeRankedSetLossCensusRowV0>>,
    bool,
) {
    match lock {
        Ok(rows) => (rows, false),
        Err(poisoned) => {
            mutex.clear_poison();
            let mut rows = poisoned.into_inner();
            rows.clear();
            (rows, true)
        }
    }
}

struct CaptureGuard;

impl Drop for CaptureGuard {
    fn drop(&mut self) {
        CAPTURE_ACTIVE.store(false, AtomicOrdering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CascadeAxisPrefixV0, CascadeRankedSetLossCensusRowV0, CascadeRankedSetLossClassV0,
        classify_cascade_ranked_set_loss, recover_captured_rows,
    };
    use crate::{
        CascadeDeclaration, CascadeKey, CascadeLevel, CascadeValue, LayerOrdinal,
        OpenWorldTieEvidence, Specificity, SpecificityExactnessV0, normalized_layer_rank,
    };

    fn declaration(
        id: &str,
        level: CascadeLevel,
        layer_ordinal: i32,
        scope_proximity: u32,
        specificity: Specificity,
        exactness: SpecificityExactnessV0,
    ) -> CascadeDeclaration {
        CascadeDeclaration {
            id: id.to_string(),
            property: omena_syntax::ident::AuthoredPropertyTextV0::new("color"),
            property_key: omena_syntax::ident::PropertyNameV0::standard("color").canonical_key(),
            value: CascadeValue::Literal(id.to_string()),
            key: CascadeKey::new(
                level,
                normalized_layer_rank(false, LayerOrdinal::new(layer_ordinal)),
                scope_proximity,
                specificity,
                0,
            ),
            open_world_tie_evidence: OpenWorldTieEvidence::NONE,
            specificity_exactness: exactness,
        }
    }

    #[test]
    fn axis_winner_exactness_changes_the_recoverability_class() {
        let lower = declaration(
            "lower",
            CascadeLevel::UserNormal,
            0,
            0,
            Specificity::new(9, 9, 9),
            SpecificityExactnessV0::Inexact,
        );
        let exact_winner = declaration(
            "winner",
            CascadeLevel::AuthorNormal,
            0,
            0,
            Specificity::ZERO,
            SpecificityExactnessV0::Exact,
        );
        assert_eq!(
            classify_cascade_ranked_set_loss(&[lower.clone(), exact_winner.clone()]),
            CascadeRankedSetLossClassV0::RecoverableAxisDominant {
                axis: CascadeAxisPrefixV0::Level
            }
        );

        let mut inexact_winner = exact_winner;
        inexact_winner.specificity_exactness = SpecificityExactnessV0::Inexact;
        assert_eq!(
            classify_cascade_ranked_set_loss(&[lower, inexact_winner]),
            CascadeRankedSetLossClassV0::AxisWinnerInexact
        );
    }

    #[test]
    fn specificity_only_winner_has_no_strict_axis_dominance() {
        let weaker = declaration(
            "weaker",
            CascadeLevel::AuthorNormal,
            0,
            0,
            Specificity::new(0, 1, 0),
            SpecificityExactnessV0::Inexact,
        );
        let stronger = declaration(
            "stronger",
            CascadeLevel::AuthorNormal,
            0,
            0,
            Specificity::new(1, 0, 0),
            SpecificityExactnessV0::Exact,
        );
        assert_eq!(
            classify_cascade_ranked_set_loss(&[weaker, stronger]),
            CascadeRankedSetLossClassV0::NoStrictAxisDominance
        );
    }

    #[test]
    fn single_inexact_candidate_is_not_vacuously_recoverable() {
        let candidate = declaration(
            "only",
            CascadeLevel::AuthorNormal,
            0,
            0,
            Specificity::ZERO,
            SpecificityExactnessV0::Inexact,
        );
        assert_eq!(
            classify_cascade_ranked_set_loss(&[candidate]),
            CascadeRankedSetLossClassV0::SingleInexactCandidate
        );
    }

    #[test]
    #[should_panic(expected = "requires an inexact declaration")]
    fn exact_only_input_is_outside_the_loss_classifier_domain() {
        let candidate = declaration(
            "exact",
            CascadeLevel::AuthorNormal,
            0,
            0,
            Specificity::ZERO,
            SpecificityExactnessV0::Exact,
        );
        let _ = classify_cascade_ranked_set_loss(&[candidate]);
    }

    #[test]
    fn poisoned_capture_storage_is_cleared_and_reported() {
        let rows = std::sync::Arc::new(
            std::sync::Mutex::<Vec<CascadeRankedSetLossCensusRowV0>>::new(Vec::new()),
        );
        let poisoned_rows = std::sync::Arc::clone(&rows);
        let poison_result = std::thread::spawn(move || {
            let _guard = match poisoned_rows.lock() {
                Ok(guard) => guard,
                Err(error) => error.into_inner(),
            };
            std::panic::resume_unwind(Box::new("poison capture storage"));
        })
        .join();
        assert!(poison_result.is_err());

        let (recovered_rows, recovered) = recover_captured_rows(rows.lock(), &rows);
        assert!(recovered);
        assert!(recovered_rows.is_empty());
        assert!(!rows.is_poisoned());
    }
}

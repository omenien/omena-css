/// Reviewed mismatch classes for observations made before a wave flushes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ReactiveDivergenceClassV0 {
    FlushConeClosureTiming,
    MidWaveReadTiming,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReactiveObservationPhaseV0 {
    DuringWave,
    Flush,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReactiveDivergenceDispositionV0 {
    BenignUntilFlush,
    Blocker,
}

impl ReactiveDivergenceClassV0 {
    /// Returns the reviewed classes without exposing their count in the type.
    pub const fn all() -> &'static [Self] {
        &[Self::FlushConeClosureTiming, Self::MidWaveReadTiming]
    }

    pub const fn id(self) -> &'static str {
        match self {
            Self::FlushConeClosureTiming => "flushConeClosureTiming",
            Self::MidWaveReadTiming => "midWaveReadTiming",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::all().iter().copied().find(|class| class.id() == id)
    }

    /// A reviewed timing difference is benign only while the graph is still
    /// stabilizing. Any mismatch that survives the flush remains a blocker.
    pub const fn disposition(
        self,
        phase: ReactiveObservationPhaseV0,
    ) -> ReactiveDivergenceDispositionV0 {
        match phase {
            ReactiveObservationPhaseV0::DuringWave => {
                ReactiveDivergenceDispositionV0::BenignUntilFlush
            }
            ReactiveObservationPhaseV0::Flush => ReactiveDivergenceDispositionV0::Blocker,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TAXONOMY: &str = include_str!("../docs/divergence-taxonomy.md");

    #[test]
    fn committed_taxonomy_and_machine_ids_are_in_lockstep() {
        for class in ReactiveDivergenceClassV0::all() {
            assert!(
                TAXONOMY.contains(&format!("`{}`", class.id())),
                "taxonomy must document {}",
                class.id()
            );
        }
        assert_eq!(TAXONOMY.matches("## Class:").count(), 2);
    }

    #[test]
    fn reviewed_timing_classes_never_suppress_a_flush_mismatch() {
        for class in ReactiveDivergenceClassV0::all() {
            assert_eq!(
                class.disposition(ReactiveObservationPhaseV0::DuringWave),
                ReactiveDivergenceDispositionV0::BenignUntilFlush
            );
            assert_eq!(
                class.disposition(ReactiveObservationPhaseV0::Flush),
                ReactiveDivergenceDispositionV0::Blocker
            );
        }
        assert!(ReactiveDivergenceClassV0::from_id("unreviewedDifference").is_none());
    }
}

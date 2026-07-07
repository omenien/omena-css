//! Tide scheduling kernel (rfcs#111): an epoch ledger with per-input
//! high-water marks, footprint-scoped job validity, and idempotent demand
//! lanes flushed through settle gates.
//!
//! The kernel is pure data — no threads, no channels, no clocks. The shell
//! feeds events in (`advance`, `deposit`, gate inputs per evaluation) and
//! routes flushes out to executors; that split is what makes the invariants
//! property-testable on synthetic event streams (rfcs#111 §12 M0).

pub mod demand;
pub mod ledger;

pub use demand::{
    TideDemandJoinV0, TideFlushV0, TideGateInputsV0, TideLaneConfigV0, TideLaneV0,
    TideRepublishDemandV0, TideSifDemandV0,
};
pub use ledger::{TideEpochLedgerV0, TideFootprintStampV0, TideFootprintV0, TideInputKindV0};

/// Publication supersession rule (rfcs#111 §8.4): per-key last-writer-wins on
/// the lexicographic order key `(epoch, tier_rank)`, so a delayed baseline
/// can never overwrite its refined sibling and ties stay idempotent.
pub fn tide_may_publish(last_published: Option<(u64, u8)>, next: (u64, u8)) -> bool {
    last_published.is_none_or(|last| next >= last)
}

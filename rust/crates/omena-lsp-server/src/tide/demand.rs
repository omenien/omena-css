//! Demand lanes and settle gates (rfcs#111 §4.1, §9.6).
//!
//! Trigger sites deposit demands — idempotently, into a set — and nothing
//! else; whether, when, and how often work executes is decided here, once,
//! at gate evaluation. The gate has two layers with strictly different
//! authority (rfcs#111 §7 I5):
//!
//! - the CORRECTNESS layer — frontier passage — is never overridable;
//! - the COURTESY layer — idleness — may be overridden by aging.
//!
//! If upstream genuinely never settles, not flushing is correct, and the
//! lane records a starvation alarm instead of forcing a flush.

use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TideDemandV0 {
    /// Full-workspace external SIF re-resolution.
    SifRefresh,
    /// Full-workspace diagnostics republish (the post-SIF follow-up).
    WorkspaceRepublish,
}

/// Per-evaluation gate inputs, fed by the shell. The kernel never reads
/// clocks or channels itself.
#[derive(Debug, Clone, Copy)]
pub struct TideGateInputsV0 {
    /// Correctness layer: the upstream frontier for this lane has passed.
    pub frontier_passed: bool,
    /// Courtesy layer: no interactive work is pending.
    pub idle: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct TideLaneConfigV0 {
    /// Courtesy aging bound, in shell ticks. A demand older than this stops
    /// waiting for idleness — but never for the frontier.
    pub aging_bound_ticks: u64,
}

/// One gated flush: the demands drained from the lane, stamped with the
/// lane generation that executors re-check at item boundaries.
#[derive(Debug, PartialEq, Eq)]
pub struct TideFlushV0 {
    pub generation: u64,
    pub demands: Vec<TideDemandV0>,
}

#[derive(Debug, Default)]
pub struct TideLaneV0 {
    demands: BTreeSet<TideDemandV0>,
    generation: u64,
    in_flight: bool,
    oldest_deposit_tick: Option<u64>,
    starvation_alarmed: bool,
    starvation_alarm_count: u64,
}

impl TideLaneV0 {
    /// Idempotent deposit; returns whether the demand was newly inserted.
    pub fn deposit(&mut self, demand: TideDemandV0, now_tick: u64) -> bool {
        if self.oldest_deposit_tick.is_none() {
            self.oldest_deposit_tick = Some(now_tick);
        }
        self.demands.insert(demand)
    }

    pub fn has_demand(&self) -> bool {
        !self.demands.is_empty()
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn in_flight(&self) -> bool {
        self.in_flight
    }

    pub fn starvation_alarm_count(&self) -> u64 {
        self.starvation_alarm_count
    }

    /// Gate evaluation. Flushes at most one tide per settle window: while a
    /// tide is in flight or the lane is empty, this is a no-op regardless of
    /// gate inputs (I1). Aging can satisfy the courtesy layer, never the
    /// correctness layer; an aged demand behind a closed frontier raises the
    /// starvation alarm once per accumulation window (I5).
    pub fn try_flush(
        &mut self,
        inputs: TideGateInputsV0,
        now_tick: u64,
        config: &TideLaneConfigV0,
    ) -> Option<TideFlushV0> {
        if self.in_flight || self.demands.is_empty() {
            return None;
        }
        let aged = self
            .oldest_deposit_tick
            .is_some_and(|oldest| now_tick.saturating_sub(oldest) >= config.aging_bound_ticks);
        if !inputs.frontier_passed {
            if aged && !self.starvation_alarmed {
                self.starvation_alarmed = true;
                self.starvation_alarm_count = self.starvation_alarm_count.saturating_add(1);
            }
            return None;
        }
        if !inputs.idle && !aged {
            return None;
        }
        self.generation = self.generation.saturating_add(1);
        self.in_flight = true;
        self.oldest_deposit_tick = None;
        self.starvation_alarmed = false;
        Some(TideFlushV0 {
            generation: self.generation,
            demands: std::mem::take(&mut self.demands).into_iter().collect(),
        })
    }

    /// Upstream input relevant to this lane changed while a tide was in
    /// flight: the settle window reopens. The running tide is disowned — its
    /// generation no longer matches, so the executor aborts at the next item
    /// boundary and its remaining publishes lose supersession (rfcs#111
    /// §9.4). Returns the new generation for executors' gen-watch.
    pub fn reopen_window(&mut self) -> u64 {
        if self.in_flight {
            self.generation = self.generation.saturating_add(1);
            self.in_flight = false;
        }
        self.generation
    }

    /// Executor completion for `generation`. Stale completions — a tide that
    /// was disowned by `reopen_window` — are ignored.
    pub fn tide_completed(&mut self, generation: u64) {
        if generation == self.generation {
            self.in_flight = false;
        }
    }
}

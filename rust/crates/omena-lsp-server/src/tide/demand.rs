//! Demand lanes and settle gates (rfcs#111 §4.1, §9.6), lattice edition.
//!
//! A lane is an LVar cell (Kuper & Newton, FHPC 2013): its content is one
//! join-semilattice value, trigger sites DEPOSIT by least-upper-bound —
//! monotone, idempotent, commutative — and nothing else; whether, when, and
//! how often work executes is decided here, once, at gate evaluation. The
//! gate is a threshold read with two layers of strictly different authority
//! (rfcs#111 §7 I5):
//!
//! - the CORRECTNESS layer — frontier passage — is never overridable;
//! - the COURTESY layer — idleness — may be overridden by aging.
//!
//! If upstream genuinely never settles, not flushing is correct, and the
//! lane records a starvation alarm instead of forcing a flush.
//!
//! PER-EPOCH SCOPING ("Freeze After Writing", Kuper et al., POPL 2014): a
//! settle-window reopen is a retraction, which a pure LVar forbids. The
//! lattice is therefore scoped to one window: demands accumulate
//! monotonically within it, a flush freezes the accumulated value into an
//! in-flight tide, and a reopen JOINS that frozen value back into the new
//! window. Coverage owed can grow or carry over, never silently shrink —
//! previously the "a disowned tide's keys are re-covered" invariant leaned
//! on every deposit being the whole workspace; with cone-shaped demands the
//! carry-over makes it structural.

use std::collections::BTreeSet;

/// One lane's demand vocabulary: a join-semilattice with a bottom element.
/// `join` must be idempotent, commutative, and associative; `deposit` and
/// the window carry-over both go through it.
pub trait TideDemandJoinV0: Clone + PartialEq {
    fn bottom() -> Self;
    fn is_bottom(&self) -> bool;
    fn join(&mut self, other: Self);
}

/// The SIF lane's demand: the two-point lattice — either a full external
/// SIF re-resolution is owed or nothing is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TideSifDemandV0 {
    owed: bool,
}

impl TideSifDemandV0 {
    pub fn refresh() -> Self {
        Self { owed: true }
    }
}

impl TideDemandJoinV0 for TideSifDemandV0 {
    fn bottom() -> Self {
        Self { owed: false }
    }

    fn is_bottom(&self) -> bool {
        !self.owed
    }

    fn join(&mut self, other: Self) {
        self.owed |= other.owed;
    }
}

/// The republish lane's demand: the powerset lattice of seed paths with an
/// absorbing top. `Cone` carries SEEDS — the reverse-dependency closure is
/// taken at FLUSH time against the then-current committed graph, so a seed
/// deposited early still fans out over edges that appear before the flush.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TideRepublishDemandV0 {
    /// ⊥ — nothing owed.
    None,
    /// Republish the reverse-dependency cones of these seed paths.
    Cone(BTreeSet<String>),
    /// ⊤ — the whole workspace.
    All,
}

impl TideRepublishDemandV0 {
    pub fn cone(seeds: impl IntoIterator<Item = String>) -> Self {
        let seeds: BTreeSet<String> = seeds.into_iter().collect();
        if seeds.is_empty() {
            Self::None
        } else {
            Self::Cone(seeds)
        }
    }
}

impl TideDemandJoinV0 for TideRepublishDemandV0 {
    fn bottom() -> Self {
        Self::None
    }

    fn is_bottom(&self) -> bool {
        matches!(self, Self::None)
    }

    fn join(&mut self, other: Self) {
        *self = match (std::mem::replace(self, Self::None), other) {
            (Self::All, _) | (_, Self::All) => Self::All,
            (Self::None, other) => other,
            (current, Self::None) => current,
            (Self::Cone(mut seeds), Self::Cone(more)) => {
                seeds.extend(more);
                Self::Cone(seeds)
            }
        };
    }
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

/// One gated flush: the lattice value frozen out of the lane, stamped with
/// the lane generation that executors re-check at item boundaries.
#[derive(Debug, PartialEq, Eq)]
pub struct TideFlushV0<D> {
    pub generation: u64,
    pub demand: D,
}

/// The frozen state of an in-flight tide: its lattice value plus the age of
/// its oldest deposit, both restored on a window reopen so carried-over
/// demand neither loses coverage nor gets a fresh (or ancient) age.
#[derive(Debug)]
struct TideInFlightV0<D> {
    demand: D,
    oldest_deposit_tick: Option<u64>,
}

#[derive(Debug)]
pub struct TideLaneV0<D: TideDemandJoinV0> {
    demand: D,
    /// The frozen in-flight tide. A window reopen joins it back into
    /// `demand` (per-epoch carry-over); a current-generation completion
    /// discharges it.
    in_flight: Option<TideInFlightV0<D>>,
    generation: u64,
    oldest_deposit_tick: Option<u64>,
    starvation_alarmed: bool,
    starvation_alarm_count: u64,
}

impl<D: TideDemandJoinV0> Default for TideLaneV0<D> {
    fn default() -> Self {
        Self {
            demand: D::bottom(),
            in_flight: None,
            generation: 0,
            oldest_deposit_tick: None,
            starvation_alarmed: false,
            starvation_alarm_count: 0,
        }
    }
}

impl<D: TideDemandJoinV0> TideLaneV0<D> {
    /// Monotone deposit: joins into the lane's lattice value. Returns
    /// whether the value actually grew.
    pub fn deposit(&mut self, demand: D, now_tick: u64) -> bool {
        if demand.is_bottom() {
            return false;
        }
        if self.oldest_deposit_tick.is_none() {
            self.oldest_deposit_tick = Some(now_tick);
        }
        let before = self.demand.clone();
        self.demand.join(demand);
        self.demand != before
    }

    pub fn has_demand(&self) -> bool {
        !self.demand.is_bottom()
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn in_flight(&self) -> bool {
        self.in_flight.is_some()
    }

    pub fn starvation_alarm_count(&self) -> u64 {
        self.starvation_alarm_count
    }

    /// Gate evaluation — the threshold read. Flushes at most one tide per
    /// settle window: while a tide is in flight or the lane is at bottom,
    /// this is a no-op regardless of gate inputs (I1). Aging can satisfy
    /// the courtesy layer, never the correctness layer; an aged demand
    /// behind a closed frontier raises the starvation alarm once per
    /// accumulation window (I5).
    pub fn try_flush(
        &mut self,
        inputs: TideGateInputsV0,
        now_tick: u64,
        config: &TideLaneConfigV0,
    ) -> Option<TideFlushV0<D>> {
        if self.in_flight.is_some() || self.demand.is_bottom() {
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
        self.starvation_alarmed = false;
        let demand = std::mem::replace(&mut self.demand, D::bottom());
        self.in_flight = Some(TideInFlightV0 {
            demand: demand.clone(),
            oldest_deposit_tick: self.oldest_deposit_tick.take(),
        });
        Some(TideFlushV0 {
            generation: self.generation,
            demand,
        })
    }

    /// Upstream input relevant to this lane changed while a tide was in
    /// flight: the settle window reopens. The running tide is disowned —
    /// its generation no longer matches, so the executor aborts at the next
    /// item boundary — and its frozen demand is JOINED back into the new
    /// window, so the coverage it owed is owed again (per-epoch carry-over;
    /// rfcs#111 §9.4). Returns the new generation for executors' gen-watch.
    pub fn reopen_window(&mut self) -> u64 {
        if let Some(disowned) = self.in_flight.take() {
            self.generation = self.generation.saturating_add(1);
            self.demand.join(disowned.demand);
            self.oldest_deposit_tick =
                match (self.oldest_deposit_tick, disowned.oldest_deposit_tick) {
                    (Some(current), Some(carried)) => Some(current.min(carried)),
                    (tick, None) | (None, tick) => tick,
                };
        }
        self.generation
    }

    /// Executor completion for `generation`: discharges the frozen demand.
    /// Stale completions — a tide that was disowned by `reopen_window` —
    /// are ignored; their obligation already carried over.
    pub fn tide_completed(&mut self, generation: u64) {
        if generation == self.generation {
            self.in_flight = None;
        }
    }
}

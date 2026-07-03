//! M0 property harness for the Tide kernel (rfcs#111 §12): synthetic event
//! streams drive the lane/ledger and machine-check the invariants I1, I5,
//! and the footprint-validity rule against an independent model. The oracle
//! exists before the wiring it gates.

use crate::tide::{
    TideDemandV0, TideEpochLedgerV0, TideFootprintV0, TideGateInputsV0, TideInputKindV0,
    TideLaneConfigV0, TideLaneV0, tide_may_publish,
};
use std::collections::BTreeSet;

/// Deterministic xorshift64 — no new dependencies, reproducible failures.
struct XorShift64(u64);

impl XorShift64 {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

const SIF_FOOTPRINT: TideFootprintV0 = TideFootprintV0::of(&[
    TideInputKindV0::DocumentSet,
    TideInputKindV0::LockfileFingerprint,
    TideInputKindV0::PackageManifest,
    TideInputKindV0::ResolutionSettings,
]);

#[test]
fn footprint_validity_ignores_unrelated_input_kinds() {
    // rfcs#111 §9.3: a keystroke cannot kill an in-flight SIF job.
    let mut ledger = TideEpochLedgerV0::default();
    ledger.advance(&[TideInputKindV0::DocumentSet]);
    let stamp = ledger.stamp(SIF_FOOTPRINT);

    ledger.advance(&[TideInputKindV0::DocumentText]);
    assert!(
        ledger.is_current(&stamp),
        "a DocumentText advance must not stale a job that never reads it"
    );

    ledger.advance(&[TideInputKindV0::DocumentSet]);
    assert!(
        !ledger.is_current(&stamp),
        "a footprint-member advance must stale the stamp"
    );
}

#[test]
fn footprint_validity_matches_model_under_random_advances() {
    for seed in 1..=20u64 {
        let mut rng = XorShift64(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let mut ledger = TideEpochLedgerV0::default();
        let mut stamp = ledger.stamp(SIF_FOOTPRINT);
        let mut model_stale = false;
        for _ in 0..2_000 {
            let kind = TideInputKindV0::ALL[rng.below(7) as usize];
            ledger.advance(&[kind]);
            if SIF_FOOTPRINT.contains(kind) {
                model_stale = true;
            }
            assert_eq!(
                ledger.is_current(&stamp),
                !model_stale,
                "validity diverged from the model after advancing {kind:?}"
            );
            if rng.below(4) == 0 {
                stamp = ledger.stamp(SIF_FOOTPRINT);
                model_stale = false;
            }
        }
    }
}

#[test]
fn lane_invariants_hold_under_random_event_streams() {
    let config = TideLaneConfigV0 {
        aging_bound_ticks: 10,
    };
    for seed in 1..=50u64 {
        let mut rng = XorShift64(seed.wrapping_mul(0x2545_F491_4F6C_DD1D));
        let mut lane = TideLaneV0::default();
        let mut model_demands: BTreeSet<TideDemandV0> = BTreeSet::new();
        let mut model_in_flight = false;
        for tick in 0..3_000u64 {
            match rng.below(6) {
                0 | 1 => {
                    let demand = if rng.below(2) == 0 {
                        TideDemandV0::SifRefresh
                    } else {
                        TideDemandV0::WorkspaceRepublish
                    };
                    lane.deposit(demand.clone(), tick);
                    model_demands.insert(demand);
                }
                2 | 3 => {
                    let inputs = TideGateInputsV0 {
                        frontier_passed: rng.below(2) == 0,
                        idle: rng.below(2) == 0,
                    };
                    let alarms_before = lane.starvation_alarm_count();
                    match lane.try_flush(inputs, tick, &config) {
                        Some(flush) => {
                            // I5: aging never satisfies the correctness layer.
                            assert!(inputs.frontier_passed, "flush behind a closed frontier");
                            // I1: one in-flight tide per lane; never from empty.
                            assert!(!model_in_flight, "second concurrent tide");
                            assert!(!model_demands.is_empty(), "flush from an empty lane");
                            let drained: Vec<_> = model_demands.iter().cloned().collect();
                            assert_eq!(flush.demands, drained, "flush must drain everything");
                            assert_eq!(flush.generation, lane.generation());
                            model_demands.clear();
                            model_in_flight = true;
                        }
                        None => {
                            if lane.starvation_alarm_count() > alarms_before {
                                assert!(
                                    !inputs.frontier_passed,
                                    "starvation alarm outside a closed frontier"
                                );
                            }
                        }
                    }
                }
                4 => {
                    // Completion, sometimes with a deliberately stale generation.
                    let generation = if rng.below(4) == 0 {
                        lane.generation().saturating_sub(1)
                    } else {
                        lane.generation()
                    };
                    let matches_current = generation == lane.generation();
                    lane.tide_completed(generation);
                    if matches_current && model_in_flight {
                        model_in_flight = false;
                    }
                }
                _ => {
                    let generation_before = lane.generation();
                    let generation_after = lane.reopen_window();
                    if model_in_flight {
                        // rfcs#111 §9.4: the running tide is disowned.
                        assert_eq!(generation_after, generation_before + 1);
                        model_in_flight = false;
                    } else {
                        assert_eq!(generation_after, generation_before);
                    }
                }
            }
            assert_eq!(
                lane.in_flight(),
                model_in_flight,
                "in-flight model diverged"
            );
        }
    }
}

#[test]
fn aging_overrides_courtesy_but_never_correctness() {
    let config = TideLaneConfigV0 {
        aging_bound_ticks: 10,
    };
    let mut lane = TideLaneV0::default();
    lane.deposit(TideDemandV0::WorkspaceRepublish, 0);

    // Aged demand behind a CLOSED frontier: no flush, one alarm per window.
    let closed = TideGateInputsV0 {
        frontier_passed: false,
        idle: true,
    };
    assert!(lane.try_flush(closed, 15, &config).is_none());
    assert_eq!(lane.starvation_alarm_count(), 1);
    assert!(lane.try_flush(closed, 16, &config).is_none());
    assert_eq!(
        lane.starvation_alarm_count(),
        1,
        "alarm fires once per accumulation window"
    );

    // Same aged demand, frontier OPEN but not idle: aging overrides courtesy.
    let open_busy = TideGateInputsV0 {
        frontier_passed: true,
        idle: false,
    };
    let flush = lane
        .try_flush(open_busy, 17, &config)
        .expect("aged demand must flush once the frontier passes");
    assert_eq!(flush.demands, vec![TideDemandV0::WorkspaceRepublish]);

    // A fresh demand while busy: courtesy holds it back until idle or aged.
    lane.tide_completed(flush.generation);
    lane.deposit(TideDemandV0::WorkspaceRepublish, 20);
    assert!(lane.try_flush(open_busy, 21, &config).is_none());
    let open_idle = TideGateInputsV0 {
        frontier_passed: true,
        idle: true,
    };
    assert!(lane.try_flush(open_idle, 22, &config).is_some());
}

#[test]
fn one_flush_per_window_and_deposit_idempotence() {
    let config = TideLaneConfigV0 {
        aging_bound_ticks: 10,
    };
    let open_idle = TideGateInputsV0 {
        frontier_passed: true,
        idle: true,
    };
    let mut lane = TideLaneV0::default();
    assert!(lane.deposit(TideDemandV0::SifRefresh, 0));
    assert!(
        !lane.deposit(TideDemandV0::SifRefresh, 1),
        "deposits are idempotent"
    );

    let flush = lane.try_flush(open_idle, 2, &config).expect("gate open");
    assert_eq!(flush.demands, vec![TideDemandV0::SifRefresh]);
    // I1: no second flush while the tide is in flight, even with demand.
    lane.deposit(TideDemandV0::SifRefresh, 3);
    assert!(lane.try_flush(open_idle, 4, &config).is_none());
    lane.tide_completed(flush.generation);
    assert!(lane.try_flush(open_idle, 5, &config).is_some());
    // Empty lane never flushes.
    assert!(lane.try_flush(open_idle, 6, &config).is_none());
}

#[test]
fn publication_supersession_order_key_is_epoch_then_tier() {
    // rfcs#111 §8.4 / I6.
    assert!(tide_may_publish(None, (1, 0)));
    assert!(
        tide_may_publish(Some((1, 0)), (1, 1)),
        "refined after baseline"
    );
    assert!(
        !tide_may_publish(Some((1, 1)), (1, 0)),
        "delayed baseline must lose"
    );
    assert!(
        !tide_may_publish(Some((2, 0)), (1, 1)),
        "older epoch must lose"
    );
    assert!(
        tide_may_publish(Some((1, 1)), (2, 0)),
        "new epoch supersedes any tier"
    );
    assert!(
        tide_may_publish(Some((1, 1)), (1, 1)),
        "ties are idempotent republish"
    );
}

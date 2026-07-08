//! Property harness for the Tide kernel: synthetic event streams drive the
//! lane/ledger and machine-check the invariants I1, I5, the footprint-
//! validity rule, and the per-epoch demand-lattice laws (join monotonicity,
//! deposit-order confluence, disown carry-over conservation) against an
//! independent model. The oracle exists before the wiring it gates.

use crate::tide::{
    TideDemandJoinV0, TideEpochLedgerV0, TideFootprintV0, TideGateInputsV0, TideInputKindV0,
    TideLaneConfigV0, TideLaneV0, TideRepublishDemandV0, tide_may_publish,
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

fn random_republish_demand(rng: &mut XorShift64) -> TideRepublishDemandV0 {
    match rng.below(5) {
        0 => TideRepublishDemandV0::All,
        _ => {
            let seeds = ["a", "b", "c", "d"];
            let mut set = BTreeSet::new();
            for seed in seeds {
                if rng.below(2) == 0 {
                    set.insert(seed.to_string());
                }
            }
            TideRepublishDemandV0::cone(set)
        }
    }
}

#[test]
fn republish_demand_join_is_a_join_semilattice() {
    let mut rng = XorShift64(0x51ED_2701_9E11_77D3);
    for _ in 0..2_000 {
        let a = random_republish_demand(&mut rng);
        let b = random_republish_demand(&mut rng);
        let c = random_republish_demand(&mut rng);

        // Idempotent.
        let mut aa = a.clone();
        aa.join(a.clone());
        assert_eq!(aa, a, "join must be idempotent");
        // Commutative.
        let mut ab = a.clone();
        ab.join(b.clone());
        let mut ba = b.clone();
        ba.join(a.clone());
        assert_eq!(ab, ba, "join must be commutative");
        // Associative.
        let mut ab_c = ab.clone();
        ab_c.join(c.clone());
        let mut bc = b.clone();
        bc.join(c.clone());
        let mut a_bc = a.clone();
        a_bc.join(bc);
        assert_eq!(ab_c, a_bc, "join must be associative");
        // Bottom is the identity.
        let mut a_bottom = a.clone();
        a_bottom.join(TideRepublishDemandV0::bottom());
        assert_eq!(a_bottom, a, "bottom must be the join identity");
    }
}

#[test]
fn sif_demand_join_is_a_join_semilattice() {
    use crate::tide::TideSifDemandV0;
    let values = [TideSifDemandV0::bottom(), TideSifDemandV0::refresh()];
    for a in values {
        for b in values {
            for c in values {
                let mut aa = a;
                aa.join(a);
                assert_eq!(aa, a, "join must be idempotent");
                let mut ab = a;
                ab.join(b);
                let mut ba = b;
                ba.join(a);
                assert_eq!(ab, ba, "join must be commutative");
                let mut ab_c = ab;
                ab_c.join(c);
                let mut bc = b;
                bc.join(c);
                let mut a_bc = a;
                a_bc.join(bc);
                assert_eq!(ab_c, a_bc, "join must be associative");
                let mut a_bottom = a;
                a_bottom.join(TideSifDemandV0::bottom());
                assert_eq!(a_bottom, a, "bottom must be the join identity");
            }
        }
    }
}

#[test]
fn deposit_order_is_confluent() {
    // LVars determinism (Kuper & Newton, FHPC 2013): monotone joins make the
    // settled value independent of deposit interleaving.
    let mut rng = XorShift64(0xC0FF_EE00_1234_5678);
    let config = TideLaneConfigV0 {
        aging_bound_ticks: 10,
    };
    let open_idle = TideGateInputsV0 {
        frontier_passed: true,
        idle: true,
    };
    for _ in 0..200 {
        let deposits: Vec<TideRepublishDemandV0> =
            (0..4).map(|_| random_republish_demand(&mut rng)).collect();
        let mut expected = TideRepublishDemandV0::bottom();
        for deposit in &deposits {
            expected.join(deposit.clone());
        }
        // Two independently shuffled orders of the same multiset.
        for _ in 0..2 {
            let mut order: Vec<usize> = (0..deposits.len()).collect();
            for index in (1..order.len()).rev() {
                let swap = rng.below(index as u64 + 1) as usize;
                order.swap(index, swap);
            }
            let mut lane = TideLaneV0::<TideRepublishDemandV0>::default();
            for position in &order {
                lane.deposit(deposits[*position].clone(), 0);
            }
            match lane.try_flush(open_idle, 1, &config) {
                Some(flush) => assert_eq!(
                    flush.demand, expected,
                    "flushed join must be order-independent"
                ),
                None => assert!(
                    expected.is_bottom(),
                    "a non-bottom joined deposit set must flush"
                ),
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
        let mut lane = TideLaneV0::<TideRepublishDemandV0>::default();
        // Model: the accumulated window value and the frozen in-flight value.
        let mut model_window = TideRepublishDemandV0::bottom();
        let mut model_in_flight: Option<TideRepublishDemandV0> = None;
        for tick in 0..3_000u64 {
            match rng.below(6) {
                0 | 1 => {
                    let demand = random_republish_demand(&mut rng);
                    lane.deposit(demand.clone(), tick);
                    model_window.join(demand);
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
                            // I1: one in-flight tide per lane; never from bottom.
                            assert!(model_in_flight.is_none(), "second concurrent tide");
                            assert!(!model_window.is_bottom(), "flush from a bottom lane");
                            // The flush freezes exactly the window's join.
                            assert_eq!(flush.demand, model_window, "flush must drain the join");
                            assert_eq!(flush.generation, lane.generation());
                            model_in_flight = Some(std::mem::replace(
                                &mut model_window,
                                TideRepublishDemandV0::bottom(),
                            ));
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
                    if matches_current && model_in_flight.is_some() {
                        model_in_flight = None;
                    }
                }
                _ => {
                    let generation_before = lane.generation();
                    let generation_after = lane.reopen_window();
                    if let Some(disowned) = model_in_flight.take() {
                        // rfcs#111 §9.4 + per-epoch carry-over: the running
                        // tide is disowned AND its coverage is owed again.
                        assert_eq!(generation_after, generation_before + 1);
                        model_window.join(disowned);
                    } else {
                        assert_eq!(generation_after, generation_before);
                    }
                }
            }
            assert_eq!(
                lane.in_flight(),
                model_in_flight.is_some(),
                "in-flight model diverged"
            );
            assert_eq!(
                lane.has_demand(),
                !model_window.is_bottom(),
                "window-value model diverged"
            );
        }
    }
}

#[test]
fn disowned_demand_carries_over_into_the_new_window() -> Result<(), &'static str> {
    let config = TideLaneConfigV0 {
        aging_bound_ticks: 10,
    };
    let open_idle = TideGateInputsV0 {
        frontier_passed: true,
        idle: true,
    };
    let mut lane = TideLaneV0::<TideRepublishDemandV0>::default();
    lane.deposit(
        TideRepublishDemandV0::cone([String::from("a"), String::from("b")]),
        0,
    );
    let flush = lane.try_flush(open_idle, 1, &config).ok_or("first flush")?;

    // The window reopens mid-tide; a new smaller cone arrives.
    lane.reopen_window();
    lane.deposit(TideRepublishDemandV0::cone([String::from("c")]), 2);
    // The stale completion must not discharge the carried demand.
    lane.tide_completed(flush.generation);

    let carried = lane
        .try_flush(open_idle, 3, &config)
        .ok_or("carry-over flush")?;
    assert_eq!(
        carried.demand,
        TideRepublishDemandV0::cone([String::from("a"), String::from("b"), String::from("c")]),
        "the disowned tide's coverage must be owed again, joined with new deposits"
    );
    Ok(())
}

#[test]
fn carried_over_demand_keeps_its_deposit_age() -> Result<(), &'static str> {
    let config = TideLaneConfigV0 {
        aging_bound_ticks: 10,
    };
    let open_idle = TideGateInputsV0 {
        frontier_passed: true,
        idle: true,
    };
    let open_busy = TideGateInputsV0 {
        frontier_passed: true,
        idle: false,
    };
    let mut lane = TideLaneV0::<TideRepublishDemandV0>::default();
    lane.deposit(TideRepublishDemandV0::All, 0);
    let _flush = lane.try_flush(open_idle, 1, &config).ok_or("flush")?;
    lane.reopen_window();
    // The carried demand is as old as its ORIGINAL deposit (tick 0), so at
    // tick 11 aging already overrides courtesy — it neither restarts young
    // nor pretends to be older than it is.
    assert!(
        lane.try_flush(open_busy, 5, &config).is_none(),
        "not yet aged at tick 5"
    );
    assert!(
        lane.try_flush(open_busy, 11, &config).is_some(),
        "aged out of courtesy by tick 11 measured from the original deposit"
    );
    Ok(())
}

#[test]
fn aging_overrides_courtesy_but_never_correctness() -> Result<(), &'static str> {
    let config = TideLaneConfigV0 {
        aging_bound_ticks: 10,
    };
    let mut lane = TideLaneV0::<TideRepublishDemandV0>::default();
    lane.deposit(TideRepublishDemandV0::All, 0);

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
        .ok_or("aged demand must flush once the frontier passes")?;
    assert_eq!(flush.demand, TideRepublishDemandV0::All);

    // A fresh demand while busy: courtesy holds it back until idle or aged.
    lane.tide_completed(flush.generation);
    lane.deposit(TideRepublishDemandV0::All, 20);
    assert!(lane.try_flush(open_busy, 21, &config).is_none());
    let open_idle = TideGateInputsV0 {
        frontier_passed: true,
        idle: true,
    };
    assert!(lane.try_flush(open_idle, 22, &config).is_some());
    Ok(())
}

#[test]
fn one_flush_per_window_and_deposit_idempotence() -> Result<(), &'static str> {
    let config = TideLaneConfigV0 {
        aging_bound_ticks: 10,
    };
    let open_idle = TideGateInputsV0 {
        frontier_passed: true,
        idle: true,
    };
    let mut lane = TideLaneV0::<TideRepublishDemandV0>::default();
    let cone = TideRepublishDemandV0::cone([String::from("a")]);
    assert!(lane.deposit(cone.clone(), 0));
    assert!(
        !lane.deposit(cone.clone(), 1),
        "joining an already-held value must report no growth"
    );

    let flush = lane
        .try_flush(open_idle, 2, &config)
        .ok_or("gate must open")?;
    assert_eq!(flush.demand, cone);
    // I1: no second flush while the tide is in flight, even with demand.
    lane.deposit(TideRepublishDemandV0::All, 3);
    assert!(lane.try_flush(open_idle, 4, &config).is_none());
    lane.tide_completed(flush.generation);
    assert!(lane.try_flush(open_idle, 5, &config).is_some());
    // Bottom lane never flushes.
    assert!(lane.try_flush(open_idle, 6, &config).is_none());
    Ok(())
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

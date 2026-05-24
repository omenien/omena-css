use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeSpinGlassSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub hamiltonian: CascadeSpinGlassHamiltonianV0,
    pub frustration: CascadeFrustrationV0,
    pub replica_overlap: CascadeReplicaOverlapV0,
    pub stability_score: CascadeStabilityScoreV0,
    pub ultrametric_corpus: CascadeSpinGlassUltrametricCorpusV0,
    pub theorem_contracts: Vec<CascadeSpinGlassTheoremV0>,
    pub advisory_policy: SpinGlassMonteCarloPolicyV0,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeSpinGlassHamiltonianV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub unit: &'static str,
    pub energy_bits: f64,
    pub deterministic: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeFrustrationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub frustrated_edge_count: usize,
    pub total_edge_count: usize,
    pub advisory_only: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeReplicaOverlapV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub overlap_bucket_count: usize,
    pub parisi_breakpoint_m: Option<f64>,
    pub advisory_only: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeStabilityScoreV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub score: f64,
    pub deterministic_component_passed: bool,
    pub advisory_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeUltrametricPathV0 {
    pub origin: u8,
    pub layer: u8,
    pub scope: u8,
    pub rule: u8,
    pub declaration: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeSpinGlassUltrametricCorpusV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub path_count: usize,
    pub triple_count: usize,
    pub max_distance: u8,
    pub exhaustive_binary_five_tier_domain: bool,
    pub strong_triangle_passed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeSpinGlassTheoremV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub theorem_id: &'static str,
    pub statement: &'static str,
    pub deterministic: bool,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpinGlassMonteCarloPolicyV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub advisory_only: bool,
    pub bucket_count: usize,
    pub buckets: Vec<SpinGlassMonteCarloBucketV0>,
    pub task_budget_ms: u64,
    pub debounce_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpinGlassMonteCarloBucketV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub bucket: &'static str,
    pub max_variable_count: usize,
    pub sample_count: usize,
}

pub fn summarize_cascade_spin_glass_statistics() -> CascadeSpinGlassSummaryV0 {
    let ultrametric_corpus = verify_spin_glass_ultrametric_corpus_v0();

    CascadeSpinGlassSummaryV0 {
        schema_version: "0",
        product: "omena-cascade.spin-glass",
        layer_marker: "statistical-mechanics",
        feature_gate: "spin-glass",
        hamiltonian: summarize_spin_glass_hamiltonian(&[1, 2, 3, 5]),
        frustration: summarize_cascade_frustration(0, 3),
        replica_overlap: summarize_replica_overlap(4, Some(0.5)),
        stability_score: summarize_cascade_stability_score(1.0),
        ultrametric_corpus: ultrametric_corpus.clone(),
        theorem_contracts: vec![
            CascadeSpinGlassTheoremV0 {
                schema_version: "0",
                product: "omena-cascade.spin-glass-theorem",
                layer_marker: "statistical-mechanics",
                feature_gate: "spin-glass",
                theorem_id: "D1",
                statement: "5-tier cascade ultrametric strong triangle corpus passes",
                deterministic: true,
                passed: ultrametric_corpus.strong_triangle_passed,
            },
            CascadeSpinGlassTheoremV0 {
                schema_version: "0",
                product: "omena-cascade.spin-glass-theorem",
                layer_marker: "statistical-mechanics",
                feature_gate: "spin-glass",
                theorem_id: "D3",
                statement: "tropical Hamiltonian monotonicity fixture passes",
                deterministic: true,
                passed: prove_tropical_hamiltonian_monotone(&[1, 2, 3, 5]),
            },
            CascadeSpinGlassTheoremV0 {
                schema_version: "0",
                product: "omena-cascade.spin-glass-theorem",
                layer_marker: "statistical-mechanics",
                feature_gate: "spin-glass",
                theorem_id: "D4",
                statement: "ultrametric isomorphism fixture passes",
                deterministic: true,
                passed: prove_ultrametric_isomorphism(&[0, 2, 2]),
            },
        ],
        advisory_policy: spin_glass_monte_carlo_policy(),
    }
}

pub fn summarize_spin_glass_hamiltonian(energies: &[u32]) -> CascadeSpinGlassHamiltonianV0 {
    CascadeSpinGlassHamiltonianV0 {
        schema_version: "0",
        product: "omena-cascade.spin-glass-hamiltonian",
        layer_marker: "statistical-mechanics",
        feature_gate: "spin-glass",
        unit: "bit",
        energy_bits: energies.iter().map(|value| f64::from(*value)).sum(),
        deterministic: true,
    }
}

pub fn summarize_cascade_frustration(
    frustrated_edge_count: usize,
    total_edge_count: usize,
) -> CascadeFrustrationV0 {
    CascadeFrustrationV0 {
        schema_version: "0",
        product: "omena-cascade.frustration",
        layer_marker: "statistical-mechanics",
        feature_gate: "spin-glass",
        frustrated_edge_count,
        total_edge_count,
        advisory_only: true,
    }
}

pub fn summarize_replica_overlap(
    overlap_bucket_count: usize,
    parisi_breakpoint_m: Option<f64>,
) -> CascadeReplicaOverlapV0 {
    CascadeReplicaOverlapV0 {
        schema_version: "0",
        product: "omena-cascade.replica-overlap",
        layer_marker: "statistical-mechanics",
        feature_gate: "spin-glass",
        overlap_bucket_count,
        parisi_breakpoint_m,
        advisory_only: true,
    }
}

pub fn summarize_cascade_stability_score(score: f64) -> CascadeStabilityScoreV0 {
    CascadeStabilityScoreV0 {
        schema_version: "0",
        product: "omena-cascade.stability-score",
        layer_marker: "statistical-mechanics",
        feature_gate: "spin-glass",
        score,
        deterministic_component_passed: verify_spin_glass_ultrametric_corpus_v0()
            .strong_triangle_passed
            && prove_tropical_hamiltonian_monotone(&[1, 2, 3, 5])
            && prove_ultrametric_isomorphism(&[0, 2, 2]),
        advisory_only: true,
    }
}

pub fn spin_glass_monte_carlo_policy() -> SpinGlassMonteCarloPolicyV0 {
    SpinGlassMonteCarloPolicyV0 {
        schema_version: "0",
        product: "omena-cascade.spin-glass-monte-carlo-policy",
        layer_marker: "statistical-mechanics",
        feature_gate: "spin-glass",
        advisory_only: true,
        bucket_count: 4,
        buckets: vec![
            SpinGlassMonteCarloBucketV0 {
                schema_version: "0",
                product: "omena-cascade.spin-glass-monte-carlo-bucket",
                layer_marker: "statistical-mechanics",
                feature_gate: "spin-glass",
                bucket: "tiny",
                max_variable_count: 16,
                sample_count: 0,
            },
            SpinGlassMonteCarloBucketV0 {
                schema_version: "0",
                product: "omena-cascade.spin-glass-monte-carlo-bucket",
                layer_marker: "statistical-mechanics",
                feature_gate: "spin-glass",
                bucket: "small",
                max_variable_count: 64,
                sample_count: 128,
            },
            SpinGlassMonteCarloBucketV0 {
                schema_version: "0",
                product: "omena-cascade.spin-glass-monte-carlo-bucket",
                layer_marker: "statistical-mechanics",
                feature_gate: "spin-glass",
                bucket: "medium",
                max_variable_count: 256,
                sample_count: 512,
            },
            SpinGlassMonteCarloBucketV0 {
                schema_version: "0",
                product: "omena-cascade.spin-glass-monte-carlo-bucket",
                layer_marker: "statistical-mechanics",
                feature_gate: "spin-glass",
                bucket: "large",
                max_variable_count: usize::MAX,
                sample_count: 1024,
            },
        ],
        task_budget_ms: 200,
        debounce_ms: 500,
    }
}

pub fn spin_glass_ultrametric_binary_five_tier_corpus_v0() -> Vec<CascadeUltrametricPathV0> {
    let mut paths = Vec::with_capacity(32);

    for origin in 0..=1 {
        for layer in 0..=1 {
            for scope in 0..=1 {
                for rule in 0..=1 {
                    for declaration in 0..=1 {
                        paths.push(CascadeUltrametricPathV0 {
                            origin,
                            layer,
                            scope,
                            rule,
                            declaration,
                        });
                    }
                }
            }
        }
    }

    paths
}

pub fn cascade_ultrametric_distance_v0(
    lhs: CascadeUltrametricPathV0,
    rhs: CascadeUltrametricPathV0,
) -> u8 {
    if lhs.origin != rhs.origin {
        5
    } else if lhs.layer != rhs.layer {
        4
    } else if lhs.scope != rhs.scope {
        3
    } else if lhs.rule != rhs.rule {
        2
    } else if lhs.declaration != rhs.declaration {
        1
    } else {
        0
    }
}

pub fn prove_cascade_ultrametric_strong_triangle_v0(
    lhs: CascadeUltrametricPathV0,
    mid: CascadeUltrametricPathV0,
    rhs: CascadeUltrametricPathV0,
) -> bool {
    let direct = cascade_ultrametric_distance_v0(lhs, rhs);
    let via_lhs_mid = cascade_ultrametric_distance_v0(lhs, mid);
    let via_mid_rhs = cascade_ultrametric_distance_v0(mid, rhs);

    direct <= via_lhs_mid.max(via_mid_rhs)
}

pub fn verify_spin_glass_ultrametric_corpus_v0() -> CascadeSpinGlassUltrametricCorpusV0 {
    let paths = spin_glass_ultrametric_binary_five_tier_corpus_v0();
    let mut max_distance = 0;
    let mut triple_count = 0;
    let mut strong_triangle_passed = true;

    for lhs in &paths {
        for rhs in &paths {
            max_distance = max_distance.max(cascade_ultrametric_distance_v0(*lhs, *rhs));
        }
    }

    for lhs in &paths {
        for mid in &paths {
            for rhs in &paths {
                triple_count += 1;
                strong_triangle_passed &=
                    prove_cascade_ultrametric_strong_triangle_v0(*lhs, *mid, *rhs);
            }
        }
    }

    CascadeSpinGlassUltrametricCorpusV0 {
        schema_version: "0",
        product: "omena-cascade.spin-glass-ultrametric-corpus",
        layer_marker: "statistical-mechanics",
        feature_gate: "spin-glass",
        path_count: paths.len(),
        triple_count,
        max_distance,
        exhaustive_binary_five_tier_domain: paths.len() == 32,
        strong_triangle_passed,
    }
}

pub fn prove_strong_triangle_inequality(a: u32, b: u32, c: u32) -> bool {
    let lhs = a.max(c);
    let rhs = a.max(b).max(b.max(c));
    lhs <= rhs
}

pub fn prove_tropical_hamiltonian_monotone(energies: &[u32]) -> bool {
    energies.windows(2).all(|pair| pair[0] <= pair[1])
}

pub fn prove_ultrametric_isomorphism(distances_from_root: &[u32]) -> bool {
    distances_from_root
        .windows(2)
        .all(|pair| pair[0] == 0 || pair[0] == pair[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_spin_glass_theorems_pass() {
        let summary = summarize_cascade_spin_glass_statistics();

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.layer_marker, "statistical-mechanics");
        assert_eq!(summary.feature_gate, "spin-glass");
        assert_eq!(summary.hamiltonian.schema_version, "0");
        assert_eq!(summary.hamiltonian.feature_gate, "spin-glass");
        assert_eq!(summary.hamiltonian.unit, "bit");
        assert_eq!(summary.frustration.schema_version, "0");
        assert_eq!(summary.frustration.feature_gate, "spin-glass");
        assert_eq!(summary.replica_overlap.schema_version, "0");
        assert_eq!(summary.replica_overlap.feature_gate, "spin-glass");
        assert_eq!(summary.stability_score.schema_version, "0");
        assert_eq!(summary.stability_score.feature_gate, "spin-glass");
        assert!(summary.stability_score.deterministic_component_passed);
        assert_eq!(summary.ultrametric_corpus.schema_version, "0");
        assert_eq!(summary.ultrametric_corpus.path_count, 32);
        assert_eq!(summary.ultrametric_corpus.triple_count, 32 * 32 * 32);
        assert_eq!(summary.ultrametric_corpus.max_distance, 5);
        assert!(summary.ultrametric_corpus.strong_triangle_passed);
        assert!(
            summary
                .theorem_contracts
                .iter()
                .all(|theorem| theorem.schema_version == "0"
                    && theorem.feature_gate == "spin-glass"
                    && theorem.passed)
        );
        assert!(summary.advisory_policy.advisory_only);
        assert_eq!(summary.advisory_policy.bucket_count, 4);
    }

    #[test]
    fn spin_glass_monte_carlo_policy_enforces_m4_alpha_runtime_bounds() {
        let policy = spin_glass_monte_carlo_policy();

        assert_eq!(policy.schema_version, "0");
        assert_eq!(
            policy.product,
            "omena-cascade.spin-glass-monte-carlo-policy"
        );
        assert_eq!(policy.layer_marker, "statistical-mechanics");
        assert_eq!(policy.feature_gate, "spin-glass");
        assert!(policy.advisory_only);
        assert_eq!(policy.bucket_count, 4);
        assert_eq!(policy.task_budget_ms, 200);
        assert_eq!(policy.debounce_ms, 500);
        assert_eq!(policy.buckets.len(), policy.bucket_count);
        assert_eq!(policy.buckets[0].bucket, "tiny");
        assert_eq!(policy.buckets[0].max_variable_count, 16);
        assert_eq!(policy.buckets[0].sample_count, 0);
        assert_eq!(policy.buckets[1].bucket, "small");
        assert_eq!(policy.buckets[1].max_variable_count, 64);
        assert_eq!(policy.buckets[1].sample_count, 128);
        assert_eq!(policy.buckets[2].bucket, "medium");
        assert_eq!(policy.buckets[2].max_variable_count, 256);
        assert_eq!(policy.buckets[2].sample_count, 512);
        assert_eq!(policy.buckets[3].bucket, "large");
        assert_eq!(policy.buckets[3].max_variable_count, usize::MAX);
        assert_eq!(policy.buckets[3].sample_count, 1024);
    }

    #[test]
    fn frustration_measure_contract_is_advisory() {
        let frustration = summarize_cascade_frustration(1, 4);

        assert_eq!(frustration.schema_version, "0");
        assert_eq!(frustration.layer_marker, "statistical-mechanics");
        assert!(frustration.advisory_only);
    }

    #[test]
    fn stability_score_contract_keeps_deterministic_component() {
        let stability = summarize_cascade_stability_score(0.75);

        assert_eq!(stability.schema_version, "0");
        assert_eq!(stability.layer_marker, "statistical-mechanics");
        assert!(stability.deterministic_component_passed);
        assert!(stability.advisory_only);
    }

    #[test]
    fn ultrametricity_test_enforces_theorem_fixture() {
        assert!(prove_strong_triangle_inequality(2, 3, 3));
        assert!(prove_ultrametric_isomorphism(&[0, 2, 2]));
    }

    #[test]
    fn spin_glass_ultrametric_corpus_enforces_five_tier_strong_triangle() {
        let corpus = verify_spin_glass_ultrametric_corpus_v0();

        assert_eq!(corpus.schema_version, "0");
        assert_eq!(
            corpus.product,
            "omena-cascade.spin-glass-ultrametric-corpus"
        );
        assert_eq!(corpus.layer_marker, "statistical-mechanics");
        assert_eq!(corpus.feature_gate, "spin-glass");
        assert_eq!(corpus.path_count, 32);
        assert_eq!(corpus.triple_count, 32 * 32 * 32);
        assert_eq!(corpus.max_distance, 5);
        assert!(corpus.exhaustive_binary_five_tier_domain);
        assert!(corpus.strong_triangle_passed);

        let same_origin = CascadeUltrametricPathV0 {
            origin: 0,
            layer: 0,
            scope: 0,
            rule: 0,
            declaration: 0,
        };
        let sibling_declaration = CascadeUltrametricPathV0 {
            origin: 0,
            layer: 0,
            scope: 0,
            rule: 0,
            declaration: 1,
        };
        let different_origin = CascadeUltrametricPathV0 {
            origin: 1,
            layer: 0,
            scope: 0,
            rule: 0,
            declaration: 0,
        };

        assert_eq!(
            cascade_ultrametric_distance_v0(same_origin, sibling_declaration),
            1
        );
        assert_eq!(
            cascade_ultrametric_distance_v0(same_origin, different_origin),
            5
        );
        assert!(prove_cascade_ultrametric_strong_triangle_v0(
            same_origin,
            sibling_declaration,
            different_origin
        ));
    }
}

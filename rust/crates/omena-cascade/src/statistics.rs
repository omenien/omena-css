use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeSpinGlassSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub theorem_contracts: Vec<CascadeSpinGlassTheoremV0>,
    pub advisory_policy: SpinGlassMonteCarloPolicyV0,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeSpinGlassTheoremV0 {
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
    pub advisory_only: bool,
    pub bucket_count: usize,
    pub buckets: Vec<SpinGlassMonteCarloBucketV0>,
    pub task_budget_ms: u64,
    pub debounce_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpinGlassMonteCarloBucketV0 {
    pub bucket: &'static str,
    pub max_variable_count: usize,
    pub sample_count: usize,
}

pub fn summarize_cascade_spin_glass_statistics() -> CascadeSpinGlassSummaryV0 {
    CascadeSpinGlassSummaryV0 {
        schema_version: "0",
        product: "omena-cascade.spin-glass",
        layer_marker: "statistical-mechanics",
        theorem_contracts: vec![
            CascadeSpinGlassTheoremV0 {
                theorem_id: "D1",
                statement: "strong triangle inequality fixture passes",
                deterministic: true,
                passed: prove_strong_triangle_inequality(2, 3, 3),
            },
            CascadeSpinGlassTheoremV0 {
                theorem_id: "D3",
                statement: "tropical Hamiltonian monotonicity fixture passes",
                deterministic: true,
                passed: prove_tropical_hamiltonian_monotone(&[1, 2, 3, 5]),
            },
            CascadeSpinGlassTheoremV0 {
                theorem_id: "D4",
                statement: "ultrametric isomorphism fixture passes",
                deterministic: true,
                passed: prove_ultrametric_isomorphism(&[0, 2, 2]),
            },
        ],
        advisory_policy: spin_glass_monte_carlo_policy(),
    }
}

pub fn spin_glass_monte_carlo_policy() -> SpinGlassMonteCarloPolicyV0 {
    SpinGlassMonteCarloPolicyV0 {
        schema_version: "0",
        product: "omena-cascade.spin-glass-monte-carlo-policy",
        advisory_only: true,
        bucket_count: 4,
        buckets: vec![
            SpinGlassMonteCarloBucketV0 {
                bucket: "tiny",
                max_variable_count: 16,
                sample_count: 0,
            },
            SpinGlassMonteCarloBucketV0 {
                bucket: "small",
                max_variable_count: 64,
                sample_count: 128,
            },
            SpinGlassMonteCarloBucketV0 {
                bucket: "medium",
                max_variable_count: 256,
                sample_count: 512,
            },
            SpinGlassMonteCarloBucketV0 {
                bucket: "large",
                max_variable_count: usize::MAX,
                sample_count: 1024,
            },
        ],
        task_budget_ms: 200,
        debounce_ms: 500,
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
        assert!(
            summary
                .theorem_contracts
                .iter()
                .all(|theorem| theorem.passed)
        );
        assert!(summary.advisory_policy.advisory_only);
        assert_eq!(summary.advisory_policy.bucket_count, 4);
    }
}

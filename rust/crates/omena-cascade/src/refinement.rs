//! Refinement entry points layered above the byte-stable cascade proof module.

use omena_refinement_trait::{
    RefinementVerdictV0, RefinementWitnessV0, refinement_provenance_v0, refinement_witness_v0,
};

use crate::{
    CascadeDeclaration, CascadeLevel, LayerFlattenInputV0, ScopeFlattenInputV0,
    StaticSupportsAssumptionV0, StaticSupportsEvalVerdictV0, evaluate_static_supports_condition,
    prove_layer_flatten_candidate, prove_scope_flatten_candidate,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CascadeRefinementContextV0 {
    pub supports_condition: Option<String>,
    pub scope_root_selector: Option<String>,
    pub layer_name: Option<String>,
    pub closed_bundle: bool,
}

impl Default for CascadeRefinementContextV0 {
    fn default() -> Self {
        Self {
            supports_condition: None,
            scope_root_selector: None,
            layer_name: None,
            closed_bundle: true,
        }
    }
}

pub fn refine_declaration_in_context(
    declaration: &CascadeDeclaration,
    context: &CascadeRefinementContextV0,
) -> RefinementWitnessV0 {
    let mut provenances = Vec::new();
    let mut verdicts = Vec::new();

    if let Some(condition) = context.supports_condition.as_deref() {
        let supports = evaluate_static_supports_condition(
            condition,
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        provenances.push(refinement_provenance_v0(
            "supports-predicate",
            Some("evaluate_static_supports_condition"),
        ));
        verdicts.push(match supports.verdict {
            StaticSupportsEvalVerdictV0::AlwaysTrue => RefinementVerdictV0::SatisfiedAll,
            StaticSupportsEvalVerdictV0::AlwaysFalse => RefinementVerdictV0::Unsatisfiable,
            StaticSupportsEvalVerdictV0::Unknown => RefinementVerdictV0::Unknown,
        });
    }

    if let Some(root_selector) = context.scope_root_selector.as_deref() {
        let scope = prove_scope_flatten_candidate(ScopeFlattenInputV0 {
            root_selector: root_selector.to_string(),
            limit_selector: None,
            scoped_rule_count: 1,
            peer_scope_count: 0,
            competing_unscoped_rule_count: 0,
            inside_layer: context.layer_name.is_some(),
        });
        provenances.push(refinement_provenance_v0(
            "scope-predicate",
            Some("prove_scope_flatten_candidate"),
        ));
        verdicts.push(if scope.accepted {
            RefinementVerdictV0::SatisfiedAll
        } else {
            RefinementVerdictV0::Unknown
        });
    }

    if context.layer_name.is_some() {
        let layer = prove_layer_flatten_candidate(LayerFlattenInputV0 {
            layer_name: context.layer_name.clone(),
            layer_rule_count: 1,
            peer_layer_count: 0,
            unlayered_rule_count: 0,
            important_declaration_count: usize::from(matches!(
                declaration.key.level,
                CascadeLevel::AuthorImportant
                    | CascadeLevel::UserImportant
                    | CascadeLevel::UserAgentImportant
            )),
            closed_bundle: context.closed_bundle,
        });
        provenances.push(refinement_provenance_v0(
            "layer-predicate",
            Some("prove_layer_flatten_candidate"),
        ));
        verdicts.push(if layer.accepted {
            RefinementVerdictV0::SatisfiedAll
        } else {
            RefinementVerdictV0::Unknown
        });
    }

    let verdict = combine_refinement_verdicts(&verdicts);
    refinement_witness_v0("cascade-refinement-conjunction", verdict, provenances)
}

fn combine_refinement_verdicts(verdicts: &[RefinementVerdictV0]) -> RefinementVerdictV0 {
    if verdicts.is_empty() {
        return RefinementVerdictV0::SatisfiedAll;
    }
    if verdicts.contains(&RefinementVerdictV0::Unsatisfiable) {
        return RefinementVerdictV0::Unsatisfiable;
    }
    if verdicts
        .iter()
        .all(|verdict| *verdict == RefinementVerdictV0::SatisfiedAll)
    {
        return RefinementVerdictV0::SatisfiedAll;
    }
    if verdicts.contains(&RefinementVerdictV0::SatisfiedAll) {
        RefinementVerdictV0::SatisfiedSome
    } else {
        RefinementVerdictV0::Unknown
    }
}

#[cfg(test)]
mod tests {
    const EXPECTED_LEGACY_PROOFS_RS_SHA256: [u8; 32] = [
        0xc6, 0x40, 0xd8, 0xd3, 0x7d, 0x7f, 0x79, 0x30, 0x7d, 0xcc, 0x19, 0x52, 0x71, 0x2c, 0x97,
        0x9b, 0x42, 0x82, 0x2a, 0x4f, 0x65, 0x0a, 0x2b, 0xd4, 0xb0, 0x70, 0x3d, 0xa7, 0x5c, 0x9b,
        0x2c, 0x3c,
    ];

    #[test]
    fn legacy_proofs_rs_byte_untouched() {
        let digest = sha256(include_bytes!("proofs.rs"));
        assert_eq!(digest, EXPECTED_LEGACY_PROOFS_RS_SHA256);
    }

    fn sha256(input: &[u8]) -> [u8; 32] {
        const H0: [u32; 8] = [
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
            0x5be0cd19,
        ];
        const K: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
            0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
            0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
            0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
            0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
            0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
            0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
            0xc67178f2,
        ];

        let mut bytes = input.to_vec();
        let bit_len = (bytes.len() as u64) * 8;
        bytes.push(0x80);
        while bytes.len() % 64 != 56 {
            bytes.push(0);
        }
        bytes.extend_from_slice(&bit_len.to_be_bytes());

        let mut state = H0;
        for chunk in bytes.chunks_exact(64) {
            let mut w = [0u32; 64];
            for (index, word) in chunk.chunks_exact(4).enumerate() {
                w[index] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
            }
            for index in 16..64 {
                let s0 = w[index - 15].rotate_right(7)
                    ^ w[index - 15].rotate_right(18)
                    ^ (w[index - 15] >> 3);
                let s1 = w[index - 2].rotate_right(17)
                    ^ w[index - 2].rotate_right(19)
                    ^ (w[index - 2] >> 10);
                w[index] = w[index - 16]
                    .wrapping_add(s0)
                    .wrapping_add(w[index - 7])
                    .wrapping_add(s1);
            }

            let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
            for index in 0..64 {
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                let ch = (e & f) ^ ((!e) & g);
                let temp1 = h
                    .wrapping_add(s1)
                    .wrapping_add(ch)
                    .wrapping_add(K[index])
                    .wrapping_add(w[index]);
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                let maj = (a & b) ^ (a & c) ^ (b & c);
                let temp2 = s0.wrapping_add(maj);
                h = g;
                g = f;
                f = e;
                e = d.wrapping_add(temp1);
                d = c;
                c = b;
                b = a;
                a = temp1.wrapping_add(temp2);
            }

            for (slot, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
                *slot = slot.wrapping_add(value);
            }
        }

        let mut digest = [0u8; 32];
        for (chunk, word) in digest.chunks_exact_mut(4).zip(state) {
            chunk.copy_from_slice(&word.to_be_bytes());
        }
        digest
    }
}

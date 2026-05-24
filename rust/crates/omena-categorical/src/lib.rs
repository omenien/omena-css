//! Categorical cascade evidence contracts for Omena CSS.
//!
//! This crate is additive: it reads cascade/Lawvere public summaries and emits
//! V0 categorical evidence without changing cascade winner selection.

pub mod beck_chevalley;
pub mod colimit;
pub mod cosheaf;
pub mod design_system_theory;
pub mod kripke;
pub mod modal;
pub mod omega;
pub mod sheaf;
pub mod site;

pub use beck_chevalley::*;
pub use colimit::*;
pub use cosheaf::*;
pub use design_system_theory::*;
pub use kripke::*;
pub use modal::*;
pub use omega::*;
pub use sheaf::*;
pub use site::*;

use serde::Serialize;

pub const CATEGORICAL_SCHEMA_VERSION_V0: &str = "0";
pub const CATEGORICAL_LAYER_MARKER_V0: &str = "categorical-semantic";
pub const CATEGORICAL_FEATURE_GATE_V0: &str = "categorical-evidence";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoricalFoundationSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub module_names: Vec<&'static str>,
    pub top_level_contract_count: usize,
    pub support_contract_count: usize,
    pub proof_primitive_roles: Vec<CascadeProofPrimitiveRoleV0>,
    pub lawvere_dependency_direction: &'static str,
    pub default_feature_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeProofPrimitiveRoleV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub primitive_name: &'static str,
    pub categorical_role: &'static str,
}

pub fn summarize_categorical_foundation_v0() -> CategoricalFoundationSummaryV0 {
    CategoricalFoundationSummaryV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.foundation-summary",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        module_names: vec![
            "site",
            "sheaf",
            "cosheaf",
            "colimit",
            "beck_chevalley",
            "omega",
            "modal",
            "kripke",
            "design_system_theory",
        ],
        top_level_contract_count: 26,
        support_contract_count: 16,
        proof_primitive_roles: cascade_proof_primitive_roles_v0(),
        lawvere_dependency_direction: "omena-categorical -> omena-lawvere",
        default_feature_enabled: false,
    }
}

pub fn cascade_proof_primitive_roles_v0() -> Vec<CascadeProofPrimitiveRoleV0> {
    [
        (
            "prove_layer_flatten_candidate",
            "Beck-Chevalley origin inversion witness",
        ),
        (
            "prove_scope_flatten_candidate",
            "scope stratification morphism witness",
        ),
        (
            "prove_box_shorthand_combination",
            "shorthand invariant functor witness",
        ),
        (
            "evaluate_static_supports_condition",
            "site-axis decidability witness",
        ),
    ]
    .into_iter()
    .map(
        |(primitive_name, categorical_role)| CascadeProofPrimitiveRoleV0 {
            schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
            product: "omena-categorical.proof-primitive-role",
            layer_marker: CATEGORICAL_LAYER_MARKER_V0,
            feature_gate: CATEGORICAL_FEATURE_GATE_V0,
            primitive_name,
            categorical_role,
        },
    )
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_gamma_categorical_surface_without_default_feature() {
        let summary = summarize_categorical_foundation_v0();
        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.layer_marker, "categorical-semantic");
        assert_eq!(summary.module_names.len(), 9);
        assert_eq!(summary.top_level_contract_count, 26);
        assert!(!summary.default_feature_enabled);
        assert_eq!(
            summary.lawvere_dependency_direction,
            "omena-categorical -> omena-lawvere"
        );
    }

    #[test]
    fn maps_actual_cascade_proof_primitives_only() {
        let roles = cascade_proof_primitive_roles_v0();
        let primitive_names = roles
            .iter()
            .map(|role| role.primitive_name)
            .collect::<Vec<_>>();
        assert_eq!(primitive_names.len(), 4);
        assert!(primitive_names.contains(&"prove_box_shorthand_combination"));
        assert!(primitive_names.contains(&"evaluate_static_supports_condition"));
        assert!(!primitive_names.contains(&"cascade_property"));
    }
}

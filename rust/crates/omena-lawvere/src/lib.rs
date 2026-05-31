//! Enriched Lawvere theory contract surface for the transform catalog.
//!
//! This crate is intentionally contract-first. It records the 40-pass catalog,
//! rank clusters, reorderability evidence, and a scaffolded parallel plan
//! without changing the existing transform executor.
//!
//! claim_level: feature-gated differential commutativity witness, not a global
//! transform-catalog theorem or default product mechanism.

use std::collections::BTreeMap;

use omena_transform_cst::{
    TRANSFORM_PASS_CATALOG_LEN, TransformDagEdgeV0, TransformPassKind, all_transform_pass_kinds,
    cascade_safe_obligation, default_transform_dag_edges,
};
use serde::Serialize;

pub const LAWVERE_THEORY_VERSION_V0: &str = "lawvere-css-transform-catalog-v0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AbstractDomainTagV0 {
    SyntaxTrivia,
    TokenValue,
    SelectorShape,
    CascadeStructural,
    SemanticGraph,
    TerminalEmission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LawvereCatalogRoleV0 {
    Generator,
    TerminalForgetfulFunctor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SaturationBudgetTierV0 {
    Full,
    Half,
    Minimal,
}

impl SaturationBudgetTierV0 {
    pub const fn fixture_count(self) -> usize {
        match self {
            Self::Minimal => 10,
            Self::Half => 50,
            Self::Full => 200,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Minimal => "Dev",
            Self::Half => "CI",
            Self::Full => "Nightly",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereGeneratorMetadataV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub theory_version: &'static str,
    pub pass_id: &'static str,
    pub ordinal: u8,
    pub title: &'static str,
    pub catalog_role: LawvereCatalogRoleV0,
    pub abstract_domain_tag: AbstractDomainTagV0,
    pub execution_rank_hint: u32,
    pub terminal_forgetful_functor: bool,
    pub reads_fixed_point: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereEquationClusterV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub execution_rank_hint: u32,
    pub pass_ids: Vec<&'static str>,
    pub generator_count: usize,
    pub saturation_budget_tier: SaturationBudgetTierV0,
    pub theory_version: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereDifferentialCorpusTierV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub theory_version: &'static str,
    pub tier: SaturationBudgetTierV0,
    pub tier_label: &'static str,
    pub fixture_count: usize,
    pub required_pass_rate_percent: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderabilityCertificateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub left_pass_id: &'static str,
    pub right_pass_id: &'static str,
    pub theory_version: &'static str,
    pub differential_tier: SaturationBudgetTierV0,
    pub commute_witness: &'static str,
    pub differential_fixture_count: usize,
    pub differential_equal_fixture_count: usize,
    pub differential_mismatch_count: usize,
    pub specificity_preserved: bool,
    pub computed_value_preserved: bool,
    pub provenance_preserved: bool,
    pub cascade_safe_witness: String,
    pub accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereDifferentialCommutativityCaseV0 {
    pub label: String,
    pub input_css: String,
    pub left_then_right_css: String,
    pub right_then_left_css: String,
    pub left_then_right_mutation_count: usize,
    pub right_then_left_mutation_count: usize,
    pub equal_output: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereDifferentialCommutativityWitnessV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub theory_version: &'static str,
    pub left_pass_id: &'static str,
    pub right_pass_id: &'static str,
    pub fixture_count: usize,
    pub equal_fixture_count: usize,
    pub mismatch_count: usize,
    pub cases: Vec<LawvereDifferentialCommutativityCaseV0>,
    pub accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassParallelPlanV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub scheduler_status: &'static str,
    pub requested_pass_ids: Vec<&'static str>,
    pub terminal_pass_ids: Vec<&'static str>,
    pub rank_clusters: Vec<LawvereEquationClusterV0>,
    pub executor_consumes_plan: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereModelTraceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub theory_version: &'static str,
    pub input_pass_ids: Vec<&'static str>,
    pub ordered_pass_ids: Vec<&'static str>,
    pub terminal_pass_ids: Vec<&'static str>,
    pub rank_clusters: Vec<LawvereEquationClusterV0>,
    pub preserves_existing_executor_signature: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereSaturationExecutionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub theory_version: &'static str,
    pub pass_id: &'static str,
    pub analysis_slot: &'static str,
    pub original_unit_analysis_path_preserved: bool,
    pub differential_tier: SaturationBudgetTierV0,
    pub differential_fixture_count: usize,
    pub iteration_limit: usize,
    pub iteration_count: usize,
    pub eclass_count: usize,
    pub enode_count: usize,
    pub accepted: bool,
    pub extracted_matches_candidate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereTheorySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub theory_version: &'static str,
    pub catalog_pass_count: usize,
    pub catalog_entry_count: usize,
    pub lawvere_generator_count: usize,
    pub terminal_forgetful_functor_count: usize,
    pub execution_rank_cluster_count: usize,
    pub equation_clusters: Vec<LawvereEquationClusterV0>,
    pub generators: Vec<LawvereGeneratorMetadataV0>,
    pub dag_edges: Vec<TransformDagEdgeV0>,
    pub saturation_budget_tiers: Vec<SaturationBudgetTierV0>,
    pub differential_corpus_tiers: Vec<LawvereDifferentialCorpusTierV0>,
    pub lawvere_saturation_feature_enabled_by_default: bool,
    pub product_path_evidence_ready: bool,
    pub mechanism_scope: &'static str,
    pub omena_categorical_dependency_forbidden: bool,
}

pub fn summarize_lawvere_theory_v0() -> LawvereTheorySummaryV0 {
    let generators = lawvere_generator_metadata_catalog_v0();
    let terminal_forgetful_functor_count = generators
        .iter()
        .filter(|generator| generator.terminal_forgetful_functor)
        .count();
    let equation_clusters = lawvere_equation_clusters_v0(
        generators
            .iter()
            .map(|generator| generator.pass_id)
            .collect::<Vec<_>>()
            .as_slice(),
    );

    LawvereTheorySummaryV0 {
        schema_version: "0",
        product: "omena-lawvere.theory-summary",
        layer_marker: "enriched-algebraic",
        feature_gate: "lawvere-saturation",
        theory_version: LAWVERE_THEORY_VERSION_V0,
        catalog_pass_count: TRANSFORM_PASS_CATALOG_LEN,
        catalog_entry_count: generators.len(),
        lawvere_generator_count: lawvere_theory_generator_count_v0(&generators),
        terminal_forgetful_functor_count,
        execution_rank_cluster_count: equation_clusters.len(),
        equation_clusters,
        generators,
        dag_edges: default_transform_dag_edges(),
        saturation_budget_tiers: vec![
            SaturationBudgetTierV0::Minimal,
            SaturationBudgetTierV0::Half,
            SaturationBudgetTierV0::Full,
        ],
        differential_corpus_tiers: lawvere_differential_corpus_tiers_v0(),
        lawvere_saturation_feature_enabled_by_default: false,
        product_path_evidence_ready: false,
        mechanism_scope: "featureGatedResearchSubstrate",
        omena_categorical_dependency_forbidden: true,
    }
}

pub fn lawvere_generator_metadata_catalog_v0() -> Vec<LawvereGeneratorMetadataV0> {
    all_transform_pass_kinds()
        .into_iter()
        .map(lawvere_generator_metadata_v0)
        .collect()
}

pub fn lawvere_generator_metadata_v0(kind: TransformPassKind) -> LawvereGeneratorMetadataV0 {
    let terminal_forgetful_functor = kind == TransformPassKind::PrintCss;
    LawvereGeneratorMetadataV0 {
        schema_version: "0",
        product: "omena-lawvere.generator-metadata",
        layer_marker: "enriched-algebraic",
        feature_gate: "lawvere-saturation",
        theory_version: LAWVERE_THEORY_VERSION_V0,
        pass_id: kind.id(),
        ordinal: kind.ordinal(),
        title: kind.title(),
        catalog_role: if terminal_forgetful_functor {
            LawvereCatalogRoleV0::TerminalForgetfulFunctor
        } else {
            LawvereCatalogRoleV0::Generator
        },
        abstract_domain_tag: abstract_domain_tag_for_pass(kind),
        execution_rank_hint: u32::from(lawvere_execution_rank_hint(kind)),
        terminal_forgetful_functor,
        reads_fixed_point: matches!(
            kind,
            TransformPassKind::StaticVarSubstitution
                | TransformPassKind::TreeShakeCustomProperty
                | TransformPassKind::DesignTokenRouting
        ),
    }
}

pub fn lawvere_equation_clusters_v0(pass_ids: &[&'static str]) -> Vec<LawvereEquationClusterV0> {
    let mut clusters = BTreeMap::<u32, Vec<&'static str>>::new();
    for kind in all_transform_pass_kinds() {
        if pass_ids.contains(&kind.id())
            && lawvere_catalog_role_v0(kind) == LawvereCatalogRoleV0::Generator
        {
            clusters
                .entry(u32::from(lawvere_execution_rank_hint(kind)))
                .or_default()
                .push(kind.id());
        }
    }
    clusters
        .into_iter()
        .map(|(execution_rank_hint, mut pass_ids)| {
            pass_ids.sort();
            let generator_count = pass_ids.len();
            LawvereEquationClusterV0 {
                schema_version: "0",
                product: "omena-lawvere.equation-cluster",
                layer_marker: "enriched-algebraic",
                feature_gate: "lawvere-saturation",
                execution_rank_hint,
                pass_ids,
                generator_count,
                saturation_budget_tier: budget_tier_for_cluster_size(generator_count),
                theory_version: LAWVERE_THEORY_VERSION_V0,
            }
        })
        .collect()
}

pub fn plan_transform_pass_parallel_layers_v0(
    requested: &[TransformPassKind],
) -> TransformPassParallelPlanV0 {
    let requested_pass_ids = requested.iter().map(|kind| kind.id()).collect::<Vec<_>>();
    TransformPassParallelPlanV0 {
        schema_version: "0",
        product: "omena-lawvere.transform-pass-parallel-plan",
        layer_marker: "enriched-algebraic",
        feature_gate: "lawvere-saturation",
        scheduler_status: "scaffoldOnly",
        requested_pass_ids: requested_pass_ids.clone(),
        terminal_pass_ids: terminal_pass_ids_from_pass_kinds(requested),
        rank_clusters: lawvere_equation_clusters_v0(requested_pass_ids.as_slice()),
        executor_consumes_plan: false,
    }
}

pub fn trace_lawvere_model_v0(
    requested: &[TransformPassKind],
    ordered_pass_ids: Vec<&'static str>,
) -> LawvereModelTraceV0 {
    let input_pass_ids = requested.iter().map(|kind| kind.id()).collect::<Vec<_>>();
    LawvereModelTraceV0 {
        schema_version: "0",
        product: "omena-lawvere.model-trace",
        layer_marker: "enriched-algebraic",
        feature_gate: "lawvere-saturation",
        theory_version: LAWVERE_THEORY_VERSION_V0,
        rank_clusters: lawvere_equation_clusters_v0(ordered_pass_ids.as_slice()),
        input_pass_ids,
        terminal_pass_ids: terminal_pass_ids_from_pass_ids(ordered_pass_ids.as_slice()),
        ordered_pass_ids,
        preserves_existing_executor_signature: true,
    }
}

pub fn reorderability_certificate_v0(
    left: TransformPassKind,
    right: TransformPassKind,
) -> ReorderabilityCertificateV0 {
    ReorderabilityCertificateV0 {
        schema_version: "0",
        product: "omena-lawvere.reorderability-certificate",
        layer_marker: "enriched-algebraic",
        feature_gate: "lawvere-saturation",
        left_pass_id: left.id(),
        right_pass_id: right.id(),
        theory_version: LAWVERE_THEORY_VERSION_V0,
        differential_tier: budget_tier_for_cluster_size(2),
        commute_witness: "requiresDifferentialCommutativityWitness",
        differential_fixture_count: 0,
        differential_equal_fixture_count: 0,
        differential_mismatch_count: 0,
        specificity_preserved: false,
        computed_value_preserved: false,
        provenance_preserved: false,
        cascade_safe_witness: format!(
            "{}:{}",
            cascade_safe_obligation(left),
            cascade_safe_obligation(right)
        ),
        accepted: false,
    }
}

pub fn lawvere_differential_commutativity_witness_v0(
    left: TransformPassKind,
    right: TransformPassKind,
    cases: Vec<LawvereDifferentialCommutativityCaseV0>,
) -> LawvereDifferentialCommutativityWitnessV0 {
    let fixture_count = cases.len();
    let equal_fixture_count = cases.iter().filter(|case| case.equal_output).count();
    let mismatch_count = fixture_count.saturating_sub(equal_fixture_count);

    LawvereDifferentialCommutativityWitnessV0 {
        schema_version: "0",
        product: "omena-lawvere.differential-commutativity-witness",
        layer_marker: "enriched-algebraic",
        feature_gate: "lawvere-saturation",
        theory_version: LAWVERE_THEORY_VERSION_V0,
        left_pass_id: left.id(),
        right_pass_id: right.id(),
        fixture_count,
        equal_fixture_count,
        mismatch_count,
        cases,
        accepted: fixture_count > 0 && mismatch_count == 0,
    }
}

pub fn reorderability_certificate_from_differential_v0(
    left: TransformPassKind,
    right: TransformPassKind,
    witness: &LawvereDifferentialCommutativityWitnessV0,
) -> ReorderabilityCertificateV0 {
    let mut certificate = reorderability_certificate_v0(left, right);
    certificate.commute_witness = "differentialCommutativityCorpus";
    certificate.differential_fixture_count = witness.fixture_count;
    certificate.differential_equal_fixture_count = witness.equal_fixture_count;
    certificate.differential_mismatch_count = witness.mismatch_count;
    certificate.specificity_preserved = witness.accepted;
    certificate.computed_value_preserved = witness.accepted;
    certificate.provenance_preserved = witness.accepted;
    certificate.accepted = witness.accepted;
    certificate
}

pub fn lawvere_differential_corpus_tiers_v0() -> Vec<LawvereDifferentialCorpusTierV0> {
    [
        SaturationBudgetTierV0::Minimal,
        SaturationBudgetTierV0::Half,
        SaturationBudgetTierV0::Full,
    ]
    .into_iter()
    .map(|tier| LawvereDifferentialCorpusTierV0 {
        schema_version: "0",
        product: "omena-lawvere.differential-corpus-tier",
        layer_marker: "enriched-algebraic",
        feature_gate: "lawvere-saturation",
        theory_version: LAWVERE_THEORY_VERSION_V0,
        tier,
        tier_label: tier.label(),
        fixture_count: tier.fixture_count(),
        required_pass_rate_percent: 100,
    })
    .collect()
}

pub fn summarize_lawvere_saturation_execution_v0(
    pass_id: &'static str,
    iteration_limit: usize,
    iteration_count: usize,
    eclass_count: usize,
    enode_count: usize,
    extracted_matches_candidate: bool,
) -> LawvereSaturationExecutionV0 {
    LawvereSaturationExecutionV0 {
        schema_version: "0",
        product: "omena-lawvere.saturation-execution",
        layer_marker: "enriched-algebraic",
        feature_gate: "lawvere-saturation",
        theory_version: LAWVERE_THEORY_VERSION_V0,
        pass_id,
        analysis_slot: "LawvereAnalysis",
        original_unit_analysis_path_preserved: true,
        differential_tier: SaturationBudgetTierV0::Minimal,
        differential_fixture_count: SaturationBudgetTierV0::Minimal.fixture_count(),
        iteration_limit,
        iteration_count,
        eclass_count,
        enode_count,
        accepted: extracted_matches_candidate,
        extracted_matches_candidate,
    }
}

pub const fn lawvere_execution_rank_hint(kind: TransformPassKind) -> u8 {
    match kind.ordinal() {
        26..=28 => 10,
        29..=39 => 20,
        14..=24 => 30,
        8..=13 | 25 => 40,
        1..=7 => 50,
        40 => 60,
        _ => 70,
    }
}

pub const fn lawvere_catalog_role_v0(kind: TransformPassKind) -> LawvereCatalogRoleV0 {
    match kind {
        TransformPassKind::PrintCss => LawvereCatalogRoleV0::TerminalForgetfulFunctor,
        _ => LawvereCatalogRoleV0::Generator,
    }
}

fn lawvere_theory_generator_count_v0(generators: &[LawvereGeneratorMetadataV0]) -> usize {
    generators
        .iter()
        .filter(|generator| generator.catalog_role == LawvereCatalogRoleV0::Generator)
        .count()
}

fn terminal_pass_ids_from_pass_kinds(requested: &[TransformPassKind]) -> Vec<&'static str> {
    requested
        .iter()
        .filter(|kind| {
            lawvere_catalog_role_v0(**kind) == LawvereCatalogRoleV0::TerminalForgetfulFunctor
        })
        .map(|kind| kind.id())
        .collect()
}

fn terminal_pass_ids_from_pass_ids(pass_ids: &[&'static str]) -> Vec<&'static str> {
    all_transform_pass_kinds()
        .into_iter()
        .filter(|kind| {
            lawvere_catalog_role_v0(*kind) == LawvereCatalogRoleV0::TerminalForgetfulFunctor
        })
        .map(|kind| kind.id())
        .filter(|pass_id| pass_ids.contains(pass_id))
        .collect()
}

const fn abstract_domain_tag_for_pass(kind: TransformPassKind) -> AbstractDomainTagV0 {
    match kind.ordinal() {
        1..=7 => AbstractDomainTagV0::TokenValue,
        8..=13 | 25 => AbstractDomainTagV0::SelectorShape,
        14..=24 => AbstractDomainTagV0::CascadeStructural,
        26..=39 => AbstractDomainTagV0::SemanticGraph,
        40 => AbstractDomainTagV0::TerminalEmission,
        _ => AbstractDomainTagV0::SyntaxTrivia,
    }
}

const fn budget_tier_for_cluster_size(size: usize) -> SaturationBudgetTierV0 {
    if size >= 10 {
        SaturationBudgetTierV0::Full
    } else if size >= 4 {
        SaturationBudgetTierV0::Half
    } else {
        SaturationBudgetTierV0::Minimal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_forty_pass_lawvere_catalog_with_schema_zero() {
        let summary = summarize_lawvere_theory_v0();

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.layer_marker, "enriched-algebraic");
        assert_eq!(summary.feature_gate, "lawvere-saturation");
        assert_eq!(summary.catalog_pass_count, TRANSFORM_PASS_CATALOG_LEN);
        assert_eq!(summary.catalog_entry_count, TRANSFORM_PASS_CATALOG_LEN);
        assert_eq!(
            summary.lawvere_generator_count,
            TRANSFORM_PASS_CATALOG_LEN - 1
        );
        assert_eq!(summary.terminal_forgetful_functor_count, 1);
        assert_eq!(summary.differential_corpus_tiers.len(), 3);
        assert!(summary.differential_corpus_tiers.iter().any(|tier| {
            tier.tier == SaturationBudgetTierV0::Minimal && tier.fixture_count == 10
        }));
        assert!(
            summary.differential_corpus_tiers.iter().any(|tier| {
                tier.tier == SaturationBudgetTierV0::Half && tier.fixture_count == 50
            })
        );
        assert!(summary.differential_corpus_tiers.iter().any(|tier| {
            tier.tier == SaturationBudgetTierV0::Full && tier.fixture_count == 200
        }));
        assert!(!summary.lawvere_saturation_feature_enabled_by_default);
        assert!(!summary.product_path_evidence_ready);
        assert_eq!(summary.mechanism_scope, "featureGatedResearchSubstrate");
        assert!(summary.omena_categorical_dependency_forbidden);
    }

    #[test]
    fn execution_rank_hint_clusters_match_planner_promote_pattern() {
        let metadata = lawvere_generator_metadata_catalog_v0();

        assert_eq!(metadata.len(), TRANSFORM_PASS_CATALOG_LEN);
        assert!(metadata.iter().any(|generator| {
            generator.pass_id == "css-modules-class-hashing" && generator.execution_rank_hint == 20
        }));
        assert!(metadata.iter().any(|generator| {
            generator.pass_id == "print-css"
                && generator.catalog_role == LawvereCatalogRoleV0::TerminalForgetfulFunctor
                && generator.terminal_forgetful_functor
                && generator.execution_rank_hint == 60
        }));
    }

    #[test]
    fn parallel_plan_is_scaffold_only_and_does_not_consume_executor() {
        let plan = plan_transform_pass_parallel_layers_v0(&[
            TransformPassKind::ColorCompression,
            TransformPassKind::NumberCompression,
            TransformPassKind::PrintCss,
        ]);

        assert_eq!(plan.schema_version, "0");
        assert_eq!(plan.scheduler_status, "scaffoldOnly");
        assert!(!plan.executor_consumes_plan);
        assert_eq!(plan.terminal_pass_ids, vec!["print-css"]);
        assert_eq!(plan.rank_clusters.len(), 1);
    }

    #[test]
    fn saturation_execution_contract_records_lawvere_analysis_slot() {
        let execution = summarize_lawvere_saturation_execution_v0(
            TransformPassKind::CalcReduction.id(),
            8,
            2,
            5,
            9,
            true,
        );

        assert_eq!(execution.schema_version, "0");
        assert_eq!(execution.layer_marker, "enriched-algebraic");
        assert_eq!(execution.feature_gate, "lawvere-saturation");
        assert_eq!(execution.analysis_slot, "LawvereAnalysis");
        assert_eq!(execution.differential_fixture_count, 10);
        assert!(execution.original_unit_analysis_path_preserved);
        assert!(execution.accepted);
    }

    #[test]
    fn rank_only_reorderability_certificate_requires_differential_witness() {
        let certificate = reorderability_certificate_v0(
            TransformPassKind::CommentStrip,
            TransformPassKind::WhitespaceStrip,
        );

        assert_eq!(
            certificate.commute_witness,
            "requiresDifferentialCommutativityWitness"
        );
        assert_eq!(certificate.differential_fixture_count, 0);
        assert!(!certificate.accepted);
    }

    #[test]
    fn differential_reorderability_certificate_accepts_only_equal_output_corpus() {
        let witness = lawvere_differential_commutativity_witness_v0(
            TransformPassKind::CommentStrip,
            TransformPassKind::WhitespaceStrip,
            vec![LawvereDifferentialCommutativityCaseV0 {
                label: "comment-whitespace".to_string(),
                input_css: ".a { color : red ; /* x */ }".to_string(),
                left_then_right_css: ".a{color:red}".to_string(),
                right_then_left_css: ".a{color:red}".to_string(),
                left_then_right_mutation_count: 2,
                right_then_left_mutation_count: 2,
                equal_output: true,
            }],
        );
        let certificate = reorderability_certificate_from_differential_v0(
            TransformPassKind::CommentStrip,
            TransformPassKind::WhitespaceStrip,
            &witness,
        );

        assert!(witness.accepted);
        assert_eq!(
            certificate.commute_witness,
            "differentialCommutativityCorpus"
        );
        assert_eq!(certificate.differential_fixture_count, 1);
        assert_eq!(certificate.differential_mismatch_count, 0);
        assert!(certificate.accepted);
    }
}

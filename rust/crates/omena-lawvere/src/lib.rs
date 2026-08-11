//! Transform-pass catalog and reorderability metadata scaffold.
//!
//! This crate is intentionally contract-first. It records the 40-pass catalog,
//! rank clusters, reorderability evidence, and a scaffolded parallel plan
//! without changing the existing transform executor.
//!
//! claim_level: feature-gated differential commutativity witness, not a global
//! transform-catalog theorem or default product mechanism.

use std::collections::BTreeMap;

use omena_evidence_graph::ObligationFamilyIdV0;
use omena_transform_cst::{
    TRANSFORM_PASS_CATALOG_LEN, TransformDagEdgeV0, TransformPassKind, all_transform_pass_kinds,
    cascade_safe_obligation, default_transform_dag_edges,
};
use serde::Serialize;

pub const TRANSFORM_CATALOG_SCHEMA_VERSION_V0: &str = "css-transform-catalog-v0";
pub const TRANSFORM_CATALOG_MECHANISM_SCOPE_V0: &str = "featureGatedDifferentialWitnessSubstrate";
pub const TRANSFORM_CATALOG_PRODUCT_PATH_EVIDENCE_READY_V0: bool = false;
pub const TRANSFORM_CATALOG_GLOBAL_TRANSFORM_THEOREM_CLAIMED_V0: bool = false;

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
pub enum TransformCatalogRoleV0 {
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
pub struct TransformCatalogGeneratorMetadataV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub theory_version: &'static str,
    pub pass_id: &'static str,
    pub ordinal: u8,
    pub title: &'static str,
    pub catalog_role: TransformCatalogRoleV0,
    pub abstract_domain_tag: AbstractDomainTagV0,
    pub execution_rank_hint: u32,
    pub terminal_forgetful_functor: bool,
    pub reads_fixed_point: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCatalogEquationClusterV0 {
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
pub struct TransformCatalogDifferentialCorpusTierV0 {
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
    pub mechanism_scope: &'static str,
    pub product_path_evidence_ready: bool,
    pub global_transform_theorem_claimed: bool,
    pub left_pass_id: &'static str,
    pub right_pass_id: &'static str,
    pub theory_version: &'static str,
    pub differential_tier: SaturationBudgetTierV0,
    pub commute_witness: &'static str,
    pub differential_fixture_count: usize,
    pub differential_equal_fixture_count: usize,
    pub differential_mismatch_count: usize,
    pub specificity_preserved: bool,
    #[serde(skip_serializing)]
    obligation_family: ObligationFamilyIdV0,
    pub computed_value_preserved: bool,
    pub provenance_preserved: bool,
    pub cascade_safe_witness: String,
    pub accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCatalogDifferentialCommutativityCaseV0 {
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
pub struct TransformCatalogDifferentialCommutativityWitnessV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub mechanism_scope: &'static str,
    pub product_path_evidence_ready: bool,
    pub global_transform_theorem_claimed: bool,
    pub theory_version: &'static str,
    pub left_pass_id: &'static str,
    pub right_pass_id: &'static str,
    pub fixture_count: usize,
    pub equal_fixture_count: usize,
    pub mismatch_count: usize,
    pub cases: Vec<TransformCatalogDifferentialCommutativityCaseV0>,
    pub accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCatalogTransformPassParallelPlanV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub mechanism_scope: &'static str,
    pub product_path_evidence_ready: bool,
    pub global_transform_theorem_claimed: bool,
    pub scheduler_status: &'static str,
    pub requested_pass_ids: Vec<&'static str>,
    pub terminal_pass_ids: Vec<&'static str>,
    pub rank_clusters: Vec<TransformCatalogEquationClusterV0>,
    pub executor_consumes_plan: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCatalogModelTraceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub mechanism_scope: &'static str,
    pub product_path_evidence_ready: bool,
    pub global_transform_theorem_claimed: bool,
    pub theory_version: &'static str,
    pub input_pass_ids: Vec<&'static str>,
    pub ordered_pass_ids: Vec<&'static str>,
    pub terminal_pass_ids: Vec<&'static str>,
    pub rank_clusters: Vec<TransformCatalogEquationClusterV0>,
    pub preserves_existing_executor_signature: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCatalogSaturationExecutionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub mechanism_scope: &'static str,
    pub product_path_evidence_ready: bool,
    pub global_transform_theorem_claimed: bool,
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
pub struct TransformCatalogMetadataSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub theory_version: &'static str,
    pub catalog_pass_count: usize,
    pub catalog_entry_count: usize,
    /// Compatibility field owned by `omena-lawvere` maintainers. Remove not
    /// before 1.0, after downstream migration and zero audited non-compat uses.
    #[deprecated(
        since = "0.4.0",
        note = "use transform_catalog_generator_count(); removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
    )]
    pub lawvere_generator_count: usize,
    pub terminal_forgetful_functor_count: usize,
    pub execution_rank_cluster_count: usize,
    pub equation_clusters: Vec<TransformCatalogEquationClusterV0>,
    pub generators: Vec<TransformCatalogGeneratorMetadataV0>,
    pub dag_edges: Vec<TransformDagEdgeV0>,
    pub saturation_budget_tiers: Vec<SaturationBudgetTierV0>,
    pub differential_corpus_tiers: Vec<TransformCatalogDifferentialCorpusTierV0>,
    /// Compatibility field owned by `omena-lawvere` maintainers. Remove not
    /// before 1.0, after downstream migration and zero audited non-compat uses.
    #[deprecated(
        since = "0.4.0",
        note = "use transform_catalog_saturation_feature_enabled_by_default(); removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
    )]
    pub lawvere_saturation_feature_enabled_by_default: bool,
    pub product_path_evidence_ready: bool,
    pub mechanism_scope: &'static str,
    pub omena_categorical_dependency_forbidden: bool,
}

impl TransformCatalogMetadataSummaryV0 {
    #[allow(deprecated)]
    pub fn transform_catalog_generator_count(&self) -> usize {
        transform_catalog_generator_count_from_legacy_field_v0(self)
    }

    #[allow(deprecated)]
    pub fn transform_catalog_saturation_feature_enabled_by_default(&self) -> bool {
        transform_catalog_saturation_default_from_legacy_field_v0(self)
    }
}

#[deprecated(
    since = "0.4.0",
    note = "compatibility field adapter owned by omena-lawvere maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
fn transform_catalog_generator_count_from_legacy_field_v0(
    summary: &TransformCatalogMetadataSummaryV0,
) -> usize {
    summary.lawvere_generator_count
}

#[deprecated(
    since = "0.4.0",
    note = "compatibility field adapter owned by omena-lawvere maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
fn transform_catalog_saturation_default_from_legacy_field_v0(
    summary: &TransformCatalogMetadataSummaryV0,
) -> bool {
    summary.lawvere_saturation_feature_enabled_by_default
}

#[allow(deprecated)]
pub fn summarize_transform_catalog_metadata_v0() -> TransformCatalogMetadataSummaryV0 {
    let generators = transform_catalog_generator_metadata_catalog_v0();
    let terminal_forgetful_functor_count = generators
        .iter()
        .filter(|generator| generator.terminal_forgetful_functor)
        .count();
    let equation_clusters = transform_catalog_equation_clusters_v0(
        generators
            .iter()
            .map(|generator| generator.pass_id)
            .collect::<Vec<_>>()
            .as_slice(),
    );

    build_transform_catalog_metadata_summary_v0(
        generators,
        terminal_forgetful_functor_count,
        equation_clusters,
    )
}

#[deprecated(
    since = "0.4.0",
    note = "constructs retained serialized fields; owned by omena-lawvere maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
fn build_transform_catalog_metadata_summary_v0(
    generators: Vec<TransformCatalogGeneratorMetadataV0>,
    terminal_forgetful_functor_count: usize,
    equation_clusters: Vec<TransformCatalogEquationClusterV0>,
) -> TransformCatalogMetadataSummaryV0 {
    TransformCatalogMetadataSummaryV0 {
        schema_version: "0",
        product: "omena-lawvere.theory-summary",
        layer_marker: "enriched-algebraic",
        feature_gate: "transform-catalog-saturation",
        theory_version: TRANSFORM_CATALOG_SCHEMA_VERSION_V0,
        catalog_pass_count: TRANSFORM_PASS_CATALOG_LEN,
        catalog_entry_count: generators.len(),
        lawvere_generator_count: transform_catalog_generator_count_v0(&generators),
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
        differential_corpus_tiers: transform_catalog_differential_corpus_tiers_v0(),
        lawvere_saturation_feature_enabled_by_default: false,
        product_path_evidence_ready: TRANSFORM_CATALOG_PRODUCT_PATH_EVIDENCE_READY_V0,
        mechanism_scope: TRANSFORM_CATALOG_MECHANISM_SCOPE_V0,
        omena_categorical_dependency_forbidden: true,
    }
}

pub fn transform_catalog_generator_metadata_catalog_v0() -> Vec<TransformCatalogGeneratorMetadataV0>
{
    all_transform_pass_kinds()
        .into_iter()
        .map(transform_catalog_generator_metadata_v0)
        .collect()
}

pub fn transform_catalog_generator_metadata_v0(
    kind: TransformPassKind,
) -> TransformCatalogGeneratorMetadataV0 {
    let terminal_forgetful_functor = kind == TransformPassKind::PrintCss;
    TransformCatalogGeneratorMetadataV0 {
        schema_version: "0",
        product: "omena-lawvere.generator-metadata",
        layer_marker: "enriched-algebraic",
        feature_gate: "transform-catalog-saturation",
        theory_version: TRANSFORM_CATALOG_SCHEMA_VERSION_V0,
        pass_id: kind.id(),
        ordinal: kind.ordinal(),
        title: kind.title(),
        catalog_role: if terminal_forgetful_functor {
            TransformCatalogRoleV0::TerminalForgetfulFunctor
        } else {
            TransformCatalogRoleV0::Generator
        },
        abstract_domain_tag: abstract_domain_tag_for_pass(kind),
        execution_rank_hint: u32::from(transform_catalog_execution_rank_hint(kind)),
        terminal_forgetful_functor,
        reads_fixed_point: matches!(
            kind,
            TransformPassKind::StaticVarSubstitution
                | TransformPassKind::TreeShakeCustomProperty
                | TransformPassKind::DesignTokenRouting
        ),
    }
}

pub fn transform_catalog_equation_clusters_v0(
    pass_ids: &[&'static str],
) -> Vec<TransformCatalogEquationClusterV0> {
    let mut clusters = BTreeMap::<u32, Vec<&'static str>>::new();
    for kind in all_transform_pass_kinds() {
        if pass_ids.contains(&kind.id())
            && transform_catalog_catalog_role_v0(kind) == TransformCatalogRoleV0::Generator
        {
            clusters
                .entry(u32::from(transform_catalog_execution_rank_hint(kind)))
                .or_default()
                .push(kind.id());
        }
    }
    clusters
        .into_iter()
        .map(|(execution_rank_hint, mut pass_ids)| {
            pass_ids.sort();
            let generator_count = pass_ids.len();
            TransformCatalogEquationClusterV0 {
                schema_version: "0",
                product: "omena-lawvere.equation-cluster",
                layer_marker: "enriched-algebraic",
                feature_gate: "transform-catalog-saturation",
                execution_rank_hint,
                pass_ids,
                generator_count,
                saturation_budget_tier: budget_tier_for_cluster_size(generator_count),
                theory_version: TRANSFORM_CATALOG_SCHEMA_VERSION_V0,
            }
        })
        .collect()
}

pub fn plan_transform_catalog_parallel_layers_v0(
    requested: &[TransformPassKind],
) -> TransformCatalogTransformPassParallelPlanV0 {
    let requested_pass_ids = requested.iter().map(|kind| kind.id()).collect::<Vec<_>>();
    TransformCatalogTransformPassParallelPlanV0 {
        schema_version: "0",
        product: "omena-lawvere.transform-pass-parallel-plan",
        layer_marker: "enriched-algebraic",
        feature_gate: "transform-catalog-saturation",
        mechanism_scope: TRANSFORM_CATALOG_MECHANISM_SCOPE_V0,
        product_path_evidence_ready: TRANSFORM_CATALOG_PRODUCT_PATH_EVIDENCE_READY_V0,
        global_transform_theorem_claimed: TRANSFORM_CATALOG_GLOBAL_TRANSFORM_THEOREM_CLAIMED_V0,
        scheduler_status: "scaffoldOnly",
        requested_pass_ids: requested_pass_ids.clone(),
        terminal_pass_ids: terminal_pass_ids_from_pass_kinds(requested),
        rank_clusters: transform_catalog_equation_clusters_v0(requested_pass_ids.as_slice()),
        executor_consumes_plan: false,
    }
}

pub fn trace_transform_catalog_model_v0(
    requested: &[TransformPassKind],
    ordered_pass_ids: Vec<&'static str>,
) -> TransformCatalogModelTraceV0 {
    let input_pass_ids = requested.iter().map(|kind| kind.id()).collect::<Vec<_>>();
    TransformCatalogModelTraceV0 {
        schema_version: "0",
        product: "omena-lawvere.model-trace",
        layer_marker: "enriched-algebraic",
        feature_gate: "transform-catalog-saturation",
        mechanism_scope: TRANSFORM_CATALOG_MECHANISM_SCOPE_V0,
        product_path_evidence_ready: TRANSFORM_CATALOG_PRODUCT_PATH_EVIDENCE_READY_V0,
        global_transform_theorem_claimed: TRANSFORM_CATALOG_GLOBAL_TRANSFORM_THEOREM_CLAIMED_V0,
        theory_version: TRANSFORM_CATALOG_SCHEMA_VERSION_V0,
        rank_clusters: transform_catalog_equation_clusters_v0(ordered_pass_ids.as_slice()),
        input_pass_ids,
        terminal_pass_ids: terminal_pass_ids_from_pass_ids(ordered_pass_ids.as_slice()),
        ordered_pass_ids,
        preserves_existing_executor_signature: true,
    }
}

pub fn transform_catalog_reorderability_certificate_v0(
    left: TransformPassKind,
    right: TransformPassKind,
) -> ReorderabilityCertificateV0 {
    ReorderabilityCertificateV0 {
        schema_version: "0",
        product: "omena-lawvere.reorderability-certificate",
        layer_marker: "enriched-algebraic",
        feature_gate: "transform-catalog-saturation",
        mechanism_scope: TRANSFORM_CATALOG_MECHANISM_SCOPE_V0,
        product_path_evidence_ready: TRANSFORM_CATALOG_PRODUCT_PATH_EVIDENCE_READY_V0,
        global_transform_theorem_claimed: TRANSFORM_CATALOG_GLOBAL_TRANSFORM_THEOREM_CLAIMED_V0,
        left_pass_id: left.id(),
        right_pass_id: right.id(),
        theory_version: TRANSFORM_CATALOG_SCHEMA_VERSION_V0,
        differential_tier: budget_tier_for_cluster_size(2),
        commute_witness: "requiresDifferentialCommutativityWitness",
        differential_fixture_count: 0,
        differential_equal_fixture_count: 0,
        differential_mismatch_count: 0,
        specificity_preserved: false,
        obligation_family: ObligationFamilyIdV0::CascadeSafetyFloor,
        computed_value_preserved: ObligationFamilyIdV0::CascadeSafetyFloor
            .preserves_computed_value(),
        provenance_preserved: false,
        cascade_safe_witness: format!(
            "{}:{}",
            cascade_safe_obligation(left),
            cascade_safe_obligation(right)
        ),
        accepted: false,
    }
}

pub fn transform_catalog_differential_commutativity_witness_v0(
    left: TransformPassKind,
    right: TransformPassKind,
    cases: Vec<TransformCatalogDifferentialCommutativityCaseV0>,
) -> TransformCatalogDifferentialCommutativityWitnessV0 {
    let fixture_count = cases.len();
    let equal_fixture_count = cases.iter().filter(|case| case.equal_output).count();
    let mismatch_count = fixture_count.saturating_sub(equal_fixture_count);

    TransformCatalogDifferentialCommutativityWitnessV0 {
        schema_version: "0",
        product: "omena-lawvere.differential-commutativity-witness",
        layer_marker: "enriched-algebraic",
        feature_gate: "transform-catalog-saturation",
        mechanism_scope: TRANSFORM_CATALOG_MECHANISM_SCOPE_V0,
        product_path_evidence_ready: TRANSFORM_CATALOG_PRODUCT_PATH_EVIDENCE_READY_V0,
        global_transform_theorem_claimed: TRANSFORM_CATALOG_GLOBAL_TRANSFORM_THEOREM_CLAIMED_V0,
        theory_version: TRANSFORM_CATALOG_SCHEMA_VERSION_V0,
        left_pass_id: left.id(),
        right_pass_id: right.id(),
        fixture_count,
        equal_fixture_count,
        mismatch_count,
        cases,
        accepted: fixture_count > 0 && mismatch_count == 0,
    }
}

pub fn transform_catalog_reorderability_certificate_from_differential_v0(
    left: TransformPassKind,
    right: TransformPassKind,
    witness: &TransformCatalogDifferentialCommutativityWitnessV0,
) -> ReorderabilityCertificateV0 {
    let mut certificate = transform_catalog_reorderability_certificate_v0(left, right);
    certificate.commute_witness = "differentialCommutativityCorpus";
    certificate.differential_fixture_count = witness.fixture_count;
    certificate.differential_equal_fixture_count = witness.equal_fixture_count;
    certificate.differential_mismatch_count = witness.mismatch_count;
    certificate.specificity_preserved = witness.accepted;
    certificate.obligation_family =
        ObligationFamilyIdV0::from_computed_value_preservation(witness.accepted);
    certificate.computed_value_preserved = certificate.obligation_family.preserves_computed_value();
    certificate.provenance_preserved = witness.accepted;
    certificate.accepted = witness.accepted;
    certificate
}

pub fn transform_catalog_differential_corpus_tiers_v0()
-> Vec<TransformCatalogDifferentialCorpusTierV0> {
    [
        SaturationBudgetTierV0::Minimal,
        SaturationBudgetTierV0::Half,
        SaturationBudgetTierV0::Full,
    ]
    .into_iter()
    .map(|tier| TransformCatalogDifferentialCorpusTierV0 {
        schema_version: "0",
        product: "omena-lawvere.differential-corpus-tier",
        layer_marker: "enriched-algebraic",
        feature_gate: "transform-catalog-saturation",
        theory_version: TRANSFORM_CATALOG_SCHEMA_VERSION_V0,
        tier,
        tier_label: tier.label(),
        fixture_count: tier.fixture_count(),
        required_pass_rate_percent: 100,
    })
    .collect()
}

pub fn summarize_transform_catalog_saturation_execution_v0(
    pass_id: &'static str,
    iteration_limit: usize,
    iteration_count: usize,
    eclass_count: usize,
    enode_count: usize,
    extracted_matches_candidate: bool,
) -> TransformCatalogSaturationExecutionV0 {
    TransformCatalogSaturationExecutionV0 {
        schema_version: "0",
        product: "omena-lawvere.saturation-execution",
        layer_marker: "enriched-algebraic",
        feature_gate: "transform-catalog-saturation",
        mechanism_scope: TRANSFORM_CATALOG_MECHANISM_SCOPE_V0,
        product_path_evidence_ready: TRANSFORM_CATALOG_PRODUCT_PATH_EVIDENCE_READY_V0,
        global_transform_theorem_claimed: TRANSFORM_CATALOG_GLOBAL_TRANSFORM_THEOREM_CLAIMED_V0,
        theory_version: TRANSFORM_CATALOG_SCHEMA_VERSION_V0,
        pass_id,
        analysis_slot: "TransformCatalogAnalysis",
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

pub const fn transform_catalog_execution_rank_hint(kind: TransformPassKind) -> u8 {
    // Mirrors the planner promote pattern (omena-transform-passes runtime::planner
    // execution_rank), keyed by catalog ordinal: target-lowering + static-eval
    // (14..=25) plus the appended relative-color/@container passes (42/43) cluster
    // together; print-css (41) is the terminal emission rank.
    match kind.ordinal() {
        27..=29 => 10,
        30..=40 => 20,
        14..=25 | 42 | 43 => 30,
        8..=13 | 26 => 40,
        1..=7 => 50,
        41 => 60,
        _ => 70,
    }
}

pub const fn transform_catalog_catalog_role_v0(kind: TransformPassKind) -> TransformCatalogRoleV0 {
    match kind {
        TransformPassKind::PrintCss => TransformCatalogRoleV0::TerminalForgetfulFunctor,
        _ => TransformCatalogRoleV0::Generator,
    }
}

fn transform_catalog_generator_count_v0(
    generators: &[TransformCatalogGeneratorMetadataV0],
) -> usize {
    generators
        .iter()
        .filter(|generator| generator.catalog_role == TransformCatalogRoleV0::Generator)
        .count()
}

fn terminal_pass_ids_from_pass_kinds(requested: &[TransformPassKind]) -> Vec<&'static str> {
    requested
        .iter()
        .filter(|kind| {
            transform_catalog_catalog_role_v0(**kind)
                == TransformCatalogRoleV0::TerminalForgetfulFunctor
        })
        .map(|kind| kind.id())
        .collect()
}

fn terminal_pass_ids_from_pass_ids(pass_ids: &[&'static str]) -> Vec<&'static str> {
    all_transform_pass_kinds()
        .into_iter()
        .filter(|kind| {
            transform_catalog_catalog_role_v0(*kind)
                == TransformCatalogRoleV0::TerminalForgetfulFunctor
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

/// Legacy wire values retained only for deprecated pre-1.0 adapters.
/// Owner: `omena-lawvere` maintainers. Removal is not before 1.0 and requires
/// downstream migration plus zero audited non-compatibility uses.
#[deprecated(
    since = "0.4.0",
    note = "legacy schema byte; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
const LEGACY_TRANSFORM_CATALOG_SCHEMA_VERSION_V0: &str = "lawvere-css-transform-catalog-v0";

#[deprecated(
    since = "0.4.0",
    note = "legacy feature byte; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
const LEGACY_TRANSFORM_CATALOG_FEATURE_GATE_V0: &str = "lawvere-saturation";

#[deprecated(
    since = "0.4.0",
    note = "legacy analysis-slot byte; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
const LEGACY_TRANSFORM_CATALOG_ANALYSIS_SLOT_V0: &str = "LawvereAnalysis";

#[allow(deprecated)]
fn restore_legacy_generator_metadata_v0(
    mut metadata: TransformCatalogGeneratorMetadataV0,
) -> TransformCatalogGeneratorMetadataV0 {
    metadata.feature_gate = LEGACY_TRANSFORM_CATALOG_FEATURE_GATE_V0;
    metadata.theory_version = LEGACY_TRANSFORM_CATALOG_SCHEMA_VERSION_V0;
    metadata
}

#[allow(deprecated)]
fn restore_legacy_equation_cluster_v0(
    mut cluster: TransformCatalogEquationClusterV0,
) -> TransformCatalogEquationClusterV0 {
    cluster.feature_gate = LEGACY_TRANSFORM_CATALOG_FEATURE_GATE_V0;
    cluster.theory_version = LEGACY_TRANSFORM_CATALOG_SCHEMA_VERSION_V0;
    cluster
}

#[allow(deprecated)]
fn restore_legacy_differential_tier_v0(
    mut tier: TransformCatalogDifferentialCorpusTierV0,
) -> TransformCatalogDifferentialCorpusTierV0 {
    tier.feature_gate = LEGACY_TRANSFORM_CATALOG_FEATURE_GATE_V0;
    tier.theory_version = LEGACY_TRANSFORM_CATALOG_SCHEMA_VERSION_V0;
    tier
}

#[allow(deprecated)]
fn restore_legacy_metadata_summary_v0(
    mut summary: TransformCatalogMetadataSummaryV0,
) -> TransformCatalogMetadataSummaryV0 {
    summary.feature_gate = LEGACY_TRANSFORM_CATALOG_FEATURE_GATE_V0;
    summary.theory_version = LEGACY_TRANSFORM_CATALOG_SCHEMA_VERSION_V0;
    summary.generators = summary
        .generators
        .into_iter()
        .map(restore_legacy_generator_metadata_v0)
        .collect();
    summary.equation_clusters = summary
        .equation_clusters
        .into_iter()
        .map(restore_legacy_equation_cluster_v0)
        .collect();
    summary.differential_corpus_tiers = summary
        .differential_corpus_tiers
        .into_iter()
        .map(restore_legacy_differential_tier_v0)
        .collect();
    summary
}

#[allow(deprecated)]
fn restore_legacy_parallel_plan_v0(
    mut plan: TransformCatalogTransformPassParallelPlanV0,
) -> TransformCatalogTransformPassParallelPlanV0 {
    plan.feature_gate = LEGACY_TRANSFORM_CATALOG_FEATURE_GATE_V0;
    plan.rank_clusters = plan
        .rank_clusters
        .into_iter()
        .map(restore_legacy_equation_cluster_v0)
        .collect();
    plan
}

#[allow(deprecated)]
fn restore_legacy_model_trace_v0(
    mut trace: TransformCatalogModelTraceV0,
) -> TransformCatalogModelTraceV0 {
    trace.feature_gate = LEGACY_TRANSFORM_CATALOG_FEATURE_GATE_V0;
    trace.theory_version = LEGACY_TRANSFORM_CATALOG_SCHEMA_VERSION_V0;
    trace.rank_clusters = trace
        .rank_clusters
        .into_iter()
        .map(restore_legacy_equation_cluster_v0)
        .collect();
    trace
}

#[allow(deprecated)]
fn restore_legacy_reorderability_certificate_v0(
    mut certificate: ReorderabilityCertificateV0,
) -> ReorderabilityCertificateV0 {
    certificate.feature_gate = LEGACY_TRANSFORM_CATALOG_FEATURE_GATE_V0;
    certificate.theory_version = LEGACY_TRANSFORM_CATALOG_SCHEMA_VERSION_V0;
    certificate
}

#[allow(deprecated)]
fn restore_legacy_differential_witness_v0(
    mut witness: TransformCatalogDifferentialCommutativityWitnessV0,
) -> TransformCatalogDifferentialCommutativityWitnessV0 {
    witness.feature_gate = LEGACY_TRANSFORM_CATALOG_FEATURE_GATE_V0;
    witness.theory_version = LEGACY_TRANSFORM_CATALOG_SCHEMA_VERSION_V0;
    witness
}

#[allow(deprecated)]
fn restore_legacy_saturation_execution_v0(
    mut execution: TransformCatalogSaturationExecutionV0,
) -> TransformCatalogSaturationExecutionV0 {
    execution.feature_gate = LEGACY_TRANSFORM_CATALOG_FEATURE_GATE_V0;
    execution.theory_version = LEGACY_TRANSFORM_CATALOG_SCHEMA_VERSION_V0;
    execution.analysis_slot = LEGACY_TRANSFORM_CATALOG_ANALYSIS_SLOT_V0;
    execution
}

/// Pre-1.0 nominal compatibility role.
/// Owner: `omena-lawvere` maintainers. Removal condition: not before 1.0,
/// after downstream migration and zero audited in-repo non-compatibility uses.
#[deprecated(
    since = "0.4.0",
    note = "use TransformCatalogRoleV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LawvereCatalogRoleV0 {
    Generator,
    TerminalForgetfulFunctor,
}

#[deprecated(
    since = "0.4.0",
    note = "use TransformCatalogGeneratorMetadataV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
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

#[deprecated(
    since = "0.4.0",
    note = "use TransformCatalogEquationClusterV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
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

#[deprecated(
    since = "0.4.0",
    note = "use TransformCatalogDifferentialCorpusTierV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
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

#[deprecated(
    since = "0.4.0",
    note = "use TransformCatalogDifferentialCommutativityCaseV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
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

#[deprecated(
    since = "0.4.0",
    note = "use TransformCatalogDifferentialCommutativityWitnessV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereDifferentialCommutativityWitnessV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub mechanism_scope: &'static str,
    pub product_path_evidence_ready: bool,
    pub global_transform_theorem_claimed: bool,
    pub theory_version: &'static str,
    pub left_pass_id: &'static str,
    pub right_pass_id: &'static str,
    pub fixture_count: usize,
    pub equal_fixture_count: usize,
    pub mismatch_count: usize,
    pub cases: Vec<LawvereDifferentialCommutativityCaseV0>,
    pub accepted: bool,
}

#[deprecated(
    since = "0.4.0",
    note = "use TransformCatalogTransformPassParallelPlanV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassParallelPlanV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub mechanism_scope: &'static str,
    pub product_path_evidence_ready: bool,
    pub global_transform_theorem_claimed: bool,
    pub scheduler_status: &'static str,
    pub requested_pass_ids: Vec<&'static str>,
    pub terminal_pass_ids: Vec<&'static str>,
    pub rank_clusters: Vec<LawvereEquationClusterV0>,
    pub executor_consumes_plan: bool,
}

#[deprecated(
    since = "0.4.0",
    note = "use TransformCatalogModelTraceV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereModelTraceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub mechanism_scope: &'static str,
    pub product_path_evidence_ready: bool,
    pub global_transform_theorem_claimed: bool,
    pub theory_version: &'static str,
    pub input_pass_ids: Vec<&'static str>,
    pub ordered_pass_ids: Vec<&'static str>,
    pub terminal_pass_ids: Vec<&'static str>,
    pub rank_clusters: Vec<LawvereEquationClusterV0>,
    pub preserves_existing_executor_signature: bool,
}

#[deprecated(
    since = "0.4.0",
    note = "use TransformCatalogSaturationExecutionV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LawvereSaturationExecutionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub mechanism_scope: &'static str,
    pub product_path_evidence_ready: bool,
    pub global_transform_theorem_claimed: bool,
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

#[deprecated(
    since = "0.4.0",
    note = "use TransformCatalogMetadataSummaryV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
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

#[deprecated(
    since = "0.4.0",
    note = "use TRANSFORM_CATALOG_MECHANISM_SCOPE_V0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub const LAWVERE_MECHANISM_SCOPE_V0: &str = "featureGatedDifferentialWitnessSubstrate";
#[deprecated(
    since = "0.4.0",
    note = "use TRANSFORM_CATALOG_PRODUCT_PATH_EVIDENCE_READY_V0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub const LAWVERE_PRODUCT_PATH_EVIDENCE_READY_V0: bool = false;
#[deprecated(
    since = "0.4.0",
    note = "use TRANSFORM_CATALOG_GLOBAL_TRANSFORM_THEOREM_CLAIMED_V0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub const LAWVERE_GLOBAL_TRANSFORM_THEOREM_CLAIMED_V0: bool = false;

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "nominal compatibility conversion owned by omena-lawvere maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
const fn into_lawvere_catalog_role_v0(role: TransformCatalogRoleV0) -> LawvereCatalogRoleV0 {
    match role {
        TransformCatalogRoleV0::Generator => LawvereCatalogRoleV0::Generator,
        TransformCatalogRoleV0::TerminalForgetfulFunctor => {
            LawvereCatalogRoleV0::TerminalForgetfulFunctor
        }
    }
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "nominal compatibility conversion owned by omena-lawvere maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn into_lawvere_generator_metadata_v0(
    metadata: TransformCatalogGeneratorMetadataV0,
) -> LawvereGeneratorMetadataV0 {
    LawvereGeneratorMetadataV0 {
        schema_version: metadata.schema_version,
        product: metadata.product,
        layer_marker: metadata.layer_marker,
        feature_gate: metadata.feature_gate,
        theory_version: metadata.theory_version,
        pass_id: metadata.pass_id,
        ordinal: metadata.ordinal,
        title: metadata.title,
        catalog_role: into_lawvere_catalog_role_v0(metadata.catalog_role),
        abstract_domain_tag: metadata.abstract_domain_tag,
        execution_rank_hint: metadata.execution_rank_hint,
        terminal_forgetful_functor: metadata.terminal_forgetful_functor,
        reads_fixed_point: metadata.reads_fixed_point,
    }
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "nominal compatibility conversion owned by omena-lawvere maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn into_lawvere_equation_cluster_v0(
    cluster: TransformCatalogEquationClusterV0,
) -> LawvereEquationClusterV0 {
    LawvereEquationClusterV0 {
        schema_version: cluster.schema_version,
        product: cluster.product,
        layer_marker: cluster.layer_marker,
        feature_gate: cluster.feature_gate,
        execution_rank_hint: cluster.execution_rank_hint,
        pass_ids: cluster.pass_ids,
        generator_count: cluster.generator_count,
        saturation_budget_tier: cluster.saturation_budget_tier,
        theory_version: cluster.theory_version,
    }
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "nominal compatibility conversion owned by omena-lawvere maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn into_lawvere_differential_tier_v0(
    tier: TransformCatalogDifferentialCorpusTierV0,
) -> LawvereDifferentialCorpusTierV0 {
    LawvereDifferentialCorpusTierV0 {
        schema_version: tier.schema_version,
        product: tier.product,
        layer_marker: tier.layer_marker,
        feature_gate: tier.feature_gate,
        theory_version: tier.theory_version,
        tier: tier.tier,
        tier_label: tier.tier_label,
        fixture_count: tier.fixture_count,
        required_pass_rate_percent: tier.required_pass_rate_percent,
    }
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "nominal compatibility conversion owned by omena-lawvere maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn into_lawvere_case_v0(
    case: TransformCatalogDifferentialCommutativityCaseV0,
) -> LawvereDifferentialCommutativityCaseV0 {
    LawvereDifferentialCommutativityCaseV0 {
        label: case.label,
        input_css: case.input_css,
        left_then_right_css: case.left_then_right_css,
        right_then_left_css: case.right_then_left_css,
        left_then_right_mutation_count: case.left_then_right_mutation_count,
        right_then_left_mutation_count: case.right_then_left_mutation_count,
        equal_output: case.equal_output,
    }
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "nominal compatibility conversion owned by omena-lawvere maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn from_lawvere_case_v0(
    case: LawvereDifferentialCommutativityCaseV0,
) -> TransformCatalogDifferentialCommutativityCaseV0 {
    TransformCatalogDifferentialCommutativityCaseV0 {
        label: case.label,
        input_css: case.input_css,
        left_then_right_css: case.left_then_right_css,
        right_then_left_css: case.right_then_left_css,
        left_then_right_mutation_count: case.left_then_right_mutation_count,
        right_then_left_mutation_count: case.right_then_left_mutation_count,
        equal_output: case.equal_output,
    }
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "nominal compatibility conversion owned by omena-lawvere maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn into_lawvere_witness_v0(
    witness: TransformCatalogDifferentialCommutativityWitnessV0,
) -> LawvereDifferentialCommutativityWitnessV0 {
    LawvereDifferentialCommutativityWitnessV0 {
        schema_version: witness.schema_version,
        product: witness.product,
        layer_marker: witness.layer_marker,
        feature_gate: witness.feature_gate,
        mechanism_scope: witness.mechanism_scope,
        product_path_evidence_ready: witness.product_path_evidence_ready,
        global_transform_theorem_claimed: witness.global_transform_theorem_claimed,
        theory_version: witness.theory_version,
        left_pass_id: witness.left_pass_id,
        right_pass_id: witness.right_pass_id,
        fixture_count: witness.fixture_count,
        equal_fixture_count: witness.equal_fixture_count,
        mismatch_count: witness.mismatch_count,
        cases: witness
            .cases
            .into_iter()
            .map(into_lawvere_case_v0)
            .collect(),
        accepted: witness.accepted,
    }
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "nominal compatibility conversion owned by omena-lawvere maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn from_lawvere_witness_v0(
    witness: LawvereDifferentialCommutativityWitnessV0,
) -> TransformCatalogDifferentialCommutativityWitnessV0 {
    TransformCatalogDifferentialCommutativityWitnessV0 {
        schema_version: witness.schema_version,
        product: witness.product,
        layer_marker: witness.layer_marker,
        feature_gate: witness.feature_gate,
        mechanism_scope: witness.mechanism_scope,
        product_path_evidence_ready: witness.product_path_evidence_ready,
        global_transform_theorem_claimed: witness.global_transform_theorem_claimed,
        theory_version: witness.theory_version,
        left_pass_id: witness.left_pass_id,
        right_pass_id: witness.right_pass_id,
        fixture_count: witness.fixture_count,
        equal_fixture_count: witness.equal_fixture_count,
        mismatch_count: witness.mismatch_count,
        cases: witness
            .cases
            .into_iter()
            .map(from_lawvere_case_v0)
            .collect(),
        accepted: witness.accepted,
    }
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "nominal compatibility conversion owned by omena-lawvere maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn into_transform_pass_parallel_plan_v0(
    plan: TransformCatalogTransformPassParallelPlanV0,
) -> TransformPassParallelPlanV0 {
    TransformPassParallelPlanV0 {
        schema_version: plan.schema_version,
        product: plan.product,
        layer_marker: plan.layer_marker,
        feature_gate: plan.feature_gate,
        mechanism_scope: plan.mechanism_scope,
        product_path_evidence_ready: plan.product_path_evidence_ready,
        global_transform_theorem_claimed: plan.global_transform_theorem_claimed,
        scheduler_status: plan.scheduler_status,
        requested_pass_ids: plan.requested_pass_ids,
        terminal_pass_ids: plan.terminal_pass_ids,
        rank_clusters: plan
            .rank_clusters
            .into_iter()
            .map(into_lawvere_equation_cluster_v0)
            .collect(),
        executor_consumes_plan: plan.executor_consumes_plan,
    }
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "nominal compatibility conversion owned by omena-lawvere maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn into_lawvere_model_trace_v0(trace: TransformCatalogModelTraceV0) -> LawvereModelTraceV0 {
    LawvereModelTraceV0 {
        schema_version: trace.schema_version,
        product: trace.product,
        layer_marker: trace.layer_marker,
        feature_gate: trace.feature_gate,
        mechanism_scope: trace.mechanism_scope,
        product_path_evidence_ready: trace.product_path_evidence_ready,
        global_transform_theorem_claimed: trace.global_transform_theorem_claimed,
        theory_version: trace.theory_version,
        input_pass_ids: trace.input_pass_ids,
        ordered_pass_ids: trace.ordered_pass_ids,
        terminal_pass_ids: trace.terminal_pass_ids,
        rank_clusters: trace
            .rank_clusters
            .into_iter()
            .map(into_lawvere_equation_cluster_v0)
            .collect(),
        preserves_existing_executor_signature: trace.preserves_existing_executor_signature,
    }
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "nominal compatibility conversion owned by omena-lawvere maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn into_lawvere_saturation_execution_v0(
    execution: TransformCatalogSaturationExecutionV0,
) -> LawvereSaturationExecutionV0 {
    LawvereSaturationExecutionV0 {
        schema_version: execution.schema_version,
        product: execution.product,
        layer_marker: execution.layer_marker,
        feature_gate: execution.feature_gate,
        mechanism_scope: execution.mechanism_scope,
        product_path_evidence_ready: execution.product_path_evidence_ready,
        global_transform_theorem_claimed: execution.global_transform_theorem_claimed,
        theory_version: execution.theory_version,
        pass_id: execution.pass_id,
        analysis_slot: execution.analysis_slot,
        original_unit_analysis_path_preserved: execution.original_unit_analysis_path_preserved,
        differential_tier: execution.differential_tier,
        differential_fixture_count: execution.differential_fixture_count,
        iteration_limit: execution.iteration_limit,
        iteration_count: execution.iteration_count,
        eclass_count: execution.eclass_count,
        enode_count: execution.enode_count,
        accepted: execution.accepted,
        extracted_matches_candidate: execution.extracted_matches_candidate,
    }
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "nominal compatibility conversion owned by omena-lawvere maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn into_lawvere_theory_summary_v0(
    summary: TransformCatalogMetadataSummaryV0,
) -> LawvereTheorySummaryV0 {
    LawvereTheorySummaryV0 {
        schema_version: summary.schema_version,
        product: summary.product,
        layer_marker: summary.layer_marker,
        feature_gate: summary.feature_gate,
        theory_version: summary.theory_version,
        catalog_pass_count: summary.catalog_pass_count,
        catalog_entry_count: summary.catalog_entry_count,
        lawvere_generator_count: summary.lawvere_generator_count,
        terminal_forgetful_functor_count: summary.terminal_forgetful_functor_count,
        execution_rank_cluster_count: summary.execution_rank_cluster_count,
        equation_clusters: summary
            .equation_clusters
            .into_iter()
            .map(into_lawvere_equation_cluster_v0)
            .collect(),
        generators: summary
            .generators
            .into_iter()
            .map(into_lawvere_generator_metadata_v0)
            .collect(),
        dag_edges: summary.dag_edges,
        saturation_budget_tiers: summary.saturation_budget_tiers,
        differential_corpus_tiers: summary
            .differential_corpus_tiers
            .into_iter()
            .map(into_lawvere_differential_tier_v0)
            .collect(),
        lawvere_saturation_feature_enabled_by_default: summary
            .lawvere_saturation_feature_enabled_by_default,
        product_path_evidence_ready: summary.product_path_evidence_ready,
        mechanism_scope: summary.mechanism_scope,
        omena_categorical_dependency_forbidden: summary.omena_categorical_dependency_forbidden,
    }
}

#[deprecated(
    since = "0.4.0",
    note = "use TRANSFORM_CATALOG_SCHEMA_VERSION_V0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
pub const LAWVERE_THEORY_VERSION_V0: &str = LEGACY_TRANSFORM_CATALOG_SCHEMA_VERSION_V0;

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use summarize_transform_catalog_metadata_v0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn summarize_lawvere_theory_v0() -> LawvereTheorySummaryV0 {
    into_lawvere_theory_summary_v0(restore_legacy_metadata_summary_v0(
        summarize_transform_catalog_metadata_v0(),
    ))
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use transform_catalog_generator_metadata_catalog_v0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn lawvere_generator_metadata_catalog_v0() -> Vec<LawvereGeneratorMetadataV0> {
    transform_catalog_generator_metadata_catalog_v0()
        .into_iter()
        .map(restore_legacy_generator_metadata_v0)
        .map(into_lawvere_generator_metadata_v0)
        .collect()
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use transform_catalog_generator_metadata_v0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn lawvere_generator_metadata_v0(kind: TransformPassKind) -> LawvereGeneratorMetadataV0 {
    into_lawvere_generator_metadata_v0(restore_legacy_generator_metadata_v0(
        transform_catalog_generator_metadata_v0(kind),
    ))
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use transform_catalog_equation_clusters_v0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn lawvere_equation_clusters_v0(pass_ids: &[&'static str]) -> Vec<LawvereEquationClusterV0> {
    transform_catalog_equation_clusters_v0(pass_ids)
        .into_iter()
        .map(restore_legacy_equation_cluster_v0)
        .map(into_lawvere_equation_cluster_v0)
        .collect()
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use plan_transform_catalog_parallel_layers_v0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn plan_transform_pass_parallel_layers_v0(
    requested: &[TransformPassKind],
) -> TransformPassParallelPlanV0 {
    into_transform_pass_parallel_plan_v0(restore_legacy_parallel_plan_v0(
        plan_transform_catalog_parallel_layers_v0(requested),
    ))
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use trace_transform_catalog_model_v0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn trace_lawvere_model_v0(
    requested: &[TransformPassKind],
    ordered_pass_ids: Vec<&'static str>,
) -> LawvereModelTraceV0 {
    into_lawvere_model_trace_v0(restore_legacy_model_trace_v0(
        trace_transform_catalog_model_v0(requested, ordered_pass_ids),
    ))
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use transform_catalog_reorderability_certificate_v0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn reorderability_certificate_v0(
    left: TransformPassKind,
    right: TransformPassKind,
) -> ReorderabilityCertificateV0 {
    restore_legacy_reorderability_certificate_v0(transform_catalog_reorderability_certificate_v0(
        left, right,
    ))
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use transform_catalog_differential_commutativity_witness_v0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn lawvere_differential_commutativity_witness_v0(
    left: TransformPassKind,
    right: TransformPassKind,
    cases: Vec<LawvereDifferentialCommutativityCaseV0>,
) -> LawvereDifferentialCommutativityWitnessV0 {
    into_lawvere_witness_v0(restore_legacy_differential_witness_v0(
        transform_catalog_differential_commutativity_witness_v0(
            left,
            right,
            cases.into_iter().map(from_lawvere_case_v0).collect(),
        ),
    ))
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use transform_catalog_reorderability_certificate_from_differential_v0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn reorderability_certificate_from_differential_v0(
    left: TransformPassKind,
    right: TransformPassKind,
    witness: &LawvereDifferentialCommutativityWitnessV0,
) -> ReorderabilityCertificateV0 {
    let canonical_witness = from_lawvere_witness_v0(witness.clone());
    restore_legacy_reorderability_certificate_v0(
        transform_catalog_reorderability_certificate_from_differential_v0(
            left,
            right,
            &canonical_witness,
        ),
    )
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use transform_catalog_differential_corpus_tiers_v0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn lawvere_differential_corpus_tiers_v0() -> Vec<LawvereDifferentialCorpusTierV0> {
    transform_catalog_differential_corpus_tiers_v0()
        .into_iter()
        .map(restore_legacy_differential_tier_v0)
        .map(into_lawvere_differential_tier_v0)
        .collect()
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use summarize_transform_catalog_saturation_execution_v0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn summarize_lawvere_saturation_execution_v0(
    pass_id: &'static str,
    iteration_limit: usize,
    iteration_count: usize,
    eclass_count: usize,
    enode_count: usize,
    extracted_matches_candidate: bool,
) -> LawvereSaturationExecutionV0 {
    into_lawvere_saturation_execution_v0(restore_legacy_saturation_execution_v0(
        summarize_transform_catalog_saturation_execution_v0(
            pass_id,
            iteration_limit,
            iteration_count,
            eclass_count,
            enode_count,
            extracted_matches_candidate,
        ),
    ))
}

#[deprecated(
    since = "0.4.0",
    note = "use transform_catalog_execution_rank_hint; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub const fn lawvere_execution_rank_hint(kind: TransformPassKind) -> u8 {
    transform_catalog_execution_rank_hint(kind)
}

#[deprecated(
    since = "0.4.0",
    note = "use transform_catalog_catalog_role_v0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
pub const fn lawvere_catalog_role_v0(kind: TransformPassKind) -> LawvereCatalogRoleV0 {
    into_lawvere_catalog_role_v0(transform_catalog_catalog_role_v0(kind))
}

#[cfg(test)]
#[deprecated(
    since = "0.4.0",
    note = "legacy metadata wire fixture owned by omena-lawvere maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
const COMPATIBILITY_TRANSFORM_CATALOG_METADATA_EXPECTED_WIRE_V0: &str = r#"{"product":"omena-lawvere.theory-summary","featureGate":"lawvere-saturation","theoryVersion":"lawvere-css-transform-catalog-v0","firstGeneratorFeatureGate":"lawvere-saturation","firstGeneratorTheoryVersion":"lawvere-css-transform-catalog-v0"}"#;

#[cfg(test)]
#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "legacy metadata wire fixture adapter owned by omena-lawvere maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn compatibility_transform_catalog_metadata_serialized_projection_v0()
-> Result<String, serde_json::Error> {
    let summary = summarize_lawvere_theory_v0();
    serde_json::to_string(&serde_json::json!({
        "product": summary.product,
        "featureGate": summary.feature_gate,
        "theoryVersion": summary.theory_version,
        "firstGeneratorFeatureGate": summary.generators[0].feature_gate,
        "firstGeneratorTheoryVersion": summary.generators[0].theory_version,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_forty_pass_transform_catalog_catalog_with_schema_zero() {
        let summary = summarize_transform_catalog_metadata_v0();

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.layer_marker, "enriched-algebraic");
        assert_eq!(summary.feature_gate, "transform-catalog-saturation");
        assert_eq!(summary.catalog_pass_count, TRANSFORM_PASS_CATALOG_LEN);
        assert_eq!(summary.catalog_entry_count, TRANSFORM_PASS_CATALOG_LEN);
        assert_eq!(
            summary.transform_catalog_generator_count(),
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
        assert!(!summary.transform_catalog_saturation_feature_enabled_by_default());
        assert_eq!(
            summary.product_path_evidence_ready,
            TRANSFORM_CATALOG_PRODUCT_PATH_EVIDENCE_READY_V0
        );
        assert_eq!(
            summary.mechanism_scope,
            TRANSFORM_CATALOG_MECHANISM_SCOPE_V0
        );
        assert!(summary.omena_categorical_dependency_forbidden);
    }

    #[test]
    #[allow(deprecated)]
    fn compatibility_and_canonical_metadata_keep_distinct_exact_wire_projections()
    -> Result<(), serde_json::Error> {
        let compatibility = compatibility_transform_catalog_metadata_serialized_projection_v0()?;
        assert_eq!(
            compatibility,
            COMPATIBILITY_TRANSFORM_CATALOG_METADATA_EXPECTED_WIRE_V0
        );

        let summary = summarize_transform_catalog_metadata_v0();
        let canonical = serde_json::to_string(&serde_json::json!({
            "product": summary.product,
            "featureGate": summary.feature_gate,
            "theoryVersion": summary.theory_version,
            "firstGeneratorFeatureGate": summary.generators[0].feature_gate,
            "firstGeneratorTheoryVersion": summary.generators[0].theory_version,
        }))?;
        assert_eq!(
            canonical,
            r#"{"product":"omena-lawvere.theory-summary","featureGate":"transform-catalog-saturation","theoryVersion":"css-transform-catalog-v0","firstGeneratorFeatureGate":"transform-catalog-saturation","firstGeneratorTheoryVersion":"css-transform-catalog-v0"}"#
        );
        Ok(())
    }

    #[test]
    fn execution_rank_hint_clusters_match_planner_promote_pattern() {
        let metadata = transform_catalog_generator_metadata_catalog_v0();

        assert_eq!(metadata.len(), TRANSFORM_PASS_CATALOG_LEN);
        assert!(metadata.iter().any(|generator| {
            generator.pass_id == "css-modules-class-hashing" && generator.execution_rank_hint == 20
        }));
        assert!(metadata.iter().any(|generator| {
            generator.pass_id == "print-css"
                && generator.catalog_role == TransformCatalogRoleV0::TerminalForgetfulFunctor
                && generator.terminal_forgetful_functor
                && generator.execution_rank_hint == 60
        }));
    }

    #[test]
    fn parallel_plan_is_scaffold_only_and_does_not_consume_executor() {
        let plan = plan_transform_catalog_parallel_layers_v0(&[
            TransformPassKind::ColorCompression,
            TransformPassKind::NumberCompression,
            TransformPassKind::PrintCss,
        ]);

        assert_eq!(plan.schema_version, "0");
        assert_eq!(plan.scheduler_status, "scaffoldOnly");
        assert!(!plan.executor_consumes_plan);
        assert_eq!(plan.mechanism_scope, TRANSFORM_CATALOG_MECHANISM_SCOPE_V0);
        assert_eq!(
            plan.product_path_evidence_ready,
            TRANSFORM_CATALOG_PRODUCT_PATH_EVIDENCE_READY_V0
        );
        assert_eq!(
            plan.global_transform_theorem_claimed,
            TRANSFORM_CATALOG_GLOBAL_TRANSFORM_THEOREM_CLAIMED_V0
        );
        assert_eq!(plan.terminal_pass_ids, vec!["print-css"]);
        assert_eq!(plan.rank_clusters.len(), 1);
    }

    #[test]
    fn saturation_execution_contract_records_transform_catalog_analysis_slot() {
        let execution = summarize_transform_catalog_saturation_execution_v0(
            TransformPassKind::CalcReduction.id(),
            8,
            2,
            5,
            9,
            true,
        );

        assert_eq!(execution.schema_version, "0");
        assert_eq!(execution.layer_marker, "enriched-algebraic");
        assert_eq!(execution.feature_gate, "transform-catalog-saturation");
        assert_eq!(execution.analysis_slot, "TransformCatalogAnalysis");
        assert_eq!(execution.differential_fixture_count, 10);
        assert!(execution.original_unit_analysis_path_preserved);
        assert_eq!(
            execution.mechanism_scope,
            TRANSFORM_CATALOG_MECHANISM_SCOPE_V0
        );
        assert_eq!(
            execution.product_path_evidence_ready,
            TRANSFORM_CATALOG_PRODUCT_PATH_EVIDENCE_READY_V0
        );
        assert_eq!(
            execution.global_transform_theorem_claimed,
            TRANSFORM_CATALOG_GLOBAL_TRANSFORM_THEOREM_CLAIMED_V0
        );
        assert!(execution.accepted);
    }

    #[test]
    fn rank_only_reorderability_certificate_requires_differential_witness() {
        let certificate = transform_catalog_reorderability_certificate_v0(
            TransformPassKind::CommentStrip,
            TransformPassKind::WhitespaceStrip,
        );

        assert_eq!(
            certificate.commute_witness,
            "requiresDifferentialCommutativityWitness"
        );
        assert_eq!(
            certificate.mechanism_scope,
            TRANSFORM_CATALOG_MECHANISM_SCOPE_V0
        );
        assert_eq!(
            certificate.product_path_evidence_ready,
            TRANSFORM_CATALOG_PRODUCT_PATH_EVIDENCE_READY_V0
        );
        assert_eq!(
            certificate.global_transform_theorem_claimed,
            TRANSFORM_CATALOG_GLOBAL_TRANSFORM_THEOREM_CLAIMED_V0
        );
        assert_eq!(certificate.differential_fixture_count, 0);
        assert!(!certificate.accepted);
    }

    #[test]
    fn reorderability_certificate_family_derivation_preserves_legacy_json_contract()
    -> Result<(), serde_json::Error> {
        let rank_only = transform_catalog_reorderability_certificate_v0(
            TransformPassKind::CommentStrip,
            TransformPassKind::WhitespaceStrip,
        );
        let rank_only_json = serde_json::to_value(&rank_only)?;
        assert_eq!(rank_only_json["computedValuePreserved"], false);
        assert!(rank_only_json.get("obligationFamily").is_none());

        let witness = transform_catalog_differential_commutativity_witness_v0(
            TransformPassKind::CommentStrip,
            TransformPassKind::WhitespaceStrip,
            vec![TransformCatalogDifferentialCommutativityCaseV0 {
                label: "comment-whitespace".to_string(),
                input_css: ".a { color : red ; /* x */ }".to_string(),
                left_then_right_css: ".a{color:red}".to_string(),
                right_then_left_css: ".a{color:red}".to_string(),
                left_then_right_mutation_count: 2,
                right_then_left_mutation_count: 2,
                equal_output: true,
            }],
        );
        let accepted = transform_catalog_reorderability_certificate_from_differential_v0(
            TransformPassKind::CommentStrip,
            TransformPassKind::WhitespaceStrip,
            &witness,
        );
        let accepted_json = serde_json::to_value(&accepted)?;

        assert_eq!(accepted_json["computedValuePreserved"], true);
        assert!(accepted_json.get("obligationFamily").is_none());
        assert_eq!(
            accepted_json["commuteWitness"],
            "differentialCommutativityCorpus"
        );

        Ok(())
    }

    #[test]
    fn differential_reorderability_certificate_accepts_only_equal_output_corpus() {
        let witness = transform_catalog_differential_commutativity_witness_v0(
            TransformPassKind::CommentStrip,
            TransformPassKind::WhitespaceStrip,
            vec![TransformCatalogDifferentialCommutativityCaseV0 {
                label: "comment-whitespace".to_string(),
                input_css: ".a { color : red ; /* x */ }".to_string(),
                left_then_right_css: ".a{color:red}".to_string(),
                right_then_left_css: ".a{color:red}".to_string(),
                left_then_right_mutation_count: 2,
                right_then_left_mutation_count: 2,
                equal_output: true,
            }],
        );
        let certificate = transform_catalog_reorderability_certificate_from_differential_v0(
            TransformPassKind::CommentStrip,
            TransformPassKind::WhitespaceStrip,
            &witness,
        );

        assert!(witness.accepted);
        assert_eq!(
            certificate.commute_witness,
            "differentialCommutativityCorpus"
        );
        assert_eq!(
            witness.mechanism_scope,
            TRANSFORM_CATALOG_MECHANISM_SCOPE_V0
        );
        assert_eq!(
            certificate.mechanism_scope,
            TRANSFORM_CATALOG_MECHANISM_SCOPE_V0
        );
        assert_eq!(
            witness.global_transform_theorem_claimed,
            TRANSFORM_CATALOG_GLOBAL_TRANSFORM_THEOREM_CLAIMED_V0
        );
        assert_eq!(
            certificate.global_transform_theorem_claimed,
            TRANSFORM_CATALOG_GLOBAL_TRANSFORM_THEOREM_CLAIMED_V0
        );
        assert_eq!(certificate.differential_fixture_count, 1);
        assert_eq!(certificate.differential_mismatch_count, 0);
        assert!(certificate.accepted);
    }
}

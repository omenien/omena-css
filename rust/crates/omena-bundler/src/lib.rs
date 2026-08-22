//! Standalone 0.x bundle planning for Omena CSS transforms.
//!
//! This crate is the standalone Rust entry point for the Omena bundler planning
//! surface. It decides which bundle/module passes are required for a style
//! source and delegates ordering to `omena-transform-passes`.
//!
//! The public types intentionally keep their `V0` suffix during the 0.x line.

#[cfg(test)]
mod carrier_hygiene_assertions;
mod emission_items;
mod emission_order;

pub use emission_items::{
    EmissionItemFactCategoryV0, EmissionItemInputV0, EmissionItemKindV0, EmissionItemOrderKeyV0,
    EmissionItemPlanV0, EmissionItemProjectionDisclosureV0, EmissionItemProjectionDispositionV0,
    EmissionItemProjectionReasonV0, EmissionItemV0, LinkedEmissionItemMaterializationErrorV0,
    LinkedEmissionItemOrderV0, LinkedEmissionItemV0, TransformBundleEmissionItemProjectionV0,
};
pub use emission_order::{
    EmissionCycleClassV0, EmissionCycleDialectV0, EmissionCycleGroupV0, EmissionCyclePolicyV0,
    EmissionDependencyFactV0, EmissionOrderKeyV0, EmissionOrderingPolicyV0, EmissionPlanV0,
};

use omena_cascade::{
    CascadeKey, CascadeLevel, LayerOrdinal, ModuleRank, OpenWorldTieEvidence, Specificity,
    normalized_layer_rank,
};
use omena_cross_file_summary::{EdgeOrderRelevanceV0, OmenaCrossFileSummaryRawEdgeKindV0};
use omena_parser::{
    ClosedWorldBundleBuildErrorV0, ClosedWorldBundleV0, ClosedWorldComposesEdgeV0,
    ClosedWorldLinkedModuleV0, ClosedWorldModuleMetadataV0,
    ClosedWorldModuleReachabilityEvidenceV0, ConfigurationHashV0, ModuleIdV0, ModuleInstanceKeyV0,
    ParsedAnimationFactKind, ParsedCssModuleComposesEdgeKind, ParsedCssModuleValueFactKind,
    ParsedEmissionSelectorFactsV0, ParsedSassModuleEdgeFactKind, ParsedSelectorFactKind,
    ParsedStyleFacts, ParsedVariableFactKind, StyleDialect, collect_style_fact_collection,
    collect_style_facts,
};
use omena_syntax::ident::{CanonicalCustomPropertyNameV0, PropertyNameV0};
use omena_transform_cst::{
    IrNodeKindV0, TransformPassKind, lower_transform_ir_from_source, transform_pass_sort_ordinal,
};
use omena_transform_passes::{TransformPassPlanV0, plan_transform_passes};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformBundleEdgeKind {
    SassUse,
    SassForward,
    SassImport,
    CssImport,
    LessImport,
    CssModuleValueImport,
    CssModuleComposesLocal,
    CssModuleComposesExternal,
    IcssImport,
}

pub const TRANSFORM_BUNDLE_EDGE_KIND_VARIANTS_V0: [TransformBundleEdgeKind; 9] = [
    TransformBundleEdgeKind::SassUse,
    TransformBundleEdgeKind::SassForward,
    TransformBundleEdgeKind::SassImport,
    TransformBundleEdgeKind::CssImport,
    TransformBundleEdgeKind::LessImport,
    TransformBundleEdgeKind::CssModuleValueImport,
    TransformBundleEdgeKind::CssModuleComposesLocal,
    TransformBundleEdgeKind::CssModuleComposesExternal,
    TransformBundleEdgeKind::IcssImport,
];

impl TransformBundleEdgeKind {
    pub const fn as_wire_label(self) -> &'static str {
        match self {
            Self::SassUse => "sassUse",
            Self::SassForward => "sassForward",
            Self::SassImport => "sassImport",
            Self::CssImport => "cssImport",
            Self::LessImport => "lessImport",
            Self::CssModuleValueImport => "cssModuleValueImport",
            Self::CssModuleComposesLocal => "cssModuleComposesLocal",
            Self::CssModuleComposesExternal => "cssModuleComposesExternal",
            Self::IcssImport => "icssImport",
        }
    }

    pub const fn order_relevance(self) -> EdgeOrderRelevanceV0 {
        self.raw_edge_kind().order_relevance()
    }

    pub const fn order_relevance_reason(self) -> &'static str {
        match self {
            Self::SassUse => "Sass module use sequence participates in evaluation order",
            Self::SassForward => "Sass forwarding sequence participates in module exposure order",
            Self::SassImport => "Sass import sequence participates in emitted rule order",
            Self::CssImport => "CSS import sequence participates in emitted rule order",
            Self::LessImport => "Less import sequence participates in evaluation order",
            Self::CssModuleValueImport => {
                "CSS Modules value imports participate in dependency evaluation order"
            }
            Self::CssModuleComposesLocal => "local composition preserves selector dependency order",
            Self::CssModuleComposesExternal => {
                "external composition preserves module dependency order"
            }
            Self::IcssImport => "ICSS imports participate in dependency evaluation order",
        }
    }

    const fn raw_edge_kind(self) -> OmenaCrossFileSummaryRawEdgeKindV0 {
        match self {
            Self::SassUse => OmenaCrossFileSummaryRawEdgeKindV0::SassUse,
            Self::SassForward => OmenaCrossFileSummaryRawEdgeKindV0::SassForward,
            Self::SassImport => OmenaCrossFileSummaryRawEdgeKindV0::SassImport,
            Self::CssImport => OmenaCrossFileSummaryRawEdgeKindV0::CssModulesImport,
            Self::LessImport => OmenaCrossFileSummaryRawEdgeKindV0::LessImport,
            Self::CssModuleValueImport => OmenaCrossFileSummaryRawEdgeKindV0::CssModulesValueImport,
            Self::CssModuleComposesLocal => OmenaCrossFileSummaryRawEdgeKindV0::ComposesLocal,
            Self::CssModuleComposesExternal => OmenaCrossFileSummaryRawEdgeKindV0::ComposesExternal,
            Self::IcssImport => OmenaCrossFileSummaryRawEdgeKindV0::CssModulesIcssImport,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformBundleEdgeV0 {
    pub kind: TransformBundleEdgeKind,
    pub source_path: String,
    pub import_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_ordinal: Option<u32>,
    pub namespace: Option<String>,
    pub local_names: Vec<String>,
    pub remote_names: Vec<String>,
    pub range_start: u32,
    pub range_end: u32,
    pub provenance_required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformBundleAssetUrlKind {
    Relative,
    AbsolutePath,
    External,
    Data,
    Fragment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformBundleAssetUrlV0 {
    pub source_path: String,
    pub raw_url: String,
    pub normalized_url: String,
    pub kind: TransformBundleAssetUrlKind,
    pub resolved_path: Option<String>,
    pub range_start: u32,
    pub range_end: u32,
    pub bundler_resolution_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformBundleAssetUrlRewriteSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub source_path: String,
    pub asset_url_count: usize,
    pub rewrite_count: usize,
    pub output_css: String,
    pub rewritten_asset_urls: Vec<TransformBundleAssetUrlV0>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformBundleChunkKind {
    Entry,
    StyleImport,
    Asset,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformBundleChunkV0 {
    pub chunk_id: String,
    pub kind: TransformBundleChunkKind,
    pub source_path: String,
    pub import_source: Option<String>,
    pub asset_url: Option<String>,
    pub resolved_path: Option<String>,
    pub depends_on: Vec<String>,
    pub split_boundary: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformBundleSourceSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub source_path: String,
    pub dialect: &'static str,
    pub bundle_edges: Vec<TransformBundleEdgeV0>,
    pub asset_urls: Vec<TransformBundleAssetUrlV0>,
    pub code_split_chunks: Vec<TransformBundleChunkV0>,
    pub required_pass_ids: Vec<&'static str>,
    pub planned_pass_ids: Vec<&'static str>,
    pub import_inline_required: bool,
    pub module_evaluation_required: bool,
    pub css_modules_resolution_required: bool,
    pub class_hashing_required: bool,
    pub value_resolution_required: bool,
    pub code_splitting_required: bool,
    pub pass_plan: TransformPassPlanV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformBundleModuleInputV0 {
    pub source_path: String,
    pub source: String,
    pub dialect: StyleDialect,
    pub configuration_hash: ConfigurationHashV0,
}

impl TransformBundleModuleInputV0 {
    pub fn new(
        source_path: impl Into<String>,
        source: impl Into<String>,
        dialect: StyleDialect,
    ) -> Self {
        Self {
            source_path: source_path.into(),
            source: source.into(),
            dialect,
            configuration_hash: ConfigurationHashV0::none(),
        }
    }

    pub fn with_configuration_hash(mut self, configuration_hash: ConfigurationHashV0) -> Self {
        self.configuration_hash = configuration_hash;
        self
    }

    pub fn module_instance_key(&self) -> ModuleInstanceKeyV0 {
        ModuleInstanceKeyV0::new(
            ModuleIdV0::new(normalize_bundle_path(PathBuf::from(&self.source_path))),
            self.configuration_hash.clone(),
        )
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformBundleParsedModuleInputV0 {
    source_path: String,
    dialect: StyleDialect,
    facts: ParsedStyleFacts,
    emission_selectors: ParsedEmissionSelectorFactsV0,
    configuration_hashes: Vec<ConfigurationHashV0>,
}

impl TransformBundleParsedModuleInputV0 {
    pub fn new(
        source_path: impl Into<String>,
        dialect: StyleDialect,
        facts: ParsedStyleFacts,
    ) -> Self {
        Self {
            source_path: source_path.into(),
            dialect,
            facts,
            emission_selectors: ParsedEmissionSelectorFactsV0::default(),
            configuration_hashes: vec![ConfigurationHashV0::none()],
        }
    }

    pub fn with_emission_selectors(
        mut self,
        emission_selectors: ParsedEmissionSelectorFactsV0,
    ) -> Self {
        self.emission_selectors = emission_selectors;
        self
    }

    pub fn with_configuration_hashes(
        mut self,
        configuration_hashes: Vec<ConfigurationHashV0>,
    ) -> Self {
        self.configuration_hashes = configuration_hashes
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if self.configuration_hashes.is_empty() {
            self.configuration_hashes.push(ConfigurationHashV0::none());
        }
        self
    }

    pub fn source_path(&self) -> &str {
        self.source_path.as_str()
    }

    pub fn configuration_hashes(&self) -> &[ConfigurationHashV0] {
        self.configuration_hashes.as_slice()
    }

    pub fn module_instance_keys(&self) -> Vec<ModuleInstanceKeyV0> {
        let module = ModuleIdV0::new(normalize_bundle_path(PathBuf::from(&self.source_path)));
        self.configuration_hashes
            .iter()
            .cloned()
            .map(|configuration| ModuleInstanceKeyV0::new(module.clone(), configuration))
            .collect()
    }
}

#[deprecated(
    note = "use TransformBundleInstanceReachabilityInputV0 so the module instance and derivation are explicit"
)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransformBundleSemanticReachabilityInputV0 {
    pub source_path: String,
    pub class_names: Vec<String>,
    pub keyframe_names: Vec<String>,
    pub value_names: Vec<String>,
    pub custom_property_names: Vec<String>,
    pub analysis: TransformBundleReachabilityAnalysisV0,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformBundleReachabilityUnanalyzedCauseV0 {
    InputNotProvided,
    AnalysisNotAttempted,
    AnalysisResultUnavailable,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(
    tag = "state",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TransformBundleReachabilityAnalysisV0 {
    Analyzed,
    Unanalyzed {
        cause: TransformBundleReachabilityUnanalyzedCauseV0,
    },
}

impl Default for TransformBundleReachabilityAnalysisV0 {
    fn default() -> Self {
        Self::Unanalyzed {
            cause: TransformBundleReachabilityUnanalyzedCauseV0::InputNotProvided,
        }
    }
}

impl TransformBundleReachabilityAnalysisV0 {
    pub fn is_analyzed(self) -> bool {
        matches!(self, Self::Analyzed)
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InstanceReachabilityDerivationV0 {
    /// Reserved for a producer that can distinguish configured module instances.
    ///
    /// InstanceAttributed remains unproduced; path-union reachability is a disclosed over-approximation.
    InstanceAttributed,
    PathUnionNoInstanceDiscriminator,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformBundleInstanceReachabilityInputV0 {
    pub module_instance: ModuleInstanceKeyV0,
    pub class_names: Vec<String>,
    pub keyframe_names: Vec<String>,
    pub value_names: Vec<String>,
    pub custom_property_names: Vec<String>,
    pub derivation: InstanceReachabilityDerivationV0,
    pub analysis: TransformBundleReachabilityAnalysisV0,
}

impl TransformBundleInstanceReachabilityInputV0 {
    pub fn new(
        module_instance: ModuleInstanceKeyV0,
        derivation: InstanceReachabilityDerivationV0,
    ) -> Self {
        Self {
            module_instance,
            class_names: Vec::new(),
            keyframe_names: Vec::new(),
            value_names: Vec::new(),
            custom_property_names: Vec::new(),
            derivation,
            analysis: TransformBundleReachabilityAnalysisV0::Analyzed,
        }
    }

    pub fn unanalyzed(
        module_instance: ModuleInstanceKeyV0,
        derivation: InstanceReachabilityDerivationV0,
        cause: TransformBundleReachabilityUnanalyzedCauseV0,
    ) -> Self {
        Self {
            analysis: TransformBundleReachabilityAnalysisV0::Unanalyzed { cause },
            ..Self::new(module_instance, derivation)
        }
    }

    pub fn has_reachable_symbols(&self) -> bool {
        !self.class_names.is_empty()
            || !self.keyframe_names.is_empty()
            || !self.value_names.is_empty()
            || !self.custom_property_names.is_empty()
    }
}

#[allow(deprecated)]
impl TransformBundleSemanticReachabilityInputV0 {
    pub fn new(source_path: impl Into<String>) -> Self {
        Self {
            source_path: source_path.into(),
            analysis: TransformBundleReachabilityAnalysisV0::Analyzed,
            ..Self::default()
        }
    }

    pub fn unanalyzed(
        source_path: impl Into<String>,
        cause: TransformBundleReachabilityUnanalyzedCauseV0,
    ) -> Self {
        Self {
            source_path: source_path.into(),
            analysis: TransformBundleReachabilityAnalysisV0::Unanalyzed { cause },
            ..Self::default()
        }
    }

    pub fn has_reachable_symbols(&self) -> bool {
        !self.class_names.is_empty()
            || !self.keyframe_names.is_empty()
            || !self.value_names.is_empty()
            || !self.custom_property_names.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkerDependencyEdgeV0 {
    pub kind: TransformBundleEdgeKind,
    pub import_source: String,
    pub import_ordinal: Option<u32>,
    pub local_names: Vec<String>,
    pub remote_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkerRuleV0 {
    pub selector_name: String,
    #[serde(serialize_with = "serialize_selector_fact_kind")]
    pub selector_kind: ParsedSelectorFactKind,
    pub range_start: u32,
    pub range_end: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkerInputV0 {
    pub source_path: String,
    #[serde(serialize_with = "serialize_style_dialect")]
    pub dialect: StyleDialect,
    pub instance: ModuleInstanceKeyV0,
    pub dependency_edges: Vec<LinkerDependencyEdgeV0>,
    pub class_names: Vec<String>,
    pub keyframe_names: Vec<String>,
    pub value_names: Vec<String>,
    pub custom_property_names: Vec<String>,
    pub ordered_rules: Vec<LinkerRuleV0>,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformBundleLinkerProjectionV0 {
    inputs: Vec<LinkerInputV0>,
    module_reachability_evidence:
        BTreeMap<ModuleInstanceKeyV0, ClosedWorldModuleReachabilityEvidenceV0>,
    module_reachability_derivations:
        BTreeMap<ModuleInstanceKeyV0, InstanceReachabilityDerivationV0>,
    module_reachability_analysis:
        BTreeMap<ModuleInstanceKeyV0, TransformBundleReachabilityAnalysisV0>,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformBundleLinkProjectionSetV0 {
    linker_projection: TransformBundleLinkerProjectionV0,
    emission_item_projection: TransformBundleEmissionItemProjectionV0,
}

impl TransformBundleLinkProjectionSetV0 {
    pub fn linker_projection(&self) -> &TransformBundleLinkerProjectionV0 {
        &self.linker_projection
    }

    pub fn emission_item_projection(&self) -> &TransformBundleEmissionItemProjectionV0 {
        &self.emission_item_projection
    }
}

impl TransformBundleLinkerProjectionV0 {
    pub fn inputs(&self) -> &[LinkerInputV0] {
        &self.inputs
    }

    pub fn module_reachability_evidence(
        &self,
        module_instance: &ModuleInstanceKeyV0,
    ) -> ClosedWorldModuleReachabilityEvidenceV0 {
        self.module_reachability_evidence
            .get(module_instance)
            .copied()
            .unwrap_or_default()
    }

    pub fn module_reachability_derivation(
        &self,
        module_instance: &ModuleInstanceKeyV0,
    ) -> Option<InstanceReachabilityDerivationV0> {
        self.module_reachability_derivations
            .get(module_instance)
            .copied()
    }

    pub fn module_reachability_analysis(
        &self,
        module_instance: &ModuleInstanceKeyV0,
    ) -> TransformBundleReachabilityAnalysisV0 {
        self.module_reachability_analysis
            .get(module_instance)
            .copied()
            .unwrap_or_default()
    }

    pub fn analyzed_empty_reachability_input_count(&self) -> usize {
        self.inputs
            .iter()
            .filter(|input| {
                self.module_reachability_analysis(&input.instance)
                    .is_analyzed()
                    && input.class_names.is_empty()
                    && input.keyframe_names.is_empty()
                    && input.value_names.is_empty()
                    && input.custom_property_names.is_empty()
            })
            .count()
    }

    pub fn unanalyzed_reachability_input_count(&self) -> usize {
        self.inputs
            .iter()
            .filter(|input| {
                !self
                    .module_reachability_analysis(&input.instance)
                    .is_analyzed()
            })
            .count()
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformBundleDependencyResolutionV0 {
    pub attempt_state: &'static str,
    pub policy_step_keys: Vec<&'static str>,
    pub resolution_kind: Option<&'static str>,
    pub candidate_count: usize,
    pub target_instance: Option<ModuleInstanceKeyV0>,
}

impl TransformBundleDependencyResolutionV0 {
    pub fn attempted(
        policy_step_keys: Vec<&'static str>,
        resolution_kind: &'static str,
        candidate_count: usize,
        target_instance: Option<ModuleInstanceKeyV0>,
    ) -> Self {
        Self {
            attempt_state: "attempted",
            policy_step_keys,
            resolution_kind: Some(resolution_kind),
            candidate_count,
            target_instance,
        }
    }

    pub fn never_attempted(policy_step_keys: Vec<&'static str>) -> Self {
        Self {
            attempt_state: "never-attempted",
            policy_step_keys,
            resolution_kind: None,
            candidate_count: 0,
            target_instance: None,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformBundleResolvedDependencyV0 {
    pub source_instance: ModuleInstanceKeyV0,
    pub edge_kind: TransformBundleEdgeKind,
    pub import_source: String,
    pub import_ordinal: Option<u32>,
    pub resolution: TransformBundleDependencyResolutionV0,
}

impl TransformBundleResolvedDependencyV0 {
    pub fn new(
        source_instance: ModuleInstanceKeyV0,
        edge_kind: TransformBundleEdgeKind,
        import_source: impl Into<String>,
        import_ordinal: Option<u32>,
        resolution: TransformBundleDependencyResolutionV0,
    ) -> Self {
        Self {
            source_instance,
            edge_kind,
            import_source: import_source.into(),
            import_ordinal,
            resolution,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BundleResolutionAuthorityV0 {
    /// Every dependency edge must have a supplied resolved record.
    Resolved,
    /// Unmatched edges fall back to importer-relative path candidates.
    #[default]
    LegacyPathInferred,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleDependencyResolutionDisclosureV0 {
    pub source_instance: ModuleInstanceKeyV0,
    pub import_source: String,
    pub import_ordinal: Option<u32>,
    pub authority: BundleResolutionAuthorityV0,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformBundleLinkOptionsV0 {
    pub emission_ordering_policy: EmissionOrderingPolicyV0,
    pub dependency_resolution_authority: BundleResolutionAuthorityV0,
}

impl TransformBundleLinkOptionsV0 {
    pub const fn with_emission_ordering_policy(
        mut self,
        emission_ordering_policy: EmissionOrderingPolicyV0,
    ) -> Self {
        self.emission_ordering_policy = emission_ordering_policy;
        self
    }

    pub const fn with_dependency_resolution_authority(
        mut self,
        dependency_resolution_authority: BundleResolutionAuthorityV0,
    ) -> Self {
        self.dependency_resolution_authority = dependency_resolution_authority;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedStylesheetRuleV0 {
    pub global_order_index: u32,
    pub module_instance: ModuleInstanceKeyV0,
    pub selector_name: String,
    pub selector_kind: &'static str,
    pub range_start: u32,
    pub range_end: u32,
}

impl LinkedStylesheetRuleV0 {
    pub fn cascade_key_with_global_source_order(
        &self,
        level: CascadeLevel,
        layer_ordinal: LayerOrdinal,
        important: bool,
        scope_proximity: u32,
        specificity: Specificity,
        module_rank: ModuleRank,
    ) -> (CascadeKey, OpenWorldTieEvidence) {
        (
            CascadeKey::new(
                level,
                normalized_layer_rank(important, Some(layer_ordinal)),
                scope_proximity,
                specificity,
                self.global_order_index,
            ),
            OpenWorldTieEvidence::new(module_rank),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalRuleOrderV0 {
    pub rules: Vec<LinkedStylesheetRuleV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedStylesheetV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub entrypoints: Vec<ModuleInstanceKeyV0>,
    pub module_instances: Vec<ModuleInstanceKeyV0>,
    #[serde(skip_serializing)]
    pub emission_plan: EmissionPlanV0,
    pub global_rule_order: GlobalRuleOrderV0,
    pub closed_world_bundle: ClosedWorldBundleV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LinkedStylesheetWithEmissionItemsV0 {
    pub linked_stylesheet: LinkedStylesheetV0,
    pub emission_item_plan: EmissionItemPlanV0,
    pub emission_item_order: LinkedEmissionItemOrderV0,
    pub projection_disclosures: Vec<EmissionItemProjectionDisclosureV0>,
    pub dependency_resolution_disclosures: Vec<BundleDependencyResolutionDisclosureV0>,
}

/// Couples legacy admission evidence with a requested emission-policy result from one prepared graph.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformBundleEmissionAdmissionV0 {
    module_id_legacy_open: bool,
    requested_policy_result:
        Result<LinkedStylesheetWithEmissionItemsV0, TransformBundleLinkErrorV0>,
}

impl TransformBundleEmissionAdmissionV0 {
    /// Reports whether legacy module-id ordering could not produce a closed linked stylesheet.
    pub const fn module_id_legacy_open(&self) -> bool {
        self.module_id_legacy_open
    }

    /// Borrows the requested emission-policy result.
    pub fn requested_policy_result(
        &self,
    ) -> &Result<LinkedStylesheetWithEmissionItemsV0, TransformBundleLinkErrorV0> {
        &self.requested_policy_result
    }

    /// Consumes the admission evidence and returns the requested emission-policy result.
    pub fn into_requested_policy_result(
        self,
    ) -> Result<LinkedStylesheetWithEmissionItemsV0, TransformBundleLinkErrorV0> {
        self.requested_policy_result
    }

    /// Consumes the evidence into the legacy admission decision and requested result.
    pub fn into_parts(
        self,
    ) -> (
        bool,
        Result<LinkedStylesheetWithEmissionItemsV0, TransformBundleLinkErrorV0>,
    ) {
        (self.module_id_legacy_open, self.requested_policy_result)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformBundleTransformedModuleV0 {
    pub module_instance: ModuleInstanceKeyV0,
    pub output_css: String,
    pub non_empty_import_replacement_count: usize,
}

impl TransformBundleTransformedModuleV0 {
    pub fn new(module_instance: ModuleInstanceKeyV0, output_css: impl Into<String>) -> Self {
        Self {
            module_instance,
            output_css: output_css.into(),
            non_empty_import_replacement_count: 0,
        }
    }

    pub const fn with_non_empty_import_replacement_count(mut self, count: usize) -> Self {
        self.non_empty_import_replacement_count = count;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedEmissionModuleRegionV0 {
    pub module_instance: ModuleInstanceKeyV0,
    pub first_global_order_index: Option<u32>,
    pub generated_start: usize,
    pub generated_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedEmissionOrderEntryRegionV0 {
    pub global_order_index: u32,
    pub module_instance: ModuleInstanceKeyV0,
    pub generated_start: usize,
    pub generated_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedEmissionArtifactV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub output_css: String,
    pub module_regions: Vec<LinkedEmissionModuleRegionV0>,
    pub order_entry_regions: Vec<LinkedEmissionOrderEntryRegionV0>,
    pub emitted_module_count: usize,
    pub global_order_entry_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LinkedEmissionMaterializationErrorV0 {
    DuplicateTransformedModule {
        module_instance: ModuleInstanceKeyV0,
    },
    MissingTransformedModule {
        module_instance: ModuleInstanceKeyV0,
    },
    UnexpectedTransformedModule {
        module_instance: ModuleInstanceKeyV0,
    },
    ImportReplacementWouldDuplicateModule {
        module_instance: ModuleInstanceKeyV0,
        replacement_count: usize,
    },
    InvalidGlobalOrderIndex {
        expected: u32,
        actual: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmissionPolicyDifferenceV0 {
    pub output_index: u32,
    pub module_id_legacy_module: Option<ModuleInstanceKeyV0>,
    pub module_id_legacy_selector: Option<String>,
    pub import_order_module: Option<ModuleInstanceKeyV0>,
    pub import_order_selector: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmissionPolicyDifferentialReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub module_id_legacy_rule_count: usize,
    pub import_order_rule_count: usize,
    pub difference_count: usize,
    pub equivalent: bool,
    pub differences: Vec<EmissionPolicyDifferenceV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformBundleLinkErrorV0 {
    MissingEntrypoint {
        source_path: String,
    },
    AmbiguousModulePath {
        source_path: String,
    },
    MissingDependency {
        source_path: String,
        import_source: String,
    },
    UnresolvedDependencyEdge {
        source_path: String,
        import_source: String,
        import_ordinal: Option<u32>,
    },
    ClosedWorldBundle {
        error: ClosedWorldBundleBuildErrorV0,
    },
    InvalidEmissionPlan {
        reason: String,
    },
    UnsupportedEmissionCycle {
        edge_kind: TransformBundleEdgeKind,
    },
    UnsupportedDialectEmissionCycle {
        dialect: EmissionCycleDialectV0,
        class: EmissionCycleClassV0,
        edge_kinds: Vec<TransformBundleEdgeKind>,
    },
}

pub fn summarize_omena_transform_bundle_from_source(
    source_path: impl Into<String>,
    source: &str,
    dialect: StyleDialect,
) -> TransformBundleSourceSummaryV0 {
    let source_path = source_path.into();
    let facts = collect_style_facts(source, dialect);
    let bundle_edges = collect_bundle_edges_from_facts(&source_path, dialect, &facts);
    let asset_urls = collect_transform_ir_bundle_asset_urls(&source_path, source, dialect);
    let code_split_chunks = plan_bundle_code_split_chunks(&source_path, &bundle_edges, &asset_urls);
    let mut required_passes =
        required_passes_for_source(&source_path, dialect, &facts, &bundle_edges);
    required_passes.sort_by_key(|pass| transform_pass_sort_ordinal(*pass));
    required_passes.dedup();
    let pass_plan = plan_transform_passes(&required_passes);
    let planned_pass_ids = pass_plan.ordered_pass_ids.clone();
    let required_pass_ids = required_passes
        .iter()
        .map(|pass| pass.id())
        .collect::<Vec<_>>();

    TransformBundleSourceSummaryV0 {
        schema_version: "0",
        product: "omena-transform-bundle.source",
        source_path,
        dialect: dialect_label(dialect),
        bundle_edges,
        asset_urls,
        code_splitting_required: code_split_chunks.len() > 1,
        code_split_chunks,
        required_pass_ids,
        planned_pass_ids,
        import_inline_required: required_passes.contains(&TransformPassKind::ImportInline),
        module_evaluation_required: required_passes.iter().any(|pass| {
            matches!(
                pass,
                TransformPassKind::ScssModuleEvaluate | TransformPassKind::LessModuleEvaluate
            )
        }),
        css_modules_resolution_required: required_passes.iter().any(|pass| {
            matches!(
                pass,
                TransformPassKind::HashCssModuleClassNames
                    | TransformPassKind::ResolveCssModulesComposes
            )
        }),
        class_hashing_required: required_passes
            .contains(&TransformPassKind::HashCssModuleClassNames),
        value_resolution_required: required_passes.contains(&TransformPassKind::ValueResolution),
        pass_plan,
    }
}

/// Legacy LinkedStylesheetV0 entry points do not expose dependency resolution provenance.
pub fn link_omena_transform_bundle_modules<P: AsRef<str>>(
    entrypoint_paths: &[P],
    modules: &[TransformBundleModuleInputV0],
) -> Result<LinkedStylesheetV0, TransformBundleLinkErrorV0> {
    link_omena_transform_bundle_modules_with_semantic_reachability(entrypoint_paths, modules, &[])
}

/// Legacy LinkedStylesheetV0 entry points do not expose dependency resolution provenance.
#[allow(deprecated)]
pub fn link_omena_transform_bundle_modules_with_semantic_reachability<P: AsRef<str>>(
    entrypoint_paths: &[P],
    modules: &[TransformBundleModuleInputV0],
    reachability_inputs: &[TransformBundleSemanticReachabilityInputV0],
) -> Result<LinkedStylesheetV0, TransformBundleLinkErrorV0> {
    link_omena_transform_bundle_modules_with_semantic_reachability_and_metadata(
        entrypoint_paths,
        modules,
        reachability_inputs,
        &[],
    )
}

/// Legacy LinkedStylesheetV0 entry points do not expose dependency resolution provenance.
#[allow(deprecated)]
pub fn link_omena_transform_bundle_modules_with_semantic_reachability_and_metadata<
    P: AsRef<str>,
>(
    entrypoint_paths: &[P],
    modules: &[TransformBundleModuleInputV0],
    reachability_inputs: &[TransformBundleSemanticReachabilityInputV0],
    module_metadata: &[ClosedWorldModuleMetadataV0],
) -> Result<LinkedStylesheetV0, TransformBundleLinkErrorV0> {
    link_omena_transform_bundle_modules_with_options(
        entrypoint_paths,
        modules,
        reachability_inputs,
        module_metadata,
        TransformBundleLinkOptionsV0::default(),
    )
}

/// Legacy LinkedStylesheetV0 entry points do not expose dependency resolution provenance.
#[allow(deprecated)]
pub fn link_omena_transform_bundle_modules_with_options<P: AsRef<str>>(
    entrypoint_paths: &[P],
    modules: &[TransformBundleModuleInputV0],
    reachability_inputs: &[TransformBundleSemanticReachabilityInputV0],
    module_metadata: &[ClosedWorldModuleMetadataV0],
    options: TransformBundleLinkOptionsV0,
) -> Result<LinkedStylesheetV0, TransformBundleLinkErrorV0> {
    let projection = project_omena_transform_bundle_linker_inputs(modules, reachability_inputs);
    link_omena_transform_bundle_projection_with_resolved_dependencies_and_options(
        entrypoint_paths,
        &projection,
        &[],
        module_metadata,
        options,
    )
}

#[allow(deprecated)]
pub fn project_omena_transform_bundle_linker_inputs(
    modules: &[TransformBundleModuleInputV0],
    reachability_inputs: &[TransformBundleSemanticReachabilityInputV0],
) -> TransformBundleLinkerProjectionV0 {
    let parsed_modules = modules
        .iter()
        .map(|module| {
            TransformBundleParsedModuleInputV0::new(
                module.source_path.as_str(),
                module.dialect,
                collect_style_facts(module.source.as_str(), module.dialect),
            )
            .with_configuration_hashes(vec![module.configuration_hash.clone()])
        })
        .collect::<Vec<_>>();
    project_omena_transform_bundle_linker_inputs_from_parsed_modules(
        parsed_modules.as_slice(),
        reachability_inputs,
    )
}

#[allow(deprecated)]
pub fn project_omena_transform_bundle_linker_and_emission_items(
    modules: &[TransformBundleModuleInputV0],
    reachability_inputs: &[TransformBundleSemanticReachabilityInputV0],
) -> TransformBundleLinkProjectionSetV0 {
    let parsed_modules = modules
        .iter()
        .map(|module| {
            let collection = collect_style_fact_collection(module.source.as_str(), module.dialect);
            TransformBundleParsedModuleInputV0::new(
                module.source_path.as_str(),
                module.dialect,
                collection.facts,
            )
            .with_emission_selectors(collection.emission_selectors)
            .with_configuration_hashes(vec![module.configuration_hash.clone()])
        })
        .collect::<Vec<_>>();
    project_omena_transform_bundle_linker_and_emission_items_from_parsed_modules(
        parsed_modules.as_slice(),
        reachability_inputs,
    )
}

#[allow(deprecated)]
pub fn project_omena_transform_bundle_linker_inputs_from_parsed_modules(
    modules: &[TransformBundleParsedModuleInputV0],
    reachability_inputs: &[TransformBundleSemanticReachabilityInputV0],
) -> TransformBundleLinkerProjectionV0 {
    let instance_reachability_inputs =
        fan_out_path_reachability_to_instances(modules, reachability_inputs);
    project_omena_transform_bundle_linker_inputs_from_parsed_modules_with_instance_reachability(
        modules,
        instance_reachability_inputs.as_slice(),
    )
}

pub fn project_omena_transform_bundle_linker_inputs_from_parsed_modules_with_instance_reachability(
    modules: &[TransformBundleParsedModuleInputV0],
    reachability_inputs: &[TransformBundleInstanceReachabilityInputV0],
) -> TransformBundleLinkerProjectionV0 {
    let mut inputs = Vec::new();
    for module in modules {
        let source_path = normalize_bundle_path(PathBuf::from(module.source_path.as_str()));
        let bundle_edges =
            collect_bundle_edges_from_facts(&source_path, module.dialect, &module.facts);
        for instance in module.module_instance_keys() {
            inputs.push(linker_input_from_module_facts(
                source_path.as_str(),
                module.dialect,
                instance,
                &module.facts,
                bundle_edges.as_slice(),
            ));
        }
    }
    let (
        module_reachability_evidence,
        module_reachability_derivations,
        module_reachability_analysis,
    ) = apply_semantic_reachability_to_linker_inputs(inputs.as_mut_slice(), reachability_inputs);
    TransformBundleLinkerProjectionV0 {
        inputs,
        module_reachability_evidence,
        module_reachability_derivations,
        module_reachability_analysis,
    }
}

#[allow(deprecated)]
pub fn project_omena_transform_bundle_linker_and_emission_items_from_parsed_modules(
    modules: &[TransformBundleParsedModuleInputV0],
    reachability_inputs: &[TransformBundleSemanticReachabilityInputV0],
) -> TransformBundleLinkProjectionSetV0 {
    let instance_reachability_inputs =
        fan_out_path_reachability_to_instances(modules, reachability_inputs);
    project_omena_transform_bundle_linker_and_emission_items_from_parsed_modules_with_instance_reachability(
        modules,
        instance_reachability_inputs.as_slice(),
    )
}

pub fn project_omena_transform_bundle_linker_and_emission_items_from_parsed_modules_with_instance_reachability(
    modules: &[TransformBundleParsedModuleInputV0],
    reachability_inputs: &[TransformBundleInstanceReachabilityInputV0],
) -> TransformBundleLinkProjectionSetV0 {
    let linker_projection =
        project_omena_transform_bundle_linker_inputs_from_parsed_modules_with_instance_reachability(
            modules,
            reachability_inputs,
        );
    let mut emission_item_inputs = Vec::new();
    for module in modules {
        let items =
            emission_items::collect_emission_items(&module.facts, &module.emission_selectors);
        let disclosure = emission_items::emission_item_projection_disclosure(&module.facts);
        for instance in module.module_instance_keys() {
            emission_item_inputs.push(EmissionItemInputV0 {
                module_instance: instance,
                items: items.clone(),
                disclosure: disclosure.clone(),
            });
        }
    }
    TransformBundleLinkProjectionSetV0 {
        linker_projection,
        emission_item_projection: TransformBundleEmissionItemProjectionV0::new(
            emission_item_inputs,
        ),
    }
}

/// Legacy LinkedStylesheetV0 entry points do not expose dependency resolution provenance.
pub fn link_omena_transform_bundle_projection_with_resolved_dependencies_and_options<
    P: AsRef<str>,
>(
    entrypoint_paths: &[P],
    projection: &TransformBundleLinkerProjectionV0,
    resolved_dependencies: &[TransformBundleResolvedDependencyV0],
    module_metadata: &[ClosedWorldModuleMetadataV0],
    options: TransformBundleLinkOptionsV0,
) -> Result<LinkedStylesheetV0, TransformBundleLinkErrorV0> {
    let entrypoint_paths = entrypoint_paths
        .iter()
        .map(|path| path.as_ref())
        .collect::<Vec<_>>();

    link_stylesheet_from_projection_with_metadata_and_options(
        entrypoint_paths.as_slice(),
        projection.inputs(),
        resolved_dependencies,
        module_metadata,
        &projection.module_reachability_evidence,
        options,
    )
}

pub fn link_omena_transform_bundle_projection_with_emission_items_and_resolved_dependencies_and_options<
    P: AsRef<str>,
>(
    entrypoint_paths: &[P],
    linker_projection: &TransformBundleLinkerProjectionV0,
    emission_item_projection: &TransformBundleEmissionItemProjectionV0,
    resolved_dependencies: &[TransformBundleResolvedDependencyV0],
    module_metadata: &[ClosedWorldModuleMetadataV0],
    options: TransformBundleLinkOptionsV0,
) -> Result<LinkedStylesheetWithEmissionItemsV0, TransformBundleLinkErrorV0> {
    let entrypoint_paths = entrypoint_paths
        .iter()
        .map(|path| path.as_ref())
        .collect::<Vec<_>>();

    link_stylesheet_from_projection_with_emission_items_and_metadata_and_options(
        entrypoint_paths.as_slice(),
        linker_projection.inputs(),
        emission_item_projection.inputs(),
        resolved_dependencies,
        module_metadata,
        &linker_projection.module_reachability_evidence,
        options,
    )
}

pub fn link_resolved_bundle<P: AsRef<str>>(
    entrypoint_paths: &[P],
    linker_projection: &TransformBundleLinkerProjectionV0,
    emission_item_projection: &TransformBundleEmissionItemProjectionV0,
    resolved_dependencies: &[TransformBundleResolvedDependencyV0],
    module_metadata: &[ClosedWorldModuleMetadataV0],
    emission_ordering_policy: EmissionOrderingPolicyV0,
) -> Result<LinkedStylesheetWithEmissionItemsV0, TransformBundleLinkErrorV0> {
    link_omena_transform_bundle_projection_with_emission_items_and_resolved_dependencies_and_options(
        entrypoint_paths,
        linker_projection,
        emission_item_projection,
        resolved_dependencies,
        module_metadata,
        TransformBundleLinkOptionsV0::default()
            .with_emission_ordering_policy(emission_ordering_policy)
            .with_dependency_resolution_authority(BundleResolutionAuthorityV0::Resolved),
    )
}

#[deprecated(
    note = "supply resolved dependencies and use link_resolved_bundle when dependency authority must be complete"
)]
pub fn link_legacy_path_inferred_bundle<P: AsRef<str>>(
    entrypoint_paths: &[P],
    linker_projection: &TransformBundleLinkerProjectionV0,
    emission_item_projection: &TransformBundleEmissionItemProjectionV0,
    resolved_dependencies: &[TransformBundleResolvedDependencyV0],
    module_metadata: &[ClosedWorldModuleMetadataV0],
    emission_ordering_policy: EmissionOrderingPolicyV0,
) -> Result<LinkedStylesheetWithEmissionItemsV0, TransformBundleLinkErrorV0> {
    link_omena_transform_bundle_projection_with_emission_items_and_resolved_dependencies_and_options(
        entrypoint_paths,
        linker_projection,
        emission_item_projection,
        resolved_dependencies,
        module_metadata,
        TransformBundleLinkOptionsV0::default()
            .with_emission_ordering_policy(emission_ordering_policy)
            .with_dependency_resolution_authority(BundleResolutionAuthorityV0::LegacyPathInferred),
    )
}

/// Evaluates legacy admission and the requested emission policy from one closed-world preparation.
pub fn evaluate_omena_transform_bundle_projection_emission_admission_with_resolved_dependencies_and_options<
    P: AsRef<str>,
>(
    entrypoint_paths: &[P],
    linker_projection: &TransformBundleLinkerProjectionV0,
    emission_item_projection: &TransformBundleEmissionItemProjectionV0,
    resolved_dependencies: &[TransformBundleResolvedDependencyV0],
    module_metadata: &[ClosedWorldModuleMetadataV0],
    options: TransformBundleLinkOptionsV0,
) -> TransformBundleEmissionAdmissionV0 {
    let entrypoint_paths = entrypoint_paths
        .iter()
        .map(|path| path.as_ref())
        .collect::<Vec<_>>();

    evaluate_stylesheet_emission_admission_from_projection(
        entrypoint_paths.as_slice(),
        linker_projection.inputs(),
        emission_item_projection.inputs(),
        resolved_dependencies,
        module_metadata,
        &linker_projection.module_reachability_evidence,
        options,
    )
}

pub fn compare_omena_transform_bundle_emission_policies<P: AsRef<str>>(
    entrypoint_paths: &[P],
    modules: &[TransformBundleModuleInputV0],
) -> Result<EmissionPolicyDifferentialReportV0, TransformBundleLinkErrorV0> {
    let module_id_legacy = link_omena_transform_bundle_modules_with_options(
        entrypoint_paths,
        modules,
        &[],
        &[],
        TransformBundleLinkOptionsV0 {
            emission_ordering_policy: EmissionOrderingPolicyV0::ModuleIdLegacy,
            ..TransformBundleLinkOptionsV0::default()
        },
    )?;
    let import_order = link_omena_transform_bundle_modules_with_options(
        entrypoint_paths,
        modules,
        &[],
        &[],
        TransformBundleLinkOptionsV0 {
            emission_ordering_policy: EmissionOrderingPolicyV0::ImportOrderPreserving,
            ..TransformBundleLinkOptionsV0::default()
        },
    )?;
    let module_id_legacy_rules = &module_id_legacy.global_rule_order.rules;
    let import_order_rules = &import_order.global_rule_order.rules;
    let mut differences = Vec::new();
    for output_index in 0..module_id_legacy_rules.len().max(import_order_rules.len()) {
        let module_id_legacy_rule = module_id_legacy_rules.get(output_index);
        let import_order_rule = import_order_rules.get(output_index);
        if module_id_legacy_rule == import_order_rule {
            continue;
        }
        differences.push(EmissionPolicyDifferenceV0 {
            output_index: u32::try_from(output_index).map_err(|_| {
                TransformBundleLinkErrorV0::InvalidEmissionPlan {
                    reason: "policy differential has more rows than the output index can represent"
                        .to_string(),
                }
            })?,
            module_id_legacy_module: module_id_legacy_rule.map(|rule| rule.module_instance.clone()),
            module_id_legacy_selector: module_id_legacy_rule.map(|rule| rule.selector_name.clone()),
            import_order_module: import_order_rule.map(|rule| rule.module_instance.clone()),
            import_order_selector: import_order_rule.map(|rule| rule.selector_name.clone()),
        });
    }
    let difference_count = differences.len();
    Ok(EmissionPolicyDifferentialReportV0 {
        schema_version: "0",
        product: "omena-bundler.emission-policy-differential",
        module_id_legacy_rule_count: module_id_legacy_rules.len(),
        import_order_rule_count: import_order_rules.len(),
        difference_count,
        equivalent: difference_count == 0,
        differences,
    })
}

pub fn materialize_omena_transform_bundle_linked_stylesheet(
    linked: &LinkedStylesheetV0,
    transformed_modules: &[TransformBundleTransformedModuleV0],
) -> Result<LinkedEmissionArtifactV0, LinkedEmissionMaterializationErrorV0> {
    let (module_order, first_order_index_by_instance) =
        legacy_materialization_module_order(linked)?;
    materialize_linked_stylesheet_in_module_order(
        linked,
        transformed_modules,
        module_order,
        &first_order_index_by_instance,
    )
}

pub fn materialize_omena_transform_bundle_linked_stylesheet_with_emission_items(
    linked: &LinkedStylesheetWithEmissionItemsV0,
    transformed_modules: &[TransformBundleTransformedModuleV0],
) -> Result<LinkedEmissionArtifactV0, LinkedEmissionItemMaterializationErrorV0> {
    let linked_modules = linked
        .linked_stylesheet
        .module_instances
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut represented_modules = BTreeSet::new();
    let mut module_order = Vec::new();
    for (expected_index, item) in linked.emission_item_order.items.iter().enumerate() {
        let expected_index = u32::try_from(expected_index).unwrap_or(u32::MAX);
        if item.global_order_index != expected_index {
            return Err(
                LinkedEmissionItemMaterializationErrorV0::InvalidItemOrderIndex {
                    expected: expected_index,
                    actual: item.global_order_index,
                },
            );
        }
        if !linked_modules.contains(&item.module_instance) {
            return Err(
                LinkedEmissionItemMaterializationErrorV0::UnknownEmissionItemModule {
                    module_instance: item.module_instance.clone(),
                },
            );
        }
        if represented_modules.insert(item.module_instance.clone()) {
            module_order.push(item.module_instance.clone());
        }
    }
    for module_instance in &linked.linked_stylesheet.module_instances {
        if !represented_modules.contains(module_instance) {
            return Err(
                LinkedEmissionItemMaterializationErrorV0::MissingEmissionItem {
                    module_instance: module_instance.clone(),
                },
            );
        }
    }

    let (_, first_order_index_by_instance) =
        selector_materialization_module_order(&linked.linked_stylesheet)?;
    materialize_linked_stylesheet_in_module_order(
        &linked.linked_stylesheet,
        transformed_modules,
        module_order,
        &first_order_index_by_instance,
    )
    .map_err(LinkedEmissionItemMaterializationErrorV0::from)
}

fn legacy_materialization_module_order(
    linked: &LinkedStylesheetV0,
) -> Result<
    (Vec<ModuleInstanceKeyV0>, BTreeMap<ModuleInstanceKeyV0, u32>),
    LinkedEmissionMaterializationErrorV0,
> {
    let (mut module_order, first_order_index_by_instance) =
        selector_materialization_module_order(linked)?;
    for module_instance in &linked.module_instances {
        if !first_order_index_by_instance.contains_key(module_instance) {
            module_order.push(module_instance.clone());
        }
    }
    Ok((module_order, first_order_index_by_instance))
}

fn selector_materialization_module_order(
    linked: &LinkedStylesheetV0,
) -> Result<
    (Vec<ModuleInstanceKeyV0>, BTreeMap<ModuleInstanceKeyV0, u32>),
    LinkedEmissionMaterializationErrorV0,
> {
    let mut first_order_index_by_instance = BTreeMap::new();
    let mut module_order = Vec::new();
    for (expected_index, rule) in linked.global_rule_order.rules.iter().enumerate() {
        let expected_index = u32::try_from(expected_index).unwrap_or(u32::MAX);
        if rule.global_order_index != expected_index {
            return Err(
                LinkedEmissionMaterializationErrorV0::InvalidGlobalOrderIndex {
                    expected: expected_index,
                    actual: rule.global_order_index,
                },
            );
        }
        if first_order_index_by_instance
            .insert(rule.module_instance.clone(), rule.global_order_index)
            .is_none()
        {
            module_order.push(rule.module_instance.clone());
        }
    }
    Ok((module_order, first_order_index_by_instance))
}

fn materialize_linked_stylesheet_in_module_order(
    linked: &LinkedStylesheetV0,
    transformed_modules: &[TransformBundleTransformedModuleV0],
    module_order: Vec<ModuleInstanceKeyV0>,
    first_order_index_by_instance: &BTreeMap<ModuleInstanceKeyV0, u32>,
) -> Result<LinkedEmissionArtifactV0, LinkedEmissionMaterializationErrorV0> {
    let linked_modules = linked
        .module_instances
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut transformed_by_instance = BTreeMap::new();
    for transformed in transformed_modules {
        if !linked_modules.contains(&transformed.module_instance) {
            return Err(
                LinkedEmissionMaterializationErrorV0::UnexpectedTransformedModule {
                    module_instance: transformed.module_instance.clone(),
                },
            );
        }
        if transformed.non_empty_import_replacement_count > 0 {
            return Err(
                LinkedEmissionMaterializationErrorV0::ImportReplacementWouldDuplicateModule {
                    module_instance: transformed.module_instance.clone(),
                    replacement_count: transformed.non_empty_import_replacement_count,
                },
            );
        }
        if transformed_by_instance
            .insert(transformed.module_instance.clone(), transformed)
            .is_some()
        {
            return Err(
                LinkedEmissionMaterializationErrorV0::DuplicateTransformedModule {
                    module_instance: transformed.module_instance.clone(),
                },
            );
        }
    }

    for module_instance in &linked.module_instances {
        if !transformed_by_instance.contains_key(module_instance) {
            return Err(
                LinkedEmissionMaterializationErrorV0::MissingTransformedModule {
                    module_instance: module_instance.clone(),
                },
            );
        }
    }

    let mut output_css = String::new();
    let mut module_regions = Vec::with_capacity(module_order.len());
    let mut generated_region_by_instance = BTreeMap::new();
    for module_instance in module_order {
        let Some(transformed) = transformed_by_instance.get(&module_instance) else {
            return Err(
                LinkedEmissionMaterializationErrorV0::MissingTransformedModule { module_instance },
            );
        };
        if !output_css.is_empty()
            && !output_css.ends_with('\n')
            && !transformed.output_css.is_empty()
        {
            output_css.push('\n');
        }
        let generated_start = output_css.len();
        output_css.push_str(&transformed.output_css);
        let generated_end = output_css.len();
        generated_region_by_instance
            .insert(module_instance.clone(), (generated_start, generated_end));
        module_regions.push(LinkedEmissionModuleRegionV0 {
            first_global_order_index: first_order_index_by_instance.get(&module_instance).copied(),
            module_instance,
            generated_start,
            generated_end,
        });
    }

    let mut order_entry_regions = Vec::with_capacity(linked.global_rule_order.rules.len());
    for rule in &linked.global_rule_order.rules {
        let Some((generated_start, generated_end)) = generated_region_by_instance
            .get(&rule.module_instance)
            .copied()
        else {
            return Err(
                LinkedEmissionMaterializationErrorV0::MissingTransformedModule {
                    module_instance: rule.module_instance.clone(),
                },
            );
        };
        order_entry_regions.push(LinkedEmissionOrderEntryRegionV0 {
            global_order_index: rule.global_order_index,
            module_instance: rule.module_instance.clone(),
            generated_start,
            generated_end,
        });
    }

    Ok(LinkedEmissionArtifactV0 {
        schema_version: "0",
        product: "omena-transform-bundle.linked-emission",
        emitted_module_count: module_regions.len(),
        global_order_entry_count: order_entry_regions.len(),
        output_css,
        module_regions,
        order_entry_regions,
    })
}

/// Legacy LinkedStylesheetV0 entry points do not expose dependency resolution provenance.
pub fn link_stylesheet_from_projection(
    entrypoint_paths: &[&str],
    inputs: &[LinkerInputV0],
) -> Result<LinkedStylesheetV0, TransformBundleLinkErrorV0> {
    link_stylesheet_from_projection_with_options(
        entrypoint_paths,
        inputs,
        TransformBundleLinkOptionsV0::default(),
    )
}

/// Legacy LinkedStylesheetV0 entry points do not expose dependency resolution provenance.
pub fn link_stylesheet_from_projection_with_options(
    entrypoint_paths: &[&str],
    inputs: &[LinkerInputV0],
    options: TransformBundleLinkOptionsV0,
) -> Result<LinkedStylesheetV0, TransformBundleLinkErrorV0> {
    link_stylesheet_from_projection_with_resolved_dependencies_and_options(
        entrypoint_paths,
        inputs,
        &[],
        options,
    )
}

/// Legacy LinkedStylesheetV0 entry points do not expose dependency resolution provenance.
pub fn link_stylesheet_from_projection_with_resolved_dependencies_and_options(
    entrypoint_paths: &[&str],
    inputs: &[LinkerInputV0],
    resolved_dependencies: &[TransformBundleResolvedDependencyV0],
    options: TransformBundleLinkOptionsV0,
) -> Result<LinkedStylesheetV0, TransformBundleLinkErrorV0> {
    link_stylesheet_from_projection_with_metadata_and_options(
        entrypoint_paths,
        inputs,
        resolved_dependencies,
        &[],
        &BTreeMap::new(),
        options,
    )
}

fn link_stylesheet_from_projection_with_metadata_and_options(
    entrypoint_paths: &[&str],
    inputs: &[LinkerInputV0],
    resolved_dependencies: &[TransformBundleResolvedDependencyV0],
    module_metadata: &[ClosedWorldModuleMetadataV0],
    module_reachability_evidence: &BTreeMap<
        ModuleInstanceKeyV0,
        ClosedWorldModuleReachabilityEvidenceV0,
    >,
    options: TransformBundleLinkOptionsV0,
) -> Result<LinkedStylesheetV0, TransformBundleLinkErrorV0> {
    let prepared = prepare_linked_stylesheet_context(
        entrypoint_paths,
        inputs,
        resolved_dependencies,
        module_metadata,
        module_reachability_evidence,
        options.dependency_resolution_authority,
    )?;
    link_stylesheet_from_prepared_context(prepared, inputs, resolved_dependencies, options)
}

fn link_stylesheet_from_prepared_context(
    prepared: PreparedLinkedStylesheetContextV0,
    inputs: &[LinkerInputV0],
    resolved_dependencies: &[TransformBundleResolvedDependencyV0],
    options: TransformBundleLinkOptionsV0,
) -> Result<LinkedStylesheetV0, TransformBundleLinkErrorV0> {
    let (emission_plan, global_rule_order) = build_linked_stylesheet_order_from_prepared_context(
        &prepared,
        inputs,
        resolved_dependencies,
        options,
    )?;
    Ok(LinkedStylesheetV0 {
        schema_version: "0",
        product: "omena-transform-bundle.linked-stylesheet",
        entrypoints: prepared.entrypoints,
        module_instances: prepared.closed_world_bundle.linked_modules().to_vec(),
        emission_plan,
        global_rule_order,
        closed_world_bundle: prepared.closed_world_bundle,
    })
}

fn build_linked_stylesheet_order_from_prepared_context(
    prepared: &PreparedLinkedStylesheetContextV0,
    inputs: &[LinkerInputV0],
    resolved_dependencies: &[TransformBundleResolvedDependencyV0],
    options: TransformBundleLinkOptionsV0,
) -> Result<(EmissionPlanV0, GlobalRuleOrderV0), TransformBundleLinkErrorV0> {
    let emission_plan = emission_order::build_emission_plan(
        inputs,
        prepared.closed_world_bundle.linked_modules(),
        &prepared.entrypoints,
        resolved_dependencies,
        options.emission_ordering_policy,
        options.dependency_resolution_authority,
    )?;
    let global_rule_order =
        emission_order::build_global_rule_order_from_plan(inputs, &emission_plan)?;
    Ok((emission_plan, global_rule_order))
}

struct PreparedLinkedStylesheetContextV0 {
    entrypoints: Vec<ModuleInstanceKeyV0>,
    closed_world_bundle: ClosedWorldBundleV0,
    dependency_resolution_disclosures: Vec<BundleDependencyResolutionDisclosureV0>,
}

fn prepare_linked_stylesheet_context(
    entrypoint_paths: &[&str],
    inputs: &[LinkerInputV0],
    resolved_dependencies: &[TransformBundleResolvedDependencyV0],
    module_metadata: &[ClosedWorldModuleMetadataV0],
    module_reachability_evidence: &BTreeMap<
        ModuleInstanceKeyV0,
        ClosedWorldModuleReachabilityEvidenceV0,
    >,
    resolution_authority: BundleResolutionAuthorityV0,
) -> Result<PreparedLinkedStylesheetContextV0, TransformBundleLinkErrorV0> {
    let instances_by_path = module_instances_by_linker_path(inputs);
    let entrypoints = entrypoint_paths
        .iter()
        .map(|path| {
            resolve_entrypoint_module_instance_by_path(path, &instances_by_path)?.ok_or_else(|| {
                TransformBundleLinkErrorV0::MissingEntrypoint {
                    source_path: normalize_bundle_path(PathBuf::from(*path)),
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (linked_modules, dependency_resolution_disclosures) =
        collect_closed_world_linked_modules_from_projection(
            inputs,
            resolved_dependencies,
            &instances_by_path,
            resolution_authority,
        )?;
    let module_metadata =
        module_metadata_with_reachability_evidence(module_metadata, module_reachability_evidence);
    let closed_world_bundle = ClosedWorldBundleV0::try_from_linked_modules_with_metadata(
        entrypoints.clone(),
        linked_modules,
        module_metadata,
    )
    .map_err(|error| TransformBundleLinkErrorV0::ClosedWorldBundle { error })?;
    Ok(PreparedLinkedStylesheetContextV0 {
        entrypoints,
        closed_world_bundle,
        dependency_resolution_disclosures,
    })
}

fn link_stylesheet_from_projection_with_emission_items_and_metadata_and_options(
    entrypoint_paths: &[&str],
    linker_inputs: &[LinkerInputV0],
    emission_item_inputs: &[EmissionItemInputV0],
    resolved_dependencies: &[TransformBundleResolvedDependencyV0],
    module_metadata: &[ClosedWorldModuleMetadataV0],
    module_reachability_evidence: &BTreeMap<
        ModuleInstanceKeyV0,
        ClosedWorldModuleReachabilityEvidenceV0,
    >,
    options: TransformBundleLinkOptionsV0,
) -> Result<LinkedStylesheetWithEmissionItemsV0, TransformBundleLinkErrorV0> {
    let prepared = prepare_linked_stylesheet_context(
        entrypoint_paths,
        linker_inputs,
        resolved_dependencies,
        module_metadata,
        module_reachability_evidence,
        options.dependency_resolution_authority,
    )?;
    link_stylesheet_from_prepared_context_with_emission_items(
        prepared,
        linker_inputs,
        emission_item_inputs,
        resolved_dependencies,
        options,
    )
}

fn evaluate_stylesheet_emission_admission_from_projection(
    entrypoint_paths: &[&str],
    linker_inputs: &[LinkerInputV0],
    emission_item_inputs: &[EmissionItemInputV0],
    resolved_dependencies: &[TransformBundleResolvedDependencyV0],
    module_metadata: &[ClosedWorldModuleMetadataV0],
    module_reachability_evidence: &BTreeMap<
        ModuleInstanceKeyV0,
        ClosedWorldModuleReachabilityEvidenceV0,
    >,
    options: TransformBundleLinkOptionsV0,
) -> TransformBundleEmissionAdmissionV0 {
    let prepared = match prepare_linked_stylesheet_context(
        entrypoint_paths,
        linker_inputs,
        resolved_dependencies,
        module_metadata,
        module_reachability_evidence,
        options.dependency_resolution_authority,
    ) {
        Ok(prepared) => prepared,
        Err(error) => {
            return TransformBundleEmissionAdmissionV0 {
                module_id_legacy_open: true,
                requested_policy_result: Err(error),
            };
        }
    };
    let module_id_legacy_open = build_linked_stylesheet_order_from_prepared_context(
        &prepared,
        linker_inputs,
        resolved_dependencies,
        TransformBundleLinkOptionsV0::default()
            .with_dependency_resolution_authority(options.dependency_resolution_authority),
    )
    .is_err();
    let requested_policy_result = link_stylesheet_from_prepared_context_with_emission_items(
        prepared,
        linker_inputs,
        emission_item_inputs,
        resolved_dependencies,
        options,
    );
    TransformBundleEmissionAdmissionV0 {
        module_id_legacy_open,
        requested_policy_result,
    }
}

fn link_stylesheet_from_prepared_context_with_emission_items(
    prepared: PreparedLinkedStylesheetContextV0,
    linker_inputs: &[LinkerInputV0],
    emission_item_inputs: &[EmissionItemInputV0],
    resolved_dependencies: &[TransformBundleResolvedDependencyV0],
    options: TransformBundleLinkOptionsV0,
) -> Result<LinkedStylesheetWithEmissionItemsV0, TransformBundleLinkErrorV0> {
    let module_plan = emission_order::build_emission_module_plan(
        linker_inputs,
        prepared.closed_world_bundle.linked_modules(),
        &prepared.entrypoints,
        resolved_dependencies,
        options.emission_ordering_policy,
        options.dependency_resolution_authority,
    )?;
    let emission_plan =
        emission_order::build_emission_plan_from_module_plan(linker_inputs, &module_plan)?;
    let legacy_global_rule_order =
        emission_order::build_global_rule_order_from_plan(linker_inputs, &emission_plan)?;
    let emission_item_plan =
        emission_items::build_emission_item_plan(emission_item_inputs, &module_plan)?;
    let emission_item_order = emission_items::build_linked_emission_item_order(
        emission_item_inputs,
        &emission_item_plan,
    )?;
    let global_rule_order =
        emission_items::build_global_rule_order_from_emission_items(&emission_item_order)?;
    if global_rule_order != legacy_global_rule_order {
        return Err(TransformBundleLinkErrorV0::InvalidEmissionPlan {
            reason: "selector projection from emission items changed global rule order".to_string(),
        });
    }
    let projection_disclosures = emission_item_inputs
        .first()
        .map(|input| input.disclosure.clone())
        .unwrap_or_default();
    if emission_item_inputs
        .iter()
        .any(|input| input.disclosure != projection_disclosures)
    {
        return Err(TransformBundleLinkErrorV0::InvalidEmissionPlan {
            reason: "emission-item projection disclosure differs between modules".to_string(),
        });
    }
    let linked_stylesheet = LinkedStylesheetV0 {
        schema_version: "0",
        product: "omena-transform-bundle.linked-stylesheet",
        entrypoints: prepared.entrypoints,
        module_instances: prepared.closed_world_bundle.linked_modules().to_vec(),
        emission_plan,
        global_rule_order,
        closed_world_bundle: prepared.closed_world_bundle,
    };
    Ok(LinkedStylesheetWithEmissionItemsV0 {
        linked_stylesheet,
        emission_item_plan,
        emission_item_order,
        projection_disclosures,
        dependency_resolution_disclosures: prepared.dependency_resolution_disclosures,
    })
}

pub fn rewrite_omena_transform_bundle_asset_urls_in_source(
    source_path: impl Into<String>,
    source: &str,
) -> TransformBundleAssetUrlRewriteSummaryV0 {
    let source_path = source_path.into();
    let asset_urls = collect_transform_ir_bundle_asset_urls(
        &source_path,
        source,
        dialect_for_bundle_source_path(&source_path),
    );
    let mut output_css = source.to_string();
    let mut rewritten_asset_urls = Vec::new();

    for asset in asset_urls.iter().rev() {
        let Some(resolved_path) = asset.resolved_path.as_deref() else {
            continue;
        };
        if !asset.bundler_resolution_required || asset.normalized_url == resolved_path {
            continue;
        }
        let range_start = asset.range_start as usize;
        let range_end = asset.range_end as usize;
        if range_start > range_end || range_end > output_css.len() {
            continue;
        }
        output_css.replace_range(range_start..range_end, &format!("url(\"{resolved_path}\")"));
        rewritten_asset_urls.push(asset.clone());
    }

    rewritten_asset_urls.reverse();
    TransformBundleAssetUrlRewriteSummaryV0 {
        schema_version: "0",
        product: "omena-transform-bundle.asset-url-rewrite",
        source_path,
        asset_url_count: asset_urls.len(),
        rewrite_count: rewritten_asset_urls.len(),
        output_css,
        rewritten_asset_urls,
    }
}

fn linker_input_from_module_facts(
    source_path: &str,
    dialect: StyleDialect,
    instance: ModuleInstanceKeyV0,
    facts: &ParsedStyleFacts,
    bundle_edges: &[TransformBundleEdgeV0],
) -> LinkerInputV0 {
    LinkerInputV0 {
        source_path: source_path.to_string(),
        dialect,
        instance,
        dependency_edges: bundle_edges
            .iter()
            .filter(|edge| bundle_edge_is_module_dependency(edge.kind))
            .filter_map(|edge| {
                edge.import_source
                    .as_ref()
                    .map(|import_source| LinkerDependencyEdgeV0 {
                        kind: edge.kind,
                        import_source: import_source.clone(),
                        import_ordinal: edge.import_ordinal,
                        local_names: edge.local_names.clone(),
                        remote_names: edge.remote_names.clone(),
                    })
            })
            .collect(),
        class_names: dedupe_names(
            facts
                .selectors
                .iter()
                .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
                .map(|selector| selector.name.clone()),
        ),
        keyframe_names: dedupe_names(
            facts
                .animations
                .iter()
                .filter(|animation| animation.kind == ParsedAnimationFactKind::KeyframesDeclaration)
                .map(|animation| animation.name.clone()),
        ),
        value_names: dedupe_names(
            facts
                .css_module_values
                .iter()
                .filter(|value| value.kind == ParsedCssModuleValueFactKind::Definition)
                .map(|value| value.name.clone()),
        ),
        custom_property_names: dedupe_custom_property_names(
            facts
                .variables
                .iter()
                .filter(|variable| {
                    variable.kind == ParsedVariableFactKind::CustomPropertyDeclaration
                })
                .map(|variable| variable.name.clone()),
        ),
        ordered_rules: collect_ordered_linker_rules(facts),
    }
}

fn collect_ordered_linker_rules(facts: &ParsedStyleFacts) -> Vec<LinkerRuleV0> {
    let mut selectors = facts.selectors.clone();
    selectors.sort_by_key(|selector| {
        (
            u32::from(selector.range.start()),
            u32::from(selector.range.end()),
            selector.kind,
            selector.name.clone(),
        )
    });
    selectors
        .into_iter()
        .map(|selector| LinkerRuleV0 {
            selector_name: selector.name,
            selector_kind: selector.kind,
            range_start: u32::from(selector.range.start()),
            range_end: u32::from(selector.range.end()),
        })
        .collect()
}

#[allow(deprecated)]
fn fan_out_path_reachability_to_instances(
    modules: &[TransformBundleParsedModuleInputV0],
    reachability_inputs: &[TransformBundleSemanticReachabilityInputV0],
) -> Vec<TransformBundleInstanceReachabilityInputV0> {
    let instances_by_path = modules.iter().fold(
        BTreeMap::<String, Vec<ModuleInstanceKeyV0>>::new(),
        |mut by_path, module| {
            by_path
                .entry(normalize_bundle_path(PathBuf::from(module.source_path())))
                .or_default()
                .extend(module.module_instance_keys());
            by_path
        },
    );
    let mut reachability_by_path =
        BTreeMap::<String, TransformBundleSemanticReachabilityInputV0>::new();
    for input in reachability_inputs {
        let normalized_path = normalize_bundle_path(PathBuf::from(&input.source_path));
        let merged = reachability_by_path
            .entry(normalized_path.clone())
            .or_insert_with(|| TransformBundleSemanticReachabilityInputV0::new(normalized_path));
        merged.analysis = merge_reachability_analysis(merged.analysis, input.analysis);
        merged.class_names.extend(input.class_names.iter().cloned());
        merged
            .keyframe_names
            .extend(input.keyframe_names.iter().cloned());
        merged.value_names.extend(input.value_names.iter().cloned());
        merged
            .custom_property_names
            .extend(input.custom_property_names.iter().cloned());
        merged.class_names = dedupe_names(merged.class_names.drain(..));
        merged.keyframe_names = dedupe_names(merged.keyframe_names.drain(..));
        merged.value_names = dedupe_names(merged.value_names.drain(..));
        merged.custom_property_names =
            dedupe_custom_property_names(merged.custom_property_names.drain(..));
    }

    reachability_by_path
        .into_iter()
        .flat_map(|(path, reachability)| {
            instances_by_path
                .get(path.as_str())
                .into_iter()
                .flatten()
                .map(move |instance| {
                    let mut input = TransformBundleInstanceReachabilityInputV0::new(
                        instance.clone(),
                        InstanceReachabilityDerivationV0::PathUnionNoInstanceDiscriminator,
                    );
                    input.class_names.clone_from(&reachability.class_names);
                    input
                        .keyframe_names
                        .clone_from(&reachability.keyframe_names);
                    input.value_names.clone_from(&reachability.value_names);
                    input
                        .custom_property_names
                        .clone_from(&reachability.custom_property_names);
                    input.analysis = reachability.analysis;
                    input
                })
        })
        .collect()
}

fn apply_semantic_reachability_to_linker_inputs(
    inputs: &mut [LinkerInputV0],
    reachability_inputs: &[TransformBundleInstanceReachabilityInputV0],
) -> (
    BTreeMap<ModuleInstanceKeyV0, ClosedWorldModuleReachabilityEvidenceV0>,
    BTreeMap<ModuleInstanceKeyV0, InstanceReachabilityDerivationV0>,
    BTreeMap<ModuleInstanceKeyV0, TransformBundleReachabilityAnalysisV0>,
) {
    let (reachability_inputs, incomplete_composes_target_instances) =
        instance_reachability_inputs_closed_over_composes(inputs, reachability_inputs);
    let module_index_by_instance = inputs
        .iter()
        .enumerate()
        .map(|(index, input)| (input.instance.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let mut evidence_by_instance = inputs
        .iter()
        .map(|input| {
            (
                input.instance.clone(),
                ClosedWorldModuleReachabilityEvidenceV0::ModuleReachabilityInputAbsent,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut derivation_by_instance = BTreeMap::new();
    let mut analysis_by_instance = inputs
        .iter()
        .map(|input| {
            (
                input.instance.clone(),
                TransformBundleReachabilityAnalysisV0::default(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    for input in reachability_inputs.values() {
        let Some(index) = module_index_by_instance
            .get(&input.module_instance)
            .copied()
        else {
            continue;
        };
        derivation_by_instance.insert(input.module_instance.clone(), input.derivation);
        analysis_by_instance.insert(input.module_instance.clone(), input.analysis);
        if incomplete_composes_target_instances.contains(&input.module_instance) {
            // A composed-name carrier with a missing side cannot justify filtering its closure
            // target. Typed absence is attached to the module whose declarations could be lost.
            analysis_by_instance.insert(
                input.module_instance.clone(),
                TransformBundleReachabilityAnalysisV0::Unanalyzed {
                    cause: TransformBundleReachabilityUnanalyzedCauseV0::AnalysisResultUnavailable,
                },
            );
            continue;
        }
        if !input.analysis.is_analyzed() {
            continue;
        }
        evidence_by_instance.insert(
            input.module_instance.clone(),
            ClosedWorldModuleReachabilityEvidenceV0::Supplied,
        );
        inputs[index].class_names.clear();
        inputs[index]
            .class_names
            .extend(input.class_names.iter().cloned());
        inputs[index].class_names = dedupe_names(inputs[index].class_names.drain(..));
        inputs[index].keyframe_names.clear();
        inputs[index]
            .keyframe_names
            .extend(input.keyframe_names.iter().cloned());
        inputs[index].keyframe_names = dedupe_names(inputs[index].keyframe_names.drain(..));
        inputs[index].value_names.clear();
        inputs[index]
            .value_names
            .extend(input.value_names.iter().cloned());
        inputs[index].value_names = dedupe_names(inputs[index].value_names.drain(..));
        inputs[index].custom_property_names.clear();
        inputs[index]
            .custom_property_names
            .extend(input.custom_property_names.iter().cloned());
        inputs[index].custom_property_names =
            dedupe_custom_property_names(inputs[index].custom_property_names.drain(..));
    }
    (
        evidence_by_instance,
        derivation_by_instance,
        analysis_by_instance,
    )
}

fn instance_reachability_inputs_closed_over_composes(
    inputs: &[LinkerInputV0],
    reachability_inputs: &[TransformBundleInstanceReachabilityInputV0],
) -> (
    BTreeMap<ModuleInstanceKeyV0, TransformBundleInstanceReachabilityInputV0>,
    BTreeSet<ModuleInstanceKeyV0>,
) {
    let instances_by_path = module_instances_by_linker_path(inputs);
    let mut by_instance =
        BTreeMap::<ModuleInstanceKeyV0, TransformBundleInstanceReachabilityInputV0>::new();
    let mut incomplete_composes_target_instances = BTreeSet::new();
    for input in reachability_inputs {
        let merged = by_instance
            .entry(input.module_instance.clone())
            .or_insert_with(|| {
                TransformBundleInstanceReachabilityInputV0::new(
                    input.module_instance.clone(),
                    input.derivation,
                )
            });
        if input.derivation == InstanceReachabilityDerivationV0::PathUnionNoInstanceDiscriminator {
            merged.derivation = InstanceReachabilityDerivationV0::PathUnionNoInstanceDiscriminator;
        }
        merged.analysis = merge_reachability_analysis(merged.analysis, input.analysis);
        merged.class_names.extend(input.class_names.iter().cloned());
        merged
            .keyframe_names
            .extend(input.keyframe_names.iter().cloned());
        merged.value_names.extend(input.value_names.iter().cloned());
        merged
            .custom_property_names
            .extend(input.custom_property_names.iter().cloned());
        merged.class_names = dedupe_names(merged.class_names.drain(..));
        merged.keyframe_names = dedupe_names(merged.keyframe_names.drain(..));
        merged.value_names = dedupe_names(merged.value_names.drain(..));
        merged.custom_property_names =
            dedupe_custom_property_names(merged.custom_property_names.drain(..));
    }

    loop {
        let snapshot = by_instance.clone();
        let mut additions = BTreeMap::<ModuleInstanceKeyV0, Vec<String>>::new();
        for input in inputs {
            let Some(source_reachability) = snapshot.get(&input.instance) else {
                continue;
            };
            if !source_reachability.analysis.is_analyzed() {
                continue;
            }
            for edge in input
                .dependency_edges
                .iter()
                .filter(|edge| edge.kind == TransformBundleEdgeKind::CssModuleComposesExternal)
            {
                let target_path =
                    import_path_candidates(input.source_path.as_str(), edge.import_source.as_str())
                        .into_iter()
                        .find(|candidate| instances_by_path.contains_key(candidate));
                let Some(target_path) = target_path else {
                    continue;
                };
                let Some(target_instances) = instances_by_path.get(&target_path) else {
                    continue;
                };
                if edge.local_names.is_empty() || edge.remote_names.is_empty() {
                    incomplete_composes_target_instances.extend(target_instances.iter().cloned());
                    continue;
                }
                for local_name in &edge.local_names {
                    if !source_reachability
                        .class_names
                        .iter()
                        .any(|reachable| reachable == local_name)
                    {
                        continue;
                    }
                    for remote_name in &edge.remote_names {
                        for target_instance in target_instances {
                            additions
                                .entry(target_instance.clone())
                                .or_default()
                                .push(remote_name.clone());
                        }
                    }
                }
            }
        }
        let mut changed = false;
        for (target_instance, class_names) in additions {
            let target = by_instance
                .entry(target_instance.clone())
                .or_insert_with(|| {
                    TransformBundleInstanceReachabilityInputV0::new(
                        target_instance,
                        InstanceReachabilityDerivationV0::PathUnionNoInstanceDiscriminator,
                    )
                });
            target.derivation = InstanceReachabilityDerivationV0::PathUnionNoInstanceDiscriminator;
            let before = target.class_names.len();
            target.class_names.extend(class_names);
            target.class_names = dedupe_names(target.class_names.drain(..));
            changed |= target.class_names.len() != before;
        }
        if !changed {
            break;
        }
    }
    (by_instance, incomplete_composes_target_instances)
}

fn merge_reachability_analysis(
    current: TransformBundleReachabilityAnalysisV0,
    incoming: TransformBundleReachabilityAnalysisV0,
) -> TransformBundleReachabilityAnalysisV0 {
    match (current, incoming) {
        (TransformBundleReachabilityAnalysisV0::Analyzed, next) => next,
        (unavailable @ TransformBundleReachabilityAnalysisV0::Unanalyzed { .. }, _) => unavailable,
    }
}

fn module_metadata_with_reachability_evidence(
    module_metadata: &[ClosedWorldModuleMetadataV0],
    module_reachability_evidence: &BTreeMap<
        ModuleInstanceKeyV0,
        ClosedWorldModuleReachabilityEvidenceV0,
    >,
) -> Vec<ClosedWorldModuleMetadataV0> {
    let mut metadata_by_instance = module_metadata
        .iter()
        .cloned()
        .map(|metadata| (metadata.module_instance().clone(), metadata))
        .collect::<BTreeMap<_, _>>();
    for (module_instance, reachability_evidence) in module_reachability_evidence {
        metadata_by_instance
            .entry(module_instance.clone())
            .and_modify(|metadata| {
                *metadata = metadata
                    .clone()
                    .with_reachability_evidence(*reachability_evidence);
            })
            .or_insert_with(|| {
                ClosedWorldModuleMetadataV0::new(module_instance.clone())
                    .with_reachability_evidence(*reachability_evidence)
            });
    }
    metadata_by_instance.into_values().collect()
}

pub(crate) fn module_instances_by_linker_path(
    inputs: &[LinkerInputV0],
) -> BTreeMap<String, Vec<ModuleInstanceKeyV0>> {
    let mut by_path = BTreeMap::<String, Vec<ModuleInstanceKeyV0>>::new();
    for input in inputs {
        by_path
            .entry(input.source_path.clone())
            .or_default()
            .push(input.instance.clone());
    }
    for instances in by_path.values_mut() {
        instances.sort();
        instances.dedup();
    }
    by_path
}

fn resolve_entrypoint_module_instance_by_path(
    source_path: &str,
    instances_by_path: &BTreeMap<String, Vec<ModuleInstanceKeyV0>>,
) -> Result<Option<ModuleInstanceKeyV0>, TransformBundleLinkErrorV0> {
    let normalized = normalize_bundle_path(PathBuf::from(source_path));
    let Some(instances) = instances_by_path.get(&normalized) else {
        return Ok(None);
    };
    match instances.as_slice() {
        [instance] => Ok(Some(instance.clone())),
        instances => {
            let unconfigured = omena_parser::ConfigurationHashV0::none();
            let mut matches = instances
                .iter()
                .filter(|instance| instance.configuration() == &unconfigured);
            let selected = matches.next().cloned();
            if selected.is_some() && matches.next().is_none() {
                Ok(selected)
            } else {
                Err(TransformBundleLinkErrorV0::AmbiguousModulePath {
                    source_path: normalized,
                })
            }
        }
    }
}

fn collect_closed_world_linked_modules_from_projection(
    inputs: &[LinkerInputV0],
    resolved_dependencies: &[TransformBundleResolvedDependencyV0],
    instances_by_path: &BTreeMap<String, Vec<ModuleInstanceKeyV0>>,
    resolution_authority: BundleResolutionAuthorityV0,
) -> Result<
    (
        Vec<ClosedWorldLinkedModuleV0>,
        Vec<BundleDependencyResolutionDisclosureV0>,
    ),
    TransformBundleLinkErrorV0,
> {
    let linked_with_disclosures = inputs
        .iter()
        .map(|input| {
            let mut linked = ClosedWorldLinkedModuleV0::new(input.instance.clone());
            let mut disclosures = Vec::new();
            for edge in &input.dependency_edges {
                let resolution = resolve_imported_module_instance_for_edge(
                    input,
                    edge,
                    resolved_dependencies,
                    instances_by_path,
                    resolution_authority,
                )?;
                let dependency = resolution.target_instance.clone().ok_or_else(|| {
                    TransformBundleLinkErrorV0::MissingDependency {
                        source_path: input.source_path.clone(),
                        import_source: edge.import_source.clone(),
                    }
                })?;
                disclosures.push(BundleDependencyResolutionDisclosureV0 {
                    source_instance: input.instance.clone(),
                    import_source: edge.import_source.clone(),
                    import_ordinal: edge.import_ordinal,
                    authority: resolution.authority,
                });
                if edge.kind == TransformBundleEdgeKind::CssModuleComposesExternal {
                    for local_name in &edge.local_names {
                        for remote_name in &edge.remote_names {
                            linked = linked.with_composes_edge(ClosedWorldComposesEdgeV0 {
                                from_module: input.instance.clone(),
                                from_symbol: local_name.clone(),
                                to_module: dependency.clone(),
                                to_symbol: remote_name.clone(),
                            });
                        }
                    }
                }
                linked = linked.with_dependency(dependency);
            }
            for name in dedupe_names(input.class_names.iter().cloned()) {
                linked = linked.with_class_name(name);
            }
            for name in dedupe_names(input.keyframe_names.iter().cloned()) {
                linked = linked.with_keyframe_name(name);
            }
            for name in dedupe_names(input.value_names.iter().cloned()) {
                linked = linked.with_value_name(name);
            }
            for name in dedupe_custom_property_names(input.custom_property_names.iter().cloned()) {
                linked = linked.with_custom_property_name(name);
            }
            linked.dependencies.sort();
            linked.dependencies.dedup();
            linked.composes_edges.sort_by(|left, right| {
                (
                    &left.from_module,
                    &left.from_symbol,
                    &left.to_module,
                    &left.to_symbol,
                )
                    .cmp(&(
                        &right.from_module,
                        &right.from_symbol,
                        &right.to_module,
                        &right.to_symbol,
                    ))
            });
            linked.composes_edges.dedup();
            linked.composes_edge_observation_count = linked.composes_edges.len();
            Ok((linked, disclosures))
        })
        .collect::<Result<Vec<_>, TransformBundleLinkErrorV0>>()?;
    let (linked_modules, disclosure_groups): (Vec<_>, Vec<_>) =
        linked_with_disclosures.into_iter().unzip();
    let mut disclosures = disclosure_groups.into_iter().flatten().collect::<Vec<_>>();
    disclosures.sort();
    disclosures.dedup();
    Ok((linked_modules, disclosures))
}

const fn bundle_edge_module_dependency_reason(
    kind: TransformBundleEdgeKind,
) -> Option<&'static str> {
    match kind {
        TransformBundleEdgeKind::SassUse => Some("loads a Sass module instance"),
        TransformBundleEdgeKind::SassForward => Some("forwards a Sass module instance"),
        TransformBundleEdgeKind::SassImport => Some("loads Sass stylesheet rules"),
        TransformBundleEdgeKind::CssImport => Some("loads CSS stylesheet rules"),
        TransformBundleEdgeKind::LessImport => Some("loads Less stylesheet rules"),
        TransformBundleEdgeKind::CssModuleValueImport => Some("loads CSS Modules values"),
        TransformBundleEdgeKind::CssModuleComposesExternal => {
            Some("loads selectors from an external CSS Module")
        }
        TransformBundleEdgeKind::IcssImport => Some("loads ICSS values"),
        TransformBundleEdgeKind::CssModuleComposesLocal => None,
    }
}

/// Returns whether an edge traverses into another stylesheet module.
///
/// Local CSS Modules composition stays outside this set because it names a
/// selector in the current module rather than a separately resolved source.
pub const fn bundle_edge_is_module_dependency(kind: TransformBundleEdgeKind) -> bool {
    bundle_edge_module_dependency_reason(kind).is_some()
}

pub(crate) fn resolve_imported_module_instance(
    source_path: &str,
    import_source: &str,
    instances_by_path: &BTreeMap<String, Vec<ModuleInstanceKeyV0>>,
) -> Result<Option<ModuleInstanceKeyV0>, TransformBundleLinkErrorV0> {
    for candidate in import_path_candidates(source_path, import_source) {
        if let Some(instances) = instances_by_path.get(candidate.as_str()) {
            return match instances.as_slice() {
                [instance] => Ok(Some(instance.clone())),
                _ => Err(TransformBundleLinkErrorV0::AmbiguousModulePath {
                    source_path: candidate,
                }),
            };
        }
    }
    Ok(None)
}

pub(crate) struct DependencyResolutionOutcomeV0 {
    pub(crate) target_instance: Option<ModuleInstanceKeyV0>,
    pub(crate) authority: BundleResolutionAuthorityV0,
}

pub(crate) fn resolve_imported_module_instance_for_edge(
    input: &LinkerInputV0,
    edge: &LinkerDependencyEdgeV0,
    resolved_dependencies: &[TransformBundleResolvedDependencyV0],
    instances_by_path: &BTreeMap<String, Vec<ModuleInstanceKeyV0>>,
    resolution_authority: BundleResolutionAuthorityV0,
) -> Result<DependencyResolutionOutcomeV0, TransformBundleLinkErrorV0> {
    let mut matches = resolved_dependencies.iter().filter(|dependency| {
        dependency.source_instance == input.instance
            && dependency.edge_kind == edge.kind
            && dependency.import_source == edge.import_source
            && dependency.import_ordinal == edge.import_ordinal
    });
    if let Some(resolved) = matches.next() {
        if matches.next().is_some() {
            return Err(TransformBundleLinkErrorV0::InvalidEmissionPlan {
                reason: format!(
                    "dependency {} in {} has more than one resolved-edge record",
                    edge.import_source,
                    input.instance.module().as_str()
                ),
            });
        }
        return Ok(DependencyResolutionOutcomeV0 {
            target_instance: resolved
                .resolution
                .target_instance
                .as_ref()
                .and_then(|target| {
                    instances_by_path
                        .get(target.module().as_str())
                        .filter(|instances| instances.contains(target))
                        .map(|_| target.clone())
                }),
            authority: BundleResolutionAuthorityV0::Resolved,
        });
    }
    if resolution_authority == BundleResolutionAuthorityV0::Resolved {
        return Err(TransformBundleLinkErrorV0::UnresolvedDependencyEdge {
            source_path: input.source_path.clone(),
            import_source: edge.import_source.clone(),
            import_ordinal: edge.import_ordinal,
        });
    }
    Ok(DependencyResolutionOutcomeV0 {
        target_instance: resolve_imported_module_instance(
            input.source_path.as_str(),
            edge.import_source.as_str(),
            instances_by_path,
        )?,
        authority: BundleResolutionAuthorityV0::LegacyPathInferred,
    })
}

fn import_path_candidates(source_path: &str, import_source: &str) -> Vec<String> {
    let base = if import_source.starts_with('/') {
        PathBuf::from(import_source)
    } else {
        Path::new(source_path)
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(import_source)
    };
    let normalized = normalize_bundle_path(base);
    let mut candidates = vec![normalized.clone()];
    if Path::new(&normalized).extension().is_none() {
        for extension in ["css", "scss", "sass", "less"] {
            candidates.push(format!("{normalized}.{extension}"));
        }
        let path = Path::new(&normalized);
        if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
            let mut partial = path.parent().unwrap_or_else(|| Path::new("")).to_path_buf();
            partial.push(format!("_{file_name}"));
            let partial = normalize_bundle_path(partial);
            for extension in ["scss", "sass"] {
                candidates.push(format!("{partial}.{extension}"));
            }
        }
    }
    candidates.sort();
    candidates.dedup();
    candidates
}

pub(crate) fn selector_kind_label(kind: ParsedSelectorFactKind) -> &'static str {
    match kind {
        ParsedSelectorFactKind::Class => "class",
        ParsedSelectorFactKind::Id => "id",
        ParsedSelectorFactKind::Placeholder => "placeholder",
    }
}

fn serialize_selector_fact_kind<S>(
    kind: &ParsedSelectorFactKind,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(selector_kind_label(*kind))
}

fn serialize_style_dialect<S>(dialect: &StyleDialect, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(dialect_label(*dialect))
}

fn dedupe_names(names: impl IntoIterator<Item = String>) -> Vec<String> {
    names
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn dedupe_custom_property_names(names: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut by_identity = BTreeMap::<CanonicalCustomPropertyNameV0, String>::new();
    for authored in names {
        by_identity
            .entry(PropertyNameV0::canonical_custom_key(&authored))
            .or_insert(authored);
    }
    by_identity.into_values().collect()
}

fn collect_bundle_edges_from_facts(
    source_path: &str,
    dialect: StyleDialect,
    facts: &omena_parser::ParsedStyleFacts,
) -> Vec<TransformBundleEdgeV0> {
    let mut edges = Vec::new();

    for edge in &facts.sass_module_edges {
        let kind = match edge.kind {
            ParsedSassModuleEdgeFactKind::Use => TransformBundleEdgeKind::SassUse,
            ParsedSassModuleEdgeFactKind::Forward => TransformBundleEdgeKind::SassForward,
            ParsedSassModuleEdgeFactKind::Import => import_edge_kind_for_dialect(dialect),
        };
        edges.push(TransformBundleEdgeV0 {
            kind,
            source_path: source_path.to_string(),
            import_source: Some(edge.source.clone()),
            import_ordinal: None,
            namespace: edge.namespace.clone(),
            local_names: Vec::new(),
            remote_names: Vec::new(),
            range_start: u32::from(edge.range.start()),
            range_end: u32::from(edge.range.end()),
            provenance_required: true,
        });
    }

    for edge in &facts.css_module_value_import_edges {
        edges.push(TransformBundleEdgeV0 {
            kind: TransformBundleEdgeKind::CssModuleValueImport,
            source_path: source_path.to_string(),
            import_source: Some(edge.import_source.clone()),
            import_ordinal: None,
            namespace: None,
            local_names: vec![edge.local_name.clone()],
            remote_names: vec![edge.remote_name.clone()],
            range_start: u32::from(edge.range.start()),
            range_end: u32::from(edge.range.end()),
            provenance_required: true,
        });
    }

    for edge in &facts.css_module_composes_edges {
        let kind = match edge.kind {
            ParsedCssModuleComposesEdgeKind::External => {
                TransformBundleEdgeKind::CssModuleComposesExternal
            }
            ParsedCssModuleComposesEdgeKind::Local | ParsedCssModuleComposesEdgeKind::Global => {
                TransformBundleEdgeKind::CssModuleComposesLocal
            }
        };
        edges.push(TransformBundleEdgeV0 {
            kind,
            source_path: source_path.to_string(),
            import_source: edge.import_source.clone(),
            import_ordinal: None,
            namespace: None,
            local_names: edge.owner_selector_names.clone(),
            remote_names: edge.target_names.clone(),
            range_start: u32::from(edge.range.start()),
            range_end: u32::from(edge.range.end()),
            provenance_required: true,
        });
    }

    for edge in &facts.icss_import_edges {
        edges.push(TransformBundleEdgeV0 {
            kind: TransformBundleEdgeKind::IcssImport,
            source_path: source_path.to_string(),
            import_source: Some(edge.import_source.clone()),
            import_ordinal: None,
            namespace: None,
            local_names: vec![edge.local_name.clone()],
            remote_names: vec![edge.remote_name.clone()],
            range_start: u32::from(edge.range.start()),
            range_end: u32::from(edge.range.end()),
            provenance_required: true,
        });
    }

    assign_parser_origin_import_ordinals(&mut edges);
    edges
}

fn assign_parser_origin_import_ordinals(edges: &mut [TransformBundleEdgeV0]) {
    let mut order_bearing_indices = edges
        .iter()
        .enumerate()
        .filter(|(_, edge)| {
            edge.import_source.is_some()
                && edge.kind.order_relevance() == EdgeOrderRelevanceV0::OrderBearing
        })
        .map(|(index, edge)| (index, edge.range_start, edge.range_end))
        .collect::<Vec<_>>();
    order_bearing_indices
        .sort_by_key(|(index, range_start, range_end)| (*range_start, *range_end, *index));
    for (ordinal, (index, _, _)) in order_bearing_indices.into_iter().enumerate() {
        edges[index].import_ordinal = u32::try_from(ordinal).ok();
    }
}

fn import_edge_kind_for_dialect(dialect: StyleDialect) -> TransformBundleEdgeKind {
    match dialect {
        StyleDialect::Css => TransformBundleEdgeKind::CssImport,
        StyleDialect::Less => TransformBundleEdgeKind::LessImport,
        StyleDialect::Scss | StyleDialect::Sass => TransformBundleEdgeKind::SassImport,
    }
}

fn collect_transform_ir_bundle_asset_urls(
    source_path: &str,
    source: &str,
    dialect: StyleDialect,
) -> Vec<TransformBundleAssetUrlV0> {
    let ir = lower_transform_ir_from_source(source, dialect, source_path);
    ir.nodes
        .iter()
        .filter(|node| !node.deleted && node.kind == IrNodeKindV0::UrlValue)
        .filter_map(|url_value| {
            let start = url_value.source_span_start;
            let end = url_value.source_span_end;
            if start >= end
                || end > source.len()
                || !source.is_char_boundary(start)
                || !source.is_char_boundary(end)
            {
                return None;
            }
            let (raw_url, normalized_url, parsed_end) = parse_bundle_url_function(source, start)?;
            if parsed_end != end {
                return None;
            }
            let (kind, resolved_path) = classify_bundle_asset_url(source_path, &normalized_url);
            Some(TransformBundleAssetUrlV0 {
                source_path: source_path.to_string(),
                raw_url,
                normalized_url,
                kind,
                resolved_path,
                range_start: start as u32,
                range_end: parsed_end as u32,
                bundler_resolution_required: matches!(
                    kind,
                    TransformBundleAssetUrlKind::Relative
                        | TransformBundleAssetUrlKind::AbsolutePath
                ),
            })
        })
        .collect()
}

#[cfg(test)]
fn raw_scan_bundle_asset_urls_for_oracle(
    source_path: &str,
    source: &str,
) -> Vec<TransformBundleAssetUrlV0> {
    let bytes = source.as_bytes();
    let mut urls = Vec::new();
    let mut index = 0usize;

    while index + 4 <= bytes.len() {
        if !bytes[index].eq_ignore_ascii_case(&b'u')
            || !bytes[index + 1].eq_ignore_ascii_case(&b'r')
            || !bytes[index + 2].eq_ignore_ascii_case(&b'l')
            || bytes[index + 3] != b'('
        {
            index += 1;
            continue;
        }
        let Some((raw_url, normalized_url, end)) = parse_bundle_url_function(source, index) else {
            index += 4;
            continue;
        };
        let (kind, resolved_path) = classify_bundle_asset_url(source_path, &normalized_url);
        urls.push(TransformBundleAssetUrlV0 {
            source_path: source_path.to_string(),
            raw_url,
            normalized_url,
            kind,
            resolved_path,
            range_start: index as u32,
            range_end: end as u32,
            bundler_resolution_required: matches!(
                kind,
                TransformBundleAssetUrlKind::Relative | TransformBundleAssetUrlKind::AbsolutePath
            ),
        });
        index = end;
    }

    urls
}

fn dialect_for_bundle_source_path(source_path: &str) -> StyleDialect {
    let extension = Path::new(source_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "scss" => StyleDialect::Scss,
        "sass" => StyleDialect::Sass,
        "less" => StyleDialect::Less,
        _ => StyleDialect::Css,
    }
}

fn parse_bundle_url_function(source: &str, start: usize) -> Option<(String, String, usize)> {
    let open_end = start.checked_add(4)?;
    let mut index = open_end;
    let mut quote = None;
    let mut escaped = false;

    while index < source.len() {
        let ch = source[index..].chars().next()?;
        let next = index + ch.len_utf8();
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            index = next;
            continue;
        }

        match ch {
            '"' | '\'' => quote = Some(ch),
            ')' => {
                let raw_url = source[start..next].to_string();
                let inner = source[open_end..index].trim();
                let normalized_url = unquote_bundle_url_inner(inner)?;
                return Some((raw_url, normalized_url, next));
            }
            _ => {}
        }
        index = next;
    }

    None
}

fn unquote_bundle_url_inner(inner: &str) -> Option<String> {
    if inner.is_empty() {
        return None;
    }
    let bytes = inner.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        return Some(inner[1..inner.len() - 1].to_string());
    }
    Some(inner.to_string())
}

fn classify_bundle_asset_url(
    source_path: &str,
    normalized_url: &str,
) -> (TransformBundleAssetUrlKind, Option<String>) {
    let lower = normalized_url.to_ascii_lowercase();
    if lower.starts_with("data:") {
        return (TransformBundleAssetUrlKind::Data, None);
    }
    if normalized_url.starts_with('#') {
        return (TransformBundleAssetUrlKind::Fragment, None);
    }
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || normalized_url.starts_with("//")
    {
        return (TransformBundleAssetUrlKind::External, None);
    }
    if normalized_url.starts_with('/') {
        return (
            TransformBundleAssetUrlKind::AbsolutePath,
            Some(normalized_url.to_string()),
        );
    }

    (
        TransformBundleAssetUrlKind::Relative,
        Some(resolve_relative_bundle_asset_path(
            source_path,
            normalized_url,
        )),
    )
}

fn resolve_relative_bundle_asset_path(source_path: &str, normalized_url: &str) -> String {
    let base = Path::new(source_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    normalize_bundle_path(base.join(normalized_url))
}

pub fn normalize_omena_transform_bundle_path(path: &str) -> String {
    normalize_bundle_path(PathBuf::from(path.replace('\\', "/"))).replace('\\', "/")
}

fn normalize_bundle_path(path: PathBuf) -> String {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => match normalized.components().next_back() {
                Some(Component::Normal(_)) => {
                    normalized.pop();
                }
                Some(Component::RootDir) => {}
                _ => normalized.push(".."),
            },
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized.to_string_lossy().into_owned()
}

fn plan_bundle_code_split_chunks(
    source_path: &str,
    bundle_edges: &[TransformBundleEdgeV0],
    asset_urls: &[TransformBundleAssetUrlV0],
) -> Vec<TransformBundleChunkV0> {
    let mut chunks: Vec<TransformBundleChunkV0> = Vec::new();
    let mut entry_dependencies = Vec::new();

    for edge in bundle_edges {
        let Some(import_source) = edge.import_source.as_ref() else {
            continue;
        };
        let chunk_id = bundle_chunk_id("style", source_path, import_source);
        if !entry_dependencies.contains(&chunk_id) {
            entry_dependencies.push(chunk_id.clone());
        }
        if chunks.iter().any(|chunk| chunk.chunk_id == chunk_id) {
            continue;
        }
        chunks.push(TransformBundleChunkV0 {
            chunk_id,
            kind: TransformBundleChunkKind::StyleImport,
            source_path: source_path.to_string(),
            import_source: Some(import_source.clone()),
            asset_url: None,
            resolved_path: None,
            depends_on: Vec::new(),
            split_boundary: "styleDependency",
        });
    }

    for asset in asset_urls {
        if !asset.bundler_resolution_required {
            continue;
        }
        let chunk_id = bundle_chunk_id("asset", source_path, asset.normalized_url.as_str());
        if !entry_dependencies.contains(&chunk_id) {
            entry_dependencies.push(chunk_id.clone());
        }
        if chunks.iter().any(|chunk| chunk.chunk_id == chunk_id) {
            continue;
        }
        chunks.push(TransformBundleChunkV0 {
            chunk_id,
            kind: TransformBundleChunkKind::Asset,
            source_path: source_path.to_string(),
            import_source: None,
            asset_url: Some(asset.normalized_url.clone()),
            resolved_path: asset.resolved_path.clone(),
            depends_on: Vec::new(),
            split_boundary: "assetDependency",
        });
    }

    entry_dependencies.sort();
    chunks.sort_by(|left, right| left.chunk_id.cmp(&right.chunk_id));
    let mut ordered = vec![TransformBundleChunkV0 {
        chunk_id: bundle_chunk_id("entry", source_path, source_path),
        kind: TransformBundleChunkKind::Entry,
        source_path: source_path.to_string(),
        import_source: None,
        asset_url: None,
        resolved_path: Some(source_path.to_string()),
        depends_on: entry_dependencies,
        split_boundary: "entry",
    }];
    ordered.extend(chunks);
    ordered
}

fn bundle_chunk_id(kind: &str, source_path: &str, target: &str) -> String {
    format!(
        "{kind}:{}:{}",
        sanitize_bundle_chunk_id_part(source_path),
        sanitize_bundle_chunk_id_part(target)
    )
}

fn sanitize_bundle_chunk_id_part(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            sanitized.push(ch);
        } else {
            sanitized.push('-');
        }
    }
    sanitized.trim_matches('-').to_string()
}

fn required_passes_for_source(
    source_path: &str,
    dialect: StyleDialect,
    facts: &omena_parser::ParsedStyleFacts,
    bundle_edges: &[TransformBundleEdgeV0],
) -> Vec<TransformPassKind> {
    let mut passes = Vec::new();

    if bundle_edges.iter().any(|edge| {
        matches!(
            edge.kind,
            TransformBundleEdgeKind::SassImport
                | TransformBundleEdgeKind::CssImport
                | TransformBundleEdgeKind::LessImport
                | TransformBundleEdgeKind::CssModuleValueImport
                | TransformBundleEdgeKind::CssModuleComposesExternal
                | TransformBundleEdgeKind::IcssImport
        )
    }) {
        passes.push(TransformPassKind::ImportInline);
    }

    if matches!(dialect, StyleDialect::Scss | StyleDialect::Sass) {
        passes.push(TransformPassKind::ScssModuleEvaluate);
    }

    if matches!(dialect, StyleDialect::Less) {
        passes.push(TransformPassKind::LessModuleEvaluate);
    }

    if is_css_module_path(source_path) && facts.selector_count > 0 {
        passes.push(TransformPassKind::HashCssModuleClassNames);
    }

    if facts.css_module_composes_edge_count > 0 {
        passes.push(TransformPassKind::ResolveCssModulesComposes);
    }

    if facts.css_module_value_count > 0 || facts.css_module_value_import_edge_count > 0 {
        passes.push(TransformPassKind::ValueResolution);
    }

    passes
}

fn is_css_module_path(source_path: &str) -> bool {
    let file_name = source_path
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(source_path)
        .to_ascii_lowercase();
    let Some((stem, extension)) = file_name.rsplit_once('.') else {
        return false;
    };
    matches!(extension, "css" | "scss" | "sass" | "less") && stem.ends_with(".module")
}

fn dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{
        InstanceReachabilityDerivationV0, LinkedStylesheetRuleV0, LinkerDependencyEdgeV0,
        LinkerInputV0, LinkerRuleV0, TRANSFORM_BUNDLE_EDGE_KIND_VARIANTS_V0,
        TransformBundleAssetUrlKind, TransformBundleChunkKind,
        TransformBundleDependencyResolutionV0, TransformBundleEdgeKind,
        TransformBundleInstanceReachabilityInputV0, TransformBundleLinkErrorV0,
        TransformBundleLinkOptionsV0, TransformBundleModuleInputV0,
        TransformBundleReachabilityAnalysisV0, TransformBundleReachabilityUnanalyzedCauseV0,
        TransformBundleResolvedDependencyV0, TransformBundleSemanticReachabilityInputV0,
        TransformBundleTransformedModuleV0, apply_semantic_reachability_to_linker_inputs,
        bundle_edge_is_module_dependency, bundle_edge_module_dependency_reason,
        carrier_hygiene_assertions, collect_transform_ir_bundle_asset_urls,
        compare_omena_transform_bundle_emission_policies, link_omena_transform_bundle_modules,
        link_omena_transform_bundle_modules_with_options,
        link_omena_transform_bundle_modules_with_semantic_reachability,
        link_omena_transform_bundle_projection_with_resolved_dependencies_and_options,
        link_stylesheet_from_projection, materialize_omena_transform_bundle_linked_stylesheet,
        normalize_omena_transform_bundle_path, project_omena_transform_bundle_linker_inputs,
        raw_scan_bundle_asset_urls_for_oracle, rewrite_omena_transform_bundle_asset_urls_in_source,
        summarize_omena_transform_bundle_from_source,
    };
    use omena_cross_file_summary::EdgeOrderRelevanceV0;
    use omena_parser::{
        ClosedWorldModuleReachabilityEvidenceV0, ConfigurationHashV0, ModuleIdV0,
        ModuleInstanceKeyV0, ParsedSelectorFactKind, StyleDialect,
    };

    #[test]
    fn public_path_normalizer_collapses_equivalent_cross_platform_spellings() {
        assert_eq!(
            normalize_omena_transform_bundle_path("/workspace/src/./nested/../Button.module.css"),
            "/workspace/src/Button.module.css"
        );
        assert_eq!(
            normalize_omena_transform_bundle_path(
                r"C:\workspace\src\.\nested\..\Button.module.css"
            ),
            "C:/workspace/src/Button.module.css"
        );
    }

    #[test]
    fn builds_bundle_plan_from_scss_and_css_modules_parser_facts() {
        let source = r#"
@use "./tokens" as tokens;
@forward "./theme";
@value primary from "./colors.module.css";
.button {
  composes: reset from "./reset.module.css";
  color: tokens.$brand;
}
"#;
        let summary = summarize_omena_transform_bundle_from_source(
            "Button.module.scss",
            source,
            StyleDialect::Scss,
        );

        assert_eq!(summary.product, "omena-transform-bundle.source");
        assert_eq!(summary.dialect, "scss");
        assert!(summary.import_inline_required);
        assert!(summary.module_evaluation_required);
        assert!(summary.css_modules_resolution_required);
        assert!(summary.class_hashing_required);
        assert!(summary.value_resolution_required);
        assert!(summary.pass_plan.violated_dag_edge_count == 0);
        assert!(summary.bundle_edges.iter().any(|edge| {
            edge.kind == TransformBundleEdgeKind::CssModuleComposesExternal
                && edge.import_source.as_deref() == Some("./reset.module.css")
        }));
        assert_eq!(
            summary.planned_pass_ids,
            vec![
                "import-inline",
                "scss-module-evaluate",
                "composes-resolution",
                "css-modules-class-hashing",
                "value-resolution"
            ]
        );
    }

    #[test]
    fn bundle_edge_catalog_has_total_order_relevance() {
        assert_eq!(TRANSFORM_BUNDLE_EDGE_KIND_VARIANTS_V0.len(), 9);
        assert!(TRANSFORM_BUNDLE_EDGE_KIND_VARIANTS_V0.iter().all(|kind| {
            kind.order_relevance() == EdgeOrderRelevanceV0::OrderBearing
                && !kind.order_relevance_reason().is_empty()
        }));
    }

    #[test]
    fn module_dependency_edge_authority_excludes_only_local_composition() {
        let module_dependencies = TRANSFORM_BUNDLE_EDGE_KIND_VARIANTS_V0
            .iter()
            .copied()
            .filter(|kind| bundle_edge_is_module_dependency(*kind))
            .collect::<Vec<_>>();

        assert_eq!(module_dependencies.len(), 8);
        assert!(!module_dependencies.contains(&TransformBundleEdgeKind::CssModuleComposesLocal));
        assert!(TRANSFORM_BUNDLE_EDGE_KIND_VARIANTS_V0.iter().all(|kind| {
            bundle_edge_is_module_dependency(*kind)
                == bundle_edge_module_dependency_reason(*kind).is_some()
        }));
    }

    #[test]
    fn recognizes_less_module_evaluation_from_dialect() {
        let summary = summarize_omena_transform_bundle_from_source(
            "Theme.module.less",
            r#"@import (reference) "tokens.less"; .card { color: @brand; }"#,
            StyleDialect::Less,
        );

        assert!(summary.module_evaluation_required);
        assert!(summary.import_inline_required);
        assert!(
            summary
                .bundle_edges
                .iter()
                .any(|edge| edge.kind == TransformBundleEdgeKind::LessImport)
        );
        assert!(summary.required_pass_ids.contains(&"less-module-evaluate"));
        assert!(!summary.required_pass_ids.contains(&"scss-module-evaluate"));
        assert!(
            summary
                .required_pass_ids
                .contains(&"css-modules-class-hashing")
        );
    }

    #[test]
    fn plans_plain_css_import_inline_without_scss_module_evaluation() {
        let summary = summarize_omena_transform_bundle_from_source(
            "App.css",
            r#"@import "./tokens.css"; .button { color: red; }"#,
            StyleDialect::Css,
        );

        assert!(summary.import_inline_required);
        assert!(!summary.module_evaluation_required);
        assert_eq!(summary.required_pass_ids, vec!["import-inline"]);
        assert_eq!(summary.planned_pass_ids, vec!["import-inline"]);
        assert!(
            summary
                .bundle_edges
                .iter()
                .any(|edge| edge.kind == TransformBundleEdgeKind::CssImport)
        );
        assert!(
            !summary
                .bundle_edges
                .iter()
                .any(|edge| edge.kind == TransformBundleEdgeKind::SassImport)
        );
    }

    #[test]
    fn rejects_module_substring_false_positive_paths() {
        let source = ".button { color: red; }";
        let backup_summary = summarize_omena_transform_bundle_from_source(
            "Button.module.backup.scss",
            source,
            StyleDialect::Scss,
        );
        let unrelated_summary = summarize_omena_transform_bundle_from_source(
            "module/Button.scss",
            source,
            StyleDialect::Scss,
        );

        assert!(!backup_summary.class_hashing_required);
        assert!(!unrelated_summary.class_hashing_required);
        assert!(
            !backup_summary
                .required_pass_ids
                .contains(&"css-modules-class-hashing")
        );
        assert!(
            !unrelated_summary
                .required_pass_ids
                .contains(&"css-modules-class-hashing")
        );
    }

    #[test]
    fn recognizes_css_module_path_by_final_stem_and_supported_extension() {
        let summary = summarize_omena_transform_bundle_from_source(
            "components\\Button.MODULE.SCSS",
            ".button { color: red; }",
            StyleDialect::Scss,
        );

        assert!(summary.class_hashing_required);
        assert!(
            summary
                .required_pass_ids
                .contains(&"css-modules-class-hashing")
        );
    }

    #[test]
    fn resolves_relative_asset_urls_from_source_path() {
        let summary = summarize_omena_transform_bundle_from_source(
            "src/components/Button.module.css",
            r#".button { background: url("../assets/icon.svg"); mask: url(/static/mask.svg); cursor: url(data:image/svg+xml,abc); filter: url(#shadow); border-image-source: URL(https://cdn.example.com/frame.png); }"#,
            StyleDialect::Css,
        );

        assert_eq!(summary.asset_urls.len(), 5);
        assert!(summary.asset_urls.iter().any(|asset| {
            asset.normalized_url == "../assets/icon.svg"
                && asset.kind == TransformBundleAssetUrlKind::Relative
                && asset.resolved_path.as_deref() == Some("src/assets/icon.svg")
                && asset.bundler_resolution_required
        }));
        assert!(summary.asset_urls.iter().any(|asset| {
            asset.normalized_url == "/static/mask.svg"
                && asset.kind == TransformBundleAssetUrlKind::AbsolutePath
                && asset.resolved_path.as_deref() == Some("/static/mask.svg")
                && asset.bundler_resolution_required
        }));

        assert!(summary.asset_urls.iter().any(|asset| {
            asset.kind == TransformBundleAssetUrlKind::Data && !asset.bundler_resolution_required
        }));
        assert!(summary.asset_urls.iter().any(|asset| {
            asset.kind == TransformBundleAssetUrlKind::Fragment
                && !asset.bundler_resolution_required
        }));
        assert!(summary.asset_urls.iter().any(|asset| {
            asset.kind == TransformBundleAssetUrlKind::External
                && !asset.bundler_resolution_required
        }));
    }

    #[test]
    fn value_ir_asset_urls_match_raw_scan_byte_identical() {
        let corpus = [
            (
                "src/components/Button.module.css",
                StyleDialect::Css,
                r#".button { background: url("../assets/icon.svg"); mask: url(/static/mask.svg); cursor: url(data:image/svg+xml,abc); filter: url(#shadow); border-image-source: URL(https://cdn.example.com/frame.png); }"#,
            ),
            (
                "src/components/Card.module.scss",
                StyleDialect::Scss,
                r#".카드 { background-image: url(./img/아이콘.svg); }"#,
            ),
            (
                "src/components/Theme.module.less",
                StyleDialect::Less,
                r#".theme { background: url('../assets/theme.svg'); }"#,
            ),
        ];

        for (source_path, dialect, source) in corpus {
            let transform_ir_urls =
                collect_transform_ir_bundle_asset_urls(source_path, source, dialect);
            let raw_urls = raw_scan_bundle_asset_urls_for_oracle(source_path, source);
            assert_eq!(transform_ir_urls, raw_urls, "{source_path}");
        }
    }

    #[test]
    fn plans_code_split_chunks_for_style_and_asset_dependencies() {
        let summary = summarize_omena_transform_bundle_from_source(
            "src/components/Button.module.css",
            r#"@import "../theme.css"; .button { background: url("../assets/icon.svg"); }"#,
            StyleDialect::Css,
        );

        assert!(summary.code_splitting_required);
        assert_eq!(summary.code_split_chunks.len(), 3);
        let entry_chunk_id = summary
            .code_split_chunks
            .iter()
            .find(|chunk| chunk.kind == TransformBundleChunkKind::Entry)
            .map(|chunk| {
                assert_eq!(chunk.split_boundary, "entry");
                assert_eq!(chunk.depends_on.len(), 2);
                chunk.chunk_id.clone()
            });
        assert!(entry_chunk_id.is_some());

        let style_chunk_id = summary
            .code_split_chunks
            .iter()
            .find(|chunk| chunk.kind == TransformBundleChunkKind::StyleImport)
            .map(|chunk| {
                assert_eq!(chunk.import_source.as_deref(), Some("../theme.css"));
                assert_eq!(chunk.split_boundary, "styleDependency");
                chunk.chunk_id.clone()
            });
        assert!(style_chunk_id.is_some());

        let asset_chunk_id = summary
            .code_split_chunks
            .iter()
            .find(|chunk| chunk.kind == TransformBundleChunkKind::Asset)
            .map(|chunk| {
                assert_eq!(chunk.asset_url.as_deref(), Some("../assets/icon.svg"));
                assert_eq!(chunk.resolved_path.as_deref(), Some("src/assets/icon.svg"));
                assert_eq!(chunk.split_boundary, "assetDependency");
                chunk.chunk_id.clone()
            });
        assert!(asset_chunk_id.is_some());
        let entry_dependencies = summary
            .code_split_chunks
            .iter()
            .find(|chunk| chunk.kind == TransformBundleChunkKind::Entry)
            .map(|chunk| chunk.depends_on.as_slice())
            .unwrap_or(&[]);
        assert!(style_chunk_id.is_some_and(|chunk_id| entry_dependencies.contains(&chunk_id)));
        assert!(asset_chunk_id.is_some_and(|chunk_id| entry_dependencies.contains(&chunk_id)));
    }

    #[test]
    fn resolves_asset_urls_after_non_ascii_source_text() {
        let summary = summarize_omena_transform_bundle_from_source(
            "src/카드.module.css",
            ".카드 { background-image: url(./img/아이콘.svg); }",
            StyleDialect::Css,
        );

        assert_eq!(summary.asset_urls.len(), 1);
        let asset = &summary.asset_urls[0];
        assert_eq!(asset.kind, TransformBundleAssetUrlKind::Relative);
        assert_eq!(asset.normalized_url, "./img/아이콘.svg");
        assert_eq!(asset.resolved_path.as_deref(), Some("src/img/아이콘.svg"));
    }

    #[test]
    fn preserves_leading_parent_segments_without_source_parent() {
        let summary = summarize_omena_transform_bundle_from_source(
            "Button.module.css",
            ".button { background-image: url(../assets/icon.svg); }",
            StyleDialect::Css,
        );

        assert_eq!(
            summary.asset_urls[0].resolved_path.as_deref(),
            Some("../assets/icon.svg")
        );
    }

    #[test]
    fn rewrites_relative_asset_urls_to_resolved_bundle_paths() {
        let summary = rewrite_omena_transform_bundle_asset_urls_in_source(
            "src/components/Button.module.css",
            r#".button { background: url("../assets/icon.svg"); mask: url(/static/mask.svg); filter: url(#shadow); }"#,
        );

        assert_eq!(summary.product, "omena-transform-bundle.asset-url-rewrite");
        assert_eq!(summary.asset_url_count, 3);
        assert_eq!(summary.rewrite_count, 1);
        assert!(summary.output_css.contains(r#"url("src/assets/icon.svg")"#));
        assert!(summary.output_css.contains("url(/static/mask.svg)"));
        assert!(summary.output_css.contains("url(#shadow)"));
        assert_eq!(
            summary
                .rewritten_asset_urls
                .first()
                .and_then(|asset| asset.resolved_path.as_deref()),
            Some("src/assets/icon.svg")
        );
    }

    #[test]
    fn linker_global_rule_order_is_a_total_order_over_linked_rules() -> Result<(), String> {
        let modules = vec![
            TransformBundleModuleInputV0::new(
                "src/app.module.css",
                r#"@import "./theme.css"; .button { color: var(--brand); }"#,
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "src/theme.css",
                r#":root { --brand: red; } .theme { color: red; }"#,
                StyleDialect::Css,
            ),
        ];

        let linked = link_omena_transform_bundle_modules(&["src/app.module.css"], &modules)
            .map_err(|err| format!("{err:?}"))?;

        assert_eq!(linked.product, "omena-transform-bundle.linked-stylesheet");
        assert_eq!(linked.entrypoints.len(), 1);
        assert_eq!(linked.module_instances.len(), 2);
        assert_eq!(
            linked
                .global_rule_order
                .rules
                .iter()
                .map(|rule| rule.global_order_index)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert!(
            linked
                .global_rule_order
                .rules
                .iter()
                .any(|rule| rule.selector_name == "button")
        );
        assert!(
            linked
                .closed_world_bundle
                .reachability()
                .class_names()
                .contains(&"theme".to_string())
        );
        assert!(
            linked
                .closed_world_bundle
                .reachability()
                .custom_property_names()
                .contains(&"--brand".to_string())
        );
        Ok(())
    }

    #[test]
    fn emission_plan_is_the_only_rule_order_authority() -> Result<(), String> {
        let first = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("a.css"));
        let second = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("b.css"));
        let inputs = [
            LinkerInputV0 {
                source_path: "a.css".to_string(),
                dialect: StyleDialect::Css,
                instance: first.clone(),
                dependency_edges: Vec::new(),
                class_names: vec!["first".to_string()],
                keyframe_names: Vec::new(),
                value_names: Vec::new(),
                custom_property_names: Vec::new(),
                ordered_rules: vec![LinkerRuleV0 {
                    selector_name: "first".to_string(),
                    selector_kind: ParsedSelectorFactKind::Class,
                    range_start: 0,
                    range_end: 6,
                }],
            },
            LinkerInputV0 {
                source_path: "b.css".to_string(),
                dialect: StyleDialect::Css,
                instance: second.clone(),
                dependency_edges: Vec::new(),
                class_names: vec!["second".to_string()],
                keyframe_names: Vec::new(),
                value_names: Vec::new(),
                custom_property_names: Vec::new(),
                ordered_rules: vec![LinkerRuleV0 {
                    selector_name: "second".to_string(),
                    selector_kind: ParsedSelectorFactKind::Class,
                    range_start: 0,
                    range_end: 7,
                }],
            },
        ];
        let mut plan = super::emission_order::build_emission_plan(
            &inputs,
            &[first.clone(), second],
            &[first],
            &[],
            super::EmissionOrderingPolicyV0::ModuleIdLegacy,
            super::BundleResolutionAuthorityV0::LegacyPathInferred,
        )
        .map_err(|error| format!("{error:?}"))?;
        let original = super::emission_order::build_global_rule_order_from_plan(&inputs, &plan)
            .map_err(|error| format!("{error:?}"))?;

        plan.entries.swap(0, 1);
        let perturbed = super::emission_order::build_global_rule_order_from_plan(&inputs, &plan)
            .map_err(|error| format!("{error:?}"))?;

        assert_eq!(original.rules[0].selector_name, "first");
        assert_eq!(perturbed.rules[0].selector_name, "second");
        assert_ne!(original, perturbed);
        Ok(())
    }

    #[test]
    fn parser_import_order_is_recorded_without_changing_default_output() -> Result<(), String> {
        fn link(imports: &str) -> Result<super::LinkedStylesheetV0, String> {
            link_omena_transform_bundle_modules(
                &["src/app.css"],
                &[
                    TransformBundleModuleInputV0::new(
                        "src/app.css",
                        format!("{imports} .app {{ color: red; }}"),
                        StyleDialect::Css,
                    ),
                    TransformBundleModuleInputV0::new(
                        "src/a.css",
                        ".a { color: blue; }",
                        StyleDialect::Css,
                    ),
                    TransformBundleModuleInputV0::new(
                        "src/z.css",
                        ".z { color: green; }",
                        StyleDialect::Css,
                    ),
                ],
            )
            .map_err(|error| format!("{error:?}"))
        }

        let a_then_z = link(r#"@import "./a.css"; @import "./z.css";"#)?;
        let z_then_a = link(r#"@import "./z.css"; @import "./a.css";"#)?;
        let targets = |linked: &super::LinkedStylesheetV0| {
            linked
                .emission_plan
                .dependency_facts
                .iter()
                .map(|fact| fact.to_module.module().as_str().to_string())
                .collect::<Vec<_>>()
        };

        assert_eq!(targets(&a_then_z), vec!["src/a.css", "src/z.css"]);
        assert_eq!(targets(&z_then_a), vec!["src/z.css", "src/a.css"]);
        assert_eq!(
            serde_json::to_vec(&a_then_z).map_err(|error| format!("{error:?}"))?,
            serde_json::to_vec(&z_then_a).map_err(|error| format!("{error:?}"))?
        );
        Ok(())
    }

    #[test]
    fn default_emission_policy_is_pinned_to_legacy_module_id_order() -> Result<(), String> {
        let modules = [
            TransformBundleModuleInputV0::new(
                "src/app.css",
                r#"@import "./z.css"; .app { color: red; }"#,
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "src/z.css",
                ".z { color: green; }",
                StyleDialect::Css,
            ),
        ];
        let implicit = link_omena_transform_bundle_modules(&["src/app.css"], &modules)
            .map_err(|error| format!("{error:?}"))?;
        let explicit = link_omena_transform_bundle_modules_with_options(
            &["src/app.css"],
            &modules,
            &[],
            &[],
            TransformBundleLinkOptionsV0 {
                emission_ordering_policy: super::EmissionOrderingPolicyV0::ModuleIdLegacy,
                ..TransformBundleLinkOptionsV0::default()
            },
        )
        .map_err(|error| format!("{error:?}"))?;

        assert_eq!(
            implicit.emission_plan.policy,
            super::EmissionOrderingPolicyV0::ModuleIdLegacy
        );
        assert_eq!(
            serde_json::to_vec(&implicit).map_err(|error| format!("{error:?}"))?,
            serde_json::to_vec(&explicit).map_err(|error| format!("{error:?}"))?
        );
        Ok(())
    }

    #[test]
    fn import_order_policy_reports_real_output_differences() -> Result<(), String> {
        let modules = [
            TransformBundleModuleInputV0::new(
                "src/app.css",
                r#"@import "./z.css"; @import "./a.css"; .app { color: red; }"#,
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "src/a.css",
                ".a { color: blue; }",
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "src/z.css",
                ".z { color: green; }",
                StyleDialect::Css,
            ),
        ];
        let linked = link_omena_transform_bundle_modules_with_options(
            &["src/app.css"],
            &modules,
            &[],
            &[],
            TransformBundleLinkOptionsV0 {
                emission_ordering_policy: super::EmissionOrderingPolicyV0::ImportOrderPreserving,
                ..TransformBundleLinkOptionsV0::default()
            },
        )
        .map_err(|error| format!("{error:?}"))?;
        let report = compare_omena_transform_bundle_emission_policies(&["src/app.css"], &modules)
            .map_err(|error| format!("{error:?}"))?;

        assert_eq!(
            linked
                .global_rule_order
                .rules
                .iter()
                .map(|rule| rule.selector_name.as_str())
                .collect::<Vec<_>>(),
            vec!["z", "a", "app"]
        );
        assert!(!report.equivalent);
        assert_eq!(report.difference_count, report.differences.len());
        assert!(report.difference_count >= 2);
        Ok(())
    }

    #[test]
    fn linked_emission_materializes_the_global_module_order() -> Result<(), String> {
        let modules = [
            TransformBundleModuleInputV0::new(
                "src/app.css",
                r#"@import "./z.css"; @import "./a.css"; .app { color: red; }"#,
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "src/a.css",
                ".a { color: blue; }",
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "src/z.css",
                ".z { color: green; }",
                StyleDialect::Css,
            ),
        ];
        let link = |policy| {
            link_omena_transform_bundle_modules_with_options(
                &["src/app.css"],
                &modules,
                &[],
                &[],
                TransformBundleLinkOptionsV0 {
                    emission_ordering_policy: policy,
                    ..TransformBundleLinkOptionsV0::default()
                },
            )
            .map_err(|error| format!("{error:?}"))
        };
        let legacy = link(super::EmissionOrderingPolicyV0::ModuleIdLegacy)?;
        let import_order = link(super::EmissionOrderingPolicyV0::ImportOrderPreserving)?;
        let transformed_modules = legacy
            .module_instances
            .iter()
            .cloned()
            .map(|module_instance| {
                let marker = module_instance.module().as_str().replace(['/', '.'], "-");
                TransformBundleTransformedModuleV0::new(
                    module_instance,
                    format!(".{marker} {{ order: linked; }}"),
                )
            })
            .collect::<Vec<_>>();

        let legacy_output =
            materialize_omena_transform_bundle_linked_stylesheet(&legacy, &transformed_modules)
                .map_err(|error| format!("{error:?}"))?;
        let import_order_output = materialize_omena_transform_bundle_linked_stylesheet(
            &import_order,
            &transformed_modules,
        )
        .map_err(|error| format!("{error:?}"))?;

        assert_ne!(legacy_output.output_css, import_order_output.output_css);
        assert_eq!(
            import_order_output
                .module_regions
                .iter()
                .map(|region| region.module_instance.module().as_str())
                .collect::<Vec<_>>(),
            vec!["src/z.css", "src/a.css", "src/app.css"]
        );
        assert_eq!(import_order_output.emitted_module_count, 3);
        assert_eq!(
            import_order_output.global_order_entry_count,
            import_order.global_rule_order.rules.len()
        );
        for transformed in &transformed_modules {
            assert_eq!(
                import_order_output
                    .output_css
                    .matches(&transformed.output_css)
                    .count(),
                1,
                "each transformed module must be emitted exactly once"
            );
        }
        for entry_region in &import_order_output.order_entry_regions {
            let module_region = import_order_output
                .module_regions
                .iter()
                .find(|region| region.module_instance == entry_region.module_instance)
                .ok_or_else(|| "ordered entry has no generated module region".to_string())?;
            assert_eq!(entry_region.generated_start, module_region.generated_start);
            assert_eq!(entry_region.generated_end, module_region.generated_end);
        }
        Ok(())
    }

    #[test]
    fn linked_emission_rejects_preinlined_module_bytes() -> Result<(), String> {
        let modules = [
            TransformBundleModuleInputV0::new(
                "src/app.css",
                r#"@import "./theme.css"; .app { color: red; }"#,
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "src/theme.css",
                ".theme { color: blue; }",
                StyleDialect::Css,
            ),
        ];
        let linked = link_omena_transform_bundle_modules(&["src/app.css"], &modules)
            .map_err(|error| format!("{error:?}"))?;
        let transformed_modules = linked
            .module_instances
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, module_instance)| {
                TransformBundleTransformedModuleV0::new(
                    module_instance,
                    format!(".module-{index} {{ order: linked; }}"),
                )
                .with_non_empty_import_replacement_count(usize::from(index == 0))
            })
            .collect::<Vec<_>>();

        let result =
            materialize_omena_transform_bundle_linked_stylesheet(&linked, &transformed_modules);

        assert!(matches!(
            result,
            Err(
                super::LinkedEmissionMaterializationErrorV0::ImportReplacementWouldDuplicateModule {
                    replacement_count: 1,
                    ..
                }
            )
        ));
        Ok(())
    }

    #[test]
    fn external_composition_cycles_are_recorded_with_an_explicit_policy() -> Result<(), String> {
        let first = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("a.css"));
        let second = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("b.css"));
        let input = |source_path: &str,
                     instance: ModuleInstanceKeyV0,
                     import_source: &str,
                     selector: &str| LinkerInputV0 {
            source_path: source_path.to_string(),
            dialect: StyleDialect::Css,
            instance,
            dependency_edges: vec![LinkerDependencyEdgeV0 {
                kind: TransformBundleEdgeKind::CssModuleComposesExternal,
                import_source: import_source.to_string(),
                import_ordinal: Some(0),
                local_names: Vec::new(),
                remote_names: Vec::new(),
            }],
            class_names: vec![selector.to_string()],
            keyframe_names: Vec::new(),
            value_names: Vec::new(),
            custom_property_names: Vec::new(),
            ordered_rules: vec![LinkerRuleV0 {
                selector_name: selector.to_string(),
                selector_kind: ParsedSelectorFactKind::Class,
                range_start: 0,
                range_end: selector.len() as u32,
            }],
        };
        let linked = link_stylesheet_from_projection(
            &["a.css"],
            &[
                input("a.css", first, "./b.css", "a"),
                input("b.css", second, "./a.css", "b"),
            ],
        )
        .map_err(|error| format!("{error:?}"))?;

        assert_eq!(linked.emission_plan.cycle_groups.len(), 1);
        let group = &linked.emission_plan.cycle_groups[0];
        assert_eq!(group.class, super::EmissionCycleClassV0::Composition);
        assert_eq!(group.dialect, super::EmissionCycleDialectV0::Css);
        assert_eq!(group.policy, super::EmissionCyclePolicyV0::ModuleIdentity);
        assert_eq!(group.members, group.chosen_order);
        Ok(())
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn dialect_import_cycles_fail_closed_with_typed_classification() {
        let fixtures = [
            (
                "css",
                StyleDialect::Css,
                super::EmissionCycleDialectV0::Css,
                TransformBundleEdgeKind::CssImport,
                "a.css",
                "b.css",
                "@import \"./b.css\"; .a { color: red; }",
                "@import \"./a.css\"; .b { color: blue; }",
            ),
            (
                "scss",
                StyleDialect::Scss,
                super::EmissionCycleDialectV0::Scss,
                TransformBundleEdgeKind::SassUse,
                "a.scss",
                "b.scss",
                "@use \"./b.scss\"; .a { color: red; }",
                "@use \"./a.scss\"; .b { color: blue; }",
            ),
            (
                "sass",
                StyleDialect::Sass,
                super::EmissionCycleDialectV0::Sass,
                TransformBundleEdgeKind::SassForward,
                "a.sass",
                "b.sass",
                "@forward \"./b.sass\"\n.a\n  color: red",
                "@forward \"./a.sass\"\n.b\n  color: blue",
            ),
            (
                "less",
                StyleDialect::Less,
                super::EmissionCycleDialectV0::Less,
                TransformBundleEdgeKind::LessImport,
                "a.less",
                "b.less",
                "@import \"./b.less\"; .a { color: red; }",
                "@import \"./a.less\"; .b { color: blue; }",
            ),
        ];

        for (
            label,
            dialect,
            expected_dialect,
            expected_edge_kind,
            first_path,
            second_path,
            first_source,
            second_source,
        ) in fixtures
        {
            let modules = vec![
                TransformBundleModuleInputV0::new(first_path, first_source, dialect),
                TransformBundleModuleInputV0::new(second_path, second_source, dialect),
            ];
            let error = link_omena_transform_bundle_modules(&[first_path], modules.as_slice())
                .expect_err("dialect import cycle must fail closed");
            assert_eq!(
                error,
                TransformBundleLinkErrorV0::UnsupportedDialectEmissionCycle {
                    dialect: expected_dialect,
                    class: super::EmissionCycleClassV0::Import,
                    edge_kinds: vec![expected_edge_kind],
                },
                "{label} import cycle must preserve its dialect and edge classification"
            );
            eprintln!(
                "EMISSION_DIALECT_CYCLE_ERROR={}",
                serde_json::to_string(&error).expect("cycle error must serialize")
            );
        }
    }

    #[test]
    fn dialect_import_acyclic_chains_remain_linkable() -> Result<(), String> {
        let fixtures = [
            (
                StyleDialect::Css,
                "a.css",
                "b.css",
                "@import \"./b.css\"; .a { color: red; }",
                ".b { color: blue; }",
            ),
            (
                StyleDialect::Scss,
                "a.scss",
                "b.scss",
                "@use \"./b.scss\"; .a { color: red; }",
                ".b { color: blue; }",
            ),
            (
                StyleDialect::Sass,
                "a.sass",
                "b.sass",
                "@forward \"./b.sass\"\n.a\n  color: red",
                ".b\n  color: blue",
            ),
            (
                StyleDialect::Less,
                "a.less",
                "b.less",
                "@import \"./b.less\"; .a { color: red; }",
                ".b { color: blue; }",
            ),
        ];

        for (dialect, first_path, second_path, first_source, second_source) in fixtures {
            let modules = vec![
                TransformBundleModuleInputV0::new(first_path, first_source, dialect),
                TransformBundleModuleInputV0::new(second_path, second_source, dialect),
            ];
            let linked = link_omena_transform_bundle_modules(&[first_path], modules.as_slice())
                .map_err(|error| format!("{dialect:?} acyclic chain: {error:?}"))?;
            assert!(linked.emission_plan.cycle_groups.is_empty());
        }
        Ok(())
    }

    #[test]
    fn unsupported_module_cycle_edge_fails_closed() {
        let first = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("a.css"));
        let second = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("b.css"));
        let input =
            |source_path: &str, instance: ModuleInstanceKeyV0, import_source: &str| LinkerInputV0 {
                source_path: source_path.to_string(),
                dialect: StyleDialect::Css,
                instance,
                dependency_edges: vec![LinkerDependencyEdgeV0 {
                    kind: TransformBundleEdgeKind::CssModuleComposesLocal,
                    import_source: import_source.to_string(),
                    import_ordinal: Some(0),
                    local_names: Vec::new(),
                    remote_names: Vec::new(),
                }],
                class_names: Vec::new(),
                keyframe_names: Vec::new(),
                value_names: Vec::new(),
                custom_property_names: Vec::new(),
                ordered_rules: Vec::new(),
            };

        let result = link_stylesheet_from_projection(
            &["a.css"],
            &[
                input("a.css", first, "./b.css"),
                input("b.css", second, "./a.css"),
            ],
        );

        assert_eq!(
            result,
            Err(TransformBundleLinkErrorV0::UnsupportedEmissionCycle {
                edge_kind: TransformBundleEdgeKind::CssModuleComposesLocal,
            })
        );
    }

    #[test]
    fn public_cascade_key_helper_normalizes_the_layer_ordinal() {
        let rule = LinkedStylesheetRuleV0 {
            global_order_index: 7,
            module_instance: ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("entry.css")),
            selector_name: "target".to_string(),
            selector_kind: "class",
            range_start: 0,
            range_end: 7,
        };
        let layer_ordinal = omena_cascade::LayerOrdinal::new(2);
        assert_eq!(layer_ordinal.map(omena_cascade::LayerOrdinal::get), Some(2));
        let Some(layer_ordinal) = layer_ordinal else {
            return;
        };
        let module_rank = omena_cascade::ModuleRank::new(3, 2, 1);
        let (key, open_world_tie_evidence) = rule.cascade_key_with_global_source_order(
            omena_cascade::CascadeLevel::AuthorNormal,
            layer_ordinal,
            false,
            0,
            omena_cascade::Specificity::new(0, 1, 0),
            module_rank,
        );

        assert_eq!(
            key.layer_rank,
            omena_cascade::normalized_layer_rank(false, Some(layer_ordinal))
        );
        assert_eq!(key.source_order, 7);
        assert_eq!(open_world_tie_evidence.module_rank, module_rank);
    }

    #[test]
    fn cascade_source_order_is_fed_by_global_rule_order() -> Result<(), String> {
        let modules = vec![
            TransformBundleModuleInputV0::new(
                "src/app.module.css",
                r#"@import "./theme.css"; .button { color: red; }"#,
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "src/theme.css",
                r#".button { color: blue; }"#,
                StyleDialect::Css,
            ),
        ];

        let linked = link_omena_transform_bundle_modules(&["src/app.module.css"], &modules)
            .map_err(|err| format!("{err:?}"))?;
        let button_rules = linked
            .global_rule_order
            .rules
            .iter()
            .filter(|rule| rule.selector_name == "button")
            .collect::<Vec<_>>();

        assert_eq!(button_rules.len(), 2);
        assert_eq!(
            button_rules
                .iter()
                .map(|rule| rule.global_order_index)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );

        let Some(layer_ordinal) = omena_cascade::LayerOrdinal::new(0) else {
            return Err("zero must remain a sentinel-safe layer ordinal".to_string());
        };
        let declarations = button_rules
            .iter()
            .map(|rule| {
                let value = if rule.global_order_index == 0 {
                    "red"
                } else {
                    "blue"
                };
                let (key, open_world_tie_evidence) = rule.cascade_key_with_global_source_order(
                    omena_cascade::CascadeLevel::AuthorNormal,
                    layer_ordinal,
                    false,
                    0,
                    omena_cascade::Specificity::new(0, 1, 0),
                    if rule.global_order_index == 0 {
                        omena_cascade::ModuleRank::new(u32::MAX, u32::MAX, u32::MAX)
                    } else {
                        omena_cascade::ModuleRank::ZERO
                    },
                );
                omena_cascade::CascadeDeclaration {
                    id: format!(
                        "{}:{}",
                        rule.module_instance.module().as_str(),
                        rule.global_order_index
                    ),
                    property: omena_cascade::AuthoredPropertyTextV0::new("color"),
                    property_key: omena_cascade::PropertyNameV0::standard("color").canonical_key(),
                    value: omena_cascade::CascadeValue::Literal(value.to_string()),
                    key,
                    open_world_tie_evidence,
                    specificity_exactness: omena_cascade::SpecificityExactnessV0::Exact,
                }
            })
            .collect::<Vec<_>>();

        let outcome = omena_cascade::cascade_property(declarations, "color");
        let omena_cascade::CascadeOutcome::Definite { winner, proof, .. } = outcome else {
            return Err("expected definite cascade winner".to_string());
        };
        assert_eq!(
            winner.value,
            omena_cascade::CascadeValue::Literal("blue".to_string())
        );
        assert_eq!(winner.key.source_order, 1);
        assert_eq!(proof.source_order, 1);
        Ok(())
    }

    #[test]
    fn cascade_closed_world_order_matches_module_rank_key_byte_identical() -> Result<(), String> {
        let modules = vec![
            TransformBundleModuleInputV0::new(
                "src/app.module.css",
                r#"@import "./theme.css"; .button { color: red; }"#,
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "src/theme.css",
                r#".button { color: blue; }"#,
                StyleDialect::Css,
            ),
        ];

        let linked = link_omena_transform_bundle_modules(&["src/app.module.css"], &modules)
            .map_err(|err| format!("{err:?}"))?;
        let Some(layer_ordinal) = omena_cascade::LayerOrdinal::new(0) else {
            return Err("zero must remain a sentinel-safe layer ordinal".to_string());
        };
        let declarations = linked
            .global_rule_order
            .rules
            .iter()
            .filter(|rule| rule.selector_name == "button")
            .map(|rule| {
                let linked_later = rule.global_order_index == 1;
                let (key, open_world_tie_evidence) = rule.cascade_key_with_global_source_order(
                    omena_cascade::CascadeLevel::AuthorNormal,
                    layer_ordinal,
                    false,
                    0,
                    omena_cascade::Specificity::new(0, 1, 0),
                    if linked_later {
                        omena_cascade::ModuleRank::new(u32::MAX, u32::MAX, u32::MAX)
                    } else {
                        omena_cascade::ModuleRank::ZERO
                    },
                );
                omena_cascade::CascadeDeclaration {
                    id: format!(
                        "{}:{}",
                        rule.module_instance.module().as_str(),
                        rule.global_order_index
                    ),
                    property: omena_cascade::AuthoredPropertyTextV0::new("color"),
                    property_key: omena_cascade::PropertyNameV0::standard("color").canonical_key(),
                    value: omena_cascade::CascadeValue::Literal(if linked_later {
                        "blue".to_string()
                    } else {
                        "red".to_string()
                    }),
                    key,
                    open_world_tie_evidence,
                    specificity_exactness: omena_cascade::SpecificityExactnessV0::Exact,
                }
            })
            .collect::<Vec<_>>();

        let linked_order_css = definite_color_css(omena_cascade::cascade_property(
            declarations.clone(),
            "color",
        ))?;
        let module_rank_keyed_css = legacy_module_rank_keyed_color_css(&declarations)?;

        assert_eq!(
            linked_order_css.as_bytes(),
            module_rank_keyed_css.as_bytes()
        );
        Ok(())
    }

    fn definite_color_css(outcome: omena_cascade::CascadeOutcome) -> Result<String, String> {
        let omena_cascade::CascadeOutcome::Definite { winner, .. } = outcome else {
            return Err("expected definite cascade winner".to_string());
        };
        let omena_cascade::CascadeValue::Literal(value) = winner.value else {
            return Err("expected literal cascade value".to_string());
        };
        Ok(format!("color:{value};"))
    }

    fn legacy_module_rank_keyed_color_css(
        declarations: &[omena_cascade::CascadeDeclaration],
    ) -> Result<String, String> {
        let mut matching = declarations.to_vec();
        matching.sort_by(|left, right| {
            legacy_module_rank_key(right)
                .cmp(&legacy_module_rank_key(left))
                .then_with(|| right.key.source_order.cmp(&left.key.source_order))
        });
        let Some(winner) = matching.first() else {
            return Err("expected cascade declarations".to_string());
        };
        let omena_cascade::CascadeValue::Literal(value) = &winner.value else {
            return Err("expected literal cascade value".to_string());
        };
        Ok(format!("color:{value};"))
    }

    fn legacy_module_rank_key(
        declaration: &omena_cascade::CascadeDeclaration,
    ) -> (
        omena_cascade::CascadeLevel,
        omena_cascade::LayerRank,
        std::cmp::Reverse<u32>,
        omena_cascade::Specificity,
        omena_cascade::ModuleRank,
    ) {
        (
            declaration.key.level,
            declaration.key.layer_rank,
            std::cmp::Reverse(declaration.key.scope_proximity),
            declaration.key.specificity,
            declaration.open_world_tie_evidence.module_rank,
        )
    }

    #[test]
    fn linker_distinguishes_configured_module_instances() {
        use omena_parser::{ConfigurationHashV0, ModuleIdV0, ModuleInstanceKeyV0};

        let module = ModuleIdV0::new("src/theme.scss");
        let blue =
            ModuleInstanceKeyV0::new(module.clone(), ConfigurationHashV0::new("with:brand=blue"));
        let red = ModuleInstanceKeyV0::new(module, ConfigurationHashV0::new("with:brand=red"));

        assert_ne!(blue, red);
        assert_eq!(blue.module(), red.module());
        assert_ne!(blue.configuration(), red.configuration());
    }

    #[test]
    fn entrypoint_prefers_the_unconfigured_instance() -> Result<(), String> {
        let modules = vec![
            TransformBundleModuleInputV0::new(
                "src/theme.scss",
                ".theme { color: black; }",
                StyleDialect::Scss,
            ),
            TransformBundleModuleInputV0::new(
                "src/theme.scss",
                ".theme { color: blue; }",
                StyleDialect::Scss,
            )
            .with_configuration_hash(ConfigurationHashV0::new("with|5:brand=4:blue")),
            TransformBundleModuleInputV0::new(
                "src/theme.scss",
                ".theme { color: red; }",
                StyleDialect::Scss,
            )
            .with_configuration_hash(ConfigurationHashV0::new("with|5:brand=3:red")),
        ];

        let linked = link_omena_transform_bundle_modules(&["src/theme.scss"], &modules)
            .map_err(|error| format!("unconfigured entrypoint should be selected: {error:?}"))?;
        assert_eq!(linked.entrypoints.len(), 1);
        assert_eq!(
            linked.entrypoints[0].configuration(),
            &ConfigurationHashV0::none()
        );
        Ok(())
    }

    #[test]
    fn entrypoint_without_an_unconfigured_instance_reports_ambiguity() {
        let modules = vec![
            TransformBundleModuleInputV0::new(
                "src/theme.scss",
                ".theme { color: blue; }",
                StyleDialect::Scss,
            )
            .with_configuration_hash(ConfigurationHashV0::new("with|5:brand=4:blue")),
            TransformBundleModuleInputV0::new(
                "src/theme.scss",
                ".theme { color: red; }",
                StyleDialect::Scss,
            )
            .with_configuration_hash(ConfigurationHashV0::new("with|5:brand=3:red")),
        ];

        assert_eq!(
            link_omena_transform_bundle_modules(&["src/theme.scss"], &modules),
            Err(TransformBundleLinkErrorV0::AmbiguousModulePath {
                source_path: "src/theme.scss".to_string(),
            })
        );
    }

    #[test]
    fn semantic_reachability_input_feeds_closed_world_bundle() -> Result<(), String> {
        let modules = vec![TransformBundleModuleInputV0::new(
            "Button.module.css",
            ".used { color: blue; } .dead { color: red; }",
            StyleDialect::Css,
        )];
        let mut reachability = TransformBundleSemanticReachabilityInputV0::new("Button.module.css");
        reachability.class_names.push("used".to_string());

        let linked = link_omena_transform_bundle_modules_with_semantic_reachability(
            &["Button.module.css"],
            &modules,
            &[reachability],
        )
        .map_err(|err| format!("semantic reachability bundle should link: {err:?}"))?;

        assert_eq!(
            linked.closed_world_bundle.reachability().class_names(),
            &["used".to_string()]
        );
        let instance = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("Button.module.css"));
        assert_eq!(
            linked
                .closed_world_bundle
                .module_reachability_evidence(&instance),
            ClosedWorldModuleReachabilityEvidenceV0::Supplied
        );
        Ok(())
    }

    #[test]
    fn analyzed_empty_semantic_reachability_narrows_the_module_to_no_symbols() -> Result<(), String>
    {
        let modules = vec![TransformBundleModuleInputV0::new(
            "Button.module.css",
            ".used { color: blue; } .dead { color: red; }",
            StyleDialect::Css,
        )];
        let reachability = TransformBundleSemanticReachabilityInputV0::new("Button.module.css");
        let projection = project_omena_transform_bundle_linker_inputs(
            modules.as_slice(),
            std::slice::from_ref(&reachability),
        );
        let instance = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("Button.module.css"));

        assert_eq!(
            projection.module_reachability_analysis(&instance),
            TransformBundleReachabilityAnalysisV0::Analyzed
        );
        assert_eq!(projection.analyzed_empty_reachability_input_count(), 1);
        assert_eq!(projection.unanalyzed_reachability_input_count(), 0);
        eprintln!(
            "REACHABILITY_ANALYSIS_CELL={{\"state\":\"analyzed\",\"cause\":null,\"analyzedEmptyCount\":{},\"unanalyzedCount\":{},\"projectedClassNameCount\":{}}}",
            projection.analyzed_empty_reachability_input_count(),
            projection.unanalyzed_reachability_input_count(),
            projection.inputs()[0].class_names.len(),
        );

        let linked = link_omena_transform_bundle_modules_with_semantic_reachability(
            &["Button.module.css"],
            &modules,
            &[reachability],
        )
        .map_err(|err| format!("semantic reachability bundle should link: {err:?}"))?;

        assert!(
            linked
                .closed_world_bundle
                .reachability()
                .class_names()
                .is_empty(),
            "an analyzed empty set must not be collapsed into missing analysis"
        );
        assert_eq!(
            linked
                .closed_world_bundle
                .module_reachability_evidence(&instance),
            ClosedWorldModuleReachabilityEvidenceV0::Supplied
        );
        Ok(())
    }

    #[test]
    fn missing_semantic_reachability_preserves_symbols_with_typed_absence() -> Result<(), String> {
        let modules = vec![TransformBundleModuleInputV0::new(
            "Button.module.css",
            ".used { color: blue; } .dead { color: red; }",
            StyleDialect::Css,
        )];
        let projection = project_omena_transform_bundle_linker_inputs(modules.as_slice(), &[]);
        let instance = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("Button.module.css"));

        assert_eq!(
            projection.module_reachability_analysis(&instance),
            TransformBundleReachabilityAnalysisV0::Unanalyzed {
                cause: TransformBundleReachabilityUnanalyzedCauseV0::InputNotProvided,
            }
        );
        assert_eq!(projection.analyzed_empty_reachability_input_count(), 0);
        assert_eq!(projection.unanalyzed_reachability_input_count(), 1);
        eprintln!(
            "REACHABILITY_ANALYSIS_CELL={{\"state\":\"unanalyzed\",\"cause\":\"inputNotProvided\",\"analyzedEmptyCount\":{},\"unanalyzedCount\":{},\"projectedClassNameCount\":{}}}",
            projection.analyzed_empty_reachability_input_count(),
            projection.unanalyzed_reachability_input_count(),
            projection.inputs()[0].class_names.len(),
        );

        let linked = link_omena_transform_bundle_modules_with_semantic_reachability(
            &["Button.module.css"],
            &modules,
            &[],
        )
        .map_err(|err| format!("semantic reachability bundle should link: {err:?}"))?;

        assert_eq!(
            linked.closed_world_bundle.reachability().class_names(),
            &["dead".to_string(), "used".to_string()]
        );
        assert_eq!(
            linked
                .closed_world_bundle
                .module_reachability_evidence(&instance),
            ClosedWorldModuleReachabilityEvidenceV0::ModuleReachabilityInputAbsent
        );
        Ok(())
    }

    #[test]
    fn instance_reachability_keeps_configured_consumers_distinct() {
        let red = ModuleInstanceKeyV0::new(
            ModuleIdV0::new("shared.module.css"),
            ConfigurationHashV0::new("with:red"),
        );
        let blue = ModuleInstanceKeyV0::new(
            ModuleIdV0::new("shared.module.css"),
            ConfigurationHashV0::new("with:blue"),
        );
        let mut inputs = vec![
            LinkerInputV0 {
                source_path: "shared.module.css".to_string(),
                dialect: StyleDialect::Css,
                instance: red.clone(),
                dependency_edges: Vec::new(),
                class_names: vec!["alpha".to_string(), "beta".to_string()],
                keyframe_names: Vec::new(),
                value_names: Vec::new(),
                custom_property_names: Vec::new(),
                ordered_rules: Vec::new(),
            },
            LinkerInputV0 {
                source_path: "shared.module.css".to_string(),
                dialect: StyleDialect::Css,
                instance: blue.clone(),
                dependency_edges: Vec::new(),
                class_names: vec!["alpha".to_string(), "beta".to_string()],
                keyframe_names: Vec::new(),
                value_names: Vec::new(),
                custom_property_names: Vec::new(),
                ordered_rules: Vec::new(),
            },
        ];
        let mut red_reachability = TransformBundleInstanceReachabilityInputV0::new(
            red.clone(),
            InstanceReachabilityDerivationV0::PathUnionNoInstanceDiscriminator,
        );
        red_reachability.class_names.push("alpha".to_string());
        let mut blue_reachability = TransformBundleInstanceReachabilityInputV0::new(
            blue.clone(),
            InstanceReachabilityDerivationV0::PathUnionNoInstanceDiscriminator,
        );
        blue_reachability.class_names.push("beta".to_string());

        let (evidence, _, _) = apply_semantic_reachability_to_linker_inputs(
            inputs.as_mut_slice(),
            &[red_reachability, blue_reachability],
        );

        carrier_hygiene_assertions::assert_configured_instance_reachability(
            inputs[0].class_names.as_slice(),
            inputs[1].class_names.as_slice(),
            &evidence,
            &red,
            &blue,
        );
    }

    #[test]
    fn instance_reachability_unions_duplicate_rows() {
        let instance = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("shared.module.css"));
        let mut inputs = vec![LinkerInputV0 {
            source_path: "shared.module.css".to_string(),
            dialect: StyleDialect::Css,
            instance: instance.clone(),
            dependency_edges: Vec::new(),
            class_names: vec!["alpha".to_string(), "beta".to_string()],
            keyframe_names: Vec::new(),
            value_names: Vec::new(),
            custom_property_names: Vec::new(),
            ordered_rules: Vec::new(),
        }];
        let mut alpha = TransformBundleInstanceReachabilityInputV0::new(
            instance.clone(),
            InstanceReachabilityDerivationV0::PathUnionNoInstanceDiscriminator,
        );
        alpha.class_names.push("alpha".to_string());
        let mut beta = TransformBundleInstanceReachabilityInputV0::new(
            instance,
            InstanceReachabilityDerivationV0::PathUnionNoInstanceDiscriminator,
        );
        beta.class_names.push("beta".to_string());

        apply_semantic_reachability_to_linker_inputs(inputs.as_mut_slice(), &[alpha, beta]);

        carrier_hygiene_assertions::assert_duplicate_instance_reachability(
            inputs[0].class_names.as_slice(),
        );
    }

    #[test]
    fn instance_reachability_dedupes_custom_property_escapes_without_folding_case() {
        let instance = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("shared.module.css"));
        let mut inputs = vec![LinkerInputV0 {
            source_path: "shared.module.css".to_string(),
            dialect: StyleDialect::Css,
            instance: instance.clone(),
            dependency_edges: Vec::new(),
            class_names: Vec::new(),
            keyframe_names: Vec::new(),
            value_names: Vec::new(),
            custom_property_names: Vec::new(),
            ordered_rules: Vec::new(),
        }];
        let mut first = TransformBundleInstanceReachabilityInputV0::new(
            instance.clone(),
            InstanceReachabilityDerivationV0::PathUnionNoInstanceDiscriminator,
        );
        first
            .custom_property_names
            .extend(["--foo".to_string(), r"--f\6f o".to_string()]);
        let mut second = TransformBundleInstanceReachabilityInputV0::new(
            instance,
            InstanceReachabilityDerivationV0::PathUnionNoInstanceDiscriminator,
        );
        second.custom_property_names.push("--FOO".to_string());

        apply_semantic_reachability_to_linker_inputs(inputs.as_mut_slice(), &[first, second]);

        assert_eq!(inputs[0].custom_property_names, ["--FOO", "--foo"]);
    }

    #[test]
    fn legacy_path_reachability_unions_normalized_rows_across_symbol_sets() {
        let modules = vec![TransformBundleModuleInputV0::new(
            "shared.module.css",
            r#"
@value primary: red;
@value secondary: blue;
@keyframes enter { from { opacity: 0; } to { opacity: 1; } }
@keyframes leave { from { opacity: 1; } to { opacity: 0; } }
:root { --primary: red; --secondary: blue; }
.alpha { animation: enter 1s; }
.beta { animation: leave 1s; }
"#,
            StyleDialect::Css,
        )];
        let mut first = TransformBundleSemanticReachabilityInputV0::new("./shared.module.css");
        first.class_names.push("alpha".to_string());
        first.keyframe_names.push("enter".to_string());
        first.value_names.push("primary".to_string());
        first.custom_property_names.push("--primary".to_string());
        let mut second = TransformBundleSemanticReachabilityInputV0::new("shared.module.css");
        second.class_names.push("beta".to_string());
        second.keyframe_names.push("leave".to_string());
        second.value_names.push("secondary".to_string());
        second.custom_property_names.push("--secondary".to_string());

        let projection = project_omena_transform_bundle_linker_inputs(&modules, &[first, second]);
        let input = &projection.inputs()[0];

        carrier_hygiene_assertions::assert_legacy_path_union(input);
    }

    fn incomplete_composes_carrier_fixture() -> (
        Vec<LinkerInputV0>,
        BTreeMap<ModuleInstanceKeyV0, ClosedWorldModuleReachabilityEvidenceV0>,
        ModuleInstanceKeyV0,
        ModuleInstanceKeyV0,
    ) {
        let source = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("entry.module.css"));
        let target = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("base.module.css"));
        let mut inputs = vec![
            LinkerInputV0 {
                source_path: "entry.module.css".to_string(),
                dialect: StyleDialect::Css,
                instance: source.clone(),
                dependency_edges: vec![LinkerDependencyEdgeV0 {
                    kind: TransformBundleEdgeKind::CssModuleComposesExternal,
                    import_source: "./base.module.css".to_string(),
                    import_ordinal: Some(0),
                    local_names: Vec::new(),
                    remote_names: vec!["base".to_string()],
                }],
                class_names: vec!["card".to_string(), "other".to_string()],
                keyframe_names: Vec::new(),
                value_names: Vec::new(),
                custom_property_names: Vec::new(),
                ordered_rules: Vec::new(),
            },
            LinkerInputV0 {
                source_path: "base.module.css".to_string(),
                dialect: StyleDialect::Css,
                instance: target.clone(),
                dependency_edges: Vec::new(),
                class_names: vec!["base".to_string(), "other".to_string()],
                keyframe_names: Vec::new(),
                value_names: Vec::new(),
                custom_property_names: Vec::new(),
                ordered_rules: Vec::new(),
            },
        ];
        let mut source_reachability = TransformBundleInstanceReachabilityInputV0::new(
            source.clone(),
            InstanceReachabilityDerivationV0::PathUnionNoInstanceDiscriminator,
        );
        source_reachability.class_names.push("card".to_string());
        let mut target_reachability = TransformBundleInstanceReachabilityInputV0::new(
            target.clone(),
            InstanceReachabilityDerivationV0::PathUnionNoInstanceDiscriminator,
        );
        target_reachability.class_names.push("base".to_string());

        let (evidence, _, _) = apply_semantic_reachability_to_linker_inputs(
            inputs.as_mut_slice(),
            &[source_reachability, target_reachability],
        );
        (inputs, evidence, source, target)
    }

    #[test]
    fn incomplete_composes_carrier_marks_closure_target_evidence_absent() {
        let (_, evidence, source, target) = incomplete_composes_carrier_fixture();
        carrier_hygiene_assertions::assert_incomplete_composes_target_evidence(
            &evidence, &source, &target,
        );
    }

    #[test]
    fn incomplete_composes_carrier_keeps_closure_target_symbols_fail_open() {
        let (inputs, _, _, _) = incomplete_composes_carrier_fixture();
        carrier_hygiene_assertions::assert_incomplete_composes_target_symbols(
            inputs[0].class_names.as_slice(),
            inputs[1].class_names.as_slice(),
        );
    }

    #[test]
    fn external_composes_names_reach_the_sealed_closed_world_bundle() -> Result<(), String> {
        let modules = vec![
            TransformBundleModuleInputV0::new(
                "entry.module.css",
                ".card { composes: base from \"./base.module.css\"; color: red; }",
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "base.module.css",
                ".base { padding: 8px; }",
                StyleDialect::Css,
            ),
        ];
        let linked = link_omena_transform_bundle_modules(&["entry.module.css"], &modules)
            .map_err(|error| format!("composes fixture should link: {error:?}"))?;

        let edges = linked.closed_world_bundle.composes_edges();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].from_module.module().as_str(), "entry.module.css");
        assert_eq!(edges[0].from_symbol, "card");
        assert_eq!(edges[0].to_module.module().as_str(), "base.module.css");
        assert_eq!(edges[0].to_symbol, "base");
        Ok(())
    }

    #[test]
    fn composes_closure_expands_module_qualified_semantic_reachability() -> Result<(), String> {
        let modules = vec![
            TransformBundleModuleInputV0::new(
                "entry.module.css",
                ".card { composes: base from \"./base.module.css\"; color: red; }",
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "base.module.css",
                ".base { padding: 8px; } .other { color: green; }",
                StyleDialect::Css,
            ),
        ];
        let mut entry_reachability =
            TransformBundleSemanticReachabilityInputV0::new("entry.module.css");
        entry_reachability.class_names.push("card".to_string());
        let mut base_reachability =
            TransformBundleSemanticReachabilityInputV0::new("base.module.css");
        base_reachability.class_names.push("other".to_string());

        let linked = link_omena_transform_bundle_modules_with_semantic_reachability(
            &["base.module.css"],
            &modules,
            &[entry_reachability, base_reachability],
        )
        .map_err(|error| format!("composes reachability fixture should link: {error:?}"))?;
        let entry = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("entry.module.css"));
        let base = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("base.module.css"));

        assert_eq!(
            linked.closed_world_bundle.linked_modules(),
            std::slice::from_ref(&base),
            "workspace scan evidence must not widen the emission module set"
        );
        assert_eq!(
            linked
                .closed_world_bundle
                .reachability()
                .symbols_for_module(&base)
                .map(|symbols| symbols.class_names()),
            Some(&["base".to_string(), "other".to_string()][..])
        );
        assert_eq!(linked.closed_world_bundle.composes_edges().len(), 1);
        assert_eq!(
            linked
                .closed_world_bundle
                .composes_origin_symbol_is_reachable(&entry, "card"),
            Some(true)
        );
        Ok(())
    }

    #[test]
    fn projection_linker_core_links_without_module_sources() -> Result<(), String> {
        let app = ModuleInstanceKeyV0::new(
            ModuleIdV0::new("src/app.module.css"),
            ConfigurationHashV0::none(),
        );
        let theme = ModuleInstanceKeyV0::new(
            ModuleIdV0::new("src/theme.css"),
            ConfigurationHashV0::none(),
        );
        let linked = link_stylesheet_from_projection(
            &["src/app.module.css"],
            &[
                LinkerInputV0 {
                    source_path: "src/app.module.css".to_string(),
                    dialect: StyleDialect::Css,
                    instance: app.clone(),
                    dependency_edges: vec![LinkerDependencyEdgeV0 {
                        kind: TransformBundleEdgeKind::CssImport,
                        import_source: "./theme.css".to_string(),
                        import_ordinal: Some(0),
                        local_names: Vec::new(),
                        remote_names: Vec::new(),
                    }],
                    class_names: vec!["app".to_string()],
                    keyframe_names: Vec::new(),
                    value_names: Vec::new(),
                    custom_property_names: Vec::new(),
                    ordered_rules: vec![LinkerRuleV0 {
                        selector_name: "app".to_string(),
                        selector_kind: ParsedSelectorFactKind::Class,
                        range_start: 0,
                        range_end: 4,
                    }],
                },
                LinkerInputV0 {
                    source_path: "src/theme.css".to_string(),
                    dialect: StyleDialect::Css,
                    instance: theme,
                    dependency_edges: Vec::new(),
                    class_names: vec!["theme".to_string()],
                    keyframe_names: Vec::new(),
                    value_names: Vec::new(),
                    custom_property_names: vec!["--brand".to_string()],
                    ordered_rules: vec![LinkerRuleV0 {
                        selector_name: "theme".to_string(),
                        selector_kind: ParsedSelectorFactKind::Class,
                        range_start: 0,
                        range_end: 6,
                    }],
                },
            ],
        )
        .map_err(|err| format!("{err:?}"))?;

        assert_eq!(linked.module_instances.len(), 2);
        assert_eq!(
            linked
                .global_rule_order
                .rules
                .iter()
                .map(|rule| rule.selector_name.as_str())
                .collect::<Vec<_>>(),
            vec!["app", "theme"]
        );
        assert!(
            linked
                .closed_world_bundle
                .reachability()
                .custom_property_names()
                .contains(&"--brand".to_string())
        );
        Ok(())
    }

    #[test]
    fn linker_reports_missing_module_dependency() {
        let modules = vec![TransformBundleModuleInputV0::new(
            "src/app.css",
            r#"@import "./missing.css"; .button { color: red; }"#,
            StyleDialect::Css,
        )];

        let err = link_omena_transform_bundle_modules(&["src/app.css"], &modules);

        assert_eq!(
            err,
            Err(TransformBundleLinkErrorV0::MissingDependency {
                source_path: "src/app.css".to_string(),
                import_source: "./missing.css".to_string(),
            })
        );
    }

    #[test]
    fn resolved_dependency_carrier_links_package_export_target() -> Result<(), String> {
        let modules = vec![
            TransformBundleModuleInputV0::new(
                "src/app.css",
                r#"@import "@acme/theme/tokens.css"; .app { color: green; }"#,
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "node_modules/@acme/theme/dist/tokens.css",
                ".token { color: rebeccapurple; }",
                StyleDialect::Css,
            ),
        ];
        let projection = project_omena_transform_bundle_linker_inputs(&modules, &[]);
        let resolved = TransformBundleResolvedDependencyV0::new(
            modules[0].module_instance_key(),
            TransformBundleEdgeKind::CssImport,
            "@acme/theme/tokens.css",
            Some(0),
            TransformBundleDependencyResolutionV0::attempted(
                vec![
                    "externalUrlBoundary",
                    "bundlerPathMapping",
                    "tsconfigPathMapping",
                    "sassPkgImporter",
                    "fileRelativeOrAbsolute",
                    "packageManifestSubpath",
                    "nodePackageFallback",
                    "sassLoadPathRoot",
                ],
                "packageStyleModule",
                1,
                Some(modules[1].module_instance_key()),
            ),
        );

        let linked = link_omena_transform_bundle_projection_with_resolved_dependencies_and_options(
            &["src/app.css"],
            &projection,
            std::slice::from_ref(&resolved),
            &[],
            TransformBundleLinkOptionsV0::default(),
        )
        .map_err(|error| format!("resolved package export should link: {error:?}"))?;

        assert_eq!(linked.module_instances.len(), 2);
        assert!(
            linked
                .module_instances
                .contains(&modules[1].module_instance_key())
        );
        assert_eq!(resolved.resolution.attempt_state, "attempted");
        assert_eq!(
            resolved.resolution.resolution_kind,
            Some("packageStyleModule")
        );
        assert_eq!(resolved.resolution.policy_step_keys.len(), 8);
        Ok(())
    }

    #[test]
    fn resolution_authority_is_enforced_per_dependency_edge() -> Result<(), String> {
        let modules = vec![
            TransformBundleModuleInputV0::new(
                "src/app.css",
                r#"@import "./tokens.css"; @import "./theme"; .app { color: green; }"#,
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "src/tokens.css",
                ".token { color: rebeccapurple; }",
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "src/theme.scss",
                ".theme { color: purple; }",
                StyleDialect::Scss,
            ),
        ];
        let projections =
            super::project_omena_transform_bundle_linker_and_emission_items(&modules, &[]);
        let resolved_tokens = TransformBundleResolvedDependencyV0::new(
            modules[0].module_instance_key(),
            TransformBundleEdgeKind::CssImport,
            "./tokens.css",
            Some(0),
            TransformBundleDependencyResolutionV0::attempted(
                vec!["fileRelativeOrAbsolute"],
                "fileRelative",
                1,
                Some(modules[1].module_instance_key()),
            ),
        );
        let resolved_theme = TransformBundleResolvedDependencyV0::new(
            modules[0].module_instance_key(),
            TransformBundleEdgeKind::CssImport,
            "./theme",
            Some(1),
            TransformBundleDependencyResolutionV0::attempted(
                vec!["fileRelativeOrAbsolute"],
                "fileRelative",
                1,
                Some(modules[2].module_instance_key()),
            ),
        );

        let strict_error = super::link_resolved_bundle(
            &["src/app.css"],
            projections.linker_projection(),
            projections.emission_item_projection(),
            std::slice::from_ref(&resolved_tokens),
            &[],
            super::EmissionOrderingPolicyV0::ImportOrderPreserving,
        );
        let legacy = super::link_legacy_path_inferred_bundle(
            &["src/app.css"],
            projections.linker_projection(),
            projections.emission_item_projection(),
            std::slice::from_ref(&resolved_tokens),
            &[],
            super::EmissionOrderingPolicyV0::ImportOrderPreserving,
        )
        .map_err(|error| format!("legacy fallback should link: {error:?}"))?;
        let inferred = legacy
            .dependency_resolution_disclosures
            .iter()
            .filter(|disclosure| {
                disclosure.authority == super::BundleResolutionAuthorityV0::LegacyPathInferred
            })
            .collect::<Vec<_>>();
        let strict = super::link_resolved_bundle(
            &["src/app.css"],
            projections.linker_projection(),
            projections.emission_item_projection(),
            &[resolved_tokens, resolved_theme],
            &[],
            super::EmissionOrderingPolicyV0::ImportOrderPreserving,
        )
        .map_err(|error| format!("complete resolved edge set should link: {error:?}"))?;
        carrier_hygiene_assertions::assert_resolution_authority(
            &strict_error,
            inferred.as_slice(),
            strict.dependency_resolution_disclosures.as_slice(),
        );
        Ok(())
    }

    #[test]
    fn emission_item_projection_preserves_the_legacy_selector_order() -> Result<(), String> {
        let modules = vec![TransformBundleModuleInputV0::new(
            "src/theme.css",
            ":root { --brand: red; }\n\
             @layer reset;\n\
             div, .theme, [hidden], *::before { color: var(--brand); }\n\
             @keyframes pulse { from { opacity: 0; } }",
            StyleDialect::Css,
        )];
        let legacy = link_omena_transform_bundle_modules(&["src/theme.css"], &modules)
            .map_err(|error| format!("legacy link failed: {error:?}"))?;
        let projections =
            super::project_omena_transform_bundle_linker_and_emission_items(&modules, &[]);
        let widened =
            super::link_omena_transform_bundle_projection_with_emission_items_and_resolved_dependencies_and_options(
                &["src/theme.css"],
                projections.linker_projection(),
                projections.emission_item_projection(),
                &[],
                &[],
                TransformBundleLinkOptionsV0::default(),
            )
            .map_err(|error| format!("emission-item link failed: {error:?}"))?;

        assert_eq!(widened.linked_stylesheet, legacy);
        assert_eq!(
            widened
                .linked_stylesheet
                .global_rule_order
                .rules
                .iter()
                .map(|rule| (rule.global_order_index, rule.selector_name.as_str()))
                .collect::<Vec<_>>(),
            vec![(0, "theme")]
        );
        assert!(widened.emission_item_order.items.iter().any(|item| {
            item.kind == super::EmissionItemKindV0::SelectorPseudoClass && item.name == ":root"
        }));
        assert!(widened.emission_item_order.items.iter().any(|item| {
            item.kind == super::EmissionItemKindV0::KeyframesDeclaration && item.name == "pulse"
        }));
        assert!(
            widened
                .emission_item_order
                .items
                .windows(2)
                .all(|pair| pair[0].global_order_index + 1 == pair[1].global_order_index)
        );
        Ok(())
    }

    #[test]
    fn emission_admission_separates_legacy_success_from_requested_projection_failure() {
        let modules = vec![TransformBundleModuleInputV0::new(
            "src/app.css",
            ".app { color: green; }",
            StyleDialect::Css,
        )];
        let projections =
            super::project_omena_transform_bundle_linker_and_emission_items(&modules, &[]);
        let empty_emission_projection =
            super::TransformBundleEmissionItemProjectionV0::new(Vec::new());

        let admission =
            super::evaluate_omena_transform_bundle_projection_emission_admission_with_resolved_dependencies_and_options(
                &["src/app.css"],
                projections.linker_projection(),
                &empty_emission_projection,
                &[],
                &[],
                TransformBundleLinkOptionsV0::default(),
            );

        assert!(!admission.module_id_legacy_open());
        assert!(matches!(
            admission.requested_policy_result(),
            Err(TransformBundleLinkErrorV0::InvalidEmissionPlan { reason })
                if reason.contains("has no emission-item input")
        ));
    }

    #[test]
    fn emission_admission_marks_both_paths_open_when_preparation_fails() {
        let modules = vec![TransformBundleModuleInputV0::new(
            "src/app.css",
            ".app { color: green; }",
            StyleDialect::Css,
        )];
        let projections =
            super::project_omena_transform_bundle_linker_and_emission_items(&modules, &[]);

        let admission =
            super::evaluate_omena_transform_bundle_projection_emission_admission_with_resolved_dependencies_and_options(
                &["src/missing.css"],
                projections.linker_projection(),
                projections.emission_item_projection(),
                &[],
                &[],
                TransformBundleLinkOptionsV0::default(),
            );

        assert!(admission.module_id_legacy_open());
        assert!(matches!(
            admission.requested_policy_result(),
            Err(TransformBundleLinkErrorV0::MissingEntrypoint { source_path })
                if source_path == "src/missing.css"
        ));
    }

    #[test]
    fn emission_item_projection_parses_each_module_once() -> Result<(), String> {
        let modules = vec![
            TransformBundleModuleInputV0::new(
                "src/reset.css",
                "html { box-sizing: border-box; }",
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "src/theme.css",
                ":root { --brand: red; }",
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "src/app.css",
                ".app { color: var(--brand); }",
                StyleDialect::Css,
            ),
        ];
        let (linked, parser_snapshot) = omena_parser::with_omena_parser_parse_instrumentation(
            || {
                let projections =
                    super::project_omena_transform_bundle_linker_and_emission_items(&modules, &[]);
                super::link_omena_transform_bundle_projection_with_emission_items_and_resolved_dependencies_and_options(
                    &["src/reset.css", "src/theme.css", "src/app.css"],
                    projections.linker_projection(),
                    projections.emission_item_projection(),
                    &[],
                    &[],
                    TransformBundleLinkOptionsV0::default(),
                )
            },
        );
        let linked = linked.map_err(|error| format!("emission-item link failed: {error:?}"))?;

        assert_eq!(parser_snapshot.parse_invocation_count, 3);
        assert_eq!(linked.linked_stylesheet.module_instances.len(), 3);
        Ok(())
    }

    fn materialize_with_emission_items(
        entrypoint: &str,
        modules: &[TransformBundleModuleInputV0],
        transformed_css: &[(&str, &str)],
    ) -> Result<
        (
            super::LinkedStylesheetWithEmissionItemsV0,
            super::LinkedEmissionArtifactV0,
        ),
        String,
    > {
        let projections =
            super::project_omena_transform_bundle_linker_and_emission_items(modules, &[]);
        let linked =
            super::link_omena_transform_bundle_projection_with_emission_items_and_resolved_dependencies_and_options(
                &[entrypoint],
                projections.linker_projection(),
                projections.emission_item_projection(),
                &[],
                &[],
                TransformBundleLinkOptionsV0 {
                    emission_ordering_policy:
                        super::EmissionOrderingPolicyV0::ImportOrderPreserving,
                    ..TransformBundleLinkOptionsV0::default()
                },
            )
            .map_err(|error| format!("emission-item link failed: {error:?}"))?;
        let transformed_modules = transformed_css
            .iter()
            .map(|(source_path, css)| {
                let module = modules
                    .iter()
                    .find(|module| module.source_path == *source_path)
                    .ok_or_else(|| format!("missing module input for {source_path}"))?;
                Ok(TransformBundleTransformedModuleV0::new(
                    module.module_instance_key(),
                    *css,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let artifact =
            super::materialize_omena_transform_bundle_linked_stylesheet_with_emission_items(
                &linked,
                &transformed_modules,
            )
            .map_err(|error| format!("emission-item materialization failed: {error:?}"))?;
        Ok((linked, artifact))
    }

    #[test]
    fn emission_items_place_element_only_import_before_the_importer() -> Result<(), String> {
        let modules = [
            TransformBundleModuleInputV0::new(
                "src/app.css",
                "@import \"./reset.css\"; div { color: red; }",
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "src/reset.css",
                "div { color: green; }",
                StyleDialect::Css,
            ),
        ];
        let (linked, artifact) = materialize_with_emission_items(
            "src/app.css",
            &modules,
            &[
                ("src/app.css", "div { color: red; }"),
                ("src/reset.css", "div { color: green; }"),
            ],
        )?;

        let mut seen_modules = BTreeSet::new();
        let first_module_occurrences = linked
            .emission_item_order
            .items
            .iter()
            .filter_map(|item| {
                seen_modules
                    .insert(item.module_instance.clone())
                    .then_some(item.module_instance.module().as_str())
            })
            .collect::<Vec<_>>();
        assert_eq!(
            first_module_occurrences,
            vec!["src/reset.css", "src/app.css"]
        );
        assert_eq!(
            artifact.output_css,
            "div { color: green; }\ndiv { color: red; }"
        );
        Ok(())
    }

    #[test]
    fn emission_items_do_not_relocate_an_element_rule_past_named_rules() -> Result<(), String> {
        let modules = [
            TransformBundleModuleInputV0::new(
                "src/app.css",
                "@import \"./reset.css\"; .card { padding: 1px; } div { color: red; }",
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "src/reset.css",
                "div { color: green; }",
                StyleDialect::Css,
            ),
        ];
        let (_, artifact) = materialize_with_emission_items(
            "src/app.css",
            &modules,
            &[
                ("src/app.css", ".card { padding: 1px; } div { color: red; }"),
                ("src/reset.css", "div { color: green; }"),
            ],
        )?;

        let green = artifact
            .output_css
            .find("green")
            .ok_or_else(|| "missing imported declaration".to_string())?;
        let card = artifact
            .output_css
            .find(".card")
            .ok_or_else(|| "missing named selector declaration".to_string())?;
        let red = artifact
            .output_css
            .find("red")
            .ok_or_else(|| "missing importing declaration".to_string())?;
        assert!(green < card && card < red);
        Ok(())
    }

    #[test]
    fn emission_item_placement_is_independent_of_module_names() -> Result<(), String> {
        let modules = [
            TransformBundleModuleInputV0::new(
                "src/zzz-app.css",
                "@import \"./aaa-reset.css\"; div { color: red; }",
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "src/aaa-reset.css",
                "div { color: green; }",
                StyleDialect::Css,
            ),
        ];
        let (_, artifact) = materialize_with_emission_items(
            "src/zzz-app.css",
            &modules,
            &[
                ("src/zzz-app.css", "div { color: red; }"),
                ("src/aaa-reset.css", "div { color: green; }"),
            ],
        )?;

        assert_eq!(
            artifact.output_css,
            "div { color: green; }\ndiv { color: red; }"
        );
        Ok(())
    }

    #[test]
    fn emission_items_preserve_cascade_layer_declaration_order() -> Result<(), String> {
        let modules = [
            TransformBundleModuleInputV0::new(
                "src/app.css",
                "@import \"./layers.css\"; @layer theme { .card { color: blue; } } \
                 @layer base { .card { color: orange; } }",
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "src/layers.css",
                "@layer base, theme;",
                StyleDialect::Css,
            ),
        ];
        let (_, artifact) = materialize_with_emission_items(
            "src/app.css",
            &modules,
            &[
                (
                    "src/app.css",
                    "@layer theme { .card { color: blue; } } \
                     @layer base { .card { color: orange; } }",
                ),
                ("src/layers.css", "@layer base, theme;"),
            ],
        )?;

        assert!(artifact.output_css.starts_with("@layer base, theme;"));
        assert!(
            artifact
                .output_css
                .find("@layer base, theme;")
                .unwrap_or(usize::MAX)
                < artifact.output_css.find("@layer theme").unwrap_or_default()
        );
        Ok(())
    }

    #[test]
    fn emission_item_materializer_preserves_empty_module_placement() -> Result<(), String> {
        let modules = [
            TransformBundleModuleInputV0::new(
                "src/app.css",
                "@import \"./empty.css\"; @import \"./license.css\"; .app { color: red; }",
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new("src/empty.css", "", StyleDialect::Css),
            TransformBundleModuleInputV0::new(
                "src/license.css",
                "/* license */",
                StyleDialect::Css,
            ),
        ];
        let projections =
            super::project_omena_transform_bundle_linker_and_emission_items(&modules, &[]);
        let linked =
            super::link_omena_transform_bundle_projection_with_emission_items_and_resolved_dependencies_and_options(
                &["src/app.css"],
                projections.linker_projection(),
                projections.emission_item_projection(),
                &[],
                &[],
                TransformBundleLinkOptionsV0 {
                    emission_ordering_policy:
                        super::EmissionOrderingPolicyV0::ImportOrderPreserving,
                    ..TransformBundleLinkOptionsV0::default()
                },
            )
            .map_err(|error| format!("emission-item link failed: {error:?}"))?;
        let transformed = modules
            .iter()
            .map(|module| {
                let output_css = match module.source_path.as_str() {
                    "src/app.css" => ".app { color: red; }",
                    "src/license.css" => "/* license */",
                    _ => "",
                };
                TransformBundleTransformedModuleV0::new(
                    module.module_instance_key(),
                    output_css.to_string(),
                )
            })
            .collect::<Vec<_>>();

        let artifact =
            super::materialize_omena_transform_bundle_linked_stylesheet_with_emission_items(
                &linked,
                &transformed,
            )
            .map_err(|error| format!("emission-item materialization failed: {error:?}"))?;
        assert_eq!(artifact.emitted_module_count, 3);
        assert_eq!(
            artifact
                .module_regions
                .iter()
                .map(|region| region.module_instance.module().as_str())
                .collect::<Vec<_>>(),
            ["src/empty.css", "src/license.css", "src/app.css"]
        );
        assert_eq!(artifact.output_css, "/* license */\n.app { color: red; }");
        Ok(())
    }

    #[test]
    fn emission_item_materializer_rejects_incomplete_module_coverage() -> Result<(), String> {
        let modules = [
            TransformBundleModuleInputV0::new(
                "src/app.css",
                "@import \"./reset.css\"; .app { color: red; }",
                StyleDialect::Css,
            ),
            TransformBundleModuleInputV0::new(
                "src/reset.css",
                "html { box-sizing: border-box; }",
                StyleDialect::Css,
            ),
        ];
        let projections =
            super::project_omena_transform_bundle_linker_and_emission_items(&modules, &[]);
        let mut linked =
            super::link_omena_transform_bundle_projection_with_emission_items_and_resolved_dependencies_and_options(
                &["src/app.css"],
                projections.linker_projection(),
                projections.emission_item_projection(),
                &[],
                &[],
                TransformBundleLinkOptionsV0 {
                    emission_ordering_policy:
                        super::EmissionOrderingPolicyV0::ImportOrderPreserving,
                    ..TransformBundleLinkOptionsV0::default()
                },
            )
            .map_err(|error| format!("emission-item link failed: {error:?}"))?;
        let missing_module = modules[1].module_instance_key();
        linked
            .emission_item_order
            .items
            .retain(|item| item.module_instance != missing_module);
        for (index, item) in linked.emission_item_order.items.iter_mut().enumerate() {
            item.global_order_index = u32::try_from(index)
                .map_err(|_| "emission-item test order exceeds u32".to_string())?;
        }
        let transformed = modules
            .iter()
            .map(|module| {
                TransformBundleTransformedModuleV0::new(
                    module.module_instance_key(),
                    module.source.clone(),
                )
            })
            .collect::<Vec<_>>();

        match super::materialize_omena_transform_bundle_linked_stylesheet_with_emission_items(
            &linked,
            &transformed,
        ) {
            Err(super::LinkedEmissionItemMaterializationErrorV0::MissingEmissionItem {
                module_instance,
            }) if module_instance == missing_module => Ok(()),
            result => Err(format!(
                "incomplete emission-item coverage returned an unexpected result: {result:?}"
            )),
        }
    }
}

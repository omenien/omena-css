use serde::{Deserialize, Serialize};
#[cfg(any(test, feature = "test-support"))]
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

pub const EVIDENCE_GRAPH_SCHEMA_VERSION_V0: &str = "0";
pub const EVIDENCE_GRAPH_PRODUCT_V0: &str = "omena-evidence-graph.graph";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GuaranteeKindV0 {
    Floor,
    SampledFixtureWitness,
    SchedulerPriorityFixtureWitness,
    MetricInputFixtureWitness,
    IncrementalLayerEvidenceOnly,
    AlphaRenamingStableHashFixtureWitness,
    NotClaimedExactTraversal,
}

impl GuaranteeKindV0 {
    pub const fn for_label_less_family() -> Self {
        Self::Floor
    }

    pub const fn existing_label(self) -> Option<&'static str> {
        match self {
            Self::Floor => None,
            Self::SampledFixtureWitness => Some("sampledFixtureWitnessNotEquivalenceProof"),
            Self::SchedulerPriorityFixtureWitness => Some("fixtureWitnessSchedulerPriority"),
            Self::MetricInputFixtureWitness => Some("fixtureWitnessMetricInput"),
            Self::IncrementalLayerEvidenceOnly => Some("m6IncrementalLayerEvidenceOnly"),
            Self::AlphaRenamingStableHashFixtureWitness => {
                Some("fixtureWitnessAlphaRenamingStableHash")
            }
            Self::NotClaimedExactTraversal => Some("notClaimedExactTraversal"),
        }
    }

    pub fn from_existing_label(label: &str) -> Option<Self> {
        match label {
            "sampledFixtureWitnessNotEquivalenceProof" => Some(Self::SampledFixtureWitness),
            "fixtureWitnessSchedulerPriority" => Some(Self::SchedulerPriorityFixtureWitness),
            "fixtureWitnessMetricInput" => Some(Self::MetricInputFixtureWitness),
            "m6IncrementalLayerEvidenceOnly" => Some(Self::IncrementalLayerEvidenceOnly),
            "fixtureWitnessAlphaRenamingStableHash" => {
                Some(Self::AlphaRenamingStableHashFixtureWitness)
            }
            "notClaimedExactTraversal" => Some(Self::NotClaimedExactTraversal),
            _ => None,
        }
    }
}

/// `GuaranteeKindV0` records what a node guarantees; this records how that
/// guarantee was earned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GuaranteeFamilyV0 {
    ByteIdentityOracle,
    ExternalReplicaDifferential,
    PropertyCorpusWitness,
    TypedInvariantWitness,
    ProseObligationDischarged,
    FloorAssumption,
    LedgerBackedObligationDischarge,
}

impl GuaranteeFamilyV0 {
    pub const fn describe(self) -> &'static str {
        match self {
            Self::ByteIdentityOracle => "byteIdentityOracle",
            Self::ExternalReplicaDifferential => "externalReplicaDifferential",
            Self::PropertyCorpusWitness => "propertyCorpusWitness",
            Self::TypedInvariantWitness => "typedInvariantWitness",
            Self::ProseObligationDischarged => "proseObligationDischarged",
            Self::FloorAssumption => "floorAssumption",
            Self::LedgerBackedObligationDischarge => "ledgerBackedObligationDischarge",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FamilyStampSealV0(());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteIdentityOracleTokenV0(FamilyStampSealV0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExternalReplicaDifferentialTokenV0(FamilyStampSealV0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropertyCorpusWitnessTokenV0(FamilyStampSealV0);

impl PropertyCorpusWitnessTokenV0 {
    pub fn from_conformance_ledger(
        record_count: usize,
        all_records_have_one_verdict: bool,
        all_passes_accounted_for: bool,
        all_families_non_vacuous_or_named_gap: bool,
    ) -> Option<Self> {
        (record_count > 0
            && all_records_have_one_verdict
            && all_passes_accounted_for
            && all_families_non_vacuous_or_named_gap)
            .then_some(Self(FamilyStampSealV0(())))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypedInvariantWitnessTokenV0(FamilyStampSealV0);

impl TypedInvariantWitnessTokenV0 {
    pub const fn from_incremental_layer_evidence() -> Self {
        Self(FamilyStampSealV0(()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProseObligationProvenanceV0(FamilyStampSealV0);

impl ProseObligationProvenanceV0 {
    pub fn from_provenance_labels(labels: &[String]) -> Option<Self> {
        labels
            .iter()
            .any(|label| {
                label.starts_with("obligation:")
                    || label.starts_with("cascadeObligationDeclared:")
                    || label.starts_with("enforcedAt:")
                    || label.starts_with("primitive:")
            })
            .then_some(Self(FamilyStampSealV0(())))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LedgerDischargeWitnessV0(FamilyStampSealV0);

impl LedgerDischargeWitnessV0 {
    pub fn from_discharge_cell_key_v0(cell_key: &str) -> Option<Self> {
        (cell_key.len() == 64 && cell_key.bytes().all(|byte| byte.is_ascii_hexdigit()))
            .then_some(Self(FamilyStampSealV0(())))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FamilyStampV0 {
    earned_via: GuaranteeFamilyV0,
    _seal: FamilyStampSealV0,
}

impl FamilyStampV0 {
    pub const fn floor_assumption() -> Self {
        Self::from_family(GuaranteeFamilyV0::FloorAssumption)
    }

    pub const fn byte_identity_oracle(_token: &ByteIdentityOracleTokenV0) -> Self {
        Self::from_family(GuaranteeFamilyV0::ByteIdentityOracle)
    }

    pub const fn external_replica_differential(
        _token: &ExternalReplicaDifferentialTokenV0,
    ) -> Self {
        Self::from_family(GuaranteeFamilyV0::ExternalReplicaDifferential)
    }

    pub const fn property_corpus_witness(_token: &PropertyCorpusWitnessTokenV0) -> Self {
        Self::from_family(GuaranteeFamilyV0::PropertyCorpusWitness)
    }

    pub const fn typed_invariant_witness(_token: &TypedInvariantWitnessTokenV0) -> Self {
        Self::from_family(GuaranteeFamilyV0::TypedInvariantWitness)
    }

    pub const fn prose_obligation_discharged(_provenance: &ProseObligationProvenanceV0) -> Self {
        Self::from_family(GuaranteeFamilyV0::ProseObligationDischarged)
    }

    pub const fn ledger_backed_obligation_discharge(_witness: &LedgerDischargeWitnessV0) -> Self {
        Self::from_family(GuaranteeFamilyV0::LedgerBackedObligationDischarge)
    }

    pub const fn earned_via(self) -> GuaranteeFamilyV0 {
        self.earned_via
    }

    const fn from_family(earned_via: GuaranteeFamilyV0) -> Self {
        Self {
            earned_via,
            _seal: FamilyStampSealV0(()),
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
thread_local! {
    static EARNED_GUARANTEE_FAMILY_READS_V0: Cell<u64> = const { Cell::new(0) };
}

#[cfg(any(test, feature = "test-support"))]
pub fn reset_earned_guarantee_family_read_count_v0() {
    EARNED_GUARANTEE_FAMILY_READS_V0.with(|count| count.set(0));
}

#[cfg(any(test, feature = "test-support"))]
pub fn earned_guarantee_family_read_count_v0() -> u64 {
    EARNED_GUARANTEE_FAMILY_READS_V0.with(Cell::get)
}

pub const REWRITE_OBLIGATION_FAMILY_PRODUCT_V0: &str =
    "omena-evidence-graph.rewrite-obligation-family-closure";
pub const REWRITE_OBLIGATION_FAMILY_COUNT_V0: usize = 45;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ObligationFamilyIdV0 {
    CascadeSafetyFloor,
    CascadeObligationDeclaration,
    ComputedValuePreservation,
    WhitespaceBoundary,
    CommentSourceMapProvenance,
    NumericLiteralEquivalence,
    DimensionComputedValue,
    ColorLiteralEquivalence,
    UrlTokenGrammar,
    StringTextAndFontValue,
    SelectorSpecificityAndCascade,
    LonghandShorthandCascadeOutcome,
    DeclarationCascadeOrder,
    RuleMergeWinnerOrder,
    SelectorIdentityAndModuleSemantics,
    SemanticMarkerRetention,
    TargetPrefixAddition,
    StalePrefixRemovalMapping,
    TargetFallbackBranch,
    ColorSpaceTargetEquivalence,
    TargetColorPrecision,
    DirectionalityOption,
    NestedSelectorSpecificity,
    ScopedMatching,
    LayerOrderComparison,
    TargetFeaturePredicate,
    MediaPredicate,
    ContainerPredicate,
    NativeCssStaticValue,
    CalcExpressionEquivalence,
    ImportWrapperProvenance,
    ScssNamespaceProvenance,
    LessNamespaceProvenance,
    SelectorIdentityMap,
    ComposedClassProvenance,
    ValueGraphResolution,
    CustomPropertyFixedPoint,
    SourceClassReachability,
    AnimationNameReachability,
    ValueGraphReachability,
    VarReachability,
    DeadMediaWitness,
    DeadSupportsWitness,
    DesignTokenPackageProvenance,
    SourceMapTransformTrace,
}

impl ObligationFamilyIdV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CascadeSafetyFloor => "cascadeSafetyFloor",
            Self::CascadeObligationDeclaration => "cascadeObligationDeclaration",
            Self::ComputedValuePreservation => "computedValuePreservation",
            Self::WhitespaceBoundary => "whitespaceBoundary",
            Self::CommentSourceMapProvenance => "commentSourceMapProvenance",
            Self::NumericLiteralEquivalence => "numericLiteralEquivalence",
            Self::DimensionComputedValue => "dimensionComputedValue",
            Self::ColorLiteralEquivalence => "colorLiteralEquivalence",
            Self::UrlTokenGrammar => "urlTokenGrammar",
            Self::StringTextAndFontValue => "stringTextAndFontValue",
            Self::SelectorSpecificityAndCascade => "selectorSpecificityAndCascade",
            Self::LonghandShorthandCascadeOutcome => "longhandShorthandCascadeOutcome",
            Self::DeclarationCascadeOrder => "declarationCascadeOrder",
            Self::RuleMergeWinnerOrder => "ruleMergeWinnerOrder",
            Self::SelectorIdentityAndModuleSemantics => "selectorIdentityAndModuleSemantics",
            Self::SemanticMarkerRetention => "semanticMarkerRetention",
            Self::TargetPrefixAddition => "targetPrefixAddition",
            Self::StalePrefixRemovalMapping => "stalePrefixRemovalMapping",
            Self::TargetFallbackBranch => "targetFallbackBranch",
            Self::ColorSpaceTargetEquivalence => "colorSpaceTargetEquivalence",
            Self::TargetColorPrecision => "targetColorPrecision",
            Self::DirectionalityOption => "directionalityOption",
            Self::NestedSelectorSpecificity => "nestedSelectorSpecificity",
            Self::ScopedMatching => "scopedMatching",
            Self::LayerOrderComparison => "layerOrderComparison",
            Self::TargetFeaturePredicate => "targetFeaturePredicate",
            Self::MediaPredicate => "mediaPredicate",
            Self::ContainerPredicate => "containerPredicate",
            Self::NativeCssStaticValue => "nativeCssStaticValue",
            Self::CalcExpressionEquivalence => "calcExpressionEquivalence",
            Self::ImportWrapperProvenance => "importWrapperProvenance",
            Self::ScssNamespaceProvenance => "scssNamespaceProvenance",
            Self::LessNamespaceProvenance => "lessNamespaceProvenance",
            Self::SelectorIdentityMap => "selectorIdentityMap",
            Self::ComposedClassProvenance => "composedClassProvenance",
            Self::ValueGraphResolution => "valueGraphResolution",
            Self::CustomPropertyFixedPoint => "customPropertyFixedPoint",
            Self::SourceClassReachability => "sourceClassReachability",
            Self::AnimationNameReachability => "animationNameReachability",
            Self::ValueGraphReachability => "valueGraphReachability",
            Self::VarReachability => "varReachability",
            Self::DeadMediaWitness => "deadMediaWitness",
            Self::DeadSupportsWitness => "deadSupportsWitness",
            Self::DesignTokenPackageProvenance => "designTokenPackageProvenance",
            Self::SourceMapTransformTrace => "sourceMapTransformTrace",
        }
    }

    pub const fn descriptor(self) -> RewriteObligationFamilyDescriptorV0 {
        match self {
            Self::CascadeSafetyFloor => {
                descriptor(self, "", GuaranteeKindV0::for_label_less_family())
            }
            Self::CascadeObligationDeclaration => descriptor(
                self,
                "must declare the rewrite-safety obligation family before cascade-sensitive rewrite evidence is emitted",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::ComputedValuePreservation => descriptor(
                self,
                "must preserve computed value semantics when a rewrite candidate claims computed-value preservation",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::WhitespaceBoundary => descriptor(
                self,
                "may remove only whitespace outside string, url, attr, and calc-sensitive token boundaries",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::CommentSourceMapProvenance => descriptor(
                self,
                "may remove comments only when source-map provenance preserves the removed span",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::NumericLiteralEquivalence => descriptor(
                self,
                "may rewrite only numerically equivalent literal tokens",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::DimensionComputedValue => descriptor(
                self,
                "may normalize only dimension values whose computed value is unchanged",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::ColorLiteralEquivalence => descriptor(
                self,
                "may rewrite only color-equivalent literal tokens",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::UrlTokenGrammar => descriptor(
                self,
                "may remove url quotes only when the unquoted token grammar remains equivalent",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::StringTextAndFontValue => descriptor(
                self,
                "may normalize string quotes and font keyword aliases only when computed text and font values remain equivalent",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::SelectorSpecificityAndCascade => descriptor(
                self,
                "must preserve selector specificity, keyframe timeline positions, and matching semantics under the cascade model",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::LonghandShorthandCascadeOutcome => descriptor(
                self,
                "must prove longhand and shorthand cascade outcomes are equivalent",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::DeclarationCascadeOrder => descriptor(
                self,
                "must preserve origin, layer, specificity, and order for every surviving declaration",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::RuleMergeWinnerOrder => descriptor(
                self,
                "must prove merged rule order cannot change declaration winners",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::SelectorIdentityAndModuleSemantics => descriptor(
                self,
                "must preserve selector identity and post-hash module semantics",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::SemanticMarkerRetention => descriptor(
                self,
                "may remove rules only when no source-visible semantic marker is attached",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::TargetPrefixAddition => descriptor(
                self,
                "must add target-required prefixed declarations without changing modern target outcomes",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::StalePrefixRemovalMapping => descriptor(
                self,
                "may remove prefixed declarations only when an explicit mapping and exact unprefixed peer prove the prefix stale",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::TargetFallbackBranch => descriptor(
                self,
                "must lower only when target data requires fallback branches and provenance tracks both branches",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::ColorSpaceTargetEquivalence => descriptor(
                self,
                "must lower only when color-space conversion is target-equivalent",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::TargetColorPrecision => descriptor(
                self,
                "must preserve color semantics within the configured target fallback precision",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::DirectionalityOption => descriptor(
                self,
                "must run only under explicit directionality options",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::NestedSelectorSpecificity => descriptor(
                self,
                "must preserve nested selector expansion and specificity",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::ScopedMatching => descriptor(
                self,
                "must preserve scoped matching semantics or emit a blocked result",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::LayerOrderComparison => descriptor(
                self,
                "must preserve layer order in CascadeKey comparison",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::TargetFeaturePredicate => descriptor(
                self,
                "may remove branches only when the target feature predicate is known",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::MediaPredicate => descriptor(
                self,
                "may remove branches only when the configured media predicate is known",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::ContainerPredicate => descriptor(
                self,
                "may remove @container branches only when the size condition is provably unsatisfiable regardless of container context",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::NativeCssStaticValue => descriptor(
                self,
                "may fold native CSS if() and function calls only when the evaluator proves a concrete static value and preserves runtime-dependent constructs verbatim",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::CalcExpressionEquivalence => descriptor(
                self,
                "may reduce only syntax-equivalent or computed-value-equivalent calc expressions",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::ImportWrapperProvenance => descriptor(
                self,
                "must preserve import-site media, supports, layer wrappers, and source provenance",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::ScssNamespaceProvenance => descriptor(
                self,
                "must preserve SCSS namespace, show/hide, mixin, variable, and source provenance facts",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::LessNamespaceProvenance => descriptor(
                self,
                "must preserve Less variable, mixin, namespace, and source provenance facts",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::SelectorIdentityMap => descriptor(
                self,
                "must rewrite every source and style reference through the same selector identity map",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::ComposedClassProvenance => descriptor(
                self,
                "must preserve exported class set and composed class provenance",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::ValueGraphResolution => descriptor(
                self,
                "must preserve @value graph resolution and cycle diagnostics",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::CustomPropertyFixedPoint => descriptor(
                self,
                "must preserve custom-property fixed-point semantics or emit a provenance-backed blocked result",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::SourceClassReachability => descriptor(
                self,
                "may remove classes only when bridge reachability proves no reachable source expression observes them",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::AnimationNameReachability => descriptor(
                self,
                "may remove keyframes only when animation-name reachability proves they are unobservable",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::ValueGraphReachability => descriptor(
                self,
                "may remove @value declarations only when value-graph traversal proves they are unreachable",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::VarReachability => descriptor(
                self,
                "may remove custom properties only when var() reachability proves they are unobservable",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::DeadMediaWitness => descriptor(
                self,
                "may remove @media branches only when target and cascade witnesses prove deadness",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::DeadSupportsWitness => descriptor(
                self,
                "may remove @supports branches only when target and cascade witnesses prove deadness",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::DesignTokenPackageProvenance => descriptor(
                self,
                "must preserve design-token provenance while routing declarations across package boundaries",
                GuaranteeKindV0::for_label_less_family(),
            ),
            Self::SourceMapTransformTrace => descriptor(
                self,
                "must emit a source-map trace for every non-trivia transformed span",
                GuaranteeKindV0::for_label_less_family(),
            ),
        }
    }

    pub const fn declares_cascade_obligation(self) -> bool {
        !matches!(self, Self::CascadeSafetyFloor)
    }

    pub const fn preserves_computed_value(self) -> bool {
        matches!(self, Self::ComputedValuePreservation)
    }

    pub const fn from_computed_value_preservation(preserved: bool) -> Self {
        if preserved {
            Self::ComputedValuePreservation
        } else {
            Self::CascadeSafetyFloor
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriteObligationFamilyDescriptorV0 {
    pub id: ObligationFamilyIdV0,
    pub family_name: &'static str,
    pub obligation: &'static str,
    pub justifiable_guarantee: GuaranteeKindV0,
}

const fn descriptor(
    id: ObligationFamilyIdV0,
    obligation: &'static str,
    justifiable_guarantee: GuaranteeKindV0,
) -> RewriteObligationFamilyDescriptorV0 {
    RewriteObligationFamilyDescriptorV0 {
        id,
        family_name: id.as_str(),
        obligation,
        justifiable_guarantee,
    }
}

pub const fn list_rewrite_obligation_families_v0()
-> [RewriteObligationFamilyDescriptorV0; REWRITE_OBLIGATION_FAMILY_COUNT_V0] {
    [
        ObligationFamilyIdV0::CascadeSafetyFloor.descriptor(),
        ObligationFamilyIdV0::CascadeObligationDeclaration.descriptor(),
        ObligationFamilyIdV0::ComputedValuePreservation.descriptor(),
        ObligationFamilyIdV0::WhitespaceBoundary.descriptor(),
        ObligationFamilyIdV0::CommentSourceMapProvenance.descriptor(),
        ObligationFamilyIdV0::NumericLiteralEquivalence.descriptor(),
        ObligationFamilyIdV0::DimensionComputedValue.descriptor(),
        ObligationFamilyIdV0::ColorLiteralEquivalence.descriptor(),
        ObligationFamilyIdV0::UrlTokenGrammar.descriptor(),
        ObligationFamilyIdV0::StringTextAndFontValue.descriptor(),
        ObligationFamilyIdV0::SelectorSpecificityAndCascade.descriptor(),
        ObligationFamilyIdV0::LonghandShorthandCascadeOutcome.descriptor(),
        ObligationFamilyIdV0::DeclarationCascadeOrder.descriptor(),
        ObligationFamilyIdV0::RuleMergeWinnerOrder.descriptor(),
        ObligationFamilyIdV0::SelectorIdentityAndModuleSemantics.descriptor(),
        ObligationFamilyIdV0::SemanticMarkerRetention.descriptor(),
        ObligationFamilyIdV0::TargetPrefixAddition.descriptor(),
        ObligationFamilyIdV0::StalePrefixRemovalMapping.descriptor(),
        ObligationFamilyIdV0::TargetFallbackBranch.descriptor(),
        ObligationFamilyIdV0::ColorSpaceTargetEquivalence.descriptor(),
        ObligationFamilyIdV0::TargetColorPrecision.descriptor(),
        ObligationFamilyIdV0::DirectionalityOption.descriptor(),
        ObligationFamilyIdV0::NestedSelectorSpecificity.descriptor(),
        ObligationFamilyIdV0::ScopedMatching.descriptor(),
        ObligationFamilyIdV0::LayerOrderComparison.descriptor(),
        ObligationFamilyIdV0::TargetFeaturePredicate.descriptor(),
        ObligationFamilyIdV0::MediaPredicate.descriptor(),
        ObligationFamilyIdV0::ContainerPredicate.descriptor(),
        ObligationFamilyIdV0::NativeCssStaticValue.descriptor(),
        ObligationFamilyIdV0::CalcExpressionEquivalence.descriptor(),
        ObligationFamilyIdV0::ImportWrapperProvenance.descriptor(),
        ObligationFamilyIdV0::ScssNamespaceProvenance.descriptor(),
        ObligationFamilyIdV0::LessNamespaceProvenance.descriptor(),
        ObligationFamilyIdV0::SelectorIdentityMap.descriptor(),
        ObligationFamilyIdV0::ComposedClassProvenance.descriptor(),
        ObligationFamilyIdV0::ValueGraphResolution.descriptor(),
        ObligationFamilyIdV0::CustomPropertyFixedPoint.descriptor(),
        ObligationFamilyIdV0::SourceClassReachability.descriptor(),
        ObligationFamilyIdV0::AnimationNameReachability.descriptor(),
        ObligationFamilyIdV0::ValueGraphReachability.descriptor(),
        ObligationFamilyIdV0::VarReachability.descriptor(),
        ObligationFamilyIdV0::DeadMediaWitness.descriptor(),
        ObligationFamilyIdV0::DeadSupportsWitness.descriptor(),
        ObligationFamilyIdV0::DesignTokenPackageProvenance.descriptor(),
        ObligationFamilyIdV0::SourceMapTransformTrace.descriptor(),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriteObligationFamilyRetirementRecordV0 {
    pub family_name: &'static str,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriteObligationFamilyCarrierBindingV0 {
    pub family: ObligationFamilyIdV0,
    pub carrier: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriteObligationFamilyClosureSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub registered_family_count: usize,
    pub carrier_bound_family_count: usize,
    pub retirement_record_count: usize,
    pub orphan_family_names: Vec<&'static str>,
    pub extra_carrier_family_names: Vec<&'static str>,
    pub retirement_records: Vec<RewriteObligationFamilyRetirementRecordV0>,
    pub untyped_carrier_count: usize,
    pub untyped_prose_arm_count: usize,
    pub closure_passed: bool,
}

pub fn summarize_rewrite_obligation_family_closure_v0(
    carrier_bindings: impl IntoIterator<Item = RewriteObligationFamilyCarrierBindingV0>,
    retirement_records: impl IntoIterator<Item = RewriteObligationFamilyRetirementRecordV0>,
    untyped_carrier_count: usize,
    untyped_prose_arm_count: usize,
) -> RewriteObligationFamilyClosureSummaryV0 {
    summarize_rewrite_obligation_family_closure_from_names_v0(
        carrier_bindings
            .into_iter()
            .map(|binding| binding.family.as_str()),
        retirement_records,
        untyped_carrier_count,
        untyped_prose_arm_count,
    )
}

pub fn summarize_rewrite_obligation_family_closure_from_names_v0(
    carrier_bound_family_names: impl IntoIterator<Item = &'static str>,
    retirement_records: impl IntoIterator<Item = RewriteObligationFamilyRetirementRecordV0>,
    untyped_carrier_count: usize,
    untyped_prose_arm_count: usize,
) -> RewriteObligationFamilyClosureSummaryV0 {
    let registered_family_names = list_rewrite_obligation_families_v0()
        .into_iter()
        .map(|descriptor| descriptor.family_name)
        .collect::<BTreeSet<_>>();
    let carrier_bound_family_names = carrier_bound_family_names
        .into_iter()
        .collect::<BTreeSet<_>>();
    let retirement_records = retirement_records.into_iter().collect::<Vec<_>>();
    let retired_family_names = retirement_records
        .iter()
        .map(|record| record.family_name)
        .collect::<BTreeSet<_>>();

    let orphan_family_names = registered_family_names
        .difference(&carrier_bound_family_names)
        .copied()
        .filter(|family| !retired_family_names.contains(family))
        .collect::<Vec<_>>();
    let extra_carrier_family_names = carrier_bound_family_names
        .difference(&registered_family_names)
        .copied()
        .collect::<Vec<_>>();
    let closure_passed = orphan_family_names.is_empty()
        && extra_carrier_family_names.is_empty()
        && untyped_carrier_count == 0
        && untyped_prose_arm_count == 0;

    RewriteObligationFamilyClosureSummaryV0 {
        schema_version: EVIDENCE_GRAPH_SCHEMA_VERSION_V0,
        product: REWRITE_OBLIGATION_FAMILY_PRODUCT_V0,
        registered_family_count: registered_family_names.len(),
        carrier_bound_family_count: carrier_bound_family_names.len(),
        retirement_record_count: retirement_records.len(),
        orphan_family_names,
        extra_carrier_family_names,
        retirement_records,
        untyped_carrier_count,
        untyped_prose_arm_count,
        closure_passed,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceNodeKeyV0 {
    pub query_identity: String,
    pub input_identity: String,
}

impl EvidenceNodeKeyV0 {
    pub fn new(query_identity: impl Into<String>, input_identity: impl Into<String>) -> Self {
        Self {
            query_identity: query_identity.into(),
            input_identity: input_identity.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceNodeSeedV0 {
    pub key: EvidenceNodeKeyV0,
    pub provenance: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision: Option<EvidenceAnalysisPrecisionV0>,
    pub guarantee: GuaranteeKindV0,
    pub earned_via: GuaranteeFamilyV0,
}

impl EvidenceNodeSeedV0 {
    pub fn new(
        key: EvidenceNodeKeyV0,
        provenance: Vec<String>,
        guarantee: GuaranteeKindV0,
    ) -> Self {
        Self::with_precision(key, provenance, None, guarantee)
    }

    pub fn with_precision(
        key: EvidenceNodeKeyV0,
        provenance: Vec<String>,
        precision: Option<EvidenceAnalysisPrecisionV0>,
        guarantee: GuaranteeKindV0,
    ) -> Self {
        Self::with_precision_and_family(
            key,
            provenance,
            precision,
            guarantee,
            FamilyStampV0::floor_assumption(),
        )
    }

    pub fn with_family(
        key: EvidenceNodeKeyV0,
        provenance: Vec<String>,
        guarantee: GuaranteeKindV0,
        family_stamp: FamilyStampV0,
    ) -> Self {
        Self::with_precision_and_family(key, provenance, None, guarantee, family_stamp)
    }

    pub fn with_precision_and_family(
        key: EvidenceNodeKeyV0,
        provenance: Vec<String>,
        precision: Option<EvidenceAnalysisPrecisionV0>,
        guarantee: GuaranteeKindV0,
        family_stamp: FamilyStampV0,
    ) -> Self {
        Self {
            key,
            provenance,
            precision,
            guarantee,
            earned_via: family_stamp.earned_via(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceAnalysisPrecisionV0 {
    pub product: String,
    pub value_domain: String,
    pub flow_sensitivity: String,
    pub context_sensitivity: String,
    pub revision_axis: String,
}

impl EvidenceAnalysisPrecisionV0 {
    pub fn new(
        product: impl Into<String>,
        value_domain: impl Into<String>,
        flow_sensitivity: impl Into<String>,
        context_sensitivity: impl Into<String>,
        revision_axis: impl Into<String>,
    ) -> Self {
        Self {
            product: product.into(),
            value_domain: value_domain.into(),
            flow_sensitivity: flow_sensitivity.into(),
            context_sensitivity: context_sensitivity.into(),
            revision_axis: revision_axis.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceDemandEdgeV0 {
    pub from_query_identity: String,
    pub to_node_key: EvidenceNodeKeyV0,
    pub edge_kind: String,
}

impl EvidenceDemandEdgeV0 {
    pub fn new(
        from_query_identity: impl Into<String>,
        to_node_key: EvidenceNodeKeyV0,
        edge_kind: impl Into<String>,
    ) -> Self {
        Self {
            from_query_identity: from_query_identity.into(),
            to_node_key,
            edge_kind: edge_kind.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceNodeV0 {
    pub key: EvidenceNodeKeyV0,
    pub provenance: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision: Option<EvidenceAnalysisPrecisionV0>,
    pub guarantee: GuaranteeKindV0,
    earned_via: GuaranteeFamilyV0,
}

impl EvidenceNodeV0 {
    pub fn earned_via(&self) -> GuaranteeFamilyV0 {
        #[cfg(any(test, feature = "test-support"))]
        EARNED_GUARANTEE_FAMILY_READS_V0.with(|count| count.set(count.get().saturating_add(1)));
        self.earned_via
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceGraphV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub nodes: Vec<EvidenceNodeV0>,
    pub edges: Vec<EvidenceDemandEdgeV0>,
}

impl EvidenceGraphV0 {
    pub fn node_input_identities(&self) -> BTreeSet<String> {
        self.nodes
            .iter()
            .map(|node| node.key.input_identity.clone())
            .collect()
    }

    pub fn edge_input_identities(&self) -> BTreeSet<String> {
        self.edges
            .iter()
            .map(|edge| edge.to_node_key.input_identity.clone())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceGraphBuildErrorV0 {
    MissingDemandNode(EvidenceNodeKeyV0),
}

pub fn build_salsa_demand_evidence_graph_v0(
    all_node_seeds: impl IntoIterator<Item = EvidenceNodeSeedV0>,
    demand_edges: impl IntoIterator<Item = EvidenceDemandEdgeV0>,
) -> Result<EvidenceGraphV0, EvidenceGraphBuildErrorV0> {
    build_evidence_graph_from_edges_v0(all_node_seeds, demand_edges)
}

pub fn build_evidence_graph_from_edges_v0(
    all_node_seeds: impl IntoIterator<Item = EvidenceNodeSeedV0>,
    demand_edges: impl IntoIterator<Item = EvidenceDemandEdgeV0>,
) -> Result<EvidenceGraphV0, EvidenceGraphBuildErrorV0> {
    let all_nodes = all_node_seeds
        .into_iter()
        .map(|seed| (seed.key.clone(), seed))
        .collect::<BTreeMap<_, _>>();
    let edges = demand_edges.into_iter().collect::<Vec<_>>();
    let demanded_keys = edges
        .iter()
        .map(|edge| edge.to_node_key.clone())
        .collect::<BTreeSet<_>>();

    let mut nodes = Vec::new();
    for key in demanded_keys {
        let Some(seed) = all_nodes.get(&key) else {
            return Err(EvidenceGraphBuildErrorV0::MissingDemandNode(key));
        };
        nodes.push(EvidenceNodeV0 {
            key: seed.key.clone(),
            provenance: seed.provenance.clone(),
            precision: seed.precision.clone(),
            guarantee: seed.guarantee,
            earned_via: seed.earned_via,
        });
    }

    Ok(EvidenceGraphV0 {
        schema_version: EVIDENCE_GRAPH_SCHEMA_VERSION_V0,
        product: EVIDENCE_GRAPH_PRODUCT_V0,
        nodes,
        edges,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node_seed(input_identity: &str) -> EvidenceNodeSeedV0 {
        EvidenceNodeSeedV0::new(
            EvidenceNodeKeyV0::new("memo_style_fact_entry", input_identity),
            vec!["fixture-provenance".to_string()],
            GuaranteeKindV0::for_label_less_family(),
        )
    }

    #[test]
    fn rewrite_obligation_family_registry_is_closed_and_label_honest() {
        let descriptors = list_rewrite_obligation_families_v0();
        assert_eq!(descriptors.len(), REWRITE_OBLIGATION_FAMILY_COUNT_V0);
        let family_names = descriptors
            .iter()
            .map(|descriptor| descriptor.family_name)
            .collect::<BTreeSet<_>>();
        assert_eq!(family_names.len(), REWRITE_OBLIGATION_FAMILY_COUNT_V0);
        for descriptor in descriptors {
            assert_eq!(descriptor.family_name, descriptor.id.as_str());
            assert!(
                [
                    GuaranteeKindV0::Floor,
                    GuaranteeKindV0::SampledFixtureWitness,
                    GuaranteeKindV0::SchedulerPriorityFixtureWitness,
                    GuaranteeKindV0::MetricInputFixtureWitness,
                    GuaranteeKindV0::IncrementalLayerEvidenceOnly,
                    GuaranteeKindV0::AlphaRenamingStableHashFixtureWitness,
                    GuaranteeKindV0::NotClaimedExactTraversal,
                ]
                .contains(&descriptor.justifiable_guarantee)
            );
        }
    }

    #[test]
    fn rewrite_obligation_family_closure_reports_both_directions() {
        let summary = summarize_rewrite_obligation_family_closure_from_names_v0(
            ["computedValuePreservation", "unknownFamily"],
            [RewriteObligationFamilyRetirementRecordV0 {
                family_name: "cascadeSafetyFloor",
                reason: "test fixture",
            }],
            0,
            0,
        );
        assert!(
            summary
                .orphan_family_names
                .contains(&"cascadeObligationDeclaration")
        );
        assert!(
            summary
                .extra_carrier_family_names
                .contains(&"unknownFamily")
        );
        assert!(!summary.closure_passed);
    }

    #[test]
    fn rewrite_obligation_family_closure_has_non_vacuous_retirement_branch() {
        let all_live_except_floor = list_rewrite_obligation_families_v0()
            .into_iter()
            .filter(|descriptor| descriptor.family_name != "cascadeSafetyFloor")
            .map(|descriptor| RewriteObligationFamilyCarrierBindingV0 {
                family: descriptor.id,
                carrier: "fixture",
            })
            .collect::<Vec<_>>();
        let summary = summarize_rewrite_obligation_family_closure_v0(
            all_live_except_floor,
            [RewriteObligationFamilyRetirementRecordV0 {
                family_name: "cascadeSafetyFloor",
                reason: "test fixture",
            }],
            0,
            0,
        );
        assert!(summary.closure_passed);
        assert_eq!(summary.retirement_record_count, 1);
    }

    #[test]
    fn guarantee_kind_round_trips_existing_labels_without_upgrading_label_less_nodes() {
        for kind in [
            GuaranteeKindV0::SampledFixtureWitness,
            GuaranteeKindV0::SchedulerPriorityFixtureWitness,
            GuaranteeKindV0::MetricInputFixtureWitness,
            GuaranteeKindV0::IncrementalLayerEvidenceOnly,
            GuaranteeKindV0::AlphaRenamingStableHashFixtureWitness,
            GuaranteeKindV0::NotClaimedExactTraversal,
        ] {
            let label = kind.existing_label();
            assert_eq!(
                label.and_then(GuaranteeKindV0::from_existing_label),
                Some(kind)
            );
        }
        assert_eq!(
            GuaranteeKindV0::for_label_less_family(),
            GuaranteeKindV0::Floor
        );
        assert_eq!(GuaranteeKindV0::Floor.existing_label(), None);
    }

    #[test]
    fn guarantee_family_descriptions_are_closed_and_honest() {
        let families = [
            (GuaranteeFamilyV0::ByteIdentityOracle, "byteIdentityOracle"),
            (
                GuaranteeFamilyV0::ExternalReplicaDifferential,
                "externalReplicaDifferential",
            ),
            (
                GuaranteeFamilyV0::PropertyCorpusWitness,
                "propertyCorpusWitness",
            ),
            (
                GuaranteeFamilyV0::TypedInvariantWitness,
                "typedInvariantWitness",
            ),
            (
                GuaranteeFamilyV0::ProseObligationDischarged,
                "proseObligationDischarged",
            ),
            (GuaranteeFamilyV0::FloorAssumption, "floorAssumption"),
            (
                GuaranteeFamilyV0::LedgerBackedObligationDischarge,
                "ledgerBackedObligationDischarge",
            ),
        ];

        assert_eq!(families.len(), 7);
        for (family, description) in families {
            assert_eq!(family.describe(), description);
        }
    }

    #[test]
    fn evidence_nodes_preserve_floor_and_mechanism_families() -> Result<(), &'static str> {
        let floor_key = EvidenceNodeKeyV0::new("floor_query", "floor_input");
        let prose_key = EvidenceNodeKeyV0::new("prose_query", "prose_input");
        let prose_labels = vec![
            "pass:rule-merge".to_string(),
            "obligation:preserve declaration winner order".to_string(),
        ];
        let Some(prose_provenance) =
            ProseObligationProvenanceV0::from_provenance_labels(&prose_labels)
        else {
            return Err("prose provenance labels must mint a wrapper");
        };
        let graph = build_evidence_graph_from_edges_v0(
            [
                EvidenceNodeSeedV0::new(
                    floor_key.clone(),
                    vec!["floor-input".to_string()],
                    GuaranteeKindV0::for_label_less_family(),
                ),
                EvidenceNodeSeedV0::with_family(
                    prose_key.clone(),
                    prose_labels,
                    GuaranteeKindV0::for_label_less_family(),
                    FamilyStampV0::prose_obligation_discharged(&prose_provenance),
                ),
            ],
            [
                EvidenceDemandEdgeV0::new("floor_query", floor_key, "fixture-edge"),
                EvidenceDemandEdgeV0::new("prose_query", prose_key, "fixture-edge"),
            ],
        )
        .map_err(|_| "fixture graph must build")?;

        assert_eq!(graph.nodes[0].guarantee, GuaranteeKindV0::Floor);
        assert_eq!(
            graph.nodes[0].earned_via(),
            GuaranteeFamilyV0::FloorAssumption
        );
        assert_eq!(graph.nodes[1].guarantee, GuaranteeKindV0::Floor);
        assert_eq!(
            graph.nodes[1].earned_via(),
            GuaranteeFamilyV0::ProseObligationDischarged
        );
        Ok(())
    }

    #[test]
    fn ledger_discharge_stamp_requires_cell_key_shape() {
        assert!(
            LedgerDischargeWitnessV0::from_discharge_cell_key_v0(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            )
            .is_some()
        );
        assert!(LedgerDischargeWitnessV0::from_discharge_cell_key_v0("not-a-cell").is_none());
    }

    #[test]
    fn earned_family_read_counter_records_access_path() -> Result<(), &'static str> {
        reset_earned_guarantee_family_read_count_v0();
        let key = EvidenceNodeKeyV0::new("counter_query", "counter_input");
        let graph = build_evidence_graph_from_edges_v0(
            [EvidenceNodeSeedV0::new(
                key.clone(),
                vec!["counter-input".to_string()],
                GuaranteeKindV0::for_label_less_family(),
            )],
            [EvidenceDemandEdgeV0::new(
                "counter_query",
                key,
                "fixture-edge",
            )],
        )
        .map_err(|_| "fixture graph must build")?;

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(earned_guarantee_family_read_count_v0(), 0);
        assert_eq!(
            graph.nodes[0].earned_via(),
            GuaranteeFamilyV0::FloorAssumption
        );
        assert_eq!(
            graph.nodes[0].earned_via(),
            GuaranteeFamilyV0::FloorAssumption
        );
        assert_eq!(earned_guarantee_family_read_count_v0(), 2);
        Ok(())
    }

    #[test]
    fn salsa_demand_graph_keys_on_edges_not_the_full_node_list() -> Result<(), &'static str> {
        let graph = build_salsa_demand_evidence_graph_v0(
            [
                node_seed("/workspace/src/App.module.scss"),
                node_seed("/workspace/src/_theme.scss"),
            ],
            [EvidenceDemandEdgeV0::new(
                "memo_workspace_diagnostics_substrate",
                EvidenceNodeKeyV0::new("memo_style_fact_entry", "/workspace/src/App.module.scss"),
                "salsa-demand-read",
            )],
        )
        .map_err(|_| "demand edge must target a known node")?;

        assert_eq!(
            graph.node_input_identities(),
            BTreeSet::from(["/workspace/src/App.module.scss".to_string()])
        );
        assert_eq!(
            graph.edge_input_identities(),
            BTreeSet::from(["/workspace/src/App.module.scss".to_string()])
        );
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.edges.len(), 1);
        Ok(())
    }

    #[test]
    fn salsa_demand_graph_rejects_fabricated_edges() {
        let result = build_salsa_demand_evidence_graph_v0(
            [node_seed("/workspace/src/App.module.scss")],
            [EvidenceDemandEdgeV0::new(
                "memo_workspace_diagnostics_substrate",
                EvidenceNodeKeyV0::new("memo_style_fact_entry", "/workspace/src/_missing.scss"),
                "salsa-demand-read",
            )],
        );

        assert_eq!(
            result,
            Err(EvidenceGraphBuildErrorV0::MissingDemandNode(
                EvidenceNodeKeyV0::new("memo_style_fact_entry", "/workspace/src/_missing.scss")
            ))
        );
    }

    #[test]
    fn graph_serializes_without_shape_specific_runtime_dependencies() -> Result<(), &'static str> {
        let graph = build_salsa_demand_evidence_graph_v0(
            [node_seed("/workspace/src/App.module.scss")],
            [EvidenceDemandEdgeV0::new(
                "memo_workspace_diagnostics_substrate",
                EvidenceNodeKeyV0::new("memo_style_fact_entry", "/workspace/src/App.module.scss"),
                "salsa-demand-read",
            )],
        )
        .map_err(|_| "demand edge must target a known node")?;
        let json = serde_json::to_value(&graph).map_err(|_| "graph must serialize")?;
        assert_eq!(json["schemaVersion"], "0");
        assert_eq!(json["product"], EVIDENCE_GRAPH_PRODUCT_V0);
        assert_eq!(json["nodes"][0]["guarantee"], "floor");
        Ok(())
    }

    #[test]
    fn graph_preserves_optional_precision_payload() -> Result<(), &'static str> {
        let key = EvidenceNodeKeyV0::new("source_diagnostic_precision", "missingSelector");
        let graph = build_salsa_demand_evidence_graph_v0(
            [EvidenceNodeSeedV0::with_precision(
                key.clone(),
                vec!["omena-query.source-syntax-index".to_string()],
                Some(EvidenceAnalysisPrecisionV0::new(
                    "omena-query.analysis-precision",
                    "classValueResolution",
                    "sourceSyntaxIndex",
                    "perSourceReference",
                    "OmenaQuerySourceDiagnosticsForFileV0.input",
                )),
                GuaranteeKindV0::for_label_less_family(),
            )],
            [EvidenceDemandEdgeV0::new(
                "source_diagnostic_precision",
                key,
                "diagnostic-evidence",
            )],
        )
        .map_err(|_| "precision edge must target a known node")?;

        let precision = graph.nodes[0]
            .precision
            .as_ref()
            .ok_or("precision payload must round-trip through the graph")?;
        assert_eq!(precision.value_domain, "classValueResolution");
        assert_eq!(precision.flow_sensitivity, "sourceSyntaxIndex");
        assert_eq!(precision.context_sensitivity, "perSourceReference");
        Ok(())
    }
}

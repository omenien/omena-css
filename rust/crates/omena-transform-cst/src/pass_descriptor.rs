//! Transform pass descriptors: observation contracts, semantic classes, and build profiles.

use serde::Serialize;

use super::{
    TransformDagEdgeV0, TransformPassKind, all_transform_pass_kinds, default_transform_dag_edges,
    transform_pass_execution_phase,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ObservationKindV0 {
    SelectorMatching,
    CascadeWinner,
    ExportedClassNames,
    CustomPropertyComputedValue,
    KeyframesReachability,
    SourceMapTrace,
    LayerRank,
    Specificity,
    Inheritance,
    DeclarationOrder,
    TargetPredicate,
    ModuleResolution,
    ImportContext,
    ValueGraphReachability,
    SemanticMarker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PassAssumptionKindV0 {
    TokenBoundary,
    SourceMapProvenance,
    EquivalentLiteralValue,
    SelectorSpecificity,
    LonghandShorthandEquivalence,
    DeclarationOrder,
    TargetEnvironment,
    Directionality,
    NestedSelectorExpansion,
    ScopedMatching,
    LayerOrder,
    StaticPredicate,
    ImportWrapperProvenance,
    ModuleNamespace,
    SelectorIdentityMap,
    ValueGraph,
    CustomPropertyFixedPoint,
    ClosedWorldReachability,
    EmissionTrace,
    SemanticMarkerRetention,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PassSemanticContractV0 {
    pub observes: Vec<ObservationKindV0>,
    pub preserves: Vec<ObservationKindV0>,
    pub requires: Vec<PassAssumptionKindV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PassObservationSurfaceV0 {
    Declared(PassSemanticContractV0),
    UnknownGap { reason: &'static str },
}

impl PassObservationSurfaceV0 {
    pub fn is_declared(&self) -> bool {
        matches!(self, Self::Declared(_))
    }

    pub fn gap_reason(&self) -> Option<&'static str> {
        match self {
            Self::Declared(_) => None,
            Self::UnknownGap { reason } => Some(reason),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassObservationRecordV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub id: &'static str,
    pub kind: TransformPassKind,
    pub surface: PassObservationSurfaceV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformPassClassV0 {
    Structural,
    TextLocal,
    ModuleEvaluation,
    Emission,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassDescriptorV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub id: &'static str,
    pub kind: TransformPassKind,
    pub pass_class: TransformPassClassV0,
    pub phase: u8,
    pub phase_order: u16,
    pub depends_on: Vec<&'static str>,
    pub conflicts_with: Vec<&'static str>,
}

impl TransformPassDescriptorV0 {
    pub fn is_structural(&self) -> bool {
        self.pass_class == TransformPassClassV0::Structural
    }

    pub fn keeps_text_local_slice_rewrite(&self) -> bool {
        self.pass_class == TransformPassClassV0::TextLocal
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformBuildProfileV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub profile_id: &'static str,
    pub pass_ids: Vec<&'static str>,
}

pub fn transform_build_profile_from_passes(
    profile_id: &'static str,
    passes: &[TransformPassKind],
) -> TransformBuildProfileV0 {
    TransformBuildProfileV0 {
        schema_version: "0",
        product: "omena-transform-cst.build-profile",
        profile_id,
        pass_ids: passes.iter().map(|pass| pass.id()).collect(),
    }
}

pub fn default_transform_pass_descriptors() -> Vec<TransformPassDescriptorV0> {
    let dag_edges = default_transform_dag_edges();
    all_transform_pass_kinds()
        .into_iter()
        .map(|kind| transform_pass_descriptor_with_edges(kind, dag_edges.as_slice()))
        .collect()
}

pub fn transform_pass_descriptor(kind: TransformPassKind) -> TransformPassDescriptorV0 {
    let dag_edges = default_transform_dag_edges();
    transform_pass_descriptor_with_edges(kind, dag_edges.as_slice())
}

pub fn default_transform_pass_observation_records() -> Vec<TransformPassObservationRecordV0> {
    all_transform_pass_kinds()
        .into_iter()
        .map(|kind| TransformPassObservationRecordV0 {
            schema_version: "0",
            product: "omena-transform-cst.pass-observation-surface",
            id: kind.id(),
            kind,
            surface: pass_observation_contract(kind),
        })
        .collect()
}

pub fn pass_observation_contract(kind: TransformPassKind) -> PassObservationSurfaceV0 {
    match kind {
        TransformPassKind::WhitespaceStrip => declared_observation_contract(
            &[ObservationKindV0::SourceMapTrace],
            &[
                ObservationKindV0::SelectorMatching,
                ObservationKindV0::CascadeWinner,
                ObservationKindV0::DeclarationOrder,
                ObservationKindV0::SourceMapTrace,
            ],
            &[PassAssumptionKindV0::TokenBoundary],
        ),
        TransformPassKind::CommentStrip => declared_observation_contract(
            &[ObservationKindV0::SourceMapTrace],
            &[
                ObservationKindV0::SelectorMatching,
                ObservationKindV0::CascadeWinner,
                ObservationKindV0::DeclarationOrder,
                ObservationKindV0::SourceMapTrace,
            ],
            &[PassAssumptionKindV0::SourceMapProvenance],
        ),
        TransformPassKind::NumberCompression
        | TransformPassKind::UnitNormalization
        | TransformPassKind::ColorCompression
        | TransformPassKind::UrlQuoteStrip
        | TransformPassKind::StringQuoteNormalize => declared_observation_contract(
            &[
                ObservationKindV0::CascadeWinner,
                ObservationKindV0::CustomPropertyComputedValue,
                ObservationKindV0::SourceMapTrace,
            ],
            &[
                ObservationKindV0::CascadeWinner,
                ObservationKindV0::CustomPropertyComputedValue,
                ObservationKindV0::SourceMapTrace,
            ],
            &[PassAssumptionKindV0::EquivalentLiteralValue],
        ),
        TransformPassKind::SelectorIsWhereCompression => declared_observation_contract(
            &[
                ObservationKindV0::SelectorMatching,
                ObservationKindV0::Specificity,
                ObservationKindV0::KeyframesReachability,
            ],
            &[
                ObservationKindV0::SelectorMatching,
                ObservationKindV0::Specificity,
                ObservationKindV0::KeyframesReachability,
            ],
            &[PassAssumptionKindV0::SelectorSpecificity],
        ),
        TransformPassKind::ShorthandCombining => declared_observation_contract(
            &[
                ObservationKindV0::CascadeWinner,
                ObservationKindV0::DeclarationOrder,
                ObservationKindV0::Inheritance,
            ],
            &[
                ObservationKindV0::CascadeWinner,
                ObservationKindV0::DeclarationOrder,
                ObservationKindV0::Inheritance,
            ],
            &[PassAssumptionKindV0::LonghandShorthandEquivalence],
        ),
        TransformPassKind::RuleDeduplication | TransformPassKind::RuleMerging => {
            declared_observation_contract(
                &[
                    ObservationKindV0::CascadeWinner,
                    ObservationKindV0::LayerRank,
                    ObservationKindV0::Specificity,
                    ObservationKindV0::DeclarationOrder,
                ],
                &[
                    ObservationKindV0::CascadeWinner,
                    ObservationKindV0::LayerRank,
                    ObservationKindV0::Specificity,
                    ObservationKindV0::DeclarationOrder,
                ],
                &[PassAssumptionKindV0::DeclarationOrder],
            )
        }
        TransformPassKind::SelectorMerging => declared_observation_contract(
            &[
                ObservationKindV0::SelectorMatching,
                ObservationKindV0::Specificity,
                ObservationKindV0::ExportedClassNames,
            ],
            &[
                ObservationKindV0::SelectorMatching,
                ObservationKindV0::Specificity,
                ObservationKindV0::ExportedClassNames,
            ],
            &[
                PassAssumptionKindV0::SelectorSpecificity,
                PassAssumptionKindV0::SelectorIdentityMap,
            ],
        ),
        TransformPassKind::EmptyRuleRemoval => declared_observation_contract(
            &[
                ObservationKindV0::SemanticMarker,
                ObservationKindV0::SourceMapTrace,
            ],
            &[
                ObservationKindV0::SemanticMarker,
                ObservationKindV0::SourceMapTrace,
            ],
            &[PassAssumptionKindV0::SemanticMarkerRetention],
        ),
        TransformPassKind::VendorPrefixing
        | TransformPassKind::StalePrefixRemoval
        | TransformPassKind::LightDarkLowering
        | TransformPassKind::ColorMixLowering
        | TransformPassKind::OklchOklabLowering
        | TransformPassKind::ColorFunctionLowering
        | TransformPassKind::RelativeColorLowering => declared_observation_contract(
            &[
                ObservationKindV0::TargetPredicate,
                ObservationKindV0::CascadeWinner,
                ObservationKindV0::CustomPropertyComputedValue,
            ],
            &[
                ObservationKindV0::CascadeWinner,
                ObservationKindV0::CustomPropertyComputedValue,
            ],
            &[PassAssumptionKindV0::TargetEnvironment],
        ),
        TransformPassKind::LogicalToPhysical => declared_observation_contract(
            &[
                ObservationKindV0::CascadeWinner,
                ObservationKindV0::Inheritance,
            ],
            &[
                ObservationKindV0::CascadeWinner,
                ObservationKindV0::Inheritance,
            ],
            &[PassAssumptionKindV0::Directionality],
        ),
        TransformPassKind::NestingUnwrap => declared_observation_contract(
            &[
                ObservationKindV0::SelectorMatching,
                ObservationKindV0::Specificity,
            ],
            &[
                ObservationKindV0::SelectorMatching,
                ObservationKindV0::Specificity,
            ],
            &[PassAssumptionKindV0::NestedSelectorExpansion],
        ),
        TransformPassKind::ScopeFlatten => declared_observation_contract(
            &[ObservationKindV0::SelectorMatching],
            &[ObservationKindV0::SelectorMatching],
            &[PassAssumptionKindV0::ScopedMatching],
        ),
        TransformPassKind::LayerFlatten => declared_observation_contract(
            &[
                ObservationKindV0::LayerRank,
                ObservationKindV0::CascadeWinner,
            ],
            &[
                ObservationKindV0::LayerRank,
                ObservationKindV0::CascadeWinner,
            ],
            &[PassAssumptionKindV0::LayerOrder],
        ),
        TransformPassKind::SupportsStaticEval
        | TransformPassKind::MediaStaticEval
        | TransformPassKind::ContainerStaticEval
        | TransformPassKind::NativeCssStaticEval => declared_observation_contract(
            &[
                ObservationKindV0::TargetPredicate,
                ObservationKindV0::CascadeWinner,
            ],
            &[ObservationKindV0::CascadeWinner],
            &[PassAssumptionKindV0::StaticPredicate],
        ),
        TransformPassKind::CalcReduction => declared_observation_contract(
            &[
                ObservationKindV0::CustomPropertyComputedValue,
                ObservationKindV0::CascadeWinner,
            ],
            &[
                ObservationKindV0::CustomPropertyComputedValue,
                ObservationKindV0::CascadeWinner,
            ],
            &[PassAssumptionKindV0::EquivalentLiteralValue],
        ),
        TransformPassKind::ImportInline => declared_observation_contract(
            &[
                ObservationKindV0::ImportContext,
                ObservationKindV0::LayerRank,
                ObservationKindV0::CascadeWinner,
                ObservationKindV0::SourceMapTrace,
            ],
            &[
                ObservationKindV0::ImportContext,
                ObservationKindV0::LayerRank,
                ObservationKindV0::CascadeWinner,
                ObservationKindV0::SourceMapTrace,
            ],
            &[PassAssumptionKindV0::ImportWrapperProvenance],
        ),
        TransformPassKind::ScssModuleEvaluate | TransformPassKind::LessModuleEvaluate => {
            declared_observation_contract(
                &[
                    ObservationKindV0::ModuleResolution,
                    ObservationKindV0::CustomPropertyComputedValue,
                    ObservationKindV0::SourceMapTrace,
                ],
                &[
                    ObservationKindV0::ModuleResolution,
                    ObservationKindV0::CustomPropertyComputedValue,
                    ObservationKindV0::SourceMapTrace,
                ],
                &[PassAssumptionKindV0::ModuleNamespace],
            )
        }
        TransformPassKind::HashCssModuleClassNames => declared_observation_contract(
            &[
                ObservationKindV0::ExportedClassNames,
                ObservationKindV0::SelectorMatching,
            ],
            &[
                ObservationKindV0::ExportedClassNames,
                ObservationKindV0::SelectorMatching,
            ],
            &[PassAssumptionKindV0::SelectorIdentityMap],
        ),
        TransformPassKind::ResolveCssModulesComposes => declared_observation_contract(
            &[
                ObservationKindV0::ExportedClassNames,
                ObservationKindV0::ModuleResolution,
            ],
            &[
                ObservationKindV0::ExportedClassNames,
                ObservationKindV0::ModuleResolution,
            ],
            &[PassAssumptionKindV0::SelectorIdentityMap],
        ),
        TransformPassKind::ValueResolution => declared_observation_contract(
            &[ObservationKindV0::ValueGraphReachability],
            &[ObservationKindV0::ValueGraphReachability],
            &[PassAssumptionKindV0::ValueGraph],
        ),
        TransformPassKind::StaticVarSubstitution => declared_observation_contract(
            &[ObservationKindV0::CustomPropertyComputedValue],
            &[ObservationKindV0::CustomPropertyComputedValue],
            &[PassAssumptionKindV0::CustomPropertyFixedPoint],
        ),
        TransformPassKind::TreeShakeClass => declared_observation_contract(
            &[
                ObservationKindV0::ExportedClassNames,
                ObservationKindV0::SelectorMatching,
            ],
            &[
                ObservationKindV0::ExportedClassNames,
                ObservationKindV0::SelectorMatching,
            ],
            &[PassAssumptionKindV0::ClosedWorldReachability],
        ),
        TransformPassKind::TreeShakeKeyframes => declared_observation_contract(
            &[ObservationKindV0::KeyframesReachability],
            &[ObservationKindV0::KeyframesReachability],
            &[PassAssumptionKindV0::ClosedWorldReachability],
        ),
        TransformPassKind::TreeShakeValue => declared_observation_contract(
            &[ObservationKindV0::ValueGraphReachability],
            &[ObservationKindV0::ValueGraphReachability],
            &[PassAssumptionKindV0::ClosedWorldReachability],
        ),
        TransformPassKind::TreeShakeCustomProperty => declared_observation_contract(
            &[
                ObservationKindV0::CustomPropertyComputedValue,
                ObservationKindV0::ValueGraphReachability,
            ],
            &[
                ObservationKindV0::CustomPropertyComputedValue,
                ObservationKindV0::ValueGraphReachability,
            ],
            &[PassAssumptionKindV0::ClosedWorldReachability],
        ),
        TransformPassKind::DeadMediaBranchRemoval
        | TransformPassKind::DeadSupportsBranchRemoval => declared_observation_contract(
            &[
                ObservationKindV0::TargetPredicate,
                ObservationKindV0::CascadeWinner,
            ],
            &[ObservationKindV0::CascadeWinner],
            &[PassAssumptionKindV0::StaticPredicate],
        ),
        TransformPassKind::DesignTokenRouting => declared_observation_contract(
            &[
                ObservationKindV0::CustomPropertyComputedValue,
                ObservationKindV0::ModuleResolution,
            ],
            &[
                ObservationKindV0::CustomPropertyComputedValue,
                ObservationKindV0::ModuleResolution,
            ],
            &[PassAssumptionKindV0::ModuleNamespace],
        ),
        TransformPassKind::PrintCss => declared_observation_contract(
            &[ObservationKindV0::SourceMapTrace],
            &[ObservationKindV0::SourceMapTrace],
            &[PassAssumptionKindV0::EmissionTrace],
        ),
    }
}

fn declared_observation_contract(
    observes: &[ObservationKindV0],
    preserves: &[ObservationKindV0],
    requires: &[PassAssumptionKindV0],
) -> PassObservationSurfaceV0 {
    PassObservationSurfaceV0::Declared(PassSemanticContractV0 {
        observes: observes.to_vec(),
        preserves: preserves.to_vec(),
        requires: requires.to_vec(),
    })
}

pub const fn transform_pass_class(kind: TransformPassKind) -> TransformPassClassV0 {
    match kind {
        TransformPassKind::NestingUnwrap
        | TransformPassKind::ScopeFlatten
        | TransformPassKind::LayerFlatten
        | TransformPassKind::RuleDeduplication
        | TransformPassKind::RuleMerging
        | TransformPassKind::SelectorMerging
        | TransformPassKind::EmptyRuleRemoval
        | TransformPassKind::SupportsStaticEval
        | TransformPassKind::MediaStaticEval
        | TransformPassKind::ContainerStaticEval
        | TransformPassKind::NativeCssStaticEval
        | TransformPassKind::DeadMediaBranchRemoval
        | TransformPassKind::DeadSupportsBranchRemoval
        | TransformPassKind::TreeShakeClass
        | TransformPassKind::TreeShakeKeyframes
        | TransformPassKind::TreeShakeValue
        | TransformPassKind::TreeShakeCustomProperty
        | TransformPassKind::ResolveCssModulesComposes
        | TransformPassKind::HashCssModuleClassNames
        | TransformPassKind::ImportInline
        | TransformPassKind::DesignTokenRouting => TransformPassClassV0::Structural,
        TransformPassKind::ScssModuleEvaluate | TransformPassKind::LessModuleEvaluate => {
            TransformPassClassV0::ModuleEvaluation
        }
        TransformPassKind::PrintCss => TransformPassClassV0::Emission,
        _ => TransformPassClassV0::TextLocal,
    }
}

const fn transform_pass_phase_order(kind: TransformPassKind) -> u16 {
    match kind {
        TransformPassKind::WhitespaceStrip => 10,
        TransformPassKind::CommentStrip => 20,
        TransformPassKind::NumberCompression => 30,
        TransformPassKind::UnitNormalization => 40,
        TransformPassKind::ColorCompression => 50,
        TransformPassKind::UrlQuoteStrip => 60,
        TransformPassKind::StringQuoteNormalize => 70,
        TransformPassKind::SelectorIsWhereCompression => 80,
        TransformPassKind::ShorthandCombining => 90,
        TransformPassKind::RuleDeduplication => 100,
        TransformPassKind::RuleMerging => 110,
        TransformPassKind::SelectorMerging => 120,
        TransformPassKind::EmptyRuleRemoval => 130,
        TransformPassKind::VendorPrefixing => 140,
        TransformPassKind::StalePrefixRemoval => 150,
        TransformPassKind::LightDarkLowering => 160,
        TransformPassKind::ColorMixLowering => 170,
        TransformPassKind::OklchOklabLowering => 180,
        TransformPassKind::ColorFunctionLowering => 190,
        TransformPassKind::LogicalToPhysical => 200,
        TransformPassKind::NestingUnwrap => 210,
        TransformPassKind::ScopeFlatten => 220,
        TransformPassKind::LayerFlatten => 230,
        TransformPassKind::SupportsStaticEval => 240,
        TransformPassKind::MediaStaticEval => 250,
        TransformPassKind::CalcReduction => 260,
        TransformPassKind::ImportInline => 270,
        TransformPassKind::ScssModuleEvaluate => 280,
        TransformPassKind::LessModuleEvaluate => 290,
        TransformPassKind::HashCssModuleClassNames => 300,
        TransformPassKind::ResolveCssModulesComposes => 310,
        TransformPassKind::ValueResolution => 320,
        TransformPassKind::StaticVarSubstitution => 330,
        TransformPassKind::TreeShakeClass => 340,
        TransformPassKind::TreeShakeKeyframes => 350,
        TransformPassKind::TreeShakeValue => 360,
        TransformPassKind::TreeShakeCustomProperty => 370,
        TransformPassKind::DeadMediaBranchRemoval => 380,
        TransformPassKind::DeadSupportsBranchRemoval => 390,
        TransformPassKind::DesignTokenRouting => 400,
        TransformPassKind::PrintCss => 410,
        TransformPassKind::RelativeColorLowering => 420,
        TransformPassKind::ContainerStaticEval => 430,
        TransformPassKind::NativeCssStaticEval => 440,
    }
}

fn transform_pass_descriptor_with_edges(
    kind: TransformPassKind,
    dag_edges: &[TransformDagEdgeV0],
) -> TransformPassDescriptorV0 {
    TransformPassDescriptorV0 {
        schema_version: "0",
        product: "omena-transform-cst.pass-descriptor",
        id: kind.id(),
        kind,
        pass_class: transform_pass_class(kind),
        phase: transform_pass_execution_phase(kind),
        phase_order: transform_pass_phase_order(kind),
        depends_on: dag_edges
            .iter()
            .filter(|edge| edge.to == kind.id())
            .map(|edge| edge.from)
            .collect(),
        conflicts_with: transform_pass_conflicts_with(kind, dag_edges),
    }
}

fn transform_pass_conflicts_with(
    kind: TransformPassKind,
    dag_edges: &[TransformDagEdgeV0],
) -> Vec<&'static str> {
    let Some(conflict_family) = transform_pass_conflict_family(kind) else {
        return Vec::new();
    };
    all_transform_pass_kinds()
        .into_iter()
        .filter(|other| *other != kind)
        .filter(|other| transform_pass_conflict_family(*other) == Some(conflict_family))
        .filter(|other| {
            transform_pass_execution_phase(*other) == transform_pass_execution_phase(kind)
        })
        .filter(|other| !dag_path_exists(kind.id(), other.id(), dag_edges))
        .filter(|other| !dag_path_exists(other.id(), kind.id(), dag_edges))
        .map(|other| other.id())
        .collect()
}

fn transform_pass_conflict_family(kind: TransformPassKind) -> Option<&'static str> {
    match kind {
        TransformPassKind::ColorMixLowering | TransformPassKind::ColorFunctionLowering => {
            Some("nested-color-function-lowering")
        }
        _ => None,
    }
}

fn dag_path_exists(from: &'static str, to: &'static str, dag_edges: &[TransformDagEdgeV0]) -> bool {
    let mut stack = vec![from];
    let mut visited = Vec::new();
    while let Some(current) = stack.pop() {
        if current == to {
            return true;
        }
        if visited.contains(&current) {
            continue;
        }
        visited.push(current);
        for edge in dag_edges.iter().filter(|edge| edge.from == current) {
            stack.push(edge.to);
        }
    }
    false
}

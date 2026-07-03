use serde::Serialize;

use super::{
    TransformDagEdgeV0, TransformPassKind, all_transform_pass_kinds, default_transform_dag_edges,
    transform_pass_execution_phase,
};

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

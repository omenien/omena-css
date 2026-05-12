//! Target feature matrix planning for Omena CSS transforms.
//!
//! This crate owns the target-sensitive lowering decision boundary. It
//! intentionally models the feature matrix directly before adding heavier browserslist /
//! caniuse-lite ingestion, so target-driven transforms can be tested without
//! coupling the core transform graph to external data formats.

use omena_transform_cst::TransformPassKind;
use omena_transform_passes::{TransformPassPlanV0, plan_transform_passes};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetFeatureSupportV0 {
    pub vendor_prefix_required: bool,
    pub supports_light_dark: bool,
    pub supports_color_mix: bool,
    pub supports_oklch_oklab: bool,
    pub supports_color_function: bool,
    pub supports_logical_properties: bool,
    pub supports_css_nesting: bool,
    pub supports_css_scope: bool,
    pub supports_cascade_layers: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetTransformOptionsV0 {
    pub allow_logical_to_physical: bool,
    pub allow_scope_flatten: bool,
    pub allow_layer_flatten: bool,
    pub enable_supports_static_eval: bool,
    pub enable_media_static_eval: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformTargetBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub managed_pass_ids: Vec<&'static str>,
    pub opt_in_pass_ids: Vec<&'static str>,
    pub target_data_source: &'static str,
    pub planner_surface: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformTargetPlanV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub target_label: String,
    pub required_pass_ids: Vec<&'static str>,
    pub blocked_pass_ids: Vec<&'static str>,
    pub planned_pass_ids: Vec<&'static str>,
    pub pass_plan: TransformPassPlanV0,
}

pub fn summarize_omena_transform_target_boundary() -> TransformTargetBoundarySummaryV0 {
    TransformTargetBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-transform-target.boundary",
        managed_pass_ids: target_managed_passes()
            .iter()
            .map(|pass| pass.id())
            .collect(),
        opt_in_pass_ids: vec![
            TransformPassKind::LogicalToPhysical.id(),
            TransformPassKind::ScopeFlatten.id(),
            TransformPassKind::LayerFlatten.id(),
        ],
        target_data_source: "explicitFeatureMatrixV0",
        planner_surface: "omena-transform-passes.plan",
    }
}

pub fn plan_target_transforms(
    target_label: impl Into<String>,
    support: TargetFeatureSupportV0,
    options: TargetTransformOptionsV0,
) -> TransformTargetPlanV0 {
    let target_label = target_label.into();
    let mut required_passes = Vec::new();
    let mut blocked_passes = Vec::new();

    if support.vendor_prefix_required {
        required_passes.push(TransformPassKind::VendorPrefixing);
    }
    if !support.supports_light_dark {
        required_passes.push(TransformPassKind::LightDarkLowering);
    }
    if !support.supports_color_mix {
        required_passes.push(TransformPassKind::ColorMixLowering);
    }
    if !support.supports_oklch_oklab {
        required_passes.push(TransformPassKind::OklchOklabLowering);
    }
    if !support.supports_color_function {
        required_passes.push(TransformPassKind::ColorFunctionLowering);
    }
    if !support.supports_logical_properties {
        push_required_or_blocked(
            TransformPassKind::LogicalToPhysical,
            options.allow_logical_to_physical,
            &mut required_passes,
            &mut blocked_passes,
        );
    }
    if !support.supports_css_nesting {
        required_passes.push(TransformPassKind::NestingUnwrap);
    }
    if !support.supports_css_scope {
        push_required_or_blocked(
            TransformPassKind::ScopeFlatten,
            options.allow_scope_flatten,
            &mut required_passes,
            &mut blocked_passes,
        );
    }
    if !support.supports_cascade_layers {
        push_required_or_blocked(
            TransformPassKind::LayerFlatten,
            options.allow_layer_flatten,
            &mut required_passes,
            &mut blocked_passes,
        );
    }
    if options.enable_supports_static_eval {
        required_passes.push(TransformPassKind::SupportsStaticEval);
    }
    if options.enable_media_static_eval {
        required_passes.push(TransformPassKind::MediaStaticEval);
    }

    required_passes.sort_by_key(|pass| pass.ordinal());
    required_passes.dedup();
    blocked_passes.sort_by_key(|pass| pass.ordinal());
    blocked_passes.dedup();

    let pass_plan = plan_transform_passes(&required_passes);
    let planned_pass_ids = pass_plan.ordered_pass_ids.clone();

    TransformTargetPlanV0 {
        schema_version: "0",
        product: "omena-transform-target.plan",
        target_label,
        required_pass_ids: required_passes.iter().map(|pass| pass.id()).collect(),
        blocked_pass_ids: blocked_passes.iter().map(|pass| pass.id()).collect(),
        planned_pass_ids,
        pass_plan,
    }
}

pub const fn modern_feature_support() -> TargetFeatureSupportV0 {
    TargetFeatureSupportV0 {
        vendor_prefix_required: false,
        supports_light_dark: true,
        supports_color_mix: true,
        supports_oklch_oklab: true,
        supports_color_function: true,
        supports_logical_properties: true,
        supports_css_nesting: true,
        supports_css_scope: true,
        supports_cascade_layers: true,
    }
}

pub const fn conservative_target_options() -> TargetTransformOptionsV0 {
    TargetTransformOptionsV0 {
        allow_logical_to_physical: false,
        allow_scope_flatten: false,
        allow_layer_flatten: false,
        enable_supports_static_eval: false,
        enable_media_static_eval: false,
    }
}

fn push_required_or_blocked(
    pass: TransformPassKind,
    allowed: bool,
    required_passes: &mut Vec<TransformPassKind>,
    blocked_passes: &mut Vec<TransformPassKind>,
) {
    if allowed {
        required_passes.push(pass);
    } else {
        blocked_passes.push(pass);
    }
}

fn target_managed_passes() -> [TransformPassKind; 11] {
    [
        TransformPassKind::VendorPrefixing,
        TransformPassKind::LightDarkLowering,
        TransformPassKind::ColorMixLowering,
        TransformPassKind::OklchOklabLowering,
        TransformPassKind::ColorFunctionLowering,
        TransformPassKind::LogicalToPhysical,
        TransformPassKind::NestingUnwrap,
        TransformPassKind::ScopeFlatten,
        TransformPassKind::LayerFlatten,
        TransformPassKind::SupportsStaticEval,
        TransformPassKind::MediaStaticEval,
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        TargetFeatureSupportV0, TargetTransformOptionsV0, conservative_target_options,
        plan_target_transforms, summarize_omena_transform_target_boundary,
    };

    #[test]
    fn exposes_p14_to_p24_target_lowering_boundary() {
        let boundary = summarize_omena_transform_target_boundary();

        assert_eq!(boundary.product, "omena-transform-target.boundary");
        assert_eq!(boundary.managed_pass_ids.len(), 11);
        assert!(boundary.managed_pass_ids.contains(&"p14-vendor-prefixing"));
        assert!(boundary.managed_pass_ids.contains(&"p24-media-static-eval"));
        assert!(boundary.opt_in_pass_ids.contains(&"p21-scope-flatten"));
    }

    #[test]
    fn plans_target_lowering_with_vendor_prefix_after_lowering_edges() {
        let support = TargetFeatureSupportV0 {
            vendor_prefix_required: true,
            supports_light_dark: false,
            supports_color_mix: false,
            supports_oklch_oklab: true,
            supports_color_function: true,
            supports_logical_properties: true,
            supports_css_nesting: false,
            supports_css_scope: false,
            supports_cascade_layers: false,
        };
        let options = TargetTransformOptionsV0 {
            allow_logical_to_physical: false,
            allow_scope_flatten: true,
            allow_layer_flatten: true,
            enable_supports_static_eval: true,
            enable_media_static_eval: true,
        };

        let plan = plan_target_transforms("legacy-webview", support, options);

        assert_eq!(plan.pass_plan.violated_dag_edge_count, 0);
        assert!(plan.required_pass_ids.contains(&"p15-light-dark-lowering"));
        assert!(plan.required_pass_ids.contains(&"p16-color-mix-lowering"));
        assert!(plan.required_pass_ids.contains(&"p14-vendor-prefixing"));
        assert!(plan.required_pass_ids.contains(&"p21-scope-flatten"));
        assert!(plan.required_pass_ids.contains(&"p22-layer-flatten"));
        let vendor_index = plan
            .planned_pass_ids
            .iter()
            .position(|id| *id == "p14-vendor-prefixing");
        let light_dark_index = plan
            .planned_pass_ids
            .iter()
            .position(|id| *id == "p15-light-dark-lowering");
        assert!(light_dark_index < vendor_index);
    }

    #[test]
    fn blocks_opt_in_flattening_when_not_explicitly_enabled() {
        let support = TargetFeatureSupportV0 {
            supports_css_scope: false,
            supports_cascade_layers: false,
            ..super::modern_feature_support()
        };

        let plan = plan_target_transforms(
            "modern-without-scope",
            support,
            conservative_target_options(),
        );

        assert!(plan.blocked_pass_ids.contains(&"p21-scope-flatten"));
        assert!(plan.blocked_pass_ids.contains(&"p22-layer-flatten"));
        assert!(!plan.required_pass_ids.contains(&"p21-scope-flatten"));
        assert!(!plan.required_pass_ids.contains(&"p22-layer-flatten"));
    }
}

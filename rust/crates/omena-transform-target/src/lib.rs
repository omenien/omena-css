//! Target feature matrix planning for Omena CSS transforms.
//!
//! This crate owns the target-sensitive lowering decision boundary. It
//! resolves standard browserslist queries through an embedded Can I Use snapshot,
//! then folds the resolved browser set into the explicit Omena transform feature
//! matrix. Named profiles stay available for product defaults and conservative
//! non-browser environments.

use std::collections::BTreeSet;

use browserslist::{Distrib, Opts, resolve as resolve_browserslist};
use omena_transform_cst::TransformPassKind;
use omena_transform_passes::{TransformPassPlanV0, plan_transform_passes};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct TargetTransformOptionsV0 {
    pub allow_logical_to_physical: bool,
    pub allow_scope_flatten: bool,
    pub allow_layer_flatten: bool,
    pub enable_supports_static_eval: bool,
    pub enable_media_static_eval: bool,
}

impl Default for TargetTransformOptionsV0 {
    fn default() -> Self {
        conservative_target_options()
    }
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
pub struct TransformTargetQueryPlanV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub query: String,
    pub normalized_query: String,
    pub profile_id: &'static str,
    pub recognized_profile: bool,
    pub target_data_source: &'static str,
    pub resolved_targets: Vec<String>,
    pub resolution_error: Option<String>,
    pub support: TargetFeatureSupportV0,
    pub transform_plan: TransformTargetPlanV0,
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
        target_data_source: "oxcBrowserslistV3+staticTargetProfileV0+explicitFeatureMatrixV0",
        planner_surface: "omena-transform-passes.plan",
    }
}

pub fn plan_target_transforms_from_query(
    query: impl Into<String>,
    options: TargetTransformOptionsV0,
) -> TransformTargetQueryPlanV0 {
    let query = query.into();
    let normalized_query = normalize_target_query(&query);
    let target_resolution = target_feature_support_for_query(&normalized_query);
    let transform_plan = plan_target_transforms(
        target_resolution.profile_id,
        target_resolution.support,
        options,
    );

    TransformTargetQueryPlanV0 {
        schema_version: "0",
        product: "omena-transform-target.query-plan",
        query,
        normalized_query,
        profile_id: target_resolution.profile_id,
        recognized_profile: target_resolution.recognized_profile,
        target_data_source: target_resolution.target_data_source,
        resolved_targets: target_resolution.resolved_targets,
        resolution_error: target_resolution.resolution_error,
        support: target_resolution.support,
        transform_plan,
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

pub const fn legacy_webview_feature_support() -> TargetFeatureSupportV0 {
    TargetFeatureSupportV0 {
        vendor_prefix_required: true,
        supports_light_dark: false,
        supports_color_mix: false,
        supports_oklch_oklab: false,
        supports_color_function: false,
        supports_logical_properties: false,
        supports_css_nesting: false,
        supports_css_scope: false,
        supports_cascade_layers: false,
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

struct TargetQueryResolutionV0 {
    profile_id: &'static str,
    recognized_profile: bool,
    target_data_source: &'static str,
    resolved_targets: Vec<String>,
    resolution_error: Option<String>,
    support: TargetFeatureSupportV0,
}

fn normalize_target_query(query: &str) -> String {
    query
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_ascii_lowercase()
}

fn target_feature_support_for_query(normalized_query: &str) -> TargetQueryResolutionV0 {
    if matches!(
        normalized_query,
        "" | "modern" | "baseline 2024" | "baseline-2024"
    ) {
        return TargetQueryResolutionV0 {
            profile_id: "modern-evergreen",
            recognized_profile: true,
            target_data_source: "staticTargetProfileV0",
            resolved_targets: Vec::new(),
            resolution_error: None,
            support: modern_feature_support(),
        };
    }

    if normalized_query == "legacy" || normalized_query == "legacy-webview" {
        return TargetQueryResolutionV0 {
            profile_id: "legacy-webview",
            recognized_profile: true,
            target_data_source: "staticTargetProfileV0",
            resolved_targets: Vec::new(),
            resolution_error: None,
            support: legacy_webview_feature_support(),
        };
    }

    match resolve_browserslist(&[normalized_query], &Opts::default()) {
        Ok(distribs) if !distribs.is_empty() => {
            let resolved_targets = distribs.iter().map(distrib_key).collect::<Vec<_>>();
            TargetQueryResolutionV0 {
                profile_id: "browserslist-resolved",
                recognized_profile: true,
                target_data_source: "oxcBrowserslistV3+featureSubsetV0",
                support: feature_support_for_resolved_targets(&distribs),
                resolved_targets,
                resolution_error: None,
            }
        }
        Ok(_) => unknown_conservative_resolution(None),
        Err(error) => unknown_conservative_resolution(Some(error.to_string())),
    }
}

fn unknown_conservative_resolution(resolution_error: Option<String>) -> TargetQueryResolutionV0 {
    TargetQueryResolutionV0 {
        profile_id: "unknown-conservative",
        recognized_profile: false,
        target_data_source: "staticTargetProfileV0",
        resolved_targets: Vec::new(),
        resolution_error,
        support: legacy_webview_feature_support(),
    }
}

fn feature_support_for_resolved_targets(distribs: &[Distrib]) -> TargetFeatureSupportV0 {
    let flexbox_fully_supported =
        target_set_is_subset_of_fully_supported_feature(distribs, "flexbox");
    let sticky_fully_supported =
        target_set_is_subset_of_fully_supported_feature(distribs, "css-sticky");

    TargetFeatureSupportV0 {
        vendor_prefix_required: !(flexbox_fully_supported && sticky_fully_supported),
        supports_light_dark: false,
        supports_color_mix: false,
        supports_oklch_oklab: target_set_is_subset_of_fully_supported_feature(
            distribs,
            "css-lch-lab",
        ),
        supports_color_function: target_set_is_subset_of_fully_supported_feature(
            distribs,
            "css-color-function",
        ),
        supports_logical_properties: target_set_is_subset_of_fully_supported_feature(
            distribs,
            "css-logical-props",
        ),
        supports_css_nesting: target_set_is_subset_of_fully_supported_feature(
            distribs,
            "css-nesting",
        ),
        supports_css_scope: target_set_is_subset_of_fully_supported_feature(
            distribs,
            "css-cascade-scope",
        ),
        supports_cascade_layers: target_set_is_subset_of_fully_supported_feature(
            distribs,
            "css-cascade-layers",
        ),
    }
}

fn target_set_is_subset_of_fully_supported_feature(distribs: &[Distrib], feature: &str) -> bool {
    let query = format!("fully supports {feature}");
    let Ok(feature_distribs) = resolve_browserslist(&[query.as_str()], &Opts::default()) else {
        return false;
    };
    if feature_distribs.is_empty() {
        return false;
    }

    let feature_targets = feature_distribs
        .iter()
        .map(distrib_key)
        .collect::<BTreeSet<_>>();
    distribs
        .iter()
        .all(|distrib| feature_targets.contains(&distrib_key(distrib)))
}

fn distrib_key(distrib: &Distrib) -> String {
    distrib.to_string()
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
        plan_target_transforms, plan_target_transforms_from_query,
        summarize_omena_transform_target_boundary,
    };

    #[test]
    fn exposes_target_lowering_boundary() {
        let boundary = summarize_omena_transform_target_boundary();

        assert_eq!(boundary.product, "omena-transform-target.boundary");
        assert_eq!(boundary.managed_pass_ids.len(), 11);
        assert_eq!(
            boundary.target_data_source,
            "oxcBrowserslistV3+staticTargetProfileV0+explicitFeatureMatrixV0"
        );
        assert!(boundary.managed_pass_ids.contains(&"vendor-prefixing"));
        assert!(boundary.managed_pass_ids.contains(&"media-static-eval"));
        assert!(boundary.opt_in_pass_ids.contains(&"scope-flatten"));
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
        assert!(plan.required_pass_ids.contains(&"light-dark-lowering"));
        assert!(plan.required_pass_ids.contains(&"color-mix-lowering"));
        assert!(plan.required_pass_ids.contains(&"vendor-prefixing"));
        assert!(plan.required_pass_ids.contains(&"scope-flatten"));
        assert!(plan.required_pass_ids.contains(&"layer-flatten"));
        let vendor_index = plan
            .planned_pass_ids
            .iter()
            .position(|id| *id == "vendor-prefixing");
        let light_dark_index = plan
            .planned_pass_ids
            .iter()
            .position(|id| *id == "light-dark-lowering");
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

        assert!(plan.blocked_pass_ids.contains(&"scope-flatten"));
        assert!(plan.blocked_pass_ids.contains(&"layer-flatten"));
        assert!(!plan.required_pass_ids.contains(&"scope-flatten"));
        assert!(!plan.required_pass_ids.contains(&"layer-flatten"));
    }

    #[test]
    fn plans_target_lowering_from_static_target_query_profiles() {
        let options = TargetTransformOptionsV0 {
            allow_logical_to_physical: true,
            allow_scope_flatten: true,
            allow_layer_flatten: true,
            enable_supports_static_eval: true,
            enable_media_static_eval: true,
        };
        let plan = plan_target_transforms_from_query("legacy-webview", options);

        assert!(plan.recognized_profile);
        assert_eq!(plan.normalized_query, "legacy-webview");
        assert_eq!(plan.profile_id, "legacy-webview");
        assert!(plan.support.vendor_prefix_required);
        assert_eq!(plan.transform_plan.pass_plan.violated_dag_edge_count, 0);
        assert!(
            plan.transform_plan
                .required_pass_ids
                .contains(&"vendor-prefixing")
        );
        assert!(
            plan.transform_plan
                .required_pass_ids
                .contains(&"nesting-unwrap")
        );
        assert!(
            plan.transform_plan
                .required_pass_ids
                .contains(&"logical-to-physical")
        );

        let modern = plan_target_transforms_from_query("modern", conservative_target_options());
        assert!(modern.recognized_profile);
        assert_eq!(modern.profile_id, "modern-evergreen");
        assert_eq!(modern.target_data_source, "staticTargetProfileV0");
        assert!(modern.transform_plan.required_pass_ids.is_empty());
    }

    #[test]
    fn plans_target_lowering_from_resolved_browserslist_query() {
        let options = TargetTransformOptionsV0 {
            allow_logical_to_physical: true,
            allow_scope_flatten: true,
            allow_layer_flatten: true,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
        };
        let plan = plan_target_transforms_from_query("ie 11", options);

        assert!(plan.recognized_profile);
        assert_eq!(plan.profile_id, "browserslist-resolved");
        assert_eq!(plan.target_data_source, "oxcBrowserslistV3+featureSubsetV0");
        assert_eq!(plan.resolved_targets, vec!["ie 11"]);
        assert_eq!(plan.resolution_error, None);
        assert!(plan.support.vendor_prefix_required);
        assert!(
            plan.transform_plan
                .required_pass_ids
                .contains(&"vendor-prefixing")
        );
        assert!(
            plan.transform_plan
                .required_pass_ids
                .contains(&"nesting-unwrap")
        );
        assert!(
            plan.transform_plan
                .required_pass_ids
                .contains(&"logical-to-physical")
        );
    }

    #[test]
    fn invalid_target_query_uses_conservative_profile_without_claiming_recognition() {
        let plan = plan_target_transforms_from_query("yuru 1.0", conservative_target_options());

        assert!(!plan.recognized_profile);
        assert_eq!(plan.profile_id, "unknown-conservative");
        assert!(plan.resolution_error.is_some());
        assert!(plan.support.vendor_prefix_required);
        assert!(
            plan.transform_plan
                .required_pass_ids
                .contains(&"vendor-prefixing")
        );
        assert!(
            plan.transform_plan
                .blocked_pass_ids
                .contains(&"scope-flatten")
        );
    }
}

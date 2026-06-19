//! Target feature matrix planning for Omena CSS transforms.
//!
//! This crate owns the target-sensitive lowering decision boundary. It
//! resolves standard browserslist queries through an embedded Can I Use snapshot,
//! then folds the resolved browser set into the explicit Omena transform feature
//! matrix. Named profiles stay available for product defaults and conservative
//! non-browser environments.

use std::{collections::BTreeSet, sync::OnceLock};

use browserslist::{Distrib, Opts, resolve as resolve_browserslist};
use omena_transform_cst::TransformPassKind;
use omena_transform_passes::{TransformPassPlanV0, plan_transform_passes};
use serde::{Deserialize, Serialize};

const BROWSER_THRESHOLDS_SOURCE: &str = include_str!("../data/browser-thresholds.toml");
const PASS_FEATURE_BINDINGS_SOURCE: &str = include_str!("../data/pass-feature-bindings.toml");
const TARGET_DATA_CONTRACT_ID: &str = "omena-transform-target-data-v0";
const TARGET_DATA_SOURCE_FILES: &[&str] = &[
    "data/compat-feature-selections.json",
    "data/browser-thresholds.toml",
    "data/pass-feature-bindings.toml",
];
const VENDOR_PREFIX_MATRIX_SOURCE: &str = "conservativeHandMaintainedMatrixV0";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct BrowserThresholdDataV0 {
    schema_version: String,
    product: String,
    refreshed_at: String,
    quorum_min_sources: usize,
    thresholds: Vec<BrowserFeatureThresholdV0>,
    parse_error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct BrowserFeatureThresholdV0 {
    table: String,
    browser: String,
    min_major: u16,
    min_minor: u16,
    caniuse_key: String,
    source_quorum: Vec<String>,
    last_verified: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct PassFeatureBindingDataV0 {
    schema_version: String,
    product: String,
    refreshed_at: String,
    bindings: Vec<PassFeatureBindingV0>,
    parse_error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct PassFeatureBindingV0 {
    pass_id: String,
    caniuse_keys: Vec<String>,
    support_table: String,
}

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
    pub drop_dark_mode_media_queries: bool,
}

impl Default for TargetTransformOptionsV0 {
    fn default() -> Self {
        conservative_target_options()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformTargetDataContractV0 {
    pub product: &'static str,
    pub contract_id: &'static str,
    pub snapshot_id: String,
    pub browser_threshold_schema_version: String,
    pub pass_feature_binding_schema_version: String,
    pub browser_threshold_refreshed_at: String,
    pub pass_feature_binding_refreshed_at: String,
    pub source_files: Vec<&'static str>,
    pub parse_error_count: usize,
    pub quorum_min_sources: usize,
    pub browser_threshold_table_count: usize,
    pub browser_threshold_entry_count: usize,
    pub pass_feature_binding_count: usize,
    pub stale_entry_count: usize,
    pub unmapped_threshold_table_count: usize,
    pub quorum_valid: bool,
    pub bindings_valid: bool,
    pub valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformTargetDataEvidenceV0 {
    pub product: &'static str,
    pub pass_id: String,
    pub support_table: String,
    pub caniuse_keys: Vec<String>,
    pub source_quorum: Vec<String>,
    pub last_verified: Vec<String>,
    pub resolved_targets: Vec<TransformTargetResolvedTargetEvidenceV0>,
    pub all_resolved_targets_supported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformTargetResolvedTargetEvidenceV0 {
    pub browser: String,
    pub version: String,
    pub supported: bool,
    pub matched_threshold: Option<TransformTargetThresholdEvidenceV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformTargetThresholdEvidenceV0 {
    pub browser: String,
    pub min_version: String,
    pub caniuse_key: String,
    pub source_quorum: Vec<String>,
    pub last_verified: String,
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
    pub browser_threshold_table_count: usize,
    pub browser_threshold_entry_count: usize,
    pub pass_feature_binding_count: usize,
    pub browser_data_source_files: Vec<&'static str>,
    pub browser_data_parse_error_count: usize,
    pub browser_data_quorum_valid: bool,
    pub browser_data_bindings_valid: bool,
    pub target_data_contract: TransformTargetDataContractV0,
    pub vendor_prefix_matrix_source: &'static str,
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
    pub target_data_contract_id: &'static str,
    pub target_data_snapshot_id: String,
    pub target_data_evidence: Vec<TransformTargetDataEvidenceV0>,
    pub vendor_prefix_matrix_source: &'static str,
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
    let browser_data = browser_threshold_data();
    let bindings = pass_feature_binding_data();
    let target_data_contract = target_data_contract_summary(browser_data, bindings);

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
        target_data_source: "oxcBrowserslistV3+browserThresholdsTomlV0+staticTargetProfileV0+explicitFeatureMatrixV0",
        planner_surface: "omena-transform-passes.plan",
        browser_threshold_table_count: browser_threshold_table_count(browser_data),
        browser_threshold_entry_count: browser_data.thresholds.len(),
        pass_feature_binding_count: bindings.bindings.len(),
        browser_data_source_files: target_data_source_files(),
        browser_data_parse_error_count: browser_data.parse_error_count + bindings.parse_error_count,
        browser_data_quorum_valid: browser_threshold_data_is_valid(browser_data),
        browser_data_bindings_valid: pass_feature_binding_data_is_valid(browser_data, bindings),
        target_data_contract,
        vendor_prefix_matrix_source: VENDOR_PREFIX_MATRIX_SOURCE,
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
        target_data_contract_id: target_resolution.target_data_contract_id,
        target_data_snapshot_id: target_resolution.target_data_snapshot_id,
        target_data_evidence: target_resolution.target_data_evidence,
        vendor_prefix_matrix_source: target_resolution.vendor_prefix_matrix_source,
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
    if options.drop_dark_mode_media_queries {
        required_passes.push(TransformPassKind::DeadMediaBranchRemoval);
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
        drop_dark_mode_media_queries: false,
    }
}

struct TargetQueryResolutionV0 {
    profile_id: &'static str,
    recognized_profile: bool,
    target_data_source: &'static str,
    target_data_contract_id: &'static str,
    target_data_snapshot_id: String,
    target_data_evidence: Vec<TransformTargetDataEvidenceV0>,
    vendor_prefix_matrix_source: &'static str,
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
            target_data_contract_id: TARGET_DATA_CONTRACT_ID,
            target_data_snapshot_id: current_target_data_snapshot_id(),
            target_data_evidence: Vec::new(),
            vendor_prefix_matrix_source: VENDOR_PREFIX_MATRIX_SOURCE,
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
            target_data_contract_id: TARGET_DATA_CONTRACT_ID,
            target_data_snapshot_id: current_target_data_snapshot_id(),
            target_data_evidence: Vec::new(),
            vendor_prefix_matrix_source: VENDOR_PREFIX_MATRIX_SOURCE,
            resolved_targets: Vec::new(),
            resolution_error: None,
            support: legacy_webview_feature_support(),
        };
    }

    match resolve_browserslist(&[normalized_query], &Opts::default()) {
        Ok(distribs) if !distribs.is_empty() => {
            let resolved_targets = distribs.iter().map(distrib_key).collect::<Vec<_>>();
            let target_data_evidence = target_data_evidence_for_resolved_targets(&distribs);
            TargetQueryResolutionV0 {
                profile_id: "browserslist-resolved",
                recognized_profile: true,
                target_data_source: "oxcBrowserslistV3+browserThresholdsTomlV0+featureSubsetV0",
                target_data_contract_id: TARGET_DATA_CONTRACT_ID,
                target_data_snapshot_id: current_target_data_snapshot_id(),
                target_data_evidence,
                vendor_prefix_matrix_source: VENDOR_PREFIX_MATRIX_SOURCE,
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
        target_data_contract_id: TARGET_DATA_CONTRACT_ID,
        target_data_snapshot_id: current_target_data_snapshot_id(),
        target_data_evidence: Vec::new(),
        vendor_prefix_matrix_source: VENDOR_PREFIX_MATRIX_SOURCE,
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
        supports_light_dark: target_set_is_subset_of_browser_threshold_table(
            distribs,
            "light_dark",
        ),
        supports_color_mix: target_set_is_subset_of_browser_threshold_table(distribs, "color_mix"),
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

fn target_set_is_subset_of_browser_threshold_table(distribs: &[Distrib], table: &str) -> bool {
    !distribs.is_empty()
        && distribs
            .iter()
            .all(|distrib| distrib_matches_browser_threshold_table(distrib, table))
}

fn distrib_matches_browser_threshold_table(distrib: &Distrib, table: &str) -> bool {
    browser_threshold_data()
        .thresholds
        .iter()
        .filter(|threshold| threshold.table == table)
        .find(|threshold| threshold.browser == distrib.name())
        .is_some_and(|threshold| {
            browser_version_at_least(distrib.version(), threshold.min_major, threshold.min_minor)
        })
}

fn browser_threshold_data() -> &'static BrowserThresholdDataV0 {
    static DATA: OnceLock<BrowserThresholdDataV0> = OnceLock::new();
    DATA.get_or_init(|| {
        parse_browser_threshold_data(BROWSER_THRESHOLDS_SOURCE).unwrap_or_else(|_| {
            BrowserThresholdDataV0 {
                parse_error_count: 1,
                ..BrowserThresholdDataV0::default()
            }
        })
    })
}

fn pass_feature_binding_data() -> &'static PassFeatureBindingDataV0 {
    static DATA: OnceLock<PassFeatureBindingDataV0> = OnceLock::new();
    DATA.get_or_init(|| {
        parse_pass_feature_binding_data(PASS_FEATURE_BINDINGS_SOURCE).unwrap_or_else(|_| {
            PassFeatureBindingDataV0 {
                parse_error_count: 1,
                ..PassFeatureBindingDataV0::default()
            }
        })
    })
}

fn target_data_source_files() -> Vec<&'static str> {
    TARGET_DATA_SOURCE_FILES.to_vec()
}

fn current_target_data_snapshot_id() -> String {
    target_data_snapshot_id(browser_threshold_data(), pass_feature_binding_data())
}

fn target_data_snapshot_id(
    browser_data: &BrowserThresholdDataV0,
    bindings: &PassFeatureBindingDataV0,
) -> String {
    format!(
        "{TARGET_DATA_CONTRACT_ID}:thresholds-{}:bindings-{}",
        browser_data.refreshed_at, bindings.refreshed_at
    )
}

fn target_data_contract_summary(
    browser_data: &BrowserThresholdDataV0,
    bindings: &PassFeatureBindingDataV0,
) -> TransformTargetDataContractV0 {
    let parse_error_count = browser_data.parse_error_count + bindings.parse_error_count;
    let stale_entry_count = browser_threshold_stale_entry_count(browser_data);
    let unmapped_threshold_table_count = unmapped_threshold_table_count(browser_data, bindings);
    let quorum_valid = browser_threshold_data_is_valid(browser_data);
    let bindings_valid = pass_feature_binding_data_is_valid(browser_data, bindings);

    TransformTargetDataContractV0 {
        product: "omena-transform-target.data-contract",
        contract_id: TARGET_DATA_CONTRACT_ID,
        snapshot_id: target_data_snapshot_id(browser_data, bindings),
        browser_threshold_schema_version: browser_data.schema_version.clone(),
        pass_feature_binding_schema_version: bindings.schema_version.clone(),
        browser_threshold_refreshed_at: browser_data.refreshed_at.clone(),
        pass_feature_binding_refreshed_at: bindings.refreshed_at.clone(),
        source_files: target_data_source_files(),
        parse_error_count,
        quorum_min_sources: browser_data.quorum_min_sources,
        browser_threshold_table_count: browser_threshold_table_count(browser_data),
        browser_threshold_entry_count: browser_data.thresholds.len(),
        pass_feature_binding_count: bindings.bindings.len(),
        stale_entry_count,
        unmapped_threshold_table_count,
        quorum_valid,
        bindings_valid,
        valid: parse_error_count == 0
            && stale_entry_count == 0
            && unmapped_threshold_table_count == 0
            && quorum_valid
            && bindings_valid,
    }
}

fn target_data_evidence_for_resolved_targets(
    distribs: &[Distrib],
) -> Vec<TransformTargetDataEvidenceV0> {
    let browser_data = browser_threshold_data();
    pass_feature_binding_data()
        .bindings
        .iter()
        .map(|binding| {
            let resolved_targets = distribs
                .iter()
                .map(|distrib| {
                    resolved_target_evidence_for_table(
                        browser_data,
                        distrib,
                        &binding.support_table,
                    )
                })
                .collect::<Vec<_>>();
            let all_resolved_targets_supported = !resolved_targets.is_empty()
                && resolved_targets.iter().all(|target| target.supported);

            TransformTargetDataEvidenceV0 {
                product: "omena-transform-target.data-evidence",
                pass_id: binding.pass_id.clone(),
                support_table: binding.support_table.clone(),
                caniuse_keys: binding.caniuse_keys.clone(),
                source_quorum: source_quorum_for_table(browser_data, &binding.support_table),
                last_verified: last_verified_for_table(browser_data, &binding.support_table),
                resolved_targets,
                all_resolved_targets_supported,
            }
        })
        .collect()
}

fn resolved_target_evidence_for_table(
    browser_data: &BrowserThresholdDataV0,
    distrib: &Distrib,
    table: &str,
) -> TransformTargetResolvedTargetEvidenceV0 {
    let threshold = browser_threshold_for_table_and_browser(browser_data, table, distrib.name());
    let supported = threshold.is_some_and(|threshold| {
        browser_version_at_least(distrib.version(), threshold.min_major, threshold.min_minor)
    });

    TransformTargetResolvedTargetEvidenceV0 {
        browser: distrib.name().to_string(),
        version: distrib.version().to_string(),
        supported,
        matched_threshold: threshold.map(threshold_evidence),
    }
}

fn threshold_evidence(threshold: &BrowserFeatureThresholdV0) -> TransformTargetThresholdEvidenceV0 {
    TransformTargetThresholdEvidenceV0 {
        browser: threshold.browser.clone(),
        min_version: format!("{}.{}", threshold.min_major, threshold.min_minor),
        caniuse_key: threshold.caniuse_key.clone(),
        source_quorum: threshold.source_quorum.clone(),
        last_verified: threshold.last_verified.clone(),
    }
}

fn browser_threshold_for_table_and_browser<'a>(
    browser_data: &'a BrowserThresholdDataV0,
    table: &str,
    browser: &str,
) -> Option<&'a BrowserFeatureThresholdV0> {
    browser_data
        .thresholds
        .iter()
        .find(|threshold| threshold.table == table && threshold.browser == browser)
}

fn source_quorum_for_table(browser_data: &BrowserThresholdDataV0, table: &str) -> Vec<String> {
    let mut sources = Vec::new();
    for threshold in browser_data
        .thresholds
        .iter()
        .filter(|threshold| threshold.table == table)
    {
        push_unique_strings(&mut sources, threshold.source_quorum.iter());
    }
    sources
}

fn last_verified_for_table(browser_data: &BrowserThresholdDataV0, table: &str) -> Vec<String> {
    let mut dates = Vec::new();
    for threshold in browser_data
        .thresholds
        .iter()
        .filter(|threshold| threshold.table == table)
    {
        push_unique_strings(&mut dates, std::iter::once(&threshold.last_verified));
    }
    dates
}

fn push_unique_strings<'a>(output: &mut Vec<String>, values: impl IntoIterator<Item = &'a String>) {
    for value in values {
        if !output.iter().any(|existing| existing == value) {
            output.push(value.clone());
        }
    }
}

fn parse_browser_threshold_data(source: &str) -> Result<BrowserThresholdDataV0, String> {
    let mut data = BrowserThresholdDataV0::default();
    let mut current_threshold: Option<BrowserFeatureThresholdV0> = None;

    for line in significant_toml_lines(source) {
        if line == "[[threshold]]" {
            if let Some(threshold) = current_threshold.take() {
                data.thresholds.push(threshold);
            }
            current_threshold = Some(BrowserFeatureThresholdV0::default());
            continue;
        }

        let (key, value) = parse_toml_assignment(&line)?;
        if let Some(threshold) = current_threshold.as_mut() {
            match key {
                "table" => threshold.table = parse_toml_string(value)?,
                "browser" => threshold.browser = parse_toml_string(value)?,
                "min_major" => threshold.min_major = parse_toml_u16(value)?,
                "min_minor" => threshold.min_minor = parse_toml_u16(value)?,
                "caniuse_key" => threshold.caniuse_key = parse_toml_string(value)?,
                "source_quorum" => threshold.source_quorum = parse_toml_string_array(value)?,
                "last_verified" => threshold.last_verified = parse_toml_string(value)?,
                _ => return Err(format!("unknown browser threshold key `{key}`")),
            }
        } else {
            match key {
                "schema_version" => data.schema_version = parse_toml_string(value)?,
                "product" => data.product = parse_toml_string(value)?,
                "refreshed_at" => data.refreshed_at = parse_toml_string(value)?,
                "quorum_min_sources" => data.quorum_min_sources = parse_toml_usize(value)?,
                _ => return Err(format!("unknown browser threshold root key `{key}`")),
            }
        }
    }

    if let Some(threshold) = current_threshold.take() {
        data.thresholds.push(threshold);
    }

    Ok(data)
}

fn parse_pass_feature_binding_data(source: &str) -> Result<PassFeatureBindingDataV0, String> {
    let mut data = PassFeatureBindingDataV0::default();
    let mut current_binding: Option<PassFeatureBindingV0> = None;

    for line in significant_toml_lines(source) {
        if line == "[[binding]]" {
            if let Some(binding) = current_binding.take() {
                data.bindings.push(binding);
            }
            current_binding = Some(PassFeatureBindingV0::default());
            continue;
        }

        let (key, value) = parse_toml_assignment(&line)?;
        if let Some(binding) = current_binding.as_mut() {
            match key {
                "pass_id" => binding.pass_id = parse_toml_string(value)?,
                "caniuse_keys" => binding.caniuse_keys = parse_toml_string_array(value)?,
                "support_table" => binding.support_table = parse_toml_string(value)?,
                _ => return Err(format!("unknown pass feature binding key `{key}`")),
            }
        } else {
            match key {
                "schema_version" => data.schema_version = parse_toml_string(value)?,
                "product" => data.product = parse_toml_string(value)?,
                "refreshed_at" => data.refreshed_at = parse_toml_string(value)?,
                _ => return Err(format!("unknown pass feature binding root key `{key}`")),
            }
        }
    }

    if let Some(binding) = current_binding.take() {
        data.bindings.push(binding);
    }

    Ok(data)
}

fn significant_toml_lines(source: &str) -> impl Iterator<Item = String> + '_ {
    source
        .lines()
        .map(|line| line.split('#').next().unwrap_or("").trim())
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

fn parse_toml_assignment(line: &str) -> Result<(&str, &str), String> {
    let Some((key, value)) = line.split_once('=') else {
        return Err(format!("invalid assignment `{line}`"));
    };
    Ok((key.trim(), value.trim()))
}

fn parse_toml_string(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
        return Err(format!("expected quoted string, got `{value}`"));
    }
    Ok(value[1..value.len() - 1].to_string())
}

fn parse_toml_u16(value: &str) -> Result<u16, String> {
    value
        .trim()
        .parse::<u16>()
        .map_err(|error| format!("invalid u16 `{value}`: {error}"))
}

fn parse_toml_usize(value: &str) -> Result<usize, String> {
    value
        .trim()
        .parse::<usize>()
        .map_err(|error| format!("invalid usize `{value}`: {error}"))
}

fn parse_toml_string_array(value: &str) -> Result<Vec<String>, String> {
    let value = value.trim();
    if value.len() < 2 || !value.starts_with('[') || !value.ends_with(']') {
        return Err(format!("expected string array, got `{value}`"));
    }
    let body = value[1..value.len() - 1].trim();
    if body.is_empty() {
        return Ok(Vec::new());
    }

    body.split(',')
        .map(|item| parse_toml_string(item.trim()))
        .collect()
}

fn browser_threshold_data_is_valid(data: &BrowserThresholdDataV0) -> bool {
    data.parse_error_count == 0
        && data.schema_version == "0"
        && data.product == "omena-transform-target.browser-thresholds"
        && is_iso_date(&data.refreshed_at)
        && data.quorum_min_sources >= 2
        && browser_threshold_table_count(data) >= 2
        && browser_threshold_stale_entry_count(data) == 0
        && data.thresholds.iter().all(|threshold| {
            !threshold.table.is_empty()
                && !threshold.browser.is_empty()
                && !threshold.caniuse_key.is_empty()
                && threshold.source_quorum.len() >= data.quorum_min_sources
                && threshold
                    .source_quorum
                    .iter()
                    .all(|source| matches!(source.as_str(), "caniuse" | "web-features" | "mdn-bcd"))
                && is_iso_date(&threshold.last_verified)
        })
}

fn pass_feature_binding_data_is_valid(
    browser_data: &BrowserThresholdDataV0,
    binding_data: &PassFeatureBindingDataV0,
) -> bool {
    binding_data.parse_error_count == 0
        && binding_data.schema_version == "0"
        && binding_data.product == "omena-transform-target.pass-feature-bindings"
        && is_iso_date(&binding_data.refreshed_at)
        && !binding_data.bindings.is_empty()
        && unmapped_threshold_table_count(browser_data, binding_data) == 0
        && binding_data.bindings.iter().all(|binding| {
            !binding.pass_id.is_empty()
                && target_managed_passes()
                    .iter()
                    .any(|pass| pass.id() == binding.pass_id)
                && !binding.caniuse_keys.is_empty()
                && binding.caniuse_keys.iter().all(|key| {
                    browser_data.thresholds.iter().any(|threshold| {
                        threshold.table == binding.support_table && threshold.caniuse_key == *key
                    })
                })
        })
}

fn browser_threshold_stale_entry_count(data: &BrowserThresholdDataV0) -> usize {
    data.thresholds
        .iter()
        .filter(|threshold| threshold.last_verified != data.refreshed_at)
        .count()
}

fn unmapped_threshold_table_count(
    browser_data: &BrowserThresholdDataV0,
    binding_data: &PassFeatureBindingDataV0,
) -> usize {
    let bound_tables = binding_data
        .bindings
        .iter()
        .map(|binding| binding.support_table.as_str())
        .collect::<BTreeSet<_>>();
    browser_data
        .thresholds
        .iter()
        .map(|threshold| threshold.table.as_str())
        .collect::<BTreeSet<_>>()
        .difference(&bound_tables)
        .count()
}

fn browser_threshold_table_count(data: &BrowserThresholdDataV0) -> usize {
    data.thresholds
        .iter()
        .map(|threshold| threshold.table.as_str())
        .collect::<BTreeSet<_>>()
        .len()
}

fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
}

fn browser_version_at_least(version: &str, min_major: u16, min_minor: u16) -> bool {
    if version.eq_ignore_ascii_case("tp") {
        return true;
    }

    let version = version.split('-').next().unwrap_or(version);
    let mut parts = version.split('.');
    let Some(major) = parts.next().and_then(|part| part.parse::<u16>().ok()) else {
        return false;
    };
    let minor = parts
        .next()
        .and_then(|part| part.parse::<u16>().ok())
        .unwrap_or(0);

    major > min_major || (major == min_major && minor >= min_minor)
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

fn target_managed_passes() -> [TransformPassKind; 12] {
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
        TransformPassKind::DeadMediaBranchRemoval,
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
        assert_eq!(boundary.managed_pass_ids.len(), 12);
        assert_eq!(
            boundary.target_data_source,
            "oxcBrowserslistV3+browserThresholdsTomlV0+staticTargetProfileV0+explicitFeatureMatrixV0"
        );
        assert_eq!(boundary.browser_threshold_table_count, 2);
        assert_eq!(boundary.browser_threshold_entry_count, 22);
        assert_eq!(boundary.pass_feature_binding_count, 2);
        assert_eq!(boundary.browser_data_parse_error_count, 0);
        assert!(boundary.browser_data_quorum_valid);
        assert!(boundary.browser_data_bindings_valid);
        assert_eq!(
            boundary.target_data_contract.contract_id,
            super::TARGET_DATA_CONTRACT_ID
        );
        assert_eq!(
            boundary.target_data_contract.snapshot_id,
            "omena-transform-target-data-v0:thresholds-2026-05-22:bindings-2026-05-22"
        );
        assert!(boundary.target_data_contract.valid);
        assert_eq!(boundary.target_data_contract.stale_entry_count, 0);
        assert_eq!(
            boundary.target_data_contract.unmapped_threshold_table_count,
            0
        );
        assert_eq!(
            boundary.vendor_prefix_matrix_source,
            super::VENDOR_PREFIX_MATRIX_SOURCE
        );
        assert!(boundary.managed_pass_ids.contains(&"vendor-prefixing"));
        assert!(boundary.managed_pass_ids.contains(&"media-static-eval"));
        assert!(boundary.opt_in_pass_ids.contains(&"scope-flatten"));
    }

    #[test]
    fn browser_data_governance_externalizes_thresholds_and_bindings() {
        let boundary = summarize_omena_transform_target_boundary();

        assert!(
            boundary
                .browser_data_source_files
                .contains(&"data/browser-thresholds.toml")
        );
        assert!(
            boundary
                .browser_data_source_files
                .contains(&"data/pass-feature-bindings.toml")
        );
        assert!(boundary.browser_data_quorum_valid);
        assert!(boundary.browser_data_bindings_valid);
        assert_eq!(
            boundary.target_data_contract.source_files,
            boundary.browser_data_source_files
        );
        assert_eq!(
            boundary
                .target_data_contract
                .browser_threshold_schema_version,
            "0"
        );
        assert_eq!(
            boundary
                .target_data_contract
                .pass_feature_binding_schema_version,
            "0"
        );
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
            drop_dark_mode_media_queries: false,
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
    fn plans_dark_mode_media_drop_as_dead_media_branch_pass() {
        let mut options = conservative_target_options();
        options.drop_dark_mode_media_queries = true;

        let plan = plan_target_transforms("modern", super::modern_feature_support(), options);

        assert!(
            plan.required_pass_ids
                .contains(&"dead-media-branch-removal")
        );
        assert!(plan.planned_pass_ids.contains(&"dead-media-branch-removal"));
    }

    #[test]
    fn plans_target_lowering_from_static_target_query_profiles() {
        let options = TargetTransformOptionsV0 {
            allow_logical_to_physical: true,
            allow_scope_flatten: true,
            allow_layer_flatten: true,
            enable_supports_static_eval: true,
            enable_media_static_eval: true,
            drop_dark_mode_media_queries: false,
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
        assert_eq!(
            modern.target_data_contract_id,
            super::TARGET_DATA_CONTRACT_ID
        );
        assert_eq!(
            modern.target_data_snapshot_id,
            "omena-transform-target-data-v0:thresholds-2026-05-22:bindings-2026-05-22"
        );
        assert!(modern.target_data_evidence.is_empty());
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
            drop_dark_mode_media_queries: false,
        };
        let plan = plan_target_transforms_from_query("ie 11", options);

        assert!(plan.recognized_profile);
        assert_eq!(plan.profile_id, "browserslist-resolved");
        assert_eq!(
            plan.target_data_source,
            "oxcBrowserslistV3+browserThresholdsTomlV0+featureSubsetV0"
        );
        assert_eq!(plan.resolved_targets, vec!["ie 11"]);
        assert_eq!(plan.resolution_error, None);
        assert_eq!(plan.target_data_contract_id, super::TARGET_DATA_CONTRACT_ID);
        assert_eq!(
            plan.target_data_snapshot_id,
            "omena-transform-target-data-v0:thresholds-2026-05-22:bindings-2026-05-22"
        );
        assert_eq!(plan.target_data_evidence.len(), 2);
        assert!(
            plan.target_data_evidence
                .iter()
                .any(|evidence| evidence.support_table == "light_dark"
                    && evidence.caniuse_keys == vec!["css-light-dark-function".to_string()]
                    && evidence.source_quorum
                        == vec![
                            "caniuse".to_string(),
                            "web-features".to_string(),
                            "mdn-bcd".to_string()
                        ]
                    && evidence.last_verified == vec!["2026-05-22".to_string()])
        );
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
    fn resolves_light_dark_and_color_mix_from_static_compatibility_matrix() {
        let chrome_122 =
            plan_target_transforms_from_query("chrome 122", conservative_target_options());
        assert_eq!(chrome_122.profile_id, "browserslist-resolved");
        assert!(!chrome_122.support.supports_light_dark);
        assert!(chrome_122.support.supports_color_mix);
        assert!(
            chrome_122
                .transform_plan
                .required_pass_ids
                .contains(&"light-dark-lowering")
        );
        assert!(
            !chrome_122
                .transform_plan
                .required_pass_ids
                .contains(&"color-mix-lowering")
        );

        let chrome_123 =
            plan_target_transforms_from_query("chrome 123", conservative_target_options());
        assert!(chrome_123.support.supports_light_dark);
        assert!(chrome_123.support.supports_color_mix);
        assert!(
            !chrome_123
                .transform_plan
                .required_pass_ids
                .contains(&"light-dark-lowering")
        );
        assert!(
            !chrome_123
                .transform_plan
                .required_pass_ids
                .contains(&"color-mix-lowering")
        );

        let safari_16_2 =
            plan_target_transforms_from_query("safari 16.2", conservative_target_options());
        assert!(!safari_16_2.support.supports_light_dark);
        assert!(safari_16_2.support.supports_color_mix);

        let safari_17_5 =
            plan_target_transforms_from_query("safari 17.5", conservative_target_options());
        assert!(safari_17_5.support.supports_light_dark);
        assert!(safari_17_5.support.supports_color_mix);
    }

    #[test]
    fn query_plan_evidence_explains_threshold_support_for_resolved_targets() {
        let chrome_122 =
            plan_target_transforms_from_query("chrome 122", conservative_target_options());

        let light_dark = chrome_122
            .target_data_evidence
            .iter()
            .find(|evidence| evidence.support_table == "light_dark");
        assert!(
            light_dark.is_some(),
            "light-dark target data evidence should be present"
        );
        let Some(light_dark) = light_dark else {
            return;
        };
        assert_eq!(light_dark.pass_id, "light-dark-lowering");
        assert_eq!(
            light_dark.caniuse_keys,
            vec!["css-light-dark-function".to_string()]
        );
        assert_eq!(
            light_dark.source_quorum,
            vec![
                "caniuse".to_string(),
                "web-features".to_string(),
                "mdn-bcd".to_string()
            ]
        );
        assert_eq!(light_dark.last_verified, vec!["2026-05-22".to_string()]);
        assert!(!light_dark.all_resolved_targets_supported);

        let chrome_light_dark = light_dark
            .resolved_targets
            .iter()
            .find(|target| target.browser == "chrome");
        assert!(
            chrome_light_dark.is_some(),
            "chrome target evidence should be present"
        );
        let Some(chrome_light_dark) = chrome_light_dark else {
            return;
        };
        assert_eq!(chrome_light_dark.version, "122");
        assert!(!chrome_light_dark.supported);
        let light_dark_threshold = chrome_light_dark.matched_threshold.as_ref();
        assert!(
            light_dark_threshold.is_some(),
            "chrome light-dark threshold should be present"
        );
        let Some(light_dark_threshold) = light_dark_threshold else {
            return;
        };
        assert_eq!(light_dark_threshold.min_version, "123.0");
        assert_eq!(light_dark_threshold.caniuse_key, "css-light-dark-function");

        let color_mix = chrome_122
            .target_data_evidence
            .iter()
            .find(|evidence| evidence.support_table == "color_mix");
        assert!(
            color_mix.is_some(),
            "color-mix target data evidence should be present"
        );
        let Some(color_mix) = color_mix else {
            return;
        };
        assert!(color_mix.all_resolved_targets_supported);
        let chrome_color_mix = color_mix
            .resolved_targets
            .iter()
            .find(|target| target.browser == "chrome");
        assert!(
            chrome_color_mix.is_some(),
            "chrome target evidence should be present"
        );
        let Some(chrome_color_mix) = chrome_color_mix else {
            return;
        };
        assert!(chrome_color_mix.supported);
        let color_mix_threshold = chrome_color_mix.matched_threshold.as_ref();
        assert!(
            color_mix_threshold.is_some(),
            "chrome color-mix threshold should be present"
        );
        let Some(color_mix_threshold) = color_mix_threshold else {
            return;
        };
        assert_eq!(color_mix_threshold.min_version, "111.0");
        assert_eq!(color_mix_threshold.caniuse_key, "css-color-mix");
    }

    #[test]
    fn resolved_multi_target_queries_fold_to_the_least_supported_feature_set() {
        let mixed_targets = plan_target_transforms_from_query(
            "chrome 123, safari 16.2",
            conservative_target_options(),
        );

        assert_eq!(mixed_targets.profile_id, "browserslist-resolved");
        assert_eq!(
            mixed_targets.resolved_targets,
            vec!["chrome 123", "safari 16.2"]
        );
        assert!(
            !mixed_targets.support.supports_light_dark,
            "safari 16.2 keeps the multi-target set below the light-dark threshold"
        );
        assert!(mixed_targets.support.supports_color_mix);
        assert!(
            mixed_targets
                .transform_plan
                .required_pass_ids
                .contains(&"light-dark-lowering")
        );
        assert!(
            !mixed_targets
                .transform_plan
                .required_pass_ids
                .contains(&"color-mix-lowering")
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

    #[test]
    fn browser_data_contract_rejects_stale_threshold_entries() {
        let mut browser_data = super::browser_threshold_data().clone();
        browser_data.thresholds[0].last_verified = "2026-05-01".to_string();

        assert_eq!(super::browser_threshold_stale_entry_count(&browser_data), 1);
        assert!(!super::browser_threshold_data_is_valid(&browser_data));

        let contract =
            super::target_data_contract_summary(&browser_data, super::pass_feature_binding_data());
        assert_eq!(contract.stale_entry_count, 1);
        assert!(!contract.valid);
    }

    #[test]
    fn browser_data_contract_rejects_unmapped_threshold_tables() {
        let mut browser_data = super::browser_threshold_data().clone();
        browser_data
            .thresholds
            .push(super::BrowserFeatureThresholdV0 {
                table: "unmapped_feature".to_string(),
                browser: "chrome".to_string(),
                min_major: 1,
                min_minor: 0,
                caniuse_key: "css-unmapped-feature".to_string(),
                source_quorum: vec![
                    "caniuse".to_string(),
                    "web-features".to_string(),
                    "mdn-bcd".to_string(),
                ],
                last_verified: browser_data.refreshed_at.clone(),
            });
        let bindings = super::pass_feature_binding_data().clone();

        assert_eq!(
            super::unmapped_threshold_table_count(&browser_data, &bindings),
            1
        );
        assert!(super::browser_threshold_data_is_valid(&browser_data));
        assert!(!super::pass_feature_binding_data_is_valid(
            &browser_data,
            &bindings
        ));

        let contract = super::target_data_contract_summary(&browser_data, &bindings);
        assert_eq!(contract.unmapped_threshold_table_count, 1);
        assert!(!contract.valid);
    }
}

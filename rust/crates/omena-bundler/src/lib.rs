//! Standalone 0.x bundle planning for Omena CSS transforms.
//!
//! This crate is the standalone Rust entry point for the Omena bundler planning
//! surface. It decides which bundle/module passes are required for a style
//! source and delegates ordering to `omena-transform-passes`.
//!
//! The public types intentionally keep their `V0` suffix during the 0.x line.

use omena_cascade::{CascadeKey, CascadeLevel, LayerRank, ModuleRank, Specificity};
use omena_parser::{
    ClosedWorldBundleBuildErrorV0, ClosedWorldBundleV0, ClosedWorldLinkedModuleV0,
    ClosedWorldModuleMetadataV0, ConfigurationHashV0, ModuleIdV0, ModuleInstanceKeyV0,
    ParsedAnimationFactKind, ParsedCssModuleComposesEdgeKind, ParsedCssModuleValueFactKind,
    ParsedSassModuleEdgeFactKind, ParsedSelectorFactKind, ParsedVariableFactKind, StyleDialect,
    collect_style_facts,
};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformBundleEdgeV0 {
    pub kind: TransformBundleEdgeKind,
    pub source_path: String,
    pub import_source: Option<String>,
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransformBundleSemanticReachabilityInputV0 {
    pub source_path: String,
    pub class_names: Vec<String>,
    pub keyframe_names: Vec<String>,
    pub value_names: Vec<String>,
    pub custom_property_names: Vec<String>,
}

impl TransformBundleSemanticReachabilityInputV0 {
    pub fn new(source_path: impl Into<String>) -> Self {
        Self {
            source_path: source_path.into(),
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
    pub instance: ModuleInstanceKeyV0,
    pub dependency_edges: Vec<LinkerDependencyEdgeV0>,
    pub class_names: Vec<String>,
    pub keyframe_names: Vec<String>,
    pub value_names: Vec<String>,
    pub custom_property_names: Vec<String>,
    pub ordered_rules: Vec<LinkerRuleV0>,
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
        layer_rank: LayerRank,
        scope_proximity: u32,
        specificity: Specificity,
        module_rank: ModuleRank,
    ) -> CascadeKey {
        CascadeKey::new(
            level,
            layer_rank,
            scope_proximity,
            specificity,
            module_rank,
            self.global_order_index,
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
    pub global_rule_order: GlobalRuleOrderV0,
    pub closed_world_bundle: ClosedWorldBundleV0,
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
    ClosedWorldBundle {
        error: ClosedWorldBundleBuildErrorV0,
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

pub fn link_omena_transform_bundle_modules<P: AsRef<str>>(
    entrypoint_paths: &[P],
    modules: &[TransformBundleModuleInputV0],
) -> Result<LinkedStylesheetV0, TransformBundleLinkErrorV0> {
    link_omena_transform_bundle_modules_with_semantic_reachability(entrypoint_paths, modules, &[])
}

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

pub fn link_omena_transform_bundle_modules_with_semantic_reachability_and_metadata<
    P: AsRef<str>,
>(
    entrypoint_paths: &[P],
    modules: &[TransformBundleModuleInputV0],
    reachability_inputs: &[TransformBundleSemanticReachabilityInputV0],
    module_metadata: &[ClosedWorldModuleMetadataV0],
) -> Result<LinkedStylesheetV0, TransformBundleLinkErrorV0> {
    let module_records = modules
        .iter()
        .map(TransformBundleModuleRecordV0::from_input)
        .collect::<Vec<_>>();
    let linker_inputs =
        project_linker_inputs_from_module_records(module_records.as_slice(), reachability_inputs);
    let entrypoint_paths = entrypoint_paths
        .iter()
        .map(|path| path.as_ref())
        .collect::<Vec<_>>();

    link_stylesheet_from_projection_with_metadata(
        entrypoint_paths.as_slice(),
        linker_inputs.as_slice(),
        module_metadata,
    )
}

pub fn link_stylesheet_from_projection(
    entrypoint_paths: &[&str],
    inputs: &[LinkerInputV0],
) -> Result<LinkedStylesheetV0, TransformBundleLinkErrorV0> {
    link_stylesheet_from_projection_with_metadata(entrypoint_paths, inputs, &[])
}

fn link_stylesheet_from_projection_with_metadata(
    entrypoint_paths: &[&str],
    inputs: &[LinkerInputV0],
    module_metadata: &[ClosedWorldModuleMetadataV0],
) -> Result<LinkedStylesheetV0, TransformBundleLinkErrorV0> {
    let instances_by_path = module_instances_by_linker_path(inputs);
    let entrypoints = entrypoint_paths
        .iter()
        .map(|path| {
            resolve_module_instance_by_path(path, &instances_by_path).ok_or_else(|| {
                TransformBundleLinkErrorV0::MissingEntrypoint {
                    source_path: normalize_bundle_path(PathBuf::from(*path)),
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let linked_modules =
        collect_closed_world_linked_modules_from_projection(inputs, &instances_by_path)?;
    let closed_world_bundle = ClosedWorldBundleV0::try_from_linked_modules_with_metadata(
        entrypoints.clone(),
        linked_modules,
        module_metadata.to_vec(),
    )
    .map_err(|error| TransformBundleLinkErrorV0::ClosedWorldBundle { error })?;
    let global_rule_order =
        build_global_rule_order_from_projection(inputs, closed_world_bundle.linked_modules());

    Ok(LinkedStylesheetV0 {
        schema_version: "0",
        product: "omena-transform-bundle.linked-stylesheet",
        entrypoints,
        module_instances: closed_world_bundle.linked_modules().to_vec(),
        global_rule_order,
        closed_world_bundle,
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

struct TransformBundleModuleRecordV0 {
    source_path: String,
    instance: ModuleInstanceKeyV0,
    facts: omena_parser::ParsedStyleFacts,
    bundle_edges: Vec<TransformBundleEdgeV0>,
}

impl TransformBundleModuleRecordV0 {
    fn from_input(input: &TransformBundleModuleInputV0) -> Self {
        let source_path = normalize_bundle_path(PathBuf::from(input.source_path.as_str()));
        let facts = collect_style_facts(input.source.as_str(), input.dialect);
        let bundle_edges = collect_bundle_edges_from_facts(&source_path, input.dialect, &facts);
        let instance = ModuleInstanceKeyV0::new(
            ModuleIdV0::new(source_path.clone()),
            input.configuration_hash.clone(),
        );
        Self {
            source_path,
            instance,
            facts,
            bundle_edges,
        }
    }
}

fn project_linker_inputs_from_module_records(
    records: &[TransformBundleModuleRecordV0],
    reachability_inputs: &[TransformBundleSemanticReachabilityInputV0],
) -> Vec<LinkerInputV0> {
    let mut inputs = records
        .iter()
        .map(linker_input_from_module_record)
        .collect::<Vec<_>>();
    apply_semantic_reachability_to_linker_inputs(inputs.as_mut_slice(), reachability_inputs);
    inputs
}

fn linker_input_from_module_record(record: &TransformBundleModuleRecordV0) -> LinkerInputV0 {
    LinkerInputV0 {
        source_path: record.source_path.clone(),
        instance: record.instance.clone(),
        dependency_edges: record
            .bundle_edges
            .iter()
            .filter(|edge| bundle_edge_is_module_dependency(edge.kind))
            .filter_map(|edge| {
                edge.import_source
                    .as_ref()
                    .map(|import_source| LinkerDependencyEdgeV0 {
                        kind: edge.kind,
                        import_source: import_source.clone(),
                    })
            })
            .collect(),
        class_names: dedupe_names(
            record
                .facts
                .selectors
                .iter()
                .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
                .map(|selector| selector.name.clone()),
        ),
        keyframe_names: dedupe_names(
            record
                .facts
                .animations
                .iter()
                .filter(|animation| animation.kind == ParsedAnimationFactKind::KeyframesDeclaration)
                .map(|animation| animation.name.clone()),
        ),
        value_names: dedupe_names(
            record
                .facts
                .css_module_values
                .iter()
                .filter(|value| value.kind == ParsedCssModuleValueFactKind::Definition)
                .map(|value| value.name.clone()),
        ),
        custom_property_names: dedupe_names(
            record
                .facts
                .variables
                .iter()
                .filter(|variable| {
                    variable.kind == ParsedVariableFactKind::CustomPropertyDeclaration
                })
                .map(|variable| variable.name.clone()),
        ),
        ordered_rules: collect_ordered_linker_rules(record),
    }
}

fn collect_ordered_linker_rules(record: &TransformBundleModuleRecordV0) -> Vec<LinkerRuleV0> {
    let mut selectors = record.facts.selectors.clone();
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

fn apply_semantic_reachability_to_linker_inputs(
    inputs: &mut [LinkerInputV0],
    reachability_inputs: &[TransformBundleSemanticReachabilityInputV0],
) {
    if reachability_inputs.is_empty() {
        return;
    }

    let instances_by_path = module_instances_by_linker_path(inputs);
    let module_index_by_instance = inputs
        .iter()
        .enumerate()
        .map(|(index, input)| (input.instance.clone(), index))
        .collect::<BTreeMap<_, _>>();

    for input in reachability_inputs {
        if !input.has_reachable_symbols() {
            continue;
        }
        let Some(instance) =
            resolve_module_instance_by_path(&input.source_path, &instances_by_path)
        else {
            continue;
        };
        let Some(index) = module_index_by_instance.get(&instance).copied() else {
            continue;
        };
        inputs[index].class_names = dedupe_names(input.class_names.iter().cloned());
        inputs[index].keyframe_names = dedupe_names(input.keyframe_names.iter().cloned());
        inputs[index].value_names = dedupe_names(input.value_names.iter().cloned());
        inputs[index].custom_property_names =
            dedupe_names(input.custom_property_names.iter().cloned());
    }
}

fn module_instances_by_linker_path(
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

fn resolve_module_instance_by_path(
    source_path: &str,
    instances_by_path: &BTreeMap<String, Vec<ModuleInstanceKeyV0>>,
) -> Option<ModuleInstanceKeyV0> {
    let normalized = normalize_bundle_path(PathBuf::from(source_path));
    let instances = instances_by_path.get(&normalized)?;
    if instances.len() == 1 {
        instances.first().cloned()
    } else {
        None
    }
}

fn collect_closed_world_linked_modules_from_projection(
    inputs: &[LinkerInputV0],
    instances_by_path: &BTreeMap<String, Vec<ModuleInstanceKeyV0>>,
) -> Result<Vec<ClosedWorldLinkedModuleV0>, TransformBundleLinkErrorV0> {
    inputs
        .iter()
        .map(|input| {
            let mut linked = ClosedWorldLinkedModuleV0::new(input.instance.clone());
            for edge in &input.dependency_edges {
                let dependency = resolve_imported_module_instance(
                    input.source_path.as_str(),
                    edge.import_source.as_str(),
                    instances_by_path,
                )?
                .ok_or_else(|| TransformBundleLinkErrorV0::MissingDependency {
                    source_path: input.source_path.clone(),
                    import_source: edge.import_source.clone(),
                })?;
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
            for name in dedupe_names(input.custom_property_names.iter().cloned()) {
                linked = linked.with_custom_property_name(name);
            }
            linked.dependencies.sort();
            linked.dependencies.dedup();
            Ok(linked)
        })
        .collect()
}

fn bundle_edge_is_module_dependency(kind: TransformBundleEdgeKind) -> bool {
    matches!(
        kind,
        TransformBundleEdgeKind::SassUse
            | TransformBundleEdgeKind::SassForward
            | TransformBundleEdgeKind::SassImport
            | TransformBundleEdgeKind::CssImport
            | TransformBundleEdgeKind::LessImport
            | TransformBundleEdgeKind::CssModuleValueImport
            | TransformBundleEdgeKind::CssModuleComposesExternal
            | TransformBundleEdgeKind::IcssImport
    )
}

fn resolve_imported_module_instance(
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

fn build_global_rule_order_from_projection(
    inputs: &[LinkerInputV0],
    linked_modules: &[ModuleInstanceKeyV0],
) -> GlobalRuleOrderV0 {
    let inputs_by_instance = inputs
        .iter()
        .map(|input| (input.instance.clone(), input))
        .collect::<BTreeMap<_, _>>();
    let mut rules = Vec::new();
    for instance in linked_modules {
        let Some(input) = inputs_by_instance.get(instance) else {
            continue;
        };
        for selector in &input.ordered_rules {
            let global_order_index = rules.len() as u32;
            rules.push(LinkedStylesheetRuleV0 {
                global_order_index,
                module_instance: instance.clone(),
                selector_name: selector.selector_name.clone(),
                selector_kind: selector_kind_label(selector.selector_kind),
                range_start: selector.range_start,
                range_end: selector.range_end,
            });
        }
    }
    GlobalRuleOrderV0 { rules }
}

fn selector_kind_label(kind: ParsedSelectorFactKind) -> &'static str {
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

fn dedupe_names(names: impl IntoIterator<Item = String>) -> Vec<String> {
    names
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
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
            namespace: None,
            local_names: vec![edge.local_name.clone()],
            remote_names: vec![edge.remote_name.clone()],
            range_start: u32::from(edge.range.start()),
            range_end: u32::from(edge.range.end()),
            provenance_required: true,
        });
    }

    edges
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
mod tests {
    use super::{
        LinkerDependencyEdgeV0, LinkerInputV0, LinkerRuleV0, TransformBundleAssetUrlKind,
        TransformBundleChunkKind, TransformBundleEdgeKind, TransformBundleLinkErrorV0,
        TransformBundleModuleInputV0, TransformBundleSemanticReachabilityInputV0,
        collect_transform_ir_bundle_asset_urls, link_omena_transform_bundle_modules,
        link_omena_transform_bundle_modules_with_semantic_reachability,
        link_stylesheet_from_projection, raw_scan_bundle_asset_urls_for_oracle,
        rewrite_omena_transform_bundle_asset_urls_in_source,
        summarize_omena_transform_bundle_from_source,
    };
    use omena_parser::{
        ConfigurationHashV0, ModuleIdV0, ModuleInstanceKeyV0, ParsedSelectorFactKind, StyleDialect,
    };

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

        let declarations = button_rules
            .iter()
            .map(|rule| {
                let value = if rule.global_order_index == 0 {
                    "red"
                } else {
                    "blue"
                };
                omena_cascade::CascadeDeclaration {
                    id: format!(
                        "{}:{}",
                        rule.module_instance.module().as_str(),
                        rule.global_order_index
                    ),
                    property: "color".to_string(),
                    value: omena_cascade::CascadeValue::Literal(value.to_string()),
                    key: rule.cascade_key_with_global_source_order(
                        omena_cascade::CascadeLevel::AuthorNormal,
                        omena_cascade::LayerRank(0),
                        0,
                        omena_cascade::Specificity::new(0, 1, 0),
                        if rule.global_order_index == 0 {
                            omena_cascade::ModuleRank::new(u32::MAX, u32::MAX, u32::MAX)
                        } else {
                            omena_cascade::ModuleRank::ZERO
                        },
                    ),
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
        let declarations = linked
            .global_rule_order
            .rules
            .iter()
            .filter(|rule| rule.selector_name == "button")
            .map(|rule| {
                let linked_later = rule.global_order_index == 1;
                omena_cascade::CascadeDeclaration {
                    id: format!(
                        "{}:{}",
                        rule.module_instance.module().as_str(),
                        rule.global_order_index
                    ),
                    property: "color".to_string(),
                    value: omena_cascade::CascadeValue::Literal(if linked_later {
                        "blue".to_string()
                    } else {
                        "red".to_string()
                    }),
                    key: rule.cascade_key_with_global_source_order(
                        omena_cascade::CascadeLevel::AuthorNormal,
                        omena_cascade::LayerRank(0),
                        0,
                        omena_cascade::Specificity::new(0, 1, 0),
                        if linked_later {
                            omena_cascade::ModuleRank::new(u32::MAX, u32::MAX, u32::MAX)
                        } else {
                            omena_cascade::ModuleRank::ZERO
                        },
                    ),
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
            declaration.key.module_rank,
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
                    instance: app.clone(),
                    dependency_edges: vec![LinkerDependencyEdgeV0 {
                        kind: TransformBundleEdgeKind::CssImport,
                        import_source: "./theme.css".to_string(),
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
}

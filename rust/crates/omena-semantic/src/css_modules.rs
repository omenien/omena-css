use std::collections::BTreeSet;

use engine_style_parser::{StyleLanguage, Stylesheet};
use omena_parser::{
    ParsedAnimationFactKind, ParsedCssModuleComposesEdgeKind, ParsedCssModuleComposesFactKind,
    ParsedCssModuleValueFactKind, ParsedIcssFactKind, ParsedSelectorFactKind, StyleDialect,
    collect_style_facts,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssModulesSemanticSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub status: &'static str,
    pub resolution_scope: &'static str,
    pub class_export_count: usize,
    pub class_export_names: Vec<String>,
    pub composes_edge_seed_count: usize,
    pub composes_local_edge_count: usize,
    pub composes_global_edge_count: usize,
    pub composes_external_edge_count: usize,
    pub composes_target_names: Vec<String>,
    pub composes_import_sources: Vec<String>,
    pub value_edge_seed_count: usize,
    pub value_import_edge_count: usize,
    pub value_definition_names: Vec<String>,
    pub value_reference_names: Vec<String>,
    pub value_import_sources: Vec<String>,
    pub icss_edge_seed_count: usize,
    pub icss_export_names: Vec<String>,
    pub icss_import_local_names: Vec<String>,
    pub icss_import_remote_names: Vec<String>,
    pub icss_import_sources: Vec<String>,
    pub keyframe_names: Vec<String>,
    pub animation_reference_names: Vec<String>,
    pub capabilities: CssModulesSemanticCapabilitiesV0,
    pub next_priorities: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssModulesSemanticCapabilitiesV0 {
    pub parser_fact_surface_ready: bool,
    pub per_file_symbol_summary_ready: bool,
    pub composes_edge_seed_ready: bool,
    pub value_edge_seed_ready: bool,
    pub icss_edge_seed_ready: bool,
    pub animation_edge_seed_ready: bool,
    pub cross_file_resolution_ready: bool,
    pub composes_closure_ready: bool,
    pub value_graph_resolution_ready: bool,
    pub cycle_detection_ready: bool,
}

pub fn summarize_css_modules_semantics(sheet: &Stylesheet) -> CssModulesSemanticSummaryV0 {
    let facts = collect_style_facts(sheet.source.as_str(), dialect_for_language(sheet.language));
    let mut class_export_names = BTreeSet::new();
    let mut composes_target_names = BTreeSet::new();
    let mut composes_import_sources = BTreeSet::new();
    let mut composes_local_edge_count = 0usize;
    let mut composes_global_edge_count = 0usize;
    let mut composes_external_edge_count = 0usize;
    let mut value_definition_names = BTreeSet::new();
    let mut value_reference_names = BTreeSet::new();
    let mut value_import_sources = BTreeSet::new();
    let mut icss_export_names = BTreeSet::new();
    let mut icss_import_local_names = BTreeSet::new();
    let mut icss_import_remote_names = BTreeSet::new();
    let mut icss_import_sources = BTreeSet::new();
    let mut keyframe_names = BTreeSet::new();
    let mut animation_reference_names = BTreeSet::new();

    for selector in facts.selectors {
        if selector.kind == ParsedSelectorFactKind::Class {
            class_export_names.insert(selector.name);
        }
    }

    for composes in facts.css_module_composes {
        match composes.kind {
            ParsedCssModuleComposesFactKind::Target => {
                composes_target_names.insert(composes.name);
            }
            ParsedCssModuleComposesFactKind::ImportSource => {
                composes_import_sources.insert(composes.name);
            }
        }
    }
    for edge in facts.css_module_composes_edges {
        match edge.kind {
            ParsedCssModuleComposesEdgeKind::Local => composes_local_edge_count += 1,
            ParsedCssModuleComposesEdgeKind::Global => composes_global_edge_count += 1,
            ParsedCssModuleComposesEdgeKind::External => composes_external_edge_count += 1,
        }
    }

    for value in facts.css_module_values {
        match value.kind {
            ParsedCssModuleValueFactKind::Definition => {
                value_definition_names.insert(value.name);
            }
            ParsedCssModuleValueFactKind::Reference => {
                value_reference_names.insert(value.name);
            }
            ParsedCssModuleValueFactKind::ImportSource => {
                value_import_sources.insert(value.name);
            }
        }
    }

    for icss in facts.icss {
        match icss.kind {
            ParsedIcssFactKind::ExportName => {
                icss_export_names.insert(icss.name);
            }
            ParsedIcssFactKind::ImportLocalName => {
                icss_import_local_names.insert(icss.name);
            }
            ParsedIcssFactKind::ImportRemoteName => {
                icss_import_remote_names.insert(icss.name);
            }
            ParsedIcssFactKind::ImportSource => {
                icss_import_sources.insert(icss.name);
            }
        }
    }

    for animation in facts.animations {
        match animation.kind {
            ParsedAnimationFactKind::KeyframesDeclaration => {
                keyframe_names.insert(animation.name);
            }
            ParsedAnimationFactKind::AnimationNameReference => {
                animation_reference_names.insert(animation.name);
            }
        }
    }

    let class_export_names: Vec<_> = class_export_names.into_iter().collect();
    let composes_target_names: Vec<_> = composes_target_names.into_iter().collect();
    let composes_import_sources: Vec<_> = composes_import_sources.into_iter().collect();
    let value_definition_names: Vec<_> = value_definition_names.into_iter().collect();
    let value_reference_names: Vec<_> = value_reference_names.into_iter().collect();
    let value_import_sources: Vec<_> = value_import_sources.into_iter().collect();
    let icss_export_names: Vec<_> = icss_export_names.into_iter().collect();
    let icss_import_local_names: Vec<_> = icss_import_local_names.into_iter().collect();
    let icss_import_remote_names: Vec<_> = icss_import_remote_names.into_iter().collect();
    let icss_import_sources: Vec<_> = icss_import_sources.into_iter().collect();
    let keyframe_names: Vec<_> = keyframe_names.into_iter().collect();
    let animation_reference_names: Vec<_> = animation_reference_names.into_iter().collect();

    CssModulesSemanticSummaryV0 {
        schema_version: "0",
        product: "omena-semantic.css-modules-semantics",
        status: "parserFactSeed",
        resolution_scope: "perFileFactSummary",
        class_export_count: class_export_names.len(),
        class_export_names,
        composes_edge_seed_count: composes_local_edge_count
            + composes_global_edge_count
            + composes_external_edge_count,
        composes_local_edge_count,
        composes_global_edge_count,
        composes_external_edge_count,
        composes_target_names,
        composes_import_sources,
        value_edge_seed_count: facts.css_module_value_import_edge_count,
        value_import_edge_count: facts.css_module_value_import_edge_count,
        value_definition_names,
        value_reference_names,
        value_import_sources,
        icss_edge_seed_count: icss_export_names.len()
            + icss_import_local_names.len()
            + icss_import_remote_names.len()
            + icss_import_sources.len(),
        icss_export_names,
        icss_import_local_names,
        icss_import_remote_names,
        icss_import_sources,
        keyframe_names,
        animation_reference_names,
        capabilities: CssModulesSemanticCapabilitiesV0 {
            parser_fact_surface_ready: true,
            per_file_symbol_summary_ready: true,
            composes_edge_seed_ready: true,
            value_edge_seed_ready: true,
            icss_edge_seed_ready: true,
            animation_edge_seed_ready: true,
            cross_file_resolution_ready: false,
            composes_closure_ready: false,
            value_graph_resolution_ready: false,
            cycle_detection_ready: false,
        },
        next_priorities: vec![
            "crossFileComposesResolution",
            "cssModulesValueGraphResolution",
            "icssImportExportResolution",
            "cycleDetection",
        ],
    }
}

fn dialect_for_language(language: StyleLanguage) -> StyleDialect {
    match language {
        StyleLanguage::Css => StyleDialect::Css,
        StyleLanguage::Scss => StyleDialect::Scss,
        StyleLanguage::Less => StyleDialect::Less,
    }
}

//! Source-fact backed bundle planning for Omena CSS transforms.
//!
//! This crate is the bridge from parser facts into the transform DAG. It
//! decides which bundle/module passes are required for a style source and
//! delegates ordering to `omena-transform-passes`.

use omena_parser::{
    ParsedCssModuleComposesEdgeKind, ParsedSassModuleEdgeFactKind, StyleDialect,
    collect_style_facts,
};
use omena_transform_cst::TransformPassKind;
use omena_transform_passes::{TransformPassPlanV0, plan_transform_passes};
use serde::Serialize;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformBundleSourceSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub source_path: String,
    pub dialect: &'static str,
    pub bundle_edges: Vec<TransformBundleEdgeV0>,
    pub required_pass_ids: Vec<&'static str>,
    pub planned_pass_ids: Vec<&'static str>,
    pub import_inline_required: bool,
    pub module_evaluation_required: bool,
    pub css_modules_resolution_required: bool,
    pub class_hashing_required: bool,
    pub value_resolution_required: bool,
    pub pass_plan: TransformPassPlanV0,
}

pub fn summarize_omena_transform_bundle_from_source(
    source_path: impl Into<String>,
    source: &str,
    dialect: StyleDialect,
) -> TransformBundleSourceSummaryV0 {
    let source_path = source_path.into();
    let facts = collect_style_facts(source, dialect);
    let bundle_edges = collect_bundle_edges_from_facts(&source_path, dialect, &facts);
    let mut required_passes =
        required_passes_for_source(&source_path, dialect, &facts, &bundle_edges);
    required_passes.sort_by_key(|pass| pass.ordinal());
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
    use super::{TransformBundleEdgeKind, summarize_omena_transform_bundle_from_source};
    use omena_parser::StyleDialect;

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
}

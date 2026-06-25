//! Standalone 0.x bundle planning for Omena CSS transforms.
//!
//! This crate is the standalone Rust entry point for the Omena bundler planning
//! surface. It decides which bundle/module passes are required for a style
//! source and delegates ordering to `omena-transform-passes`.
//!
//! The public types intentionally keep their `V0` suffix during the 0.x line.

use omena_parser::{
    ParsedCssModuleComposesEdgeKind, ParsedSassModuleEdgeFactKind, StyleDialect,
    collect_style_facts,
};
use omena_transform_cst::TransformPassKind;
use omena_transform_passes::{TransformPassPlanV0, plan_transform_passes};
use serde::Serialize;
use std::path::{Component, Path, PathBuf};

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

pub fn summarize_omena_transform_bundle_from_source(
    source_path: impl Into<String>,
    source: &str,
    dialect: StyleDialect,
) -> TransformBundleSourceSummaryV0 {
    let source_path = source_path.into();
    let facts = collect_style_facts(source, dialect);
    let bundle_edges = collect_bundle_edges_from_facts(&source_path, dialect, &facts);
    let asset_urls = collect_bundle_asset_urls(&source_path, source);
    let code_split_chunks = plan_bundle_code_split_chunks(&source_path, &bundle_edges, &asset_urls);
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

pub fn rewrite_omena_transform_bundle_asset_urls_in_source(
    source_path: impl Into<String>,
    source: &str,
) -> TransformBundleAssetUrlRewriteSummaryV0 {
    let source_path = source_path.into();
    let asset_urls = collect_bundle_asset_urls(&source_path, source);
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

pub fn omena_bundler_public_surface_probe_do_not_ship() -> &'static str {
    "probe"
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

fn collect_bundle_asset_urls(source_path: &str, source: &str) -> Vec<TransformBundleAssetUrlV0> {
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
        TransformBundleAssetUrlKind, TransformBundleChunkKind, TransformBundleEdgeKind,
        rewrite_omena_transform_bundle_asset_urls_in_source,
        summarize_omena_transform_bundle_from_source,
    };
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
}

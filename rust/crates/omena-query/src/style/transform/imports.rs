use super::*;
use omena_query_transform_runner::{
    TransformImportInlineV0, inline_css_imports,
    rewrite_omena_transform_bundle_asset_urls_in_source,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn derive_import_inlines_for_transform_context(
    entry: &OmenaQueryStyleFactEntry,
    entries: &[OmenaQueryStyleFactEntry],
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    resolution_context: TransformResolutionContext<'_>,
) -> Vec<TransformImportInlineV0> {
    let entries_by_path = entries
        .iter()
        .map(|entry| (entry.style_path.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    entry
        .facts
        .sass_module_edges
        .iter()
        .filter(|edge| edge.kind == "sassImport")
        .filter_map(|edge| {
            let resolved = resolution_context.resolve_style_module_source(
                entry.style_path.as_str(),
                edge.source.as_str(),
                available_style_paths,
            )?;
            let replacement_css = resolve_import_inline_replacement_for_transform_context(
                resolved.as_str(),
                &entries_by_path,
                available_style_paths,
                source_by_path,
                resolution_context,
                &mut BTreeSet::new(),
            )?;
            Some(TransformImportInlineV0 {
                import_source: edge.source.clone(),
                replacement_css,
            })
        })
        .collect()
}

pub(super) fn resolve_import_inline_replacement_for_transform_context(
    style_path: &str,
    entries_by_path: &BTreeMap<&str, &OmenaQueryStyleFactEntry>,
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    resolution_context: TransformResolutionContext<'_>,
    visiting: &mut BTreeSet<String>,
) -> Option<String> {
    let mut source = rewrite_omena_transform_bundle_asset_urls_in_source(
        style_path,
        source_by_path.get(style_path)?,
    )
    .output_css;
    if !visiting.insert(style_path.to_string()) {
        return Some(source);
    }
    let Some(entry) = entries_by_path.get(style_path) else {
        visiting.remove(style_path);
        return Some(source);
    };
    let nested_inlines = derive_import_inlines_for_transform_context_entry(
        entry,
        entries_by_path,
        available_style_paths,
        source_by_path,
        resolution_context,
        visiting,
    );
    visiting.remove(style_path);

    let dialect = omena_parser_dialect_for_style_path(style_path);
    if !nested_inlines.is_empty() {
        let (inlined_source, mutation_count) =
            inline_css_imports(&source, dialect, &nested_inlines);
        if mutation_count > 0 {
            source = inlined_source;
        }
    }
    let scss_module_uses =
        super::static_stylesheet::derive_static_scss_module_use_evaluations_for_transform_context(
            entry,
            available_style_paths,
            source_by_path,
            resolution_context,
        );
    let source =
        super::static_stylesheet::derive_scss_use_aware_static_stylesheet_module_evaluation_source(
            source.as_str(),
            dialect,
            &scss_module_uses,
        );
    Some(source.into_owned())
}

fn derive_import_inlines_for_transform_context_entry(
    entry: &OmenaQueryStyleFactEntry,
    entries_by_path: &BTreeMap<&str, &OmenaQueryStyleFactEntry>,
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    resolution_context: TransformResolutionContext<'_>,
    visiting: &mut BTreeSet<String>,
) -> Vec<TransformImportInlineV0> {
    entry
        .facts
        .sass_module_edges
        .iter()
        .filter(|edge| edge.kind == "sassImport")
        .filter_map(|edge| {
            let resolved = resolution_context.resolve_style_module_source(
                entry.style_path.as_str(),
                edge.source.as_str(),
                available_style_paths,
            )?;
            let replacement_css = resolve_import_inline_replacement_for_transform_context(
                resolved.as_str(),
                entries_by_path,
                available_style_paths,
                source_by_path,
                resolution_context,
                visiting,
            )?;
            Some(TransformImportInlineV0 {
                import_source: edge.source.clone(),
                replacement_css,
            })
        })
        .collect()
}

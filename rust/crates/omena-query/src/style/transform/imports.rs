use super::*;
use omena_query_transform_runner::{TransformImportInlineV0, inline_css_imports};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn derive_import_inlines_for_transform_context(
    entry: &OmenaQueryStyleFactEntry,
    entries: &[OmenaQueryStyleFactEntry],
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
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
            let resolved = resolve_style_module_source(
                entry.style_path.as_str(),
                edge.source.as_str(),
                available_style_paths,
                package_manifests,
            )?;
            let replacement_css = resolve_import_inline_replacement_for_transform_context(
                resolved.as_str(),
                &entries_by_path,
                available_style_paths,
                source_by_path,
                package_manifests,
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
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    visiting: &mut BTreeSet<String>,
) -> Option<String> {
    let source = source_by_path.get(style_path)?.clone();
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
        package_manifests,
        visiting,
    );
    visiting.remove(style_path);

    if nested_inlines.is_empty() {
        return Some(source);
    }
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let (inlined_source, mutation_count) = inline_css_imports(&source, dialect, &nested_inlines);
    if mutation_count > 0 {
        Some(inlined_source)
    } else {
        Some(source)
    }
}

fn derive_import_inlines_for_transform_context_entry(
    entry: &OmenaQueryStyleFactEntry,
    entries_by_path: &BTreeMap<&str, &OmenaQueryStyleFactEntry>,
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    visiting: &mut BTreeSet<String>,
) -> Vec<TransformImportInlineV0> {
    entry
        .facts
        .sass_module_edges
        .iter()
        .filter(|edge| edge.kind == "sassImport")
        .filter_map(|edge| {
            let resolved = resolve_style_module_source(
                entry.style_path.as_str(),
                edge.source.as_str(),
                available_style_paths,
                package_manifests,
            )?;
            let replacement_css = resolve_import_inline_replacement_for_transform_context(
                resolved.as_str(),
                entries_by_path,
                available_style_paths,
                source_by_path,
                package_manifests,
                visiting,
            )?;
            Some(TransformImportInlineV0 {
                import_source: edge.source.clone(),
                replacement_css,
            })
        })
        .collect()
}

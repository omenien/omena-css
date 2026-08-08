use super::super::parser_facade::lex_omena_query_omena_parser_style_source;
use super::*;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use omena_query_core::canonicalize_css_value;
use omena_query_transform_runner::{
    TransformClassNameRewriteV0, TransformCssModuleComposesResolutionV0,
    TransformCssModuleValueResolutionV0,
    resolve_static_css_modules_local_value_resolutions_from_source,
};
use omena_syntax::{SyntaxKind, ident::decode_css_identifier_escapes};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

const CSS_MODULE_TOKEN_HASH_WIDTH_V0: usize = 6;

pub(in crate::style) fn module_instance_key_relative_to_root(
    module_instance: &omena_parser::ModuleInstanceKeyV0,
    workspace_root: &str,
) -> Result<omena_parser::ModuleInstanceKeyV0, String> {
    if workspace_root.trim().is_empty() {
        return Err(
            "CSS Modules token identity requires a non-empty caller workspace root".to_string(),
        );
    }
    let normalized_root = crate::types::normalize_omena_query_style_path(workspace_root);
    let normalized_root = if normalized_root.is_empty() {
        ".".to_string()
    } else {
        normalized_root
    };
    let normalized_module =
        crate::types::normalize_omena_query_style_path(module_instance.module().as_str());
    let module_is_absolute = normalized_module.starts_with('/')
        || normalized_module
            .as_bytes()
            .get(1)
            .is_some_and(|separator| *separator == b':');
    let relative_module = if module_is_absolute {
        let root = normalized_root.trim_end_matches('/');
        normalized_module
            .strip_prefix(root)
            .and_then(|suffix| suffix.strip_prefix('/'))
            .ok_or_else(|| {
                format!(
                    "CSS Modules token identity module {normalized_module:?} is outside caller workspace root {normalized_root:?}"
                )
            })?
            .to_string()
    } else {
        if normalized_module == ".." || normalized_module.starts_with("../") {
            return Err(format!(
                "CSS Modules token identity module {normalized_module:?} escapes caller workspace root {normalized_root:?}"
            ));
        }
        normalized_module
    };
    if relative_module.is_empty() {
        return Err(format!(
            "CSS Modules token identity module {:?} resolves to the caller workspace root",
            module_instance.module().as_str()
        ));
    }
    Ok(omena_parser::ModuleInstanceKeyV0::unconfigured(
        omena_parser::ModuleIdV0::new(relative_module),
    ))
}

pub(in crate::style) fn derive_class_name_rewrites_for_transform_context(
    entry: &OmenaQueryStyleFactEntry,
) -> Vec<TransformClassNameRewriteV0> {
    let module_instance =
        omena_parser::ModuleInstanceKeyV0::unconfigured(omena_parser::ModuleIdV0::new(
            crate::types::normalize_omena_query_style_path(entry.style_path.as_str()),
        ));
    derive_class_name_rewrites_for_module_instance(entry, &module_instance)
}

pub(in crate::style) fn derive_class_name_rewrites_for_module_instance(
    entry: &OmenaQueryStyleFactEntry,
    module_instance: &omena_parser::ModuleInstanceKeyV0,
) -> Vec<TransformClassNameRewriteV0> {
    if !style_path_is_css_module_path(entry.style_path.as_str()) {
        return Vec::new();
    }

    let mut unique_class_names: Vec<String> = Vec::new();
    for name in &entry.facts.class_selector_names {
        if !unique_class_names.contains(name) {
            unique_class_names.push(name.clone());
        }
    }

    unique_class_names
        .into_iter()
        .map(|name| TransformClassNameRewriteV0 {
            original_name: name.clone(),
            rewritten_name: stable_transform_context_class_rewrite(module_instance, &name),
        })
        .collect()
}

pub(super) fn derive_css_module_composes_resolutions_for_transform_context(
    entry: &OmenaQueryStyleFactEntry,
    entries: &[OmenaQueryStyleFactEntry],
    available_style_paths: &BTreeSet<&str>,
    resolution_context: TransformResolutionContext<'_>,
) -> Vec<TransformCssModuleComposesResolutionV0> {
    let facts_by_path = entries
        .iter()
        .map(|entry| (entry.style_path.as_str(), entry.facts.clone()))
        .collect::<BTreeMap<_, _>>();
    let composes_graph = collect_css_modules_composes_adjacency_with_path_mappings(
        &facts_by_path,
        available_style_paths,
        resolution_context.package_manifests,
        resolution_context.bundler_path_mappings,
        resolution_context.tsconfig_path_mappings,
        resolution_context.disk_style_path_identities,
    );
    let mut resolutions = BTreeMap::<String, BTreeSet<String>>::new();

    for edge in &entry.facts.css_module_composes_edges {
        for owner in &edge.owner_selector_names {
            let exports = resolutions.entry(owner.clone()).or_default();
            exports.insert(owner.clone());
            for target in &edge.target_names {
                exports.insert(target.clone());
            }
            for target in css_module_composes_closure_for_context(
                &composes_graph,
                entry.style_path.as_str(),
                owner,
            ) {
                exports.insert(target.selector_name);
            }
        }
    }

    resolutions
        .into_iter()
        .map(
            |(local_class_name, exported_class_names)| TransformCssModuleComposesResolutionV0 {
                local_class_name,
                exported_class_names: exported_class_names.into_iter().collect(),
            },
        )
        .collect()
}

pub(super) fn derive_css_module_value_resolutions_for_transform_context(
    entry: &OmenaQueryStyleFactEntry,
    entries: &[OmenaQueryStyleFactEntry],
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    resolution_context: TransformResolutionContext<'_>,
) -> Vec<TransformCssModuleValueResolutionV0> {
    let facts_by_path = entries
        .iter()
        .map(|entry| (entry.style_path.as_str(), entry.facts.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut resolutions_by_name = BTreeMap::<String, String>::new();
    let mut blocked_names = Vec::<String>::new();

    for edge in &entry.facts.css_module_value_import_edges {
        if blocked_names.iter().any(|name| name == &edge.local_name) {
            continue;
        }
        let Some(resolved_style_path) = resolution_context.resolve_style_module_source(
            entry.style_path.as_str(),
            edge.import_source.as_str(),
            available_style_paths,
        ) else {
            continue;
        };
        let Some(source) = source_by_path.get(resolved_style_path.as_str()) else {
            continue;
        };
        if source.is_empty() {
            continue;
        }
        let Some(resolved_value) = resolve_css_module_value_for_transform_context(
            resolved_style_path.as_str(),
            edge.remote_name.as_str(),
            &facts_by_path,
            available_style_paths,
            source_by_path,
            resolution_context,
            &mut BTreeSet::new(),
        ) else {
            continue;
        };

        if let Some(existing) = resolutions_by_name.get(&edge.local_name) {
            if existing != &resolved_value {
                resolutions_by_name.remove(&edge.local_name);
                blocked_names.push(edge.local_name.clone());
            }
            continue;
        }
        resolutions_by_name.insert(edge.local_name.clone(), resolved_value);
    }

    resolutions_by_name
        .into_iter()
        .map(
            |(local_name, resolved_value)| TransformCssModuleValueResolutionV0 {
                local_name,
                resolved_value,
            },
        )
        .collect()
}

pub(super) fn style_path_is_css_module_path(style_path: &str) -> bool {
    let file_name = style_path
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(style_path)
        .to_ascii_lowercase();
    let Some((stem, extension)) = file_name.rsplit_once('.') else {
        return false;
    };
    matches!(extension, "css" | "scss" | "sass" | "less") && stem.ends_with(".module")
}

fn resolve_css_module_value_for_transform_context(
    style_path: &str,
    value_name: &str,
    facts_by_path: &BTreeMap<&str, OmenaQueryOmenaParserStyleFactsV0>,
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    resolution_context: TransformResolutionContext<'_>,
    visiting: &mut BTreeSet<(String, String)>,
) -> Option<String> {
    let visit_key = (style_path.to_string(), value_name.to_string());
    if !visiting.insert(visit_key.clone()) {
        return None;
    }

    let resolved = resolve_css_module_value_for_transform_context_inner(
        style_path,
        value_name,
        facts_by_path,
        available_style_paths,
        source_by_path,
        resolution_context,
        visiting,
    );
    visiting.remove(&visit_key);
    resolved
}

fn resolve_css_module_value_for_transform_context_inner(
    style_path: &str,
    value_name: &str,
    facts_by_path: &BTreeMap<&str, OmenaQueryOmenaParserStyleFactsV0>,
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    resolution_context: TransformResolutionContext<'_>,
    visiting: &mut BTreeSet<(String, String)>,
) -> Option<String> {
    let facts = facts_by_path.get(style_path)?;
    let source = source_by_path.get(style_path)?;
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let local_resolutions =
        resolve_static_css_modules_local_value_resolutions_from_source(source.as_str(), dialect)
            .into_iter()
            .map(|resolution| {
                (
                    resolution.local_name,
                    canonical_css_module_value_resolution_text(&resolution.resolved_value),
                )
            })
            .collect::<BTreeMap<_, _>>();

    let imported_resolutions = resolve_css_module_imported_values_for_transform_context(
        style_path,
        facts,
        facts_by_path,
        available_style_paths,
        source_by_path,
        resolution_context,
        visiting,
    );
    if let Some(imported_value) = imported_resolutions.get(value_name) {
        return Some(imported_value.clone());
    }

    let local_value = local_resolutions.get(value_name)?;
    let resolved = substitute_css_module_value_resolution_references(
        local_value.as_str(),
        dialect,
        &imported_resolutions,
    )
    .unwrap_or_else(|| local_value.clone());
    Some(canonical_css_module_value_resolution_text(&resolved))
}

fn resolve_css_module_imported_values_for_transform_context(
    style_path: &str,
    facts: &OmenaQueryOmenaParserStyleFactsV0,
    facts_by_path: &BTreeMap<&str, OmenaQueryOmenaParserStyleFactsV0>,
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    resolution_context: TransformResolutionContext<'_>,
    visiting: &mut BTreeSet<(String, String)>,
) -> BTreeMap<String, String> {
    let mut resolutions = BTreeMap::<String, String>::new();
    let mut blocked_names = BTreeSet::<String>::new();
    for edge in &facts.css_module_value_import_edges {
        if blocked_names.contains(&edge.local_name) {
            continue;
        }
        let Some(target_style_path) = resolution_context.resolve_style_module_source(
            style_path,
            edge.import_source.as_str(),
            available_style_paths,
        ) else {
            continue;
        };
        let Some(resolved_value) = resolve_css_module_value_for_transform_context(
            target_style_path.as_str(),
            edge.remote_name.as_str(),
            facts_by_path,
            available_style_paths,
            source_by_path,
            resolution_context,
            visiting,
        ) else {
            continue;
        };
        if let Some(existing) = resolutions.get(&edge.local_name) {
            if existing != &resolved_value {
                resolutions.remove(&edge.local_name);
                blocked_names.insert(edge.local_name.clone());
            }
            continue;
        }
        resolutions.insert(edge.local_name.clone(), resolved_value);
    }
    resolutions
}

fn substitute_css_module_value_resolution_references(
    value: &str,
    dialect: OmenaParserStyleDialect,
    resolutions_by_name: &BTreeMap<String, String>,
) -> Option<String> {
    let lexed = lex_omena_query_omena_parser_style_source(value, dialect);
    let mut replacements = Vec::new();
    for token in lexed.tokens() {
        if token.kind != SyntaxKind::Ident {
            continue;
        }
        let Some(resolved_value) = resolutions_by_name.get(&token.text) else {
            continue;
        };
        replacements.push((
            transform_token_start(token),
            transform_token_end(token),
            resolved_value.clone(),
        ));
    }
    if replacements.is_empty() {
        return None;
    }
    let (output, mutation_count) = apply_transform_source_replacements(value, replacements);
    (mutation_count > 0).then_some(output)
}

fn canonical_css_module_value_resolution_text(value: &str) -> String {
    canonicalize_css_value(value)
        .map(|value| value.serialized)
        .unwrap_or_else(|| value.trim().to_string())
}

fn css_module_composes_closure_for_context(
    graph: &BTreeMap<CssModulesComposesNode, BTreeSet<CssModulesComposesNode>>,
    style_path: &str,
    selector_name: &str,
) -> BTreeSet<CssModulesComposesNode> {
    let start = CssModulesComposesNode::new(style_path, selector_name);
    let mut closure = BTreeSet::new();
    let mut pending = VecDeque::from([start]);

    while let Some(current) = pending.pop_front() {
        let Some(targets) = graph.get(&current) else {
            continue;
        };
        for target in targets {
            if closure.insert(target.clone()) {
                pending.push_back(target.clone());
            }
        }
    }

    closure
}

fn stable_transform_context_class_rewrite(
    module_instance: &omena_parser::ModuleInstanceKeyV0,
    name: &str,
) -> String {
    let canonical_name = decode_css_identifier_escapes(name);
    let sanitized = canonical_name
        .as_ref()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    let sanitized = if sanitized.is_empty() {
        "class"
    } else {
        sanitized.as_str()
    };
    let mut digest = Sha256::new();
    digest.update(module_instance.module().as_str().as_bytes());
    digest.update([0]);
    digest.update(name.as_bytes());
    let hash = URL_SAFE_NO_PAD.encode(digest.finalize());
    format!("_{}_{}", &hash[..CSS_MODULE_TOKEN_HASH_WIDTH_V0], sanitized)
}

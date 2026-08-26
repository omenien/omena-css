use super::*;
use omena_query_transform_runner::TransformDesignTokenRouteV0;
use omena_syntax::ident::{CanonicalCustomPropertyNameV0, PropertyNameV0};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub(super) fn derive_design_token_routes_for_transform_context(
    entry: &OmenaQueryStyleFactEntry,
    entries: &[OmenaQueryStyleFactEntry],
    resolution_context: TransformResolutionContext<'_>,
) -> Vec<TransformDesignTokenRouteV0> {
    let workspace_declarations = entries
        .iter()
        .flat_map(|entry| {
            collect_omena_bridge_design_token_workspace_declarations_from_source(
                entry.style_path.as_str(),
                entry.style_source.as_str(),
            )
        })
        .collect::<Vec<_>>();
    let reachable_declarations = filter_import_reachable_design_token_workspace_declarations(
        entry.style_path.as_str(),
        entries,
        &workspace_declarations,
        resolution_context.package_manifests,
        OmenaQueryStylePathResolutionInputsV0 {
            bundler_path_mappings: resolution_context.bundler_path_mappings,
            tsconfig_path_mappings: resolution_context.tsconfig_path_mappings,
            disk_style_path_identities: resolution_context.disk_style_path_identities,
        },
        resolution_context.resolver_identity_index,
    );
    let (local_decl_keys, local_refs) = local_custom_property_index_names(entry);

    let mut routes = Vec::new();
    let mut routed = BTreeSet::new();
    let mut queued = BTreeSet::new();
    let mut pending = VecDeque::new();

    for (property_key, authored) in local_refs
        .iter()
        .filter(|(property_key, _)| !local_decl_keys.contains(*property_key))
    {
        if queued.insert(property_key.clone()) {
            pending.push_back((property_key.clone(), authored.clone()));
        }
    }

    while let Some((property_key, authored)) = pending.pop_front() {
        if routed.contains(&property_key) {
            continue;
        }
        let Some(candidate) = unique_external_design_token_route_declaration(
            &property_key,
            entry.style_path.as_str(),
            &reachable_declarations,
        ) else {
            continue;
        };
        routed.insert(property_key);
        for (dependency_key, dependency_authored) in
            collect_design_token_route_value_references(&candidate.value)
        {
            if !local_decl_keys.contains(&dependency_key) && queued.insert(dependency_key.clone()) {
                pending.push_back((dependency_key, dependency_authored));
            }
        }
        routes.push(TransformDesignTokenRouteV0 {
            token_name: authored,
            routed_value: candidate.value.clone(),
        });
    }

    routes
}

fn local_custom_property_index_names(
    entry: &OmenaQueryStyleFactEntry,
) -> (
    BTreeSet<CanonicalCustomPropertyNameV0>,
    BTreeMap<CanonicalCustomPropertyNameV0, String>,
) {
    let (declaration_names, reference_names) =
        if let Some(index) = entry.semantic_runtime_index.as_ref() {
            (
                index.custom_property_decl_names.as_slice(),
                index.custom_property_ref_names.as_slice(),
            )
        } else {
            (
                entry.facts.custom_property_decl_names.as_slice(),
                entry.facts.custom_property_ref_names.as_slice(),
            )
        };

    (
        declaration_names
            .iter()
            .filter_map(|name| PropertyNameV0::from_authored(name).as_custom_key())
            .collect(),
        reference_names
            .iter()
            .filter_map(|name| {
                PropertyNameV0::from_authored(name)
                    .as_custom_key()
                    .map(|property_key| (property_key, name.clone()))
            })
            .collect(),
    )
}

fn unique_external_design_token_route_declaration<'a>(
    property_key: &CanonicalCustomPropertyNameV0,
    target_style_path: &str,
    reachable_declarations: &'a [DesignTokenWorkspaceDeclarationFactV0],
) -> Option<&'a DesignTokenWorkspaceDeclarationFactV0> {
    let candidates = reachable_declarations
        .iter()
        .filter(|declaration| declaration.file_path != target_style_path)
        .filter(|declaration| declaration.property_key == *property_key)
        .filter(|declaration| design_token_route_value_is_safe(&declaration.value))
        .collect::<Vec<_>>();
    let [candidate] = candidates.as_slice() else {
        return None;
    };
    Some(candidate)
}

fn design_token_route_value_is_safe(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty() && !value.chars().any(|ch| matches!(ch, ';' | '{' | '}'))
}

fn collect_design_token_route_value_references(
    value: &str,
) -> Vec<(CanonicalCustomPropertyNameV0, String)> {
    let mut references = Vec::new();
    let mut seen = BTreeSet::new();
    let mut index = 0usize;
    let mut quote: Option<char> = None;

    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = value[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if value[index..]
                .get(.."var(".len())
                .is_some_and(|text| text.eq_ignore_ascii_case("var(")) =>
            {
                let left_paren_index = index + "var".len();
                if let Some(close_index) =
                    matching_design_token_route_function_call_end(value, left_paren_index)
                    && let Some((property_key, authored)) = design_token_route_first_argument_name(
                        &value[left_paren_index + 1..close_index],
                    )
                    && seen.insert(property_key.clone())
                {
                    references.push((property_key, authored));
                }
                index += ch.len_utf8();
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    references
}

fn matching_design_token_route_function_call_end(
    value: &str,
    left_paren_index: usize,
) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = left_paren_index;
    let mut quote: Option<char> = None;

    while index < value.len() {
        let ch = value[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = value[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            '(' => {
                depth += 1;
                index += ch.len_utf8();
            }
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
                index += ch.len_utf8();
            }
            _ => index += ch.len_utf8(),
        }
    }

    None
}

fn design_token_route_first_argument_name(
    arguments: &str,
) -> Option<(CanonicalCustomPropertyNameV0, String)> {
    let mut index = 0usize;
    let mut depth = 0usize;
    let mut quote: Option<char> = None;

    while index < arguments.len() {
        let ch = arguments[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = arguments[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            '(' => {
                depth += 1;
                index += ch.len_utf8();
            }
            ')' => {
                depth = depth.checked_sub(1)?;
                index += ch.len_utf8();
            }
            ',' if depth == 0 => return normalize_design_token_route_name(&arguments[..index]),
            _ => index += ch.len_utf8(),
        }
    }

    normalize_design_token_route_name(arguments)
}

fn normalize_design_token_route_name(
    name: &str,
) -> Option<(CanonicalCustomPropertyNameV0, String)> {
    let property = PropertyNameV0::from_authored(name);
    let property_key = property.as_custom_key()?;
    (property_key.as_str().len() > 2).then(|| (property_key, property.authored_text().to_string()))
}

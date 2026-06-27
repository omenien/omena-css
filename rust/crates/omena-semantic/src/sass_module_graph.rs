use std::collections::{BTreeMap, BTreeSet, VecDeque};

use omena_cross_file_summary::{
    HypergraphClosureMode, HypergraphClosurePath, collect_directed_graph_cycles,
    collect_hypergraph_transitive_closure_paths,
    collect_hypergraph_transitive_closure_paths_with_mode,
};
use omena_parser::{LexedToken, StyleDialect, summarize_omena_parser_style_facts};
use omena_resolver::canonicalize_omena_resolver_style_identity_path;
use omena_syntax::SyntaxKind;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassModuleGraphEdgeFactV0 {
    pub from_style_path: String,
    pub edge_kind: &'static str,
    pub source: String,
    pub rule_ordinal: usize,
    pub namespace_kind: Option<&'static str>,
    pub namespace: Option<String>,
    pub forward_prefix: Option<String>,
    pub visibility_filter_kind: Option<&'static str>,
    pub visibility_filter_names: Vec<String>,
    pub resolved_style_path: Option<String>,
    pub status: &'static str,
    pub configuration_signature: String,
    pub configuration_variable_count: usize,
    pub invalid_configuration_variable_names: Vec<String>,
    pub module_instance_identity_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassModuleGraphClosureSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub status: &'static str,
    pub module_edge_count: usize,
    pub graph_closure_edge_count: usize,
    pub cycle_count: usize,
    pub graph_closure_edges: Vec<SassModuleGraphClosureEdgeV0>,
    pub cycles: Vec<SassModuleCycleV0>,
    pub capabilities: SassModuleGraphClosureCapabilitiesV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassModuleGraphClosureCapabilitiesV0 {
    pub semantic_layer_owned: bool,
    pub graph_closure_ready: bool,
    pub cycle_detection_ready: bool,
    pub namespace_show_hide_filter_ready: bool,
    pub configured_module_instance_identity_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassModuleGraphClosureEdgeV0 {
    pub from_style_path: String,
    pub target_style_path: String,
    pub edge_kind: &'static str,
    pub depth: usize,
    pub path: Vec<String>,
    pub namespace_kind: Option<&'static str>,
    pub namespace: Option<String>,
    pub forward_prefix: Option<String>,
    pub visibility_filter_kind: Option<&'static str>,
    pub visibility_filter_names: Vec<String>,
    pub configuration_signature: String,
    pub configuration_variable_count: usize,
    pub invalid_configuration_variable_names: Vec<String>,
    pub module_instance_identity_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassModuleCycleV0 {
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleImportReachabilityEdgeFactV0 {
    pub from_style_path: String,
    pub target_style_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleImportReachabilitySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub status: &'static str,
    pub target_style_path: String,
    pub edge_count: usize,
    pub reachable_style_path_count: usize,
    pub reachable_style_paths: Vec<StyleImportReachabilityFactV0>,
    pub capabilities: StyleImportReachabilityCapabilitiesV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleImportReachabilityFactV0 {
    pub style_path: String,
    pub distance: usize,
    pub order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleImportReachabilityCapabilitiesV0 {
    pub semantic_layer_owned: bool,
    pub transitive_reachability_ready: bool,
    pub stable_distance_ready: bool,
    pub stable_order_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SassModuleVariableOverrideV0 {
    pub value: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct SassModuleUseConfigurationRequestV0<'a> {
    pub from_style_path: &'a str,
    pub rule_ordinal: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct SassModuleForwardConfigurationRequestV0<'a> {
    pub from_style_path: &'a str,
    pub target_style_path: &'a str,
    pub rule_ordinal: usize,
    pub inherited_variable_overrides: &'a BTreeMap<String, String>,
    pub forward_prefix: Option<&'a str>,
    pub visibility_filter_kind: Option<&'static str>,
    pub visibility_filter_names: &'a [String],
    pub configurable_names: &'a BTreeSet<String>,
}

pub trait SassModuleGraphConfigurationResolverV0 {
    fn use_variable_overrides(
        &self,
        request: SassModuleUseConfigurationRequestV0<'_>,
    ) -> BTreeMap<String, String>;

    fn forward_effective_variable_overrides(
        &self,
        request: SassModuleForwardConfigurationRequestV0<'_>,
    ) -> BTreeMap<String, String>;

    fn configurable_names(&self, target_style_path: &str) -> BTreeSet<String>;
}

pub trait SassModuleConfigurableNamesResolverV0 {
    fn local_configurable_names(&self, style_path: &str, style_source: &str) -> BTreeSet<String>;

    fn resolve_module_source(
        &self,
        from_style_path: &str,
        source: &str,
        available_style_paths: &BTreeSet<&str>,
    ) -> Option<String>;
}

pub fn derive_sass_module_configurable_variable_names(
    style_path: &str,
    style_source: &str,
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    resolver: &impl SassModuleConfigurableNamesResolverV0,
) -> BTreeSet<String> {
    let mut visiting = BTreeSet::new();
    derive_sass_module_configurable_variable_names_inner(
        style_path,
        style_source,
        available_style_paths,
        source_by_path,
        resolver,
        &mut visiting,
    )
}

fn derive_sass_module_configurable_variable_names_inner(
    style_path: &str,
    style_source: &str,
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    resolver: &impl SassModuleConfigurableNamesResolverV0,
    visiting: &mut BTreeSet<String>,
) -> BTreeSet<String> {
    let identity_path = canonicalize_omena_resolver_style_identity_path(style_path);
    if !visiting.insert(identity_path.clone()) {
        return BTreeSet::new();
    }

    let mut names = resolver.local_configurable_names(style_path, style_source);
    let facts = summarize_omena_parser_style_facts(style_source, StyleDialect::Scss);
    for (forward_rule_ordinal, edge) in facts
        .sass_module_edges
        .iter()
        .filter(|edge| edge.kind == "sassForward")
        .enumerate()
    {
        let Some(resolved) =
            resolver.resolve_module_source(style_path, edge.source.as_str(), available_style_paths)
        else {
            continue;
        };
        let Some(source) = source_by_path.get(resolved.as_str()) else {
            continue;
        };
        let child_names = derive_sass_module_configurable_variable_names_inner(
            resolved.as_str(),
            source,
            available_style_paths,
            source_by_path,
            resolver,
            visiting,
        );
        let non_default_forward_overrides =
            derive_sass_module_forward_variable_overrides_at_ordinal(
                style_source,
                forward_rule_ordinal,
            )
            .into_iter()
            .filter_map(|(name, override_entry)| (!override_entry.is_default).then_some(name))
            .collect::<BTreeSet<_>>();
        let child_names = child_names
            .into_iter()
            .filter(|name| !non_default_forward_overrides.contains(name))
            .collect::<BTreeSet<_>>();
        let export_prefix =
            derive_sass_forward_export_prefix_at_ordinal(style_source, forward_rule_ordinal);
        names.extend(filter_sass_forward_configurable_variable_names(
            child_names,
            export_prefix.as_deref(),
            edge.visibility_filter_kind,
            &edge.visibility_filter_names,
        ));
    }

    visiting.remove(identity_path.as_str());
    names
}

pub fn summarize_sass_module_configuration_signature(
    variable_overrides: &BTreeMap<String, String>,
) -> String {
    if variable_overrides.is_empty() {
        return "with:none".to_string();
    }
    let mut key = String::from("with");
    for (name, value) in variable_overrides {
        key.push('|');
        key.push_str(name.len().to_string().as_str());
        key.push(':');
        key.push_str(name);
        key.push('=');
        key.push_str(value.len().to_string().as_str());
        key.push(':');
        key.push_str(value);
    }
    key
}

pub fn summarize_sass_module_instance_identity_key(
    style_path: &str,
    variable_overrides: &BTreeMap<String, String>,
) -> String {
    let canonical_path = canonicalize_omena_resolver_style_identity_path(style_path);
    let mut key = format!("path:{}:{canonical_path}", canonical_path.len());
    key.push('|');
    key.push_str(summarize_sass_module_configuration_signature(variable_overrides).as_str());
    key
}

pub fn resolve_sass_module_effective_variable_overrides(
    style_path: &str,
    variable_overrides: &BTreeMap<String, String>,
    loaded_module_overrides_by_path: &mut BTreeMap<String, BTreeMap<String, String>>,
) -> Option<BTreeMap<String, String>> {
    let canonical_path = canonicalize_omena_resolver_style_identity_path(style_path);
    match loaded_module_overrides_by_path.get(canonical_path.as_str()) {
        Some(existing_overrides) if variable_overrides.is_empty() => {
            Some(existing_overrides.clone())
        }
        Some(existing_overrides) => {
            (existing_overrides == variable_overrides).then(|| variable_overrides.clone())
        }
        None => {
            loaded_module_overrides_by_path.insert(canonical_path, variable_overrides.clone());
            Some(variable_overrides.clone())
        }
    }
}

pub fn sass_module_configuration_variables_are_valid(
    variable_overrides: &BTreeMap<String, String>,
    configurable_names: &BTreeSet<String>,
) -> bool {
    variable_overrides
        .keys()
        .all(|name| configurable_names.contains(name))
}

pub fn derive_sass_module_rule_variable_overrides_at_ordinal(
    style_source: &str,
    at_keyword: &str,
    rule_ordinal: usize,
) -> BTreeMap<String, String> {
    sass_module_rule_source_at_ordinal(style_source, at_keyword, rule_ordinal)
        .map(parse_sass_module_use_variable_overrides_from_rule)
        .unwrap_or_default()
}

pub fn derive_sass_module_forward_variable_overrides_at_ordinal(
    style_source: &str,
    forward_rule_ordinal: usize,
) -> BTreeMap<String, SassModuleVariableOverrideV0> {
    sass_module_rule_source_at_ordinal(style_source, "@forward", forward_rule_ordinal)
        .map(parse_sass_module_forward_variable_overrides_from_rule)
        .unwrap_or_default()
}

pub fn derive_sass_module_forward_variable_override_values_at_ordinal(
    style_source: &str,
    forward_rule_ordinal: usize,
) -> BTreeMap<String, String> {
    derive_sass_module_forward_variable_overrides_at_ordinal(style_source, forward_rule_ordinal)
        .into_iter()
        .map(|(name, override_entry)| (name, override_entry.value))
        .collect()
}

pub fn derive_sass_module_forward_effective_variable_overrides_at_ordinal(
    style_source: &str,
    forward_rule_ordinal: usize,
    inherited_variable_overrides: &BTreeMap<String, String>,
    export_prefix: Option<&str>,
    visibility_filter_kind: Option<&'static str>,
    visibility_filter_names: &[String],
    configurable_names: &BTreeSet<String>,
) -> BTreeMap<String, String> {
    let explicit_variable_overrides = derive_sass_module_forward_variable_overrides_at_ordinal(
        style_source,
        forward_rule_ordinal,
    );
    derive_sass_forward_effective_variable_overrides(
        &explicit_variable_overrides,
        inherited_variable_overrides,
        export_prefix,
        visibility_filter_kind,
        visibility_filter_names,
        configurable_names,
    )
}

pub fn derive_sass_forward_effective_variable_overrides(
    explicit_variable_overrides: &BTreeMap<String, SassModuleVariableOverrideV0>,
    inherited_variable_overrides: &BTreeMap<String, String>,
    export_prefix: Option<&str>,
    visibility_filter_kind: Option<&'static str>,
    visibility_filter_names: &[String],
    configurable_names: &BTreeSet<String>,
) -> BTreeMap<String, String> {
    let mut variable_overrides = explicit_variable_overrides
        .iter()
        .filter(|(_, override_entry)| override_entry.is_default)
        .map(|(name, override_entry)| (name.clone(), override_entry.value.clone()))
        .collect::<BTreeMap<_, _>>();
    variable_overrides.extend(
        inherited_variable_overrides
            .iter()
            .filter_map(|(name, value)| {
                let internal_name = sass_forward_internal_variable_name_for_exposed_name(
                    name.as_str(),
                    export_prefix,
                )?;
                sass_forward_exposed_variable_is_visible(
                    name.as_str(),
                    visibility_filter_kind,
                    visibility_filter_names,
                )
                .then_some((internal_name, value.clone()))
            })
            .filter(|(name, _)| configurable_names.contains(name))
            .collect::<BTreeMap<_, _>>(),
    );
    variable_overrides.extend(
        explicit_variable_overrides
            .iter()
            .filter(|(_, override_entry)| !override_entry.is_default)
            .map(|(name, override_entry)| (name.clone(), override_entry.value.clone())),
    );
    variable_overrides
}

pub fn filter_sass_forward_configurable_variable_names(
    names: BTreeSet<String>,
    prefix: Option<&str>,
    visibility_filter_kind: Option<&'static str>,
    visibility_filter_names: &[String],
) -> BTreeSet<String> {
    names
        .into_iter()
        .filter_map(|name| {
            let exposed_name = prefix
                .map(|prefix| prefix.replace('*', name.as_str()))
                .unwrap_or(name);
            sass_forward_exposed_variable_is_visible(
                exposed_name.as_str(),
                visibility_filter_kind,
                visibility_filter_names,
            )
            .then(|| canonical_sass_variable_name(exposed_name.as_str()))
        })
        .collect()
}

pub fn filter_sass_forward_exports(
    exports: BTreeMap<String, String>,
    filter_kind: Option<&'static str>,
    filter_names: &[String],
) -> BTreeMap<String, String> {
    match filter_kind {
        Some("show") => exports
            .into_iter()
            .filter(|(name, _)| {
                filter_names
                    .iter()
                    .any(|filter| sass_variable_names_equal(filter, name))
            })
            .collect(),
        Some("hide") => exports
            .into_iter()
            .filter(|(name, _)| {
                !filter_names
                    .iter()
                    .any(|filter| sass_variable_names_equal(filter, name))
            })
            .collect(),
        _ => exports,
    }
}

pub fn prefix_sass_forward_exports(
    exports: BTreeMap<String, String>,
    prefix: Option<&str>,
) -> BTreeMap<String, String> {
    let Some(prefix) = prefix else {
        return exports;
    };
    exports
        .into_iter()
        .map(|(name, value)| (prefix.replace('*', name.as_str()), value))
        .collect()
}

pub fn derive_sass_forward_export_prefix_at_ordinal(
    style_source: &str,
    forward_rule_ordinal: usize,
) -> Option<String> {
    sass_module_rule_source_at_ordinal(style_source, "@forward", forward_rule_ordinal)
        .and_then(parse_sass_forward_export_prefix_from_rule)
}

fn sass_module_rule_source_at_ordinal<'a>(
    style_source: &'a str,
    at_keyword: &str,
    rule_ordinal: usize,
) -> Option<&'a str> {
    let lexed = omena_parser::lex(style_source, StyleDialect::Scss);
    let tokens = lexed.tokens();
    let mut depth = 0usize;
    let mut index = 0usize;
    let mut current_rule_ordinal = 0usize;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case(at_keyword) =>
            {
                let Some(end_index) = sass_module_rule_semicolon(tokens, index) else {
                    index += 1;
                    continue;
                };
                if sass_module_rule_source_name(tokens, index + 1, end_index).is_some() {
                    if current_rule_ordinal == rule_ordinal {
                        let start = token_start(&tokens[index]);
                        let end = token_end(&tokens[end_index]);
                        return style_source.get(start..end);
                    }
                    current_rule_ordinal += 1;
                }
                index = end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    None
}

fn sass_module_rule_semicolon(tokens: &[LexedToken], at_use_index: usize) -> Option<usize> {
    let mut index = at_use_index + 1;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::Semicolon => return Some(index),
            SyntaxKind::LeftBrace | SyntaxKind::RightBrace => return None,
            _ => index += 1,
        }
    }
    None
}

fn sass_module_rule_source_name(
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
) -> Option<String> {
    tokens[start_index..end_index]
        .iter()
        .find(|token| matches!(token.kind, SyntaxKind::String | SyntaxKind::Url))
        .map(|token| token.text.trim_matches('"').trim_matches('\'').to_string())
}

fn parse_sass_module_use_variable_overrides_from_rule(
    rule_source: &str,
) -> BTreeMap<String, String> {
    parse_sass_module_rule_override_content(rule_source)
        .map(parse_sass_use_variable_override_list)
        .unwrap_or_default()
}

fn parse_sass_module_forward_variable_overrides_from_rule(
    rule_source: &str,
) -> BTreeMap<String, SassModuleVariableOverrideV0> {
    parse_sass_module_rule_override_content(rule_source)
        .map(|content| parse_sass_variable_override_list(content, true))
        .unwrap_or_default()
}

fn parse_sass_module_rule_override_content(rule_source: &str) -> Option<&str> {
    let lexed = omena_parser::lex(rule_source, StyleDialect::Scss);
    let tokens = lexed.tokens();
    let with_index = tokens
        .iter()
        .position(|token| token.text.eq_ignore_ascii_case("with"))?;
    let left_paren_index = tokens[with_index + 1..]
        .iter()
        .position(|token| token.kind == SyntaxKind::LeftParen)
        .map(|offset| with_index + 1 + offset)?;
    let right_paren_index = matching_right_paren(tokens, left_paren_index)?;
    let start = token_end(&tokens[left_paren_index]);
    let end = token_start(&tokens[right_paren_index]);
    rule_source.get(start..end)
}

fn parse_sass_use_variable_override_list(content: &str) -> BTreeMap<String, String> {
    parse_sass_variable_override_list(content, false)
        .into_iter()
        .map(|(name, override_entry)| (name, override_entry.value))
        .collect()
}

fn parse_sass_variable_override_list(
    content: &str,
    allow_default_flag: bool,
) -> BTreeMap<String, SassModuleVariableOverrideV0> {
    let mut overrides = BTreeMap::new();
    for entry in split_top_level_commas(content) {
        if entry.trim().is_empty() {
            continue;
        }
        let Some((name, value)) = parse_sass_variable_override(entry.trim(), allow_default_flag)
        else {
            return BTreeMap::new();
        };
        overrides.insert(name, value);
    }
    overrides
}

fn parse_sass_variable_override(
    entry: &str,
    allow_default_flag: bool,
) -> Option<(String, SassModuleVariableOverrideV0)> {
    let colon_index = top_level_colon_index(entry)?;
    let name = entry[..colon_index].trim().strip_prefix('$')?;
    if name.is_empty() || !name.chars().all(sass_identifier_char) {
        return None;
    }
    let (value, is_default) =
        split_sass_forward_default_flag(entry[colon_index + 1..].trim(), allow_default_flag)?;
    if !sass_variable_override_value_is_safe(value) {
        return None;
    }
    Some((
        canonical_sass_variable_name(name),
        SassModuleVariableOverrideV0 {
            value: value.to_string(),
            is_default,
        },
    ))
}

fn parse_sass_forward_export_prefix_from_rule(rule_source: &str) -> Option<String> {
    let lexed = omena_parser::lex(rule_source, StyleDialect::Scss);
    let tokens = lexed.tokens();
    let source_index = tokens
        .iter()
        .position(|token| matches!(token.kind, SyntaxKind::String | SyntaxKind::Url))?;
    let as_index = tokens[source_index + 1..]
        .iter()
        .position(|token| token.text.eq_ignore_ascii_case("as"))
        .map(|offset| source_index + 1 + offset)?;
    let prefix_start_index = tokens[as_index + 1..]
        .iter()
        .position(|token| token.kind != SyntaxKind::Whitespace)
        .map(|offset| as_index + 1 + offset)?;
    let prefix_end_index = tokens[prefix_start_index..]
        .iter()
        .position(|token| {
            token.kind == SyntaxKind::Semicolon
                || matches!(
                    token.text.to_ascii_lowercase().as_str(),
                    "show" | "hide" | "with"
                )
        })
        .map(|offset| prefix_start_index + offset)
        .unwrap_or(tokens.len());
    let prefix_end = tokens
        .get(prefix_end_index)
        .map(token_start)
        .unwrap_or(rule_source.len());
    let prefix = rule_source
        .get(token_start(&tokens[prefix_start_index])..prefix_end)?
        .trim();
    sass_forward_export_prefix_is_safe(prefix).then(|| prefix.to_string())
}

fn sass_forward_export_prefix_is_safe(prefix: &str) -> bool {
    prefix.contains('*')
        && prefix
            .chars()
            .all(|ch| sass_identifier_char(ch) || ch == '*')
}

fn sass_forward_exposed_variable_is_visible(
    exposed_name: &str,
    visibility_filter_kind: Option<&'static str>,
    visibility_filter_names: &[String],
) -> bool {
    match visibility_filter_kind {
        Some("show") => visibility_filter_names
            .iter()
            .any(|filter| sass_variable_names_equal(filter, exposed_name)),
        Some("hide") => !visibility_filter_names
            .iter()
            .any(|filter| sass_variable_names_equal(filter, exposed_name)),
        _ => true,
    }
}

fn sass_forward_internal_variable_name_for_exposed_name(
    exposed_name: &str,
    export_prefix: Option<&str>,
) -> Option<String> {
    let exposed_name = canonical_sass_variable_name(exposed_name);
    let Some(export_prefix) = export_prefix else {
        return Some(exposed_name);
    };
    let star_offset = export_prefix.find('*')?;
    let prefix_before_star = canonical_sass_variable_name(&export_prefix[..star_offset]);
    let prefix_after_star =
        canonical_sass_variable_name(&export_prefix[star_offset + '*'.len_utf8()..]);
    let without_prefix = exposed_name.strip_prefix(prefix_before_star.as_str())?;
    let without_suffix = if prefix_after_star.is_empty() {
        without_prefix
    } else {
        without_prefix.strip_suffix(prefix_after_star.as_str())?
    };
    (!without_suffix.is_empty()).then(|| canonical_sass_variable_name(without_suffix))
}

fn split_sass_forward_default_flag(value: &str, allow_default_flag: bool) -> Option<(&str, bool)> {
    if !allow_default_flag {
        return Some((value, false));
    }
    let lower = value.to_ascii_lowercase();
    let Some(before_default) = lower.strip_suffix("!default") else {
        return Some((value, false));
    };
    let value_before_default = &value[..before_default.len()];
    let stripped = value_before_default.trim_end();
    (!stripped.is_empty()).then_some((stripped, true))
}

fn split_top_level_commas(content: &str) -> Vec<&str> {
    let mut entries = Vec::new();
    let mut start = 0usize;
    let mut delimiter_stack = Vec::<char>::new();
    let mut quote = None;
    let mut escaped = false;

    for (index, ch) in content.char_indices() {
        if let Some(quote_ch) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' | '[' => delimiter_stack.push(ch),
            ')' if delimiter_stack.last() == Some(&'(') => {
                delimiter_stack.pop();
            }
            ']' if delimiter_stack.last() == Some(&'[') => {
                delimiter_stack.pop();
            }
            ',' if delimiter_stack.is_empty() => {
                entries.push(&content[start..index]);
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    entries.push(&content[start..]);
    entries
}

fn top_level_colon_index(content: &str) -> Option<usize> {
    let mut delimiter_stack = Vec::<char>::new();
    let mut quote = None;
    let mut escaped = false;

    for (index, ch) in content.char_indices() {
        if let Some(quote_ch) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' | '[' => delimiter_stack.push(ch),
            ')' if delimiter_stack.last() == Some(&'(') => {
                delimiter_stack.pop();
            }
            ']' if delimiter_stack.last() == Some(&'[') => {
                delimiter_stack.pop();
            }
            ':' if delimiter_stack.is_empty() => return Some(index),
            _ => {}
        }
    }
    None
}

fn matching_right_paren(tokens: &[LexedToken], left_paren_index: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(left_paren_index) {
        match token.kind {
            SyntaxKind::LeftParen => depth += 1,
            SyntaxKind::RightParen => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn sass_variable_override_value_is_safe(value: &str) -> bool {
    !value.is_empty()
        && !value
            .chars()
            .any(|ch| matches!(ch, ';' | '{' | '}' | '!' | '$' | '@'))
}

fn canonical_sass_variable_name(name: &str) -> String {
    name.trim()
        .strip_prefix('$')
        .unwrap_or_else(|| name.trim())
        .replace('_', "-")
}

fn sass_variable_names_equal(left: &str, right: &str) -> bool {
    canonical_sass_variable_name(left) == canonical_sass_variable_name(right)
}

fn sass_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')
}

fn token_start(token: &LexedToken) -> usize {
    let start: u32 = token.range.start().into();
    start as usize
}

fn token_end(token: &LexedToken) -> usize {
    let end: u32 = token.range.end().into();
    end as usize
}

pub fn summarize_sass_module_graph_closure(
    edges: &[SassModuleGraphEdgeFactV0],
    configuration_resolver: &impl SassModuleGraphConfigurationResolverV0,
) -> SassModuleGraphClosureSummaryV0 {
    let (graph_closure_edges, cycles) =
        summarize_sass_module_graph_closure_edges(edges, configuration_resolver);
    SassModuleGraphClosureSummaryV0 {
        schema_version: "0",
        product: "omena-semantic.sass-module-graph-closure",
        status: "semanticLayerOwnedClosure",
        module_edge_count: edges.len(),
        graph_closure_edge_count: graph_closure_edges.len(),
        cycle_count: cycles.len(),
        graph_closure_edges,
        cycles,
        capabilities: SassModuleGraphClosureCapabilitiesV0 {
            semantic_layer_owned: true,
            graph_closure_ready: true,
            cycle_detection_ready: true,
            namespace_show_hide_filter_ready: true,
            configured_module_instance_identity_ready: true,
        },
    }
}

pub fn summarize_style_import_reachability(
    target_style_path: &str,
    edges: &[StyleImportReachabilityEdgeFactV0],
) -> StyleImportReachabilitySummaryV0 {
    let mut graph = BTreeMap::<String, BTreeSet<String>>::new();
    for edge in edges {
        graph
            .entry(edge.from_style_path.clone())
            .or_default()
            .insert(edge.target_style_path.clone());
    }

    let (closure_paths, _) =
        collect_hypergraph_transitive_closure_paths(&graph, |style_path: &String| {
            style_path.clone()
        });
    let mut seen = BTreeSet::new();
    let mut reachable_style_paths = Vec::new();
    for path in closure_paths
        .into_iter()
        .filter(|path| path.origin == target_style_path)
    {
        if path.target == target_style_path || !seen.insert(path.target.clone()) {
            continue;
        }
        let order = reachable_style_paths.len();
        reachable_style_paths.push(StyleImportReachabilityFactV0 {
            style_path: path.target,
            distance: path.depth,
            order,
        });
    }

    StyleImportReachabilitySummaryV0 {
        schema_version: "0",
        product: "omena-semantic.style-import-reachability",
        status: "semanticLayerOwnedReachability",
        target_style_path: target_style_path.to_string(),
        edge_count: edges.len(),
        reachable_style_path_count: reachable_style_paths.len(),
        reachable_style_paths,
        capabilities: StyleImportReachabilityCapabilitiesV0 {
            semantic_layer_owned: true,
            transitive_reachability_ready: true,
            stable_distance_ready: true,
            stable_order_ready: true,
        },
    }
}

fn summarize_sass_module_graph_closure_edges(
    edges: &[SassModuleGraphEdgeFactV0],
    configuration_resolver: &impl SassModuleGraphConfigurationResolverV0,
) -> (Vec<SassModuleGraphClosureEdgeV0>, Vec<SassModuleCycleV0>) {
    let mut resolved_edges = edges
        .iter()
        .filter(|edge| edge.status == "resolved" && edge.resolved_style_path.is_some())
        .collect::<Vec<_>>();
    resolved_edges.sort_by_key(|edge| {
        (
            edge.from_style_path.clone(),
            edge.resolved_style_path.clone().unwrap_or_default(),
            edge.edge_kind,
            edge.rule_ordinal,
            edge.source.clone(),
        )
    });

    let mut graph = BTreeMap::<String, BTreeSet<String>>::new();
    let mut metadata_by_step =
        BTreeMap::<(String, String), Vec<SassModuleGraphClosureStepMetadata>>::new();
    for edge in resolved_edges {
        let Some(target_style_path) = edge.resolved_style_path.clone() else {
            continue;
        };
        graph
            .entry(edge.from_style_path.clone())
            .or_default()
            .insert(target_style_path.clone());
        metadata_by_step
            .entry((edge.from_style_path.clone(), target_style_path))
            .or_default()
            .push(SassModuleGraphClosureStepMetadata::from(edge));
    }

    let cycle_paths = collect_directed_graph_cycles(&graph);

    if test_force_rawallpaths_closure() {
        let (closure_paths, _) = collect_hypergraph_transitive_closure_paths_with_mode(
            &graph,
            &mut |style_path: &String| style_path.clone(),
            HypergraphClosureMode::RawAllPaths,
        );
        let closure_edges = sass_module_graph_closure_edges_from_paths(
            closure_paths,
            &metadata_by_step,
            configuration_resolver,
        );
        return finalize_sass_module_graph_closure(closure_edges, cycle_paths);
    }

    let (mut closure_edges, capped) = collect_sass_module_graph_closure_edges_via_worklist(
        &graph,
        &metadata_by_step,
        configuration_resolver,
        SASS_MODULE_CLOSURE_STATE_CAP,
    );
    if capped {
        let (closure_paths, _) =
            collect_hypergraph_transitive_closure_paths(&graph, |style_path: &String| {
                style_path.clone()
            });
        closure_edges = sass_module_graph_closure_edges_from_paths(
            closure_paths,
            &metadata_by_step,
            configuration_resolver,
        );
    }
    finalize_sass_module_graph_closure(closure_edges, cycle_paths)
}

fn finalize_sass_module_graph_closure(
    mut closure_edges: Vec<SassModuleGraphClosureEdgeV0>,
    cycle_paths: Vec<Vec<String>>,
) -> (Vec<SassModuleGraphClosureEdgeV0>, Vec<SassModuleCycleV0>) {
    let mut cycles = cycle_paths
        .into_iter()
        .map(|path| SassModuleCycleV0 { path })
        .collect::<Vec<_>>();
    closure_edges.sort_by_key(|edge| {
        (
            edge.from_style_path.clone(),
            edge.depth,
            edge.target_style_path.clone(),
            edge.edge_kind,
            edge.configuration_signature.clone(),
            edge.module_instance_identity_key
                .clone()
                .unwrap_or_default(),
            edge.path.clone(),
        )
    });
    closure_edges.dedup();
    cycles.sort_by_key(|cycle| cycle.path.clone());
    (closure_edges, cycles)
}

#[derive(Debug, Clone)]
struct SassModuleGraphClosureStepMetadata {
    rule_ordinal: usize,
    edge_kind: &'static str,
    namespace_kind: Option<&'static str>,
    namespace: Option<String>,
    forward_prefix: Option<String>,
    visibility_filter_kind: Option<&'static str>,
    visibility_filter_names: Vec<String>,
    configuration_signature: String,
    configuration_variable_count: usize,
    invalid_configuration_variable_names: Vec<String>,
    module_instance_identity_key: Option<String>,
}

impl From<&SassModuleGraphEdgeFactV0> for SassModuleGraphClosureStepMetadata {
    fn from(edge: &SassModuleGraphEdgeFactV0) -> Self {
        Self {
            rule_ordinal: edge.rule_ordinal,
            edge_kind: edge.edge_kind,
            namespace_kind: edge.namespace_kind,
            namespace: edge.namespace.clone(),
            forward_prefix: edge.forward_prefix.clone(),
            visibility_filter_kind: edge.visibility_filter_kind,
            visibility_filter_names: edge.visibility_filter_names.clone(),
            configuration_signature: edge.configuration_signature.clone(),
            configuration_variable_count: edge.configuration_variable_count,
            invalid_configuration_variable_names: edge.invalid_configuration_variable_names.clone(),
            module_instance_identity_key: edge.module_instance_identity_key.clone(),
        }
    }
}

const SASS_MODULE_CLOSURE_STATE_CAP: usize = 1 << 16;

thread_local! {
    static FORCE_RAWALLPATHS_CLOSURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn test_force_rawallpaths_closure() -> bool {
    FORCE_RAWALLPATHS_CLOSURE.with(std::cell::Cell::get)
}

pub fn with_sass_module_rawallpaths_closure_for_test<R>(body: impl FnOnce() -> R) -> R {
    FORCE_RAWALLPATHS_CLOSURE.with(|cell| cell.set(true));
    let result = body();
    FORCE_RAWALLPATHS_CLOSURE.with(|cell| cell.set(false));
    result
}

fn sass_module_graph_closure_edge(
    origin: &str,
    target: &str,
    depth: usize,
    path: Vec<String>,
    metadata: SassModuleGraphClosureStepMetadata,
) -> SassModuleGraphClosureEdgeV0 {
    SassModuleGraphClosureEdgeV0 {
        from_style_path: origin.to_string(),
        target_style_path: target.to_string(),
        edge_kind: metadata.edge_kind,
        depth,
        path,
        namespace_kind: metadata.namespace_kind,
        namespace: metadata.namespace,
        forward_prefix: metadata.forward_prefix,
        visibility_filter_kind: metadata.visibility_filter_kind,
        visibility_filter_names: metadata.visibility_filter_names,
        configuration_signature: metadata.configuration_signature,
        configuration_variable_count: metadata.configuration_variable_count,
        invalid_configuration_variable_names: metadata.invalid_configuration_variable_names,
        module_instance_identity_key: metadata.module_instance_identity_key,
    }
}

fn sass_module_graph_closure_edges_from_paths(
    closure_paths: Vec<HypergraphClosurePath<String>>,
    metadata_by_step: &BTreeMap<(String, String), Vec<SassModuleGraphClosureStepMetadata>>,
    configuration_resolver: &impl SassModuleGraphConfigurationResolverV0,
) -> Vec<SassModuleGraphClosureEdgeV0> {
    closure_paths
        .into_iter()
        .flat_map(
            |HypergraphClosurePath {
                 origin,
                 target,
                 depth,
                 path_labels,
             }| {
                derive_sass_module_graph_closure_path_metadata(
                    path_labels.as_slice(),
                    metadata_by_step,
                    configuration_resolver,
                )
                .into_iter()
                .map(move |metadata| {
                    sass_module_graph_closure_edge(
                        &origin,
                        &target,
                        depth,
                        path_labels.clone(),
                        metadata,
                    )
                })
                .collect::<Vec<_>>()
            },
        )
        .collect()
}

fn collect_sass_module_graph_closure_edges_via_worklist(
    graph: &BTreeMap<String, BTreeSet<String>>,
    metadata_by_step: &BTreeMap<(String, String), Vec<SassModuleGraphClosureStepMetadata>>,
    configuration_resolver: &impl SassModuleGraphConfigurationResolverV0,
    per_origin_state_cap: usize,
) -> (Vec<SassModuleGraphClosureEdgeV0>, bool) {
    let mut edges = Vec::new();
    for origin in graph.keys() {
        let mut visited = BTreeSet::<(String, BTreeMap<String, String>)>::new();
        let mut pending = VecDeque::<(String, BTreeMap<String, String>, usize, Vec<String>)>::new();
        visited.insert((origin.clone(), BTreeMap::new()));
        pending.push_back((origin.clone(), BTreeMap::new(), 0, vec![origin.clone()]));
        let mut state_count = 0usize;
        while let Some((node, inherited_overrides, depth, path)) = pending.pop_front() {
            state_count += 1;
            if state_count > per_origin_state_cap {
                return (edges, true);
            }
            let Some(targets) = graph.get(node.as_str()) else {
                continue;
            };
            for target in targets {
                if path.contains(target) {
                    continue;
                }
                let Some(step_metadata) = metadata_by_step.get(&(node.clone(), target.clone()))
                else {
                    continue;
                };
                for metadata in step_metadata {
                    let variable_overrides =
                        derive_sass_module_graph_closure_step_variable_overrides(
                            node.as_str(),
                            target.as_str(),
                            metadata,
                            &inherited_overrides,
                            configuration_resolver,
                        );
                    let applied = apply_sass_module_graph_closure_step_configuration(
                        metadata.clone(),
                        target.as_str(),
                        variable_overrides.clone(),
                        configuration_resolver,
                    );
                    let mut edge_path = path.clone();
                    edge_path.push(target.clone());
                    edges.push(sass_module_graph_closure_edge(
                        origin,
                        target,
                        depth + 1,
                        edge_path.clone(),
                        applied,
                    ));
                    let next_state = (target.clone(), variable_overrides);
                    if visited.insert(next_state.clone()) {
                        pending.push_back((target.clone(), next_state.1, depth + 1, edge_path));
                    }
                }
            }
        }
    }
    (edges, false)
}

fn derive_sass_module_graph_closure_path_metadata(
    path_labels: &[String],
    metadata_by_step: &BTreeMap<(String, String), Vec<SassModuleGraphClosureStepMetadata>>,
    configuration_resolver: &impl SassModuleGraphConfigurationResolverV0,
) -> Vec<SassModuleGraphClosureStepMetadata> {
    let mut states = vec![(BTreeMap::<String, String>::new(), None)];

    for step in path_labels.windows(2) {
        let Some(from_style_path) = step.first() else {
            return Vec::new();
        };
        let Some(target_style_path) = step.get(1) else {
            return Vec::new();
        };
        let Some(step_metadata) =
            metadata_by_step.get(&(from_style_path.clone(), target_style_path.clone()))
        else {
            return Vec::new();
        };
        let mut next_states = Vec::new();
        for (inherited_variable_overrides, _) in &states {
            for metadata in step_metadata {
                let variable_overrides = derive_sass_module_graph_closure_step_variable_overrides(
                    from_style_path,
                    target_style_path,
                    metadata,
                    inherited_variable_overrides,
                    configuration_resolver,
                );
                let applied_metadata = apply_sass_module_graph_closure_step_configuration(
                    metadata.clone(),
                    target_style_path,
                    variable_overrides.clone(),
                    configuration_resolver,
                );
                next_states.push((variable_overrides, Some(applied_metadata)));
            }
        }
        states = next_states;
    }

    states
        .into_iter()
        .filter_map(|(_, metadata)| metadata)
        .collect()
}

fn derive_sass_module_graph_closure_step_variable_overrides(
    from_style_path: &str,
    target_style_path: &str,
    metadata: &SassModuleGraphClosureStepMetadata,
    inherited_variable_overrides: &BTreeMap<String, String>,
    configuration_resolver: &impl SassModuleGraphConfigurationResolverV0,
) -> BTreeMap<String, String> {
    match metadata.edge_kind {
        "sassForward" => {
            let configurable_names = configuration_resolver.configurable_names(target_style_path);
            configuration_resolver.forward_effective_variable_overrides(
                SassModuleForwardConfigurationRequestV0 {
                    from_style_path,
                    target_style_path,
                    rule_ordinal: metadata.rule_ordinal,
                    inherited_variable_overrides,
                    forward_prefix: metadata.forward_prefix.as_deref(),
                    visibility_filter_kind: metadata.visibility_filter_kind,
                    visibility_filter_names: &metadata.visibility_filter_names,
                    configurable_names: &configurable_names,
                },
            )
        }
        "sassUse" => {
            configuration_resolver.use_variable_overrides(SassModuleUseConfigurationRequestV0 {
                from_style_path,
                rule_ordinal: metadata.rule_ordinal,
            })
        }
        _ => BTreeMap::new(),
    }
}

fn apply_sass_module_graph_closure_step_configuration(
    mut metadata: SassModuleGraphClosureStepMetadata,
    target_style_path: &str,
    variable_overrides: BTreeMap<String, String>,
    configuration_resolver: &impl SassModuleGraphConfigurationResolverV0,
) -> SassModuleGraphClosureStepMetadata {
    let configurable_names = configuration_resolver.configurable_names(target_style_path);
    metadata.invalid_configuration_variable_names = variable_overrides
        .keys()
        .filter(|name| !configurable_names.contains(*name))
        .cloned()
        .collect();
    metadata.configuration_signature =
        summarize_sass_module_configuration_signature(&variable_overrides);
    metadata.configuration_variable_count = variable_overrides.len();
    metadata.module_instance_identity_key = Some(summarize_sass_module_instance_identity_key(
        target_style_path,
        &variable_overrides,
    ));
    metadata
}

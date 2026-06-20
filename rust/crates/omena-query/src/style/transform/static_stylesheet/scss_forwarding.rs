use super::super::super::stylesheet_evaluation::{
    canonical_static_scss_variable_name, static_scss_variable_names_equal,
};
use super::super::parser_facade::lex_omena_query_omena_parser_style_source;
use super::super::{
    apply_transform_source_replacements, transform_token_end, transform_token_start,
};
use super::{scss_module_rules, scss_variable_overrides};
use crate::OmenaParserStyleDialect;
use omena_syntax::SyntaxKind;
use std::collections::{BTreeMap, BTreeSet};

use scss_variable_overrides::StaticScssModuleVariableOverride;

#[derive(Debug, Clone)]
pub(super) struct StaticScssModuleForwardEvaluation {
    pub(super) source: String,
    pub(super) forward_rule_ordinal: usize,
    pub(super) module_identity_key: String,
    pub(super) module_output_css: String,
    pub(super) variable_exports: BTreeMap<String, String>,
    pub(super) configurable_variable_names: BTreeSet<String>,
}

pub(super) fn inline_static_scss_forward_rules(
    source: &str,
    dialect: OmenaParserStyleDialect,
    forward_evaluations: &[StaticScssModuleForwardEvaluation],
    emitted_module_identity_keys: &mut BTreeSet<String>,
) -> (String, usize) {
    let lexed = lex_omena_query_omena_parser_style_source(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut depth = 0usize;
    let mut forward_rule_ordinal = 0usize;
    let mut index = 0usize;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@forward") =>
            {
                let Some(end_index) =
                    scss_module_rules::static_scss_use_rule_semicolon(tokens, index)
                else {
                    index += 1;
                    continue;
                };
                let start = transform_token_start(&tokens[index]);
                let end = transform_token_end(&tokens[end_index]);
                if let Some(source_name) = scss_module_rules::static_scss_module_rule_source_name(
                    tokens,
                    index + 1,
                    end_index,
                ) {
                    let matching_forward = forward_evaluations.iter().find(|forward| {
                        forward.forward_rule_ordinal == forward_rule_ordinal
                            && forward.source == source_name
                    });
                    forward_rule_ordinal += 1;
                    if let Some(forward) = matching_forward {
                        let replacement = if emitted_module_identity_keys
                            .insert(forward.module_identity_key.clone())
                        {
                            forward.module_output_css.clone()
                        } else {
                            String::new()
                        };
                        replacements.push((start, end, replacement));
                    }
                }
                index = end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    apply_transform_source_replacements(source, replacements)
}

pub(super) fn derive_static_scss_module_forward_variable_overrides_at_ordinal(
    style_source: &str,
    forward_rule_ordinal: usize,
) -> BTreeMap<String, StaticScssModuleVariableOverride> {
    scss_module_rules::static_scss_module_rule_source_at_ordinal(
        style_source,
        "@forward",
        forward_rule_ordinal,
    )
    .map(parse_static_scss_forward_variable_overrides_from_rule)
    .unwrap_or_default()
}

pub(in crate::style::transform) fn derive_static_scss_module_forward_variable_override_values_at_ordinal(
    style_source: &str,
    forward_rule_ordinal: usize,
) -> BTreeMap<String, String> {
    derive_static_scss_module_forward_variable_overrides_at_ordinal(
        style_source,
        forward_rule_ordinal,
    )
    .into_iter()
    .map(|(name, override_entry)| (name, override_entry.value))
    .collect()
}

pub(super) fn derive_static_scss_forward_effective_variable_overrides(
    explicit_variable_overrides: &BTreeMap<String, StaticScssModuleVariableOverride>,
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
                let internal_name = static_scss_forward_internal_variable_name_for_exposed_name(
                    name.as_str(),
                    export_prefix,
                )?;
                static_scss_forward_exposed_variable_is_visible(
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

pub(in crate::style::transform) fn derive_static_scss_module_forward_effective_variable_override_values_for_resolution_at_ordinal(
    style_source: &str,
    forward_rule_ordinal: usize,
    inherited_variable_overrides: &BTreeMap<String, String>,
    export_prefix: Option<&str>,
    visibility_filter_kind: Option<&'static str>,
    visibility_filter_names: &[String],
    configurable_names: &BTreeSet<String>,
) -> BTreeMap<String, String> {
    let explicit_variable_overrides =
        derive_static_scss_module_forward_variable_overrides_at_ordinal(
            style_source,
            forward_rule_ordinal,
        );
    derive_static_scss_forward_effective_variable_overrides(
        &explicit_variable_overrides,
        inherited_variable_overrides,
        export_prefix,
        visibility_filter_kind,
        visibility_filter_names,
        configurable_names,
    )
}

pub(super) fn filter_static_scss_forward_configurable_variable_names(
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
            static_scss_forward_exposed_variable_is_visible(
                exposed_name.as_str(),
                visibility_filter_kind,
                visibility_filter_names,
            )
            .then(|| canonical_static_scss_variable_name(exposed_name.as_str()))
        })
        .collect()
}

fn static_scss_forward_exposed_variable_is_visible(
    exposed_name: &str,
    visibility_filter_kind: Option<&'static str>,
    visibility_filter_names: &[String],
) -> bool {
    match visibility_filter_kind {
        Some("show") => visibility_filter_names
            .iter()
            .any(|filter| static_scss_variable_names_equal(filter, exposed_name)),
        Some("hide") => !visibility_filter_names
            .iter()
            .any(|filter| static_scss_variable_names_equal(filter, exposed_name)),
        _ => true,
    }
}

fn static_scss_forward_internal_variable_name_for_exposed_name(
    exposed_name: &str,
    export_prefix: Option<&str>,
) -> Option<String> {
    let exposed_name = canonical_static_scss_variable_name(exposed_name);
    let Some(export_prefix) = export_prefix else {
        return Some(exposed_name);
    };
    let star_offset = export_prefix.find('*')?;
    let prefix_before_star = canonical_static_scss_variable_name(&export_prefix[..star_offset]);
    let prefix_after_star =
        canonical_static_scss_variable_name(&export_prefix[star_offset + '*'.len_utf8()..]);
    let without_prefix = exposed_name.strip_prefix(prefix_before_star.as_str())?;
    let without_suffix = if prefix_after_star.is_empty() {
        without_prefix
    } else {
        without_prefix.strip_suffix(prefix_after_star.as_str())?
    };
    (!without_suffix.is_empty()).then(|| canonical_static_scss_variable_name(without_suffix))
}

pub(super) fn filter_static_scss_forward_exports(
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
                    .any(|filter| static_scss_variable_names_equal(filter, name))
            })
            .collect(),
        Some("hide") => exports
            .into_iter()
            .filter(|(name, _)| {
                !filter_names
                    .iter()
                    .any(|filter| static_scss_variable_names_equal(filter, name))
            })
            .collect(),
        _ => exports,
    }
}

pub(super) fn prefix_static_scss_forward_exports(
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

pub(super) fn derive_static_scss_forward_export_prefix_at_ordinal(
    style_source: &str,
    forward_rule_ordinal: usize,
) -> Option<String> {
    scss_module_rules::static_scss_module_rule_source_at_ordinal(
        style_source,
        "@forward",
        forward_rule_ordinal,
    )
    .and_then(parse_static_scss_forward_export_prefix_from_rule)
}

fn parse_static_scss_forward_export_prefix_from_rule(rule_source: &str) -> Option<String> {
    let lexed =
        lex_omena_query_omena_parser_style_source(rule_source, OmenaParserStyleDialect::Scss);
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
        .map(transform_token_start)
        .unwrap_or(rule_source.len());
    let prefix = rule_source
        .get(transform_token_start(&tokens[prefix_start_index])..prefix_end)?
        .trim();
    static_scss_forward_export_prefix_is_safe(prefix).then(|| prefix.to_string())
}

fn static_scss_forward_export_prefix_is_safe(prefix: &str) -> bool {
    prefix.contains('*')
        && prefix
            .chars()
            .all(|ch| scss_module_rules::static_scss_identifier_char(ch) || ch == '*')
}

fn parse_static_scss_forward_variable_overrides_from_rule(
    rule_source: &str,
) -> BTreeMap<String, StaticScssModuleVariableOverride> {
    let lexed =
        lex_omena_query_omena_parser_style_source(rule_source, OmenaParserStyleDialect::Scss);
    let tokens = lexed.tokens();
    let Some(with_index) = tokens
        .iter()
        .position(|token| token.text.eq_ignore_ascii_case("with"))
    else {
        return BTreeMap::new();
    };
    let Some(left_paren_index) = tokens[with_index + 1..]
        .iter()
        .position(|token| token.kind == SyntaxKind::LeftParen)
        .map(|offset| with_index + 1 + offset)
    else {
        return BTreeMap::new();
    };
    let Some(right_paren_index) =
        scss_variable_overrides::static_scss_matching_right_paren(tokens, left_paren_index)
    else {
        return BTreeMap::new();
    };
    let start = transform_token_end(&tokens[left_paren_index]);
    let end = transform_token_start(&tokens[right_paren_index]);
    rule_source
        .get(start..end)
        .map(scss_variable_overrides::parse_static_scss_forward_variable_override_list)
        .unwrap_or_default()
}

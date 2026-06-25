//! Vendor-prefix transform analysis and proof candidates.
//!
//! This domain records stale-prefix removal evidence before execution commits
//! to a concrete stylesheet mutation.

use std::sync::OnceLock;

use omena_parser::{LexedToken, StyleDialect};
use omena_syntax::SyntaxKind;

use crate::runtime::lex_cache::lex_cached as lex;

use crate::helpers::{
    blocks::at_rule_block_start,
    declarations::{SimpleDeclarationSlice, collect_simple_declarations_in_block},
    tokens::{matching_right_brace_index, skip_whitespace_tokens, token_end, token_start},
};
use crate::model::TransformVendorPrefixPolicyV0;

const VENDOR_PREFIX_MATRIX_SOURCE: &str = include_str!("../../data/vendor-prefix-matrix.toml");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleVendorPrefixRemovalProofCandidateV0 {
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub unprefixed_peer_span_start: usize,
    pub unprefixed_peer_span_end: usize,
    pub prefixed_property: String,
    pub unprefixed_property: &'static str,
    pub value: String,
    pub important: bool,
}

pub(crate) fn add_css_vendor_prefixes_with_lexer_and_policy(
    source: &str,
    dialect: StyleDialect,
    policy: TransformVendorPrefixPolicyV0,
) -> (String, usize) {
    if policy.is_empty() {
        return (source.to_string(), 0);
    }
    let (source_with_supports_fallbacks, supports_mutation_count) =
        add_supports_vendor_prefix_fallbacks_with_lexer(source, dialect, policy);
    let lexed = lex(&source_with_supports_fallbacks, dialect);
    let tokens = lexed.tokens();
    let mut insertions =
        collect_vendor_prefix_insertions(&source_with_supports_fallbacks, tokens, policy);
    if insertions.is_empty() {
        return (source_with_supports_fallbacks, supports_mutation_count);
    }
    insertions.sort_by_key(|(position, _)| *position);

    let mut output = String::with_capacity(source_with_supports_fallbacks.len());
    let mut cursor = 0;
    for (position, insertion) in &insertions {
        if *position > cursor {
            output.push_str(&source_with_supports_fallbacks[cursor..*position]);
        }
        output.push_str(insertion);
        cursor = *position;
    }
    if cursor < source_with_supports_fallbacks.len() {
        output.push_str(&source_with_supports_fallbacks[cursor..]);
    }

    (output, supports_mutation_count + insertions.len())
}

pub(crate) fn remove_stale_css_vendor_prefixes_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut removals = collect_stale_vendor_prefix_removals(tokens);
    if removals.is_empty() {
        return (source.to_string(), 0);
    }
    removals.sort_by_key(|(start, _)| *start);

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut applied_count = 0usize;
    for (start, end) in removals {
        if start < cursor {
            continue;
        }
        if start > cursor {
            output.push_str(&source[cursor..start]);
        }
        cursor = end;
        applied_count += 1;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, applied_count)
}

pub(crate) fn collect_stale_vendor_prefix_removal_proof_candidates_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> Vec<StaleVendorPrefixRemovalProofCandidateV0> {
    let lexed = lex(source, dialect);
    collect_stale_vendor_prefix_removal_proof_candidates(lexed.tokens())
}

fn collect_stale_vendor_prefix_removals(tokens: &[LexedToken]) -> Vec<(usize, usize)> {
    collect_stale_vendor_prefix_removal_proof_candidates(tokens)
        .into_iter()
        .map(|candidate| (candidate.source_span_start, candidate.source_span_end))
        .collect()
}

fn collect_stale_vendor_prefix_removal_proof_candidates(
    tokens: &[LexedToken],
) -> Vec<StaleVendorPrefixRemovalProofCandidateV0> {
    let mut candidates = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            let declarations = collect_simple_declarations_in_block(tokens, index, close_index);
            for declaration in &declarations {
                let Some((unprefixed_property, _peer)) =
                    exact_unprefixed_peer_for_stale_prefix(declaration, &declarations)
                else {
                    continue;
                };
                candidates.push(StaleVendorPrefixRemovalProofCandidateV0 {
                    source_span_start: declaration.start,
                    source_span_end: declaration.end,
                    unprefixed_peer_span_start: _peer.start,
                    unprefixed_peer_span_end: _peer.end,
                    prefixed_property: declaration.property.clone(),
                    unprefixed_property,
                    value: declaration.value.clone(),
                    important: declaration.important,
                });
            }
            index += 1;
            continue;
        }
        index += 1;
    }

    candidates
}

fn exact_unprefixed_peer_for_stale_prefix<'a>(
    declaration: &SimpleDeclarationSlice,
    declarations: &'a [SimpleDeclarationSlice],
) -> Option<(&'static str, &'a SimpleDeclarationSlice)> {
    let unprefixed_property = unprefixed_property_for_stale_prefix(&declaration.property)?;
    let peer = declarations.iter().find(|candidate| {
        candidate.property == unprefixed_property
            && candidate.value == declaration.value
            && candidate.important == declaration.important
    })?;
    Some((unprefixed_property, peer))
}

fn collect_vendor_prefix_insertions(
    source: &str,
    tokens: &[LexedToken],
    policy: TransformVendorPrefixPolicyV0,
) -> Vec<(usize, String)> {
    let mut insertions = Vec::new();
    if policy.webkit {
        insertions.extend(collect_keyframes_vendor_prefix_insertions(source, tokens));
    }
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            let declarations = collect_simple_declarations_in_block(tokens, index, close_index);
            for declaration in &declarations {
                for prefixed_property in prefixed_properties_for(&declaration.property)
                    .into_iter()
                    .filter(|prefixed_property| policy.allows_prefix(prefixed_property))
                {
                    if declarations
                        .iter()
                        .any(|candidate| candidate.property == prefixed_property)
                    {
                        continue;
                    }
                    insertions.push((
                        declaration.start,
                        format!("{prefixed_property}: {}; ", declaration.value),
                    ));
                }
                for prefixed_value in prefixed_values_for(&declaration.property, &declaration.value)
                    .into_iter()
                    .filter(|prefixed_value| policy.allows_prefix(prefixed_value))
                {
                    if declarations.iter().any(|candidate| {
                        candidate.property == declaration.property
                            && candidate.value.eq_ignore_ascii_case(prefixed_value)
                    }) {
                        continue;
                    }
                    insertions.push((
                        declaration.start,
                        format!("{}: {prefixed_value}; ", declaration.property),
                    ));
                }
                for (prefixed_property, prefixed_value) in
                    prefixed_declarations_for(&declaration.property, &declaration.value)
                        .into_iter()
                        .filter(|(prefixed_property, _)| policy.allows_prefix(prefixed_property))
                {
                    if declarations.iter().any(|candidate| {
                        candidate.property == prefixed_property
                            && candidate
                                .value
                                .eq_ignore_ascii_case(prefixed_value.as_str())
                    }) {
                        continue;
                    }
                    insertions.push((
                        declaration.start,
                        format!("{prefixed_property}: {prefixed_value}; "),
                    ));
                }
            }
            index += 1;
            continue;
        }
        index += 1;
    }

    insertions
}

fn add_supports_vendor_prefix_fallbacks_with_lexer(
    source: &str,
    dialect: StyleDialect,
    policy: TransformVendorPrefixPolicyV0,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::AtKeyword
            && tokens[index].text.eq_ignore_ascii_case("@supports")
            && let Some(block_start) = at_rule_block_start(tokens, index + 1)
        {
            let condition_start = token_end(&tokens[index]);
            let condition_end = token_start(&tokens[block_start]);
            let condition = source[condition_start..condition_end].trim();
            if let Some(fallback_condition) = prefixed_supports_condition_for(condition, policy) {
                replacements.push((
                    condition_start,
                    condition_end,
                    format!(" {fallback_condition} "),
                ));
            }
        }
        index += 1;
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &replacements {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

fn collect_keyframes_vendor_prefix_insertions(
    source: &str,
    tokens: &[LexedToken],
) -> Vec<(usize, String)> {
    let prefixed_names = collect_keyframes_names(tokens, "@-webkit-keyframes");
    let mut insertions = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::AtKeyword
            && tokens[index].text.eq_ignore_ascii_case("@keyframes")
            && let Some(name) = keyframes_name_after(tokens, index)
            && !prefixed_names
                .iter()
                .any(|prefixed_name| prefixed_name == &name.to_ascii_lowercase())
            && let Some(block_start) = at_rule_block_start(tokens, index + 1)
            && let Some(block_end) = matching_right_brace_index(tokens, block_start)
        {
            let start = token_start(&tokens[index]);
            let end = token_end(&tokens[block_end]);
            let original = &source[start..end];
            let prefixed = original.replacen(&tokens[index].text, "@-webkit-keyframes", 1);
            insertions.push((start, format!("{prefixed} ")));
            index = block_end + 1;
            continue;
        }
        index += 1;
    }

    insertions
}

fn collect_keyframes_names(tokens: &[LexedToken], at_keyword: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::AtKeyword
            && tokens[index].text.eq_ignore_ascii_case(at_keyword)
            && let Some(name) = keyframes_name_after(tokens, index)
        {
            names.push(name.to_ascii_lowercase());
        }
        index += 1;
    }
    names
}

fn keyframes_name_after(tokens: &[LexedToken], at_keyword_index: usize) -> Option<&str> {
    let name_index = skip_whitespace_tokens(tokens, at_keyword_index + 1, tokens.len());
    let name_token = tokens.get(name_index)?;
    matches!(name_token.kind, SyntaxKind::Ident | SyntaxKind::String)
        .then_some(name_token.text.as_str())
}

fn prefixed_supports_condition_for(
    condition: &str,
    policy: TransformVendorPrefixPolicyV0,
) -> Option<String> {
    let feature = parse_single_supports_feature_query(condition)?;
    let mut alternatives = vec![condition.trim().to_string()];
    for prefixed_property in prefixed_properties_for(&feature.property)
        .into_iter()
        .filter(|prefixed_property| policy.allows_prefix(prefixed_property))
    {
        alternatives.push(format!("({prefixed_property}: {})", feature.value));
    }
    for prefixed_value in prefixed_values_for(&feature.property, feature.value)
        .into_iter()
        .filter(|prefixed_value| policy.allows_prefix(prefixed_value))
    {
        alternatives.push(format!("({}: {prefixed_value})", feature.property));
    }
    dedupe_case_insensitive(&mut alternatives);
    (alternatives.len() > 1).then(|| format!("({})", alternatives.join(" or ")))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SupportsFeatureQuery<'a> {
    property: String,
    value: &'a str,
}

fn parse_single_supports_feature_query(condition: &str) -> Option<SupportsFeatureQuery<'_>> {
    let inner = strip_balanced_outer_parentheses(condition.trim())?.trim();
    if inner.is_empty()
        || inner
            .split_whitespace()
            .next()
            .is_some_and(|word| matches!(word.to_ascii_lowercase().as_str(), "not" | "selector"))
    {
        return None;
    }
    let colon_index = top_level_colon_index(inner)?;
    let property = inner[..colon_index].trim().to_ascii_lowercase();
    let value = inner[colon_index + 1..].trim();
    if property.is_empty()
        || property.starts_with('-')
        || property.chars().any(char::is_whitespace)
        || value.is_empty()
    {
        return None;
    }
    Some(SupportsFeatureQuery { property, value })
}

fn strip_balanced_outer_parentheses(mut condition: &str) -> Option<&str> {
    loop {
        let trimmed = condition.trim();
        if !(trimmed.starts_with('(') && trimmed.ends_with(')')) {
            return Some(trimmed);
        }
        if !outer_parentheses_wrap(trimmed) {
            return Some(trimmed);
        }
        condition = &trimmed[1..trimmed.len() - 1];
    }
}

fn outer_parentheses_wrap(value: &str) -> bool {
    let mut depth = 0usize;
    for (index, byte) in value.bytes().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => {
                if depth == 0 {
                    return false;
                }
                depth -= 1;
                if depth == 0 && index + 1 < value.len() {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

fn top_level_colon_index(value: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (index, byte) in value.bytes().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b':' if depth == 0 => return Some(index),
            _ => {}
        }
    }
    None
}

fn dedupe_case_insensitive(values: &mut Vec<String>) {
    let mut deduped = Vec::with_capacity(values.len());
    for value in values.drain(..) {
        if deduped
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(&value))
        {
            continue;
        }
        deduped.push(value);
    }
    *values = deduped;
}

#[derive(Debug, Clone, Default)]
struct VendorPrefixMatrixV0 {
    property_rules: Vec<VendorPrefixPropertyRuleV0>,
    value_rules: Vec<VendorPrefixValueRuleV0>,
    declaration_rules: Vec<VendorPrefixDeclarationRuleV0>,
}

#[derive(Debug, Clone, Default)]
struct VendorPrefixPropertyRuleV0 {
    name: String,
    prefixes: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct VendorPrefixValueRuleV0 {
    property: String,
    value: String,
    prefixes: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct VendorPrefixDeclarationRuleV0 {
    property: String,
    entries: Vec<VendorPrefixDeclarationEntryV0>,
}

#[derive(Debug, Clone)]
struct VendorPrefixDeclarationEntryV0 {
    property: String,
    value_transform: String,
}

fn vendor_prefix_matrix() -> &'static VendorPrefixMatrixV0 {
    static MATRIX: OnceLock<VendorPrefixMatrixV0> = OnceLock::new();
    MATRIX.get_or_init(|| parse_vendor_prefix_matrix(VENDOR_PREFIX_MATRIX_SOURCE))
}

fn prefixed_properties_for(property: &str) -> Vec<&'static str> {
    vendor_prefix_matrix()
        .property_rules
        .iter()
        .find(|rule| rule.name == property)
        .map(|rule| rule.prefixes.iter().map(String::as_str).collect())
        .unwrap_or_default()
}

fn unprefixed_property_for_stale_prefix(property: &str) -> Option<&'static str> {
    match property {
        "-moz-appearance" | "-webkit-appearance" => Some("appearance"),
        "-webkit-backdrop-filter" => Some("backdrop-filter"),
        "-webkit-backface-visibility" => Some("backface-visibility"),
        "-webkit-border-image" => Some("border-image"),
        "-webkit-box-decoration-break" => Some("box-decoration-break"),
        "-webkit-clip-path" => Some("clip-path"),
        "-moz-column-count" | "-webkit-column-count" => Some("column-count"),
        "-moz-column-fill" => Some("column-fill"),
        "-moz-column-gap" | "-webkit-column-gap" => Some("column-gap"),
        "-moz-column-rule" | "-webkit-column-rule" => Some("column-rule"),
        "-moz-column-rule-color" | "-webkit-column-rule-color" => Some("column-rule-color"),
        "-moz-column-rule-style" | "-webkit-column-rule-style" => Some("column-rule-style"),
        "-moz-column-rule-width" | "-webkit-column-rule-width" => Some("column-rule-width"),
        "-webkit-column-span" => Some("column-span"),
        "-moz-column-width" | "-webkit-column-width" => Some("column-width"),
        "-moz-columns" | "-webkit-columns" => Some("columns"),
        "-webkit-filter" => Some("filter"),
        "-ms-hyphens" | "-webkit-hyphens" => Some("hyphens"),
        "-webkit-mask-clip" => Some("mask-clip"),
        "-webkit-mask-composite" => Some("mask-composite"),
        "-webkit-mask-image" => Some("mask-image"),
        "-webkit-mask-mode" => Some("mask-mode"),
        "-webkit-mask-origin" => Some("mask-origin"),
        "-webkit-mask-position" => Some("mask-position"),
        "-webkit-mask-repeat" => Some("mask-repeat"),
        "-webkit-mask-size" => Some("mask-size"),
        "-webkit-perspective" => Some("perspective"),
        "-webkit-perspective-origin" => Some("perspective-origin"),
        "-webkit-print-color-adjust" => Some("print-color-adjust"),
        "-moz-tab-size" => Some("tab-size"),
        "-webkit-text-size-adjust" => Some("text-size-adjust"),
        "-ms-touch-action" => Some("touch-action"),
        "-ms-transform" | "-webkit-transform" => Some("transform"),
        "-ms-transform-origin" | "-webkit-transform-origin" => Some("transform-origin"),
        "-webkit-transform-style" => Some("transform-style"),
        "-moz-user-select" | "-ms-user-select" | "-webkit-user-select" => Some("user-select"),
        _ => None,
    }
}

fn prefixed_values_for(property: &str, value: &str) -> Vec<&'static str> {
    let normalized = value.trim().to_ascii_lowercase();
    vendor_prefix_matrix()
        .value_rules
        .iter()
        .find(|rule| rule.property == property && rule.value == normalized)
        .map(|rule| rule.prefixes.iter().map(String::as_str).collect())
        .unwrap_or_default()
}

fn prefixed_declarations_for(property: &str, value: &str) -> Vec<(&'static str, String)> {
    let normalized = value.trim().to_ascii_lowercase();
    vendor_prefix_matrix()
        .declaration_rules
        .iter()
        .find(|rule| rule.property == property)
        .map(|rule| {
            rule.entries
                .iter()
                .filter_map(|entry| {
                    declaration_value_for_transform(entry.value_transform.as_str(), &normalized)
                        .map(|value| (entry.property.as_str(), value))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn declaration_value_for_transform(transform: &str, normalized: &str) -> Option<String> {
    match transform {
        "identity" => Some(normalized.to_string()),
        "flex-align" => flex_align_value(normalized).map(str::to_string),
        "flex-pack" => flex_pack_value(normalized).map(str::to_string),
        "flex-direction-orient" => {
            flex_direction_values(normalized).map(|(orient, _)| orient.to_string())
        }
        "flex-direction-direction" => {
            flex_direction_values(normalized).map(|(_, direction)| direction.to_string())
        }
        _ => None,
    }
}

fn parse_vendor_prefix_matrix(source: &str) -> VendorPrefixMatrixV0 {
    let mut matrix = VendorPrefixMatrixV0::default();
    let mut current_property: Option<VendorPrefixPropertyRuleV0> = None;
    let mut current_value: Option<VendorPrefixValueRuleV0> = None;
    let mut current_declaration: Option<VendorPrefixDeclarationRuleV0> = None;

    for line in vendor_prefix_matrix_logical_lines(source) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match line {
            "[[property]]" => {
                flush_vendor_prefix_matrix_rule(
                    &mut matrix,
                    &mut current_property,
                    &mut current_value,
                    &mut current_declaration,
                );
                current_property = Some(VendorPrefixPropertyRuleV0::default());
                continue;
            }
            "[[value]]" => {
                flush_vendor_prefix_matrix_rule(
                    &mut matrix,
                    &mut current_property,
                    &mut current_value,
                    &mut current_declaration,
                );
                current_value = Some(VendorPrefixValueRuleV0::default());
                continue;
            }
            "[[declaration]]" => {
                flush_vendor_prefix_matrix_rule(
                    &mut matrix,
                    &mut current_property,
                    &mut current_value,
                    &mut current_declaration,
                );
                current_declaration = Some(VendorPrefixDeclarationRuleV0::default());
                continue;
            }
            _ => {}
        }

        let Some((raw_key, raw_value)) = line.split_once('=') else {
            continue;
        };
        let key = raw_key.trim();
        let value = raw_value.trim();
        if let Some(rule) = current_property.as_mut() {
            match key {
                "name" => rule.name = parse_toml_string(value).unwrap_or_default(),
                "prefixes" => rule.prefixes = parse_toml_string_array(value),
                _ => {}
            }
            continue;
        }
        if let Some(rule) = current_value.as_mut() {
            match key {
                "property" => rule.property = parse_toml_string(value).unwrap_or_default(),
                "value" => rule.value = parse_toml_string(value).unwrap_or_default(),
                "prefixes" => rule.prefixes = parse_toml_string_array(value),
                _ => {}
            }
            continue;
        }
        if let Some(rule) = current_declaration.as_mut() {
            match key {
                "property" => rule.property = parse_toml_string(value).unwrap_or_default(),
                "entries" => rule.entries = parse_declaration_entries(value),
                _ => {}
            }
        }
    }

    flush_vendor_prefix_matrix_rule(
        &mut matrix,
        &mut current_property,
        &mut current_value,
        &mut current_declaration,
    );
    matrix
}

fn vendor_prefix_matrix_logical_lines(source: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut pending_array_line: Option<String> = None;

    for line in source.lines().map(str::trim) {
        if let Some(pending) = pending_array_line.as_mut() {
            pending.push(' ');
            pending.push_str(line);
            if line.ends_with(']')
                && let Some(line) = pending_array_line.take()
            {
                lines.push(line);
            }
            continue;
        }

        if line
            .split_once('=')
            .is_some_and(|(_, value)| value.trim().starts_with('[') && !value.trim().ends_with(']'))
        {
            pending_array_line = Some(line.to_string());
            continue;
        }

        lines.push(line.to_string());
    }

    if let Some(line) = pending_array_line {
        lines.push(line);
    }
    lines
}

fn flush_vendor_prefix_matrix_rule(
    matrix: &mut VendorPrefixMatrixV0,
    current_property: &mut Option<VendorPrefixPropertyRuleV0>,
    current_value: &mut Option<VendorPrefixValueRuleV0>,
    current_declaration: &mut Option<VendorPrefixDeclarationRuleV0>,
) {
    if let Some(rule) = current_property.take()
        && !rule.name.is_empty()
    {
        matrix.property_rules.push(rule);
    }
    if let Some(rule) = current_value.take()
        && !(rule.property.is_empty() || rule.value.is_empty())
    {
        matrix.value_rules.push(rule);
    }
    if let Some(rule) = current_declaration.take()
        && !rule.property.is_empty()
    {
        matrix.declaration_rules.push(rule);
    }
}

fn parse_toml_string_array(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    let Some(inner) = trimmed
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Vec::new();
    };
    inner.split(',').filter_map(parse_toml_string).collect()
}

fn parse_toml_string(value: &str) -> Option<String> {
    value
        .trim()
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(str::to_string)
}

fn parse_declaration_entries(value: &str) -> Vec<VendorPrefixDeclarationEntryV0> {
    parse_toml_string_array(value)
        .into_iter()
        .filter_map(|entry| {
            let (property, value_transform) = entry.split_once(':')?;
            Some(VendorPrefixDeclarationEntryV0 {
                property: property.to_string(),
                value_transform: value_transform.to_string(),
            })
        })
        .collect()
}

fn flex_align_value(value: &str) -> Option<&'static str> {
    match value {
        "flex-start" | "start" => Some("start"),
        "flex-end" | "end" => Some("end"),
        "center" => Some("center"),
        "baseline" => Some("baseline"),
        "stretch" => Some("stretch"),
        _ => None,
    }
}

fn flex_pack_value(value: &str) -> Option<&'static str> {
    match value {
        "flex-start" | "start" | "left" => Some("start"),
        "flex-end" | "end" | "right" => Some("end"),
        "center" => Some("center"),
        "space-between" => Some("justify"),
        "space-around" | "space-evenly" => Some("distribute"),
        _ => None,
    }
}

fn flex_direction_values(value: &str) -> Option<(&'static str, &'static str)> {
    match value {
        "row" => Some(("horizontal", "normal")),
        "row-reverse" => Some(("horizontal", "reverse")),
        "column" => Some(("vertical", "normal")),
        "column-reverse" => Some(("vertical", "reverse")),
        _ => None,
    }
}

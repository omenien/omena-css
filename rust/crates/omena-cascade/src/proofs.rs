//! Conservative proof builders for cascade-sensitive rewrites.
//!
//! The routines here do not rewrite CSS directly. They produce acceptance or
//! blocker witnesses for shorthand combination, static supports evaluation, and
//! scope/layer flattening so transform passes can remain proof-driven.

use crate::{
    BoxLonghandInputV0, LayerFlattenInputV0, LayerFlattenProofV0, LonghandMergeInputV0,
    LonghandMergeProofV0, ScopeFlattenInputV0, ScopeFlattenProofV0, ShorthandCombinationProofV0,
    StaticSupportsAssumptionV0, StaticSupportsEvalVerdictV0, StaticSupportsEvalWitnessV0,
};

pub fn prove_longhand_merge(
    shorthand_property: &str,
    expected_longhands: &[&str],
    longhands: &[LonghandMergeInputV0],
) -> LonghandMergeProofV0 {
    if expected_longhands.is_empty() {
        return shorthand_combination_proof(
            shorthand_property,
            false,
            Some("unsupported shorthand property"),
            longhands,
            "",
        );
    }

    if longhands.len() != expected_longhands.len() {
        return shorthand_combination_proof(
            shorthand_property,
            false,
            Some("incomplete longhand set"),
            longhands,
            "",
        );
    }

    if longhands
        .iter()
        .zip(expected_longhands.iter())
        .any(|(actual, expected)| actual.property != *expected)
    {
        return shorthand_combination_proof(
            shorthand_property,
            false,
            Some("longhands are not in canonical merge order"),
            longhands,
            "",
        );
    }

    if longhands.iter().any(|longhand| longhand.important) {
        return shorthand_combination_proof(
            shorthand_property,
            false,
            Some("important longhands require explicit cascade equivalence proof"),
            longhands,
            "",
        );
    }

    if longhands.iter().any(|longhand| longhand.value.is_empty()) {
        return shorthand_combination_proof(
            shorthand_property,
            false,
            Some("empty longhand value"),
            longhands,
            "",
        );
    }

    if longhands
        .windows(2)
        .any(|pair| pair[1].source_order != pair[0].source_order + 1)
    {
        return shorthand_combination_proof(
            shorthand_property,
            false,
            Some("intervening declaration may change cascade outcome"),
            longhands,
            "",
        );
    }

    shorthand_combination_proof(
        shorthand_property,
        true,
        None,
        longhands,
        "longhands are adjacent, non-important, and in canonical merge order",
    )
}

pub fn prove_box_shorthand_combination(
    shorthand_property: &str,
    longhands: &[BoxLonghandInputV0],
) -> ShorthandCombinationProofV0 {
    let expected = match box_shorthand_longhands(shorthand_property) {
        Some(expected) => expected,
        None => {
            return shorthand_combination_proof(
                shorthand_property,
                false,
                Some("unsupported shorthand property"),
                longhands,
                "",
            );
        }
    };
    prove_longhand_merge(shorthand_property, &expected, longhands)
}

pub fn evaluate_static_supports_condition(
    condition: &str,
    assumption: StaticSupportsAssumptionV0,
) -> StaticSupportsEvalWitnessV0 {
    let normalized_condition = normalize_ascii_whitespace(condition);
    let (verdict, reason) = match assumption {
        StaticSupportsAssumptionV0::ModernBrowser => {
            evaluate_modern_static_supports_condition(&normalized_condition)
        }
    };

    StaticSupportsEvalWitnessV0 {
        schema_version: "0",
        product: "omena-cascade.supports-static-eval",
        condition: normalized_condition,
        assumption,
        verdict,
        reason,
        provenance_preserved: verdict != StaticSupportsEvalVerdictV0::Unknown,
    }
}

fn evaluate_modern_static_supports_condition(
    condition: &str,
) -> (StaticSupportsEvalVerdictV0, &'static str) {
    if let Some(inner) = strip_supports_grouping_parens(condition) {
        return evaluate_modern_static_supports_condition(inner);
    }

    if let Some(parts) = parse_static_supports_logical_parts(condition, "or") {
        let verdicts = parts
            .iter()
            .map(|part| evaluate_modern_static_supports_condition(part).0)
            .collect::<Vec<_>>();
        if verdicts.contains(&StaticSupportsEvalVerdictV0::AlwaysTrue) {
            return (
                StaticSupportsEvalVerdictV0::AlwaysTrue,
                "modern-browser assumption accepts a true simple declaration inside disjunction",
            );
        }
        if verdicts
            .iter()
            .all(|verdict| *verdict == StaticSupportsEvalVerdictV0::AlwaysFalse)
        {
            return (
                StaticSupportsEvalVerdictV0::AlwaysFalse,
                "modern-browser assumption rejects all simple declarations inside disjunction",
            );
        }
        return (
            StaticSupportsEvalVerdictV0::Unknown,
            "unsupported supports disjunction member",
        );
    }

    if let Some(parts) = parse_static_supports_logical_parts(condition, "and") {
        let verdicts = parts
            .iter()
            .map(|part| evaluate_modern_static_supports_condition(part).0)
            .collect::<Vec<_>>();
        if verdicts.contains(&StaticSupportsEvalVerdictV0::AlwaysFalse) {
            return (
                StaticSupportsEvalVerdictV0::AlwaysFalse,
                "modern-browser assumption rejects a false simple declaration inside conjunction",
            );
        }
        if verdicts
            .iter()
            .all(|verdict| *verdict == StaticSupportsEvalVerdictV0::AlwaysTrue)
        {
            return (
                StaticSupportsEvalVerdictV0::AlwaysTrue,
                "modern-browser assumption accepts all simple declarations inside conjunction",
            );
        }
        return (
            StaticSupportsEvalVerdictV0::Unknown,
            "unsupported supports conjunction member",
        );
    }

    if let Some(inner) = parse_static_supports_not_condition(condition) {
        return match evaluate_modern_static_supports_condition(inner).0 {
            StaticSupportsEvalVerdictV0::AlwaysTrue => (
                StaticSupportsEvalVerdictV0::AlwaysFalse,
                "modern-browser assumption rejects negated supported condition queries",
            ),
            StaticSupportsEvalVerdictV0::AlwaysFalse => (
                StaticSupportsEvalVerdictV0::AlwaysTrue,
                "modern-browser assumption accepts negated unsupported condition queries",
            ),
            StaticSupportsEvalVerdictV0::Unknown => (
                StaticSupportsEvalVerdictV0::Unknown,
                "unsupported negated supports condition shape",
            ),
        };
    }

    if let Some(selector) = parse_supports_selector_condition(condition) {
        return match evaluate_modern_supports_selector_condition(selector) {
            StaticSupportsEvalVerdictV0::AlwaysTrue => (
                StaticSupportsEvalVerdictV0::AlwaysTrue,
                "modern-browser assumption accepts selector() feature queries",
            ),
            StaticSupportsEvalVerdictV0::AlwaysFalse => (
                StaticSupportsEvalVerdictV0::AlwaysFalse,
                "modern-browser assumption rejects known obsolete selector() feature queries",
            ),
            StaticSupportsEvalVerdictV0::Unknown => (
                StaticSupportsEvalVerdictV0::Unknown,
                "unsupported selector() feature query",
            ),
        };
    }

    if let Some((function_name, argument)) = parse_supports_font_condition(condition) {
        return match evaluate_modern_supports_font_condition(function_name, argument) {
            StaticSupportsEvalVerdictV0::AlwaysTrue => (
                StaticSupportsEvalVerdictV0::AlwaysTrue,
                "modern-browser assumption accepts known font feature queries",
            ),
            StaticSupportsEvalVerdictV0::AlwaysFalse => (
                StaticSupportsEvalVerdictV0::AlwaysFalse,
                "modern-browser assumption rejects known obsolete font feature queries",
            ),
            StaticSupportsEvalVerdictV0::Unknown => (
                StaticSupportsEvalVerdictV0::Unknown,
                "unsupported font feature query",
            ),
        };
    }

    if let Some((property, value)) = parse_simple_supports_declaration(condition) {
        return match evaluate_modern_simple_supports_declaration(property, value) {
            StaticSupportsEvalVerdictV0::AlwaysTrue => (
                StaticSupportsEvalVerdictV0::AlwaysTrue,
                "modern-browser assumption accepts simple declaration feature queries",
            ),
            StaticSupportsEvalVerdictV0::AlwaysFalse => (
                StaticSupportsEvalVerdictV0::AlwaysFalse,
                "modern-browser assumption rejects known obsolete declaration feature queries",
            ),
            StaticSupportsEvalVerdictV0::Unknown => (
                StaticSupportsEvalVerdictV0::Unknown,
                "unsupported simple declaration feature query",
            ),
        };
    }

    (
        StaticSupportsEvalVerdictV0::Unknown,
        "unsupported supports condition shape",
    )
}

pub fn prove_scope_flatten_candidate(input: ScopeFlattenInputV0) -> ScopeFlattenProofV0 {
    let blocked_reason = if input.limit_selector.is_some() {
        Some("scope limit selector cannot be encoded by the conservative flatten predicate")
    } else if input.root_selector.trim() != ":root" {
        Some("non-root scope flattening requires selector/proximity equivalence proof")
    } else if input.peer_scope_count > 0 {
        Some("peer scopes may change scope-proximity cascade ordering")
    } else if input.competing_unscoped_rule_count > 0 {
        Some("unscoped competitors may observe changed scope-proximity ordering")
    } else if input.inside_layer {
        Some("layer plus scope composition requires product cascade proof")
    } else {
        None
    };
    let accepted = blocked_reason.is_none();
    ScopeFlattenProofV0 {
        schema_version: "0",
        product: "omena-cascade.scope-flatten-proof",
        accepted,
        blocked_reason,
        root_selector: input.root_selector,
        provenance_preserved: accepted,
        cascade_safe_witness: if accepted {
            "root scope without limit, peer scopes, unscoped competition, or layer context"
        } else {
            "scope proximity cannot be erased by local syntax alone"
        }
        .to_string(),
    }
}

pub fn prove_layer_flatten_candidate(input: LayerFlattenInputV0) -> LayerFlattenProofV0 {
    let blocked_reason = if !input.closed_bundle {
        Some("layer flattening requires a closed bundle witness")
    } else if input.peer_layer_count > 0 {
        Some("peer layers may change layer-rank cascade ordering")
    } else if input.unlayered_rule_count > 0 {
        Some("unlayered rules compete differently from layered normal rules")
    } else if input.important_declaration_count > 0 {
        Some("important declarations invert layer ordering")
    } else {
        None
    };
    let accepted = blocked_reason.is_none();
    LayerFlattenProofV0 {
        schema_version: "0",
        product: "omena-cascade.layer-flatten-proof",
        accepted,
        blocked_reason,
        layer_name: input.layer_name,
        provenance_preserved: accepted,
        cascade_safe_witness: if accepted {
            "closed bundle with a single non-important layer and no unlayered competitors"
        } else {
            "layer rank cannot be erased by local syntax alone"
        }
        .to_string(),
    }
}

fn parse_static_supports_not_condition(condition: &str) -> Option<&str> {
    supports_keyword_at(condition, 0, "not")
        .then(|| condition["not".len()..].trim())
        .filter(|inner| !inner.is_empty())
}

fn parse_simple_supports_declaration(condition: &str) -> Option<(&str, &str)> {
    let inner = condition.strip_prefix('(')?.strip_suffix(')')?.trim();
    let (property, value) = inner.split_once(':')?;
    let property = property.trim();
    let value = value.trim();
    if property.is_empty()
        || value.is_empty()
        || property.contains(|ch: char| !is_supports_declaration_token_char(ch))
        || value.contains(['{', '}', ';'])
        || !supports_declaration_value_has_balanced_parentheses(value)
    {
        return None;
    }
    Some((property, value))
}

fn supports_declaration_value_has_balanced_parentheses(value: &str) -> bool {
    let mut depth = 0usize;
    for ch in value.chars() {
        match ch {
            '(' => depth += 1,
            ')' => {
                let Some(next_depth) = depth.checked_sub(1) else {
                    return false;
                };
                depth = next_depth;
            }
            _ => {}
        }
    }
    depth == 0
}

fn parse_supports_selector_condition(condition: &str) -> Option<&str> {
    parse_supports_function_argument(condition, "selector")
}

fn parse_supports_font_condition(condition: &str) -> Option<(&'static str, &str)> {
    if let Some(argument) = parse_supports_function_argument(condition, "font-tech") {
        return Some(("font-tech", argument));
    }
    parse_supports_function_argument(condition, "font-format")
        .map(|argument| ("font-format", argument))
}

fn parse_supports_function_argument<'a>(
    condition: &'a str,
    function_name: &str,
) -> Option<&'a str> {
    let candidate = condition.get(..function_name.len())?;
    if !candidate.eq_ignore_ascii_case(function_name)
        || condition[function_name.len()..]
            .chars()
            .next()
            .is_some_and(is_supports_ident_char)
    {
        return None;
    }
    let arguments = condition[function_name.len()..].trim_start();
    let inner = arguments.strip_prefix('(')?.strip_suffix(')')?.trim();
    (!inner.is_empty()
        && supports_outer_parens_wrap_entire_condition(arguments)
        && !inner.contains(['{', '}', ';']))
    .then_some(inner)
}

fn strip_supports_grouping_parens(condition: &str) -> Option<&str> {
    let inner = condition.strip_prefix('(')?.strip_suffix(')')?.trim();
    if parse_simple_supports_declaration(condition).is_some()
        || !supports_outer_parens_wrap_entire_condition(condition)
        || inner.is_empty()
    {
        return None;
    }
    Some(inner)
}

fn supports_outer_parens_wrap_entire_condition(condition: &str) -> bool {
    let mut depth = 0usize;
    for (index, ch) in condition.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 && index + ch.len_utf8() < condition.len() {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

fn parse_static_supports_logical_parts<'a>(
    condition: &'a str,
    operator: &str,
) -> Option<Vec<&'a str>> {
    match operator {
        "and" | "or" => {}
        _ => return None,
    }
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut part_start = 0usize;
    let mut index = 0usize;

    while index < condition.len() {
        let ch = condition[index..].chars().next()?;
        match ch {
            '(' => {
                depth += 1;
                index += ch.len_utf8();
            }
            ')' => {
                depth = depth.saturating_sub(1);
                index += ch.len_utf8();
            }
            _ if depth == 0 && supports_keyword_at(condition, index, operator) => {
                parts.push(condition[part_start..index].trim());
                index += operator.len();
                part_start = index;
            }
            _ => index += ch.len_utf8(),
        }
    }

    if parts.is_empty() {
        return None;
    }
    parts.push(condition[part_start..].trim());
    parts.iter().all(|part| !part.is_empty()).then_some(parts)
}

fn evaluate_modern_simple_supports_declaration(
    property: &str,
    value: &str,
) -> StaticSupportsEvalVerdictV0 {
    if starts_with_ascii_case_insensitive(property, "-ms-")
        || starts_with_ascii_case_insensitive(value, "-ms-")
    {
        StaticSupportsEvalVerdictV0::AlwaysFalse
    } else {
        StaticSupportsEvalVerdictV0::AlwaysTrue
    }
}

fn evaluate_modern_supports_selector_condition(selector: &str) -> StaticSupportsEvalVerdictV0 {
    if selector.to_ascii_lowercase().contains("-ms-") {
        StaticSupportsEvalVerdictV0::AlwaysFalse
    } else {
        StaticSupportsEvalVerdictV0::AlwaysTrue
    }
}

fn evaluate_modern_supports_font_condition(
    function_name: &str,
    argument: &str,
) -> StaticSupportsEvalVerdictV0 {
    let Some(argument) = normalize_supports_font_feature_argument(argument) else {
        return StaticSupportsEvalVerdictV0::Unknown;
    };

    match (function_name, argument.as_str()) {
        (
            "font-tech",
            "color-cbdt" | "color-colrv0" | "color-colrv1" | "color-sbix" | "color-svg"
            | "features-aat" | "features-graphite" | "features-opentype" | "palettes"
            | "variations",
        ) => StaticSupportsEvalVerdictV0::AlwaysTrue,
        ("font-format", "collection" | "opentype" | "truetype" | "woff" | "woff2") => {
            StaticSupportsEvalVerdictV0::AlwaysTrue
        }
        ("font-format", "embedded-opentype" | "svg") => StaticSupportsEvalVerdictV0::AlwaysFalse,
        (_, argument) if argument.starts_with("-ms-") => StaticSupportsEvalVerdictV0::AlwaysFalse,
        _ => StaticSupportsEvalVerdictV0::Unknown,
    }
}

fn normalize_supports_font_feature_argument(argument: &str) -> Option<String> {
    let normalized = normalize_ascii_whitespace(argument).to_ascii_lowercase();
    let unquoted = normalized
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            normalized
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(&normalized);

    (!unquoted.is_empty()
        && !unquoted.contains(|ch: char| {
            ch.is_ascii_whitespace() || matches!(ch, '(' | ')' | '{' | '}' | ';' | ',')
        }))
    .then(|| unquoted.to_string())
}

fn is_supports_declaration_token_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_')
}

fn starts_with_ascii_case_insensitive(text: &str, prefix: &str) -> bool {
    text.get(..prefix.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(prefix))
}

fn supports_keyword_at(text: &str, index: usize, keyword: &str) -> bool {
    text[index..]
        .get(..keyword.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(keyword))
        && text[..index]
            .chars()
            .next_back()
            .is_none_or(|ch| !is_supports_ident_char(ch))
        && text[index + keyword.len()..]
            .chars()
            .next()
            .is_none_or(|ch| !is_supports_ident_char(ch))
}

fn is_supports_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-'
}

fn normalize_ascii_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn box_shorthand_longhands(shorthand_property: &str) -> Option<[&'static str; 4]> {
    match shorthand_property {
        "margin" => Some(["margin-top", "margin-right", "margin-bottom", "margin-left"]),
        "padding" => Some([
            "padding-top",
            "padding-right",
            "padding-bottom",
            "padding-left",
        ]),
        "border-color" => Some([
            "border-top-color",
            "border-right-color",
            "border-bottom-color",
            "border-left-color",
        ]),
        "border-style" => Some([
            "border-top-style",
            "border-right-style",
            "border-bottom-style",
            "border-left-style",
        ]),
        "border-width" => Some([
            "border-top-width",
            "border-right-width",
            "border-bottom-width",
            "border-left-width",
        ]),
        "scroll-margin" => Some([
            "scroll-margin-top",
            "scroll-margin-right",
            "scroll-margin-bottom",
            "scroll-margin-left",
        ]),
        "scroll-padding" => Some([
            "scroll-padding-top",
            "scroll-padding-right",
            "scroll-padding-bottom",
            "scroll-padding-left",
        ]),
        _ => None,
    }
}

fn shorthand_combination_proof(
    shorthand_property: &str,
    accepted: bool,
    blocked_reason: Option<&'static str>,
    longhands: &[BoxLonghandInputV0],
    witness: &str,
) -> ShorthandCombinationProofV0 {
    ShorthandCombinationProofV0 {
        schema_version: "0",
        product: "omena-cascade.shorthand-combination-proof",
        shorthand_property: shorthand_property.to_string(),
        accepted,
        blocked_reason,
        ordered_longhand_properties: longhands
            .iter()
            .map(|longhand| longhand.property.clone())
            .collect(),
        provenance_preserved: accepted,
        cascade_safe_witness: witness.to_string(),
    }
}

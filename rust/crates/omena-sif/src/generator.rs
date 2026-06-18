use omena_value_lattice::canonicalize_css_value;

use crate::{
    OmenaLifExportsV1, OmenaLifLessDetachedRulesetExportV1, OmenaLifLessMixinExportV1,
    OmenaLifLessVariableExportV1, OmenaSifCallableExportV1, OmenaSifExportsV1,
    OmenaSifForwardExportV1, OmenaSifGeneratorV1, OmenaSifParameterV1, OmenaSifPlaceholderExportV1,
    OmenaSifSourceSyntaxV1, OmenaSifSourceV1, OmenaSifV1, OmenaSifVariableExportV1,
};

pub const OMENA_STATIC_SIF_GENERATOR_NAME_V1: &str = "omena-sifgen-static";
pub const OMENA_STATIC_SIF_GENERATOR_VERSION_V1: &str = "0.1.0";
pub const OMENA_STATIC_SIF_GENERATOR_TOOLCHAIN_ID_V1: &str = "omena-sifgen-static@0.1.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmenaSifStaticGeneratorInputV1<'a> {
    pub canonical_url: &'a str,
    pub source: &'a str,
    pub syntax: OmenaSifSourceSyntaxV1,
}

pub fn generate_static_omena_sif_v1(
    input: OmenaSifStaticGeneratorInputV1<'_>,
) -> Result<OmenaSifV1, serde_json::Error> {
    let exports = parse_static_sass_exports_v1(input.source);
    OmenaSifV1::from_static_exports(
        input.canonical_url,
        default_static_omena_sif_generator_v1(),
        OmenaSifSourceV1 {
            syntax: input.syntax,
        },
        exports,
        Vec::new(),
        input.source.as_bytes(),
    )
}

pub fn generate_static_omena_lif_exports_v1(
    input: OmenaSifStaticGeneratorInputV1<'_>,
) -> OmenaLifExportsV1 {
    parse_static_lif_exports_v1(input.source, input.syntax)
}

pub fn default_static_omena_sif_generator_v1() -> OmenaSifGeneratorV1 {
    OmenaSifGeneratorV1 {
        name: OMENA_STATIC_SIF_GENERATOR_NAME_V1.to_string(),
        version: OMENA_STATIC_SIF_GENERATOR_VERSION_V1.to_string(),
        toolchain_id: OMENA_STATIC_SIF_GENERATOR_TOOLCHAIN_ID_V1.to_string(),
    }
}

pub fn parse_static_lif_exports_v1(
    source: &str,
    syntax: OmenaSifSourceSyntaxV1,
) -> OmenaLifExportsV1 {
    match syntax {
        OmenaSifSourceSyntaxV1::Css => OmenaLifExportsV1::default(),
        OmenaSifSourceSyntaxV1::Scss | OmenaSifSourceSyntaxV1::Sass => {
            OmenaLifExportsV1::from_sif_exports(parse_static_sass_exports_v1(source))
        }
        OmenaSifSourceSyntaxV1::Less => parse_static_less_lif_exports_v1(source),
    }
}

pub fn parse_static_sass_exports_v1(source: &str) -> OmenaSifExportsV1 {
    let mut exports = OmenaSifExportsV1::default();

    for statement in split_top_level_sass_statements(source) {
        let statement = statement.trim();
        if statement.is_empty() {
            continue;
        }
        if let Some(variable) = parse_static_sass_variable_export(statement) {
            exports.variables.push(variable);
            continue;
        }
        if let Some(mixin) = parse_static_sass_callable_export(statement, "@mixin", true) {
            exports.mixins.push(mixin);
            continue;
        }
        if let Some(function) = parse_static_sass_callable_export(statement, "@function", false) {
            exports.functions.push(function);
            continue;
        }
        if let Some(placeholder) = parse_static_sass_placeholder_export(statement) {
            exports.placeholders.push(placeholder);
            continue;
        }
        if let Some(forward) = parse_static_sass_forward_export(statement) {
            exports.forwards.push(forward);
        }
    }

    exports
        .variables
        .sort_by(|left, right| left.name.cmp(&right.name));
    exports
        .mixins
        .sort_by(|left, right| left.name.cmp(&right.name));
    exports
        .functions
        .sort_by(|left, right| left.name.cmp(&right.name));
    exports
        .placeholders
        .sort_by(|left, right| left.name.cmp(&right.name));
    exports
        .forwards
        .sort_by(|left, right| left.canonical_url.cmp(&right.canonical_url));
    exports
}

pub fn parse_static_less_lif_exports_v1(source: &str) -> OmenaLifExportsV1 {
    let mut exports = OmenaLifExportsV1::default();

    for statement in split_top_level_sass_statements(source) {
        let statement = statement.trim();
        if statement.is_empty() {
            continue;
        }
        if let Some(detached_ruleset) = parse_static_less_detached_ruleset_export(statement) {
            exports.less_detached_rulesets.push(detached_ruleset);
            continue;
        }
        if let Some(variable) = parse_static_less_variable_export(statement) {
            exports.less_variables.push(variable);
            continue;
        }
        if let Some(mixin) = parse_static_less_mixin_export(statement) {
            exports.less_mixins.push(mixin);
        }
    }

    exports
        .less_variables
        .sort_by(|left, right| left.name.cmp(&right.name));
    exports
        .less_mixins
        .sort_by(|left, right| left.name.cmp(&right.name));
    exports.less_detached_rulesets.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then(left.member_names.cmp(&right.member_names))
    });
    exports
}

fn split_top_level_sass_statements(source: &str) -> Vec<&str> {
    let mut statements = Vec::new();
    let mut start = 0usize;
    let mut brace_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escape = false;

    for (index, character) in source.char_indices() {
        if let Some(quote_character) = quote {
            if escape {
                escape = false;
            } else if character == '\\' {
                escape = true;
            } else if character == quote_character {
                quote = None;
            }
            continue;
        }

        if character == '"' || character == '\'' {
            quote = Some(character);
            continue;
        }

        match character {
            '{' => brace_depth = brace_depth.saturating_add(1),
            '}' => {
                brace_depth = brace_depth.saturating_sub(1);
                if brace_depth == 0 {
                    let end = index + character.len_utf8();
                    push_trimmed_statement(&mut statements, source, start, end);
                    start = end;
                }
            }
            ';' if brace_depth == 0 => {
                let end = index + character.len_utf8();
                push_trimmed_statement(&mut statements, source, start, end);
                start = end;
            }
            _ => {}
        }
    }

    if start < source.len() {
        push_trimmed_statement(&mut statements, source, start, source.len());
    }

    statements
}

fn push_trimmed_statement<'a>(
    statements: &mut Vec<&'a str>,
    source: &'a str,
    start: usize,
    end: usize,
) {
    let Some(statement) = source.get(start..end) else {
        return;
    };
    let statement = statement.trim();
    if !statement.is_empty() {
        statements.push(statement);
    }
}

fn parse_static_sass_variable_export(statement: &str) -> Option<OmenaSifVariableExportV1> {
    let statement = strip_statement_semicolon(statement);
    let colon_index = top_level_character_index(statement, ':')?;
    let name = statement.get(..colon_index)?.trim();
    if !is_static_sass_variable_name(name) {
        return None;
    }
    let raw_value = statement.get(colon_index + 1..)?.trim();
    let defaulted = raw_value.contains("!default");
    let value_repr = canonical_sif_value_repr(raw_value.replace("!default", "").trim());

    Some(OmenaSifVariableExportV1 {
        name: name.to_string(),
        defaulted,
        value_repr: if value_repr.is_empty() {
            None
        } else {
            Some(value_repr)
        },
    })
}

fn parse_static_less_variable_export(statement: &str) -> Option<OmenaLifLessVariableExportV1> {
    let statement = strip_statement_semicolon(statement);
    let colon_index = top_level_character_index(statement, ':')?;
    let name = statement.get(..colon_index)?.trim();
    if !is_static_less_variable_name(name) {
        return None;
    }
    let raw_value = statement.get(colon_index + 1..)?.trim();
    if raw_value.starts_with('{') {
        return None;
    }
    let value_repr = canonical_sif_value_repr(raw_value);

    Some(OmenaLifLessVariableExportV1 {
        name: name.to_string(),
        value_repr: if value_repr.is_empty() {
            None
        } else {
            Some(value_repr)
        },
    })
}

fn parse_static_less_detached_ruleset_export(
    statement: &str,
) -> Option<OmenaLifLessDetachedRulesetExportV1> {
    let statement = strip_statement_semicolon(statement);
    let colon_index = top_level_character_index(statement, ':')?;
    let name = statement.get(..colon_index)?.trim();
    if !is_static_less_variable_name(name) {
        return None;
    }
    let raw_value = statement.get(colon_index + 1..)?.trim();
    let body = raw_value.strip_prefix('{')?.trim();
    let body = body.strip_suffix('}')?.trim();
    let mut member_names = split_top_level_sass_statements(body)
        .into_iter()
        .filter_map(|member| {
            let member = strip_statement_semicolon(member.trim());
            let colon_index = top_level_character_index(member, ':')?;
            let name = member.get(..colon_index)?.trim();
            if is_static_less_map_member_name(name) {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    member_names.sort();
    member_names.dedup();

    Some(OmenaLifLessDetachedRulesetExportV1 {
        name: name.to_string(),
        member_names,
    })
}

fn parse_static_less_mixin_export(statement: &str) -> Option<OmenaLifLessMixinExportV1> {
    let statement = statement.trim_start();
    let header_end = statement.find('{').unwrap_or(statement.len());
    let header = statement.get(..header_end)?.trim();
    let guarded = header.contains(" when ");
    let header_before_guard = header.split(" when ").next()?.trim();
    let open_paren = header_before_guard.find('(');
    let (name, parameters) = if let Some(open_paren) = open_paren {
        let close_paren = matching_close_paren_index(header_before_guard, open_paren)?;
        let name = header_before_guard.get(..open_paren)?.trim();
        let raw_parameters = header_before_guard.get(open_paren + 1..close_paren)?;
        (name, parse_static_less_parameters(raw_parameters))
    } else {
        (
            header_before_guard
                .split_whitespace()
                .next()
                .unwrap_or(header_before_guard),
            Vec::new(),
        )
    };
    if !is_static_less_mixin_name(name) {
        return None;
    }

    Some(OmenaLifLessMixinExportV1 {
        name: name.to_string(),
        parameters,
        guarded,
    })
}

fn canonical_sif_value_repr(value: &str) -> String {
    canonicalize_css_value(value)
        .map(|value| value.serialized)
        .unwrap_or_else(|| value.to_string())
}

fn parse_static_sass_callable_export(
    statement: &str,
    keyword: &str,
    detect_content: bool,
) -> Option<OmenaSifCallableExportV1> {
    let statement = statement.trim_start();
    let rest = statement.strip_prefix(keyword)?.trim_start();
    let header_end = rest.find('{').unwrap_or(rest.len());
    let header = rest.get(..header_end)?.trim();
    let open_paren = header.find('(');
    let (name, parameters) = if let Some(open_paren) = open_paren {
        let close_paren = matching_close_paren_index(header, open_paren)?;
        let name = header.get(..open_paren)?.trim();
        let raw_parameters = header.get(open_paren + 1..close_paren)?;
        (name, parse_static_sass_parameters(raw_parameters))
    } else {
        (header, Vec::new())
    };
    if !is_static_sass_identifier(name) {
        return None;
    }

    Some(OmenaSifCallableExportV1 {
        name: name.to_string(),
        parameters,
        accepts_content: detect_content && statement.contains("@content"),
    })
}

fn parse_static_sass_placeholder_export(statement: &str) -> Option<OmenaSifPlaceholderExportV1> {
    let statement = statement.trim_start();
    if !statement.starts_with('%') {
        return None;
    }
    let name = statement
        .split(|character: char| character.is_whitespace() || character == '{' || character == ',')
        .next()?;
    if name.len() <= 1 {
        return None;
    }
    Some(OmenaSifPlaceholderExportV1 {
        name: name.to_string(),
    })
}

fn parse_static_sass_forward_export(statement: &str) -> Option<OmenaSifForwardExportV1> {
    let statement = strip_statement_semicolon(statement).trim_start();
    let rest = statement.strip_prefix("@forward")?.trim_start();
    let (canonical_url, after_url) = parse_quoted_string_prefix(rest)?;
    let mut prefix = None;
    let mut show = Vec::new();
    let mut hide = Vec::new();
    let mut rest = after_url.trim_start();

    while !rest.is_empty() {
        if let Some(after_keyword) = rest.strip_prefix("as ") {
            let (value, after_value) = take_until_forward_keyword(after_keyword.trim_start());
            let value = value.trim();
            if !value.is_empty() {
                prefix = Some(value.to_string());
            }
            rest = after_value.trim_start();
            continue;
        }
        if let Some(after_keyword) = rest.strip_prefix("show ") {
            let (value, after_value) = take_until_forward_keyword(after_keyword.trim_start());
            show = split_static_sass_symbol_list(value);
            rest = after_value.trim_start();
            continue;
        }
        if let Some(after_keyword) = rest.strip_prefix("hide ") {
            let (value, after_value) = take_until_forward_keyword(after_keyword.trim_start());
            hide = split_static_sass_symbol_list(value);
            rest = after_value.trim_start();
            continue;
        }
        break;
    }

    Some(OmenaSifForwardExportV1 {
        canonical_url,
        prefix,
        show,
        hide,
    })
}

fn parse_static_sass_parameters(raw_parameters: &str) -> Vec<OmenaSifParameterV1> {
    split_top_level_commas(raw_parameters)
        .into_iter()
        .filter_map(|parameter| {
            let parameter = parameter.trim();
            if parameter.is_empty() {
                return None;
            }
            let variadic = parameter.ends_with("...");
            let parameter = parameter.trim_end_matches("...").trim();
            let colon_index = top_level_character_index(parameter, ':');
            let (name, default_value_repr) = if let Some(colon_index) = colon_index {
                let name = parameter.get(..colon_index)?.trim();
                let default_value = parameter.get(colon_index + 1..)?.trim();
                (
                    name,
                    if default_value.is_empty() {
                        None
                    } else {
                        Some(canonical_sif_value_repr(default_value))
                    },
                )
            } else {
                (parameter, None)
            };
            if !is_static_sass_variable_name(name) {
                return None;
            }
            Some(OmenaSifParameterV1 {
                name: name.to_string(),
                default_value_repr,
                variadic,
            })
        })
        .collect()
}

fn parse_static_less_parameters(raw_parameters: &str) -> Vec<OmenaSifParameterV1> {
    split_top_level_commas(raw_parameters)
        .into_iter()
        .filter_map(|parameter| {
            let parameter = parameter.trim();
            if parameter.is_empty() {
                return None;
            }
            let variadic = parameter.ends_with("...");
            let parameter = parameter.trim_end_matches("...").trim();
            let colon_index = top_level_character_index(parameter, ':');
            let (name, default_value_repr) = if let Some(colon_index) = colon_index {
                let name = parameter.get(..colon_index)?.trim();
                let default_value = parameter.get(colon_index + 1..)?.trim();
                (
                    name,
                    if default_value.is_empty() {
                        None
                    } else {
                        Some(canonical_sif_value_repr(default_value))
                    },
                )
            } else {
                (parameter, None)
            };
            if !is_static_less_variable_name(name) {
                return None;
            }
            Some(OmenaSifParameterV1 {
                name: name.to_string(),
                default_value_repr,
                variadic,
            })
        })
        .collect()
}

fn split_top_level_commas(value: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut paren_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escape = false;

    for (index, character) in value.char_indices() {
        if let Some(quote_character) = quote {
            if escape {
                escape = false;
            } else if character == '\\' {
                escape = true;
            } else if character == quote_character {
                quote = None;
            }
            continue;
        }
        if character == '"' || character == '\'' {
            quote = Some(character);
            continue;
        }
        match character {
            '(' | '[' => paren_depth = paren_depth.saturating_add(1),
            ')' | ']' => paren_depth = paren_depth.saturating_sub(1),
            ',' if paren_depth == 0 => {
                if let Some(part) = value.get(start..index) {
                    parts.push(part);
                }
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    if start <= value.len()
        && let Some(part) = value.get(start..)
    {
        parts.push(part);
    }
    parts
}

fn top_level_character_index(value: &str, target: char) -> Option<usize> {
    let mut paren_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escape = false;

    for (index, character) in value.char_indices() {
        if let Some(quote_character) = quote {
            if escape {
                escape = false;
            } else if character == '\\' {
                escape = true;
            } else if character == quote_character {
                quote = None;
            }
            continue;
        }
        if character == '"' || character == '\'' {
            quote = Some(character);
            continue;
        }
        match character {
            '(' | '[' => paren_depth = paren_depth.saturating_add(1),
            ')' | ']' => paren_depth = paren_depth.saturating_sub(1),
            _ if character == target && paren_depth == 0 => return Some(index),
            _ => {}
        }
    }
    None
}

fn matching_close_paren_index(value: &str, open_index: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escape = false;

    for (index, character) in value
        .char_indices()
        .filter(|(index, _)| *index >= open_index)
    {
        if let Some(quote_character) = quote {
            if escape {
                escape = false;
            } else if character == '\\' {
                escape = true;
            } else if character == quote_character {
                quote = None;
            }
            continue;
        }
        if character == '"' || character == '\'' {
            quote = Some(character);
            continue;
        }
        match character {
            '(' => depth = depth.saturating_add(1),
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_quoted_string_prefix(value: &str) -> Option<(String, &str)> {
    let mut chars = value.char_indices();
    let (_, quote) = chars.next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let mut escape = false;
    for (index, character) in chars {
        if escape {
            escape = false;
            continue;
        }
        if character == '\\' {
            escape = true;
            continue;
        }
        if character == quote {
            let content = value.get(quote.len_utf8()..index)?.to_string();
            let rest = value.get(index + character.len_utf8()..)?;
            return Some((content, rest));
        }
    }
    None
}

fn take_until_forward_keyword(value: &str) -> (&str, &str) {
    let mut best_index = value.len();
    for keyword in [" as ", " show ", " hide ", " with "] {
        if let Some(index) = value.find(keyword) {
            best_index = best_index.min(index);
        }
    }
    let left = value.get(..best_index).unwrap_or(value);
    let right = value.get(best_index..).unwrap_or("");
    (left, right)
}

fn split_static_sass_symbol_list(value: &str) -> Vec<String> {
    split_top_level_commas(value)
        .into_iter()
        .flat_map(|part| part.split_whitespace())
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn strip_statement_semicolon(statement: &str) -> &str {
    statement.trim().trim_end_matches(';').trim()
}

fn is_static_sass_variable_name(value: &str) -> bool {
    value.starts_with('$')
        && value.len() > 1
        && value.chars().skip(1).all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
}

fn is_static_less_variable_name(value: &str) -> bool {
    value.starts_with('@')
        && value.len() > 1
        && value.chars().skip(1).all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
}

fn is_static_sass_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
}

fn is_static_less_mixin_name(value: &str) -> bool {
    value
        .strip_prefix(['.', '#'])
        .is_some_and(is_static_sass_identifier)
}

fn is_static_less_map_member_name(value: &str) -> bool {
    is_static_sass_identifier(value)
        || is_static_less_variable_name(value)
        || (value.starts_with('$') && is_static_sass_identifier(value.trim_start_matches('$')))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OMENA_SIF_HASH_ALGORITHM_V1, write_omena_sif_json_v1};

    #[test]
    fn static_generator_extracts_tier_a_sass_exports_without_evaluation()
    -> Result<(), serde_json::Error> {
        let source = r#"
$brand: color.mix(#fff, #000) !default;
$gap: 1rem;
%surface { color: $brand; }
@mixin button($variant: primary, $rest...) { @content; color: $brand; }
@function double($value) { @return $value * 2; }
@forward "./tokens" as token-* show $token-brand, button hide $token-gap;
"#;

        let sif = generate_static_omena_sif_v1(OmenaSifStaticGeneratorInputV1 {
            canonical_url: "pkg:design-system/_index.scss",
            source,
            syntax: OmenaSifSourceSyntaxV1::Scss,
        })?;

        assert_eq!(sif.exports.variables.len(), 2);
        assert_eq!(sif.exports.variables[0].name, "$brand");
        assert_eq!(
            sif.exports.variables[0].value_repr.as_deref(),
            Some("color.mix(#fff, #000)")
        );
        assert!(sif.exports.variables[0].defaulted);
        assert_eq!(sif.exports.mixins[0].name, "button");
        assert!(sif.exports.mixins[0].accepts_content);
        assert_eq!(sif.exports.mixins[0].parameters[0].name, "$variant");
        assert_eq!(
            sif.exports.mixins[0].parameters[0]
                .default_value_repr
                .as_deref(),
            Some("primary")
        );
        assert!(sif.exports.mixins[0].parameters[1].variadic);
        assert_eq!(sif.exports.functions[0].name, "double");
        assert_eq!(sif.exports.placeholders[0].name, "%surface");
        assert_eq!(sif.exports.forwards[0].canonical_url, "./tokens");
        assert_eq!(sif.exports.forwards[0].prefix.as_deref(), Some("token-*"));
        assert_eq!(sif.exports.forwards[0].show, vec!["$token-brand", "button"]);
        assert_eq!(sif.exports.forwards[0].hide, vec!["$token-gap"]);
        assert_eq!(sif.fingerprints.hash_algorithm, OMENA_SIF_HASH_ALGORITHM_V1);
        assert!(sif.fingerprints.leaf_hash.as_str().starts_with("blake3:"));
        Ok(())
    }

    #[test]
    fn static_generator_extracts_less_lif_exports_without_evaluation() {
        let source = r#"
@brand: #fff;
@tokens: { primary: @brand; @gap: 2px; };
.button(@gap: 1rem, @rest...) when (@gap > 0) { color: @brand; }
"#;

        let lif_exports = generate_static_omena_lif_exports_v1(OmenaSifStaticGeneratorInputV1 {
            canonical_url: "pkg:design-system/tokens.less",
            source,
            syntax: OmenaSifSourceSyntaxV1::Less,
        });

        assert!(lif_exports.sif_exports.variables.is_empty());
        assert_eq!(lif_exports.less_variables.len(), 1);
        assert_eq!(lif_exports.less_variables[0].name, "@brand");
        assert_eq!(
            lif_exports.less_variables[0].value_repr.as_deref(),
            Some("#fff")
        );
        assert_eq!(lif_exports.less_detached_rulesets.len(), 1);
        assert_eq!(lif_exports.less_detached_rulesets[0].name, "@tokens");
        assert_eq!(
            lif_exports.less_detached_rulesets[0].member_names,
            vec!["@gap", "primary"]
        );
        assert_eq!(lif_exports.less_mixins.len(), 1);
        assert_eq!(lif_exports.less_mixins[0].name, ".button");
        assert!(lif_exports.less_mixins[0].guarded);
        assert_eq!(lif_exports.less_mixins[0].parameters[0].name, "@gap");
        assert_eq!(
            lif_exports.less_mixins[0].parameters[0]
                .default_value_repr
                .as_deref(),
            Some("1rem")
        );
        assert!(lif_exports.less_mixins[0].parameters[1].variadic);
    }

    #[test]
    fn static_generator_round_trips_canonical_sif_json() -> Result<(), serde_json::Error> {
        let sif = generate_static_omena_sif_v1(OmenaSifStaticGeneratorInputV1 {
            canonical_url: "file:///workspace/tokens.scss",
            source: "$brand: red !default;",
            syntax: OmenaSifSourceSyntaxV1::Scss,
        })?;

        let json = write_omena_sif_json_v1(&sif)?;

        assert!(json.contains(r#""toolchainId":"omena-sifgen-static@0.1.0""#));
        assert!(json.contains(r#""name":"$brand""#));
        Ok(())
    }

    #[test]
    fn static_generator_canonicalizes_value_representations() -> Result<(), serde_json::Error> {
        let sif = generate_static_omena_sif_v1(OmenaSifStaticGeneratorInputV1 {
            canonical_url: "file:///workspace/tokens.scss",
            source: "$gap: 0px !default; $ratio: 0%; @mixin space($size: 0px, $ratio: 0%) {}",
            syntax: OmenaSifSourceSyntaxV1::Scss,
        })?;

        assert_eq!(sif.exports.variables[0].value_repr.as_deref(), Some("0"));
        assert_eq!(sif.exports.variables[1].value_repr.as_deref(), Some("0%"));
        assert_eq!(
            sif.exports.mixins[0].parameters[0]
                .default_value_repr
                .as_deref(),
            Some("0")
        );
        assert_eq!(
            sif.exports.mixins[0].parameters[1]
                .default_value_repr
                .as_deref(),
            Some("0%")
        );
        Ok(())
    }
}

use omena_parser::LexedToken;

use crate::helpers::{
    ascii::normalize_ascii_whitespace,
    declarations::{SimpleDeclarationSlice, declaration_ranges_are_adjacent},
    values::{
        split_top_level_value_arguments, split_top_level_whitespace_value_components,
        static_css_string_value,
    },
};

pub(crate) fn font_shorthand_replacement_for_declarations(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Option<(usize, usize, String)> {
    let [
        style,
        variant_caps,
        weight,
        stretch,
        size,
        line_height,
        family,
    ] = declarations
    else {
        return None;
    };
    if style.property != "font-style"
        || variant_caps.property != "font-variant-caps"
        || weight.property != "font-weight"
        || stretch.property != "font-stretch"
        || size.property != "font-size"
        || line_height.property != "line-height"
        || family.property != "font-family"
        || !declaration_ranges_are_adjacent(tokens, declarations)
    {
        return None;
    }

    let important = style.important;
    let values = declarations
        .iter()
        .map(|declaration| {
            if declaration.important != important {
                return None;
            }
            font_longhand_value_without_important(declaration)
        })
        .collect::<Option<Vec<_>>>()?;
    let [
        style,
        variant_caps,
        weight,
        stretch,
        size,
        line_height,
        family,
    ] = values.as_slice()
    else {
        return None;
    };
    let shorthand = compressed_font_shorthand_value(
        style,
        variant_caps,
        weight,
        stretch,
        size,
        line_height,
        family,
    )?;
    let important = if important { "!important" } else { "" };

    Some((
        declarations.first()?.start,
        declarations.last()?.end,
        format!("font: {shorthand}{important};"),
    ))
}

pub(crate) fn compress_existing_font_shorthand_value(value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    let mut parsed = None;
    for size_index in 0..components.len().saturating_sub(1) {
        let Some((size, line_height)) = parse_font_size_component(&components[size_index]) else {
            continue;
        };
        let Some((style, variant_caps, weight, stretch)) =
            parse_font_prefix_components(&components[..size_index])
        else {
            continue;
        };
        parsed = Some((
            size_index,
            style,
            variant_caps,
            weight,
            stretch,
            size,
            line_height,
        ));
    }
    let (size_index, style, variant_caps, weight, stretch, size, line_height) = parsed?;
    let family = normalize_font_family_value(&components[size_index + 1..].join(" "));
    if family.is_empty() {
        return None;
    }

    let replacement = compressed_font_shorthand_value(
        &style,
        &variant_caps,
        &weight,
        &stretch,
        &size,
        &line_height,
        &family,
    )?;
    (replacement.len() < normalize_ascii_whitespace(value).len()).then_some(replacement)
}

fn parse_font_size_component(component: &str) -> Option<(String, String)> {
    if let Some((size, line_height)) = component.split_once('/') {
        if size.is_empty() || line_height.is_empty() || !is_supported_font_size_token(size) {
            return None;
        }
        return Some((size.to_string(), line_height.to_string()));
    }
    is_supported_font_size_token(component).then(|| (component.to_string(), "normal".to_string()))
}

fn parse_font_prefix_components(components: &[String]) -> Option<(String, String, String, String)> {
    let mut style = "normal".to_string();
    let mut variant_caps = "normal".to_string();
    let mut weight = "normal".to_string();
    let mut stretch = "normal".to_string();

    for component in components {
        if component.eq_ignore_ascii_case("normal") {
            continue;
        }
        if is_supported_font_style(component) {
            if !style.eq_ignore_ascii_case("normal") {
                return None;
            }
            style = component.to_string();
            continue;
        }
        if is_supported_font_variant_caps(component) {
            if !variant_caps.eq_ignore_ascii_case("normal") {
                return None;
            }
            variant_caps = component.to_string();
            continue;
        }
        if is_supported_font_weight_component(component) {
            if !weight.eq_ignore_ascii_case("normal") {
                return None;
            }
            weight = component.to_string();
            continue;
        }
        if is_supported_font_stretch_component(component) {
            if !stretch.eq_ignore_ascii_case("normal") {
                return None;
            }
            stretch = component.to_string();
            continue;
        }
        return None;
    }

    Some((style, variant_caps, weight, stretch))
}

fn is_supported_font_size_token(value: &str) -> bool {
    let lower = value.trim().to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "xx-small"
            | "x-small"
            | "small"
            | "medium"
            | "large"
            | "x-large"
            | "xx-large"
            | "xxx-large"
            | "larger"
            | "smaller"
    ) {
        return true;
    }
    if matches!(
        lower.as_str(),
        "initial" | "inherit" | "unset" | "revert" | "revert-layer"
    ) {
        return false;
    }
    lower.starts_with("calc(")
        || lower.starts_with("clamp(")
        || lower.starts_with("min(")
        || lower.starts_with("max(")
        || has_css_length_or_percentage_unit(&lower)
}

fn has_css_length_or_percentage_unit(value: &str) -> bool {
    let units = [
        "%", "cap", "ch", "cm", "dvb", "dvh", "dvi", "dvmax", "dvmin", "dvw", "em", "ex", "ic",
        "in", "lh", "lvb", "lvh", "lvi", "lvmax", "lvmin", "lvw", "mm", "pc", "pt", "px", "q",
        "rem", "rlh", "svb", "svh", "svi", "svmax", "svmin", "svw", "vb", "vh", "vi", "vmax",
        "vmin", "vw",
    ];
    units.iter().any(|unit| {
        let Some(number) = value.strip_suffix(unit) else {
            return false;
        };
        !number.is_empty() && number.parse::<f64>().is_ok()
    })
}

fn is_supported_font_weight_component(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "normal"
            | "bold"
            | "bolder"
            | "lighter"
            | "100"
            | "200"
            | "300"
            | "400"
            | "500"
            | "600"
            | "700"
            | "800"
            | "900"
    )
}

fn is_supported_font_stretch_component(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "ultra-condensed"
            | "extra-condensed"
            | "condensed"
            | "semi-condensed"
            | "normal"
            | "semi-expanded"
            | "expanded"
            | "extra-expanded"
            | "ultra-expanded"
    ) || value
        .strip_suffix('%')
        .is_some_and(|number| !number.is_empty() && number.parse::<f64>().is_ok())
}

fn compressed_font_shorthand_value(
    style: &str,
    variant_caps: &str,
    weight: &str,
    stretch: &str,
    size: &str,
    line_height: &str,
    family: &str,
) -> Option<String> {
    if size.is_empty()
        || family.is_empty()
        || is_css_wide_keyword(size)
        || is_css_wide_keyword(family)
    {
        return None;
    }
    if !is_supported_font_style(style) || !is_supported_font_variant_caps(variant_caps) {
        return None;
    }
    let weight = normalize_font_weight_value(weight);
    let stretch = normalize_font_stretch_value(stretch);
    let family = normalize_font_family_value(family);

    let mut components = Vec::new();
    if !style.eq_ignore_ascii_case("normal") {
        components.push(style.to_string());
    }
    if !variant_caps.eq_ignore_ascii_case("normal") {
        components.push(variant_caps.to_string());
    }
    if !is_default_font_weight(&weight) {
        components.push(weight);
    }
    if !is_default_font_stretch(&stretch) {
        components.push(stretch);
    }

    if line_height.eq_ignore_ascii_case("normal") {
        components.push(size.to_string());
    } else {
        components.push(format!("{size}/{line_height}"));
    }
    components.push(family);

    Some(components.join(" "))
}

fn font_longhand_value_without_important(declaration: &SimpleDeclarationSlice) -> Option<String> {
    let mut components = split_top_level_whitespace_value_components(&declaration.value)?;
    if declaration.important
        && components.last().is_some_and(|component| {
            component.eq_ignore_ascii_case("!important")
                || component.eq_ignore_ascii_case("important")
        })
    {
        components.pop();
    }
    if components.is_empty() {
        return None;
    }
    Some(normalize_ascii_whitespace(&components.join(" ")))
}

fn is_default_font_weight(value: &str) -> bool {
    value.eq_ignore_ascii_case("normal") || value == "400"
}

fn is_default_font_stretch(value: &str) -> bool {
    value.eq_ignore_ascii_case("normal") || value == "100%"
}

fn normalize_font_weight_value(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => "400".to_string(),
        "bold" => "700".to_string(),
        _ => value.to_string(),
    }
}

fn normalize_font_stretch_value(value: &str) -> String {
    let normalized = match value.trim().to_ascii_lowercase().as_str() {
        "ultra-condensed" => "50%",
        "extra-condensed" => "62.5%",
        "condensed" => "75%",
        "semi-condensed" => "87.5%",
        "normal" => "100%",
        "semi-expanded" => "112.5%",
        "expanded" => "125%",
        "extra-expanded" => "150%",
        "ultra-expanded" => "200%",
        _ => return value.to_string(),
    };
    normalized.to_string()
}

fn normalize_font_family_value(value: &str) -> String {
    let Some(families) = split_top_level_value_arguments(value) else {
        return value.to_string();
    };
    families
        .into_iter()
        .map(|family| {
            static_css_string_value(&family)
                .and_then(|quoted| unquote_static_font_family_name(&quoted))
                .unwrap_or(family)
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn unquote_static_font_family_name(value: &str) -> Option<String> {
    let parts = value.split_ascii_whitespace().collect::<Vec<_>>();
    if parts.is_empty()
        || parts
            .iter()
            .any(|part| !is_safe_unquoted_font_family_identifier(part))
    {
        return None;
    }
    Some(parts.join(" "))
}

fn is_safe_unquoted_font_family_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if value.starts_with("--") && value.len() > 2 {
        return chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
            && !is_reserved_unquoted_font_family_identifier(value);
    }
    if first == '-' {
        let Some(second) = chars.next() else {
            return false;
        };
        if !(second.is_ascii_alphabetic() || second == '_') {
            return false;
        }
        return chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
            && !is_reserved_unquoted_font_family_identifier(value);
    }
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    if !chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_')) {
        return false;
    }
    !is_reserved_unquoted_font_family_identifier(value)
}

fn is_reserved_unquoted_font_family_identifier(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "serif"
            | "sans-serif"
            | "monospace"
            | "cursive"
            | "fantasy"
            | "system-ui"
            | "ui-serif"
            | "ui-sans-serif"
            | "ui-monospace"
            | "ui-rounded"
            | "math"
            | "emoji"
            | "fangsong"
            | "inherit"
            | "initial"
            | "unset"
            | "revert"
            | "revert-layer"
    )
}

fn is_supported_font_style(value: &str) -> bool {
    value.eq_ignore_ascii_case("normal")
        || value.eq_ignore_ascii_case("italic")
        || value.eq_ignore_ascii_case("oblique")
}

fn is_supported_font_variant_caps(value: &str) -> bool {
    value.eq_ignore_ascii_case("normal") || value.eq_ignore_ascii_case("small-caps")
}

fn is_css_wide_keyword(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "inherit" | "initial" | "unset" | "revert" | "revert-layer"
    )
}

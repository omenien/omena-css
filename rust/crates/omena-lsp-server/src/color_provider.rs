//! `textDocument/documentColor` / `colorPresentation` for VARIABLE
//! REFERENCES: the built-in SCSS service already decorates color LITERALS,
//! so this provider fills exactly the gap it cannot — resolving `$token`
//! and `var(--token)` reference sites to their declared color values
//! through the same render machinery the hover uses (alias chains and
//! external token facts included). Literal sites are deliberately NOT
//! emitted here: two providers decorating the same range would double the
//! swatch.

use crate::external_sif_symbols::external_sif_sass_symbol_target_for_candidate;
use crate::protocol::document_uri_from_params;
use crate::state::{LspShellState, LspStyleHoverCandidate, LspTextDocumentState};
use crate::style_hover_candidates_for_document;
use omena_query::{
    resolve_omena_query_sass_forward_sources,
    resolve_omena_query_sass_module_use_sources_for_candidate,
    summarize_omena_query_sass_module_sources,
    summarize_omena_query_style_hover_render_parts_for_hover_position,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// Reference-site kinds this provider resolves. Declarations are excluded:
/// their value literal sits on the same line and already gets the built-in
/// swatch.
const COLOR_CANDIDATE_KINDS: &[&str] = &["sassVariableReference"];

/// documentColor fires on every edit; a degenerate document must stay
/// bounded even before the per-name memo kicks in.
const MAX_COLOR_CANDIDATES: usize = 2_048;

pub(crate) fn resolve_lsp_document_color(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    let Some(document) = state.document(document_uri.as_str()) else {
        return json!([]);
    };
    let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
        return json!([]);
    };
    let color_candidates = candidates
        .iter()
        .filter(|candidate| COLOR_CANDIDATE_KINDS.contains(&candidate.kind))
        .take(MAX_COLOR_CANDIDATES)
        .collect::<Vec<_>>();
    if color_candidates.is_empty() {
        return json!([]);
    }
    // Cross-request cache: documentColor re-fires on every edit and scroll,
    // and the resolution walk below reads unadmitted dependency chains from
    // disk. The key is honest, not minimal — ANY corpus text or membership
    // move invalidates — because the declarations backing these values can
    // live in any reachable module.
    let cache_key = (
        document.version,
        state
            .tide_ledger
            .mark(crate::tide::TideInputKindV0::DocumentText),
        state
            .tide_ledger
            .mark(crate::tide::TideInputKindV0::DocumentSet),
    );
    if let Ok(cache) = state.document_color_cache.lock()
        && let Some((cached_key, cached)) = cache.get(document_uri.as_str())
        && *cached_key == cache_key
    {
        return cached.clone();
    }

    // ONE declaration walk per request: same-named references resolve
    // identically, and every name shares the namespace's transitive module
    // closure — resolving per name would re-read and re-parse unadmitted
    // dependency chains from disk once per variable (measured at seconds
    // per tab open on a large workspace).
    let mut declarations = SassVariableDeclarationIndexV0::default();
    let mut color_by_name: BTreeMap<(Option<&str>, &str), Option<[f64; 4]>> = BTreeMap::new();
    let mut informations = Vec::new();
    for candidate in color_candidates {
        let key = (candidate.namespace.as_deref(), candidate.name.as_str());
        let color = *color_by_name.entry(key).or_insert_with(|| {
            declarations
                .resolve_value(state, document, candidate)
                .as_deref()
                .and_then(parse_css_color)
        });
        if let Some([red, green, blue, alpha]) = color {
            informations.push(json!({
                "range": candidate.range,
                "color": {
                    "red": red,
                    "green": green,
                    "blue": blue,
                    "alpha": alpha,
                },
            }));
        }
    }
    let informations = json!(informations);
    if let Ok(mut cache) = state.document_color_cache.lock() {
        cache.insert(document_uri, (cache_key, informations.clone()));
        // The cache never outgrows the open-tab working set by much.
        if cache.len() > 64 {
            let stale = cache.keys().next().cloned();
            if let Some(stale) = stale {
                cache.remove(stale.as_str());
            }
        }
    }
    informations
}

/// Lazily-built per-request declaration index: for each namespace the
/// requesting document can reach, walk the transitive module closure ONCE
/// (the same targets + forward expansion the symbol resolver uses) and
/// collect every variable declaration into a name map. Value extraction
/// runs the document-local hover render at the declaration — the same
/// authority that fills the hover's `Value:` line.
#[derive(Default)]
struct SassVariableDeclarationIndexV0 {
    by_namespace: BTreeMap<Option<String>, BTreeMap<String, Option<String>>>,
}

impl SassVariableDeclarationIndexV0 {
    fn resolve_value(
        &mut self,
        state: &LspShellState,
        document: &LspTextDocumentState,
        candidate: &LspStyleHoverCandidate,
    ) -> Option<String> {
        // External token facts first: their exported value representation
        // is a plain map lookup, and they are exactly the names whose
        // filesystem resolution would be the expensive failure path.
        if let Some(target) =
            external_sif_sass_symbol_target_for_candidate(state, document, candidate)
            && target.value_repr.is_some()
        {
            return target.value_repr;
        }
        let namespace = candidate.namespace.clone();
        if !self.by_namespace.contains_key(&namespace) {
            let values = collect_reachable_variable_values(state, document, candidate);
            self.by_namespace.insert(namespace.clone(), values);
        }
        self.by_namespace
            .get(&namespace)
            .and_then(|values| values.get(candidate.name.as_str()))
            .cloned()
            .flatten()
    }
}

/// Decorations may not cost what analysis costs: the walk stays inside the
/// ADMITTED corpus (no disk document rebuilds) and resolves only RELATIVE
/// specifiers (non-relative ones are the resolver's expensive failure
/// path, and their values arrive through external token facts above). A
/// name this bounded walk cannot see simply gets no swatch.
const MAX_COLOR_WALK_DOCUMENTS: usize = 64;

fn collect_reachable_variable_values(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> BTreeMap<String, Option<String>> {
    let mut values = BTreeMap::new();
    // Unnamespaced references see the requesting document's own
    // declarations first — same precedence as the symbol resolver.
    if candidate.namespace.is_none() {
        collect_document_variable_values(document, &mut values);
    }
    let mut visited = BTreeSet::from([document.uri.clone()]);
    let mut queue = relative_module_target_uris(state, document, candidate.namespace.as_deref());
    while let Some(target_uri) = queue.pop() {
        if visited.len() > MAX_COLOR_WALK_DOCUMENTS {
            break;
        }
        if !visited.insert(target_uri.clone()) {
            continue;
        }
        let Some(target_document) = state.document(target_uri.as_str()) else {
            continue;
        };
        collect_document_variable_values(target_document, &mut values);
        // Follow @use and @forward chains, still relative-only.
        queue.extend(relative_module_target_uris(state, target_document, None));
    }
    values
}

/// The requesting document's `@use`/`@forward` targets, restricted to
/// relative specifiers resolved through the same resolver navigation uses.
fn relative_module_target_uris(
    state: &LspShellState,
    document: &LspTextDocumentState,
    namespace: Option<&str>,
) -> Vec<String> {
    let Some(sources) =
        summarize_omena_query_sass_module_sources(document.uri.as_str(), document.text.as_str())
    else {
        return Vec::new();
    };
    let mut uris = Vec::new();
    let mut push_relative = |specifier: &str| {
        if !specifier.starts_with('.') {
            return;
        }
        if let Some(uri) = crate::resolve_lsp_style_uri_for_specifier(state, document, specifier) {
            uris.push(uri);
        }
    };
    for source in resolve_omena_query_sass_module_use_sources_for_candidate(&sources, namespace) {
        push_relative(source.as_str());
    }
    for forward_source in resolve_omena_query_sass_forward_sources(&sources) {
        push_relative(forward_source.as_str());
    }
    uris
}

fn collect_document_variable_values(
    document: &LspTextDocumentState,
    values: &mut BTreeMap<String, Option<String>>,
) {
    for declaration in document
        .style_candidates
        .iter()
        .filter(|declaration| declaration.kind == "sassVariableDeclaration")
    {
        values.entry(declaration.name.clone()).or_insert_with(|| {
            summarize_omena_query_style_hover_render_parts_for_hover_position(
                document.text.as_str(),
                declaration.kind,
                declaration.name.as_str(),
                declaration.range.start,
            )
            .value
        });
    }
}

/// `textDocument/colorPresentation`: the label the editor writes back when
/// the user picks a color from the swatch. The reference site itself must
/// keep its variable name, so the presentation is the plain hex of the
/// picked color — the editor only shows it; replacing a token reference
/// with a literal stays an explicit user action.
pub(crate) fn resolve_lsp_color_presentation(params: Option<&Value>) -> Value {
    let Some(color) = params.and_then(|value| value.get("color")) else {
        return json!([]);
    };
    let component = |name: &str| -> f64 {
        color
            .get(name)
            .and_then(Value::as_f64)
            .unwrap_or_default()
            .clamp(0.0, 1.0)
    };
    let to_byte = |value: f64| -> u8 { (value * 255.0).round().clamp(0.0, 255.0) as u8 };
    let (red, green, blue, alpha) = (
        to_byte(component("red")),
        to_byte(component("green")),
        to_byte(component("blue")),
        to_byte(component("alpha")),
    );
    let label = if alpha == u8::MAX {
        format!("#{red:02x}{green:02x}{blue:02x}")
    } else {
        format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}")
    };
    json!([{ "label": label }])
}

/// Parse the CSS color forms a token value realistically takes: hex
/// (#rgb/#rgba/#rrggbb/#rrggbbaa), rgb()/rgba(), hsl()/hsla(). Anything
/// else — named colors, gradients, further variable aliases — yields no
/// swatch rather than a guess.
pub(crate) fn parse_css_color(value: &str) -> Option<[f64; 4]> {
    let value = value.trim().trim_end_matches("!default").trim();
    if let Some(hex) = value.strip_prefix('#') {
        return parse_hex_color(hex);
    }
    let lower = value.to_ascii_lowercase();
    for (prefix, is_hsl) in [
        ("rgba(", false),
        ("rgb(", false),
        ("hsla(", true),
        ("hsl(", true),
    ] {
        if let Some(rest) = lower.strip_prefix(prefix) {
            let body = rest.strip_suffix(')')?;
            return parse_color_function(body, is_hsl);
        }
    }
    None
}

fn parse_hex_color(hex: &str) -> Option<[f64; 4]> {
    if !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let nibble = |index: usize| u8::from_str_radix(&hex[index..=index], 16).ok();
    let byte = |index: usize| u8::from_str_radix(&hex[index..index + 2], 16).ok();
    let channels = match hex.len() {
        3 | 4 => {
            let mut channels = [255u8; 4];
            for (slot, channel) in channels.iter_mut().take(hex.len()).enumerate() {
                let value = nibble(slot)?;
                *channel = value << 4 | value;
            }
            channels
        }
        6 | 8 => {
            let mut channels = [255u8; 4];
            for (slot, channel) in channels.iter_mut().take(hex.len() / 2).enumerate() {
                *channel = byte(slot * 2)?;
            }
            channels
        }
        _ => return None,
    };
    Some([
        f64::from(channels[0]) / 255.0,
        f64::from(channels[1]) / 255.0,
        f64::from(channels[2]) / 255.0,
        f64::from(channels[3]) / 255.0,
    ])
}

fn parse_color_function(body: &str, is_hsl: bool) -> Option<[f64; 4]> {
    let parts = body
        .replace(['/', ','], " ")
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    if parts.len() != 3 && parts.len() != 4 {
        return None;
    }
    let alpha = if parts.len() == 4 {
        parse_alpha_component(parts[3].as_str())?
    } else {
        1.0
    };
    if is_hsl {
        let hue = parts[0].trim_end_matches("deg").parse::<f64>().ok()?;
        let saturation = parse_percent_component(parts[1].as_str())?;
        let lightness = parse_percent_component(parts[2].as_str())?;
        let [red, green, blue] = hsl_to_rgb(hue, saturation, lightness);
        return Some([red, green, blue, alpha]);
    }
    let channel = |part: &str| -> Option<f64> {
        if let Some(percent) = part.strip_suffix('%') {
            Some((percent.parse::<f64>().ok()? / 100.0).clamp(0.0, 1.0))
        } else {
            Some((part.parse::<f64>().ok()? / 255.0).clamp(0.0, 1.0))
        }
    };
    Some([
        channel(parts[0].as_str())?,
        channel(parts[1].as_str())?,
        channel(parts[2].as_str())?,
        alpha,
    ])
}

fn parse_alpha_component(part: &str) -> Option<f64> {
    if let Some(percent) = part.strip_suffix('%') {
        Some((percent.parse::<f64>().ok()? / 100.0).clamp(0.0, 1.0))
    } else {
        Some(part.parse::<f64>().ok()?.clamp(0.0, 1.0))
    }
}

fn parse_percent_component(part: &str) -> Option<f64> {
    Some((part.strip_suffix('%')?.parse::<f64>().ok()? / 100.0).clamp(0.0, 1.0))
}

fn hsl_to_rgb(hue: f64, saturation: f64, lightness: f64) -> [f64; 3] {
    let hue = hue.rem_euclid(360.0) / 60.0;
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let secondary = chroma * (1.0 - (hue.rem_euclid(2.0) - 1.0).abs());
    let (red, green, blue) = match hue as u32 {
        0 => (chroma, secondary, 0.0),
        1 => (secondary, chroma, 0.0),
        2 => (0.0, chroma, secondary),
        3 => (0.0, secondary, chroma),
        4 => (secondary, 0.0, chroma),
        _ => (chroma, 0.0, secondary),
    };
    let match_lightness = lightness - chroma / 2.0;
    [
        (red + match_lightness).clamp(0.0, 1.0),
        (green + match_lightness).clamp(0.0, 1.0),
        (blue + match_lightness).clamp(0.0, 1.0),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_forms() -> Result<(), &'static str> {
        assert_eq!(
            parse_css_color("#09ab49").ok_or("6-digit hex")?,
            [9.0 / 255.0, 171.0 / 255.0, 73.0 / 255.0, 1.0],
        );
        assert_eq!(parse_css_color("#fff").ok_or("3-digit hex")?, [1.0; 4]);
        assert_eq!(
            parse_css_color("#00000080").ok_or("8-digit hex")?[3],
            128.0 / 255.0,
        );
        assert_eq!(
            parse_css_color("#09ab49 !default").ok_or("default suffix")?[1],
            171.0 / 255.0
        );
        assert!(
            parse_css_color("#12345").is_none(),
            "5 digits is not a color"
        );
        assert!(
            parse_css_color("$green500").is_none(),
            "aliases yield no swatch"
        );
        Ok(())
    }

    #[test]
    fn parses_functional_forms() -> Result<(), &'static str> {
        assert_eq!(
            parse_css_color("rgb(9, 171, 73)").ok_or("rgb")?,
            [9.0 / 255.0, 171.0 / 255.0, 73.0 / 255.0, 1.0],
        );
        assert_eq!(
            parse_css_color("rgba(255, 0, 0, 0.5)").ok_or("rgba")?[3],
            0.5
        );
        assert_eq!(
            parse_css_color("rgb(100% 0% 0% / 25%)").ok_or("space syntax")?,
            [1.0, 0.0, 0.0, 0.25],
        );
        let [red, green, blue, alpha] = parse_css_color("hsl(120, 100%, 50%)").ok_or("hsl")?;
        assert!(red.abs() < 1e-9 && (green - 1.0).abs() < 1e-9 && blue.abs() < 1e-9);
        assert_eq!(alpha, 1.0);
        assert!(parse_css_color("linear-gradient(#fff, #000)").is_none());
        Ok(())
    }

    #[test]
    fn presentation_renders_hex_labels() {
        let presentation = resolve_lsp_color_presentation(Some(&serde_json::json!({
            "color": {"red": 9.0 / 255.0, "green": 171.0 / 255.0, "blue": 73.0 / 255.0, "alpha": 1.0},
        })));
        assert_eq!(presentation[0]["label"], "#09ab49");
        let translucent = resolve_lsp_color_presentation(Some(&serde_json::json!({
            "color": {"red": 0.0, "green": 0.0, "blue": 0.0, "alpha": 0.5},
        })));
        assert_eq!(translucent[0]["label"], "#00000080");
    }
}

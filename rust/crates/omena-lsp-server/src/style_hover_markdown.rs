use crate::{LspStyleHoverCandidate, protocol::file_label_from_uri};
use omena_query::{
    AbstractPropertyValueNarrowingV0, AbstractPropertyValueV0, OmenaQueryStyleHoverRenderPartsV0,
    is_omena_query_sass_symbol_candidate_kind, is_omena_query_sass_symbol_declaration_kind,
    omena_query_sass_symbol_kind_from_candidate_kind,
};

pub(crate) fn render_style_hover_candidate_markdown_from_parts(
    document_uri: &str,
    candidate: &LspStyleHoverCandidate,
    render_parts: &OmenaQueryStyleHoverRenderPartsV0,
) -> String {
    let location = format!(
        "{}:{}",
        file_label_from_uri(document_uri),
        candidate.range.start.line + 1
    );
    match candidate.kind {
        "selector" => {
            let narrowing_markdown =
                render_property_value_narrowings_markdown(&render_parts.property_value_narrowings);
            format!(
                "**`.{}`** - _{}_\n\n```scss\n{}\n```{}",
                candidate.name, location, render_parts.snippet, narrowing_markdown
            )
        }
        "customPropertyReference" => {
            format!(
                "**`var({})`** - _{}_\n\n```scss\n{}\n```",
                candidate.name, location, render_parts.snippet
            )
        }
        "customPropertyDeclaration" => {
            format!(
                "**`{}`** - _{}_\n\n```scss\n{}\n```",
                candidate.name, location, render_parts.snippet
            )
        }
        kind if is_omena_query_sass_symbol_candidate_kind(kind) => {
            render_sass_symbol_hover_markdown(candidate, location.as_str(), render_parts)
        }
        _ => candidate.name.clone(),
    }
}

fn render_property_value_narrowings_markdown(
    narrowings: &[AbstractPropertyValueNarrowingV0],
) -> String {
    if narrowings.is_empty() {
        return String::new();
    }
    let lines = narrowings
        .iter()
        .take(6)
        .map(|narrowing| {
            format!(
                "- `{}`: {}{}",
                narrowing.property_name,
                render_property_value_narrowing_value(narrowing),
                render_property_value_narrowing_context(narrowing)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("\n\nCascade narrowed values:\n{lines}")
}

fn render_property_value_narrowing_value(narrowing: &AbstractPropertyValueNarrowingV0) -> String {
    if let Some(display_value) = narrowing.display_value.as_deref() {
        return format!("`{display_value}`");
    }
    if !narrowing.display_values.is_empty() {
        return narrowing
            .display_values
            .iter()
            .map(|value| format!("`{value}`"))
            .collect::<Vec<_>>()
            .join(" | ");
    }
    render_abstract_property_value(&narrowing.value)
}

fn render_abstract_property_value(value: &AbstractPropertyValueV0) -> String {
    match value {
        AbstractPropertyValueV0::Bottom { .. } => "`<bottom>`".to_string(),
        AbstractPropertyValueV0::Exact { value, .. } => format!("`{value}`"),
        AbstractPropertyValueV0::FiniteSet { values, .. } => values
            .iter()
            .map(|value| format!("`{value}`"))
            .collect::<Vec<_>>()
            .join(" | "),
        AbstractPropertyValueV0::CustomPropertyReference {
            custom_property_name,
            ..
        } => {
            format!("`var({custom_property_name})`")
        }
        AbstractPropertyValueV0::Top { .. } => "`<top>`".to_string(),
    }
}

fn render_property_value_narrowing_context(narrowing: &AbstractPropertyValueNarrowingV0) -> String {
    let mut context = Vec::new();
    if !narrowing.requested_condition_context.is_empty() {
        context.push(narrowing.requested_condition_context.join(" / "));
    }
    if let Some(layer_name) = narrowing.requested_layer_name.as_deref() {
        context.push(format!("@layer {layer_name}"));
    } else if narrowing.requested_layer_scope == "exactLayer" {
        context.push("unlayered".to_string());
    }
    if context.is_empty() {
        String::new()
    } else {
        format!(" ({})", context.join(", "))
    }
}

fn render_sass_symbol_hover_markdown(
    candidate: &LspStyleHoverCandidate,
    location: &str,
    render_parts: &OmenaQueryStyleHoverRenderPartsV0,
) -> String {
    let label = render_sass_symbol_label(candidate);
    match omena_query_sass_symbol_kind_from_candidate_kind(candidate.kind) {
        Some("variable") if is_omena_query_sass_symbol_declaration_kind(candidate.kind) => {
            if let Some(value) = render_parts.value.as_deref() {
                return format!(
                    "**`{}`** - _{}_\n\nValue: `{}`\n\n```scss\n{}\n```",
                    label, location, value, render_parts.snippet
                );
            }
            format!(
                "**`{}`** - _{}_\n\n```scss\n{}\n```",
                label, location, render_parts.snippet
            )
        }
        Some("mixin" | "function")
            if is_omena_query_sass_symbol_declaration_kind(candidate.kind) =>
        {
            let rendered_label = render_parts.signature.as_deref().unwrap_or(label.as_str());
            format!(
                "**`{}`** - _{}_\n\n```scss\n{}\n```",
                rendered_label, location, render_parts.snippet
            )
        }
        _ => {
            format!(
                "**`{}`** - _{}_\n\n```scss\n{}\n```",
                label, location, render_parts.snippet
            )
        }
    }
}

fn render_sass_symbol_label(candidate: &LspStyleHoverCandidate) -> String {
    let namespace_prefix = candidate
        .namespace
        .as_deref()
        .map(|namespace| format!("{namespace}."))
        .unwrap_or_default();
    match omena_query_sass_symbol_kind_from_candidate_kind(candidate.kind) {
        Some("variable") => format!("{namespace_prefix}${}", candidate.name),
        Some("mixin") if is_omena_query_sass_symbol_declaration_kind(candidate.kind) => {
            format!("@mixin {}", candidate.name)
        }
        Some("mixin") => format!("@include {namespace_prefix}{}", candidate.name),
        Some("function") => format!("{namespace_prefix}{}()", candidate.name),
        _ => candidate.name.clone(),
    }
}

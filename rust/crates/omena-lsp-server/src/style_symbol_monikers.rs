use std::collections::BTreeSet;

use omena_query::{
    OmenaWorkspaceMonikerInput,
    is_omena_query_sass_symbol_declaration_kind as is_sass_symbol_declaration_kind,
    omena_query_sass_symbol_kind_from_candidate_kind as sass_symbol_kind_from_candidate_kind,
    omena_workspace_moniker,
};

use crate::{
    ExternalSifSassSymbolTarget, LspShellState, external_sif_sass_symbol_target_for_candidate,
    foreign_style_identity::style_foreign_sass_symbol_moniker,
    sass_symbol_definitions_for_candidate,
    state::{LspStyleHoverCandidate, LspTextDocumentState},
};

pub(super) fn style_symbol_monikers_for_candidate(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> BTreeSet<String> {
    if candidate.kind.starts_with("customProperty") {
        return BTreeSet::from([style_custom_property_moniker(
            document.workspace_folder_uri.as_deref(),
            candidate.name.as_str(),
        )]);
    }
    if is_sass_symbol_declaration_kind(candidate.kind) {
        return BTreeSet::from([style_sass_symbol_moniker_for_document(
            state, document, candidate,
        )]);
    }
    let definitions = sass_symbol_definitions_for_candidate(state, document, candidate);
    if definitions.is_empty() {
        if let Some(target) =
            external_sif_sass_symbol_target_for_candidate(state, document, candidate)
        {
            return BTreeSet::from([style_external_sif_sass_symbol_moniker(&target)]);
        }
        return BTreeSet::from([style_unresolved_sass_symbol_moniker(
            document.workspace_folder_uri.as_deref(),
            candidate,
        )]);
    }
    definitions
        .into_iter()
        .map(|(uri, definition)| {
            style_sass_symbol_moniker_for_uri(state, uri.as_str(), &definition)
        })
        .collect()
}

pub(super) fn style_symbol_role_for_candidate(candidate: &LspStyleHoverCandidate) -> &'static str {
    if candidate.kind.ends_with("Declaration") {
        "definition"
    } else {
        "reference"
    }
}

pub(super) fn style_custom_property_moniker(
    workspace_folder_uri: Option<&str>,
    name: &str,
) -> String {
    omena_workspace_moniker(OmenaWorkspaceMonikerInput::CssCustomProperty {
        workspace_folder_uri,
        name,
    })
}

pub(super) fn style_sass_symbol_moniker_for_document(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> String {
    style_sass_symbol_moniker_for_uri(state, document.uri.as_str(), candidate)
}

pub(super) fn style_sass_symbol_moniker_for_uri(
    state: &LspShellState,
    uri: &str,
    candidate: &LspStyleHoverCandidate,
) -> String {
    if let Some(moniker) = style_foreign_sass_symbol_moniker(state, uri, candidate) {
        return moniker;
    }
    style_sass_symbol_moniker(uri, candidate)
}

pub(super) fn style_unresolved_sass_symbol_moniker(
    workspace_folder_uri: Option<&str>,
    candidate: &LspStyleHoverCandidate,
) -> String {
    let family = sass_symbol_kind_from_candidate_kind(candidate.kind).unwrap_or("symbol");
    omena_workspace_moniker(OmenaWorkspaceMonikerInput::SassUnresolvedSymbol {
        workspace_folder_uri,
        family,
        namespace: candidate.namespace.as_deref(),
        name: candidate.name.as_str(),
    })
}

pub(super) fn style_external_sif_sass_symbol_moniker(
    target: &ExternalSifSassSymbolTarget,
) -> String {
    format!(
        "sass-symbol-foreign:sif:{}@{}#{}:{}",
        target.canonical_url, target.interface_hash, target.family, target.name
    )
}

pub(super) fn render_external_sif_sass_symbol_hover_markdown(
    target: &ExternalSifSassSymbolTarget,
) -> String {
    let label = match target.family {
        "variable" => format!("`${}`", format_args!("${}", target.name)),
        "mixin" => format!("`@mixin {}`", target.name),
        "function" => format!("`{}()`", target.name),
        _ => format!("`{}`", target.name),
    };
    let mut lines = vec![
        label,
        String::new(),
        format!("External Sass interface from `{}`.", target.canonical_url),
        "Source location is unavailable for this SIF-backed symbol.".to_string(),
    ];
    if let Some(value_repr) = target
        .value_repr
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        lines.push(String::new());
        lines.push(format!("Value: `{value_repr}`"));
    }
    lines.join("\n")
}

fn style_sass_symbol_moniker(uri: &str, candidate: &LspStyleHoverCandidate) -> String {
    let family = sass_symbol_kind_from_candidate_kind(candidate.kind).unwrap_or("symbol");
    omena_workspace_moniker(OmenaWorkspaceMonikerInput::SassSymbol {
        definition_uri: uri,
        family,
        name: candidate.name.as_str(),
    })
}

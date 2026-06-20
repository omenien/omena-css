use omena_query::{
    OmenaQuerySourceSelectorOccurrenceV0, OmenaWorkspaceOccurrenceFamilyV0,
    OmenaWorkspaceOccurrenceKindV0, OmenaWorkspaceOccurrenceRoleV0,
    OmenaWorkspaceOccurrenceSurfaceV0, OmenaWorkspaceOccurrenceV0,
};

use crate::{
    file_uri_equivalent,
    state::{
        LspDocumentOrigin, LspStyleHoverCandidate, LspStyleSymbolOccurrenceV0, LspTextDocumentState,
    },
};

pub(super) fn workspace_occurrence_from_source_selector_occurrence_for_lsp(
    occurrence: &OmenaQuerySourceSelectorOccurrenceV0,
) -> OmenaWorkspaceOccurrenceV0 {
    OmenaWorkspaceOccurrenceV0 {
        moniker: occurrence.moniker.clone(),
        uri: occurrence.uri.clone(),
        name: occurrence.selector_name.clone(),
        range: occurrence.range,
        kind: occurrence.kind,
        role: occurrence.role,
        surface: occurrence.source,
        family: Some(OmenaWorkspaceOccurrenceFamilyV0::CssModuleSelector),
        namespace: None,
        target_style_uri: occurrence.target_style_uri.clone(),
        rename_target: occurrence.rename_target,
    }
}

pub(super) fn source_selector_occurrence_from_workspace_occurrence_for_lsp(
    occurrence: OmenaWorkspaceOccurrenceV0,
) -> Option<OmenaQuerySourceSelectorOccurrenceV0> {
    (occurrence.family == Some(OmenaWorkspaceOccurrenceFamilyV0::CssModuleSelector)).then_some(
        OmenaQuerySourceSelectorOccurrenceV0 {
            moniker: occurrence.moniker,
            uri: occurrence.uri,
            selector_name: occurrence.name,
            range: occurrence.range,
            kind: occurrence.kind,
            role: occurrence.role,
            source: occurrence.surface,
            target_style_uri: occurrence.target_style_uri,
            rename_target: occurrence.rename_target,
        },
    )
}

pub(super) fn workspace_occurrence_kind_from_source_reference_kind_for_lsp(
    kind: &str,
) -> OmenaWorkspaceOccurrenceKindV0 {
    match kind {
        "sourceSelectorPrefixReference" => {
            OmenaWorkspaceOccurrenceKindV0::SourceSelectorPrefixReference
        }
        _ => OmenaWorkspaceOccurrenceKindV0::SourceSelectorReference,
    }
}

pub(super) fn workspace_occurrence_matches_target_style(
    occurrence: &OmenaWorkspaceOccurrenceV0,
    target_style_uri: Option<&str>,
) -> bool {
    target_style_uri.is_none_or(|target_uri| {
        occurrence
            .target_style_uri
            .as_deref()
            .is_none_or(|candidate_target_uri| {
                file_uri_equivalent(candidate_target_uri, target_uri)
            })
    })
}

pub(super) fn style_symbol_occurrence_from_workspace_occurrence_for_lsp(
    occurrence: &OmenaWorkspaceOccurrenceV0,
) -> Option<LspStyleSymbolOccurrenceV0> {
    let family = occurrence.family?;
    if family == OmenaWorkspaceOccurrenceFamilyV0::CssModuleSelector {
        return None;
    }
    Some(LspStyleSymbolOccurrenceV0 {
        moniker: occurrence.moniker.clone(),
        uri: occurrence.uri.clone(),
        kind: occurrence.kind,
        family,
        name: occurrence.name.clone(),
        range: occurrence.range,
        role: occurrence.role,
        namespace: occurrence.namespace.clone(),
    })
}

pub(super) fn workspace_occurrence_from_style_symbol_occurrence(
    document: &LspTextDocumentState,
    occurrence: &LspStyleSymbolOccurrenceV0,
) -> OmenaWorkspaceOccurrenceV0 {
    OmenaWorkspaceOccurrenceV0 {
        moniker: occurrence.moniker.clone(),
        uri: occurrence.uri.clone(),
        name: occurrence.name.clone(),
        range: occurrence.range,
        kind: occurrence.kind,
        role: occurrence.role,
        surface: OmenaWorkspaceOccurrenceSurfaceV0::OmenaLspStyleIndex,
        family: Some(occurrence.family),
        namespace: occurrence.namespace.clone(),
        target_style_uri: None,
        rename_target: document.origin == LspDocumentOrigin::Local,
    }
}

pub(super) fn style_symbol_occurrence_for_candidate(
    moniker: String,
    uri: &str,
    candidate: &LspStyleHoverCandidate,
    family: &'static str,
    role: &'static str,
) -> LspStyleSymbolOccurrenceV0 {
    LspStyleSymbolOccurrenceV0 {
        moniker,
        uri: uri.to_string(),
        kind: workspace_occurrence_kind_from_style_symbol_kind(candidate.kind)
            .unwrap_or(OmenaWorkspaceOccurrenceKindV0::CustomPropertyReference),
        family: workspace_occurrence_family_from_style_symbol_family(family)
            .unwrap_or(OmenaWorkspaceOccurrenceFamilyV0::Symbol),
        name: candidate.name.clone(),
        range: candidate.range,
        role: workspace_occurrence_role_from_style_symbol_role(role),
        namespace: candidate.namespace.clone(),
    }
}

fn workspace_occurrence_kind_from_style_symbol_kind(
    kind: &str,
) -> Option<OmenaWorkspaceOccurrenceKindV0> {
    match kind {
        "customPropertyDeclaration" => {
            Some(OmenaWorkspaceOccurrenceKindV0::CustomPropertyDeclaration)
        }
        "customPropertyReference" => Some(OmenaWorkspaceOccurrenceKindV0::CustomPropertyReference),
        "sassVariableDeclaration" => Some(OmenaWorkspaceOccurrenceKindV0::SassVariableDeclaration),
        "sassVariableReference" => Some(OmenaWorkspaceOccurrenceKindV0::SassVariableReference),
        "sassMixinDeclaration" => Some(OmenaWorkspaceOccurrenceKindV0::SassMixinDeclaration),
        "sassMixinInclude" => Some(OmenaWorkspaceOccurrenceKindV0::SassMixinInclude),
        "sassFunctionDeclaration" => Some(OmenaWorkspaceOccurrenceKindV0::SassFunctionDeclaration),
        "sassFunctionCall" => Some(OmenaWorkspaceOccurrenceKindV0::SassFunctionCall),
        _ => None,
    }
}

fn workspace_occurrence_role_from_style_symbol_role(role: &str) -> OmenaWorkspaceOccurrenceRoleV0 {
    if role == "definition" {
        OmenaWorkspaceOccurrenceRoleV0::Definition
    } else {
        OmenaWorkspaceOccurrenceRoleV0::Reference
    }
}

fn workspace_occurrence_family_from_style_symbol_family(
    family: &str,
) -> Option<OmenaWorkspaceOccurrenceFamilyV0> {
    match family {
        "customProperty" => Some(OmenaWorkspaceOccurrenceFamilyV0::CustomProperty),
        "variable" => Some(OmenaWorkspaceOccurrenceFamilyV0::Variable),
        "mixin" => Some(OmenaWorkspaceOccurrenceFamilyV0::Mixin),
        "function" => Some(OmenaWorkspaceOccurrenceFamilyV0::Function),
        "symbol" => Some(OmenaWorkspaceOccurrenceFamilyV0::Symbol),
        _ => None,
    }
}

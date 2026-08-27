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

#[allow(
    clippy::expect_used,
    reason = "the producer filter and exhaustive mapping test make this a fail-closed invariant"
)]
pub(super) fn style_symbol_occurrence_for_candidate(
    moniker: String,
    uri: &str,
    candidate: &LspStyleHoverCandidate,
    family: &'static str,
    role: &'static str,
) -> LspStyleSymbolOccurrenceV0 {
    let mut name = String::new();
    let _ = omena_syntax::ident::render_authored(&candidate.name, &mut name);
    let kind = workspace_occurrence_kind_from_style_symbol_kind(candidate.kind)
        .expect("filtered style symbol occurrence kind must be mapped");
    let family = workspace_occurrence_family_from_style_symbol_family(family)
        .expect("filtered style symbol occurrence family must be mapped");
    assert_eq!(
        kind.family(),
        family,
        "style symbol occurrence kind and family must name the same identity domain"
    );
    LspStyleSymbolOccurrenceV0 {
        moniker,
        uri: uri.to_string(),
        kind,
        family,
        name,
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
        "sassMixinReference" => Some(OmenaWorkspaceOccurrenceKindV0::SassMixinReference),
        "sassFunctionDeclaration" => Some(OmenaWorkspaceOccurrenceKindV0::SassFunctionDeclaration),
        "sassFunctionCall" => Some(OmenaWorkspaceOccurrenceKindV0::SassFunctionCall),
        "sassFunctionReference" => Some(OmenaWorkspaceOccurrenceKindV0::SassFunctionReference),
        "sassSymbolDeclaration" => Some(OmenaWorkspaceOccurrenceKindV0::SassSymbolDeclaration),
        "sassSymbolReference" => Some(OmenaWorkspaceOccurrenceKindV0::SassSymbolReference),
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

#[cfg(test)]
mod tests {
    use omena_query::{
        OmenaWorkspaceOccurrenceFamilyV0, OmenaWorkspaceOccurrenceSurfaceV0,
        OmenaWorkspaceOccurrenceV0, ParserPositionV0, ParserRangeV0,
        is_omena_query_sass_symbol_candidate_kind,
        omena_query_sass_symbol_kind_from_candidate_kind,
    };
    use omena_syntax::ident::{AuthoredPropertyTextV0, PropertyNameV0};

    use super::{
        style_symbol_occurrence_for_candidate, workspace_occurrence_kind_from_style_symbol_kind,
    };
    use crate::state::{LspStyleHoverCandidate, LspStyleSymbolOccurrenceV0};

    fn custom_property_style_occurrence(name: &str) -> LspStyleSymbolOccurrenceV0 {
        let candidate = LspStyleHoverCandidate {
            kind: "customPropertyDeclaration",
            name: AuthoredPropertyTextV0::new(name),
            selector_key: None,
            property_key: Some(PropertyNameV0::canonical_custom_key(name)),
            range: ParserRangeV0 {
                start: ParserPositionV0 {
                    line: 0,
                    character: 0,
                },
                end: ParserPositionV0 {
                    line: 0,
                    character: 7,
                },
            },
            source: "fixture",
            target_style_uri: None,
            namespace: None,
        };
        style_symbol_occurrence_for_candidate(
            "custom-property:file:///workspace/app.css#--token".to_string(),
            "file:///workspace/app.css",
            &candidate,
            "customProperty",
            "definition",
        )
    }

    fn sass_mixin_reference_style_occurrence(name: &str) -> LspStyleSymbolOccurrenceV0 {
        let candidate = LspStyleHoverCandidate {
            kind: "sassMixinReference",
            name: AuthoredPropertyTextV0::new(name),
            selector_key: None,
            property_key: None,
            range: ParserRangeV0 {
                start: ParserPositionV0 {
                    line: 0,
                    character: 0,
                },
                end: ParserPositionV0 {
                    line: 0,
                    character: 5,
                },
            },
            source: "fixture",
            target_style_uri: None,
            namespace: None,
        };
        style_symbol_occurrence_for_candidate(
            "sass-mixin:file:///workspace/app.scss#mixin".to_string(),
            "file:///workspace/app.scss",
            &candidate,
            "mixin",
            "reference",
        )
    }

    #[test]
    fn style_symbol_occurrence_identity_uses_custom_property_keys_without_changing_wire_name() {
        let escaped = custom_property_style_occurrence(r"--to\6b en");
        let plain = custom_property_style_occurrence("--token");

        assert_eq!(escaped, plain);
        let mut occurrences = vec![escaped.clone(), plain];
        occurrences.sort();
        occurrences.dedup();
        assert_eq!(occurrences.len(), 1);
        let serialized = serde_json::to_value(&escaped).unwrap_or_default();
        assert_eq!(serialized["name"], r"--to\6b en");
    }

    #[test]
    fn sass_mixin_reference_identity_preserves_escape_distinct_authored_names() {
        let escaped = sass_mixin_reference_style_occurrence(r"m\69 xin");
        let plain = sass_mixin_reference_style_occurrence("mixin");
        assert_eq!(escaped.kind.as_str(), "sassMixinReference");
        assert_ne!(escaped, plain);

        let to_workspace_occurrence =
            |occurrence: LspStyleSymbolOccurrenceV0| OmenaWorkspaceOccurrenceV0 {
                moniker: occurrence.moniker,
                uri: occurrence.uri,
                name: occurrence.name,
                range: occurrence.range,
                kind: occurrence.kind,
                role: occurrence.role,
                surface: OmenaWorkspaceOccurrenceSurfaceV0::OmenaLspStyleIndex,
                family: Some(occurrence.family),
                namespace: occurrence.namespace,
                target_style_uri: None,
                rename_target: true,
            };
        let mut occurrences = vec![
            to_workspace_occurrence(escaped),
            to_workspace_occurrence(plain),
        ];
        occurrences.sort();
        occurrences.dedup();
        assert_eq!(occurrences.len(), 2);
    }

    #[test]
    #[allow(
        clippy::expect_used,
        reason = "the assertion below proves every producer-accepted kind is mapped"
    )]
    fn every_accepted_sass_candidate_kind_has_a_lossless_occurrence_mapping() {
        let rows = [
            (
                "sassVariableDeclaration",
                "variable",
                OmenaWorkspaceOccurrenceFamilyV0::Variable,
            ),
            (
                "sassVariableReference",
                "variable",
                OmenaWorkspaceOccurrenceFamilyV0::Variable,
            ),
            (
                "sassMixinDeclaration",
                "mixin",
                OmenaWorkspaceOccurrenceFamilyV0::Mixin,
            ),
            (
                "sassMixinInclude",
                "mixin",
                OmenaWorkspaceOccurrenceFamilyV0::Mixin,
            ),
            (
                "sassMixinReference",
                "mixin",
                OmenaWorkspaceOccurrenceFamilyV0::Mixin,
            ),
            (
                "sassFunctionDeclaration",
                "function",
                OmenaWorkspaceOccurrenceFamilyV0::Function,
            ),
            (
                "sassFunctionCall",
                "function",
                OmenaWorkspaceOccurrenceFamilyV0::Function,
            ),
            (
                "sassFunctionReference",
                "function",
                OmenaWorkspaceOccurrenceFamilyV0::Function,
            ),
            (
                "sassSymbolDeclaration",
                "symbol",
                OmenaWorkspaceOccurrenceFamilyV0::Symbol,
            ),
            (
                "sassSymbolReference",
                "symbol",
                OmenaWorkspaceOccurrenceFamilyV0::Symbol,
            ),
        ];

        assert_eq!(rows.len(), 10);
        for (candidate_kind, expected_query_family, expected_occurrence_family) in rows {
            assert!(is_omena_query_sass_symbol_candidate_kind(candidate_kind));
            assert_eq!(
                omena_query_sass_symbol_kind_from_candidate_kind(candidate_kind),
                Some(expected_query_family)
            );
            let occurrence_kind = workspace_occurrence_kind_from_style_symbol_kind(candidate_kind)
                .expect("every accepted Sass candidate kind must map without a fallback");
            assert_eq!(occurrence_kind.as_str(), candidate_kind);
            assert_eq!(occurrence_kind.family(), expected_occurrence_family);
        }
    }
}

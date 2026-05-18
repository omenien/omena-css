use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, PathBuf};

use super::*;

pub fn summarize_omena_query_refs_for_class(
    selector_name: &str,
    target_style_uri: Option<&str>,
    include_declaration: bool,
    definitions: &[OmenaQueryStyleSelectorDefinitionV0],
    references: &[OmenaQuerySourceSelectorReferenceCandidateV0],
) -> OmenaQueryRefsForClassV0 {
    let mut locations = Vec::new();

    if include_declaration {
        locations.extend(
            definitions
                .iter()
                .filter(|definition| definition.name == selector_name)
                .filter(|definition| {
                    target_style_uri.is_none_or(|target_uri| {
                        file_uri_equivalent(target_uri, definition.uri.as_str())
                    })
                })
                .map(|definition| OmenaQueryReferenceLocationV0 {
                    uri: definition.uri.clone(),
                    range: definition.range,
                    name: definition.name.clone(),
                    role: "definition",
                    source: "omenaQueryStyleSelectorDefinitions",
                }),
        );
    }

    for reference in references {
        let reference_candidate = OmenaQuerySourceSelectorCandidateV0 {
            kind: reference.kind,
            name: reference.name.clone(),
            range: reference.range,
            source: reference.source,
            target_style_uri: reference.target_style_uri.clone(),
        };
        if !source_selector_candidate_matches_target_uri(&reference_candidate, target_style_uri) {
            continue;
        }
        let selector_names = resolve_omena_query_source_candidate_selector_names(
            &reference_candidate,
            definitions,
            target_style_uri,
        );
        if selector_names.iter().any(|name| name == selector_name) {
            locations.push(OmenaQueryReferenceLocationV0 {
                uri: reference.uri.clone(),
                range: reference.range,
                name: selector_name.to_string(),
                role: "reference",
                source: "omenaQuerySourceSelectorReferences",
            });
        }
    }

    locations.sort_by_key(|location| {
        (
            reference_location_role_rank(location.role),
            location.uri.clone(),
            location.range.start.line,
            location.range.start.character,
        )
    });
    locations.dedup_by(|left, right| left.uri == right.uri && left.range == right.range);

    OmenaQueryRefsForClassV0 {
        schema_version: "0",
        product: "omena-query.refs-for-class",
        selector_name: selector_name.to_string(),
        target_style_uri: target_style_uri.map(ToString::to_string),
        include_declaration,
        location_count: locations.len(),
        locations,
        ready_surfaces: vec!["refsForClass", "workspaceWideSelectorReferences"],
    }
}

pub fn summarize_omena_query_rename_plan(
    selector_name: &str,
    new_name: &str,
    target_style_uri: Option<&str>,
    definitions: &[OmenaQueryStyleSelectorDefinitionV0],
    references: &[OmenaQuerySourceSelectorReferenceEditTargetV0],
) -> OmenaQueryRenamePlanV0 {
    let edits = resolve_omena_query_selector_rename_edits(
        selector_name,
        new_name,
        target_style_uri,
        definitions,
        references,
    );
    OmenaQueryRenamePlanV0 {
        schema_version: "0",
        product: "omena-query.rename-plan",
        selector_name: selector_name.to_string(),
        new_name: new_name.to_string(),
        target_style_uri: target_style_uri.map(ToString::to_string),
        edit_count: edits.len(),
        edits,
        ready_surfaces: vec!["renamePlan", "workspaceWideSelectorRename"],
    }
}

pub fn summarize_omena_query_refs_for_workspace_class(
    selector_name: &str,
    target_style_uri: Option<&str>,
    include_declaration: bool,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQueryRefsForClassV0 {
    let definitions = summarize_omena_query_style_selector_definitions(style_sources);
    let references = collect_omena_query_source_selector_reference_candidates(
        style_sources,
        source_documents,
        package_manifests,
    );
    summarize_omena_query_refs_for_class(
        selector_name,
        target_style_uri,
        include_declaration,
        definitions.as_slice(),
        references.as_slice(),
    )
}

pub fn summarize_omena_query_rename_plan_for_workspace_class(
    selector_name: &str,
    new_name: &str,
    target_style_uri: Option<&str>,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQueryRenamePlanV0 {
    let definitions = summarize_omena_query_style_selector_definitions(style_sources);
    let references = collect_omena_query_source_selector_reference_edit_targets(
        style_sources,
        source_documents,
        package_manifests,
    );
    summarize_omena_query_rename_plan(
        selector_name,
        new_name,
        target_style_uri,
        definitions.as_slice(),
        references.as_slice(),
    )
}

pub fn summarize_omena_query_missing_selector_diagnostic(
    target_style_uri: &str,
    target_style_source: &str,
    selector_name: &str,
    source_reference_range: ParserRangeV0,
) -> OmenaQuerySourceDiagnosticV0 {
    let insertion_range = end_of_source_range(target_style_source);
    let has_existing_style_content = !target_style_source.trim().is_empty();
    OmenaQuerySourceDiagnosticV0 {
        code: "missingSelector",
        range: source_reference_range,
        message: format!(
            "CSS Module selector '.{selector_name}' not found in indexed style tokens."
        ),
        create_selector: Some(OmenaQueryCreateSelectorActionV0 {
            uri: target_style_uri.to_string(),
            range: insertion_range,
            new_text: if has_existing_style_content {
                format!("\n\n.{selector_name} {{\n}}\n")
            } else {
                format!(".{selector_name} {{\n}}\n")
            },
            selector_name: selector_name.to_string(),
        }),
    }
}

pub fn summarize_omena_query_source_diagnostics_for_file(
    source_uri: &str,
    candidates: &[OmenaQuerySourceMissingSelectorDiagnosticCandidateV0],
) -> OmenaQuerySourceDiagnosticsForFileV0 {
    let diagnostics = candidates
        .iter()
        .map(|candidate| {
            summarize_omena_query_missing_selector_diagnostic(
                candidate.target_style_uri.as_str(),
                candidate.target_style_source.as_str(),
                candidate.selector_name.as_str(),
                candidate.source_reference_range,
            )
        })
        .collect::<Vec<_>>();
    OmenaQuerySourceDiagnosticsForFileV0 {
        schema_version: "0",
        product: "omena-query.diagnostics-for-file",
        file_uri: source_uri.to_string(),
        file_kind: "source",
        diagnostic_count: diagnostics.len(),
        diagnostics,
        ready_surfaces: vec![
            "sourceMissingSelectorDiagnostics",
            "crossLanguageDiagnostics",
        ],
    }
}

pub fn summarize_omena_query_source_diagnostics_for_workspace_file(
    source_path: &str,
    source_source: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQuerySourceDiagnosticsForFileV0 {
    let available_style_paths = style_sources
        .iter()
        .map(|source| source.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let style_sources_by_path = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<BTreeMap<_, _>>();
    let definitions = summarize_omena_query_style_selector_definitions(style_sources);
    let imports = summarize_omena_query_source_import_declarations(source_source);
    let mut imported_style_bindings = Vec::new();
    let mut classnames_bind_bindings = Vec::new();
    let mut diagnostics = Vec::new();

    for import in imports.imports {
        if import.specifier == "classnames/bind" {
            classnames_bind_bindings.push(import.binding);
            continue;
        }

        if !is_query_source_style_module_specifier(import.specifier.as_str()) {
            continue;
        }

        match resolve_style_module_source(
            source_path,
            import.specifier.as_str(),
            &available_style_paths,
            package_manifests,
        ) {
            Some(style_path) => {
                imported_style_bindings.push(OmenaQuerySourceImportedStyleBindingV0 {
                    binding: import.binding,
                    style_uri: style_path,
                })
            }
            None => diagnostics.push(OmenaQuerySourceDiagnosticV0 {
                code: "missingModule",
                range: parser_range_for_byte_span(source_source, import.specifier_byte_span),
                message: format!(
                    "Cannot resolve CSS Module '{}'. The file does not exist.",
                    import.specifier
                ),
                create_selector: None,
            }),
        }
    }

    if !imported_style_bindings.is_empty() {
        let index = summarize_omena_query_source_syntax_index(
            source_source,
            imported_style_bindings,
            classnames_bind_bindings,
        );
        for reference in index.selector_references {
            let Some(target_style_uri) = reference.target_style_uri.as_deref() else {
                continue;
            };
            let Some(selector_name) = reference.selector_name.clone().or_else(|| {
                source_reference_text_selector_name(source_source, reference.byte_span)
            }) else {
                continue;
            };
            let candidate = OmenaQuerySourceSelectorCandidateV0 {
                kind: match reference.match_kind {
                    OmenaQuerySourceSelectorReferenceMatchKindV0::Exact => {
                        "sourceSelectorReference"
                    }
                    OmenaQuerySourceSelectorReferenceMatchKindV0::Prefix => {
                        "sourceSelectorPrefixReference"
                    }
                },
                name: selector_name.clone(),
                range: parser_range_for_byte_span(source_source, reference.byte_span),
                source: "omenaQuerySourceSyntaxIndex",
                target_style_uri: Some(target_style_uri.to_string()),
            };
            if !resolve_omena_query_style_selector_definitions_for_source_candidate(
                &candidate,
                definitions.as_slice(),
            )
            .is_empty()
            {
                continue;
            }
            diagnostics.push(
                summarize_omena_query_unresolved_source_reference_diagnostic(
                    source_source,
                    &reference,
                    selector_name.as_str(),
                    style_sources_by_path.get(target_style_uri).copied(),
                ),
            );
        }
    }

    diagnostics.sort_by_key(|diagnostic| {
        (
            diagnostic.range.start.line,
            diagnostic.range.start.character,
            diagnostic.code,
            diagnostic.message.clone(),
        )
    });
    diagnostics.dedup_by(|left, right| {
        left.code == right.code && left.range == right.range && left.message == right.message
    });

    OmenaQuerySourceDiagnosticsForFileV0 {
        schema_version: "0",
        product: "omena-query.diagnostics-for-file",
        file_uri: source_path.to_string(),
        file_kind: "source",
        diagnostic_count: diagnostics.len(),
        diagnostics,
        ready_surfaces: vec![
            "sourceMissingModuleDiagnostics",
            "sourceMissingSelectorDiagnostics",
            "sourceResolvedClassDiagnostics",
            "crossLanguageDiagnostics",
        ],
    }
}

fn summarize_omena_query_style_selector_definitions(
    style_sources: &[OmenaQueryStyleSourceInputV0],
) -> Vec<OmenaQueryStyleSelectorDefinitionV0> {
    let mut definitions = Vec::new();
    for source in style_sources {
        let Some(candidates) = summarize_omena_query_style_hover_candidates(
            source.style_path.as_str(),
            source.style_source.as_str(),
        ) else {
            continue;
        };
        definitions.extend(candidates.candidates.into_iter().filter_map(|candidate| {
            (candidate.kind == "selector").then(|| OmenaQueryStyleSelectorDefinitionV0 {
                uri: source.style_path.clone(),
                name: candidate.name,
                range: candidate.range,
            })
        }));
    }
    definitions.sort_by_key(|definition| {
        (
            definition.uri.clone(),
            definition.range.start.line,
            definition.range.start.character,
            definition.name.clone(),
        )
    });
    definitions.dedup();
    definitions
}

fn collect_omena_query_source_selector_reference_candidates(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaQuerySourceSelectorReferenceCandidateV0> {
    collect_omena_query_source_selector_references(
        style_sources,
        source_documents,
        package_manifests,
    )
    .into_iter()
    .map(|reference| reference.candidate)
    .collect()
}

fn collect_omena_query_source_selector_reference_edit_targets(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaQuerySourceSelectorReferenceEditTargetV0> {
    collect_omena_query_source_selector_references(
        style_sources,
        source_documents,
        package_manifests,
    )
    .into_iter()
    .filter_map(|reference| {
        reference
            .is_exact
            .then_some(OmenaQuerySourceSelectorReferenceEditTargetV0 {
                uri: reference.candidate.uri,
                name: reference.candidate.name,
                range: reference.candidate.range,
                target_style_uri: reference.candidate.target_style_uri,
            })
    })
    .collect()
}

fn collect_omena_query_source_selector_references(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaQueryWorkspaceSourceReferenceCandidateV0> {
    let available_style_paths = style_sources
        .iter()
        .map(|source| source.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let mut references = Vec::new();

    for document in source_documents {
        let imports = summarize_omena_query_source_import_declarations(&document.source_source);
        let mut imported_style_bindings = Vec::new();
        let mut classnames_bind_bindings = Vec::new();

        for import in imports.imports {
            if import.specifier == "classnames/bind" {
                classnames_bind_bindings.push(import.binding);
                continue;
            }
            let Some(style_uri) = resolve_style_module_source(
                &document.source_path,
                &import.specifier,
                &available_style_paths,
                package_manifests,
            ) else {
                continue;
            };
            imported_style_bindings.push(OmenaQuerySourceImportedStyleBindingV0 {
                binding: import.binding,
                style_uri,
            });
        }

        if imported_style_bindings.is_empty() {
            continue;
        }

        let mut index = summarize_omena_query_source_syntax_index(
            &document.source_source,
            imported_style_bindings,
            classnames_bind_bindings,
        );
        canonicalize_omena_query_source_selector_references(&mut index.selector_references);

        for reference in index.selector_references {
            let Some(name) = reference.selector_name.clone().or_else(|| {
                source_reference_text_selector_name(&document.source_source, reference.byte_span)
            }) else {
                continue;
            };
            let is_exact = matches!(
                reference.match_kind,
                OmenaQuerySourceSelectorReferenceMatchKindV0::Exact
            );
            references.push(OmenaQueryWorkspaceSourceReferenceCandidateV0 {
                is_exact,
                candidate: OmenaQuerySourceSelectorReferenceCandidateV0 {
                    uri: document.source_path.clone(),
                    kind: if is_exact {
                        "sourceSelectorReference"
                    } else {
                        "sourceSelectorPrefixReference"
                    },
                    name,
                    range: parser_range_for_byte_span(&document.source_source, reference.byte_span),
                    source: "omenaQuerySourceSyntaxIndex",
                    target_style_uri: reference.target_style_uri,
                },
            });
        }
    }

    references.sort_by_key(|reference| {
        (
            reference.candidate.uri.clone(),
            reference.candidate.range.start.line,
            reference.candidate.range.start.character,
            reference.candidate.name.clone(),
        )
    });
    references.dedup_by(|left, right| {
        left.candidate.uri == right.candidate.uri
            && left.candidate.range == right.candidate.range
            && left.candidate.name == right.candidate.name
            && left.candidate.target_style_uri == right.candidate.target_style_uri
    });
    references
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OmenaQueryWorkspaceSourceReferenceCandidateV0 {
    is_exact: bool,
    candidate: OmenaQuerySourceSelectorReferenceCandidateV0,
}

fn summarize_omena_query_unresolved_source_reference_diagnostic(
    source: &str,
    reference: &OmenaQuerySourceSelectorReferenceFactV0,
    selector_name: &str,
    target_style_source: Option<&str>,
) -> OmenaQuerySourceDiagnosticV0 {
    let range = parser_range_for_byte_span(source, reference.byte_span);
    let reference_text = source
        .get(reference.byte_span.start..reference.byte_span.end)
        .unwrap_or_default()
        .trim_matches(['"', '\'', '`']);
    let code = match reference.match_kind {
        OmenaQuerySourceSelectorReferenceMatchKindV0::Exact if reference_text == selector_name => {
            "missingStaticClass"
        }
        OmenaQuerySourceSelectorReferenceMatchKindV0::Exact => "missingResolvedClassValues",
        OmenaQuerySourceSelectorReferenceMatchKindV0::Prefix if reference_text == selector_name => {
            "missingTemplatePrefix"
        }
        OmenaQuerySourceSelectorReferenceMatchKindV0::Prefix => "missingResolvedClassDomain",
    };
    let create_selector = reference
        .target_style_uri
        .as_deref()
        .zip(target_style_source)
        .filter(|_| {
            matches!(
                reference.match_kind,
                OmenaQuerySourceSelectorReferenceMatchKindV0::Exact
            )
        })
        .and_then(|(target_style_uri, target_style_source)| {
            summarize_omena_query_missing_selector_diagnostic(
                target_style_uri,
                target_style_source,
                selector_name,
                range,
            )
            .create_selector
        });

    OmenaQuerySourceDiagnosticV0 {
        code,
        range,
        message: query_source_diagnostic_message(code, selector_name),
        create_selector,
    }
}

fn query_source_diagnostic_message(code: &str, selector_name: &str) -> String {
    match code {
        "missingStaticClass" => format!("Class '.{selector_name}' not found in target CSS Module."),
        "missingTemplatePrefix" => {
            format!("No class starting with '{selector_name}' found in target CSS Module.")
        }
        "missingResolvedClassValues" => {
            format!("Missing class for possible value: '{selector_name}'.")
        }
        "missingResolvedClassDomain" => {
            format!("No class matched resolved prefix '{selector_name}'.")
        }
        _ => "Source diagnostic reported by omena-query.".to_string(),
    }
}

fn is_query_source_style_module_specifier(specifier: &str) -> bool {
    specifier.contains(".module.")
        || specifier.ends_with(".css")
        || specifier.ends_with(".scss")
        || specifier.ends_with(".sass")
        || specifier.ends_with(".less")
}

pub fn resolve_omena_query_source_provider_candidates(
    source_candidates: Vec<OmenaQuerySourceSelectorCandidateV0>,
    definitions: &[OmenaQueryStyleSelectorDefinitionV0],
) -> OmenaQuerySourceProviderCandidateResolutionV0 {
    if definitions.is_empty() {
        return OmenaQuerySourceProviderCandidateResolutionV0 {
            schema_version: "0",
            product: "omena-query.source-provider-candidate-resolution",
            matched: Vec::new(),
            unresolved: Vec::new(),
        };
    }

    let (mut matched, mut unresolved): (Vec<_>, Vec<_>) =
        source_candidates.into_iter().partition(|candidate| {
            definitions.iter().any(|definition| {
                source_selector_candidate_matches_definition(candidate, definition)
            })
        });
    matched.sort();
    unresolved.sort();
    OmenaQuerySourceProviderCandidateResolutionV0 {
        schema_version: "0",
        product: "omena-query.source-provider-candidate-resolution",
        matched,
        unresolved,
    }
}

pub fn resolve_omena_query_style_selector_definitions_for_source_candidate(
    candidate: &OmenaQuerySourceSelectorCandidateV0,
    definitions: &[OmenaQueryStyleSelectorDefinitionV0],
) -> Vec<OmenaQueryStyleSelectorDefinitionV0> {
    let mut matched = definitions
        .iter()
        .filter(|definition| source_selector_candidate_matches_definition(candidate, definition))
        .cloned()
        .collect::<Vec<_>>();
    matched.sort_by_key(|definition| {
        (
            definition.uri.clone(),
            definition.range.start.line,
            definition.range.start.character,
            definition.name.clone(),
        )
    });
    matched.dedup();
    matched
}

pub fn resolve_omena_query_source_candidate_selector_names(
    candidate: &OmenaQuerySourceSelectorCandidateV0,
    definitions: &[OmenaQueryStyleSelectorDefinitionV0],
    target_style_uri: Option<&str>,
) -> Vec<String> {
    if candidate.kind != "sourceSelectorPrefixReference" {
        return vec![candidate.name.clone()];
    }

    let mut names = definitions
        .iter()
        .filter(|definition| source_selector_candidate_matches_definition(candidate, definition))
        .filter(|definition| {
            candidate
                .target_style_uri
                .as_deref()
                .or(target_style_uri)
                .is_none_or(|target_uri| file_uri_equivalent(target_uri, definition.uri.as_str()))
        })
        .map(|definition| definition.name.clone())
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    names
}

pub fn resolve_omena_query_selector_rename_edits(
    selector_name: &str,
    new_name: &str,
    target_style_uri: Option<&str>,
    definitions: &[OmenaQueryStyleSelectorDefinitionV0],
    references: &[OmenaQuerySourceSelectorReferenceEditTargetV0],
) -> Vec<OmenaQueryWorkspaceTextEditV0> {
    let replacement = new_name.trim_start_matches('.');
    if replacement.is_empty() {
        return Vec::new();
    }

    let mut edits = definitions
        .iter()
        .filter(|definition| definition.name == selector_name)
        .filter(|definition| {
            target_style_uri
                .is_none_or(|target_uri| file_uri_equivalent(target_uri, definition.uri.as_str()))
        })
        .map(|definition| OmenaQueryWorkspaceTextEditV0 {
            uri: definition.uri.clone(),
            range: definition.range,
            new_text: replacement.to_string(),
        })
        .chain(
            references
                .iter()
                .filter(|reference| reference.name == selector_name)
                .filter(|reference| {
                    source_reference_matches_target_style(reference, target_style_uri)
                })
                .map(|reference| OmenaQueryWorkspaceTextEditV0 {
                    uri: reference.uri.clone(),
                    range: reference.range,
                    new_text: replacement.to_string(),
                }),
        )
        .collect::<Vec<_>>();
    edits.sort_by_key(|edit| {
        (
            edit.uri.clone(),
            edit.range.start.line,
            edit.range.start.character,
            edit.range.end.line,
            edit.range.end.character,
        )
    });
    edits
}

fn source_selector_candidate_matches_definition(
    candidate: &OmenaQuerySourceSelectorCandidateV0,
    definition: &OmenaQueryStyleSelectorDefinitionV0,
) -> bool {
    let selector_matches = if candidate.kind == "sourceSelectorPrefixReference" {
        definition.name.starts_with(candidate.name.as_str())
    } else {
        definition.name == candidate.name
    };
    selector_matches
        && candidate
            .target_style_uri
            .as_deref()
            .is_none_or(|target_uri| file_uri_equivalent(target_uri, definition.uri.as_str()))
}

fn source_reference_matches_target_style(
    reference: &OmenaQuerySourceSelectorReferenceEditTargetV0,
    target_style_uri: Option<&str>,
) -> bool {
    target_style_uri.is_none_or(|target_uri| {
        reference
            .target_style_uri
            .as_deref()
            .is_none_or(|candidate_target_uri| {
                file_uri_equivalent(candidate_target_uri, target_uri)
            })
    })
}

fn source_selector_candidate_matches_target_uri(
    candidate: &OmenaQuerySourceSelectorCandidateV0,
    target_style_uri: Option<&str>,
) -> bool {
    target_style_uri.is_none_or(|target_uri| {
        candidate
            .target_style_uri
            .as_deref()
            .is_none_or(|candidate_target_uri| {
                file_uri_equivalent(candidate_target_uri, target_uri)
            })
    })
}

fn reference_location_role_rank(role: &str) -> u8 {
    match role {
        "definition" => 0,
        "reference" => 1,
        _ => 2,
    }
}

fn file_uri_equivalent(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    match (
        file_uri_to_normalized_path(left),
        file_uri_to_normalized_path(right),
    ) {
        (Some(left_path), Some(right_path)) => left_path == right_path,
        _ => false,
    }
}

fn file_uri_to_normalized_path(uri: &str) -> Option<PathBuf> {
    let raw_path = uri.strip_prefix("file://")?;
    Some(normalize_path(PathBuf::from(percent_decode_uri_path(
        raw_path,
    )?)))
}

fn percent_decode_uri_path(raw_path: &str) -> Option<String> {
    let bytes = raw_path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = bytes.get(index + 1).and_then(|byte| hex_value(*byte))?;
            let low = bytes.get(index + 2).and_then(|byte| hex_value(*byte))?;
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(_) | Component::RootDir | Component::Prefix(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

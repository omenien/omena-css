use crate::{
    LspStyleHoverCandidate, LspTextDocumentState,
    protocol::{is_style_document_uri, parser_range_for_byte_span},
};
use omena_query::{
    OmenaQuerySourceImportedStyleBindingV0 as ImportedStyleBinding,
    OmenaQuerySourceSelectorReferenceFactV0 as SourceSelectorReferenceFact,
    OmenaQuerySourceSelectorReferenceMatchKindV0 as SourceSelectorReferenceMatchKind,
    OmenaQuerySourceSyntaxIndexV0 as SourceSyntaxIndex, OmenaQueryStyleResolutionInputsV0,
    StyleLanguage, collect_omena_query_vue_style_module_bindings,
    resolve_omena_query_style_uri_for_specifier_with_resolution_inputs,
    summarize_omena_query_source_import_declarations_for_source_language,
    summarize_omena_query_source_syntax_index_for_source_language,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceImportIndex {
    pub(crate) imported_style_bindings: Vec<ImportedStyleBinding>,
    pub(crate) classnames_bind_bindings: Vec<String>,
    pub(crate) has_unresolved_style_import: bool,
}

pub(crate) fn source_selector_candidates_from_index(
    document: &LspTextDocumentState,
    index: &SourceSyntaxIndex,
) -> Vec<LspStyleHoverCandidate> {
    let mut candidates: Vec<LspStyleHoverCandidate> = index
        .selector_references
        .iter()
        .map(|reference| source_reference_candidate(document, reference))
        .collect();
    candidates.sort();
    candidates.dedup();
    candidates
}

pub(crate) fn build_source_syntax_index(
    document: &LspTextDocumentState,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> SourceSyntaxIndex {
    if is_style_document_uri(document.uri.as_str()) {
        return SourceSyntaxIndex::default();
    }

    let imports = collect_source_imports(document, resolution_inputs);
    summarize_omena_query_source_syntax_index_for_source_language(
        document.uri.as_str(),
        document.text.as_str(),
        Some(document.language_id.as_str()),
        imports.imported_style_bindings,
        imports.classnames_bind_bindings,
    )
}

pub(crate) fn collect_source_imports(
    document: &LspTextDocumentState,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> SourceImportIndex {
    let source = document.text.as_str();
    let mut imports = SourceImportIndex {
        imported_style_bindings: Vec::new(),
        classnames_bind_bindings: Vec::new(),
        has_unresolved_style_import: false,
    };
    let summary = summarize_omena_query_source_import_declarations_for_source_language(
        document.uri.as_str(),
        source,
        Some(document.language_id.as_str()),
    );
    for import in summary.imports {
        if import.specifier == "classnames/bind" {
            imports.classnames_bind_bindings.push(import.binding);
        } else if StyleLanguage::from_module_path(import.specifier.as_str()).is_some() {
            if let Some(style_uri) =
                resolve_omena_query_style_uri_for_specifier_with_resolution_inputs(
                    document.uri.as_str(),
                    document.workspace_folder_uri.as_deref(),
                    import.specifier.as_str(),
                    resolution_inputs,
                )
            {
                imports.imported_style_bindings.push(ImportedStyleBinding {
                    binding: import.binding,
                    style_uri,
                });
            } else {
                imports.has_unresolved_style_import = true;
            }
        }
    }
    if is_vue_document(document) && has_vue_module_style_block(source) {
        for binding in collect_omena_query_vue_style_module_bindings(
            document.uri.as_str(),
            source,
            Some(document.language_id.as_str()),
        ) {
            imports.imported_style_bindings.push(ImportedStyleBinding {
                binding,
                style_uri: document.uri.clone(),
            });
        }
    }
    imports
        .imported_style_bindings
        .sort_by(|left, right| left.binding.cmp(&right.binding));
    imports
        .imported_style_bindings
        .dedup_by(|left, right| left.binding == right.binding && left.style_uri == right.style_uri);
    imports.classnames_bind_bindings.sort();
    imports.classnames_bind_bindings.dedup();
    imports
}

fn is_vue_document(document: &LspTextDocumentState) -> bool {
    document.language_id == "vue" || document.uri.ends_with(".vue")
}

fn has_vue_module_style_block(source: &str) -> bool {
    let lower = source.to_ascii_lowercase();
    let mut cursor = 0usize;
    while let Some(relative_start) = lower[cursor..].find("<style") {
        let tag_start = cursor + relative_start;
        let Some(relative_tag_end) = lower[tag_start..].find('>') else {
            break;
        };
        let tag = &lower[tag_start..tag_start + relative_tag_end + 1];
        if tag.contains("module") {
            return true;
        }
        cursor = tag_start + relative_tag_end + 1;
    }
    false
}

fn source_reference_candidate(
    document: &LspTextDocumentState,
    reference: &SourceSelectorReferenceFact,
) -> LspStyleHoverCandidate {
    let name = reference.selector_name.clone().unwrap_or_else(|| {
        document.text[reference.byte_span.start..reference.byte_span.end].to_string()
    });
    LspStyleHoverCandidate {
        kind: match reference.match_kind {
            SourceSelectorReferenceMatchKind::Exact => "sourceSelectorReference",
            SourceSelectorReferenceMatchKind::Prefix => "sourceSelectorPrefixReference",
        },
        name,
        range: parser_range_for_byte_span(document.text.as_str(), reference.byte_span),
        source: "omenaQuerySourceSyntaxIndex",
        target_style_uri: reference.target_style_uri.clone(),
        namespace: None,
    }
}

use omena_query::{
    OmenaQuerySourceDomainClassReferenceFactV0 as SourceDomainClassReferenceFact,
    OmenaQueryStyleIntelligenceGraphBindingV0 as StyleIntelligenceGraphBinding,
    OmenaQueryStyleIntelligenceHoverV0 as StyleIntelligenceHover,
    OmenaQueryStyleIntelligenceSnapshotV0 as StyleIntelligenceSnapshot, ParserPositionV0,
    ParserRangeV0, omena_query_style_intelligence_hover_at_offset,
};
use serde_json::{Value, json};

use crate::{
    LspShellState, LspTextDocumentState,
    protocol::{byte_offset_for_parser_position, parser_range_for_byte_span},
    style_selector_definitions_from_open_documents,
};

const UTILITY_PROVIDER_ID: &str = "tailwind-uno-utility-domain";

fn source_domain_reference_at_position(
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<&SourceDomainClassReferenceFact> {
    let offset = byte_offset_for_parser_position(document.text.as_str(), position)?;
    document
        .source_syntax_index
        .domain_class_references
        .iter()
        .find(|reference| offset >= reference.byte_span.start && offset <= reference.byte_span.end)
}

pub(crate) fn source_domain_reference_trace_at_position(
    state: &LspShellState,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<Value> {
    let reference = source_domain_reference_at_position(document, position)?;
    let hover = provider_hover_for_reference(state, document, reference)?;
    let current = hover.current_option.as_deref().unwrap_or("*");
    let validity = source_domain_reference_validity(&hover);
    let range = parser_range_for_byte_span(document.text.as_str(), reference.byte_span);
    let definitions = hover
        .graph_bindings
        .iter()
        .map(|binding| {
            json!({
                "uri": binding.uri,
                "name": binding.class_name,
                "range": binding.range,
                "source": "styleSelectorDefinitionIndex",
            })
        })
        .collect::<Vec<_>>();
    let rendered_markdown = render_source_domain_reference_hover_text(&hover);
    let (resolution_path, ready_surfaces) = if hover.provider_id == UTILITY_PROVIDER_ID {
        (
            vec![
                "sourceSyntaxIndex",
                "classValueUniverseProvider",
                "styleSelectorDefinitionIndex",
                "sourceDomainReferenceHover",
            ],
            vec![
                "explainHoverTraceRpc",
                "sourceSyntaxIndex",
                "classValueUniverseProvider",
                "styleSelectorDefinitionIndex",
            ],
        )
    } else {
        (
            vec![
                "sourceSyntaxIndex",
                "classValueUniverseProvider",
                "sourceDomainReferenceHover",
            ],
            vec![
                "explainHoverTraceRpc",
                "sourceSyntaxIndex",
                "classValueUniverseProvider",
            ],
        )
    };

    Some(json!({
        "schemaVersion": "0",
        "product": "omena-lsp-server.explain-hover-trace",
        "documentUri": document.uri.as_str(),
        "workspaceFolderUri": document.workspace_folder_uri.as_deref(),
        "fileKind": "source",
        "languageId": document.language_id.as_str(),
        "queryPosition": position,
        "matched": true,
        "reason": "domainClassReferenceResolved",
        "hoverKind": "domainClassReference",
        "range": range,
        "sourceOwner": hover.owner_name,
        "domain": hover.domain,
        "axisName": hover.axis_name,
        "optionName": reference.option_name,
        "prefix": reference.prefix,
        "currentOption": current,
        "validity": validity,
        "knownOptions": hover.known_options,
        "knownPatterns": hover.known_patterns,
        "unresolvedReasons": hover.unresolved_reasons,
        "candidateCount": 1,
        "definitionCount": definitions.len(),
        "candidates": [],
        "definitions": definitions,
        "renderedMarkdown": rendered_markdown,
        "resolutionPath": resolution_path,
        "readySurfaces": ready_surfaces,
    }))
}

pub(crate) fn source_domain_reference_hover_at_position(
    state: &LspShellState,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<(ParserRangeV0, String)> {
    let reference = source_domain_reference_at_position(document, position)?;
    let hover = provider_hover_for_reference(state, document, reference)?;
    Some((
        parser_range_for_byte_span(document.text.as_str(), reference.byte_span),
        render_source_domain_reference_hover_text(&hover),
    ))
}

fn provider_hover_for_reference(
    state: &LspShellState,
    document: &LspTextDocumentState,
    reference: &SourceDomainClassReferenceFact,
) -> Option<StyleIntelligenceHover> {
    let graph_bindings = if reference.plugin_id == UTILITY_PROVIDER_ID {
        reference
            .option_name
            .as_deref()
            .into_iter()
            .flat_map(|class_name| {
                style_selector_definitions_from_open_documents(
                    state,
                    class_name,
                    document.workspace_folder_uri.as_deref(),
                )
                .into_iter()
                .map(move |(uri, definition)| StyleIntelligenceGraphBinding {
                    class_name: class_name.to_string(),
                    uri,
                    range: definition.range,
                })
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let snapshot = StyleIntelligenceSnapshot::with_graph_bindings(
        &document.source_syntax_index,
        graph_bindings.as_slice(),
    );
    omena_query_style_intelligence_hover_at_offset(&snapshot, reference.byte_span.start)
}

fn source_domain_reference_validity(hover: &StyleIntelligenceHover) -> &'static str {
    hover
        .current_option
        .as_ref()
        .map_or("prefix option", |option| {
            if hover.known_options.iter().any(|known| known == option) {
                "known option"
            } else if hover.unresolved_reasons.is_empty() {
                "unknown option"
            } else {
                "indeterminate option"
            }
        })
}

fn render_source_domain_reference_hover_text(hover: &StyleIntelligenceHover) -> String {
    let current = hover.current_option.as_deref().unwrap_or("*");
    let validity = source_domain_reference_validity(hover);
    let known_options =
        if hover.known_options.is_empty() && hover.provider_id == UTILITY_PROVIDER_ID {
            "No enumerated options are indexed.".to_string()
        } else if hover.known_options.is_empty() {
            "No known options indexed.".to_string()
        } else {
            format!("Known options: `{}`.", hover.known_options.join("`, `"))
        };
    let patterns = if hover.known_patterns.is_empty() {
        String::new()
    } else {
        format!("\n\nPatterns: `{}`.", hover.known_patterns.join("`, `"))
    };
    let unresolved = if hover.unresolved_reasons.is_empty() {
        String::new()
    } else {
        format!(
            "\n\nUnresolved config: `{}`. Unknown classes remain indeterminate.",
            hover.unresolved_reasons.join("`, `")
        )
    };
    let graph = if hover.provider_id != UTILITY_PROVIDER_ID {
        String::new()
    } else if hover.graph_bindings.is_empty() {
        "\n\nNo matching selector definition is indexed in the CSS graph.".to_string()
    } else {
        let locations = hover
            .graph_bindings
            .iter()
            .map(|binding| {
                format!(
                    "`{}` at {}:{}",
                    binding.uri,
                    binding.range.start.line + 1,
                    binding.range.start.character + 1
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("\n\nCSS graph bindings: {locations}.")
    };
    format!(
        "**`{}.{}.{}`**\n\n{} from `{}`.\n\n{}{}{}{}",
        hover.owner_name,
        hover.axis_name,
        current,
        validity,
        hover.domain,
        known_options,
        patterns,
        unresolved,
        graph,
    )
}

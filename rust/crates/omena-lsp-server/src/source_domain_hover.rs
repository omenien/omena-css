use omena_query::{
    OmenaQuerySourceDomainClassReferenceFactV0 as SourceDomainClassReferenceFact, ParserPositionV0,
    ParserRangeV0,
};
use serde_json::{Value, json};

use crate::{
    LspTextDocumentState,
    protocol::{byte_offset_for_parser_position, parser_range_for_byte_span},
    source_completion::source_domain_reference_option_names,
};

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
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<Value> {
    let reference = source_domain_reference_at_position(document, position)?;
    let options = source_domain_reference_option_names(&document.source_syntax_index, reference);
    let current = source_domain_reference_current_option(reference);
    let validity = source_domain_reference_validity(reference, options.as_slice());
    let range = parser_range_for_byte_span(document.text.as_str(), reference.byte_span);

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
        "sourceOwner": reference.owner_name,
        "domain": reference.domain,
        "axisName": reference.axis_name,
        "optionName": reference.option_name,
        "prefix": reference.prefix,
        "currentOption": current,
        "validity": validity,
        "knownOptions": options,
        "candidateCount": 1,
        "definitionCount": 0,
        "candidates": [],
        "definitions": [],
        "renderedMarkdown": render_source_domain_reference_hover_text(reference, options.as_slice()),
        "resolutionPath": ["sourceSyntaxIndex", "classValueUniverseProvider", "sourceDomainReferenceHover"],
        "readySurfaces": ["explainHoverTraceRpc", "sourceSyntaxIndex", "classValueUniverseProvider"],
    }))
}

pub(crate) fn source_domain_reference_hover_at_position(
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<(ParserRangeV0, String)> {
    let reference = source_domain_reference_at_position(document, position)?;
    let options = source_domain_reference_option_names(&document.source_syntax_index, reference);
    Some((
        parser_range_for_byte_span(document.text.as_str(), reference.byte_span),
        render_source_domain_reference_hover_text(reference, options.as_slice()),
    ))
}

fn source_domain_reference_current_option(reference: &SourceDomainClassReferenceFact) -> &str {
    reference
        .option_name
        .as_deref()
        .or(reference.prefix.as_deref())
        .unwrap_or("*")
}

fn source_domain_reference_validity(
    reference: &SourceDomainClassReferenceFact,
    options: &[String],
) -> &'static str {
    reference
        .option_name
        .as_ref()
        .map(|option| {
            if options.iter().any(|known| known == option) {
                "known option"
            } else {
                "unknown option"
            }
        })
        .unwrap_or("prefix option")
}

fn render_source_domain_reference_hover_text(
    reference: &SourceDomainClassReferenceFact,
    options: &[String],
) -> String {
    let current = reference
        .option_name
        .as_deref()
        .or(reference.prefix.as_deref())
        .unwrap_or("*");
    let validity = source_domain_reference_validity(reference, options);
    let known_options = if options.is_empty() {
        "No known options indexed.".to_string()
    } else {
        format!("Known options: `{}`.", options.join("`, `"))
    };
    format!(
        "**`{}.{}.{}`**\n\n{} from `{}`.\n\n{}",
        reference.owner_name,
        reference.axis_name,
        current,
        validity,
        reference.domain,
        known_options
    )
}

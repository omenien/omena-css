use crate::{
    LspShellState,
    open_document_inputs::style_sources_from_open_documents,
    protocol::{
        byte_offset_for_parser_position, document_uri_from_params, file_label_from_uri,
        is_style_document_uri, lsp_range_from_value,
    },
    resolution_inputs_for_workspace_uri,
};
use omena_query::{
    OmenaQueryCodeActionV0, ParserPositionV0,
    summarize_omena_query_style_refactor_code_actions_with_resolution_inputs,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) fn resolve_lsp_code_actions(state: &LspShellState, params: Option<&Value>) -> Value {
    let diagnostics = params
        .and_then(|value| value.get("context"))
        .and_then(|value| value.get("diagnostics"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    let mut actions: Vec<Value> = diagnostics
        .iter()
        .enumerate()
        .filter_map(|(index, diagnostic)| {
            let payload = diagnostic
                .pointer("/data/createCustomProperty")
                .and_then(Value::as_object)?;
            let uri = payload.get("uri").and_then(Value::as_str)?;
            let range = payload.get("range")?;
            let new_text = payload.get("newText").and_then(Value::as_str)?;
            let property_name = payload.get("propertyName").and_then(Value::as_str)?;
            let mut changes = serde_json::Map::new();
            changes.insert(
                uri.to_string(),
                json!([
                    {
                        "range": range,
                        "newText": new_text,
                    },
                ]),
            );

            Some(json!({
                "title": format!("Add '{}' to {}", property_name, file_label_from_uri(uri)),
                "kind": "quickfix",
                "diagnostics": [diagnostic],
                "edit": {
                    "changes": Value::Object(changes),
                },
                "data": {
                    "source": "omenaQueryStyleDiagnosticsForFile",
                    "diagnosticIndex": index,
                },
            }))
        })
        .chain(diagnostics.iter().enumerate().filter_map(|(index, diagnostic)| {
            let payload = diagnostic
                .pointer("/data/createSelector")
                .and_then(Value::as_object)?;
            let uri = payload.get("uri").and_then(Value::as_str)?;
            let range = payload.get("range")?;
            let new_text = payload.get("newText").and_then(Value::as_str)?;
            let selector_name = payload.get("selectorName").and_then(Value::as_str)?;
            let mut changes = serde_json::Map::new();
            changes.insert(
                uri.to_string(),
                json!([
                    {
                        "range": range,
                        "newText": new_text,
                    },
                ]),
            );

            Some(json!({
                "title": format!("Add '.{}' to {}", selector_name, file_label_from_uri(uri)),
                "kind": "quickfix",
                "diagnostics": [diagnostic],
                "edit": {
                    "changes": Value::Object(changes),
                },
                "data": {
                    "source": "omenaQuerySourceSyntaxIndex",
                    "diagnosticIndex": index,
                },
            }))
        }))
        .collect();

    actions.extend(resolve_lsp_suppression_code_actions(
        state,
        params,
        diagnostics,
    ));

    if diagnostics.is_empty() {
        actions.extend(resolve_lsp_refactor_code_actions(state, params));
    }

    if actions.is_empty() {
        Value::Null
    } else {
        json!(actions)
    }
}

fn resolve_lsp_suppression_code_actions(
    state: &LspShellState,
    params: Option<&Value>,
    diagnostics: &[Value],
) -> Vec<Value> {
    let document_uri = document_uri_from_params(params);
    let Some(document) = state.document(document_uri.as_str()) else {
        return Vec::new();
    };
    if !is_style_document_uri(document.uri.as_str()) {
        return Vec::new();
    }

    let mut actions = Vec::new();
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        let Some(code) = diagnostic.get("code").and_then(Value::as_str) else {
            continue;
        };
        let Some(line) = diagnostic
            .pointer("/range/start/line")
            .and_then(Value::as_u64)
            .and_then(|line| usize::try_from(line).ok())
        else {
            continue;
        };
        let character = diagnostic
            .pointer("/range/start/character")
            .and_then(Value::as_u64)
            .and_then(|character| usize::try_from(character).ok())
            .unwrap_or(0);

        let indent = source_line_indent(document.text.as_str(), line);
        let insert_range = json!({
            "start": {
                "line": line,
                "character": 0,
            },
            "end": {
                "line": line,
                "character": 0,
            },
        });
        let mut changes = serde_json::Map::new();
        changes.insert(
            document.uri.clone(),
            json!([
                {
                    "range": insert_range,
                    "newText": format!(
                        "{indent}/* omena-ignore-next-line {code} [reason: 'TODO'] */\n"
                    ),
                },
            ]),
        );

        actions.push(json!({
            "title": "Suppress this diagnostic on the next line",
            "kind": "quickfix",
            "diagnostics": [diagnostic],
            "edit": {
                "changes": Value::Object(changes),
            },
            "data": {
                "source": "omenaLspDiagnosticSuppressionCodeAction",
                "diagnosticIndex": index,
                "code": code,
            },
        }));

        if let Some(block_line) =
            enclosing_style_block_open_line(document.text.as_str(), line, character)
        {
            let block_indent = source_line_indent(document.text.as_str(), block_line);
            let block_insert_range = json!({
                "start": {
                    "line": block_line,
                    "character": 0,
                },
                "end": {
                    "line": block_line,
                    "character": 0,
                },
            });
            let mut block_changes = serde_json::Map::new();
            block_changes.insert(
                document.uri.clone(),
                json!([
                    {
                        "range": block_insert_range,
                        "newText": format!(
                            "{block_indent}/* omena-ignore {code} [reason: 'TODO'] */\n"
                        ),
                    },
                ]),
            );

            actions.push(json!({
                "title": "Suppress diagnostics in this block",
                "kind": "quickfix",
                "diagnostics": [diagnostic],
                "edit": {
                    "changes": Value::Object(block_changes),
                },
                "data": {
                    "source": "omenaLspDiagnosticSuppressionCodeAction",
                    "diagnosticIndex": index,
                    "code": code,
                    "scope": "block",
                },
            }));
        }
    }
    actions
}

fn source_line_indent(source: &str, line: usize) -> String {
    source
        .lines()
        .nth(line)
        .map(|text| {
            text.chars()
                .take_while(|character| character.is_whitespace())
                .collect()
        })
        .unwrap_or_default()
}

fn enclosing_style_block_open_line(source: &str, line: usize, character: usize) -> Option<usize> {
    let offset = byte_offset_for_parser_position(source, ParserPositionV0 { line, character })?;
    let prefix = source.get(..offset)?;
    let mut block_stack = Vec::new();
    let mut current_line = 0usize;
    let mut quote: Option<char> = None;
    let mut in_block_comment = false;
    let mut characters = prefix.chars().peekable();

    while let Some(character) = characters.next() {
        if character == '\n' {
            current_line += 1;
            continue;
        }
        if in_block_comment {
            if character == '*' && characters.peek() == Some(&'/') {
                characters.next();
                in_block_comment = false;
            }
            continue;
        }
        if let Some(quote_character) = quote {
            if character == '\\' {
                if characters.peek().is_some() {
                    characters.next();
                }
            } else if character == quote_character {
                quote = None;
            }
            continue;
        }
        if character == '/' && characters.peek() == Some(&'*') {
            characters.next();
            in_block_comment = true;
            continue;
        }
        match character {
            '"' | '\'' => quote = Some(character),
            '{' => block_stack.push(current_line),
            '}' => {
                block_stack.pop();
            }
            _ => {}
        }
    }

    block_stack.last().copied()
}

fn resolve_lsp_refactor_code_actions(state: &LspShellState, params: Option<&Value>) -> Vec<Value> {
    let document_uri = document_uri_from_params(params);
    let Some(document) = state.document(document_uri.as_str()) else {
        return Vec::new();
    };
    if !is_style_document_uri(document.uri.as_str()) {
        return Vec::new();
    }
    let Some(range) = params
        .and_then(|value| value.get("range"))
        .and_then(lsp_range_from_value)
    else {
        return Vec::new();
    };

    let style_sources = style_sources_from_open_documents(
        state,
        document.workspace_folder_uri.as_deref(),
        Some(document.uri.as_str()),
    );
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref());
    let actions = summarize_omena_query_style_refactor_code_actions_with_resolution_inputs(
        document.uri.as_str(),
        style_sources.as_slice(),
        document.text.as_str(),
        range,
        &[],
        &resolution_inputs,
    )
    .actions;
    render_omena_query_lsp_code_actions(actions)
}

fn render_omena_query_lsp_code_actions(actions: Vec<OmenaQueryCodeActionV0>) -> Vec<Value> {
    actions
        .into_iter()
        .enumerate()
        .map(|(index, action)| {
            let mut changes_by_uri = BTreeMap::<String, Vec<Value>>::new();
            for edit in action.edits {
                changes_by_uri.entry(edit.uri).or_default().push(json!({
                    "range": edit.range,
                    "newText": edit.new_text,
                }));
            }

            let changes = changes_by_uri
                .into_iter()
                .map(|(uri, edits)| (uri, Value::Array(edits)))
                .collect::<serde_json::Map<_, _>>();

            json!({
                "title": action.title,
                "kind": action.kind,
                "edit": {
                    "changes": Value::Object(changes),
                },
                "data": {
                    "source": action.source,
                    "actionIndex": index,
                },
            })
        })
        .collect()
}

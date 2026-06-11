use super::*;
use std::sync::Arc;

fn open_narrowing_workspace(state: &mut LspShellState) {
    handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": "file:///workspace-a",
                        "name": "workspace-a",
                    },
                ],
            },
        }),
    );
    for (uri, text) in [
        (
            "file:///workspace-a/src/App.module.scss",
            "@use \"./theme\";\n.btn { color: red; }\n.btn { color: green; }",
        ),
        (
            "file:///workspace-a/src/_theme.scss",
            ".btn { color: blue; }\n.unrelated { color: black; }",
        ),
    ] {
        handle_lsp_message(
            state,
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": uri,
                        "languageId": "scss",
                        "version": 1,
                        "text": text,
                    },
                },
            }),
        );
    }
}

fn hover_btn_markdown(state: &mut LspShellState, id: u64) -> Result<String, std::io::Error> {
    let hover_response = handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
                "position": {
                    "line": 1,
                    "character": 2,
                },
            },
        }),
    );
    hover_response
        .as_ref()
        .and_then(|value| value.pointer("/result/contents/value"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| std::io::Error::other("style hover should render markdown value"))
}

#[test]
fn cascade_narrowing_substrate_memo_reuses_while_corpus_unchanged() -> TestResult {
    let mut state = LspShellState::default();
    open_narrowing_workspace(&mut state);

    let style_sources = style_sources_from_open_documents(
        &state,
        Some("file:///workspace-a"),
        Some("file:///workspace-a/src/App.module.scss"),
    );
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(&state, Some("file:///workspace-a"));
    let first = cascade_narrowing_substrate_for_style_sources(
        &state,
        style_sources.as_slice(),
        &resolution_inputs,
    );
    let second = cascade_narrowing_substrate_for_style_sources(
        &state,
        style_sources.as_slice(),
        &resolution_inputs,
    );
    assert!(
        Arc::ptr_eq(&first, &second),
        "unchanged narrowing inputs must reuse the memoized substrate"
    );

    // The hover provider runs through the same memo, so the request after the direct
    // calls must still see the populated entry.
    let hover_text = hover_btn_markdown(&mut state, 2)?;
    assert!(
        hover_text.contains("Cascade narrowed values:") && hover_text.contains("`blue`"),
        "{hover_text}"
    );
    assert!(state.cascade_narrowing_substrate_memo.borrow().is_some());
    Ok(())
}

#[test]
fn cascade_narrowing_substrate_memo_rebuilds_after_document_change() -> TestResult {
    let mut state = LspShellState::default();
    open_narrowing_workspace(&mut state);

    let before_text = hover_btn_markdown(&mut state, 2)?;
    assert!(
        before_text.contains("`blue`"),
        "imported module value should participate before the edit: {before_text}"
    );
    let before_substrate = state
        .cascade_narrowing_substrate_memo
        .borrow()
        .as_ref()
        .map(|memo| Arc::clone(&memo.substrate))
        .ok_or_else(|| std::io::Error::other("hover must populate the narrowing substrate memo"))?;

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/_theme.scss",
                    "version": 2,
                },
                "contentChanges": [
                    {
                        "text": ".btn { color: purple; }",
                    },
                ],
            },
        }),
    );

    let after_text = hover_btn_markdown(&mut state, 3)?;
    assert!(
        after_text.contains("`purple`") && !after_text.contains("`blue`"),
        "memoized narrowing must never serve the pre-edit corpus: {after_text}"
    );
    let after_substrate = state
        .cascade_narrowing_substrate_memo
        .borrow()
        .as_ref()
        .map(|memo| Arc::clone(&memo.substrate))
        .ok_or_else(|| {
            std::io::Error::other("hover must repopulate the narrowing substrate memo")
        })?;
    assert!(
        !Arc::ptr_eq(&before_substrate, &after_substrate),
        "changed corpus must rebuild the substrate"
    );
    Ok(())
}

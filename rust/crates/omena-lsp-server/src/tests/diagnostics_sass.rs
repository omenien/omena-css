use super::*;

#[test]
fn resolves_graph_aware_sass_diagnostics_from_opened_style_documents() {
    let mut state = LspShellState::default();
    for (uri, text) in [
        (
            "file:///workspace-a/src/App.module.scss",
            "@use \"./tokens\" as tokens;\n.button { color: tokens.$brand; padding: $missing; }",
        ),
        ("file:///workspace-a/src/_tokens.scss", "$brand: red;"),
    ] {
        handle_lsp_message(
            &mut state,
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

    let diagnostics_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
            },
        }),
    );
    let diagnostics = diagnostics_response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .expect("style diagnostics response contains an array");
    let missing_sass_messages = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.pointer("/code") == Some(&json!("missingSassSymbol")))
        .map(|diagnostic| {
            diagnostic
                .pointer("/message")
                .and_then(Value::as_str)
                .expect("missing Sass symbol diagnostic has a message")
        })
        .collect::<Vec<_>>();

    assert_eq!(
        missing_sass_messages,
        vec!["Sass variable '$missing' not found in the visible Sass module graph."]
    );
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.pointer("/code") == Some(&json!("missingSassSymbol"))
                && diagnostic.pointer("/data/provenance/1")
                    == Some(&json!("omena-query.graph-aware-sass-diagnostics"))
        }),
        "Rust LSP style diagnostics should consume graph-aware omena-query Sass diagnostics"
    );
}

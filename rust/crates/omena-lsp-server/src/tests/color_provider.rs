//! documentColor resolves VARIABLE REFERENCES to swatches through the same
//! machinery the hover uses — declarations and literals stay with the
//! built-in service — plus the post-settle hover-substrate warmup dispatch.

use super::*;

#[test]
fn document_color_resolves_variable_references_to_swatches() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-document-color-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let src_dir = workspace_path.join("src");
    fs::create_dir_all(src_dir.as_path())?;
    fs::write(
        src_dir.join("_tokens.scss").as_path(),
        "$green500: #09ab49;\n$spacing: 12px;\n",
    )?;

    let style_path = src_dir.join("App.module.scss");
    let style_text = "@use \"./tokens\" as *;\n.badge { color: $green500; padding: $spacing; }\n";
    let style_uri = path_to_file_uri(style_path.as_path());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": path_to_file_uri(workspace_path.as_path()),
                        "name": "workspace",
                    },
                ],
            },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": style_text,
                },
            },
        }),
    );

    // In production the background workspace index admits every style file
    // into the corpus; the fixture mirrors that admission by opening the
    // tokens document (an unadmitted foreign declaration yields no swatch,
    // matching the candidate machinery's visibility).
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": path_to_file_uri(src_dir.join("_tokens.scss").as_path()),
                    "languageId": "scss",
                    "version": 1,
                    "text": "$green500: #09ab49;\n$spacing: 12px;\n",
                },
            },
        }),
    );

    let response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/documentColor",
            "params": {
                "textDocument": { "uri": style_uri },
            },
        }),
    );
    let informations = response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .cloned()
        .ok_or("documentColor must answer with an array")?;
    assert_eq!(
        informations.len(),
        1,
        "exactly the color-valued reference gets a swatch (not $spacing): {informations:?}"
    );
    let information = &informations[0];
    let channel = |name: &str| -> f64 {
        information
            .pointer(&format!("/color/{name}"))
            .and_then(Value::as_f64)
            .unwrap_or(-1.0)
    };
    assert!((channel("red") - 9.0 / 255.0).abs() < 1e-9);
    assert!((channel("green") - 171.0 / 255.0).abs() < 1e-9);
    assert!((channel("blue") - 73.0 / 255.0).abs() < 1e-9);
    assert!((channel("alpha") - 1.0).abs() < 1e-9);
    let reference_offset = fixture_find(
        style_text,
        "$green500;",
        "style fixture references the token",
    )?;
    let expected_start = parser_position_for_byte_offset(style_text, reference_offset);
    assert_eq!(
        information
            .pointer("/range/start/line")
            .and_then(Value::as_u64),
        Some(expected_start.line as u64),
        "the swatch sits on the reference site"
    );

    let presentation = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/colorPresentation",
            "params": {
                "textDocument": { "uri": style_uri },
                "color": information.get("color"),
                "range": information.get("range"),
            },
        }),
    );
    assert_eq!(
        presentation
            .as_ref()
            .and_then(|value| value.pointer("/result/0/label"))
            .and_then(Value::as_str),
        Some("#09ab49"),
    );
    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

#[test]
fn hover_substrate_warmup_dispatch_resolves_silently() -> TestResult {
    let mut state = LspShellState::default();
    assert!(
        crate::hover_substrate_warmup_dispatch(&state).is_none(),
        "no open style document, nothing to warm"
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace/src/Warm.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": "$brand: #fff;\n.warm { color: $brand; }\n",
                },
            },
        }),
    );
    let dispatch = crate::hover_substrate_warmup_dispatch(&state)
        .ok_or("an open style document with candidates must yield a warmup dispatch")?;
    assert_eq!(
        dispatch.message.get("method").and_then(Value::as_str),
        Some(crate::HOVER_SUBSTRATE_WARMUP_METHOD),
    );
    assert!(
        crate::resolve_dispatched_query_response(&dispatch).is_none(),
        "the warmup dispatch must never produce a client-visible response"
    );
    Ok(())
}

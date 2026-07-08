//! documentLink (style dependency specifiers → resolved files) and
//! workspace/symbol (name search over precomputed style declarations).

use super::*;

fn open(state: &mut LspShellState, uri: &str, language: &str, text: &str) {
    handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": language,
                    "version": 1,
                    "text": text,
                },
            },
        }),
    );
}

#[test]
fn document_links_resolve_style_dependency_specifiers() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-document-links-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let src_dir = workspace_path.join("src");
    fs::create_dir_all(src_dir.as_path())?;
    fs::write(src_dir.join("_tokens.scss").as_path(), "$brand: #fff;\n")?;
    let style_path = src_dir.join("App.module.scss");
    let style_text = "@use \"./tokens\" as *;\n.root { color: $brand; }\n";
    fs::write(style_path.as_path(), style_text)?;
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
                    {"uri": path_to_file_uri(workspace_path.as_path()), "name": "links"},
                ],
            },
        }),
    );
    open(&mut state, style_uri.as_str(), "scss", style_text);

    let response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/documentLink",
            "params": { "textDocument": { "uri": style_uri } },
        }),
    );
    let links = response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .cloned()
        .ok_or("documentLink must answer with an array")?;
    assert_eq!(links.len(), 1, "one @use specifier, one link: {links:?}");
    let target = links[0]
        .pointer("/target")
        .and_then(Value::as_str)
        .ok_or("link carries a target")?;
    assert!(
        target.ends_with("_tokens.scss"),
        "the link targets the RESOLVED partial, not the literal specifier: {target}"
    );
    let start_character = links[0]
        .pointer("/range/start/character")
        .and_then(Value::as_u64)
        .ok_or("link carries a range")?;
    assert_eq!(
        start_character, 6,
        "the range sits INSIDE the quotes of `@use \"./tokens\"`"
    );
    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

#[test]
fn workspace_symbols_search_style_declarations() -> TestResult {
    let mut state = LspShellState::default();
    open(
        &mut state,
        "file:///ws-symbols/src/App.module.scss",
        "scss",
        ".navBadge { color: red; }\n.plain { color: blue; }\n",
    );
    open(
        &mut state,
        "file:///ws-symbols/src/_tokens.scss",
        "scss",
        "$navSpacing: 12px;\n",
    );

    // Foreign (node_modules) stylesheets are admitted for resolution but
    // must never surface their internals in the symbol search.
    open(
        &mut state,
        "file:///ws-symbols/node_modules/pkg/dist/_lib.scss",
        "scss",
        ".navLibraryInternal { color: green; }\n",
    );

    let response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "workspace/symbol",
            "params": { "query": "nav" },
        }),
    );
    let symbols = response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .cloned()
        .ok_or("workspace/symbol must answer with an array")?;
    let names = symbols
        .iter()
        .filter_map(|symbol| symbol.pointer("/name").and_then(Value::as_str))
        .collect::<Vec<_>>();
    assert!(
        names.contains(&".navBadge"),
        "class declarations are searchable: {names:?}"
    );
    assert!(
        names.contains(&"$navSpacing"),
        "sass variable declarations are searchable: {names:?}"
    );
    assert!(
        !names.contains(&".plain"),
        "non-matching names stay out: {names:?}"
    );
    assert!(
        !names.contains(&".navLibraryInternal"),
        "foreign-origin declarations stay out: {names:?}"
    );
    Ok(())
}

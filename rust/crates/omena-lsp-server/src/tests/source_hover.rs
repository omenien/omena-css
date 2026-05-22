use super::*;

#[test]
fn source_hover_renders_unopened_target_style_rule_from_disk() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-disk-style-hover-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let src_dir = workspace_path.join("src");
    let source_path = src_dir.join("App.tsx");
    let style_path = src_dir.join("App.module.scss");
    fs::create_dir_all(src_dir.as_path())?;
    fs::write(style_path.as_path(), ".foo { color: red; }\n")?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let source_uri = path_to_file_uri(source_path.as_path());
    let source_text = "import bind from \"classnames/bind\";\nimport styles from \"./App.module.scss\";\nconst cx = bind.bind(styles);\nexport const view = <div className={cx(\"foo\")} />;";
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
                        "uri": workspace_uri,
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
                    "uri": source_uri,
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": source_text,
                },
            },
        }),
    );

    let hover_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": path_to_file_uri(source_path.as_path()),
                },
                "position": parser_position_for_byte_offset(
                    source_text,
                    fixture_find(source_text, "\"foo\"", "source fixture contains foo")? + 1,
                ),
            },
        }),
    );
    let hover_text = hover_response
        .as_ref()
        .and_then(|value| value.pointer("/result/contents/value"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    assert!(
        hover_text.contains("color: red"),
        "hover text: {hover_text}"
    );

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

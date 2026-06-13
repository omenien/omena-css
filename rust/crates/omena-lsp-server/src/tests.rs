use super::*;

#[path = "tests/code_actions.rs"]
mod code_actions;
#[path = "tests/diagnostics_cascade.rs"]
mod diagnostics_cascade;
#[path = "tests/diagnostics_publish.rs"]
mod diagnostics_publish;
#[path = "tests/diagnostics_sass.rs"]
mod diagnostics_sass;
#[path = "tests/diagnostics_style.rs"]
mod diagnostics_style;
#[path = "tests/disk_cache.rs"]
mod disk_cache;
#[path = "tests/explain_hover_trace.rs"]
mod explain_hover_trace;
#[path = "tests/hover_sass.rs"]
mod hover_sass;
#[path = "tests/hover_style.rs"]
mod hover_style;
#[path = "tests/lifecycle.rs"]
mod lifecycle;
#[path = "tests/lifecycle_cancellation.rs"]
mod lifecycle_cancellation;
#[path = "tests/lifecycle_capabilities.rs"]
mod lifecycle_capabilities;
#[path = "tests/lifecycle_configuration.rs"]
mod lifecycle_configuration;
#[path = "tests/lifecycle_text_sync.rs"]
mod lifecycle_text_sync;
#[path = "tests/narrowing_substrate.rs"]
mod narrowing_substrate;
#[path = "tests/query_dispatch.rs"]
mod query_dispatch;
#[path = "tests/sass_resolution_package.rs"]
mod sass_resolution_package;
#[path = "tests/sass_resolution_symlink.rs"]
mod sass_resolution_symlink;
#[path = "tests/sass_resolution_watch.rs"]
mod sass_resolution_watch;
#[path = "tests/sass_symbols_forwarding.rs"]
mod sass_symbols_forwarding;
#[path = "tests/sass_symbols_imports.rs"]
mod sass_symbols_imports;
#[path = "tests/sass_symbols_maps.rs"]
mod sass_symbols_maps;
#[path = "tests/source_completion.rs"]
mod source_completion;
#[path = "tests/source_dynamic.rs"]
mod source_dynamic;
#[path = "tests/source_hover.rs"]
mod source_hover;
#[path = "tests/source_imports.rs"]
mod source_imports;
#[path = "tests/source_language_astro.rs"]
mod source_language_astro;
#[path = "tests/source_language_html.rs"]
mod source_language_html;
#[path = "tests/source_language_markdown.rs"]
mod source_language_markdown;
#[path = "tests/source_language_template.rs"]
mod source_language_template;
#[path = "tests/source_resolution.rs"]
mod source_resolution;
#[path = "tests/source_resolution_bundler.rs"]
mod source_resolution_bundler;
#[path = "tests/source_resolution_tsconfig.rs"]
mod source_resolution_tsconfig;
#[path = "tests/source_resolution_watch.rs"]
mod source_resolution_watch;
#[path = "tests/source_text_offsets.rs"]
mod source_text_offsets;
#[path = "tests/source_type_facts.rs"]
mod source_type_facts;
#[path = "tests/style_completion.rs"]
mod style_completion;
#[path = "tests/style_context.rs"]
mod style_context;
#[path = "tests/style_indexing.rs"]
mod style_indexing;
#[path = "tests/svelte_component.rs"]
mod svelte_component;
#[path = "tests/vue_sfc.rs"]
mod vue_sfc;
#[path = "tests/workspace_folders.rs"]
mod workspace_folders;
#[path = "tests/workspace_indexing.rs"]
mod workspace_indexing;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn fixture_parent<'a>(
    path: &'a Path,
    context: &'static str,
) -> Result<&'a Path, Box<dyn std::error::Error>> {
    path.parent()
        .ok_or_else(|| std::io::Error::other(context).into())
}

fn fixture_find(
    source: &str,
    needle: &str,
    context: &'static str,
) -> Result<usize, Box<dyn std::error::Error>> {
    source
        .find(needle)
        .ok_or_else(|| std::io::Error::other(context).into())
}

fn assert_source_binding_target(state: &LspShellState, source_uri: &str, expected_style_uri: &str) {
    let imported_style_bindings = state
        .document(source_uri)
        .map(|document| document.source_syntax_index.imported_style_bindings.clone());
    assert_eq!(
        imported_style_bindings.as_deref(),
        Some(
            [ImportedStyleBinding {
                binding: "styles".to_string(),
                style_uri: expected_style_uri.to_string(),
            }]
            .as_slice()
        ),
    );
}

fn assert_definition_response_single_target(response: &Option<Value>, expected_uri: &str) {
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/uri")),
        Some(&json!(expected_uri)),
    );
    assert_eq!(
        response
            .as_ref()
            .and_then(|value| value.pointer("/result/1/uri")),
        None,
    );
}

#[test]
fn workspace_folder_compatibility_normalizes_percent_encoded_route_groups() {
    assert!(workspace_folder_uri_equivalent(
        "file:///workspace/app/(marketing)",
        "file:///workspace/app/%28marketing%29",
    ));
}

#[test]
fn path_to_file_uri_percent_encodes_route_group_paths() {
    let uri = path_to_file_uri(Path::new("/workspace/app/(marketing)/Card.module.scss"));

    assert_eq!(
        uri,
        "file:///workspace/app/%28marketing%29/Card.module.scss"
    );
    assert!(file_uri_equivalent(
        uri.as_str(),
        "file:///workspace/app/(marketing)/Card.module.scss",
    ));
}

#[cfg(unix)]
#[test]
fn document_map_uses_canonical_identity_for_symlinked_document_paths() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_symlinked_document_identity_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let real_src = root.join("real/src");
    let link_src = root.join("linked-src");
    let real_style = real_src.join("Button.module.scss");
    let linked_style = link_src.join("Button.module.scss");
    fs::create_dir_all(real_src.as_path())?;
    fs::write(real_style.as_path(), ".button { color: red; }")?;
    std::os::unix::fs::symlink(real_src.as_path(), link_src.as_path())?;

    let real_uri = raw_test_file_uri(real_style.as_path());
    let linked_uri = raw_test_file_uri(linked_style.as_path());
    assert!(file_uri_equivalent(real_uri.as_str(), linked_uri.as_str()));

    let mut state = LspShellState::default();
    for (uri, text) in [
        (linked_uri.as_str(), ".button { color: red; }"),
        (real_uri.as_str(), ".button { color: blue; }"),
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

    assert_eq!(state.document_count(), 1);
    assert_eq!(state.open_document_uris.len(), 1);
    assert!(state.document(real_uri.as_str()).is_some());
    assert!(state.document(linked_uri.as_str()).is_some());
    assert_eq!(state.snapshot().documents.len(), 1);
    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}

fn raw_test_file_uri(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy())
}

#[test]
fn codelens_keeps_references_when_workspace_owner_uri_encoding_differs() {
    let workspace_uri = "file:///workspace/(group-a)";
    let encoded_workspace_uri = "file:///workspace/%28group-a%29";
    let source_uri = "file:///workspace/%28group-a%29/src/App.tsx";
    let style_uri = "file:///workspace/(group-a)/src/App.module.scss";
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
                        "name": "group-a",
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
                    "text": "import bind from \"classnames/bind\";\nimport styles from \"./App.module.scss\";\nconst cx = bind.bind(styles);\nexport const view = <div className={cx(\"foo\")} />;",
                },
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
                    "text": ".foo { color: red; }",
                },
            },
        }),
    );
    if let Some(document) = state.documents.get_mut(source_uri) {
        std::sync::Arc::make_mut(document).workspace_folder_uri =
            Some(encoded_workspace_uri.to_string());
    }

    let code_lens_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/codeLens",
            "params": {
                "textDocument": {
                    "uri": style_uri,
                },
            },
        }),
    );
    assert_eq!(
        code_lens_response
            .as_ref()
            .and_then(|value| value.pointer("/result/0/command/title")),
        Some(&json!("1 reference")),
    );
}

use super::*;
use std::path::Path;

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
#[path = "tests/resolver_identity_index.rs"]
mod resolver_identity_index;
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
#[path = "tests/tide_kernel.rs"]
mod tide_kernel;
#[cfg(all(
    feature = "salsa-style-diagnostics",
    feature = "parallel-style-diagnostics"
))]
#[path = "tests/tide_republish_executor.rs"]
mod tide_republish_executor;
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

#[test]
fn lsp_file_identity_feeds_incremental_revision_salsa_key() {
    let uri = "file:///workspace/src/Card.module.scss";
    let text = ".card { color: red; }";
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "scss",
                    "version": 7,
                    "text": text,
                },
            },
        }),
    );
    assert!(
        state.document_file_id(uri).is_some(),
        "opened document should have an LSP file identity"
    );
    let Some(file_id) = state.document_file_id(uri) else {
        return;
    };
    let syntax_id = omena_query::syntax_node_id_for_omena_query_style_source(
        text,
        omena_query::OmenaParserStyleDialect::Scss,
    );
    let db = salsa::DatabaseImpl::default();
    let key_input = omena_incremental::SalsaIncrementalFileRevisionInputV0::new(
        &db,
        file_id.incremental_key(),
        omena_incremental::IncrementalRevisionV0 { value: 7 },
        syntax_id.clone(),
    );

    assert_eq!(
        omena_incremental::read_salsa_file_revision_syntax_key(&db, key_input),
        format!(
            "file={};revision=7;syntax={}",
            file_id.incremental_key(),
            syntax_id
        )
    );
}

#[cfg(unix)]
#[test]
fn admitted_document_identity_comparisons_do_not_recanonicalize_paths() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_file_identity_cache_{}_{}",
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

    crate::protocol::reset_file_uri_identity_cache_for_test();
    let mut state = LspShellState::default();
    for uri in [linked_uri.as_str(), real_uri.as_str()] {
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
                        "text": ".button { color: red; }",
                    },
                },
            }),
        );
    }

    crate::protocol::reset_file_uri_identity_canonicalize_syscall_count_for_test();
    assert!(state.document(real_uri.as_str()).is_some());
    assert!(state.document(linked_uri.as_str()).is_some());
    assert!(state.has_open_document_uri(real_uri.as_str()));
    assert!(state.has_open_document_uri(linked_uri.as_str()));
    assert!(file_uri_equivalent(real_uri.as_str(), linked_uri.as_str()));
    assert_eq!(
        crate::protocol::file_uri_identity_canonicalize_syscall_count_for_test(),
        0,
        "admitted equivalent path comparisons must hit the identity cache"
    );

    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}

#[test]
fn document_map_uses_canonical_identity_for_case_varied_paths_when_supported() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_case_document_identity_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    fs::create_dir_all(root.as_path())?;
    let real_style = root.join("CaseStyle.module.scss");
    let case_style = root.join("casestyle.module.scss");
    fs::write(real_style.as_path(), ".case { color: red; }")?;

    let real_uri = raw_test_file_uri(real_style.as_path());
    let case_uri = raw_test_file_uri(case_style.as_path());
    if !file_uri_equivalent(real_uri.as_str(), case_uri.as_str()) {
        let _ = fs::remove_dir_all(root.as_path());
        return Ok(());
    }

    let mut state = LspShellState::default();
    for (uri, text) in [
        (real_uri.as_str(), ".case { color: red; }"),
        (case_uri.as_str(), ".case { color: blue; }"),
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
    assert!(state.document(case_uri.as_str()).is_some());
    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}

#[cfg(unix)]
#[test]
fn document_map_uses_canonical_identity_for_case_varied_symlink_paths_when_supported() -> TestResult
{
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_case_symlink_document_identity_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    let real_src = root.join("RealSrc");
    let link_src = root.join("linked-src");
    let real_style = real_src.join("CaseStyle.module.scss");
    let linked_case_style = link_src.join("casestyle.module.scss");
    fs::create_dir_all(real_src.as_path())?;
    fs::write(real_style.as_path(), ".case { color: red; }")?;
    std::os::unix::fs::symlink(real_src.as_path(), link_src.as_path())?;

    let real_uri = raw_test_file_uri(real_style.as_path());
    let linked_case_uri = raw_test_file_uri(linked_case_style.as_path());
    if !file_uri_equivalent(real_uri.as_str(), linked_case_uri.as_str()) {
        let _ = fs::remove_dir_all(root.as_path());
        return Ok(());
    }
    assert_eq!(
        LspShellState::document_storage_uri(real_uri.as_str()),
        LspShellState::document_storage_uri(linked_case_uri.as_str())
    );

    let mut state = LspShellState::default();
    for (uri, text) in [
        (real_uri.as_str(), ".case { color: red; }"),
        (linked_case_uri.as_str(), ".case { color: blue; }"),
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
    assert_eq!(
        state.document_file_id(real_uri.as_str()),
        state.document_file_id(linked_case_uri.as_str())
    );
    assert!(state.document(real_uri.as_str()).is_some());
    assert!(state.document(linked_case_uri.as_str()).is_some());
    let _ = fs::remove_dir_all(root.as_path());
    Ok(())
}

#[test]
fn workspace_wave_document_admission_keeps_canonicalize_syscalls_linear() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "omena_lsp_file_identity_wave_{}_{}",
        std::process::id(),
        current_time_millis()
    ));
    fs::create_dir_all(root.as_path())?;
    let mut uris = Vec::new();
    for index in 0..24 {
        let path = root.join(format!("Style{index}.module.scss"));
        fs::write(path.as_path(), format!(".item{index} {{ color: red; }}"))?;
        uris.push(raw_test_file_uri(path.as_path()));
    }

    crate::protocol::reset_file_uri_identity_cache_for_test();
    let mut state = LspShellState::default();
    let resolution_inputs = resolution_inputs_for_workspace_uri(&state, None);
    for uri in uris.iter().take(12) {
        state.insert_document(
            uri.as_str(),
            lsp_text_document_state(
                uri.clone(),
                None,
                "scss".to_string(),
                0,
                ".item { color: red; }".to_string(),
                &resolution_inputs,
            ),
        );
    }

    crate::protocol::reset_file_uri_identity_canonicalize_syscall_count_for_test();
    for uri in uris.iter().skip(12) {
        state.insert_document(
            uri.as_str(),
            lsp_text_document_state(
                uri.clone(),
                None,
                "scss".to_string(),
                0,
                ".item { color: red; }".to_string(),
                &resolution_inputs,
            ),
        );
    }
    let canonicalize_count =
        crate::protocol::file_uri_identity_canonicalize_syscall_count_for_test();
    assert!(
        canonicalize_count <= 12,
        "second wave must canonicalize only newly admitted paths, not old-store comparisons: {canonicalize_count}"
    );

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
    if let Some(document) = state.document_mut(source_uri) {
        document.workspace_folder_uri = Some(encoded_workspace_uri.to_string());
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

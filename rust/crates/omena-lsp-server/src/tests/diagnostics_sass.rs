#![allow(clippy::expect_used)]

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

#[test]
fn style_diagnostics_surface_sass_module_identity_conflicts_from_lsp() {
    let mut state = LspShellState::default();
    for (uri, text) in [
        (
            "file:///workspace-a/src/App.module.scss",
            "@use \"./theme\" as theme;",
        ),
        (
            "file:///workspace-a/src/_theme.scss",
            "@forward \"./tokens\" with ($brand: red); @forward \"./tokens\" with ($brand: blue);",
        ),
        (
            "file:///workspace-a/src/_tokens.scss",
            "$brand: blue !default;",
        ),
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
    let conflict = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.pointer("/code") == Some(&json!("sassModuleConfigurationConflict"))
        })
        .expect("LSP style diagnostics should surface Sass module identity conflicts");

    assert_eq!(
        conflict.pointer("/data/provenance/2"),
        Some(&json!("omena-query.style-diagnostics"))
    );
    assert!(
        conflict
            .pointer("/message")
            .and_then(Value::as_str)
            .is_some_and(|message| {
                message.contains("_tokens.scss")
                    && message.contains("brand=3:red")
                    && message.contains("brand=4:blue")
            }),
        "diagnostic should describe the conflicting configured module instance: {conflict:?}"
    );
}

#[test]
fn style_diagnostics_resolve_sass_symbols_through_tsconfig_path_alias() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-sass-diagnostics-tsconfig-alias-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let app_style_path = workspace_path.join("src").join("App.module.scss");
    let tokens_style_path = workspace_path
        .join("src")
        .join("styles")
        .join("_tokens.scss");
    fs::create_dir_all(fixture_parent(
        app_style_path.as_path(),
        "app style fixture path has parent directory",
    )?)?;
    fs::create_dir_all(fixture_parent(
        tokens_style_path.as_path(),
        "tokens style fixture path has parent directory",
    )?)?;
    fs::write(
        workspace_path.join("tsconfig.json"),
        r#"{"compilerOptions":{"baseUrl":".","paths":{"$styles/*":["src/styles/*"]}}}"#,
    )?;

    let workspace_uri = path_to_file_uri(workspace_path.as_path());
    let app_style_uri = path_to_file_uri(app_style_path.as_path());
    let tokens_style_uri = path_to_file_uri(tokens_style_path.as_path());

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
                        "name": "workspace-a",
                    },
                ],
            },
        }),
    );
    for (uri, text) in [
        (
            app_style_uri.as_str(),
            "@import \"$styles/_tokens.scss\";\n.button { color: $brand; padding: $missing; }",
        ),
        (tokens_style_uri.as_str(), "$brand: red;"),
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
            "id": 2,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": app_style_uri,
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
        .filter_map(|diagnostic| diagnostic.pointer("/message").and_then(Value::as_str))
        .collect::<Vec<_>>();

    assert_eq!(
        missing_sass_messages,
        vec!["Sass variable '$missing' not found in the visible Sass module graph."],
        "tsconfig path aliases should make imported Sass symbols visible without hiding unresolved controls"
    );

    let _ = fs::remove_dir_all(workspace_path.as_path());
    Ok(())
}

#[test]
fn style_diagnostics_keep_relative_import_symbols_visible() {
    let mut state = LspShellState::default();
    for (uri, text) in [
        (
            "file:///workspace-a/src/App.module.scss",
            "@import \"./tokens\";\n.button { color: $brand; padding: $missing; }",
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
        .filter_map(|diagnostic| diagnostic.pointer("/message").and_then(Value::as_str))
        .collect::<Vec<_>>();

    assert_eq!(
        missing_sass_messages,
        vec!["Sass variable '$missing' not found in the visible Sass module graph."],
        "relative Sass imports should keep imported symbols visible without hiding unresolved controls"
    );
}

#[test]
fn style_diagnostics_surface_streaming_ifds_cross_file_reachability_from_lsp() {
    let mut state = LspShellState::default();
    for (uri, text) in [
        (
            "file:///workspace-a/src/Button.module.scss",
            "@use \"./tokens\" as tokens;\n.root { color: tokens.$brand; }",
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

    let importer_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/Button.module.scss",
                },
            },
        }),
    );
    let importer_diagnostics = importer_response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .expect("style diagnostics response contains an array");
    let streaming = importer_diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.pointer("/code") == Some(&json!("crossFileStreamingReachability"))
        })
        .expect("LSP style diagnostics should surface streaming-IFDS reachability");
    assert_eq!(
        streaming.pointer("/data/provenance/1"),
        Some(&json!(
            "omena-streaming-ifds.cross-file-reachability-report"
        )),
    );
    assert!(
        streaming
            .pointer("/message")
            .and_then(Value::as_str)
            .is_some_and(|message| message
                == "cross-file dataflow reaches 1 module(s) via resolved edges; paths are omitted from diagnostics"),
        "streaming diagnostic should summarize reachability without publishing paths: {streaming:?}"
    );

    let leaf_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/_tokens.scss",
                },
            },
        }),
    );
    let leaf_diagnostics = leaf_response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .expect("leaf style diagnostics response contains an array");
    assert!(
        leaf_diagnostics.iter().all(|diagnostic| {
            diagnostic.pointer("/code") != Some(&json!("crossFileStreamingReachability"))
        }),
        "leaf module should not surface cross-file streaming reachability: {leaf_diagnostics:?}"
    );
}

#[test]
fn style_diagnostics_surface_unified_cross_file_scc_from_lsp() {
    let mut state = LspShellState::default();
    for (uri, text) in [
        (
            "file:///workspace-a/src/a.module.scss",
            r#".a { composes: b from "./b.module.scss"; }"#,
        ),
        (
            "file:///workspace-a/src/b.module.scss",
            r#".b { composes: a from "./a.module.scss"; }"#,
        ),
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

    let response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/a.module.scss",
                },
            },
        }),
    );
    let diagnostics = response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .expect("style diagnostics response contains an array");
    let cycle = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.pointer("/code") == Some(&json!("crossFileStyleCycle")))
        .expect("LSP style diagnostics should surface unified SCC cycles");

    assert_eq!(
        cycle.pointer("/data/crossFileScc/featureGate"),
        Some(&json!("cross-file-scc-v0"))
    );
    assert_eq!(
        cycle.pointer("/data/crossFileScc/connectivityBackend"),
        Some(&json!("exactTarjanScc"))
    );
    assert_eq!(
        cycle.pointer("/data/crossFileScc/polylogBoundScope"),
        Some(&json!("notClaimedExactTraversal"))
    );
    assert_eq!(
        cycle.pointer("/data/crossFileScc/theoremClaimed"),
        Some(&json!(false))
    );
}

#[test]
fn style_diagnostics_surface_replica_ensemble_inconsistency_from_lsp() {
    let mut state = LspShellState::default();
    for (uri, text) in [
        (
            "file:///workspace-a/src/App.module.scss",
            "@use \"./theme\";\n.button { color: red; }\n.button { color: green; }",
        ),
        (
            "file:///workspace-a/src/_theme.scss",
            ".button { color: red; }\n.button { color: blue; }",
        ),
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

    let response = handle_lsp_message(
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
    let diagnostics = response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .expect("style diagnostics response contains an array");
    let ensemble = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.pointer("/code") == Some(&json!("replicaEnsembleInconsistency"))
        })
        .expect("LSP style diagnostics should surface replica-ensemble inconsistency");
    assert_eq!(
        ensemble.pointer("/data/provenance/2"),
        Some(&json!("omena-ensemble.cross-file-inconsistency-report")),
    );
    assert!(
        ensemble
            .pointer("/message")
            .and_then(Value::as_str)
            .is_some_and(|message| message.contains("not a default product decision mechanism")),
        "ensemble diagnostic should keep hint-scope wording: {ensemble:?}"
    );

    let mut consistent_state = LspShellState::default();
    for (uri, text) in [
        (
            "file:///workspace-a/src/App.module.scss",
            "@use \"./theme\";\n.button { color: red; }\n.button { color: green; }",
        ),
        (
            "file:///workspace-a/src/_theme.scss",
            ".button { color: red; }\n.button { color: green; }",
        ),
    ] {
        handle_lsp_message(
            &mut consistent_state,
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

    let consistent_response = handle_lsp_message(
        &mut consistent_state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
            },
        }),
    );
    let consistent_diagnostics = consistent_response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .expect("consistent style diagnostics response contains an array");
    assert!(
        consistent_diagnostics.iter().all(|diagnostic| {
            diagnostic.pointer("/code") != Some(&json!("replicaEnsembleInconsistency"))
        }),
        "matching replica winners must not surface ensemble inconsistency: {consistent_diagnostics:?}"
    );
}

#[test]
fn style_diagnostics_surface_rg_flow_only_when_deep_analysis_is_enabled() {
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/RgFlow.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": ":root {\n  --seed: 1px;\n  --a: var(--seed);\n  --b: var(--seed);\n  --c: var(--seed);\n  --d: var(--seed);\n}\n",
                },
            },
        }),
    );

    let default_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/RgFlow.module.scss",
                },
            },
        }),
    );
    let default_diagnostics = default_response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .expect("default style diagnostics response contains an array");
    assert!(
        default_diagnostics
            .iter()
            .all(|diagnostic| diagnostic.pointer("/code") != Some(&json!("rgFlowRelevantOperator"))),
        "RG-flow must stay off by default: {default_diagnostics:?}"
    );

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeConfiguration",
            "params": {
                "settings": {
                    "omena": {
                        "diagnostics": {
                            "deepAnalysis": true,
                        },
                    },
                },
            },
        }),
    );

    let deep_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/RgFlow.module.scss",
                },
            },
        }),
    );
    let deep_diagnostics = deep_response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .expect("deep-analysis style diagnostics response contains an array");
    let rg_flow = deep_diagnostics
        .iter()
        .find(|diagnostic| diagnostic.pointer("/code") == Some(&json!("rgFlowRelevantOperator")))
        .expect("opt-in deep analysis should surface RG-flow hint");
    assert_eq!(
        rg_flow.pointer("/data/provenance/3"),
        Some(&json!("omena-rg-flow.coupling-jacobian-spectrum")),
    );
    assert_eq!(
        rg_flow.pointer("/severity"),
        Some(&json!(4)),
        "RG-flow remains a hint-level opt-in diagnostic"
    );
}

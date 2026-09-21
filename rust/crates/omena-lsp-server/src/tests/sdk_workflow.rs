#![allow(
    clippy::expect_used,
    reason = "Contract fixtures assert exact success and refusal outcomes"
)]

use super::*;

const WORKSPACE_ROOT: &str = "file:///workspace-a";
const STYLE_URI: &str = "file:///workspace-a/src/App.module.scss";

#[test]
fn sdk_workflows_read_the_open_document_snapshot() -> Result<(), String> {
    let mut state = LspShellState::default();
    open_style(&mut state, 1, ".card { color: red; }");

    let snapshot = request(
        &mut state,
        1,
        "snapshot",
        json!({
            "workspaceRoot": WORKSPACE_ROOT,
        }),
    )?;
    let snapshot_id = snapshot
        .pointer("/result/snapshotId")
        .cloned()
        .ok_or_else(|| "snapshot response must contain an identity".to_string())?;

    let diagnostics = request(
        &mut state,
        2,
        "diagnostics",
        json!({
            "snapshotId": snapshot_id,
            "stylePath": STYLE_URI,
            "styleSource": ".card { color: red; }",
        }),
    )?;
    assert_eq!(
        diagnostics.pointer("/result/summary/classSelectorCount"),
        Some(&json!(1)),
    );
    assert_eq!(
        diagnostics.pointer("/result/snapshotId"),
        snapshot.pointer("/result/snapshotId"),
    );

    let query = request(
        &mut state,
        3,
        "query",
        json!({
            "snapshotId": snapshot_id,
            "queryKind": "styleSummary",
            "input": { "stylePath": STYLE_URI },
        }),
    )?;
    assert_eq!(
        query.pointer("/result/payload/selectorNames/0"),
        Some(&json!("card")),
    );
    Ok(())
}

#[test]
fn sdk_workflow_rejects_a_snapshot_after_document_change() -> Result<(), String> {
    let mut state = LspShellState::default();
    open_style(&mut state, 1, ".card { color: red; }");
    let snapshot = request(
        &mut state,
        1,
        "snapshot",
        json!({
            "workspaceRoot": WORKSPACE_ROOT,
        }),
    )?;
    let snapshot_id = snapshot
        .pointer("/result/snapshotId")
        .cloned()
        .ok_or_else(|| "snapshot response must contain an identity".to_string())?;

    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": { "uri": STYLE_URI, "version": 2 },
                "contentChanges": [{ "text": ".card { color: blue; }" }],
            },
        }),
    );

    let response = request(
        &mut state,
        2,
        "query",
        json!({
            "snapshotId": snapshot_id,
            "queryKind": "styleSummary",
            "input": { "stylePath": STYLE_URI },
        }),
    )?;
    assert_eq!(
        response.pointer("/error/data/error/class"),
        Some(&json!("workspace")),
    );
    assert_eq!(
        response.pointer("/error/data/error/context/code"),
        Some(&json!("workspace.snapshot-mismatch")),
    );
    Ok(())
}

#[test]
fn sdk_workflow_keeps_workspace_roots_isolated() -> Result<(), String> {
    let mut state = LspShellState::default();
    open_style(&mut state, 1, ".card { color: red; }");
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-b/src/Other.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": ".other { color: blue; }",
                },
            },
        }),
    );
    let snapshot = request(
        &mut state,
        1,
        "snapshot",
        json!({
            "workspaceRoot": WORKSPACE_ROOT,
        }),
    )?;
    let response = request(
        &mut state,
        2,
        "query",
        json!({
            "snapshotId": snapshot["result"]["snapshotId"],
            "queryKind": "styleSummary",
            "input": { "stylePath": "file:///workspace-b/src/Other.module.scss" },
        }),
    )?;
    assert_eq!(
        response.pointer("/error/data/error/context/code"),
        Some(&json!("workspace.style-path-not-found")),
    );
    Ok(())
}

fn open_style(state: &mut LspShellState, version: i64, text: &str) {
    handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": STYLE_URI,
                    "languageId": "scss",
                    "version": version,
                    "text": text,
                },
            },
        }),
    );
}

fn request(
    state: &mut LspShellState,
    id: u64,
    operation: &str,
    request: Value,
) -> Result<Value, String> {
    handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": SDK_WORKFLOW_REQUEST,
            "params": {
                "workspaceRoot": WORKSPACE_ROOT,
                "operation": operation,
                "request": request,
            },
        }),
    )
    .ok_or_else(|| "SDK workflow request must return a response".to_string())
}

fn export_bound_snapshot(state: &mut LspShellState, root: &str) -> Result<Value, String> {
    let response = handle_lsp_message(
        state,
        json!({
            "jsonrpc":"2.0", "id":90, "method":SDK_WORKFLOW_REQUEST, "params": {
                "contractVersion":"1", "workspaceRoot":root, "operation":"exportSnapshot",
                "request":{"workspaceRoot":root},
            }
        }),
    )
    .ok_or_else(|| "bound export must respond".to_string())?;
    if response.get("error").is_some() {
        return Err(format!("bound export failed: {response}"));
    }
    Ok(response["result"].clone())
}

fn exported_binding(value: &Value) -> Result<omena_query::OmenaWorkspaceSnapshotBindingV0, String> {
    serde_json::from_value(value["snapshotBinding"].clone()).map_err(|error| error.to_string())
}

#[test]
fn bound_sdk_two_roots_keep_reachable_sif_and_trust_inputs_reconstructable() -> TestResult {
    fn imported(
        exported: &Value,
        root: &std::path::Path,
    ) -> Result<omena_query::OmenaSdkWorkspaceV0, Box<dyn std::error::Error>> {
        Ok(omena_query::OmenaSdkWorkspaceV0::open_imported_snapshot(
            omena_query::OmenaSdkSnapshotRequestV0 {
                workspace_root: path_to_file_uri(root),
            },
            serde_json::from_value(exported["response"]["styleSources"].clone())?,
            serde_json::from_value(exported["response"]["snapshotInputs"].clone())?,
            exported_binding(exported)?,
            &omena_query::load_omena_query_workspace_utility_class_intelligence(root, None),
        )?)
    }
    let fixture = std::env::temp_dir().join(format!("omena-sdk-two-roots-{}", std::process::id()));
    let root_a = fixture.join("a");
    let root_b = fixture.join("b");
    std::fs::create_dir_all(&root_a)?;
    std::fs::create_dir_all(&root_b)?;
    std::fs::create_dir_all(fixture.join("shared"))?;
    let leaf = fixture.join("shared/_leaf.scss");
    std::fs::write(&leaf, "$brand: red !default;\n")?;
    std::fs::write(root_a.join("_tokens.scss"), "@forward '../shared/leaf';\n")?;
    std::fs::write(root_b.join("_tokens.scss"), "$brand: blue !default;\n")?;
    let text = "@use './tokens' as tokens;\n.card { color: tokens.$brand; }\n";
    let a_uri = path_to_file_uri(&root_a);
    let b_uri = path_to_file_uri(&root_b);
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "workspaceFolders":[{"uri":a_uri,"name":"a"},{"uri":b_uri,"name":"b"}],
        }}),
    );
    for root in [&root_a, &root_b] {
        let style_uri = path_to_file_uri(&root.join("App.module.scss"));
        std::fs::write(root.join("App.module.scss"), text)?;
        handle_lsp_message(
            &mut state,
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
                "textDocument":{"uri":style_uri,"languageId":"scss","version":1,"text":text},
            }}),
        );
    }
    crate::external_sif_loader::refresh_external_sifs_for_state(&mut state);
    let a = export_bound_snapshot(&mut state, &a_uri)?;
    let b = export_bound_snapshot(&mut state, &b_uri)?;
    let import_a = imported(&a, &root_a);
    let import_b = imported(&b, &root_b);
    if let Some(directory) = std::env::var_os("OMENA_WORKSPACE_SNAPSHOT_TEST_RECEIPTS") {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(std::path::PathBuf::from(directory).join("two-root-import.json"))?;
        serde_json::to_writer_pretty(
            file,
            &json!({
                "aExport":a, "bExport":b,
                "aImportError":import_a.as_ref().err().map(ToString::to_string),
                "bImportError":import_b.as_ref().err().map(ToString::to_string),
                "admittedGlobalSifs":state.resolution.external_sifs,
                "admittedGlobalTrust":state.resolution.external_sif_trust_records,
            }),
        )?;
    }
    if import_a.is_err() || import_b.is_err() {
        std::fs::remove_dir_all(&fixture)?;
    }
    assert!(
        import_a.is_ok() && import_b.is_ok(),
        "each actual LSP root export must independently reconstruct: A={:?}, B={:?}",
        import_a.as_ref().err(),
        import_b.as_ref().err()
    );
    let imported_a = import_a?;
    let imported_b = import_b?;
    assert_eq!(imported_a.snapshot_binding(), Some(&exported_binding(&a)?));
    assert_eq!(imported_b.snapshot_binding(), Some(&exported_binding(&b)?));
    let select = |exported: &Value| -> Result<_, Box<dyn std::error::Error>> {
        Ok(
            omena_query::select_omena_workspace_snapshot_external_sifs_v0(
                &serde_json::from_value::<Vec<omena_query::OmenaQueryStyleSourceInputV0>>(
                    exported["response"]["styleSources"].clone(),
                )?,
                &serde_json::from_value(
                    exported["response"]["snapshotInputs"]["resolutionInputs"].clone(),
                )?,
                &state.resolution.external_sifs,
                &state.resolution.external_sif_trust_records,
                &state.resolution.external_sif_resolution_edges,
            )?,
        )
    };
    let (a_sifs, a_trust, _) = select(&a)?;
    let (b_sifs, b_trust, _) = select(&b)?;
    assert!(
        a_sifs
            .iter()
            .any(|sif| sif.sif.canonical_url == path_to_file_uri(&leaf))
    );
    assert!(
        a_sifs
            .iter()
            .all(|sif| !sif.sif.canonical_url.starts_with(&b_uri))
    );
    assert!(
        b_sifs
            .iter()
            .all(|sif| sif.sif.canonical_url.starts_with(&b_uri))
    );
    assert!(!a_trust.is_empty());
    assert!(!b_trust.is_empty());
    let a_first = exported_binding(&a)?;
    let reader = state
        .sdk_snapshot_publishers
        .borrow()
        .get(&a_uri)
        .ok_or("A owner expected")?
        .reader();
    std::fs::write(root_b.join("_tokens.scss"), "$brand: green !default;\n")?;
    crate::external_sif_loader::refresh_external_sifs_for_state(&mut state);
    let a_after_b = export_bound_snapshot(&mut state, &a_uri)?;
    let b_after_b = export_bound_snapshot(&mut state, &b_uri)?;
    imported(&a_after_b, &root_a)?;
    imported(&b_after_b, &root_b)?;
    assert_eq!(
        a["response"]["snapshotInputs"],
        a_after_b["response"]["snapshotInputs"]
    );
    assert_eq!(
        a_first.input_commitment(),
        exported_binding(&a_after_b)?.input_commitment()
    );
    assert_ne!(
        exported_binding(&b)?.input_commitment(),
        exported_binding(&b_after_b)?.input_commitment()
    );
    let a_before_leaf = exported_binding(&a_after_b)?;
    std::fs::write(&leaf, "$brand: orange !default;\n")?;
    assert!(
        imported(&a_after_b, &root_a).is_err(),
        "actual changed reachable SIF cannot reconstruct old commitment"
    );
    crate::external_sif_loader::refresh_external_sifs_for_state(&mut state);
    assert!(reader.with_current_binding(&a_before_leaf, || ()).is_err());
    let a_after_leaf = export_bound_snapshot(&mut state, &a_uri)?;
    assert_ne!(
        a_before_leaf.input_commitment(),
        exported_binding(&a_after_leaf)?.input_commitment()
    );
    imported(&a_after_leaf, &root_a)?;
    imported(&export_bound_snapshot(&mut state, &b_uri)?, &root_b)?;
    std::fs::remove_dir_all(fixture)?;
    Ok(())
}

#[test]
fn bound_sdk_changed_style_bytes_cannot_reuse_diagnostic_binding() -> TestResult {
    fn diagnostics(
        state: &mut LspShellState,
        binding: &omena_query::OmenaWorkspaceSnapshotBindingV0,
    ) -> Result<Value, String> {
        let text = state
            .document(STYLE_URI)
            .ok_or("open style expected")?
            .text
            .clone();
        let response = handle_lsp_message(state, json!({
            "jsonrpc":"2.0", "id":91, "method":SDK_WORKFLOW_REQUEST, "params": {
                "contractVersion":"1", "workspaceRoot":WORKSPACE_ROOT,
                "operation":"diagnostics", "snapshotBinding":binding,
                "request":{"snapshotId":binding.snapshot_id(),"stylePath":STYLE_URI,"styleSource":text},
            }
        })).ok_or("bound diagnostics must respond")?;
        if response.get("error").is_some() {
            return Err(format!("{response}"));
        }
        Ok(response["result"]["response"].clone())
    }

    let mut state = LspShellState::default();
    open_style(&mut state, 1, ".card { color: red; }");
    let before = exported_binding(&export_bound_snapshot(&mut state, WORKSPACE_ROOT)?)?;
    let before_payload = diagnostics(&mut state, &before)?;
    let before_bytes = serde_json::to_vec(&before_payload)?;
    let unchanged = exported_binding(&export_bound_snapshot(&mut state, WORKSPACE_ROOT)?)?;
    let unchanged_bytes = serde_json::to_vec(&diagnostics(&mut state, &unchanged)?)?;
    assert_eq!(unchanged, before);
    assert_eq!(unchanged_bytes, before_bytes);
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc":"2.0", "method":"textDocument/didChange", "params": {
                "textDocument":{"uri":STYLE_URI,"version":2},
                "contentChanges":[{"text":".card { color: red; }\n.next { color: blue; }"}],
            }
        }),
    );
    let after = exported_binding(&export_bound_snapshot(&mut state, WORKSPACE_ROOT)?)?;
    let after_payload = diagnostics(&mut state, &after)?;
    let after_bytes = serde_json::to_vec(&after_payload)?;
    assert_eq!(before_payload["summary"]["classSelectorCount"], json!(1));
    assert_eq!(after_payload["summary"]["classSelectorCount"], json!(2));
    assert_ne!(
        before_bytes, after_bytes,
        "fixture must change the real semantic payload"
    );
    if let Some(path) = std::env::var_os("OMENA_SNAPSHOT_MUTATION_RECEIPT") {
        let path = std::path::PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "mutation receipt must have an explicit absolute path"
        );
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        std::io::Write::write_all(
            &mut file,
            &serde_json::to_vec_pretty(&json!({
                "beforeBinding":before, "unchangedBinding":unchanged, "afterBinding":after,
                "beforePayloadBytes":String::from_utf8(before_bytes.clone())?,
                "unchangedPayloadBytes":String::from_utf8(unchanged_bytes)?,
                "afterPayloadBytes":String::from_utf8(after_bytes.clone())?,
                "sameBinding":before==after, "payloadsDiffer":before_bytes!=after_bytes,
            }))?,
        )?;
    }
    if before == after {
        assert_eq!(
            before_bytes, after_bytes,
            "one admitted binding must not accompany different complete diagnostic bytes"
        );
    }
    assert_ne!(
        before, after,
        "changed admitted style bytes must advance the contract binding"
    );
    Ok(())
}

#[test]
fn bound_sdk_settings_and_manifest_changes_revoke_the_actual_owner() -> Result<(), String> {
    let mut state = LspShellState::default();
    open_style(&mut state, 1, ".card { color: red; }");
    let first = exported_binding(&export_bound_snapshot(&mut state, WORKSPACE_ROOT)?)?;
    let reader = state
        .sdk_snapshot_publishers
        .borrow()
        .get(WORKSPACE_ROOT)
        .ok_or_else(|| "actual LSP publisher expected".to_string())?
        .reader();
    assert!(reader.with_current_binding(&first, || ()).is_ok());
    crate::settings::apply_diagnostic_settings(&mut state, Some(&json!({"deepAnalysis":true})));
    assert!(reader.with_current_binding(&first, || ()).is_err());
    let second = exported_binding(&export_bound_snapshot(&mut state, WORKSPACE_ROOT)?)?;
    assert_ne!(first, second);
    crate::settings::apply_diagnostic_settings(&mut state, Some(&json!({"deepAnalysis":true})));
    let unchanged = exported_binding(&export_bound_snapshot(&mut state, WORKSPACE_ROOT)?)?;
    assert_eq!(
        second, unchanged,
        "accepted unchanged inputs retain the portable binding"
    );

    let manifest_path =
        std::env::temp_dir().join(format!("omena-sdk-manifest-{}.json", std::process::id()));
    std::fs::write(&manifest_path, "{\"name\":\"snapshot-fixture\"}").map_err(|e| e.to_string())?;
    assert!(crate::settings::apply_resolution_settings(
        &mut state,
        Some(&json!({
            "packageManifestPaths":[manifest_path.to_string_lossy()],
        }))
    ));
    assert!(reader.with_current_binding(&second, || ()).is_err());
    let third = exported_binding(&export_bound_snapshot(&mut state, WORKSPACE_ROOT)?)?;
    assert_ne!(second, third);
    crate::settings::apply_feature_settings(&mut state, Some(&json!({"hover":false})));
    assert!(reader.with_current_binding(&third, || ()).is_err());
    let fourth = exported_binding(&export_bound_snapshot(&mut state, WORKSPACE_ROOT)?)?;
    assert_ne!(third, fourth);
    std::fs::remove_file(manifest_path).map_err(|e| e.to_string())?;
    Ok(())
}

#[test]
fn bound_sdk_resolver_configuration_changes_revoke_the_actual_owner() -> TestResult {
    let root = std::env::temp_dir().join(format!("omena-sdk-resolver-{}", std::process::id()));
    std::fs::create_dir_all(root.join("src"))?;
    let config_path = root.join("tsconfig.json");
    let first_config = r#"{"compilerOptions":{"baseUrl":".","paths":{"@styles/*":["src/*"]}}}"#;
    std::fs::write(&config_path, first_config)?;
    let root_uri = path_to_file_uri(&root);
    let style_uri = path_to_file_uri(&root.join("src/App.module.scss"));
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc":"2.0", "id":1, "method":"initialize", "params": {
                "workspaceFolders":[{"uri":root_uri,"name":"snapshot-resolver"}],
            }
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc":"2.0", "method":"textDocument/didOpen", "params": {
                "textDocument":{"uri":style_uri,"languageId":"scss","version":1,"text":".card {}"},
            }
        }),
    );
    let first_export = export_bound_snapshot(&mut state, &root_uri)?;
    let first = exported_binding(&first_export)?;
    let reader = state
        .sdk_snapshot_publishers
        .borrow()
        .get(&root_uri)
        .ok_or("actual resolver owner expected")?
        .reader();
    let first_inputs = state
        .resolution
        .workspace_style_resolution_inputs
        .get(&root_uri)
        .ok_or("actual loaded resolver inputs expected")?
        .clone();
    assert!(!first_inputs.tsconfig_path_mappings.is_empty());
    crate::workspace_resolution::refresh_workspace_resolution_inputs(&mut state);
    assert_eq!(
        exported_binding(&export_bound_snapshot(&mut state, &root_uri)?)?,
        first,
        "an unchanged admitted resolver configuration retains the same binding"
    );

    std::fs::write(
        &config_path,
        r#"{"compilerOptions":{"baseUrl":".","paths":{"@styles/*":["alternate/*"]}}}"#,
    )?;
    crate::workspace_resolution::refresh_workspace_resolution_inputs(&mut state);
    assert!(reader.with_current_binding(&first, || ()).is_err());
    let second_export = export_bound_snapshot(&mut state, &root_uri)?;
    let second = exported_binding(&second_export)?;
    assert_ne!(second, first);
    assert_ne!(
        state
            .resolution
            .workspace_style_resolution_inputs
            .get(&root_uri),
        Some(&first_inputs)
    );
    let mut mixed_inputs: omena_query::OmenaWorkspaceSnapshotTransferV0 =
        serde_json::from_value(first_export["response"]["snapshotInputs"].clone())?;
    mixed_inputs.resolution_inputs = serde_json::from_value(
        second_export["response"]["snapshotInputs"]["resolutionInputs"].clone(),
    )?;
    let utility = omena_query::load_omena_query_workspace_utility_class_intelligence(&root, None);
    let refusal = omena_query::OmenaSdkWorkspaceV0::open_imported_snapshot(
        omena_query::OmenaSdkSnapshotRequestV0 {
            workspace_root: root_uri,
        },
        serde_json::from_value(first_export["response"]["styleSources"].clone())?,
        mixed_inputs,
        first,
        &utility,
    )
    .expect_err("new actual resolver facts cannot travel under the old commitment");
    assert_eq!(refusal.context.code, "workspace.snapshot-binding-mismatch");
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn bound_sdk_local_bridge_recomputes_trust_and_fences_accepted_sif_updates() -> TestResult {
    let root = std::env::temp_dir().join(format!("omena-sdk-bridge-{}", std::process::id()));
    std::fs::create_dir_all(root.join("src"))?;
    std::fs::create_dir_all(root.join("vendor"))?;
    let external = root.join("vendor/_tokens.scss");
    std::fs::write(&external, "$brand: red !default;\n")?;
    let external_uri = path_to_file_uri(&external);
    let style_path = root.join("src/App.module.scss");
    let style_uri = path_to_file_uri(&style_path);
    let root_uri = path_to_file_uri(&root);
    let style_text =
        format!("@use \"{external_uri}\" as tokens;\n.card {{ color: tokens.$brand; }}\n");
    std::fs::write(&style_path, &style_text)?;
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc":"2.0", "id":1, "method":"initialize", "params": {
                "workspaceFolders":[{"uri":root_uri,"name":"snapshot-bridge"}],
            }
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc":"2.0", "method":"textDocument/didOpen", "params": {
                "textDocument":{"uri":style_uri,"languageId":"scss","version":1,"text":style_text},
            }
        }),
    );
    assert!(
        !state.resolution.external_sifs.is_empty(),
        "actual local bridge admission must run"
    );
    assert!(!state.resolution.external_sif_trust_records.is_empty());
    let exported = export_bound_snapshot(&mut state, &root_uri)?;
    let first = exported_binding(&exported)?;
    let reader = state
        .sdk_snapshot_publishers
        .borrow()
        .get(&root_uri)
        .ok_or("actual LSP owner expected")?
        .reader();
    let styles = serde_json::from_value(exported["response"]["styleSources"].clone())?;
    let transfer = serde_json::from_value(exported["response"]["snapshotInputs"].clone())?;
    let utility = omena_query::load_omena_query_workspace_utility_class_intelligence(&root, None);
    let imported = omena_query::OmenaSdkWorkspaceV0::open_imported_snapshot(
        omena_query::OmenaSdkSnapshotRequestV0 {
            workspace_root: root_uri.clone(),
        },
        styles,
        transfer,
        first.clone(),
        &utility,
    )?;
    assert_eq!(imported.snapshot_binding(), Some(&first));
    let view = imported.snapshot_read_view()?;
    assert_eq!(
        view.owner()
            .ok_or("reader expected")?
            .with_current_write_binding(&first, || ())
            .expect_err("import is not native write authority")
            .context
            .code,
        "workspace.snapshot-write-owner-required"
    );
    crate::external_sif_loader::refresh_external_sifs_for_state(&mut state);
    assert!(
        reader.with_current_binding(&first, || ()).is_ok(),
        "same admitted SIF/trust facts remain current"
    );
    assert_eq!(
        exported_binding(&export_bound_snapshot(&mut state, &root_uri)?)?,
        first
    );

    std::fs::write(&external, "$brand: blue !default;\n")?;
    let refused = omena_query::OmenaSdkWorkspaceV0::open_imported_snapshot(
        omena_query::OmenaSdkSnapshotRequestV0 {
            workspace_root: root_uri.clone(),
        },
        serde_json::from_value(exported["response"]["styleSources"].clone())?,
        serde_json::from_value(exported["response"]["snapshotInputs"].clone())?,
        first.clone(),
        &utility,
    )
    .expect_err("changed actual bridge bytes cannot import the old semantic commitment");
    assert_eq!(refused.context.code, "workspace.snapshot-binding-mismatch");
    crate::external_sif_loader::refresh_external_sifs_for_state(&mut state);
    assert!(reader.with_current_binding(&first, || ()).is_err());
    let second = exported_binding(&export_bound_snapshot(&mut state, &root_uri)?)?;
    assert_ne!(first, second);

    crate::external_sif_loader::enable_deferred_external_sif_refresh(&mut state);
    let same_job =
        crate::external_sif_loader::prepare_deferred_external_sif_refresh_job(&mut state)
            .ok_or("same-content deferred SIF job expected")?;
    let same_result = crate::external_sif_loader::collect_deferred_external_sif_refresh(same_job);
    assert!(
        !crate::external_sif_loader::apply_deferred_external_sif_refresh_result(
            &mut state,
            same_result
        )
    );
    assert!(reader.with_current_binding(&second, || ()).is_ok());
    assert_eq!(
        exported_binding(&export_bound_snapshot(&mut state, &root_uri)?)?,
        second
    );

    crate::external_sif_loader::enable_deferred_external_sif_refresh(&mut state);
    let old_job = crate::external_sif_loader::prepare_deferred_external_sif_refresh_job(&mut state)
        .ok_or("pending deferred SIF job expected")?;
    std::fs::write(&external, "$brand: green !default;\n")?;
    let old_result = crate::external_sif_loader::collect_deferred_external_sif_refresh(old_job);
    assert_ne!(
        old_result.external_sifs, state.resolution.external_sifs,
        "the pending result must contain a real semantic SIF delta"
    );
    let added_external = root.join("vendor/_spacing.scss");
    std::fs::write(&added_external, "$gap: 1px !default;\n")?;
    let added_external_uri = path_to_file_uri(&added_external);
    let added_uri = path_to_file_uri(&root.join("src/Added.module.scss"));
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc":"2.0", "method":"textDocument/didOpen", "params": {
                "textDocument":{"uri":added_uri,"languageId":"scss","version":1,
                    "text":format!("@use \"{added_external_uri}\" as spacing; .added {{ gap: spacing.$gap; }}")},
            }
        }),
    );
    let third = exported_binding(&export_bound_snapshot(&mut state, &root_uri)?)?;
    let admitted_sifs = state.resolution.external_sifs.clone();
    let admitted_trust = state.resolution.external_sif_trust_records.clone();
    assert!(
        !state.tide_ledger.is_current(&old_result.stamp),
        "an actual bridge topology change must stale the pending result's footprint"
    );
    assert!(
        !crate::external_sif_loader::apply_deferred_external_sif_refresh_result(
            &mut state, old_result
        )
    );
    assert_eq!(state.resolution.external_sifs, admitted_sifs);
    assert_eq!(state.resolution.external_sif_trust_records, admitted_trust);
    assert!(
        reader.with_current_binding(&third, || ()).is_ok(),
        "stamp refusal must leave the actual current owner readable"
    );
    assert_eq!(
        exported_binding(&export_bound_snapshot(&mut state, &root_uri)?)?,
        third
    );

    let current_job =
        crate::external_sif_loader::prepare_deferred_external_sif_refresh_job(&mut state)
            .ok_or("staled work must retain a fresh deferred demand")?;
    let current_result =
        crate::external_sif_loader::collect_deferred_external_sif_refresh(current_job);
    assert!(state.tide_ledger.is_current(&current_result.stamp));
    assert!(
        crate::external_sif_loader::apply_deferred_external_sif_refresh_result(
            &mut state,
            current_result
        )
    );
    assert!(reader.with_current_binding(&third, || ()).is_err());
    assert_ne!(
        exported_binding(&export_bound_snapshot(&mut state, &root_uri)?)?,
        third
    );
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn bound_sdk_watched_open_sif_source_preserves_editor_and_reconstructs_admitted_inputs()
-> TestResult {
    use crate::external_sif_loader::{
        apply_deferred_external_sif_refresh_result, collect_deferred_external_sif_refresh,
        enable_deferred_external_sif_refresh, external_sif_source_uri_is_dependency,
        prepare_deferred_external_sif_refresh_job, refresh_external_sifs_for_state,
    };
    struct Fixture(std::path::PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn notify(state: &mut LspShellState, method: &str, params: Value) {
        handle_lsp_message(
            state,
            json!({"jsonrpc":"2.0","method":method,"params":params}),
        );
    }
    fn observe_import(
        state: &mut LspShellState,
        root: &std::path::Path,
        style_uri: &str,
        label: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let root_uri = path_to_file_uri(root);
        let exported = export_bound_snapshot(state, &root_uri)?;
        let binding = exported_binding(&exported)?;
        let imported = omena_query::OmenaSdkWorkspaceV0::open_imported_snapshot(
            omena_query::OmenaSdkSnapshotRequestV0 {
                workspace_root: root_uri.clone(),
            },
            serde_json::from_value(exported["response"]["styleSources"].clone())?,
            serde_json::from_value(exported["response"]["snapshotInputs"].clone())?,
            binding.clone(),
            &omena_query::load_omena_query_workspace_utility_class_intelligence(root, None),
        );
        let request = json!({"snapshotId":binding.snapshot_id(),"stylePath":style_uri,
            "styleSource":state.document(style_uri).ok_or("owned style expected")?.text});
        let owner = handle_lsp_message(
            state,
            json!({"jsonrpc":"2.0","id":92,
            "method":SDK_WORKFLOW_REQUEST,"params":{"contractVersion":"1",
                "workspaceRoot":root_uri,"snapshotBinding":binding,"operation":"diagnostics",
                "request":request}}),
        )
        .ok_or("owner diagnostics expected")?;
        let receiver = imported
            .as_ref()
            .ok()
            .map(|workspace| {
                workspace
                    .execute_diagnostics(
                        serde_json::from_value(request.clone()).expect("valid fixture request"),
                    )
                    .map(|value| serde_json::to_value(value).expect("serializable diagnostics"))
            })
            .transpose()?;
        if let Some(directory) = std::env::var_os("OMENA_WORKSPACE_SNAPSHOT_TEST_RECEIPTS") {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(std::path::PathBuf::from(directory).join(format!("open-sif-{label}.json")))?;
            serde_json::to_writer_pretty(
                file,
                &json!({"label":label,"export":exported,
                "ownerSifs":state.resolution.external_sifs,
                "ownerTrust":state.resolution.external_sif_trust_records,
                "importError":imported.as_ref().err().map(ToString::to_string),
                "ownerPayload":owner,"receiverPayload":receiver,
                "state":state.snapshot()}),
            )?;
        }
        assert!(
            imported.is_ok(),
            "watched open SIF owner inputs must independently reconstruct: {label}: {:?}",
            imported.as_ref().err()
        );
        assert_eq!(
            owner["result"]["response"],
            receiver.ok_or("receiver payload expected")?,
            "complete owner/receiver diagnostics must agree: {label}"
        );
        Ok(exported)
    }
    let fixture = Fixture(std::env::temp_dir().join(format!(
        "omena-open-sif-watch-{}-{}",
        std::process::id(),
        current_time_millis()
    )));
    let root = &fixture.0;
    std::fs::create_dir_all(root)?;
    let tokens_path = root.join("_tokens.scss");
    let tokens_uri = path_to_file_uri(&tokens_path);
    let style_uri = path_to_file_uri(&root.join("App.module.scss"));
    let other_uri = path_to_file_uri(&root.join("Other.scss"));
    let root_uri = path_to_file_uri(root);
    let blue = "$brand: blue !default;\n";
    let green = "$brand: green !default;\n";
    let purple = "$brand: purple !default;\n";
    let orange = "$brand: orange !default;\n";
    let style_text = format!("@use '{tokens_uri}' as tokens;\n.card {{ color: tokens.$brand; }}\n");
    std::fs::write(&tokens_path, blue)?;
    std::fs::write(root.join("App.module.scss"), &style_text)?;
    std::fs::write(root.join("Other.scss"), ".other {}\n")?;
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({"jsonrpc":"2.0","id":1,"method":"initialize",
        "params":{"workspaceFolders":[{"uri":root_uri,"name":"open-sif-watch"}]}}),
    );
    for (uri, text) in [
        (&tokens_uri, blue),
        (&style_uri, style_text.as_str()),
        (&other_uri, ".other {}\n"),
    ] {
        notify(
            &mut state,
            "textDocument/didOpen",
            json!({"textDocument":{
            "uri":uri,"languageId":"scss","version":1,"text":text}}),
        );
    }
    refresh_external_sifs_for_state(&mut state);
    assert!(external_sif_source_uri_is_dependency(&state, &tokens_uri));
    assert!(!external_sif_source_uri_is_dependency(&state, &other_uri));
    let unknown_alias = format!("{root_uri}/./_tokens.scss");
    assert!(
        state.known_document_file_id(&unknown_alias).is_none(),
        "known alias lookup must not canonicalize an unadmitted spelling"
    );
    assert!(!external_sif_source_uri_is_dependency(
        &state,
        &unknown_alias
    ));
    let first = observe_import(&mut state, root, &style_uri, "initial")?;
    let blue_sifs = state.resolution.external_sifs.clone();
    let generation_count = state.external_sif_bridge_generation_count;
    notify(
        &mut state,
        "textDocument/didChange",
        json!({"textDocument":{"uri":tokens_uri,"version":2},
        "contentChanges":[{"text":green}]}),
    );
    assert_eq!(std::fs::read_to_string(&tokens_path)?, blue);
    assert_eq!(state.resolution.external_sifs, blue_sifs);
    assert_eq!(
        state.external_sif_bridge_generation_count, generation_count,
        "unsaved text alone must not regenerate disk SIFs"
    );
    observe_import(&mut state, root, &style_uri, "unsaved")?;
    notify(
        &mut state,
        "workspace/didChangeWatchedFiles",
        json!({"changes":[{"uri":other_uri,"type":2}]}),
    );
    assert_eq!(
        state.external_sif_bridge_generation_count, generation_count,
        "an unrelated open document watch must not regenerate SIFs"
    );
    assert_eq!(state.resolution.external_sifs, blue_sifs);
    std::fs::write(&tokens_path, green)?;
    notify(
        &mut state,
        "workspace/didChangeWatchedFiles",
        json!({"changes":[{"uri":tokens_uri,"type":2}]}),
    );
    let saved = observe_import(&mut state, root, &style_uri, "saved")?;
    assert_ne!(state.resolution.external_sifs, blue_sifs);
    assert_eq!(
        state
            .document(&tokens_uri)
            .ok_or("open tokens expected")?
            .text,
        green
    );
    assert_eq!(
        state
            .document(&tokens_uri)
            .ok_or("open tokens expected")?
            .version,
        2
    );
    assert_ne!(
        exported_binding(&first)?.input_commitment(),
        exported_binding(&saved)?.input_commitment()
    );
    notify(
        &mut state,
        "workspace/didChangeWatchedFiles",
        json!({"changes":[{"uri":tokens_uri,"type":2}]}),
    );
    let unchanged = observe_import(&mut state, root, &style_uri, "unchanged-watch")?;
    assert_eq!(
        exported_binding(&saved)?.input_commitment(),
        exported_binding(&unchanged)?.input_commitment()
    );

    enable_deferred_external_sif_refresh(&mut state);
    let old_job =
        prepare_deferred_external_sif_refresh_job(&mut state).ok_or("old SIF job expected")?;
    let old_result = collect_deferred_external_sif_refresh(old_job);
    notify(
        &mut state,
        "textDocument/didChange",
        json!({"textDocument":{"uri":tokens_uri,"version":3},
        "contentChanges":[{"text":purple}]}),
    );
    assert!(
        state.tide_ledger.is_current(&old_result.stamp),
        "unsaved text must not stale an in-flight disk SIF job"
    );
    assert!(!state.tide_sif_lane.has_demand());
    notify(
        &mut state,
        "workspace/didChangeWatchedFiles",
        json!({"changes":[{"uri":other_uri,"type":2}]}),
    );
    assert!(state.tide_ledger.is_current(&old_result.stamp));
    assert!(!state.tide_sif_lane.has_demand());
    let before_watch = state.resolution.external_sifs.clone();
    std::fs::write(&tokens_path, orange)?;
    notify(
        &mut state,
        "workspace/didChangeWatchedFiles",
        json!({"changes":[{"uri":tokens_uri,"type":2}]}),
    );
    if let Some(directory) = std::env::var_os("OMENA_WORKSPACE_SNAPSHOT_TEST_RECEIPTS") {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(std::path::PathBuf::from(directory).join("open-sif-inflight.json"))?;
        serde_json::to_writer_pretty(
            file,
            &json!({
                "oldSifs":old_result.external_sifs,"oldTrust":old_result.trust_records,
                "admittedSifs":state.resolution.external_sifs,
                "admittedTrust":state.resolution.external_sif_trust_records,
                "oldStampCurrent":state.tide_ledger.is_current(&old_result.stamp),
                "hasQueuedDemand":state.tide_sif_lane.has_demand(),
                "editor":state.document(&tokens_uri),"diskText":std::fs::read_to_string(&tokens_path)?,
            }),
        )?;
    }
    assert!(
        !state.tide_ledger.is_current(&old_result.stamp),
        "watched admitted disk content must stale the actual in-flight SIF stamp"
    );
    assert!(state.tide_sif_lane.has_demand());
    assert!(!apply_deferred_external_sif_refresh_result(
        &mut state, old_result
    ));
    assert_eq!(state.resolution.external_sifs, before_watch);
    let fresh_job =
        prepare_deferred_external_sif_refresh_job(&mut state).ok_or("fresh SIF demand retained")?;
    let fresh_result = collect_deferred_external_sif_refresh(fresh_job);
    assert!(apply_deferred_external_sif_refresh_result(
        &mut state,
        fresh_result
    ));
    assert_eq!(
        state
            .document(&tokens_uri)
            .ok_or("open tokens expected")?
            .text,
        purple,
        "watched disk refresh must preserve the unsaved owner buffer"
    );
    assert_eq!(
        state
            .document(&tokens_uri)
            .ok_or("open tokens expected")?
            .version,
        3
    );
    observe_import(
        &mut state,
        root,
        &style_uri,
        "deferred-disk-differs-from-buffer",
    )?;

    // Existing aliases remain attributed to the previous admitted target even
    // after deletion. Unknown spellings never gain identity from this lookup.
    notify(
        &mut state,
        "textDocument/didOpen",
        json!({"textDocument":{
        "uri":unknown_alias,"languageId":"scss","version":3,"text":purple}}),
    );
    let known_id = state
        .known_document_file_id(&tokens_uri)
        .ok_or("known target expected")?;
    assert_eq!(state.known_document_file_id(&unknown_alias), Some(known_id));
    std::fs::remove_file(&tokens_path)?;
    assert_eq!(state.known_document_file_id(&tokens_uri), Some(known_id));
    assert!(external_sif_source_uri_is_dependency(&state, &tokens_uri));
    assert!(external_sif_source_uri_is_dependency(
        &state,
        &unknown_alias
    ));
    notify(
        &mut state,
        "workspace/didChangeWatchedFiles",
        json!({"changes":[{"uri":unknown_alias,"type":3}]}),
    );
    assert!(state.tide_sif_lane.has_demand());
    assert_eq!(
        state
            .document(&tokens_uri)
            .ok_or("open tokens expected")?
            .text,
        purple
    );
    let deletion = prepare_deferred_external_sif_refresh_job(&mut state)
        .ok_or("deletion SIF demand expected")?;
    assert!(apply_deferred_external_sif_refresh_result(
        &mut state,
        collect_deferred_external_sif_refresh(deletion)
    ));
    assert!(
        state
            .resolution
            .external_sifs
            .iter()
            .all(|input| input.sif.canonical_url != tokens_uri)
    );
    observe_import(
        &mut state,
        root,
        &style_uri,
        "deleted-disk-preserves-buffer",
    )?;
    Ok(())
}

struct BoundSifWatchFixture(std::path::PathBuf);
impl BoundSifWatchFixture {
    fn new(label: &str) -> Result<Self, std::io::Error> {
        let root = std::env::temp_dir().join(format!(
            "omena-sif-watch-{label}-{}-{}",
            std::process::id(),
            current_time_millis()
        ));
        std::fs::create_dir_all(&root)?;
        Ok(Self(root))
    }
}
impl Drop for BoundSifWatchFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn sif_watch_notify(state: &mut LspShellState, method: &str, params: Value) {
    handle_lsp_message(
        state,
        json!({"jsonrpc":"2.0","method":method,"params":params}),
    );
}

fn observe_bound_sif_watch_import(
    state: &mut LspShellState,
    root: &std::path::Path,
    style_uri: &str,
    label: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let root_uri = path_to_file_uri(root);
    let exported = export_bound_snapshot(state, &root_uri)?;
    let binding = exported_binding(&exported)?;
    let imported = omena_query::OmenaSdkWorkspaceV0::open_imported_snapshot(
        omena_query::OmenaSdkSnapshotRequestV0 {
            workspace_root: root_uri.clone(),
        },
        serde_json::from_value(exported["response"]["styleSources"].clone())?,
        serde_json::from_value(exported["response"]["snapshotInputs"].clone())?,
        binding.clone(),
        &omena_query::load_omena_query_workspace_utility_class_intelligence(root, None),
    );
    let request = json!({"snapshotId":binding.snapshot_id(),"stylePath":style_uri,
        "styleSource":state.document(style_uri).ok_or("style expected")?.text});
    let owner = handle_lsp_message(
        state,
        json!({"jsonrpc":"2.0","id":93,
        "method":SDK_WORKFLOW_REQUEST,"params":{"contractVersion":"1","workspaceRoot":root_uri,
        "snapshotBinding":binding,"operation":"diagnostics","request":request}}),
    )
    .ok_or("owner response expected")?;
    let receiver = imported
        .as_ref()
        .ok()
        .map(|workspace| {
            workspace
                .execute_diagnostics(
                    serde_json::from_value(request.clone()).expect("valid request"),
                )
                .map(|payload| serde_json::to_value(payload).expect("serializable payload"))
        })
        .transpose()?;
    let (selected, selected_trust, selected_edges) =
        omena_query::select_omena_workspace_snapshot_external_sifs_v0(
            &serde_json::from_value::<Vec<omena_query::OmenaQueryStyleSourceInputV0>>(
                exported["response"]["styleSources"].clone(),
            )?,
            &serde_json::from_value(
                exported["response"]["snapshotInputs"]["resolutionInputs"].clone(),
            )?,
            &state.resolution.external_sifs,
            &state.resolution.external_sif_trust_records,
            &state.resolution.external_sif_resolution_edges,
        )?;
    if let Some(directory) = std::env::var_os("OMENA_WORKSPACE_SNAPSHOT_TEST_RECEIPTS") {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(std::path::PathBuf::from(directory).join(format!("sif-target-{label}.json")))?;
        serde_json::to_writer_pretty(
            file,
            &json!({"export":exported,"ownerSifs":state.resolution.external_sifs,
            "ownerTrust":state.resolution.external_sif_trust_records,
            "bridgeTargets":state.resolution.bridge_external_sif_urls,"selectedSifs":selected,
            "selectedTrust":selected_trust,"selectedEdges":selected_edges,
            "ownerEdges":state.resolution.external_sif_resolution_edges,"ownerPayload":owner,"receiverPayload":receiver,
            "importError":imported.as_ref().err().map(ToString::to_string),"state":state.snapshot()}),
        )?;
    }
    assert!(
        imported.is_ok(),
        "watched SIF resolution target must reconstruct: {label}: {:?}",
        imported.as_ref().err()
    );
    assert_eq!(
        owner["result"]["response"],
        receiver.ok_or("receiver expected")?
    );
    Ok(exported)
}

#[test]
fn bound_sdk_watched_package_sif_keeps_backing_file_invalidation_dependency() -> TestResult {
    use crate::external_sif_loader::*;
    let fixture = BoundSifWatchFixture::new("package")?;
    let root = &fixture.0;
    let package = root.join("node_modules/@design/tokens");
    std::fs::create_dir_all(&package)?;
    let target = package.join("_index.scss");
    let target_uri = path_to_file_uri(&target);
    let manifest = package.join("package.json");
    std::fs::write(
        &manifest,
        r#"{"name":"@design/tokens","version":"1.0.0","sass":"./_index.scss"}"#,
    )?;
    std::fs::write(&target, "$brand: blue !default;\n")?;
    let app = root.join("App.module.scss");
    let app_uri = path_to_file_uri(&app);
    let text = "@use 'pkg:@design/tokens' as tokens;\n.card { color: tokens.$brand; }\n";
    std::fs::write(&app, text)?;
    let root_uri = path_to_file_uri(root);
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({"jsonrpc":"2.0","id":1,"method":"initialize",
        "params":{"workspaceFolders":[{"uri":root_uri,"name":"package"}]}}),
    );
    sif_watch_notify(
        &mut state,
        "workspace/didChangeConfiguration",
        json!({"settings":{"omena":{
        "resolution":{"packageManifestPaths":[manifest.to_string_lossy()]}}}}),
    );
    for (uri, source) in [(&app_uri, text), (&target_uri, "$brand: blue !default;\n")] {
        sif_watch_notify(
            &mut state,
            "textDocument/didOpen",
            json!({"textDocument":{
            "uri":uri,"languageId":"scss","version":1,"text":source}}),
        );
    }
    refresh_external_sifs_for_state(&mut state);
    assert!(
        state
            .resolution
            .external_sifs
            .iter()
            .any(|input| input.sif.canonical_url == "pkg:@design/tokens")
    );
    assert!(state.known_document_file_id("pkg:@design/tokens").is_none());
    assert!(
        state
            .resolution
            .bridge_external_sif_urls
            .contains(&target_uri)
    );
    assert!(external_sif_source_uri_is_dependency(&state, &target_uri));
    let before = observe_bound_sif_watch_import(&mut state, root, &app_uri, "package-before")?;
    enable_deferred_external_sif_refresh(&mut state);
    let old = collect_deferred_external_sif_refresh(
        prepare_deferred_external_sif_refresh_job(&mut state).ok_or("old package job expected")?,
    );
    std::fs::write(&target, "$brand: green !default;\n")?;
    sif_watch_notify(
        &mut state,
        "workspace/didChangeWatchedFiles",
        json!({"changes":[{"uri":target_uri,"type":2}]}),
    );
    assert!(
        !state.tide_ledger.is_current(&old.stamp),
        "package backing-file watch must stale in-flight SIF work"
    );
    assert!(!apply_deferred_external_sif_refresh_result(&mut state, old));
    let next = collect_deferred_external_sif_refresh(
        prepare_deferred_external_sif_refresh_job(&mut state)
            .ok_or("current package job expected")?,
    );
    assert!(apply_deferred_external_sif_refresh_result(&mut state, next));
    let after = observe_bound_sif_watch_import(&mut state, root, &app_uri, "package-after")?;
    assert_ne!(
        exported_binding(&before)?.input_commitment(),
        exported_binding(&after)?.input_commitment()
    );
    assert_eq!(
        state
            .document(&target_uri)
            .ok_or("open package buffer expected")?
            .text,
        "$brand: blue !default;\n"
    );
    Ok(())
}

#[test]
fn bound_sdk_watched_relative_sif_target_recreation_restores_admission() -> TestResult {
    use crate::external_sif_loader::refresh_external_sifs_for_state;
    let fixture = BoundSifWatchFixture::new("recreation")?;
    let root = &fixture.0;
    let target = root.join("_tokens.scss");
    let target_uri = path_to_file_uri(&target);
    let app = root.join("App.module.scss");
    let app_uri = path_to_file_uri(&app);
    let text = "@use './tokens' as tokens;\n.card { color: tokens.$brand; }\n";
    std::fs::write(&target, "$brand: blue !default;\n")?;
    std::fs::write(&app, text)?;
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({"jsonrpc":"2.0","id":1,"method":"initialize",
        "params":{"workspaceFolders":[{"uri":path_to_file_uri(root),"name":"recreation"}]}}),
    );
    for (uri, source) in [(&app_uri, text), (&target_uri, "$brand: blue !default;\n")] {
        sif_watch_notify(
            &mut state,
            "textDocument/didOpen",
            json!({"textDocument":{
            "uri":uri,"languageId":"scss","version":1,"text":source}}),
        );
    }
    refresh_external_sifs_for_state(&mut state);
    observe_bound_sif_watch_import(&mut state, root, &app_uri, "relative-before")?;
    std::fs::remove_file(&target)?;
    sif_watch_notify(
        &mut state,
        "workspace/didChangeWatchedFiles",
        json!({"changes":[{"uri":target_uri,"type":3}]}),
    );
    assert!(state.resolution.external_sifs.is_empty());
    observe_bound_sif_watch_import(&mut state, root, &app_uri, "relative-deleted")?;
    std::fs::write(&target, "$brand: green !default;\n")?;
    sif_watch_notify(
        &mut state,
        "workspace/didChangeWatchedFiles",
        json!({"changes":[{"uri":target_uri,"type":1}]}),
    );
    observe_bound_sif_watch_import(&mut state, root, &app_uri, "relative-recreated")?;
    assert!(
        state
            .resolution
            .external_sifs
            .iter()
            .any(|input| input.sif.canonical_url == target_uri),
        "recreated relative target must restore independently admitted SIF facts"
    );
    assert_eq!(
        state
            .document(&target_uri)
            .ok_or("open buffer retained")?
            .text,
        "$brand: blue !default;\n"
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn bound_sdk_watched_symlink_retarget_preserves_resolved_semantic_commitment() -> TestResult {
    use crate::external_sif_loader::refresh_external_sifs_for_state;
    let fixture = BoundSifWatchFixture::new("retarget")?;
    let root = fixture.0.join("workspace");
    std::fs::create_dir_all(&root)?;
    let first = fixture.0.join("first.scss");
    let second = fixture.0.join("second.scss");
    std::fs::write(&first, "$brand: blue !default;\n")?;
    std::fs::write(&second, "$brand: green !default;\n")?;
    let alias = root.join("_tokens.scss");
    std::os::unix::fs::symlink(&first, &alias)?;
    let root_uri = path_to_file_uri(&root);
    let alias_uri = format!("{root_uri}/_tokens.scss");
    let app = root.join("App.module.scss");
    let app_uri = path_to_file_uri(&app);
    let text = "@use './tokens' as tokens;\n.card { color: tokens.$brand; }\n";
    std::fs::write(&app, text)?;
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({"jsonrpc":"2.0","id":1,"method":"initialize",
        "params":{"workspaceFolders":[{"uri":root_uri,"name":"retarget"}]}}),
    );
    for (uri, source) in [(&app_uri, text), (&alias_uri, "$brand: blue !default;\n")] {
        sif_watch_notify(
            &mut state,
            "textDocument/didOpen",
            json!({"textDocument":{
            "uri":uri,"languageId":"scss","version":1,"text":source}}),
        );
    }
    refresh_external_sifs_for_state(&mut state);
    let before = observe_bound_sif_watch_import(&mut state, &root, &app_uri, "retarget-before")?;
    let known = state
        .known_document_file_id(&alias_uri)
        .ok_or("owner admitted alias expected")?;
    std::fs::remove_file(&alias)?;
    std::os::unix::fs::symlink(&second, &alias)?;
    assert_eq!(state.known_document_file_id(&alias_uri), Some(known));
    sif_watch_notify(
        &mut state,
        "workspace/didChangeWatchedFiles",
        json!({"changes":[{"uri":alias_uri,"type":2}]}),
    );
    let after = observe_bound_sif_watch_import(&mut state, &root, &app_uri, "retarget-after")?;
    assert!(
        state
            .resolution
            .external_sifs
            .iter()
            .any(|input| input.sif.canonical_url == path_to_file_uri(&second))
    );
    assert_ne!(
        exported_binding(&before)?.input_commitment(),
        exported_binding(&after)?.input_commitment(),
        "resolved symlink SIF target change must remain in the snapshot commitment"
    );
    assert_eq!(
        state
            .document(&alias_uri)
            .ok_or("editor alias retained")?
            .text,
        "$brand: blue !default;\n"
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn bound_sdk_contextual_sif_swap_binds_mapping_with_identical_target_set() -> TestResult {
    use crate::external_sif_loader::*;
    let fixture = BoundSifWatchFixture::new("context-swap")?;
    let root = fixture.0.join("workspace");
    let first = fixture.0.join("first.scss");
    let second = fixture.0.join("second.scss");
    std::fs::write(&first, "$first: blue !default;\n")?;
    std::fs::write(&second, "$second: green !default;\n")?;
    let root_uri = path_to_file_uri(&root);
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({"jsonrpc":"2.0","id":1,"method":"initialize",
        "params":{"workspaceFolders":[{"uri":root_uri,"name":"context-swap"}]}}),
    );
    let mut apps = Vec::new();
    let mut aliases = Vec::new();
    for (directory, target, symbol) in [("a", &first, "first"), ("b", &second, "second")] {
        let text = format!("@use './tokens' as tokens;\n.card {{ color: tokens.${symbol}; }}\n");
        let directory = root.join(directory);
        std::fs::create_dir_all(&directory)?;
        let alias = directory.join("_tokens.scss");
        std::os::unix::fs::symlink(target, &alias)?;
        let app = directory.join("App.module.scss");
        std::fs::write(&app, &text)?;
        let uri = path_to_file_uri(&app);
        sif_watch_notify(
            &mut state,
            "textDocument/didOpen",
            json!({"textDocument":{
            "uri":uri,"languageId":"scss","version":1,"text":text}}),
        );
        apps.push(uri);
        aliases.push(alias);
    }
    refresh_external_sifs_for_state(&mut state);
    let before = observe_bound_sif_watch_import(&mut state, &root, &apps[0], "swap-before-a")?;
    observe_bound_sif_watch_import(&mut state, &root, &apps[1], "swap-before-b")?;
    let target_set =
        |sifs: &[omena_query::OmenaQueryExternalSifInputV0]| -> Result<_, serde_json::Error> {
            sifs.iter()
                .map(serde_json::to_string)
                .collect::<Result<std::collections::BTreeSet<_>, _>>()
        };
    fn assert_semantic_target(
        state: &LspShellState,
        exported: &Value,
        app: &str,
        expected_target: &str,
        label: &str,
    ) -> TestResult {
        let styles: Vec<omena_query::OmenaQueryStyleSourceInputV0> =
            serde_json::from_value(exported["response"]["styleSources"].clone())?;
        let resolution = serde_json::from_value(
            exported["response"]["snapshotInputs"]["resolutionInputs"].clone(),
        )?;
        let (selected, _, _) = omena_query::select_omena_workspace_snapshot_external_sifs_v0(
            &styles,
            &resolution,
            &state.resolution.external_sifs,
            &state.resolution.external_sif_trust_records,
            &state.resolution.external_sif_resolution_edges,
        )?;
        let query = |sifs: &[omena_query::OmenaQueryExternalSifInputV0]| {
            omena_query::summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs(
                app, &styles, &[], &[], None, omena_query::OmenaQueryExternalModuleModeV0::Sif, sifs, &resolution,
            ).expect("actual consumer style exists")
        };
        let mut expected = state
            .resolution
            .external_sifs
            .iter()
            .find(|input| input.sif.canonical_url == expected_target)
            .ok_or("expected admitted target")?
            .clone();
        expected.canonical_url = "./tokens".to_string();
        let actual_bytes = serde_json::to_string(&query(&selected))?;
        let expected_bytes = serde_json::to_string(&query(&[expected]))?;
        if let Some(directory) = std::env::var_os("OMENA_WORKSPACE_SNAPSHOT_TEST_RECEIPTS") {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(
                    std::path::PathBuf::from(directory)
                        .join(format!("context-semantic-{label}.json")),
                )?;
            serde_json::to_writer_pretty(
                file,
                &json!({"stylePath":app,"expectedTarget":expected_target,"actualFullPayloadBytes":actual_bytes,"expectedFullPayloadBytes":expected_bytes,"selectedSifs":selected}),
            )?;
        }
        if label.starts_with("before") {
            let value: Value = serde_json::from_str(&actual_bytes)?;
            assert!(
                value["diagnostics"]
                    .as_array()
                    .ok_or("diagnostics array")?
                    .iter()
                    .all(|diagnostic| {
                        !matches!(
                            diagnostic["code"].as_str(),
                            Some("missing-module" | "missingSassSymbol")
                        )
                    }),
                "valid contextual consumers must have their intended module and symbol: {label}"
            );
        }
        assert_eq!(
            actual_bytes, expected_bytes,
            "actual contextual SIF consumer must resolve the intended target: {label}"
        );
        Ok(())
    }
    assert_semantic_target(
        &state,
        &before,
        &apps[0],
        &path_to_file_uri(&first),
        "before-a",
    )?;
    assert_semantic_target(
        &state,
        &before,
        &apps[1],
        &path_to_file_uri(&second),
        "before-b",
    )?;
    let prior_targets = target_set(&state.resolution.external_sifs)?;
    let prior_trust = state.resolution.external_sif_trust_records.clone();
    let prior_edges = state.resolution.external_sif_resolution_edges.clone();
    for (alias, target) in [(&aliases[0], &second), (&aliases[1], &first)] {
        std::fs::remove_file(alias)?;
        std::os::unix::fs::symlink(target, alias)?;
    }
    // A pure export must retain already-admitted provenance, even after disk changes.
    let unadmitted = export_bound_snapshot(&mut state, &root_uri)?;
    assert_eq!(exported_binding(&before)?, exported_binding(&unadmitted)?);
    assert!(
        omena_query::OmenaSdkWorkspaceV0::open_imported_snapshot(
            omena_query::OmenaSdkSnapshotRequestV0 {
                workspace_root: root_uri.clone()
            },
            serde_json::from_value(unadmitted["response"]["styleSources"].clone())?,
            serde_json::from_value(unadmitted["response"]["snapshotInputs"].clone())?,
            exported_binding(&unadmitted)?,
            &omena_query::load_omena_query_workspace_utility_class_intelligence(&root, None),
        )
        .is_err(),
        "independent receiver must reject the old effective mapping after retarget"
    );
    enable_deferred_external_sif_refresh(&mut state);
    // The existing confirmed backing dependency event clears resolver identity
    // caches and requests the real owner admission. Unknown aliases are not invented.
    sif_watch_notify(
        &mut state,
        "workspace/didChangeWatchedFiles",
        json!({"changes":[{
        "uri":path_to_file_uri(&first),"type":2}]}),
    );
    let pending_binding = exported_binding(&export_bound_snapshot(&mut state, &root_uri)?)?;
    let reader = state
        .sdk_snapshot_publishers
        .borrow()
        .get(&root_uri)
        .ok_or("owner expected")?
        .reader();
    let next = collect_deferred_external_sif_refresh(
        prepare_deferred_external_sif_refresh_job(&mut state).ok_or("mapping refresh expected")?,
    );
    assert_eq!(
        prior_targets,
        target_set(&next.external_sifs)?,
        "both actual full target sets must stay identical"
    );
    assert_eq!(
        prior_trust,
        external_sif_trust_record_map(next.trust_records.clone())
    );
    assert_ne!(
        prior_edges, next.resolution_edges,
        "the actual importer to target map must change"
    );
    assert!(apply_deferred_external_sif_refresh_result(&mut state, next));
    assert!(
        reader
            .with_current_binding(&pending_binding, || ())
            .is_err(),
        "accepted mapping delta must revoke the actual current owner"
    );
    assert!(
        state.tide_republish_lane.has_demand(),
        "mapping delta must owe diagnostics"
    );
    let after = observe_bound_sif_watch_import(&mut state, &root, &apps[0], "swap-after-a")?;
    observe_bound_sif_watch_import(&mut state, &root, &apps[1], "swap-after-b")?;
    assert_semantic_target(
        &state,
        &after,
        &apps[0],
        &path_to_file_uri(&second),
        "after-a",
    )?;
    assert_semantic_target(
        &state,
        &after,
        &apps[1],
        &path_to_file_uri(&first),
        "after-b",
    )?;
    assert_eq!(
        before["response"]["styleSources"],
        after["response"]["styleSources"]
    );
    assert_ne!(
        exported_binding(&before)?.input_commitment(),
        exported_binding(&after)?.input_commitment(),
        "same full SIF/trust set with swapped effective resolution must change commitment"
    );
    Ok(())
}

#[test]
fn bound_sdk_contextual_relative_import_topology_schedules_admission() -> TestResult {
    use crate::external_sif_loader::*;
    let fixture = BoundSifWatchFixture::new("relative-topology")?;
    let root = &fixture.0;
    std::fs::write(root.join("_first.scss"), "$brand: blue;\n")?;
    std::fs::write(root.join("_second.scss"), "$brand: green;\n")?;
    let app = root.join("App.module.scss");
    let uri = path_to_file_uri(&app);
    let before = "@use './first' as tokens; .card { color: tokens.$brand; }";
    std::fs::write(&app, before)?;
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({"jsonrpc":"2.0","id":1,"method":"initialize",
        "params":{"workspaceFolders":[{"uri":path_to_file_uri(root),"name":"topology"}]}}),
    );
    sif_watch_notify(
        &mut state,
        "textDocument/didOpen",
        json!({"textDocument":{
        "uri":uri,"languageId":"scss","version":1,"text":before}}),
    );
    refresh_external_sifs_for_state(&mut state);
    enable_deferred_external_sif_refresh(&mut state);
    let current = collect_deferred_external_sif_refresh(
        prepare_deferred_external_sif_refresh_job(&mut state).ok_or("initial job")?,
    );
    apply_deferred_external_sif_refresh_result(&mut state, current);
    let generation_count = state.external_sif_bridge_generation_count;
    let body_only = "@use './first' as tokens; .card { color: tokens.$brand; padding: 2px; }";
    sif_watch_notify(
        &mut state,
        "textDocument/didChange",
        json!({"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":body_only}]}),
    );
    assert!(
        !state.tide_sif_lane.has_demand(),
        "unchanged relative topology must retain the cutoff"
    );
    assert_eq!(state.external_sif_bridge_generation_count, generation_count);
    let next = "@use './second' as tokens; .card { color: tokens.$brand; }";
    sif_watch_notify(
        &mut state,
        "textDocument/didChange",
        json!({"textDocument":{"uri":uri,"version":3},"contentChanges":[{"text":next}]}),
    );
    assert!(
        state.tide_sif_lane.has_demand(),
        "changed regular relative import must request admission"
    );
    let current = collect_deferred_external_sif_refresh(
        prepare_deferred_external_sif_refresh_job(&mut state).ok_or("changed topology job")?,
    );
    assert!(apply_deferred_external_sif_refresh_result(
        &mut state, current
    ));
    observe_bound_sif_watch_import(&mut state, root, &uri, "relative-topology-after")?;
    assert!(
        state
            .resolution
            .external_sif_resolution_edges
            .iter()
            .any(|edge| edge.specifier == "./second")
    );
    assert!(
        !state
            .resolution
            .external_sif_resolution_edges
            .iter()
            .any(|edge| edge.specifier == "./first")
    );
    assert_eq!(state.document(&uri).ok_or("editor retained")?.text, next);
    Ok(())
}

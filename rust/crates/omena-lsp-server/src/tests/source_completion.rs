use super::*;

#[test]
fn completes_utility_classes_from_workspace_config_through_provider_plane() -> TestResult {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-lsp-utility-completion-{}-{}",
        std::process::id(),
        current_time_millis()
    ));
    let src_dir = workspace_path.join("src");
    fs::create_dir_all(src_dir.as_path())?;
    fs::write(
        workspace_path.join("tailwind.config.ts"),
        r##"export default {
          safelist: ["bg-brand"],
          theme: { extend: { colors: { brand: "#123" } } }
        }"##,
    )?;
    let source_path = src_dir.join("App.tsx");
    let source_text = r#"export const App = () => <div className="bg-" />;"#;
    let source_uri = path_to_file_uri(source_path.as_path());
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "workspaceFolders": [{
                "uri": path_to_file_uri(workspace_path.as_path()),
                "name": "workspace",
            }] },
        }),
    );
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": source_uri,
                "languageId": "typescriptreact",
                "version": 1,
                "text": source_text,
            } },
        }),
    );

    let response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/completion",
            "params": {
                "textDocument": { "uri": path_to_file_uri(source_path.as_path()) },
                "position": parser_position_for_byte_offset(
                    source_text,
                    fixture_find(source_text, "bg-\"", "source fixture contains prefix")? + 3,
                ),
            },
        }),
    );
    let items = response
        .as_ref()
        .and_then(|value| value.pointer("/result/items"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("completion response should contain items"))?;
    let labels = items
        .iter()
        .filter_map(|item| item.get("label").and_then(Value::as_str))
        .collect::<Vec<_>>();
    assert!(labels.contains(&"bg-brand"), "labels: {labels:?}");
    assert!(labels.contains(&"bg-[...]"), "labels: {labels:?}");
    assert_eq!(
        labels.iter().filter(|label| **label == "bg-brand").count(),
        1,
        "duplicate config sources must converge to one server-local completion"
    );
    let unique = labels
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        labels.len(),
        unique.len(),
        "provider labels must be deduplicated"
    );

    let _ = fs::remove_dir_all(workspace_path);
    Ok(())
}

#[test]
fn narrows_source_completion_candidates_by_property_access_prefix() -> TestResult {
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
                        "uri": "file:///workspace-a",
                        "name": "workspace-a",
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
                    "uri": "file:///workspace-a/src/App.tsx",
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": "import styles from \"./App.module.scss\";\nconst view = styles.ro;",
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
                    "uri": "file:///workspace-a/src/App.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": "@use \"./theme\";\n.root { display: block; }\n.row { display: flex; }\n.active { color: red; }",
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
                    "uri": "file:///workspace-a/src/_theme.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": ".root { display: grid; }",
                },
            },
        }),
    );

    let completion_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.tsx",
                },
                "position": {
                    "line": 1,
                    "character": 22,
                },
            },
        }),
    );

    let items = completion_response
        .as_ref()
        .and_then(|value| value.pointer("/result/items"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("completion response should contain items"))?;
    let labels: Vec<String> = items
        .iter()
        .filter_map(|item| {
            item.get("label")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .collect();
    assert_eq!(labels, vec!["root".to_string(), "row".to_string()]);
    assert_eq!(
        items
            .first()
            .and_then(|item| item.pointer("/data/rankingSource")),
        Some(&json!("targetAndPrefixNarrowing")),
    );
    let root_documentation = items
        .first()
        .and_then(|item| item.pointer("/documentation/value"))
        .and_then(Value::as_str)
        .ok_or_else(|| std::io::Error::other("root completion should carry documentation"))?;
    assert!(
        root_documentation.contains("Cascade narrowed values:"),
        "{root_documentation}"
    );
    assert!(
        root_documentation.contains("- `display`: `block`"),
        "{root_documentation}"
    );
    assert!(
        root_documentation.contains("`grid`"),
        "reachable imported module value should participate: {root_documentation}"
    );
    assert!(
        items
            .first()
            .and_then(|item| item.get("sortText"))
            .and_then(Value::as_str)
            .is_some_and(|sort_text| sort_text.starts_with("10-00-00-"))
    );
    Ok(())
}

#[test]
fn completes_variant_recipe_options_from_class_value_universe() -> TestResult {
    let source_uri = "file:///workspace-a/src/App.tsx";
    let source_text = r#"import { cva } from "class-variance-authority";
const button = cva("btn", {
  variants: {
    intent: {
      primary: "btn-primary",
      secondary: "btn-secondary",
    },
  },
});
const view = button({ intent: "pri" });
"#;
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
                        "uri": "file:///workspace-a",
                        "name": "workspace-a",
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

    let completion_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": parser_position_for_byte_offset(
                    source_text,
                    fixture_find(source_text, "pri\"", "source fixture contains partial option")? + 3,
                ),
            },
        }),
    );
    let items = completion_response
        .as_ref()
        .and_then(|value| value.pointer("/result/items"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("completion response should contain items"))?;
    let labels = items
        .iter()
        .filter_map(|item| item.get("label").and_then(Value::as_str))
        .collect::<Vec<_>>();

    assert_eq!(labels, vec!["primary"]);
    assert_eq!(
        items
            .first()
            .and_then(|item| item.pointer("/data/rankingSource")),
        Some(&json!("classValueUniverseProvider"))
    );
    Ok(())
}

#[test]
fn ranks_source_completion_with_value_domain_projection() -> TestResult {
    let source_uri = "file:///workspace-a/src/App.tsx";
    let style_uri = "file:///workspace-a/src/App.module.scss";
    let source_text = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
const view = <div className={cx(`item--${variant}`)} />;
"#;
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
                        "uri": "file:///workspace-a",
                        "name": "workspace-a",
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
                    "text": ".item--large {}\n.item--primary {}\n.item--secondary {}\n.item--muted {}\n",
                },
            },
        }),
    );

    let expression_id = state
        .document(source_uri)
        .and_then(|document| document.source_syntax_index.type_fact_targets.first())
        .map(|target| target.expression_id.clone())
        .ok_or_else(|| std::io::Error::other("expected a source type-fact target"))?;
    apply_source_type_fact_results_to_document(
        &mut state,
        source_uri,
        &[TsgoTypeFactResultEntryV0 {
            file_path: "/workspace-a/src/App.tsx".to_string(),
            expression_id,
            resolved_type: TsgoResolvedTypeV0 {
                kind: "union",
                values: vec!["primary".to_string(), "secondary".to_string()],
            },
        }],
    );

    let position = parser_position_for_byte_offset(
        source_text,
        fixture_find(
            source_text,
            "variant}`",
            "source fixture contains dynamic selector expression",
        )?,
    );
    let completion_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": source_uri,
                },
                "position": position,
            },
        }),
    );

    let items = completion_response
        .as_ref()
        .and_then(|value| value.pointer("/result/items"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("completion response should contain items"))?;
    let labels = items
        .iter()
        .filter_map(|item| item.get("label").and_then(Value::as_str))
        .collect::<Vec<_>>();
    assert_eq!(
        labels,
        vec![
            "item--primary",
            "item--secondary",
            "item--large",
            "item--muted"
        ]
    );
    assert_eq!(
        items
            .first()
            .and_then(|item| item.pointer("/data/rankingSource")),
        Some(&json!("valueDomainSelectorProjection")),
    );
    assert!(
        items
            .first()
            .and_then(|item| item.get("sortText"))
            .and_then(Value::as_str)
            .is_some_and(|sort_text| sort_text.starts_with("00-00-"))
    );
    Ok(())
}

use super::*;

#[test]
fn resolves_unnecessary_tags_for_cascade_style_diagnostics() -> TestResult {
    let mut state = LspShellState::default();
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
                    "text": ":root { --brand: red; }\n.btn { color: red; color: blue; }",
                },
            },
        }),
    );

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
        .ok_or_else(|| std::io::Error::other("style diagnostics result"))?;
    let unreachable = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.pointer("/code") == Some(&json!("unreachableDeclaration")))
        .ok_or_else(|| std::io::Error::other("unreachable declaration diagnostic"))?;
    assert_eq!(unreachable.pointer("/severity"), Some(&json!(4)));
    assert_eq!(unreachable.pointer("/tags"), Some(&json!([1])));
    // Since the checker orchestrator boundary split (8e632aa2), the product
    // cascade diagnostics carry the upstream orchestrator-gate product as the
    // most-upstream provenance label, followed by the owning checker rules.
    assert_eq!(
        unreachable.pointer("/data/provenance/0"),
        Some(&json!("omena-query-checker-orchestrator.cascade-gate")),
    );
    assert_eq!(
        unreachable.pointer("/data/provenance/1"),
        Some(&json!("omena-checker.cascade-rules")),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeNarrowing/product"),
        Some(&json!("omena-query.cascade-narrowing-evidence")),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeNarrowing/selector"),
        Some(&json!(".btn")),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeNarrowing/selectorClassNames"),
        Some(&json!(["btn"])),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeNarrowing/propertyName"),
        Some(&json!("color")),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeNarrowing/propertyValueNarrowing/product"),
        Some(&json!("omena-abstract-value.property-value-narrowing")),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeNarrowing/propertyValueNarrowing/matchedCandidateCount"),
        Some(&json!(2)),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeNarrowing/elementClassIteration/product"),
        Some(&json!("omena-abstract-value.reduced-product-iteration")),
    );
    Ok(())
}

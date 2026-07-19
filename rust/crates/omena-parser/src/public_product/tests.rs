use super::*;

const SUMMARY_CONTRACT_SOURCE: &str = r#"@value tone: red;
@forward "tokens" AS token-* show $color;
@mixin spacing($gap: 1rem, $mode: compact) {
  --inside: blue;
}
@function scale($factor: 2) { @return $factor; }
@keyframes fade { from { opacity: 0; } to { opacity: 1; } }
.card {
  --brand: blue;
  animation-name: fade;
  animation: fade 1s;
}
"#;

fn span_text(source: &str, span: ParserByteSpanV0) -> Option<&str> {
    source.get(span.start..span.end)
}

#[test]
fn product_summary_preserves_syntax_derived_values_and_spans() {
    let summary = summarize_css_modules_intermediate(SUMMARY_CONTRACT_SOURCE, StyleDialect::Scss);

    let value = summary
        .values
        .decl_facts
        .iter()
        .find(|fact| fact.name == "tone");
    assert!(value.is_some(), "CSS Modules value definition is missing");
    let Some(value) = value else {
        return;
    };
    assert_eq!(value.value, "red");
    assert_eq!(
        span_text(SUMMARY_CONTRACT_SOURCE, value.rule_byte_span),
        Some("@value tone: red;")
    );

    let forward = summary
        .sass
        .module_forward_edges
        .iter()
        .find(|fact| fact.source == "tokens");
    assert!(forward.is_some(), "Sass forward edge is missing");
    let Some(forward) = forward else {
        return;
    };
    assert_eq!(forward.prefix, "token-");
    assert_eq!(
        span_text(SUMMARY_CONTRACT_SOURCE, forward.rule_byte_span),
        Some("@forward \"tokens\" AS token-* show $color;")
    );

    assert_eq!(
        summary.sass.variable_parameter_names,
        ["factor", "gap", "mode"]
    );
    assert!(["factor", "gap", "mode"].iter().all(|name| {
        !summary
            .sass
            .variable_decl_names
            .iter()
            .any(|item| item == name)
    }));

    let keyframes = summary
        .keyframes
        .decl_facts
        .iter()
        .find(|fact| fact.name == "fade");
    assert!(keyframes.is_some(), "keyframes declaration is missing");
    let Some(keyframes) = keyframes else {
        return;
    };
    assert_eq!(
        span_text(SUMMARY_CONTRACT_SOURCE, keyframes.rule_byte_span),
        Some("@keyframes fade { from { opacity: 0; } to { opacity: 1; } }")
    );

    let custom_property = summary
        .custom_properties
        .decl_facts
        .iter()
        .find(|fact| fact.name == "--brand");
    assert!(
        custom_property.is_some(),
        "custom property declaration is missing"
    );
    let Some(custom_property) = custom_property else {
        return;
    };
    assert_eq!(custom_property.value, "blue");
    assert_eq!(
        span_text(SUMMARY_CONTRACT_SOURCE, custom_property.rule_byte_span),
        Some("\n.card {\n  --brand: blue;\n  animation-name: fade;\n  animation: fade 1s;\n}")
    );

    let animation_properties = summary
        .keyframes
        .ref_facts
        .iter()
        .map(|fact| fact.property)
        .collect::<Vec<_>>();
    assert!(animation_properties.contains(&"animation-name"));
    assert!(animation_properties.contains(&"animation"));
}

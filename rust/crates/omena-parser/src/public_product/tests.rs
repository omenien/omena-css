use super::*;

const SUMMARY_CONTRACT_SOURCE: &str = r#"@value tone: red;
@forward "tokens" as token-* show $color;
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

fn span_text(source: &str, span: ParserByteSpanV0) -> &str {
    source
        .get(span.start..span.end)
        .expect("summary span must stay inside the source")
}

#[test]
fn product_summary_preserves_syntax_derived_values_and_spans() {
    let summary = summarize_css_modules_intermediate(SUMMARY_CONTRACT_SOURCE, StyleDialect::Scss);

    let value = summary
        .values
        .decl_facts
        .iter()
        .find(|fact| fact.name == "tone")
        .expect("CSS Modules value definition must be summarized");
    assert_eq!(value.value, "red");
    assert_eq!(
        span_text(SUMMARY_CONTRACT_SOURCE, value.rule_byte_span),
        "@value tone: red;"
    );

    let forward = summary
        .sass
        .module_forward_edges
        .iter()
        .find(|fact| fact.source == "tokens")
        .expect("Sass forward edge must be summarized");
    assert_eq!(forward.prefix, "token-");
    assert_eq!(
        span_text(SUMMARY_CONTRACT_SOURCE, forward.rule_byte_span),
        "@forward \"tokens\" as token-* show $color;"
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
        .find(|fact| fact.name == "fade")
        .expect("keyframes declaration must be summarized");
    assert_eq!(
        span_text(SUMMARY_CONTRACT_SOURCE, keyframes.rule_byte_span),
        "@keyframes fade { from { opacity: 0; } to { opacity: 1; } }"
    );

    let custom_property = summary
        .custom_properties
        .decl_facts
        .iter()
        .find(|fact| fact.name == "--brand")
        .expect("custom property declaration must be summarized");
    assert_eq!(custom_property.value, "blue");
    assert_eq!(
        span_text(SUMMARY_CONTRACT_SOURCE, custom_property.rule_byte_span),
        "\n.card {\n  --brand: blue;\n  animation-name: fade;\n  animation: fade 1s;\n}"
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

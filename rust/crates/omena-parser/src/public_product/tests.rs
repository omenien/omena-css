use super::*;
use crate::collect_style_facts;

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

fn assert_authored_property_text(
    actual: &omena_syntax::ident::AuthoredPropertyTextV0,
    expected: &str,
) {
    let mut rendered = String::new();
    let result = actual.write_into(&mut rendered);
    assert!(result.is_ok(), "writing into a String must not fail");
    assert_eq!(rendered, expected);
}

#[test]
fn lexical_class_scanner_matches_live_parser_facts() {
    for source in [
        ".card .title { color: red; }",
        "[data-x=\"a.b\"] { color: red; }",
        "[data-x=a.b] { color: red; }",
        r".a\.b { color: red; }",
        r".\31 23 { color: red; }",
        ".카드.café { color: red; }",
    ] {
        let header = source.split_once('{').map_or(source, |(header, _)| header);
        let lexical_names = omena_syntax::ident::class_selector_names(header)
            .into_iter()
            .map(|entry| entry.name.into_raw())
            .collect::<Vec<_>>();
        let parser_names = collect_style_facts(source, StyleDialect::Css)
            .selectors
            .into_iter()
            .filter(|fact| fact.kind == ParsedSelectorFactKind::Class)
            .map(|fact| fact.name)
            .collect::<Vec<_>>();

        // Either live producer can emit a different list for the source above;
        // this comparison fails instead of adapting an authored expectation.
        assert_eq!(lexical_names, parser_names, "source: {source}");
    }
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
        .find(|fact| fact.property_key.as_str() == "--brand");
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

#[test]
fn declaration_syntax_facts_keep_url_delimiters_inside_cst_values() {
    for source in [
        ".a { background-image: url(a;b.png)!important; background-image: url(clean.png); }",
        ".a { background-image: url(a}b.png)!important; background-image: url(clean.png); }",
    ] {
        let facts = collect_parser_declaration_syntax_facts(source, StyleDialect::Css);
        assert_eq!(facts.len(), 2, "{facts:#?}");
        assert_authored_property_text(&facts[0].property_name, "background-image");
        let expected_value = source
            .split_once("background-image: ")
            .and_then(|(_, tail)| tail.split_once("!important"))
            .map(|(value, _)| value);
        assert_eq!(
            span_text(source, facts[0].value_span).map(str::trim),
            expected_value
        );
        assert_eq!(facts[0].value_text, expected_value.unwrap_or_default());
        assert!(
            !span_text(source, facts[0].byte_span)
                .unwrap_or_default()
                .ends_with(';')
        );
        assert!(facts[0].important);
        assert_eq!(facts[0].source_order, 0);
        assert_eq!(facts[0].selector_contexts.len(), 1);
        assert_eq!(facts[0].selector_contexts[0].selector_members, [".a"]);
        assert!(!facts[0].selector_contexts[0].reset_to_root);
        assert_eq!(facts[1].source_order, 1);
        assert!(!facts[1].important);
    }
}

#[test]
fn declaration_syntax_facts_record_nested_selectors_and_cst_conditions() {
    let source = r#"
@media (min-width: 40rem) {
  .card, .panel {
    &:hover { color: red ! /* keep */ important; }
  }
}
"#;
    let facts = collect_parser_declaration_syntax_facts(source, StyleDialect::Scss);
    assert_eq!(facts.len(), 1, "{facts:#?}");
    let fact = &facts[0];
    assert_authored_property_text(&fact.property_name, "color");
    assert_eq!(
        span_text(source, fact.value_span).map(str::trim),
        Some("red")
    );
    assert_eq!(fact.value_text, "red");
    assert!(fact.important);
    assert_eq!(fact.condition_contexts, ["@media (min-width: 40rem)"]);
    assert_eq!(fact.selector_contexts.len(), 2);
    assert_eq!(
        fact.selector_contexts[0].selector_members,
        [".card", ".panel"]
    );
    assert_eq!(fact.selector_contexts[1].selector_members, ["&:hover"]);
}

#[test]
fn declaration_syntax_facts_normalize_comment_trivia_from_property_and_value_text() {
    let source = ".a { color /* property */ : red /* value */; }";
    let facts = collect_parser_declaration_syntax_facts(source, StyleDialect::Css);
    assert_eq!(facts.len(), 1);
    assert_authored_property_text(&facts[0].property_name, "color");
    assert_eq!(facts[0].value_text, "red");
}

#[test]
fn declaration_syntax_facts_preserve_authored_property_names_with_canonical_keys() {
    let source = r#".a { COLOR: red; --FOO: blue; --f\6f o: green; }"#;
    let facts = collect_parser_declaration_syntax_facts(source, StyleDialect::Css);

    assert_eq!(facts.len(), 3, "{facts:#?}");
    assert_authored_property_text(&facts[0].property_name, "COLOR");
    assert_eq!(facts[0].property_key.as_str(), "color");
    assert_authored_property_text(&facts[1].property_name, "--FOO");
    assert_eq!(facts[1].property_key.as_str(), "--FOO");
    assert_authored_property_text(&facts[2].property_name, r"--f\6f o");
    assert_eq!(facts[2].property_key.as_str(), "--foo");
    assert_ne!(facts[1].property_key, facts[2].property_key);
}

#[test]
fn declaration_syntax_fact_identity_uses_sealed_property_keys() -> Result<(), String> {
    let decoded =
        collect_parser_declaration_syntax_facts(r#".a { --foo: red; }"#, StyleDialect::Css)
            .into_iter()
            .next()
            .ok_or_else(|| "decoded declaration fact is required".to_string())?;
    let mut escaped = decoded.clone();
    escaped.property_name = AuthoredPropertyTextV0::new(r"--f\6f o");

    assert_eq!(escaped, decoded);
    Ok(())
}

#[test]
fn parser_custom_property_fact_collection_identity_uses_custom_property_keys() {
    let facts = |property: &str| ParserIndexCustomPropertyFactsV0 {
        decl_names: vec![AuthoredPropertyTextV0::new(property)],
        ref_names: vec![AuthoredPropertyTextV0::new(property)],
        ..ParserIndexCustomPropertyFactsV0::default()
    };

    assert_eq!(facts(r"--f\6f o"), facts("--foo"));
    assert_ne!(facts("--foo"), facts("--FOO"));
}

#[test]
fn parser_custom_property_decl_fact_identity_uses_sealed_keys() {
    let summary = summarize_css_modules_intermediate(r#":root { --foo: red; }"#, StyleDialect::Css);
    assert_eq!(summary.custom_properties.decl_facts.len(), 1);
    let decoded = summary.custom_properties.decl_facts[0].clone();
    let mut escaped = decoded.clone();
    escaped.name = AuthoredPropertyTextV0::new(r"--f\6f o");

    assert_eq!(escaped, decoded);
    assert_eq!(escaped.cmp(&decoded), std::cmp::Ordering::Equal);
}

#[test]
fn parser_custom_property_ref_fact_identity_uses_sealed_keys() {
    let summary =
        summarize_css_modules_intermediate(r#".a { color: var(--foo); }"#, StyleDialect::Css);
    assert_eq!(summary.custom_properties.ref_facts.len(), 1);
    let decoded = summary.custom_properties.ref_facts[0].clone();
    let mut escaped = decoded.clone();
    escaped.name = AuthoredPropertyTextV0::new(r"--f\6f o");

    assert_eq!(escaped, decoded);
    assert_eq!(escaped.cmp(&decoded), std::cmp::Ordering::Equal);
}

#[test]
fn declaration_only_projection_matches_the_full_product_index() {
    let source = r#"
@media (min-width: 40rem) {
  @layer theme {
    .card, .panel { &:hover { color /* property */ : red !important; } }
  }
}
"#;
    let parsed = parse(source, StyleDialect::Scss);
    let full = ProductSyntaxIndexV0::new(source, &parsed);
    let declarations = ProductSyntaxIndexV0::declarations_from_parse(source, &parsed);

    assert_eq!(declarations.as_slice(), full.declarations());
}

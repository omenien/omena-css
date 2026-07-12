use super::*;

#[test]
fn width_budget_changes_selector_list_layout() {
    let source = ".componentAlphaLongerState, .componentBetaLongerState, .componentGammaLongerState, .componentDeltaLongerState, .componentEpsilonLongerState { color:red; }";
    let narrow = render_pretty_css_through_transform_ir(
        source,
        StyleDialect::Css,
        "width-narrow",
        PrettyFormatOptionsV0 {
            line_width: 80,
            indent_width: 2,
        },
    );
    let medium = render_pretty_css_through_transform_ir(
        source,
        StyleDialect::Css,
        "width-medium",
        PrettyFormatOptionsV0 {
            line_width: 100,
            indent_width: 2,
        },
    );
    let wide = render_pretty_css_through_transform_ir(
        source,
        StyleDialect::Css,
        "width-wide",
        PrettyFormatOptionsV0 {
            line_width: 120,
            indent_width: 2,
        },
    );

    assert_ne!(narrow.css, medium.css);
    assert_ne!(medium.css, wide.css);
    assert_ne!(narrow.css, wide.css);
}

#[test]
fn comments_strings_and_custom_property_values_are_preserved() {
    let source = "/* lead */ .card,.panel{--label:\"a,b\";color:var(--brand);background-image:url(https://cdn.example.com/a,b.png);/* tail */}";
    let rendered = render_pretty_css_through_transform_ir(
        source,
        StyleDialect::Css,
        "trivia",
        default_pretty_format_options(),
    );

    assert!(rendered.css.contains("/* lead */"));
    assert!(rendered.css.contains("/* tail */"));
    assert!(rendered.css.contains("\"a,b\""));
    assert!(rendered.css.contains("var(--brand)"));
    assert!(
        rendered
            .css
            .contains("url(https://cdn.example.com/a,b.png)")
    );
}

#[test]
fn scss_line_comments_are_preserved_without_treating_url_schemes_as_comments() {
    let source = ".card { // keep this\n background:url(https://cdn.example.com/card.png); }";
    let rendered = render_pretty_css_through_transform_ir(
        source,
        StyleDialect::Scss,
        "scss-comments",
        default_pretty_format_options(),
    );

    assert!(rendered.css.contains("// keep this"));
    assert!(rendered.css.contains("https://cdn.example.com/card.png"));
}

#[test]
fn indented_sass_and_parse_errors_report_stable_fallback() {
    let sass = render_pretty_css_through_transform_ir(
        ".card\n  color: red\n",
        StyleDialect::Sass,
        "sass",
        default_pretty_format_options(),
    );
    assert_eq!(sass.css, ".card\n  color: red\n");
    assert_eq!(
        sass.report.fallback_reasons,
        vec!["indented-sass-stable-fallback"]
    );

    let invalid = render_pretty_css_through_transform_ir(
        ".card { color: red;",
        StyleDialect::Css,
        "invalid",
        default_pretty_format_options(),
    );
    assert_eq!(invalid.css, ".card { color: red;");
    assert_eq!(
        invalid.report.fallback_reasons,
        vec!["parse-error-stable-fallback"]
    );
}

#[test]
fn coverage_manifest_classifies_every_transform_ir_kind() {
    let labels = FORMAT_IR_COVERAGE_MANIFEST_V0
        .iter()
        .map(|entry| entry.node_kind)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        labels,
        BTreeSet::from([
            "at-rule",
            "declaration",
            "selector",
            "style-rule",
            "url-value",
            "value",
        ])
    );
}

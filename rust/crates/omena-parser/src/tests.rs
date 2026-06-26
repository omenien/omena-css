use cstree::{green::GreenNode, syntax::SyntaxNode};
use std::collections::{BTreeMap, BTreeSet};

use super::*;

#[test]
fn builds_cst_root_for_plain_css() {
    let result = parse(".button { color: red; }", StyleDialect::Css);

    assert_eq!(result.syntax().kind(), SyntaxKind::Root);
    assert_eq!(result.dialect(), StyleDialect::Css);
    assert!(
        result.errors().is_empty(),
        "unexpected parse errors: {:?}",
        result.errors()
    );
    assert!(result.token_count() > 0);

    let kinds = node_kinds(&result.syntax());
    assert!(kinds.contains(&SyntaxKind::Rule));
    assert!(kinds.contains(&SyntaxKind::SelectorList));
    assert!(kinds.contains(&SyntaxKind::DeclarationList));
    assert!(kinds.contains(&SyntaxKind::Declaration));
    assert!(kinds.contains(&SyntaxKind::PropertyName));
    assert!(kinds.contains(&SyntaxKind::Value));
}

#[test]
fn parse_only_matches_parse_tree_entry() {
    let text = ".button { color: red; }";

    let parsed = parse(text, StyleDialect::Css);
    let parse_only_result = parse_only(text, StyleDialect::Css);

    assert_eq!(parse_only_result, parsed);
    assert_eq!(parse_only_result.green(), parsed.green());
    assert_eq!(parse_only_result.syntax().kind(), parsed.syntax().kind());
}

#[test]
fn exposes_css_syntax_parser_entry_points() {
    let rule_list = parse_entry_point(
        ".button { color: red; } @media (width >= 1px) { .card { color: blue; } }",
        StyleDialect::Css,
        ParseEntryPoint::RuleList,
    );
    let rule = parse_entry_point(
        ".button { color: red; }",
        StyleDialect::Css,
        ParseEntryPoint::Rule,
    );
    let declaration_list = parse_entry_point(
        "color: red; width: calc(1px + 2px);",
        StyleDialect::Css,
        ParseEntryPoint::DeclarationList,
    );
    let declaration = parse_entry_point(
        "color: red;",
        StyleDialect::Css,
        ParseEntryPoint::Declaration,
    );
    let value = parse_entry_point(
        "clamp(1rem, calc(2px + 3px), 4rem)",
        StyleDialect::Css,
        ParseEntryPoint::Value,
    );
    let component_value = parse_entry_point(
        "calc(100% - var(--gap))",
        StyleDialect::Css,
        ParseEntryPoint::ComponentValue,
    );
    let component_value_list = parse_entry_point(
        "red + calc(1px + 2px) [data-state]",
        StyleDialect::Css,
        ParseEntryPoint::ComponentValueList,
    );
    let comma_separated_component_value_list = parse_entry_point(
        "red, calc(1px + 2px), [data-state]",
        StyleDialect::Css,
        ParseEntryPoint::CommaSeparatedComponentValueList,
    );
    let simple_block = parse_entry_point(
        "{ color: red; [data-state] }",
        StyleDialect::Css,
        ParseEntryPoint::SimpleBlock,
    );
    let unclosed_simple_block = parse_entry_point(
        "{ color: red",
        StyleDialect::Css,
        ParseEntryPoint::SimpleBlock,
    );

    assert!(rule_list.errors().is_empty());
    assert!(rule.errors().is_empty());
    assert!(declaration_list.errors().is_empty());
    assert!(declaration.errors().is_empty());
    assert!(value.errors().is_empty());
    assert!(component_value.errors().is_empty());
    assert!(component_value_list.errors().is_empty());
    assert!(comma_separated_component_value_list.errors().is_empty());
    assert!(simple_block.errors().is_empty());
    assert_eq!(unclosed_simple_block.errors().len(), 1);
    assert!(node_kinds(&rule_list.syntax()).contains(&SyntaxKind::RuleList));
    assert!(node_kinds(&rule.syntax()).contains(&SyntaxKind::Rule));
    assert!(node_kinds(&declaration_list.syntax()).contains(&SyntaxKind::DeclarationList));
    assert!(node_kinds(&declaration.syntax()).contains(&SyntaxKind::Declaration));
    assert!(node_kinds(&value.syntax()).contains(&SyntaxKind::Value));
    assert!(node_kinds(&value.syntax()).contains(&SyntaxKind::CalcFunction));
    assert!(node_kinds(&component_value.syntax()).contains(&SyntaxKind::ComponentValue));
    assert!(node_kinds(&component_value.syntax()).contains(&SyntaxKind::FunctionCall));
    assert!(node_kinds(&component_value_list.syntax()).contains(&SyntaxKind::ComponentValueList));
    assert!(
        node_kinds(&comma_separated_component_value_list.syntax())
            .contains(&SyntaxKind::CommaSeparatedComponentValueList)
    );
    assert!(node_kinds(&simple_block.syntax()).contains(&SyntaxKind::SimpleBlock));
    assert!(node_kinds(&simple_block.syntax()).contains(&SyntaxKind::ComponentValue));
    assert!(node_kinds(&unclosed_simple_block.syntax()).contains(&SyntaxKind::BogusSimpleBlock));
}

#[test]
fn tokenizes_multibyte_source_without_boundary_errors() {
    let result = parse(".카드 { --간격: \"좋음\"; }", StyleDialect::Css);

    assert!(
        result.errors().is_empty(),
        "unexpected parse errors: {:?}",
        result.errors()
    );
    assert!(result.token_count() >= 8);
}

#[test]
fn facts_from_cst_materializes_syntax_root_once() {
    let source = r#"@use "./tokens" as t;
@mixin tone { color: $brand; }
:export { exported: local; }
:import("./theme.module.css") { imported: theme; }
@value spacing from "./spacing.module.css";
.button { --brand: red; animation: fade 1s; composes: base from "./base.module.css"; }
@keyframes fade { from { opacity: 0; } to { opacity: 1; } }"#;
    let expected = collect_style_facts(source, StyleDialect::Scss);
    let parsed = parse(source, StyleDialect::Scss);

    reset_omena_parser_syntax_root_materialization_count();
    let actual = facts_from_cst(source, &parsed);

    assert_eq!(actual, expected);
    assert_eq!(omena_parser_syntax_root_materialization_count(), 1);
}

#[test]
fn reports_unterminated_constructs_without_panicking() {
    let comment = parse("/* open", StyleDialect::Css);
    let string = parse(".a { content: \"open; }", StyleDialect::Css);
    let block = parse(".a { color: red", StyleDialect::Css);

    assert_eq!(
        comment.errors().first().map(|error| error.code),
        Some(ParseErrorCode::UnterminatedBlockComment),
    );
    assert_eq!(
        string.errors().first().map(|error| error.code),
        Some(ParseErrorCode::UnterminatedString),
    );
    assert_eq!(
        block.errors().first().map(|error| error.code),
        Some(ParseErrorCode::UnexpectedCharacter),
    );
    assert!(node_kinds(&block.syntax()).contains(&SyntaxKind::BogusTrivia));
}

#[test]
fn classifies_initial_dialect_tokens() {
    let scss = parse("$gap: 1rem;", StyleDialect::Scss);
    let less = parse("@gap: 1rem;", StyleDialect::Less);
    let less_at_rule = parse("@media screen {}", StyleDialect::Less);
    let scss_kinds = node_kinds(&scss.syntax());
    let less_kinds = node_kinds(&less.syntax());

    assert_eq!(scss.syntax().kind(), SyntaxKind::Root);
    assert_eq!(less.syntax().kind(), SyntaxKind::Root);
    assert_eq!(less_at_rule.syntax().kind(), SyntaxKind::Root);
    assert!(scss.errors().is_empty());
    assert!(less.errors().is_empty());
    assert!(less_at_rule.errors().is_empty());
    assert!(scss_kinds.contains(&SyntaxKind::ScssVariableDeclaration));
    assert!(less_kinds.contains(&SyntaxKind::LessVariableDeclaration));
}

#[test]
fn exposes_lex_result_for_tokenizer_gates() {
    let scss = lex("$gap: 1rem;", StyleDialect::Scss);
    let less = lex("@gap: 1rem;", StyleDialect::Less);
    let less_at_rule = lex("@media screen {}", StyleDialect::Less);
    let css_slashes = lex("// not a css comment", StyleDialect::Css);
    let scss_slashes = lex("// scss comment", StyleDialect::Scss);

    assert_eq!(
        scss.tokens().first().map(|token| token.kind),
        Some(SyntaxKind::ScssVariable)
    );
    assert_eq!(
        scss.tokens().first().map(|token| token.text.as_str()),
        Some("$gap")
    );
    assert_eq!(
        less.tokens().first().map(|token| token.kind),
        Some(SyntaxKind::LessVariable)
    );
    assert_eq!(
        less_at_rule.tokens().first().map(|token| token.kind),
        Some(SyntaxKind::AtKeyword),
    );
    assert_eq!(
        css_slashes.tokens().first().map(|token| token.kind),
        Some(SyntaxKind::Slash)
    );
    assert_eq!(
        scss_slashes.tokens().first().map(|token| token.kind),
        Some(SyntaxKind::LineComment),
    );
}

#[test]
fn summarizes_parser_lex_as_parser_owned_product() {
    let summary = summarize_omena_parser_lex(".card { color: red; }", StyleDialect::Css);

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-parser.lex-result");
    assert_eq!(summary.dialect, "css");
    assert_eq!(summary.parser_error_count, 0);
    assert!(summary.tokens.iter().any(|token| token.text == "card"));
}

#[test]
fn tokenizes_css_attribute_matchers_as_single_tokens() {
    let result = lex(
        ".a[data-state~=\"active\"][lang|=\"en\"][href^=\"/docs\"][href$=\".pdf\"][class*=\"btn\"] { width += 1px; }",
        StyleDialect::Css,
    );
    let kinds: Vec<SyntaxKind> = result.tokens().iter().map(|token| token.kind).collect();

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::IncludesMatch));
    assert!(kinds.contains(&SyntaxKind::DashMatch));
    assert!(kinds.contains(&SyntaxKind::PrefixMatch));
    assert!(kinds.contains(&SyntaxKind::SuffixMatch));
    assert!(kinds.contains(&SyntaxKind::SubstringMatch));
    assert!(kinds.contains(&SyntaxKind::PlusEquals));
}

#[test]
fn tokenizes_important_annotation_as_single_token() {
    let result = lex(".a { color: red !IMPORTANT; }", StyleDialect::Css);
    let kinds: Vec<SyntaxKind> = result.tokens().iter().map(|token| token.kind).collect();

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::Important));
    assert!(!kinds.contains(&SyntaxKind::Delim));
}

#[test]
fn tokenizes_cdo_cdc_and_ignores_them_at_top_level() {
    let result = parse("<!-- .a { color: red; } -->", StyleDialect::Css);
    let token_kinds = token_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(token_kinds.contains(&SyntaxKind::Cdo));
    assert!(token_kinds.contains(&SyntaxKind::Cdc));
    assert!(node_kinds(&result.syntax()).contains(&SyntaxKind::Rule));
}

#[test]
fn tokenizes_css_identifier_escapes_without_unexpected_errors() {
    let result = parse(".\\31 0 { color: var(--\\67 ap); }", StyleDialect::Css);
    let token_kinds = token_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(token_kinds.contains(&SyntaxKind::Ident));
    assert!(token_kinds.contains(&SyntaxKind::CustomPropertyName));
    assert!(node_kinds(&result.syntax()).contains(&SyntaxKind::ClassSelector));
}

#[test]
fn tokenizes_bare_hash_as_delim_and_hash_names_as_hash() {
    let bare = lex("# { color: red; }", StyleDialect::Css);
    let named = lex("#main { color: red; }", StyleDialect::Css);
    let escaped = lex("#\\31 0 { color: red; }", StyleDialect::Css);
    let bare_kinds: Vec<SyntaxKind> = bare.tokens().iter().map(|token| token.kind).collect();
    let named_kinds: Vec<SyntaxKind> = named.tokens().iter().map(|token| token.kind).collect();
    let escaped_kinds: Vec<SyntaxKind> = escaped.tokens().iter().map(|token| token.kind).collect();

    assert!(bare.errors().is_empty());
    assert!(named.errors().is_empty());
    assert!(escaped.errors().is_empty());
    assert!(bare_kinds.contains(&SyntaxKind::Delim));
    assert!(!bare_kinds.contains(&SyntaxKind::Hash));
    assert!(named_kinds.contains(&SyntaxKind::Hash));
    assert!(escaped_kinds.contains(&SyntaxKind::Hash));
}

#[test]
fn tokenizes_dash_started_idents_and_custom_properties_by_ident_rules() {
    let vendor = lex("-webkit-transform", StyleDialect::Css);
    let custom = lex("--brand", StyleDialect::Css);
    let escaped_custom = lex("--\\31 0", StyleDialect::Css);
    let bare_dash = lex("--:", StyleDialect::Css);
    let vendor_kinds: Vec<SyntaxKind> = vendor.tokens().iter().map(|token| token.kind).collect();
    let custom_kinds: Vec<SyntaxKind> = custom.tokens().iter().map(|token| token.kind).collect();
    let escaped_custom_kinds: Vec<SyntaxKind> = escaped_custom
        .tokens()
        .iter()
        .map(|token| token.kind)
        .collect();
    let bare_dash_kinds: Vec<SyntaxKind> =
        bare_dash.tokens().iter().map(|token| token.kind).collect();

    assert!(vendor.errors().is_empty());
    assert!(custom.errors().is_empty());
    assert!(escaped_custom.errors().is_empty());
    assert!(bare_dash.errors().is_empty());
    assert!(vendor_kinds.contains(&SyntaxKind::Ident));
    assert!(!vendor_kinds.contains(&SyntaxKind::Minus));
    assert!(custom_kinds.contains(&SyntaxKind::CustomPropertyName));
    assert!(escaped_custom_kinds.contains(&SyntaxKind::CustomPropertyName));
    assert!(!bare_dash_kinds.contains(&SyntaxKind::CustomPropertyName));
    assert!(bare_dash_kinds.contains(&SyntaxKind::Ident));
}

#[test]
fn tokenizes_signed_and_leading_dot_numbers_as_single_numeric_tokens() {
    let signed_number = lex("+1.5", StyleDialect::Css);
    let signed_dimension = lex("-2px", StyleDialect::Css);
    let leading_dot = lex(".5", StyleDialect::Css);
    let spaced_plus = lex("+ 1.5", StyleDialect::Css);
    let trailing_dot = lex("1.", StyleDialect::Css);
    let signed_number_kinds: Vec<SyntaxKind> = signed_number
        .tokens()
        .iter()
        .map(|token| token.kind)
        .collect();
    let signed_dimension_kinds: Vec<SyntaxKind> = signed_dimension
        .tokens()
        .iter()
        .map(|token| token.kind)
        .collect();
    let leading_dot_kinds: Vec<SyntaxKind> = leading_dot
        .tokens()
        .iter()
        .map(|token| token.kind)
        .collect();
    let spaced_plus_kinds: Vec<SyntaxKind> = spaced_plus
        .tokens()
        .iter()
        .map(|token| token.kind)
        .collect();
    let trailing_dot_kinds: Vec<SyntaxKind> = trailing_dot
        .tokens()
        .iter()
        .map(|token| token.kind)
        .collect();

    assert!(signed_number.errors().is_empty());
    assert!(signed_dimension.errors().is_empty());
    assert!(leading_dot.errors().is_empty());
    assert!(spaced_plus.errors().is_empty());
    assert!(trailing_dot.errors().is_empty());
    assert_eq!(signed_number_kinds, vec![SyntaxKind::Number]);
    assert_eq!(signed_dimension_kinds, vec![SyntaxKind::Dimension]);
    assert_eq!(leading_dot_kinds, vec![SyntaxKind::Number]);
    assert!(spaced_plus_kinds.contains(&SyntaxKind::Plus));
    assert!(spaced_plus_kinds.contains(&SyntaxKind::Number));
    assert_eq!(
        trailing_dot_kinds,
        vec![SyntaxKind::Number, SyntaxKind::Dot]
    );
}

#[test]
fn tokenizes_exponent_numbers_before_dimension_suffixes() {
    let exponent = lex("1e3", StyleDialect::Css);
    let signed_exponent = lex("1e-3", StyleDialect::Css);
    let exponent_dimension = lex("1e3px", StyleDialect::Css);
    let plain_dimension = lex("1em", StyleDialect::Css);
    let exponent_kinds: Vec<SyntaxKind> =
        exponent.tokens().iter().map(|token| token.kind).collect();
    let signed_exponent_kinds: Vec<SyntaxKind> = signed_exponent
        .tokens()
        .iter()
        .map(|token| token.kind)
        .collect();
    let exponent_dimension_kinds: Vec<SyntaxKind> = exponent_dimension
        .tokens()
        .iter()
        .map(|token| token.kind)
        .collect();
    let plain_dimension_kinds: Vec<SyntaxKind> = plain_dimension
        .tokens()
        .iter()
        .map(|token| token.kind)
        .collect();

    assert!(exponent.errors().is_empty());
    assert!(signed_exponent.errors().is_empty());
    assert!(exponent_dimension.errors().is_empty());
    assert!(plain_dimension.errors().is_empty());
    assert_eq!(exponent_kinds, vec![SyntaxKind::Number]);
    assert_eq!(signed_exponent_kinds, vec![SyntaxKind::Number]);
    assert_eq!(exponent_dimension_kinds, vec![SyntaxKind::Dimension]);
    assert_eq!(plain_dimension_kinds, vec![SyntaxKind::Dimension]);
}

#[test]
fn tokenizes_null_and_bom_without_unexpected_errors() {
    let result = parse("\u{feff}.a\0b { content: \0; }", StyleDialect::Css);
    let lexed = lex(
        "\u{feff}.a\0b { background: url(foo\0bar); }",
        StyleDialect::Css,
    );
    let token_kinds = token_kinds(&result.syntax());
    let ident = lexed
        .tokens()
        .iter()
        .find(|token| token.kind == SyntaxKind::Ident)
        .map(|token| token.text.as_str());
    let url = lexed
        .tokens()
        .iter()
        .find(|token| token.kind == SyntaxKind::Url)
        .map(|token| token.text.as_str());

    assert!(result.errors().is_empty());
    assert!(lexed.errors().is_empty());
    assert_eq!(
        lexed.tokens().first().map(|token| token.kind),
        Some(SyntaxKind::Dot)
    );
    assert_eq!(ident, Some("a\u{fffd}b"));
    assert_eq!(url, Some("url(foo\u{fffd}bar)"));
    assert!(
        !lexed
            .tokens()
            .iter()
            .any(|token| token.text.contains('\0') || token.text.contains('\u{feff}'))
    );
    assert!(token_kinds.contains(&SyntaxKind::Whitespace));
    assert!(token_kinds.contains(&SyntaxKind::Ident));
    assert!(node_kinds(&result.syntax()).contains(&SyntaxKind::ClassSelector));
}

#[test]
fn tokenizes_unquoted_urls_and_bad_urls() {
    let good = lex(".a { background: url(images/bg.png); }", StyleDialect::Css);
    let bad = lex(".a { background: url(foo\"bar); }", StyleDialect::Css);
    let bad_whitespace = lex(".a { background: url(foo bar); }", StyleDialect::Css);
    let bad_escape = lex(".a { background: url(foo\\\nbar); }", StyleDialect::Css);
    let trailing_whitespace = lex(".a { background: url(foo \n ); }", StyleDialect::Css);
    let quoted = lex(
        ".a { background: url(\"images/bg.png\"); }",
        StyleDialect::Css,
    );
    let good_kinds: Vec<SyntaxKind> = good.tokens().iter().map(|token| token.kind).collect();
    let bad_kinds: Vec<SyntaxKind> = bad.tokens().iter().map(|token| token.kind).collect();
    let bad_whitespace_kinds: Vec<SyntaxKind> = bad_whitespace
        .tokens()
        .iter()
        .map(|token| token.kind)
        .collect();
    let bad_escape_kinds: Vec<SyntaxKind> =
        bad_escape.tokens().iter().map(|token| token.kind).collect();
    let trailing_whitespace_kinds: Vec<SyntaxKind> = trailing_whitespace
        .tokens()
        .iter()
        .map(|token| token.kind)
        .collect();
    let quoted_kinds: Vec<SyntaxKind> = quoted.tokens().iter().map(|token| token.kind).collect();

    assert!(good.errors().is_empty());
    assert!(good_kinds.contains(&SyntaxKind::Url));
    assert!(bad_kinds.contains(&SyntaxKind::BadUrl));
    assert!(!bad.errors().is_empty());
    assert!(bad_whitespace_kinds.contains(&SyntaxKind::BadUrl));
    assert!(!bad_whitespace.errors().is_empty());
    assert!(bad_escape_kinds.contains(&SyntaxKind::BadUrl));
    assert!(!bad_escape.errors().is_empty());
    assert!(trailing_whitespace.errors().is_empty());
    assert!(trailing_whitespace_kinds.contains(&SyntaxKind::Url));
    assert!(quoted_kinds.contains(&SyntaxKind::Ident));
    assert!(quoted_kinds.contains(&SyntaxKind::String));
    assert!(!quoted_kinds.contains(&SyntaxKind::Url));
}

#[test]
fn tokenizes_unicode_ranges() {
    let result = lex(
        "@font-face { unicode-range: U+00A0-00FF, u+4??; }",
        StyleDialect::Css,
    );
    let kinds: Vec<SyntaxKind> = result.tokens().iter().map(|token| token.kind).collect();

    assert!(result.errors().is_empty());
    assert_eq!(
        kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::UnicodeRange)
            .count(),
        2
    );
}

#[test]
fn tokenizes_scss_interpolation_delimiters() {
    let scss = lex(
        ".button-#{$variant} { color: #{$color}; }",
        StyleDialect::Scss,
    );
    let css = lex(".button-#{$variant} { color: red; }", StyleDialect::Css);
    let scss_kinds: Vec<SyntaxKind> = scss.tokens().iter().map(|token| token.kind).collect();
    let css_kinds: Vec<SyntaxKind> = css.tokens().iter().map(|token| token.kind).collect();

    assert!(scss.errors().is_empty());
    assert!(scss_kinds.contains(&SyntaxKind::ScssInterpolationStart));
    assert!(scss_kinds.contains(&SyntaxKind::ScssInterpolationEnd));
    assert!(!css_kinds.contains(&SyntaxKind::ScssInterpolationStart));
}

#[test]
fn tokenizes_scss_placeholder_selectors() {
    let scss = lex("%button { color: red; }", StyleDialect::Scss);
    let css = lex("%button { color: red; }", StyleDialect::Css);
    let scss_kinds: Vec<SyntaxKind> = scss.tokens().iter().map(|token| token.kind).collect();
    let css_kinds: Vec<SyntaxKind> = css.tokens().iter().map(|token| token.kind).collect();

    assert!(scss.errors().is_empty());
    assert!(scss_kinds.contains(&SyntaxKind::ScssPlaceholder));
    assert!(css_kinds.contains(&SyntaxKind::Percent));
    assert!(!css_kinds.contains(&SyntaxKind::ScssPlaceholder));
}

#[test]
fn tokenizes_sass_indented_block_markers() {
    let result = lex(
        ".card\n  color: red // comment\n  .title\n    color: blue\n",
        StyleDialect::Sass,
    );
    let kinds: Vec<SyntaxKind> = result.tokens().iter().map(|token| token.kind).collect();

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::LineComment));
    assert!(kinds.contains(&SyntaxKind::SassIndentedNewline));
    assert!(kinds.contains(&SyntaxKind::SassOptionalSemicolon));
    assert_eq!(
        kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::SassIndent)
            .count(),
        2
    );
    assert_eq!(
        kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::SassDedent)
            .count(),
        2
    );
}

#[test]
fn tokenizes_less_interpolation_delimiters() {
    let less = lex(
        ".button-@{variant} { color: @{color}; }",
        StyleDialect::Less,
    );
    let css = lex(".button-@{variant} { color: red; }", StyleDialect::Css);
    let less_kinds: Vec<SyntaxKind> = less.tokens().iter().map(|token| token.kind).collect();
    let css_kinds: Vec<SyntaxKind> = css.tokens().iter().map(|token| token.kind).collect();

    assert!(less.errors().is_empty());
    assert!(less_kinds.contains(&SyntaxKind::LessInterpolationStart));
    assert!(less_kinds.contains(&SyntaxKind::LessInterpolationEnd));
    assert!(!css_kinds.contains(&SyntaxKind::LessInterpolationStart));
}

#[test]
fn tokenizes_less_escaped_strings() {
    let less = lex(".a { filter: ~\"alpha(opacity=50)\"; }", StyleDialect::Less);
    let css = lex(".a { filter: ~\"alpha(opacity=50)\"; }", StyleDialect::Css);
    let less_kinds: Vec<SyntaxKind> = less.tokens().iter().map(|token| token.kind).collect();
    let css_kinds: Vec<SyntaxKind> = css.tokens().iter().map(|token| token.kind).collect();

    assert!(less.errors().is_empty());
    assert!(less_kinds.contains(&SyntaxKind::LessEscapedString));
    assert!(!css_kinds.contains(&SyntaxKind::LessEscapedString));
    assert!(css_kinds.contains(&SyntaxKind::Tilde));
    assert!(css_kinds.contains(&SyntaxKind::String));
}

#[test]
fn tokenizes_less_property_variables_without_breaking_suffix_matchers() {
    let less = lex(
        ".a { background: $color; [data-x$=y] {} }",
        StyleDialect::Less,
    );
    let scss = lex(".a { background: $color; }", StyleDialect::Scss);
    let less_kinds: Vec<SyntaxKind> = less.tokens().iter().map(|token| token.kind).collect();
    let scss_kinds: Vec<SyntaxKind> = scss.tokens().iter().map(|token| token.kind).collect();

    assert!(less.errors().is_empty());
    assert!(scss.errors().is_empty());
    assert!(less_kinds.contains(&SyntaxKind::LessPropertyVariableToken));
    assert!(less_kinds.contains(&SyntaxKind::SuffixMatch));
    assert!(!less_kinds.contains(&SyntaxKind::ScssVariable));
    assert!(scss_kinds.contains(&SyntaxKind::ScssVariable));
}

#[test]
fn tokenizes_newline_bad_strings() {
    let result = lex(".a { content: \"bad\nstill-here: red; }", StyleDialect::Css);
    let kinds: Vec<SyntaxKind> = result.tokens().iter().map(|token| token.kind).collect();

    assert!(kinds.contains(&SyntaxKind::BadString));
    assert!(
        result
            .errors()
            .iter()
            .any(|error| error.code == ParseErrorCode::UnterminatedString)
    );
}

#[test]
fn exposes_recovery_token_sets() {
    assert!(RECOVERY_TOP.contains(SyntaxKind::AtKeyword));
    assert!(RECOVERY_DECLARATION.contains(SyntaxKind::Semicolon));
    assert!(RECOVERY_SELECTOR.contains(SyntaxKind::LeftBrace));
    assert!(!RECOVERY_SELECTOR.is_empty());
}

#[test]
fn builds_at_rule_and_bogus_nodes_for_partial_input() {
    let at_rule = parse("@media screen { .a { color: red; } }", StyleDialect::Css);
    let missing_colon = parse(".a { color red; }", StyleDialect::Css);
    let missing_block = parse(".a color: red;", StyleDialect::Css);

    assert!(node_kinds(&at_rule.syntax()).contains(&SyntaxKind::AtRule));
    assert!(node_kinds(&missing_colon.syntax()).contains(&SyntaxKind::BogusDeclaration));
    assert!(node_kinds(&missing_block.syntax()).contains(&SyntaxKind::BogusRule));
}

#[test]
fn builds_bogus_nodes_for_selector_and_value_recovery() {
    let missing_class_name = parse(". { color: red; }", StyleDialect::Css);
    let missing_attribute_end = parse(".a[data-active { color: red; }", StyleDialect::Css);
    let missing_value_rhs = parse(".a { width: calc(1 + ); }", StyleDialect::Css);
    let unexpected_value_token = parse(".a { color: @; }", StyleDialect::Css);

    assert_eq!(
        missing_class_name.errors().first().map(|error| error.code),
        Some(ParseErrorCode::ExpectedSelectorName)
    );
    assert_eq!(
        missing_attribute_end
            .errors()
            .first()
            .map(|error| error.code),
        Some(ParseErrorCode::UnterminatedAttributeSelector)
    );
    assert!(
        missing_value_rhs
            .errors()
            .iter()
            .any(|error| error.code == ParseErrorCode::ExpectedValue)
    );
    assert!(node_kinds(&missing_class_name.syntax()).contains(&SyntaxKind::BogusSelector));
    assert!(node_kinds(&missing_attribute_end.syntax()).contains(&SyntaxKind::BogusSelector));
    assert!(node_kinds(&missing_value_rhs.syntax()).contains(&SyntaxKind::BogusValue));
    assert!(node_kinds(&unexpected_value_token.syntax()).contains(&SyntaxKind::BogusValue));
}

#[test]
fn recovers_empty_declaration_values_without_rejecting_custom_properties() {
    let result = parse(".a { color: ; width: ; --empty: ; }", StyleDialect::Css);
    let kinds = node_kinds(&result.syntax());
    let empty_value_errors = result
        .errors()
        .iter()
        .filter(|error| error.message == "expected declaration value")
        .count();
    let bogus_value_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::BogusValue)
        .count();

    assert_eq!(empty_value_errors, 2);
    assert_eq!(bogus_value_count, 2);
    assert!(kinds.contains(&SyntaxKind::CustomPropertyValue));
}

#[test]
fn recovers_empty_variable_values_without_rejecting_less_detached_rulesets() {
    let scss = parse("$gap: ;", StyleDialect::Scss);
    let less = parse("@gap: ; @ruleset: { color: red; };", StyleDialect::Less);
    let scss_kinds = node_kinds(&scss.syntax());
    let less_kinds = node_kinds(&less.syntax());
    let empty_value_errors = scss
        .errors()
        .iter()
        .chain(less.errors())
        .filter(|error| error.message == "expected variable value")
        .count();

    assert_eq!(empty_value_errors, 2);
    assert!(scss_kinds.contains(&SyntaxKind::BogusValue));
    assert!(less_kinds.contains(&SyntaxKind::BogusValue));
    assert!(less_kinds.contains(&SyntaxKind::LessDetachedRulesetNode));
}

#[test]
fn recovers_missing_semicolons_between_declarations() {
    let result = parse(
        ".a { color: red background: blue; margin: 0 padding: 1rem; }",
        StyleDialect::Css,
    );
    let custom_property = parse(
        ".a { --token: red background: blue; color: red; }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());
    let custom_property_kinds = node_kinds(&custom_property.syntax());
    let declaration_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::Declaration)
        .count();
    let custom_property_declaration_count = custom_property_kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::Declaration)
        .count();
    let missing_semicolon_errors = result
        .errors()
        .iter()
        .filter(|error| error.message == "expected semicolon between declarations")
        .count();

    assert_eq!(declaration_count, 4);
    assert_eq!(missing_semicolon_errors, 2);
    assert_eq!(custom_property_declaration_count, 2);
    assert!(custom_property.errors().is_empty());
    assert!(custom_property_kinds.contains(&SyntaxKind::CustomPropertyValue));
}

#[test]
fn populates_core_bogus_nodes_for_recoverable_structures() {
    let missing_function_close = parse(".a { width: calc(1 + ; color: red; }", StyleDialect::Css);
    let missing_media_close = parse(
        "@media (min-width: { .a { color: red; } }",
        StyleDialect::Css,
    );
    let mixed_media_close = parse(
        "@media screen, (min-width: { .a { color: red; } }",
        StyleDialect::Css,
    );
    let missing_supports_close = parse(
        "@supports (display: { .a { color: red; } }",
        StyleDialect::Css,
    );
    let missing_container_close = parse(
        "@container (inline-size > { .a { color: red; } }",
        StyleDialect::Css,
    );
    let missing_unknown_prelude_close =
        parse("@unknown (min-width: { color: red; }", StyleDialect::Css);
    let missing_scope_close = parse("@scope (.a { .b { color: red; } }", StyleDialect::Css);
    let empty_layer_statement = parse("@layer ;", StyleDialect::Css);
    let missing_keyframe_block = parse("@keyframes fade { from opacity: 0; }", StyleDialect::Css);
    let unclosed_rule = parse(".a { color: red;", StyleDialect::Css);

    assert!(node_kinds(&missing_function_close.syntax()).contains(&SyntaxKind::BogusFunctionCall));
    assert!(
        node_kinds(&missing_function_close.syntax()).contains(&SyntaxKind::BogusFunctionArguments)
    );
    assert!(node_kinds(&missing_media_close.syntax()).contains(&SyntaxKind::BogusMediaQuery));
    assert!(node_kinds(&mixed_media_close.syntax()).contains(&SyntaxKind::MediaQuery));
    assert!(node_kinds(&mixed_media_close.syntax()).contains(&SyntaxKind::BogusMediaQuery));
    assert!(
        node_kinds(&missing_supports_close.syntax()).contains(&SyntaxKind::BogusSupportsCondition)
    );
    assert!(
        node_kinds(&missing_container_close.syntax())
            .contains(&SyntaxKind::BogusContainerCondition)
    );
    assert!(
        node_kinds(&missing_unknown_prelude_close.syntax())
            .contains(&SyntaxKind::BogusAtRulePrelude)
    );
    assert!(node_kinds(&missing_scope_close.syntax()).contains(&SyntaxKind::BogusScopeRange));
    assert!(node_kinds(&empty_layer_statement.syntax()).contains(&SyntaxKind::BogusLayerName));
    assert!(node_kinds(&missing_keyframe_block.syntax()).contains(&SyntaxKind::BogusKeyframeBlock));
    assert!(node_kinds(&unclosed_rule.syntax()).contains(&SyntaxKind::BogusDeclarationList));
    assert!(node_kinds(&unclosed_rule.syntax()).contains(&SyntaxKind::BogusTrivia));
}

#[test]
fn populates_dialect_and_selector_bogus_nodes() {
    let invalid_compound = parse("%bad { color: red; }", StyleDialect::Css);
    let dangling_combinator = parse(".a > { color: red; }", StyleDialect::Css);
    let missing_property = parse(".a { : red; }", StyleDialect::Css);
    let missing_colon_recovery = parse("$gap 1rem;", StyleDialect::Scss);
    let unexpected_value_token = parse(".a { width: ?; }", StyleDialect::Css);
    let missing_at_rule_name = parse("@ ;", StyleDialect::Css);
    let missing_scss_variable_colon = parse("$gap;", StyleDialect::Scss);
    let missing_less_variable_colon = parse("@gap;", StyleDialect::Less);
    let missing_scss_blocks = parse("@mixin card; @function double; @if $x;", StyleDialect::Scss);
    let inconsistent_sass_indentation =
        parse(".card\n  color: red\n color: blue\n", StyleDialect::Sass);
    let missing_less_mixin_block = parse(".theme(@tone);", StyleDialect::Less);
    let missing_less_guard_condition = parse(".theme() when { color: red; }", StyleDialect::Less);

    assert!(node_kinds(&invalid_compound.syntax()).contains(&SyntaxKind::BogusCompoundSelector));
    assert!(node_kinds(&dangling_combinator.syntax()).contains(&SyntaxKind::BogusCombinator));
    assert!(node_kinds(&missing_property.syntax()).contains(&SyntaxKind::BogusPropertyName));
    assert!(node_kinds(&missing_colon_recovery.syntax()).contains(&SyntaxKind::BogusRecovery));
    assert!(node_kinds(&unexpected_value_token.syntax()).contains(&SyntaxKind::BogusToken));
    assert!(node_kinds(&missing_at_rule_name.syntax()).contains(&SyntaxKind::BogusAtRule));
    assert!(
        node_kinds(&missing_scss_variable_colon.syntax()).contains(&SyntaxKind::BogusScssVariable)
    );
    assert!(
        node_kinds(&missing_less_variable_colon.syntax()).contains(&SyntaxKind::BogusLessVariable)
    );
    assert!(node_kinds(&missing_scss_blocks.syntax()).contains(&SyntaxKind::BogusScssMixin));
    assert!(node_kinds(&missing_scss_blocks.syntax()).contains(&SyntaxKind::BogusScssFunction));
    assert!(node_kinds(&missing_scss_blocks.syntax()).contains(&SyntaxKind::BogusScssControl));
    assert!(
        node_kinds(&inconsistent_sass_indentation.syntax())
            .contains(&SyntaxKind::BogusSassIndentation)
    );
    assert!(node_kinds(&missing_less_mixin_block.syntax()).contains(&SyntaxKind::BogusLessMixin));
    assert!(
        node_kinds(&missing_less_guard_condition.syntax()).contains(&SyntaxKind::BogusLessGuard)
    );
}

#[test]
fn populates_every_declared_bogus_kind_in_recovery_corpus() {
    let mut actual = BTreeSet::new();
    let mut collect = |result: ParseResult| {
        actual.extend(
            node_kinds(&result.syntax())
                .into_iter()
                .filter(|kind| kind.is_bogus()),
        );
    };

    collect(parse("{ color: red; }", StyleDialect::Css));
    collect(parse(". { color: red; }", StyleDialect::Css));
    collect(parse("%bad { color: red; }", StyleDialect::Css));
    collect(parse(".a > { color: red; }", StyleDialect::Css));
    collect(parse(".a { : red; width: ?; }", StyleDialect::Css));
    collect(parse(
        ".a { width: ; height: calc(1 + ; }",
        StyleDialect::Css,
    ));
    collect(parse(".a { color: [red; }", StyleDialect::Css));
    collect(parse(".a { font-family: system, ; }", StyleDialect::Css));
    collect(parse("@ ;", StyleDialect::Css));
    collect(parse(
        "@unknown (min-width: { color: red; }",
        StyleDialect::Css,
    ));
    collect(parse(
        "@media screen, (min-width: { .a { color: red; } }",
        StyleDialect::Css,
    ));
    collect(parse(
        "@supports (display: { .a { color: red; } }",
        StyleDialect::Css,
    ));
    collect(parse(
        "@container (inline-size > { .a { color: red; } }",
        StyleDialect::Css,
    ));
    collect(parse("@layer ;", StyleDialect::Css));
    collect(parse(
        "@scope (.a { .b { color: red; } }",
        StyleDialect::Css,
    ));
    collect(parse(
        "@keyframes fade { from opacity: 0; }",
        StyleDialect::Css,
    ));
    collect(parse(
        "@value from; .bad { composes: from; } .missing { composes base; }",
        StyleDialect::Scss,
    ));
    collect(parse(
        "@use \"theme\" with ($gap: 1rem; .card { color: red; }",
        StyleDialect::Scss,
    ));
    collect(parse("$bad-map: (a: 1;", StyleDialect::Scss));
    collect(parse("$bad-entry: (a: 1, b:);", StyleDialect::Scss));
    collect(parse("$bad-list: (1, 2;", StyleDialect::Scss));
    collect(parse("@if { .a { color: red; } }", StyleDialect::Scss));
    collect(parse(
        "@mixin card; @function double; @if $x;",
        StyleDialect::Scss,
    ));
    collect(parse("$gap;", StyleDialect::Scss));
    collect(parse(".a { content: \"unterminated\n }", StyleDialect::Css));
    collect(parse(".a { color: #{$tone; }", StyleDialect::Scss));
    collect(parse(
        ".card\n  color: red\n color: blue\n",
        StyleDialect::Sass,
    ));
    collect(parse("@gap;", StyleDialect::Less));
    collect(parse(".theme(@tone);", StyleDialect::Less));
    collect(parse(".theme() when { color: red; }", StyleDialect::Less));
    collect(parse("@detached: { .a { color: red; }", StyleDialect::Less));
    collect(parse("$gap 1rem;", StyleDialect::Scss));
    collect(parse_entry_point(
        "[red",
        StyleDialect::Css,
        ParseEntryPoint::SimpleBlock,
    ));
    collect(parse_entry_point(
        "red, ;",
        StyleDialect::Css,
        ParseEntryPoint::CommaSeparatedComponentValueList,
    ));

    let declared = SyntaxKind::ALL
        .iter()
        .copied()
        .filter(|kind| kind.is_bogus())
        .collect::<BTreeSet<_>>();
    let missing = declared.difference(&actual).copied().collect::<Vec<_>>();

    assert!(missing.is_empty(), "missing bogus kinds: {missing:?}");
}

#[test]
fn parses_css_module_value_and_composes_cst_nodes() {
    let result = parse(
        "@value primary: #fff; @value accent: primary; @value secondary as localSecondary from \"./tokens.module.scss\"; .btn { composes: base utility from \"./base.module.scss\"; }",
        StyleDialect::Scss,
    );
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::CssModuleExportBlock));
    assert!(kinds.contains(&SyntaxKind::CssModuleImportBlock));
    assert!(kinds.contains(&SyntaxKind::TokenDefinition));
    assert!(kinds.contains(&SyntaxKind::TokenReference));
    assert!(kinds.contains(&SyntaxKind::CssModuleComposesDeclaration));
    assert!(kinds.contains(&SyntaxKind::CssModuleComposesTarget));
    assert!(kinds.contains(&SyntaxKind::CssModuleFromClause));
}

#[test]
fn extracts_css_module_value_style_facts() {
    let facts = collect_style_facts(
        "@value primary: #fff; @value accent: primary; @value secondary as localSecondary from \"./tokens.module.scss\"; .btn { color: accent; }",
        StyleDialect::Css,
    );
    let definitions = facts
        .css_module_values
        .iter()
        .filter(|value| value.kind == ParsedCssModuleValueFactKind::Definition)
        .map(|value| value.name.as_str())
        .collect::<Vec<_>>();
    let references = facts
        .css_module_values
        .iter()
        .filter(|value| value.kind == ParsedCssModuleValueFactKind::Reference)
        .map(|value| value.name.as_str())
        .collect::<Vec<_>>();
    let import_sources = facts
        .css_module_values
        .iter()
        .filter(|value| value.kind == ParsedCssModuleValueFactKind::ImportSource)
        .map(|value| value.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(facts.css_module_value_count, 7);
    assert_eq!(definitions, vec!["primary", "accent", "localSecondary"]);
    assert_eq!(references, vec!["primary", "secondary", "accent"]);
    assert_eq!(import_sources, vec!["./tokens.module.scss"]);
    assert_eq!(facts.css_module_value_import_edge_count, 1);
    assert_eq!(
        facts.css_module_value_import_edges[0].remote_name,
        "secondary"
    );
    assert_eq!(
        facts.css_module_value_import_edges[0].local_name,
        "localSecondary"
    );
    assert_eq!(
        facts.css_module_value_import_edges[0].import_source,
        "./tokens.module.scss"
    );
    assert_eq!(facts.css_module_value_definition_edge_count, 1);
    assert_eq!(
        facts.css_module_value_definition_edges[0].definition_name,
        "accent"
    );
    assert_eq!(
        facts.css_module_value_definition_edges[0].reference_names,
        vec!["primary"]
    );
}

#[test]
fn extracts_css_module_value_path_alias_import_edges() {
    let facts = collect_style_facts(
        "@value colors: \"./colors.module.scss\"; @value primary, secondary as accent from colors; .btn { color: primary; border-color: accent; }",
        StyleDialect::Css,
    );
    let definitions = facts
        .css_module_values
        .iter()
        .filter(|value| value.kind == ParsedCssModuleValueFactKind::Definition)
        .map(|value| value.name.as_str())
        .collect::<Vec<_>>();
    let import_sources = facts
        .css_module_values
        .iter()
        .filter(|value| value.kind == ParsedCssModuleValueFactKind::ImportSource)
        .map(|value| value.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(definitions, vec!["primary", "accent"]);
    assert_eq!(import_sources, vec!["./colors.module.scss"]);
    assert_eq!(facts.css_module_value_import_edge_count, 2);
    assert_eq!(
        facts.css_module_value_import_edges[0].remote_name,
        "primary"
    );
    assert_eq!(facts.css_module_value_import_edges[0].local_name, "primary");
    assert_eq!(
        facts.css_module_value_import_edges[0].import_source,
        "./colors.module.scss"
    );
    assert_eq!(
        facts.css_module_value_import_edges[1].remote_name,
        "secondary"
    );
    assert_eq!(facts.css_module_value_import_edges[1].local_name, "accent");
    assert_eq!(
        facts.css_module_value_import_edges[1].import_source,
        "./colors.module.scss"
    );
}

#[test]
fn extracts_css_module_composes_style_facts() {
    let facts = collect_style_facts(
        ".btn { composes: base utility from \"./base.module.scss\"; } .global { composes: reset from global; }",
        StyleDialect::Css,
    );
    let targets = facts
        .css_module_composes
        .iter()
        .filter(|composes| composes.kind == ParsedCssModuleComposesFactKind::Target)
        .map(|composes| composes.name.as_str())
        .collect::<Vec<_>>();
    let import_sources = facts
        .css_module_composes
        .iter()
        .filter(|composes| composes.kind == ParsedCssModuleComposesFactKind::ImportSource)
        .map(|composes| composes.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(facts.css_module_composes_count, 5);
    assert_eq!(targets, vec!["base", "utility", "reset"]);
    assert_eq!(import_sources, vec!["./base.module.scss", "global"]);
    assert_eq!(facts.css_module_composes_edge_count, 2);
    assert_eq!(
        facts.css_module_composes_edges[0].kind,
        ParsedCssModuleComposesEdgeKind::External
    );
    assert_eq!(
        facts.css_module_composes_edges[0].owner_selector_names,
        vec!["btn"]
    );
    assert_eq!(
        facts.css_module_composes_edges[0].target_names,
        vec!["base", "utility"]
    );
    assert_eq!(
        facts.css_module_composes_edges[0].import_source.as_deref(),
        Some("./base.module.scss")
    );
    assert_eq!(
        facts.css_module_composes_edges[1].kind,
        ParsedCssModuleComposesEdgeKind::Global
    );
    assert_eq!(
        facts.css_module_composes_edges[1].owner_selector_names,
        vec!["global"]
    );
    assert_eq!(
        facts.css_module_composes_edges[1].target_names,
        vec!["reset"]
    );
    assert_eq!(
        facts.css_module_composes_edges[1].import_source.as_deref(),
        Some("global")
    );
}

#[test]
fn parses_icss_import_export_blocks() {
    let result = parse(
        ":export { primary: #fff; } :import(\"./tokens.css\") { imported: primary; } .btn { composes: imported; }",
        StyleDialect::Css,
    );
    let invalid = parse(":import { imported: primary; }", StyleDialect::Css);
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::CssModuleExportBlock));
    assert!(kinds.contains(&SyntaxKind::CssModuleImportBlock));
    assert!(
        invalid
            .errors()
            .iter()
            .any(|error| error.message == "expected ICSS import source")
    );
}

#[test]
fn extracts_icss_style_facts() {
    let facts = collect_style_facts(
        ":export { primary: #fff; secondary: accent; } :import(\"./tokens.css\") { imported: primary; tone: themeTone; }",
        StyleDialect::Css,
    );
    let export_names = facts
        .icss
        .iter()
        .filter(|icss| icss.kind == ParsedIcssFactKind::ExportName)
        .map(|icss| icss.name.as_str())
        .collect::<Vec<_>>();
    let import_local_names = facts
        .icss
        .iter()
        .filter(|icss| icss.kind == ParsedIcssFactKind::ImportLocalName)
        .map(|icss| icss.name.as_str())
        .collect::<Vec<_>>();
    let import_remote_names = facts
        .icss
        .iter()
        .filter(|icss| icss.kind == ParsedIcssFactKind::ImportRemoteName)
        .map(|icss| icss.name.as_str())
        .collect::<Vec<_>>();
    let import_sources = facts
        .icss
        .iter()
        .filter(|icss| icss.kind == ParsedIcssFactKind::ImportSource)
        .map(|icss| icss.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(facts.icss_count, 7);
    assert_eq!(export_names, vec!["primary", "secondary"]);
    assert_eq!(import_local_names, vec!["imported", "tone"]);
    assert_eq!(import_remote_names, vec!["primary", "themeTone"]);
    assert_eq!(import_sources, vec!["./tokens.css"]);
    assert_eq!(facts.icss_import_edge_count, 2);
    assert_eq!(facts.icss_import_edges[0].local_name, "imported");
    assert_eq!(facts.icss_import_edges[0].remote_name, "primary");
    assert_eq!(facts.icss_import_edges[0].import_source, "./tokens.css");
    assert_eq!(facts.icss_import_edges[1].local_name, "tone");
    assert_eq!(facts.icss_import_edges[1].remote_name, "themeTone");
    assert_eq!(facts.icss_import_edges[1].import_source, "./tokens.css");
    assert_eq!(facts.icss_export_edge_count, 1);
    assert_eq!(facts.icss_export_edges[0].export_name, "secondary");
    assert_eq!(facts.icss_export_edges[0].reference_names, vec!["accent"]);
}

#[test]
fn recovers_css_module_value_and_composes_bogus_nodes() {
    let result = parse(
        "@value from; .bad { composes: from; } .missing { composes base; } .invalid { composes: base from 123; } @value bad as alias from 123; .multi { composes: a from \"./a.css\", b from \"./b.css\"; }",
        StyleDialect::Scss,
    );
    let kinds = node_kinds(&result.syntax());
    let invalid_from_source_count = result
        .errors()
        .iter()
        .filter(|error| error.message == "invalid CSS Modules from-clause source")
        .count();
    let multiple_from_count = result
        .errors()
        .iter()
        .filter(|error| error.message == "multiple composes from clauses are not allowed")
        .count();

    assert!(kinds.contains(&SyntaxKind::BogusCssModuleBlock));
    assert!(kinds.contains(&SyntaxKind::BogusFromClause));
    assert!(kinds.contains(&SyntaxKind::BogusComposesTarget));
    assert!(kinds.contains(&SyntaxKind::BogusComposesDeclaration));
    assert_eq!(invalid_from_source_count, 2);
    assert_eq!(multiple_from_count, 1);
}

#[test]
fn validates_composes_outside_css_module_global_scope() {
    let invalid = parse(
        ":global(.reset) { composes: base; } :global { .utility { composes: base; } } :local(.ok) { composes: base; }",
        StyleDialect::Css,
    );
    let outer_local = parse(
        ":local { :global(.ok) { composes: base; } }",
        StyleDialect::Css,
    );
    let mixed_local_global = parse(".foo :global(.bar) { composes: base; }", StyleDialect::Css);
    let global_composes_count = invalid
        .errors()
        .iter()
        .filter(|error| error.message == "composes is not allowed inside :global scope")
        .count();

    assert_eq!(global_composes_count, 2);
    assert!(
        !outer_local
            .errors()
            .iter()
            .any(|error| error.message == "composes is not allowed inside :global scope")
    );
    assert!(
        !mixed_local_global
            .errors()
            .iter()
            .any(|error| error.message == "composes is not allowed inside :global scope")
    );
}

#[test]
fn parses_registered_group_at_rule_blocks() {
    let result = parse(
        "@media screen and (min-width: 40rem) { .card { color: red; } }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::AtRule));
    assert!(kinds.contains(&SyntaxKind::MediaRule));
    assert!(kinds.contains(&SyntaxKind::RuleList));
    assert!(kinds.contains(&SyntaxKind::Rule));
    assert!(kinds.contains(&SyntaxKind::ClassSelector));
}

#[test]
fn parses_conditional_at_rule_preludes() {
    let result = parse(
        "@media screen and (min-width: 40rem), print { .card { color: red; } } @supports (display: grid) { .grid { display: grid; } } @container card (inline-size > 40rem) { .item { color: blue; } }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::MediaQueryList));
    assert_eq!(
        kinds
            .iter()
            .filter(|kind| **kind == SyntaxKind::MediaQuery)
            .count(),
        2
    );
    assert!(kinds.contains(&SyntaxKind::MediaFeature));
    assert!(kinds.contains(&SyntaxKind::SupportsCondition));
    assert!(kinds.contains(&SyntaxKind::ContainerCondition));
}

#[test]
fn validates_media_query_list_preludes() {
    let result = parse(
        "@media { .a { color: red; } } @media , screen { .b { color: blue; } } @media screen, { .c { color: green; } } @media 1 { .d { color: black; } } @media screen and (min-width: 40rem), print { .e { color: white; } }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());
    let invalid_media_errors = result
        .errors()
        .iter()
        .filter(|error| error.message == "invalid @media prelude")
        .count();
    let bogus_media_queries = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::BogusMediaQuery)
        .count();

    assert_eq!(invalid_media_errors, 4);
    assert_eq!(bogus_media_queries, 4);
    assert!(kinds.contains(&SyntaxKind::MediaQuery));
}

#[test]
fn validates_supports_rule_preludes() {
    let result = parse(
        "@supports { .a { color: red; } } @supports display: grid { .b { color: blue; } } @supports not { .c { color: green; } } @supports (display: grid) { .d { color: black; } } @supports selector(:has(*)) { .e { color: white; } }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());
    let invalid_supports_errors = result
        .errors()
        .iter()
        .filter(|error| error.message == "invalid @supports prelude")
        .count();
    let bogus_supports_conditions = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::BogusSupportsCondition)
        .count();

    assert_eq!(invalid_supports_errors, 3);
    assert_eq!(bogus_supports_conditions, 3);
    assert!(kinds.contains(&SyntaxKind::SupportsCondition));
}

#[test]
fn validates_container_rule_preludes() {
    let result = parse(
        "@container { .a { color: red; } } @container card { .b { color: blue; } } @container 1 (width > 0) { .c { color: green; } } @container style(--theme: dark) { .d { color: white; } } @container card style(--theme: dark) { .e { color: black; } }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());
    let invalid_container_errors = result
        .errors()
        .iter()
        .filter(|error| error.message == "invalid @container prelude")
        .count();
    let bogus_container_conditions = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::BogusContainerCondition)
        .count();

    assert_eq!(invalid_container_errors, 3);
    assert_eq!(bogus_container_conditions, 3);
    assert!(kinds.contains(&SyntaxKind::ContainerCondition));
}

#[test]
fn classifies_css_at_rules_case_insensitively() {
    let source = "@MEDIA (width >= 1px) { .card { color: red; } } @KEYFRAMES fade { from { opacity: 0; } to { opacity: 1; } }";
    let result = parse(source, StyleDialect::Css);
    let facts = collect_style_facts(source, StyleDialect::Css);
    let kinds = node_kinds(&result.syntax());
    let at_rule_names: Vec<&str> = facts
        .at_rules
        .iter()
        .map(|at_rule| at_rule.name.as_str())
        .collect();

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::MediaRule));
    assert!(kinds.contains(&SyntaxKind::KeyframesRule));
    assert!(
        facts
            .selectors
            .iter()
            .any(|selector| selector.name == "card")
    );
    assert_eq!(at_rule_names, vec!["@media", "@keyframes"]);
}

#[test]
fn parses_import_layer_supports_media_prelude() {
    let result = parse(
        "@import url(\"theme.css\") layer(app.theme) supports(display: grid) screen and (min-width: 40rem);",
        StyleDialect::Css,
    );
    let less = parse(
        "@import (reference) \"theme.less\" screen and (min-width: 40rem);",
        StyleDialect::Less,
    );
    let kinds = node_kinds(&result.syntax());
    let less_kinds = node_kinds(&less.syntax());

    assert!(result.errors().is_empty());
    assert!(less.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::ImportRule));
    assert!(kinds.contains(&SyntaxKind::UrlValue));
    assert!(kinds.contains(&SyntaxKind::LayerName));
    assert!(kinds.contains(&SyntaxKind::SupportsCondition));
    assert!(kinds.contains(&SyntaxKind::MediaQueryList));
    assert!(kinds.contains(&SyntaxKind::MediaFeature));
    assert!(less_kinds.contains(&SyntaxKind::ImportRule));
    assert!(less_kinds.contains(&SyntaxKind::AtRulePrelude));
    assert!(less_kinds.contains(&SyntaxKind::MediaQueryList));
}

#[test]
fn validates_import_sources() {
    let result = parse(
        "@import ; @import layer(app); @import 1; @import url(foo bar);",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());
    let invalid_import_errors = result
        .errors()
        .iter()
        .filter(|error| error.message == "invalid @import source")
        .count();
    let bogus_preludes = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::BogusAtRulePrelude)
        .count();

    assert_eq!(invalid_import_errors, 4);
    assert_eq!(bogus_preludes, 4);
}

#[test]
fn validates_import_optional_tails() {
    let result = parse(
        "@import \"a.css\" layer(); @import \"b.css\" layer(1); @import \"c.css\" supports(); @import \"d.css\" supports screen; @import \"ok.css\" layer(app.theme) supports(display: grid) screen;",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());
    let invalid_layer_tail_errors = result
        .errors()
        .iter()
        .filter(|error| error.message == "invalid @import layer tail")
        .count();
    let invalid_supports_tail_errors = result
        .errors()
        .iter()
        .filter(|error| error.message == "invalid @import supports tail")
        .count();
    let bogus_layer_names = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::BogusLayerName)
        .count();
    let bogus_supports_conditions = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::BogusSupportsCondition)
        .count();

    assert_eq!(invalid_layer_tail_errors, 2);
    assert_eq!(invalid_supports_tail_errors, 2);
    assert_eq!(bogus_layer_names, 2);
    assert_eq!(bogus_supports_conditions, 2);
    assert!(kinds.contains(&SyntaxKind::LayerName));
    assert!(kinds.contains(&SyntaxKind::SupportsCondition));
    assert!(kinds.contains(&SyntaxKind::MediaQueryList));
}

#[test]
fn parses_layer_and_scope_preludes() {
    let result = parse(
        "@layer reset, app.ui; @layer components { .card { color: red; } } @layer { .anon { color: blue; } } @scope (.card) to (.card-content) { .title { color: red; } }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::LayerRule));
    assert!(kinds.contains(&SyntaxKind::LayerName));
    assert!(kinds.contains(&SyntaxKind::ScopeRule));
    assert!(kinds.contains(&SyntaxKind::ScopeRange));
    assert!(kinds.contains(&SyntaxKind::RuleList));
}

#[test]
fn validates_layer_rule_preludes() {
    let result = parse(
        "@layer , reset; @layer app.; @layer 1; @layer ok.name;",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());
    let invalid_layer_errors = result
        .errors()
        .iter()
        .filter(|error| error.message == "invalid @layer prelude")
        .count();
    let bogus_layer_names = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::BogusLayerName)
        .count();

    assert_eq!(invalid_layer_errors, 3);
    assert_eq!(bogus_layer_names, 3);
    assert!(kinds.contains(&SyntaxKind::LayerName));
}

#[test]
fn validates_scope_rule_preludes() {
    let result = parse(
        "@scope { .a { color: red; } } @scope .a { .b { color: blue; } } @scope (.a) to { .c { color: green; } } @scope (.a) to (.b) { .d { color: black; } }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());
    let invalid_scope_errors = result
        .errors()
        .iter()
        .filter(|error| error.message == "invalid @scope prelude")
        .count();
    let bogus_scope_ranges = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::BogusScopeRange)
        .count();

    assert_eq!(invalid_scope_errors, 3);
    assert_eq!(bogus_scope_ranges, 3);
    assert!(kinds.contains(&SyntaxKind::ScopeRange));
}

#[test]
fn validates_page_rule_preludes() {
    let result = parse(
        "@page { margin: 1cm; } @page :first { margin: 2cm; } @page chapter:left, appendix:right { margin: 3cm; } @page 1 { margin: 4cm; } @page chapter, { margin: 5cm; } @page chapter first { margin: 6cm; }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());
    let invalid_page_errors = result
        .errors()
        .iter()
        .filter(|error| error.message == "invalid @page prelude")
        .count();
    let bogus_preludes = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::BogusAtRulePrelude)
        .count();

    assert_eq!(invalid_page_errors, 3);
    assert_eq!(bogus_preludes, 3);
    assert!(kinds.contains(&SyntaxKind::PageRule));
    assert!(kinds.contains(&SyntaxKind::AtRulePrelude));
}

#[test]
fn parses_registered_keyframes_and_declaration_at_rules() {
    let keyframes = parse(
        "@keyframes fade { from { opacity: 0; } to { opacity: 1; } }",
        StyleDialect::Css,
    );
    let font_face = parse(
        "@font-face { font-family: \"Demo\"; src: url(demo.woff2); }",
        StyleDialect::Css,
    );
    let page_margin = parse(
        "@page :first { margin: 1cm; @top-left { content: \"A\"; } @bottom-center { content: counter(page); } }",
        StyleDialect::Css,
    );
    let conditional_l5 = parse(
        "@when media(width >= 1px) { .a { color: red; } } @else { .b { color: blue; } }",
        StyleDialect::Css,
    );
    let modern_declaration_rules = parse(
        "@counter-style thumbs { system: cyclic; symbols: \"yes\"; suffix: \" \"; } @font-palette-values --brand { font-family: Demo; base-palette: 1; } @color-profile --display-p3 { src: url(p3.icc); } @position-try --popover { inset-area: top; }",
        StyleDialect::Css,
    );
    let native_function_rule = parse(
        "@function --gap(--size <length>: 1rem) returns <length> { result: var(--size); }",
        StyleDialect::Css,
    );
    let font_feature_values = parse(
        "@font-feature-values Demo { @stylistic { nice: 1; } @styleset { alt: 2; } @character-variant { nice: 3 4; } @swash { fancy: 1; } @ornaments { leaf: 1; } @annotation { circled: 1; } @historical-forms { old: 1; } } @view-transition { navigation: auto; }",
        StyleDialect::Css,
    );
    let less_css_at_rules = parse(
        "@font-feature-values Demo { @styleset { alt: 2; } } @view-transition { navigation: auto; }",
        StyleDialect::Less,
    );
    let nesting_and_custom_media = parse(
        ".card { @nest &__icon { color: red; &--active { color: blue; } } } @custom-media --narrow (width < 40rem);",
        StyleDialect::Css,
    );
    let keyframe_kinds = node_kinds(&keyframes.syntax());
    let font_face_kinds = node_kinds(&font_face.syntax());
    let page_margin_kinds = node_kinds(&page_margin.syntax());
    let conditional_l5_kinds = node_kinds(&conditional_l5.syntax());
    let modern_declaration_kinds = node_kinds(&modern_declaration_rules.syntax());
    let native_function_kinds = node_kinds(&native_function_rule.syntax());
    let font_feature_value_kinds = node_kinds(&font_feature_values.syntax());
    let less_css_at_rule_kinds = node_kinds(&less_css_at_rules.syntax());
    let nesting_and_custom_media_kinds = node_kinds(&nesting_and_custom_media.syntax());

    assert!(keyframes.errors().is_empty());
    assert!(font_face.errors().is_empty());
    assert!(page_margin.errors().is_empty());
    assert!(conditional_l5.errors().is_empty());
    assert!(modern_declaration_rules.errors().is_empty());
    assert!(native_function_rule.errors().is_empty());
    assert!(font_feature_values.errors().is_empty());
    assert!(less_css_at_rules.errors().is_empty());
    assert!(nesting_and_custom_media.errors().is_empty());
    assert!(keyframe_kinds.contains(&SyntaxKind::KeyframesRule));
    assert!(keyframe_kinds.contains(&SyntaxKind::AtRulePrelude));
    assert!(keyframe_kinds.contains(&SyntaxKind::KeyframeBlock));
    assert!(font_face_kinds.contains(&SyntaxKind::FontFaceRule));
    assert!(font_face_kinds.contains(&SyntaxKind::DeclarationList));
    assert!(page_margin_kinds.contains(&SyntaxKind::PageRule));
    assert!(page_margin_kinds.contains(&SyntaxKind::PageMarginRule));
    assert!(conditional_l5_kinds.contains(&SyntaxKind::WhenRule));
    assert!(conditional_l5_kinds.contains(&SyntaxKind::ElseRule));
    assert!(conditional_l5_kinds.contains(&SyntaxKind::RuleList));
    assert!(modern_declaration_kinds.contains(&SyntaxKind::CounterStyleRule));
    assert!(modern_declaration_kinds.contains(&SyntaxKind::FontPaletteValuesRule));
    assert!(modern_declaration_kinds.contains(&SyntaxKind::ColorProfileRule));
    assert!(modern_declaration_kinds.contains(&SyntaxKind::PositionTryRule));
    assert!(modern_declaration_kinds.contains(&SyntaxKind::DeclarationList));
    assert!(native_function_kinds.contains(&SyntaxKind::FunctionRule));
    assert!(native_function_kinds.contains(&SyntaxKind::DeclarationList));
    assert!(!native_function_kinds.contains(&SyntaxKind::ScssFunctionDeclaration));
    assert!(font_feature_value_kinds.contains(&SyntaxKind::FontFeatureValuesRule));
    assert!(font_feature_value_kinds.contains(&SyntaxKind::FontFeatureValuesStylisticRule));
    assert!(font_feature_value_kinds.contains(&SyntaxKind::FontFeatureValuesStylesetRule));
    assert!(font_feature_value_kinds.contains(&SyntaxKind::FontFeatureValuesCharacterVariantRule));
    assert!(font_feature_value_kinds.contains(&SyntaxKind::FontFeatureValuesSwashRule));
    assert!(font_feature_value_kinds.contains(&SyntaxKind::FontFeatureValuesOrnamentsRule));
    assert!(font_feature_value_kinds.contains(&SyntaxKind::FontFeatureValuesAnnotationRule));
    assert!(font_feature_value_kinds.contains(&SyntaxKind::FontFeatureValuesHistoricalFormsRule));
    assert!(font_feature_value_kinds.contains(&SyntaxKind::ViewTransitionRule));
    assert!(less_css_at_rule_kinds.contains(&SyntaxKind::FontFeatureValuesRule));
    assert!(less_css_at_rule_kinds.contains(&SyntaxKind::FontFeatureValuesStylesetRule));
    assert!(less_css_at_rule_kinds.contains(&SyntaxKind::ViewTransitionRule));
    assert!(nesting_and_custom_media_kinds.contains(&SyntaxKind::NestRule));
    assert!(nesting_and_custom_media_kinds.contains(&SyntaxKind::CustomMediaRule));
    assert!(nesting_and_custom_media_kinds.contains(&SyntaxKind::DeclarationList));
}

#[test]
fn validates_property_at_rule_names() {
    let valid = parse(
        "@property --accent { syntax: \"<color>\"; inherits: false; initial-value: red; }",
        StyleDialect::Css,
    );
    let dynamic = parse(
        "@property #{$name} { syntax: \"<color>\"; inherits: false; initial-value: red; }",
        StyleDialect::Scss,
    );
    let invalid = parse(
        "@property accent { syntax: \"<color>\"; inherits: false; initial-value: red; }",
        StyleDialect::Css,
    );
    let invalid_property_name_count = invalid
        .errors()
        .iter()
        .filter(|error| error.message == "invalid @property name")
        .count();

    assert!(valid.errors().is_empty());
    assert!(dynamic.errors().is_empty());
    assert_eq!(invalid_property_name_count, 1);
}

#[test]
fn validates_named_declaration_at_rule_preludes() {
    let valid = parse(
        "@counter-style thumbs { system: cyclic; symbols: \"yes\"; } @font-palette-values --brand { font-family: Demo; } @color-profile --display-p3 { src: url(p3.icc); } @position-try --popover { inset-area: top; } @custom-media --narrow (width < 40rem);",
        StyleDialect::Css,
    );
    let dynamic = parse(
        "@counter-style #{$style} { system: cyclic; symbols: \"yes\"; } @font-palette-values #{$palette} { font-family: Demo; } @custom-media #{$query} (width < 40rem);",
        StyleDialect::Scss,
    );
    let invalid = parse(
        "@counter-style --bad { system: cyclic; } @font-palette-values brand { font-family: Demo; } @color-profile display-p3 { src: url(p3.icc); } @position-try popover { inset-area: top; } @custom-media narrow (width < 40rem); @custom-media --missing;",
        StyleDialect::Css,
    );
    let custom_property_name_errors = invalid
        .errors()
        .iter()
        .filter(|error| error.message == "invalid at-rule custom property name")
        .count();
    let custom_media_prelude_errors = invalid
        .errors()
        .iter()
        .filter(|error| error.message == "invalid @custom-media prelude")
        .count();
    let counter_style_name_errors = invalid
        .errors()
        .iter()
        .filter(|error| error.message == "invalid @counter-style name")
        .count();

    assert!(valid.errors().is_empty());
    assert!(dynamic.errors().is_empty());
    assert_eq!(custom_property_name_errors, 3);
    assert_eq!(custom_media_prelude_errors, 2);
    assert_eq!(counter_style_name_errors, 1);
}

#[test]
fn validates_charset_and_namespace_at_rule_preludes() {
    let valid = parse(
        "@charset \"UTF-8\"; @namespace \"http://www.w3.org/1999/xhtml\"; @namespace svg url(\"http://www.w3.org/2000/svg\"); @namespace math url(http://www.w3.org/1998/Math/MathML);",
        StyleDialect::Css,
    );
    let dynamic = parse(
        "@namespace #{$url}; @namespace svg #{$url};",
        StyleDialect::Scss,
    );
    let invalid = parse("@charset UTF-8; @namespace svg;", StyleDialect::Css);
    let charset_errors = invalid
        .errors()
        .iter()
        .filter(|error| error.message == "invalid @charset prelude")
        .count();
    let namespace_errors = invalid
        .errors()
        .iter()
        .filter(|error| error.message == "invalid @namespace prelude")
        .count();

    assert!(valid.errors().is_empty());
    assert!(dynamic.errors().is_empty());
    assert_eq!(charset_errors, 1);
    assert_eq!(namespace_errors, 1);
}

#[test]
fn validates_keyframes_at_rule_names() {
    let valid = parse(
        "@keyframes fade { from { opacity: 0; } } @keyframes \"slide\" { to { opacity: 1; } }",
        StyleDialect::Css,
    );
    let dynamic = parse(
        "@keyframes #{$animation-name} { from { opacity: 0; } }",
        StyleDialect::Scss,
    );
    let invalid = parse(
        "@keyframes 50% { from { opacity: 0; } } @keyframes fade extra { to { opacity: 1; } }",
        StyleDialect::Css,
    );
    let invalid_name_errors = invalid
        .errors()
        .iter()
        .filter(|error| error.message == "invalid @keyframes name")
        .count();

    assert!(valid.errors().is_empty());
    assert!(dynamic.errors().is_empty());
    assert_eq!(invalid_name_errors, 2);
}

#[test]
fn validates_keyframe_selector_lists() {
    let valid = parse(
        "@keyframes fade { from { opacity: 0; } 50%, 75% { opacity: .5; } to { opacity: 1; } }",
        StyleDialect::Css,
    );
    let dynamic = parse(
        "@keyframes fade { #{$step} { opacity: .5; } }",
        StyleDialect::Scss,
    );
    let invalid = parse(
        "@keyframes fade { middle { opacity: .5; } 120px { opacity: 1; } 50%, { opacity: .8; } }",
        StyleDialect::Css,
    );
    let invalid_selector_errors = invalid
        .errors()
        .iter()
        .filter(|error| error.message == "invalid keyframe selector")
        .count();

    assert!(valid.errors().is_empty());
    assert!(dynamic.errors().is_empty());
    assert_eq!(invalid_selector_errors, 3);
}

#[test]
fn validates_empty_block_at_rule_preludes() {
    let valid = parse(
        "@font-face { font-family: Demo; } @starting-style { .card { opacity: 0; } } @view-transition { navigation: auto; } @page { @top-left { content: \"A\"; } } @font-feature-values Demo { @styleset { alt: 2; } }",
        StyleDialect::Css,
    );
    let invalid = parse(
        "@font-face Demo { font-family: Demo; } @starting-style demo { .card { opacity: 0; } } @view-transition demo { navigation: auto; } @page { @top-left header { content: \"A\"; } } @font-feature-values Demo { @styleset alt { alt: 2; } }",
        StyleDialect::Css,
    );
    let unexpected_prelude_errors = invalid
        .errors()
        .iter()
        .filter(|error| error.message == "unexpected at-rule prelude")
        .count();

    assert!(valid.errors().is_empty());
    assert_eq!(unexpected_prelude_errors, 5);
}

#[test]
fn validates_font_feature_values_preludes() {
    let valid = parse(
        "@font-feature-values Demo, \"Brand Font\" { @styleset { alt: 2; } }",
        StyleDialect::Css,
    );
    let dynamic = parse(
        "@font-feature-values #{$family} { @styleset { alt: 2; } }",
        StyleDialect::Scss,
    );
    let invalid = parse(
        "@font-feature-values { @styleset { alt: 2; } } @font-feature-values 123 { @styleset { alt: 2; } }",
        StyleDialect::Css,
    );
    let invalid_family_name_errors = invalid
        .errors()
        .iter()
        .filter(|error| error.message == "invalid @font-feature-values family name")
        .count();

    assert!(valid.errors().is_empty());
    assert!(dynamic.errors().is_empty());
    assert_eq!(invalid_family_name_errors, 2);
}

#[test]
fn classifies_initial_scss_at_rule_nodes() {
    let module_rules = parse(
        "@use \"sass:map\"; @forward \"tokens\";",
        StyleDialect::Scss,
    );
    let mixin_rule = parse("@mixin card($gap) { padding: $gap; }", StyleDialect::Scss);
    let module_kinds = node_kinds(&module_rules.syntax());
    let mixin_kinds = node_kinds(&mixin_rule.syntax());

    assert!(module_rules.errors().is_empty());
    assert!(mixin_rule.errors().is_empty());
    assert!(module_kinds.contains(&SyntaxKind::ScssUseRule));
    assert!(module_kinds.contains(&SyntaxKind::ScssForwardRule));
    assert!(mixin_kinds.contains(&SyntaxKind::ScssMixinDeclaration));
}

#[test]
fn parses_scss_module_config_preludes() {
    let result = parse(
        "@use \"theme\" as * with ($gap: 1rem, $enabled: true); @forward \"tokens\" as token-* show $color, mixin with ($color: red);",
        StyleDialect::Scss,
    );
    let kinds = node_kinds(&result.syntax());
    let config_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::ScssModuleConfig)
        .count();

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::ScssUseRule));
    assert!(kinds.contains(&SyntaxKind::ScssForwardRule));
    assert_eq!(config_count, 2);
}

#[test]
fn validates_scss_module_prelude_clauses() {
    let invalid = parse(
        "@use as *; @use \"theme\" as ; @use \"theme\" show foo; @forward \"tokens\" hide ; @forward \"tokens\" with $gap;",
        StyleDialect::Scss,
    );

    assert_eq!(
        invalid
            .errors()
            .iter()
            .filter(|error| error.message == "expected SCSS module source")
            .count(),
        1
    );
    assert_eq!(
        invalid
            .errors()
            .iter()
            .filter(|error| error.message == "expected SCSS module namespace")
            .count(),
        1
    );
    assert_eq!(
        invalid
            .errors()
            .iter()
            .filter(|error| error.message == "unexpected SCSS module visibility clause")
            .count(),
        1
    );
    assert_eq!(
        invalid
            .errors()
            .iter()
            .filter(|error| error.message == "expected SCSS module visibility name")
            .count(),
        1
    );
    assert_eq!(
        invalid
            .errors()
            .iter()
            .filter(|error| error.message == "expected SCSS module configuration")
            .count(),
        1
    );
}

#[test]
fn recovers_unclosed_scss_module_config_as_bogus() {
    let result = parse(
        "@use \"theme\" with ($gap: 1rem; .card { color: red; }",
        StyleDialect::Scss,
    );
    let kinds = node_kinds(&result.syntax());

    assert!(
        result
            .errors()
            .iter()
            .any(|error| error.message == "unterminated parenthesized prelude")
    );
    assert!(kinds.contains(&SyntaxKind::BogusScssModuleConfig));
    assert!(!kinds.contains(&SyntaxKind::ScssModuleConfig));
}

#[test]
fn parses_scss_placeholder_selectors_and_extend_refs() {
    let result = parse(
        "%button { color: red; } .primary { @extend %button; }",
        StyleDialect::Scss,
    );
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::ScssPlaceholderSelector));
    assert!(kinds.contains(&SyntaxKind::ScssExtendRule));
    assert!(token_kinds(&result.syntax()).contains(&SyntaxKind::ScssPlaceholder));
}

#[test]
fn captures_extend_target_facts_with_kind_and_optional_flag() {
    // RFC-0007-E1 (#45): the `@extend` target is captured as a fact (previously discarded).
    let facts = collect_style_facts(
        ".a { @extend %surface; } .b { @extend .missing; } .c { @extend %gone !optional; }",
        StyleDialect::Scss,
    );

    assert_eq!(facts.extend_target_count, 3);
    let captured = facts
        .extend_targets
        .iter()
        .map(|target| (target.kind, target.name.as_str(), target.optional))
        .collect::<Vec<_>>();
    assert!(captured.contains(&(ParsedExtendTargetFactKind::Placeholder, "surface", false)));
    assert!(captured.contains(&(ParsedExtendTargetFactKind::Class, "missing", false)));
    // `!optional` is recorded on the target so the validation rule can skip it.
    assert!(captured.contains(&(ParsedExtendTargetFactKind::Placeholder, "gone", true)));
}

#[test]
fn parses_structured_scss_at_rule_bodies() {
    let result = parse(
        "@mixin card($gap) { .item { gap: $gap; } } @function double($x) { @return $x * 2; } @if $enabled { .on { color: green; } } @for $i from 1 through 3 { .n { order: $i; } } @each $k, $v in $map { .e { color: $v; } } @while $enabled { .w { color: red; } }",
        StyleDialect::Scss,
    );
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::ScssMixinDeclaration));
    assert!(kinds.contains(&SyntaxKind::ScssFunctionDeclaration));
    assert!(kinds.contains(&SyntaxKind::ScssReturnRule));
    assert!(kinds.contains(&SyntaxKind::ScssControlIf));
    assert!(kinds.contains(&SyntaxKind::ScssControlFor));
    assert!(kinds.contains(&SyntaxKind::ScssControlEach));
    assert!(kinds.contains(&SyntaxKind::ScssControlWhile));
    assert!(kinds.contains(&SyntaxKind::DeclarationList));
    assert!(kinds.contains(&SyntaxKind::Rule));
    assert!(kinds.contains(&SyntaxKind::ClassSelector));
    assert!(kinds.contains(&SyntaxKind::ScssVariableReference));
}

#[test]
fn validates_scss_control_preludes() {
    let invalid = parse(
        "@if { .a { color: red; } } @while { .b { color: red; } } @for i from 1 through 3 { .c { color: red; } } @for $i from 1 { .d { color: red; } } @each item of $items { .e { color: red; } }",
        StyleDialect::Scss,
    );
    let invalid_control_prelude_count = invalid
        .errors()
        .iter()
        .filter(|error| error.message == "invalid SCSS control prelude")
        .count();

    assert_eq!(invalid_control_prelude_count, 5);
}

#[test]
fn extracts_scss_control_block_style_facts() {
    let facts = collect_style_facts(
        "@if $enabled { .on { color: green; } } @for $i from 1 through 3 { .n { order: $i; } } @each $k, $v in $map { .e { color: $v; } } @while $enabled { .w { color: red; } }",
        StyleDialect::Scss,
    );
    let class_names = facts
        .selectors
        .iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .map(|selector| selector.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(class_names, vec!["on", "n", "e", "w"]);
}

#[test]
fn extracts_scss_include_content_block_style_facts() {
    let source =
        ".card { @include interactive($tone) using ($state) { &--active { color: red; } } }";
    let parsed = parse(source, StyleDialect::Scss);
    let facts = collect_style_facts(source, StyleDialect::Scss);
    let class_names = facts
        .selectors
        .iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .map(|selector| selector.name.as_str())
        .collect::<Vec<_>>();

    assert!(parsed.errors().is_empty());
    assert!(node_kinds(&parsed.syntax()).contains(&SyntaxKind::ScssIncludeRule));
    assert_eq!(class_names, vec!["card", "card--active"]);
}

#[test]
fn parses_scss_nested_property_blocks() {
    let result = parse(
        ".card { font: { size: 1rem; weight: 600; } border: 1px solid { color: red; } }",
        StyleDialect::Scss,
    );
    let kinds = node_kinds(&result.syntax());
    let nested_property_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::ScssNestedProperty)
        .count();

    assert!(result.errors().is_empty());
    assert_eq!(nested_property_count, 2);
    assert!(kinds.contains(&SyntaxKind::DeclarationList));
    assert!(kinds.contains(&SyntaxKind::Value));
    assert!(kinds.contains(&SyntaxKind::DimensionValue));
}

#[test]
fn parses_sass_indented_nested_property_blocks() {
    let result = parse(
        ".card\n  font:\n    size: 1rem\n    weight: 600\n",
        StyleDialect::Sass,
    );
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::ScssNestedProperty));
    assert!(kinds.contains(&SyntaxKind::SassIndentedBlock));
    assert!(kinds.contains(&SyntaxKind::DeclarationList));
    assert!(kinds.contains(&SyntaxKind::DimensionValue));
}

#[test]
fn parses_scss_utility_at_rules() {
    let result = parse(
        "@mixin slot { @content; } @at-root { .rooted { color: red; } } @warn $message; @debug $message; @error $message;",
        StyleDialect::Scss,
    );
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::ScssContentRule));
    assert!(kinds.contains(&SyntaxKind::ScssAtRootRule));
    assert!(kinds.contains(&SyntaxKind::ScssWarnRule));
    assert!(kinds.contains(&SyntaxKind::ScssDebugRule));
    assert!(kinds.contains(&SyntaxKind::ScssErrorRule));
    assert!(kinds.contains(&SyntaxKind::Rule));
}

#[test]
fn structures_css_value_function_calls() {
    let result = parse(".a { width: calc(var(--gap) + 1rem); }", StyleDialect::Css);
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::Value));
    assert!(kinds.contains(&SyntaxKind::FunctionCall));
    assert!(kinds.contains(&SyntaxKind::FunctionArguments));
    assert!(kinds.contains(&SyntaxKind::CalcFunction));
    assert!(kinds.contains(&SyntaxKind::VarFunction));
    assert!(kinds.contains(&SyntaxKind::BinaryExpression));
}

#[test]
fn structures_modern_css_value_functions() {
    let result = parse(
        ".a { color: color-mix(in oklch, var(--brand), white 20%); accent-color: device-cmyk(0 1 1 0); width: clamp(1rem, 2vw, 3rem); margin: if(media(width >= 1px): 1rem; else: 2rem); content: attr(data-label string, \"x\"); padding: env(safe-area-inset-top); background-image: linear-gradient(red, blue); transform: translateX(1rem) rotate(10deg); filter: blur(2px) brightness(1.1); image-set: image-set(url(a.png) 1x); offset-path: path(\"M0,0 L1,1\"); }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());

    assert!(
        result.errors().is_empty(),
        "unexpected parse errors: {:?}",
        result.errors()
    );
    assert!(kinds.contains(&SyntaxKind::ColorValue));
    assert!(kinds.contains(&SyntaxKind::MathFunction));
    assert!(kinds.contains(&SyntaxKind::AttrFunction));
    assert!(kinds.contains(&SyntaxKind::EnvFunction));
    assert!(kinds.contains(&SyntaxKind::IfFunction));
    assert!(kinds.contains(&SyntaxKind::VarFunction));
    assert!(kinds.contains(&SyntaxKind::GradientFunction));
    assert!(kinds.contains(&SyntaxKind::TransformFunction));
    assert!(kinds.contains(&SyntaxKind::FilterFunction));
    assert!(kinds.contains(&SyntaxKind::ImageFunction));
    assert!(kinds.contains(&SyntaxKind::ShapeFunction));
}

#[test]
fn validates_color_function_micro_grammars() {
    let valid = parse(
        ".a { color: color-mix(in srgb, red, blue 30%); background: light-dark(white, black); border-color: contrast-color(red); }",
        StyleDialect::Css,
    );
    let dynamic = parse(
        ".a { color: color-mix(#{$space}, red, blue); }",
        StyleDialect::Scss,
    );
    let invalid = parse(
        ".a { color: color-mix(srgb, red, blue); background: light-dark(white); border-color: contrast-color(red, blue); outline-color: color-mix(in srgb, red); }",
        StyleDialect::Css,
    );
    let invalid_argument_head_count = invalid
        .errors()
        .iter()
        .filter(|error| error.message == "invalid function argument head")
        .count();
    let invalid_argument_count = invalid
        .errors()
        .iter()
        .filter(|error| error.message == "invalid function argument count")
        .count();

    assert!(valid.errors().is_empty());
    assert!(dynamic.errors().is_empty());
    assert_eq!(invalid_argument_head_count, 1);
    assert_eq!(invalid_argument_count, 3);
}

#[test]
fn classifies_css_value_functions_case_insensitively() {
    let result = parse(
        ".a { width: CALC(1px + 2px); color: COLOR-MIX(in srgb, red, blue); transform: TRANSLATEX(1px); filter: BLUR(2px); clip-path: POLYGON(0 0, 100% 0, 100% 100%); }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::CalcFunction));
    assert!(kinds.contains(&SyntaxKind::ColorValue));
    assert!(kinds.contains(&SyntaxKind::TransformFunction));
    assert!(kinds.contains(&SyntaxKind::FilterFunction));
    assert!(kinds.contains(&SyntaxKind::ShapeFunction));
}

#[test]
fn validates_values_l4_math_function_argument_counts() {
    let valid = parse(
        ".a { width: calc(1px + 2px); min-width: min(1px, 2px); max-width: max(1px); margin: round(nearest, 10px, 3px); padding: hypot(3px, 4px); opacity: log(8, 2); }",
        StyleDialect::Css,
    );
    let invalid = parse(
        ".a { width: calc(1px, 2px); min-width: min(); max-width: clamp(1px, 2px); margin: mod(10px); padding: sin(); opacity: atan2(1); }",
        StyleDialect::Css,
    );
    let invalid_argument_count = invalid
        .errors()
        .iter()
        .filter(|error| error.message == "invalid function argument count")
        .count();

    assert!(valid.errors().is_empty());
    assert_eq!(invalid_argument_count, 6);
}

#[test]
fn validates_values_l4_math_function_empty_arguments() {
    let valid_fallback = parse(
        ".a { color: var(--brand,); padding: env(safe-area-inset-top,); }",
        StyleDialect::Css,
    );
    let invalid = parse(
        ".a { width: min(, 1px); height: max(1px,); inset: clamp(1px, , 3px); }",
        StyleDialect::Css,
    );
    let empty_argument_count = invalid
        .errors()
        .iter()
        .filter(|error| error.message == "empty function argument")
        .count();

    assert!(valid_fallback.errors().is_empty());
    assert_eq!(empty_argument_count, 3);
}

#[test]
fn validates_var_env_attr_function_argument_heads() {
    let valid = parse(
        ".a { color: var(--brand, red, blue); padding: env(safe-area-inset-top, 0px); content: attr(data-label string, \"x\"); }",
        StyleDialect::Css,
    );
    let dynamic = parse(
        ".a { color: var(#{$name}); padding: env($area); content: attr(#{$attribute}); }",
        StyleDialect::Scss,
    );
    let invalid = parse(
        ".a { color: var(color); padding: env(, 0px); content: attr(123); }",
        StyleDialect::Css,
    );
    let invalid_head_count = invalid
        .errors()
        .iter()
        .filter(|error| error.message == "invalid function argument head")
        .count();

    assert!(valid.errors().is_empty());
    assert!(dynamic.errors().is_empty());
    assert_eq!(invalid_head_count, 3);
}

#[test]
fn structures_css_value_atoms_and_function_argument_lists() {
    let result = parse(
        ".a { color: #fff; width: clamp(1rem, calc(2px + 3px), 4rem); opacity: 50%; z-index: 1; font-family: system, \"Demo\"; unicode-range: U+00A0-00FF; }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());
    let dimension_value_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::DimensionValue)
        .count();
    let number_value_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::NumberValue)
        .count();
    let percentage_value_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::PercentageValue)
        .count();

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::ColorValue));
    assert!(kinds.contains(&SyntaxKind::ValueList));
    assert!(kinds.contains(&SyntaxKind::CalcFunction));
    assert!(kinds.contains(&SyntaxKind::BinaryExpression));
    assert!(kinds.contains(&SyntaxKind::IdentifierValue));
    assert!(kinds.contains(&SyntaxKind::StringValue));
    assert!(kinds.contains(&SyntaxKind::UnicodeRangeValue));
    assert!(dimension_value_count >= 4);
    assert!(number_value_count >= 1);
    assert!(percentage_value_count >= 1);
}

#[test]
fn parses_custom_property_values_as_component_value_lists() {
    let result = parse(
        ".a { --api: { display: none }; --empty: ; color: red; }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());
    let tokens = token_kinds(&result.syntax());
    let component_value_list_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::ComponentValueList)
        .count();

    assert!(result.errors().is_empty());
    assert!(tokens.contains(&SyntaxKind::CustomPropertyName));
    assert!(kinds.contains(&SyntaxKind::CustomPropertyValue));
    assert!(kinds.contains(&SyntaxKind::SimpleBlock));
    assert_eq!(component_value_list_count, 2);
    assert!(!kinds.contains(&SyntaxKind::BogusValue));
}

#[test]
fn structures_top_level_value_lists_without_function_comma_confusion() {
    let result = parse(
        ".a { font-family: system, sans-serif; color: color-mix(in oklch, red, blue); }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::ValueList));
    assert!(!kinds.contains(&SyntaxKind::BogusValueList));
    assert!(kinds.contains(&SyntaxKind::ColorValue));
}

#[test]
fn structures_bracketed_value_atoms_and_recovery() {
    let closed = parse(
        ".grid { grid-template-columns: [full-start] minmax(0, 1fr) [full-end]; }",
        StyleDialect::Css,
    );
    let missing_close = parse(
        ".grid { grid-template-columns: [full-start 1fr; }",
        StyleDialect::Css,
    );

    assert!(closed.errors().is_empty());
    assert!(node_kinds(&closed.syntax()).contains(&SyntaxKind::BracketedValue));
    assert!(node_kinds(&missing_close.syntax()).contains(&SyntaxKind::BogusBracketedValue));
}

#[test]
fn recovers_bogus_top_level_value_lists() {
    let result = parse(".a { font-family: system, ; }", StyleDialect::Css);
    let kinds = node_kinds(&result.syntax());

    assert!(kinds.contains(&SyntaxKind::BogusValueList));
}

#[test]
fn keeps_important_annotation_in_declaration_values() {
    let result = parse(".a { color: red !important; }", StyleDialect::Css);
    let split = parse(
        ".a { color: red ! /* keep */ important; }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());
    let split_kinds = node_kinds(&split.syntax());

    assert!(result.errors().is_empty());
    assert!(split.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::Declaration));
    assert!(kinds.contains(&SyntaxKind::Value));
    assert!(kinds.contains(&SyntaxKind::ImportantAnnotation));
    assert!(split_kinds.contains(&SyntaxKind::ImportantAnnotation));
    assert!(token_kinds(&result.syntax()).contains(&SyntaxKind::Important));
    assert!(token_kinds(&split.syntax()).contains(&SyntaxKind::Ident));
}

#[test]
fn structures_url_values() {
    let result = parse(
        ".a { background: url(images/bg.png); mask: url(\"icons/mask.svg\"); }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());
    let url_value_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::UrlValue)
        .count();

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::Value));
    assert!(kinds.contains(&SyntaxKind::FunctionCall));
    assert_eq!(url_value_count, 2);
    assert!(token_kinds(&result.syntax()).contains(&SyntaxKind::Url));
}

#[test]
fn structures_bad_strings_as_bogus_values() {
    let result = parse(".a { content: \"bad\ncolor: red; }", StyleDialect::Css);
    let kinds = node_kinds(&result.syntax());

    assert!(
        result
            .errors()
            .iter()
            .any(|error| error.code == ParseErrorCode::UnterminatedString)
    );
    assert!(kinds.contains(&SyntaxKind::BogusValue));
    assert!(token_kinds(&result.syntax()).contains(&SyntaxKind::BadString));
}

#[test]
fn structures_scss_interpolation_in_selector_property_and_value() {
    let result = parse(
        ".button-#{$variant} { #{$prop}: #{$value}; }",
        StyleDialect::Scss,
    );
    let kinds = node_kinds(&result.syntax());
    let interpolation_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::Interpolation)
        .count();

    assert!(result.errors().is_empty());
    assert_eq!(interpolation_count, 3);
    assert!(kinds.contains(&SyntaxKind::ClassSelector));
    assert!(kinds.contains(&SyntaxKind::PropertyName));
    assert!(kinds.contains(&SyntaxKind::Value));
}

#[test]
fn structures_less_interpolation_in_selector_property_and_value() {
    let result = parse(
        ".button-@{variant} { @{prop}: @{value}; }",
        StyleDialect::Less,
    );
    let kinds = node_kinds(&result.syntax());
    let interpolation_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::Interpolation)
        .count();

    assert!(result.errors().is_empty());
    assert_eq!(interpolation_count, 3);
    assert!(kinds.contains(&SyntaxKind::ClassSelector));
    assert!(kinds.contains(&SyntaxKind::PropertyName));
    assert!(kinds.contains(&SyntaxKind::Value));
}

#[test]
fn structures_less_escaped_strings_as_values() {
    let result = parse(".a { filter: ~\"alpha(opacity=50)\"; }", StyleDialect::Less);
    let kinds = node_kinds(&result.syntax());
    let token_kinds = token_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::Value));
    assert!(token_kinds.contains(&SyntaxKind::LessEscapedString));
}

#[test]
fn structures_less_property_variables_as_values() {
    let result = parse(".a { color: red; background: $color; }", StyleDialect::Less);
    let kinds = node_kinds(&result.syntax());
    let token_kinds = token_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::LessPropertyVariable));
    assert!(token_kinds.contains(&SyntaxKind::LessPropertyVariableToken));
}

#[test]
fn structures_unclosed_interpolation_as_bogus() {
    let scss = parse(".button-#{$variant", StyleDialect::Scss);
    let less = parse(".button-@{variant", StyleDialect::Less);

    assert!(node_kinds(&scss.syntax()).contains(&SyntaxKind::BogusInterpolation));
    assert!(node_kinds(&less.syntax()).contains(&SyntaxKind::BogusInterpolation));
    assert!(
        scss.errors()
            .iter()
            .any(|error| error.code == ParseErrorCode::UnexpectedCharacter)
    );
    assert!(
        less.errors()
            .iter()
            .any(|error| error.code == ParseErrorCode::UnexpectedCharacter)
    );
}

#[test]
fn structures_css_value_unary_and_precedence_expressions() {
    let result = parse(".a { margin: -(1rem + 2px) * 3; }", StyleDialect::Css);
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::UnaryExpression));
    assert!(kinds.contains(&SyntaxKind::ParenthesizedExpression));
    assert!(kinds.contains(&SyntaxKind::BinaryExpression));
}

#[test]
fn keeps_unary_prefix_binding_above_multiplication() {
    let max_infix_right_binding_power = [
        SyntaxKind::Plus,
        SyntaxKind::Minus,
        SyntaxKind::Star,
        SyntaxKind::Slash,
        SyntaxKind::Percent,
    ]
    .into_iter()
    .filter_map(infix_binding_power)
    .map(|(_, right_binding_power)| right_binding_power)
    .max();
    assert_eq!(max_infix_right_binding_power, Some(10));
    assert!(UNARY_PREFIX_RIGHT_BINDING_POWER > max_infix_right_binding_power.unwrap_or(0));

    let result = parse(".a { margin: - 2 * 3; }", StyleDialect::Css);
    assert!(result.errors().is_empty());

    let syntax = result.syntax();
    let has_expected_unary_left = syntax.descendants().any(|node| {
        if node.kind() != SyntaxKind::BinaryExpression
            || source_text(node).as_deref() != Some("- 2 * 3")
        {
            return false;
        }
        let child_kinds = node
            .children()
            .map(|child| child.kind())
            .collect::<Vec<_>>();
        child_kinds.first() == Some(&SyntaxKind::UnaryExpression)
            && node.children().any(|child| {
                child.kind() == SyntaxKind::UnaryExpression
                    && source_text(child)
                        .as_deref()
                        .is_some_and(|text| text.trim() == "- 2")
            })
    });

    assert!(has_expected_unary_left);
}

#[test]
fn structures_scss_value_comparison_and_logical_expressions() {
    let result = parse(
        ".a { width: $a + $b == $c; height: $x >= $y; color: $x != $y; margin: not $x or $y && $z; }",
        StyleDialect::Scss,
    );
    assert!(result.errors().is_empty(), "{:?}", result.errors());

    let syntax = result.syntax();
    let binary_texts = node_texts(&syntax, SyntaxKind::BinaryExpression);
    assert!(
        binary_texts.iter().any(|text| text == "$a + $b == $c"),
        "{binary_texts:?}",
    );
    assert!(
        binary_texts.iter().any(|text| text.trim() == "$a + $b"),
        "{binary_texts:?}",
    );
    assert!(
        binary_texts.iter().any(|text| text == "$x >= $y"),
        "{binary_texts:?}",
    );
    assert!(
        binary_texts.iter().any(|text| text == "$x != $y"),
        "{binary_texts:?}",
    );
    assert!(
        binary_texts.iter().any(|text| text == "not $x or $y && $z"),
        "{binary_texts:?}",
    );
    assert!(
        binary_texts.iter().any(|text| text == "$y && $z"),
        "{binary_texts:?}",
    );

    let comparison_left_kind = syntax
        .descendants()
        .find(|node| {
            node.kind() == SyntaxKind::BinaryExpression
                && source_text(node).as_deref() == Some("$a + $b == $c")
        })
        .and_then(|node| node.children().next().map(|child| child.kind()));
    assert_eq!(
        comparison_left_kind,
        Some(SyntaxKind::BinaryExpression),
        "{binary_texts:?}",
    );

    let unary_texts = node_texts(&syntax, SyntaxKind::UnaryExpression);
    assert!(unary_texts.iter().any(|text| text.trim() == "not $x"));
}

#[test]
fn keeps_comparison_tokens_partitioned_by_dialect_and_parser_context() {
    for dialect in [
        StyleDialect::Css,
        StyleDialect::Scss,
        StyleDialect::Sass,
        StyleDialect::Less,
    ] {
        let result = parse(partition_fixture_for_dialect(dialect), dialect);
        assert!(
            result.errors().is_empty(),
            "{dialect:?}: {:?}",
            result.errors()
        );

        let syntax = result.syntax();
        let binary_texts = node_texts(&syntax, SyntaxKind::BinaryExpression);
        assert!(
            !binary_texts.iter().any(|text| {
                text.contains("width >")
                    || text.contains("width >=")
                    || text.contains("a > b")
                    || text.contains("a || b")
            }),
            "{dialect:?}: unexpected context binary expressions: {binary_texts:?}",
        );

        let media_feature_texts = node_texts(&syntax, SyntaxKind::MediaFeature);
        assert!(
            media_feature_texts
                .iter()
                .any(|text| text.contains("width > 100px")),
            "{dialect:?}: {media_feature_texts:?}",
        );
        assert!(
            media_feature_texts
                .iter()
                .any(|text| text.contains("width >= 100px")),
            "{dialect:?}: {media_feature_texts:?}",
        );

        let container_texts = node_texts(&syntax, SyntaxKind::ContainerCondition);
        assert!(
            container_texts
                .iter()
                .any(|text| text.contains("width >= 1px")),
            "{dialect:?}: {container_texts:?}",
        );

        let combinator_texts = node_texts(&syntax, SyntaxKind::Combinator);
        assert!(
            combinator_texts.iter().any(|text| text == ">"),
            "{dialect:?}: {combinator_texts:?}",
        );
        assert!(
            combinator_texts.iter().any(|text| text == "||"),
            "{dialect:?}: {combinator_texts:?}",
        );
    }

    let css_value = parse(".a { width: a > b; gap: a || b; }", StyleDialect::Css);
    let css_binary_texts = node_texts(&css_value.syntax(), SyntaxKind::BinaryExpression);
    assert!(
        !css_binary_texts
            .iter()
            .any(|text| text == "a > b" || text == "a || b"),
        "{css_binary_texts:?}",
    );
}

#[test]
fn structures_sass_maps_lists_and_conditions() {
    let result = parse(
        "$theme: (gap: 1rem, color: red); $sizes: (1, 2, 3); .a { margin: 1 2 3; } @if $gap > 1rem { .a { color: red; } }",
        StyleDialect::Scss,
    );
    assert!(result.errors().is_empty(), "{:?}", result.errors());

    let syntax = result.syntax();
    let kinds = node_kinds(&syntax);
    assert!(kinds.contains(&SyntaxKind::ScssMap));
    assert!(kinds.contains(&SyntaxKind::ScssMapEntry));
    assert!(kinds.contains(&SyntaxKind::ScssList));
    assert!(kinds.contains(&SyntaxKind::ScssCondition));
    assert!(kinds.contains(&SyntaxKind::BinaryExpression));

    let map_texts = node_texts(&syntax, SyntaxKind::ScssMap);
    assert!(
        map_texts
            .iter()
            .any(|text| text == "(gap: 1rem, color: red)")
    );
    let list_texts = node_texts(&syntax, SyntaxKind::ScssList);
    assert!(
        list_texts.iter().any(|text| text == "(1, 2, 3)"),
        "{list_texts:?}"
    );
    assert!(
        list_texts.iter().any(|text| text.trim() == "1 2 3"),
        "{list_texts:?}",
    );
    let condition_texts = node_texts(&syntax, SyntaxKind::ScssCondition);
    assert!(
        condition_texts
            .iter()
            .any(|text| text.trim() == "$gap > 1rem")
    );
}

#[test]
fn recovers_bogus_sass_maps_lists_and_conditions() {
    let scss = parse(
        "$bad: (a: 1, b:); @if { .a { color: red; } }",
        StyleDialect::Scss,
    );
    let less = parse(".theme() when { color: red; }", StyleDialect::Less);
    let scss_kinds = node_kinds(&scss.syntax());
    let less_kinds = node_kinds(&less.syntax());

    assert!(scss_kinds.contains(&SyntaxKind::ScssMap));
    assert!(scss_kinds.contains(&SyntaxKind::BogusScssMapEntry));
    assert!(scss_kinds.contains(&SyntaxKind::BogusScssCondition));
    assert!(less_kinds.contains(&SyntaxKind::BogusLessCondition));
}

#[test]
fn structures_dialect_variable_references_in_values() {
    let scss = parse(".a { margin: $gap; }", StyleDialect::Scss);
    let less = parse(".a { margin: @gap; }", StyleDialect::Less);

    assert!(scss.errors().is_empty());
    assert!(less.errors().is_empty());
    assert!(node_kinds(&scss.syntax()).contains(&SyntaxKind::ScssVariableReference));
    assert!(node_kinds(&less.syntax()).contains(&SyntaxKind::LessVariableReference));
}

#[test]
fn structures_scss_variable_flags() {
    let result = parse(
        "$gap: 1rem ! /* keep */ default !global;",
        StyleDialect::Scss,
    );
    let kinds = node_kinds(&result.syntax());
    let flag_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::ScssVariableFlag)
        .count();

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::ScssVariableDeclaration));
    assert_eq!(flag_count, 2);
}

#[test]
fn parses_less_mixin_declarations_calls_and_guards() {
    let result = parse(
        ".theme(@color) when (iscolor(@color)) { color: @color; .rounded(); } .card { .theme(#fff); }",
        StyleDialect::Less,
    );
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::LessMixinDeclaration));
    assert!(kinds.contains(&SyntaxKind::LessMixinGuard));
    assert!(kinds.contains(&SyntaxKind::LessMixinCall));
    assert!(kinds.contains(&SyntaxKind::LessVariableReference));
    assert!(kinds.contains(&SyntaxKind::Rule));
}

#[test]
fn parses_less_extend_pseudo_class_without_mixin_confusion() {
    let less = parse(
        ".nav:extend(.inline all) { color: red; }",
        StyleDialect::Less,
    );
    let css = parse(
        ".nav:extend(.inline all) { color: red; }",
        StyleDialect::Css,
    );
    let less_kinds = node_kinds(&less.syntax());
    let css_kinds = node_kinds(&css.syntax());

    assert!(less.errors().is_empty());
    assert!(css.errors().is_empty());
    assert!(less_kinds.contains(&SyntaxKind::Rule));
    assert!(less_kinds.contains(&SyntaxKind::LessExtendRule));
    assert!(less_kinds.contains(&SyntaxKind::PseudoSelectorArgument));
    assert!(!less_kinds.contains(&SyntaxKind::LessMixinDeclaration));
    assert!(!css_kinds.contains(&SyntaxKind::LessExtendRule));
    assert!(css_kinds.contains(&SyntaxKind::PseudoClassSelector));
}

#[test]
fn parses_less_detached_ruleset_variable_values() {
    let result = parse(
        "@rules: { color: red; .rounded(); }; .card { color: blue; }",
        StyleDialect::Less,
    );
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::LessVariableDeclaration));
    assert!(kinds.contains(&SyntaxKind::LessDetachedRulesetNode));
    assert!(kinds.contains(&SyntaxKind::DeclarationList));
    assert!(kinds.contains(&SyntaxKind::Declaration));
    assert!(kinds.contains(&SyntaxKind::LessMixinCall));
    assert!(kinds.contains(&SyntaxKind::Rule));
}

#[test]
fn recovers_unclosed_less_detached_rulesets_as_bogus() {
    let result = parse("@rules: { color: red;", StyleDialect::Less);
    let kinds = node_kinds(&result.syntax());

    assert!(kinds.contains(&SyntaxKind::BogusLessDetachedRuleset));
    assert!(
        result
            .errors()
            .iter()
            .any(|error| error.code == ParseErrorCode::UnexpectedCharacter)
    );
}

#[test]
fn parses_less_namespace_access_calls() {
    let result = parse(
        ".card { #bundle > .rounded(); color: blue; }",
        StyleDialect::Less,
    );
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::LessNamespaceAccess));
    assert!(kinds.contains(&SyntaxKind::LessMixinCall));
    assert!(kinds.contains(&SyntaxKind::Declaration));
}

#[test]
fn keeps_nested_selectors_separate_from_less_namespace_access() {
    let result = parse(
        ".card { #child > .leaf { color: red; } }",
        StyleDialect::Less,
    );
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::Rule));
    assert!(!kinds.contains(&SyntaxKind::LessNamespaceAccess));
}

#[test]
fn extracts_initial_style_facts_from_parser_surface() {
    let facts = collect_style_facts(
        "@use \"tokens\"; $gap: 1rem; %surface { color: red; } .card#main { --space: $gap; }",
        StyleDialect::Scss,
    );

    assert_eq!(facts.product, "omena-parser.style-facts");
    assert_eq!(facts.dialect, StyleDialect::Scss);
    assert_eq!(facts.selector_count, 3);
    assert_eq!(facts.variable_count, 3);
    assert_eq!(facts.at_rule_count, 1);
    assert!(facts.selectors.iter().any(|selector| {
        selector.kind == ParsedSelectorFactKind::Class && selector.name == "card"
    }));
    assert!(facts.selectors.iter().any(|selector| {
        selector.kind == ParsedSelectorFactKind::Id && selector.name == "main"
    }));
    assert!(facts.selectors.iter().any(|selector| {
        selector.kind == ParsedSelectorFactKind::Placeholder && selector.name == "surface"
    }));
    assert!(facts.variables.iter().any(|variable| {
        variable.kind == ParsedVariableFactKind::ScssDeclaration && variable.name == "$gap"
    }));
    assert!(facts.variables.iter().any(|variable| {
        variable.kind == ParsedVariableFactKind::ScssReference && variable.name == "$gap"
    }));
    assert!(facts.variables.iter().any(|variable| {
        variable.kind == ParsedVariableFactKind::CustomPropertyDeclaration
            && variable.name == "--space"
    }));
    assert_eq!(facts.at_rules[0].node_kind, Some(SyntaxKind::ScssUseRule));
}

#[test]
fn summarizes_style_facts_as_parser_owned_product() {
    let summary = summarize_omena_parser_style_facts(
        "@use \"tokens\"; $gap: 1rem; .card { --space: $gap; }",
        StyleDialect::Scss,
    );

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-parser.style-facts");
    assert_eq!(summary.dialect, "scss");
    assert_eq!(summary.parser_error_count, 0);
    assert_eq!(summary.class_selector_names, vec!["card".to_string()]);
    assert_eq!(summary.variable_names, vec!["$gap".to_string()]);
    assert_eq!(summary.custom_property_names, vec!["--space".to_string()]);
    assert_eq!(summary.sass_module_use_sources, vec!["tokens".to_string()]);
}

#[test]
fn extracts_sass_symbol_style_facts() {
    let facts = collect_style_facts(
        "@mixin tone($color) { color: $color; } @function double($x) { @return $x * 2; } .card { @include tone(red); width: double(2px); }",
        StyleDialect::Scss,
    );
    let symbol_kinds = facts
        .sass_symbols
        .iter()
        .map(|symbol| (symbol.kind, symbol.name.as_str(), symbol.role))
        .collect::<Vec<_>>();

    assert_eq!(facts.sass_symbol_count, 8);
    assert!(symbol_kinds.contains(&(
        ParsedSassSymbolFactKind::MixinDeclaration,
        "tone",
        "declaration"
    )));
    assert!(symbol_kinds.contains(&(ParsedSassSymbolFactKind::MixinInclude, "tone", "include")));
    assert!(symbol_kinds.contains(&(
        ParsedSassSymbolFactKind::FunctionDeclaration,
        "double",
        "declaration"
    )));
    assert!(symbol_kinds.contains(&(ParsedSassSymbolFactKind::FunctionCall, "double", "call")));
    assert!(symbol_kinds.contains(&(
        ParsedSassSymbolFactKind::VariableDeclaration,
        "color",
        "declaration"
    )));
    assert!(symbol_kinds.contains(&(
        ParsedSassSymbolFactKind::VariableReference,
        "color",
        "reference"
    )));
}

#[test]
fn extracts_sass_function_calls_with_hyphen_underscore_equivalent_names() {
    let facts = collect_style_facts(
        "@function gap_value($x) { @return $x; } .card { width: gap-value(2px); }",
        StyleDialect::Scss,
    );
    let symbol_kinds = facts
        .sass_symbols
        .iter()
        .map(|symbol| (symbol.kind, symbol.name.as_str(), symbol.role))
        .collect::<Vec<_>>();

    assert!(symbol_kinds.contains(&(
        ParsedSassSymbolFactKind::FunctionDeclaration,
        "gap_value",
        "declaration"
    )));
    assert!(symbol_kinds.contains(&(ParsedSassSymbolFactKind::FunctionCall, "gap-value", "call")));
}

#[test]
fn each_loop_bindings_are_declarations_not_references() {
    // RFC-0007 #41 FP: `@each $k, $v in $map` -> $k/$v must be declarations
    // (bindings), and the iterable `$map` after `in` stays a reference.
    let facts = collect_style_facts(
        "@each $k, $v in $map { .e { color: $v; height: $k; } }",
        StyleDialect::Scss,
    );
    let symbol_kinds = facts
        .sass_symbols
        .iter()
        .map(|symbol| (symbol.kind, symbol.name.as_str()))
        .collect::<Vec<_>>();

    // Loop bindings are declarations.
    assert!(
        symbol_kinds.contains(&(ParsedSassSymbolFactKind::VariableDeclaration, "k")),
        "$k binding should be a declaration, got {symbol_kinds:?}"
    );
    assert!(
        symbol_kinds.contains(&(ParsedSassSymbolFactKind::VariableDeclaration, "v")),
        "$v binding should be a declaration, got {symbol_kinds:?}"
    );
    // The iterable after `in` stays a reference (over-correction guard).
    assert!(
        symbol_kinds.contains(&(ParsedSassSymbolFactKind::VariableReference, "map")),
        "$map iterable must remain a reference, got {symbol_kinds:?}"
    );
}

#[test]
fn for_loop_binding_is_declaration_and_bounds_stay_references() {
    // RFC-0007 #41 FP: `@for $i from $start through $end` -> $i is a binding;
    // $start/$end bounds remain references.
    let facts = collect_style_facts(
        "@for $i from $start through $end { .n { order: $i; } }",
        StyleDialect::Scss,
    );
    let symbol_kinds = facts
        .sass_symbols
        .iter()
        .map(|symbol| (symbol.kind, symbol.name.as_str()))
        .collect::<Vec<_>>();

    assert!(
        symbol_kinds.contains(&(ParsedSassSymbolFactKind::VariableDeclaration, "i")),
        "$i binding should be a declaration, got {symbol_kinds:?}"
    );
    // Bounds after `from`/`through` remain references (over-correction guard).
    assert!(
        symbol_kinds.contains(&(ParsedSassSymbolFactKind::VariableReference, "start")),
        "$start bound must remain a reference, got {symbol_kinds:?}"
    );
    assert!(
        symbol_kinds.contains(&(ParsedSassSymbolFactKind::VariableReference, "end")),
        "$end bound must remain a reference, got {symbol_kinds:?}"
    );
}

#[test]
fn while_and_undefined_references_still_flag_as_references() {
    // Over-correction guard: a genuinely undefined `$var` reference and a
    // `@while` condition variable must REMAIN references (so missingSassSymbol
    // still fires for true positives). Only `@each`/`@for` bindings are exempt.
    let facts = collect_style_facts(
        "@while $enabled { .w { color: $undefined; } }",
        StyleDialect::Scss,
    );
    let symbol_kinds = facts
        .sass_symbols
        .iter()
        .map(|symbol| (symbol.kind, symbol.name.as_str()))
        .collect::<Vec<_>>();

    // `@while` introduces no bindings -> condition var stays a reference.
    assert!(
        symbol_kinds.contains(&(ParsedSassSymbolFactKind::VariableReference, "enabled")),
        "@while condition must remain a reference, got {symbol_kinds:?}"
    );
    // A genuinely undefined reference still surfaces as a reference.
    assert!(
        symbol_kinds.contains(&(ParsedSassSymbolFactKind::VariableReference, "undefined")),
        "undefined var must remain a reference, got {symbol_kinds:?}"
    );
    // No false binding was introduced.
    assert!(
        !symbol_kinds
            .iter()
            .any(|(kind, _)| *kind == ParsedSassSymbolFactKind::VariableDeclaration),
        "no @while binding should be synthesized, got {symbol_kinds:?}"
    );
}

#[test]
fn each_single_binding_and_function_iterable_classification() {
    // Single-binding `@each $i in fn($x)` form: $i is a binding; the iterable
    // expression `$x` (inside a function call after `in`) stays a reference.
    let facts = collect_style_facts(
        "@each $i in to-list($x) { .e { order: $i; } }",
        StyleDialect::Scss,
    );
    let symbol_kinds = facts
        .sass_symbols
        .iter()
        .map(|symbol| (symbol.kind, symbol.name.as_str()))
        .collect::<Vec<_>>();

    assert!(
        symbol_kinds.contains(&(ParsedSassSymbolFactKind::VariableDeclaration, "i")),
        "$i binding should be a declaration, got {symbol_kinds:?}"
    );
    assert!(
        symbol_kinds.contains(&(ParsedSassSymbolFactKind::VariableReference, "x")),
        "$x inside the iterable must remain a reference, got {symbol_kinds:?}"
    );
}

#[test]
fn extracts_namespaced_sass_symbol_style_facts() {
    let facts = collect_style_facts(
        r#"@use "./tokens" as tokens; .card { color: tokens.$brand; @include tokens.tone(red); width: tokens.double(2px); }"#,
        StyleDialect::Scss,
    );
    let symbol_kinds = facts
        .sass_symbols
        .iter()
        .map(|symbol| {
            (
                symbol.kind,
                symbol.name.as_str(),
                symbol.role,
                symbol.namespace.as_deref(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(facts.sass_symbol_count, 3);
    assert!(symbol_kinds.contains(&(
        ParsedSassSymbolFactKind::VariableReference,
        "brand",
        "reference",
        Some("tokens")
    )));
    assert!(symbol_kinds.contains(&(
        ParsedSassSymbolFactKind::MixinInclude,
        "tone",
        "include",
        Some("tokens")
    )));
    assert!(symbol_kinds.contains(&(
        ParsedSassSymbolFactKind::FunctionCall,
        "double",
        "call",
        Some("tokens")
    )));
    assert_eq!(facts.sass_include_count, 1);
    assert_eq!(facts.sass_includes[0].name, "tone");
    assert_eq!(facts.sass_includes[0].namespace.as_deref(), Some("tokens"));
    assert_eq!(facts.sass_includes[0].params, "(red)");
}

#[test]
fn extracts_sass_module_edge_style_facts() {
    let facts = collect_style_facts(
        r#"@use "./tokens" as tokens; @use "./reset" as *; @use "sass:map"; @forward "./theme" show $brand, tone; @import "legacy", url("print.css");"#,
        StyleDialect::Scss,
    );

    assert_eq!(facts.sass_module_edge_count, 6);
    assert!(facts.sass_module_edges.iter().any(|edge| {
        edge.kind == ParsedSassModuleEdgeFactKind::Use
            && edge.source == "./tokens"
            && edge.namespace_kind == Some("alias")
            && edge.namespace.as_deref() == Some("tokens")
    }));
    assert!(facts.sass_module_edges.iter().any(|edge| {
        edge.kind == ParsedSassModuleEdgeFactKind::Use
            && edge.source == "./reset"
            && edge.namespace_kind == Some("wildcard")
            && edge.namespace.is_none()
    }));
    assert!(facts.sass_module_edges.iter().any(|edge| {
        edge.kind == ParsedSassModuleEdgeFactKind::Use
            && edge.source == "sass:map"
            && edge.namespace_kind == Some("default")
            && edge.namespace.as_deref() == Some("map")
    }));
    assert!(facts.sass_module_edges.iter().any(|edge| {
        edge.kind == ParsedSassModuleEdgeFactKind::Forward
            && edge.source == "./theme"
            && edge.visibility_filter_kind == Some("show")
            && edge.visibility_filter_names == vec!["brand", "tone"]
    }));
    assert!(facts.sass_module_edges.iter().any(|edge| {
        edge.kind == ParsedSassModuleEdgeFactKind::Import && edge.source == "legacy"
    }));
}

#[test]
fn captures_media_qualifier_on_sass_import_edge() {
    // RFC-0007-D1 (#44): a trailing media qualifier keeps the import as plain CSS.
    let facts = collect_style_facts(
        r#"@import "foo" screen; @import "bar" (min-width: 100px); @import "partial"; @import "a", "b" screen;"#,
        StyleDialect::Scss,
    );

    // (source, expected media_qualified): Ident qualifier (`screen`), paren
    // media-feature qualifier (`(min-width: 100px)`), bare partial (over-correction
    // guard — must stay unqualified), and the comma-peer pair `@import "a", "b" screen`
    // where only `"b"` carries the trailing qualifier (per-target classification).
    for (source, expected) in [
        ("foo", true),
        ("bar", true),
        ("partial", false),
        ("a", false),
        ("b", true),
    ] {
        assert!(
            facts.sass_module_edges.iter().any(|edge| {
                edge.kind == ParsedSassModuleEdgeFactKind::Import
                    && edge.source == source
                    && edge.media_qualified == expected
            }),
            "@import \"{source}\" should have media_qualified == {expected}"
        );
    }
}

#[test]
fn extracts_animation_name_style_facts() {
    let facts = collect_style_facts(
        "@keyframes fade { from { opacity: 0; } to { opacity: 1; } } @keyframes \"slide\" { to { opacity: 1; } } .card { animation-name: fade, \"slide\", none; }",
        StyleDialect::Css,
    );
    let keyframe_names = facts
        .animations
        .iter()
        .filter(|animation| animation.kind == ParsedAnimationFactKind::KeyframesDeclaration)
        .map(|animation| animation.name.as_str())
        .collect::<Vec<_>>();
    let reference_names = facts
        .animations
        .iter()
        .filter(|animation| animation.kind == ParsedAnimationFactKind::AnimationNameReference)
        .map(|animation| animation.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(facts.animation_count, 4);
    assert_eq!(keyframe_names, vec!["fade", "slide"]);
    assert_eq!(reference_names, vec!["fade", "slide"]);
}

#[test]
fn extracts_animation_shorthand_style_facts() {
    let facts = collect_style_facts(
        "@keyframes fade { to { opacity: 1; } } @keyframes \"slide\" { to { opacity: 1; } } .card { animation: 1s ease-in fade, \"slide\" 2s linear both, none 1s, var(--anim) 1s; }",
        StyleDialect::Css,
    );
    let keyframe_names = facts
        .animations
        .iter()
        .filter(|animation| animation.kind == ParsedAnimationFactKind::KeyframesDeclaration)
        .map(|animation| animation.name.as_str())
        .collect::<Vec<_>>();
    let reference_names = facts
        .animations
        .iter()
        .filter(|animation| animation.kind == ParsedAnimationFactKind::AnimationNameReference)
        .map(|animation| animation.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(facts.animation_count, 4);
    assert_eq!(keyframe_names, vec!["fade", "slide"]);
    assert_eq!(reference_names, vec!["fade", "slide"]);
}

#[test]
fn ignores_interpolated_unit_suffix_in_animation_shorthand() {
    // `#{$dur}s` lexes the trailing `s` as a bare ident; it is a duration unit, not the
    // animation name. Regression guard for RFC 0007-L (#52) missingKeyframes FP.
    let facts = collect_style_facts(
        "@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } } .a { animation: #{$dur}s fadeIn; }",
        StyleDialect::Scss,
    );
    let reference_names = facts
        .animations
        .iter()
        .filter(|animation| animation.kind == ParsedAnimationFactKind::AnimationNameReference)
        .map(|animation| animation.name.as_str())
        .collect::<Vec<_>>();

    // The unit `s` must NOT be picked up as an animation-name reference, but the real name must.
    assert!(
        !reference_names.contains(&"s"),
        "time unit `s` must not be misread as an animation name: {reference_names:?}"
    );
    assert_eq!(reference_names, vec!["fadeIn"]);
}

#[test]
fn ignores_interpolated_millisecond_unit_in_animation_shorthand() {
    let facts = collect_style_facts(
        "@keyframes fadeIn { to { opacity: 1; } } .a { animation: #{$dur}ms fadeIn; }",
        StyleDialect::Scss,
    );
    let reference_names = facts
        .animations
        .iter()
        .filter(|animation| animation.kind == ParsedAnimationFactKind::AnimationNameReference)
        .map(|animation| animation.name.as_str())
        .collect::<Vec<_>>();

    assert!(
        !reference_names.contains(&"ms"),
        "time unit `ms` must not be misread as an animation name: {reference_names:?}"
    );
    assert_eq!(reference_names, vec!["fadeIn"]);
}

#[test]
fn still_extracts_real_animation_name_with_literal_duration() {
    // Over-correction guard: a genuinely-missing `@keyframes` name (here `spinX`, never
    // declared) must STILL be extracted as a reference so it can be flagged downstream.
    // The literal `0.6s` duration must not suppress the real name.
    let facts = collect_style_facts(".a { animation: 0.6s spinX; }", StyleDialect::Scss);
    let reference_names = facts
        .animations
        .iter()
        .filter(|animation| animation.kind == ParsedAnimationFactKind::AnimationNameReference)
        .map(|animation| animation.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(reference_names, vec!["spinX"]);
}

#[test]
fn keeps_at_rule_header_dashed_idents_out_of_custom_property_facts() {
    let facts = collect_style_facts(
        "@property --accent { syntax: \"<color>\"; inherits: false; initial-value: red; } @font-palette-values --brand { font-family: Demo; } @color-profile --display-p3 { src: url(p3.icc); } @position-try --popover { inset-area: top; }",
        StyleDialect::Css,
    );
    let custom_properties: Vec<&str> = facts
        .variables
        .iter()
        .filter(|variable| {
            matches!(
                variable.kind,
                ParsedVariableFactKind::CustomPropertyDeclaration
                    | ParsedVariableFactKind::CustomPropertyReference
            )
        })
        .map(|variable| variable.name.as_str())
        .collect();

    assert_eq!(custom_properties, vec!["--accent"]);
}

#[test]
fn extracts_all_top_level_classes_from_complex_selector_headers() {
    let facts = collect_style_facts(
        "#app.theme > .card:has(> .icon) { color: red; }",
        StyleDialect::Css,
    );
    let class_names: Vec<&str> = facts
        .selectors
        .iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .map(|selector| selector.name.as_str())
        .collect();

    assert_eq!(class_names, vec!["theme", "card"]);
}

#[test]
fn extracts_css_nesting_at_rule_selector_facts() {
    let facts = collect_style_facts(
        ".card { @nest &__icon { color: red; &--active { color: blue; } } }",
        StyleDialect::Css,
    );
    let class_names: Vec<&str> = facts
        .selectors
        .iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .map(|selector| selector.name.as_str())
        .collect();

    assert_eq!(
        class_names,
        vec!["card", "card__icon", "card__icon--active"]
    );
}

#[test]
fn parses_mid_typing_char_boundary_edits_without_panicking() {
    let fixtures = [
        (
            StyleDialect::Css,
            ".card { color: color-mix(in oklch, red, blue); }",
        ),
        (
            StyleDialect::Scss,
            "@use \"tokens\" with ($gap: 1rem); .card { &__아이콘 { color: $gap; } }",
        ),
        (
            StyleDialect::Sass,
            ".card\n  color: red\n  &__icon\n    color: blue\n",
        ),
        (
            StyleDialect::Less,
            "@tone: red; .card() when (iscolor(@tone)) { color: @tone; }",
        ),
    ];
    let insertions = [" ", "{", "}", ":", "@media (", "한"];

    for (dialect, source) in fixtures {
        for offset in char_boundary_offsets(source) {
            for insertion in insertions {
                let mut edited = source.to_string();
                edited.insert_str(offset, insertion);
                let _ = parse(&edited, dialect);
            }
        }
    }
}

#[test]
fn parses_deterministic_malformed_byte_corpus_without_panicking() {
    let mut byte_fixtures = vec![
        Vec::new(),
        b"\0".to_vec(),
        b"\xef\xbb\xbf.card { color: red; }".to_vec(),
        b".a { content: \"unterminated".to_vec(),
        b".a { background: url(foo bar) }".to_vec(),
        b"@media screen { .a { color: red".to_vec(),
        b".a { --x: { [ ( ; }".to_vec(),
        vec![0xff, b'.', b'a', b' ', b'{', b'}'],
        vec![0xe1, 0x84, b'.', b'a', b'{', b'c', b':', b'r'],
    ];
    for seed in 0..32u32 {
        byte_fixtures.push(deterministic_byte_fixture(seed));
    }

    for bytes in byte_fixtures {
        let source = String::from_utf8_lossy(&bytes).into_owned();
        for dialect in [
            StyleDialect::Css,
            StyleDialect::Scss,
            StyleDialect::Sass,
            StyleDialect::Less,
        ] {
            let parse_result = std::panic::catch_unwind(|| parse(&source, dialect));
            assert!(
                parse_result.is_ok(),
                "parse panicked for dialect={dialect:?} source={source:?}"
            );
            let Ok(parse_result) = parse_result else {
                continue;
            };

            let lex_result = std::panic::catch_unwind(|| lex(&source, dialect));
            assert!(
                lex_result.is_ok(),
                "lex panicked for dialect={dialect:?} source={source:?}"
            );
            let Ok(lex_result) = lex_result else {
                continue;
            };

            assert_eq!(parse_result.syntax().kind(), SyntaxKind::Root);
            assert_lex_ranges_are_char_boundaries(&source, lex_result.tokens());
        }
    }
}

#[test]
fn preserves_lossless_cst_text_for_valid_corpus() {
    let fixtures = [
        (
            StyleDialect::Css,
            ".card { color: red; --space: calc(1rem + 2px); }",
        ),
        (
            StyleDialect::Scss,
            "@use \"tokens\"; .card { &__icon { color: $accent; } }",
        ),
        (
            StyleDialect::Sass,
            ".card\n  color: red\n  &__icon\n    color: blue\n",
        ),
        (
            StyleDialect::Less,
            "@tone: red; .card() when (iscolor(@tone)) { color: @tone; }",
        ),
    ];

    for (dialect, source) in fixtures {
        let result = parse(source, dialect);
        let syntax = result.syntax();

        assert_eq!(syntax.kind(), SyntaxKind::Root);
        assert_eq!(source_text(&syntax).as_deref(), Some(source));
        assert_eq!(result.source_text().as_deref(), Some(source));

        let reparsed = parse(&result.source_text().unwrap_or_default(), dialect);
        assert_eq!(reparsed.source_text().as_deref(), Some(source));
        assert_eq!(reparsed.syntax().kind(), SyntaxKind::Root);
    }
}

#[test]
fn extracts_nested_bem_style_facts_with_parent_context() {
    let facts = collect_style_facts(
        ".card { &__icon { &--small { color: red; } } --space: 1rem; color: var(--space); }",
        StyleDialect::Scss,
    );
    let class_names: Vec<&str> = facts
        .selectors
        .iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .map(|selector| selector.name.as_str())
        .collect();
    let custom_properties: Vec<&str> = facts
        .variables
        .iter()
        .map(|variable| variable.name.as_str())
        .collect();

    assert_eq!(class_names, vec!["card", "card__icon", "card__icon--small"]);
    assert!(custom_properties.contains(&"--space"));
    assert!(!custom_properties.contains(&"--small"));
    assert_eq!(facts.error_count, 0);
}

#[test]
fn extracts_non_bem_ampersand_suffix_style_facts() {
    let facts = collect_style_facts(
        ".btn { &-legacy {} &_legacy {} &suffix {} }",
        StyleDialect::Scss,
    );
    let class_names: Vec<&str> = facts
        .selectors
        .iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .map(|selector| selector.name.as_str())
        .collect();

    assert_eq!(
        class_names,
        vec!["btn", "btn-legacy", "btn_legacy", "btnsuffix"]
    );
    assert_eq!(facts.error_count, 0);
}

#[test]
fn ignores_non_defining_selector_function_arguments() {
    let facts = collect_style_facts(
        ".btn:is(.active, .primary):has(#target, %surface) { color: red; }",
        StyleDialect::Scss,
    );
    let class_names: Vec<&str> = facts
        .selectors
        .iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .map(|selector| selector.name.as_str())
        .collect();
    let id_names: Vec<&str> = facts
        .selectors
        .iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Id)
        .map(|selector| selector.name.as_str())
        .collect();
    let placeholder_names: Vec<&str> = facts
        .selectors
        .iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Placeholder)
        .map(|selector| selector.name.as_str())
        .collect();

    assert_eq!(class_names, vec!["btn"]);
    assert!(id_names.is_empty());
    assert!(placeholder_names.is_empty());
}

#[test]
fn filters_css_module_global_scope_selector_facts() {
    let facts = collect_style_facts(
        ":global { .reset { color: red; } } :global(.standalone) { color: red; } .card :global(.child) { color: red; } :local(.button) { color: blue; }",
        StyleDialect::Css,
    );
    let outer_local = collect_style_facts(
        ":local { :global { .kept { color: green; } } }",
        StyleDialect::Css,
    );
    let class_names = facts
        .selectors
        .iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .map(|selector| selector.name.as_str())
        .collect::<Vec<_>>();
    let outer_local_class_names = outer_local
        .selectors
        .iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .map(|selector| selector.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(class_names, vec!["card", "button"]);
    assert_eq!(outer_local_class_names, vec!["kept"]);
}

#[test]
fn extracts_css_module_local_id_selector_facts() {
    let facts = collect_style_facts(
        ":local(#panel) { color: red; } :global(#reset) { color: red; } .card :global(#child) { color: blue; }",
        StyleDialect::Css,
    );
    let class_names = facts
        .selectors
        .iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .map(|selector| selector.name.as_str())
        .collect::<Vec<_>>();
    let id_names = facts
        .selectors
        .iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Id)
        .map(|selector| selector.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(class_names, vec!["card"]);
    assert_eq!(id_names, vec!["panel"]);
}

#[test]
fn extracts_css_module_local_selector_list_facts() {
    let facts = collect_style_facts(
        ":local(.button, .link:hover) { color: red; } :global(.reset, .theme) { color: blue; }",
        StyleDialect::Css,
    );
    let class_names = facts
        .selectors
        .iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .map(|selector| selector.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(class_names, vec!["button", "link"]);
}

#[test]
fn keeps_trailing_local_selector_group_classes() {
    let facts = collect_style_facts(
        ":local(.button) .icon, :local(.card).active { color: red; }",
        StyleDialect::Css,
    );
    let mut class_names = facts
        .selectors
        .iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .map(|selector| selector.name.as_str())
        .collect::<Vec<_>>();
    class_names.sort_unstable();

    assert_eq!(class_names, vec!["active", "button", "card", "icon"]);
}

#[test]
fn parses_functional_pseudo_selector_lists_with_bogus_item_recovery() {
    let result = parse(
        ".btn:is(#it/typo, .ok):where(.wide, .compact) { color: red; }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());
    let selector_list_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::SelectorList)
        .count();
    let class_selector_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::ClassSelector)
        .count();

    assert!(kinds.contains(&SyntaxKind::Rule));
    assert!(kinds.contains(&SyntaxKind::Declaration));
    assert!(kinds.contains(&SyntaxKind::PseudoSelectorArgument));
    assert!(kinds.contains(&SyntaxKind::BogusSelector));
    assert!(!kinds.contains(&SyntaxKind::BogusRule));
    assert!(selector_list_count >= 3);
    assert!(class_selector_count >= 4);
    assert!(
        result
            .errors()
            .iter()
            .any(|error| error.message == "invalid selector in selector list")
    );
}

#[test]
fn parses_not_arguments_as_strict_selector_lists() {
    let forgiving = parse(".btn:is(#it/typo, .ok) { color: red; }", StyleDialect::Css);
    let strict = parse(".btn:not(#it/typo, .ok) { color: red; }", StyleDialect::Css);
    let forgiving_kinds = node_kinds(&forgiving.syntax());
    let strict_kinds = node_kinds(&strict.syntax());

    assert!(forgiving_kinds.contains(&SyntaxKind::BogusSelector));
    assert!(!forgiving_kinds.contains(&SyntaxKind::BogusSelectorList));
    assert!(strict_kinds.contains(&SyntaxKind::BogusSelector));
    assert!(strict_kinds.contains(&SyntaxKind::BogusSelectorList));
}

#[test]
fn parses_nth_child_of_selector_lists_as_cst_nodes() {
    let result = parse(
        ".grid > :nth-child(2n + 1 of .item, [data-active]) { color: red; }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());
    let selector_list_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::SelectorList)
        .count();

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::NthSelectorArgument));
    assert!(kinds.contains(&SyntaxKind::NthSelectorFormula));
    assert!(kinds.contains(&SyntaxKind::NthSelectorOfSelectorList));
    assert!(kinds.contains(&SyntaxKind::ClassSelector));
    assert!(kinds.contains(&SyntaxKind::AttributeSelector));
    assert!(selector_list_count >= 2);
}

#[test]
fn parses_nth_of_type_arguments_as_formula_cst_nodes() {
    let result = parse("li:nth-of-type(2n + 1) { color: red; }", StyleDialect::Css);
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::NthSelectorArgument));
    assert!(kinds.contains(&SyntaxKind::NthSelectorFormula));
    assert!(!kinds.contains(&SyntaxKind::NthSelectorOfSelectorList));
}

#[test]
fn parses_has_arguments_as_relative_selector_lists() {
    let result = parse(
        ".card:has(> .icon, + [data-active], :has(~ .nested)) { color: red; }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());
    let relative_selector_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::RelativeSelector)
        .count();
    let relative_list_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::RelativeSelectorList)
        .count();

    assert!(result.errors().is_empty());
    assert_eq!(relative_list_count, 2);
    assert_eq!(relative_selector_count, 4);
    assert!(kinds.contains(&SyntaxKind::Combinator));
    assert!(kinds.contains(&SyntaxKind::AttributeSelector));
    assert!(kinds.contains(&SyntaxKind::PseudoClassSelector));
}

#[test]
fn parses_lang_and_dir_arguments_as_cst_nodes() {
    let result = parse(
        ":lang(en-US, \"ko\") .card:dir(rtl) { color: red; }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());
    let language_tag_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::LanguageTag)
        .count();

    assert!(
        result.errors().is_empty(),
        "unexpected parse errors: {:?}",
        result.errors()
    );
    assert!(kinds.contains(&SyntaxKind::LanguageSelectorArgument));
    assert!(kinds.contains(&SyntaxKind::DirectionalitySelectorArgument));
    assert_eq!(language_tag_count, 2);
}

#[test]
fn decomposes_selector_lists_into_selector_nodes() {
    let result = parse(
        ".card:hover > #title, article.card || .icon[data-active] { color: red; }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::Selector));
    assert!(kinds.contains(&SyntaxKind::ComplexSelector));
    assert!(kinds.contains(&SyntaxKind::CompoundSelector));
    assert!(kinds.contains(&SyntaxKind::ClassSelector));
    assert!(kinds.contains(&SyntaxKind::IdSelector));
    assert!(kinds.contains(&SyntaxKind::TypeSelector));
    assert!(kinds.contains(&SyntaxKind::PseudoClassSelector));
    assert!(kinds.contains(&SyntaxKind::AttributeSelector));
    assert!(kinds.contains(&SyntaxKind::Combinator));
}

#[test]
fn parses_namespace_qualified_selectors() {
    let result = parse(
        "@namespace svg url(\"http://www.w3.org/2000/svg\"); svg|a, *|button, |main, svg|*, *|* { color: red; }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());
    let namespace_prefix_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::NamespacePrefix)
        .count();
    let type_selector_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::TypeSelector)
        .count();
    let universal_selector_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::UniversalSelector)
        .count();

    assert!(result.errors().is_empty());
    assert_eq!(namespace_prefix_count, 5);
    assert_eq!(type_selector_count, 3);
    assert_eq!(universal_selector_count, 2);
}

#[test]
fn decomposes_attribute_matchers_into_cst_nodes() {
    let result = parse(
        ".a[data-state~=\"active\"][lang|=\"en\"][href^=\"/docs\"][href$=\".pdf\"][class*=\"btn\"][data-mode=\"x\" i] { color: red; }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());
    let matcher_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::AttributeMatcher)
        .count();
    let name_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::AttributeName)
        .count();
    let value_count = kinds
        .iter()
        .filter(|kind| **kind == SyntaxKind::AttributeValue)
        .count();

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::AttributeSelector));
    assert_eq!(matcher_count, 6);
    assert_eq!(name_count, 6);
    assert_eq!(value_count, 6);
    assert!(kinds.contains(&SyntaxKind::AttributeModifier));
}

#[test]
fn decomposes_css_module_scope_functions_into_cst_nodes() {
    let result = parse(
        ":local(.button) { color: red; } :global(.reset) { box-sizing: border-box; }",
        StyleDialect::Css,
    );
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::PseudoClassSelector));
    assert!(kinds.contains(&SyntaxKind::PseudoSelectorArgument));
    assert!(kinds.contains(&SyntaxKind::CssModuleLocalBlock));
    assert!(kinds.contains(&SyntaxKind::CssModuleGlobalBlock));
}

#[test]
fn decomposes_nested_and_pseudo_element_selectors() {
    let result = parse("&::before { content: \"\"; }", StyleDialect::Scss);
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::NestingSelectorNode));
    assert!(kinds.contains(&SyntaxKind::PseudoElementSelector));
}

#[test]
fn parses_sass_indented_blocks_as_rule_declaration_lists() {
    let result = parse(
        ".card\n  color: red\n  .title\n    color: blue\n",
        StyleDialect::Sass,
    );
    let kinds = node_kinds(&result.syntax());

    assert!(result.errors().is_empty());
    assert!(kinds.contains(&SyntaxKind::SassIndentedBlock));
    assert!(kinds.contains(&SyntaxKind::Rule));
    assert!(kinds.contains(&SyntaxKind::DeclarationList));
    assert!(kinds.contains(&SyntaxKind::Declaration));
    assert!(kinds.contains(&SyntaxKind::ClassSelector));
}

#[test]
fn extracts_sass_indented_nested_bem_style_facts() {
    let facts = collect_style_facts(".card\n  &__icon\n    color: red\n", StyleDialect::Sass);
    let class_names: Vec<&str> = facts
        .selectors
        .iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .map(|selector| selector.name.as_str())
        .collect();

    assert_eq!(class_names, vec!["card", "card__icon"]);
    assert_eq!(facts.error_count, 0);
}

#[test]
fn exposes_typed_cst_wrapper_slice() {
    let result = parse(
        ".card { color: red; --accent: blue; } @media (width >= 1px) { .button { color: var(--accent); } }",
        StyleDialect::Css,
    );
    let cst = result.cst();
    let stylesheet = cst.stylesheet();
    let rules = cst.rules();
    let selectors = cst.selectors();
    let declarations = cst.declarations();
    let values = cst.values();
    let component_values = parse_entry_point(
        "calc(1px + 2px)",
        StyleDialect::Css,
        ParseEntryPoint::ComponentValue,
    )
    .cst()
    .component_values();
    let simple_blocks = parse_entry_point(
        "{ color: red; (width >= 1px) }",
        StyleDialect::Css,
        ParseEntryPoint::SimpleBlock,
    )
    .cst()
    .simple_blocks();
    let component_value_lists = parse_entry_point(
        "red calc(1px + 2px)",
        StyleDialect::Css,
        ParseEntryPoint::ComponentValueList,
    )
    .cst()
    .component_value_lists();
    let comma_separated_component_value_lists = parse_entry_point(
        "red, calc(1px + 2px)",
        StyleDialect::Css,
        ParseEntryPoint::CommaSeparatedComponentValueList,
    )
    .cst()
    .comma_separated_component_value_lists();
    let custom_property_values = result.cst().custom_property_values();
    let at_rules = cst.at_rules();

    assert_eq!(
        stylesheet.as_ref().map(TypedCstNode::kind),
        Some(SyntaxKind::Stylesheet)
    );
    assert_eq!(rules.len(), 2);
    assert_eq!(selectors.len(), 2);
    assert_eq!(declarations.len(), 3);
    assert_eq!(values.len(), 3);
    assert!(!component_values.is_empty());
    assert!(!simple_blocks.is_empty());
    assert!(!component_value_lists.is_empty());
    assert!(!comma_separated_component_value_lists.is_empty());
    assert_eq!(custom_property_values.len(), 1);
    assert!(!at_rules.is_empty());
    assert!(
        at_rules
            .iter()
            .any(|at_rule| at_rule.kind() == SyntaxKind::MediaRule)
    );
    assert!(
        stylesheet
            .and_then(|node| RuleCstNode::cast(node.into_syntax()))
            .is_none()
    );
}

#[test]
fn exposes_typed_bogus_cst_wrapper_slice() {
    let result = parse(".card { color: @; width: ?; }", StyleDialect::Css);
    let cst = result.cst();
    let bogus_kinds: Vec<SyntaxKind> = cst.bogus_nodes().iter().map(TypedCstNode::kind).collect();

    assert!(cst.has_bogus_nodes());
    assert!(bogus_kinds.contains(&SyntaxKind::BogusValue));
    assert!(bogus_kinds.contains(&SyntaxKind::BogusToken));
    assert!(bogus_kinds.iter().all(|kind| kind.is_bogus()));
}

#[test]
fn consumes_parser_style_fact_names_through_typed_interner() {
    let db = salsa::DatabaseImpl::default();
    let summary = summarize_parser_semantic_name_consumption(
        r#"@use "./tokens" as t;
@mixin tone { color: $brand; }
.button { --brand: red; animation: fade 1s; composes: base from "./base.module.css"; }
@keyframes fade { from { opacity: 0; } to { opacity: 1; } }"#,
        StyleDialect::Scss,
        &db,
    );

    assert_eq!(summary.product, "omena-parser.semantic-name-consumption");
    assert_eq!(summary.dialect, StyleDialect::Scss);
    assert_eq!(summary.invalid_name_count, 0);
    assert_eq!(summary.semantic_name_count, summary.interned_name_count);
    assert!(summary.class_name_count >= 2);
    assert!(summary.custom_property_name_count >= 1);
    assert!(summary.css_ident_count >= 1);
    assert!(summary.keyframes_name_count >= 1);
    assert!(summary.mixin_name_count >= 1);
    assert!(summary.file_path_count >= 1);
    assert!(
        summary
            .ready_surfaces
            .contains(&"parserSemanticNameConsumption")
    );
}

#[test]
fn summarizes_parser_cst_equivalence_contract() {
    let summary = summarize_parser_cst_equivalence(
        r#"@media (min-width: 1px) { .card { --tone: red; color: var(--tone); } }"#,
        StyleDialect::Css,
    );

    assert_eq!(summary.product, "omena-parser.cst-equivalence");
    assert_eq!(summary.dialect, StyleDialect::Css);
    assert_eq!(summary.root_kind, SyntaxKind::Root);
    assert!(summary.parser_node_count > 1);
    assert!(summary.parser_token_count > 1);
    assert!(summary.typed_wrapper_count > 4);
    assert!(summary.source_text_round_trip_ready);
    assert!(summary.syntax_kind_round_trip_ready);
    assert!(summary.zero_unknown_kind_ready);
    assert!(summary.typed_cst_wrapper_ready);
    assert!(summary.ready_surfaces.contains(&"parserCstEquivalence"));
}

#[test]
fn summarizes_green_field_parser_boundary() {
    let summary = summarize_parser_boundary();

    assert_eq!(summary.product, "omena-parser.boundary");
    assert_eq!(summary.dialect_count, 4);
    assert_eq!(summary.shared_name_kind_count, 8);
    assert!(summary.ready_surfaces.contains(&"selectorCstSkeleton"));
    assert!(summary.ready_surfaces.contains(&"lexedTokenTextSurface"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"recursiveDescentParserCore")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"recursiveDescentCoverageSummary")
    );
    assert!(summary.ready_surfaces.contains(&"atRuleRegistrySkeleton"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"prattValueExpressionSkeleton")
    );
    assert!(summary.ready_surfaces.contains(&"prattValueParserCore"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"prattValueCoverageSummary")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"attributeMatcherTokenization")
    );
    assert!(summary.ready_surfaces.contains(&"attributeMatcherCstNodes"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"attributeNameValueModifierCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"specializedValueFunctionCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"caseInsensitiveFunctionRegistry")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"caseInsensitiveAtRuleRegistry")
    );
    assert!(summary.ready_surfaces.contains(&"identifierValueCstNodes"));
    assert!(summary.ready_surfaces.contains(&"stringValueCstNodes"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"unicodeRangeValueCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"cssModuleScopeFunctionCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"cssModuleGlobalSelectorFactFiltering")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"cssModuleLocalIdSelectorFacts")
    );
    assert!(summary.ready_surfaces.contains(&"cssModuleValueStyleFacts"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"cssModuleValueDeclarationReferenceFacts")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"cssModuleComposesStyleFacts")
    );
    assert!(summary.ready_surfaces.contains(&"icssStyleFacts"));
    assert!(summary.ready_surfaces.contains(&"animationNameStyleFacts"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"animationShorthandStyleFacts")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"scssStructuredBlockAtRules")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"scssControlPreludeValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"scssControlStyleFactExtraction")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"scssIncludeContentBlockStyleFacts")
    );
    assert!(summary.ready_surfaces.contains(&"scssUtilityAtRules"));
    assert!(summary.ready_surfaces.contains(&"scssVariableFlagCstNodes"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"scssModulePreludeSourceValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"scssModulePreludeClauseValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"lessMixinDeclarationCstNodes")
    );
    assert!(summary.ready_surfaces.contains(&"lessMixinCallCstNodes"));
    assert!(summary.ready_surfaces.contains(&"lessMixinGuardCstNodes"));
    assert!(summary.ready_surfaces.contains(&"lessExtendPseudoCstNodes"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"lessDetachedRulesetCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"lessNamespaceAccessCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"lessPropertyVariableTokenization")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"lessPropertyVariableCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"lessEscapedStringTokenization")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"lessEscapedStringValueCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"importantAnnotationTokenization")
    );
    assert!(summary.ready_surfaces.contains(&"urlTokenization"));
    assert!(summary.ready_surfaces.contains(&"urlValueCstNodes"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"quotedUrlFunctionValueCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"conditionalAtRulePreludeCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"supportsAtRulePreludeValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"conditionalLevel5AtRuleCstNodes")
    );
    assert!(summary.ready_surfaces.contains(&"mediaQueryCstNodes"));
    assert!(summary.ready_surfaces.contains(&"mediaQueryListValidation"));
    assert!(summary.ready_surfaces.contains(&"importPreludeCstNodes"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"importSourcePreludeValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"importTailPreludeValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"customMediaPreludeValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"propertyAtRuleNameValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"namedAtRulePreludeValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"containerAtRulePreludeValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"charsetNamespaceAtRulePreludeValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"keyframesAtRuleNameValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"emptyBlockAtRulePreludeValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"layerScopePreludeCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"layerAtRulePreludeValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"scopeAtRulePreludeValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"pageAtRulePreludeValidation")
    );
    assert!(summary.ready_surfaces.contains(&"pageMarginAtRuleCstNodes"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"modernDeclarationAtRuleCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"fontFeatureValuesAtRuleCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"fontFeatureValuesPreludeValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"keyframeSelectorListValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"viewTransitionAtRuleCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"genericAtRulePreludeCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"bogusAtRulePreludeCstNodes")
    );
    assert!(summary.ready_surfaces.contains(&"nestingAtRuleCstNodes"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"customMediaAtRuleCstNodes")
    );
    assert!(summary.ready_surfaces.contains(&"cssColorFunctionCstNodes"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"colorFunctionArgumentChecks")
    );
    assert!(summary.ready_surfaces.contains(&"gradientFunctionCstNodes"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"transformFunctionCstNodes")
    );
    assert!(summary.ready_surfaces.contains(&"filterFunctionCstNodes"));
    assert!(summary.ready_surfaces.contains(&"imageFunctionCstNodes"));
    assert!(summary.ready_surfaces.contains(&"shapeFunctionCstNodes"));
    assert!(summary.ready_surfaces.contains(&"envAttrFunctionCstNodes"));
    assert!(summary.ready_surfaces.contains(&"mathFunctionCstNodes"));
    assert!(summary.ready_surfaces.contains(&"mathFunctionArityChecks"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"mathFunctionEmptyArgumentChecks")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"varEnvAttrFunctionHeadChecks")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"scssInterpolationTokenization")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"scssInterpolationCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"lessInterpolationTokenization")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"lessInterpolationCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"interpolationBogusRecovery")
    );
    assert!(summary.ready_surfaces.contains(&"unicodeRangeTokenization"));
    assert!(summary.ready_surfaces.contains(&"badStringTokenRecovery"));
    assert!(summary.ready_surfaces.contains(&"badStringValueBogusNodes"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"emptyDeclarationValueRecovery")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"emptyVariableValueRecovery")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"missingSemicolonDeclarationRecovery")
    );
    assert!(summary.ready_surfaces.contains(&"coreBogusPopulationSlice"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"dialectBogusPopulationSlice")
    );
    assert!(summary.ready_surfaces.contains(&"cssModuleValueCstNodes"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"cssModuleComposesCstNodes")
    );
    assert!(summary.ready_surfaces.contains(&"icssModuleBlockCstNodes"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"icssImportSourceValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"cssModuleFromClauseSourceValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"cssModuleComposesMultipleFromValidation")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"cssModuleGlobalComposesValidation")
    );
    assert!(summary.ready_surfaces.contains(&"cssModuleBogusRecovery"));
    assert!(summary.ready_surfaces.contains(&"valueListCstNodes"));
    assert!(summary.ready_surfaces.contains(&"valueListBogusRecovery"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"genericRecoveryBogusNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"lightningCssDifferentialCorpusSlice")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"midTypingNoPanicPropertySlice")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"deterministicPanicFreeCorpus")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"losslessCstTextRoundTripSmoke")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"parseResultSourceTextSurface")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"parseSourceParseRoundTripSmoke")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"typedNumericValueAtomCstNodes")
    );
    assert!(summary.ready_surfaces.contains(&"bracketedValueCstNodes"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"importantAnnotationCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"splitImportantAnnotationCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"unexpectedValueTokenBogusNodes")
    );
    assert!(summary.ready_surfaces.contains(&"cdoCdcTokenization"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"cssIdentifierEscapeTokenization")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"nullAndBomInputPreprocessingSlice")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"hashDelimiterTokenization")
    );
    assert!(summary.ready_surfaces.contains(&"cssDashIdentTokenization"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"signedNumericTokenization")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"exponentNumericTokenization")
    );
    assert!(summary.ready_surfaces.contains(&"badUrlWhitespaceRecovery"));
    assert!(summary.ready_surfaces.contains(&"parserEntryPointApiSlice"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"ruleListEntryPointApiSlice")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"componentValueEntryPointApiSlice")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"componentValueListEntryPointApiSlice")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"commaSeparatedComponentValueListEntryPointApiSlice")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"simpleBlockEntryPointApiSlice")
    );
    assert!(summary.ready_surfaces.contains(&"typedCstWrapperSlice"));
    assert!(summary.ready_surfaces.contains(&"parserCstEquivalence"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"typedBogusCstWrapperSlice")
    );
    assert!(summary.ready_surfaces.contains(&"componentValueCstNodes"));
    assert!(summary.ready_surfaces.contains(&"simpleBlockCstNodes"));
    assert!(summary.ready_surfaces.contains(&"fullBogusPopulation"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"componentValueListCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"commaSeparatedComponentValueListCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"customPropertyAnyValueComponentList")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"customPropertyValueCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"functionalPseudoSelectorListCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"strictNotPseudoSelectorListCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"nthSelectorOfSelectorListCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"nthSelectorFormulaCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"hasRelativeSelectorListCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"langDirSelectorArgumentCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"namespaceQualifiedSelectorCstNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"selectorFunctionArgumentFactExclusion")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"missingBlockCloseBogusTrivia")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"initialDialectStatementNodes")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"scssNestedPropertyCstNodes")
    );
    assert!(summary.ready_surfaces.contains(&"scssModuleConfigCstNodes"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"scssModuleConfigBogusRecovery")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"scssPlaceholderSelectorCstNodes")
    );
    assert!(summary.ready_surfaces.contains(&"recoveryBogusSkeleton"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"styleFactExtractionSurface")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"parserSemanticNameConsumption")
    );
    assert!(summary.ready_surfaces.contains(&"differentialCorpus"));
    assert!(!summary.not_ready_surfaces.contains(&"differentialCorpus"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"lightningCssSelectorIdAndAtRuleDifferentialSlice")
    );
    assert!(!summary.not_ready_surfaces.contains(&"fullPrattValueParser"));
    assert!(
        summary
            .not_ready_surfaces
            .contains(&"fullPropertyValueGrammarRegistry")
    );
    assert!(
        !summary
            .not_ready_surfaces
            .contains(&"fullRecursiveDescentGrammar")
    );
    assert!(
        summary
            .not_ready_surfaces
            .contains(&"completeExternalSpecMirror")
    );
    assert!(summary.ready_surfaces.contains(&"productCutoverGate"));
    assert!(!summary.not_ready_surfaces.contains(&"productCutover"));
}

#[test]
fn summarizes_recursive_descent_parser_coverage_without_claiming_full_spec_mirror() {
    let summary = summarize_recursive_descent_parser_coverage();

    assert_eq!(summary.product, "omena-parser.recursive-descent-coverage");
    assert_eq!(summary.dialect_count, 4);
    assert_eq!(summary.entry_point_count, 10);
    assert!(summary.selector_surface_count >= 12);
    assert!(summary.at_rule_surface_count >= 19);
    assert!(summary.dialect_extension_surface_count >= 17);
    assert!(summary.recovery_surface_count >= 8);
    assert!(
        summary
            .ready_surfaces
            .contains(&"recursiveDescentParserCore")
    );
    assert!(summary.ready_surfaces.contains(&"sassIndentedBlocks"));
    assert!(
        summary
            .next_surfaces
            .contains(&"completeExternalSpecMirror")
    );
}

#[test]
fn summarizes_pratt_value_parser_coverage_without_overclaiming_property_grammar() {
    let summary = summarize_pratt_value_parser_coverage();

    assert_eq!(summary.product, "omena-parser.pratt-value-coverage");
    assert!(summary.infix_operator_kinds.contains(&SyntaxKind::Plus));
    assert!(summary.infix_operator_kinds.contains(&SyntaxKind::Star));
    assert!(summary.prefix_operator_kinds.contains(&SyntaxKind::Minus));
    assert!(
        summary
            .value_expression_node_kinds
            .contains(&SyntaxKind::BinaryExpression)
    );
    assert!(
        summary
            .value_expression_node_kinds
            .contains(&SyntaxKind::FunctionArguments)
    );
    assert!(summary.specialized_function_family_count >= 10);
    assert!(summary.css_values_l4_math_function_count >= 20);
    assert!(summary.css_color_function_count >= 14);
    assert!(summary.ready_surfaces.contains(&"prattValueParserCore"));
    assert!(
        summary
            .next_surfaces
            .contains(&"fullPropertyValueGrammarRegistry")
    );
}

fn char_boundary_offsets(source: &str) -> Vec<usize> {
    source
        .char_indices()
        .map(|(offset, _)| offset)
        .chain(std::iter::once(source.len()))
        .collect()
}

fn deterministic_byte_fixture(seed: u32) -> Vec<u8> {
    let mut state = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    let len = (state as usize % 96) + 1;
    let mut bytes = Vec::with_capacity(len);
    for _ in 0..len {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        bytes.push((state >> 24) as u8);
    }
    bytes
}

fn keyframes_declaration_names(facts: &ParsedStyleFacts) -> Vec<&str> {
    facts
        .animations
        .iter()
        .filter(|animation| animation.kind == ParsedAnimationFactKind::KeyframesDeclaration)
        .map(|animation| animation.name.as_str())
        .collect()
}

fn animation_name_references(facts: &ParsedStyleFacts) -> Vec<&str> {
    facts
        .animations
        .iter()
        .filter(|animation| animation.kind == ParsedAnimationFactKind::AnimationNameReference)
        .map(|animation| animation.name.as_str())
        .collect()
}

// RFC-0007-C / #43 C1: a vendor-prefixed `@-webkit-keyframes` registers the same bare
// keyframes name as the unprefixed `@keyframes`, so `animation: spin` resolves (no FP).
#[test]
fn vendor_prefixed_keyframes_registers_bare_name() {
    let facts = collect_style_facts(
        "@-webkit-keyframes spin { from { opacity: 0; } to { opacity: 1; } } .x { animation: spin 1s linear; }",
        StyleDialect::Scss,
    );
    assert!(keyframes_declaration_names(&facts).contains(&"spin"));

    // -moz- prefix is normalized identically.
    let moz = collect_style_facts(
        "@-moz-keyframes pulse { from { opacity: 0; } to { opacity: 1; } }",
        StyleDialect::Css,
    );
    assert!(keyframes_declaration_names(&moz).contains(&"pulse"));
}

// RFC-0007-C / #43 C1 over-correction guard: a genuinely missing `@keyframes` (no
// declaration of any prefix) must still surface as an unresolved animation-name reference.
#[test]
fn missing_keyframes_without_declaration_still_referenced() {
    let facts = collect_style_facts(".x { animation: spin 1s linear; }", StyleDialect::Scss);
    let declared = keyframes_declaration_names(&facts);
    let referenced = animation_name_references(&facts);
    assert!(!declared.contains(&"spin"));
    assert!(referenced.contains(&"spin"));
}

// RFC-0007-C / #43 C2: a literal fragment immediately adjacent to an interpolation
// (`#{$p}-spin` post-interpolation, `spin-#{$p}` pre-interpolation) is part of a
// statically-unknown name and must not be emitted as a keyframes reference.
#[test]
fn interpolation_adjacent_fragment_is_not_a_keyframes_reference() {
    let post = collect_style_facts(
        "$p: brand; .x { animation: #{$p}-spin 1s; }",
        StyleDialect::Scss,
    );
    assert!(animation_name_references(&post).is_empty());

    let pre = collect_style_facts(
        "$p: brand; .x { animation: spin-#{$p} 1s; }",
        StyleDialect::Scss,
    );
    assert!(animation_name_references(&pre).is_empty());

    // animation-name longhand path is covered by the same guard.
    let longhand = collect_style_facts(
        "$p: brand; .x { animation-name: #{$p}-spin; }",
        StyleDialect::Scss,
    );
    assert!(animation_name_references(&longhand).is_empty());
}

// RFC-0007-C / #43 C2 over-correction guard: a fully-static name merely *near* an
// interpolation but separated from it by whitespace (`#{$p} spin`) is a real
// space-delimited keyframes reference and must NOT be suppressed.
#[test]
fn static_name_separated_from_interpolation_is_a_keyframes_reference() {
    let facts = collect_style_facts(
        "$p: brand; .x { animation: #{$p} spin 1s; }",
        StyleDialect::Scss,
    );
    assert!(animation_name_references(&facts).contains(&"spin"));
}

fn custom_property_reference_fallback(facts: &ParsedStyleFacts, name: &str) -> Option<bool> {
    facts
        .variables
        .iter()
        .find(|fact| {
            fact.kind == ParsedVariableFactKind::CustomPropertyReference && fact.name == name
        })
        .map(|fact| fact.has_fallback)
}

// RFC-0007-C / #43 C3: a `var(--x, fallback)` reference carries a `has_fallback` bit so the
// `missingCustomProperty` lint can skip it (the fallback guarantees a value).
#[test]
fn var_reference_with_fallback_sets_has_fallback_bit() {
    let facts = collect_style_facts(
        ".x { --declared: red; color: var(--undeclared, blue); }",
        StyleDialect::Css,
    );
    assert_eq!(
        custom_property_reference_fallback(&facts, "--undeclared"),
        Some(true)
    );
}

// RFC-0007-C / #43 C3 over-correction guard: a fallback-less `var(--x)` keeps
// `has_fallback == false` so a genuinely missing custom property still fires.
#[test]
fn var_reference_without_fallback_keeps_has_fallback_false() {
    let facts = collect_style_facts(
        ".x { --declared: red; color: var(--undeclared); }",
        StyleDialect::Css,
    );
    assert_eq!(
        custom_property_reference_fallback(&facts, "--undeclared"),
        Some(false)
    );
}

// RFC-0007-C / #43 C3 over-correction guard: per-`var()` scoping. In
// `var(--a, var(--b))` only the outer `--a` carries a fallback; the nested fallback-less
// `--b` stays a live `missingCustomProperty` candidate.
#[test]
fn nested_var_fallback_is_scoped_per_call() {
    let facts = collect_style_facts(
        ".x { --declared: red; color: var(--a, var(--b)); }",
        StyleDialect::Css,
    );
    assert_eq!(
        custom_property_reference_fallback(&facts, "--a"),
        Some(true)
    );
    assert_eq!(
        custom_property_reference_fallback(&facts, "--b"),
        Some(false)
    );
}

#[test]
fn reusable_parse_cache_shares_exact_green_nodes_after_small_edit() {
    let mut cache = ParseReuseCache::default();
    let first = parse_with_reuse_cache(
        ".alpha { color: red; } .beta { color: blue; }",
        StyleDialect::Css,
        &mut cache,
    );
    let second = parse_with_reuse_cache(
        ".alpha { color: green; } .beta { color: blue; }",
        StyleDialect::Css,
        &mut cache,
    );

    assert_eq!(
        shared_green_node_storage_count_for_test(first.green(), second.green()),
        16
    );
}

#[test]
fn reusable_parse_cache_shares_exact_green_nodes_after_sparse_edit() {
    let mut cache = ParseReuseCache::default();
    let first = parse_with_reuse_cache(
        ".card { color: red; padding: 1rem; margin: 2rem; border-color: black; } .icon { width: 1rem; height: 1rem; }",
        StyleDialect::Css,
        &mut cache,
    );
    let second = parse_with_reuse_cache(
        ".card { color: red; padding: 1rem; margin: 3rem; border-color: black; } .icon { width: 1rem; height: 1rem; }",
        StyleDialect::Css,
        &mut cache,
    );

    assert_eq!(
        shared_green_node_storage_count_for_test(first.green(), second.green()),
        26
    );
}

#[test]
fn syntax_and_hir_ids_stay_stable_for_unchanged_region_and_change_for_edit() {
    let first = parse(
        ".alpha { color: red; } .beta { color: blue; }",
        StyleDialect::Css,
    );
    let second = parse(
        ".alpha { color: green; } .beta { color: blue; }",
        StyleDialect::Css,
    );
    let first_syntax = first.syntax();
    let second_syntax = second.syntax();
    let first_alpha = rule_node_containing_for_test(&first_syntax, ".alpha");
    let second_alpha = rule_node_containing_for_test(&second_syntax, ".alpha");
    let first_beta = rule_node_containing_for_test(&first_syntax, ".beta");
    let second_beta = rule_node_containing_for_test(&second_syntax, ".beta");
    assert!(first_alpha.is_some(), "missing first alpha rule");
    assert!(second_alpha.is_some(), "missing second alpha rule");
    assert!(first_beta.is_some(), "missing first beta rule");
    assert!(second_beta.is_some(), "missing second beta rule");
    let first_alpha = first_alpha.unwrap_or(&first_syntax);
    let second_alpha = second_alpha.unwrap_or(&second_syntax);
    let first_beta = first_beta.unwrap_or(&first_syntax);
    let second_beta = second_beta.unwrap_or(&second_syntax);

    let first_beta_id = syntax_node_id(first_beta);
    let second_beta_id = syntax_node_id(second_beta);
    assert_eq!(
        first_beta_id.as_str().as_bytes(),
        second_beta_id.as_str().as_bytes()
    );
    assert_eq!(
        hir_id_for_syntax_node(first_beta).as_str().as_bytes(),
        hir_id_for_syntax_node(second_beta).as_str().as_bytes()
    );

    assert_ne!(syntax_node_id(first_alpha), syntax_node_id(second_alpha));
    assert_ne!(
        hir_id_for_syntax_node(first_alpha),
        hir_id_for_syntax_node(second_alpha)
    );
}

fn assert_lex_ranges_are_char_boundaries(source: &str, tokens: &[LexedToken]) {
    for token in tokens {
        let start = u32::from(token.range.start()) as usize;
        let end = u32::from(token.range.end()) as usize;
        assert!(
            source.is_char_boundary(start),
            "token start is not a char boundary: token={token:?} source={source:?}"
        );
        assert!(
            source.is_char_boundary(end),
            "token end is not a char boundary: token={token:?} source={source:?}"
        );
    }
}

fn source_text(node: &SyntaxNode<SyntaxKind>) -> Option<String> {
    let mut text = String::new();
    for token in node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
    {
        if let Some(resolver) = token.resolver() {
            text.push_str(token.resolve_text(&**resolver));
        } else if let Some(static_text) = token.static_text() {
            text.push_str(static_text);
        } else {
            return None;
        }
    }
    Some(text)
}

#[allow(unsafe_code)]
fn green_node_storage_key_for_test(node: &GreenNode) -> usize {
    assert_eq!(
        std::mem::size_of::<GreenNode>(),
        std::mem::size_of::<usize>()
    );
    // cstree exposes structural equality but not green-node pointer identity.
    // This test-only gate observes the internal ThinArc pointer so cache reuse
    // is checked by exact storage sharing instead of by value equality.
    unsafe { *(node as *const GreenNode as *const usize) }
}

fn collect_green_node_storage_keys_for_test(node: &GreenNode, keys: &mut Vec<usize>) {
    keys.push(green_node_storage_key_for_test(node));
    for child in node.children() {
        if let Some(&child_node) = child.as_node() {
            collect_green_node_storage_keys_for_test(child_node, keys);
        }
    }
}

fn shared_green_node_storage_count_for_test(first: &GreenNode, second: &GreenNode) -> usize {
    let mut first_counts = BTreeMap::new();
    let mut first_keys = Vec::new();
    collect_green_node_storage_keys_for_test(first, &mut first_keys);
    for key in first_keys {
        *first_counts.entry(key).or_insert(0usize) += 1;
    }

    let mut second_keys = Vec::new();
    collect_green_node_storage_keys_for_test(second, &mut second_keys);
    let mut shared = 0usize;
    for key in second_keys {
        if let Some(count) = first_counts.get_mut(&key)
            && *count > 0
        {
            *count -= 1;
            shared += 1;
        }
    }
    shared
}

fn rule_node_containing_for_test<'a>(
    root: &'a SyntaxNode<SyntaxKind>,
    needle: &str,
) -> Option<&'a SyntaxNode<SyntaxKind>> {
    root.descendants().find(|node| {
        node.kind() == SyntaxKind::Rule
            && node
                .try_resolved()
                .map(|resolved| resolved.text().to_string().contains(needle))
                .unwrap_or(false)
    })
}

fn node_kinds(node: &SyntaxNode<SyntaxKind>) -> Vec<SyntaxKind> {
    let mut kinds = vec![node.kind()];
    for child in node.children() {
        kinds.extend(node_kinds(child));
    }
    kinds
}

fn node_texts(node: &SyntaxNode<SyntaxKind>, kind: SyntaxKind) -> Vec<String> {
    node.descendants()
        .filter(|node| node.kind() == kind)
        .filter_map(source_text)
        .collect()
}

fn token_kinds(node: &SyntaxNode<SyntaxKind>) -> Vec<SyntaxKind> {
    node.descendants_with_tokens()
        .filter_map(|element| element.into_token().map(|token| token.kind()))
        .collect()
}

fn partition_fixture_for_dialect(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Sass => {
            "@media (width > 100px)\n  a > b, a || b\n    color: red\n@media (width >= 100px)\n  .m\n    color: red\n@container (width >= 1px)\n  .c\n    color: blue\n"
        }
        StyleDialect::Css | StyleDialect::Scss | StyleDialect::Less => {
            "@media (width > 100px) { a > b, a || b { color: red; } } @media (width >= 100px) { .m { color: red; } } @container (width >= 1px) { .c { color: blue; } }"
        }
    }
}

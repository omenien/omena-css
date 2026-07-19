use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;

use super::{ParsedStyleFacts, product_facts_from_cst};
use crate::{StyleDialect, css_keyword, lex, parse};

#[derive(Debug)]
struct CorpusInput {
    label: String,
    dialect: StyleDialect,
    source: String,
}

#[test]
fn product_fact_projection_matches_the_previous_corpus_output() {
    let actual = product_fact_corpus_snapshot();
    assert_eq!(
        actual.as_bytes(),
        include_bytes!("product_facts_legacy_corpus.snap"),
        "product fact projection changed the checked-in corpus output"
    );
}

#[test]
fn product_fact_projection_uses_one_full_fact_authority() {
    let source = include_str!("mod.rs");
    let function = source.split_once("pub(crate) fn product_facts_from_cst");
    assert!(function.is_some(), "product fact projection entry point");
    let Some((_, function)) = function else {
        return;
    };
    let body = function.split_once('{');
    assert!(body.is_some(), "product fact projection body");
    let Some((_, body)) = body else {
        return;
    };
    let body = body.split_once("\n}").map_or(body, |(body, _)| body);

    assert_eq!(body.matches("facts_from_cst(").count(), 1);
    assert_eq!(body.matches("ProductFacts::from").count(), 1);
    assert_eq!(body.matches("collect_").count(), 0);
}

fn product_fact_corpus_snapshot() -> String {
    let corpus = checked_in_corpus();
    let mut snapshot = String::new();
    snapshot.push_str(&format!("caseCount={}\n", corpus.len()));
    for input in corpus {
        let parsed = parse(&input.source, input.dialect);
        let facts = product_facts_from_cst(&input.source, &parsed);
        snapshot.push_str(&format!(
            "case dialect={:?} sourceFingerprint={:016x}",
            input.dialect,
            stable_source_fingerprint(input.source.as_bytes())
        ));
        snapshot.push('\n');
        snapshot.push_str(&format!("{facts:#?}\n"));
    }
    snapshot
}

fn stable_source_fingerprint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

#[test]
fn product_fact_collectors_preserve_the_checked_in_corpus() {
    let corpus = checked_in_corpus();
    assert!(
        corpus.len() >= 50,
        "product-fact corpus unexpectedly shrank: {}",
        corpus.len()
    );

    for input in corpus {
        let parsed = parse(&input.source, input.dialect);
        let actual = product_facts_from_cst(&input.source, &parsed);
        let guarded = source_spelling_guarded_facts(&input.source, input.dialect, actual.clone());
        assert_eq!(actual, guarded, "product-fact drift for {}", input.label);
    }
}

#[test]
fn product_fact_collectors_follow_case_insensitive_syntax() {
    let cases = [
        (
            "animation",
            StyleDialect::Css,
            "@KEYFRAMES Spin { to { opacity: 1; } } .card { ANIMATION-NAME: Spin; }",
            (2, 0, 0),
        ),
        (
            "css-module-value",
            StyleDialect::Css,
            "@VALUE tone: red; .card { color: tone; }",
            (0, 2, 0),
        ),
        (
            "css-module-composes",
            StyleDialect::Css,
            ".card { COMPOSES: base; }",
            (0, 0, 1),
        ),
    ];

    for (label, dialect, source, expected_counts) in cases {
        let parsed = parse(source, dialect);
        let actual = product_facts_from_cst(source, &parsed);
        let guarded = source_spelling_guarded_facts(source, dialect, actual.clone());
        assert_eq!(
            actual, guarded,
            "{label} must follow token-level keyword identity"
        );
        assert_eq!(
            (
                actual.animation_count,
                actual.css_module_value_count,
                actual.css_module_composes_count,
            ),
            expected_counts,
            "{label} fact counts"
        );
    }
}

#[test]
fn product_fact_projection_keeps_excluded_categories_empty() {
    let scss_source = r#"
@use "theme";
@mixin tone() { color: red; }
@include tone();
%base { color: red; }
.card { @extend %base; }
:import("./tokens.css") { tone: remote; }
:export { local: tone; }
@media (width > 1px) { .wide { color: red; } }
"#;
    let scss_parsed = parse(scss_source, StyleDialect::Scss);
    let full_scss = super::facts_from_cst(scss_source, &scss_parsed);
    assert!(full_scss.sass_include_count > 0);
    assert!(full_scss.extend_target_count > 0);
    assert!(full_scss.icss_count > 0);
    assert!(full_scss.icss_import_edge_count > 0);
    assert!(full_scss.icss_export_edge_count > 0);
    assert!(full_scss.at_rule_count > 0);

    let product_scss = product_facts_from_cst(scss_source, &scss_parsed);
    assert_product_exclusions(&product_scss);

    let css_source = "@mixin tone() {} @use \"theme\"; %base {}";
    let css_parsed = parse(css_source, StyleDialect::Css);
    let full_css = super::facts_from_cst(css_source, &css_parsed);
    assert!(
        full_css.sass_symbol_count > 0
            || full_css.sass_module_edge_count > 0
            || full_css.sass_placeholder_definition_count > 0,
        "fixture must exercise the non-Sass projection gate"
    );
    let product_css = product_facts_from_cst(css_source, &css_parsed);
    assert_eq!(product_css.sass_symbol_count, 0);
    assert!(product_css.sass_symbols.is_empty());
    assert_eq!(product_css.sass_module_edge_count, 0);
    assert!(product_css.sass_module_edges.is_empty());
    assert_eq!(product_css.sass_placeholder_definition_count, 0);
    assert!(product_css.sass_placeholder_definitions.is_empty());
}

fn assert_product_exclusions(facts: &ParsedStyleFacts) {
    assert_eq!(facts.sass_include_count, 0);
    assert!(facts.sass_includes.is_empty());
    assert_eq!(facts.extend_target_count, 0);
    assert!(facts.extend_targets.is_empty());
    assert_eq!(facts.icss_count, 0);
    assert!(facts.icss.is_empty());
    assert_eq!(facts.icss_import_edge_count, 0);
    assert!(facts.icss_import_edges.is_empty());
    assert_eq!(facts.icss_export_edge_count, 0);
    assert!(facts.icss_export_edges.is_empty());
    assert_eq!(facts.at_rule_count, 0);
    assert!(facts.at_rules.is_empty());
}

fn source_spelling_guarded_facts(
    source: &str,
    dialect: StyleDialect,
    mut facts: ParsedStyleFacts,
) -> ParsedStyleFacts {
    let lexed = lex(source, dialect);
    let has_animation_syntax = lexed.tokens().iter().any(|token| {
        css_keyword(&token.text).equals("@keyframes")
            || css_keyword(&token.text)
                .strip_prefix("animation")
                .is_some_and(|rest| rest.is_empty() || rest.starts_with('-'))
    });
    let has_value_syntax = lexed
        .tokens()
        .iter()
        .any(|token| css_keyword(&token.text).equals("@value"));
    let has_composes_syntax = lexed
        .tokens()
        .iter()
        .any(|token| css_keyword(&token.text).equals("composes"));

    if !has_animation_syntax {
        facts.animation_count = 0;
        facts.animations.clear();
    }
    if !has_value_syntax {
        facts.css_module_value_count = 0;
        facts.css_module_values.clear();
        facts.css_module_value_import_edge_count = 0;
        facts.css_module_value_import_edges.clear();
        facts.css_module_value_definition_edge_count = 0;
        facts.css_module_value_definition_edges.clear();
    }
    if !has_composes_syntax {
        facts.css_module_composes_count = 0;
        facts.css_module_composes.clear();
        facts.css_module_composes_edge_count = 0;
        facts.css_module_composes_edges.clear();
    }
    facts
}

fn checked_in_corpus() -> Vec<CorpusInput> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .ancestors()
        .nth(3)
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let diff_test_root = manifest_dir
        .parent()
        .map(|path| path.join("omena-diff-test"))
        .unwrap_or_default();
    let mut corpus = BTreeMap::<(String, String), CorpusInput>::new();

    for root in [
        repo_root.join("src"),
        repo_root.join("test"),
        repo_root.join("examples"),
        diff_test_root.clone(),
    ] {
        collect_corpus_files(&root, &diff_test_root, &mut corpus);
    }

    corpus.into_values().collect()
}

fn collect_corpus_files(
    root: &Path,
    diff_test_root: &Path,
    corpus: &mut BTreeMap<(String, String), CorpusInput>,
) {
    if is_generated_or_dependency_directory(root) {
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_corpus_files(&path, diff_test_root, corpus);
            continue;
        }

        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if let Some(dialect) = dialect_from_extension(extension) {
            if let Ok(source) = fs::read_to_string(&path)
                && !source.trim().is_empty()
            {
                insert_corpus_input(path.display().to_string(), dialect, source, corpus);
            }
        } else if extension == "json"
            && path.starts_with(diff_test_root)
            && let Ok(source) = fs::read_to_string(&path)
            && let Ok(value) = serde_json::from_str::<Value>(&source)
        {
            collect_json_sources(&value, &path.display().to_string(), corpus);
        }
    }
}

fn is_generated_or_dependency_directory(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        matches!(
            name.to_str(),
            Some("node_modules" | "target" | "dist" | ".git")
        )
    })
}

fn collect_json_sources(
    value: &Value,
    origin: &str,
    corpus: &mut BTreeMap<(String, String), CorpusInput>,
) {
    match value {
        Value::Array(values) => {
            for value in values {
                collect_json_sources(value, origin, corpus);
            }
        }
        Value::Object(object) => {
            if let Some(source) = object.get("source").and_then(Value::as_str) {
                let dialect = object
                    .get("dialect")
                    .and_then(Value::as_str)
                    .and_then(dialect_from_name)
                    .unwrap_or(StyleDialect::Css);
                let label = object
                    .get("id")
                    .or_else(|| object.get("label"))
                    .and_then(Value::as_str)
                    .map_or_else(|| origin.to_string(), |name| format!("{origin}:{name}"));
                insert_corpus_input(label, dialect, source.to_string(), corpus);
            }
            for value in object.values() {
                collect_json_sources(value, origin, corpus);
            }
        }
        _ => {}
    }
}

fn insert_corpus_input(
    label: String,
    dialect: StyleDialect,
    source: String,
    corpus: &mut BTreeMap<(String, String), CorpusInput>,
) {
    let key = (format!("{dialect:?}"), source.clone());
    corpus.entry(key).or_insert(CorpusInput {
        label,
        dialect,
        source,
    });
}

fn dialect_from_extension(extension: &str) -> Option<StyleDialect> {
    dialect_from_name(extension)
}

fn dialect_from_name(name: &str) -> Option<StyleDialect> {
    match name {
        "css" => Some(StyleDialect::Css),
        "scss" => Some(StyleDialect::Scss),
        "sass" => Some(StyleDialect::Sass),
        "less" => Some(StyleDialect::Less),
        _ => None,
    }
}

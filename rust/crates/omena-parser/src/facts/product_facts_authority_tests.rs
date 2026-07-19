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

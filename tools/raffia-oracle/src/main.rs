use std::collections::BTreeSet;
use std::io::{self, Read};

use omena_parser::{StyleDialect, parse};
use raffia::{Parser, Syntax, ast::Stylesheet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OracleFixture {
    id: String,
    corpus: String,
    dialect: String,
    source: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OracleReport {
    id: String,
    corpus: String,
    dialect: String,
    omena_complete_tree: bool,
    omena_node_count: usize,
    omena_token_count: usize,
    omena_bogus_kinds: Vec<String>,
    omena_error_codes: Vec<String>,
    raffia_parse_ok: bool,
    raffia_recoverable_error_count: usize,
    raffia_debug_len: usize,
    raffia_error: Option<String>,
    relation: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OracleSummary {
    product: &'static str,
    gate_mode: &'static str,
    fixture_count: usize,
    reports: Vec<OracleReport>,
}

fn main() {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("read oracle fixtures from stdin");
    let fixtures = serde_json::from_str::<Vec<OracleFixture>>(&input)
        .expect("oracle fixtures must be valid JSON");
    let reports = fixtures
        .into_iter()
        .map(evaluate_fixture)
        .collect::<Vec<_>>();
    let summary = OracleSummary {
        product: "omena-diff-test.raffia-advisory",
        gate_mode: "advisory",
        fixture_count: reports.len(),
        reports,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&summary).expect("serialize oracle summary")
    );
}

fn evaluate_fixture(fixture: OracleFixture) -> OracleReport {
    let omena_dialect = omena_dialect(fixture.dialect.as_str());
    let parsed = parse(fixture.source.as_str(), omena_dialect);
    let omena_complete_tree =
        u32::from(parsed.syntax().text_range().len()) as usize == fixture.source.len();
    let omena_node_count = parsed.syntax().descendants().count() + 1;
    let omena_token_count = parsed.token_count();
    let omena_bogus_kinds = parsed
        .syntax()
        .descendants()
        .filter(|node| node.kind().is_bogus())
        .map(|node| format!("{:?}", node.kind()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let omena_error_codes = parsed
        .errors()
        .iter()
        .map(|error| format!("{:?}", error.code))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let mut raffia_parser = Parser::new(
        fixture.source.as_str(),
        raffia_syntax(fixture.dialect.as_str()),
    );
    let raffia_result = raffia_parser.parse::<Stylesheet>();
    let raffia_recoverable_error_count = raffia_parser.recoverable_errors().len();
    let (raffia_parse_ok, raffia_debug_len, raffia_error) = match raffia_result {
        Ok(ast) => (true, format!("{:#?}", ast).len(), None),
        Err(error) => (false, 0, Some(format!("{error:?}"))),
    };
    let relation = match (omena_complete_tree, raffia_parse_ok) {
        (true, true) if omena_bogus_kinds.is_empty() && omena_error_codes.is_empty() => {
            "both-clean"
        }
        (true, true) => "omena-recovers-raffia-parses",
        (true, false) => "omena-complete-raffia-drop",
        (false, true) => "omena-incomplete-raffia-parses",
        (false, false) => "both-drop",
    };

    OracleReport {
        id: fixture.id,
        corpus: fixture.corpus,
        dialect: fixture.dialect,
        omena_complete_tree,
        omena_node_count,
        omena_token_count,
        omena_bogus_kinds,
        omena_error_codes,
        raffia_parse_ok,
        raffia_recoverable_error_count,
        raffia_debug_len,
        raffia_error,
        relation,
    }
}

fn omena_dialect(value: &str) -> StyleDialect {
    match value {
        "css" => StyleDialect::Css,
        "scss" => StyleDialect::Scss,
        "sass" => StyleDialect::Sass,
        "less" => StyleDialect::Less,
        other => panic!("unsupported Omena dialect: {other}"),
    }
}

fn raffia_syntax(value: &str) -> Syntax {
    match value {
        "css" => Syntax::Css,
        "scss" => Syntax::Scss,
        "sass" => Syntax::Sass,
        "less" => Syntax::Less,
        other => panic!("unsupported raffia syntax: {other}"),
    }
}

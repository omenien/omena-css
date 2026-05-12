use omena_parser::{StyleDialect, summarize_omena_parser_lex};
use serde::Deserialize;
use std::io::{self, Read};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LexInputV0 {
    style_source: String,
    dialect: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdin = String::new();
    io::stdin().read_to_string(&mut stdin)?;
    let input: LexInputV0 = serde_json::from_str(&stdin)?;
    let dialect = parse_style_dialect(input.dialect.as_str())?;
    let summary = summarize_omena_parser_lex(&input.style_source, dialect);
    serde_json::to_writer_pretty(io::stdout(), &summary)?;
    Ok(())
}

fn parse_style_dialect(dialect: &str) -> Result<StyleDialect, Box<dyn std::error::Error>> {
    match dialect {
        "css" => Ok(StyleDialect::Css),
        "scss" => Ok(StyleDialect::Scss),
        "sass" => Ok(StyleDialect::Sass),
        "less" => Ok(StyleDialect::Less),
        other => Err(format!("unsupported omena parser lex dialect: {other}").into()),
    }
}

use omena_parser::{StyleDialect, summarize_omena_parser_parity_lite};
use std::{
    env,
    io::{self, Read},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = env::args()
        .nth(1)
        .ok_or("usage: omena-parser-summary <style-file-path>")?;
    let mut stdin = String::new();
    io::stdin().read_to_string(&mut stdin)?;
    let summary = summarize_omena_parser_parity_lite(&stdin, dialect_for_path(&file_path));
    serde_json::to_writer_pretty(io::stdout(), &summary)?;
    Ok(())
}

fn dialect_for_path(file_path: &str) -> StyleDialect {
    if file_path.ends_with(".sass") || file_path.ends_with(".module.sass") {
        StyleDialect::Sass
    } else if file_path.ends_with(".scss") || file_path.ends_with(".module.scss") {
        StyleDialect::Scss
    } else if file_path.ends_with(".less") || file_path.ends_with(".module.less") {
        StyleDialect::Less
    } else {
        StyleDialect::Css
    }
}

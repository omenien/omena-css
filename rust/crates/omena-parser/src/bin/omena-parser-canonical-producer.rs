use std::io::{self, Read};

#[path = "omena-parser-css-modules-intermediate.rs"]
mod css_modules_intermediate;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = std::env::args()
        .nth(1)
        .ok_or("usage: omena-parser-canonical-producer <style-file-path>")?;
    let mut source = String::new();
    io::stdin().read_to_string(&mut source)?;
    let summary = css_modules_intermediate::summarize_parser_canonical_producer_signal(
        &source,
        css_modules_intermediate::dialect_for_path(&file_path),
    );
    serde_json::to_writer_pretty(io::stdout(), &summary)?;
    Ok(())
}

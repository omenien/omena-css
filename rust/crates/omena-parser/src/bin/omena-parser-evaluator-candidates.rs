use std::io::{self, Read};

mod parser_public_product_support;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = std::env::args()
        .nth(1)
        .ok_or("usage: omena-parser-evaluator-candidates <style-file-path>")?;
    let mut source = String::new();
    io::stdin().read_to_string(&mut source)?;
    let summary = parser_public_product_support::summarize_parser_evaluator_candidates(
        &source,
        parser_public_product_support::dialect_for_path(&file_path),
    );
    serde_json::to_writer_pretty(io::stdout(), &summary)?;
    Ok(())
}

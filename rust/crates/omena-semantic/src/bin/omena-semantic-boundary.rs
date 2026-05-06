use std::io::{self, Read};

use omena_semantic::{
    summarize_css_modules_semantics_from_source,
    summarize_omena_parser_style_semantic_boundary_from_source,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(path) = std::env::args().nth(1) else {
        return Err("expected module file path argument".into());
    };
    let mut source = String::new();
    io::stdin().read_to_string(&mut source)?;
    if summarize_css_modules_semantics_from_source(&path, &source).is_none() {
        return Err("unsupported style module path".into());
    }
    let summary = summarize_omena_parser_style_semantic_boundary_from_source(&path, &source);
    serde_json::to_writer_pretty(io::stdout(), &summary)?;
    Ok(())
}

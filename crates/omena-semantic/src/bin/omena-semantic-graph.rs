use std::io::{self, Read};

use engine_input_producers::EngineInputV2;
use omena_semantic::summarize_style_semantic_graph_from_source;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StyleSemanticGraphInput {
    style_path: String,
    style_source: String,
    engine_input: EngineInputV2,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input_json = String::new();
    io::stdin().read_to_string(&mut input_json)?;
    let input: StyleSemanticGraphInput = serde_json::from_str(&input_json)?;
    let Some(summary) = summarize_style_semantic_graph_from_source(
        &input.style_path,
        &input.style_source,
        &input.engine_input,
    ) else {
        return Err("unsupported style module path".into());
    };
    serde_json::to_writer_pretty(io::stdout(), &summary)?;
    Ok(())
}

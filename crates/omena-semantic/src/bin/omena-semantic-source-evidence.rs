use std::io::{self, Read};

use engine_input_producers::EngineInputV2;
use omena_semantic::summarize_source_input_evidence;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input_json = String::new();
    io::stdin().read_to_string(&mut input_json)?;
    let input: EngineInputV2 = serde_json::from_str(&input_json)?;
    let summary = summarize_source_input_evidence(&input);
    serde_json::to_writer_pretty(io::stdout(), &summary)?;
    Ok(())
}

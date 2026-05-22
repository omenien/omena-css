use omena_benchmarks::summarize_style_corpus_snapshot;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = summarize_style_corpus_snapshot();
    println!("{}", serde_json::to_string_pretty(&snapshot)?);
    Ok(())
}

use omena_diff_test::summarize_omena_source_precision_baseline;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let baseline = summarize_omena_source_precision_baseline();
    println!("{}", serde_json::to_string_pretty(&baseline)?);
    Ok(())
}

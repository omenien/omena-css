use omena_benchmarks::summarize_headline_axis_fidelity;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = summarize_headline_axis_fidelity()?;
    if !(snapshot.source_map_vlq_valid
        && snapshot.source_map_positions_valid
        && snapshot.css_modules_moat_preserved_through_minify
        && !snapshot.speed_claim_ready
        && !snapshot.runtime_loop_headline_ready)
    {
        return Err("headline-axis fidelity snapshot is missing required evidence".into());
    }
    println!("{}", serde_json::to_string_pretty(&snapshot)?);
    Ok(())
}

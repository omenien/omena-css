use omena_benchmarks::summarize_criterion_surface_snapshot;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = summarize_criterion_surface_snapshot();
    if !(snapshot.includes_legacy_parser_oracle_lane
        && snapshot.includes_omena_parser_lane
        && snapshot.includes_parser_product_lanes
        && snapshot.includes_semantic_lane
        && snapshot.includes_abstract_value_lane
        && snapshot.m4_corpus_expansion_reflected
        && snapshot.symmetric_parser_product_boundary)
    {
        return Err("criterion surface snapshot is missing required M4 benchmark evidence".into());
    }

    println!("{}", serde_json::to_string_pretty(&snapshot)?);
    Ok(())
}

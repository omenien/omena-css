use omena_benchmarks::summarize_bundler_productization_benchmark_surface;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = summarize_bundler_productization_benchmark_surface();
    if !(snapshot.corpus_sample_count == 3
        && snapshot.includes_napi_in_process_lane
        && snapshot.includes_cli_spawn_lane
        && snapshot.includes_lightningcss_comparator_lane
        && snapshot.includes_memory_rss_metric
        && snapshot.includes_provenance_mode_split
        && !snapshot.speed_claim_ready)
    {
        return Err(
            "bundler productization benchmark surface is missing required measurement evidence"
                .into(),
        );
    }

    println!("{}", serde_json::to_string_pretty(&snapshot)?);
    Ok(())
}

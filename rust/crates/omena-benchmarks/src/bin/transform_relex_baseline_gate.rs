use std::{env, fs, path::Path};

use omena_benchmarks::{
    render_transform_relex_baseline_snapshot_json, validate_transform_relex_baseline_snapshot,
};

const EXPECTED_BASELINE: &str = include_str!("../../fixtures/transform-relex-baseline-v0.json");

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    if env::args().skip(1).any(|arg| arg == "--regen") {
        let snapshot = render_transform_relex_baseline_snapshot_json()?;
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let path = Path::new(manifest_dir).join("fixtures/transform-relex-baseline-v0.json");
        fs::write(&path, snapshot).map_err(|error| error.to_string())?;
        println!("transform-relex-baseline: regenerated {}", path.display());
        return Ok(());
    }

    validate_transform_relex_baseline_snapshot(EXPECTED_BASELINE)?;
    println!("transform-relex-baseline: ok");
    Ok(())
}

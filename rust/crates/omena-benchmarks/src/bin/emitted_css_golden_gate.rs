use std::{env, fs, path::Path};

use omena_benchmarks::{
    render_emitted_css_golden_gate_snapshot_json, validate_emitted_css_golden_gate_snapshot,
};

const EXPECTED_GOLDEN: &str = include_str!("../../fixtures/emitted-css-golden-v0.json");

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    if env::args().skip(1).any(|arg| arg == "--regen") {
        let snapshot = render_emitted_css_golden_gate_snapshot_json()?;
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let path = Path::new(manifest_dir).join("fixtures/emitted-css-golden-v0.json");
        fs::write(&path, snapshot).map_err(|error| error.to_string())?;
        println!("emitted-css-golden-gate: regenerated {}", path.display());
        return Ok(());
    }

    validate_emitted_css_golden_gate_snapshot(EXPECTED_GOLDEN)?;
    println!("emitted-css-golden-gate: ok");
    Ok(())
}

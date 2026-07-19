use omena_diff_test::{
    LinkedEmissionByteDifferentialPerturbationV0, summarize_linked_emission_byte_differential_v0,
};

fn main() {
    let perturbation = if std::env::args().any(|arg| arg == "--inject-unexpected-divergence") {
        LinkedEmissionByteDifferentialPerturbationV0::AddUnexpectedRule
    } else if std::env::args().any(|arg| arg == "--force-equivalent") {
        LinkedEmissionByteDifferentialPerturbationV0::CollapseToLegacyBytes
    } else {
        LinkedEmissionByteDifferentialPerturbationV0::None
    };
    let report =
        summarize_linked_emission_byte_differential_v0(perturbation).unwrap_or_else(|error| {
            eprintln!("linked emission byte differential failed: {error}");
            std::process::exit(1);
        });
    println!(
        "{}",
        serde_json::to_string_pretty(&report).unwrap_or_else(|error| {
            eprintln!("linked emission byte differential could not be serialized: {error}");
            std::process::exit(1);
        })
    );
}

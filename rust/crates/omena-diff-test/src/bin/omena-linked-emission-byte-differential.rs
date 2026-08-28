use omena_diff_test::{
    LinkedEmissionByteDifferentialPerturbationV0,
    summarize_linked_emission_byte_differential_envelope_v0,
};

fn main() {
    let perturbation = if std::env::args().any(|arg| arg == "--inject-unexpected-divergence") {
        LinkedEmissionByteDifferentialPerturbationV0::AddUnexpectedRule
    } else if std::env::args().any(|arg| arg == "--inject-linked-asset-url-drift") {
        LinkedEmissionByteDifferentialPerturbationV0::DriftLinkedAssetUrl
    } else if std::env::args().any(|arg| arg == "--force-equivalent") {
        LinkedEmissionByteDifferentialPerturbationV0::CollapseToLegacyBytes
    } else if std::env::args().any(|arg| arg == "--inject-cross-module-declaration-loss") {
        LinkedEmissionByteDifferentialPerturbationV0::DropReachableCrossModuleDeclaration
    } else if std::env::args().any(|arg| arg == "--inject-composed-declaration-loss") {
        LinkedEmissionByteDifferentialPerturbationV0::DropComposedDeclaration
    } else if std::env::args().any(|arg| arg == "--inject-live-declaration-loss") {
        LinkedEmissionByteDifferentialPerturbationV0::DropLiveDeclaration
    } else if std::env::args().any(|arg| arg == "--inject-unclaimed-linked-token") {
        LinkedEmissionByteDifferentialPerturbationV0::AddUnclaimedLinkedToken
    } else if std::env::args().any(|arg| arg == "--inject-composes-liveness-loss") {
        LinkedEmissionByteDifferentialPerturbationV0::DropComposesReachability
    } else if std::env::args().any(|arg| arg == "--inject-incompatible-style-paths") {
        LinkedEmissionByteDifferentialPerturbationV0::BreakEnginePathEquivalence
    } else if std::env::args().any(|arg| arg == "--inject-unattributed-reference") {
        LinkedEmissionByteDifferentialPerturbationV0::AddUnattributedReachabilityReference
    } else if std::env::args().any(|arg| arg == "--inject-authored-liveness-flip") {
        LinkedEmissionByteDifferentialPerturbationV0::FlipAuthoredLivenessExpectation
    } else if std::env::args().any(|arg| arg == "--inject-missing-fixture") {
        LinkedEmissionByteDifferentialPerturbationV0::DropFixture
    } else if std::env::args().any(|arg| arg == "--inject-linked-rule-misattribution") {
        LinkedEmissionByteDifferentialPerturbationV0::MisattributeLinkedRule
    } else {
        LinkedEmissionByteDifferentialPerturbationV0::None
    };
    let envelope = summarize_linked_emission_byte_differential_envelope_v0(perturbation)
        .unwrap_or_else(|error| {
            eprintln!("linked emission byte differential failed: {error}");
            std::process::exit(1);
        });
    println!(
        "{}",
        serde_json::to_string_pretty(&envelope).unwrap_or_else(|error| {
            eprintln!("linked emission byte differential could not be serialized: {error}");
            std::process::exit(1);
        })
    );
}

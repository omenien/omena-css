use omena_cascade::{
    CascadeDeclaration, CascadeKey, CascadeLevel, CascadeValue, LayerOrdinal, ModuleRank,
    Specificity, SpecificityExactnessV0, classify_cascade_ranked_set_loss, normalized_layer_rank,
};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FixtureResult {
    name: &'static str,
    classification: omena_cascade::CascadeRankedSetLossClassV0,
}

fn main() {
    let lower = declaration(
        "lower",
        CascadeLevel::UserNormal,
        Specificity::new(9, 9, 9),
        SpecificityExactnessV0::Inexact,
    );
    let inexact_axis_winner = declaration(
        "inexact-axis-winner",
        CascadeLevel::AuthorNormal,
        Specificity::ZERO,
        SpecificityExactnessV0::Inexact,
    );
    let exact_axis_winner = declaration(
        "exact-axis-winner",
        CascadeLevel::AuthorNormal,
        Specificity::ZERO,
        SpecificityExactnessV0::Exact,
    );
    let specificity_only_winner = declaration(
        "specificity-only-winner",
        CascadeLevel::AuthorNormal,
        Specificity::new(0, 1, 0),
        SpecificityExactnessV0::Exact,
    );
    let specificity_only_runner_up = declaration(
        "specificity-only-runner-up",
        CascadeLevel::AuthorNormal,
        Specificity::ZERO,
        SpecificityExactnessV0::Inexact,
    );
    let single_inexact = declaration(
        "single-inexact",
        CascadeLevel::AuthorNormal,
        Specificity::ZERO,
        SpecificityExactnessV0::Inexact,
    );

    let results = [
        FixtureResult {
            name: "inexactAxisWinner",
            classification: classify_cascade_ranked_set_loss(&[lower.clone(), inexact_axis_winner]),
        },
        FixtureResult {
            name: "exactAxisWinner",
            classification: classify_cascade_ranked_set_loss(&[lower, exact_axis_winner]),
        },
        FixtureResult {
            name: "specificityOnlyWinner",
            classification: classify_cascade_ranked_set_loss(&[
                specificity_only_winner,
                specificity_only_runner_up,
            ]),
        },
        FixtureResult {
            name: "singleInexactCandidate",
            classification: classify_cascade_ranked_set_loss(&[single_inexact]),
        },
    ];
    println!(
        "{}",
        serde_json::to_string(&results).expect("classifier fixture must serialize")
    );
}

fn declaration(
    id: &str,
    level: CascadeLevel,
    specificity: Specificity,
    exactness: SpecificityExactnessV0,
) -> CascadeDeclaration {
    CascadeDeclaration {
        id: id.to_string(),
        property: "color".to_string(),
        value: CascadeValue::Literal(id.to_string()),
        key: CascadeKey::new(
            level,
            normalized_layer_rank(false, LayerOrdinal::new(0)),
            0,
            specificity,
            ModuleRank::ZERO,
            0,
        ),
        specificity_exactness: exactness,
    }
}

use omena_cascade::{
    CascadeDeclaration, CascadeKey, CascadeLevel, CascadeValue, LayerOrdinal, OpenWorldTieEvidence,
    Specificity, SpecificityExactnessV0, classify_cascade_ranked_set_loss, normalized_layer_rank,
};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FixtureResult {
    name: &'static str,
    classification: omena_cascade::CascadeRankedSetLossClassV0,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lower = declaration(
        "lower",
        CascadeLevel::UserNormal,
        0,
        Specificity::new(9, 9, 9),
        SpecificityExactnessV0::Inexact,
    );
    let inexact_axis_winner = declaration(
        "inexact-axis-winner",
        CascadeLevel::AuthorNormal,
        0,
        Specificity::ZERO,
        SpecificityExactnessV0::Inexact,
    );
    let exact_axis_winner = declaration(
        "exact-axis-winner",
        CascadeLevel::AuthorNormal,
        0,
        Specificity::ZERO,
        SpecificityExactnessV0::Exact,
    );
    let inexact_lower_layer = declaration(
        "inexact-lower-layer",
        CascadeLevel::AuthorNormal,
        0,
        Specificity::new(9, 9, 9),
        SpecificityExactnessV0::Inexact,
    );
    let exact_layer_winner = declaration(
        "exact-layer-winner",
        CascadeLevel::AuthorNormal,
        2,
        Specificity::ZERO,
        SpecificityExactnessV0::Exact,
    );
    let specificity_only_winner = declaration(
        "specificity-only-winner",
        CascadeLevel::AuthorNormal,
        0,
        Specificity::new(0, 1, 0),
        SpecificityExactnessV0::Exact,
    );
    let specificity_only_runner_up = declaration(
        "specificity-only-runner-up",
        CascadeLevel::AuthorNormal,
        0,
        Specificity::ZERO,
        SpecificityExactnessV0::Inexact,
    );
    let single_inexact = declaration(
        "single-inexact",
        CascadeLevel::AuthorNormal,
        0,
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
            name: "exactLayerWinner",
            classification: classify_cascade_ranked_set_loss(&[
                inexact_lower_layer,
                exact_layer_winner,
            ]),
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
    println!("{}", serde_json::to_string(&results)?);
    Ok(())
}

fn declaration(
    id: &str,
    level: CascadeLevel,
    layer_ordinal: i32,
    specificity: Specificity,
    exactness: SpecificityExactnessV0,
) -> CascadeDeclaration {
    CascadeDeclaration {
        id: id.to_string(),
        property: omena_syntax::ident::AuthoredPropertyTextV0::new("color"),
        property_key: omena_syntax::ident::PropertyNameV0::standard("color").canonical_key(),
        value: CascadeValue::Literal(id.to_string()),
        key: CascadeKey::new(
            level,
            normalized_layer_rank(false, LayerOrdinal::new(layer_ordinal)),
            0,
            specificity,
            0,
        ),
        open_world_tie_evidence: OpenWorldTieEvidence::NONE,
        specificity_exactness: exactness,
    }
}

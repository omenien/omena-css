//! Evaluate parsed `omena-fixture-v0` expectations against engine output.
//!
//! RFC 0003 (#37 wiring-gap): the fixture grammar *parses* assertion forms
//! (`diagnostic` / `no-diagnostic` / `count` / `cascade-outcome` /
//! `cascade-witness` / `boundary-state`) and classifies them with
//! [`OmenaFixtureExpectationKindV0`], but nothing turned a classified
//! expectation into a pass/fail. This module closes the evaluator gap for the
//! P0-independent families.
//!
//! ## Dependency-light wiring
//!
//! `omena-testkit` must stay free of an `omena-query` dependency to preserve
//! the workspace DAG (the test substrate sits *below* the engine). So the
//! evaluator never names an engine type: the consumer supplies diagnostics as
//! [`OmenaFixtureDiagnosticV0`] (a minimal `{ code }` projection of
//! `OmenaQueryStyleDiagnosticV0`) and boundary states as
//! [`OmenaFixtureBoundaryStateV0`] (a `{ reference, state }` projection of the
//! resolver boundary-state lattice). A real engine-backed consumer such as
//! `omena-diff-test` — which *already* depends on `omena-query` — maps the
//! shipped diagnostic/boundary structs into these projections.
//!
//! ## Live vs deferred families
//!
//! Live now (backed by shipped functions):
//! - `diagnostic` / `no-diagnostic` / `count` — matched against diagnostic codes.
//! - `boundary-state` — matched against the resolver boundary-state lattice.
//! - `cascade-outcome` / `cascade-witness` — matched against per-declaration
//!   cascade winners/witnesses. #33's in-process bridge now generates the SIFs
//!   the resolver-generator consumes, so a consumer can run the cascade and
//!   project each scope's winner id plus its witness (also-considered)
//!   declaration ids into [`OmenaFixtureCascadeV0`]. The evaluator stays free of
//!   an `omena-cascade` / `omena-query` dependency by matching only against that
//!   projection (see "Dependency-light wiring" above).

use serde::Serialize;

use crate::fixture::{OmenaFixtureExpectationKindV0, OmenaFixtureExpectationV0, OmenaFixtureV0};

/// Minimal diagnostic projection consumed by the fixture evaluator.
///
/// The engine produces `OmenaQueryStyleDiagnosticV0`; the consumer projects
/// each one to its `code` so `omena-testkit` need not depend on `omena-query`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaFixtureDiagnosticV0 {
    /// Stable diagnostic code, e.g. `missingSassSymbol` or `missingKeyframes`.
    pub code: String,
}

impl OmenaFixtureDiagnosticV0 {
    /// Build a diagnostic projection from a diagnostic code.
    pub fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }
}

/// Minimal boundary-state projection consumed by the fixture evaluator.
///
/// The resolver produces `OmenaResolverBoundaryStateV0`; the consumer projects
/// the external reference id and the lattice state name (`resolved` / `partial`
/// / `stale` / `missing` / `unresolved`) so the test substrate stays free of an
/// `omena-resolver` dependency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaFixtureBoundaryStateV0 {
    /// External reference id the fixture names, e.g. `ext-1`.
    pub reference: String,
    /// Lattice state name as produced by the resolver boundary lattice.
    pub state: String,
}

impl OmenaFixtureBoundaryStateV0 {
    /// Build a boundary-state projection from a reference id and state name.
    pub fn new(reference: impl Into<String>, state: impl Into<String>) -> Self {
        Self {
            reference: reference.into(),
            state: state.into(),
        }
    }
}

/// Minimal cascade-outcome projection consumed by the fixture evaluator.
///
/// The engine produces a `CascadeOutcome` (`omena-cascade`) per resolved scope;
/// the consumer projects the winning declaration id and the witness set (the
/// winner plus every also-considered/challenger declaration that participated
/// in the cascade comparison) so `omena-testkit` need not depend on
/// `omena-cascade`. A `cascade-outcome <id>` expectation passes when `<id>`
/// equals [`winner_id`](Self::winner_id); a `cascade-witness <id>` expectation
/// passes when `<id>` appears in [`witness_ids`](Self::witness_ids).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaFixtureCascadeV0 {
    /// Winning declaration id for the resolved scope, e.g. `decl-1`.
    pub winner_id: String,
    /// Declaration ids that participated in the cascade as witnesses.
    ///
    /// This is the winner plus every also-considered/challenger declaration the
    /// engine ranked, so a `cascade-witness` assertion can name any declaration
    /// that took part in the comparison, not just the winner.
    pub witness_ids: Vec<String>,
}

impl OmenaFixtureCascadeV0 {
    /// Build a cascade projection from a winner id and its witness declaration ids.
    ///
    /// The winner id is always treated as a witness, so callers need not repeat
    /// it in `witness_ids`.
    pub fn new(
        winner_id: impl Into<String>,
        witness_ids: impl IntoIterator<Item = String>,
    ) -> Self {
        let winner_id = winner_id.into();
        let mut witnesses = vec![winner_id.clone()];
        for id in witness_ids {
            if !witnesses.contains(&id) {
                witnesses.push(id);
            }
        }
        Self {
            winner_id,
            witness_ids: witnesses,
        }
    }
}

/// Outcome of evaluating one fixture expectation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaFixtureExpectationOutcomeV0 {
    /// Original expectation key, e.g. `diagnostic` or `boundary-state ext-1 resolved`.
    pub key: String,
    /// Classified expectation family.
    pub kind: OmenaFixtureExpectationKindV0,
    /// Whether the expectation was actually evaluated (false for deferred families).
    pub evaluated: bool,
    /// Whether the expectation is satisfied. Always `false` when `evaluated` is `false`.
    pub satisfied: bool,
    /// Human-readable detail explaining the pass/fail/deferral.
    pub detail: String,
}

/// Evaluate every parsed expectation in `fixture` against supplied engine
/// output, returning one [`OmenaFixtureExpectationOutcomeV0`] per expectation.
///
/// `diagnostics` is the flattened diagnostic set the engine produced for the
/// fixture's files; `boundary_states` is the resolver boundary-state set keyed
/// by external reference id; `cascades` is the per-scope cascade projection the
/// resolver-generator produced from #33's SIFs. All shipped families
/// (`diagnostic` / `no-diagnostic` / `count` / `boundary-state` /
/// `cascade-outcome` / `cascade-witness`) are evaluated here.
pub fn evaluate_omena_fixture_v0(
    fixture: &OmenaFixtureV0,
    diagnostics: &[OmenaFixtureDiagnosticV0],
    boundary_states: &[OmenaFixtureBoundaryStateV0],
    cascades: &[OmenaFixtureCascadeV0],
) -> Vec<OmenaFixtureExpectationOutcomeV0> {
    fixture
        .expectations
        .iter()
        .map(|expectation| evaluate_one(expectation, diagnostics, boundary_states, cascades))
        .collect()
}

/// Evaluate a fixture against diagnostics produced lazily by an injected
/// closure, keeping the engine dependency on the *consumer* side.
///
/// `produce_diagnostics` is invoked once per fixture file and the results are
/// flattened before delegating to [`evaluate_omena_fixture_v0`]. `cascades` is the
/// per-scope cascade projection the resolver-generator produced from #33's SIFs.
/// A consumer that already depends on `omena-query` (such as `omena-diff-test`)
/// wires the real `summarize_omena_query_style_diagnostics_for_file` here
/// without forcing an engine dependency into this crate.
pub fn evaluate_omena_fixture_v0_with<F>(
    fixture: &OmenaFixtureV0,
    boundary_states: &[OmenaFixtureBoundaryStateV0],
    cascades: &[OmenaFixtureCascadeV0],
    mut produce_diagnostics: F,
) -> Vec<OmenaFixtureExpectationOutcomeV0>
where
    F: FnMut(&crate::fixture::OmenaFixtureFileV0) -> Vec<OmenaFixtureDiagnosticV0>,
{
    let diagnostics = fixture
        .files
        .iter()
        .flat_map(&mut produce_diagnostics)
        .collect::<Vec<_>>();
    evaluate_omena_fixture_v0(fixture, &diagnostics, boundary_states, cascades)
}

fn evaluate_one(
    expectation: &OmenaFixtureExpectationV0,
    diagnostics: &[OmenaFixtureDiagnosticV0],
    boundary_states: &[OmenaFixtureBoundaryStateV0],
    cascades: &[OmenaFixtureCascadeV0],
) -> OmenaFixtureExpectationOutcomeV0 {
    let kind = expectation.kind();
    match kind {
        OmenaFixtureExpectationKindV0::Diagnostic => {
            evaluate_diagnostic(expectation, diagnostics, kind)
        }
        OmenaFixtureExpectationKindV0::NoDiagnostic => {
            evaluate_no_diagnostic(expectation, diagnostics, kind)
        }
        OmenaFixtureExpectationKindV0::Count => evaluate_count(expectation, diagnostics, kind),
        OmenaFixtureExpectationKindV0::BoundaryState => {
            evaluate_boundary_state(expectation, boundary_states, kind)
        }
        OmenaFixtureExpectationKindV0::CascadeOutcome => {
            evaluate_cascade_outcome(expectation, cascades, kind)
        }
        OmenaFixtureExpectationKindV0::CascadeWitness => {
            evaluate_cascade_witness(expectation, cascades, kind)
        }
        OmenaFixtureExpectationKindV0::Product
        | OmenaFixtureExpectationKindV0::Assertion
        | OmenaFixtureExpectationKindV0::Unknown => deferred(
            expectation,
            kind,
            "product-owned expectation family is not engine-evaluated by the testkit",
        ),
    }
}

/// Extract the diagnostic code an expectation targets.
///
/// `diagnostic` carries the code in a `code: <name>` body line; the other
/// families carry it as the first token after the keyword in the key, e.g.
/// `no-diagnostic missingSassSymbol` or `count missingKeyframes:2`.
fn expectation_diagnostic_code(expectation: &OmenaFixtureExpectationV0) -> Option<String> {
    if let Some(code) = code_from_key_tail(&expectation.key) {
        return Some(code);
    }
    code_from_value_body(&expectation.value)
}

fn code_from_key_tail(key: &str) -> Option<String> {
    let tail = key.split_whitespace().nth(1)?;
    // A `count` body is `<code>:<n>`; keep only the code half.
    let code = tail.split(':').next().unwrap_or(tail);
    if code.is_empty() {
        None
    } else {
        Some(code.to_string())
    }
}

fn code_from_value_body(value: &str) -> Option<String> {
    value.lines().find_map(|line| {
        line.trim()
            .strip_prefix("code:")
            .map(|code| code.trim().to_string())
            .filter(|code| !code.is_empty())
    })
}

fn count_diagnostics_with_code(diagnostics: &[OmenaFixtureDiagnosticV0], code: &str) -> usize {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == code)
        .count()
}

fn evaluate_diagnostic(
    expectation: &OmenaFixtureExpectationV0,
    diagnostics: &[OmenaFixtureDiagnosticV0],
    kind: OmenaFixtureExpectationKindV0,
) -> OmenaFixtureExpectationOutcomeV0 {
    let Some(code) = expectation_diagnostic_code(expectation) else {
        return malformed(
            expectation,
            kind,
            "diagnostic expectation is missing a code",
        );
    };
    let observed = count_diagnostics_with_code(diagnostics, &code);
    let satisfied = observed > 0;
    let detail = if satisfied {
        format!("diagnostic `{code}` present ({observed} occurrence(s))")
    } else {
        format!("diagnostic `{code}` expected but absent")
    };
    outcome(expectation, kind, satisfied, detail)
}

fn evaluate_no_diagnostic(
    expectation: &OmenaFixtureExpectationV0,
    diagnostics: &[OmenaFixtureDiagnosticV0],
    kind: OmenaFixtureExpectationKindV0,
) -> OmenaFixtureExpectationOutcomeV0 {
    let Some(code) = expectation_diagnostic_code(expectation) else {
        return malformed(
            expectation,
            kind,
            "no-diagnostic expectation is missing a code",
        );
    };
    let observed = count_diagnostics_with_code(diagnostics, &code);
    let satisfied = observed == 0;
    let detail = if satisfied {
        format!("diagnostic `{code}` correctly absent")
    } else {
        format!("diagnostic `{code}` present but expected absent ({observed} occurrence(s))")
    };
    outcome(expectation, kind, satisfied, detail)
}

fn evaluate_count(
    expectation: &OmenaFixtureExpectationV0,
    diagnostics: &[OmenaFixtureDiagnosticV0],
    kind: OmenaFixtureExpectationKindV0,
) -> OmenaFixtureExpectationOutcomeV0 {
    let Some((code, expected)) = parse_count_target(&expectation.key) else {
        return malformed(
            expectation,
            kind,
            "count expectation must be `count <code>:<n>`",
        );
    };
    let observed = count_diagnostics_with_code(diagnostics, &code);
    let satisfied = observed == expected;
    let detail = format!("diagnostic `{code}` count expected {expected}, observed {observed}");
    outcome(expectation, kind, satisfied, detail)
}

fn parse_count_target(key: &str) -> Option<(String, usize)> {
    let tail = key.split_whitespace().nth(1)?;
    let (code, count) = tail.split_once(':')?;
    if code.is_empty() {
        return None;
    }
    let expected = count.trim().parse::<usize>().ok()?;
    Some((code.to_string(), expected))
}

fn evaluate_boundary_state(
    expectation: &OmenaFixtureExpectationV0,
    boundary_states: &[OmenaFixtureBoundaryStateV0],
    kind: OmenaFixtureExpectationKindV0,
) -> OmenaFixtureExpectationOutcomeV0 {
    let Some((reference, expected_state)) = parse_boundary_target(&expectation.key) else {
        return malformed(
            expectation,
            kind,
            "boundary-state expectation must be `boundary-state <ref> <state>`",
        );
    };
    let Some(actual) = boundary_states
        .iter()
        .find(|state| state.reference == reference)
    else {
        return outcome(
            expectation,
            kind,
            false,
            format!("boundary reference `{reference}` not present in resolver output"),
        );
    };
    // The fixture writes lattice states as `Resolved` / `Partial` / ...; the
    // resolver lattice projects them lowercase via `as_str`. Compare case-insensitively.
    let satisfied = actual.state.eq_ignore_ascii_case(&expected_state);
    let detail = if satisfied {
        format!("boundary `{reference}` state `{}` matches", actual.state)
    } else {
        format!(
            "boundary `{reference}` expected `{expected_state}`, observed `{}`",
            actual.state
        )
    };
    outcome(expectation, kind, satisfied, detail)
}

fn parse_boundary_target(key: &str) -> Option<(String, String)> {
    let mut parts = key.split_whitespace();
    parts.next()?; // discard `boundary-state` keyword
    let reference = parts.next()?;
    let state = parts.next()?;
    if reference.is_empty() || state.is_empty() {
        return None;
    }
    Some((reference.to_string(), state.to_string()))
}

/// Extract the declaration id a cascade family targets: the first token after
/// the keyword, e.g. `decl-1` in `cascade-outcome decl-1`.
fn cascade_target_id(key: &str) -> Option<String> {
    let id = key.split_whitespace().nth(1)?;
    if id.is_empty() {
        None
    } else {
        Some(id.to_string())
    }
}

fn evaluate_cascade_outcome(
    expectation: &OmenaFixtureExpectationV0,
    cascades: &[OmenaFixtureCascadeV0],
    kind: OmenaFixtureExpectationKindV0,
) -> OmenaFixtureExpectationOutcomeV0 {
    let Some(expected_winner) = cascade_target_id(&expectation.key) else {
        return malformed(
            expectation,
            kind,
            "cascade-outcome expectation must be `cascade-outcome <declaration-id>`",
        );
    };
    let satisfied = cascades
        .iter()
        .any(|cascade| cascade.winner_id == expected_winner);
    let detail = if satisfied {
        format!("cascade winner `{expected_winner}` matches")
    } else {
        let observed = cascades
            .iter()
            .map(|cascade| cascade.winner_id.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        format!("cascade winner `{expected_winner}` expected, observed winners [{observed}]")
    };
    outcome(expectation, kind, satisfied, detail)
}

fn evaluate_cascade_witness(
    expectation: &OmenaFixtureExpectationV0,
    cascades: &[OmenaFixtureCascadeV0],
    kind: OmenaFixtureExpectationKindV0,
) -> OmenaFixtureExpectationOutcomeV0 {
    let Some(expected_witness) = cascade_target_id(&expectation.key) else {
        return malformed(
            expectation,
            kind,
            "cascade-witness expectation must be `cascade-witness <declaration-id>`",
        );
    };
    let satisfied = cascades
        .iter()
        .any(|cascade| cascade.witness_ids.iter().any(|id| id == &expected_witness));
    let detail = if satisfied {
        format!("cascade witness `{expected_witness}` participated in the cascade")
    } else {
        format!("cascade witness `{expected_witness}` expected but never participated")
    };
    outcome(expectation, kind, satisfied, detail)
}

fn outcome(
    expectation: &OmenaFixtureExpectationV0,
    kind: OmenaFixtureExpectationKindV0,
    satisfied: bool,
    detail: impl Into<String>,
) -> OmenaFixtureExpectationOutcomeV0 {
    OmenaFixtureExpectationOutcomeV0 {
        key: expectation.key.clone(),
        kind,
        evaluated: true,
        satisfied,
        detail: detail.into(),
    }
}

fn malformed(
    expectation: &OmenaFixtureExpectationV0,
    kind: OmenaFixtureExpectationKindV0,
    detail: impl Into<String>,
) -> OmenaFixtureExpectationOutcomeV0 {
    // A malformed live assertion is a failure, not a deferral: the corpus
    // declared something the evaluator should check and could not.
    outcome(expectation, kind, false, detail)
}

fn deferred(
    expectation: &OmenaFixtureExpectationV0,
    kind: OmenaFixtureExpectationKindV0,
    detail: impl Into<String>,
) -> OmenaFixtureExpectationOutcomeV0 {
    OmenaFixtureExpectationOutcomeV0 {
        key: expectation.key.clone(),
        kind,
        evaluated: false,
        satisfied: false,
        detail: detail.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::parse_omena_fixture_v0;

    fn diag(code: &str) -> OmenaFixtureDiagnosticV0 {
        OmenaFixtureDiagnosticV0::new(code)
    }

    fn boundary(reference: &str, state: &str) -> OmenaFixtureBoundaryStateV0 {
        OmenaFixtureBoundaryStateV0::new(reference, state)
    }

    fn cascade(winner: &str, witnesses: &[&str]) -> OmenaFixtureCascadeV0 {
        OmenaFixtureCascadeV0::new(winner, witnesses.iter().map(|id| id.to_string()))
    }

    const DIAGNOSTIC_FIXTURE: &str = r#"//- src/Card.module.scss dialect:scss
.card { color: red; }
--- expect: diagnostic
code: missingSassSymbol
--- expect: no-diagnostic missingKeyframes
--- expect: count missingSassSymbol:1
"#;

    #[test]
    fn evaluates_passing_diagnostic_family() -> Result<(), String> {
        let fixture = parse_omena_fixture_v0(DIAGNOSTIC_FIXTURE)?;
        // missingSassSymbol present once, missingKeyframes absent.
        let diagnostics = [diag("missingSassSymbol")];
        let outcomes = evaluate_omena_fixture_v0(&fixture, &diagnostics, &[], &[]);

        assert_eq!(outcomes.len(), 3);
        assert!(outcomes.iter().all(|outcome| outcome.evaluated));
        assert!(
            outcomes.iter().all(|outcome| outcome.satisfied),
            "all diagnostic-family assertions should pass: {outcomes:?}"
        );
        Ok(())
    }

    #[test]
    fn fails_when_expected_diagnostic_is_absent() -> Result<(), String> {
        let fixture = parse_omena_fixture_v0(DIAGNOSTIC_FIXTURE)?;
        // Wrong engine output: the expected `missingSassSymbol` never appears.
        let diagnostics = [diag("missingKeyframes")];
        let outcomes = evaluate_omena_fixture_v0(&fixture, &diagnostics, &[], &[]);

        let diagnostic = &outcomes[0];
        assert_eq!(diagnostic.kind, OmenaFixtureExpectationKindV0::Diagnostic);
        assert!(diagnostic.evaluated);
        assert!(
            !diagnostic.satisfied,
            "absent expected diagnostic must fail: {diagnostic:?}"
        );

        // no-diagnostic missingKeyframes now fails because it appeared.
        let no_diagnostic = &outcomes[1];
        assert_eq!(
            no_diagnostic.kind,
            OmenaFixtureExpectationKindV0::NoDiagnostic
        );
        assert!(!no_diagnostic.satisfied);

        // count missingSassSymbol:1 now fails: observed 0.
        let count = &outcomes[2];
        assert_eq!(count.kind, OmenaFixtureExpectationKindV0::Count);
        assert!(!count.satisfied);
        Ok(())
    }

    #[test]
    fn correct_fixture_does_not_spuriously_fail() -> Result<(), String> {
        // Over-correction guard: a fully-correct fixture must report zero
        // failures across all live families.
        let fixture = parse_omena_fixture_v0(
            r#"//- src/Card.module.scss dialect:scss
.card { color: red; }
--- expect: count missingSassSymbol:2
--- expect: no-diagnostic missingKeyframes
--- expect: boundary-state ext-1 Resolved
"#,
        )?;
        let diagnostics = [diag("missingSassSymbol"), diag("missingSassSymbol")];
        let boundaries = [boundary("ext-1", "resolved")];
        let outcomes = evaluate_omena_fixture_v0(&fixture, &diagnostics, &boundaries, &[]);

        let live_failures = outcomes
            .iter()
            .filter(|outcome| outcome.evaluated && !outcome.satisfied)
            .collect::<Vec<_>>();
        assert!(
            live_failures.is_empty(),
            "correct fixture must not spuriously fail: {live_failures:?}"
        );
        Ok(())
    }

    #[test]
    fn boundary_state_family_matches_lattice_case_insensitively() -> Result<(), String> {
        let fixture = parse_omena_fixture_v0(
            r#"//- src/Card.module.scss dialect:scss
.card { color: red; }
--- expect: boundary-state ext-1 Resolved
--- expect: boundary-state ext-2 Partial
"#,
        )?;
        // ext-1 matches (case-insensitive), ext-2 is Stale not Partial → fail.
        let boundaries = [boundary("ext-1", "resolved"), boundary("ext-2", "stale")];
        let outcomes = evaluate_omena_fixture_v0(&fixture, &[], &boundaries, &[]);

        assert!(outcomes[0].satisfied, "{:?}", outcomes[0]);
        assert!(!outcomes[1].satisfied, "{:?}", outcomes[1]);
        Ok(())
    }

    #[test]
    fn missing_boundary_reference_fails_evaluated() -> Result<(), String> {
        let fixture = parse_omena_fixture_v0(
            r#"//- src/Card.module.scss dialect:scss
.card { color: red; }
--- expect: boundary-state ext-9 Resolved
"#,
        )?;
        let outcomes = evaluate_omena_fixture_v0(&fixture, &[], &[], &[]);

        assert!(outcomes[0].evaluated);
        assert!(!outcomes[0].satisfied);
        assert!(outcomes[0].detail.contains("not present"));
        Ok(())
    }

    const CASCADE_FIXTURE: &str = r#"//- src/Card.module.scss dialect:scss
.card { color: red; }
--- expect: cascade-outcome decl-1
--- expect: cascade-witness decl-2
"#;

    #[test]
    fn evaluates_passing_cascade_families() -> Result<(), String> {
        let fixture = parse_omena_fixture_v0(CASCADE_FIXTURE)?;
        // decl-1 won; decl-2 participated as an also-considered witness.
        let cascades = [cascade("decl-1", &["decl-2"])];
        let outcomes = evaluate_omena_fixture_v0(&fixture, &[], &[], &cascades);

        assert_eq!(outcomes.len(), 2);
        let outcome_kinds = outcomes.iter().map(|o| o.kind).collect::<Vec<_>>();
        assert_eq!(
            outcome_kinds,
            vec![
                OmenaFixtureExpectationKindV0::CascadeOutcome,
                OmenaFixtureExpectationKindV0::CascadeWitness,
            ]
        );
        assert!(
            outcomes.iter().all(|outcome| outcome.evaluated),
            "cascade families are now evaluated, not deferred: {outcomes:?}"
        );
        assert!(
            outcomes.iter().all(|outcome| outcome.satisfied),
            "correct cascade fixture must pass: {outcomes:?}"
        );
        Ok(())
    }

    #[test]
    fn fails_on_wrong_cascade_winner_and_absent_witness() -> Result<(), String> {
        let fixture = parse_omena_fixture_v0(CASCADE_FIXTURE)?;
        // Wrong engine output: decl-9 won (not decl-1) and decl-2 never appears.
        let cascades = [cascade("decl-9", &["decl-7"])];
        let outcomes = evaluate_omena_fixture_v0(&fixture, &[], &[], &cascades);

        let outcome = &outcomes[0];
        assert_eq!(outcome.kind, OmenaFixtureExpectationKindV0::CascadeOutcome);
        assert!(outcome.evaluated);
        assert!(
            !outcome.satisfied,
            "wrong cascade winner must fail: {outcome:?}"
        );

        let witness = &outcomes[1];
        assert_eq!(witness.kind, OmenaFixtureExpectationKindV0::CascadeWitness);
        assert!(witness.evaluated);
        assert!(
            !witness.satisfied,
            "absent cascade witness must fail: {witness:?}"
        );
        Ok(())
    }

    #[test]
    fn cascade_winner_is_always_its_own_witness() -> Result<(), String> {
        // Over-correction guard: a `cascade-witness` naming the winner passes
        // even when the consumer did not repeat the winner in the witness list.
        let fixture = parse_omena_fixture_v0(
            r#"//- src/Card.module.scss dialect:scss
.card { color: red; }
--- expect: cascade-witness decl-1
"#,
        )?;
        let cascades = [cascade("decl-1", &[])];
        let outcomes = evaluate_omena_fixture_v0(&fixture, &[], &[], &cascades);

        assert!(outcomes[0].evaluated);
        assert!(outcomes[0].satisfied, "{:?}", outcomes[0]);
        Ok(())
    }

    #[test]
    fn cascade_family_fails_when_no_cascade_supplied() -> Result<(), String> {
        // The seed-corpus path passes no cascades: a cascade assertion must then
        // fail as absent (evaluated), never silently pass.
        let fixture = parse_omena_fixture_v0(CASCADE_FIXTURE)?;
        let outcomes = evaluate_omena_fixture_v0(&fixture, &[], &[], &[]);

        for outcome in &outcomes {
            assert!(
                outcome.evaluated,
                "cascade family is now evaluated: {outcome:?}"
            );
            assert!(
                !outcome.satisfied,
                "cascade assertion with no engine cascade must fail: {outcome:?}"
            );
        }
        Ok(())
    }

    #[test]
    fn malformed_cascade_outcome_fails_as_evaluated() -> Result<(), String> {
        let fixture = parse_omena_fixture_v0(
            r#"//- src/Card.module.scss dialect:scss
.card { color: red; }
--- expect: cascade-outcome
"#,
        )?;
        let cascades = [cascade("decl-1", &[])];
        let outcomes = evaluate_omena_fixture_v0(&fixture, &[], &[], &cascades);

        assert_eq!(
            outcomes[0].kind,
            OmenaFixtureExpectationKindV0::CascadeOutcome
        );
        assert!(outcomes[0].evaluated);
        assert!(!outcomes[0].satisfied);
        Ok(())
    }

    #[test]
    fn injected_closure_variant_flattens_per_file_diagnostics() -> Result<(), String> {
        let fixture = parse_omena_fixture_v0(DIAGNOSTIC_FIXTURE)?;
        // The consumer supplies the engine via a closure; here we emit the
        // expected diagnostic for the scss file only.
        let outcomes = evaluate_omena_fixture_v0_with(&fixture, &[], &[], |file| {
            if file.path.ends_with(".scss") {
                vec![diag("missingSassSymbol")]
            } else {
                Vec::new()
            }
        });

        assert!(
            outcomes.iter().all(|outcome| outcome.satisfied),
            "closure-fed evaluation should pass: {outcomes:?}"
        );
        Ok(())
    }

    #[test]
    fn count_zero_passes_when_diagnostic_absent() -> Result<(), String> {
        let fixture = parse_omena_fixture_v0(
            r#"//- src/Card.module.scss dialect:scss
.card { color: red; }
--- expect: count unreachableDeclaration:0
"#,
        )?;
        let outcomes = evaluate_omena_fixture_v0(&fixture, &[], &[], &[]);

        assert!(outcomes[0].evaluated);
        assert!(outcomes[0].satisfied, "{:?}", outcomes[0]);
        Ok(())
    }

    #[test]
    fn malformed_count_fails_as_evaluated() -> Result<(), String> {
        let fixture = parse_omena_fixture_v0(
            r#"//- src/Card.module.scss dialect:scss
.card { color: red; }
--- expect: count missingSassSymbol
"#,
        )?;
        let outcomes = evaluate_omena_fixture_v0(&fixture, &[diag("missingSassSymbol")], &[], &[]);

        assert_eq!(outcomes[0].kind, OmenaFixtureExpectationKindV0::Count);
        assert!(outcomes[0].evaluated);
        assert!(!outcomes[0].satisfied);
        Ok(())
    }
}

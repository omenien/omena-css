use std::{env, error::Error, fs};

use omena_abstract_value::{
    AbstractCssTypedValueV0, AbstractCssValueV0, CssValueGrammarVerdictV0,
    match_and_type_standard_property_value_v0,
};
use omena_value_lattice::ValueNodeV0;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedManifest {
    schema_version: String,
    product: String,
    cases: Vec<SeedCase>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SeedCase {
    id: String,
    property: String,
    value: String,
    expected_valid: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EvidenceReport {
    schema_version: &'static str,
    product: &'static str,
    source_product: String,
    case_count: usize,
    all_expectations_satisfied: bool,
    cases: Vec<EvidenceCase>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EvidenceCase {
    id: String,
    property: String,
    value: String,
    expected_valid: bool,
    verdict: &'static str,
    typed: bool,
    typed_kind: Option<&'static str>,
    scalar_leaf_count: usize,
    root_node_kind: Option<&'static str>,
    raw_preserved: bool,
    expectation_satisfied: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args()
        .nth(1)
        .ok_or("usage: value_grammar_evidence <seed-manifest.json>")?;
    let source = fs::read_to_string(path)?;
    let manifest: SeedManifest = serde_json::from_str(source.as_str())?;
    if manifest.schema_version != "0" {
        return Err("unsupported seed manifest schema".into());
    }
    let cases = manifest
        .cases
        .into_iter()
        .map(evaluate_case)
        .collect::<Vec<_>>();
    let report = EvidenceReport {
        schema_version: "0",
        product: "omena-abstract-value.value-grammar-evidence",
        source_product: manifest.product,
        case_count: cases.len(),
        all_expectations_satisfied: cases.iter().all(|case| case.expectation_satisfied),
        cases,
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn evaluate_case(case: SeedCase) -> EvidenceCase {
    let result = match_and_type_standard_property_value_v0(&case.property, &case.value);
    let matched = result.verdict.is_matched();
    let typed_kind = match &result.abstract_value {
        AbstractCssValueV0::Exact {
            typed: Some(typed), ..
        } => Some(match typed.as_ref() {
            AbstractCssTypedValueV0::Exact { .. } => "exact",
            AbstractCssTypedValueV0::Compound { .. } => "compound",
            AbstractCssTypedValueV0::FiniteSet { .. } => "finiteSet",
            AbstractCssTypedValueV0::Top => "top",
        }),
        _ => None,
    };
    let raw_preserved = matches!(
        &result.abstract_value,
        AbstractCssValueV0::Raw { value } if value == &case.value
    );
    let scalar_leaf_count = result
        .projection
        .as_ref()
        .map_or(0, |projection| projection.scalar_leaves.len());
    let root_node_kind = result
        .projection
        .as_ref()
        .map(|projection| value_node_kind(projection.lattice.root()));
    let expectation_satisfied = if case.expected_valid {
        matched && typed_kind.is_some()
    } else {
        result.verdict.is_definite_mismatch() && raw_preserved
    };
    EvidenceCase {
        id: case.id,
        property: case.property,
        value: case.value.clone(),
        expected_valid: case.expected_valid,
        verdict: verdict_kind(&result.verdict),
        typed: typed_kind.is_some(),
        typed_kind,
        scalar_leaf_count,
        root_node_kind,
        raw_preserved,
        expectation_satisfied,
    }
}

fn verdict_kind(verdict: &CssValueGrammarVerdictV0) -> &'static str {
    match verdict {
        CssValueGrammarVerdictV0::Matched { .. } => "matched",
        CssValueGrammarVerdictV0::Unmatched { .. } => "unmatched",
        CssValueGrammarVerdictV0::NotMatchedWithinBudget { .. } => "notMatchedWithinBudget",
        CssValueGrammarVerdictV0::GrammarDefect { .. } => "grammarDefect",
    }
}

fn value_node_kind(node: &ValueNodeV0<'_>) -> &'static str {
    match node {
        ValueNodeV0::Raw { .. } => "raw",
        ValueNodeV0::Number { .. } => "number",
        ValueNodeV0::Color { .. } => "color",
        ValueNodeV0::Keyword { .. } => "keyword",
        ValueNodeV0::List { .. } => "list",
        ValueNodeV0::Function { .. } => "function",
        ValueNodeV0::SassMap { .. } => "sassMap",
        ValueNodeV0::SassList { .. } => "sassList",
    }
}

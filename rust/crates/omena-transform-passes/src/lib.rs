//! Transform pass registry and DAG planner for the post-v5 omena-css track.
//!
//! This crate consumes `omena-transform-cst` contracts. It does not duplicate
//! the pass catalog; its job is to register every P01-P40 pass and produce a
//! DAG-respecting execution plan for downstream transform crates.

use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;
use omena_transform_cst::{
    TRANSFORM_PASS_CATALOG_LEN, TransformDagEdgeV0, TransformLayer, TransformPassContractV0,
    TransformPassKind, all_transform_pass_kinds, default_transform_dag_edges,
    default_transform_pass_contracts,
};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformPassExecutionStatus {
    RegistryAndPlannerReady,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassRegistryEntryV0 {
    pub contract: TransformPassContractV0,
    pub module_family: &'static str,
    pub query_family: &'static str,
    pub execution_status: TransformPassExecutionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassesBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub registry_entries: Vec<TransformPassRegistryEntryV0>,
    pub dag_edges: Vec<TransformDagEdgeV0>,
    pub pass_count: usize,
    pub full_catalog_registered: bool,
    pub semantic_aware_pass_count: usize,
    pub cascade_aware_pass_count: usize,
    pub planner_enforces_dag_edges: bool,
    pub execution_runtime_ready: bool,
    pub implemented_mutation_pass_ids: Vec<&'static str>,
    pub next_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassPlanV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub requested_pass_ids: Vec<&'static str>,
    pub ordered_pass_ids: Vec<&'static str>,
    pub satisfied_dag_edge_count: usize,
    pub violated_dag_edge_count: usize,
    pub all_requested_registered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformPassRuntimeStatus {
    Applied,
    NoChange,
    PlannedOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassExecutionOutcomeV0 {
    pub pass_id: &'static str,
    pub status: TransformPassRuntimeStatus,
    pub input_byte_len: usize,
    pub output_byte_len: usize,
    pub mutation_count: usize,
    pub provenance_preserved: bool,
    pub detail: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformExecutionSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub input_byte_len: usize,
    pub output_byte_len: usize,
    pub requested_pass_ids: Vec<&'static str>,
    pub ordered_pass_ids: Vec<&'static str>,
    pub executed_pass_ids: Vec<&'static str>,
    pub planned_only_pass_ids: Vec<&'static str>,
    pub mutation_count: usize,
    pub provenance_preserved: bool,
    pub output_css: String,
    pub outcomes: Vec<TransformPassExecutionOutcomeV0>,
    pub pass_plan: TransformPassPlanV0,
}

pub fn summarize_omena_transform_passes_boundary() -> TransformPassesBoundarySummaryV0 {
    let registry_entries = default_transform_pass_contracts()
        .into_iter()
        .map(registry_entry_for_contract)
        .collect::<Vec<_>>();
    let pass_count = registry_entries.len();
    let semantic_aware_pass_count = registry_entries
        .iter()
        .filter(|entry| entry.contract.layer == TransformLayer::SemanticAware)
        .count();
    let cascade_aware_pass_count = registry_entries
        .iter()
        .filter(|entry| entry.contract.reads_cascade_model)
        .count();

    TransformPassesBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-transform-passes.boundary",
        registry_entries,
        dag_edges: default_transform_dag_edges(),
        pass_count,
        full_catalog_registered: pass_count == TRANSFORM_PASS_CATALOG_LEN,
        semantic_aware_pass_count,
        cascade_aware_pass_count,
        planner_enforces_dag_edges: true,
        execution_runtime_ready: true,
        implemented_mutation_pass_ids: implemented_mutation_pass_ids(),
        next_surfaces: vec![
            "remainingPassMutationEngines",
            "transformSalsaQueries",
            "omena-transform-bundle",
            "omena-transform-print",
        ],
    }
}

pub fn plan_transform_passes(requested: &[TransformPassKind]) -> TransformPassPlanV0 {
    let requested_pass_ids = requested.iter().map(|pass| pass.id()).collect::<Vec<_>>();
    let ordered_passes = order_passes_by_dag(requested);
    let ordered_pass_ids = ordered_passes
        .iter()
        .map(|pass| pass.id())
        .collect::<Vec<_>>();
    let dag_edges = default_transform_dag_edges();
    let satisfied_dag_edge_count = dag_edges
        .iter()
        .filter(|edge| {
            edge_applies(edge, &ordered_pass_ids) && edge_is_satisfied(edge, &ordered_pass_ids)
        })
        .count();
    let violated_dag_edge_count = dag_edges
        .iter()
        .filter(|edge| {
            edge_applies(edge, &ordered_pass_ids) && !edge_is_satisfied(edge, &ordered_pass_ids)
        })
        .count();

    TransformPassPlanV0 {
        schema_version: "0",
        product: "omena-transform-passes.plan",
        requested_pass_ids,
        ordered_pass_ids,
        satisfied_dag_edge_count,
        violated_dag_edge_count,
        all_requested_registered: requested.iter().all(pass_is_registered),
    }
}

pub fn execute_transform_passes_on_source(
    source: &str,
    requested: &[TransformPassKind],
) -> TransformExecutionSummaryV0 {
    execute_transform_passes_on_source_with_dialect(source, StyleDialect::Css, requested)
}

pub fn execute_transform_passes_on_source_with_dialect(
    source: &str,
    dialect: StyleDialect,
    requested: &[TransformPassKind],
) -> TransformExecutionSummaryV0 {
    let pass_plan = plan_transform_passes(requested);
    let requested_pass_ids = requested.iter().map(|pass| pass.id()).collect::<Vec<_>>();
    let ordered_pass_ids = pass_plan.ordered_pass_ids.clone();
    let mut output_css = source.to_string();
    let mut outcomes = Vec::new();

    for pass_id in &ordered_pass_ids {
        let pass = transform_pass_kind_from_id(pass_id);
        let input_byte_len = output_css.len();
        let outcome = match pass {
            Some(TransformPassKind::WhitespaceStrip) => {
                let (next_css, mutation_count) = normalize_css_whitespace(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "normalized lexer trivia where adjacent token boundaries remain unambiguous",
                }
            }
            Some(TransformPassKind::CommentStrip) => {
                let (next_css, mutation_count) = strip_css_comments(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "removed CSS block comments outside string literals",
                }
            }
            Some(TransformPassKind::NumberCompression) => {
                let (next_css, mutation_count) = compress_css_numbers(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "compressed lexer numeric tokens without touching identifiers or strings",
                }
            }
            Some(TransformPassKind::UnitNormalization) => {
                let (next_css, mutation_count) = normalize_css_units(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "normalized zero length units only inside property contexts that accept unitless zero",
                }
            }
            Some(TransformPassKind::ColorCompression) => {
                let (next_css, mutation_count) = compress_css_colors(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "compressed declaration-leading hex color tokens",
                }
            }
            Some(TransformPassKind::UrlQuoteStrip) => {
                let (next_css, mutation_count) = strip_css_url_quotes(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "stripped quotes from safe url() string arguments",
                }
            }
            Some(TransformPassKind::StringQuoteNormalize) => {
                let (next_css, mutation_count) = normalize_css_string_quotes(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "normalized safe single-quoted CSS string tokens",
                }
            }
            Some(TransformPassKind::PrintCss) => TransformPassExecutionOutcomeV0 {
                pass_id,
                status: TransformPassRuntimeStatus::NoChange,
                input_byte_len,
                output_byte_len: output_css.len(),
                mutation_count: 0,
                provenance_preserved: true,
                detail: "observed final emission boundary",
            },
            Some(_) | None => TransformPassExecutionOutcomeV0 {
                pass_id,
                status: TransformPassRuntimeStatus::PlannedOnly,
                input_byte_len,
                output_byte_len: output_css.len(),
                mutation_count: 0,
                provenance_preserved: true,
                detail: "registered in DAG planner; mutation engine not implemented yet",
            },
        };
        outcomes.push(outcome);
    }

    let executed_pass_ids = outcomes
        .iter()
        .filter(|outcome| outcome.status != TransformPassRuntimeStatus::PlannedOnly)
        .map(|outcome| outcome.pass_id)
        .collect::<Vec<_>>();
    let planned_only_pass_ids = outcomes
        .iter()
        .filter(|outcome| outcome.status == TransformPassRuntimeStatus::PlannedOnly)
        .map(|outcome| outcome.pass_id)
        .collect::<Vec<_>>();
    let mutation_count = outcomes
        .iter()
        .map(|outcome| outcome.mutation_count)
        .sum::<usize>();
    let provenance_preserved = outcomes.iter().all(|outcome| outcome.provenance_preserved);
    let output_byte_len = output_css.len();

    TransformExecutionSummaryV0 {
        schema_version: "0",
        product: "omena-transform-passes.execution",
        input_byte_len: source.len(),
        output_byte_len,
        requested_pass_ids,
        ordered_pass_ids,
        executed_pass_ids,
        planned_only_pass_ids,
        mutation_count,
        provenance_preserved,
        output_css,
        outcomes,
        pass_plan,
    }
}

pub fn implemented_mutation_pass_ids() -> Vec<&'static str> {
    vec![
        TransformPassKind::WhitespaceStrip.id(),
        TransformPassKind::CommentStrip.id(),
        TransformPassKind::NumberCompression.id(),
        TransformPassKind::UnitNormalization.id(),
        TransformPassKind::ColorCompression.id(),
        TransformPassKind::UrlQuoteStrip.id(),
        TransformPassKind::StringQuoteNormalize.id(),
        TransformPassKind::PrintCss.id(),
    ]
}

fn registry_entry_for_contract(contract: TransformPassContractV0) -> TransformPassRegistryEntryV0 {
    TransformPassRegistryEntryV0 {
        module_family: module_family_for_pass(contract.kind),
        query_family: query_family_for_pass(contract.kind),
        execution_status: TransformPassExecutionStatus::RegistryAndPlannerReady,
        contract,
    }
}

fn module_family_for_pass(kind: TransformPassKind) -> &'static str {
    match kind.ordinal() {
        1..=7 => "commodity-token",
        8 | 25 => "egg-backed",
        9..=13 => "cascade-proven-structural",
        14..=24 => "target-lowering",
        26..=28 => "module-bundle",
        29..=32 => "css-modules-resolution",
        33..=39 => "semantic-reachability",
        40 => "emission",
        _ => "unknown",
    }
}

fn query_family_for_pass(kind: TransformPassKind) -> &'static str {
    match kind.layer() {
        TransformLayer::SemanticAware => "semantic-aware-transform-query",
        TransformLayer::Commodity => "commodity-transform-query",
        TransformLayer::Emission => "emission-transform-query",
        TransformLayer::SemanticReadOnly => "semantic-read-only-query",
    }
}

fn order_passes_by_dag(requested: &[TransformPassKind]) -> Vec<TransformPassKind> {
    let mut remaining = dedupe_requested_passes(requested);
    remaining.sort_by_key(|kind| (execution_rank(*kind), kind.ordinal()));

    let mut ordered = Vec::with_capacity(remaining.len());
    while !remaining.is_empty() {
        let next_index = match remaining
            .iter()
            .position(|candidate| !has_incoming_edge_from_remaining(*candidate, &remaining))
        {
            Some(index) => index,
            None => 0,
        };
        ordered.push(remaining.remove(next_index));
    }

    ordered
}

fn dedupe_requested_passes(requested: &[TransformPassKind]) -> Vec<TransformPassKind> {
    let mut unique = Vec::new();
    for pass in requested {
        if !unique.contains(pass) {
            unique.push(*pass);
        }
    }
    unique
}

fn has_incoming_edge_from_remaining(
    candidate: TransformPassKind,
    remaining: &[TransformPassKind],
) -> bool {
    default_transform_dag_edges().iter().any(|edge| {
        edge.to == candidate.id()
            && remaining
                .iter()
                .any(|other| other.id() == edge.from && *other != candidate)
    })
}

fn edge_applies(edge: &TransformDagEdgeV0, ordered_pass_ids: &[&'static str]) -> bool {
    ordered_pass_ids.contains(&edge.from) && ordered_pass_ids.contains(&edge.to)
}

fn edge_is_satisfied(edge: &TransformDagEdgeV0, ordered_pass_ids: &[&'static str]) -> bool {
    let from = position_of_pass_id(edge.from, ordered_pass_ids);
    let to = position_of_pass_id(edge.to, ordered_pass_ids);
    match (from, to) {
        (Some(from), Some(to)) => from < to,
        _ => false,
    }
}

fn position_of_pass_id(pass_id: &'static str, ordered_pass_ids: &[&'static str]) -> Option<usize> {
    ordered_pass_ids
        .iter()
        .position(|ordered_pass_id| *ordered_pass_id == pass_id)
}

fn pass_is_registered(pass: &TransformPassKind) -> bool {
    default_transform_pass_contracts()
        .iter()
        .any(|contract| contract.kind == *pass)
}

fn transform_pass_kind_from_id(pass_id: &str) -> Option<TransformPassKind> {
    all_transform_pass_kinds()
        .into_iter()
        .find(|kind| kind.id() == pass_id)
}

fn execution_rank(kind: TransformPassKind) -> u8 {
    match kind.ordinal() {
        26..=28 => 10,
        29..=39 => 20,
        14..=24 => 30,
        8..=13 | 25 => 40,
        1..=7 => 50,
        40 => 60,
        _ => 70,
    }
}

fn strip_css_comments(source: &str, dialect: StyleDialect) -> (String, usize) {
    strip_css_comments_with_lexer(source, dialect)
}

fn compress_css_numbers(source: &str, dialect: StyleDialect) -> (String, usize) {
    compress_css_numbers_with_lexer(source, dialect)
}

fn compress_css_colors(source: &str, dialect: StyleDialect) -> (String, usize) {
    compress_css_colors_with_lexer(source, dialect)
}

fn normalize_css_units(source: &str, dialect: StyleDialect) -> (String, usize) {
    normalize_css_units_with_lexer(source, dialect)
}

fn strip_css_url_quotes(source: &str, dialect: StyleDialect) -> (String, usize) {
    strip_css_url_quotes_with_lexer(source, dialect)
}

fn normalize_css_string_quotes(source: &str, dialect: StyleDialect) -> (String, usize) {
    normalize_css_string_quotes_with_lexer(source, dialect)
}

fn normalize_css_whitespace(source: &str, dialect: StyleDialect) -> (String, usize) {
    normalize_css_whitespace_with_lexer(source, dialect)
}

fn normalize_css_string_quotes_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    rewrite_lexer_tokens(source, dialect, |kind, text| {
        if kind == SyntaxKind::String {
            return normalize_css_string_token_quotes(text);
        }
        None
    })
}

fn normalize_css_string_token_quotes(text: &str) -> Option<String> {
    if !text.starts_with('\'') || !text.ends_with('\'') || text.len() < 2 {
        return None;
    }
    let inner = &text[1..text.len() - 1];
    if inner
        .chars()
        .any(|ch| matches!(ch, '"' | '\\' | '\n' | '\r'))
    {
        return None;
    }

    Some(format!("\"{inner}\""))
}

fn strip_css_url_quotes_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut output = String::with_capacity(source.len());
    let mut index = 0;
    let mut mutation_count = 0;

    while index < tokens.len() {
        if let Some((replacement, consumed)) = rewrite_safe_quoted_url(tokens, index) {
            output.push_str(&replacement);
            mutation_count += 1;
            index += consumed;
            continue;
        }

        output.push_str(&tokens[index].text);
        index += 1;
    }

    (output, mutation_count)
}

fn rewrite_safe_quoted_url(
    tokens: &[omena_parser::LexedToken],
    index: usize,
) -> Option<(String, usize)> {
    let ident = tokens.get(index)?;
    let left_paren = tokens.get(index + 1)?;
    let string = tokens.get(index + 2)?;
    let right_paren = tokens.get(index + 3)?;

    if ident.kind != SyntaxKind::Ident
        || !ident.text.eq_ignore_ascii_case("url")
        || left_paren.kind != SyntaxKind::LeftParen
        || string.kind != SyntaxKind::String
        || right_paren.kind != SyntaxKind::RightParen
    {
        return None;
    }

    let inner = unquote_safe_url_string(&string.text)?;
    Some((format!("{}({inner})", ident.text), 4))
}

fn unquote_safe_url_string(text: &str) -> Option<&str> {
    let quote = text.as_bytes().first().copied()?;
    if quote != b'\'' && quote != b'"' {
        return None;
    }
    if text.as_bytes().last().copied() != Some(quote) || text.len() < 2 {
        return None;
    }

    let inner = &text[1..text.len() - 1];
    if inner
        .chars()
        .any(|ch| ch.is_whitespace() || matches!(ch, '"' | '\'' | '(' | ')' | '\\'))
    {
        return None;
    }

    Some(inner)
}

fn compress_css_colors_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut output = String::with_capacity(source.len());
    let mut mutation_count = 0;

    for (index, token) in tokens.iter().enumerate() {
        let replacement = if token.kind == SyntaxKind::Hash
            && previous_non_comment_token_kind(tokens, index) == Some(SyntaxKind::Colon)
        {
            compress_hex_color_token_text(&token.text)
        } else {
            None
        };

        if let Some(replacement) = replacement {
            if replacement != token.text {
                mutation_count += 1;
            }
            output.push_str(&replacement);
        } else {
            output.push_str(&token.text);
        }
    }

    (output, mutation_count)
}

fn normalize_css_units_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let mut output = String::with_capacity(source.len());
    let mut mutation_count = 0;
    let mut property_candidate: Option<String> = None;
    let mut active_property: Option<String> = None;
    let mut awaiting_property = false;

    for token in lexed.tokens() {
        if is_declaration_boundary_start(token.kind) {
            awaiting_property = true;
            property_candidate = None;
            active_property = None;
        } else if is_declaration_boundary_end(token.kind) {
            awaiting_property = token.kind == SyntaxKind::Semicolon;
            property_candidate = None;
            active_property = None;
        } else if token.kind == SyntaxKind::Colon && awaiting_property {
            active_property = property_candidate.clone();
            awaiting_property = false;
        } else if awaiting_property
            && !is_comment_token(token.kind)
            && token.kind != SyntaxKind::Whitespace
        {
            if matches!(
                token.kind,
                SyntaxKind::Ident | SyntaxKind::CustomPropertyName
            ) {
                property_candidate = Some(token.text.to_ascii_lowercase());
            } else {
                awaiting_property = false;
                property_candidate = None;
            }
        }

        let replacement = if token.kind == SyntaxKind::Dimension
            && active_property
                .as_deref()
                .is_some_and(is_zero_length_unit_property)
        {
            normalize_zero_length_dimension_token(&token.text)
        } else {
            None
        };

        if let Some(replacement) = replacement {
            if replacement != token.text {
                mutation_count += 1;
            }
            output.push_str(&replacement);
        } else {
            output.push_str(&token.text);
        }
    }

    (output, mutation_count)
}

fn is_declaration_boundary_start(kind: SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::LeftBrace | SyntaxKind::Semicolon)
}

fn is_declaration_boundary_end(kind: SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::RightBrace | SyntaxKind::Semicolon)
}

fn is_zero_length_unit_property(property: &str) -> bool {
    matches!(
        property,
        "margin"
            | "margin-block"
            | "margin-block-end"
            | "margin-block-start"
            | "margin-bottom"
            | "margin-inline"
            | "margin-inline-end"
            | "margin-inline-start"
            | "margin-left"
            | "margin-right"
            | "margin-top"
            | "padding"
            | "padding-block"
            | "padding-block-end"
            | "padding-block-start"
            | "padding-bottom"
            | "padding-inline"
            | "padding-inline-end"
            | "padding-inline-start"
            | "padding-left"
            | "padding-right"
            | "padding-top"
            | "inset"
            | "inset-block"
            | "inset-block-end"
            | "inset-block-start"
            | "inset-inline"
            | "inset-inline-end"
            | "inset-inline-start"
            | "top"
            | "right"
            | "bottom"
            | "left"
            | "width"
            | "min-width"
            | "max-width"
            | "height"
            | "min-height"
            | "max-height"
            | "block-size"
            | "min-block-size"
            | "max-block-size"
            | "inline-size"
            | "min-inline-size"
            | "max-inline-size"
            | "gap"
            | "row-gap"
            | "column-gap"
    )
}

fn normalize_zero_length_dimension_token(text: &str) -> Option<String> {
    let split = numeric_prefix_end(text)?;
    let (number, unit) = text.split_at(split);
    if !is_zero_number_prefix(number) || !is_css_length_unit(unit) {
        return None;
    }

    Some("0".to_string())
}

fn is_zero_number_prefix(number: &str) -> bool {
    number.parse::<f64>().is_ok_and(|value| value == 0.0)
}

fn is_css_length_unit(unit: &str) -> bool {
    matches!(
        unit.to_ascii_lowercase().as_str(),
        "cap"
            | "ch"
            | "cm"
            | "em"
            | "ex"
            | "ic"
            | "in"
            | "lh"
            | "mm"
            | "pc"
            | "pt"
            | "px"
            | "q"
            | "rem"
            | "rlh"
            | "vb"
            | "vh"
            | "vi"
            | "vmax"
            | "vmin"
            | "vw"
    )
}

fn compress_hex_color_token_text(text: &str) -> Option<String> {
    let hex = text.strip_prefix('#')?;
    if !matches!(hex.len(), 3 | 4 | 6 | 8) || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return None;
    }

    let lower = hex.to_ascii_lowercase();
    let compressed = match lower.len() {
        6 if can_shorten_hex_pairs(&lower) => shorten_hex_pairs(&lower),
        8 if can_shorten_hex_pairs(&lower) => shorten_hex_pairs(&lower),
        _ => lower,
    };
    let rewritten = format!("#{compressed}");
    (rewritten != text).then_some(rewritten)
}

fn can_shorten_hex_pairs(hex: &str) -> bool {
    hex.as_bytes()
        .chunks_exact(2)
        .all(|pair| pair[0] == pair[1])
}

fn shorten_hex_pairs(hex: &str) -> String {
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| pair[0] as char)
        .collect()
}

fn compress_css_numbers_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    rewrite_lexer_tokens(source, dialect, |kind, text| {
        if matches!(
            kind,
            SyntaxKind::Number | SyntaxKind::Percentage | SyntaxKind::Dimension
        ) {
            return compress_numeric_token_text(text);
        }
        None
    })
}

fn rewrite_lexer_tokens(
    source: &str,
    dialect: StyleDialect,
    mut rewrite: impl FnMut(SyntaxKind, &str) -> Option<String>,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut mutation_count = 0;

    for token in lexed.tokens() {
        let start = u32::from(token.range.start()) as usize;
        let end = u32::from(token.range.end()) as usize;
        if start > cursor {
            output.push_str(&source[cursor..start]);
        }
        if let Some(replacement) = rewrite(token.kind, &token.text) {
            if replacement != token.text {
                mutation_count += 1;
            }
            output.push_str(&replacement);
        } else {
            output.push_str(&source[start..end]);
        }
        cursor = end;
    }

    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, mutation_count)
}

fn compress_numeric_token_text(text: &str) -> Option<String> {
    let split = numeric_prefix_end(text)?;
    let (number, suffix) = text.split_at(split);
    let compressed = compress_number_prefix(number);
    let rewritten = format!("{compressed}{suffix}");
    (rewritten != text).then_some(rewritten)
}

fn numeric_prefix_end(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut index = 0;

    if matches!(bytes.get(index), Some(b'+') | Some(b'-')) {
        index += 1;
    }

    let integer_start = index;
    while matches!(bytes.get(index), Some(b'0'..=b'9')) {
        index += 1;
    }
    let saw_integer_digit = index > integer_start;

    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let fraction_start = index;
        while matches!(bytes.get(index), Some(b'0'..=b'9')) {
            index += 1;
        }
        if !saw_integer_digit && index == fraction_start {
            return None;
        }
    } else if !saw_integer_digit {
        return None;
    }

    if matches!(bytes.get(index), Some(b'e') | Some(b'E')) {
        let exponent_marker = index;
        let mut exponent_index = index + 1;
        if matches!(bytes.get(exponent_index), Some(b'+') | Some(b'-')) {
            exponent_index += 1;
        }
        let exponent_digit_start = exponent_index;
        while matches!(bytes.get(exponent_index), Some(b'0'..=b'9')) {
            exponent_index += 1;
        }
        if exponent_index > exponent_digit_start {
            index = exponent_index;
        } else {
            index = exponent_marker;
        }
    }

    Some(index)
}

fn compress_number_prefix(number: &str) -> String {
    let (sign, unsigned) = match number.as_bytes().first() {
        Some(b'+') | Some(b'-') => (&number[..1], &number[1..]),
        _ => ("", number),
    };
    let Some((before_dot, after_dot)) = unsigned.split_once('.') else {
        return number.to_string();
    };

    let trimmed_fraction = after_dot.trim_end_matches('0');
    let mut compressed_unsigned = if trimmed_fraction.is_empty() {
        before_dot.to_string()
    } else {
        format!("{before_dot}.{trimmed_fraction}")
    };

    if let Some(rest) = compressed_unsigned.strip_prefix("0.") {
        compressed_unsigned = format!(".{rest}");
    }

    if compressed_unsigned.is_empty() {
        compressed_unsigned.push('0');
    }

    format!("{sign}{compressed_unsigned}")
}

fn normalize_css_whitespace_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut output = String::with_capacity(source.len());
    let mut mutation_count = 0;

    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Whitespace && token.kind != SyntaxKind::SassIndentedNewline {
            output.push_str(&token.text);
            continue;
        }

        let replacement = whitespace_replacement_for_tokens(
            previous_non_comment_token_kind(tokens, index),
            next_non_comment_token_kind(tokens, index),
        );
        if replacement != token.text {
            mutation_count += 1;
        }
        output.push_str(replacement);
    }

    (output, mutation_count)
}

fn whitespace_replacement_for_tokens(
    previous: Option<SyntaxKind>,
    next: Option<SyntaxKind>,
) -> &'static str {
    match (previous, next) {
        (None, _) | (_, None) => "",
        (Some(previous), Some(next))
            if can_remove_whitespace_after(previous) || can_remove_whitespace_before(next) =>
        {
            ""
        }
        _ => " ",
    }
}

fn previous_non_comment_token_kind(
    tokens: &[omena_parser::LexedToken],
    index: usize,
) -> Option<SyntaxKind> {
    tokens[..index]
        .iter()
        .rev()
        .find(|token| !is_comment_token(token.kind) && token.kind != SyntaxKind::Whitespace)
        .map(|token| token.kind)
}

fn next_non_comment_token_kind(
    tokens: &[omena_parser::LexedToken],
    index: usize,
) -> Option<SyntaxKind> {
    tokens
        .get(index + 1..)
        .unwrap_or_default()
        .iter()
        .find(|token| !is_comment_token(token.kind) && token.kind != SyntaxKind::Whitespace)
        .map(|token| token.kind)
}

fn can_remove_whitespace_after(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::LeftBrace
            | SyntaxKind::RightBrace
            | SyntaxKind::LeftParen
            | SyntaxKind::LeftBracket
            | SyntaxKind::Comma
            | SyntaxKind::Semicolon
    )
}

fn can_remove_whitespace_before(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::LeftBrace
            | SyntaxKind::RightBrace
            | SyntaxKind::RightParen
            | SyntaxKind::RightBracket
            | SyntaxKind::Comma
            | SyntaxKind::Semicolon
    )
}

fn strip_css_comments_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut removed_comment_count = 0;

    for token in lexed.tokens() {
        let start = u32::from(token.range.start()) as usize;
        let end = u32::from(token.range.end()) as usize;
        if start > cursor {
            output.push_str(&source[cursor..start]);
        }
        if is_comment_token(token.kind) {
            removed_comment_count += 1;
        } else {
            output.push_str(&source[start..end]);
        }
        cursor = end;
    }

    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, removed_comment_count)
}

fn is_comment_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::LineComment | SyntaxKind::BlockComment | SyntaxKind::ScssSilentComment
    )
}

#[cfg(test)]
mod tests {
    use super::{
        TransformPassRuntimeStatus, execute_transform_passes_on_source,
        execute_transform_passes_on_source_with_dialect, plan_transform_passes,
        summarize_omena_transform_passes_boundary,
    };
    use omena_parser::StyleDialect;
    use omena_transform_cst::{TRANSFORM_PASS_CATALOG_LEN, TransformPassKind};

    #[test]
    fn registry_covers_full_p01_to_p40_catalog() {
        let boundary = summarize_omena_transform_passes_boundary();

        assert_eq!(boundary.schema_version, "0");
        assert_eq!(boundary.product, "omena-transform-passes.boundary");
        assert_eq!(boundary.pass_count, TRANSFORM_PASS_CATALOG_LEN);
        assert!(boundary.full_catalog_registered);
        assert_eq!(boundary.semantic_aware_pass_count, 14);
        assert!(boundary.cascade_aware_pass_count >= 9);
        assert!(boundary.planner_enforces_dag_edges);
        assert!(boundary.execution_runtime_ready);
        assert_eq!(
            boundary.implemented_mutation_pass_ids,
            vec![
                "p01-whitespace-strip",
                "p02-comment-strip",
                "p03-number-compression",
                "p04-unit-normalization",
                "p05-color-compression",
                "p06-url-quote-strip",
                "p07-string-quote-normalize",
                "p40-print-css"
            ]
        );
        assert!(boundary.registry_entries.iter().any(|entry| {
            entry.contract.kind == TransformPassKind::TreeShakeClass
                && entry.module_family == "semantic-reachability"
        }));
    }

    #[test]
    fn planner_respects_var_before_calc_before_print_edges() {
        let plan = plan_transform_passes(&[
            TransformPassKind::PrintCss,
            TransformPassKind::CalcReduction,
            TransformPassKind::StaticVarSubstitution,
        ]);

        assert_eq!(plan.violated_dag_edge_count, 0);
        assert!(plan.all_requested_registered);
        assert_eq!(
            plan.ordered_pass_ids,
            vec![
                "p32-custom-property-static-resolve",
                "p25-calc-reduction",
                "p40-print-css"
            ]
        );
    }

    #[test]
    fn planner_respects_composes_before_hash_before_selector_merge_edges() {
        let plan = plan_transform_passes(&[
            TransformPassKind::SelectorMerging,
            TransformPassKind::HashCssModuleClassNames,
            TransformPassKind::ResolveCssModulesComposes,
        ]);

        assert_eq!(plan.violated_dag_edge_count, 0);
        assert_eq!(
            plan.ordered_pass_ids,
            vec![
                "p30-composes-resolution",
                "p29-css-modules-class-hashing",
                "p12-selector-merging"
            ]
        );
    }

    #[test]
    fn execution_runtime_applies_comment_strip_without_touching_strings() {
        let source = r#".a { color: red; /* remove */ content: "/* keep */"; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::CommentStrip,
                TransformPassKind::HashCssModuleClassNames,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.product, "omena-transform-passes.execution");
        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            r#".a { color: red;  content: "/* keep */"; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["p02-comment-strip", "p40-print-css"]
        );
        assert_eq!(
            execution.planned_only_pass_ids,
            vec!["p29-css-modules-class-hashing"]
        );
        assert!(execution.provenance_preserved);
        assert_eq!(execution.pass_plan.violated_dag_edge_count, 0);
        assert!(execution.outcomes.iter().any(|outcome| {
            outcome.pass_id == "p02-comment-strip"
                && outcome.status == TransformPassRuntimeStatus::Applied
                && outcome.mutation_count == 1
        }));
        assert!(execution.outcomes.iter().any(|outcome| {
            outcome.pass_id == "p29-css-modules-class-hashing"
                && outcome.status == TransformPassRuntimeStatus::PlannedOnly
        }));
    }

    #[test]
    fn execution_runtime_applies_conservative_whitespace_normalization() {
        let source = r#".a , .b { color : red ; content: "x y"; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::WhitespaceStrip,
                TransformPassKind::CommentStrip,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 7);
        assert_eq!(
            execution.output_css,
            r#".a,.b{color : red;content: "x y";}"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["p01-whitespace-strip", "p02-comment-strip", "p40-print-css"]
        );
    }

    #[test]
    fn execution_runtime_compresses_numeric_tokens_only() {
        let source =
            r#".a { width: 0.50rem; opacity: 1.0; margin: -0.25px 10.00%; content: "0.50"; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::NumberCompression,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 4);
        assert_eq!(
            execution.output_css,
            r#".a { width: .5rem; opacity: 1; margin: -.25px 10%; content: "0.50"; }"#
        );
    }

    #[test]
    fn execution_runtime_normalizes_zero_length_units_with_property_context() {
        let source = r#".a { margin: 0px 0.0rem -0em; rotate: 0deg; animation-delay: 0s; --x: 0px; width: 10px; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::UnitNormalization,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 3);
        assert_eq!(
            execution.output_css,
            r#".a { margin: 0 0 0; rotate: 0deg; animation-delay: 0s; --x: 0px; width: 10px; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["p04-unit-normalization", "p40-print-css"]
        );
    }

    #[test]
    fn execution_runtime_compresses_declaration_leading_hex_colors_only() {
        let source = r#".a { color: #FFFFFF; box-shadow: 0 0 #AABBCC; } #FFFFFF { color: red; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::ColorCompression,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            r#".a { color: #fff; box-shadow: 0 0 #AABBCC; } #FFFFFF { color: red; }"#
        );
    }

    #[test]
    fn execution_runtime_strips_safe_url_quotes_only() {
        let source = r#".a { background: url("img/icon.svg"); mask: url("has space.svg"); content: "url(\"keep\")"; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::UrlQuoteStrip,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            r#".a { background: url(img/icon.svg); mask: url("has space.svg"); content: "url(\"keep\")"; }"#
        );
    }

    #[test]
    fn execution_runtime_normalizes_safe_single_quoted_strings_only() {
        let source =
            r#".a { font-family: 'Demo'; content: 'has "quote"'; background: url('asset.svg'); }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::StringQuoteNormalize,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 2);
        assert_eq!(
            execution.output_css,
            r#".a { font-family: "Demo"; content: 'has "quote"'; background: url("asset.svg"); }"#
        );
    }

    #[test]
    fn execution_runtime_uses_dialect_lexer_for_scss_silent_comments() {
        let source = ".a { // remove\n  color: red;\n  content: \"// keep\";\n}";
        let execution = execute_transform_passes_on_source_with_dialect(
            source,
            StyleDialect::Scss,
            &[TransformPassKind::CommentStrip],
        );

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            ".a { \n  color: red;\n  content: \"// keep\";\n}"
        );
    }
}

use cstree::text::{TextRange, TextSize};
use omena_abstract_value::{
    RegisteredPropertySyntaxV0, RegisteredSyntaxMatchV0, parse_registered_property_syntax_v0,
    registered_syntax_match,
};
use omena_cascade::{
    StaticSupportsAssumptionV0, StaticSupportsEvalVerdictV0, StaticSupportsEvalWitnessV0,
    evaluate_static_supports_condition,
};
use omena_parser::{LexedToken, StyleDialect, lex};
use omena_syntax::SyntaxKind;
use omena_transform_cst::{IrNodeKindV0, IrNodeV0, TransformIrV0};
use serde::Serialize;
use std::collections::BTreeMap;

use crate::control_flow::{analyze_scss_control_flow_values, build_scss_control_flow_graph};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalNativeCssFunctionSurfaceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub function_count: usize,
    pub parameter_count: usize,
    pub typed_parameter_count: usize,
    pub supported_parameter_syntax_count: usize,
    pub unsupported_parameter_syntax_count: usize,
    pub result_count: usize,
    pub functions: Vec<OmenaScssEvalNativeCssFunctionV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalNativeCssFunctionV0 {
    pub name: String,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub name_span_start: usize,
    pub name_span_end: usize,
    pub return_syntax_source: Option<String>,
    pub return_syntax: Option<RegisteredPropertySyntaxV0>,
    pub parameter_count: usize,
    pub typed_parameter_count: usize,
    pub result_count: usize,
    pub parameters: Vec<OmenaScssEvalNativeCssFunctionParameterV0>,
    pub results: Vec<OmenaScssEvalNativeCssFunctionResultV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalNativeCssFunctionParameterV0 {
    pub name: String,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub name_span_start: usize,
    pub name_span_end: usize,
    pub syntax_source: Option<String>,
    pub syntax: Option<RegisteredPropertySyntaxV0>,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalNativeCssFunctionResultV0 {
    pub value: String,
    pub source_span_start: usize,
    pub source_span_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalNativeCssIfFunctionDecisionSurfaceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub function_count: usize,
    pub foldable_function_count: usize,
    pub preserved_function_count: usize,
    pub static_supports_branch_count: usize,
    pub runtime_branch_count: usize,
    pub functions: Vec<OmenaScssEvalNativeCssIfFunctionDecisionV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalNativeCssIfFunctionDecisionV0 {
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub branch_count: usize,
    pub decision: &'static str,
    pub reason: &'static str,
    pub selected_branch_index: Option<usize>,
    pub selected_value: Option<String>,
    pub branches: Vec<OmenaScssEvalNativeCssIfFunctionBranchV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalNativeCssIfFunctionBranchV0 {
    pub branch_index: usize,
    pub condition: String,
    pub condition_kind: &'static str,
    pub verdict: &'static str,
    pub value: Option<String>,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub supports_witness: Option<StaticSupportsEvalWitnessV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalNativeCssFunctionCallEvaluationSurfaceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub call_count: usize,
    pub foldable_call_count: usize,
    pub preserved_call_count: usize,
    pub structural_error_count: usize,
    pub runtime_dependent_call_count: usize,
    pub missing_result_count: usize,
    pub calls: Vec<OmenaScssEvalNativeCssFunctionCallEvaluationV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalNativeCssFunctionCallEvaluationV0 {
    pub name: String,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub argument_count: usize,
    pub matched_function_count: usize,
    pub matched_function_source_span_start: Option<usize>,
    pub matched_function_source_span_end: Option<usize>,
    pub decision: &'static str,
    pub reason: &'static str,
    pub evaluated_value: Option<String>,
    pub arguments: Vec<OmenaScssEvalNativeCssFunctionCallArgumentV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalNativeCssFunctionCallArgumentV0 {
    pub argument_index: usize,
    pub value: String,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub static_value: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalNativeCssStaticEditPlanV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub edit_count: usize,
    pub when_rule_edit_count: usize,
    pub if_function_edit_count: usize,
    pub function_call_edit_count: usize,
    pub output_changed: bool,
    pub edited_css: String,
    pub edits: Vec<OmenaScssEvalNativeCssStaticEditV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalNativeCssStaticEditV0 {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
    pub edit_kind: &'static str,
}

pub fn summarize_native_css_function_surface(
    source: &str,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalNativeCssFunctionSurfaceV0> {
    if dialect != StyleDialect::Css {
        return None;
    }
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let functions = collect_native_css_functions(source, tokens);
    let parameter_count = functions
        .iter()
        .map(|function| function.parameter_count)
        .sum();
    let typed_parameter_count = functions
        .iter()
        .map(|function| function.typed_parameter_count)
        .sum();
    let supported_parameter_syntax_count = functions
        .iter()
        .flat_map(|function| function.parameters.iter())
        .filter(|parameter| parameter_syntax_is_supported(parameter.syntax.as_ref()))
        .count();
    let unsupported_parameter_syntax_count = functions
        .iter()
        .flat_map(|function| function.parameters.iter())
        .filter(|parameter| {
            matches!(
                parameter.syntax,
                Some(RegisteredPropertySyntaxV0::Unsupported { .. })
            )
        })
        .count();
    let result_count = functions.iter().map(|function| function.result_count).sum();

    Some(OmenaScssEvalNativeCssFunctionSurfaceV0 {
        schema_version: "0",
        product: "omena-scss-eval.native-css-function-surface",
        mode: "recognitionOnly",
        dialect: "css",
        function_count: functions.len(),
        parameter_count,
        typed_parameter_count,
        supported_parameter_syntax_count,
        unsupported_parameter_syntax_count,
        result_count,
        functions,
    })
}

pub fn summarize_native_css_function_call_evaluations(
    source: &str,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalNativeCssFunctionCallEvaluationSurfaceV0> {
    if dialect != StyleDialect::Css {
        return None;
    }
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let functions = collect_native_css_functions(source, tokens);
    let calls = collect_native_css_function_call_evaluations(source, tokens, &functions);
    let call_count = calls.len();
    let foldable_call_count = calls
        .iter()
        .filter(|call| call.decision == "foldToStaticValue")
        .count();
    let preserved_call_count = calls
        .iter()
        .filter(|call| call.decision == "preserveVerbatim")
        .count();
    let structural_error_count = calls
        .iter()
        .filter(|call| call.decision == "structuralError")
        .count();
    let runtime_dependent_call_count = calls
        .iter()
        .filter(|call| call.reason.contains("runtime") || call.reason.contains("cascade"))
        .count();
    let missing_result_count = calls
        .iter()
        .filter(|call| call.reason == "function has no result declaration")
        .count();

    Some(OmenaScssEvalNativeCssFunctionCallEvaluationSurfaceV0 {
        schema_version: "0",
        product: "omena-scss-eval.native-css-function-call-evaluations",
        mode: "oracleOnlyPruneButKeep",
        dialect: "css",
        call_count,
        foldable_call_count,
        preserved_call_count,
        structural_error_count,
        runtime_dependent_call_count,
        missing_result_count,
        calls,
    })
}

pub fn summarize_native_css_static_edit_plan(
    source: &str,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalNativeCssStaticEditPlanV0> {
    if dialect != StyleDialect::Css {
        return None;
    }

    let if_function_decisions = summarize_native_css_if_function_decisions(source, dialect)?;
    let function_call_evaluations =
        summarize_native_css_function_call_evaluations(source, dialect)?;
    let mut edits = Vec::new();
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let functions = collect_native_css_functions(source, tokens);
    let when_rule_truthiness_by_start = native_css_branch_truthiness_by_start(source, "@when");
    let if_function_truthiness_by_start = native_css_branch_truthiness_by_start(source, "if()");
    edits.extend(native_css_when_rule_static_edits(
        source,
        tokens,
        &when_rule_truthiness_by_start,
    ));
    edits.extend(native_css_if_function_static_edits(
        &if_function_decisions,
        &if_function_truthiness_by_start,
        &functions,
    ));
    edits.extend(native_css_function_call_static_edits(
        &function_call_evaluations,
        &functions,
    ));
    let edits = normalize_native_css_static_edits(source, edits)?;
    let edited_css = apply_native_css_static_edits(source, &edits);
    let edit_count = edits.len();
    let when_rule_edit_count = edits
        .iter()
        .filter(|edit| edit.edit_kind == "whenRuleBranchFold")
        .count();
    let if_function_edit_count = edits
        .iter()
        .filter(|edit| edit.edit_kind == "ifFunctionValueFold")
        .count();
    let function_call_edit_count = edits
        .iter()
        .filter(|edit| edit.edit_kind == "functionCallValueFold")
        .count();
    let output_changed = edited_css != source;

    Some(OmenaScssEvalNativeCssStaticEditPlanV0 {
        schema_version: "0",
        product: "omena-scss-eval.native-css-static-edit-plan",
        mode: "staticSubsetPruneButKeep",
        dialect: "css",
        edit_count,
        when_rule_edit_count,
        if_function_edit_count,
        function_call_edit_count,
        output_changed,
        edited_css,
        edits,
    })
}

pub fn summarize_native_css_static_edit_plan_from_transform_ir(
    ir: &TransformIrV0,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalNativeCssStaticEditPlanV0> {
    if dialect != StyleDialect::Css {
        return None;
    }

    let source = ir.source_text();
    let functions = collect_native_css_functions_from_transform_ir(source, dialect, ir);
    let if_function_decisions = native_css_if_function_decision_surface(
        collect_native_css_if_function_decisions_from_transform_ir(source, dialect, ir),
    );
    let function_call_evaluations = native_css_function_call_evaluation_surface(
        collect_native_css_function_call_evaluations_from_transform_ir(
            source, dialect, ir, &functions,
        ),
    );
    let when_rule_truthiness_by_start = native_css_branch_truthiness_by_start(source, "@when");
    let if_function_truthiness_by_start = native_css_branch_truthiness_by_start(source, "if()");
    let mut edits = native_css_when_rule_static_edits_from_transform_ir(
        source,
        dialect,
        ir,
        &when_rule_truthiness_by_start,
    );
    edits.extend(native_css_if_function_static_edits(
        &if_function_decisions,
        &if_function_truthiness_by_start,
        &functions,
    ));
    edits.extend(native_css_function_call_static_edits(
        &function_call_evaluations,
        &functions,
    ));
    native_css_static_edit_plan_from_edits(source, edits)
}

fn native_css_if_function_decision_surface(
    functions: Vec<OmenaScssEvalNativeCssIfFunctionDecisionV0>,
) -> OmenaScssEvalNativeCssIfFunctionDecisionSurfaceV0 {
    let function_count = functions.len();
    let foldable_function_count = functions
        .iter()
        .filter(|function| function.decision == "foldToStaticValue")
        .count();
    let preserved_function_count = function_count.saturating_sub(foldable_function_count);
    let static_supports_branch_count = functions
        .iter()
        .flat_map(|function| function.branches.iter())
        .filter(|branch| branch.condition_kind == "supports")
        .count();
    let runtime_branch_count = functions
        .iter()
        .flat_map(|function| function.branches.iter())
        .filter(|branch| matches!(branch.condition_kind, "media" | "style"))
        .count();

    OmenaScssEvalNativeCssIfFunctionDecisionSurfaceV0 {
        schema_version: "0",
        product: "omena-scss-eval.native-css-if-function-decisions",
        mode: "oracleOnlyPruneButKeep",
        dialect: "css",
        function_count,
        foldable_function_count,
        preserved_function_count,
        static_supports_branch_count,
        runtime_branch_count,
        functions,
    }
}

fn native_css_function_call_evaluation_surface(
    calls: Vec<OmenaScssEvalNativeCssFunctionCallEvaluationV0>,
) -> OmenaScssEvalNativeCssFunctionCallEvaluationSurfaceV0 {
    let call_count = calls.len();
    let foldable_call_count = calls
        .iter()
        .filter(|call| call.decision == "foldToStaticValue")
        .count();
    let preserved_call_count = calls
        .iter()
        .filter(|call| call.decision == "preserveVerbatim")
        .count();
    let structural_error_count = calls
        .iter()
        .filter(|call| call.decision == "structuralError")
        .count();
    let runtime_dependent_call_count = calls
        .iter()
        .filter(|call| call.reason.contains("runtime") || call.reason.contains("cascade"))
        .count();
    let missing_result_count = calls
        .iter()
        .filter(|call| call.reason == "function has no result declaration")
        .count();

    OmenaScssEvalNativeCssFunctionCallEvaluationSurfaceV0 {
        schema_version: "0",
        product: "omena-scss-eval.native-css-function-call-evaluations",
        mode: "oracleOnlyPruneButKeep",
        dialect: "css",
        call_count,
        foldable_call_count,
        preserved_call_count,
        structural_error_count,
        runtime_dependent_call_count,
        missing_result_count,
        calls,
    }
}

fn native_css_static_edit_plan_from_edits(
    source: &str,
    edits: Vec<OmenaScssEvalNativeCssStaticEditV0>,
) -> Option<OmenaScssEvalNativeCssStaticEditPlanV0> {
    let edits = normalize_native_css_static_edits(source, edits)?;
    let edited_css = apply_native_css_static_edits(source, &edits);
    let edit_count = edits.len();
    let when_rule_edit_count = edits
        .iter()
        .filter(|edit| edit.edit_kind == "whenRuleBranchFold")
        .count();
    let if_function_edit_count = edits
        .iter()
        .filter(|edit| edit.edit_kind == "ifFunctionValueFold")
        .count();
    let function_call_edit_count = edits
        .iter()
        .filter(|edit| edit.edit_kind == "functionCallValueFold")
        .count();
    let output_changed = edited_css != source;

    Some(OmenaScssEvalNativeCssStaticEditPlanV0 {
        schema_version: "0",
        product: "omena-scss-eval.native-css-static-edit-plan",
        mode: "staticSubsetPruneButKeep",
        dialect: "css",
        edit_count,
        when_rule_edit_count,
        if_function_edit_count,
        function_call_edit_count,
        output_changed,
        edited_css,
        edits,
    })
}

pub fn summarize_native_css_if_function_decisions(
    source: &str,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalNativeCssIfFunctionDecisionSurfaceV0> {
    if dialect != StyleDialect::Css {
        return None;
    }
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    Some(native_css_if_function_decision_surface(
        collect_native_css_if_function_decisions(source, tokens),
    ))
}

fn collect_native_css_functions(
    source: &str,
    tokens: &[LexedToken],
) -> Vec<OmenaScssEvalNativeCssFunctionV0> {
    tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            (token.kind == SyntaxKind::AtKeyword && token.text.eq_ignore_ascii_case("@function"))
                .then(|| collect_native_css_function(source, tokens, index))
                .flatten()
        })
        .collect()
}

fn collect_native_css_if_function_decisions(
    source: &str,
    tokens: &[LexedToken],
) -> Vec<OmenaScssEvalNativeCssIfFunctionDecisionV0> {
    tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            (token.kind == SyntaxKind::Ident && token.text.eq_ignore_ascii_case("if"))
                .then(|| collect_native_css_if_function_decision(source, tokens, index))
                .flatten()
        })
        .collect()
}

fn collect_native_css_functions_from_transform_ir(
    source: &str,
    dialect: StyleDialect,
    ir: &TransformIrV0,
) -> Vec<OmenaScssEvalNativeCssFunctionV0> {
    let mut functions_by_key = BTreeMap::new();
    for node in active_transform_ir_nodes_by_kind(ir, &[IrNodeKindV0::AtRule]) {
        let Some(tokens) = lex_native_css_transform_ir_node(source, dialect, node) else {
            continue;
        };
        let Some(at_keyword_index) = next_non_trivia_token_index(&tokens, 0) else {
            continue;
        };
        let Some(at_keyword) = tokens.get(at_keyword_index) else {
            continue;
        };
        if at_keyword.kind != SyntaxKind::AtKeyword
            || !at_keyword.text.eq_ignore_ascii_case("@function")
        {
            continue;
        }
        let Some(function) = collect_native_css_function(source, &tokens, at_keyword_index) else {
            continue;
        };
        functions_by_key
            .entry((
                function.source_span_start,
                function.source_span_end,
                function.name.clone(),
            ))
            .or_insert(function);
    }
    functions_by_key.into_values().collect()
}

fn collect_native_css_if_function_decisions_from_transform_ir(
    source: &str,
    dialect: StyleDialect,
    ir: &TransformIrV0,
) -> Vec<OmenaScssEvalNativeCssIfFunctionDecisionV0> {
    let mut functions_by_span = BTreeMap::new();
    for node in active_transform_ir_nodes_by_kind(
        ir,
        &[
            IrNodeKindV0::Value,
            IrNodeKindV0::Declaration,
            IrNodeKindV0::AtRule,
        ],
    ) {
        let Some(tokens) = lex_native_css_transform_ir_node(source, dialect, node) else {
            continue;
        };
        for (index, token) in tokens.iter().enumerate() {
            if token.kind != SyntaxKind::Ident || !token.text.eq_ignore_ascii_case("if") {
                continue;
            }
            let Some(function) = collect_native_css_if_function_decision(source, &tokens, index)
            else {
                continue;
            };
            functions_by_span
                .entry((function.source_span_start, function.source_span_end))
                .or_insert(function);
        }
    }
    functions_by_span.into_values().collect()
}

fn collect_native_css_function_call_evaluations_from_transform_ir(
    source: &str,
    dialect: StyleDialect,
    ir: &TransformIrV0,
    functions: &[OmenaScssEvalNativeCssFunctionV0],
) -> Vec<OmenaScssEvalNativeCssFunctionCallEvaluationV0> {
    let mut calls_by_key = BTreeMap::new();
    for node in active_transform_ir_nodes_by_kind(
        ir,
        &[
            IrNodeKindV0::Value,
            IrNodeKindV0::Declaration,
            IrNodeKindV0::AtRule,
        ],
    ) {
        if !ir_node_source(source, node).is_some_and(|node_source| node_source.contains("--")) {
            continue;
        }
        let Some(tokens) = lex_native_css_transform_ir_node(source, dialect, node) else {
            continue;
        };
        for (index, token) in tokens.iter().enumerate() {
            if !matches!(
                token.kind,
                SyntaxKind::Ident | SyntaxKind::CustomPropertyName
            ) || !token.text.starts_with("--")
                || native_css_function_name_is_declaration(&tokens, index)
            {
                continue;
            }
            let Some(call) =
                collect_native_css_function_call_evaluation(source, &tokens, functions, index)
            else {
                continue;
            };
            calls_by_key
                .entry((
                    call.source_span_start,
                    call.source_span_end,
                    call.name.clone(),
                ))
                .or_insert(call);
        }
    }
    calls_by_key.into_values().collect()
}

fn native_css_when_rule_static_edits_from_transform_ir(
    source: &str,
    dialect: StyleDialect,
    ir: &TransformIrV0,
    truthiness_by_start: &BTreeMap<usize, bool>,
) -> Vec<OmenaScssEvalNativeCssStaticEditV0> {
    let mut edits_by_key = BTreeMap::new();
    for node in active_transform_ir_nodes_by_kind(ir, &[IrNodeKindV0::AtRule]) {
        let span_end = native_css_when_rule_span_end(source, dialect, ir, node)
            .unwrap_or(node.source_span_end);
        let Some(tokens) =
            lex_native_css_transform_ir_span(source, dialect, node.source_span_start, span_end)
        else {
            continue;
        };
        let Some(when_index) = next_non_trivia_token_index(&tokens, 0) else {
            continue;
        };
        let Some(when_token) = tokens.get(when_index) else {
            continue;
        };
        if when_token.kind != SyntaxKind::AtKeyword
            || !when_token.text.eq_ignore_ascii_case("@when")
        {
            continue;
        }
        let Some((edit, _)) = collect_native_css_when_rule_static_edit(
            source,
            &tokens,
            when_index,
            truthiness_by_start,
        ) else {
            continue;
        };
        edits_by_key
            .entry((edit.start, edit.end, edit.edit_kind))
            .or_insert(edit);
    }
    edits_by_key.into_values().collect()
}

fn native_css_when_rule_span_end(
    source: &str,
    dialect: StyleDialect,
    ir: &TransformIrV0,
    node: &IrNodeV0,
) -> Option<usize> {
    let tokens = lex_native_css_transform_ir_node(source, dialect, node)?;
    let when_index = next_non_trivia_token_index(&tokens, 0)?;
    let when_token = tokens.get(when_index)?;
    if when_token.kind != SyntaxKind::AtKeyword || !when_token.text.eq_ignore_ascii_case("@when") {
        return None;
    }
    let else_node = immediate_native_css_else_at_rule_node(source, dialect, ir, node)?;
    Some(else_node.source_span_end)
}

fn immediate_native_css_else_at_rule_node<'a>(
    source: &str,
    dialect: StyleDialect,
    ir: &'a TransformIrV0,
    node: &IrNodeV0,
) -> Option<&'a IrNodeV0> {
    let next = ir
        .nodes
        .iter()
        .filter(|candidate| {
            !candidate.deleted && candidate.source_span_start >= node.source_span_end
        })
        .min_by_key(|candidate| candidate.source_span_start)?;
    if next.kind != IrNodeKindV0::AtRule
        || !source_span_is_trivia_only(
            source,
            dialect,
            node.source_span_end,
            next.source_span_start,
        )
    {
        return None;
    }
    let tokens = lex_native_css_transform_ir_node(source, dialect, next)?;
    let at_keyword_index = next_non_trivia_token_index(&tokens, 0)?;
    let at_keyword = tokens.get(at_keyword_index)?;
    (at_keyword.kind == SyntaxKind::AtKeyword && at_keyword.text.eq_ignore_ascii_case("@else"))
        .then_some(next)
}

fn active_transform_ir_nodes_by_kind<'a>(
    ir: &'a TransformIrV0,
    kinds: &[IrNodeKindV0],
) -> Vec<&'a IrNodeV0> {
    let mut nodes = ir
        .nodes
        .iter()
        .filter(|node| !node.deleted && kinds.contains(&node.kind))
        .collect::<Vec<_>>();
    nodes.sort_by_key(|node| (node.source_span_start, node.source_span_end, node.node_id));
    nodes
}

fn ir_node_source<'source>(source: &'source str, node: &IrNodeV0) -> Option<&'source str> {
    source.get(node.source_span_start..node.source_span_end)
}

fn lex_native_css_transform_ir_node(
    source: &str,
    dialect: StyleDialect,
    node: &IrNodeV0,
) -> Option<Vec<LexedToken>> {
    lex_native_css_transform_ir_span(
        source,
        dialect,
        node.source_span_start,
        node.source_span_end,
    )
}

fn lex_native_css_transform_ir_span(
    source: &str,
    dialect: StyleDialect,
    source_span_start: usize,
    source_span_end: usize,
) -> Option<Vec<LexedToken>> {
    if source_span_start > source_span_end
        || source_span_end > source.len()
        || !source.is_char_boundary(source_span_start)
        || !source.is_char_boundary(source_span_end)
    {
        return None;
    }
    let source_slice = source.get(source_span_start..source_span_end)?;
    lex(source_slice, dialect)
        .tokens()
        .iter()
        .map(|token| shift_lexed_token(token, source_span_start))
        .collect()
}

fn shift_lexed_token(token: &LexedToken, source_span_start: usize) -> Option<LexedToken> {
    let shifted_start = token_start(token).checked_add(source_span_start)?;
    let shifted_end = token_end(token).checked_add(source_span_start)?;
    Some(LexedToken {
        kind: token.kind,
        range: TextRange::new(
            TextSize::from(u32::try_from(shifted_start).ok()?),
            TextSize::from(u32::try_from(shifted_end).ok()?),
        ),
        text: token.text.clone(),
    })
}

fn source_span_is_trivia_only(
    source: &str,
    dialect: StyleDialect,
    source_span_start: usize,
    source_span_end: usize,
) -> bool {
    lex_native_css_transform_ir_span(source, dialect, source_span_start, source_span_end)
        .is_some_and(|tokens| tokens.iter().all(|token| token.kind.is_trivia()))
}

fn native_css_when_rule_static_edits(
    source: &str,
    tokens: &[LexedToken],
    truthiness_by_start: &BTreeMap<usize, bool>,
) -> Vec<OmenaScssEvalNativeCssStaticEditV0> {
    let mut edits = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let Some(token) = tokens.get(index) else {
            break;
        };
        if token.kind == SyntaxKind::AtKeyword
            && token.text.eq_ignore_ascii_case("@when")
            && let Some((edit, next_index)) =
                collect_native_css_when_rule_static_edit(source, tokens, index, truthiness_by_start)
        {
            edits.push(edit);
            index = next_index.saturating_add(1);
            continue;
        }
        index += 1;
    }
    edits
}

fn collect_native_css_when_rule_static_edit(
    source: &str,
    tokens: &[LexedToken],
    when_index: usize,
    truthiness_by_start: &BTreeMap<usize, bool>,
) -> Option<(OmenaScssEvalNativeCssStaticEditV0, usize)> {
    let block_start_index = next_matching_token_index(tokens, when_index + 1, |token| {
        matches!(
            token.kind,
            SyntaxKind::LeftBrace | SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon
        )
    })?;
    if tokens.get(block_start_index)?.kind != SyntaxKind::LeftBrace {
        return None;
    }
    let block_end_index = matching_token_index(
        tokens,
        block_start_index,
        SyntaxKind::LeftBrace,
        SyntaxKind::RightBrace,
    )?;
    let else_rule = collect_immediate_native_css_else_rule(source, tokens, block_end_index + 1);

    let source_span_start = token_start(tokens.get(when_index)?);
    match truthiness_by_start.get(&source_span_start).copied() {
        Some(true) => {
            let source_span_end = else_rule
                .as_ref()
                .map(|else_rule| else_rule.source_span_end)
                .unwrap_or_else(|| token_end(&tokens[block_end_index]));
            let replacement =
                block_inner_source(source, tokens, block_start_index, block_end_index)?;
            Some((
                OmenaScssEvalNativeCssStaticEditV0 {
                    start: source_span_start,
                    end: source_span_end,
                    replacement,
                    edit_kind: "whenRuleBranchFold",
                },
                else_rule
                    .as_ref()
                    .map(|else_rule| else_rule.block_end_index)
                    .unwrap_or(block_end_index),
            ))
        }
        Some(false) => {
            let Some(else_rule) = else_rule else {
                return Some((
                    OmenaScssEvalNativeCssStaticEditV0 {
                        start: source_span_start,
                        end: token_end(tokens.get(block_end_index)?),
                        replacement: String::new(),
                        edit_kind: "whenRuleBranchFold",
                    },
                    block_end_index,
                ));
            };
            if !else_rule.header_text.trim().is_empty() {
                return None;
            }
            let replacement = block_inner_source(
                source,
                tokens,
                else_rule.block_start_index,
                else_rule.block_end_index,
            )?;
            Some((
                OmenaScssEvalNativeCssStaticEditV0 {
                    start: source_span_start,
                    end: else_rule.source_span_end,
                    replacement,
                    edit_kind: "whenRuleBranchFold",
                },
                else_rule.block_end_index,
            ))
        }
        _ => None,
    }
}

fn native_css_branch_truthiness_by_start(
    source: &str,
    at_rule_name: &str,
) -> BTreeMap<usize, bool> {
    let Some(graph) = build_scss_control_flow_graph(source, StyleDialect::Css) else {
        return BTreeMap::new();
    };
    let Some(analysis) = analyze_scss_control_flow_values(source, StyleDialect::Css) else {
        return BTreeMap::new();
    };
    let source_start_by_when_node_key = graph
        .blocks
        .iter()
        .filter(|block| block.block.at_rule_name.eq_ignore_ascii_case(at_rule_name))
        .map(|block| {
            (
                block.node_key.as_str().to_string(),
                block.block.source_span_start,
            )
        })
        .collect::<BTreeMap<_, _>>();

    analysis
        .blocks
        .iter()
        .filter_map(|block| {
            let truthy = match block.transfer_truthiness {
                Some("truthy") => true,
                Some("falsey") => false,
                _ => return None,
            };
            source_start_by_when_node_key
                .get(block.node_key.as_str())
                .copied()
                .map(|source_span_start| (source_span_start, truthy))
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeCssElseRuleV0 {
    header_text: String,
    block_start_index: usize,
    block_end_index: usize,
    source_span_end: usize,
}

fn collect_immediate_native_css_else_rule(
    source: &str,
    tokens: &[LexedToken],
    start_index: usize,
) -> Option<NativeCssElseRuleV0> {
    let else_index = next_non_trivia_token_index(tokens, start_index)?;
    let else_token = tokens.get(else_index)?;
    if else_token.kind != SyntaxKind::AtKeyword || !else_token.text.eq_ignore_ascii_case("@else") {
        return None;
    }
    let block_start_index = next_matching_token_index(tokens, else_index + 1, |token| {
        matches!(
            token.kind,
            SyntaxKind::LeftBrace | SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon
        )
    })?;
    if tokens.get(block_start_index)?.kind != SyntaxKind::LeftBrace {
        return None;
    }
    let block_end_index = matching_token_index(
        tokens,
        block_start_index,
        SyntaxKind::LeftBrace,
        SyntaxKind::RightBrace,
    )?;
    Some(NativeCssElseRuleV0 {
        header_text: trimmed_source_between_tokens(
            source,
            tokens,
            else_index + 1,
            block_start_index,
        )
        .unwrap_or_default(),
        block_start_index,
        block_end_index,
        source_span_end: token_end(tokens.get(block_end_index)?),
    })
}

fn block_inner_source(
    source: &str,
    tokens: &[LexedToken],
    block_start_index: usize,
    block_end_index: usize,
) -> Option<String> {
    let start = token_end(tokens.get(block_start_index)?);
    let end = token_start(tokens.get(block_end_index)?);
    source.get(start..end).map(ToString::to_string)
}

fn native_css_if_function_static_edits(
    surface: &OmenaScssEvalNativeCssIfFunctionDecisionSurfaceV0,
    truthiness_by_start: &BTreeMap<usize, bool>,
    functions: &[OmenaScssEvalNativeCssFunctionV0],
) -> Vec<OmenaScssEvalNativeCssStaticEditV0> {
    surface
        .functions
        .iter()
        .filter(|function| {
            !native_css_if_function_is_inside_function_declaration(function, functions)
        })
        .filter_map(|function| {
            let replacement = function.selected_value.as_ref()?;
            (function.decision == "foldToStaticValue"
                && native_css_if_function_edge_ir_allows_fold(function, truthiness_by_start))
            .then(|| OmenaScssEvalNativeCssStaticEditV0 {
                start: function.source_span_start,
                end: function.source_span_end,
                replacement: replacement.clone(),
                edit_kind: "ifFunctionValueFold",
            })
        })
        .collect()
}

fn native_css_if_function_is_inside_function_declaration(
    function: &OmenaScssEvalNativeCssIfFunctionDecisionV0,
    functions: &[OmenaScssEvalNativeCssFunctionV0],
) -> bool {
    functions.iter().any(|declaration| {
        declaration.source_span_start <= function.source_span_start
            && function.source_span_end <= declaration.source_span_end
    })
}

fn native_css_if_function_edge_ir_allows_fold(
    function: &OmenaScssEvalNativeCssIfFunctionDecisionV0,
    truthiness_by_start: &BTreeMap<usize, bool>,
) -> bool {
    let Some(edge_ir_truthy) = truthiness_by_start
        .get(&function.source_span_start)
        .copied()
    else {
        return false;
    };
    match function.selected_branch_index {
        Some(0) => edge_ir_truthy,
        Some(selected_branch_index) => {
            !edge_ir_truthy
                && function
                    .branches
                    .get(selected_branch_index)
                    .is_some_and(|branch| branch.verdict == "else")
                && function
                    .branches
                    .iter()
                    .take(selected_branch_index)
                    .all(|branch| branch.verdict == "alwaysFalse")
        }
        None => false,
    }
}

fn native_css_function_call_static_edits(
    surface: &OmenaScssEvalNativeCssFunctionCallEvaluationSurfaceV0,
    functions: &[OmenaScssEvalNativeCssFunctionV0],
) -> Vec<OmenaScssEvalNativeCssStaticEditV0> {
    surface
        .calls
        .iter()
        .filter(|call| !native_css_function_call_is_inside_function_declaration(call, functions))
        .filter_map(|call| {
            let replacement = call.evaluated_value.as_ref()?;
            (call.decision == "foldToStaticValue").then(|| OmenaScssEvalNativeCssStaticEditV0 {
                start: call.source_span_start,
                end: call.source_span_end,
                replacement: replacement.clone(),
                edit_kind: "functionCallValueFold",
            })
        })
        .collect()
}

fn native_css_function_call_is_inside_function_declaration(
    call: &OmenaScssEvalNativeCssFunctionCallEvaluationV0,
    functions: &[OmenaScssEvalNativeCssFunctionV0],
) -> bool {
    functions.iter().any(|function| {
        function.source_span_start <= call.source_span_start
            && call.source_span_end <= function.source_span_end
    })
}

fn normalize_native_css_static_edits(
    source: &str,
    mut edits: Vec<OmenaScssEvalNativeCssStaticEditV0>,
) -> Option<Vec<OmenaScssEvalNativeCssStaticEditV0>> {
    edits.sort_by_key(|edit| edit.start);
    edits.dedup_by(|left, right| {
        left.start == right.start
            && left.end == right.end
            && left.replacement == right.replacement
            && left.edit_kind == right.edit_kind
    });
    let mut normalized: Vec<OmenaScssEvalNativeCssStaticEditV0> = Vec::new();
    let mut index = 0usize;
    while index < edits.len() {
        let mut edit = edits.get(index)?.clone();
        if edit.start > edit.end || edit.end > source.len() {
            return None;
        }
        let mut contained = Vec::new();
        let mut next_index = index + 1;
        while let Some(next_edit) = edits.get(next_index)
            && next_edit.start < edit.end
        {
            if next_edit.start > next_edit.end || next_edit.end > source.len() {
                return None;
            }
            if next_edit.end <= edit.end {
                contained.push(next_edit.clone());
                next_index += 1;
                continue;
            }
            return None;
        }
        compose_contained_native_css_static_edits(source, &mut edit, &contained)?;
        normalized.push(edit);
        index = next_index;
    }
    Some(normalized)
}

fn compose_contained_native_css_static_edits(
    source: &str,
    outer: &mut OmenaScssEvalNativeCssStaticEditV0,
    contained: &[OmenaScssEvalNativeCssStaticEditV0],
) -> Option<()> {
    if outer.edit_kind != "whenRuleBranchFold" || contained.is_empty() {
        return Some(());
    }
    let Some((replacement_source_start, replacement_source_end)) =
        native_css_static_edit_replacement_source_span(source, outer)
    else {
        return Some(());
    };
    for inner in contained.iter().rev().filter(|inner| {
        replacement_source_start <= inner.start && inner.end <= replacement_source_end
    }) {
        let relative_start = inner.start - replacement_source_start;
        let relative_end = inner.end - replacement_source_start;
        if !outer.replacement.is_char_boundary(relative_start)
            || !outer.replacement.is_char_boundary(relative_end)
        {
            return None;
        }
        outer
            .replacement
            .replace_range(relative_start..relative_end, inner.replacement.as_str());
    }
    Some(())
}

fn native_css_static_edit_replacement_source_span(
    source: &str,
    edit: &OmenaScssEvalNativeCssStaticEditV0,
) -> Option<(usize, usize)> {
    if edit.replacement.is_empty() {
        return None;
    }
    let edited_source = source.get(edit.start..edit.end)?;
    let mut matches = edited_source.match_indices(edit.replacement.as_str());
    let (relative_start, _) = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    let start = edit.start + relative_start;
    Some((start, start + edit.replacement.len()))
}

fn apply_native_css_static_edits(
    source: &str,
    edits: &[OmenaScssEvalNativeCssStaticEditV0],
) -> String {
    let mut output = source.to_string();
    for edit in edits.iter().rev() {
        output.replace_range(edit.start..edit.end, edit.replacement.as_str());
    }
    output
}

fn collect_native_css_function_call_evaluations(
    source: &str,
    tokens: &[LexedToken],
    functions: &[OmenaScssEvalNativeCssFunctionV0],
) -> Vec<OmenaScssEvalNativeCssFunctionCallEvaluationV0> {
    tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            (matches!(
                token.kind,
                SyntaxKind::Ident | SyntaxKind::CustomPropertyName
            ) && token.text.starts_with("--")
                && !native_css_function_name_is_declaration(tokens, index))
            .then(|| collect_native_css_function_call_evaluation(source, tokens, functions, index))
            .flatten()
        })
        .collect()
}

fn collect_native_css_function_call_evaluation(
    source: &str,
    tokens: &[LexedToken],
    functions: &[OmenaScssEvalNativeCssFunctionV0],
    name_index: usize,
) -> Option<OmenaScssEvalNativeCssFunctionCallEvaluationV0> {
    let name = tokens.get(name_index)?;
    let left_paren_index = next_non_trivia_token_index(tokens, name_index + 1)?;
    if tokens.get(left_paren_index)?.kind != SyntaxKind::LeftParen {
        return None;
    }
    let right_paren_index = matching_token_index(
        tokens,
        left_paren_index,
        SyntaxKind::LeftParen,
        SyntaxKind::RightParen,
    )?;
    let arguments = collect_native_css_function_call_arguments(
        source,
        tokens,
        left_paren_index + 1,
        right_paren_index,
    );
    let matches = functions
        .iter()
        .filter(|function| function.name == name.text)
        .collect::<Vec<_>>();
    let matched_function_count = matches.len();
    let (
        decision,
        reason,
        evaluated_value,
        matched_function_source_span_start,
        matched_function_source_span_end,
    ) = decide_native_css_function_call(&arguments, &matches, functions);

    Some(OmenaScssEvalNativeCssFunctionCallEvaluationV0 {
        name: name.text.clone(),
        source_span_start: token_start(name),
        source_span_end: token_end(tokens.get(right_paren_index)?),
        argument_count: arguments.len(),
        matched_function_count,
        matched_function_source_span_start,
        matched_function_source_span_end,
        decision,
        reason,
        evaluated_value,
        arguments,
    })
}

fn collect_native_css_function_call_arguments(
    source: &str,
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
) -> Vec<OmenaScssEvalNativeCssFunctionCallArgumentV0> {
    split_top_level_ranges(tokens, start_index, end_index, SyntaxKind::Comma)
        .into_iter()
        .enumerate()
        .filter_map(|(argument_index, (start, end))| {
            let value = trimmed_source_between_tokens(source, tokens, start, end)?;
            let source_span_start = first_non_trivia_token_index_until(tokens, start, end)
                .and_then(|index| tokens.get(index))
                .map(token_start)?;
            let source_span_end = previous_non_trivia_token_index_until(tokens, start, end)
                .and_then(|index| tokens.get(index))
                .map(token_end)
                .unwrap_or(source_span_start);
            Some(OmenaScssEvalNativeCssFunctionCallArgumentV0 {
                argument_index,
                static_value: native_css_if_value_is_fully_static(&value),
                value,
                source_span_start,
                source_span_end,
            })
        })
        .collect()
}

fn decide_native_css_function_call(
    arguments: &[OmenaScssEvalNativeCssFunctionCallArgumentV0],
    matches: &[&OmenaScssEvalNativeCssFunctionV0],
    functions: &[OmenaScssEvalNativeCssFunctionV0],
) -> (
    &'static str,
    &'static str,
    Option<String>,
    Option<usize>,
    Option<usize>,
) {
    let Some(function) = matches.first().copied() else {
        return (
            "preserveVerbatim",
            "function resolution is unavailable",
            None,
            None,
            None,
        );
    };
    if matches.len() != 1 {
        return (
            "preserveVerbatim",
            "function resolution is ambiguous",
            None,
            None,
            None,
        );
    }
    if arguments.len() > function.parameters.len() {
        return (
            "structuralError",
            "call has more arguments than declared parameters",
            None,
            Some(function.source_span_start),
            Some(function.source_span_end),
        );
    }
    if native_css_function_required_argument_is_missing(arguments, function) {
        return (
            "structuralError",
            "required argument is missing",
            None,
            Some(function.source_span_start),
            Some(function.source_span_end),
        );
    }
    if function.results.is_empty() {
        return (
            "structuralError",
            "function has no result declaration",
            None,
            Some(function.source_span_start),
            Some(function.source_span_end),
        );
    }
    if function.results.len() != 1 {
        return (
            "preserveVerbatim",
            "function has multiple result declarations",
            None,
            Some(function.source_span_start),
            Some(function.source_span_end),
        );
    }
    if native_css_function_has_guaranteed_result_cycle(function, functions) {
        return (
            "structuralError",
            "function result call graph contains a guaranteed cycle",
            None,
            Some(function.source_span_start),
            Some(function.source_span_end),
        );
    }

    let Some(bindings) = bind_native_css_function_arguments(arguments, function) else {
        return (
            "preserveVerbatim",
            "argument or default value depends on runtime or cascade state",
            None,
            Some(function.source_span_start),
            Some(function.source_span_end),
        );
    };
    match native_css_function_parameter_syntax_gate(&bindings, function) {
        NativeCssFunctionSyntaxGateV0::Accepts => {}
        NativeCssFunctionSyntaxGateV0::Rejects { reason } => {
            return (
                "structuralError",
                reason,
                None,
                Some(function.source_span_start),
                Some(function.source_span_end),
            );
        }
        NativeCssFunctionSyntaxGateV0::Unknown { reason } => {
            return (
                "preserveVerbatim",
                reason,
                None,
                Some(function.source_span_start),
                Some(function.source_span_end),
            );
        }
    }
    let result = &function.results[0].value;
    let Some(evaluated_value) = evaluate_native_css_function_result_value(result, &bindings) else {
        return (
            "preserveVerbatim",
            "result value depends on runtime or cascade state",
            None,
            Some(function.source_span_start),
            Some(function.source_span_end),
        );
    };
    match native_css_function_return_syntax_gate(evaluated_value.as_str(), function) {
        NativeCssFunctionSyntaxGateV0::Accepts => {}
        NativeCssFunctionSyntaxGateV0::Rejects { reason } => {
            return (
                "structuralError",
                reason,
                None,
                Some(function.source_span_start),
                Some(function.source_span_end),
            );
        }
        NativeCssFunctionSyntaxGateV0::Unknown { reason } => {
            return (
                "preserveVerbatim",
                reason,
                None,
                Some(function.source_span_start),
                Some(function.source_span_end),
            );
        }
    }

    (
        "foldToStaticValue",
        "unique function call resolved to a static result value",
        Some(evaluated_value),
        Some(function.source_span_start),
        Some(function.source_span_end),
    )
}

fn native_css_function_has_guaranteed_result_cycle(
    function: &OmenaScssEvalNativeCssFunctionV0,
    functions: &[OmenaScssEvalNativeCssFunctionV0],
) -> bool {
    native_css_result_reaches_function(
        function.name.as_str(),
        function.name.as_str(),
        functions,
        &mut Vec::new(),
    )
}

fn native_css_result_reaches_function(
    target_name: &str,
    current_name: &str,
    functions: &[OmenaScssEvalNativeCssFunctionV0],
    visited: &mut Vec<String>,
) -> bool {
    if visited.iter().any(|name| name == current_name) {
        return false;
    }
    visited.push(current_name.to_string());
    let Some(function) = native_css_unique_function_by_name(functions, current_name) else {
        return false;
    };
    for result in &function.results {
        let Some(next_name) = native_css_exact_function_call_name(result.value.as_str()) else {
            continue;
        };
        if next_name == target_name {
            return true;
        }
        if native_css_result_reaches_function(target_name, next_name, functions, visited) {
            return true;
        }
    }
    false
}

fn native_css_unique_function_by_name<'a>(
    functions: &'a [OmenaScssEvalNativeCssFunctionV0],
    name: &str,
) -> Option<&'a OmenaScssEvalNativeCssFunctionV0> {
    let mut matches = functions.iter().filter(|function| function.name == name);
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

fn native_css_exact_function_call_name(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    let open_index = trimmed.find('(')?;
    let name = trimmed[..open_index].trim();
    if !name.starts_with("--") {
        return None;
    }
    let call_tail = &trimmed[open_index..];
    let close_index = matching_closing_paren_byte_index(call_tail)?;
    call_tail[close_index + 1..]
        .trim()
        .is_empty()
        .then_some(name)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeCssFunctionSyntaxGateV0 {
    Accepts,
    Rejects { reason: &'static str },
    Unknown { reason: &'static str },
}

fn native_css_function_parameter_syntax_gate(
    bindings: &[(String, String)],
    function: &OmenaScssEvalNativeCssFunctionV0,
) -> NativeCssFunctionSyntaxGateV0 {
    for parameter in &function.parameters {
        let Some(syntax) = parameter.syntax_source.as_deref() else {
            continue;
        };
        let Some((_, value)) = bindings.iter().find(|(name, _)| name == &parameter.name) else {
            continue;
        };
        match registered_syntax_match(syntax, value) {
            RegisteredSyntaxMatchV0::Accepts => {}
            RegisteredSyntaxMatchV0::Rejects => {
                return NativeCssFunctionSyntaxGateV0::Rejects {
                    reason: "argument does not match parameter syntax",
                };
            }
            RegisteredSyntaxMatchV0::Unknown => {
                return NativeCssFunctionSyntaxGateV0::Unknown {
                    reason: "argument syntax match is unknown",
                };
            }
        }
    }
    NativeCssFunctionSyntaxGateV0::Accepts
}

fn native_css_function_return_syntax_gate(
    evaluated_value: &str,
    function: &OmenaScssEvalNativeCssFunctionV0,
) -> NativeCssFunctionSyntaxGateV0 {
    let Some(syntax) = function.return_syntax_source.as_deref() else {
        return NativeCssFunctionSyntaxGateV0::Accepts;
    };
    match registered_syntax_match(syntax, evaluated_value) {
        RegisteredSyntaxMatchV0::Accepts => NativeCssFunctionSyntaxGateV0::Accepts,
        RegisteredSyntaxMatchV0::Rejects => NativeCssFunctionSyntaxGateV0::Rejects {
            reason: "result value does not match return syntax",
        },
        RegisteredSyntaxMatchV0::Unknown => NativeCssFunctionSyntaxGateV0::Unknown {
            reason: "result return syntax match is unknown",
        },
    }
}

fn bind_native_css_function_arguments(
    arguments: &[OmenaScssEvalNativeCssFunctionCallArgumentV0],
    function: &OmenaScssEvalNativeCssFunctionV0,
) -> Option<Vec<(String, String)>> {
    function
        .parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| {
            if let Some(argument) = arguments.get(index) {
                return argument
                    .static_value
                    .then(|| (parameter.name.clone(), argument.value.clone()));
            }
            let default_value = parameter.default_value.as_deref()?;
            native_css_if_value_is_fully_static(default_value)
                .then(|| (parameter.name.clone(), default_value.to_string()))
        })
        .collect()
}

fn native_css_function_required_argument_is_missing(
    arguments: &[OmenaScssEvalNativeCssFunctionCallArgumentV0],
    function: &OmenaScssEvalNativeCssFunctionV0,
) -> bool {
    function
        .parameters
        .iter()
        .enumerate()
        .any(|(index, parameter)| index >= arguments.len() && parameter.default_value.is_none())
}

fn evaluate_native_css_function_result_value(
    result: &str,
    bindings: &[(String, String)],
) -> Option<String> {
    if let Some(parameter_name) = extract_exact_var_reference(result) {
        return bindings
            .iter()
            .find_map(|(name, value)| (name == parameter_name).then(|| value.clone()));
    }
    if let Some(value) = evaluate_native_css_function_result_if_value(result) {
        return Some(value);
    }
    if native_css_if_value_is_fully_static(result) {
        return Some(result.trim().to_string());
    }
    None
}

fn evaluate_native_css_function_result_if_value(result: &str) -> Option<String> {
    let trimmed = result.trim();
    if !trimmed
        .get(..2)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("if"))
    {
        return None;
    }
    let source = format!(".native-result {{ value: {trimmed}; }}");
    let value_start = source.find(trimmed)?;
    let decisions = summarize_native_css_if_function_decisions(&source, StyleDialect::Css)?;
    let decision = decisions.functions.first()?;
    if decisions.functions.len() != 1
        || decision.decision != "foldToStaticValue"
        || decision.source_span_start != value_start
        || decision.source_span_end != value_start + trimmed.len()
    {
        return None;
    }
    let truthiness_by_start = native_css_branch_truthiness_by_start(&source, "if()");
    native_css_if_function_edge_ir_allows_fold(decision, &truthiness_by_start)
        .then(|| decision.selected_value.clone())
        .flatten()
}

fn collect_native_css_if_function_decision(
    source: &str,
    tokens: &[LexedToken],
    if_index: usize,
) -> Option<OmenaScssEvalNativeCssIfFunctionDecisionV0> {
    let left_paren_index = next_non_trivia_token_index(tokens, if_index + 1)?;
    if tokens.get(left_paren_index)?.kind != SyntaxKind::LeftParen {
        return None;
    }
    let right_paren_index = matching_token_index(
        tokens,
        left_paren_index,
        SyntaxKind::LeftParen,
        SyntaxKind::RightParen,
    )?;
    let branches = collect_native_css_if_function_branches(
        source,
        tokens,
        left_paren_index + 1,
        right_paren_index,
    );
    if branches.is_empty() {
        return None;
    }
    let (decision, reason, selected_branch_index, selected_value) =
        decide_native_css_if_function(&branches);

    Some(OmenaScssEvalNativeCssIfFunctionDecisionV0 {
        source_span_start: token_start(tokens.get(if_index)?),
        source_span_end: token_end(tokens.get(right_paren_index)?),
        branch_count: branches.len(),
        decision,
        reason,
        selected_branch_index,
        selected_value,
        branches,
    })
}

fn collect_native_css_if_function_branches(
    source: &str,
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
) -> Vec<OmenaScssEvalNativeCssIfFunctionBranchV0> {
    split_top_level_ranges(tokens, start_index, end_index, SyntaxKind::Semicolon)
        .into_iter()
        .enumerate()
        .filter_map(|(branch_index, (start, end))| {
            collect_native_css_if_function_branch(source, tokens, branch_index, start, end)
        })
        .collect()
}

fn collect_native_css_if_function_branch(
    source: &str,
    tokens: &[LexedToken],
    branch_index: usize,
    start_index: usize,
    end_index: usize,
) -> Option<OmenaScssEvalNativeCssIfFunctionBranchV0> {
    let condition_end = first_top_level_token_index(tokens, start_index, end_index, |token| {
        token.kind == SyntaxKind::Colon
    })?;
    let condition = trimmed_source_between_tokens(source, tokens, start_index, condition_end)?;
    let value = trimmed_source_between_tokens(source, tokens, condition_end + 1, end_index);
    let (condition_kind, verdict, supports_witness) =
        classify_native_css_if_function_condition(&condition);
    let source_span_start = first_non_trivia_token_index_until(tokens, start_index, end_index)
        .and_then(|index| tokens.get(index))
        .map(token_start)?;
    let source_span_end = previous_non_trivia_token_index_until(tokens, start_index, end_index)
        .and_then(|index| tokens.get(index))
        .map(token_end)
        .unwrap_or(source_span_start);

    Some(OmenaScssEvalNativeCssIfFunctionBranchV0 {
        branch_index,
        condition,
        condition_kind,
        verdict,
        value,
        source_span_start,
        source_span_end,
        supports_witness,
    })
}

fn classify_native_css_if_function_condition(
    condition: &str,
) -> (
    &'static str,
    &'static str,
    Option<StaticSupportsEvalWitnessV0>,
) {
    if condition.trim().eq_ignore_ascii_case("else") {
        return ("else", "else", None);
    }
    if let Some(inner) = extract_named_function_inner(condition, "supports") {
        let normalized_condition = normalize_supports_condition_for_if(inner);
        let witness = evaluate_static_supports_condition(
            &normalized_condition,
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        let verdict = static_supports_verdict_label(witness.verdict);
        return ("supports", verdict, Some(witness));
    }
    if extract_named_function_inner(condition, "media").is_some() {
        return ("media", "runtime", None);
    }
    if extract_named_function_inner(condition, "style").is_some() {
        return ("style", "runtime", None);
    }

    ("unknown", "unknown", None)
}

fn decide_native_css_if_function(
    branches: &[OmenaScssEvalNativeCssIfFunctionBranchV0],
) -> (&'static str, &'static str, Option<usize>, Option<String>) {
    for branch in branches {
        match branch.verdict {
            "alwaysFalse" => continue,
            "alwaysTrue" | "else" => {
                let Some(value) = branch.value.as_deref() else {
                    return (
                        "preserveVerbatim",
                        "selected branch has an empty token stream",
                        None,
                        None,
                    );
                };
                if native_css_if_value_is_fully_static(value) {
                    return (
                        "foldToStaticValue",
                        "all earlier branches are statically false and the selected value is static",
                        Some(branch.branch_index),
                        Some(value.to_string()),
                    );
                }
                return (
                    "preserveVerbatim",
                    "selected branch value depends on runtime or cascade state",
                    None,
                    None,
                );
            }
            "runtime" => {
                return (
                    "preserveVerbatim",
                    "encountered runtime condition before a static winner",
                    None,
                    None,
                );
            }
            _ => {
                return (
                    "preserveVerbatim",
                    "encountered unknown condition before a static winner",
                    None,
                    None,
                );
            }
        }
    }

    (
        "preserveVerbatim",
        "no statically selected branch",
        None,
        None,
    )
}

fn collect_native_css_function(
    source: &str,
    tokens: &[LexedToken],
    at_keyword_index: usize,
) -> Option<OmenaScssEvalNativeCssFunctionV0> {
    let name_index = next_non_trivia_token_index(tokens, at_keyword_index + 1)?;
    let name = tokens.get(name_index)?;
    if !name.text.starts_with("--") {
        return None;
    }
    let left_paren_index = next_non_trivia_token_index(tokens, name_index + 1)?;
    if tokens.get(left_paren_index)?.kind != SyntaxKind::LeftParen {
        return None;
    }
    let right_paren_index = matching_token_index(
        tokens,
        left_paren_index,
        SyntaxKind::LeftParen,
        SyntaxKind::RightParen,
    )?;
    let block_start_index = next_matching_token_index(tokens, right_paren_index + 1, |token| {
        matches!(
            token.kind,
            SyntaxKind::LeftBrace | SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon
        )
    })?;
    if tokens.get(block_start_index)?.kind != SyntaxKind::LeftBrace {
        return None;
    }
    let block_end_index = matching_token_index(
        tokens,
        block_start_index,
        SyntaxKind::LeftBrace,
        SyntaxKind::RightBrace,
    )
    .unwrap_or(block_start_index);
    let parameters = collect_native_css_function_parameters(
        source,
        tokens,
        left_paren_index + 1,
        right_paren_index,
    );
    let (return_syntax_source, return_syntax) = collect_native_css_function_return_syntax(
        source,
        tokens,
        right_paren_index,
        block_start_index,
    );
    let results =
        collect_native_css_function_results(source, tokens, block_start_index, block_end_index);
    let source_span_start = token_start(tokens.get(at_keyword_index)?);
    let source_span_end = token_end(tokens.get(block_end_index)?);
    let typed_parameter_count = parameters
        .iter()
        .filter(|parameter| parameter.syntax.is_some())
        .count();

    Some(OmenaScssEvalNativeCssFunctionV0 {
        name: name.text.clone(),
        source_span_start,
        source_span_end,
        name_span_start: token_start(name),
        name_span_end: token_end(name),
        return_syntax_source,
        return_syntax,
        parameter_count: parameters.len(),
        typed_parameter_count,
        result_count: results.len(),
        parameters,
        results,
    })
}

fn collect_native_css_function_parameters(
    source: &str,
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
) -> Vec<OmenaScssEvalNativeCssFunctionParameterV0> {
    split_top_level_ranges(tokens, start_index, end_index, SyntaxKind::Comma)
        .into_iter()
        .filter_map(|(start, end)| {
            collect_native_css_function_parameter(source, tokens, start, end)
        })
        .collect()
}

fn collect_native_css_function_parameter(
    source: &str,
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
) -> Option<OmenaScssEvalNativeCssFunctionParameterV0> {
    let name_index = next_non_trivia_token_index_until(tokens, start_index, end_index)?;
    let name = tokens.get(name_index)?;
    if !name.text.starts_with("--") {
        return None;
    }
    let separator_index = first_top_level_token_index(tokens, name_index + 1, end_index, |token| {
        matches!(token.kind, SyntaxKind::Colon)
    });
    let syntax_end_index = separator_index.unwrap_or(end_index);
    let syntax_source =
        trimmed_source_between_tokens(source, tokens, name_index + 1, syntax_end_index);
    let syntax = syntax_source
        .as_deref()
        .map(parse_registered_property_syntax_v0);
    let default_value = separator_index
        .and_then(|index| trimmed_source_between_tokens(source, tokens, index + 1, end_index));
    let source_span_start = token_start(name);
    let source_span_end = previous_non_trivia_token_index_until(tokens, start_index, end_index)
        .and_then(|index| tokens.get(index))
        .map(token_end)
        .unwrap_or_else(|| token_end(name));

    Some(OmenaScssEvalNativeCssFunctionParameterV0 {
        name: name.text.clone(),
        source_span_start,
        source_span_end,
        name_span_start: token_start(name),
        name_span_end: token_end(name),
        syntax_source,
        syntax,
        default_value,
    })
}

fn collect_native_css_function_return_syntax(
    source: &str,
    tokens: &[LexedToken],
    right_paren_index: usize,
    block_start_index: usize,
) -> (Option<String>, Option<RegisteredPropertySyntaxV0>) {
    let Some(returns_index) =
        first_top_level_token_index(tokens, right_paren_index + 1, block_start_index, |token| {
            token.kind == SyntaxKind::Ident && token.text.eq_ignore_ascii_case("returns")
        })
    else {
        return (None, None);
    };
    let source =
        trimmed_source_between_tokens(source, tokens, returns_index + 1, block_start_index);
    let syntax = source.as_deref().map(parse_registered_property_syntax_v0);
    (source, syntax)
}

fn collect_native_css_function_results(
    source: &str,
    tokens: &[LexedToken],
    block_start_index: usize,
    block_end_index: usize,
) -> Vec<OmenaScssEvalNativeCssFunctionResultV0> {
    let mut results = Vec::new();
    let mut index = block_start_index + 1;
    while index < block_end_index {
        let Some(property_index) =
            next_non_trivia_token_index_until(tokens, index, block_end_index)
        else {
            break;
        };
        let property = &tokens[property_index];
        if property.kind != SyntaxKind::Ident || !property.text.eq_ignore_ascii_case("result") {
            index = property_index.saturating_add(1);
            continue;
        }
        let Some(colon_index) =
            next_non_trivia_token_index_until(tokens, property_index + 1, block_end_index)
        else {
            index = property_index.saturating_add(1);
            continue;
        };
        if tokens[colon_index].kind != SyntaxKind::Colon {
            index = property_index.saturating_add(1);
            continue;
        }
        let value_end =
            first_top_level_token_index(tokens, colon_index + 1, block_end_index, |token| {
                matches!(
                    token.kind,
                    SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon
                )
            })
            .unwrap_or(block_end_index);
        if let Some(value) =
            trimmed_source_between_tokens(source, tokens, colon_index + 1, value_end)
        {
            let value_span_start =
                first_non_trivia_token_index_until(tokens, colon_index + 1, value_end)
                    .and_then(|index| tokens.get(index))
                    .map(token_start)
                    .unwrap_or_else(|| token_end(&tokens[colon_index]));
            let value_span_end =
                previous_non_trivia_token_index_until(tokens, colon_index + 1, value_end)
                    .and_then(|index| tokens.get(index))
                    .map(token_end)
                    .unwrap_or(value_span_start);
            results.push(OmenaScssEvalNativeCssFunctionResultV0 {
                value,
                source_span_start: value_span_start,
                source_span_end: value_span_end,
            });
        }
        index = value_end.saturating_add(1);
    }
    results
}

fn split_top_level_ranges(
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
    separator: SyntaxKind,
) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut range_start = start_index;
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;

    for index in start_index..end_index {
        let Some(token) = tokens.get(index) else {
            break;
        };
        match token.kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBrace => brace_depth += 1,
            SyntaxKind::RightBrace => brace_depth = brace_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            kind if kind == separator
                && paren_depth == 0
                && brace_depth == 0
                && bracket_depth == 0 =>
            {
                ranges.push((range_start, index));
                range_start = index + 1;
            }
            _ => {}
        }
    }
    if range_start < end_index {
        ranges.push((range_start, end_index));
    }
    ranges
}

fn first_top_level_token_index(
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
    predicate: impl Fn(&LexedToken) -> bool,
) -> Option<usize> {
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;

    for index in start_index..end_index {
        let token = tokens.get(index)?;
        match token.kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBrace => brace_depth += 1,
            SyntaxKind::RightBrace => brace_depth = brace_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        if paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 && predicate(token) {
            return Some(index);
        }
    }
    None
}

fn matching_token_index(
    tokens: &[LexedToken],
    open_index: usize,
    open_kind: SyntaxKind,
    close_kind: SyntaxKind,
) -> Option<usize> {
    let mut depth = 0usize;
    for index in open_index..tokens.len() {
        match tokens.get(index)?.kind {
            kind if kind == open_kind => depth += 1,
            kind if kind == close_kind => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn next_matching_token_index(
    tokens: &[LexedToken],
    start_index: usize,
    predicate: impl Fn(&LexedToken) -> bool,
) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .skip(start_index)
        .find_map(|(index, token)| predicate(token).then_some(index))
}

fn next_non_trivia_token_index(tokens: &[LexedToken], start_index: usize) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .skip(start_index)
        .find_map(|(index, token)| (!token.kind.is_trivia()).then_some(index))
}

fn next_non_trivia_token_index_until(
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .take(end_index)
        .skip(start_index)
        .find_map(|(index, token)| (!token.kind.is_trivia()).then_some(index))
}

fn first_non_trivia_token_index_until(
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
) -> Option<usize> {
    next_non_trivia_token_index_until(tokens, start_index, end_index)
}

fn previous_non_trivia_token_index_until(
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .take(end_index)
        .skip(start_index)
        .rev()
        .find_map(|(index, token)| (!token.kind.is_trivia()).then_some(index))
}

fn previous_non_trivia_token_index(tokens: &[LexedToken], before_index: usize) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .take(before_index)
        .rev()
        .find_map(|(index, token)| (!token.kind.is_trivia()).then_some(index))
}

fn trimmed_source_between_tokens(
    source: &str,
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
) -> Option<String> {
    let start = first_non_trivia_token_index_until(tokens, start_index, end_index)
        .and_then(|index| tokens.get(index))
        .map(token_start)?;
    let end = previous_non_trivia_token_index_until(tokens, start_index, end_index)
        .and_then(|index| tokens.get(index))
        .map(token_end)?;
    source
        .get(start..end)
        .map(str::trim)
        .and_then(|value| (!value.is_empty()).then(|| value.to_string()))
}

fn token_start(token: &LexedToken) -> usize {
    u32::from(token.range.start()) as usize
}

fn token_end(token: &LexedToken) -> usize {
    u32::from(token.range.end()) as usize
}

fn parameter_syntax_is_supported(syntax: Option<&RegisteredPropertySyntaxV0>) -> bool {
    matches!(
        syntax,
        Some(RegisteredPropertySyntaxV0::Universal | RegisteredPropertySyntaxV0::Supported { .. })
    )
}

fn extract_named_function_inner<'a>(condition: &'a str, name: &str) -> Option<&'a str> {
    let trimmed = condition.trim();
    let prefix = trimmed.get(..name.len())?;
    if !prefix.eq_ignore_ascii_case(name) {
        return None;
    }
    let rest = trimmed[name.len()..].trim_start();
    if !rest.starts_with('(') {
        return None;
    }
    let close_index = matching_closing_paren_byte_index(rest)?;
    rest[close_index + 1..]
        .trim()
        .is_empty()
        .then_some(&rest[1..close_index])
}

fn extract_exact_var_reference(value: &str) -> Option<&str> {
    let inner = extract_named_function_inner(value, "var")?;
    let name = inner.trim();
    (name.starts_with("--") && !name.contains(',')).then_some(name)
}

fn native_css_function_name_is_declaration(tokens: &[LexedToken], name_index: usize) -> bool {
    previous_non_trivia_token_index(tokens, name_index)
        .and_then(|index| tokens.get(index))
        .is_some_and(|token| {
            token.kind == SyntaxKind::AtKeyword && token.text.eq_ignore_ascii_case("@function")
        })
}

fn matching_closing_paren_byte_index(value: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (index, ch) in value.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn normalize_supports_condition_for_if(condition: &str) -> String {
    let trimmed = condition.trim();
    if trimmed.starts_with('(') {
        trimmed.to_string()
    } else {
        format!("({trimmed})")
    }
}

fn static_supports_verdict_label(verdict: StaticSupportsEvalVerdictV0) -> &'static str {
    match verdict {
        StaticSupportsEvalVerdictV0::AlwaysTrue => "alwaysTrue",
        StaticSupportsEvalVerdictV0::AlwaysFalse => "alwaysFalse",
        StaticSupportsEvalVerdictV0::Unknown => "unknown",
    }
}

fn native_css_if_value_is_fully_static(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    ![
        "var(",
        "attr(",
        "env(",
        "style(",
        "media(",
        "supports(",
        "if(",
        "--",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

#[cfg(test)]
mod tests {
    use omena_abstract_value::RegisteredPropertySyntaxV0;
    use omena_parser::StyleDialect;
    use omena_transform_cst::lower_transform_ir_from_source;

    use super::{
        native_css_branch_truthiness_by_start, summarize_native_css_function_call_evaluations,
        summarize_native_css_function_surface, summarize_native_css_if_function_decisions,
        summarize_native_css_static_edit_plan,
        summarize_native_css_static_edit_plan_from_transform_ir,
    };

    #[test]
    fn native_css_function_surface_reuses_registered_property_syntax() {
        let source = "@function --gap(--size <length>: 1rem, --tone <color>) returns <length> { result: var(--size); }";
        let report = summarize_native_css_function_surface(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.mode, "recognitionOnly");
        assert_eq!(report.function_count, 1);
        assert_eq!(report.parameter_count, 2);
        assert_eq!(report.typed_parameter_count, 2);
        assert_eq!(report.supported_parameter_syntax_count, 2);
        assert_eq!(report.result_count, 1);
        assert_eq!(report.functions[0].name, "--gap");
        assert_eq!(
            report.functions[0].return_syntax,
            Some(RegisteredPropertySyntaxV0::Supported {
                alternatives: vec![
                    omena_abstract_value::RegisteredPropertySyntaxAlternativeV0::Sequence {
                        components: vec![
                            omena_abstract_value::RegisteredPropertySyntaxComponentV0 {
                                base: omena_abstract_value::RegisteredPropertySyntaxBaseV0::Length,
                                multiplier:
                                    omena_abstract_value::RegisteredPropertySyntaxMultiplierV0::One,
                            },
                        ],
                    },
                ]
            })
        );
        assert_eq!(report.functions[0].parameters[0].name, "--size");
        assert_eq!(
            report.functions[0].parameters[0].syntax_source.as_deref(),
            Some("<length>")
        );
        assert_eq!(
            report.functions[0].parameters[0].default_value.as_deref(),
            Some("1rem")
        );
        assert_eq!(report.functions[0].results[0].value, "var(--size)");
    }

    #[test]
    fn native_css_function_surface_stays_css_dialect_only() {
        let source = "@function gap($size) { @return $size; }";

        assert!(summarize_native_css_function_surface(source, StyleDialect::Scss).is_none());
    }

    #[test]
    fn native_css_function_call_evaluation_folds_static_result_binding() {
        let source = "@function --gap(--size <length>: 1rem) returns <length> { result: var(--size); } .card { gap: --gap(2rem); }";
        let report = summarize_native_css_function_call_evaluations(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.mode, "oracleOnlyPruneButKeep");
        assert_eq!(report.call_count, 1);
        assert_eq!(report.foldable_call_count, 1);
        assert_eq!(report.preserved_call_count, 0);
        assert_eq!(report.structural_error_count, 0);
        assert_eq!(report.calls[0].name, "--gap");
        assert_eq!(report.calls[0].argument_count, 1);
        assert_eq!(report.calls[0].matched_function_count, 1);
        assert_eq!(report.calls[0].decision, "foldToStaticValue");
        assert_eq!(report.calls[0].evaluated_value.as_deref(), Some("2rem"));
        assert_eq!(report.calls[0].arguments[0].value, "2rem");
        assert!(report.calls[0].arguments[0].static_value);
    }

    #[test]
    fn native_css_function_call_evaluation_folds_static_if_result_through_edge_ir() {
        let source = "@function --gap() returns <length> { result: if(supports(display: grid): 2rem; else: 1rem); } .card { gap: --gap(); }";
        let report = summarize_native_css_function_call_evaluations(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.call_count, 1);
        assert_eq!(report.foldable_call_count, 1);
        assert_eq!(report.preserved_call_count, 0);
        assert_eq!(report.calls[0].decision, "foldToStaticValue");
        assert_eq!(report.calls[0].evaluated_value.as_deref(), Some("2rem"));
    }

    #[test]
    fn native_css_function_call_evaluation_preserves_runtime_argument() {
        let source = "@function --gap(--size <length>: 1rem) returns <length> { result: var(--size); } .card { gap: --gap(var(--space)); }";
        let report = summarize_native_css_function_call_evaluations(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.call_count, 1);
        assert_eq!(report.foldable_call_count, 0);
        assert_eq!(report.preserved_call_count, 1);
        assert_eq!(report.runtime_dependent_call_count, 1);
        assert_eq!(report.calls[0].decision, "preserveVerbatim");
        assert_eq!(
            report.calls[0].reason,
            "argument or default value depends on runtime or cascade state"
        );
        assert!(!report.calls[0].arguments[0].static_value);
    }

    #[test]
    fn native_css_function_call_evaluation_rejects_parameter_syntax_mismatch() {
        let source = "@function --gap(--size <length>) returns <length> { result: var(--size); } .card { gap: --gap(red); }";
        let report = summarize_native_css_function_call_evaluations(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.call_count, 1);
        assert_eq!(report.foldable_call_count, 0);
        assert_eq!(report.structural_error_count, 1);
        assert_eq!(report.calls[0].decision, "structuralError");
        assert_eq!(
            report.calls[0].reason,
            "argument does not match parameter syntax"
        );
        assert_eq!(report.calls[0].evaluated_value, None);
    }

    #[test]
    fn native_css_function_call_evaluation_rejects_return_syntax_mismatch() {
        let source = "@function --tone(--size <length>) returns <color> { result: var(--size); } .card { color: --tone(2rem); }";
        let report = summarize_native_css_function_call_evaluations(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.call_count, 1);
        assert_eq!(report.foldable_call_count, 0);
        assert_eq!(report.structural_error_count, 1);
        assert_eq!(report.calls[0].decision, "structuralError");
        assert_eq!(
            report.calls[0].reason,
            "result value does not match return syntax"
        );
        assert_eq!(report.calls[0].evaluated_value, None);
    }

    #[test]
    fn native_css_function_call_evaluation_preserves_unknown_return_syntax_match() {
        let source = "@function --tone(--value) returns <color> { result: var(--value); } .card { color: --tone(customvalue); }";
        let report = summarize_native_css_function_call_evaluations(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.call_count, 1);
        assert_eq!(report.foldable_call_count, 0);
        assert_eq!(report.preserved_call_count, 1);
        assert_eq!(report.structural_error_count, 0);
        assert_eq!(report.calls[0].decision, "preserveVerbatim");
        assert_eq!(
            report.calls[0].reason,
            "result return syntax match is unknown"
        );
        assert_eq!(report.calls[0].evaluated_value, None);
    }

    #[test]
    fn native_css_function_call_evaluation_surfaces_direct_result_cycle() {
        let source =
            "@function --loop() returns <length> { result: --loop(); } .card { width: --loop(); }";
        let report = summarize_native_css_function_call_evaluations(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.foldable_call_count, 0);
        assert!(report.structural_error_count >= 1);
        assert!(report.calls.iter().any(|call| {
            call.name == "--loop"
                && call.decision == "structuralError"
                && call.reason == "function result call graph contains a guaranteed cycle"
        }));
    }

    #[test]
    fn native_css_function_call_evaluation_surfaces_mutual_result_cycle() {
        let source = "@function --a() returns <length> { result: --b(); } @function --b() returns <length> { result: --a(); } .card { width: --a(); height: --b(); }";
        let report = summarize_native_css_function_call_evaluations(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.foldable_call_count, 0);
        assert!(report.structural_error_count >= 2);
        assert!(report.calls.iter().any(|call| {
            call.name == "--a"
                && call.decision == "structuralError"
                && call.reason == "function result call graph contains a guaranteed cycle"
        }));
        assert!(report.calls.iter().any(|call| {
            call.name == "--b"
                && call.decision == "structuralError"
                && call.reason == "function result call graph contains a guaranteed cycle"
        }));
    }

    #[test]
    fn native_css_function_call_evaluation_preserves_ambiguous_result_cycle() {
        let source = "@function --a() returns <length> { result: --b(); } @function --b() returns <length> { result: --a(); } @function --b() returns <length> { result: 1px; } .card { width: --a(); }";
        let report = summarize_native_css_function_call_evaluations(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.foldable_call_count, 0);
        assert_eq!(report.structural_error_count, 0);
        assert!(report.calls.iter().any(|call| {
            call.name == "--a"
                && call.decision == "preserveVerbatim"
                && call.reason == "result value depends on runtime or cascade state"
        }));
    }

    #[test]
    fn native_css_function_call_evaluation_surfaces_missing_result() {
        let source = "@function --gap(--size <length>) returns <length> { color: red; } .card { gap: --gap(2rem); }";
        let report = summarize_native_css_function_call_evaluations(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.call_count, 1);
        assert_eq!(report.foldable_call_count, 0);
        assert_eq!(report.structural_error_count, 1);
        assert_eq!(report.missing_result_count, 1);
        assert_eq!(report.calls[0].decision, "structuralError");
        assert_eq!(report.calls[0].reason, "function has no result declaration");
    }

    #[test]
    fn native_css_function_call_evaluation_surfaces_missing_required_argument() {
        let source = "@function --gap(--size <length>) returns <length> { result: var(--size); } .card { gap: --gap(); }";
        let report = summarize_native_css_function_call_evaluations(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.call_count, 1);
        assert_eq!(report.foldable_call_count, 0);
        assert_eq!(report.structural_error_count, 1);
        assert_eq!(report.calls[0].decision, "structuralError");
        assert_eq!(report.calls[0].reason, "required argument is missing");
    }

    #[test]
    fn native_css_function_call_evaluation_stays_css_dialect_only() {
        let source = "@function --gap(--size <length>) returns <length> { result: var(--size); } .card { gap: --gap(2rem); }";

        assert!(
            summarize_native_css_function_call_evaluations(source, StyleDialect::Scss).is_none()
        );
    }

    #[test]
    fn native_css_static_edit_plan_folds_static_if_and_function_call_values() {
        let source = "@function --gap(--size <length>: 1rem) returns <length> { result: var(--size); } .card { gap: --gap(2rem); display: if(supports(display: grid): grid; else: block); }";
        let report = summarize_native_css_static_edit_plan(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.mode, "staticSubsetPruneButKeep");
        assert_eq!(report.edit_count, 2);
        assert_eq!(report.if_function_edit_count, 1);
        assert_eq!(report.function_call_edit_count, 1);
        assert!(report.output_changed);
        assert!(report.edited_css.contains("gap: 2rem"));
        assert!(report.edited_css.contains("display: grid"));
        assert!(!report.edited_css.contains("--gap(2rem)"));
        assert!(!report.edited_css.contains("if(supports"));
        assert_eq!(report.edits[0].edit_kind, "functionCallValueFold");
        assert_eq!(report.edits[1].edit_kind, "ifFunctionValueFold");
    }

    #[test]
    fn native_css_static_edit_plan_from_transform_ir_matches_source_plan() {
        let source = "@function --gap(--size <length>: 1rem) returns <length> { result: var(--size); } .card { gap: --gap(2rem); } @when supports(display: grid) { .grid { display: if(supports(display: grid): grid; else: block); } } @else { .fallback { display: block; } }";
        let ir = lower_transform_ir_from_source(source, StyleDialect::Css, "native-css-static");
        let source_plan = summarize_native_css_static_edit_plan(source, StyleDialect::Css);
        let ir_plan =
            summarize_native_css_static_edit_plan_from_transform_ir(&ir, StyleDialect::Css);

        assert!(source_plan.is_some());
        assert!(ir_plan.is_some());
        let Some(source_plan) = source_plan else {
            return;
        };
        let Some(ir_plan) = ir_plan else {
            return;
        };

        assert_eq!(ir_plan.edits, source_plan.edits);
        assert_eq!(ir_plan.edit_count, source_plan.edit_count);
        assert_eq!(
            ir_plan.when_rule_edit_count,
            source_plan.when_rule_edit_count
        );
        assert_eq!(
            ir_plan.if_function_edit_count,
            source_plan.if_function_edit_count
        );
        assert_eq!(
            ir_plan.function_call_edit_count,
            source_plan.function_call_edit_count
        );
        assert_eq!(ir_plan.output_changed, source_plan.output_changed);
        assert_eq!(ir_plan.edited_css, source_plan.edited_css);
    }

    #[test]
    fn native_css_static_edit_plan_preserves_function_body_calls() {
        let source = "@function --inner() returns <length> { result: 1px; } @function --outer() returns <length> { result: --inner(); } .card { width: --inner(); }";
        let report = summarize_native_css_static_edit_plan(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.edit_count, 1);
        assert_eq!(report.function_call_edit_count, 1);
        assert!(report.output_changed);
        assert!(report.edited_css.contains("result: --inner();"));
        assert!(report.edited_css.contains("width: 1px"));
    }

    #[test]
    fn native_css_static_edit_plan_folds_function_result_if_without_rewriting_function_body() {
        let source = "@function --gap() returns <length> { result: if(supports(display: grid): 2rem; else: 1rem); } .card { gap: --gap(); }";
        let report = summarize_native_css_static_edit_plan(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.edit_count, 1);
        assert_eq!(report.if_function_edit_count, 0);
        assert_eq!(report.function_call_edit_count, 1);
        assert!(report.output_changed);
        assert!(
            report
                .edited_css
                .contains("result: if(supports(display: grid): 2rem; else: 1rem);")
        );
        assert!(report.edited_css.contains("gap: 2rem"));
        assert!(!report.edited_css.contains("--gap();"));
    }

    #[test]
    fn native_css_static_edit_plan_folds_static_when_rule_true_branch() {
        let source = "@when supports(display: grid) { .grid { display: grid; } } @else { .fallback { display: block; } }";
        let report = summarize_native_css_static_edit_plan(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.edit_count, 1);
        assert_eq!(report.when_rule_edit_count, 1);
        assert_eq!(report.if_function_edit_count, 0);
        assert_eq!(report.function_call_edit_count, 0);
        assert!(report.output_changed);
        assert!(report.edited_css.contains(".grid { display: grid; }"));
        assert!(!report.edited_css.contains("@when"));
        assert!(!report.edited_css.contains(".fallback"));
        assert_eq!(report.edits[0].edit_kind, "whenRuleBranchFold");
    }

    #[test]
    fn native_css_static_edit_plan_consumes_edge_ir_when_truthiness() {
        let source = "@when supports(display: grid) { .grid { display: grid; } } @else { .fallback { display: block; } }";
        let truthiness_by_start = native_css_branch_truthiness_by_start(source, "@when");

        assert_eq!(truthiness_by_start.get(&0), Some(&true));

        let report = summarize_native_css_static_edit_plan(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.when_rule_edit_count, 1);
        assert!(report.edited_css.contains(".grid { display: grid; }"));
        assert!(!report.edited_css.contains(".fallback"));
    }

    #[test]
    fn native_css_static_edit_plan_consumes_edge_ir_if_function_truthiness() {
        let source = ".card { display: if(supports(display: grid): grid; else: block); }";
        let truthiness_by_start = native_css_branch_truthiness_by_start(source, "if()");

        assert_eq!(truthiness_by_start.len(), 1);
        assert!(truthiness_by_start.values().all(|truthy| *truthy));

        let report = summarize_native_css_static_edit_plan(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.if_function_edit_count, 1);
        assert!(report.edited_css.contains("display: grid"));
        assert!(!report.edited_css.contains("if(supports"));
    }

    #[test]
    fn native_css_static_edit_plan_composes_nested_static_when_body_edits() {
        let source = "@when supports(display: grid) { .grid { display: if(supports(display: grid): grid; else: block); } } @else { .fallback { display: block; } }";
        let report = summarize_native_css_static_edit_plan(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.edit_count, 1);
        assert_eq!(report.when_rule_edit_count, 1);
        assert!(report.output_changed);
        assert!(report.edited_css.contains(".grid { display: grid; }"));
        assert!(!report.edited_css.contains("@when"));
        assert!(!report.edited_css.contains("if(supports"));
        assert!(!report.edited_css.contains(".fallback"));
    }

    #[test]
    fn native_css_static_edit_plan_folds_static_when_rule_else_branch() {
        let source = "@when supports(display: -ms-grid) { .grid { display: grid; } } @else { .fallback { display: block; } }";
        let report = summarize_native_css_static_edit_plan(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.edit_count, 1);
        assert_eq!(report.when_rule_edit_count, 1);
        assert!(report.edited_css.contains(".fallback { display: block; }"));
        assert!(!report.edited_css.contains("@when"));
        assert!(!report.edited_css.contains(".grid"));
    }

    #[test]
    fn native_css_static_edit_plan_preserves_runtime_when_rule() {
        let source = "@when media(width >= 1px) { .grid { display: grid; } } @else { .fallback { display: block; } }";
        let report = summarize_native_css_static_edit_plan(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.edit_count, 0);
        assert_eq!(report.when_rule_edit_count, 0);
        assert!(!report.output_changed);
        assert_eq!(report.edited_css, source);
    }

    #[test]
    fn native_css_static_edit_plan_preserves_runtime_native_values() {
        let source = "@function --gap(--size <length>: 1rem) returns <length> { result: var(--size); } .card { gap: --gap(var(--space)); margin: if(media(width >= 1px): 1rem; else: 2rem); }";
        let report = summarize_native_css_static_edit_plan(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.edit_count, 0);
        assert_eq!(report.if_function_edit_count, 0);
        assert_eq!(report.function_call_edit_count, 0);
        assert!(!report.output_changed);
        assert_eq!(report.edited_css, source);
    }

    #[test]
    fn native_css_static_edit_plan_stays_css_dialect_only() {
        let source = ".card { display: if(supports(display: grid): grid; else: block); }";

        assert!(summarize_native_css_static_edit_plan(source, StyleDialect::Scss).is_none());
    }

    #[test]
    fn native_css_if_function_decision_folds_static_supports_branch() {
        let source = ".card { display: if(supports(display: grid): grid; else: block); }";
        let report = summarize_native_css_if_function_decisions(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.mode, "oracleOnlyPruneButKeep");
        assert_eq!(report.function_count, 1);
        assert_eq!(report.foldable_function_count, 1);
        assert_eq!(report.preserved_function_count, 0);
        assert_eq!(report.static_supports_branch_count, 1);
        assert_eq!(report.runtime_branch_count, 0);
        assert_eq!(report.functions[0].decision, "foldToStaticValue");
        assert_eq!(report.functions[0].selected_branch_index, Some(0));
        assert_eq!(report.functions[0].selected_value.as_deref(), Some("grid"));
        assert_eq!(report.functions[0].branches[0].condition_kind, "supports");
        assert_eq!(report.functions[0].branches[0].verdict, "alwaysTrue");
        assert!(
            report.functions[0].branches[0]
                .supports_witness
                .as_ref()
                .is_some_and(|witness| witness.product == "omena-cascade.supports-static-eval")
        );
    }

    #[test]
    fn native_css_if_function_decision_preserves_runtime_media_branch() {
        let source = ".card { margin: if(media(width >= 1px): 1rem; else: 2rem); }";
        let report = summarize_native_css_if_function_decisions(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.function_count, 1);
        assert_eq!(report.foldable_function_count, 0);
        assert_eq!(report.preserved_function_count, 1);
        assert_eq!(report.static_supports_branch_count, 0);
        assert_eq!(report.runtime_branch_count, 1);
        assert_eq!(report.functions[0].decision, "preserveVerbatim");
        assert_eq!(
            report.functions[0].reason,
            "encountered runtime condition before a static winner"
        );
        assert_eq!(report.functions[0].selected_branch_index, None);
        assert_eq!(report.functions[0].branches[0].condition_kind, "media");
        assert_eq!(report.functions[0].branches[0].verdict, "runtime");
    }

    #[test]
    fn native_css_if_function_decision_preserves_runtime_value() {
        let source = ".card { color: if(supports(color: red): var(--accent); else: blue); }";
        let report = summarize_native_css_if_function_decisions(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.function_count, 1);
        assert_eq!(report.functions[0].decision, "preserveVerbatim");
        assert_eq!(
            report.functions[0].reason,
            "selected branch value depends on runtime or cascade state"
        );
        assert_eq!(report.functions[0].selected_value, None);
    }

    #[test]
    fn native_css_if_function_decision_stays_css_dialect_only() {
        let source = ".card { width: if(supports(width: 1px): 1px; else: 2px); }";

        assert!(summarize_native_css_if_function_decisions(source, StyleDialect::Scss).is_none());
    }
}

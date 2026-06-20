use std::collections::{BTreeMap, BTreeSet};

use omena_abstract_value::AbstractCssValueV0;
use omena_parser::StyleDialect;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalStaticStylesheetEvaluationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub evaluator: &'static str,
    pub dialect: &'static str,
    pub product_output_source: &'static str,
    pub legacy_output_retained_as_oracle: bool,
    pub legacy_output_consumed_until_cutover: bool,
    pub evaluated_css: String,
    pub native_edit_output: String,
    pub replacement_count: usize,
    pub native_replacement_legacy_reflection_count: usize,
    pub native_replacement_legacy_unreflected_count: usize,
    pub native_edit_count: usize,
    pub native_value_edit_count: usize,
    pub native_structural_edit_count: usize,
    pub native_edit_output_matches_evaluated_css: bool,
    pub resolved_replacements: Vec<OmenaScssEvalResolvedReplacementV0>,
    pub native_edits: Vec<OmenaScssEvalStaticStylesheetNativeEditV0>,
    pub value_resolution: OmenaScssEvalStaticValueResolutionReportV0,
    pub oracle: crate::OmenaScssEvalOracleReportV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalResolvedReplacementV0 {
    pub name: String,
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub rendered_value: Option<String>,
    pub abstract_value: AbstractCssValueV0,
    pub abstract_value_kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalStaticStylesheetNativeEditV0 {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
    pub edit_kind: &'static str,
    pub abstract_value: Option<AbstractCssValueV0>,
    pub abstract_value_kind: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalStaticValueResolutionReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub fuel_limit: usize,
    pub reference_count: usize,
    pub resolved_count: usize,
    pub raw_count: usize,
    pub top_count: usize,
    pub cycle_count: usize,
    pub fuel_exhausted_count: usize,
    pub unresolved_reference_count: usize,
    pub unsupported_dynamic_count: usize,
    pub values: Vec<OmenaScssEvalStaticValueResolutionV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalStaticValueResolutionV0 {
    pub name: String,
    pub start: usize,
    pub end: usize,
    pub source_text: String,
    pub rendered_value: Option<String>,
    pub abstract_value: AbstractCssValueV0,
    pub abstract_value_kind: &'static str,
    pub outcome: &'static str,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StaticStylesheetVariableKind {
    Scss,
    Less,
}

impl StaticStylesheetVariableKind {
    pub(super) fn for_dialect(dialect: StyleDialect) -> Option<Self> {
        match dialect {
            StyleDialect::Scss | StyleDialect::Sass => Some(Self::Scss),
            StyleDialect::Less => Some(Self::Less),
            StyleDialect::Css => None,
        }
    }

    pub(super) fn evaluator_label(self) -> &'static str {
        match self {
            Self::Scss => "omena-query-static-scss-variable-evaluator",
            Self::Less => "omena-query-static-less-variable-evaluator",
        }
    }

    pub(super) fn reference_prefix(self) -> char {
        match self {
            Self::Scss => '$',
            Self::Less => '@',
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct StaticStylesheetVariableDeclaration {
    pub(super) value: String,
    pub(super) span_start: usize,
    pub(super) span_end: usize,
    pub(super) removal_spans: Vec<(usize, usize)>,
    pub(super) is_default: bool,
    pub(super) is_global: bool,
}

#[derive(Debug, Clone)]
pub(super) struct StaticStylesheetScopedVariableDeclaration {
    pub(super) name: String,
    pub(super) scope_id: usize,
    pub(super) declaration: StaticStylesheetVariableDeclaration,
    pub(super) removal_spans: Vec<(usize, usize)>,
}

#[derive(Debug, Clone)]
pub(super) struct StaticStylesheetEvaluationEdit {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) replacement: String,
}

pub(super) struct StaticScssMixinEvaluationEdits {
    pub(super) edits: Vec<StaticStylesheetEvaluationEdit>,
    pub(super) preserved_raw_include_count: usize,
}

pub(super) struct StaticScssFunctionEvaluationEdits {
    pub(super) edits: Vec<StaticStylesheetEvaluationEdit>,
    pub(super) replacements: Vec<OmenaScssEvalResolvedReplacementV0>,
    pub(super) preserved_raw_call_count: usize,
}

pub(super) struct StaticLessMixinEvaluationEdits {
    pub(super) edits: Vec<StaticStylesheetEvaluationEdit>,
    pub(super) preserved_non_rendering_call_count: usize,
}

pub(super) struct StaticLessDetachedRulesetEvaluationEdits {
    pub(super) edits: Vec<StaticStylesheetEvaluationEdit>,
    pub(super) preserved_raw_call_count: usize,
}

pub(super) struct StaticLessDetachedRulesetAccessorEvaluationEdits {
    pub(super) edits: Vec<StaticStylesheetEvaluationEdit>,
    pub(super) preserved_raw_accessor_count: usize,
    pub(super) preserved_declaration_keys: BTreeSet<(usize, String)>,
}

pub(super) struct StaticLessMixinAccessorEvaluationEdits {
    pub(super) edits: Vec<StaticStylesheetEvaluationEdit>,
    pub(super) preserved_raw_accessor_count: usize,
}

pub(super) enum StaticLessBodyPropertyValueOutcome {
    Resolved(String),
    MemberNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaticLessResolvedValue {
    pub(super) text: String,
    pub(super) escaped: bool,
}

#[derive(Debug, Clone)]
pub(super) struct StaticStylesheetPropertyDeclaration {
    pub(super) span_start: usize,
    pub(super) value: String,
}

#[derive(Debug, Clone)]
pub(super) struct StaticScssFunctionDeclaration {
    pub(super) name: String,
    pub(super) parameters: Vec<StaticScssFunctionParameter>,
    pub(super) local_variables: Vec<StaticScssFunctionLocalVariable>,
    pub(super) return_clauses: Vec<StaticScssFunctionReturnClause>,
    pub(super) span_start: usize,
    pub(super) span_end: usize,
    pub(super) body_start: usize,
    pub(super) body_end: usize,
}

#[derive(Debug, Clone)]
pub(super) struct StaticScssMixinDeclaration {
    pub(super) name: String,
    pub(super) parameters: Vec<StaticScssFunctionParameter>,
    pub(super) span_start: usize,
    pub(super) span_end: usize,
    pub(super) body_start: usize,
    pub(super) body_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaticScssFunctionLocalVariable {
    pub(super) name: String,
    pub(super) value: String,
    pub(super) span_start: usize,
    pub(super) scope_start: usize,
    pub(super) scope_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaticScssFunctionReturnClause {
    pub(super) condition: Option<String>,
    pub(super) value: String,
    pub(super) span_start: usize,
    pub(super) loop_headers: Vec<StaticScssLoopHeader>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaticScssLoopHeader {
    pub(super) text: String,
    pub(super) span_start: usize,
    pub(super) body_start: usize,
    pub(super) body_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaticScssFunctionParameter {
    pub(super) name: String,
    pub(super) default_value: Option<String>,
    pub(super) variadic: bool,
    pub(super) pattern_value: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct StaticScssFunctionCall {
    pub(super) name: String,
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) arguments: Vec<StaticScssFunctionArgument>,
}

#[derive(Debug, Clone)]
pub(super) struct StaticScssMixinIncludeCall {
    pub(super) name: String,
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) arguments: Vec<StaticScssFunctionArgument>,
    pub(super) content_body: Option<String>,
    pub(super) content_parameters: Vec<String>,
}

#[derive(Debug, Clone)]
pub(super) struct StaticScssMixinRenderResult {
    pub(super) body: String,
    pub(super) used_mixin_declaration_names: BTreeSet<String>,
    pub(super) used_function_declaration_names: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub(super) struct StaticScssMixinBodyLocalDeclaration {
    pub(super) name: String,
    pub(super) declaration: StaticStylesheetVariableDeclaration,
}

#[derive(Debug, Clone)]
pub(super) struct StaticLessMixinDeclaration {
    pub(super) name: String,
    pub(super) parameters: Vec<StaticScssFunctionParameter>,
    pub(super) guard: Option<String>,
    pub(super) span_start: usize,
    pub(super) span_end: usize,
    pub(super) body_start: usize,
    pub(super) body_end: usize,
}

#[derive(Debug, Clone)]
pub(super) struct StaticLessMixinCall {
    pub(super) namespace: Option<String>,
    pub(super) namespace_arguments: Vec<StaticScssFunctionArgument>,
    pub(super) name: String,
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) important: bool,
    pub(super) arguments: Vec<StaticScssFunctionArgument>,
}

#[derive(Debug, Clone)]
pub(super) struct StaticLessMixinAccessor {
    pub(super) name: String,
    pub(super) member: String,
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) arguments: Vec<StaticScssFunctionArgument>,
}

#[derive(Debug, Clone)]
pub(super) struct StaticLessDetachedRulesetDeclaration {
    pub(super) name: String,
    pub(super) scope_id: usize,
    pub(super) span_start: usize,
    pub(super) span_end: usize,
    pub(super) body_start: usize,
    pub(super) body_end: usize,
}

#[derive(Debug, Clone)]
pub(super) struct StaticLessDetachedRulesetCall {
    pub(super) name: String,
    pub(super) start: usize,
    pub(super) end: usize,
}

pub(super) enum StaticLessDetachedRulesetCallRenderOutcome {
    Rendered(StaticLessMixinRenderResult),
    PreservedRaw,
}

#[derive(Debug, Clone)]
pub(super) struct StaticLessDetachedRulesetAccessor {
    pub(super) name: String,
    pub(super) member: String,
    pub(super) start: usize,
    pub(super) end: usize,
}

pub(super) enum StaticLessDetachedRulesetAccessorRenderOutcome {
    Rendered(String),
    PreservedRaw,
}

#[derive(Debug, Clone)]
pub(super) struct StaticLessMixinBodyLocalDeclaration {
    pub(super) name: String,
    pub(super) declaration: StaticStylesheetVariableDeclaration,
}

#[derive(Debug, Clone)]
pub(super) struct StaticLessMixinRenderResult {
    pub(super) body: String,
    pub(super) used_declaration_names: BTreeSet<String>,
}

pub(super) enum StaticLessMixinCallRenderOutcome {
    Rendered(StaticLessMixinRenderResult),
    KnownNoOutput {
        used_declaration_names: BTreeSet<String>,
    },
    PreservedNoOutput,
}

pub(super) enum StaticLessMixinRenderOutcome {
    Rendered(StaticLessMixinRenderResult),
    GuardNotMatched,
    GuardUnknown,
}

#[derive(Debug, Clone)]
pub(super) struct StaticLessMixinAccessorRenderResult {
    pub(super) value: String,
    pub(super) used_declaration_name: String,
}

pub(super) enum StaticLessMixinAccessorCallRenderOutcome {
    Rendered(StaticLessMixinAccessorRenderResult),
    PreservedRaw,
}

pub(super) enum StaticLessMixinAccessorRenderOutcome {
    Rendered(StaticLessMixinAccessorRenderResult),
    GuardNotMatched,
    GuardUnknown,
    MemberNotFound,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct StaticLessMixinRenderContext<'a> {
    pub(super) source: &'a str,
    pub(super) declarations: &'a [StaticLessMixinDeclaration],
    pub(super) detached_ruleset_declarations: &'a [StaticLessDetachedRulesetDeclaration],
    pub(super) scopes: &'a [StaticStylesheetScope],
    pub(super) variable_declarations:
        &'a BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    pub(super) property_declarations:
        &'a BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    pub(super) captured_values: &'a BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct StaticScssFunctionResolutionContext<'a> {
    pub(super) dialect: StyleDialect,
    pub(super) declarations: &'a [StaticScssFunctionDeclaration],
    pub(super) mixin_declarations: &'a [StaticScssMixinDeclaration],
    pub(super) scopes: &'a [StaticStylesheetScope],
    pub(super) variable_declarations: &'a [StaticStylesheetScopedVariableDeclaration],
    pub(super) active_functions: &'a BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaticScssFunctionArgument {
    pub(super) name: Option<String>,
    pub(super) value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaticScssFunctionLocalScope {
    pub(super) end_index: usize,
    pub(super) span_start: usize,
    pub(super) span_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaticStylesheetScope {
    pub(super) parent_id: Option<usize>,
    pub(super) body_start: usize,
    pub(super) end: usize,
}

use std::collections::{BTreeMap, BTreeSet};

use omena_parser::StyleDialect;

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

#[derive(Debug, Clone)]
pub(super) struct StaticLessDetachedRulesetAccessor {
    pub(super) name: String,
    pub(super) member: String,
    pub(super) start: usize,
    pub(super) end: usize,
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
    PreservedNoOutput,
}

pub(super) enum StaticLessMixinRenderOutcome {
    Rendered(StaticLessMixinRenderResult),
    GuardNotMatched,
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

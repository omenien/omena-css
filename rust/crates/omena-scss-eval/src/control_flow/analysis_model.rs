use std::collections::BTreeMap;

use omena_abstract_value::AbstractCssValueV0;

use super::{
    OmenaScssEvalCallArgumentValueV0, OmenaScssEvalCallLocalBindingV0,
    OmenaScssEvalCallParameterValueV0, OmenaScssEvalCallReturnNodeV0,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ScssCallReturnCandidate {
    pub(super) kind: &'static str,
    pub(super) symbol_kind: &'static str,
    pub(super) role: &'static str,
    pub(super) name: Option<String>,
    pub(super) namespace: Option<String>,
    pub(super) parameter_names: Vec<String>,
    pub(super) parameter_values: Vec<OmenaScssEvalCallParameterValueV0>,
    pub(super) local_binding_values: Vec<OmenaScssEvalCallLocalBindingV0>,
    pub(super) argument_values: Vec<OmenaScssEvalCallArgumentValueV0>,
    pub(super) return_text: Option<String>,
    pub(super) return_value: Option<AbstractCssValueV0>,
    pub(super) body_has_control_flow: bool,
    pub(super) body_has_loop_control_flow: bool,
    pub(super) return_inside_loop_control_flow: bool,
    pub(super) return_loop_header_text: Option<String>,
    pub(super) return_loop_header_texts: Vec<String>,
    pub(super) return_loop_body_texts: Vec<String>,
    pub(super) return_condition_text: Option<String>,
    pub(super) return_negated_condition_texts: Vec<String>,
    pub(super) source_span_start: usize,
    pub(super) source_span_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ScssReturnCondition {
    pub(super) condition_text: Option<String>,
    pub(super) negated_condition_texts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ScssLoopReturnResolution {
    Active(AbstractCssValueV0),
    Inactive,
    Unknown,
}

pub(super) struct ScssCallReturnResolutionContext<'a> {
    pub(super) nodes: &'a [OmenaScssEvalCallReturnNodeV0],
    pub(super) call_graph: &'a BTreeMap<String, Vec<String>>,
    pub(super) active_stack: &'a [String],
    pub(super) global_variable_declarations: &'a [ScssGlobalVariableDeclaration],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ScssCallBoundReturnActivity {
    Active,
    Inactive,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ScssGlobalVariableDeclaration {
    pub(super) name: String,
    pub(super) declaration_start: usize,
}

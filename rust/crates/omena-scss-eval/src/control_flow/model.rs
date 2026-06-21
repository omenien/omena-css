use omena_abstract_value::AbstractCssValueV0;
use omena_transform_cst::StableNodeKeyV0;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalControlFlowIrSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub node_key_type: &'static str,
    pub flat_css_cfg_built: bool,
    pub merged_cross_file_graph: bool,
    pub block_count: usize,
    pub branch_block_count: usize,
    pub loop_block_count: usize,
    pub back_edge_count: usize,
    pub edge_count: usize,
    pub blocks: Vec<OmenaScssEvalControlFlowBlockV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalControlFlowBlockV0 {
    pub node_key: StableNodeKeyV0,
    pub kind: &'static str,
    pub at_rule_name: String,
    pub header_text: String,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub successor_count: usize,
    pub has_back_edge: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalControlFlowValueAnalysisV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub value_type: &'static str,
    pub max_iterations: usize,
    pub converged: bool,
    pub iteration_count: usize,
    pub block_count: usize,
    pub back_edge_count: usize,
    pub loop_carried_binding_count: usize,
    pub widened_to_top_count: usize,
    pub flat_css_cfg_built: bool,
    pub merged_cross_file_graph: bool,
    pub blocks: Vec<OmenaScssEvalControlFlowValueBlockV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalControlFlowWideningWitnessV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub value_type: &'static str,
    pub policy: &'static str,
    pub max_iterations: usize,
    pub node_count: usize,
    pub converged: bool,
    pub iteration_count: usize,
    pub widened_to_top_count: usize,
    pub output_top_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalTypedValueLatticeWitnessV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub value_type: &'static str,
    pub payload_type: &'static str,
    pub policy: &'static str,
    pub sample_value_count: usize,
    pub typed_payload_count: usize,
    pub raw_value_count: usize,
    pub untyped_exact_or_finite_count: usize,
    pub typed_coverage_basis_points: usize,
    pub typed_advisory_comparison_count: usize,
    pub typed_advisory_true_count: usize,
    pub typed_prune_consumer_enabled: bool,
    pub type_kind_counts: Vec<OmenaScssEvalTypedValueKindCountV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalTypedValueKindCountV0 {
    pub kind: &'static str,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalControlFlowValueBlockV0 {
    pub node_key: StableNodeKeyV0,
    pub kind: &'static str,
    pub transfer_kind: &'static str,
    pub transfer_value: Option<AbstractCssValueV0>,
    pub transfer_value_kind: Option<&'static str>,
    pub transfer_truthiness: Option<&'static str>,
    pub predecessor_node_keys: Vec<StableNodeKeyV0>,
    pub loop_carried_bindings: Vec<String>,
    pub loop_carried_binding_values: Vec<OmenaScssEvalControlFlowBindingValueV0>,
    pub input_value: AbstractCssValueV0,
    pub input_value_kind: &'static str,
    pub output_value: AbstractCssValueV0,
    pub output_value_kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalControlFlowBindingValueV0 {
    pub name: String,
    pub value: AbstractCssValueV0,
    pub value_kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalCallReturnIrSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub node_key_type: &'static str,
    pub recursion_cap: usize,
    pub flat_css_cfg_built: bool,
    pub merged_cross_file_graph: bool,
    pub node_count: usize,
    pub declaration_node_count: usize,
    pub call_node_count: usize,
    pub return_node_count: usize,
    pub return_value_count: usize,
    pub exact_return_value_count: usize,
    pub finite_set_return_value_count: usize,
    pub raw_return_value_count: usize,
    pub top_return_value_count: usize,
    pub bottom_return_value_count: usize,
    pub call_resolved_return_value_count: usize,
    pub exact_call_resolved_return_value_count: usize,
    pub finite_set_call_resolved_return_value_count: usize,
    pub raw_call_resolved_return_value_count: usize,
    pub top_call_resolved_return_value_count: usize,
    pub bottom_call_resolved_return_value_count: usize,
    pub call_argument_value_count: usize,
    pub exact_call_argument_value_count: usize,
    pub finite_set_call_argument_value_count: usize,
    pub raw_call_argument_value_count: usize,
    pub top_call_argument_value_count: usize,
    pub bottom_call_argument_value_count: usize,
    pub edge_count: usize,
    pub recursive_edge_count: usize,
    pub capped_recursive_call_count: usize,
    pub max_stack_depth_observed: usize,
    pub nodes: Vec<OmenaScssEvalCallReturnNodeV0>,
    pub edges: Vec<OmenaScssEvalCallReturnEdgeV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalCallReturnNodeV0 {
    pub node_key: StableNodeKeyV0,
    pub kind: &'static str,
    pub symbol_kind: &'static str,
    pub role: &'static str,
    pub name: Option<String>,
    pub namespace: Option<String>,
    pub parameter_names: Vec<String>,
    pub parameter_values: Vec<OmenaScssEvalCallParameterValueV0>,
    pub local_binding_values: Vec<OmenaScssEvalCallLocalBindingV0>,
    pub argument_values: Vec<OmenaScssEvalCallArgumentValueV0>,
    pub return_text: Option<String>,
    pub return_value: Option<AbstractCssValueV0>,
    pub return_value_kind: Option<&'static str>,
    pub call_resolved_return_value: Option<AbstractCssValueV0>,
    pub call_resolved_return_value_kind: Option<&'static str>,
    pub body_has_control_flow: bool,
    pub body_has_loop_control_flow: bool,
    pub return_inside_loop_control_flow: bool,
    pub return_loop_header_text: Option<String>,
    pub return_loop_header_texts: Vec<String>,
    pub return_loop_body_texts: Vec<String>,
    pub return_condition_text: Option<String>,
    pub return_negated_condition_texts: Vec<String>,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub containing_declaration_node_key: Option<StableNodeKeyV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalCallParameterValueV0 {
    pub name: String,
    pub default_value_text: Option<String>,
    pub default_value: Option<AbstractCssValueV0>,
    pub default_value_kind: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalCallArgumentValueV0 {
    pub name: Option<String>,
    pub text: String,
    pub value: AbstractCssValueV0,
    pub value_kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalCallLocalBindingV0 {
    pub name: String,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub scope_span_start: usize,
    pub scope_span_end: usize,
    pub value_text: String,
    pub value: AbstractCssValueV0,
    pub value_kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalCallReturnEdgeV0 {
    pub source_node_key: StableNodeKeyV0,
    pub target_node_key: StableNodeKeyV0,
    pub kind: &'static str,
    pub recursive: bool,
    pub capped_by_recursion_cap: bool,
}

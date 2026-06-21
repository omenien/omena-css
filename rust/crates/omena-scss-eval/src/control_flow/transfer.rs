use omena_abstract_value::{
    AbstractCssValueV0, BoundedJoinFixpointNodeV0, MAX_FLOW_ANALYSIS_ITERATIONS,
    analyze_bounded_join_fixpoint, join_abstract_css_values,
};

use crate::{abstract_css_value_kind, value_eval::static_scss_literal_truthiness};

use super::{ScssControlFlowAnalysisNode, model::OmenaScssEvalControlFlowBindingValueV0};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ScssControlFlowTransfer {
    BranchCondition {
        value: AbstractCssValueV0,
    },
    LoopCondition {
        bindings: Vec<ScssControlFlowBindingValue>,
        value: AbstractCssValueV0,
    },
    LoopCarried {
        bindings: Vec<ScssControlFlowBindingValue>,
        value: AbstractCssValueV0,
    },
    PassThrough,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ScssControlFlowBindingValue {
    pub(super) name: String,
    pub(super) value: AbstractCssValueV0,
}

impl ScssControlFlowTransfer {
    pub(super) const fn kind_label(&self) -> &'static str {
        match self {
            Self::BranchCondition { .. } => "branchCondition",
            Self::LoopCondition { .. } => "loopCondition",
            Self::LoopCarried { .. } => "loopCarriedBindings",
            Self::PassThrough => "passThrough",
        }
    }

    pub(super) fn loop_carried_bindings(&self) -> Vec<String> {
        match self {
            Self::LoopCondition { bindings, .. } | Self::LoopCarried { bindings, .. } => bindings
                .iter()
                .map(|binding| binding.name.clone())
                .collect(),
            Self::BranchCondition { .. } | Self::PassThrough => Vec::new(),
        }
    }

    pub(super) fn loop_carried_binding_values(
        &self,
    ) -> Vec<OmenaScssEvalControlFlowBindingValueV0> {
        match self {
            Self::LoopCondition { bindings, .. } | Self::LoopCarried { bindings, .. } => bindings
                .iter()
                .map(|binding| OmenaScssEvalControlFlowBindingValueV0 {
                    name: binding.name.clone(),
                    value_kind: abstract_css_value_kind(&binding.value),
                    value: binding.value.clone(),
                })
                .collect(),
            Self::BranchCondition { .. } | Self::PassThrough => Vec::new(),
        }
    }

    pub(super) fn transfer_value(&self) -> Option<AbstractCssValueV0> {
        match self {
            Self::BranchCondition { value }
            | Self::LoopCondition { value, .. }
            | Self::LoopCarried { value, .. } => Some(value.clone()),
            Self::PassThrough => None,
        }
    }

    pub(super) fn transfer_truthiness(&self) -> Option<&'static str> {
        match self {
            Self::BranchCondition { value } | Self::LoopCondition { value, .. } => {
                scss_static_truthiness_label(value)
            }
            Self::LoopCarried { .. } | Self::PassThrough => None,
        }
    }

    fn apply(&self, input_value: &AbstractCssValueV0) -> AbstractCssValueV0 {
        match self {
            Self::BranchCondition { value }
            | Self::LoopCondition { value, .. }
            | Self::LoopCarried { value, .. } => join_abstract_css_values(input_value, value),
            Self::PassThrough => input_value.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ScssControlFlowFixpointResult {
    pub(super) converged: bool,
    pub(super) iteration_count: usize,
    pub(super) widened_to_top_count: usize,
    pub(super) input_values: Vec<AbstractCssValueV0>,
    pub(super) output_values: Vec<AbstractCssValueV0>,
}

pub(super) fn run_scss_control_flow_fixpoint(
    nodes: &[ScssControlFlowAnalysisNode],
) -> ScssControlFlowFixpointResult {
    let flow_nodes = nodes
        .iter()
        .map(|node| BoundedJoinFixpointNodeV0 {
            id: node.block.node_key.0.clone(),
            predecessor_ids: node
                .predecessor_indices
                .iter()
                .filter_map(|index| nodes.get(*index).map(|node| node.block.node_key.0.clone()))
                .collect(),
            transfer: node.transfer.clone(),
        })
        .collect::<Vec<_>>();
    let fixpoint = analyze_bounded_join_fixpoint(
        &flow_nodes,
        MAX_FLOW_ANALYSIS_ITERATIONS,
        AbstractCssValueV0::Bottom,
        AbstractCssValueV0::Top,
        join_abstract_css_values,
        |input_value, transfer| transfer.apply(input_value),
    );
    let input_values = fixpoint
        .nodes
        .iter()
        .map(|node| node.input_value.clone())
        .collect::<Vec<_>>();
    let mut output_values = fixpoint
        .nodes
        .iter()
        .map(|node| node.output_value.clone())
        .collect::<Vec<_>>();
    let widened_to_top_count = if fixpoint.converged {
        0
    } else {
        output_values
            .iter_mut()
            .filter(|value| !matches!(value, AbstractCssValueV0::Top))
            .map(|value| {
                *value = AbstractCssValueV0::Top;
            })
            .count()
    };

    ScssControlFlowFixpointResult {
        converged: fixpoint.converged,
        iteration_count: fixpoint.iteration_count,
        widened_to_top_count,
        input_values,
        output_values,
    }
}

pub(super) fn scss_static_truthiness_label(value: &AbstractCssValueV0) -> Option<&'static str> {
    match value {
        AbstractCssValueV0::Exact { value, .. } => scss_static_truthiness_label_from_text(value),
        AbstractCssValueV0::FiniteSet { values, .. } => {
            let mut truthiness = values
                .iter()
                .filter_map(|value| {
                    scss_static_truthiness_label(&AbstractCssValueV0::Exact {
                        typed: None,
                        value: value.clone(),
                    })
                })
                .collect::<Vec<_>>();
            truthiness.sort_unstable();
            truthiness.dedup();
            (truthiness.len() == 1).then_some(truthiness[0])
        }
        AbstractCssValueV0::Raw { value } => scss_static_truthiness_label_from_text(value),
        AbstractCssValueV0::Bottom | AbstractCssValueV0::Top => None,
    }
}

fn scss_static_truthiness_label_from_text(value: &str) -> Option<&'static str> {
    static_scss_literal_truthiness(value).map(|truthy| if truthy { "truthy" } else { "falsey" })
}

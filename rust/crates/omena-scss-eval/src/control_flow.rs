use omena_parser::StyleDialect;

mod analysis_model;
mod arguments;
mod blocks;
mod call_resolution;
mod call_return_nodes;
mod call_return_resolution;
mod cst_tokens;
mod edge_ir;
mod header_values;
mod lexical;
mod loop_values;
mod model;
mod oracle_corpus;
mod prune_reachability;
mod return_candidates;
mod summaries;
mod symbol_candidates;
mod tokens;
mod transfer;
mod value_analysis;
mod variables;

use transfer::ScssControlFlowTransfer;
pub(crate) use value_analysis::analyze_scss_control_flow_values_with_initial_bindings;
pub(super) use value_analysis::summarize_scss_control_flow_widening_witness;
pub use value_analysis::{analyze_scss_control_flow_values, summarize_typed_value_lattice_witness};

pub use edge_ir::build_scss_control_flow_graph;
pub use model::{
    OmenaScssEvalCallArgumentValueV0, OmenaScssEvalCallLocalBindingV0,
    OmenaScssEvalCallParameterValueV0, OmenaScssEvalCallReturnEdgeV0,
    OmenaScssEvalCallReturnIrSummaryV0, OmenaScssEvalCallReturnNodeV0,
    OmenaScssEvalControlFlowBindingValueV0, OmenaScssEvalControlFlowBlockIdV0,
    OmenaScssEvalControlFlowBlockV0, OmenaScssEvalControlFlowEdgeV0,
    OmenaScssEvalControlFlowGraphBlockV0, OmenaScssEvalControlFlowGraphV0,
    OmenaScssEvalControlFlowIrSummaryV0, OmenaScssEvalControlFlowPruneReachabilityV0,
    OmenaScssEvalControlFlowValueAnalysisV0, OmenaScssEvalControlFlowValueBlockV0,
    OmenaScssEvalControlFlowWideningWitnessV0, OmenaScssEvalTypedValueKindCountV0,
    OmenaScssEvalTypedValueLatticeWitnessV0,
};
pub use oracle_corpus::{
    OmenaScssEvalControlFlowOracleCorpusFixtureReportV0,
    OmenaScssEvalControlFlowOracleCorpusReportV0, summarize_scss_control_flow_oracle_corpus,
};
pub use prune_reachability::summarize_scss_control_flow_prune_reachability;
pub(crate) use prune_reachability::summarize_scss_control_flow_prune_reachability_with_initial_bindings;
pub use summaries::{
    summarize_scss_call_return_ir, summarize_scss_control_flow_ir,
    summarize_scss_control_flow_ir_scanner_oracle,
};

const SCSS_CALL_RETURN_RECURSION_LIMIT: usize = 32;

const fn dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScssControlFlowAnalysisNode {
    block: OmenaScssEvalControlFlowBlockV0,
    predecessor_indices: Vec<usize>,
    transfer: ScssControlFlowTransfer,
}

#[cfg(test)]
mod tests;

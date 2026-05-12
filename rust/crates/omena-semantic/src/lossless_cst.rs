use serde::Serialize;

use crate::ParserLosslessCstFactsV0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LosslessCstContractV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub source_byte_len: usize,
    pub token_count: usize,
    pub root_node_count: usize,
    pub diagnostic_count: usize,
    pub span_invariants: LosslessCstSpanInvariantsV0,
    pub consumer_readiness: LosslessCstConsumerReadinessV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LosslessCstSpanInvariantsV0 {
    pub token_spans_within_source: bool,
    pub node_spans_within_source: bool,
    pub byte_span_contract_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LosslessCstConsumerReadinessV0 {
    pub precise_rename_base_ready: bool,
    pub formatter_base_ready: bool,
    pub recovery_diagnostics_observed: bool,
}

pub fn summarize_lossless_cst_contract(facts: &ParserLosslessCstFactsV0) -> LosslessCstContractV0 {
    let byte_span_contract_ready =
        facts.all_token_spans_within_source && facts.all_node_spans_within_source;

    LosslessCstContractV0 {
        schema_version: "0",
        product: "omena-semantic.lossless-cst-contract",
        source_byte_len: facts.source_byte_len,
        token_count: facts.token_count,
        root_node_count: facts.root_node_count,
        diagnostic_count: facts.diagnostic_count,
        span_invariants: LosslessCstSpanInvariantsV0 {
            token_spans_within_source: facts.all_token_spans_within_source,
            node_spans_within_source: facts.all_node_spans_within_source,
            byte_span_contract_ready,
        },
        consumer_readiness: LosslessCstConsumerReadinessV0 {
            precise_rename_base_ready: byte_span_contract_ready,
            formatter_base_ready: byte_span_contract_ready,
            recovery_diagnostics_observed: facts.diagnostic_count > 0,
        },
    }
}

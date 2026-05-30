//! Context-sensitive (k-CFA) dynamic-className M-tier diagnostics for the
//! consumer-facing `omena-query` source path.
//!
//! The cascade/RG-flow query diagnostics already route through the checker
//! orchestrator gate. This module adds the missing M-tier handoff: dynamic
//! className call sites are flowed through the real k-limited call-string
//! analysis (`analyze_k_limited_call_site_flows`) and the joined per-context
//! exit values are fed into the checker M-tier rules. The emitted
//! `no-unknown-dynamic-class` / `no-imprecise-value` / `no-impossible-selector`
//! diagnostics therefore reflect the k-CFA-joined values that the LSP surface
//! consumes, not only the 0/1-CFA result, and the diagnostic set changes when
//! the context-depth bound `k` changes.

use super::*;

use omena_query_checker_orchestrator::{
    AbstractClassValueV0, OmenaQueryCheckerKLimitedFlowContextV0,
    run_omena_query_checker_k_limited_flow_m_tier_gate_v0,
};

/// Deserializable abstract class value used to seed a dynamic-className call-site
/// exit value for context-sensitive M-tier analysis.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum OmenaQueryDynamicClassValueInputV0 {
    Bottom,
    Exact {
        value: String,
    },
    FiniteSet {
        values: Vec<String>,
    },
    Prefix {
        prefix: String,
    },
    Suffix {
        suffix: String,
    },
    PrefixSuffix {
        prefix: String,
        suffix: String,
        #[serde(default)]
        min_length: usize,
    },
    Top,
}

impl OmenaQueryDynamicClassValueInputV0 {
    fn into_abstract_class_value(self) -> AbstractClassValueV0 {
        match self {
            Self::Bottom => AbstractClassValueV0::Bottom,
            Self::Exact { value } => AbstractClassValueV0::Exact { value },
            Self::FiniteSet { values } => AbstractClassValueV0::FiniteSet { values },
            Self::Prefix { prefix } => AbstractClassValueV0::Prefix {
                prefix,
                provenance: None,
            },
            Self::Suffix { suffix } => AbstractClassValueV0::Suffix {
                suffix,
                provenance: None,
            },
            Self::PrefixSuffix {
                prefix,
                suffix,
                min_length,
            } => AbstractClassValueV0::PrefixSuffix {
                prefix,
                suffix,
                min_length,
                provenance: None,
            },
            Self::Top => AbstractClassValueV0::Top,
        }
    }
}

/// A dynamic-className call site observed in the analysed source document.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryDynamicClassnameCallSiteV0 {
    pub callee_key: String,
    pub call_site_stack: Vec<String>,
    pub exit_value: OmenaQueryDynamicClassValueInputV0,
    pub reference_range: ParserRangeV0,
}

/// Input contract for the consumer-facing context-sensitive M-tier diagnostic
/// surface.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryDynamicClassnameMTierInputV0 {
    pub source_uri: String,
    pub selector_universe: Vec<String>,
    pub max_context_depth: usize,
    pub call_sites: Vec<OmenaQueryDynamicClassnameCallSiteV0>,
}

/// Run the context-sensitive M-tier dynamic-className analysis at the supplied
/// call-string bound `k` and lower the result into consumer source diagnostics.
///
/// Each call site contributes a context whose exit value is joined per the
/// k-limited context key inside `analyze_k_limited_call_site_flows`. The joined
/// value drives the checker M-tier rules, so increasing `k` separates call sites
/// that share a callee and removes the over-approximating join — which removes
/// diagnostics that were only present because of the join.
pub fn summarize_omena_query_dynamic_classname_m_tier_diagnostics_with_context_depth(
    input: &OmenaQueryDynamicClassnameMTierInputV0,
) -> OmenaQuerySourceDiagnosticsForFileV0 {
    let mut range_by_context: BTreeMap<(String, Vec<String>), ParserRangeV0> = BTreeMap::new();
    let contexts = input
        .call_sites
        .iter()
        .map(|call_site| {
            range_by_context.insert(
                (
                    call_site.callee_key.clone(),
                    call_site.call_site_stack.clone(),
                ),
                call_site.reference_range,
            );
            OmenaQueryCheckerKLimitedFlowContextV0 {
                callee_key: call_site.callee_key.clone(),
                call_site_stack: call_site.call_site_stack.clone(),
                exit_value: call_site.exit_value.clone().into_abstract_class_value(),
            }
        })
        .collect::<Vec<_>>();

    let gate = run_omena_query_checker_k_limited_flow_m_tier_gate_v0(
        &contexts,
        &input.selector_universe,
        input.max_context_depth,
    );

    let mut diagnostics = Vec::new();
    if gate.enforcement_passed {
        for context in &gate.contexts {
            let range = range_by_context
                .get(&(context.callee_key.clone(), context.call_site_stack.clone()))
                .copied()
                .unwrap_or_default();
            for evaluation in &context.evaluations {
                diagnostics.push(OmenaQuerySourceDiagnosticV0 {
                    code: dynamic_classname_m_tier_diagnostic_code(evaluation.rule_code_name),
                    severity: evaluation.severity_name,
                    provenance: vec![
                        "omena-query-checker-orchestrator.k-limited-flow-m-tier-gate",
                        "omena-abstract-value.k-limited-call-site-flow",
                        "omena-checker.m-tier-rules",
                        "omena-query.dynamic-classname",
                    ],
                    range,
                    message: evaluation.message.clone(),
                    create_selector: None,
                });
            }
        }
    }

    diagnostics.sort_by(|left, right| {
        (
            left.range.start.line,
            left.range.start.character,
            left.code,
            &left.message,
        )
            .cmp(&(
                right.range.start.line,
                right.range.start.character,
                right.code,
                &right.message,
            ))
    });
    apply_omena_query_checker_product_gate_to_source_diagnostics(&mut diagnostics);

    OmenaQuerySourceDiagnosticsForFileV0 {
        schema_version: "0",
        product: "omena-query.diagnostics-for-file",
        file_uri: input.source_uri.clone(),
        file_kind: "source",
        diagnostic_count: diagnostics.len(),
        diagnostics,
        ready_surfaces: vec![
            "dynamicClassnameMTierDiagnostics",
            "kLimitedCallSiteFlow",
            "checkerMTierEvaluation",
            "checkerProductDiagnosticGate",
        ],
    }
}

fn dynamic_classname_m_tier_diagnostic_code(rule_code_name: &str) -> &'static str {
    match rule_code_name {
        "no-unknown-dynamic-class" => "noUnknownDynamicClass",
        "no-imprecise-value" => "noImpreciseValue",
        "no-impossible-selector" => "noImpossibleSelector",
        _ => "dynamicClassDomain",
    }
}

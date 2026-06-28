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
//!
//! The call sites can either be supplied through the explicit input contract
//! (`OmenaQueryDynamicClassnameMTierInputV0`) or harvested directly from a source
//! file's syntax-index template type-fact targets, so the default workspace
//! diagnostic path raises these M-tier diagnostics without an external producer.

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
    let mut diagnostics = collect_omena_query_dynamic_classname_m_tier_diagnostics(
        &input.call_sites,
        &input.selector_universe,
        input.max_context_depth,
    );

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

/// Default call-string bound `k` used by the workspace diagnostic path. The LSP
/// default is context-sensitive (k = 2): dynamic-className call sites that share
/// a callee binding are kept apart so their abstract exit values are not joined
/// into an over-approximating root context. A context-insensitive run (k = 0)
/// would merge them and emit a different diagnostic set.
pub(super) const OMENA_QUERY_WORKSPACE_DYNAMIC_CLASSNAME_CONTEXT_DEPTH: usize = 2;

/// Harvest dynamic-className call-site contexts from a source file's syntax-index
/// type-fact targets (the same template-interpolation projections the engine
/// expression-domain producer consumes) and lower them into context-sensitive
/// M-tier source diagnostics for the default workspace path.
///
/// Each `type_fact_target` is a `prefix${expr}suffix` className projection. The
/// harvested callee key is the projected expression's binding path, so two
/// template call sites that interpolate the same binding share a callee and are
/// joined at `k = 0` but separated at the workspace default `k`. The per-target
/// byte span becomes the diagnostic range and the distinguishing tail of the
/// call-string, so increasing `k` genuinely re-partitions the contexts.
///
/// Soundness of `no-unknown-dynamic-class` requires the selector universe to be
/// scoped to the module the className is actually bound to. A target that carries
/// a resolved `target_style_uri` (e.g. a `cx(`prefix-${x}`)` call bound to a
/// specific imported CSS Module) is evaluated against ONLY that module's selectors
/// (`selector_universe_by_uri`), so a `btn-` prefix is matched against the bound
/// module, not the union of every imported module — which would otherwise let a
/// `btn-*` selector in a different module mask a genuinely-empty intersection. A
/// target with no resolved URI (a bare `className={`btn-${x}`}` literal with no
/// binding context) has no single module to scope to, so it falls back to the
/// union (`union_selector_universe`); `no-unknown-dynamic-class` then fires only
/// when the prefix is provably empty against the whole union, the conservative
/// stopgap that never cross-attributes a match to the wrong module.
///
/// `no-imprecise-value` is suppressed for harvested affix templates: an
/// interpolation is inherently Top/imprecise, so a hint per template is
/// information-free noise. The k-CFA precision is used to NARROW the candidate set
/// (drive `no-unknown-dynamic-class` / `no-impossible-selector`), not to restate
/// that an interpolation is imprecise.
pub(super) fn harvest_omena_query_dynamic_classname_m_tier_diagnostics(
    source_path: &str,
    source: &str,
    type_fact_targets: &[OmenaQuerySourceTypeFactTargetV0],
    union_selector_universe: &[String],
    selector_universe_by_uri: &BTreeMap<String, Vec<String>>,
    max_context_depth: usize,
) -> Vec<OmenaQuerySourceDiagnosticV0> {
    // Partition harvested call sites by the resolved module they are bound to so
    // each scope is evaluated against its CORRECTLY-scoped selector universe. A
    // `None` scope (no resolved binding) is evaluated against the union.
    let mut call_sites_by_scope: BTreeMap<
        Option<String>,
        Vec<OmenaQueryDynamicClassnameCallSiteV0>,
    > = BTreeMap::new();
    for target in type_fact_targets {
        let Some(exit_value) =
            harvested_abstract_class_value(target.prefix.as_str(), target.suffix.as_str())
        else {
            continue;
        };
        let callee_key = harvested_callee_key(target.expression_id.as_str());
        call_sites_by_scope
            .entry(target.target_style_uri.clone())
            .or_default()
            .push(OmenaQueryDynamicClassnameCallSiteV0 {
                callee_key,
                call_site_stack: vec![
                    source_path.to_string(),
                    format!("{}:{}", target.byte_span.start, target.byte_span.end),
                ],
                exit_value,
                reference_range: parser_range_for_byte_span(source, target.byte_span),
            });
    }

    if call_sites_by_scope.is_empty() {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    for (scope_uri, call_sites) in &call_sites_by_scope {
        let scoped_universe = match scope_uri {
            Some(uri) => selector_universe_by_uri
                .get(uri)
                .map(Vec::as_slice)
                .unwrap_or(union_selector_universe),
            None => union_selector_universe,
        };
        diagnostics.extend(collect_omena_query_dynamic_classname_m_tier_diagnostics(
            call_sites,
            scoped_universe,
            max_context_depth,
        ));
    }

    // Suppress the information-free `no-imprecise-value` hint on harvested affix
    // templates: an interpolation being imprecise is expected and non-actionable.
    diagnostics.retain(|diagnostic| diagnostic.code != "noImpreciseValue");
    diagnostics
}

/// Map a harvested template projection (`prefix${expr}suffix`) to the abstract
/// class value the interpolation guarantees. A bare `${expr}` with neither a
/// prefix nor a suffix carries no static structure and is skipped (no M-tier
/// obligation to discharge).
fn harvested_abstract_class_value(
    prefix: &str,
    suffix: &str,
) -> Option<OmenaQueryDynamicClassValueInputV0> {
    match (prefix.is_empty(), suffix.is_empty()) {
        (true, true) => None,
        (false, true) => Some(OmenaQueryDynamicClassValueInputV0::Prefix {
            prefix: prefix.to_string(),
        }),
        (true, false) => Some(OmenaQueryDynamicClassValueInputV0::Suffix {
            suffix: suffix.to_string(),
        }),
        (false, false) => Some(OmenaQueryDynamicClassValueInputV0::PrefixSuffix {
            prefix: prefix.to_string(),
            suffix: suffix.to_string(),
            min_length: prefix.len() + suffix.len(),
        }),
    }
}

/// Recover the projected expression's binding path from a syntax-index
/// type-fact expression id of the form
/// `omena-bridge-source-type-fact:{path}:{start}:{end}`. The path is the callee
/// identity that lets call sites interpolating the same binding share a context.
fn harvested_callee_key(expression_id: &str) -> String {
    let trimmed = expression_id
        .strip_prefix("omena-bridge-source-type-fact:")
        .unwrap_or(expression_id);
    // Drop the trailing `:{start}:{end}` span suffix, keeping the binding path.
    let mut segments = trimmed.rsplitn(3, ':');
    let _end = segments.next();
    let _start = segments.next();
    match segments.next() {
        Some(path) if !path.is_empty() => path.to_string(),
        _ => trimmed.to_string(),
    }
}

/// Run the real k-limited (k-CFA) call-string M-tier analysis on the supplied
/// dynamic-className call sites and lower each per-context M-tier evaluation into
/// an unsorted, ungated list of consumer source diagnostics anchored at the
/// originating call site's reference range.
///
/// This is the shared core used by both the explicit input-contract surface and
/// the default workspace diagnostic assembly. The `max_context_depth` bound `k`
/// drives `analyze_k_limited_call_site_flows`: at a low `k`, call sites that
/// share a callee collapse into one context and their exit values are joined, so
/// the emitted diagnostics differ from a higher-`k` run that keeps the call sites
/// separate. The caller is responsible for sorting and applying the checker
/// product diagnostic gate.
pub(super) fn collect_omena_query_dynamic_classname_m_tier_diagnostics(
    call_sites: &[OmenaQueryDynamicClassnameCallSiteV0],
    selector_universe: &[String],
    max_context_depth: usize,
) -> Vec<OmenaQuerySourceDiagnosticV0> {
    let mut range_by_context: BTreeMap<(String, Vec<String>), ParserRangeV0> = BTreeMap::new();
    let contexts = call_sites
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
        selector_universe,
        max_context_depth,
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
                    provenance: omena_query_evidence_graph_provenance![
                        "omena-query-checker-orchestrator.k-limited-flow-m-tier-gate",
                        "omena-abstract-value.k-limited-call-site-flow",
                        "omena-checker.m-tier-rules",
                        "omena-query.dynamic-classname",
                    ],
                    range,
                    message: evaluation.message.clone(),
                    precision: Some(source_diagnostic_precision(
                        "classValueFlow",
                        "kLimitedCallSiteFlow",
                        "kLimitedDynamicClassname",
                    )),
                    suggestion: None,
                    create_selector: None,
                });
            }
        }
    }
    diagnostics
}

fn dynamic_classname_m_tier_diagnostic_code(rule_code_name: &str) -> &'static str {
    match rule_code_name {
        "no-unknown-dynamic-class" => "noUnknownDynamicClass",
        "no-imprecise-value" => "noImpreciseValue",
        "no-impossible-selector" => "noImpossibleSelector",
        _ => "dynamicClassDomain",
    }
}

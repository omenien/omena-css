//! Promotion evidence for parser facts entering the semantic layer.
//!
//! These summaries are used by gate checks to verify that parser output is not
//! merely present, but is promoted into semantic contracts with the expected
//! counts, certainty, and downstream readiness signals.

use engine_input_producers::EngineInputV2;
use serde::Serialize;

use crate::{ParserBoundarySyntaxFactsV0, StyleSemanticFactsV0, summarize_source_input_evidence};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticPromotionEvidenceSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub items: Vec<SemanticPromotionEvidenceItemV0>,
    pub blocking_gaps: Vec<&'static str>,
    pub next_priorities: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticPromotionEvidenceItemV0 {
    pub evidence: &'static str,
    pub status: &'static str,
    pub provider: &'static str,
    pub observed_count: usize,
    pub reason: String,
}

pub fn summarize_semantic_promotion_evidence(
    parser_facts: &ParserBoundarySyntaxFactsV0,
    semantic_facts: &StyleSemanticFactsV0,
) -> SemanticPromotionEvidenceSummaryV0 {
    let source_span_ready = parser_facts.lossless_cst.all_token_spans_within_source
        && parser_facts.lossless_cst.all_node_spans_within_source;
    let unresolved_sass_count = semantic_facts
        .sass
        .selectors_with_unresolved_variable_refs_names
        .len()
        + semantic_facts
            .sass
            .selectors_with_unresolved_mixin_includes_names
            .len();
    let rewrite_blocker_count =
        semantic_facts.selector_identity.nested_unsafe_names.len() + unresolved_sass_count;
    let custom_property_seed_count = parser_facts.custom_properties.decl_names.len()
        + parser_facts.custom_properties.ref_names.len();

    let items = vec![
        SemanticPromotionEvidenceItemV0 {
            evidence: "selectorCanonicalId",
            status: if semantic_facts.selector_identity.canonical_names.is_empty() {
                "gap"
            } else {
                "ready"
            },
            provider: "StyleSelectorIdentityFactsV0.canonicalNames",
            observed_count: semantic_facts.selector_identity.canonical_names.len(),
            reason: format!(
                "{} canonical selector ids are exposed",
                semantic_facts.selector_identity.canonical_names.len()
            ),
        },
        SemanticPromotionEvidenceItemV0 {
            evidence: "sourceSpan",
            status: if source_span_ready { "ready" } else { "gap" },
            provider: "ParserLosslessCstFactsV0",
            observed_count: parser_facts.lossless_cst.token_count,
            reason: format!(
                "token spans valid={} node spans valid={}",
                parser_facts.lossless_cst.all_token_spans_within_source,
                parser_facts.lossless_cst.all_node_spans_within_source
            ),
        },
        SemanticPromotionEvidenceItemV0 {
            evidence: "bindingOrigin",
            status: "partial",
            provider: "StyleSassSemanticFactsV0.selectorSymbolFacts",
            observed_count: semantic_facts.sass.selector_symbol_facts.len(),
            reason: format!(
                "{} Sass selector symbol facts are exposed; cross-file origin still needs source bridge evidence",
                semantic_facts.sass.selector_symbol_facts.len()
            ),
        },
        SemanticPromotionEvidenceItemV0 {
            evidence: "styleModuleEdge",
            status: if parser_facts.sass.module_use_edges.is_empty() {
                "partial"
            } else {
                "ready"
            },
            provider: "ParserSassSyntaxFactsV0.moduleUseEdges",
            observed_count: parser_facts.sass.module_use_edges.len(),
            reason: format!(
                "{} Sass module use edges are exposed",
                parser_facts.sass.module_use_edges.len()
            ),
        },
        SemanticPromotionEvidenceItemV0 {
            evidence: "valueDomainExplanation",
            status: "partial",
            provider: "ParserIndexValueFactsV0",
            observed_count: parser_facts.values.ref_names.len(),
            reason: format!(
                "{} value refs are exposed; source-side expression-domain explanation remains external",
                parser_facts.values.ref_names.len()
            ),
        },
        SemanticPromotionEvidenceItemV0 {
            evidence: "designTokenSeed",
            status: if custom_property_seed_count == 0 {
                "partial"
            } else {
                "ready"
            },
            provider: "ParserIndexCustomPropertyFactsV0",
            observed_count: custom_property_seed_count,
            reason: format!(
                "{} CSS custom property declarations and {} var() references are exposed",
                parser_facts.custom_properties.decl_names.len(),
                parser_facts.custom_properties.ref_names.len()
            ),
        },
        SemanticPromotionEvidenceItemV0 {
            evidence: "rewriteSafetyBlocker",
            status: if rewrite_blocker_count == 0 {
                "ready"
            } else {
                "partial"
            },
            provider: "StyleSelectorIdentityFactsV0 + StyleSassSemanticFactsV0",
            observed_count: rewrite_blocker_count,
            reason: format!("{rewrite_blocker_count} selector or Sass blockers are exposed"),
        },
        SemanticPromotionEvidenceItemV0 {
            evidence: "referenceSiteIdentity",
            status: "gap",
            provider: "EngineInputV2.selector-usage",
            observed_count: 0,
            reason: "reference sites are produced outside this parser-backed semantic boundary"
                .to_string(),
        },
        SemanticPromotionEvidenceItemV0 {
            evidence: "certaintyReason",
            status: "gap",
            provider: "EngineInputV2.type-facts",
            observed_count: 0,
            reason: "source-side certainty reasons are not yet carried into omena-semantic"
                .to_string(),
        },
    ];

    SemanticPromotionEvidenceSummaryV0 {
        schema_version: "0",
        product: "omena-semantic.promotion-evidence",
        items,
        blocking_gaps: vec!["referenceSiteIdentity", "certaintyReason"],
        next_priorities: vec!["referenceSiteIdentity", "certaintyReason", "bindingOrigin"],
    }
}

pub fn summarize_semantic_promotion_evidence_with_source_input(
    parser_facts: &ParserBoundarySyntaxFactsV0,
    semantic_facts: &StyleSemanticFactsV0,
    input: &EngineInputV2,
) -> SemanticPromotionEvidenceSummaryV0 {
    let source_evidence = summarize_source_input_evidence(input);
    let mut summary = summarize_semantic_promotion_evidence(parser_facts, semantic_facts);

    for item in &mut summary.items {
        match item.evidence {
            "bindingOrigin" => {
                item.status = source_evidence.binding_origin.status;
                item.provider = "EngineInputV2.class-expressions";
                item.observed_count = source_evidence.binding_origin.expression_count;
                item.reason = format!(
                    "{} source class expressions expose binding origins",
                    source_evidence.binding_origin.expression_count
                );
            }
            "styleModuleEdge" => {
                item.status = source_evidence.style_module_edge.status;
                item.provider = "EngineInputV2.source-style-edges";
                item.observed_count = source_evidence.style_module_edge.source_style_edge_count;
                item.reason = format!(
                    "{} source-to-style module edges are linked",
                    source_evidence.style_module_edge.source_style_edge_count
                );
            }
            "valueDomainExplanation" => {
                item.status = source_evidence.value_domain_explanation.status;
                item.provider = "EngineInputV2.expression-semantics";
                item.observed_count = source_evidence.value_domain_explanation.expression_count;
                item.reason = format!(
                    "{} source expressions expose value-domain explanations",
                    source_evidence.value_domain_explanation.expression_count
                );
            }
            "referenceSiteIdentity" => {
                item.status = source_evidence.reference_site_identity.status;
                item.provider = "EngineInputV2.selector-usage";
                item.observed_count = source_evidence.reference_site_identity.reference_site_count;
                item.reason = format!(
                    "{} selector reference sites are identity-preserving",
                    source_evidence.reference_site_identity.reference_site_count
                );
            }
            "certaintyReason" => {
                item.status = source_evidence.certainty_reason.status;
                item.provider = "EngineInputV2.expression-semantics";
                item.observed_count = source_evidence.certainty_reason.expression_count;
                item.reason = format!(
                    "{} source expressions expose selector certainty reasons",
                    source_evidence.certainty_reason.expression_count
                );
            }
            _ => {}
        }
    }

    summary.blocking_gaps = summary
        .items
        .iter()
        .filter(|item| item.status == "gap")
        .map(|item| item.evidence)
        .collect();
    summary.next_priorities = source_evidence.blocking_gaps;
    summary
}

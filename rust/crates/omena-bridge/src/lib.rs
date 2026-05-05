use engine_input_producers::EngineInputV2;
use engine_style_parser::{
    ParserBoundarySyntaxFactsV0, StyleSemanticFactsV0, Stylesheet, parse_style_module,
};
pub use omena_semantic::{
    DesignTokenExternalDeclarationCandidateScopeV0, DesignTokenWorkspaceDeclarationFactV0,
};
use omena_semantic::{
    DesignTokenSemanticSummaryV0, LosslessCstContractV0, SelectorIdentityEngineSummaryV0,
};
use serde::Serialize;

mod promotion_evidence;
mod selector_references;
mod source_evidence;
mod source_imports;

pub use promotion_evidence::{
    SemanticPromotionEvidenceItemV0, SemanticPromotionEvidenceSummaryV0,
    summarize_omena_bridge_promotion_evidence_with_source_input,
    summarize_omena_bridge_semantic_promotion_evidence,
};
pub use selector_references::{
    SelectorEditableDirectReferenceSiteV0, SelectorReferenceEngineSummaryV0,
    SelectorReferenceSiteV0, SelectorReferenceSummaryV0,
    summarize_omena_bridge_selector_reference_engine,
};
pub use source_evidence::{
    BindingOriginEvidenceV0, CertaintyReasonEvidenceV0, ReferenceSiteIdentityEvidenceV0,
    SourceInputPromotionEvidenceSummaryV0, StyleModuleEdgeEvidenceV0,
    ValueDomainExplanationEvidenceV0, summarize_omena_bridge_source_input_evidence,
};
pub use source_imports::{
    SourceImportDeclarationSummaryV0, SourceImportDeclarationV0,
    summarize_omena_bridge_source_import_declarations,
};

pub fn collect_omena_bridge_design_token_workspace_declarations(
    style_path: &str,
    sheet: &Stylesheet,
) -> Vec<DesignTokenWorkspaceDeclarationFactV0> {
    let parser_facts = omena_semantic::summarize_parser_contract_facts(sheet);
    omena_semantic::collect_design_token_workspace_declarations(style_path, &parser_facts)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaBridgeBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub bridge_name: &'static str,
    pub graph_product: &'static str,
    pub delegated_semantic_boundary_product: &'static str,
    pub selector_reference_product: &'static str,
    pub source_input_evidence_product: &'static str,
    pub binder_plugin_product: &'static str,
    pub bridge_owned_surfaces: Vec<&'static str>,
    pub cme_coupled_surfaces: Vec<&'static str>,
    pub next_decoupling_targets: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BinderPluginBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub owner_crate: &'static str,
    pub contract_name: &'static str,
    pub external_plugin_abi_stable: bool,
    pub default_plugin: BinderPluginSummaryV0,
    pub request_path_policy: Vec<&'static str>,
    pub next_plugin_targets: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BinderPluginSummaryV0 {
    pub id: &'static str,
    pub version: &'static str,
    pub stability: &'static str,
    pub domains: Vec<&'static str>,
    pub owns_surfaces: Vec<&'static str>,
    pub import_targets: Vec<&'static str>,
    pub utility_targets: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleSemanticGraphSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub language: &'static str,
    pub parser_facts: ParserBoundarySyntaxFactsV0,
    pub semantic_facts: StyleSemanticFactsV0,
    pub design_token_semantics: DesignTokenSemanticSummaryV0,
    pub selector_identity_engine: SelectorIdentityEngineSummaryV0,
    pub selector_reference_engine: SelectorReferenceEngineSummaryV0,
    pub source_input_evidence: SourceInputPromotionEvidenceSummaryV0,
    pub promotion_evidence: SemanticPromotionEvidenceSummaryV0,
    pub lossless_cst_contract: LosslessCstContractV0,
}

pub fn summarize_omena_bridge_boundary() -> OmenaBridgeBoundarySummaryV0 {
    OmenaBridgeBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-bridge.cme-semantic-bridge",
        bridge_name: "cme-semantic-bridge",
        graph_product: "omena-semantic.style-semantic-graph",
        delegated_semantic_boundary_product: "omena-semantic.style-semantic-boundary",
        selector_reference_product: "omena-semantic.selector-references",
        source_input_evidence_product: "omena-semantic.source-input-evidence",
        binder_plugin_product: "omena-bridge.binder-plugin-boundary",
        bridge_owned_surfaces: vec![
            "styleSemanticGraph",
            "styleSemanticGraphFromSource",
            "selectorReferenceEngine",
            "sourceInputEvidence",
            "sourceImportDeclarations",
            "promotionEvidenceWithSourceInput",
            "binderPluginBoundary",
        ],
        cme_coupled_surfaces: vec![
            "EngineInputV2",
            "sourceInputEvidence",
            "selectorReferenceEngine",
            "promotionEvidenceWithSourceInput",
            "styleSemanticGraphFromSource",
            "cssModulesClassnameBinding",
        ],
        next_decoupling_targets: Vec::new(),
    }
}

pub fn summarize_omena_bridge_binder_plugin_boundary() -> BinderPluginBoundarySummaryV0 {
    BinderPluginBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-bridge.binder-plugin-boundary",
        owner_crate: "omena-bridge",
        contract_name: "BinderPluginV0",
        external_plugin_abi_stable: false,
        default_plugin: BinderPluginSummaryV0 {
            id: "css-modules-classnames-bind",
            version: "0",
            stability: "builtIn",
            domains: vec!["css-modules"],
            owns_surfaces: vec![
                "styleImportRecognition",
                "classUtilityRecognition",
                "classReferenceExtraction",
                "sourceExpressionProjection",
            ],
            import_targets: vec!["*.module.css", "*.module.scss", "*.module.less"],
            utility_targets: vec!["classnames/bind", "classnames", "clsx", "clsx/lite"],
        },
        request_path_policy: vec![
            "builtInPluginsOnlyUntilAbiStabilizes",
            "pluginOutputFeedsEngineInputV2",
            "sourceExpressionProjectionMustPreserveBindingIdentity",
            "styleImportResolutionMustRemainTargetAware",
        ],
        next_plugin_targets: vec![
            "tailwind-utility-domain",
            "vanilla-extract-recipe-domain",
            "vue-style-module-domain",
        ],
    }
}

pub fn summarize_omena_bridge_style_semantic_graph(
    sheet: &Stylesheet,
    input: &EngineInputV2,
) -> StyleSemanticGraphSummaryV0 {
    summarize_omena_bridge_style_semantic_graph_for_path(sheet, input, None)
}

pub fn summarize_omena_bridge_style_semantic_graph_for_path(
    sheet: &Stylesheet,
    input: &EngineInputV2,
    style_path: Option<&str>,
) -> StyleSemanticGraphSummaryV0 {
    summarize_omena_bridge_style_semantic_graph_for_path_with_workspace_declarations(
        sheet,
        input,
        style_path,
        &[],
    )
}

pub fn summarize_omena_bridge_style_semantic_graph_for_path_with_workspace_declarations(
    sheet: &Stylesheet,
    input: &EngineInputV2,
    style_path: Option<&str>,
    workspace_declarations: &[DesignTokenWorkspaceDeclarationFactV0],
) -> StyleSemanticGraphSummaryV0 {
    summarize_omena_bridge_style_semantic_graph_for_path_with_scoped_workspace_declarations(
        sheet,
        input,
        style_path,
        workspace_declarations,
        DesignTokenExternalDeclarationCandidateScopeV0::Workspace,
    )
}

pub fn summarize_omena_bridge_style_semantic_graph_for_path_with_scoped_workspace_declarations(
    sheet: &Stylesheet,
    input: &EngineInputV2,
    style_path: Option<&str>,
    workspace_declarations: &[DesignTokenWorkspaceDeclarationFactV0],
    candidate_scope: DesignTokenExternalDeclarationCandidateScopeV0,
) -> StyleSemanticGraphSummaryV0 {
    let boundary = omena_semantic::summarize_style_semantic_boundary(sheet);
    let parser_facts = boundary.parser_facts;
    let semantic_facts = boundary.semantic_facts;
    let design_token_semantics =
        omena_semantic::summarize_design_token_semantics_with_scoped_workspace_declarations(
            &parser_facts,
            &semantic_facts,
            style_path,
            workspace_declarations,
            candidate_scope,
        );
    let selector_identity_engine = boundary.selector_identity_engine;
    let selector_reference_engine =
        summarize_omena_bridge_selector_reference_engine(input, style_path);
    let source_input_evidence = summarize_omena_bridge_source_input_evidence(input);
    let promotion_evidence = summarize_omena_bridge_promotion_evidence_with_source_input(
        &parser_facts,
        &semantic_facts,
        input,
    );
    let lossless_cst_contract = boundary.lossless_cst_contract;

    StyleSemanticGraphSummaryV0 {
        schema_version: "0",
        product: "omena-semantic.style-semantic-graph",
        language: boundary.language,
        parser_facts,
        semantic_facts,
        design_token_semantics,
        selector_identity_engine,
        selector_reference_engine,
        source_input_evidence,
        promotion_evidence,
        lossless_cst_contract,
    }
}

pub fn summarize_omena_bridge_style_semantic_graph_from_source(
    style_path: &str,
    style_source: &str,
    input: &EngineInputV2,
) -> Option<StyleSemanticGraphSummaryV0> {
    let sheet = parse_style_module(style_path, style_source)?;
    Some(summarize_omena_bridge_style_semantic_graph_for_path(
        &sheet,
        input,
        Some(style_path),
    ))
}

#[cfg(test)]
mod tests {
    use engine_input_producers::{
        ClassExpressionInputV2, EngineInputV2, PositionV2, RangeV2, SourceAnalysisInputV2,
        SourceDocumentV2, StringTypeFactsV2, StyleAnalysisInputV2, StyleDocumentV2,
        StyleSelectorV2, TypeFactEntryV2,
    };
    use engine_style_parser::parse_style_module;

    use super::{
        summarize_omena_bridge_binder_plugin_boundary, summarize_omena_bridge_boundary,
        summarize_omena_bridge_promotion_evidence_with_source_input,
        summarize_omena_bridge_selector_reference_engine,
        summarize_omena_bridge_source_import_declarations,
        summarize_omena_bridge_source_input_evidence, summarize_omena_bridge_style_semantic_graph,
        summarize_omena_bridge_style_semantic_graph_from_source,
    };

    #[test]
    fn declares_cme_coupled_bridge_boundary() {
        let boundary = summarize_omena_bridge_boundary();

        assert_eq!(boundary.schema_version, "0");
        assert_eq!(boundary.product, "omena-bridge.cme-semantic-bridge");
        assert_eq!(
            boundary.graph_product,
            "omena-semantic.style-semantic-graph"
        );
        assert_eq!(
            boundary.delegated_semantic_boundary_product,
            "omena-semantic.style-semantic-boundary"
        );
        assert_eq!(
            boundary.selector_reference_product,
            "omena-semantic.selector-references"
        );
        assert_eq!(
            boundary.source_input_evidence_product,
            "omena-semantic.source-input-evidence"
        );
        assert_eq!(
            boundary.binder_plugin_product,
            "omena-bridge.binder-plugin-boundary"
        );
        assert!(
            boundary
                .bridge_owned_surfaces
                .contains(&"styleSemanticGraphFromSource")
        );
        assert!(
            boundary
                .bridge_owned_surfaces
                .contains(&"selectorReferenceEngine")
        );
        assert!(
            boundary
                .bridge_owned_surfaces
                .contains(&"sourceInputEvidence")
        );
        assert!(
            boundary
                .bridge_owned_surfaces
                .contains(&"promotionEvidenceWithSourceInput")
        );
        assert!(
            boundary
                .bridge_owned_surfaces
                .contains(&"sourceImportDeclarations")
        );
        assert!(
            boundary
                .bridge_owned_surfaces
                .contains(&"binderPluginBoundary")
        );
        assert!(
            boundary
                .cme_coupled_surfaces
                .contains(&"promotionEvidenceWithSourceInput")
        );
        assert!(
            boundary.next_decoupling_targets.is_empty(),
            "all current omena-bridge decoupling targets should be bridge-owned"
        );
    }

    #[test]
    fn declares_built_in_binder_plugin_boundary() {
        let boundary = summarize_omena_bridge_binder_plugin_boundary();

        assert_eq!(boundary.schema_version, "0");
        assert_eq!(boundary.product, "omena-bridge.binder-plugin-boundary");
        assert_eq!(boundary.contract_name, "BinderPluginV0");
        assert!(
            !boundary.external_plugin_abi_stable,
            "the first boundary cut should not promise a stable external plugin ABI"
        );
        assert_eq!(boundary.default_plugin.id, "css-modules-classnames-bind");
        assert_eq!(boundary.default_plugin.stability, "builtIn");
        assert!(boundary.default_plugin.domains.contains(&"css-modules"));
        assert!(
            boundary
                .default_plugin
                .owns_surfaces
                .contains(&"classReferenceExtraction")
        );
        assert!(
            boundary
                .default_plugin
                .utility_targets
                .contains(&"classnames/bind")
        );
        assert!(
            boundary
                .request_path_policy
                .contains(&"pluginOutputFeedsEngineInputV2")
        );
        assert!(
            boundary
                .next_plugin_targets
                .contains(&"tailwind-utility-domain")
        );
    }

    #[test]
    fn summarizes_source_import_declarations_for_css_modules_binding_inputs() {
        let summary = summarize_omena_bridge_source_import_declarations(
            r#"
import bind from "classnames/bind";
import styles from "./Button.module.scss";
import * as tokens from "./tokens.module.css";
import { type BadgeProps } from "./types";
const lazy = import("./ignored.module.scss");
"#,
        );

        assert_eq!(summary.product, "omena-bridge.source-import-declarations");
        assert_eq!(summary.import_count, 3);
        assert_eq!(
            summary
                .imports
                .iter()
                .map(|import| (import.binding.as_str(), import.specifier.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("bind", "classnames/bind"),
                ("styles", "./Button.module.scss"),
                ("tokens", "./tokens.module.css"),
            ]
        );
    }

    #[test]
    fn exposes_source_input_evidence_through_bridge() {
        let evidence = summarize_omena_bridge_source_input_evidence(&sample_engine_input());

        assert_eq!(evidence.product, "omena-semantic.source-input-evidence");
        assert_eq!(evidence.reference_site_identity.status, "ready");
        assert_eq!(evidence.reference_site_identity.reference_site_count, 1);
        assert_eq!(evidence.certainty_reason.status, "ready");
        assert_eq!(evidence.binding_origin.status, "ready");
        assert!(evidence.blocking_gaps.is_empty());
    }

    #[test]
    fn exposes_style_semantic_graph_through_bridge() -> Result<(), String> {
        let sheet = parse_style_module("Component.module.scss", ".button { color: red; }")
            .ok_or_else(|| "SCSS module path should parse".to_string())?;
        let graph = summarize_omena_bridge_style_semantic_graph(&sheet, &sample_engine_input());

        assert_eq!(graph.product, "omena-semantic.style-semantic-graph");
        assert_eq!(graph.selector_reference_engine.selector_count, 1);
        assert_eq!(graph.selector_reference_engine.referenced_selector_count, 1);
        assert_eq!(
            graph.source_input_evidence.product,
            "omena-semantic.source-input-evidence"
        );
        assert!(graph.promotion_evidence.blocking_gaps.is_empty());
        Ok(())
    }

    #[test]
    fn exposes_style_semantic_graph_from_source_through_bridge() -> Result<(), String> {
        let graph = summarize_omena_bridge_style_semantic_graph_from_source(
            "/tmp/Component.module.scss",
            ".button { color: red; }",
            &sample_engine_input(),
        )
        .ok_or_else(|| "bridge should parse SCSS module source".to_string())?;

        assert_eq!(graph.product, "omena-semantic.style-semantic-graph");
        assert_eq!(
            graph.selector_reference_engine.style_path,
            Some("/tmp/Component.module.scss".to_string())
        );
        assert_eq!(
            graph.source_input_evidence.reference_site_identity.status,
            "ready"
        );
        Ok(())
    }

    #[test]
    fn owns_selector_reference_engine_without_changing_host_product() {
        let bridge_references = summarize_omena_bridge_selector_reference_engine(
            &sample_engine_input(),
            Some("/tmp/Component.module.scss"),
        );
        let semantic_references = omena_semantic::summarize_selector_reference_engine(
            &sample_engine_input(),
            Some("/tmp/Component.module.scss"),
        );

        assert_eq!(
            bridge_references.product,
            "omena-semantic.selector-references"
        );
        assert_eq!(bridge_references.product, semantic_references.product);
        assert_eq!(bridge_references.style_path, semantic_references.style_path);
        assert_eq!(
            bridge_references.selector_count,
            semantic_references.selector_count
        );
        assert_eq!(
            bridge_references.referenced_selector_count,
            semantic_references.referenced_selector_count
        );
        assert_eq!(
            bridge_references.total_reference_sites,
            semantic_references.total_reference_sites
        );
        assert_eq!(
            bridge_references.selectors[0].canonical_id,
            semantic_references.selectors[0].canonical_id
        );
        assert_eq!(
            bridge_references.selectors[0].editable_direct_reference_count,
            semantic_references.selectors[0].editable_direct_reference_count
        );
    }

    #[test]
    fn owns_source_input_evidence_without_changing_host_product() {
        let bridge_evidence = summarize_omena_bridge_source_input_evidence(&sample_engine_input());
        let semantic_evidence =
            omena_semantic::summarize_source_input_evidence(&sample_engine_input());

        assert_bridge_source_input_evidence_matches_semantic(&bridge_evidence, &semantic_evidence);
    }

    #[test]
    fn owns_source_backed_promotion_evidence_without_changing_host_product() -> Result<(), String> {
        let sheet = parse_style_module("Component.module.scss", ".button { color: red; }")
            .ok_or_else(|| "SCSS module path should parse".to_string())?;
        let boundary = omena_semantic::summarize_style_semantic_boundary(&sheet);
        let input = sample_engine_input();
        let bridge_evidence = summarize_omena_bridge_promotion_evidence_with_source_input(
            &boundary.parser_facts,
            &boundary.semantic_facts,
            &input,
        );
        let semantic_evidence =
            omena_semantic::summarize_semantic_promotion_evidence_with_source_input(
                &boundary.parser_facts,
                &boundary.semantic_facts,
                &input,
            );

        assert_bridge_promotion_evidence_matches_semantic(&bridge_evidence, &semantic_evidence);
        Ok(())
    }

    #[test]
    fn owns_graph_assembly_without_changing_host_product() -> Result<(), String> {
        let sheet = parse_style_module("Component.module.scss", ".button { color: red; }")
            .ok_or_else(|| "SCSS module path should parse".to_string())?;
        let bridge_graph =
            summarize_omena_bridge_style_semantic_graph(&sheet, &sample_engine_input());
        let semantic_graph =
            omena_semantic::summarize_style_semantic_graph(&sheet, &sample_engine_input());

        assert_eq!(bridge_graph.product, "omena-semantic.style-semantic-graph");
        assert_eq!(bridge_graph.product, semantic_graph.product);
        assert_eq!(bridge_graph.language, semantic_graph.language);
        assert_eq!(
            bridge_graph.selector_reference_engine.product,
            semantic_graph.selector_reference_engine.product
        );
        assert_eq!(
            bridge_graph.selector_reference_engine.selector_count,
            semantic_graph.selector_reference_engine.selector_count
        );
        assert_eq!(
            bridge_graph.selector_reference_engine.total_reference_sites,
            semantic_graph
                .selector_reference_engine
                .total_reference_sites
        );
        assert_bridge_source_input_evidence_matches_semantic(
            &bridge_graph.source_input_evidence,
            &semantic_graph.source_input_evidence,
        );
        assert_eq!(
            bridge_graph.design_token_semantics,
            semantic_graph.design_token_semantics
        );
        assert_bridge_promotion_evidence_matches_semantic(
            &bridge_graph.promotion_evidence,
            &semantic_graph.promotion_evidence,
        );
        Ok(())
    }

    fn assert_bridge_promotion_evidence_matches_semantic(
        bridge: &super::SemanticPromotionEvidenceSummaryV0,
        semantic: &omena_semantic::SemanticPromotionEvidenceSummaryV0,
    ) {
        assert_eq!(bridge.schema_version, semantic.schema_version);
        assert_eq!(bridge.product, semantic.product);
        assert_eq!(bridge.blocking_gaps, semantic.blocking_gaps);
        assert_eq!(bridge.next_priorities, semantic.next_priorities);
        assert_eq!(bridge.items.len(), semantic.items.len());

        for (bridge_item, semantic_item) in bridge.items.iter().zip(&semantic.items) {
            assert_eq!(bridge_item.evidence, semantic_item.evidence);
            assert_eq!(bridge_item.status, semantic_item.status);
            assert_eq!(bridge_item.provider, semantic_item.provider);
            assert_eq!(bridge_item.observed_count, semantic_item.observed_count);
            assert_eq!(bridge_item.reason, semantic_item.reason);
        }
    }

    fn assert_bridge_source_input_evidence_matches_semantic(
        bridge: &super::SourceInputPromotionEvidenceSummaryV0,
        semantic: &omena_semantic::SourceInputPromotionEvidenceSummaryV0,
    ) {
        assert_eq!(bridge.schema_version, semantic.schema_version);
        assert_eq!(bridge.product, semantic.product);
        assert_eq!(bridge.input_version, semantic.input_version);
        assert_eq!(
            bridge.reference_site_identity.status,
            semantic.reference_site_identity.status
        );
        assert_eq!(
            bridge.reference_site_identity.selector_count,
            semantic.reference_site_identity.selector_count
        );
        assert_eq!(
            bridge.reference_site_identity.reference_site_count,
            semantic.reference_site_identity.reference_site_count
        );
        assert_eq!(
            bridge.reference_site_identity.direct_reference_site_count,
            semantic.reference_site_identity.direct_reference_site_count
        );
        assert_eq!(
            bridge.reference_site_identity.expanded_reference_site_count,
            semantic
                .reference_site_identity
                .expanded_reference_site_count
        );
        assert_eq!(
            bridge
                .reference_site_identity
                .style_dependency_reference_site_count,
            semantic
                .reference_site_identity
                .style_dependency_reference_site_count
        );
        assert_eq!(
            bridge.reference_site_identity.editable_direct_site_count,
            semantic.reference_site_identity.editable_direct_site_count
        );
        assert_eq!(
            bridge.reference_site_identity.reference_kind_counts,
            semantic.reference_site_identity.reference_kind_counts
        );
        assert_eq!(
            bridge.certainty_reason.status,
            semantic.certainty_reason.status
        );
        assert_eq!(
            bridge.certainty_reason.expression_count,
            semantic.certainty_reason.expression_count
        );
        assert_eq!(
            bridge.certainty_reason.exact_count,
            semantic.certainty_reason.exact_count
        );
        assert_eq!(
            bridge.certainty_reason.inferred_count,
            semantic.certainty_reason.inferred_count
        );
        assert_eq!(
            bridge.certainty_reason.possible_count,
            semantic.certainty_reason.possible_count
        );
        assert_eq!(
            bridge.certainty_reason.missing_reason_count,
            semantic.certainty_reason.missing_reason_count
        );
        assert_eq!(
            bridge.certainty_reason.reason_counts,
            semantic.certainty_reason.reason_counts
        );
        assert_eq!(
            bridge.certainty_reason.shape_kind_counts,
            semantic.certainty_reason.shape_kind_counts
        );
        assert_eq!(
            bridge.certainty_reason.shape_label_counts,
            semantic.certainty_reason.shape_label_counts
        );
        assert_eq!(bridge.binding_origin.status, semantic.binding_origin.status);
        assert_eq!(
            bridge.binding_origin.expression_count,
            semantic.binding_origin.expression_count
        );
        assert_eq!(
            bridge.binding_origin.direct_class_name_count,
            semantic.binding_origin.direct_class_name_count
        );
        assert_eq!(
            bridge.binding_origin.root_binding_count,
            semantic.binding_origin.root_binding_count
        );
        assert_eq!(
            bridge.binding_origin.access_path_count,
            semantic.binding_origin.access_path_count
        );
        assert_eq!(
            bridge.binding_origin.access_path_segment_count,
            semantic.binding_origin.access_path_segment_count
        );
        assert_eq!(
            bridge.binding_origin.expression_kind_counts,
            semantic.binding_origin.expression_kind_counts
        );
        assert_eq!(
            bridge.style_module_edge.status,
            semantic.style_module_edge.status
        );
        assert_eq!(
            bridge.style_module_edge.source_style_edge_count,
            semantic.style_module_edge.source_style_edge_count
        );
        assert_eq!(
            bridge.style_module_edge.distinct_style_module_count,
            semantic.style_module_edge.distinct_style_module_count
        );
        assert_eq!(
            bridge.style_module_edge.missing_style_document_edge_count,
            semantic.style_module_edge.missing_style_document_edge_count
        );
        assert_eq!(
            bridge.style_module_edge.composed_edge_count,
            semantic.style_module_edge.composed_edge_count
        );
        assert_eq!(
            bridge.style_module_edge.imported_composed_edge_count,
            semantic.style_module_edge.imported_composed_edge_count
        );
        assert_eq!(
            bridge.style_module_edge.global_composed_edge_count,
            semantic.style_module_edge.global_composed_edge_count
        );
        assert_eq!(
            bridge.value_domain_explanation.status,
            semantic.value_domain_explanation.status
        );
        assert_eq!(
            bridge.value_domain_explanation.expression_count,
            semantic.value_domain_explanation.expression_count
        );
        assert_eq!(
            bridge.value_domain_explanation.exact_expression_count,
            semantic.value_domain_explanation.exact_expression_count
        );
        assert_eq!(
            bridge
                .value_domain_explanation
                .finite_value_expression_count,
            semantic
                .value_domain_explanation
                .finite_value_expression_count
        );
        assert_eq!(
            bridge.value_domain_explanation.constrained_expression_count,
            semantic
                .value_domain_explanation
                .constrained_expression_count
        );
        assert_eq!(
            bridge.value_domain_explanation.unknown_expression_count,
            semantic.value_domain_explanation.unknown_expression_count
        );
        assert_eq!(
            bridge.value_domain_explanation.finite_value_count,
            semantic.value_domain_explanation.finite_value_count
        );
        assert_eq!(
            bridge.value_domain_explanation.derivation_count,
            semantic.value_domain_explanation.derivation_count
        );
        assert_eq!(
            bridge.value_domain_explanation.derivation_step_count,
            semantic.value_domain_explanation.derivation_step_count
        );
        assert_eq!(
            bridge.value_domain_explanation.value_domain_kind_counts,
            semantic.value_domain_explanation.value_domain_kind_counts
        );
        assert_eq!(
            bridge.value_domain_explanation.constraint_kind_counts,
            semantic.value_domain_explanation.constraint_kind_counts
        );
        assert_eq!(
            bridge.value_domain_explanation.derivation_product_counts,
            semantic.value_domain_explanation.derivation_product_counts
        );
        assert_eq!(
            bridge
                .value_domain_explanation
                .derivation_reduced_kind_counts,
            semantic
                .value_domain_explanation
                .derivation_reduced_kind_counts
        );
        assert_eq!(
            bridge.value_domain_explanation.derivation_operation_counts,
            semantic
                .value_domain_explanation
                .derivation_operation_counts
        );
        assert_eq!(bridge.blocking_gaps, semantic.blocking_gaps);
        assert_eq!(bridge.next_priorities, semantic.next_priorities);
    }

    fn sample_engine_input() -> EngineInputV2 {
        EngineInputV2 {
            version: "2".to_string(),
            sources: vec![SourceAnalysisInputV2 {
                document: SourceDocumentV2 {
                    class_expressions: vec![ClassExpressionInputV2 {
                        id: "expr-literal".to_string(),
                        kind: "literal".to_string(),
                        scss_module_path: "/tmp/Component.module.scss".to_string(),
                        range: range(4, 12, 4, 18),
                        class_name: Some("button".to_string()),
                        root_binding_decl_id: None,
                        access_path: None,
                    }],
                },
            }],
            styles: vec![StyleAnalysisInputV2 {
                file_path: "/tmp/Component.module.scss".to_string(),
                document: StyleDocumentV2 {
                    selectors: vec![StyleSelectorV2 {
                        name: "button".to_string(),
                        view_kind: "canonical".to_string(),
                        canonical_name: Some("button".to_string()),
                        range: range(0, 1, 0, 7),
                        nested_safety: Some("flat".to_string()),
                        composes: None,
                        bem_suffix: None,
                    }],
                },
            }],
            type_facts: vec![TypeFactEntryV2 {
                file_path: "/tmp/Component.tsx".to_string(),
                expression_id: "expr-literal".to_string(),
                facts: StringTypeFactsV2 {
                    kind: "exact".to_string(),
                    constraint_kind: None,
                    values: Some(vec!["button".to_string()]),
                    prefix: None,
                    suffix: None,
                    min_len: None,
                    max_len: None,
                    char_must: None,
                    char_may: None,
                    may_include_other_chars: None,
                },
            }],
        }
    }

    fn range(
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> RangeV2 {
        RangeV2 {
            start: PositionV2 {
                line: start_line,
                character: start_character,
            },
            end: PositionV2 {
                line: end_line,
                character: end_character,
            },
        }
    }
}
